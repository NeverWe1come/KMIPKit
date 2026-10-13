"""Private CFFI bridge with the manifest-defined native ABI handshake."""

from __future__ import annotations

import importlib.util
import os
from pathlib import Path
from types import ModuleType

from .._native import find_bundled_native_library, load_cffi_library


def _load_generated_ffi(module_path: Path) -> ModuleType:
    spec = importlib.util.spec_from_file_location(
        "kmipkit._ffi._generated_ffi", module_path
    )
    if spec is None or spec.loader is None:
        raise ImportError("generated CFFI declarations are unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_native_package = Path(__file__).parent.parent / "_native_ffi"
try:
    _ffi_module = _load_generated_ffi(_native_package / "ffi.py")
except (ImportError, OSError):
    raise ImportError("KMIPKit CFFI declarations are unavailable") from None

ffi = _ffi_module.ffi
_bundled_path = (
    _native_package / "_native_library_selected_by_override"
    if "KMIPKIT_NATIVE_LIBRARY" in os.environ
    else find_bundled_native_library(_native_package)
)
lib = load_cffi_library(
    ffi=ffi,
    bundled_path=_bundled_path,
    expected_abi_version=1,
)

__all__ = ["ffi", "lib"]
