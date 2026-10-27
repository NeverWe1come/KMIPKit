"""Native CFFI Re-key response ownership tests for OASIS KMIP 2.1 §6.1.46."""

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
_BATCH_IDS = (b"item-0", b"item-1", b"item-2")
_CORRELATION = b"\x00\xa1\x7f"


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _enumeration(tag: int, value: int) -> bytes:
    return _ttlv_item(tag, 5, struct.pack(">I", value))


def _rekey_response_message() -> bytes:
    version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _ttlv_structure(
        0x42007A,
        version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 3)),
    )
    payload = _ttlv_structure(
        0x42007C,
        _ttlv_item(0x420094, 7, b"replacement-key"),
        _ttlv_item(0x541234, 8, b"\x01\x00\xff"),
    )
    success = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x04),
        _ttlv_item(0x420093, 8, _BATCH_IDS[0]),
        _enumeration(0x42007F, 0),
        payload,
    )
    failure = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x04),
        _ttlv_item(0x420093, 8, _BATCH_IDS[1]),
        _enumeration(0x42007F, 1),
        _enumeration(0x42007E, 5),
    )
    pending = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x04),
        _ttlv_item(0x420093, 8, _BATCH_IDS[2]),
        _enumeration(0x42007F, 2),
        _ttlv_item(0x420006, 8, _CORRELATION),
        _ttlv_structure(0x42007C),
    )
    return _ttlv_structure(0x42007B, header, success, failure, pending)


def _read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    output = bytearray()
    while len(output) < length:
        chunk = stream.recv(length - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a full TTLV frame")
        output.extend(chunk)
    return bytes(output)


def _read_ttlv_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    return header + _read_exact(stream, int.from_bytes(header[4:8], "big"))


def _item_tags(structure: ttlv.Structure) -> tuple[int, ...]:
    with ttlv.ttlv_structure_view(structure) as view:
        return _view_item_tags(view)


def _view_item_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    tags: list[int] = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with ttlv.ttlv_structure_view_item_at(view, index) as item:
            with ttlv.ttlv_item_view_tag(item) as tag:
                tags.append(ttlv.tag_value(tag))
    return tuple(tags)


def _start_mtls_peer() -> tuple[int, list[bytes], queue.Queue[BaseException], threading.Thread, socket.socket]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []

    def serve_one_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                require_tls13_mtls_peer(stream)
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_rekey_response_message())
        except OSError as failure:
            if listener.fileno() != -1:
                peer_failures.put(failure)
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_response, daemon=True)
    peer.start()
    return port, received_requests, peer_failures, peer, listener


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


def test_rekey_success_ttlv_outlives_source_and_failures_expose_no_success_result() -> None:
    port, received_requests, peer_failures, peer, listener = _start_mtls_peer()
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    batch_result: client.BatchResult | None = None
    rekey_response: client.RekeyResponse | None = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                batch = client.Batch.create()
                batch.set_asynchronous_indicator(1)
                for identifier, batch_id in zip(
                    ("existing-success", "existing-failure", "existing-pending"), _BATCH_IDS,
                    strict=True,
                ):
                    item = extensions.client_batch_item_rekey(
                        extensions.UniqueIdentifier.text_string(identifier)
                    )
                    batch.add_item(extensions.with_unique_batch_item_id(item, batch_id))
                batch_result = kmip_client.execute(batch, limits)

        assert len(received_requests) == 1
        assert received_requests[0].count(
            b"\x42\x00\x5c\x05\x00\x00\x00\x04\x00\x00\x00\x04"
        ) == 3
        assert peer_failures.empty()
        assert batch_result is not None and len(batch_result.items) == 3
        success, failure, pending = batch_result.items
        assert success.operation == 0x04
        assert success.result_status == 0
        rekey_response = success.rekey_response
        assert rekey_response is not None
        assert rekey_response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "replacement-key"
        )
        assert failure.operation == 0x04
        assert failure.result_status == 1
        assert failure.result_reason == 5
        assert failure.unique_identifier is None
        assert failure.rekey_response is None
        assert pending.operation == 0x04
        assert pending.result_status == 2
        assert pending.outcome is client.BatchOutcome.PENDING
        assert pending.pending_correlation_value == _CORRELATION
        assert pending.unique_identifier is None
        assert pending.rekey_response is None

        batch_result.close()
        assert not rekey_response.response_ttlv.closed
        assert _item_tags(rekey_response.response_ttlv) == (0x42005C, 0x420093, 0x42007F, 0x42007C)
        with ttlv.ttlv_structure_view(rekey_response.response_ttlv) as response_view:
            with ttlv.ttlv_structure_view_item_at(response_view, 3) as payload_item:
                with ttlv.ttlv_item_view_value(payload_item) as payload_value:
                    with ttlv.ttlv_value_view_structure(payload_value) as payload:
                            assert _view_item_tags(payload) == (0x420094, 0x541234)
        assert "replacement-key" not in repr(rekey_response)
    finally:
        if rekey_response is not None:
            rekey_response.close()
        if batch_result is not None:
            batch_result.close()
        listener.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
