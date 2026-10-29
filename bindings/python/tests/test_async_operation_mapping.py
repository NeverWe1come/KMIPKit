"""Explicit asynchronous operation mapping through the Python client facade."""

from __future__ import annotations

from types import SimpleNamespace

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
        output.struct_size = ffi.sizeof(
            "kmipkit_client_async_operation_metadata_v1_t"
        )
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
        kmipkit_client_async_operation_outcome_release=lambda value: released.append(value),
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
            assert outcome.asynchronous_correlation_value == expected_response_correlation
            assert outcome.response_payload_ttlv is None
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [outcome_handle]
