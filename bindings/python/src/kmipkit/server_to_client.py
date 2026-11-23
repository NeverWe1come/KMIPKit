"""Server-to-Client (S2C) and Role Reversal protocol extensions for KMIPKit.

Provides ClientListener, ClientListenerConnection, RoleReversedSession,
IncomingServerBatch, ServerRequestHandler, ServerRequestDispatcher, and S2C utilities.
"""

from __future__ import annotations

import ctypes
import os
import sys
from abc import ABC, abstractmethod
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any

from . import errors, ttlv
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._ffi import ffi, lib, secret_callback_library
from ._handles import (
    NativeHandle,
    _NATIVE_CALL_LOCK,
    _integer_range,
    _owned_bytes,
    _wipe_staging_buffer,
    _wrap_owned_handle,
)
from ._protocol_version import ProtocolVersion
from .client import (
    CredentialEncoding,
    SetEndpointRoleResponse,
    TransportInput,
    _SecretBytes,
    _snapshot_transport_input,
    _snapshot_transport_inputs,
    _transport_source,
    _utf8_bytes,
    _validate_transport_input,
    _validate_transport_input_group,
)
from .extensions import ExtensionInformation, WireEncoding
from .messages import RequestBatchItemView, ResponseMessage

if TYPE_CHECKING:
    pass

_MAX_INPUT_BYTES = 16 * 1024 * 1024
_MAX_TEXT_BYTES = 4096


# --- Ctypes structures and callback types for ServerRequestDispatcher ---

class _CtypesServerRequestItemViewV1(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("operation", ctypes.c_uint32),
        ("unique_batch_item_id_data", ctypes.POINTER(ctypes.c_uint8)),
        ("unique_batch_item_id_length", ctypes.c_uint64),
    ]


class _CtypesByteSpanV1(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_uint8)),
        ("length", ctypes.c_uint64),
    ]


class _CtypesS2cOperationResultV1(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("result_status", ctypes.c_uint32),
        ("has_result_reason", ctypes.c_uint32),
        ("result_reason", ctypes.c_uint32),
        ("reserved", ctypes.c_uint32),
        ("result_message_data", ctypes.POINTER(ctypes.c_uint8)),
        ("result_message_length", ctypes.c_uint64),
    ]


class _CtypesS2cCapabilitiesV1(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("operations", ctypes.POINTER(ctypes.c_uint32)),
        ("operations_count", ctypes.c_uint32),
        ("object_types", ctypes.POINTER(ctypes.c_uint32)),
        ("object_types_count", ctypes.c_uint32),
        ("vendor_identification_data", ctypes.POINTER(ctypes.c_uint8)),
        ("vendor_identification_length", ctypes.c_uint64),
        ("extension_tags", ctypes.POINTER(ctypes.c_uint32)),
        ("extension_tags_count", ctypes.c_uint32),
        ("reserved", ctypes.c_uint32),
    ]


class _CtypesS2cCapabilitiesV2(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("base", _CtypesS2cCapabilitiesV1),
        ("extension_information", ctypes.POINTER(ctypes.c_void_p)),
        ("extension_information_count", ctypes.c_uint32),
        ("reserved", ctypes.c_uint32),
    ]


_DISCOVER_VERSIONS_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.POINTER(ctypes.c_uint32)),
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)

_NOTIFY_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)

_PUT_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)

_QUERY_V5_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.c_uint32,
    ctypes.POINTER(_CtypesS2cCapabilitiesV1),
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)

_SET_ENDPOINT_ROLE_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_uint8),
    ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(_CtypesByteSpanV1),
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)

_QUERY_V6_CALLBACK = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.POINTER(_CtypesServerRequestItemViewV1),
    ctypes.c_uint32,
    ctypes.POINTER(_CtypesS2cCapabilitiesV2),
    ctypes.POINTER(_CtypesS2cOperationResultV1),
)


class _CtypesServerRequestHandlerV5(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("user_data", ctypes.c_void_p),
        ("handle_discover_versions", _DISCOVER_VERSIONS_CALLBACK),
        ("handle_notify", _NOTIFY_CALLBACK),
        ("handle_put", _PUT_CALLBACK),
        ("handle_query", _QUERY_V5_CALLBACK),
        ("handle_set_endpoint_role", _SET_ENDPOINT_ROLE_CALLBACK),
    ]


class _CtypesServerRequestHandlerV6(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("abi_version", ctypes.c_uint32),
        ("base", _CtypesServerRequestHandlerV5),
        ("handle_query", _QUERY_V6_CALLBACK),
    ]


# --- Data models ---

@dataclass(frozen=True, slots=True)
class ItemView:
    """View of an incoming server-to-client request item."""

    operation: int
    unique_batch_item_id: bytes | None = None


@dataclass(frozen=True, slots=True)
class S2cOperationResult:
    """Explicit operation result returned by S2C callbacks."""

    result_status: int = 0
    result_reason: int | None = None
    result_message: str | None = None

    def __post_init__(self) -> None:
        _integer_range(self.result_status, 0, 0xFFFF_FFFF)
        if self.result_reason is not None:
            _integer_range(self.result_reason, 0, 0xFFFF_FFFF)
        if self.result_message is not None and not isinstance(self.result_message, str):
            errors.raise_for_status(4)

    @classmethod
    def success(cls) -> S2cOperationResult:
        return cls(result_status=0)

    @classmethod
    def failure(
        cls,
        result_status: int = 1,
        result_reason: int | None = 0x05,
        result_message: str | None = "operation not supported",
    ) -> S2cOperationResult:
        return cls(
            result_status=result_status,
            result_reason=result_reason,
            result_message=result_message,
        )


@dataclass(slots=True)
class S2cEndpointRoleResponse:
    """Response fields for a Set Endpoint Role operation."""

    endpoint_role: int
    vendor_identification: str | None = None

    def __post_init__(self) -> None:
        _integer_range(self.endpoint_role, 0, 0xFFFF_FFFF)
        if self.vendor_identification is not None and not isinstance(
            self.vendor_identification, str
        ):
            errors.raise_for_status(4)


@dataclass(slots=True)
class S2cCapabilities:
    """Advertised client capabilities for S2C Query responses."""

    operations: Sequence[int] = ()
    object_types: Sequence[int] = ()
    vendor_identification: str | bytes | None = None
    extension_tags: Sequence[int] = ()
    extension_information: Sequence[ExtensionInformation] = ()

    def __post_init__(self) -> None:
        if len(self.operations) > 1024:
            errors.raise_for_status(6)
        for op in self.operations:
            _integer_range(op, 0, 0xFFFF_FFFF)
        if len(self.object_types) > 1024:
            errors.raise_for_status(6)
        for ot in self.object_types:
            _integer_range(ot, 0, 0xFFFF_FFFF)
        if len(self.extension_tags) > 1024:
            errors.raise_for_status(6)
        for et in self.extension_tags:
            _integer_range(et, 0, 0xFFFF_FFFF)
        if len(self.extension_information) > 1024:
            errors.raise_for_status(6)
        for ext in self.extension_information:
            if not isinstance(ext, ExtensionInformation):
                errors.raise_for_status(4)


@dataclass(slots=True)
class PutItemDetails:
    """Details of an incoming S2C Put request item."""

    put_function: int
    unique_identifier: bytes | None
    replaced_unique_identifier: bytes | None
    object_item: ttlv.TtlvItem | None = None
    attributes: ttlv.Structure | None = None

    def close(self) -> None:
        if self.object_item is not None:
            self.object_item.close()
            self.object_item = None
        if self.attributes is not None:
            self.attributes.close()
            self.attributes = None

    def __enter__(self) -> PutItemDetails:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()


# --- ServerRequestHandler ABC ---

class ServerRequestHandler(ABC):
    """Callback interface for processing incoming server-to-client operations."""

    @abstractmethod
    def handle_discover_versions(
        self,
        item_view: ItemView,
        client_versions: tuple[ProtocolVersion, ...],
    ) -> tuple[Sequence[ProtocolVersion], S2cOperationResult]:
        """Handle Discover Versions. Must be implemented by caller."""
        ...

    def handle_notify(
        self,
        item_view: ItemView,
        unique_identifier: bytes | None,
        attributes_ttlv: bytes | None,
        deleted_references_ttlv: bytes | None = None,
    ) -> S2cOperationResult:
        """Handle Notify. Defaults to Operation Failed (status=1, reason=5), never silent success."""
        return S2cOperationResult.failure(1, 0x05, "operation not supported")

    @abstractmethod
    def handle_put(
        self,
        item_view: ItemView,
        put_details: PutItemDetails,
    ) -> S2cOperationResult:
        """Handle Put. Must be implemented by caller."""
        ...

    @abstractmethod
    def handle_query(
        self,
        item_view: ItemView,
        query_function_flags: int,
    ) -> tuple[S2cCapabilities, S2cOperationResult]:
        """Handle Query. Must be implemented by caller."""
        ...

    def handle_set_endpoint_role(
        self,
        item_view: ItemView,
        target_role: int,
        vendor_identification: str | None = None,
    ) -> tuple[S2cEndpointRoleResponse | None, S2cOperationResult]:
        """Handle Set Endpoint Role. Defaults to Operation Failed (status=1, reason=5), never silent success."""
        return None, S2cOperationResult.failure(1, 0x05, "operation not supported")


# --- Listener Transport & Options ---

@dataclass(slots=True)
class ListenerOptions:
    """Optional settings for ClientListener."""

    target_path: str | None = None
    revocation_lists: Sequence[bytes | Path] = ()
    wire_encoding: WireEncoding | None = None
    tls12_compat: bool = False

    def __post_init__(self) -> None:
        if self.target_path is not None:
            if not isinstance(self.target_path, str) or not self.target_path.startswith("/"):
                errors.raise_for_status(4)
        if not isinstance(self.tls12_compat, bool):
            errors.raise_for_status(4)
        if self.wire_encoding is not None and not isinstance(
            self.wire_encoding, WireEncoding
        ):
            errors.raise_for_status(4)


class ListenerTransportConfiguration:
    """Validated transport configuration for ClientListener with automatic private key zeroization."""

    __slots__ = (
        "bind_host",
        "bind_port",
        "flags",
        "server_certificates",
        "server_certificates_encoding",
        "server_private_key",
        "server_private_key_encoding",
        "trust_certificates",
        "trust_certificates_encoding",
        "read_timeout_ms",
        "write_timeout_ms",
        "max_request_bytes",
        "target_path",
        "revocation_lists",
        "revocation_lists_encoding",
        "wire_encoding",
        "_closed",
    )

    def __init__(
        self,
        bind_host: str = "127.0.0.1",
        bind_port: int = 0,
        flags: int = 0,
        server_certificates: Sequence[bytes | Path] = (),
        server_certificates_encoding: CredentialEncoding = CredentialEncoding.PEM,
        server_private_key: bytes | bytearray | Path | None = None,
        server_private_key_encoding: CredentialEncoding = CredentialEncoding.PEM,
        trust_certificates: Sequence[bytes | Path] = (),
        trust_certificates_encoding: CredentialEncoding = CredentialEncoding.PEM,
        read_timeout_ms: int = 30000,
        write_timeout_ms: int = 30000,
        max_request_bytes: int = 16 * 1024 * 1024,
        target_path: str | None = None,
        revocation_lists: Sequence[bytes | Path] = (),
        revocation_lists_encoding: CredentialEncoding = CredentialEncoding.PEM,
        wire_encoding: WireEncoding | None = None,
        tls12_compat: bool = False,
    ) -> None:
        if not isinstance(bind_host, str) or not bind_host:
            errors.raise_for_status(4)
        self.bind_host = bind_host
        self.bind_port = _integer_range(bind_port, 0, 65535)
        self.flags = flags | (4 if tls12_compat else 0)
        self.server_certificates = tuple(server_certificates)
        self.server_certificates_encoding = server_certificates_encoding
        if server_private_key is None:
            self.server_private_key: bytearray | Path = bytearray()
        elif isinstance(server_private_key, Path):
            self.server_private_key = server_private_key
        else:
            self.server_private_key = bytearray(server_private_key)
        self.server_private_key_encoding = server_private_key_encoding
        self.trust_certificates = tuple(trust_certificates)
        self.trust_certificates_encoding = trust_certificates_encoding
        self.read_timeout_ms = _integer_range(read_timeout_ms, 0, sys.maxsize)
        self.write_timeout_ms = _integer_range(write_timeout_ms, 0, sys.maxsize)
        self.max_request_bytes = _integer_range(max_request_bytes, 1, sys.maxsize)
        self.target_path = target_path
        self.revocation_lists = tuple(revocation_lists)
        self.revocation_lists_encoding = revocation_lists_encoding
        self.wire_encoding = wire_encoding
        self._closed = False

    def is_tls12_compat(self) -> bool:
        return (self.flags & 4) != 0

    def close(self) -> None:
        if not self._closed:
            self._closed = True
            if isinstance(self.server_private_key, bytearray):
                self.server_private_key[:] = b"\x00" * len(self.server_private_key)

    def __enter__(self) -> ListenerTransportConfiguration:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass

    @classmethod
    def builder(cls) -> ListenerTransportConfigurationBuilder:
        return ListenerTransportConfigurationBuilder()


class ListenerTransportConfigurationBuilder:
    """Builder for ListenerTransportConfiguration."""

    def __init__(self) -> None:
        self._bind_host: str = "127.0.0.1"
        self._bind_port: int = 0
        self._tls12_compat: bool = False
        self._server_certificates: list[bytes | Path] = []
        self._server_certificates_encoding: CredentialEncoding = CredentialEncoding.PEM
        self._server_private_key: bytearray | Path | None = None
        self._server_private_key_encoding: CredentialEncoding = CredentialEncoding.PEM
        self._trust_certificates: list[bytes | Path] = []
        self._trust_certificates_encoding: CredentialEncoding = CredentialEncoding.PEM
        self._read_timeout_ms: int = 30000
        self._write_timeout_ms: int = 30000
        self._max_request_bytes: int = 16 * 1024 * 1024
        self._target_path: str | None = None
        self._revocation_lists: list[bytes | Path] = []
        self._revocation_lists_encoding: CredentialEncoding = CredentialEncoding.PEM
        self._wire_encoding: WireEncoding | None = None
        self._built = False

    def bind_host(self, host: str) -> ListenerTransportConfigurationBuilder:
        self._bind_host = host
        return self

    def bind_port(self, port: int) -> ListenerTransportConfigurationBuilder:
        self._bind_port = port
        return self

    def tls12_compat(self, enabled: bool = True) -> ListenerTransportConfigurationBuilder:
        self._tls12_compat = enabled
        return self

    def add_server_certificate(
        self, cert: bytes | Path, encoding: CredentialEncoding = CredentialEncoding.PEM
    ) -> ListenerTransportConfigurationBuilder:
        self._server_certificates.append(cert)
        self._server_certificates_encoding = encoding
        return self

    def server_private_key(
        self, key: bytes | bytearray | Path, encoding: CredentialEncoding = CredentialEncoding.PEM
    ) -> ListenerTransportConfigurationBuilder:
        if isinstance(key, Path):
            self._server_private_key = key
        else:
            self._server_private_key = bytearray(key)
        self._server_private_key_encoding = encoding
        return self

    def add_trust_certificate(
        self, cert: bytes | Path, encoding: CredentialEncoding = CredentialEncoding.PEM
    ) -> ListenerTransportConfigurationBuilder:
        self._trust_certificates.append(cert)
        self._trust_certificates_encoding = encoding
        return self

    def add_revocation_list(
        self, crl: bytes | Path, encoding: CredentialEncoding = CredentialEncoding.PEM
    ) -> ListenerTransportConfigurationBuilder:
        self._revocation_lists.append(crl)
        self._revocation_lists_encoding = encoding
        return self

    def read_timeout_ms(self, timeout_ms: int) -> ListenerTransportConfigurationBuilder:
        if timeout_ms < 0:
            errors.raise_for_status(4)
        self._read_timeout_ms = timeout_ms
        return self

    def write_timeout_ms(self, timeout_ms: int) -> ListenerTransportConfigurationBuilder:
        if timeout_ms < 0:
            errors.raise_for_status(4)
        self._write_timeout_ms = timeout_ms
        return self

    def max_request_bytes(self, max_bytes: int) -> ListenerTransportConfigurationBuilder:
        if max_bytes <= 0:
            errors.raise_for_status(4)
        self._max_request_bytes = max_bytes
        return self

    def target_path(self, path: str) -> ListenerTransportConfigurationBuilder:
        self._target_path = path
        return self

    def wire_encoding(self, encoding: WireEncoding) -> ListenerTransportConfigurationBuilder:
        self._wire_encoding = encoding
        return self

    def build(self) -> ListenerTransportConfiguration:
        if self._built:
            errors.raise_for_status(4)
        self._built = True
        return ListenerTransportConfiguration(
            bind_host=self._bind_host,
            bind_port=self._bind_port,
            server_certificates=self._server_certificates,
            server_certificates_encoding=self._server_certificates_encoding,
            server_private_key=self._server_private_key,
            server_private_key_encoding=self._server_private_key_encoding,
            trust_certificates=self._trust_certificates,
            trust_certificates_encoding=self._trust_certificates_encoding,
            read_timeout_ms=self._read_timeout_ms,
            write_timeout_ms=self._write_timeout_ms,
            max_request_bytes=self._max_request_bytes,
            target_path=self._target_path,
            revocation_lists=self._revocation_lists,
            revocation_lists_encoding=self._revocation_lists_encoding,
            wire_encoding=self._wire_encoding,
            tls12_compat=self._tls12_compat,
        )

    def close(self) -> None:
        if isinstance(self._server_private_key, bytearray):
            self._server_private_key[:] = b"\x00" * len(self._server_private_key)
            self._server_private_key = None

    def __enter__(self) -> ListenerTransportConfigurationBuilder:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass


def _client_ptr(c_type: str, pointer: Any) -> Any:
    if pointer is None or pointer == _client_ffi.NULL or pointer == ffi.NULL:
        return _client_ffi.NULL
    try:
        address = int(_client_ffi.cast("uintptr_t", pointer))
    except (TypeError, ValueError):
        address = int(ffi.cast("uintptr_t", pointer))
    return _client_ffi.cast(c_type, address)


def _base_ptr(c_type: str, pointer: Any) -> Any:
    if pointer is None or pointer == _client_ffi.NULL or pointer == ffi.NULL:
        return ffi.NULL
    try:
        address = int(_client_ffi.cast("uintptr_t", pointer))
    except (TypeError, ValueError):
        address = int(ffi.cast("uintptr_t", pointer))
    return ffi.cast(c_type, address)


class _S2cHandle(NativeHandle):
    """Native handle released through the client CFFI library."""

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            if self.closed:
                return
            handle, owner = self._take()
            getattr(_client_lib, self._release_name)(handle)
            del owner


def _build_native_listener_config(
    transport: ListenerTransportConfiguration,
    options: ListenerOptions | None,
) -> tuple[Any, Any, list[tuple[Any, int]], list[Any]]:
    """Builds kmipkit_listener_transport_config_v1_t and options record with zeroization tracking."""
    wipe_buffers: list[tuple[Any, int]] = []
    owners: list[Any] = []

    def make_input_array(items: tuple[bytes | Path, ...], encoding: CredentialEncoding) -> tuple[Any, int]:
        if not items:
            return _client_ffi.NULL, 0
        arr = _client_ffi.new("kmipkit_transport_input_v1_t[]", len(items))
        for i, item in enumerate(items):
            if isinstance(item, Path):
                raw = str(item).encode("utf-8")
                buf = _client_ffi.new("uint8_t[]", raw)
                owners.append(buf)
                arr[i].source_kind = 2
                arr[i].encoding = int(encoding)
                arr[i].data.data = buf
                arr[i].data.length = len(raw)
            else:
                raw = bytes(item)
                buf = _client_ffi.new("uint8_t[]", raw)
                owners.append(buf)
                arr[i].source_kind = 1
                arr[i].encoding = int(encoding)
                arr[i].data.data = buf
                arr[i].data.length = len(raw)
        owners.append(arr)
        return arr, len(items)

    # Host span
    host_bytes = transport.bind_host.encode("utf-8")
    host_buf = _client_ffi.new("uint8_t[]", host_bytes)
    owners.append(host_buf)
    host_span = _client_ffi.new("kmipkit_byte_span_v1_t *", {"data": host_buf, "length": len(host_bytes)})[0]

    # Certs
    certs_arr, certs_count = make_input_array(
        transport.server_certificates, transport.server_certificates_encoding
    )

    # Key
    key_record = _client_ffi.new("kmipkit_transport_input_v1_t *")
    if isinstance(transport.server_private_key, Path):
        key_bytes = str(transport.server_private_key).encode("utf-8")
        key_buf = _client_ffi.new("uint8_t[]", key_bytes)
        owners.append(key_buf)
        key_record.source_kind = 2
        key_record.encoding = int(transport.server_private_key_encoding)
        key_record.data.data = key_buf
        key_record.data.length = len(key_bytes)
    else:
        raw_key = bytes(transport.server_private_key)
        if raw_key:
            key_buf = _client_ffi.new("uint8_t[]", raw_key)
            wipe_buffers.append((key_buf, len(raw_key)))
            key_record.source_kind = 1
            key_record.encoding = int(transport.server_private_key_encoding)
            key_record.data.data = key_buf
            key_record.data.length = len(raw_key)
        else:
            key_record.source_kind = 1
            key_record.encoding = int(transport.server_private_key_encoding)
            key_record.data.data = _client_ffi.NULL
            key_record.data.length = 0

    # Trust certs
    trust_arr, trust_count = make_input_array(
        transport.trust_certificates, transport.trust_certificates_encoding
    )

    # Native timeouts
    read_timeout = _client_ffi.new(
        "kmipkit_timeout_limit_v1_t *",
        {
            "kind": 1,
            "seconds": transport.read_timeout_ms // 1000,
            "nanoseconds": (transport.read_timeout_ms % 1000) * 1_000_000,
            "reserved": 0,
            "reserved2": 0,
        },
    )[0]
    write_timeout = _client_ffi.new(
        "kmipkit_timeout_limit_v1_t *",
        {
            "kind": 1,
            "seconds": transport.write_timeout_ms // 1000,
            "nanoseconds": (transport.write_timeout_ms % 1000) * 1_000_000,
            "reserved": 0,
            "reserved2": 0,
        },
    )[0]

    flags = transport.flags
    if options is not None and options.tls12_compat:
        flags |= 4

    config = _client_ffi.new("kmipkit_listener_transport_config_v1_t *")
    config.struct_size = _client_ffi.sizeof("kmipkit_listener_transport_config_v1_t")
    config.abi_version = 1
    config.flags = flags
    config.bind_port = transport.bind_port
    config.reserved = 0
    config.bind_host = host_span
    config.server_certificates = certs_arr
    config.server_certificate_count = certs_count
    config.server_private_key = key_record[0]
    config.trust_certificates = trust_arr
    config.trust_certificate_count = trust_count
    config.read_timeout = read_timeout
    config.write_timeout = write_timeout
    config.max_request_bytes = transport.max_request_bytes

    native_options = _client_ffi.NULL
    if options is not None:
        opts = _client_ffi.new("kmipkit_listener_options_v1_t *")
        opts.struct_size = _client_ffi.sizeof("kmipkit_listener_options_v1_t")
        opts.abi_version = 1
        if options.target_path is not None:
            tp_bytes = options.target_path.encode("utf-8")
            tp_buf = _client_ffi.new("uint8_t[]", tp_bytes)
            owners.append(tp_buf)
            opts.target_path.data = tp_buf
            opts.target_path.length = len(tp_bytes)
        else:
            opts.target_path.data = _client_ffi.NULL
            opts.target_path.length = 0

        rev_arr, rev_count = make_input_array(
            options.revocation_lists, CredentialEncoding.PEM
        )
        opts.revocation_lists = rev_arr
        opts.revocation_list_count = rev_count
        opts.wire_encoding = int(options.wire_encoding) if options.wire_encoding is not None else 0
        opts.reserved = 0
        native_options = opts

    return config, native_options, wipe_buffers, owners


# --- ClientListener & Connections ---

class ClientListener(_S2cHandle):
    """Server-to-Client listener handle."""

    _release_name = "kmipkit_client_listener_release"

    @classmethod
    def bind_raw_tls(
        cls,
        host: str,
        port: int,
        transport: ListenerTransportConfiguration,
        options: ListenerOptions | None = None,
    ) -> ClientListener:
        if not isinstance(host, str) or not host:
            errors.raise_for_status(4)
        transport.bind_host = host
        transport.bind_port = port
        config, native_opts, wipe_buffers, owners = _build_native_listener_config(transport, options)
        try:
            output = _client_ffi.new("kmipkit_client_listener_t **")
            with _NATIVE_CALL_LOCK:
                if native_opts == _client_ffi.NULL:
                    status = _client_lib.kmipkit_client_listener_bind_raw_tls(config, output)
                else:
                    status = _client_lib.kmipkit_client_listener_bind_raw_tls_with_options_v1(
                        config, native_opts, output
                    )
            errors.raise_for_status(status)
            if output[0] == _client_ffi.NULL:
                errors.raise_for_status(7)
            return _wrap_owned_handle(cls, output[0])
        finally:
            for buf, _ in wipe_buffers:
                _wipe_staging_buffer(buf)
            del owners

    @classmethod
    def bind_https(
        cls,
        host: str,
        port: int,
        transport: ListenerTransportConfiguration,
        options: ListenerOptions | None = None,
    ) -> ClientListener:
        if not isinstance(host, str) or not host:
            errors.raise_for_status(4)
        transport.bind_host = host
        transport.bind_port = port
        config, native_opts, wipe_buffers, owners = _build_native_listener_config(transport, options)
        try:
            output = _client_ffi.new("kmipkit_client_listener_t **")
            with _NATIVE_CALL_LOCK:
                if native_opts == _client_ffi.NULL:
                    status = _client_lib.kmipkit_client_listener_bind_https(config, output)
                else:
                    status = _client_lib.kmipkit_client_listener_bind_https_with_options_v1(
                        config, native_opts, output
                    )
            errors.raise_for_status(status)
            if output[0] == _client_ffi.NULL:
                errors.raise_for_status(7)
            return _wrap_owned_handle(cls, output[0])
        finally:
            for buf, _ in wipe_buffers:
                _wipe_staging_buffer(buf)
            del owners

    def get_port(self) -> int:
        port = _client_ffi.new("uint16_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(_client_lib.kmipkit_client_listener_get_port(self._pointer(), port))
        return int(port[0])

    def accept(self, timeout_ms: int | None = None) -> ClientListenerConnection:
        raw_timeout = 0 if timeout_ms is None else _integer_range(timeout_ms, 0, sys.maxsize)
        output = _client_ffi.new("kmipkit_client_listener_connection_t **")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(_client_lib.kmipkit_client_listener_accept(self._pointer(), raw_timeout, output))
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(ClientListenerConnection, output[0])


class ClientListenerConnection(_S2cHandle):
    """Accepted S2C connection handle."""

    _release_name = "kmipkit_client_listener_connection_release"

    def recv_batch(
        self, timeout_ms: int | None = None, limits: ttlv.CodecLimits | None = None
    ) -> IncomingServerBatch:
        raw_timeout = 0 if timeout_ms is None else _integer_range(timeout_ms, 0, sys.maxsize)
        created_limits = ttlv.codec_limits_defaults() if limits is None else None
        actual_limits = created_limits if created_limits is not None else limits
        try:
            output = _client_ffi.new("kmipkit_incoming_server_batch_t **")
            limits_ptr = _client_ptr("kmipkit_codec_limits_t *", actual_limits._pointer())
            with _NATIVE_CALL_LOCK:
                errors.raise_for_status(
                    _client_lib.kmipkit_client_listener_connection_recv_batch(
                        self._pointer(), limits_ptr, raw_timeout, output
                    )
                )
        finally:
            if created_limits is not None:
                created_limits.close()
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(IncomingServerBatch, output[0])

    def send_response(
        self,
        response: ResponseMessage | bytes | bytearray | memoryview,
        timeout_ms: int | None = None,
    ) -> None:
        raw_timeout = 0 if timeout_ms is None else _integer_range(timeout_ms, 0, sys.maxsize)
        if isinstance(response, ResponseMessage):
            resp_handle = _client_ptr("kmipkit_response_message_t *", response._pointer())
            with _NATIVE_CALL_LOCK:
                errors.raise_for_status(
                    _client_lib.kmipkit_client_listener_connection_send_response(
                        self._pointer(), resp_handle, raw_timeout
                    )
                )
        else:
            raw = bytes(response)
            resp_msg = ResponseMessage.from_bytes(raw)
            try:
                resp_handle = _client_ptr("kmipkit_response_message_t *", resp_msg._pointer())
                with _NATIVE_CALL_LOCK:
                    errors.raise_for_status(
                        _client_lib.kmipkit_client_listener_connection_send_response(
                            self._pointer(), resp_handle, raw_timeout
                        )
                    )
            finally:
                resp_msg.close()

    def into_role_reversed(self) -> RoleReversedSession:
        output = _client_ffi.new("kmipkit_role_reversed_session_t **")
        with _NATIVE_CALL_LOCK:
            handle, owner = self._take()
            try:
                status = _client_lib.kmipkit_client_listener_connection_into_role_reversed(
                    handle, output
                )
            except Exception:
                self._restore(handle, owner)
                raise
            if status != 0:
                self._restore(handle, owner)
                errors.raise_for_status(status)
            del owner
            _client_lib.kmipkit_client_listener_connection_release(handle)
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(RoleReversedSession, output[0])


class RoleReversedSession(_S2cHandle):
    """Role-reversed session handle."""

    _release_name = "kmipkit_role_reversed_session_release"

    def recv_batch(
        self, timeout_ms: int | None = None, limits: ttlv.CodecLimits | None = None
    ) -> IncomingServerBatch:
        raw_timeout = 0 if timeout_ms is None else _integer_range(timeout_ms, 0, sys.maxsize)
        created_limits = ttlv.codec_limits_defaults() if limits is None else None
        actual_limits = created_limits if created_limits is not None else limits
        try:
            output = _client_ffi.new("kmipkit_incoming_server_batch_t **")
            limits_ptr = _client_ptr("kmipkit_codec_limits_t *", actual_limits._pointer())
            with _NATIVE_CALL_LOCK:
                errors.raise_for_status(
                    _client_lib.kmipkit_role_reversed_session_recv_batch(
                        self._pointer(), limits_ptr, raw_timeout, output
                    )
                )
        finally:
            if created_limits is not None:
                created_limits.close()
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(IncomingServerBatch, output[0])

    def send_response(
        self,
        response: ResponseMessage | bytes | bytearray | memoryview,
        timeout_ms: int | None = None,
    ) -> None:
        raw_timeout = 0 if timeout_ms is None else _integer_range(timeout_ms, 0, sys.maxsize)
        if isinstance(response, ResponseMessage):
            resp_handle = _client_ptr("kmipkit_response_message_t *", response._pointer())
            with _NATIVE_CALL_LOCK:
                errors.raise_for_status(
                    _client_lib.kmipkit_role_reversed_session_send_response(
                        self._pointer(), resp_handle, raw_timeout
                    )
                )
        else:
            raw = bytes(response)
            resp_msg = ResponseMessage.from_bytes(raw)
            try:
                resp_handle = _client_ptr("kmipkit_response_message_t *", resp_msg._pointer())
                with _NATIVE_CALL_LOCK:
                    errors.raise_for_status(
                        _client_lib.kmipkit_role_reversed_session_send_response(
                            self._pointer(), resp_handle, raw_timeout
                        )
                    )
            finally:
                resp_msg.close()


class IncomingServerBatch(_S2cHandle):
    """Batch of operations received from the server in S2C mode."""

    _release_name = "kmipkit_incoming_server_batch_release"

    @classmethod
    def from_bytes(
        cls,
        data: bytes | bytearray | memoryview,
        encoding: WireEncoding | ttlv.CodecLimits = WireEncoding.TTLV,
        limits: ttlv.CodecLimits | None = None,
    ) -> IncomingServerBatch:
        if isinstance(encoding, ttlv.CodecLimits):
            limits = encoding
            encoding = WireEncoding.TTLV
        raw = bytes(data)
        created_limits = ttlv.codec_limits_defaults() if limits is None else None
        actual_limits = created_limits if created_limits is not None else limits
        try:
            output = _client_ffi.new("kmipkit_incoming_server_batch_t **")
            limits_ptr = _client_ptr("kmipkit_codec_limits_t *", actual_limits._pointer())
            with _NATIVE_CALL_LOCK:
                if encoding == WireEncoding.TTLV:
                    status = _client_lib.kmipkit_incoming_server_batch_decode_ttlv(
                        raw, len(raw), limits_ptr, output
                    )
                else:
                    status = _client_lib.kmipkit_incoming_server_batch_decode(
                        raw, len(raw), int(encoding), limits_ptr, output
                    )
        finally:
            if created_limits is not None:
                created_limits.close()
        errors.raise_for_status(status)
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(cls, output[0])
        errors.raise_for_status(status)
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(cls, output[0])

    @classmethod
    def decode_ttlv(
        cls,
        data: bytes | bytearray | memoryview,
        limits: ttlv.CodecLimits | None = None,
    ) -> IncomingServerBatch:
        return cls.from_bytes(data, encoding=WireEncoding.TTLV, limits=limits)

    def protocol_version(self) -> ProtocolVersion:
        major = _client_ffi.new("uint32_t *")
        minor = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_incoming_server_batch_protocol_version(
                    self._pointer(), major, minor
                )
            )
        return ProtocolVersion(int(major[0]), int(minor[0]))

    def item_count(self) -> int:
        count = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_incoming_server_batch_item_count(
                    self._pointer(), count
                )
            )
        return int(count[0])

    def operation_at(self, index: int) -> int:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        op = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_incoming_server_batch_item_operation(
                    self._pointer(), raw_idx, op
                )
            )
        return int(op[0])

    def unique_batch_item_id_at(self, index: int) -> bytes | None:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        buf = _client_ffi.new("uint8_t[256]")
        written = _client_ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_incoming_server_batch_item_unique_batch_item_id(
                self._pointer(), raw_idx, buf, 256, written
            )
        errors.raise_for_status(status)
        if written[0] == 0:
            return None
        return bytes(_client_ffi.buffer(buf, int(written[0])))

    def discover_versions_item(self, index: int) -> tuple[ProtocolVersion, ...]:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        buf = _client_ffi.new("uint32_t[128]")
        count = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_incoming_server_batch_item_discover_versions(
                self._pointer(), raw_idx, buf, 128, count
            )
        errors.raise_for_status(status)
        res = []
        for i in range(int(count[0])):
            packed = buf[i]
            res.append(ProtocolVersion(packed >> 16, packed & 0xFFFF))
        return tuple(res)

    def notify_item(self, index: int) -> bytes | None:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        buf = _client_ffi.new("uint8_t[256]")
        written = _client_ffi.new("uint64_t *")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_incoming_server_batch_item_notify(
                self._pointer(), raw_idx, buf, 256, written
            )
        errors.raise_for_status(status)
        if written[0] == 0:
            return None
        return bytes(_client_ffi.buffer(buf, int(written[0])))

    def put_item(self, index: int) -> PutItemDetails:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        fn = _client_ffi.new("uint32_t *")
        uid_buf = _client_ffi.new("uint8_t[256]")
        uid_len = _client_ffi.new("uint64_t *")
        ruid_buf = _client_ffi.new("uint8_t[256]")
        ruid_len = _client_ffi.new("uint64_t *")
        obj_item = _client_ffi.new("kmipkit_ttlv_item_t **")
        attrs = _client_ffi.new("kmipkit_ttlv_structure_t **")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_incoming_server_batch_item_put_v2(
                self._pointer(),
                raw_idx,
                fn,
                uid_buf,
                256,
                uid_len,
                ruid_buf,
                256,
                ruid_len,
                obj_item,
                attrs,
            )
        errors.raise_for_status(status)
        uid = bytes(_client_ffi.buffer(uid_buf, int(uid_len[0]))) if uid_len[0] > 0 else None
        ruid = (
            bytes(_client_ffi.buffer(ruid_buf, int(ruid_len[0]))) if ruid_len[0] > 0 else None
        )
        item_obj = (
            _wrap_owned_handle(
                ttlv.TtlvItem, _base_ptr("kmipkit_ttlv_item_t *", obj_item[0])
            )
            if obj_item[0] != _client_ffi.NULL
            else None
        )
        attrs_obj = (
            _wrap_owned_handle(
                ttlv.Structure, _base_ptr("kmipkit_ttlv_structure_t *", attrs[0])
            )
            if attrs[0] != _client_ffi.NULL
            else None
        )
        return PutItemDetails(
            put_function=int(fn[0]),
            unique_identifier=uid,
            replaced_unique_identifier=ruid,
            object_item=item_obj,
            attributes=attrs_obj,
        )

    def query_item(self, index: int) -> int:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        flags = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_incoming_server_batch_item_query(
                    self._pointer(), raw_idx, flags
                )
            )
        return int(flags[0])

    def set_endpoint_role_item(self, index: int) -> int:
        raw_idx = _integer_range(index, 0, 0xFFFF_FFFF)
        role = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_incoming_server_batch_item_set_endpoint_role(
                    self._pointer(), raw_idx, role
                )
            )
        return int(role[0])


# --- ServerRequestDispatcher ---

class ServerRequestDispatcher(_S2cHandle):
    """Dispatcher executing server-to-client operations via C ABI."""

    _release_name = "kmipkit_server_request_dispatcher_release"

    @classmethod
    def create(cls) -> ServerRequestDispatcher:
        output = _client_ffi.new("kmipkit_server_request_dispatcher_t **")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(_client_lib.kmipkit_server_request_dispatcher_create(output))
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(cls, output[0])

    def set_query_metadata(self, metadata: ttlv.Structure) -> None:
        """Set named Query capability fields that the callback leaves unspecified."""
        if not isinstance(metadata, ttlv.Structure):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(_client_lib.kmipkit_server_request_dispatcher_set_query_metadata(
                self._pointer(), _client_ptr("kmipkit_ttlv_structure_t *", metadata._pointer())
            ))

    def dispatch_v6(
        self,
        batch: IncomingServerBatch,
        handler: ServerRequestHandler,
    ) -> ResponseMessage:
        """Dispatch batch items to handler callbacks and produce a ResponseMessage."""
        if not isinstance(batch, IncomingServerBatch) or not isinstance(handler, ServerRequestHandler):
            errors.raise_for_status(4)

        # Staged C data alive for dispatch duration
        retained: list[Any] = []

        def set_operation_result(
            out_res_ptr: Any,
            res: S2cOperationResult,
        ) -> None:
            if not out_res_ptr:
                return
            rec = out_res_ptr.contents
            rec.struct_size = ctypes.sizeof(_CtypesS2cOperationResultV1)
            rec.abi_version = 1
            rec.result_status = res.result_status
            rec.has_result_reason = 1 if res.result_reason is not None else 0
            rec.result_reason = res.result_reason or 0
            rec.reserved = 0
            if res.result_message is not None:
                msg_bytes = res.result_message.encode("utf-8")
                msg_arr = (ctypes.c_uint8 * len(msg_bytes))(*msg_bytes)
                retained.append(msg_arr)
                rec.result_message_data = ctypes.cast(msg_arr, ctypes.POINTER(ctypes.c_uint8))
                rec.result_message_length = len(msg_bytes)
            else:
                rec.result_message_data = None
                rec.result_message_length = 0

        def py_discover_versions(
            user_data: Any,
            item_view_ptr: Any,
            requested_packed: Any,
            requested_count: int,
            supported_out_ptr: Any,
            supported_count_out_ptr: Any,
            out_result_ptr: Any,
        ) -> int:
            try:
                op = item_view_ptr.contents.operation
                item_id = None
                if item_view_ptr.contents.unique_batch_item_id_length > 0 and item_view_ptr.contents.unique_batch_item_id_data:
                    item_id = bytes(
                        ctypes.string_at(
                            item_view_ptr.contents.unique_batch_item_id_data,
                            item_view_ptr.contents.unique_batch_item_id_length,
                        )
                    )
                iv = ItemView(op, item_id)
                versions = []
                for i in range(requested_count):
                    val = requested_packed[i]
                    versions.append(ProtocolVersion(val >> 16, val & 0xFFFF))
                supported, res = handler.handle_discover_versions(iv, tuple(versions))
                set_operation_result(out_result_ptr, res)

                # Pack supported versions
                packed_list = []
                for pv in supported:
                    if isinstance(pv, ProtocolVersion):
                        packed_list.append((pv.major << 16) | (pv.minor & 0xFFFF))
                    elif isinstance(pv, (tuple, list)) and len(pv) >= 2:
                        packed_list.append((int(pv[0]) << 16) | (int(pv[1]) & 0xFFFF))
                arr = (ctypes.c_uint32 * len(packed_list))(*packed_list)
                retained.append(arr)
                supported_out_ptr[0] = ctypes.cast(arr, ctypes.POINTER(ctypes.c_uint32))
                supported_count_out_ptr[0] = len(packed_list)
                return res.result_status
            except Exception:
                return 5

        def py_notify(
            user_data: Any,
            item_view_ptr: Any,
            uid_data: Any,
            uid_len: int,
            attrs_data: Any,
            attrs_len: int,
            del_data: Any,
            del_len: int,
            out_result_ptr: Any,
        ) -> int:
            try:
                op = item_view_ptr.contents.operation
                item_id = None
                if item_view_ptr.contents.unique_batch_item_id_length > 0 and item_view_ptr.contents.unique_batch_item_id_data:
                    item_id = bytes(
                        ctypes.string_at(
                            item_view_ptr.contents.unique_batch_item_id_data,
                            item_view_ptr.contents.unique_batch_item_id_length,
                        )
                    )
                iv = ItemView(op, item_id)
                uid = bytes(ctypes.string_at(uid_data, uid_len)) if uid_len > 0 and uid_data else None
                attrs = bytes(ctypes.string_at(attrs_data, attrs_len)) if attrs_len > 0 and attrs_data else None
                del_refs = bytes(ctypes.string_at(del_data, del_len)) if del_len > 0 and del_data else None
                res = handler.handle_notify(iv, uid, attrs, del_refs)
                set_operation_result(out_result_ptr, res)
                return res.result_status
            except Exception:
                return 5

        def py_put(
            user_data: Any,
            item_view_ptr: Any,
            put_function: int,
            uid_data: Any,
            uid_len: int,
            ruid_data: Any,
            ruid_len: int,
            obj_data: Any,
            obj_len: int,
            attrs_data: Any,
            attrs_len: int,
            out_result_ptr: Any,
        ) -> int:
            try:
                op = item_view_ptr.contents.operation
                item_id = None
                if item_view_ptr.contents.unique_batch_item_id_length > 0 and item_view_ptr.contents.unique_batch_item_id_data:
                    item_id = bytes(
                        ctypes.string_at(
                            item_view_ptr.contents.unique_batch_item_id_data,
                            item_view_ptr.contents.unique_batch_item_id_length,
                        )
                    )
                iv = ItemView(op, item_id)
                uid = bytes(ctypes.string_at(uid_data, uid_len)) if uid_len > 0 and uid_data else None
                ruid = bytes(ctypes.string_at(ruid_data, ruid_len)) if ruid_len > 0 and ruid_data else None

                obj_item = None
                if obj_len > 0 and obj_data:
                    raw_obj = bytes(ctypes.string_at(obj_data, obj_len))
                    try:
                        obj_item = ttlv.ttlv_decode(raw_obj)
                    except Exception:
                        obj_item = None

                attrs_struct = None
                if attrs_len > 0 and attrs_data:
                    raw_attrs = bytes(ctypes.string_at(attrs_data, attrs_len))
                    try:
                        attrs_struct = ttlv.ttlv_decode(raw_attrs)
                    except Exception:
                        attrs_struct = None

                put_details = PutItemDetails(
                    put_function=put_function,
                    unique_identifier=uid,
                    replaced_unique_identifier=ruid,
                    object_item=obj_item,
                    attributes=attrs_struct,
                )
                try:
                    res = handler.handle_put(iv, put_details)
                finally:
                    put_details.close()
                set_operation_result(out_result_ptr, res)
                return res.result_status
            except Exception:
                return 5

        def py_query(
            user_data: Any,
            item_view_ptr: Any,
            query_flags: int,
            out_caps_ptr: Any,
            out_result_ptr: Any,
        ) -> int:
            try:
                op = item_view_ptr.contents.operation
                item_id = None
                if item_view_ptr.contents.unique_batch_item_id_length > 0 and item_view_ptr.contents.unique_batch_item_id_data:
                    item_id = bytes(
                        ctypes.string_at(
                            item_view_ptr.contents.unique_batch_item_id_data,
                            item_view_ptr.contents.unique_batch_item_id_length,
                        )
                    )
                iv = ItemView(op, item_id)
                caps, res = handler.handle_query(iv, query_flags)
                set_operation_result(out_result_ptr, res)

                if out_caps_ptr:
                    caps_rec = out_caps_ptr.contents
                    caps_rec.struct_size = ctypes.sizeof(_CtypesS2cCapabilitiesV2)
                    caps_rec.abi_version = 2
                    caps_rec.base.struct_size = ctypes.sizeof(_CtypesS2cCapabilitiesV1)
                    caps_rec.base.abi_version = 1

                    if caps.operations:
                        ops_arr = (ctypes.c_uint32 * len(caps.operations))(*caps.operations)
                        retained.append(ops_arr)
                        caps_rec.base.operations = ctypes.cast(ops_arr, ctypes.POINTER(ctypes.c_uint32))
                        caps_rec.base.operations_count = len(caps.operations)
                    else:
                        caps_rec.base.operations = None
                        caps_rec.base.operations_count = 0

                    if caps.object_types:
                        ots_arr = (ctypes.c_uint32 * len(caps.object_types))(*caps.object_types)
                        retained.append(ots_arr)
                        caps_rec.base.object_types = ctypes.cast(ots_arr, ctypes.POINTER(ctypes.c_uint32))
                        caps_rec.base.object_types_count = len(caps.object_types)
                    else:
                        caps_rec.base.object_types = None
                        caps_rec.base.object_types_count = 0

                    if caps.vendor_identification is not None:
                        vid_bytes = (
                            caps.vendor_identification.encode("utf-8")
                            if isinstance(caps.vendor_identification, str)
                            else bytes(caps.vendor_identification)
                        )
                        vid_arr = (ctypes.c_uint8 * len(vid_bytes))(*vid_bytes)
                        retained.append(vid_arr)
                        caps_rec.base.vendor_identification_data = ctypes.cast(
                            vid_arr, ctypes.POINTER(ctypes.c_uint8)
                        )
                        caps_rec.base.vendor_identification_length = len(vid_bytes)
                    else:
                        caps_rec.base.vendor_identification_data = None
                        caps_rec.base.vendor_identification_length = 0

                    if caps.extension_tags:
                        tags_arr = (ctypes.c_uint32 * len(caps.extension_tags))(*caps.extension_tags)
                        retained.append(tags_arr)
                        caps_rec.base.extension_tags = ctypes.cast(tags_arr, ctypes.POINTER(ctypes.c_uint32))
                        caps_rec.base.extension_tags_count = len(caps.extension_tags)
                    else:
                        caps_rec.base.extension_tags = None
                        caps_rec.base.extension_tags_count = 0

                    if caps.extension_information:
                        ext_ptrs = [
                            ctypes.c_void_p(int(_client_ffi.cast("uintptr_t", ext._pointer())))
                            for ext in caps.extension_information
                        ]
                        ext_arr = (ctypes.c_void_p * len(ext_ptrs))(*ext_ptrs)
                        retained.append(ext_arr)
                        caps_rec.extension_information = ctypes.cast(
                            ext_arr, ctypes.POINTER(ctypes.c_void_p)
                        )
                        caps_rec.extension_information_count = len(ext_ptrs)
                    else:
                        caps_rec.extension_information = None
                        caps_rec.extension_information_count = 0
                return res.result_status
            except Exception:
                return 5

        def py_set_endpoint_role(
            user_data: Any,
            item_view_ptr: Any,
            target_role: int,
            vendor_id_data: Any,
            vendor_id_len: int,
            out_role_ptr: Any,
            out_vendor_ptr: Any,
            out_result_ptr: Any,
        ) -> int:
            try:
                op = item_view_ptr.contents.operation
                item_id = None
                if item_view_ptr.contents.unique_batch_item_id_length > 0 and item_view_ptr.contents.unique_batch_item_id_data:
                    item_id = bytes(
                        ctypes.string_at(
                            item_view_ptr.contents.unique_batch_item_id_data,
                            item_view_ptr.contents.unique_batch_item_id_length,
                        )
                    )
                iv = ItemView(op, item_id)
                vendor_id = (
                    bytes(ctypes.string_at(vendor_id_data, vendor_id_len)).decode("utf-8", errors="strict")
                    if vendor_id_len > 0 and vendor_id_data
                    else None
                )
                resp, res = handler.handle_set_endpoint_role(iv, target_role, vendor_id)
                set_operation_result(out_result_ptr, res)
                if resp is not None:
                    if out_role_ptr:
                        out_role_ptr.contents.value = resp.endpoint_role
                    if out_vendor_ptr and resp.vendor_identification is not None:
                        vid_b = resp.vendor_identification.encode("utf-8")
                        vid_arr = (ctypes.c_uint8 * len(vid_b))(*vid_b)
                        retained.append(vid_arr)
                        out_vendor_ptr.contents.data = ctypes.cast(vid_arr, ctypes.POINTER(ctypes.c_uint8))
                        out_vendor_ptr.contents.length = len(vid_b)
                return res.result_status
            except Exception:
                return 5

        # Build v6 handler record
        cb_disc = _DISCOVER_VERSIONS_CALLBACK(py_discover_versions)
        cb_notif = _NOTIFY_CALLBACK(py_notify)
        cb_put = _PUT_CALLBACK(py_put)
        cb_q5 = _QUERY_V5_CALLBACK(lambda *args: 0)
        cb_ser = _SET_ENDPOINT_ROLE_CALLBACK(py_set_endpoint_role)
        cb_q6 = _QUERY_V6_CALLBACK(py_query)

        retained.extend([cb_disc, cb_notif, cb_put, cb_q5, cb_ser, cb_q6])

        h_v5 = _CtypesServerRequestHandlerV5(
            struct_size=ctypes.sizeof(_CtypesServerRequestHandlerV5),
            abi_version=5,
            user_data=None,
            handle_discover_versions=cb_disc,
            handle_notify=cb_notif,
            handle_put=cb_put,
            handle_query=cb_q5,
            handle_set_endpoint_role=cb_ser,
        )
        h_v6 = _CtypesServerRequestHandlerV6(
            struct_size=ctypes.sizeof(_CtypesServerRequestHandlerV6),
            abi_version=6,
            base=h_v5,
            handle_query=cb_q6,
        )

        lib_c = secret_callback_library()
        dispatch_fn = lib_c.kmipkit_server_request_dispatcher_dispatch_v6
        dispatch_fn.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.POINTER(_CtypesServerRequestHandlerV6),
            ctypes.POINTER(ctypes.c_void_p),
        ]
        dispatch_fn.restype = ctypes.c_int32

        raw_disp = ctypes.c_void_p(int(_client_ffi.cast("uintptr_t", self._pointer())))
        raw_batch = ctypes.c_void_p(int(_client_ffi.cast("uintptr_t", batch._pointer())))
        out_resp = ctypes.c_void_p(0)

        with _NATIVE_CALL_LOCK:
            status = dispatch_fn(raw_disp, raw_batch, ctypes.byref(h_v6), ctypes.byref(out_resp))
        errors.raise_for_status(status)
        if not out_resp.value:
            errors.raise_for_status(7)
        return ResponseMessage._from_native_address(out_resp.value)


# --- S2cUtilities ---

def _to_protocol_version(v: Any) -> ProtocolVersion:
    if isinstance(v, ProtocolVersion):
        return v
    if isinstance(v, (tuple, list)) and len(v) == 2:
        return ProtocolVersion(int(v[0]), int(v[1]))
    errors.raise_for_status(4)


class S2cUtilities:
    """Public standalone utilities for Server-to-Client protocol operations."""

    @staticmethod
    def negotiate_discover_versions(
        requested_version: ProtocolVersion | tuple[int, int],
        supported_versions: Sequence[ProtocolVersion | tuple[int, int]],
    ) -> ProtocolVersion | None:
        """Negotiates the highest mutually supported version for single requested version."""
        req_v = _to_protocol_version(requested_version)
        packed_supported = []
        for v in supported_versions:
            norm_v = _to_protocol_version(v)
            packed_supported.append((norm_v.major << 16) | (norm_v.minor & 0xFFFF))
        arr = _client_ffi.new("uint32_t[]", packed_supported)
        out_maj = _client_ffi.new("uint32_t *")
        out_min = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_s2c_discover_versions_negotiate(
                req_v.major,
                req_v.minor,
                arr,
                len(packed_supported),
                out_maj,
                out_min,
            )
        errors.raise_for_status(status)
        if out_maj[0] == 0 and out_min[0] == 0:
            return None
        return ProtocolVersion(int(out_maj[0]), int(out_min[0]))

    @staticmethod
    def negotiate_discover_versions_list(
        requested_versions: Sequence[ProtocolVersion | tuple[int, int]],
        supported_versions: Sequence[ProtocolVersion | tuple[int, int]],
    ) -> tuple[ProtocolVersion, ...]:
        """Returns the negotiated version list in preference order."""
        req_norm = [_to_protocol_version(v) for v in requested_versions]
        sup_norm = [_to_protocol_version(v) for v in supported_versions]

        req_arr = _client_ffi.new("kmipkit_protocol_version_v1_t[]", len(req_norm))
        for i, v in enumerate(req_norm):
            req_arr[i].major = v.major
            req_arr[i].minor = v.minor

        sup_arr = _client_ffi.new("kmipkit_protocol_version_v1_t[]", len(sup_norm))
        for i, v in enumerate(sup_norm):
            sup_arr[i].major = v.major
            sup_arr[i].minor = v.minor

        capacity = len(sup_norm) + 16
        out_arr = _client_ffi.new("kmipkit_protocol_version_v1_t[]", capacity)
        out_count = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_s2c_discover_versions_negotiate_v2(
                req_arr,
                len(req_norm),
                sup_arr,
                len(sup_norm),
                out_arr,
                capacity,
                out_count,
            )
        errors.raise_for_status(status)
        res = []
        for i in range(int(out_count[0])):
            res.append(ProtocolVersion(int(out_arr[i].major), int(out_arr[i].minor)))
        return tuple(res)

    @staticmethod
    def put_effective_function(requested_function: int, replaced_object_exists: bool) -> int:
        """Computes effective Put function (degrades Replace to New if object does not exist)."""
        raw_fn = _integer_range(requested_function, 0, 0xFFFF_FFFF)
        out_fn = _client_ffi.new("uint32_t *")
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_s2c_put_effective_function(
                    raw_fn, 1 if replaced_object_exists else 0, out_fn
                )
            )
        return int(out_fn[0])

    @staticmethod
    def build_query_response_ttlv(
        query_function_flags: int,
        vendor_identification: str | bytes | None = None,
        capabilities: S2cCapabilities | None = None,
        version: ProtocolVersion | None = None,
    ) -> ttlv.Structure:
        """Builds a Query response payload TTLV structure."""
        raw_flags = _integer_range(query_function_flags, 0, 0xFFFF_FFFF)
        output = _client_ffi.new("kmipkit_ttlv_structure_t **")
        if capabilities is None:
            if vendor_identification is not None:
                vid_bytes = (
                    vendor_identification.encode("utf-8")
                    if isinstance(vendor_identification, str)
                    else bytes(vendor_identification)
                )
                vid_buf = _client_ffi.new("uint8_t[]", vid_bytes)
                vid_len = len(vid_bytes)
            else:
                vid_buf = _client_ffi.NULL
                vid_len = 0
            with _NATIVE_CALL_LOCK:
                status = _client_lib.kmipkit_s2c_query_respond_ttlv(
                    raw_flags, vid_buf, vid_len, output
                )
        else:
            ver_pv = version if version is not None else ProtocolVersion(2, 1)
            c_ver = _client_ffi.new(
                "kmipkit_protocol_version_v1_t *",
                {"major": ver_pv.major, "minor": ver_pv.minor},
            )[0]
            # Build v2 capabilities record
            caps_rec = _client_ffi.new("kmipkit_s2c_capabilities_v2_t *")
            caps_rec.struct_size = _client_ffi.sizeof("kmipkit_s2c_capabilities_v2_t")
            caps_rec.abi_version = 2
            caps_rec.base.struct_size = _client_ffi.sizeof("kmipkit_s2c_capabilities_v1_t")
            caps_rec.base.abi_version = 1

            owners: list[Any] = []
            if capabilities.operations:
                ops_arr = _client_ffi.new("uint32_t[]", list(capabilities.operations))
                owners.append(ops_arr)
                caps_rec.base.operations = ops_arr
                caps_rec.base.operations_count = len(capabilities.operations)

            if capabilities.object_types:
                ots_arr = _client_ffi.new("uint32_t[]", list(capabilities.object_types))
                owners.append(ots_arr)
                caps_rec.base.object_types = ots_arr
                caps_rec.base.object_types_count = len(capabilities.object_types)

            vid = capabilities.vendor_identification or vendor_identification
            if vid is not None:
                vid_b = vid.encode("utf-8") if isinstance(vid, str) else bytes(vid)
                vid_buf = _client_ffi.new("uint8_t[]", vid_b)
                owners.append(vid_buf)
                caps_rec.base.vendor_identification_data = vid_buf
                caps_rec.base.vendor_identification_length = len(vid_b)

            if capabilities.extension_tags:
                tags_arr = _client_ffi.new("uint32_t[]", list(capabilities.extension_tags))
                owners.append(tags_arr)
                caps_rec.base.extension_tags = tags_arr
                caps_rec.base.extension_tags_count = len(capabilities.extension_tags)

            if capabilities.extension_information:
                ext_arr = _client_ffi.new(
                    "kmipkit_extension_information_t *[]",
                    [
                        _client_ptr(
                            "kmipkit_extension_information_t *", ext._pointer()
                        )
                        for ext in capabilities.extension_information
                    ],
                )
                owners.append(ext_arr)
                caps_rec.extension_information = ext_arr
                caps_rec.extension_information_count = len(capabilities.extension_information)

            with _NATIVE_CALL_LOCK:
                status = _client_lib.kmipkit_s2c_query_respond_with_capabilities_v2(
                    raw_flags, c_ver, caps_rec, output
                )
            del owners

        errors.raise_for_status(status)
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(
            ttlv.Structure, _base_ptr("kmipkit_ttlv_structure_t *", output[0])
        )

    @staticmethod
    def build_response_ttlv(
        version: ProtocolVersion,
        items: Sequence[ttlv.Structure],
    ) -> ttlv.Structure:
        """Constructs an outer Response Message TTLV structure."""
        if not isinstance(version, ProtocolVersion):
            errors.raise_for_status(4)
        ptrs = [
            _client_ptr("kmipkit_ttlv_structure_t *", item._pointer())
            for item in items
        ]
        arr = _client_ffi.new("kmipkit_ttlv_structure_t *[]", ptrs)
        output = _client_ffi.new("kmipkit_ttlv_structure_t **")
        with _NATIVE_CALL_LOCK:
            status = _client_lib.kmipkit_s2c_build_response_ttlv(
                version.major, version.minor, arr, len(items), output
            )
        errors.raise_for_status(status)
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(
            ttlv.Structure, _base_ptr("kmipkit_ttlv_structure_t *", output[0])
        )

    @staticmethod
    def validate_notify_request(item_view: RequestBatchItemView) -> None:
        if not isinstance(item_view, RequestBatchItemView):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_s2c_notify_request_validate(item_view._pointer())
            )

    @staticmethod
    def validate_put_request(item_view: RequestBatchItemView) -> None:
        if not isinstance(item_view, RequestBatchItemView):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_s2c_put_request_validate(item_view._pointer())
            )

    @staticmethod
    def validate_query_request(item_view: RequestBatchItemView) -> None:
        if not isinstance(item_view, RequestBatchItemView):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_s2c_query_request_validate(item_view._pointer())
            )

    @staticmethod
    def validate_discover_versions_request(item_view: RequestBatchItemView) -> None:
        if not isinstance(item_view, RequestBatchItemView):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            errors.raise_for_status(
                _client_lib.kmipkit_s2c_discover_versions_request_validate(item_view._pointer())
            )


__all__ = [
    "ClientListener",
    "ClientListenerConnection",
    "IncomingServerBatch",
    "ItemView",
    "ListenerOptions",
    "ListenerTransportConfiguration",
    "ListenerTransportConfigurationBuilder",
    "PutItemDetails",
    "RoleReversedSession",
    "S2cCapabilities",
    "S2cEndpointRoleResponse",
    "S2cOperationResult",
    "S2cUtilities",
    "ServerRequestDispatcher",
    "ServerRequestHandler",
]
