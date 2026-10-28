from __future__ import annotations

import queue
import ssl
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest
from _test_support import (
    mtls_transport_config,
    operation_response_message,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_enumeration,
    ttlv_structure,
)

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_SET_DEFAULTS_ID = b"set-defaults-0"


def _set_defaults_response_message(
    *,
    result_status: int = 0,
    result_reason: int | None = None,
    result_message: str | None = None,
    pending_correlation_value: bytes | None = None,
) -> bytes:
    return operation_response_message(
        0x36,
        _SET_DEFAULTS_ID,
        result_status=result_status,
        result_reason=result_reason,
        result_message=result_message,
        pending_correlation_value=pending_correlation_value,
    )


def _start_set_defaults_mtls_peer(
    response_message: bytes | None = None,
) -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(
            _set_defaults_response_message()
            if response_message is None
            else response_message
        )

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    return port, received, failures, peer


def _set_defaults_transport(port: int) -> client.TransportConfig:
    return mtls_transport_config(port, _TLS_FIXTURES)


def test_set_defaults_builder_preserves_omitted_and_present_structure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure_handle = object()
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_client_native_structure",
        lambda value, *, required: ffi.NULL if value is None else structure_handle,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 41,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_set_defaults() == 41
    assert extensions.client_batch_item_set_defaults(object()) == 41
    assert calls == [
        ("kmipkit_client_batch_item_set_defaults", (ffi.NULL,)),
        ("kmipkit_client_batch_item_set_defaults", (structure_handle,)),
    ]


def test_set_defaults_response_owns_generic_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_defaults_ttlv=response_ttlv
        ),
    )
    monkeypatch.setattr(
        ttlv, "_structure_from_native_address", lambda _address: structure
    )

    result = client._set_defaults_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert isinstance(result, client.SetDefaultsResponse)
    assert result.response_ttlv is structure
    assert "123" not in repr(result)
    result.close()
    assert structure.closed


def test_set_defaults_response_releases_unexpected_native_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_defaults_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._set_defaults_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [native_handle]


def test_set_defaults_response_releases_native_handle_when_wrapping_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []
    conversion_error = RuntimeError("conversion failed")

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    def fail_conversion(_address: int) -> object:
        raise conversion_error

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_defaults_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(ttlv, "_structure_from_native_address", fail_conversion)

    with pytest.raises(RuntimeError) as captured:
        client._set_defaults_response(SimpleNamespace(_pointer=lambda: object()))

    assert captured.value is conversion_error
    assert released == [native_handle]


def test_set_defaults_native_mtls_response_is_owned_and_preserves_empty_success_payload() -> (
    None
):
    port, received, failures, peer = _start_set_defaults_mtls_peer()
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    response = None
    defaults_information = ttlv.ttlv_structure_create()
    try:
        with (
            configuration,
            client.Client.create(
                configuration, _set_defaults_transport(port)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            item = extensions.client_batch_item_set_defaults(defaults_information)
            defaults_information.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(item, _SET_DEFAULTS_ID)
            )
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete its response"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x36) in received[0]
        assert ttlv_structure(0x420152) in received[0]
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x36 and item_result.result_status == 0
        response = item_result.set_defaults_response
        assert isinstance(response, client.SetDefaultsResponse)
        assert "response_ttlv" not in repr(response)
        with (
            ttlv.ttlv_structure_view(response.response_ttlv) as view,
            ttlv.ttlv_structure_view_item_at(view, 3) as payload,
            ttlv.ttlv_item_view_value(payload) as payload_value,
            ttlv.ttlv_value_view_structure(payload_value) as payload_view,
        ):
            assert ttlv.ttlv_structure_view_item_count(view) == 4
            assert ttlv.ttlv_structure_view_item_count(payload_view) == 0
        result.close()
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
    finally:
        defaults_information.close()
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)


@pytest.mark.parametrize(
    ("status", "reason", "message", "correlation", "outcome"),
    (
        pytest.param(
            1, 7, "defaults denied", None, client.BatchOutcome.COMPLETED, id="failure"
        ),
        pytest.param(
            2,
            None,
            None,
            b"set-defaults-pending-correlation",
            client.BatchOutcome.PENDING,
            id="pending",
        ),
    ),
)
def test_set_defaults_native_mtls_failure_and_pending_keep_common_metadata_only(
    status: int,
    reason: int | None,
    message: str | None,
    correlation: bytes | None,
    outcome: client.BatchOutcome,
) -> None:
    response_message = _set_defaults_response_message(
        result_status=status,
        result_reason=reason,
        result_message=message,
        pending_correlation_value=correlation,
    )
    port, received, failures, peer = _start_set_defaults_mtls_peer(response_message)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    defaults_information = ttlv.ttlv_structure_create()
    try:
        with (
            configuration,
            client.Client.create(
                configuration, _set_defaults_transport(port)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            if status == 2:
                batch.set_asynchronous_indicator(1)
            item = extensions.client_batch_item_set_defaults(defaults_information)
            defaults_information.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(item, _SET_DEFAULTS_ID)
            )
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete its response"
        assert len(received) == 1
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x36
        assert item_result.outcome == outcome
        assert item_result.result_status == status
        assert item_result.result_reason == reason
        assert item_result.result_message == message
        assert item_result.pending_correlation_value == correlation
        assert item_result.set_defaults_response is None
    finally:
        defaults_information.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
