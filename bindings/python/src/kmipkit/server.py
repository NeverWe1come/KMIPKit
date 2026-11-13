"""Typed Python facade for KMIP server-to-client listener exchanges.

The listener accepts and answers requests through KMIPKit's Rust-owned C ABI.
Private key staging buffers owned by this module are overwritten after native
configuration creation; Python runtime copies remain outside KMIPKit's control.
"""

from __future__ import annotations

import math
from enum import IntEnum
from typing import Callable, Sequence, TypeAlias

from . import errors, extensions, ttlv
from ._ffi import ffi, lib
from ._handles import (
    _NATIVE_HANDLE_TOKEN,
    NativeHandle,
    _NATIVE_CALL_LOCK,
    _integer_range,
    _wrap_owned_handle,
)
from ._protocol_version import ProtocolVersion

_MAX_INPUT_BYTES = 16 * 1024 * 1024
_MAX_INPUTS = 4_096
_MAX_TEXT_BYTES = 4_096
_RAW_TLS = 1
_HTTPS = 2
_TTLV = 1
_JSON = 2
_XML = 3
_MEMORY = 1
_PEM = 1
_DER = 2
_DISCOVER_VERSIONS = 30
_NOTIFY = 27
_PUT = 28
_QUERY = 24
_SET_ENDPOINT_ROLE = 50


class ListenerTransport(IntEnum):
    """Transport selected for the KMIP listener."""

    RAW_TLS = _RAW_TLS
    HTTPS = _HTTPS


class WireEncoding(IntEnum):
    """KMIP message encoding selected for requests and listener responses."""

    TTLV = _TTLV
    JSON = _JSON
    XML = _XML


def _status(status: int) -> None:
    errors.raise_for_status(int(status))


def _native_handle(c_type: str, function: Callable[..., int], *arguments: object):
    output = ffi.new(f"{c_type} **")
    _status(function(*arguments, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    return output[0]


def _span(data: bytes | bytearray):
    buffer = ffi.new("uint8_t[]", data) if data else ffi.NULL
    span = ffi.new("kmipkit_byte_span_v1_t *")
    span.data = buffer
    span.length = len(data)
    return span[0], buffer


def _text(value: str, *, optional: bool = False) -> bytes:
    if optional and value == "":
        return b""
    if not isinstance(value, str) or not value or "\x00" in value:
        errors.raise_for_status(4)
    encoded = value.encode("utf-8")
    if len(encoded) > _MAX_TEXT_BYTES:
        errors.raise_for_status(5)
    return encoded


def _timeout(value: float | None) -> tuple[int, int]:
    if value is None:
        return 0, 0
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        errors.raise_for_status(4)
    seconds = float(value)
    if not math.isfinite(seconds) or seconds < 0:
        errors.raise_for_status(4)
    total_nanoseconds = int(seconds * 1_000_000_000)
    whole_seconds, nanoseconds = divmod(total_nanoseconds, 1_000_000_000)
    if whole_seconds > (1 << 64) - 1:
        errors.raise_for_status(5)
    return whole_seconds, nanoseconds


def _input_array(
    values: Sequence[bytes | bytearray | memoryview], encoding: int, *, secret: bool = False
):
    if not isinstance(values, (tuple, list)) or not values or len(values) > _MAX_INPUTS:
        errors.raise_for_status(4)
    buffers = []
    inputs = ffi.new(f"kmipkit_transport_input_v1_t[{len(values)}]")
    total = 0
    for index, value in enumerate(values):
        if not isinstance(value, (bytes, bytearray, memoryview)) or len(value) == 0:
            errors.raise_for_status(4)
        data = bytearray(value) if secret else bytes(value)
        total += len(data)
        if total > _MAX_INPUT_BYTES:
            errors.raise_for_status(5)
        buffer = (
            ffi.from_buffer("uint8_t[]", data)
            if secret
            else ffi.new("uint8_t[]", data)
        )
        inputs[index].source_kind = _MEMORY
        inputs[index].encoding = encoding
        inputs[index].data.data = buffer
        inputs[index].data.length = len(data)
        buffers.append((data, buffer) if secret else buffer)
    return inputs, buffers


class ServerListenerConfiguration(NativeHandle):
    """Owned listener TLS and encoding configuration.

    Certificate chains and trust roots are copied during construction. The
    private-key staging buffer is overwritten after the native ABI copies it.
    """

    _release_name = "kmipkit_server_listener_config_release"
    __slots__ = ("_transport_kind",)

    def __init__(
        self,
        *,
        bind_host: str,
        bind_port: int,
        listener_certificate_chain: Sequence[bytes | bytearray | memoryview],
        listener_private_key: bytes | bytearray | memoryview,
        trust_certificates: Sequence[bytes | bytearray | memoryview],
        encoding: str = "pem",
        transport: ListenerTransport = ListenerTransport.RAW_TLS,
        wire_encoding: WireEncoding = WireEncoding.TTLV,
        target_path: str | None = None,
        revocation_lists: Sequence[bytes | bytearray | memoryview] = (),
        max_request_bytes: int = 0,
    ) -> None:
        host = _text(bind_host)
        port = _integer_range(bind_port, 0, 65_535)
        if not isinstance(transport, ListenerTransport) or not isinstance(
            wire_encoding, WireEncoding
        ):
            errors.raise_for_status(4)
        if encoding == "pem":
            native_encoding = _PEM
        elif encoding == "der":
            native_encoding = _DER
        else:
            errors.raise_for_status(4)
        path = b"" if target_path is None else _text(target_path)
        if transport is ListenerTransport.RAW_TLS and path:
            errors.raise_for_status(4)
        if not isinstance(listener_private_key, (bytes, bytearray, memoryview)):
            errors.raise_for_status(4)
        key = bytearray(listener_private_key)
        if not key or len(key) > _MAX_INPUT_BYTES:
            key[:] = b"\x00" * len(key)
            errors.raise_for_status(4)
        try:
            certificates, certificate_buffers = _input_array(
                listener_certificate_chain, native_encoding
            )
            authorities, authority_buffers = _input_array(trust_certificates, native_encoding)
            revocations = ffi.NULL
            revocation_buffers = []
            if revocation_lists:
                revocations, revocation_buffers = _input_array(revocation_lists, native_encoding)
            key_input = ffi.new("kmipkit_transport_input_v1_t *")
            key_buffer = ffi.from_buffer("uint8_t[]", key)
            key_input.source_kind = _MEMORY
            key_input.encoding = native_encoding
            key_input.data.data = key_buffer
            key_input.data.length = len(key)
            host_span, host_buffer = _span(host)
            path_span, path_buffer = _span(path)
            config = ffi.new("kmipkit_server_listener_config_v1_t *")
            config.struct_size = ffi.sizeof("kmipkit_server_listener_config_v1_t")
            config.abi_version = 1
            config.transport_kind = int(transport)
            config.wire_encoding = int(wire_encoding)
            config.bind_host = host_span
            config.bind_port = port
            config.listener_certificates = certificates
            config.listener_certificate_count = len(listener_certificate_chain)
            config.listener_private_key = key_input[0]
            config.trust_certificates = authorities
            config.trust_certificate_count = len(trust_certificates)
            config.revocation_lists = revocations
            config.revocation_list_count = len(revocation_lists)
            config.target_path = path_span
            config.max_request_bytes = _integer_range(
                max_request_bytes, 0, (1 << 64) - 1
            )
            with _NATIVE_CALL_LOCK:
                handle = _native_handle(
                    "kmipkit_server_listener_config_t",
                    lib.kmipkit_server_listener_config_create_v1,
                    config,
                )
            super().__init__(handle, _token=_NATIVE_HANDLE_TOKEN)
            self._transport_kind = transport
        finally:
            key[:] = b"\x00" * len(key)

    def __new__(cls, *arguments: object, **keywords: object):
        return object.__new__(cls)

    @property
    def transport_kind(self) -> ListenerTransport:
        """Return the transport selected when this configuration was built."""
        return self._transport_kind

    def __repr__(self) -> str:
        return f"ServerListenerConfiguration(closed={self.closed})"


class ServerListener(NativeHandle):
    """Bound TLS listener receiving typed server-initiated KMIP requests."""

    _release_name = "kmipkit_server_listener_release"

    @classmethod
    def bind(cls, configuration: ServerListenerConfiguration) -> ServerListener:
        if not isinstance(configuration, ServerListenerConfiguration):
            errors.raise_for_status(4)
        config = configuration._pointer()
        if configuration.transport_kind is ListenerTransport.RAW_TLS:
            function = lib.kmipkit_server_listener_bind_raw_tls
        elif configuration.transport_kind is ListenerTransport.HTTPS:
            function = lib.kmipkit_server_listener_bind_https
        else:
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            handle = _native_handle(
                "kmipkit_server_listener_t", function, config
            )
        return _wrap_owned_handle(cls, handle, owner=configuration)

    @property
    def port(self) -> int:
        output = ffi.new("uint16_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_listener_port(self._pointer(), output))
        return int(output[0])

    def accept(self, timeout: float | None = None) -> ServerConnection:
        seconds, nanoseconds = _timeout(timeout)
        with _NATIVE_CALL_LOCK:
            handle = _native_handle(
                "kmipkit_server_connection_t",
                lib.kmipkit_server_listener_accept,
                self._pointer(),
                seconds,
                nanoseconds,
            )
        return _wrap_owned_handle(ServerConnection, handle, owner=self)


class ServerConnection(NativeHandle):
    """Accepted server connection with one explicit receive and send flow."""

    _release_name = "kmipkit_server_connection_release"

    def receive(
        self, limits: ttlv.CodecLimits | None = None, timeout: float | None = None
    ) -> ServerRequestBatch:
        owned_limits = ttlv.codec_limits_defaults() if limits is None else None
        selected = owned_limits if owned_limits is not None else limits
        if not isinstance(selected, ttlv.CodecLimits):
            errors.raise_for_status(4)
        seconds, nanoseconds = _timeout(timeout)
        try:
            with _NATIVE_CALL_LOCK:
                handle = _native_handle(
                    "kmipkit_server_incoming_batch_t",
                    lib.kmipkit_server_connection_recv_batch,
                    self._pointer(),
                    selected._pointer(),
                    seconds,
                    nanoseconds,
                )
        finally:
            if owned_limits is not None:
                owned_limits.close()
        return _wrap_owned_handle(ServerRequestBatch, handle, owner=self)

    def send(self, response: ServerResponseMessage, timeout: float | None = None) -> None:
        if not isinstance(response, ServerResponseMessage):
            errors.raise_for_status(4)
        seconds, nanoseconds = _timeout(timeout)
        with _NATIVE_CALL_LOCK:
            _status(
                lib.kmipkit_server_connection_send_response(
                    self._pointer(), response._pointer(), seconds, nanoseconds
                )
            )

    def serve_once(
        self,
        handler: ServerRequestHandler,
        limits: ttlv.CodecLimits | None = None,
        timeout: float | None = None,
    ) -> None:
        """Receive one batch, map each item through ``handler``, and send one response.

        The caller owns the connection and chooses whether and how often to call
        this method. No request is retried or polled automatically.
        """
        if not callable(handler):
            errors.raise_for_status(4)
        with self.receive(limits, timeout) as batch:
            with batch.respond(handler) as response:
                self.send(response, timeout)


class ServerRequestBatch(NativeHandle):
    """Owned decoded batch of typed server-to-client request objects."""

    _release_name = "kmipkit_server_incoming_batch_release"

    @property
    def protocol_version(self) -> ProtocolVersion:
        output = ffi.new("kmipkit_protocol_version_v1_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_incoming_batch_protocol_version(self._pointer(), output))
        return ProtocolVersion(int(output.major), int(output.minor))

    def items(self) -> tuple[ServerRequest, ...]:
        count = ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_incoming_batch_count(self._pointer(), count))
        requests = []
        for index in range(int(count[0])):
            with _NATIVE_CALL_LOCK:
                handle = _native_handle(
                    "kmipkit_server_request_item_t",
                    lib.kmipkit_server_incoming_batch_item_at,
                    self._pointer(),
                    index,
                )
            operation_out = ffi.new("uint32_t *")
            with _NATIVE_CALL_LOCK:
                _status(lib.kmipkit_server_request_item_operation(handle, operation_out))
            request_type = _REQUEST_TYPES.get(int(operation_out[0]), ServerRequest)
            requests.append(_wrap_owned_handle(request_type, handle, owner=self))
        return tuple(requests)

    def response_builder(self) -> ServerResponseBuilder:
        with _NATIVE_CALL_LOCK:
            handle = _native_handle(
                "kmipkit_server_response_builder_t",
                lib.kmipkit_server_response_builder_create,
                self._pointer(),
            )
        return _wrap_owned_handle(ServerResponseBuilder, handle, owner=self)

    def respond(self, handler: ServerRequestHandler) -> ServerResponseMessage:
        """Build one typed response for each item by invoking ``handler`` once."""
        if not callable(handler):
            errors.raise_for_status(4)
        requests = self.items()
        builder = self.response_builder()
        responses: list[ServerResponse] = []
        try:
            for request in requests:
                reply = handler(request)
                if not isinstance(reply, ServerReply):
                    errors.raise_for_status(4)
                if isinstance(reply.response, ServerResponse):
                    responses.append(reply.response)
                builder.add(request, reply)
            return builder.build()
        finally:
            builder.close()
            for response in responses:
                response.close()
            for request in requests:
                request.close()


class ServerRequest(NativeHandle):
    """Common typed view for one server-originated request; it has no client send API."""

    _release_name = "kmipkit_server_request_item_release"
    @property
    def operation(self) -> int:
        output = ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_operation(self._pointer(), output))
        return int(output[0])

    @property
    def unique_identifier(self) -> bytes | None:
        output = ffi.new("kmipkit_byte_span_v1_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_unique_identifier(self._pointer(), output))
        length = int(output.length)
        if length == 0:
            return None
        return bytes(ffi.buffer(output.data, length))


class DiscoverVersionsRequest(ServerRequest):
    """Typed Discover Versions request received by the listener."""

    @property
    def requested_versions(self) -> tuple[ProtocolVersion, ...]:
        count = ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_discover_versions_count(self._pointer(), count))
        versions = []
        for index in range(int(count[0])):
            output = ffi.new("kmipkit_protocol_version_v1_t *")
            with _NATIVE_CALL_LOCK:
                _status(
                    lib.kmipkit_server_request_item_discover_version_at(
                        self._pointer(), index, output
                    )
                )
            versions.append(ProtocolVersion(int(output.major), int(output.minor)))
        return tuple(versions)


class NotifyRequest(ServerRequest):
    """Typed Notify request received by the listener."""

    @property
    def deleted_attribute_references(self) -> tuple[extensions.AttributeReference, ...]:
        count = ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            _status(
                lib.kmipkit_server_request_item_notify_deleted_attribute_reference_count(
                    self._pointer(), count
                )
            )
        references = []
        for index in range(int(count[0])):
            output = ffi.new("kmipkit_attribute_reference_v1_t *")
            with _NATIVE_CALL_LOCK:
                _status(
                    lib.kmipkit_server_request_item_notify_deleted_attribute_reference_at(
                        self._pointer(), index, output
                    )
                )
            if int(output.kind) == 1:
                references.append(extensions.AttributeReference.tag(int(output.tag_value)))
            else:
                vendor = bytes(ffi.buffer(output.vendor_identification_data,
                                          int(output.vendor_identification_length))).decode("utf-8")
                name = bytes(ffi.buffer(output.attribute_name_data,
                                        int(output.attribute_name_length))).decode("utf-8")
                references.append(extensions.AttributeReference.name(vendor, name))
        return tuple(references)


class PutObject(NativeHandle):
    """Typed KMIP managed object carried by a server-originated Put request."""

    _release_name = "kmipkit_server_put_object_release"

    @property
    def object_type(self) -> int:
        output = ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_put_object_type(self._pointer(), output))
        return int(output[0])

    @property
    def tag(self) -> int:
        output = ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_put_object_tag(self._pointer(), output))
        return int(output[0])


class PutRequest(ServerRequest):
    """Typed Put request received by the listener."""

    @property
    def put_function(self) -> int:
        output = ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_put_function(self._pointer(), output))
        return int(output[0])

    def put_object(self) -> PutObject:
        with _NATIVE_CALL_LOCK:
            handle = _native_handle(
                "kmipkit_server_put_object_t",
                lib.kmipkit_server_request_item_put_object,
                self._pointer(),
            )
        return _wrap_owned_handle(PutObject, handle, owner=self)


class QueryRequest(ServerRequest):
    """Typed Query request received by the listener."""

    @property
    def functions(self) -> tuple[int, ...]:
        count = ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_query_functions_count(self._pointer(), count))
        values = []
        for index in range(int(count[0])):
            output = ffi.new("uint32_t *")
            with _NATIVE_CALL_LOCK:
                _status(
                    lib.kmipkit_server_request_item_query_function_at(
                        self._pointer(), index, output
                    )
                )
            values.append(int(output[0]))
        return tuple(values)


class SetEndpointRoleRequest(ServerRequest):
    """Typed Set Endpoint Role request received by the listener."""

    @property
    def endpoint_role(self) -> int:
        output = ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            _status(lib.kmipkit_server_request_item_endpoint_role(self._pointer(), output))
        return int(output[0])


_REQUEST_TYPES = {
    _DISCOVER_VERSIONS: DiscoverVersionsRequest,
    _NOTIFY: NotifyRequest,
    _PUT: PutRequest,
    _QUERY: QueryRequest,
    _SET_ENDPOINT_ROLE: SetEndpointRoleRequest,
}


class ServerResponse(NativeHandle):
    """One typed success response payload paired with a matching request."""

    _release_name = "kmipkit_server_response_item_release"
    __slots__ = ("operation",)

    def __init__(self, handle, *, _token: object, owner: object = None, operation: int):
        super().__init__(handle, _token=_token, owner=owner)
        self.operation = operation


class NotifyResponse(ServerResponse):
    """Empty typed success response for Notify."""

    @classmethod
    def for_request(cls, request: NotifyRequest) -> NotifyResponse:
        return _response_for_request(cls, request, lib.kmipkit_server_notify_response_create)


class PutResponse(ServerResponse):
    """Empty typed success response for Put."""

    @classmethod
    def for_request(cls, request: PutRequest) -> PutResponse:
        return _response_for_request(cls, request, lib.kmipkit_server_put_response_create)


class DiscoverVersionsResponse(ServerResponse):
    """Typed Discover Versions response with explicitly selected versions."""

    @classmethod
    def for_request(
        cls, request: DiscoverVersionsRequest, versions: Sequence[ProtocolVersion]
    ) -> DiscoverVersionsResponse:
        if not isinstance(request, DiscoverVersionsRequest) or not isinstance(versions, (list, tuple)):
            errors.raise_for_status(4)
        selected = []
        for version in versions:
            if not isinstance(version, ProtocolVersion):
                errors.raise_for_status(4)
            selected.append((version.major, version.minor))
        values = ffi.new(f"kmipkit_protocol_version_v1_t[{len(selected)}]") if selected else ffi.NULL
        for index, (major, minor) in enumerate(selected):
            values[index].major = major
            values[index].minor = minor
        return _response_for_request(
            cls, request, lib.kmipkit_server_discover_versions_response_create, values, len(selected)
        )


class QueryResponse(ServerResponse):
    """Typed Query response with explicitly supplied capability collections."""

    @classmethod
    def for_request(
        cls,
        request: QueryRequest,
        *,
        operations: Sequence[int] = (),
        object_types: Sequence[int] = (),
        extension_tags: Sequence[int] = (),
        vendor_identification: str | None = None,
    ) -> QueryResponse:
        if not isinstance(request, QueryRequest):
            errors.raise_for_status(4)
        arrays = []
        for values in (operations, object_types, extension_tags):
            if not isinstance(values, (tuple, list)) or len(values) > _MAX_INPUTS:
                errors.raise_for_status(4)
            converted = [_integer_range(value, 0, 0xFFFF_FFFF) for value in values]
            array = ffi.new(f"uint32_t[{len(converted)}]", converted) if converted else ffi.NULL
            arrays.append((array, len(converted)))
        vendor, vendor_buffer = _span(
            b"" if vendor_identification is None else _text(vendor_identification)
        )
        return _response_for_request(
            cls,
            request,
            lib.kmipkit_server_query_response_create,
            arrays[0][0], arrays[0][1], arrays[1][0], arrays[1][1],
            arrays[2][0], arrays[2][1], vendor,
            _keepalive=(arrays, vendor_buffer),
        )


class SetEndpointRoleResponse(ServerResponse):
    """Typed Set Endpoint Role response."""

    @classmethod
    def for_request(
        cls, request: SetEndpointRoleRequest, endpoint_role: int,
        vendor_identification: str | None = None,
    ) -> SetEndpointRoleResponse:
        if not isinstance(request, SetEndpointRoleRequest):
            errors.raise_for_status(4)
        role = _integer_range(endpoint_role, 0, 0xFFFF_FFFF)
        vendor, vendor_buffer = _span(
            b"" if vendor_identification is None else _text(vendor_identification)
        )
        return _response_for_request(
            cls, request, lib.kmipkit_server_set_endpoint_role_response_create,
            role, vendor, _keepalive=vendor_buffer,
        )


def _response_for_request(cls, request, function, *arguments, _keepalive=None):
    if not isinstance(request, ServerRequest):
        errors.raise_for_status(4)
    with _NATIVE_CALL_LOCK:
        handle = _native_handle(
            "kmipkit_server_response_item_t", function, request._pointer(), *arguments
        )
    return _wrap_owned_handle(cls, handle, owner=request, operation=request.operation)


class ServerResultStatus(IntEnum):
    """KMIP Result Status values used by listener failure responses."""

    OPERATION_FAILED = 1
    OPERATION_PENDING = 2
    OPERATION_UNDONE = 3


class ServerResultReason(IntEnum):
    """Common KMIP Result Reason values; raw values remain accepted."""

    GENERAL_FAILURE = 1
    ITEM_NOT_FOUND = 2
    OPERATION_NOT_SUPPORTED = 3
    PERMISSION_DENIED = 4
    CRYPTOGRAPHIC_FAILURE = 5


class ServerReply:
    """Handler outcome paired with one received server-originated request."""

    __slots__ = ("response", "status", "reason", "message")

    def __init__(self, response=None, status=None, reason=None, message=None):
        self.response = response
        self.status = status
        self.reason = reason
        self.message = message

    @classmethod
    def success(cls, response: ServerResponse) -> ServerReply:
        if not isinstance(response, ServerResponse):
            errors.raise_for_status(4)
        return cls(response=response)

    @classmethod
    def failure(
        cls, status: ServerResultStatus | int, reason: ServerResultReason | int | None,
        message: str | None = None,
    ) -> ServerReply:
        result_status = _integer_range(status, 1, 0xFFFF_FFFF)
        result_reason = None if reason is None else _integer_range(reason, 0, 0xFFFF_FFFF)
        if message is not None:
            _text(message, optional=True)
        return cls(status=result_status, reason=result_reason, message=message)


ServerRequestHandler: TypeAlias = Callable[[ServerRequest], ServerReply]


class ServerResponseBuilder(NativeHandle):
    """Builds one response batch with a typed success or explicit failure per request."""

    _release_name = "kmipkit_server_response_builder_release"

    def add(self, request: ServerRequest, reply: ServerReply) -> None:
        if not isinstance(request, ServerRequest) or not isinstance(reply, ServerReply):
            errors.raise_for_status(4)
        if request._owner is not self._owner:
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            if reply.response is not None:
                if not isinstance(reply.response, ServerResponse):
                    errors.raise_for_status(4)
                _status(
                    lib.kmipkit_server_response_builder_add_success(
                        self._pointer(), request._pointer(), reply.response._pointer()
                    )
                )
            else:
                message, buffer = _span(
                    b"" if reply.message is None else _text(reply.message, optional=True)
                )
                _status(
                    lib.kmipkit_server_response_builder_add_failure(
                        self._pointer(), request._pointer(), reply.status,
                        int(reply.reason is not None), 0 if reply.reason is None else reply.reason,
                        message,
                    )
                )
                del buffer

    def build(self) -> ServerResponseMessage:
        with _NATIVE_CALL_LOCK:
            handle = _native_handle(
                "kmipkit_server_response_t",
                lib.kmipkit_server_response_builder_build,
                self._pointer(),
            )
        return _wrap_owned_handle(ServerResponseMessage, handle, owner=self._owner)


class ServerResponseMessage(NativeHandle):
    """Owned typed response batch ready for one explicit connection send."""

    _release_name = "kmipkit_server_response_release"

    def __repr__(self) -> str:
        return f"ServerResponseMessage(closed={self.closed})"


__all__ = [
    "DiscoverVersionsRequest", "DiscoverVersionsResponse", "ListenerTransport",
    "NotifyRequest", "NotifyResponse", "PutObject", "PutRequest", "PutResponse",
    "QueryRequest", "QueryResponse", "ServerConnection", "ServerListener",
    "ServerListenerConfiguration", "ServerReply", "ServerRequest", "ServerRequestBatch",
    "ServerRequestHandler", "ServerResponse", "ServerResponseBuilder",
    "ServerResponseMessage", "ServerResultReason", "ServerResultStatus",
    "SetEndpointRoleRequest", "SetEndpointRoleResponse", "WireEncoding",
]
