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
