"""Typed Python facade for the approved shared KMIPKit client ABI.

This module exposes transport configuration, typed ordered batches, and common
outcome metadata. Per-operation builders remain owned by their operation
specifications; callers can add only the typed batch items those specs expose.
KMIPKit-owned credential inputs use mutable buffers that are wiped before
release. Copies returned to Python callers and copies retained by Python or
foreign runtimes remain subject to those runtimes' memory-management limits.
"""

from __future__ import annotations

import hashlib
import ipaddress
import sys
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any, Self
from urllib.parse import urlsplit

from . import errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _NATIVE_CALL_LOCK,
    _integer_range,
    _owned_bytes,
    _wrap_owned_handle,
)


_MAX_INPUT_BYTES = 16 * 1024 * 1024
_MAX_TEXT_BYTES = 4_096
_MAX_BATCH_ITEMS = 4_096
_MAX_TRANSPORT_CONFIG_ITEMS = 4_096
_MAX_U64 = (1 << 64) - 1
_MIN_I64 = -(1 << 63)
_MAX_I64 = (1 << 63) - 1
_MIN_I32 = -(1 << 31)
_MAX_I32 = (1 << 31) - 1


def _unsigned(value: int, bits: int, *, allow_zero: bool = True) -> int:
    number = _integer_range(value, 0, (1 << bits) - 1)
    if not allow_zero and number == 0:
        errors.raise_for_status(4)
    return number


def _is_ip_literal(host: str) -> bool:
    try:
        ipaddress.ip_address(host.strip("[]"))
    except ValueError:
        return False
    return True


class CredentialEncoding(IntEnum):
    """Explicit encoding used for certificate, key, and revocation material."""

    PEM = 1
    DER = 2


class EndpointKind(IntEnum):
    """The supported KMIP client transport endpoint forms."""

    RAW_TLS = 1
    HTTPS = 2


class TrustKind(IntEnum):
    """The explicit trust source used to verify the KMIP server."""

    PLATFORM = 1
    CERTIFICATES = 2


class TimeoutKind(IntEnum):
    """Whether a transport timeout uses the default, a bound, or no bound."""

    DEFAULT = 0
    BOUNDED = 1
    UNBOUNDED = 2


class BatchOutcome(IntEnum):
    """Common outcome shape exposed by T076; unknown values remain numeric."""

    COMPLETED = 1
    PENDING = 2


@dataclass(frozen=True, slots=True)
class ProtocolVersion:
    """A KMIP protocol version pair returned by Discover Versions."""

    major: int
    minor: int


@dataclass(frozen=True, slots=True, repr=False)
class BatchItemMetadataV2:
    """Version 2 metadata with an exact, redacted Result Message field.

    The message is copied into a Python ``str`` before releasing the native
    response item and is bounded by the response codec limit (16 MiB by
    default). Python does not guarantee deterministic zeroization of that
    external copy after it becomes unreachable.
    """

    operation: int
    outcome: BatchOutcome | int
    result_status: int
    result_reason: int | None
    result_message: str | None = field(repr=False)

    def __repr__(self) -> str:
        return (
            "BatchItemMetadataV2(operation="
            f"{self.operation}, outcome={self.outcome!r}, result_status={self.result_status}, "
            f"result_reason={self.result_reason!r}, result_message=redacted)"
        )

    def __post_init__(self) -> None:
        object.__setattr__(self, "operation", _unsigned(self.operation, 32))
        raw_outcome = _unsigned(self.outcome, 32)
        try:
            outcome: BatchOutcome | int = BatchOutcome(raw_outcome)
        except ValueError:
            outcome = raw_outcome
        object.__setattr__(self, "outcome", outcome)
        object.__setattr__(self, "result_status", _unsigned(self.result_status, 32))
        if self.result_reason is not None:
            object.__setattr__(self, "result_reason", _unsigned(self.result_reason, 32))
        if self.result_message is not None:
            if not isinstance(self.result_message, str):
                errors.raise_for_status(4)
            try:
                self.result_message.encode("utf-8", errors="strict")
            except UnicodeEncodeError:
                errors.raise_for_status(4)


@dataclass(frozen=True, slots=True)
class Endpoint:
    """A raw TLS host/port or HTTPS URI, with an optional TLS name for IP hosts."""

    kind: EndpointKind
    address: str = field(repr=False)
    port: int = 0
    tls_server_name: str | None = field(default=None, repr=False)
    target_uri: str | None = field(default=None, repr=False)

    def __post_init__(self) -> None:
        if not isinstance(self.kind, EndpointKind):
            errors.raise_for_status(4)
        _nonempty_text(self.address)
        if self.kind is EndpointKind.RAW_TLS:
            port = _unsigned(self.port, 16, allow_zero=False)
            object.__setattr__(self, "port", port)
            if self.tls_server_name is not None:
                _nonempty_text(self.tls_server_name)
                if not _is_ip_literal(self.address):
                    errors.raise_for_status(4)
            if self.target_uri is not None:
                errors.raise_for_status(4)
            return
        if self.kind is EndpointKind.HTTPS:
            port = _unsigned(self.port, 16)
            object.__setattr__(self, "port", port)
            if port != 0:
                errors.raise_for_status(4)
            if self.tls_server_name is not None:
                _nonempty_text(self.tls_server_name)
                try:
                    host = urlsplit(self.address).hostname
                except ValueError:
                    errors.raise_for_status(4)
                if host is None or not _is_ip_literal(host):
                    errors.raise_for_status(4)
            if self.target_uri is not None:
                _nonempty_text(self.target_uri)
            return
        errors.raise_for_status(4)

    @classmethod
    def raw_tls(
        cls, host: str, port: int, *, tls_server_name: str | None = None
    ) -> Self:
        """Create a raw TLS endpoint, optionally overriding the name for an IP host."""
        validated = Endpoint(EndpointKind.RAW_TLS, host, port, tls_server_name)
        return cls(
            validated.kind,
            validated.address,
            validated.port,
            validated.tls_server_name,
            validated.target_uri,
        )

    @classmethod
    def https(
        cls,
        uri: str,
        *,
        tls_server_name: str | None = None,
        target_uri: str | None = None,
    ) -> Self:
        """Create HTTPS with an optional TLS name for an IP endpoint."""
        validated = Endpoint(
            EndpointKind.HTTPS, uri, tls_server_name=tls_server_name, target_uri=target_uri
        )
        return cls(
            validated.kind,
            validated.address,
            validated.port,
            validated.tls_server_name,
            validated.target_uri,
        )


class _SecretBytes:
    """Mutable owned bytes that are wiped before their backing storage is released."""

    __slots__ = ("_buffer", "__weakref__")

    def __init__(self, source: bytes | bytearray | memoryview) -> None:
        self._buffer = bytearray(source)

    @property
    def buffer(self) -> bytearray:
        """Return the privately owned mutable storage for internal FFI borrowing."""
        return self._buffer

    def clone(self) -> _SecretBytes:
        return _SecretBytes(self._buffer)

    def to_bytes(self) -> bytes:
        return bytes(self._buffer)

    def wipe(self) -> None:
        memoryview(self._buffer)[:] = bytes(len(self._buffer))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, _SecretBytes):
            return NotImplemented
        return self._buffer == other._buffer

    def __del__(self) -> None:
        try:
            self.wipe()
        except Exception:
            # Destructors must not surface failures during interpreter shutdown.
            pass


@dataclass(frozen=True, slots=True, init=False, eq=False)
class TransportInput:
    """Credential or trust material supplied as bytes or a local file path."""

    source: bytes | Path = field(repr=False)
    encoding: CredentialEncoding

    def __init__(self, source: bytes | Path, encoding: CredentialEncoding) -> None:
        if not isinstance(encoding, CredentialEncoding):
            errors.raise_for_status(4)
        if isinstance(source, bytes):
            try:
                view = memoryview(source)
                if view.nbytes > _MAX_INPUT_BYTES:
                    errors.raise_for_status(6)
                owned_source: _SecretBytes | Path = _SecretBytes(view)
            except (BufferError, TypeError, ValueError):
                errors.raise_for_status(4)
        elif isinstance(source, Path):
            try:
                path_bytes = _utf8_bytes(str(source))
            except (TypeError, ValueError, UnicodeEncodeError):
                errors.raise_for_status(4)
            if (
                not path_bytes
                or b"\x00" in path_bytes
                or len(path_bytes) > _MAX_TEXT_BYTES
            ):
                errors.raise_for_status(4)
            owned_source = Path(path_bytes.decode("utf-8"))
        else:
            errors.raise_for_status(4)
        object.__setattr__(self, "source", owned_source)
        object.__setattr__(self, "encoding", encoding)
        _validate_transport_input(self)

    @classmethod
    def _from_secret(cls, source: _SecretBytes, encoding: CredentialEncoding) -> Self:
        value = object.__new__(cls)
        object.__setattr__(value, "source", source)
        object.__setattr__(value, "encoding", encoding)
        _validate_transport_input(value)
        return value

    def __getattribute__(self, name: str) -> Any:
        if name == "source":
            source = TransportInput.source.__get__(self, TransportInput)
            if isinstance(source, _SecretBytes):
                return source.to_bytes()
            return source
        return object.__getattribute__(self, name)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TransportInput):
            return NotImplemented
        return (
            self.encoding == other.encoding
            and _transport_source(self) == _transport_source(other)
        )

    def __hash__(self) -> int:
        source = _transport_source(self)
        if isinstance(source, _SecretBytes):
            digest = hashlib.blake2b(source.buffer, digest_size=16).digest()
            return hash((digest, self.encoding))
        return hash((source, self.encoding))

    @classmethod
    def memory(
        cls, data: bytes | bytearray | memoryview, encoding: CredentialEncoding
    ) -> Self:
        """Copy one in-memory credential input using its explicit encoding."""
        if not isinstance(encoding, CredentialEncoding):
            errors.raise_for_status(4)
        if not isinstance(data, (bytes, bytearray, memoryview)):
            errors.raise_for_status(4)
        try:
            view = memoryview(data)
            if view.nbytes > _MAX_INPUT_BYTES:
                errors.raise_for_status(6)
        except (TypeError, ValueError):
            errors.raise_for_status(4)
        if not view.nbytes:
            errors.raise_for_status(4)
        return cls._from_secret(_SecretBytes(view), encoding)

    @classmethod
    def file(cls, path: str | Path, encoding: CredentialEncoding) -> Self:
        """Use a local file as credential input without reading it in Python."""
        if not isinstance(encoding, CredentialEncoding):
            errors.raise_for_status(4)
        try:
            path_value = Path(path)
            path_text = str(path_value)
            path_bytes = _utf8_bytes(path_text)
        except (TypeError, ValueError, UnicodeEncodeError):
            errors.raise_for_status(4)
        if not path_bytes or b"\x00" in path_bytes or len(path_bytes) > _MAX_TEXT_BYTES:
            errors.raise_for_status(4)
        return cls(path_value, encoding)


def _validate_transport_input(value: TransportInput) -> None:
    if not isinstance(value, TransportInput):
        errors.raise_for_status(4)
    if not isinstance(value.encoding, CredentialEncoding):
        errors.raise_for_status(4)
    source = _transport_source(value)
    if isinstance(source, _SecretBytes):
        byte_count = len(source.buffer)
        if byte_count == 0:
            errors.raise_for_status(4)
        if byte_count > _MAX_INPUT_BYTES:
            errors.raise_for_status(6)
        return
    if not isinstance(source, Path):
        errors.raise_for_status(4)
    try:
        path_text = str(source)
        path_bytes = _utf8_bytes(path_text)
    except (TypeError, ValueError, UnicodeEncodeError):
        errors.raise_for_status(4)
    if not path_bytes or b"\x00" in path_bytes or len(path_bytes) > _MAX_TEXT_BYTES:
        errors.raise_for_status(4)


def _transport_source(value: TransportInput) -> _SecretBytes | Path:
    """Read the canonical source slot without producing a public bytes copy."""
    return TransportInput.source.__get__(value, TransportInput)


def _validate_transport_input_group(values: tuple[TransportInput, ...]) -> None:
    if len(values) > _MAX_TRANSPORT_CONFIG_ITEMS:
        errors.raise_for_status(6)
    total = 0
    for value in values:
        if not isinstance(value, TransportInput):
            errors.raise_for_status(4)
        _validate_transport_input(value)
        source = _transport_source(value)
        if isinstance(source, _SecretBytes):
            total += len(source.buffer)
            if total > _MAX_INPUT_BYTES:
                errors.raise_for_status(6)


def _bounded_transport_inputs(values: object) -> tuple[TransportInput, ...]:
    if isinstance(values, tuple):
        count = tuple.__len__(values)
        if count > _MAX_TRANSPORT_CONFIG_ITEMS:
            errors.raise_for_status(6)
        if type(values) is tuple:
            return values
        return tuple(tuple.__iter__(values))
    if isinstance(values, list):
        if list.__len__(values) > _MAX_TRANSPORT_CONFIG_ITEMS:
            errors.raise_for_status(6)
        return tuple(list.__iter__(values))
    errors.raise_for_status(4)


@dataclass(frozen=True, slots=True)
class TrustSource:
    """An explicit platform-root or caller-supplied certificate trust source."""

    kind: TrustKind
    certificates: tuple[TransportInput, ...] = field(default=(), repr=False)

    def __post_init__(self) -> None:
        if not isinstance(self.kind, TrustKind):
            errors.raise_for_status(4)
        certificates = _bounded_transport_inputs(self.certificates)
        object.__setattr__(self, "certificates", certificates)
        if self.kind is TrustKind.PLATFORM:
            if certificates:
                errors.raise_for_status(4)
            return
        if self.kind is not TrustKind.CERTIFICATES or not certificates:
            errors.raise_for_status(4)
        _validate_transport_input_group(certificates)

    @classmethod
    def platform(cls) -> Self:
        """Use the platform trust store; certificate-chain validation stays enabled."""
        return cls(TrustKind.PLATFORM)

    @classmethod
    def certificate_authorities(
        cls, certificates: tuple[TransportInput, ...] | list[TransportInput]
    ) -> Self:
        """Use only the explicit server certificate authorities supplied by the caller."""
        values = _bounded_transport_inputs(certificates)
        if not values or any(not isinstance(item, TransportInput) for item in values):
            errors.raise_for_status(4)
        return cls(TrustKind.CERTIFICATES, values)


@dataclass(frozen=True, slots=True)
class TimeoutLimit:
    """One connect, write, read, or total timeout selection."""

    kind: TimeoutKind = TimeoutKind.DEFAULT
    seconds: int = 0
    nanoseconds: int = 0

    def __post_init__(self) -> None:
        if not isinstance(self.kind, TimeoutKind):
            errors.raise_for_status(4)
        seconds = _unsigned(self.seconds, 64)
        nanoseconds = _unsigned(self.nanoseconds, 32)
        object.__setattr__(self, "seconds", seconds)
        object.__setattr__(self, "nanoseconds", nanoseconds)
        if self.kind is TimeoutKind.DEFAULT:
            if seconds != 0 or nanoseconds != 0:
                errors.raise_for_status(4)
        elif self.kind is TimeoutKind.UNBOUNDED:
            if seconds != 0 or nanoseconds != 0:
                errors.raise_for_status(4)
        elif nanoseconds < 1_000_000_000:
            return
        else:
            errors.raise_for_status(4)

    @classmethod
    def default(cls) -> Self:
        """Select the canonical transport timeout default."""
        return cls()

    @classmethod
    def bounded(cls, seconds: int, nanoseconds: int = 0) -> Self:
        """Select a bounded timeout without converting through floating point."""
        return cls(TimeoutKind.BOUNDED, seconds, nanoseconds)

    @classmethod
    def unbounded(cls) -> Self:
        """Disable this individual timeout bound."""
        return cls(TimeoutKind.UNBOUNDED)


@dataclass(frozen=True, slots=True)
class RequestOptions:
    """Optional connect, write, read, and total timeout overrides for one exchange."""

    connect_timeout: TimeoutLimit = TimeoutLimit()
    write_timeout: TimeoutLimit = TimeoutLimit()
    read_timeout: TimeoutLimit = TimeoutLimit()
    total_timeout: TimeoutLimit = TimeoutLimit()

    def __post_init__(self) -> None:
        if any(
            not isinstance(value, TimeoutLimit)
            for value in (
                self.connect_timeout,
                self.write_timeout,
                self.read_timeout,
                self.total_timeout,
            )
        ):
            errors.raise_for_status(4)


def _validated_request_options(options: RequestOptions) -> RequestOptions:
    """Copy the approved dataclass fields so subclasses cannot bypass validation."""
    if not isinstance(options, RequestOptions):
        errors.raise_for_status(4)

    fields = (
        RequestOptions.connect_timeout.__get__(options, RequestOptions),
        RequestOptions.write_timeout.__get__(options, RequestOptions),
        RequestOptions.read_timeout.__get__(options, RequestOptions),
        RequestOptions.total_timeout.__get__(options, RequestOptions),
    )
    normalized: list[TimeoutLimit] = []
    for value in fields:
        if not isinstance(value, TimeoutLimit):
            errors.raise_for_status(4)
        normalized.append(
            TimeoutLimit(
                TimeoutLimit.kind.__get__(value, TimeoutLimit),
                TimeoutLimit.seconds.__get__(value, TimeoutLimit),
                TimeoutLimit.nanoseconds.__get__(value, TimeoutLimit),
            )
        )
    return RequestOptions(*normalized)


def _request_options_record(options: RequestOptions) -> Any:
    """Build the versioned FFI record while retaining explicit timeout kinds."""
    validated = _validated_request_options(options)
    record = ffi.new("kmipkit_request_options_v1_t *")
    record.struct_size = ffi.sizeof("kmipkit_request_options_v1_t")
    record.abi_version = 1
    for name, value in (
        ("connect_timeout", validated.connect_timeout),
        ("write_timeout", validated.write_timeout),
        ("read_timeout", validated.read_timeout),
        ("total_timeout", validated.total_timeout),
    ):
        target = getattr(record, name)
        target.kind = int(value.kind)
        target.reserved = 0
        target.seconds = value.seconds
        target.nanoseconds = value.nanoseconds
        target.reserved2 = 0
    return record


@dataclass(frozen=True, slots=True)
class TransportConfig:
    """Validated-input description for TLS 1.3 mutual-TLS client construction."""

    endpoint: Endpoint = field(repr=False)
    client_certificates: tuple[TransportInput, ...] = field(repr=False)
    client_private_key: TransportInput = field(repr=False)
    trust: TrustSource = field(repr=False)
    revocation_lists: tuple[TransportInput, ...] = field(default=(), repr=False)
    connect_timeout: TimeoutLimit = TimeoutLimit()
    write_timeout: TimeoutLimit = TimeoutLimit()
    read_timeout: TimeoutLimit = TimeoutLimit()
    total_timeout: TimeoutLimit = TimeoutLimit()
    max_request_bytes: int = 16 * 1024 * 1024

    def __post_init__(self) -> None:
        if not isinstance(self.endpoint, Endpoint):
            errors.raise_for_status(4)
        # Read the base dataclass slots and reconstruct a validated value so a
        # subclass cannot suppress Endpoint.__post_init__ and bypass FFI checks.
        endpoint = Endpoint(
            Endpoint.kind.__get__(self.endpoint, Endpoint),
            Endpoint.address.__get__(self.endpoint, Endpoint),
            Endpoint.port.__get__(self.endpoint, Endpoint),
            Endpoint.tls_server_name.__get__(self.endpoint, Endpoint),
            Endpoint.target_uri.__get__(self.endpoint, Endpoint),
        )
        object.__setattr__(self, "endpoint", endpoint)
        _validate_transport_input(self.client_private_key)
        if not isinstance(self.trust, TrustSource):
            errors.raise_for_status(4)
        client_certificates = _bounded_transport_inputs(self.client_certificates)
        object.__setattr__(self, "client_certificates", client_certificates)
        if not self.client_certificates or any(
            not isinstance(item, TransportInput) for item in self.client_certificates
        ):
            errors.raise_for_status(4)
        _validate_transport_input_group(self.client_certificates)
        revocation_lists = _bounded_transport_inputs(self.revocation_lists)
        object.__setattr__(self, "revocation_lists", revocation_lists)
        if any(not isinstance(item, TransportInput) for item in self.revocation_lists):
            errors.raise_for_status(4)
        if any(
            isinstance(_transport_source(item), Path) for item in self.revocation_lists
        ):
            errors.raise_for_status(4)
        _validate_transport_input_group(self.revocation_lists)
        for timeout in (
            self.connect_timeout,
            self.write_timeout,
            self.read_timeout,
            self.total_timeout,
        ):
            if not isinstance(timeout, TimeoutLimit):
                errors.raise_for_status(4)
        _unsigned(self.max_request_bytes, 64, allow_zero=False)


def _snapshot_endpoint(value: Endpoint) -> Endpoint:
    if not isinstance(value, Endpoint):
        errors.raise_for_status(4)
    # Base slot access and construction prevent subclass hooks from skipping validation.
    try:
        return Endpoint(
            Endpoint.kind.__get__(value, Endpoint),
            Endpoint.address.__get__(value, Endpoint),
            Endpoint.port.__get__(value, Endpoint),
            Endpoint.tls_server_name.__get__(value, Endpoint),
            Endpoint.target_uri.__get__(value, Endpoint),
        )
    except (AttributeError, TypeError):
        errors.raise_for_status(4)


def _snapshot_transport_input(value: TransportInput) -> TransportInput:
    if not isinstance(value, TransportInput):
        errors.raise_for_status(4)
    # Reconstruct through the base dataclass so its input checks always run.
    try:
        source = _transport_source(value)
        encoding = TransportInput.encoding.__get__(value, TransportInput)
    except (AttributeError, TypeError):
        errors.raise_for_status(4)
    if isinstance(source, _SecretBytes):
        return TransportInput._from_secret(source.clone(), encoding)
    return TransportInput(source, encoding)


def _snapshot_transport_inputs(values: object) -> tuple[TransportInput, ...]:
    bounded_values = _bounded_transport_inputs(values)
    return tuple(_snapshot_transport_input(value) for value in bounded_values)


def _snapshot_trust_source(value: TrustSource) -> TrustSource:
    if not isinstance(value, TrustSource):
        errors.raise_for_status(4)
    # TrustSource subclasses can also replace post-init validation.
    try:
        kind = TrustSource.kind.__get__(value, TrustSource)
        certificates = TrustSource.certificates.__get__(value, TrustSource)
    except (AttributeError, TypeError):
        errors.raise_for_status(4)
    return TrustSource(kind, _snapshot_transport_inputs(certificates))


def _snapshot_timeout(value: TimeoutLimit) -> TimeoutLimit:
    if not isinstance(value, TimeoutLimit):
        errors.raise_for_status(4)
    # Copy the declared slots rather than reading potentially overridden properties.
    try:
        kind = TimeoutLimit.kind.__get__(value, TimeoutLimit)
        seconds = TimeoutLimit.seconds.__get__(value, TimeoutLimit)
        nanoseconds = TimeoutLimit.nanoseconds.__get__(value, TimeoutLimit)
    except (AttributeError, TypeError):
        errors.raise_for_status(4)
    return TimeoutLimit(kind, seconds, nanoseconds)


def _validated_transport_config(value: TransportConfig) -> TransportConfig:
    """Rebuild config records through base constructors at the FFI boundary."""
    if not isinstance(value, TransportConfig):
        errors.raise_for_status(4)
    # Direct slot reads make the values sent to the ABI the values revalidated here.
    try:
        endpoint = TransportConfig.endpoint.__get__(value, TransportConfig)
        certificates = TransportConfig.client_certificates.__get__(
            value, TransportConfig
        )
        private_key = TransportConfig.client_private_key.__get__(value, TransportConfig)
        trust = TransportConfig.trust.__get__(value, TransportConfig)
        revocations = TransportConfig.revocation_lists.__get__(value, TransportConfig)
        connect_timeout = TransportConfig.connect_timeout.__get__(
            value, TransportConfig
        )
        write_timeout = TransportConfig.write_timeout.__get__(value, TransportConfig)
        read_timeout = TransportConfig.read_timeout.__get__(value, TransportConfig)
        total_timeout = TransportConfig.total_timeout.__get__(value, TransportConfig)
        max_request_bytes = TransportConfig.max_request_bytes.__get__(
            value, TransportConfig
        )
    except (AttributeError, TypeError):
        errors.raise_for_status(4)
    return TransportConfig(
        endpoint=_snapshot_endpoint(endpoint),
        client_certificates=_snapshot_transport_inputs(certificates),
        client_private_key=_snapshot_transport_input(private_key),
        trust=_snapshot_trust_source(trust),
        revocation_lists=_snapshot_transport_inputs(revocations),
        connect_timeout=_snapshot_timeout(connect_timeout),
        write_timeout=_snapshot_timeout(write_timeout),
        read_timeout=_snapshot_timeout(read_timeout),
        total_timeout=_snapshot_timeout(total_timeout),
        max_request_bytes=max_request_bytes,
    )


class _ClientHandle(NativeHandle):
    """Native handle released through the private T076 CFFI library."""

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            if self.closed:
                return
            handle, owner = self._take()
            getattr(lib, self._release_name)(handle)
            del owner


class Batch(_ClientHandle):
    """One-shot, ordered typed request batch; supports deterministic cleanup."""

    _release_name = "kmipkit_client_batch_release"

    @classmethod
    def create(cls) -> Self:
        """Create an empty ordered batch builder."""
        output = ffi.new("kmipkit_client_batch_t **")
        _raise_abi_status(lib.kmipkit_client_batch_create(output))
        return _wrap_owned_handle(cls, output[0])

    def add_item(self, item: extensions.ClientBatchItem) -> Self:
        """Append one typed item, consuming its wrapper only when accepted."""
        if not isinstance(item, extensions.ClientBatchItem):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            batch_pointer = self._pointer()
            item._acquire_ownership_lock()
            try:
                original = item._pointer()
                slot = ffi.new(
                    "kmipkit_client_batch_item_t **",
                    ffi.cast("kmipkit_client_batch_item_t *", original),
                )
                original, owner = item._take()
                try:
                    status = lib.kmipkit_client_batch_add_item(batch_pointer, slot)
                except (OverflowError, TypeError, ValueError):
                    item._restore(original, owner)
                    raise errors.InvalidInputError() from None
                if slot[0] != ffi.NULL:
                    item._restore(original, owner)
                _raise_abi_status(status)
            finally:
                item._release_ownership_lock()
        return self

    def set_request_timestamp(self, timestamp: int) -> Self:
        """Set the caller-selected KMIP request timestamp in signed Unix seconds."""
        value = _integer_range(timestamp, _MIN_I64, _MAX_I64)
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_batch_set_request_time_stamp(self._pointer(), value)
            )
        return self

    def set_ordered(self, enabled: bool) -> Self:
        """Set the KMIP Batch Order Option explicitly."""
        if not isinstance(enabled, bool):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_batch_set_order_option(self._pointer(), int(enabled))
            )
        return self

    def add_error_continuation_value(self, value: int) -> Self:
        """Append a caller-selected KMIP Batch Error Continuation Option value."""
        raw = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_batch_add_error_continuation_value(
                    self._pointer(), raw
                )
            )
        return self

    def set_asynchronous_indicator(self, value: int) -> Self:
        """Set the caller-selected KMIP Asynchronous Indicator enumeration value."""
        raw = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_batch_set_asynchronous_indicator(self._pointer(), raw)
            )
        return self

    def set_client_correlation_value(self, value: str | bytes) -> Self:
        """Set exact UTF-8 correlation bytes for the outgoing request."""
        raw, data = _owned_bytes(value, maximum=_MAX_INPUT_BYTES)
        pointer = data if raw else ffi.NULL
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_batch_set_client_correlation_value(
                    self._pointer(), pointer, len(raw)
                )
            )
        return self


@dataclass(frozen=True, slots=True)
class BatchItemResult:
    """Owned, payload-free common metadata for one ordered batch response item."""

    outcome: BatchOutcome | int
    result_status: int
    result_reason: int | None
    unique_batch_item_id: bytes | None = field(repr=False)
    pending_correlation_value: bytes | None = field(repr=False)
    supported_versions: tuple[ProtocolVersion, ...] | None = None
    operation: int = 0
    result_message: str | None = field(default=None, repr=False)
    create_object_type: int | None = None
    unique_identifier: extensions.UniqueIdentifier | None = field(default=None, repr=False)
    private_key_unique_identifier: extensions.UniqueIdentifier | None = field(
        default=None, repr=False
    )
    public_key_unique_identifier: extensions.UniqueIdentifier | None = field(
        default=None, repr=False
    )
    split_key_unique_identifiers: tuple[extensions.UniqueIdentifier, ...] | None = field(
        default=None, repr=False
    )
    hash_response: HashResponse | None = field(default=None, repr=False)
    mac_response: MacResponse | None = field(default=None, repr=False)
    mac_verify_response: MacVerifyResponse | None = field(default=None, repr=False)
    sign_response: SignResponse | None = field(default=None, repr=False)
    signature_verify_response: SignatureVerifyResponse | None = field(default=None, repr=False)
    certify_response: CertifyResponse | None = field(default=None, repr=False)
    get_usage_allocation_response: GetUsageAllocationResponse | None = field(
        default=None, repr=False
    )

    @property
    def metadata_v2(self) -> BatchItemMetadataV2:
        """Return the versioned common metadata while preserving exact message presence."""
        return BatchItemMetadataV2(
            self.operation,
            self.outcome,
            self.result_status,
            self.result_reason,
            self.result_message,
        )


@dataclass(frozen=True, slots=True)
class HashResponse:
    """Typed Hash response fields copied from the matching successful batch item."""

    data: bytes | None = field(repr=False)
    correlation_value: bytes | None = field(repr=False)


@dataclass(frozen=True, slots=True)
class CertifyResponse:
    """Typed Certify response containing the generated certificate identifier."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)


@dataclass(frozen=True, slots=True)
class GetUsageAllocationResponse:
    """Successful Get Usage Allocation response and its owned complete generic TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True)
class MacResponse:
    """Typed MAC response fields; byte payloads are omitted from repr output."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    mac_data: bytes | None = field(repr=False)
    correlation_value: bytes | None = field(repr=False)


@dataclass(frozen=True, slots=True)
class MacVerifyResponse:
    """Typed MAC Verify result with a forward-compatible raw Validity Indicator."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    validity_indicator_raw: int | None
    correlation_value: bytes | None = field(repr=False)


@dataclass(frozen=True, slots=True)
class SignResponse:
    """Typed Sign response fields; signature bytes are omitted from repr output."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    signature_data: bytes | None = field(repr=False)
    correlation_value: bytes | None = field(repr=False)


@dataclass(frozen=True, slots=True)
class SignatureVerifyResponse:
    """Typed Signature Verify result preserving unknown raw Validity Indicator values."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    validity_indicator_raw: int | None
    recovered_data: bytes | None = field(repr=False)
    correlation_value: bytes | None = field(repr=False)


class BatchResult(_ClientHandle):
    """Owns a native batch response and exposes copied, ordered outcome metadata."""

    _release_name = "kmipkit_client_batch_response_release"

    def __init__(
        self,
        handle: Any,
        items: tuple[BatchItemResult, ...],
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token)
        self.items = items


class Client(_ClientHandle):
    """Production client constructed only from validated configuration records."""

    _release_name = "kmipkit_client_release"

    @classmethod
    def create(
        cls,
        configuration: extensions.ClientConfiguration,
        transport: TransportConfig,
    ) -> Self:
        """Construct a TLS client from separate extension and transport configuration."""
        if not isinstance(configuration, extensions.ClientConfiguration):
            errors.raise_for_status(4)
        if not isinstance(transport, TransportConfig):
            errors.raise_for_status(4)
        transport = _validated_transport_config(transport)
        owners: tuple[object, ...] = ()
        try:
            record, owners = _transport_record(transport)
            transport_output = ffi.new("kmipkit_transport_config_t **")
            client_output = ffi.new("kmipkit_client_t **")
            with _NATIVE_CALL_LOCK:
                try:
                    status = lib.kmipkit_transport_config_create_v1(
                        record, transport_output
                    )
                finally:
                    _wipe_client_transport_secrets(owners, transport)
                    owners = ()
                _raise_abi_status(status)
                transport_slot = ffi.new(
                    "kmipkit_transport_config_t **", transport_output[0]
                )
                try:
                    status = lib.kmipkit_client_create(
                        ffi.cast(
                            "kmipkit_client_configuration_t *",
                            configuration._pointer(),
                        ),
                        transport_slot,
                        client_output,
                    )
                except (OverflowError, TypeError, ValueError):
                    if transport_slot[0] != ffi.NULL:
                        lib.kmipkit_transport_config_release(transport_slot[0])
                    raise errors.InvalidInputError() from None
                if transport_slot[0] != ffi.NULL:
                    lib.kmipkit_transport_config_release(transport_slot[0])
                _raise_abi_status(status)
        finally:
            _wipe_client_transport_secrets(owners, transport)
        if client_output[0] == ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(cls, client_output[0])

    def execute(
        self, batch: Batch, limits: ttlv.CodecLimits
    ) -> BatchResult:
        """Execute exactly one typed batch and copy ordered common result metadata."""
        if not isinstance(batch, Batch) or not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        response_output = ffi.new("kmipkit_client_batch_response_t **")
        error_response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
        with _NATIVE_CALL_LOCK:
            batch._acquire_ownership_lock()
            try:
                client_pointer = self._pointer()
                limits_pointer = ffi.cast(
                    "kmipkit_codec_limits_t *", limits._pointer()
                )
                original = batch._pointer()
                batch_slot = ffi.new("kmipkit_client_batch_t **", original)
                original, owner = batch._take()
                try:
                    status = lib.kmipkit_client_execute_with_error_ttlv_v1(
                        client_pointer,
                        batch_slot,
                        limits_pointer,
                        response_output,
                        error_response_ttlv_output,
                    )
                except (OverflowError, TypeError, ValueError):
                    batch._restore(original, owner)
                    raise errors.InvalidInputError() from None
                if batch_slot[0] != ffi.NULL:
                    batch._restore(original, owner)
                _raise_client_status(status, error_response_ttlv_output[0])
            finally:
                batch._release_ownership_lock()
            if response_output[0] == ffi.NULL:
                errors.raise_for_status(7)
            return _batch_result(response_output[0])

    def execute_with_options(
        self,
        batch: Batch,
        limits: ttlv.CodecLimits,
        options: RequestOptions,
    ) -> BatchResult:
        """Execute one batch with exchange-local timeout overrides."""
        if (
            not isinstance(batch, Batch)
            or not isinstance(limits, ttlv.CodecLimits)
            or not isinstance(options, RequestOptions)
        ):
            errors.raise_for_status(4)
        request_options = _request_options_record(options)
        response_output = ffi.new("kmipkit_client_batch_response_t **")
        error_response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
        with _NATIVE_CALL_LOCK:
            batch._acquire_ownership_lock()
            try:
                client_pointer = self._pointer()
                limits_pointer = ffi.cast(
                    "kmipkit_codec_limits_t *", limits._pointer()
                )
                original = batch._pointer()
                batch_slot = ffi.new("kmipkit_client_batch_t **", original)
                original, owner = batch._take()
                try:
                    status = lib.kmipkit_client_execute_with_options_and_error_ttlv_v1(
                        client_pointer,
                        batch_slot,
                        limits_pointer,
                        request_options,
                        response_output,
                        error_response_ttlv_output,
                    )
                except (OverflowError, TypeError, ValueError):
                    batch._restore(original, owner)
                    raise errors.InvalidInputError() from None
                if batch_slot[0] != ffi.NULL:
                    batch._restore(original, owner)
                _raise_client_status(status, error_response_ttlv_output[0])
            finally:
                batch._release_ownership_lock()
            if response_output[0] == ffi.NULL:
                errors.raise_for_status(7)
            return _batch_result(response_output[0])


def _batch_result(handle: Any) -> BatchResult:
    response = _wrap_owned_handle(BatchResult, handle, ())
    items: list[BatchItemResult] = []
    try:
        count = ffi.new("uint64_t *")
        _raise_abi_status(
            lib.kmipkit_client_batch_response_item_count(response._pointer(), count)
        )
        if count[0] > _MAX_BATCH_ITEMS:
            errors.raise_for_status(6)
        for index in range(int(count[0])):
            item_output = ffi.new("kmipkit_client_batch_item_response_t **")
            _raise_abi_status(
                lib.kmipkit_client_batch_response_item_at(
                    response._pointer(), index, item_output
                )
            )
            if item_output[0] == ffi.NULL:
                errors.raise_for_status(7)
            item = _wrap_owned_handle(_ClientBatchItemResponse, item_output[0])
            try:
                metadata = ffi.new("kmipkit_client_batch_item_metadata_v2_t *")
                metadata.struct_size = ffi.sizeof(
                    "kmipkit_client_batch_item_metadata_v2_t"
                )
                metadata.abi_version = 2
                _raise_abi_status(
                    lib.kmipkit_client_batch_item_response_metadata_v2(
                        item._pointer(), metadata
                    )
                )
                if metadata.abi_version != 2 or metadata.struct_size != ffi.sizeof(
                    "kmipkit_client_batch_item_metadata_v2_t"
                ):
                    errors.raise_for_status(4)
                if metadata.reserved != 0 or metadata.has_result_message not in (0, 1):
                    errors.raise_for_status(4)
                try:
                    outcome: BatchOutcome | int = BatchOutcome(metadata.outcome_kind)
                except ValueError:
                    outcome = int(metadata.outcome_kind)
                if metadata.has_result_reason not in (0, 1):
                    errors.raise_for_status(4)
                result_reason = (
                    int(metadata.result_reason)
                    if metadata.has_result_reason == 1
                    else None
                )
                message_bytes = _metadata_result_message(metadata)
                try:
                    result_message = (
                        None if message_bytes is None else message_bytes.decode("utf-8", errors="strict")
                    )
                except UnicodeDecodeError:
                    errors.raise_for_status(4)
                unique_id = _response_span(
                    item, lib.kmipkit_client_batch_item_response_unique_batch_item_id
                )
                correlation = _response_span(
                    item,
                    lib.kmipkit_client_batch_item_response_pending_correlation_value,
                )
                supported_versions = _discover_versions_supported_versions(item)
                create_object_type, create_identifier = _create_response(item)
                private_identifier, public_identifier = _create_key_pair_response(item)
                split_identifiers = _create_split_key_response(item)
                certify_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x06
                ):
                    certify_response = _certify_response(item)
                allocation_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x11
                ):
                    allocation_response = _get_usage_allocation_response(item)
                cryptographic_responses = (None, None, None, None, None)
                if outcome == BatchOutcome.COMPLETED and int(metadata.result_status) == 0:
                    cryptographic_responses = _cryptographic_responses(
                        item, int(metadata.operation)
                    )
                items.append(
                    BatchItemResult(
                        outcome,
                        int(metadata.result_status),
                        result_reason,
                        unique_id,
                        correlation,
                        supported_versions,
                        int(metadata.operation),
                        result_message,
                        create_object_type,
                        create_identifier,
                        private_identifier,
                        public_identifier,
                        split_identifiers,
                        *cryptographic_responses,
                        certify_response,
                        allocation_response,
                    )
                )
            finally:
                item.close()
        response.items = tuple(items)
        return response
    except Exception:
        response.close()
        raise


class _ClientBatchItemResponse(_ClientHandle):
    _release_name = "kmipkit_client_batch_item_response_release"


def _metadata_result_message(metadata: Any) -> bytes | None:
    if metadata.has_result_message == 0:
        if metadata.result_message_data != ffi.NULL or metadata.result_message_length != 0:
            errors.raise_for_status(4)
        return None
    if metadata.result_message_length > _MAX_INPUT_BYTES:
        errors.raise_for_status(6)
    if metadata.result_message_length != 0 and metadata.result_message_data == ffi.NULL:
        errors.raise_for_status(4)
    return (
        bytes(ffi.buffer(metadata.result_message_data, int(metadata.result_message_length)))
        if metadata.result_message_length
        else b""
    )


def _initialize_unique_identifier_output() -> Any:
    output = ffi.new("kmipkit_unique_identifier_v1_t *")
    output.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    output.abi_version = 1
    return output


def _read_unique_identifier(output: Any) -> extensions.UniqueIdentifier:
    if (
        output.struct_size != ffi.sizeof("kmipkit_unique_identifier_v1_t")
        or output.abi_version != 1
        or output.reserved != 0
    ):
        errors.raise_for_status(4)
    kind = int(output.kind)
    if kind == int(extensions.UniqueIdentifierKind.TEXT_STRING):
        if output.text_length > _MAX_INPUT_BYTES or (
            output.text_length != 0 and output.text_data == ffi.NULL
        ):
            errors.raise_for_status(4)
        raw = (
            bytes(ffi.buffer(output.text_data, int(output.text_length)))
            if output.text_length
            else b""
        )
        try:
            return extensions.UniqueIdentifier.text_string(raw.decode("utf-8", errors="strict"))
        except UnicodeDecodeError:
            errors.raise_for_status(4)
    if kind == int(extensions.UniqueIdentifierKind.ENUMERATION):
        if output.text_data != ffi.NULL or output.text_length != 0 or output.integer_value != 0:
            errors.raise_for_status(4)
        return extensions.UniqueIdentifier.enumeration(int(output.enumeration_value))
    if kind == int(extensions.UniqueIdentifierKind.INTEGER):
        if (
            output.text_data != ffi.NULL
            or output.text_length != 0
            or output.enumeration_value != 0
        ):
            errors.raise_for_status(4)
        return extensions.UniqueIdentifier.integer(int(output.integer_value))
    errors.raise_for_status(4)


def _create_response(
    item: _ClientBatchItemResponse,
) -> tuple[int | None, extensions.UniqueIdentifier | None]:
    present = ffi.new("uint8_t *")
    object_type = ffi.new("uint32_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_create(
            item._pointer(), present, object_type, identifier
        )
    )
    if present[0] == 0:
        return None, None
    if present[0] != 1:
        errors.raise_for_status(4)
    return int(object_type[0]), _read_unique_identifier(identifier)


def _create_key_pair_response(
    item: _ClientBatchItemResponse,
) -> tuple[extensions.UniqueIdentifier | None, extensions.UniqueIdentifier | None]:
    present = ffi.new("uint8_t *")
    private_identifier = _initialize_unique_identifier_output()
    public_identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_create_key_pair(
            item._pointer(), present, private_identifier, public_identifier
        )
    )
    if present[0] == 0:
        return None, None
    if present[0] != 1:
        errors.raise_for_status(4)
    return _read_unique_identifier(private_identifier), _read_unique_identifier(public_identifier)


def _create_split_key_response(
    item: _ClientBatchItemResponse,
) -> tuple[extensions.UniqueIdentifier, ...] | None:
    present = ffi.new("uint8_t *")
    count = ffi.new("uint64_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_create_split_key_count(item._pointer(), present, count)
    )
    if present[0] == 0:
        if count[0] != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1 or count[0] < 1:
        errors.raise_for_status(4)
    if count[0] > sys.maxsize:
        errors.raise_for_status(6)
    identifiers: list[extensions.UniqueIdentifier] = []
    for index in range(int(count[0])):
        output = _initialize_unique_identifier_output()
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                item._pointer(), index, output
            )
        )
        identifiers.append(_read_unique_identifier(output))
    return tuple(identifiers)


def _optional_output_span(present: Any, value: Any) -> bytes | None:
    if present[0] == 0:
        if value.data != ffi.NULL or value.length != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1 or value.length > _MAX_INPUT_BYTES:
        errors.raise_for_status(4)
    if value.length and value.data == ffi.NULL:
        errors.raise_for_status(4)
    return bytes(ffi.buffer(value.data, int(value.length))) if value.length else b""


def _optional_output_identifier(value: Any) -> extensions.UniqueIdentifier | None:
    if value.kind == 0:
        if value.text_data != ffi.NULL or value.text_length != 0:
            errors.raise_for_status(4)
        return None
    return _read_unique_identifier(value)


def _hash_response(item: _ClientBatchItemResponse) -> HashResponse | None:
    present = ffi.new("uint8_t *")
    data_present = ffi.new("uint8_t *")
    data = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_hash(
            item._pointer(), present, data_present, data, correlation_present, correlation
        )
    )
    if present[0] == 0:
        if data_present[0] != 0 or correlation_present[0] != 0:
            errors.raise_for_status(4)
        _optional_output_span(data_present, data)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    return HashResponse(
        _optional_output_span(data_present, data),
        _optional_output_span(correlation_present, correlation),
    )


def _certify_response(item: _ClientBatchItemResponse) -> CertifyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_certify(item._pointer(), present, identifier)
    )
    if present[0] == 0:
        if identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    return CertifyResponse(unique_identifier)


def _get_usage_allocation_response(
    item: _ClientBatchItemResponse,
) -> GetUsageAllocationResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_usage_allocation(
            item._pointer(), present, identifier
        )
    )
    if present[0] == 0:
        if identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            item._pointer(), response_ttlv_present, response_ttlv_output
        )
    )
    response_ttlv_handle = response_ttlv_output[0]
    if response_ttlv_present[0] != 1 or response_ttlv_handle == ffi.NULL:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        errors.raise_for_status(4)
    try:
        response_ttlv = ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", response_ttlv_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        raise
    return GetUsageAllocationResponse(unique_identifier, response_ttlv)


def _mac_response(item: _ClientBatchItemResponse) -> MacResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    mac_present = ffi.new("uint8_t *")
    mac_data = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_mac(
            item._pointer(), present, identifier, mac_present, mac_data,
            correlation_present, correlation,
        )
    )
    if present[0] == 0:
        if mac_present[0] != 0 or correlation_present[0] != 0 or identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_span(mac_present, mac_data)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    return MacResponse(
        _optional_output_identifier(identifier),
        _optional_output_span(mac_present, mac_data),
        _optional_output_span(correlation_present, correlation),
    )


def _mac_verify_response(item: _ClientBatchItemResponse) -> MacVerifyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    validity_present = ffi.new("uint8_t *")
    validity = ffi.new("uint32_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_mac_verify(
            item._pointer(), present, identifier, validity_present, validity,
            correlation_present, correlation,
        )
    )
    if present[0] == 0:
        if validity_present[0] != 0 or validity[0] != 0 or correlation_present[0] != 0 or identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1 or validity_present[0] not in (0, 1):
        errors.raise_for_status(4)
    return MacVerifyResponse(
        _optional_output_identifier(identifier),
        int(validity[0]) if validity_present[0] == 1 else None,
        _optional_output_span(correlation_present, correlation),
    )


def _sign_response(item: _ClientBatchItemResponse) -> SignResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    signature_present = ffi.new("uint8_t *")
    signature_data = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_sign(
            item._pointer(), present, identifier, signature_present, signature_data,
            correlation_present, correlation,
        )
    )
    if present[0] == 0:
        if signature_present[0] != 0 or correlation_present[0] != 0 or identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_span(signature_present, signature_data)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    return SignResponse(
        _optional_output_identifier(identifier),
        _optional_output_span(signature_present, signature_data),
        _optional_output_span(correlation_present, correlation),
    )


def _signature_verify_response(
    item: _ClientBatchItemResponse,
) -> SignatureVerifyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    validity_present = ffi.new("uint8_t *")
    validity = ffi.new("uint32_t *")
    recovered_present = ffi.new("uint8_t *")
    recovered_data = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_signature_verify(
            item._pointer(), present, identifier, validity_present, validity,
            recovered_present, recovered_data, correlation_present, correlation,
        )
    )
    if present[0] == 0:
        if (
            validity_present[0] != 0
            or validity[0] != 0
            or recovered_present[0] != 0
            or correlation_present[0] != 0
            or identifier.kind != 0
        ):
            errors.raise_for_status(4)
        _optional_output_span(recovered_present, recovered_data)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1 or validity_present[0] not in (0, 1):
        errors.raise_for_status(4)
    return SignatureVerifyResponse(
        _optional_output_identifier(identifier),
        int(validity[0]) if validity_present[0] == 1 else None,
        _optional_output_span(recovered_present, recovered_data),
        _optional_output_span(correlation_present, correlation),
    )


def _cryptographic_responses(
    item: _ClientBatchItemResponse, operation: int
) -> tuple[
    HashResponse | None,
    MacResponse | None,
    MacVerifyResponse | None,
    SignResponse | None,
    SignatureVerifyResponse | None,
]:
    if operation == 0x27:
        return _hash_response(item), None, None, None, None
    if operation == 0x23:
        return None, _mac_response(item), None, None, None
    if operation == 0x24:
        return None, None, _mac_verify_response(item), None, None
    if operation == 0x21:
        return None, None, None, _sign_response(item), None
    if operation == 0x22:
        return None, None, None, None, _signature_verify_response(item)
    return None, None, None, None, None


def _discover_versions_supported_versions(
    item: _ClientBatchItemResponse,
) -> tuple[ProtocolVersion, ...] | None:
    function = lib.kmipkit_client_batch_item_response_discover_versions_supported_versions
    present = ffi.new("uint8_t *")
    count = ffi.new("uint64_t *")
    _raise_abi_status(function(item._pointer(), present, ffi.NULL, 0, count))
    if present[0] == 0:
        if count[0] != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    required = int(count[0])
    if required > sys.maxsize // ffi.sizeof("kmipkit_protocol_version_v1_t"):
        errors.raise_for_status(6)
    if required == 0:
        return ()

    values = ffi.new("kmipkit_protocol_version_v1_t[]", required)
    copied_present = ffi.new("uint8_t *")
    copied_count = ffi.new("uint64_t *")
    _raise_abi_status(
        function(item._pointer(), copied_present, values, required, copied_count)
    )
    if copied_present[0] != 1 or copied_count[0] != required:
        errors.raise_for_status(4)
    return tuple(
        ProtocolVersion(int(values[index].major), int(values[index].minor))
        for index in range(required)
    )


def _response_span(item: _ClientBatchItemResponse, function: Any) -> bytes | None:
    present = ffi.new("uint8_t *")
    value = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(function(item._pointer(), present, value))
    if present[0] == 0:
        if value.data != ffi.NULL or value.length != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    if value.length > _MAX_INPUT_BYTES:
        errors.raise_for_status(6)
    if value.length and value.data == ffi.NULL:
        errors.raise_for_status(4)
    return bytes(ffi.buffer(value.data, int(value.length))) if value.length else b""


def _transport_record(configuration: TransportConfig) -> tuple[Any, tuple[object, ...]]:
    """Build one bounded versioned C record and keep borrowed spans alive for the call."""
    owners: list[object] = []

    def span(value: bytes | _SecretBytes) -> Any:
        if isinstance(value, _SecretBytes):
            source = value.clone()
            byte_count = len(source.buffer)
            owners.append(source)
            data = ffi.from_buffer("uint8_t[]", source.buffer)
            owners.append(data)
        else:
            byte_count = memoryview(value).nbytes
            data = ffi.new("uint8_t[]", value) if byte_count else ffi.NULL
            if byte_count:
                owners.append(data)
        if byte_count == 0:
            return ffi.new("kmipkit_byte_span_v1_t *", {"data": ffi.NULL, "length": 0})[0]
        return ffi.new(
            "kmipkit_byte_span_v1_t *",
            {"data": data, "length": byte_count},
        )[0]

    def input_record(value: TransportInput) -> Any:
        _validate_transport_input(value)
        source = _transport_source(value)
        if isinstance(source, _SecretBytes):
            source_kind = 1
            data: bytes | _SecretBytes = source
        else:
            source_kind = 2
            data = _utf8_bytes(str(source))
            if not data or len(data) > _MAX_TEXT_BYTES:
                errors.raise_for_status(4)
        record = ffi.new("kmipkit_transport_input_v1_t *")
        record.source_kind = source_kind
        record.encoding = int(value.encoding)
        record.data = span(data)
        return record[0]

    def input_array(values: tuple[TransportInput, ...]) -> Any:
        if not values:
            return ffi.NULL
        _validate_transport_input_group(values)
        array = ffi.new("kmipkit_transport_input_v1_t[]", len(values))
        for index, value in enumerate(values):
            array[index] = input_record(value)
        owners.append(array)
        return array

    def timeout(value: TimeoutLimit) -> Any:
        return ffi.new(
            "kmipkit_timeout_limit_v1_t *",
            {
                "kind": int(value.kind),
                "reserved": 0,
                "seconds": value.seconds,
                "nanoseconds": value.nanoseconds,
                "reserved2": 0,
            },
        )[0]

    try:
        endpoint_bytes = _utf8_bytes(configuration.endpoint.address)
        endpoint = span(endpoint_bytes)
        certificates = input_array(configuration.client_certificates)
        key = input_record(configuration.client_private_key)
        trust_certificates = input_array(configuration.trust.certificates)
        revocation_lists = input_array(configuration.revocation_lists)
        flags = 0
        server_name = ffi.new(
            "kmipkit_byte_span_v1_t *", {"data": ffi.NULL, "length": 0}
        )[0]
        if configuration.endpoint.tls_server_name is not None:
            flags |= 1
            server_name = span(_utf8_bytes(configuration.endpoint.tls_server_name))
        target_uri = ffi.new(
            "kmipkit_byte_span_v1_t *", {"data": ffi.NULL, "length": 0}
        )[0]
        if configuration.endpoint.target_uri is not None:
            flags |= 2
            target_uri = span(_utf8_bytes(configuration.endpoint.target_uri))
        record = ffi.new("kmipkit_transport_config_v1_t *")
        record.struct_size = ffi.sizeof("kmipkit_transport_config_v1_t")
        record.abi_version = 1
        record.endpoint_kind = int(configuration.endpoint.kind)
        record.trust_kind = int(configuration.trust.kind)
        record.flags = flags
        record.reserved = 0
        record.endpoint = endpoint
        record.endpoint_port = configuration.endpoint.port
        record.reserved2 = 0
        record.client_certificates = certificates
        record.client_certificate_count = len(configuration.client_certificates)
        record.client_private_key = key
        record.trust_certificates = trust_certificates
        record.trust_certificate_count = len(configuration.trust.certificates)
        record.revocation_lists = revocation_lists
        record.revocation_list_count = len(configuration.revocation_lists)
        record.tls_server_name = server_name
        record.target_uri = target_uri
        record.connect_timeout = timeout(configuration.connect_timeout)
        record.write_timeout = timeout(configuration.write_timeout)
        record.read_timeout = timeout(configuration.read_timeout)
        record.total_timeout = timeout(configuration.total_timeout)
        record.max_request_bytes = configuration.max_request_bytes
        owners.append(record)
        return record, tuple(owners)
    except BaseException:
        _wipe_transport_secrets(owners)
        raise


def _wipe_transport_secrets(owners: tuple[object, ...] | list[object]) -> None:
    """Wipe every mutable credential owner borrowed by an ABI record."""
    for owner in owners:
        if isinstance(owner, _SecretBytes):
            owner.wipe()


def _wipe_client_transport_secrets(
    owners: tuple[object, ...] | list[object], configuration: TransportConfig
) -> None:
    """Wipe ABI spans and the per-call key snapshot after native copying."""
    _wipe_transport_secrets(owners)
    key_source = _transport_source(configuration.client_private_key)
    if isinstance(key_source, _SecretBytes):
        key_source.wipe()


def _read_last_error() -> tuple[int, int, int, int]:
    status = ffi.new("int32_t *")
    category = ffi.new("uint32_t *")
    cause = ffi.new("uint32_t *")
    delivery = ffi.new("uint32_t *")
    if lib.kmipkit_last_error_get(status, category, cause, delivery) != 0:
        return 7, 5, 10, 0
    return int(status[0]), int(category[0]), int(cause[0]), int(delivery[0])


def _raise_client_status(status: int, error_response_ttlv: Any = ffi.NULL) -> None:
    if status == 0:
        if error_response_ttlv != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(error_response_ttlv)
            errors.raise_for_status(7)
        return
    _status, category, cause, delivery = _read_last_error()
    response_ttlv = None
    if error_response_ttlv != ffi.NULL:
        if category == int(errors.ClientErrorCategory.PROTOCOL):
            address = int(ffi.cast("uintptr_t", error_response_ttlv))
            try:
                response_ttlv = ttlv._structure_from_native_address(address)
            except Exception:
                lib.kmipkit_ttlv_structure_release(error_response_ttlv)
                errors.raise_for_status(7)
        else:
            lib.kmipkit_ttlv_structure_release(error_response_ttlv)
    try:
        failure = errors.ClientExecutionError(
            status=status,
            category=_known_enum(errors.ClientErrorCategory, category),
            cause=_known_enum(errors.ClientErrorCause, cause),
            delivery=_known_enum(errors.DeliveryState, delivery),
            response_ttlv=response_ttlv,
        )
    except Exception:
        if response_ttlv is not None:
            response_ttlv.close()
        raise
    raise failure


def _known_enum(enum_type: type[IntEnum], value: int) -> IntEnum | int:
    try:
        return enum_type(value)
    except ValueError:
        return value


def _raise_abi_status(status: int) -> None:
    errors.raise_for_status(int(status))


def _utf8_bytes(value: str) -> bytes:
    if not isinstance(value, str):
        errors.raise_for_status(4)
    try:
        return str.encode(value, "utf-8")
    except UnicodeEncodeError:
        errors.raise_for_status(4)


def _nonempty_text(value: str) -> str:
    encoded = _utf8_bytes(value)
    if not encoded or b"\x00" in encoded:
        errors.raise_for_status(4)
    if len(encoded) > _MAX_TEXT_BYTES:
        errors.raise_for_status(6)
    return value


__all__ = [
    "Batch",
    "BatchItemMetadataV2",
    "BatchItemResult",
    "BatchOutcome",
    "BatchResult",
    "CertifyResponse",
    "Client",
    "CredentialEncoding",
    "Endpoint",
    "EndpointKind",
    "HashResponse",
    "GetUsageAllocationResponse",
    "MacResponse",
    "MacVerifyResponse",
    "RequestOptions",
    "SignResponse",
    "SignatureVerifyResponse",
    "TimeoutKind",
    "TimeoutLimit",
    "TransportConfig",
    "TransportInput",
    "TrustKind",
    "TrustSource",
]
