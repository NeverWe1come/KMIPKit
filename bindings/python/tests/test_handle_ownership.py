"""Focused ownership tests for the private Python C ABI adapter."""

from __future__ import annotations

import gc
import importlib.util
import sys
import threading
import types
import unittest
from itertools import count
from pathlib import Path
from typing import Any

PACKAGE_PATH = Path(__file__).resolve().parents[1] / "src" / "kmipkit"
_PACKAGE_IDS = count()


def _load_adapter() -> tuple[types.ModuleType, types.ModuleType, str]:
    package_name = f"_kmipkit_handle_test_{next(_PACKAGE_IDS)}"
    package = types.ModuleType(package_name)
    package.__path__ = [str(PACKAGE_PATH)]
    sys.modules[package_name] = package

    errors_spec = importlib.util.spec_from_file_location(
        f"{package_name}.errors", PACKAGE_PATH / "errors.py"
    )
    if errors_spec is None or errors_spec.loader is None:
        raise AssertionError("could not load Python error categories")
    errors = importlib.util.module_from_spec(errors_spec)
    sys.modules[errors_spec.name] = errors
    errors_spec.loader.exec_module(errors)

    class FakeFFI:
        NULL = object()

        def new(self, declaration: str, initializer: object = None) -> object:
            if initializer is None:
                return [self.NULL]
            return declaration, initializer

    class FakeLibrary:
        releases = 0
        fail_release = False

        def release_handle(self, handle: object) -> None:
            del handle
            self.releases += 1
            if self.fail_release:
                raise RuntimeError("synthetic native cleanup failure")

        def __getattr__(self, name: str) -> object:
            if name.endswith("_release"):
                return self.release_handle
            if name.startswith("kmipkit_"):
                return lambda *_arguments: 0
            raise AttributeError(name)

    native = types.ModuleType(f"{package_name}._ffi")
    native.ffi = FakeFFI()
    native.lib = FakeLibrary()
    sys.modules[native.__name__] = native

    handles_spec = importlib.util.spec_from_file_location(
        f"{package_name}._handles", PACKAGE_PATH / "_handles.py"
    )
    if handles_spec is None or handles_spec.loader is None:
        raise AssertionError("could not load the private handle adapter")
    handles = importlib.util.module_from_spec(handles_spec)
    sys.modules[handles_spec.name] = handles
    handles_spec.loader.exec_module(handles)
    return handles, native, package_name


def _unload_adapter(package_name: str) -> None:
    for module_name in tuple(sys.modules):
        if module_name == package_name or module_name.startswith(f"{package_name}."):
            del sys.modules[module_name]


def _load_ttlv_adapter(package_name: str) -> types.ModuleType:
    ttlv_spec = importlib.util.spec_from_file_location(
        f"{package_name}.ttlv", PACKAGE_PATH / "ttlv.py"
    )
    if ttlv_spec is None or ttlv_spec.loader is None:
        raise AssertionError("could not load the Python TTLV adapter")
    ttlv = importlib.util.module_from_spec(ttlv_spec)
    sys.modules[ttlv_spec.name] = ttlv
    ttlv_spec.loader.exec_module(ttlv)
    return ttlv


class ConsumedHandleTests(unittest.TestCase):
    def test_native_view_call_pins_handle_until_cffi_returns(self) -> None:
        handles, native, package_name = _load_adapter()
        try:
            ttlv = _load_ttlv_adapter(package_name)
            view = ttlv.TtlvStructureView(object())
            native_call_entered = threading.Event()
            resume_native_call = threading.Event()
            close_started = threading.Event()
            close_completed = threading.Event()
            call_errors: list[BaseException] = []

            def pause_native_call(*_arguments: object) -> int:
                native_call_entered.set()
                if not resume_native_call.wait(timeout=2):
                    raise AssertionError("native call was not resumed")
                return 0

            ttlv._scalar = pause_native_call

            def inspect_view() -> None:
                try:
                    ttlv.ttlv_structure_view_item_count(view)
                except BaseException as error:  # noqa: BLE001 - cross-thread assertion capture.
                    call_errors.append(error)

            def close_view() -> None:
                close_started.set()
                view.close()
                close_completed.set()

            call_thread = threading.Thread(target=inspect_view)
            close_thread = threading.Thread(target=close_view)
            call_thread.start()
            self.assertTrue(native_call_entered.wait(timeout=2))
            close_thread.start()
            self.assertTrue(close_started.wait(timeout=2))
            closed_while_in_flight = close_completed.wait(timeout=0.05)
            resume_native_call.set()
            call_thread.join(timeout=2)
            close_thread.join(timeout=2)

            self.assertFalse(closed_while_in_flight)
            self.assertFalse(call_thread.is_alive())
            self.assertFalse(close_thread.is_alive())
            self.assertEqual(call_errors, [])
            self.assertEqual(native.lib.releases, 1)
        finally:
            _unload_adapter(package_name)

    def test_handle_marshalling_rejects_invalid_inputs_and_preserves_null_array_rules(
        self,
    ) -> None:
        handles, native, package_name = _load_adapter()
        try:
            with self.assertRaises(handles.errors.InvalidInputError):
                handles.NativeHandle(native.ffi.NULL)
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._owned_bytes("\ud800")
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._owned_bytes(object())
            raw, pointer = handles._owned_bytes(bytearray(b"value"))
            self.assertEqual(raw, b"value")
            self.assertEqual(pointer, ("uint8_t[]", b"value"))

            raw, pointer = handles._owned_bytes(memoryview(b"view"))
            self.assertEqual(raw, b"view")
            self.assertEqual(pointer, ("uint8_t[]", b"view"))
            with self.assertRaises(handles.errors.ResourceLimitError):
                handles._owned_bytes(memoryview(b"oversized"), maximum=4)
            released = memoryview(b"released")
            released.release()
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._owned_bytes(released)

            with self.assertRaises(handles.errors.InvalidInputError):
                handles._new_handle("kmipkit_probe_t", lambda *_: 0)
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._invoke(lambda *_: (_ for _ in ()).throw(TypeError()))
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._invoke_consuming(lambda *_: 0, (object(),), (1,))
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._invoke_consuming(lambda *_: 0, (object(),), (0,))

            class ProbeHandle(handles.NativeHandle):
                _release_name = "release_handle"

            source = ProbeHandle(object())
            with self.assertRaises(handles.errors.InvalidInputError):
                source._restore(object(), None)
            with self.assertRaises(handles.errors.InvalidInputError):
                handles._invoke_consuming(lambda *_: 0, (source,), (0, 0))
            self.assertIs(native.ffi.NULL, handles._array_or_null("kmipkit_probe_t", []))
            self.assertEqual(
                handles._array_or_null("kmipkit_probe_t", [source]),
                ("kmipkit_probe_t *[]", [source._pointer()]),
            )
            source.close()

            native.lib.fail_release = True
            finalizer_probe = ProbeHandle(object())
            del finalizer_probe
            gc.collect()
            native.lib.fail_release = False
            self.assertGreaterEqual(native.lib.releases, 2)
        finally:
            native.lib.fail_release = False
            _unload_adapter(package_name)

    def test_close_and_transfer_cannot_both_claim_the_native_handle(self) -> None:
        handles, native, package_name = _load_adapter()
        try:
            native_handle = object()
            pointer_read = threading.Event()
            continue_transfer = threading.Event()
            close_started = threading.Event()

            class PausingProbeHandle(handles.NativeHandle):
                _release_name = "release_handle"

                def _pointer(self) -> Any:
                    pointer = super()._pointer()
                    pointer_read.set()
                    if not continue_transfer.wait(timeout=2):
                        raise AssertionError("transfer was not resumed")
                    return pointer

            source = PausingProbeHandle(native_handle)
            transferred: list[object] = []

            def take() -> None:
                transferred.append(source._take()[0])

            def close() -> None:
                close_started.set()
                source.close()

            transfer_thread = threading.Thread(target=take)
            close_thread = threading.Thread(target=close)
            transfer_thread.start()
            self.assertTrue(pointer_read.wait(timeout=2))
            close_thread.start()
            self.assertTrue(close_started.wait(timeout=2))
            continue_transfer.set()
            transfer_thread.join(timeout=2)
            close_thread.join(timeout=2)

            self.assertFalse(transfer_thread.is_alive())
            self.assertFalse(close_thread.is_alive())
            if transferred:
                native.lib.release_handle(transferred[0])
            source.close()
            self.assertEqual(native.lib.releases, 1)
        finally:
            _unload_adapter(package_name)

    def test_close_after_native_consumes_handle_does_not_release_twice(self) -> None:
        handles, native, package_name = _load_adapter()
        try:
            native_handle = object()
            source = type(
                "ProbeHandle",
                (handles.NativeHandle,),
                {"_release_name": "release_handle"},
            )(native_handle)

            def consume(handle: Any) -> int:
                self.assertIs(handle, native_handle)
                native.lib.release_handle(handle)
                return 0

            self.assertEqual(handles._invoke_consuming(consume, (source,), (0,)), 0)
            self.assertTrue(source.closed)
            source.close()
            self.assertEqual(native.lib.releases, 1)
        finally:
            _unload_adapter(package_name)

    def test_cffi_conversion_error_restores_handle_for_deterministic_close(
        self,
    ) -> None:
        handles, native, package_name = _load_adapter()
        try:
            native_handle = object()
            source = type(
                "ProbeHandle",
                (handles.NativeHandle,),
                {"_release_name": "release_handle"},
            )(native_handle)

            def reject_before_native_call(_handle: Any) -> int:
                raise TypeError("argument conversion")

            with self.assertRaises(handles.errors.InvalidInputError):
                handles._invoke_consuming(reject_before_native_call, (source,), (0,))
            self.assertFalse(source.closed)
            source.close()
            self.assertEqual(native.lib.releases, 1)
        finally:
            _unload_adapter(package_name)


if __name__ == "__main__":
    unittest.main()
