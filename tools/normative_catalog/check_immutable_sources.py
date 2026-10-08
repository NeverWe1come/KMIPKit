"""Protect pinned OASIS copies while allowing narrow project-authored additions."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


OASIS_ROOT = "specification/oasis/"
UPSTREAM_ROOT = "specification/oasis/kmip-2.1/upstream/"
FIXTURE_ROOT = "specification/oasis/kmip-2.1/fixtures/"
PROJECT_INVENTORY_PATHS = {
    "specification/oasis/kmip-2.1/README.md",
    "specification/oasis/kmip-2.1/SOURCES.md",
}


class ImmutableSourceError(ValueError):
    """Raised when protected OASIS source paths differ from their base."""


def _git(root: Path, *arguments: str) -> bytes:
    try:
        result = subprocess.run(
            ["git", *arguments],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise ImmutableSourceError("could not inspect the repository's Git tree") from error
    return result.stdout


def _changed_paths(root: Path, base_sha: str, *, staged: bool) -> dict[str, set[str]]:
    """Return NUL-delimited changed OASIS paths and statuses relative to ``base_sha``."""
    arguments = ["diff"]
    if staged:
        arguments.append("--cached")
    output = _git(
        root,
        *arguments,
        "--name-status",
        "-z",
        "--no-renames",
        base_sha,
        "--",
        OASIS_ROOT,
    )
    fields = output.split(b"\0")
    if fields and not fields[-1]:
        fields.pop()
    if len(fields) % 2:
        raise ImmutableSourceError("Git returned a malformed changed-path listing")
    changed: dict[str, set[str]] = {}
    for index in range(0, len(fields), 2):
        try:
            status = fields[index].decode("ascii")
        except UnicodeDecodeError as error:
            raise ImmutableSourceError("Git returned an invalid changed-path status") from error
        path = os.fsdecode(fields[index + 1])
        changed.setdefault(path, set()).add(status)
    return changed


def _base_inventory_paths(root: Path, base_sha: str) -> set[str]:
    """Return project-authored inventory files present in the exact base commit."""
    output = _git(
        root,
        "ls-tree",
        "--full-tree",
        "-r",
        "--name-only",
        "-z",
        base_sha,
        "--",
        *sorted(PROJECT_INVENTORY_PATHS),
    )
    return {os.fsdecode(path) for path in output.split(b"\0") if path}


def _base_fixture_paths(root: Path, base_sha: str) -> set[str]:
    """Return fixture paths already present in the exact base commit."""
    output = _git(
        root,
        "ls-tree",
        "--full-tree",
        "-r",
        "--name-only",
        "-z",
        base_sha,
        "--",
        FIXTURE_ROOT,
    )
    return {os.fsdecode(path) for path in output.split(b"\0") if path}


def _untracked_fixture_count(root: Path) -> tuple[bool, int]:
    """Stream untracked OASIS paths and reject the first path outside fixtures."""
    try:
        process = subprocess.Popen(
            ["git", "ls-files", "--others", "-z", "--", OASIS_ROOT],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except OSError as error:
        raise ImmutableSourceError("could not inspect the repository's Git tree") from error
    if process.stdout is None:
        process.kill()
        process.wait()
        raise ImmutableSourceError("Git did not provide a readable untracked-path stream")
    fixture_count = 0
    remainder = b""
    try:
        while chunk := process.stdout.read(4096):
            paths = (remainder + chunk).split(b"\0")
            remainder = paths.pop()
            for path in paths:
                if path and not os.fsdecode(path).startswith(FIXTURE_ROOT):
                    process.kill()
                    process.wait()
                    return True, fixture_count
                if path:
                    fixture_count += 1
        return_code = process.wait()
    except OSError as error:
        process.kill()
        process.wait()
        raise ImmutableSourceError("could not inspect untracked OASIS paths") from error
    finally:
        process.stdout.close()
    if remainder:
        raise ImmutableSourceError("Git returned a malformed untracked-path listing")
    if return_code != 0:
        raise ImmutableSourceError("could not inspect the repository's Git tree")
    return False, fixture_count


def check_immutable_sources(repo_root: Path, base_sha: str) -> dict[str, object]:
    """Protect upstream sources while permitting tracked inventory edits and new fixtures."""
    root = repo_root.resolve(strict=True)
    if re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", base_sha) is None:
        raise ImmutableSourceError("base must be a full lowercase Git commit SHA")

    try:
        resolved = _git(root, "rev-parse", "--verify", f"{base_sha}^{{commit}}").decode("ascii").strip()
    except UnicodeDecodeError as error:
        raise ImmutableSourceError("Git returned an invalid base commit identifier") from error
    if resolved != base_sha:
        raise ImmutableSourceError("base SHA does not resolve to that exact commit")

    changed_statuses = _changed_paths(root, base_sha, staged=False)
    for path, statuses in _changed_paths(root, base_sha, staged=True).items():
        changed_statuses.setdefault(path, set()).update(statuses)
    existing_fixture_paths = _base_fixture_paths(root, base_sha)
    existing_inventory_paths = _base_inventory_paths(root, base_sha)
    unsupported_paths = {
        path
        for path, statuses in changed_statuses.items()
        if path.startswith(UPSTREAM_ROOT)
        or (
            path in PROJECT_INVENTORY_PATHS
            and (path not in existing_inventory_paths or statuses != {"M"})
        )
        or (
            path.startswith(FIXTURE_ROOT)
            and (path in existing_fixture_paths or "A" not in statuses or not statuses <= {"A", "M"})
        )
        or (path not in PROJECT_INVENTORY_PATHS and not path.startswith(FIXTURE_ROOT))
    }
    has_unsupported_untracked, untracked_fixture_count = _untracked_fixture_count(root)
    if unsupported_paths or has_unsupported_untracked:
        raise ImmutableSourceError(
            "only project OASIS README/SOURCES edits and new KMIP 2.1 fixture additions are permitted"
        )
    return {
        "base_sha": base_sha,
        "changed_path_count": len(changed_statuses) + untracked_fixture_count,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-sha", required=True, help="exact full SHA of the pull-request base commit")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    arguments = parser.parse_args(argv)
    try:
        result = check_immutable_sources(arguments.repo_root, arguments.base_sha)
    except ImmutableSourceError as error:
        print(f"immutable-source check failed: {error}", file=sys.stderr)
        return 1
    print(f"Pinned OASIS upstream sources match base {result['base_sha']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
