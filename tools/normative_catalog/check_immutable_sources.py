"""Compare the complete OASIS source subtree with an exact Git base commit."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


class ImmutableSourceError(ValueError):
    """Raised when the protected OASIS source tree differs from its base."""


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


def check_immutable_sources(repo_root: Path, base_sha: str) -> dict[str, object]:
    """Require every tracked and untracked OASIS path to match ``base_sha``."""
    root = repo_root.resolve(strict=True)
    if re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", base_sha) is None:
        raise ImmutableSourceError("base must be a full lowercase Git commit SHA")

    try:
        resolved = _git(root, "rev-parse", "--verify", f"{base_sha}^{{commit}}").decode("ascii").strip()
    except UnicodeDecodeError as error:
        raise ImmutableSourceError("Git returned an invalid base commit identifier") from error
    if resolved != base_sha:
        raise ImmutableSourceError("base SHA does not resolve to that exact commit")

    tracked_diff = _git(
        root,
        "diff",
        "--no-renames",
        "--raw",
        "--full-index",
        "--no-ext-diff",
        base_sha,
        "--",
        "specification/oasis/",
    )
    untracked = _git(root, "ls-files", "--others", "-z", "--", "specification/oasis/")
    if tracked_diff or untracked:
        raise ImmutableSourceError("specification/oasis differs from the exact base commit")
    return {"base_sha": base_sha, "changed_path_count": 0}


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
    print(f"OASIS source tree matches base {result['base_sha']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
