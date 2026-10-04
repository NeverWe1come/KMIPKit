"""Bounded, repository-confined file access for catalog tooling."""

from __future__ import annotations

import os
import secrets
import stat
from contextlib import contextmanager
from pathlib import Path, PurePosixPath
from typing import Iterator


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


def _windows_file_api() -> tuple[object, object, int]:
    """Load the small Win32 API surface needed for handle-based path checks."""
    import ctypes
    from ctypes import wintypes

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.CreateFileW.argtypes = (
        wintypes.LPCWSTR,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.LPVOID,
        wintypes.DWORD,
        wintypes.DWORD,
        wintypes.HANDLE,
    )
    kernel32.CreateFileW.restype = wintypes.HANDLE
    kernel32.GetFileInformationByHandleEx.argtypes = (
        wintypes.HANDLE,
        ctypes.c_int,
        wintypes.LPVOID,
        wintypes.DWORD,
    )
    kernel32.GetFileInformationByHandleEx.restype = wintypes.BOOL
    kernel32.GetFinalPathNameByHandleW.argtypes = (
        wintypes.HANDLE,
        wintypes.LPWSTR,
        wintypes.DWORD,
        wintypes.DWORD,
    )
    kernel32.GetFinalPathNameByHandleW.restype = wintypes.DWORD
    kernel32.CloseHandle.argtypes = (wintypes.HANDLE,)
    kernel32.CloseHandle.restype = wintypes.BOOL
    return kernel32, wintypes, wintypes.HANDLE(-1).value


def _windows_handle_attributes(kernel32: object, handle: object) -> int:
    import ctypes
    from ctypes import wintypes

    class FileAttributeTagInfo(ctypes.Structure):
        _fields_ = (("FileAttributes", wintypes.DWORD), ("ReparseTag", wintypes.DWORD))

    information = FileAttributeTagInfo()
    succeeded = kernel32.GetFileInformationByHandleEx(  # type: ignore[attr-defined]
        handle,
        9,  # FileAttributeTagInfo
        ctypes.byref(information),
        ctypes.sizeof(information),
    )
    if not succeeded:
        raise OSError(ctypes.get_last_error(), "could not inspect a repository path handle")
    return int(information.FileAttributes)


def _windows_final_path(kernel32: object, handle: object) -> str:
    import ctypes

    buffer = ctypes.create_unicode_buffer(32_768)
    length = kernel32.GetFinalPathNameByHandleW(  # type: ignore[attr-defined]
        handle,
        buffer,
        len(buffer),
        0,
    )
    if length == 0 or length >= len(buffer):
        raise OSError(ctypes.get_last_error(), "could not resolve a repository path handle")
    path = buffer.value
    if path.startswith("\\\\?\\UNC\\"):
        return "\\\\" + path[8:]
    if path.startswith("\\\\?\\"):
        return path[4:]
    return path


def _windows_is_within_root(root: Path, final_path: str) -> bool:
    resolved_root = os.path.normcase(os.path.abspath(root))
    resolved_path = os.path.normcase(os.path.abspath(final_path))
    try:
        return os.path.commonpath((resolved_root, resolved_path)) == resolved_root
    except ValueError:
        return False


def _windows_open_handle(
    kernel32: object,
    wintypes: object,
    path: Path,
    *,
    directory: bool,
    lock_for_replacement: bool = False,
) -> object:
    import ctypes

    generic_read = 0x80000000
    file_read_attributes = 0x0080
    delete_access = 0x00010000
    share_read_write = 0x00000001 | 0x00000002
    open_existing = 3
    open_reparse_point = 0x00200000
    backup_semantics = 0x02000000 if directory else 0
    handle = kernel32.CreateFileW(  # type: ignore[attr-defined]
        str(path),
        file_read_attributes | (delete_access if lock_for_replacement else 0)
        if directory
        else generic_read | file_read_attributes,
        share_read_write,
        None,
        open_existing,
        open_reparse_point | backup_semantics,
        None,
    )
    if handle == wintypes.HANDLE(-1).value:  # type: ignore[attr-defined]
        raise OSError(ctypes.get_last_error(), "could not open a repository path handle", str(path))
    return handle


@contextmanager
def _windows_directory_guard(root: Path, components: tuple[str, ...]) -> Iterator[None]:
    """Hold each directory without delete sharing, preventing path replacement."""
    import ctypes

    kernel32, wintypes, _ = _windows_file_api()
    handles: list[object] = []
    resolved_root = root.resolve(strict=True)
    current = resolved_root
    try:
        for index, component in enumerate((None, *components)):
            if component is not None:
                current = current / component
            handle = _windows_open_handle(
                kernel32,
                wintypes,
                current,
                directory=True,
                lock_for_replacement=index > 0,
            )
            try:
                attributes = _windows_handle_attributes(kernel32, handle)
                if attributes & 0x400 or not attributes & 0x10:
                    raise PathSecurityError("repository directory is a reparse point or non-directory")
                final_path = _windows_final_path(kernel32, handle)
                if not _windows_is_within_root(resolved_root, final_path):
                    raise PathSecurityError("repository directory resolves outside the repository")
                if os.path.normcase(os.path.abspath(final_path)) != os.path.normcase(os.path.abspath(current)):
                    raise PathSecurityError("repository directory handle resolves to an unexpected path")
            except BaseException:
                kernel32.CloseHandle(handle)  # type: ignore[attr-defined]
                raise
            handles.append(handle)
        yield
    except OSError as error:
        raise PathSecurityError("repository directories could not be locked safely") from error
    finally:
        for handle in reversed(handles):
            kernel32.CloseHandle(handle)  # type: ignore[attr-defined]


@contextmanager
def _windows_open_confined_file(root: Path, components: tuple[str, ...]) -> Iterator[int]:
    """Open a leaf under held, verified Windows directory handles."""
    import ctypes
    import msvcrt

    kernel32, wintypes, _ = _windows_file_api()
    resolved_root = root.resolve(strict=True)
    current = resolved_root.joinpath(*components[:-1])
    with _windows_directory_guard(root, components[:-1]):
        handle = _windows_open_handle(kernel32, wintypes, current / components[-1], directory=False)
        descriptor: int | None = None
        try:
            attributes = _windows_handle_attributes(kernel32, handle)
            if attributes & (0x400 | 0x10):
                raise PathSecurityError("repository input is a reparse point or directory")
            final_path = _windows_final_path(kernel32, handle)
            if not _windows_is_within_root(resolved_root, final_path):
                raise PathSecurityError("repository file resolves outside the repository")
            descriptor = msvcrt.open_osfhandle(handle, os.O_RDONLY | getattr(os, "O_BINARY", 0))
            handle = None
            yield descriptor
        except OSError as error:
            raise PathSecurityError("repository file could not be opened safely") from error
        finally:
            if descriptor is not None:
                os.close(descriptor)
            if handle is not None:
                kernel32.CloseHandle(handle)  # type: ignore[attr-defined]


def safe_read_bytes(root: Path, relative_path: str | Path, *, max_bytes: int) -> bytes:
    """Read at most ``max_bytes + 1`` bytes from a regular in-root file."""
    if max_bytes < 0:
        raise ValueError("max_bytes must be non-negative")
    path = confined_path(root, relative_path)
    if os.name == "nt":
        try:
            with _windows_open_confined_file(root, _parts(relative_path)) as descriptor:
                metadata = os.fstat(descriptor)
                if not stat.S_ISREG(metadata.st_mode):
                    raise PathSecurityError("repository input must be a regular file")
                with os.fdopen(descriptor, "rb", closefd=False) as stream:
                    raw = stream.read(max_bytes + 1)
                if len(raw) > max_bytes:
                    raise PathSecurityError("repository file exceeds its size limit")
                return raw
        except OSError as error:
            raise PathSecurityError("repository file could not be opened safely") from error
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

    if os.name == "nt":
        components = _parts(relative_path)
        current = confined_path(root, relative_path, allow_missing_leaf=True)
        parent = current.parent
        leaf = current.name
        temporary: Path | None = None
        try:
            with _windows_directory_guard(root, components[:-1]):
                try:
                    if current.exists() or current.is_symlink():
                        metadata = current.lstat()
                        if _reparse(metadata) or not stat.S_ISREG(metadata.st_mode):
                            raise PathSecurityError("report output must not be a symlink or non-file")
                    temporary = parent / f".{leaf}.{secrets.token_hex(8)}.tmp"
                    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0)
                    descriptor = os.open(temporary, flags, 0o600)
                    with os.fdopen(descriptor, "wb") as stream:
                        stream.write(data)
                        stream.flush()
                        os.fsync(stream.fileno())
                    if current.exists() or current.is_symlink():
                        metadata = current.lstat()
                        if _reparse(metadata) or not stat.S_ISREG(metadata.st_mode):
                            raise PathSecurityError("report output must not be a symlink or non-file")
                    os.replace(temporary, current)
                finally:
                    if temporary is not None:
                        try:
                            temporary.unlink()
                        except FileNotFoundError:
                            pass
            return
        except (OSError, PathSecurityError) as error:
            raise PathSecurityError("report could not be written atomically") from error

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
