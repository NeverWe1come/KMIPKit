"""Private C ABI handle and marshalling helpers.

Pointers remain in this module and in private handle fields; public accessors
return Python-owned values and never return CFFI pointers.
"""

from __future__ import annotations

from collections.abc import Callable
from functools import wraps
from threading import RLock
from types import FunctionType
from typing import Any, Self

from . import errors
from ._ffi import ffi, lib


_NATIVE_CALL_LOCK = RLock()


def synchronized_native_call(function: Callable[..., Any]) -> Callable[..., Any]:
    """Keep wrapper handles alive while one public operation crosses CFFI."""

    @wraps(function)
    def invoke(*arguments: object, **keywords: object) -> Any:
        with _NATIVE_CALL_LOCK:
            return function(*arguments, **keywords)

    return invoke


def synchronize_public_functions(namespace: dict[str, object], names: list[str]) -> None:
    """Serialize exported CFFI operations against native-handle release."""
    for name in names:
        function = namespace.get(name)
        if isinstance(function, FunctionType):
            namespace[name] = synchronized_native_call(function)


class NativeHandle:
    """Own one opaque C handle and release it deterministically."""

    __slots__ = ("__handle", "__lock", "_owner")
    _release_name: str

    def __init__(self, handle: Any, owner: object | None = None) -> None:
        if handle == ffi.NULL:
            errors.raise_for_status(4)
        self.__handle = handle
        self.__lock = RLock()
        self._owner = owner

    def _pointer(self) -> Any:
        with self.__lock:
            handle = self.__handle
            if handle == ffi.NULL:
                errors.raise_for_status(4)
            return handle

    def _take(self) -> tuple[Any, object | None]:
        """Transfer this handle to a manifest-declared consuming ABI call."""
        with self.__lock:
            handle = self._pointer()
            owner = self._owner
            self.__handle = ffi.NULL
            self._owner = None
            return handle, owner

    def _restore(self, handle: Any, owner: object | None) -> None:
        """Restore a transfer when CFFI rejects arguments before entering C."""
        with self.__lock:
            if self.__handle != ffi.NULL:
                errors.raise_for_status(4)
            self.__handle = handle
            self._owner = owner

    def _acquire_ownership_lock(self) -> None:
        self.__lock.acquire()

    def _release_ownership_lock(self) -> None:
        self.__lock.release()

    @property
    def closed(self) -> bool:
        with self.__lock:
            return self.__handle == ffi.NULL

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            with self.__lock:
                handle = self.__handle
                if handle == ffi.NULL:
                    return
                self.__handle = ffi.NULL
                owner = self._owner
                self._owner = None
            getattr(lib, self._release_name)(handle)
            del owner

    def __enter__(self) -> Self:
        self._pointer()
        return self

    def __exit__(self, exc_type: object, exc_value: object, traceback: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001, S110 - finalizers must not raise during shutdown.
            # Finalizers must not surface native cleanup failures at shutdown.
            pass

    def __repr__(self) -> str:
        return f"{type(self).__name__}(closed={self.closed})"


def _new_handle(
    c_type: str,
    function: Callable[..., int],
    *arguments: object,
    consumed: tuple[int, ...] = (),
) -> Any:
    """Call an out-handle ABI function and return its private CFFI value."""
    output = ffi.new(f"{c_type} **")
    call_arguments = (*arguments, output)
    status = (
        _invoke_consuming(function, call_arguments, consumed)
        if consumed
        else _invoke(function, *call_arguments)
    )
    errors.raise_for_status(status)
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    return output[0]


def _scalar(
    c_type: str,
    function: Callable[..., int],
    *arguments: object,
) -> int:
    output = ffi.new(f"{c_type} *")
    errors.raise_for_status(_invoke(function, *arguments, output))
    return int(output[0])


def _invoke(function: Callable[..., int], *arguments: object) -> int:
    """Normalize CFFI argument conversion failures without echoing inputs."""
    try:
        return int(function(*arguments))
    except (OverflowError, TypeError, ValueError):
        raise errors.InvalidInputError() from None


def _invoke_consuming(
    function: Callable[..., int],
    arguments: tuple[object, ...],
    consumed_indices: tuple[int, ...],
) -> int:
    """Transfer selected handle arguments and call their consuming C ABI function."""
    handles: list[tuple[int, NativeHandle]] = []
    seen: set[int] = set()
    for index in consumed_indices:
        if index < 0 or index >= len(arguments):
            errors.raise_for_status(4)
        handle = arguments[index]
        if not isinstance(handle, NativeHandle) or id(handle) in seen:
            errors.raise_for_status(4)
        seen.add(id(handle))
        handles.append((index, handle))

    locked_handles = sorted((handle for _, handle in handles), key=id)
    for handle in locked_handles:
        handle._acquire_ownership_lock()
    try:
        # Check every source before transferring any of them.
        for _, handle in handles:
            handle._pointer()
        call_arguments = list(arguments)
        tickets: list[tuple[NativeHandle, Any, object | None]] = []
        for index, handle in handles:
            pointer, owner = handle._take()
            call_arguments[index] = pointer
            tickets.append((handle, pointer, owner))
        try:
            return int(function(*call_arguments))
        except (OverflowError, TypeError, ValueError):
            for handle, pointer, owner in reversed(tickets):
                handle._restore(pointer, owner)
            raise errors.InvalidInputError() from None
    finally:
        for handle in reversed(locked_handles):
            handle._release_ownership_lock()


def _owned_bytes(value: str | bytes | bytearray | memoryview) -> tuple[bytes, Any]:
    """Encode or copy variable-length input for the duration of one ABI call."""
    if isinstance(value, str):
        try:
            raw = value.encode("utf-8")
        except UnicodeEncodeError:
            raise errors.InvalidInputError() from None
    elif isinstance(value, (bytes, bytearray, memoryview)):
        raw = bytes(value)
    else:
        errors.raise_for_status(4)
    return raw, ffi.new("uint8_t[]", raw)


def _handle_array(
    c_type: str, values: list[NativeHandle] | tuple[NativeHandle, ...]
) -> Any:
    """Build a typed opaque-handle array while keeping raw pointers private."""
    return ffi.new(f"{c_type} *[]", [value._pointer() for value in values])


def _array_or_null(
    c_type: str, values: list[NativeHandle] | tuple[NativeHandle, ...]
) -> Any:
    if not values:
        return ffi.NULL
    return _handle_array(c_type, values)
