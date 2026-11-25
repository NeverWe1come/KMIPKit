from __future__ import annotations

import importlib
import importlib.machinery
import importlib.util
from pathlib import Path
from types import ModuleType
from typing import Callable

import pytest


class _FailingGeneratedLoader:
    def __init__(self, failure_type: type[Exception]) -> None:
        self.failure_type = failure_type

    def create_module(
        self, _spec: importlib.machinery.ModuleSpec
    ) -> ModuleType | None:
        return None

    def exec_module(self, _module: ModuleType) -> None:
        raise self.failure_type("private generated declaration detail")


def _execute_ffi_initializer(
    monkeypatch: pytest.MonkeyPatch,
    spec_factory: Callable[..., object],
) -> None:
    package = importlib.import_module("kmipkit._ffi")
    assert package.__file__ is not None
    initializer = Path(package.__file__)
    factory = importlib.util.spec_from_file_location
    initializer_spec = factory(
        "kmipkit._ffi",
        initializer,
        submodule_search_locations=[str(initializer.parent)],
    )
    assert initializer_spec is not None
    assert initializer_spec.loader is not None
    module = importlib.util.module_from_spec(initializer_spec)

    monkeypatch.setattr(importlib.util, "spec_from_file_location", spec_factory)
    initializer_spec.loader.exec_module(module)


@pytest.mark.parametrize("failure_type", [ImportError, OSError])
def test_generated_ffi_execution_failures_are_normalized(
    monkeypatch: pytest.MonkeyPatch, failure_type: type[Exception]
) -> None:
    def failing_spec(name: str, _path: str | Path) -> importlib.machinery.ModuleSpec:
        return importlib.machinery.ModuleSpec(name, _FailingGeneratedLoader(failure_type))

    with pytest.raises(ImportError) as error:
        _execute_ffi_initializer(monkeypatch, failing_spec)

    assert str(error.value) == "KMIPKit CFFI declarations are unavailable"
    assert error.value.__suppress_context__


def test_generated_ffi_missing_spec_is_normalized(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def missing_spec(_name: str, _path: str | Path) -> None:
        return None

    with pytest.raises(ImportError) as error:
        _execute_ffi_initializer(monkeypatch, missing_spec)

    assert str(error.value) == "KMIPKit CFFI declarations are unavailable"
    assert error.value.__suppress_context__


def test_generated_ffi_missing_loader_is_normalized(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def missing_loader(name: str, _path: str | Path) -> importlib.machinery.ModuleSpec:
        return importlib.machinery.ModuleSpec(name, loader=None)

    with pytest.raises(ImportError) as error:
        _execute_ffi_initializer(monkeypatch, missing_loader)

    assert str(error.value) == "KMIPKit CFFI declarations are unavailable"
    assert error.value.__suppress_context__


def test_sphinx_build_env_mocks_native_ffi_and_client_ffi(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from unittest.mock import MagicMock

    monkeypatch.setenv("SPHINX_BUILD", "1")

    ffi_pkg = importlib.import_module("kmipkit._ffi")
    assert ffi_pkg.__file__ is not None
    ffi_spec = importlib.util.spec_from_file_location(
        "kmipkit._ffi",
        Path(ffi_pkg.__file__),
        submodule_search_locations=[str(Path(ffi_pkg.__file__).parent)],
    )
    assert ffi_spec is not None and ffi_spec.loader is not None
    ffi_module = importlib.util.module_from_spec(ffi_spec)
    ffi_spec.loader.exec_module(ffi_module)
    assert isinstance(ffi_module.ffi, MagicMock)
    assert isinstance(ffi_module.lib, MagicMock)
    assert ffi_module._bundled_path == Path("/dev/null")
    assert ffi_module._native_library_path == Path("/dev/null")

    client_pkg = importlib.import_module("kmipkit._client_ffi")
    assert client_pkg.__file__ is not None
    client_spec = importlib.util.spec_from_file_location(
        "kmipkit._client_ffi",
        Path(client_pkg.__file__),
    )
    assert client_spec is not None and client_spec.loader is not None
    client_module = importlib.util.module_from_spec(client_spec)
    client_spec.loader.exec_module(client_module)
    assert isinstance(client_module.ffi, MagicMock)
    assert isinstance(client_module.lib, MagicMock)


def test_client_ffi_dlopen_oserror_is_normalized(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import cffi

    monkeypatch.delenv("SPHINX_BUILD", raising=False)
    client_pkg = importlib.import_module("kmipkit._client_ffi")
    assert client_pkg.__file__ is not None
    client_spec = importlib.util.spec_from_file_location(
        "kmipkit._client_ffi",
        Path(client_pkg.__file__),
    )
    assert client_spec is not None and client_spec.loader is not None
    client_module = importlib.util.module_from_spec(client_spec)

    def failing_dlopen(self: object, _path: str) -> object:
        raise OSError("synthetic dlopen failure")

    monkeypatch.setattr(cffi.FFI, "dlopen", failing_dlopen)
    with pytest.raises(ImportError) as error:
        client_spec.loader.exec_module(client_module)

    assert str(error.value) == "KMIPKit native client ABI is unavailable"
    assert error.value.__suppress_context__
