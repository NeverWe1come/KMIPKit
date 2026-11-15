"""Derived Python binding checks for OASIS KMIP 2.1 §6.1.43, Tables 291–294."""

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
    ttlv_item,
    unique_identifier_ttlv_response_test_fixture,
)
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_REGISTER_ID = b"register-0"


def _register_response_message() -> bytes:
    return operation_response_message(
        0x03,
        _REGISTER_ID,
        payload_items=(
            ttlv_item(0x420094, 7, b"registered-server-id"),
            ttlv_item(0x541234, 8, b"\x01\x00\xff"),
        ),
    )


def _start_register_mtls_peer() -> tuple[
    int, list[bytes], queue.Queue[Exception], threading.Thread
]:
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_register_response_message())

    port, failures, peer = start_tls13_mtls_peer(_FIXTURES, handle)
    return port, received, failures, peer


def _register_transport(port: int) -> client.TransportConfig:
    return mtls_transport_config(port, _FIXTURES)


def _register_opaque_object(limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    fields = ttlv.ttlv_structure_create()
    fields = ttlv.ttlv_structure_with_item(
        fields,
        _typed_ttlv_item(0x420059, ttlv.ttlv_value_enumeration(0x80000001), limits),
        limits,
    )
    fields = ttlv.ttlv_structure_with_item(
        fields,
        _typed_ttlv_item(
            0x42005A,
            ttlv.ttlv_value_byte_string(b"register-object-fixture", limits),
            limits,
        ),
        limits,
    )
    return _typed_ttlv_item(0x42005B, ttlv.ttlv_value_structure(fields, limits), limits)


def _typed_ttlv_item(
    tag_number: int, value: ttlv.Value, limits: ttlv.CodecLimits
) -> ttlv.TtlvItem:
    with (
        ttlv.raw_tag_from_raw(tag_number) as raw_tag,
        ttlv.raw_tag_try_checked(raw_tag) as tag,
    ):
        return ttlv.ttlv_item_create(tag, value, limits)


def _response_item() -> SimpleNamespace:
    return SimpleNamespace(_pointer=lambda: object())


def test_register_success_response_preserves_identifier_and_owned_ttlv(
    monkeypatch,
) -> None:
    fixture = unique_identifier_ttlv_response_test_fixture(
        "register", b"registered-object", 42, lambda _value: None
    )
    owned = fixture.owned_ttlv
    monkeypatch.setattr(client, "lib", SimpleNamespace(**fixture.response_symbols))
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == fixture.native_ttlv_address else None,
    )

    response = client._register_response(_response_item())
    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
        "registered-object"
    )
    assert response.response_ttlv is owned
    assert "registered-object" not in repr(response)
    response.close()
    assert owned.closed


def test_register_failure_or_pending_has_no_success_response(monkeypatch) -> None:
    def absent(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output.kind = 0
        output.text_data = ffi.NULL
        output.text_length = 0
        output.enumeration_value = 0
        output.integer_value = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_register=absent),
    )
    assert client._register_response(_response_item()) is None


def test_register_success_requires_a_required_identifier(monkeypatch) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 43)
    released: list[object] = []

    def missing_identifier(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = 0
        output.text_data = ffi.NULL
        output.text_length = 0
        output.enumeration_value = 0
        output.integer_value = 0
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=missing_identifier,
            kmipkit_client_batch_item_response_register_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._register_response(_response_item())
    assert released == []


def test_register_rejects_invalid_response_presence_flag(monkeypatch) -> None:
    def invalid_presence(_item: object, present: object, _output: object) -> int:
        present[0] = 2
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_register=invalid_presence),
    )

    with pytest.raises(errors.InvalidInputError):
        client._register_response(_response_item())


def test_register_rejects_identifier_on_absent_response(monkeypatch) -> None:
    def inconsistent_absence(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = ffi.NULL
        output.text_length = 0
        output.enumeration_value = 0
        output.integer_value = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=inconsistent_absence,
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._register_response(_response_item())


def test_register_releases_unflagged_native_ttlv_handle(monkeypatch) -> None:
    identifier = b"register-object"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 44)
    released: list[object] = []

    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_buffer
        output.text_length = len(identifier)
        return 0

    def unflagged_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=typed,
            kmipkit_client_batch_item_response_register_ttlv=unflagged_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._register_response(_response_item())

    assert released == [native_ttlv]


def test_register_releases_native_ttlv_when_wrapper_creation_fails(monkeypatch) -> None:
    identifier = b"register-object"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 45)
    released: list[object] = []

    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_buffer
        output.text_length = len(identifier)
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    def reject_wrapper(_address: int) -> object:
        raise RuntimeError("invalid native handle")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=typed,
            kmipkit_client_batch_item_response_register_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(ttlv, "_structure_from_native_address", reject_wrapper)

    with pytest.raises(RuntimeError, match="invalid native handle"):
        client._register_response(_response_item())

    assert released == [native_ttlv]


def test_register_native_mtls_response_is_owned_and_preserves_ordered_ttlv() -> None:
    port, received, failures, peer = _start_register_mtls_peer()
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    response = None
    try:
        with (
            configuration,
            client.Client.create(configuration, _register_transport(port)) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            request = extensions.RegisterRequest.create(
                8, ttlv.ttlv_structure_create(), _register_opaque_object(limits)
            )
            batch = client.Batch.create()
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    extensions.client_batch_item_register(request), _REGISTER_ID
                )
            )
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x03) in received[0]
        assert result is not None and len(result.items) == 1
        item = result.items[0]
        assert item.operation == 0x03 and item.result_status == 0
        response = item.register_response
        assert response is not None
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "registered-server-id"
        )
        assert "registered-server-id" not in repr(response)
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
        result.close()
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
    finally:
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
