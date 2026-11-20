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
APPROVED_VERSIONS = {"kmip-2.0", "kmip-2.1", "kmip-3.0"}
APPROVED_VERSION_DOCS = {"README.md", "SOURCES.md", "CHECKSUMS.sha256"}


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


def _base_upstream_paths(root: Path, base_sha: str) -> set[str]:
    """Return upstream paths already present in the exact base commit."""
    output = _git(
        root,
        "ls-tree",
        "--full-tree",
        "-r",
        "--name-only",
        "-z",
        base_sha,
        "--",
        OASIS_ROOT,
    )
    return {
        os.fsdecode(path)
        for path in output.split(b"\0")
        if path and "/upstream/" in os.fsdecode(path)
    }


def _is_permitted_change(
    path: str,
    statuses: set[str],
    existing_fixture_paths: set[str],
    existing_inventory_paths: set[str],
    existing_upstream_paths: set[str],
) -> bool:
    """Allow edits to tracked inventories, additions of new fixtures, and new version trees."""
    parts = Path(path).as_posix().split("/")
    if len(parts) < 3 or parts[0] != "specification" or parts[1] != "oasis":
        return False
    version = parts[2]
    if version not in APPROVED_VERSIONS:
        return False

    if len(parts) >= 4 and parts[3] == "upstream":
        if path in existing_upstream_paths:
            return False  # Upstream cannot be modified once pinned
        return statuses <= {"A"}  # New upstream additions are permitted

    if len(parts) >= 4 and parts[3] == "fixtures":
        return (
            path not in existing_fixture_paths
            and "A" in statuses
            and statuses <= {"A", "M"}
        )

    if len(parts) == 4 and parts[3] in APPROVED_VERSION_DOCS:
        if version == "kmip-2.1":
            return path in existing_inventory_paths and statuses == {"M"}
        if path in existing_inventory_paths:
            return statuses == {"M"}
        return statuses <= {"A"}

    return False


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
                if not path:
                    continue
                decoded = os.fsdecode(path)
                parts = decoded.split("/")
                if len(parts) >= 4 and parts[2] in APPROVED_VERSIONS and parts[3] == "fixtures":
                    fixture_count += 1
                else:
                    process.kill()
                    process.wait()
                    return True, fixture_count
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
    existing_upstream_paths = _base_upstream_paths(root, base_sha)
    unsupported_paths = {
        path
        for path, statuses in changed_statuses.items()
        if not _is_permitted_change(
            path,
            statuses,
            existing_fixture_paths,
            existing_inventory_paths,
            existing_upstream_paths,
        )
    }
    has_unsupported_untracked, untracked_fixture_count = _untracked_fixture_count(root)
    if unsupported_paths or has_unsupported_untracked:
        raise ImmutableSourceError(
            "only project OASIS README/SOURCES edits, approved version additions, and fixtures are permitted"
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
