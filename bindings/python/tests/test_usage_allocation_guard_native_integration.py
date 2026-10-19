"""Project-derived native binding checks for KMIP 2.1 §§6.1.17 and 6.1.23.

These tests are not official OASIS conformance cases.
"""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

import pytest

from kmipkit import client, errors, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_OBJECT_ID = b"guarded-python-key"
_ALLOCATION_BATCH_ID = b"usage-allocation"
_ENCRYPT_BATCH_ID = b"guarded-encrypt"
_ALLOCATION_COUNT = 3
_USAGE_UNIT = 0xFEDC_BA98


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = (
        tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    )
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _response_message(operation: int, batch_id: bytes, *payload_fields: bytes) -> bytes:
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
    response_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", operation)),
        _ttlv_item(0x420093, 8, batch_id),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        _ttlv_structure(0x42007C, *payload_fields),
    )
    return _ttlv_structure(0x42007B, response_header, response_item)


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


def _start_mtls_peer() -> tuple[
    int, list[bytes], queue.Queue[Exception], threading.Thread
]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(2)
    listener.settimeout(15)
    port = listener.getsockname()[1]
    received_requests: list[bytes] = []
    peer_failures: queue.Queue[Exception] = queue.Queue()
    responses = (
        _response_message(
            0x11,
            _ALLOCATION_BATCH_ID,
            _ttlv_item(0x420094, 7, _OBJECT_ID),
        ),
        _response_message(
            0x1F,
            _ENCRYPT_BATCH_ID,
            _ttlv_item(0x420094, 7, _OBJECT_ID),
            _ttlv_item(0x4200C2, 8, b"ciphertext"),
        ),
    )

    def serve_two_responses() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            for response in responses:
                raw, _address = listener.accept()
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    assert stream.version() == "TLSv1.3"
                    assert stream.getpeercert(binary_form=True)
                    received_requests.append(_read_ttlv_frame(stream))
                    stream.sendall(response)
        # Forward peer assertion and socket failures to the owning test thread.
        except Exception as failure:  # noqa: BLE001
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_two_responses, daemon=True)
    peer.start()
    return port, received_requests, peer_failures, peer


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
        total_timeout=client.TimeoutLimit.bounded(10),
    )


def test_native_python_guard_executes_encrypt_and_rejects_over_budget_before_transport() -> (
    None
):
    port, received_requests, peer_failures, peer = _start_mtls_peer()
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    allocation_result = None
    encrypt_result = None
    context = None
    try:
        with (
            configuration,
            client.Client.create(configuration, _transport(port)) as kmip_client,
            ttlv.codec_limits_defaults() as limits,
        ):
            allocation = client.Batch.create()
            allocation.add_item(
                extensions.with_unique_batch_item_id(
                    extensions.client_batch_item_get_usage_allocation(
                        extensions.UniqueIdentifier.text_string(
                            _OBJECT_ID.decode("ascii")
                        ),
                        _ALLOCATION_COUNT,
                    ),
                    _ALLOCATION_BATCH_ID,
                )
            )
            allocation_result = kmip_client.execute(allocation, limits)
            context = allocation_result.items[0].create_usage_allocation_context(
                _ALLOCATION_COUNT, _USAGE_UNIT
            )
            allocation_result.close()

            data = ttlv.ttlv_value_byte_string(b"plaintext", limits)
            try:
                encrypt_item = extensions.client_batch_item_encrypt(
                    unique_identifier=extensions.UniqueIdentifier.text_string(
                        _OBJECT_ID.decode("ascii")
                    ),
                    data=data,
                )
            finally:
                data.close()
            encrypt_item = extensions.with_unique_batch_item_id(
                encrypt_item, _ENCRYPT_BATCH_ID
            )
            encrypt_result = kmip_client.execute_guarded_protection_item(
                context, encrypt_item, 2, limits
            )
            assert len(encrypt_result.items) == 1
            assert encrypt_result.items[0].encrypt_response is not None
            assert encrypt_result.items[0].encrypt_response.data == b"ciphertext"
            assert context.allocated_units == 3
            assert context.reserved_units == 2
            assert context.available_units == 1

            excess_item = extensions.client_batch_item_encrypt(
                unique_identifier=extensions.UniqueIdentifier.text_string(
                    _OBJECT_ID.decode("ascii")
                )
            )
            with pytest.raises(errors.ClientExecutionError):
                kmip_client.execute_guarded_protection_item(
                    context, excess_item, 2, limits
                )
            assert excess_item.closed
            assert context.reserved_units == 2
            assert context.available_units == 1

        assert len(received_requests) == 2
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x11)) in received_requests[0]
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x1F)) in received_requests[1]
        assert peer_failures.empty()
    finally:
        if encrypt_result is not None:
            encrypt_result.close()
        if allocation_result is not None:
            allocation_result.close()
        if context is not None:
            context.close()
        peer.join(timeout=15)
        assert not peer.is_alive(), "the loopback mTLS peer must complete"
        if not peer_failures.empty():
            raise AssertionError(
                "the loopback mTLS peer failed"
            ) from peer_failures.get()
