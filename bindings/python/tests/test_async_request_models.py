"""KMIP 2.1 §6.1.5, §6.1.38, §6.1.39, and §6.1.41 request model parity."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import async_operations, errors, ttlv
from kmipkit.async_operations import (
    CancelRequest,
    PollRequest,
    ProcessRequest,
    QueryAsyncRequestsRequest,
)


CORRELATION_VALUE_TAG = 0x00420006
CORRELATION_VALUES_TAG = 0x00420176
OPERATIONS_TAG = 0x0042014F
OPERATION_TAG = 0x0042005C


def _tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    result: list[int] = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            tag = ttlv.ttlv_item_view_tag(item)
            try:
                result.append(ttlv.tag_value(tag))
            finally:
                tag.close()
        finally:
            item.close()
    return tuple(result)


def _bytes_at(view: ttlv.TtlvStructureView, index: int) -> bytes:
    item = ttlv.ttlv_structure_view_item_at(view, index)
    try:
        value = ttlv.ttlv_item_view_value(item)
        try:
            length = ttlv.ttlv_value_view_byte_length(value)
            return bytes(ttlv.ttlv_value_view_byte_at(value, offset) for offset in range(length))
        finally:
            value.close()
    finally:
        item.close()


def _enumeration_at(view: ttlv.TtlvStructureView, index: int) -> int:
    item = ttlv.ttlv_structure_view_item_at(view, index)
    try:
        value = ttlv.ttlv_item_view_value(item)
        try:
            return ttlv.ttlv_value_view_enumeration(value)
        finally:
            value.close()
    finally:
        item.close()


def test_cancel_poll_and_process_build_payloads_without_executing_operations() -> None:
    correlation = b"\x00\xff\x80"
    for request_type in (CancelRequest, PollRequest, ProcessRequest):
        with request_type.create(correlation) as request:
            if request_type in (CancelRequest, PollRequest):
                observed: list[bytes] = []

                def inspect(value: memoryview) -> bytes:
                    assert value.readonly
                    observed.append(value.tobytes())
                    return value.tobytes()

                assert request.with_asynchronous_correlation_value(inspect) == correlation
                assert observed == [correlation]

            with request.to_ttlv_payload() as payload:
                with ttlv.ttlv_structure_view(payload) as view:
                    assert _tags(view) == (CORRELATION_VALUE_TAG,)
                    assert _bytes_at(view, 0) == correlation


def test_query_request_preserves_absent_and_present_ordered_filters() -> None:
    with QueryAsyncRequestsRequest.create() as empty_request:
        with empty_request.to_ttlv_payload() as empty_payload:
            with ttlv.ttlv_structure_view(empty_payload) as view:
                assert _tags(view) == ()

    correlation_values = (b"\x00\xff", b"\x10\x00", b"\x00\xff")
    operation_values = (0x0000001A, 0xF1234567)
    request = QueryAsyncRequestsRequest.create()
    request = request.with_correlation_values(correlation_values)
    with request.with_operations(operation_values) as filtered_request:
        with filtered_request.to_ttlv_payload() as payload:
            with ttlv.ttlv_structure_view(payload) as view:
                assert _tags(view) == (CORRELATION_VALUES_TAG, OPERATIONS_TAG)

                correlation_item = ttlv.ttlv_structure_view_item_at(view, 0)
                try:
                    correlation_value = ttlv.ttlv_item_view_value(correlation_item)
                    try:
                        correlations = ttlv.ttlv_value_view_structure(correlation_value)
                        try:
                            assert _tags(correlations) == (CORRELATION_VALUE_TAG,) * 3
                            assert tuple(
                                _bytes_at(correlations, index) for index in range(3)
                            ) == correlation_values
                        finally:
                            correlations.close()
                    finally:
                        correlation_value.close()
                finally:
                    correlation_item.close()

                operations_item = ttlv.ttlv_structure_view_item_at(view, 1)
                try:
                    operations_value = ttlv.ttlv_item_view_value(operations_item)
                    try:
                        operations = ttlv.ttlv_value_view_structure(operations_value)
                        try:
                            assert _tags(operations) == (OPERATION_TAG,) * 2
                            assert tuple(
                                _enumeration_at(operations, index) for index in range(2)
                            ) == operation_values
                        finally:
                            operations.close()
                    finally:
                        operations_value.close()
                finally:
                    operations_item.close()


def test_query_filter_wipes_staged_values_when_iteration_fails(monkeypatch: pytest.MonkeyPatch) -> None:
    source = bytearray(b"correlation")
    wiped: list[bytes] = []
    original_wipe = async_operations._wipe

    def record_wipe(staging: bytearray) -> None:
        original_wipe(staging)
        wiped.append(bytes(staging))

    def values():
        yield source
        raise RuntimeError("filter source failed")

    monkeypatch.setattr(async_operations, "_wipe", record_wipe)
    with QueryAsyncRequestsRequest.create() as request:
        with pytest.raises(RuntimeError, match="filter source failed"):
            request.with_correlation_values(values())

    assert source == b"correlation"
    assert wiped == [bytes(len(source))]


def test_query_filter_wipes_staged_values_when_span_allocation_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = bytearray(b"correlation")
    wiped: list[bytes] = []
    original_wipe = async_operations._wipe

    def record_wipe(staging: bytearray) -> None:
        original_wipe(staging)
        wiped.append(bytes(staging))

    def fail_allocation(*_arguments: object) -> object:
        raise MemoryError("span allocation failed")

    monkeypatch.setattr(async_operations, "_wipe", record_wipe)
    with QueryAsyncRequestsRequest.create() as request:
        monkeypatch.setattr(async_operations, "ffi", SimpleNamespace(new=fail_allocation))
        with pytest.raises(MemoryError, match="span allocation failed"):
            request.with_correlation_values([source])

    assert source == b"correlation"
    assert wiped == [bytes(len(source))]


def test_request_wipes_staged_value_when_buffer_view_creation_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = bytearray(b"correlation")
    staged_values: list[bytearray] = []
    original_staging = async_operations._correlation_staging

    def capture_staging(value: bytes | bytearray | memoryview) -> bytearray:
        staging = original_staging(value)
        staged_values.append(staging)
        return staging

    def fail_buffer_view(*_arguments: object) -> object:
        raise MemoryError("buffer view allocation failed")

    monkeypatch.setattr(async_operations, "_correlation_staging", capture_staging)
    monkeypatch.setattr(
        async_operations,
        "ffi",
        SimpleNamespace(from_buffer=fail_buffer_view, NULL=None),
    )
    with pytest.raises(MemoryError, match="buffer view allocation failed"):
        CancelRequest.create(source)

    assert source == b"correlation"
    assert staged_values == [bytearray(len(source))]


def test_correlation_staging_rejects_invalid_released_and_over_limit_buffers(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        async_operations._correlation_staging(object())  # type: ignore[arg-type]

    released = memoryview(b"correlation")
    released.release()
    with pytest.raises(errors.InvalidInputError):
        async_operations._correlation_staging(released)

    monkeypatch.setattr(async_operations, "_MAX_INPUT_BYTES", 2)
    with pytest.raises(errors.ResourceLimitError):
        async_operations._correlation_staging(b"too long")


def test_correlation_snapshot_wipes_current_and_prior_buffers_on_size_limit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(async_operations, "_MAX_INPUT_BYTES", 2)
    wiped: list[bytes] = []
    original_wipe = async_operations._wipe

    def record_wipe(staging: bytearray) -> None:
        original_wipe(staging)
        wiped.append(bytes(staging))

    monkeypatch.setattr(async_operations, "_wipe", record_wipe)
    with pytest.raises(errors.ResourceLimitError):
        async_operations._correlation_values([b"a", b"bb"])

    assert wiped == [b"\x00\x00", b"\x00"]


def test_correlation_and_operation_filters_reject_non_iterables_and_invalid_values(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        async_operations._correlation_values(None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        async_operations._with_operations(None, None)  # type: ignore[arg-type]

    with QueryAsyncRequestsRequest.create() as request:
        with pytest.raises(errors.InvalidInputError):
            request.with_operations([True])
        with pytest.raises(errors.InvalidInputError):
            request.with_operations([-1])

        monkeypatch.setattr(async_operations, "_MAX_FILTER_COUNT", 0)
        with pytest.raises(errors.ResourceLimitError):
            request.with_operations([1])


def test_async_request_correlation_callback_validation_and_span_cleanup() -> None:
    for request_type in (CancelRequest, PollRequest):
        with request_type.create(b"correlation") as request:
            with pytest.raises(errors.InvalidInputError):
                request.with_asynchronous_correlation_value(None)  # type: ignore[arg-type]

    ffi = async_operations.ffi
    storage = ffi.new("uint8_t[]", b"secret")
    span = ffi.new("kmipkit_byte_span_v1_t *")
    span.data = ffi.cast("const uint8_t *", storage)
    span.length = 6
    borrowed: list[bytearray] = []

    def fail(view: memoryview) -> None:
        assert view.readonly
        borrowed.append(view.obj)
        raise RuntimeError("callback failed")

    with pytest.raises(RuntimeError, match="callback failed"):
        async_operations._call_with_span(span, fail)
    assert borrowed == [bytearray(6)]

    with pytest.raises(errors.InvalidInputError):
        async_operations._call_with_span(
            [SimpleNamespace(data=ffi.NULL, length=1)], lambda _view: None
        )


def test_async_request_callback_tolerates_callback_releasing_view() -> None:
    ffi = async_operations.ffi
    storage = ffi.new("uint8_t[]", b"secret")
    span = ffi.new("kmipkit_byte_span_v1_t *")
    span.data = ffi.cast("const uint8_t *", storage)
    span.length = 6
    borrowed: list[bytearray] = []

    def release(view: memoryview) -> None:
        borrowed.append(view.obj)
        view.release()

    assert async_operations._call_with_span(span, release) is None
    assert borrowed == [bytearray(6)]
