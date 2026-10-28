from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

from _test_support import mtls_transport_config

from kmipkit import _handles, client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_TICKET_TYPE = 0xFEED_BEEF
_TICKET_VALUE = b"login-ticket-native-secret"
_TICKET_TAGS = (0x42014A, 0x42014B)
_PING_BATCH_ITEM_ID = b"ping-fixture"
_QUERY_BATCH_ITEM_ID = b"query-fixture"


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _login_response_message() -> bytes:
    protocol_version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    message_header = _ttlv_structure(
        0x42007A,
        protocol_version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    ticket = _ttlv_structure(
        0x420149,
        _ttlv_item(0x42014A, 5, struct.pack(">I", _TICKET_TYPE)),
        _ttlv_item(0x42014B, 8, _TICKET_VALUE),
    )
    login_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x2D)),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        _ttlv_structure(0x42007C, ticket),
    )
    return _ttlv_structure(0x42007B, message_header, login_item)


def _ping_query_response_message() -> bytes:
    protocol_version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    message_header = _ttlv_structure(
        0x42007A,
        protocol_version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 2)),
    )
    ping_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x3B)),
        _ttlv_item(0x420093, 8, _PING_BATCH_ITEM_ID),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        _ttlv_structure(0x42007C),
    )
    query_payload = _ttlv_structure(
        0x42007C,
        _ttlv_structure(0x42015F),
    )
    query_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x18)),
        _ttlv_item(0x420093, 8, _QUERY_BATCH_ITEM_ID),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        query_payload,
    )
    return _ttlv_structure(0x42007B, message_header, ping_item, query_item)


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


def _ticket_tags(ticket: ttlv.Structure) -> tuple[int, ...]:
    tags: list[int] = []
    with ttlv.ttlv_structure_view(ticket) as view:
        for index in range(ttlv.ttlv_structure_view_item_count(view)):
            with ttlv.ttlv_structure_view_item_at(view, index) as item:
                with ttlv.ttlv_item_view_tag(item) as tag:
                    tags.append(ttlv.tag_value(tag))
                expected_type = (ttlv.ItemType.Enumeration, ttlv.ItemType.ByteString)[index]
                assert ttlv.ttlv_item_view_type(item) is expected_type
    return tuple(tags)


def test_successful_login_uses_native_cffi_ticket_accessors_and_owned_ttlv() -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []

    def serve_one_login_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_login_response_message())
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_login_response, daemon=True)
    peer.start()
    transport = mtls_transport_config(port, _FIXTURES)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    native_releases: list[object] = []

    class TtlvLibraryReleaseSpy:
        def __init__(self, native_library: object) -> None:
            self._native_library = native_library

        def __getattr__(self, name: str) -> object:
            function = getattr(self._native_library, name)
            if name != "kmipkit_ttlv_structure_release":
                return function

            def release(handle: object) -> object:
                native_releases.append(handle)
                return function(handle)

            return release

    result = None
    login_response = None
    ticket_structure = None
    try:
        with configuration, client.Client.create(configuration, transport) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                batch = client.Batch.create()
                batch.add_item(extensions.client_batch_item_login())
                result = kmip_client.execute(batch, limits)
        assert not peer_failures.qsize(), "the loopback mTLS peer must complete its response"
        assert len(received_requests) == 1
        assert b"\x42\x00\x5c\x05\x00\x00\x00\x04\x00\x00\x00\x2d" in received_requests[0]
        assert result is not None and len(result.items) == 1
        login_response = result.items[0].login_response
        assert login_response is not None
        assert login_response.ticket_type_raw == _TICKET_TYPE
        assert login_response.ticket_value == _TICKET_VALUE
        assert _TICKET_VALUE.decode("ascii") not in repr(login_response)
        ticket_structure = login_response.ticket_ttlv
        assert ticket_structure is not None
        assert _ticket_tags(ticket_structure) == _TICKET_TAGS

        native_library = _handles.lib
        _handles.lib = TtlvLibraryReleaseSpy(native_library)
        try:
            login_response.close()
        finally:
            _handles.lib = native_library
        assert login_response.ticket_value is None
        assert login_response.ticket_ttlv is None
        assert ticket_structure.closed
        assert len(native_releases) == 1
    finally:
        if login_response is not None:
            login_response.close()
        if result is not None:
            result.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()


def test_ping_and_query_round_trip_through_native_cffi_over_tls_13() -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []

    def serve_ping_query_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_ping_query_response_message())
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_ping_query_response, daemon=True)
    peer.start()
    transport = mtls_transport_config(port, _FIXTURES)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    query_response = None
    try:
        with configuration, client.Client.create(configuration, transport) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                batch = client.Batch.create()
                batch.add_item(
                    extensions.with_unique_batch_item_id(
                        extensions.client_batch_item_ping(), _PING_BATCH_ITEM_ID
                    )
                )
                batch.add_item(
                    extensions.with_unique_batch_item_id(
                        extensions.client_batch_item_query(
                            [1, 2, 1], ["group-b", "group-a"]
                        ),
                        _QUERY_BATCH_ITEM_ID,
                    )
                )
                result = kmip_client.execute(batch, limits)

        assert not peer_failures.qsize(), "the loopback mTLS peer must complete its response"
        assert len(received_requests) == 1
        request = received_requests[0]
        assert _ttlv_structure(0x420079) in request
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x3B)) in request
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x18)) in request
        assert _ttlv_item(0x420074, 5, struct.pack(">I", 1)) in request
        assert _ttlv_item(0x420074, 5, struct.pack(">I", 2)) in request
        assert _ttlv_item(0x420056, 7, b"group-b") in request
        assert _ttlv_item(0x420056, 7, b"group-a") in request

        assert result is not None and len(result.items) == 2
        ping_result, query_result = result.items
        assert ping_result.operation == 0x3B
        assert ping_result.ping_response is True
        assert query_result.operation == 0x18
        query_response = query_result.query_response
        assert query_response is not None
        with ttlv.ttlv_structure_view(query_response) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 1
            with ttlv.ttlv_structure_view_item_at(view, 0) as item:
                with ttlv.ttlv_item_view_tag(item) as tag:
                    assert ttlv.tag_value(tag) == 0x42015F
                assert ttlv.ttlv_item_view_type(item) is ttlv.ItemType.Structure
    finally:
        if query_response is not None:
            query_response.close()
        if result is not None:
            result.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
