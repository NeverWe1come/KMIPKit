"""Request-option forwarding for the explicit asynchronous operations."""

from __future__ import annotations

from types import SimpleNamespace
from typing import Callable

import pytest

from kmipkit import client, errors, ttlv
from kmipkit._client_ffi import ffi


@pytest.mark.parametrize(
    ("method_name", "native_symbol", "invoke"),
    [
        (
            "execute_poll_with_options",
            "kmipkit_client_execute_poll_with_options_v1",
            lambda instance, limits, options: instance.execute_poll_with_options(
                b"\x00\xff\x80", limits, options
            ),
        ),
        (
            "execute_cancel_with_options",
            "kmipkit_client_execute_cancel_with_options_v1",
            lambda instance, limits, options: instance.execute_cancel_with_options(
                b"\x00\xff\x80", limits, options
            ),
        ),
        (
            "execute_process_with_options",
            "kmipkit_client_execute_process_with_options_v1",
            lambda instance, limits, options: instance.execute_process_with_options(
                b"\x00\xff\x80", limits, options, asynchronous_indicator=2
            ),
        ),
        (
            "execute_query_async_requests_with_options",
            "kmipkit_client_execute_query_async_requests_with_options_v1",
            lambda instance, limits, options: instance.execute_query_async_requests_with_options(
                limits,
                options,
                correlation_values=(b"\x00\xff\x80",),
                operations=(0xF0000001,),
                asynchronous_indicator=2,
            ),
        ),
    ],
)
def test_async_operation_options_call_the_matching_native_method_once(
    monkeypatch: pytest.MonkeyPatch,
    method_name: str,
    native_symbol: str,
    invoke: Callable[[client.Client, ttlv.CodecLimits, client.RequestOptions], object],
) -> None:
    calls: list[tuple[object, ...]] = []

    def native_method(*arguments: object) -> int:
        calls.append(arguments)
        return 4

    def last_error(status: object, category: object, cause: object, delivery: object) -> int:
        status[0] = 4  # type: ignore[index]
        category[0] = int(errors.ClientErrorCategory.VALIDATION)  # type: ignore[index]
        cause[0] = int(errors.ClientErrorCause.INVALID_INPUT)  # type: ignore[index]
        delivery[0] = int(errors.DeliveryState.NOT_SENT)  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(**{native_symbol: native_method, "kmipkit_last_error_get": last_error}),
    )
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)
    options = client.RequestOptions(
        client.TimeoutLimit.bounded(1, 2),
        client.TimeoutLimit.bounded(3, 4),
        client.TimeoutLimit.bounded(5, 6),
        client.TimeoutLimit.bounded(7, 8),
    )

    try:
        with pytest.raises(errors.ClientExecutionError):
            invoke(instance, limits, options)
        assert len(calls) == 1, method_name
        options_record = calls[0][-2]
        assert options_record.struct_size == ffi.sizeof(
            "kmipkit_request_options_v1_t"
        )
        assert options_record.abi_version == 1
        assert [
            (
                int(getattr(options_record, name).kind),
                int(getattr(options_record, name).seconds),
                int(getattr(options_record, name).nanoseconds),
            )
            for name in (
                "connect_timeout",
                "write_timeout",
                "read_timeout",
                "total_timeout",
            )
        ] == [(1, 1, 2), (1, 3, 4), (1, 5, 6), (1, 7, 8)]
    finally:
        limits.close()
