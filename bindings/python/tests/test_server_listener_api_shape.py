"""Consumer contract for typed server-to-client listener flows."""

from __future__ import annotations

import socket
import ssl
import threading
import traceback
from pathlib import Path

import pytest
from kmipkit.errors import InvalidInputError, KmipKitError
from kmipkit import ttlv
from kmipkit.attribute_operations import AttributeSet
from kmipkit._protocol_version import ProtocolVersion

from kmipkit.server import (
    DiscoverVersionsRequest,
    DiscoverVersionsResponse,
    NotifyRequest,
    NotifyResponse,
    PutRequest,
    PutResponse,
    QueryRequest,
    QueryResponse,
    ServerConnection,
    ServerListener,
    ServerListenerConfiguration,
    ServerRequestBatch,
    ServerRequestHandler,
    ServerReply,
    ServerResponse,
    ServerResponseBuilder,
    ServerResultReason,
    ServerResultStatus,
    SetEndpointRoleRequest,
    SetEndpointRoleResponse,
)


_FIXTURES = Path(__file__).parent / "fixtures" / "server-listener-mtls"


def _listener_configuration() -> ServerListenerConfiguration:
    return ServerListenerConfiguration(
        bind_host="127.0.0.1",
        bind_port=0,
        listener_certificate_chain=((_FIXTURES / "server-cert.pem").read_bytes(),),
        listener_private_key=(_FIXTURES / "server-key.pem").read_bytes(),
        trust_certificates=((_FIXTURES / "ca.pem").read_bytes(),),
    )


def _tls13_client_context() -> ssl.SSLContext:
    context = ssl.create_default_context(
        ssl.Purpose.SERVER_AUTH, cafile=str(_FIXTURES / "ca.pem")
    )
    context.load_cert_chain(_FIXTURES / "client-cert.pem", _FIXTURES / "client-key.pem")
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.maximum_version = ssl.TLSVersion.TLSv1_3
    return context


def _ttlv(tag: int, item_type: int, value: bytes) -> bytes:
    padding = (-len(value)) % 8
    return (
        tag.to_bytes(3, "big")
        + bytes((item_type,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes(padding)
    )


def _structure(tag: int, *children: bytes) -> bytes:
    return _ttlv(tag, 1, b"".join(children))


def _integer(tag: int, value: int) -> bytes:
    return _ttlv(tag, 2, value.to_bytes(4, "big", signed=True))


def _enumeration(tag: int, value: int) -> bytes:
    return _ttlv(tag, 5, value.to_bytes(4, "big"))


def _discover_versions_request() -> bytes:
    version = _structure(0x420069, _integer(0x42006A, 2), _integer(0x42006B, 1))
    header = _structure(0x420077, version, _integer(0x42000D, 1))
    batch_item = _structure(
        0x42000F,
        _enumeration(0x42005C, 30),
        _ttlv(0x420093, 8, b"batch-01"),
        _structure(0x420079),
    )
    return _structure(0x420078, header, batch_item)


def _notify_request() -> bytes:
    version = _structure(0x420069, _integer(0x42006A, 2), _integer(0x42006B, 1))
    header = _structure(0x420077, version, _integer(0x42000D, 1))
    payload = _structure(
        0x420079,
        _ttlv(0x420094, 7, b"object-01"),
        _structure(0x420125, _integer(0x42002A, 256)),
    )
    batch_item = _structure(
        0x42000F,
        _enumeration(0x42005C, 27),
        _ttlv(0x420093, 8, b"batch-01"),
        payload,
    )
    return _structure(0x420078, header, batch_item)


def _request_batch(*items: bytes) -> bytes:
    version = _structure(0x420069, _integer(0x42006A, 2), _integer(0x42006B, 1))
    header = _structure(0x420077, version, _integer(0x42000D, len(items)))
    return _structure(0x420078, header, *items)


def _request_batch_item(operation: int, batch_id: bytes, *payload: bytes) -> bytes:
    return _structure(
        0x42000F,
        _enumeration(0x42005C, operation),
        _ttlv(0x420093, 8, batch_id),
        _structure(0x420079, *payload),
    )


def _read_ttlv_frame(stream: ssl.SSLSocket) -> bytes:
    header = bytearray()
    while len(header) < 8:
        chunk = stream.recv(8 - len(header))
        if not chunk:
            raise AssertionError("listener closed before a complete response header")
        header.extend(chunk)
    value_length = int.from_bytes(header[4:8], "big")
    remaining = (value_length + 7) & ~7
    body = bytearray()
    while len(body) < remaining:
        chunk = stream.recv(remaining - len(body))
        if not chunk:
            raise AssertionError("listener closed before a complete response body")
        body.extend(chunk)
    return bytes(header + body)


def _run_tls13_mtls_exchange(
    request_bytes: bytes, handler: ServerRequestHandler
) -> tuple[bytes, list[type[object]]]:
    configuration = _listener_configuration()
    failures: list[str] = []
    seen: list[type[object]] = []

    def recording_handler(request: object) -> ServerReply:
        seen.append(type(request))
        return handler(request)

    with configuration, ServerListener.bind(configuration) as listener:
        port = listener.port

        def serve() -> None:
            try:
                with listener.accept(timeout=5) as connection:
                    connection.serve_once(recording_handler, timeout=5)
            except BaseException:
                failures.append(traceback.format_exc())

        worker = threading.Thread(target=serve, daemon=True)
        worker.start()
        context = _tls13_client_context()
        client_failure: BaseException | None = None
        response = b""
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=5) as raw:
                with context.wrap_socket(raw, server_hostname="127.0.0.1") as stream:
                    stream.settimeout(5)
                    stream.sendall(request_bytes)
                    response = _read_ttlv_frame(stream)
        except BaseException as failure:
            client_failure = failure
        worker.join(timeout=7)

    assert client_failure is None, (
        f"client failure={client_failure!r}; server={failures!r}"
    )
    assert not worker.is_alive()
    assert failures == []
    return response, seen


def test_server_listener_exports_typed_request_and_response_families() -> None:
    assert DiscoverVersionsRequest is not None
    assert NotifyRequest is not None
    assert PutRequest is not None
    assert QueryRequest is not None
    assert SetEndpointRoleRequest is not None
    assert DiscoverVersionsResponse is not None
    assert NotifyResponse is not None
    assert PutResponse is not None
    assert QueryResponse is not None
    assert SetEndpointRoleResponse is not None
    assert hasattr(NotifyRequest, "attributes")


def test_server_initiated_requests_have_no_client_send_route() -> None:
    for request_type in (
        DiscoverVersionsRequest,
        NotifyRequest,
        PutRequest,
        QueryRequest,
        SetEndpointRoleRequest,
    ):
        assert not hasattr(request_type, "send")


def test_named_listener_configuration_rejects_invalid_port() -> None:
    with pytest.raises(InvalidInputError):
        ServerListenerConfiguration(
            bind_host="127.0.0.1",
            bind_port=65536,
            listener_certificate_chain=(b"certificate",),
            listener_private_key=b"private key",
            trust_certificates=(b"authority",),
        )


def test_listener_accept_timeout_is_bounded_and_releases_listener() -> None:
    configuration = _listener_configuration()
    with configuration, ServerListener.bind(configuration) as listener:
        with pytest.raises(KmipKitError):
            listener.accept(timeout=0.01)
        listener.close()
        listener.close()


def test_handler_can_return_typed_success_or_failure() -> None:
    assert ServerResultReason.ITEM_NOT_FOUND == 1
    assert ServerResultReason.OPERATION_NOT_SUPPORTED == 5
    assert ServerResultReason.CRYPTOGRAPHIC_FAILURE == 10
    assert ServerResultReason.PERMISSION_DENIED == 12
    handler: ServerRequestHandler = lambda request: ServerReply.failure(
        ServerResultStatus.OPERATION_FAILED,
        ServerResultReason.OPERATION_NOT_SUPPORTED,
        "unsupported",
    )
    assert callable(handler)
    assert ServerListener is not None
    assert ServerConnection is not None
    assert ServerRequestBatch is not None
    assert ServerReply is not None
    assert ServerResponseBuilder is not None


def test_reply_constructor_cannot_bypass_typed_success_or_failure_factories() -> None:
    with pytest.raises(InvalidInputError):
        ServerReply(status=ServerResultStatus.OPERATION_FAILED)


def test_listener_serves_typed_discover_versions_over_tls13_mtls() -> None:
    def handler(request: object) -> ServerReply:
        assert isinstance(request, DiscoverVersionsRequest)
        return ServerReply.success(
            DiscoverVersionsResponse.for_request(request, (ProtocolVersion(2, 1),))
        )

    response, seen = _run_tls13_mtls_exchange(_discover_versions_request(), handler)
    assert seen == [DiscoverVersionsRequest]
    assert response[:4] == bytes.fromhex("42007b01")
    assert bytes.fromhex("42007f050000000400000000") in response
    assert bytes.fromhex("42006a020000000400000002") in response
    assert bytes.fromhex("42006b020000000400000001") in response


def test_listener_dispatches_all_five_typed_operations_over_tls13_mtls() -> None:
    discover = _request_batch_item(30, b"batch-01")
    notify = _request_batch_item(
        27,
        b"batch-02",
        _ttlv(0x420094, 7, b"object-01"),
        _structure(0x420125, _integer(0x42002A, 256)),
    )
    put = _request_batch_item(
        28,
        b"batch-03",
        _enumeration(0x420070, 1),
        _ttlv(0x420094, 7, b"object-02"),
        _structure(
            0x42008F,
            _structure(0x420040, _enumeration(0x420042, 1)),
        ),
    )
    query = _request_batch_item(24, b"batch-04", _enumeration(0x420074, 1))
    endpoint_role = _request_batch_item(
        50, b"batch-05", _enumeration(0x420151, 2)
    )
    request_bytes = _request_batch(discover, notify, put, query, endpoint_role)
    def handler(request: object) -> ServerReply:
        if isinstance(request, DiscoverVersionsRequest):
            assert request.requested_versions == ()
            return ServerReply.success(
                DiscoverVersionsResponse.for_request(request, (ProtocolVersion(2, 1),))
            )
        if isinstance(request, NotifyRequest):
            return ServerReply.success(NotifyResponse.for_request(request))
        if isinstance(request, PutRequest):
            assert request.put_function == 1
            with request.put_object() as obj:
                assert obj.tag == 0x42008F
            return ServerReply.success(PutResponse.for_request(request))
        if isinstance(request, QueryRequest):
            assert request.functions == (1,)
            return ServerReply.success(
                QueryResponse.for_request(request, operations=(24, 30))
            )
        assert isinstance(request, SetEndpointRoleRequest)
        assert request.endpoint_role == 2
        return ServerReply.success(SetEndpointRoleResponse.for_request(request, 1))

    response, seen = _run_tls13_mtls_exchange(request_bytes, handler)
    assert seen == [
        DiscoverVersionsRequest,
        NotifyRequest,
        PutRequest,
        QueryRequest,
        SetEndpointRoleRequest,
    ]
    assert response.count(bytes.fromhex("42007f050000000400000000")) == 5
    assert bytes.fromhex("420151050000000400000001") in response


def test_listener_returns_typed_handler_failure_over_tls13_mtls() -> None:
    def handler(_request: object) -> ServerReply:
        return ServerReply.failure(
            ServerResultStatus.OPERATION_FAILED,
            ServerResultReason.OPERATION_NOT_SUPPORTED,
            "unsupported",
        )

    response, _ = _run_tls13_mtls_exchange(_discover_versions_request(), handler)
    assert bytes.fromhex("42007f050000000400000001") in response
    assert bytes.fromhex("42007e050000000400000005") in response


def test_listener_preserves_notify_changed_attributes_over_tls13_mtls() -> None:
    def handler(request: object) -> ServerReply:
        assert isinstance(request, NotifyRequest)
        attributes = request.attributes
        assert isinstance(attributes, AttributeSet)
        with attributes, attributes.as_items() as items:
            with ttlv.ttlv_structure_view(items) as view:
                assert ttlv.ttlv_structure_view_item_count(view) == 1
        return ServerReply.success(NotifyResponse.for_request(request))

    response, seen = _run_tls13_mtls_exchange(_notify_request(), handler)
    assert seen == [NotifyRequest]
    assert bytes.fromhex("42007f050000000400000000") in response


def test_listener_rejects_malformed_ttlv_over_tls13_mtls() -> None:
    configuration = _listener_configuration()
    failures: list[BaseException] = []
    with configuration, ServerListener.bind(configuration) as listener:
        port = listener.port

        def receive() -> None:
            try:
                with listener.accept(timeout=5) as connection:
                    connection.receive(timeout=5)
            except BaseException as failure:
                failures.append(failure)

        worker = threading.Thread(target=receive, daemon=True)
        worker.start()
        context = _tls13_client_context()
        with socket.create_connection(("127.0.0.1", port), timeout=5) as raw:
            with context.wrap_socket(raw, server_hostname="127.0.0.1") as stream:
                stream.settimeout(5)
                stream.sendall(bytes.fromhex("42007801000000010000000000000000"))
        worker.join(timeout=7)

    assert not worker.is_alive()
    assert len(failures) == 1
    assert isinstance(failures[0], KmipKitError)
