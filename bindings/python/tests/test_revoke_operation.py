"""Revoke binding checks derived from KMIP 2.1 §§4.47, 6.1.44, and 11.48."""

from __future__ import annotations

import queue
import ssl
import struct
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
    ttlv_structure,
)

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_REVOKE_ID = b"revoke-item"
_REVOKE_TARGET = b"revoke-target"
_COMPROMISE_DATE = 1_700_000_000


def _response_message() -> bytes:
    return operation_response_message(
        0x13,
        _REVOKE_ID,
        payload_items=(
            ttlv_item(0x420094, 7, b"revoked-server-id"),
            ttlv_item(0x541234, 8, b"\x01\x00\xff"),
        ),
    )


def _start_revoke_mtls_peer() -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_response_message())

    port, failures, peer = start_tls13_mtls_peer(_FIXTURES, handle)
    return port, received, failures, peer


def _transport(port: int) -> client.TransportConfig:
    return mtls_transport_config(port, _FIXTURES)


def _view_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    tags = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with (
            ttlv.ttlv_structure_view_item_at(view, index) as item,
            ttlv.ttlv_item_view_tag(item) as tag,
        ):
            tags.append(ttlv.tag_value(tag))
    return tuple(tags)


def _byte_string_at(
    view: ttlv.TtlvStructureView, index: int, expected_tag: int
) -> bytes:
    with ttlv.ttlv_structure_view_item_at(view, index) as item:
        with ttlv.ttlv_item_view_tag(item) as tag:
            assert ttlv.tag_value(tag) == expected_tag
        assert ttlv.ttlv_item_view_type(item) == ttlv.ItemType.ByteString
        with ttlv.ttlv_item_view_value(item) as value:
            length = ttlv.ttlv_value_view_byte_length(value)
            return bytes(
                ttlv.ttlv_value_view_byte_at(value, byte) for byte in range(length)
            )


def test_revoke_request_retains_unsigned_unknown_reason_and_nested_message() -> None:
    reason = extensions.RevocationReason.create(0xFFFF_FFFF, "operator audit")
    assert reason.raw_code == 0xFFFF_FFFF
    assert reason.message == "operator audit"
    assert repr(reason) == "RevocationReason(raw_code=4294967295, message=redacted)"
    request = extensions.RevokeRequest.create(None, reason, None)
    item = extensions.client_batch_item_revoke(request)
    item.close()


def test_revocation_reason_rejects_non_string_message_and_wrong_reason_type() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.RevocationReason.create(1, object())  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        extensions.RevokeRequest.create(None, object(), None)  # type: ignore[arg-type]


def test_revoke_rejects_compromise_date_for_other_reason_before_send() -> None:
    reason = extensions.RevocationReason.create(4, None)
    request = extensions.RevokeRequest.create(None, reason, 1)
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_revoke(request)


def test_revoke_success_response_preserves_identifier_and_owned_ttlv(monkeypatch) -> None:
    identifier = b"revoked-object"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 42)

    class OwnedStructure:
        def __init__(self) -> None:
            self.closed = False

        def close(self) -> None:
            self.closed = True

    owned = OwnedStructure()

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

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_revoke=typed,
            kmipkit_client_batch_item_response_revoke_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == 42 else None,
    )

    response = client._revoke_response(SimpleNamespace(_pointer=lambda: object()))
    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string("revoked-object")
    assert response.response_ttlv is owned
    assert "revoked-object" not in repr(response)
    response.close()
    assert owned.closed


def test_revoke_absent_response_requires_absent_identifier(monkeypatch) -> None:
    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output.kind = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_revoke=typed),
    )
    assert client._revoke_response(SimpleNamespace(_pointer=lambda: object())) is None

    def inconsistent(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_revoke=inconsistent),
    )
    with pytest.raises(errors.InvalidInputError):
        client._revoke_response(SimpleNamespace(_pointer=lambda: object()))


def test_revoke_response_rejects_invalid_presence_and_missing_identifier(monkeypatch) -> None:
    def invalid_presence(_item: object, present: object, _output: object) -> int:
        present[0] = 2
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_revoke=invalid_presence),
    )
    with pytest.raises(errors.InvalidInputError):
        client._revoke_response(SimpleNamespace(_pointer=lambda: object()))

    def missing_identifier(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_revoke=missing_identifier),
    )
    with pytest.raises(errors.InvalidInputError):
        client._revoke_response(SimpleNamespace(_pointer=lambda: object()))


def test_revoke_response_releases_malformed_ttlv_handle(monkeypatch) -> None:
    identifier = b"revoke-response"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 43)
    released: list[object] = []

    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_buffer
        output.text_length = len(identifier)
        return 0

    def mismatched_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_revoke=typed,
            kmipkit_client_batch_item_response_revoke_ttlv=mismatched_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._revoke_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == [native_ttlv]


def test_revoke_response_releases_handle_when_ttlv_wrapper_fails(monkeypatch) -> None:
    identifier = b"revoke-response"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 44)
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

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_revoke=typed,
            kmipkit_client_batch_item_response_revoke_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("invalid native handle")),
    )

    with pytest.raises(RuntimeError, match="invalid native handle"):
        client._revoke_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == [native_ttlv]


def test_revoke_native_mtls_preserves_nested_request_and_success_response() -> None:
    port, received, failures, peer = _start_revoke_mtls_peer()
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
            client.Client.create(configuration, _transport(port)) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            reason = extensions.RevocationReason.create(2, "operator audit")
            request = extensions.RevokeRequest.create(
                extensions.UniqueIdentifier.text_string(_REVOKE_TARGET.decode()),
                reason,
                _COMPROMISE_DATE,
            )
            item = extensions.client_batch_item_revoke(request)
            batch = client.Batch.create()
            batch.add_item(extensions.with_unique_batch_item_id(item, _REVOKE_ID))
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete"
        assert len(received) == 1
        assert ttlv_structure(
            0x420081,
            ttlv_enumeration(0x420082, 2),
            ttlv_item(0x420080, 7, b"operator audit"),
        ) in received[0]
        assert ttlv_item(
            0x420021, 9, struct.pack(">q", _COMPROMISE_DATE)
        ) in received[0]
        assert ttlv_item(0x420094, 7, _REVOKE_TARGET) in received[0]
        assert ttlv_item(0x420094, 7, b"object") not in received[0]
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x13 and item_result.result_status == 0
        response = item_result.revoke_response
        assert response is not None
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "revoked-server-id"
        )
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert _view_tags(view) == (0x42005C, 0x420093, 0x42007F, 0x42007C)
            with (
                ttlv.ttlv_structure_view_item_at(view, 3) as payload_item,
                ttlv.ttlv_item_view_value(payload_item) as payload_value,
                ttlv.ttlv_value_view_structure(payload_value) as payload,
            ):
                assert _view_tags(payload) == (0x420094, 0x541234)
                assert _byte_string_at(payload, 1, 0x541234) == b"\x01\x00\xff"
        result.close()
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert _view_tags(view) == (0x42005C, 0x420093, 0x42007F, 0x42007C)
            with (
                ttlv.ttlv_structure_view_item_at(view, 3) as payload_item,
                ttlv.ttlv_item_view_value(payload_item) as payload_value,
                ttlv.ttlv_value_view_structure(payload_value) as payload,
            ):
                assert _view_tags(payload) == (0x420094, 0x541234)
                assert _byte_string_at(payload, 1, 0x541234) == b"\x01\x00\xff"
    finally:
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
