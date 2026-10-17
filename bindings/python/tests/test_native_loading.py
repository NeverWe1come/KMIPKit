from __future__ import annotations

from pathlib import Path

import pytest

from kmipkit._native import (
    NativeAbiMismatchError,
    NativeLibraryLoadError,
    find_bundled_native_library,
    load_cffi_library,
    load_native_library,
)


class FakePointer(list[int]):
    def __init__(self) -> None:
        super().__init__([0])


class FakeCffi:
    def __init__(self, library: object) -> None:
        self.library = library
        self.loaded: list[str] = []

    def new(self, _: str) -> FakePointer:
        return FakePointer()

    def dlopen(self, path: str) -> object:
        self.loaded.append(path)
        return self.library


class VersionLibrary:
    def __init__(self, abi_version: int = 1, status: int = 0) -> None:
        self.abi_version = abi_version
        self.status = status

    def kmipkit_version_get(
        self, major: FakePointer, minor: FakePointer, patch: FakePointer, abi: FakePointer
    ) -> int:
        major[0], minor[0], patch[0], abi[0] = 1, 0, 0, self.abi_version
        return self.status


def test_cffi_loader_reads_canonical_abi_version_from_native_symbol(tmp_path: Path) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    native = VersionLibrary()
    ffi = FakeCffi(native)

    result = load_cffi_library(
        ffi=ffi,
        bundled_path=package_library,
        expected_abi_version=1,
        environ={},
    )

    assert result is native
    assert ffi.loaded == [str(package_library)]


def test_cffi_loader_redacts_nonzero_native_version_query(
    tmp_path: Path,
) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    ffi = FakeCffi(VersionLibrary(status=1))

    with pytest.raises(NativeLibraryLoadError) as error:
        load_cffi_library(
            ffi=ffi,
            bundled_path=package_library,
            expected_abi_version=1,
            environ={},
        )

    assert str(error.value) == (
        "KMIPKit could not verify native library ABI compatibility."
    )
    assert error.value.__suppress_context__


def test_bundled_native_library_redacts_directory_listing_oserror(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    package_directory = tmp_path / "kmipkit" / "_native_ffi"
    package_directory.mkdir(parents=True)
    original_iterdir = Path.iterdir

    def fail_directory_listing(directory: Path):
        if directory == package_directory:
            raise OSError("private directory detail")
        return original_iterdir(directory)

    monkeypatch.setattr(Path, "iterdir", fail_directory_listing)

    with pytest.raises(NativeLibraryLoadError) as error:
        find_bundled_native_library(package_directory)

    assert "private directory detail" not in str(error.value)
    assert str(package_directory) not in str(error.value)
    assert error.value.__suppress_context__


def test_load_uses_library_adjacent_to_installed_package(tmp_path: Path) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    loaded: list[str] = []
    library = object()

    result = load_native_library(
        bundled_path=package_library,
        expected_abi_version=4,
        backend=lambda path: loaded.append(path) or library,
        abi_version=lambda _: 4,
        environ={},
    )

    assert result is library
    assert loaded == [str(package_library)]


def test_load_uses_only_the_explicit_administrator_path_when_configured(
    tmp_path: Path,
) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    admin_library = tmp_path / "admin" / "native.dll"
    admin_library.parent.mkdir()
    admin_library.touch()
    loaded: list[str] = []
    library = object()

    result = load_native_library(
        bundled_path=package_library,
        expected_abi_version=4,
        backend=lambda path: loaded.append(path) or library,
        abi_version=lambda _: 4,
        environ={"KMIPKIT_NATIVE_LIBRARY": str(admin_library)},
    )

    assert result is library
    assert loaded == [str(admin_library)]


def test_missing_administrator_path_fails_without_bundled_fallback(
    tmp_path: Path,
) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    private_path = tmp_path / "admin" / "private-native-token.dll"
    loaded: list[str] = []

    with pytest.raises(NativeLibraryLoadError) as error:
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=4,
            backend=lambda path: loaded.append(path),
            abi_version=lambda _: 4,
            environ={"KMIPKIT_NATIVE_LIBRARY": str(private_path)},
        )

    assert loaded == []
    assert "private-native-token" not in str(error.value)


def test_loader_failure_is_redacted(tmp_path: Path) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    private_detail = "private-native-token"

    def failing_backend(_: str) -> object:
        raise OSError(private_detail)

    with pytest.raises(NativeLibraryLoadError) as error:
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=4,
            backend=failing_backend,
            abi_version=lambda _: 4,
            environ={},
        )

    assert private_detail not in str(error.value)
    assert error.value.__suppress_context__


def test_wrong_architecture_binary_failure_is_redacted(tmp_path: Path) -> None:
    package_library = tmp_path / "kmipkit" / "_native_ffi" / "_native_ffi.so"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    loaded: list[str] = []

    def reject_wrong_architecture(path: str) -> object:
        loaded.append(path)
        raise OSError(f"invalid ELF class for native library at {path}")

    with pytest.raises(NativeLibraryLoadError) as error:
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=1,
            backend=reject_wrong_architecture,
            abi_version=lambda _: 1,
            environ={},
        )

    assert loaded == [str(package_library)]
    assert str(error.value) == (
        "KMIPKit could not load the selected native library; install a package "
        "for this OS and architecture or set KMIPKIT_NATIVE_LIBRARY to a "
        "compatible library."
    )
    assert "invalid ELF class" not in str(error.value)
    assert str(package_library) not in str(error.value)
    assert error.value.__suppress_context__


def test_abi_inspection_failure_is_redacted(
    tmp_path: Path,
) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()
    private_detail = "private-native-token"

    def private_abi_version(_: object) -> int:
        raise RuntimeError(private_detail)

    with pytest.raises(NativeLibraryLoadError) as error:
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=4,
            backend=lambda _: object(),
            abi_version=private_abi_version,
            environ={},
        )

    assert private_detail not in str(error.value)
    assert error.value.__suppress_context__


def test_reported_abi_mismatch_has_a_distinct_error(tmp_path: Path) -> None:
    package_library = tmp_path / "kmipkit" / "_ffi" / "_ffi.dll"
    package_library.parent.mkdir(parents=True)
    package_library.touch()

    with pytest.raises(NativeAbiMismatchError) as error:
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=4,
            backend=lambda _: object(),
            abi_version=lambda _: 3,
            environ={},
        )

    assert "3" not in str(error.value)
    assert "4" not in str(error.value)
    assert str(error.value) == (
        "KMIPKit native library has an incompatible ABI version; install a "
        "matching package or set KMIPKIT_NATIVE_LIBRARY to a compatible library."
    )


@pytest.mark.parametrize("name", ["_native_ffi.dll", "_native_ffi.so", "_native_ffi.dylib"])
def test_bundled_native_library_finds_shared_library_by_platform_suffix(
    tmp_path: Path, name: str
) -> None:
    package_directory = tmp_path / "kmipkit" / "_native_ffi"
    package_directory.mkdir(parents=True)
    candidate = package_directory / name
    candidate.touch()

    assert find_bundled_native_library(package_directory) == candidate


def test_bundled_native_library_rejects_missing_or_ambiguous_artifacts(
    tmp_path: Path,
) -> None:
    package_directory = tmp_path / "kmipkit" / "_native_ffi"
    package_directory.mkdir(parents=True)

    with pytest.raises(NativeLibraryLoadError) as missing_error:
        find_bundled_native_library(package_directory)
    assert str(package_directory) not in str(missing_error.value)

    (package_directory / "_native_ffi.dll").touch()
    (package_directory / "_native_ffi.so").touch()
    with pytest.raises(NativeLibraryLoadError) as ambiguous_error:
        find_bundled_native_library(package_directory)
    assert str(package_directory) not in str(ambiguous_error.value)
