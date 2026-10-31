"""Explicit asynchronous operation mapping through the Python client facade."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, ttlv
from kmipkit._client_ffi import ffi


def test_poll_preserves_binary_correlation_and_exposes_pending_outcome(
    monkeypatch,
) -> None:
    expected_request_correlation = b"\x00\xff\x80\x13\x00"
    expected_response_correlation = b"\x00\x81\xfe\x00"
    native_correlation = ffi.new("uint8_t[]", expected_response_correlation)
    outcome_handle = ffi.cast("kmipkit_client_async_operation_outcome_t *", 42)
    released: list[object] = []
    observed_request: list[bytes] = []

    def execute_poll(_client, data, length, _limits, output) -> int:
        observed_request.append(bytes(ffi.buffer(data, int(length))))
        output[0] = outcome_handle
        return 0

    def metadata(_outcome, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
        output.abi_version = 1
        output.operation = 0x1A
        output.outcome_kind = int(client.BatchOutcome.PENDING)
        output.result_status = 2
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_cancellation_result = 0
        output.cancellation_result = 0
        output.has_result_message = 0
        output.reserved = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def correlation(_outcome, present, span) -> int:
        present[0] = 1
        span.data = native_correlation
        span.length = len(expected_response_correlation)
        return 0

    def response_payload(_outcome, present, output) -> int:
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    fake = SimpleNamespace(
        kmipkit_client_execute_poll=execute_poll,
        kmipkit_client_async_operation_outcome_release=lambda value: released.append(
            value
        ),
        kmipkit_client_async_operation_outcome_metadata_v1=metadata,
        kmipkit_client_async_operation_outcome_correlation_value=correlation,
        kmipkit_client_async_operation_outcome_response_payload_ttlv=response_payload,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    client_handle = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_poll(
            client_handle, expected_request_correlation, limits
        )
        try:
            assert isinstance(outcome, client.AsyncOperationOutcome)
            assert observed_request == [expected_request_correlation]
            assert outcome.metadata_v1.operation == 0x1A
            assert outcome.metadata_v1.outcome == client.BatchOutcome.PENDING
            assert outcome.metadata_v1.result_status == 2
            borrowed: list[memoryview] = []

            def inspect_correlation(value: memoryview) -> None:
                assert value.readonly
                assert bytes(value) == expected_response_correlation
                borrowed.append(value[:])

            assert outcome.with_asynchronous_correlation_value(inspect_correlation)
            assert bytes(borrowed[0]) == bytes(len(expected_response_correlation))
            assert outcome.response_payload_ttlv is None
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [outcome_handle]


def _outcome_stubs(
    operation: int,
    released: list[object],
    *,
    response_correlation: bytes | None = None,
    cancellation_result: int | None = None,
) -> SimpleNamespace:
    outcome_handle = ffi.cast("kmipkit_client_async_operation_outcome_t *", 42)
    native_correlation = (
        ffi.new("uint8_t[]", response_correlation)
        if response_correlation is not None
        else ffi.NULL
    )

    def metadata(_outcome, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
        output.abi_version = 1
        output.operation = operation
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)
        output.result_status = 0
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_cancellation_result = int(cancellation_result is not None)
        output.cancellation_result = cancellation_result or 0
        output.has_result_message = 0
        output.reserved = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def correlation(_outcome, present, span) -> int:
        present[0] = int(response_correlation is not None)
        span.data = native_correlation
        span.length = len(response_correlation or b"")
        return 0

    def response_payload(_outcome, present, output) -> int:
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    return SimpleNamespace(
        outcome_handle=outcome_handle,
        kmipkit_client_async_operation_outcome_release=lambda value: released.append(
            value
        ),
        kmipkit_client_async_operation_outcome_metadata_v1=metadata,
        kmipkit_client_async_operation_outcome_correlation_value=correlation,
        kmipkit_client_async_operation_outcome_response_payload_ttlv=response_payload,
    )


def test_cancel_preserves_the_exact_binary_correlation(monkeypatch) -> None:
    expected = b"\x00\xff\x80\x12\x00"
    echoed = b"\x80\x00\xfe"
    cancellation_result = 0xF000_0001
    observed: list[bytes] = []
    released: list[object] = []
    stubs = _outcome_stubs(
        0x05,
        released,
        response_correlation=echoed,
        cancellation_result=cancellation_result,
    )

    def execute_cancel(_client, data, length, _limits, output) -> int:
        observed.append(bytes(ffi.buffer(data, int(length))))
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_cancel": execute_cancel,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_cancel(instance, expected, limits)
        try:
            assert observed == [expected]
            assert outcome.metadata_v1.operation == 0x19
            assert outcome.metadata_v1.cancellation_result == cancellation_result
            observed_echo: list[bytes] = []
            assert outcome.with_asynchronous_correlation_value(
                lambda value: observed_echo.append(bytes(value))
            )
            assert observed_echo == [echoed]
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle]


def test_async_correlation_callback_wipes_borrowed_snapshot_when_callback_raises(
    monkeypatch,
) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    fake = SimpleNamespace(**stubs.__dict__)
    monkeypatch.setattr(client, "lib", fake)
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    retained: list[memoryview] = []

    def fail_after_read(value: memoryview) -> None:
        retained.append(value[:])
        assert bytes(value) == expected
        raise RuntimeError("callback failure")

    with pytest.raises(RuntimeError, match="callback failure"):
        outcome.with_asynchronous_correlation_value(fail_after_read)

    assert bytes(retained[0]) == bytes(len(expected))
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_async_correlation_buffer_is_wiped_if_readonly_view_creation_fails(
    monkeypatch,
) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    monkeypatch.setattr(client, "lib", SimpleNamespace(**stubs.__dict__))
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    buffers: list[bytearray] = []

    def tracked_bytearray(value=b"") -> bytearray:
        buffer = bytearray(value)
        buffers.append(buffer)
        return buffer

    def fail_view(_buffer):
        raise MemoryError("readonly view allocation failed")

    monkeypatch.setattr(client, "bytearray", tracked_bytearray, raising=False)
    monkeypatch.setattr(client, "memoryview", fail_view, raising=False)

    with pytest.raises(MemoryError, match="readonly view allocation failed"):
        outcome.with_asynchronous_correlation_value(lambda _value: None)

    assert buffers == [bytearray(len(expected))]
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_async_correlation_wipe_does_not_allocate_a_zero_fill_copy(monkeypatch) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    monkeypatch.setattr(client, "lib", SimpleNamespace(**stubs.__dict__))
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    retained: list[memoryview] = []
    builtin_bytes = bytes

    def reject_sized_bytes(value=b""):
        if isinstance(value, int):
            raise MemoryError("zero-fill allocation attempted")
        return builtin_bytes(value)

    monkeypatch.setattr(client, "bytes", reject_sized_bytes, raising=False)
    assert outcome.with_asynchronous_correlation_value(
        lambda value: retained.append(value[:])
    )

    assert builtin_bytes(retained[0]) == builtin_bytes(len(expected))
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_process_forwards_an_explicit_indicator_once(monkeypatch) -> None:
    expected = b"\x80\x00\xfe"
    indicator = 1
    observed: list[tuple[bytes, int, int]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x28, released)

    def execute_process(
        _client, data, length, has_indicator, indicator_value, _limits, output
    ) -> int:
        observed.append(
            (
                bytes(ffi.buffer(data, int(length))),
                int(has_indicator),
                int(indicator_value),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_process": execute_process,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_process(
            instance, expected, limits, asynchronous_indicator=indicator
        )
        try:
            assert observed == [(expected, 1, indicator)]
            assert outcome.metadata_v1.operation == 0x28
            assert not outcome.with_asynchronous_correlation_value(
                lambda _value: pytest.fail("unexpected correlation callback")
            )
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle]


def test_query_async_requests_preserves_present_empty_and_binary_filters(
    monkeypatch,
) -> None:
    expected_correlations = (b"\x00\xff\x10", b"\x80\x00\xfe")
    expected_operations = (0x0000_0005, 0xF000_0001, 0x0000_0005)
    observed: list[tuple[tuple[bytes, ...], tuple[int, ...], tuple[int, ...]]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x39, released)

    def execute_query(
        _client,
        has_correlations,
        spans,
        correlation_count,
        has_operations,
        operations,
        operation_count,
        has_indicator,
        indicator,
        _limits,
        output,
    ) -> int:
        correlation_values = tuple(
            bytes(ffi.buffer(spans[index].data, int(spans[index].length)))
            for index in range(int(correlation_count))
        )
        operation_values = tuple(
            int(operations[index]) for index in range(int(operation_count))
        )
        observed.append(
            (
                correlation_values,
                operation_values,
                (
                    int(has_correlations),
                    int(has_operations),
                    int(has_indicator),
                    int(indicator),
                ),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_query_async_requests": execute_query,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_query_async_requests(
            instance,
            limits,
            correlation_values=expected_correlations,
            operations=expected_operations,
        )
        try:
            assert observed == [
                (expected_correlations, expected_operations, (1, 1, 0, 0))
            ]
            assert outcome.metadata_v1.operation == 0x39
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle]


def test_query_async_requests_distinguishes_absent_from_present_empty_filters(
    monkeypatch,
) -> None:
    observed: list[tuple[int, int, int, int]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x39, released)

    def execute_query(
        _client,
        has_correlations,
        _spans,
        _correlation_count,
        has_operations,
        _operations,
        _operation_count,
        has_indicator,
        indicator,
        _limits,
        output,
    ) -> int:
        observed.append(
            (
                int(has_correlations),
                int(has_operations),
                int(has_indicator),
                int(indicator),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_query_async_requests": execute_query,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        absent = client.Client.execute_query_async_requests(instance, limits)
        empty = client.Client.execute_query_async_requests(
            instance, limits, correlation_values=(), operations=()
        )
        try:
            assert observed == [(0, 0, 0, 0), (1, 1, 0, 0)]
        finally:
            absent.close()
            empty.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle, stubs.outcome_handle]
