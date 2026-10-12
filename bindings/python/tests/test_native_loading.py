from __future__ import annotations

from pathlib import Path

import pytest

from kmipkit._native import (
    NativeAbiMismatchError,
    NativeLibraryLoadError,
    load_native_library,
)


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


def test_abi_mismatch_fails_without_exposing_version_details(
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

    with pytest.raises(NativeAbiMismatchError):
        load_native_library(
            bundled_path=package_library,
            expected_abi_version=4,
            backend=lambda _: object(),
            abi_version=lambda _: 3,
            environ={},
        )
