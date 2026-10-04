"""Bounded, repository-confined file access for catalog tooling."""

from __future__ import annotations

import os
import secrets
import stat
from pathlib import Path, PurePosixPath


class PathSecurityError(ValueError):
    """Raised when a catalog path is unsafe or exceeds its input bound."""


def _reparse(metadata: os.stat_result) -> bool:
    reparse_attribute = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    return stat.S_ISLNK(metadata.st_mode) or bool(getattr(metadata, "st_file_attributes", 0) & reparse_attribute)


def _parts(relative_path: str | Path) -> tuple[str, ...]:
    raw = str(relative_path)
    parsed = PurePosixPath(raw)
    if (
        not raw
        or "\\" in raw
        or ":" in raw
        or parsed.is_absolute()
        or any(part in {"", ".", ".."} for part in raw.split("/"))
    ):
        raise PathSecurityError("path must be a confined repository-relative POSIX path")
    return parsed.parts


def confined_path(root: Path, relative_path: str | Path, *, allow_missing_leaf: bool = False) -> Path:
    """Return a path after validating every existing component against links."""
    try:
        resolved_root = root.resolve(strict=True)
    except OSError as error:
        raise PathSecurityError("repository root is unavailable") from error
    components = _parts(relative_path)
    current = resolved_root
    for index, component in enumerate(components):
        current = current / component
        is_leaf = index == len(components) - 1
        try:
            metadata = current.lstat()
        except FileNotFoundError as error:
            if allow_missing_leaf and is_leaf:
                break
            raise PathSecurityError("repository path is unavailable") from error
        except OSError as error:
            raise PathSecurityError("repository path cannot be inspected") from error
        if _reparse(metadata):
            raise PathSecurityError("repository paths cannot contain symlinks or reparse points")
        if not is_leaf and not stat.S_ISDIR(metadata.st_mode):
            raise PathSecurityError("repository path parent is not a directory")
        try:
            resolved = current.resolve(strict=True)
            resolved.relative_to(resolved_root)
        except (OSError, ValueError) as error:
            raise PathSecurityError("repository path resolves outside the repository") from error
    return current


def safe_read_bytes(root: Path, relative_path: str | Path, *, max_bytes: int) -> bytes:
    """Read at most ``max_bytes + 1`` bytes from a regular in-root file."""
    if max_bytes < 0:
        raise ValueError("max_bytes must be non-negative")
    path = confined_path(root, relative_path)
    try:
        if os.name == "posix" and os.open in os.supports_dir_fd:
            components = _parts(relative_path)
            descriptor = _open_confined_file(root, components)
        else:
            flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
            descriptor = os.open(path, flags)
    except OSError as error:
        raise PathSecurityError("repository file could not be opened safely") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or _reparse(metadata):
            raise PathSecurityError("repository input must be a regular file")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            raw = stream.read(max_bytes + 1)
        if len(raw) > max_bytes:
            raise PathSecurityError("repository file exceeds its size limit")
        return raw
    finally:
        os.close(descriptor)


def _open_rooted_directory(root: Path, components: tuple[str, ...]) -> int:
    directory_flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(root.resolve(strict=True), directory_flags)
    try:
        for component in components:
            child = os.open(component, directory_flags, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        return descriptor
    except OSError:
        os.close(descriptor)
        raise


def _open_confined_file(root: Path, components: tuple[str, ...]) -> int:
    directory_fd = _open_rooted_directory(root, components[:-1])
    try:
        flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
        return os.open(components[-1], flags, dir_fd=directory_fd)
    finally:
        os.close(directory_fd)


def _existing_leaf_is_regular(parent_fd: int, leaf: str) -> None:
    try:
        metadata = os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
    except FileNotFoundError:
        return
    if _reparse(metadata) or not stat.S_ISREG(metadata.st_mode):
        raise PathSecurityError("report output must not be a symlink or non-file")


def atomic_write_bytes(root: Path, relative_path: str | Path, data: bytes) -> None:
    """Atomically replace an in-root regular file without following links."""
    path = confined_path(root, relative_path, allow_missing_leaf=True)
    parent = path.parent
    leaf = path.name

    if (
        os.name == "posix"
        and os.open in os.supports_dir_fd
        and os.stat in os.supports_dir_fd
        and os.stat in os.supports_follow_symlinks
        and os.replace in os.supports_dir_fd
        and os.unlink in os.supports_dir_fd
    ):
        try:
            parent_fd = _open_rooted_directory(root, _parts(relative_path)[:-1])
        except OSError as error:
            raise PathSecurityError("report output directory could not be opened safely") from error
        temporary_name = f".{leaf}.{secrets.token_hex(8)}.tmp"
        descriptor: int | None = None
        try:
            _existing_leaf_is_regular(parent_fd, leaf)
            descriptor = os.open(
                temporary_name,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
                0o600,
                dir_fd=parent_fd,
            )
            with os.fdopen(descriptor, "wb") as stream:
                descriptor = None
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
            _existing_leaf_is_regular(parent_fd, leaf)
            os.replace(temporary_name, leaf, src_dir_fd=parent_fd, dst_dir_fd=parent_fd)
            os.fsync(parent_fd)
        except OSError as error:
            raise PathSecurityError("report could not be written atomically") from error
        finally:
            if descriptor is not None:
                os.close(descriptor)
            try:
                os.unlink(temporary_name, dir_fd=parent_fd)
            except FileNotFoundError:
                pass
            os.close(parent_fd)
        return

    current = confined_path(root, relative_path, allow_missing_leaf=True)
    try:
        if current.exists() or current.is_symlink():
            metadata = current.lstat()
            if _reparse(metadata) or not stat.S_ISREG(metadata.st_mode):
                raise PathSecurityError("report output must not be a symlink or non-file")
        temporary = current.with_name(f".{leaf}.{secrets.token_hex(8)}.tmp")
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
        descriptor = os.open(temporary, flags, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        current = confined_path(root, relative_path, allow_missing_leaf=True)
        if current.exists() or current.is_symlink():
            metadata = current.lstat()
            if _reparse(metadata) or not stat.S_ISREG(metadata.st_mode):
                raise PathSecurityError("report output must not be a symlink or non-file")
        os.replace(temporary, current)
    except OSError as error:
        raise PathSecurityError("report could not be written atomically") from error
    finally:
        if "temporary" in locals():
            try:
                temporary.unlink()
            except FileNotFoundError:
                pass
