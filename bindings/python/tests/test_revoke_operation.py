"""Revoke binding checks derived from KMIP 2.1 §§4.47, 6.1.44, and 11.48."""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_REVOKE_ID = b"revoke-item"
_COMPROMISE_DATE = 1_700_000_000


def _ttlv_item(tag: int, kind: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((kind,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _ttlv_enumeration(tag: int, value: int) -> bytes:
    return _ttlv_item(tag, 5, struct.pack(">I", value))


def _response_message() -> bytes:
    version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _ttlv_structure(
        0x42007A,
        version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    payload = _ttlv_structure(
        0x42007C,
        _ttlv_item(0x420094, 7, b"revoked-server-id"),
        _ttlv_item(0x541234, 8, b"\x01\x00\xff"),
    )
    item = _ttlv_structure(
        0x42000F,
        _ttlv_enumeration(0x42005C, 0x13),
        _ttlv_item(0x420093, 8, _REVOKE_ID),
        _ttlv_enumeration(0x42007F, 0),
        payload,
    )
    return _ttlv_structure(0x42007B, header, item)


def _read_exact(stream: ssl.SSLSocket, size: int) -> bytes:
    output = bytearray()
    while len(output) < size:
        chunk = stream.recv(size - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a complete request")
        output.extend(chunk)
    return bytes(output)


def _start_revoke_mtls_peer() -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    received: list[bytes] = []
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _ = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                assert stream.version() == "TLSv1.3"
                assert stream.getpeercert(binary_form=True)
                prefix = _read_exact(stream, 8)
                message_length = int.from_bytes(prefix[4:8], "big")
                received.append(prefix + _read_exact(stream, message_length))
                stream.sendall(_response_message())
        except Exception as error:  # noqa: BLE001 - surface peer errors from the fixture thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return listener.getsockname()[1], received, failures, peer


def _transport(port: int) -> client.TransportConfig:
    return client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(client.TransportInput.file(
            _FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
        ),),
        client_private_key=client.TransportInput.file(
            _FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities((client.TransportInput.file(
            _FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
        ),)),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )


def test_revoke_request_retains_unsigned_unknown_reason_and_nested_message() -> None:
    reason = extensions.RevocationReason.create(0xFFFF_FFFF, "operator audit")
    assert reason.raw_code == 0xFFFF_FFFF
    assert reason.message == "operator audit"
    request = extensions.RevokeRequest.create(None, reason, None)
    item = extensions.client_batch_item_revoke(request)
    item.close()


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
            request = extensions.RevokeRequest.create(None, reason, _COMPROMISE_DATE)
            item = extensions.client_batch_item_revoke(request)
            batch = client.Batch.create()
            batch.add_item(extensions.with_unique_batch_item_id(item, _REVOKE_ID))
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete"
        assert len(received) == 1
        assert _ttlv_structure(
            0x420081,
            _ttlv_enumeration(0x420082, 2),
            _ttlv_item(0x420080, 7, b"operator audit"),
        ) in received[0]
        assert _ttlv_item(
            0x420021, 10, struct.pack(">q", _COMPROMISE_DATE)
        ) in received[0]
        assert _ttlv_item(0x420094, 7, b"object") not in received[0]
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x13 and item_result.result_status == 0
        response = item_result.revoke_response
        assert response is not None
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "revoked-server-id"
        )
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 2
        result.close()
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 2
    finally:
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
