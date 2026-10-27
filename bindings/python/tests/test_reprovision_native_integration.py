"""Derived KMIP 2.1 §6.1.48 Tables 313–314 coverage over mutual TLS."""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from typing import NamedTuple

from _test_support import require_tls13_mtls_peer
from kmipkit import client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_BATCH_IDS = (b"reprovision-0", b"reprovision-1")


class _TtlvSpan(NamedTuple):
    tag: int
    item_type: int
    value_start: int
    value_length: int
    end: int


def _item(tag: int, kind: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((kind,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _enum(tag: int, value: int) -> bytes:
    return _item(tag, 5, struct.pack(">I", value))


def _response_without_identifiers() -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 2)),
    )
    items = tuple(
        _structure(
            0x42000F,
            _enum(0x42005C, 0x35),
            _item(0x420093, 8, batch_id),
            _enum(0x42007F, 0),
            _structure(0x42007C),
        )
        for batch_id in _BATCH_IDS
    )
    return _structure(0x42007B, header, *items)


def _read_exact(stream: ssl.SSLSocket, size: int) -> bytes:
    output = bytearray()
    while len(output) < size:
        chunk = stream.recv(size - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a complete request")
        output.extend(chunk)
    return bytes(output)


def _read_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    return header + _read_exact(stream, int.from_bytes(header[4:8], "big"))


def _start_mtls_peer(
    response: bytes,
) -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    received: list[bytes] = []
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _ = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                require_tls13_mtls_peer(stream)
                received.append(_read_frame(stream))
                stream.sendall(response)
        except Exception as error:  # noqa: BLE001 - surface peer errors in the test thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, received, failures, peer


def _transport(port: int) -> client.TransportConfig:
    return client.TransportConfig(
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


def _parse_item(encoded: bytes, offset: int, parent_end: int) -> _TtlvSpan:
    assert 0 <= offset <= parent_end - 8
    tag = int.from_bytes(encoded[offset : offset + 3], "big")
    item_type = encoded[offset + 3]
    value_length = int.from_bytes(encoded[offset + 4 : offset + 8], "big")
    value_start = offset + 8
    item_end = value_start + value_length + (-value_length % 8)
    assert item_end <= parent_end
    return _TtlvSpan(tag, item_type, value_start, value_length, item_end)


def _children(encoded: bytes, parent: _TtlvSpan) -> list[_TtlvSpan]:
    assert parent.item_type == 1
    value_end = parent.value_start + parent.value_length
    children: list[_TtlvSpan] = []
    offset = parent.value_start
    while offset < value_end:
        child = _parse_item(encoded, offset, value_end)
        children.append(child)
        offset = child.end
    assert offset == value_end
    return children


def _request_payloads(encoded: bytes) -> list[list[_TtlvSpan]]:
    message = _parse_item(encoded, 0, len(encoded))
    assert message.tag == 0x420078 and message.end == len(encoded)
    payloads = []
    for batch_item in _children(encoded, message):
        if batch_item.tag != 0x42000F:
            continue
        operation = next(
            field
            for field in _children(encoded, batch_item)
            if field.tag == 0x42005C
        )
        assert int.from_bytes(
            encoded[operation.value_start : operation.value_start + operation.value_length],
            "big",
        ) == 0x35
        request_payload = next(
            field
            for field in _children(encoded, batch_item)
            if field.tag == 0x420079
        )
        payloads.append(_children(encoded, request_payload))
    return payloads


def _marker_structure(limits: ttlv.CodecLimits) -> ttlv.Structure:
    with (
        ttlv.raw_tag_from_raw(0x541101) as raw_tag,
        ttlv.raw_tag_try_checked(raw_tag) as tag,
        ttlv.ttlv_value_byte_string(b"certificate-request-fixture", limits) as value,
        ttlv.ttlv_item_create(tag, value, limits) as item,
    ):
        structure = ttlv.ttlv_structure_create()
        try:
            return ttlv.ttlv_structure_with_item(structure, item, limits)
        finally:
            structure.close()


def test_reprovision_sends_parameterless_and_certificate_request_items() -> None:
    port, received, failures, peer = _start_mtls_peer(_response_without_identifiers())
    config = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    reprovision_response = None
    try:
        with (
            config,
            client.Client.create(config, _transport(port)) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    extensions.client_batch_item_reprovision(), _BATCH_IDS[0]
                )
            )
            with _marker_structure(limits) as certificate_request:
                batch.add_item(
                    extensions.with_unique_batch_item_id(
                        extensions.client_batch_item_reprovision(
                            certificate_request=certificate_request
                        ),
                        _BATCH_IDS[1],
                    )
                )
            result = kmip.execute(batch, limits)

        assert len(received) == 1
        payloads = _request_payloads(received[0])
        assert len(payloads) == 2
        assert [tuple(field.tag for field in payload) for payload in payloads] == [
            (),
            (0x420018,),
        ]
        certificate_request = payloads[1][0]
        assert tuple(
            field.tag for field in _children(received[0], certificate_request)
        ) == (0x541101,)
        assert failures.empty() and result is not None
        assert all(item.operation == 0x35 and item.result_status == 0 for item in result.items)
        reprovision_response = result.items[0].reprovision_response
        assert reprovision_response is not None
        assert reprovision_response.unique_identifier is None
        assert not reprovision_response.response_ttlv.closed
        assert "certificate-request-fixture" not in repr(reprovision_response)
        with ttlv.ttlv_structure_view(reprovision_response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
        result.close()
    finally:
        if reprovision_response is not None:
            reprovision_response.close()
        if result is not None:
            result.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("mTLS peer did not finish")
        if not failures.empty():
            raise failures.get()
