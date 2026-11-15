"""Typed, in-memory KMIP 2.1 asynchronous operation request models."""

from __future__ import annotations

from collections.abc import Callable, Iterable
from typing import Any, TypeVar

from . import errors, ttlv
from ._ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _NATIVE_CALL_LOCK,
    _integer_range,
    _new_handle,
    _wrap_owned_handle,
)

_MAX_INPUT_BYTES = 16 * 1024 * 1024
_MAX_FILTER_COUNT = 100_000
_T = TypeVar("_T")
_RequestT = TypeVar("_RequestT", bound=NativeHandle)


def _correlation_staging(value: bytes | bytearray | memoryview) -> bytearray:
    if not isinstance(value, (bytes, bytearray, memoryview)):
        errors.raise_for_status(4)
    try:
        view = memoryview(value)
    except (TypeError, ValueError):
        errors.raise_for_status(4)
    if view.nbytes > _MAX_INPUT_BYTES:
        errors.raise_for_status(6)
    try:
        return bytearray(view)
    except (BufferError, TypeError, ValueError):
        errors.raise_for_status(4)


def _wipe(staging: bytearray) -> None:
    for index in range(len(staging)):
        staging[index] = 0


def _create_request(
    wrapper: type[_RequestT],
    c_type: str,
    function: Callable[..., int],
    value: bytes | bytearray | memoryview,
) -> _RequestT:
    staging = _correlation_staging(value)
    try:
        source = ffi.from_buffer("const uint8_t[]", staging) if staging else ffi.NULL
        with _NATIVE_CALL_LOCK:
            handle = _new_handle(
                c_type,
                function,
                source,
                len(staging),
            )
        return _wrap_owned_handle(wrapper, handle)
    finally:
        _wipe(staging)


def _request_payload(
    request: NativeHandle, function: Callable[..., int]
) -> ttlv.Structure:
    with _NATIVE_CALL_LOCK:
        output = _new_handle(
            "kmipkit_ttlv_structure_t",
            function,
            request._pointer(),
        )
    return ttlv._structure_from_native_address(int(ffi.cast("uintptr_t", output)))


def _correlation_values(values: Iterable[bytes | bytearray | memoryview]) -> list[bytearray]:
    try:
        iterator = iter(values)
    except TypeError:
        errors.raise_for_status(4)
    staged: list[bytearray] = []
    total_bytes = 0
    try:
        for value in iterator:
            if len(staged) >= _MAX_FILTER_COUNT:
                errors.raise_for_status(6)
            item = _correlation_staging(value)
            if len(item) > _MAX_INPUT_BYTES - total_bytes:
                _wipe(item)
                errors.raise_for_status(6)
            total_bytes += len(item)
            try:
                staged.append(item)
            except BaseException:
                _wipe(item)
                raise
        return staged
    except BaseException:
        for previous in staged:
            _wipe(previous)
        raise


def _with_correlation_values(
    request: QueryAsyncRequestsRequest,
    values: Iterable[bytes | bytearray | memoryview],
) -> QueryAsyncRequestsRequest:
    staged = _correlation_values(values)
    try:
        spans = ffi.new("kmipkit_byte_span_v1_t[]", len(staged)) if staged else ffi.NULL
        for index, item in enumerate(staged):
            spans[index].data = ffi.from_buffer("const uint8_t[]", item) if item else ffi.NULL
            spans[index].length = len(item)
        with _NATIVE_CALL_LOCK:
            handle = _new_handle(
                "kmipkit_query_async_requests_request_t",
                lib.kmipkit_query_async_requests_request_with_correlation_values,
                request._pointer(),
                spans,
                len(staged),
            )
        return _wrap_owned_handle(QueryAsyncRequestsRequest, handle)
    finally:
        for item in staged:
            _wipe(item)


def _with_operations(
    request: QueryAsyncRequestsRequest, operations: Iterable[int]
) -> QueryAsyncRequestsRequest:
    try:
        iterator = iter(operations)
    except TypeError:
        errors.raise_for_status(4)
    values: list[int] = []
    for operation in iterator:
        if len(values) >= _MAX_FILTER_COUNT:
            errors.raise_for_status(6)
        values.append(_integer_range(operation, 0, 0xFFFF_FFFF))
    array = ffi.new("uint32_t[]", values) if values else ffi.NULL
    with _NATIVE_CALL_LOCK:
        handle = _new_handle(
            "kmipkit_query_async_requests_request_t",
            lib.kmipkit_query_async_requests_request_with_operations,
            request._pointer(),
            array,
            len(values),
        )
    return _wrap_owned_handle(QueryAsyncRequestsRequest, handle)


class CancelRequest(NativeHandle):
    """Owned Cancel request model; construction and encoding perform no network I/O."""

    _release_name = "kmipkit_cancel_request_release"

    @classmethod
    def create(cls, asynchronous_correlation_value: bytes | bytearray | memoryview) -> CancelRequest:
        """Create a request from the exact opaque correlation bytes."""
        return _create_request(
            cls,
            "kmipkit_cancel_request_t",
            lib.kmipkit_cancel_request_create,
            asynchronous_correlation_value,
        )

    def with_asynchronous_correlation_value(
        self, callback: Callable[[memoryview], _T]
    ) -> _T:
        """Lend a read-only correlation view to ``callback`` and wipe its backing copy afterward."""
        if not callable(callback):
            errors.raise_for_status(4)
        span = ffi.new("kmipkit_byte_span_v1_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                int(
                    lib.kmipkit_cancel_request_asynchronous_correlation_value(
                        self._pointer(), span
                    )
                )
            )
            return _call_with_span(span, callback)

    def to_ttlv_payload(self) -> ttlv.Structure:
        """Return an owned generic TTLV copy of the Cancel Request Payload."""
        return _request_payload(self, lib.kmipkit_cancel_request_to_ttlv_payload)


class PollRequest(NativeHandle):
    """Owned Poll request model; construction and encoding perform no network I/O."""

    _release_name = "kmipkit_poll_request_release"

    @classmethod
    def create(cls, asynchronous_correlation_value: bytes | bytearray | memoryview) -> PollRequest:
        """Create a request from the exact opaque correlation bytes."""
        return _create_request(
            cls,
            "kmipkit_poll_request_t",
            lib.kmipkit_poll_request_create,
            asynchronous_correlation_value,
        )

    def with_asynchronous_correlation_value(
        self, callback: Callable[[memoryview], _T]
    ) -> _T:
        """Lend a read-only correlation view to ``callback`` and wipe its backing copy afterward."""
        if not callable(callback):
            errors.raise_for_status(4)
        span = ffi.new("kmipkit_byte_span_v1_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                int(
                    lib.kmipkit_poll_request_asynchronous_correlation_value(
                        self._pointer(), span
                    )
                )
            )
            return _call_with_span(span, callback)

    def to_ttlv_payload(self) -> ttlv.Structure:
        """Return an owned generic TTLV copy of the Poll Request Payload."""
        return _request_payload(self, lib.kmipkit_poll_request_to_ttlv_payload)


class ProcessRequest(NativeHandle):
    """Owned Process request model; construction and encoding perform no network I/O."""

    _release_name = "kmipkit_process_request_release"

    @classmethod
    def create(cls, asynchronous_correlation_value: bytes | bytearray | memoryview) -> ProcessRequest:
        """Create a request from the exact opaque correlation bytes."""
        return _create_request(
            cls,
            "kmipkit_process_request_t",
            lib.kmipkit_process_request_create,
            asynchronous_correlation_value,
        )

    def to_ttlv_payload(self) -> ttlv.Structure:
        """Return an owned generic TTLV copy of the Process Request Payload."""
        return _request_payload(self, lib.kmipkit_process_request_to_ttlv_payload)


class QueryAsyncRequestsRequest(NativeHandle):
    """Owned Query Asynchronous Requests model with lossless optional filters."""

    _release_name = "kmipkit_query_async_requests_request_release"

    @classmethod
    def create(cls) -> QueryAsyncRequestsRequest:
        """Create an unfiltered query request model."""
        with _NATIVE_CALL_LOCK:
            handle = _new_handle(
                "kmipkit_query_async_requests_request_t",
                lib.kmipkit_query_async_requests_request_create,
            )
        return _wrap_owned_handle(cls, handle)

    def with_correlation_values(
        self, values: Iterable[bytes | bytearray | memoryview]
    ) -> QueryAsyncRequestsRequest:
        """Return a copy with an ordered correlation filter; an empty iterable is present-empty."""
        return _with_correlation_values(self, values)

    def with_operations(self, operations: Iterable[int]) -> QueryAsyncRequestsRequest:
        """Return a copy with ordered raw Operation values, including future values."""
        return _with_operations(self, operations)

    def to_ttlv_payload(self) -> ttlv.Structure:
        """Return an owned generic TTLV copy of the query Request Payload."""
        return _request_payload(self, lib.kmipkit_query_async_requests_request_to_ttlv_payload)


def _call_with_span(span: Any, callback: Callable[[memoryview], _T]) -> _T:
    data = span[0].data
    length = int(span[0].length)
    if length < 0 or length > _MAX_INPUT_BYTES or (length > 0 and data == ffi.NULL):
        errors.raise_for_status(4)
    if length == 0 and data != ffi.NULL:
        errors.raise_for_status(4)
    secret = bytearray(ffi.buffer(data, length)) if length else bytearray()
    borrowed: memoryview | None = None
    try:
        borrowed = memoryview(secret).toreadonly()
        return callback(borrowed)
    finally:
        if borrowed is not None:
            try:
                borrowed.release()
            except ValueError:
                pass
        _wipe(secret)
    return True


__all__ = [
    "CancelRequest",
    "PollRequest",
    "ProcessRequest",
    "QueryAsyncRequestsRequest",
]
