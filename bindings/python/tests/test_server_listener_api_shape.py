"""Consumer contract for typed server-to-client listener flows."""

from __future__ import annotations

import pytest

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
    ServerResponse,
    ServerResponseBuilder,
    ServerResultReason,
    SetEndpointRoleRequest,
    SetEndpointRoleResponse,
)


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
    with pytest.raises(ValueError):
        ServerListenerConfiguration(
            bind_host="127.0.0.1",
            bind_port=65536,
            listener_certificate_chain=(b"certificate",),
            listener_private_key=b"private key",
            trust_certificates=(b"authority",),
        )


def test_handler_can_return_typed_success_or_failure() -> None:
    handler: ServerRequestHandler = lambda request: ServerResponse.failure(
        request, ServerResultReason.OPERATION_NOT_SUPPORTED, "unsupported"
    )
    assert callable(handler)
    assert ServerListener is not None
    assert ServerConnection is not None
    assert ServerRequestBatch is not None
    assert ServerResponseBuilder is not None
