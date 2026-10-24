from __future__ import annotations

import queue
import socket
import ssl
import struct
import sys
import threading
from pathlib import Path

import pytest

from kmipkit import client, errors, extensions, ttlv


_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_DEFAULT_MAX_MESSAGE_BYTES = 16 * 1024 * 1024
_RESPONSE_DATA = bytes(range(256)) * 65_536 + b"\xA5"
_BATCH_ITEM_ID = b"large-crypto-response"
_UNIQUE_IDENTIFIER = b"large-response-key"


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _response_message(operation: str, data: bytes) -> bytes:
    operation_code = {"encrypt": 0x1F, "decrypt": 0x20}[operation]
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
    response_payload = _ttlv_structure(
        0x42007C,
        _ttlv_item(0x420094, 7, _UNIQUE_IDENTIFIER),
        _ttlv_item(0x4200C2, 8, data),
    )
    response_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", operation_code)),
        _ttlv_item(0x420093, 8, _BATCH_ITEM_ID),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        response_payload,
    )
    return _ttlv_structure(0x42007B, response_header, response_item)


def _oversized_response_header() -> bytes:
    # Raw TLS rejects the declared complete frame size before allocating its body.
    body_length = _DEFAULT_MAX_MESSAGE_BYTES + 8
    return (
        (0x42007B).to_bytes(3, "big")
        + bytes((1,))
        + body_length.to_bytes(4, "big")
    )


def _query_response_message() -> bytes:
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
    response_payload = _ttlv_structure(0x42007C, _ttlv_structure(0x42015F))
    response_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x18)),
        _ttlv_item(0x420093, 8, _BATCH_ITEM_ID),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        response_payload,
    )
    return _ttlv_structure(0x42007B, response_header, response_item)


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
    body_length = int.from_bytes(header[4:8], "big")
    return header + _read_exact(stream, body_length)


def _start_mtls_peer(response: bytes) -> tuple[
    int, list[bytes], queue.Queue[BaseException], threading.Thread
]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(20)
    port = listener.getsockname()[1]
    received_requests: list[bytes] = []
    peer_failures: queue.Queue[BaseException] = queue.Queue()

    def serve_one_response() -> None:
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
                assert stream.version() == "TLSv1.3"
                assert stream.getpeercert(binary_form=True)
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(response)
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_response, daemon=True)
    peer.start()
    return port, received_requests, peer_failures, peer


def _transport(port: int) -> client.TransportConfig:
    client_certificate = client.TransportInput.file(
        _FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
    )
    client_key = client.TransportInput.file(
        _FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
    )
    server_certificate = client.TransportInput.file(
        _FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
    )
    return client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(client_certificate,),
        client_private_key=client_key,
        trust=client.TrustSource.certificate_authorities((server_certificate,)),
        connect_timeout=client.TimeoutLimit.bounded(10),
        write_timeout=client.TimeoutLimit.bounded(10),
        read_timeout=client.TimeoutLimit.bounded(30),
        total_timeout=client.TimeoutLimit.bounded(30),
    )


def _request_batch(operation: str, limits: ttlv.CodecLimits) -> client.Batch:
    data = ttlv.ttlv_value_byte_string(b"small-request-data", limits)
    try:
        make_item = getattr(extensions, f"client_batch_item_{operation}")
        item = make_item(
            unique_identifier=extensions.UniqueIdentifier.text_string(
                _UNIQUE_IDENTIFIER.decode("ascii")
            ),
            data=data,
        )
    finally:
        data.close()

    batch = client.Batch.create()
    batch.add_item(extensions.with_unique_batch_item_id(item, _BATCH_ITEM_ID))
    return batch


def _assert_request(operation: str, request: bytes) -> None:
    operation_code = {"encrypt": 0x1F, "decrypt": 0x20}[operation]
    assert _ttlv_item(0x42005C, 5, struct.pack(">I", operation_code)) in request
    assert _ttlv_item(0x420093, 8, _BATCH_ITEM_ID) in request


def _finish_peer(
    peer: threading.Thread, peer_failures: queue.Queue[BaseException]
) -> None:
    active_failure = sys.exc_info()[0] is not None
    peer.join(timeout=1 if active_failure else 35)
    if active_failure:
        return
    assert not peer.is_alive(), "the loopback mTLS peer did not finish"
    if not peer_failures.empty():
        raise AssertionError("the loopback mTLS peer failed") from peer_failures.get()


@pytest.mark.parametrize("operation", ("encrypt", "decrypt"))
def test_large_crypto_response_data_survives_closing_native_source_handles(
    operation: str,
) -> None:
    port, received_requests, peer_failures, peer = _start_mtls_peer(
        _response_message(operation, _RESPONSE_DATA)
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    response = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            with ttlv.codec_limits_create(32 * 1024 * 1024, 64, 100_000) as limits:
                batch = _request_batch(operation, limits)
                result = kmip_client.execute(batch, limits)
        assert len(received_requests) == 1
        _assert_request(operation, received_requests[0])
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        response = getattr(item_result, f"{operation}_response")
        assert response is not None

        # Closing the native aggregate and client handles must not invalidate the
        # separately owned Python response bytes.
        result.close()
        assert result.closed
        assert response.data == _RESPONSE_DATA
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
            _UNIQUE_IDENTIFIER.decode("ascii")
        )
    finally:
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        _finish_peer(peer, peer_failures)


@pytest.mark.parametrize("operation", ("encrypt", "decrypt"))
def test_default_codec_limits_reject_oversized_crypto_response_frame(
    operation: str,
) -> None:
    port, received_requests, peer_failures, peer = _start_mtls_peer(
        _oversized_response_header()
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                batch = _request_batch(operation, limits)
                with pytest.raises(errors.ClientExecutionError) as error:
                    kmip_client.execute(batch, limits)

        assert error.value.client_category is errors.ClientErrorCategory.TRANSPORT
        assert error.value.cause is errors.ClientErrorCause.TRANSPORT_OTHER
        assert error.value.delivery is errors.DeliveryState.RESPONSE_STARTED
        assert error.value.response_ttlv is None
        assert len(received_requests) == 1
        _assert_request(operation, received_requests[0])
    finally:
        _finish_peer(peer, peer_failures)


def test_query_with_100001_functions_and_groups_reaches_native_tls_peer() -> None:
    port, received_requests, peer_failures, peer = _start_mtls_peer(
        _query_response_message()
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    query_response = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            with ttlv.codec_limits_create(32 * 1024 * 1024, 64, 400_000) as limits:
                item = extensions.client_batch_item_query_with_limits(
                    [1] * 100_001,
                    ["large-object-group"] * 100_001,
                    limits,
                )
                batch = client.Batch.create()
                batch.add_item(extensions.with_unique_batch_item_id(item, _BATCH_ITEM_ID))
                result = kmip_client.execute(batch, limits)

        assert len(received_requests) == 1
        request = received_requests[0]
        assert request.count(_ttlv_item(0x420074, 5, struct.pack(">I", 1))) == 100_001
        assert request.count(
            _ttlv_item(0x420056, 7, b"large-object-group")
        ) == 100_001
        assert _ttlv_item(0x420093, 8, _BATCH_ITEM_ID) in request
        assert result is not None and len(result.items) == 1
        assert result.items[0].operation == 0x18
        query_response = result.items[0].query_response
        assert query_response is not None
    finally:
        if query_response is not None:
            query_response.close()
        if result is not None:
            result.close()
        _finish_peer(peer, peer_failures)
