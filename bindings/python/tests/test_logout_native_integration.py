"""Native-backed Python Logout flow over mutual TLS."""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

from _test_support import require_tls13_mtls_peer
from kmipkit import client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_TICKET_TYPE = 0x0102_0304
_TICKET_VALUE = b"logout-fixture-ticket"


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = (
        tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    )
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _logout_response_message() -> bytes:
    protocol_version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    response_header = _ttlv_structure(
        0x42007A,
        protocol_version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    logout_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x2E)),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        _ttlv_structure(0x42007C),
    )
    return _ttlv_structure(0x42007B, response_header, logout_item)


def _read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    output = bytearray()
    while len(output) < length:
        chunk = stream.recv(length - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a complete TTLV frame")
        output.extend(chunk)
    return bytes(output)


def _read_ttlv_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    return header + _read_exact(stream, int.from_bytes(header[4:8], "big"))


def _item(tag_value: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    raw_tag = ttlv.raw_tag_from_raw(tag_value)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    try:
        return ttlv.ttlv_item_create(tag, value, limits)
    finally:
        tag.close()
        raw_tag.close()


def _ticket_structure(limits: ttlv.CodecLimits) -> ttlv.Structure:
    ticket = ttlv.ttlv_structure_create()
    ticket = ttlv.ttlv_structure_with_item(
        ticket,
        _item(0x42014A, ttlv.ttlv_value_enumeration(_TICKET_TYPE), limits),
        limits,
    )
    ticket = ttlv.ttlv_structure_with_item(
        ticket,
        _item(0x42014B, ttlv.ttlv_value_byte_string(_TICKET_VALUE, limits), limits),
        limits,
    )
    return ticket


def test_logout_uses_native_cffi_and_exposes_success_response_presence() -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(3)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[Exception] = queue.Queue()
    received_requests: list[bytes] = []

    def serve_logout_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                require_tls13_mtls_peer(stream)
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_logout_response_message())
        except (AssertionError, OSError, ssl.SSLError) as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_logout_response, daemon=True)
    peer.start()
    transport = client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(
            client.TransportInput.file(
                _FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
            ),
        ),
        client_private_key=client.TransportInput.file(
            _FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities(
            (
                client.TransportInput.file(
                    _FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
                ),
            )
        ),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    ticket = None
    try:
        with (
            configuration,
            client.Client.create(configuration, transport) as kmip_client,
            ttlv.codec_limits_defaults() as limits,
        ):
            ticket = _ticket_structure(limits)
            batch = client.Batch.create()
            batch.add_item(extensions.client_batch_item_logout(ticket))
            ticket.close()
            ticket = None
            result = kmip_client.execute(batch, limits)

        assert peer_failures.empty(), "the loopback mTLS peer must complete"
        assert len(received_requests) == 1
        request = received_requests[0]
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x2E)) in request
        assert (
            _ttlv_structure(
                0x420149,
                _ttlv_item(0x42014A, 5, struct.pack(">I", _TICKET_TYPE)),
                _ttlv_item(0x42014B, 8, _TICKET_VALUE),
            )
            in request
        )
        assert result is not None and len(result.items) == 1
        logout_result = result.items[0]
        assert logout_result.operation == 0x2E
        assert logout_result.logout_response is True
    finally:
        if ticket is not None:
            ticket.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
