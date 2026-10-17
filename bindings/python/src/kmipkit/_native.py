"""Private native-library selection and ABI validation helpers.

``KMIPKIT_NATIVE_LIBRARY`` is the only administrator override. When it is
unset, callers provide the library bundled adjacent to the installed package.
This module never downloads or searches for executable code.
"""

from __future__ import annotations

import os
from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Protocol, TypeVar


_Library = TypeVar("_Library")
_ADMIN_LIBRARY_ENV = "KMIPKIT_NATIVE_LIBRARY"


class _CffiPointer(Protocol):
    def __getitem__(self, index: int, /) -> int: ...


class _NativeLibrary(Protocol):
    def kmipkit_version_get(
        self,
        major: _CffiPointer,
        minor: _CffiPointer,
        patch: _CffiPointer,
        abi_version: _CffiPointer,
        /,
    ) -> int: ...


class _Cffi(Protocol):
    def new(self, cdecl: str, /) -> _CffiPointer: ...

    def dlopen(self, path: str, /) -> _NativeLibrary: ...


class NativeLibraryLoadError(RuntimeError):
    """A native library could not be loaded or its ABI could not be checked."""


class NativeAbiMismatchError(NativeLibraryLoadError):
    """The loaded native library does not expose the expected ABI version."""


def load_native_library(
    *,
    bundled_path: str | Path,
    expected_abi_version: int,
    backend: Callable[[str], _Library],
    abi_version: Callable[[_Library], int],
    environ: Mapping[str, str] | None = None,
) -> _Library:
    """Load a bundled or explicitly selected library and verify its ABI.

    ``backend`` and ``abi_version`` are injected so package loading can be
    tested without depending on a particular C ABI handshake symbol.
    """
    environment = os.environ if environ is None else environ
    administrator_path = environment.get(_ADMIN_LIBRARY_ENV)
    selected_path = (
        Path(administrator_path)
        if administrator_path is not None
        else Path(bundled_path)
    )

    if not selected_path.is_file():
        raise NativeLibraryLoadError(
            "KMIPKit native library is unavailable; install a matching package "
            "or set KMIPKIT_NATIVE_LIBRARY to an administrator-provided library."
        )

    try:
        library = backend(str(selected_path))
    except Exception:
        raise NativeLibraryLoadError(
            "KMIPKit could not load the selected native library; install a package "
            "for this OS and architecture or set KMIPKIT_NATIVE_LIBRARY to a "
            "compatible library."
        ) from None

    try:
        actual_abi_version = abi_version(library)
    except Exception:
        raise NativeLibraryLoadError(
            "KMIPKit could not verify native library ABI compatibility."
        ) from None

    if (
        type(actual_abi_version) is not int
        or actual_abi_version != expected_abi_version
    ):
        raise NativeAbiMismatchError(
            "KMIPKit native library has an incompatible ABI version; install a "
            "matching package or set KMIPKIT_NATIVE_LIBRARY to a compatible library."
        )

    return library


def find_bundled_native_library(package_directory: str | Path) -> Path:
    """Find the sole regular shared library installed beside generated CFFI glue."""
    directory = Path(package_directory)
    try:
        candidates = sorted(
            path
            for path in directory.iterdir()
            if path.suffix.lower() in {".dll", ".dylib", ".so"}
            and not path.is_symlink()
            and path.is_file()
        )
    except OSError:
        raise NativeLibraryLoadError(
            "KMIPKit native library is unavailable; install a matching package or "
            "set KMIPKIT_NATIVE_LIBRARY to an administrator-provided library."
        ) from None

    if len(candidates) != 1:
        raise NativeLibraryLoadError(
            "KMIPKit native library is unavailable; install a matching package or "
            "set KMIPKIT_NATIVE_LIBRARY to an administrator-provided library."
        )
    return candidates[0]


def load_cffi_library(
    *,
    ffi: _Cffi,
    bundled_path: str | Path,
    expected_abi_version: int,
    environ: Mapping[str, str] | None = None,
) -> object:
    """Load through CFFI and verify the ABI with the canonical version symbol."""

    def read_abi_version(library: _NativeLibrary) -> int:
        major = ffi.new("uint32_t *")
        minor = ffi.new("uint32_t *")
        patch = ffi.new("uint32_t *")
        abi_version = ffi.new("uint32_t *")
        status = library.kmipkit_version_get(major, minor, patch, abi_version)
        if status != 0:
            raise RuntimeError("native version query failed")
        return abi_version[0]

    return load_native_library(
        bundled_path=bundled_path,
        expected_abi_version=expected_abi_version,
        backend=ffi.dlopen,
        abi_version=read_abi_version,
        environ=environ,
    )
