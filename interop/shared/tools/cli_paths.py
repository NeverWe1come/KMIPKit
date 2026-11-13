"""Path validation for file locations supplied to interop command-line tools."""
from __future__ import annotations

from pathlib import Path
import re


def resolve_within(path: Path, roots: tuple[Path, ...], *, must_exist: bool = False) -> Path:
    """Resolve a CLI path and require it to stay in a caller-approved area."""
    resolved = path.resolve(strict=must_exist)
    allowed = tuple(root.resolve(strict=True) for root in roots)
    if not any(resolved == root or root in resolved.parents for root in allowed):
        raise ValueError(f"path is outside the allowed directories: {path}")
    return resolved


def validate_git_commit(commit: str) -> str:
    """Accept only full immutable SHA-1/SHA-256 object identifiers."""
    if not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", commit):
        raise ValueError("library_commit must be a full lowercase Git object id")
    return commit


def validate_git_path(path: str) -> str:
    """Accept a plain repository-relative source path, not a Git option/spec."""
    if (not re.fullmatch(r"[A-Za-z0-9_./-]+", path) or path.startswith(("/", "-"))
            or any(part in {"", ".", ".."} for part in path.split("/"))):
        raise ValueError("source path must be a normalized repository-relative path")
    return path
