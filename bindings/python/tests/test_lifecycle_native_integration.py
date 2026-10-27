"""Native CFFI lifecycle request and non-success response checks over mTLS."""

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
_DESTROY_CORRELATION = b"\x00destroy-pending\xff"
_RECOVER_CORRELATION = b"\x80recover\x00\xff"
_BATCH_IDS = (b"life-activate", b"life-archive", b"life-destroy", b"life-recover")


class _Span(NamedTuple):
    tag: int
    item_type: int
    value_start: int
    value_length: int
    end: int


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((item_type,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _integer(tag: int, value: int) -> bytes:
    return _item(tag, 2, struct.pack(">i", value))


def _long(tag: int, value: int) -> bytes:
    return _item(tag, 9, struct.pack(">q", value))


def _enumeration(tag: int, value: int) -> bytes:
    return _item(tag, 5, struct.pack(">I", value))


def _response_item(
    operation: int,
    batch_id: bytes,
    status: int,
    *,
    reason: int | None = None,
    message: bytes | None = None,
    correlation: bytes | None = None,
) -> bytes:
    children = [
        _enumeration(0x42005C, operation),
        _item(0x420093, 8, batch_id),
        _enumeration(0x42007F, status),
    ]
    if reason is not None:
        children.append(_enumeration(0x42007E, reason))
    if message is not None:
        children.append(_item(0x42007D, 7, message))
    if correlation is not None:
        children.extend((_item(0x420006, 8, correlation), _structure(0x42007C)))
    return _structure(0x42000F, *children)


def _failure_pending_response() -> bytes:
    version = _structure(
        0x420069,
        _integer(0x42006A, 2),
        _integer(0x42006B, 1),
    )
    header = _structure(
        0x42007A,
        version,
        _long(0x420092, 1),
        _integer(0x42000D, 4),
    )
    return _structure(
        0x42007B,
        header,
        _response_item(0x12, _BATCH_IDS[0], 1, reason=5, message=b"activate rejected"),
        _response_item(0x15, _BATCH_IDS[1], 1, reason=5, message=b"archive rejected"),
        _response_item(
            0x14, _BATCH_IDS[2], 2, correlation=_DESTROY_CORRELATION
        ),
        _response_item(
            0x16, _BATCH_IDS[3], 2, correlation=_RECOVER_CORRELATION
        ),
    )


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


def _start_peer(
    response: bytes,
) -> tuple[int, list[bytes], queue.Queue[BaseException], threading.Thread]:
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(2)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    requests: list[bytes] = []
    failures: queue.Queue[BaseException] = queue.Queue()

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
                requests.append(_read_frame(stream))
                stream.sendall(response)
                stream.settimeout(0.25)
                try:
                    unexpected = stream.recv(1)
                except TimeoutError:
                    unexpected = b""
                if unexpected:
                    requests.append(b"unexpected follow-up on the original connection")
            listener.settimeout(0.25)
            try:
                extra, _ = listener.accept()
            except TimeoutError:
                pass
            else:
                with extra:
                    requests.append(b"unexpected follow-up connection")
        except BaseException as error:  # noqa: BLE001 - report peer failures in the test thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, requests, failures, peer


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


def _parse_item(encoded: bytes, offset: int, parent_end: int) -> _Span:
    assert 0 <= offset <= parent_end - 8
    tag = int.from_bytes(encoded[offset : offset + 3], "big")
    item_type = encoded[offset + 3]
    value_length = int.from_bytes(encoded[offset + 4 : offset + 8], "big")
    value_start = offset + 8
    end = value_start + value_length + (-value_length % 8)
    assert end <= parent_end
    return _Span(tag, item_type, value_start, value_length, end)


def _children(encoded: bytes, parent: _Span) -> list[_Span]:
    assert parent.item_type == 1
    end = parent.value_start + parent.value_length
    children: list[_Span] = []
    offset = parent.value_start
    while offset < end:
        child = _parse_item(encoded, offset, end)
        children.append(child)
        offset = child.end
    assert offset == end
    return children


def _lifecycle_request_identifiers(encoded: bytes) -> dict[int, tuple[bytes, int, bytes]]:
    message = _parse_item(encoded, 0, len(encoded))
    assert message.tag == 0x420078 and message.end == len(encoded)
    observed: dict[int, tuple[bytes, int, bytes]] = {}
    for batch_item in _children(encoded, message):
        if batch_item.tag != 0x42000F:
            continue
        fields = _children(encoded, batch_item)
        operation = next(field for field in fields if field.tag == 0x42005C)
        operation_value = int.from_bytes(
            encoded[operation.value_start : operation.value_start + operation.value_length],
            "big",
        )
        batch_id = next(field for field in fields if field.tag == 0x420093)
        payload = next(field for field in fields if field.tag == 0x420079)
        identifiers = [field for field in _children(encoded, payload) if field.tag == 0x420094]
        assert len(identifiers) == 1
        identifier = identifiers[0]
        observed[operation_value] = (
            encoded[batch_id.value_start : batch_id.value_start + batch_id.value_length],
            identifier.item_type,
            encoded[
                identifier.value_start : identifier.value_start + identifier.value_length
            ],
        )
    return observed


def test_lifecycle_ids_and_failure_pending_responses_cross_native_cffi_once() -> None:
    port, requests, peer_failures, peer = _start_peer(_failure_pending_response())
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            batch = client.Batch.create()
            batch.set_asynchronous_indicator(1)
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_activate(
                    extensions.UniqueIdentifier.text_string("activate-target")
                ),
                _BATCH_IDS[0],
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_archive(
                    extensions.UniqueIdentifier.enumeration(0x8FED_CBA9)
                ),
                _BATCH_IDS[1],
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_destroy(
                    extensions.UniqueIdentifier.integer(-723)
                ),
                _BATCH_IDS[2],
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_recover(
                    extensions.UniqueIdentifier.text_string("recover-target")
                ),
                _BATCH_IDS[3],
            ))
            with ttlv.codec_limits_defaults() as limits:
                result = kmip_client.execute(batch, limits)

        peer.join(timeout=5)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert len(requests) == 1
        assert result is not None and len(result.items) == 4
        activate, archive, destroy, recover = result.items
        assert activate.operation == 0x12 and activate.result_status == 1
        assert activate.outcome is client.BatchOutcome.COMPLETED
        assert activate.result_reason == 5 and activate.result_message == "activate rejected"
        assert activate.activate_response is None
        assert activate.pending_correlation_value is None
        assert archive.operation == 0x15 and archive.result_status == 1
        assert archive.outcome is client.BatchOutcome.COMPLETED
        assert archive.result_reason == 5 and archive.result_message == "archive rejected"
        assert archive.archive_response is None
        assert destroy.operation == 0x14 and destroy.result_status == 2
        assert destroy.outcome is client.BatchOutcome.PENDING
        assert destroy.pending_correlation_value == _DESTROY_CORRELATION
        assert destroy.destroy_response is None
        assert recover.operation == 0x16 and recover.result_status == 2
        assert recover.outcome is client.BatchOutcome.PENDING
        assert recover.pending_correlation_value == _RECOVER_CORRELATION
        assert recover.recover_response is None

        assert _lifecycle_request_identifiers(requests[0]) == {
            0x12: (_BATCH_IDS[0], 7, b"activate-target"),
            0x15: (_BATCH_IDS[1], 5, struct.pack(">I", 0x8FED_CBA9)),
            0x14: (_BATCH_IDS[2], 2, struct.pack(">i", -723)),
            0x16: (_BATCH_IDS[3], 7, b"recover-target"),
        }
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=5)
        if not peer_failures.empty():
            raise peer_failures.get()
