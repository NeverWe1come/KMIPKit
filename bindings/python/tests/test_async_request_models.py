"""KMIP 2.1 §6.1.5, §6.1.38, §6.1.39, and §6.1.41 request model parity."""

from __future__ import annotations

from kmipkit import ttlv
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
