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
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any, Self
from urllib.parse import urlsplit

from . import errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import (
    _NATIVE_CALL_LOCK,
    NativeHandle,
    _integer_range,
    _owned_bytes,
    _wipe_staging_buffer,
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
_JOIN_SPLIT_KEY_OPERATION = 0x29


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


def _stage_correlation_value(
    value: bytes | bytearray | memoryview,
) -> tuple[bytes, Any]:
    if not isinstance(value, (bytes, bytearray, memoryview)):
        errors.raise_for_status(4)
    return _owned_bytes(value, maximum=_MAX_INPUT_BYTES)


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


@dataclass(frozen=True, slots=True, repr=False)
class AsyncOperationMetadataV1:
    """Common result fields returned by one asynchronous KMIP operation."""

    operation: int
    outcome: BatchOutcome | int
    result_status: int
    result_reason: int | None
    cancellation_result: int | None
    result_message: str | None = field(repr=False)

    def __repr__(self) -> str:
        return (
            "AsyncOperationMetadataV1(operation="
            f"{self.operation}, outcome={self.outcome!r}, "
            f"result_status={self.result_status}, "
            f"result_reason={self.result_reason!r}, "
            f"cancellation_result={self.cancellation_result!r}, "
            "result_message=redacted)"
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
        if self.cancellation_result is not None:
            object.__setattr__(
                self, "cancellation_result", _unsigned(self.cancellation_result, 32)
            )
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


class AsyncOperationOutcome(_ClientHandle):
    """Caller-owned result of one explicit asynchronous KMIP exchange."""

    _release_name = "kmipkit_client_async_operation_outcome_release"

    @property
    def metadata_v1(self) -> AsyncOperationMetadataV1:
        """Copy common result metadata while redacting Result Message in repr."""
        metadata = ffi.new("kmipkit_client_async_operation_metadata_v1_t *")
        metadata.struct_size = ffi.sizeof(
            "kmipkit_client_async_operation_metadata_v1_t"
        )
        metadata.abi_version = 1
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_async_operation_outcome_metadata_v1(
                    self._pointer(), metadata
                )
            )
            if (
                metadata.struct_size
                != ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
                or metadata.abi_version != 1
                or metadata.reserved != 0
                or metadata.has_result_reason not in (0, 1)
                or metadata.has_cancellation_result not in (0, 1)
                or metadata.has_result_message not in (0, 1)
            ):
                errors.raise_for_status(4)
            if metadata.has_result_message:
                if metadata.result_message_length > _MAX_INPUT_BYTES or (
                    metadata.result_message_length
                    and metadata.result_message_data == ffi.NULL
                ):
                    errors.raise_for_status(4)
                message_bytes = (
                    bytes(
                        ffi.buffer(
                            metadata.result_message_data,
                            int(metadata.result_message_length),
                        )
                    )
                    if metadata.result_message_length
                    else b""
                )
                try:
                    result_message = message_bytes.decode("utf-8", errors="strict")
                except UnicodeDecodeError:
                    errors.raise_for_status(4)
            else:
                if (
                    metadata.result_message_data != ffi.NULL
                    or metadata.result_message_length != 0
                ):
                    errors.raise_for_status(4)
                result_message = None
            return AsyncOperationMetadataV1(
                operation=int(metadata.operation),
                outcome=_known_enum(BatchOutcome, int(metadata.outcome_kind)),
                result_status=int(metadata.result_status),
                result_reason=int(metadata.result_reason)
                if metadata.has_result_reason
                else None,
                cancellation_result=(
                    int(metadata.cancellation_result)
                    if metadata.has_cancellation_result
                    else None
                ),
                result_message=result_message,
            )

    @property
    def asynchronous_correlation_value(self) -> bytes | None:
        """Copy the exact binary correlation bytes associated with the result."""
        present = ffi.new("uint8_t *")
        value = ffi.new("kmipkit_byte_span_v1_t *")
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_async_operation_outcome_correlation_value(
                    self._pointer(), present, value
                )
            )
            return _optional_output_span(present, value)

    @property
    def response_payload_ttlv(self) -> ttlv.Structure | None:
        """Copy a successful generic Response Payload into an owned TTLV handle."""
        present = ffi.new("uint8_t *")
        output = ffi.new("kmipkit_ttlv_structure_t **")
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_client_async_operation_outcome_response_payload_ttlv(
                    self._pointer(), present, output
                )
            )
            if present[0] == 0:
                if output[0] != ffi.NULL:
                    lib.kmipkit_ttlv_structure_release(output[0])
                    errors.raise_for_status(4)
                return None
            if present[0] != 1 or output[0] == ffi.NULL:
                if output[0] != ffi.NULL:
                    lib.kmipkit_ttlv_structure_release(output[0])
                errors.raise_for_status(4)
            address = int(ffi.cast("uintptr_t", output[0]))
            try:
                return ttlv._structure_from_native_address(address)
            except Exception:
                lib.kmipkit_ttlv_structure_release(output[0])
                raise


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


class _BatchItemResultNativeState:
    __slots__ = (
        "_native_usage_allocation_response",
        "_rng_retrieve_response",
        "_rng_seed_response",
    )


@dataclass(frozen=True, slots=True)
class BatchItemResult(_BatchItemResultNativeState):
    """Copied metadata plus any live native result handle required by an approved capability."""

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
    check_response: CheckResponse | None = field(default=None, repr=False)
    delegated_login_response: DelegatedLoginResponse | None = field(default=None, repr=False)
    derive_key_response: DeriveKeyResponse | None = field(default=None, repr=False)
    export_response: ExportResponse | None = field(default=None, repr=False)
    get_constraints_response: GetConstraintsResponse | None = field(default=None, repr=False)
    get_attribute_list_response: GetAttributeListResponse | None = field(
        default=None, repr=False
    )
    get_attributes_response: GetAttributesResponse | None = field(default=None, repr=False)
    add_attribute_response: AddAttributeResponse | None = field(default=None, repr=False)
    adjust_attribute_response: AdjustAttributeResponse | None = field(default=None, repr=False)
    set_constraints_response: SetConstraintsResponse | None = field(default=None, repr=False)
    _get_response: GetResponse | None = field(default=None, repr=False)
    _locate_response: LocateResponse | None = field(default=None, repr=False)
    import_response: ImportResponse | None = field(default=None, repr=False)
    join_split_key_response: JoinSplitKeyResponse | None = field(default=None, repr=False)
    interop_response: InteropResponse | None = field(default=None, repr=False)
    log_response: LogResponse | None = field(default=None, repr=False)
    validate_response: ValidateResponse | None = field(default=None, repr=False)
    login_response: LoginResponse | None = field(default=None, repr=False)
    logout_response: bool | None = None
    obtain_lease_response: ObtainLeaseResponse | None = field(default=None, repr=False)
    encrypt_response: EncryptResponse | None = field(default=None, repr=False)
    decrypt_response: DecryptResponse | None = field(default=None, repr=False)
    ping_response: bool | None = None
    query_response: ttlv.Structure | None = field(default=None, repr=False)
    pkcs11_response: Pkcs11Response | None = field(default=None, repr=False)
    recertify_response: RecertifyResponse | None = field(default=None, repr=False)
    rekey_response: RekeyResponse | None = field(default=None, repr=False)
    rekey_key_pair_response: RekeyKeyPairResponse | None = field(
        default=None, repr=False
    )
    reprovision_response: ReProvisionResponse | None = field(default=None, repr=False)
    register_response: RegisterResponse | None = field(default=None, repr=False)
    revoke_response: RevokeResponse | None = field(default=None, repr=False)
    set_defaults_response: SetDefaultsResponse | None = field(default=None, repr=False)
    activate_response: ActivateResponse | None = field(default=None, repr=False)
    archive_response: ArchiveResponse | None = field(default=None, repr=False)
    destroy_response: DestroyResponse | None = field(default=None, repr=False)
    recover_response: RecoverResponse | None = field(default=None, repr=False)
    delete_attribute_response: DeleteAttributeResponse | None = field(default=None, repr=False)
    modify_attribute_response: ModifyAttributeResponse | None = field(default=None, repr=False)
    set_attribute_response: SetAttributeResponse | None = field(
        default=None, repr=False
    )

    def __post_init__(self) -> None:
        object.__setattr__(self, "_native_usage_allocation_response", None)
        object.__setattr__(self, "_rng_retrieve_response", None)
        object.__setattr__(self, "_rng_seed_response", None)

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

    def create_usage_allocation_context(
        self, requested_units: int, usage_limits_unit: int
    ) -> UsageAllocationContext:
        """Create local accounting only from this live successful native allocation result."""
        native_response = self._native_usage_allocation_response
        if (
            self.operation != 0x11
            or self.result_status != 0
            or self.outcome != BatchOutcome.COMPLETED
            or native_response is None
        ):
            errors.raise_for_status(4)
        requested = _integer_range(requested_units, 1, _MAX_I64)
        unit = _unsigned(usage_limits_unit, 32)
        output = ffi.new("kmipkit_usage_allocation_context_t **")
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_usage_allocation_context_create(
                    native_response._pointer(), requested, unit, output
                )
            )
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(UsageAllocationContext, output[0])

    def get_response(self) -> GetResponse | None:
        """Return Get's typed result and its owned opaque Any Object, when available."""
        return self._get_response

    def locate_response(self) -> LocateResponse | None:
        """Return Locate's optional count and identifiers in server order, when available."""
        return self._locate_response

    def rng_retrieve_response(self) -> RngRetrieveResponse | None:
        """Return successful RNG Retrieve output and its owned generic response TTLV."""
        return self._rng_retrieve_response

    def rng_seed_response(self) -> RngSeedResponse | None:
        """Return successful RNG Seed Data Length and its owned generic response TTLV."""
        return self._rng_seed_response


@dataclass(slots=True, init=False, eq=False, repr=False)
class EncryptResponse:
    """Successful Encrypt fields in wipeable KMIPKit-owned buffers."""

    _unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    _data: _SecretBytes | None = field(repr=False)
    _iv_counter_nonce: _SecretBytes | None = field(repr=False)
    _correlation_value: _SecretBytes | None = field(repr=False)
    _authenticated_encryption_tag: _SecretBytes | None = field(repr=False)
    _closed: bool = field(default=False, init=False, repr=False)

    def __init__(
        self,
        unique_identifier: extensions.UniqueIdentifier,
        data: bytes | _SecretBytes | None,
        iv_counter_nonce: bytes | _SecretBytes | None,
        correlation_value: bytes | _SecretBytes | None,
        authenticated_encryption_tag: bytes | _SecretBytes | None,
    ) -> None:
        self._unique_identifier = unique_identifier
        self._data = _owned_secret_bytes(data)
        self._iv_counter_nonce = _owned_secret_bytes(iv_counter_nonce)
        self._correlation_value = _owned_secret_bytes(correlation_value)
        self._authenticated_encryption_tag = (
            _owned_secret_bytes(authenticated_encryption_tag)
        )
        self._closed = False

    @property
    def unique_identifier(self) -> extensions.UniqueIdentifier:
        """Return the response identifier while this response remains open."""
        self._ensure_open()
        return self._unique_identifier

    @property
    def data(self) -> bytes | None:
        self._ensure_open()
        return None if self._data is None else self._data.to_bytes()

    @property
    def iv_counter_nonce(self) -> bytes | None:
        self._ensure_open()
        return None if self._iv_counter_nonce is None else self._iv_counter_nonce.to_bytes()

    @property
    def correlation_value(self) -> bytes | None:
        self._ensure_open()
        return None if self._correlation_value is None else self._correlation_value.to_bytes()

    @property
    def authenticated_encryption_tag(self) -> bytes | None:
        self._ensure_open()
        return (
            None if self._authenticated_encryption_tag is None
            else self._authenticated_encryption_tag.to_bytes()
        )

    def close(self) -> None:
        """Wipe KMIPKit-owned response bytes; caller copies remain caller-owned."""
        if self._closed:
            return
        for name in (
            "_data", "_iv_counter_nonce", "_correlation_value", "_authenticated_encryption_tag"
        ):
            value = getattr(self, name)
            if value is not None:
                value.wipe()
                setattr(self, name, None)
        self._closed = True

    def _ensure_open(self) -> None:
        if self._closed:
            raise RuntimeError("Encrypt response is closed")

    def __repr__(self) -> str:
        return "EncryptResponse(fields=redacted)"


@dataclass(slots=True, init=False, eq=False, repr=False)
class DecryptResponse:
    """Successful Decrypt fields in wipeable KMIPKit-owned buffers."""

    _unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    _data: _SecretBytes | None = field(repr=False)
    _correlation_value: _SecretBytes | None = field(repr=False)
    _closed: bool = field(default=False, init=False, repr=False)

    def __init__(
        self,
        unique_identifier: extensions.UniqueIdentifier,
        data: bytes | _SecretBytes | None,
        correlation_value: bytes | _SecretBytes | None,
    ) -> None:
        self._unique_identifier = unique_identifier
        self._data = _owned_secret_bytes(data)
        self._correlation_value = _owned_secret_bytes(correlation_value)
        self._closed = False

    @property
    def unique_identifier(self) -> extensions.UniqueIdentifier:
        """Return the response identifier while this response remains open."""
        self._ensure_open()
        return self._unique_identifier

    @property
    def data(self) -> bytes | None:
        self._ensure_open()
        return None if self._data is None else self._data.to_bytes()

    @property
    def correlation_value(self) -> bytes | None:
        self._ensure_open()
        return None if self._correlation_value is None else self._correlation_value.to_bytes()

    def close(self) -> None:
        """Wipe KMIPKit-owned response bytes; caller copies remain caller-owned."""
        if self._closed:
            return
        for name in ("_data", "_correlation_value"):
            value = getattr(self, name)
            if value is not None:
                value.wipe()
                setattr(self, name, None)
        self._closed = True

    def _ensure_open(self) -> None:
        if self._closed:
            raise RuntimeError("Decrypt response is closed")

    def __repr__(self) -> str:
        return "DecryptResponse(fields=redacted)"


@dataclass(frozen=True, slots=True)
class HashResponse:
    """Typed Hash response fields copied from the matching successful batch item."""

    data: bytes | None = field(repr=False)
    correlation_value: bytes | None = field(repr=False)


@dataclass(frozen=True, slots=True)
class CertifyResponse:
    """Successful Certify response and its owned complete generic TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class RecertifyResponse:
    """Successful §6.1.45 result with its owned complete generic response TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "RecertifyResponse(unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class RekeyResponse:
    """Successful §6.1.46 result with its owned complete generic response TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "RekeyResponse(unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class ReProvisionResponse:
    """Successful §6.1.48 result with its owned generic response TTLV."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "ReProvisionResponse(unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class RegisterResponse:
    """Successful §6.1.43 result with its required identifier and owned response TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "RegisterResponse(unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class RevokeResponse:
    """Successful §6.1.44 result with its required identifier and complete response TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "RevokeResponse(unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class ActivateResponse:
    """Successful KMIP 2.1 §6.1.1 response identifier."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)

    def __repr__(self) -> str:
        return "ActivateResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class ArchiveResponse:
    """Successful KMIP 2.1 §6.1.4 response identifier."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)

    def __repr__(self) -> str:
        return "ArchiveResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class DestroyResponse:
    """Successful KMIP 2.1 §6.1.15 response identifier."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)

    def __repr__(self) -> str:
        return "DestroyResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class RecoverResponse:
    """Successful KMIP 2.1 §6.1.42 response identifier."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)

    def __repr__(self) -> str:
        return "RecoverResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class RekeyKeyPairResponse:
    """Successful §6.1.47 result with both replacement IDs and complete response TTLV."""

    private_key_unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    public_key_unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "RekeyKeyPairResponse(private_key_unique_identifier=redacted, public_key_unique_identifier=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True)
class CheckResponse:
    """Typed Check fields and the owned complete response TTLV, including failures."""

    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    usage_limits_count: int | None
    cryptographic_usage_mask: int | None
    lease_time: int | None
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(slots=True, repr=False)
class DelegatedLoginResponse:
    """Raw-preserving Ticket fields and the owned generic Ticket TTLV copy.

    Python-owned Ticket bytes are kept in a mutable buffer and wiped by
    ``close``. Bytes returned to the caller and copies retained by the Python
    runtime cannot be deterministically zeroized.
    """

    ticket_type_raw: int | None
    _ticket_value: _SecretBytes | None = field(repr=False)
    ticket_ttlv: ttlv.Structure | None = field(repr=False)

    @property
    def ticket_value(self) -> bytes | None:
        """Return a caller-owned Ticket Value copy when a Ticket was present."""
        return None if self._ticket_value is None else self._ticket_value.to_bytes()

    def close(self) -> None:
        """Wipe owned Ticket Value bytes and release the generic Ticket TTLV copy."""
        if self._ticket_value is not None:
            self._ticket_value.wipe()
            self._ticket_value = None
        if self.ticket_ttlv is not None:
            self.ticket_ttlv.close()
            self.ticket_ttlv = None

    def __repr__(self) -> str:
        return (
            "DelegatedLoginResponse(ticket_type_raw="
            f"{self.ticket_type_raw!r}, ticket_value=redacted, ticket_ttlv=preserved)"
        )


@dataclass(slots=True, repr=False)
class LoginResponse:
    """Raw-preserving Login Ticket fields and the owned generic Ticket TTLV copy.

    Python-owned Ticket bytes are kept in a mutable buffer and wiped by
    ``close``. Bytes returned to the caller and copies retained by the Python
    runtime cannot be deterministically zeroized.
    """

    ticket_type_raw: int | None
    _ticket_value: _SecretBytes | None = field(repr=False)
    ticket_ttlv: ttlv.Structure | None = field(repr=False)

    @property
    def ticket_value(self) -> bytes | None:
        """Return a caller-owned Ticket Value copy when a Ticket was present."""
        return None if self._ticket_value is None else self._ticket_value.to_bytes()

    def close(self) -> None:
        """Wipe owned Ticket Value bytes and release the generic Ticket TTLV copy."""
        if self._ticket_value is not None:
            self._ticket_value.wipe()
            self._ticket_value = None
        if self.ticket_ttlv is not None:
            self.ticket_ttlv.close()
            self.ticket_ttlv = None

    def __repr__(self) -> str:
        return (
            "LoginResponse(ticket_type_raw="
            f"{self.ticket_type_raw!r}, ticket_value=redacted, ticket_ttlv=preserved)"
        )


@dataclass(frozen=True, slots=True)
class GetUsageAllocationResponse:
    """Successful Get Usage Allocation response and its owned complete generic TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class ObtainLeaseResponse:
    """Successful §6.1.35 fields with an owned ordered generic TTLV copy."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    lease_time: int
    last_change_date: int
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return (
            "ObtainLeaseResponse(unique_identifier=redacted, "
            f"lease_time={self.lease_time!r}, last_change_date={self.last_change_date!r}, "
            "response_ttlv=preserved)"
        )


@dataclass(frozen=True, slots=True)
class ImportResponse:
    """Successful Import response and its owned complete generic TTLV copy."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True)
class JoinSplitKeyResponse:
    """Successful Join Split Key response and its owned complete generic TTLV copy."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class GetResponse:
    """Typed Get fields with an independently owned, redacted Any Object copy."""

    object_type_raw: int | None
    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    object: ttlv.Structure | None = field(repr=False)

    def close(self) -> None:
        """Release the KMIPKit-owned Any Object copy; caller-made copies are separate."""
        if self.object is not None:
            self.object.close()

    def __repr__(self) -> str:
        return (
            "GetResponse(object_type_raw="
            f"{self.object_type_raw!r}, unique_identifier=redacted, object=redacted)"
        )


@dataclass(frozen=True, slots=True)
class LocateResponse:
    """Optional Located Items and ordered Unique Identifiers returned by the server."""

    located_items: int | None
    unique_identifiers: tuple[extensions.UniqueIdentifier, ...]


@dataclass(frozen=True, slots=True)
class GetConstraintsResponse:
    """Successful Get Constraints response and its owned complete generic TTLV."""

    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True)
class GetAttributeListResponse:
    """Successful Get Attribute List response with typed fields and owned generic payload."""

    unique_identifier: extensions.UniqueIdentifier
    attribute_references: tuple[extensions.AttributeReference, ...]
    response_payload_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned response payload structure."""
        self.response_payload_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class GetAttributesResponse:
    """Successful Get Attributes response, preserving whether Attributes was present."""

    unique_identifier: str = field(repr=False)
    attributes: ttlv.Structure | None = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned generic Attributes structure when present."""
        if self.attributes is not None:
            self.attributes.close()

    def __repr__(self) -> str:
        return "GetAttributesResponse(unique_identifier=redacted, attributes=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class AddAttributeResponse:
    """Successful Add Attribute response with its required Text String identifier."""

    unique_identifier: str = field(repr=False)

    def __repr__(self) -> str:
        return "AddAttributeResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class SetAttributeResponse:
    """Successful Set Attribute response with its required Text String identifier."""

    unique_identifier: str = field(repr=False)

    def __repr__(self) -> str:
        return "SetAttributeResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class ModifyAttributeResponse:
    """Successful Modify Attribute response with its required Text String identifier."""

    unique_identifier: str = field(repr=False)

    def __repr__(self) -> str:
        return "ModifyAttributeResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class AdjustAttributeResponse:
    """Successful Adjust Attribute response with its required Text String identifier."""

    unique_identifier: str = field(repr=False)

    def __repr__(self) -> str:
        return "AdjustAttributeResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class DeleteAttributeResponse:
    """Successful Delete Attribute response with its required Text String identifier."""

    unique_identifier: str = field(repr=False)

    def __repr__(self) -> str:
        return "DeleteAttributeResponse(unique_identifier=redacted)"


@dataclass(frozen=True, slots=True)
class SetConstraintsResponse:
    """Successful Set Constraints response and its owned complete generic TTLV."""

    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True)
class SetDefaultsResponse:
    """Successful Set Defaults response and its owned complete generic TTLV."""

    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class ValidateResponse:
    """Successful Validate result with the raw open indicator and owned generic TTLV."""

    validity_indicator_raw: int
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return (
            "ValidateResponse(validity_indicator_raw="
            f"{self.validity_indicator_raw}, response_ttlv=redacted)"
        )


@dataclass(frozen=True, slots=True, repr=False)
class InteropResponse:
    """Interop result with an owned complete generic TTLV for Success, Failure, or Pending."""

    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "InteropResponse(response_ttlv=redacted)"


@dataclass(slots=True, init=False, eq=False, repr=False)
class Pkcs11Response:
    """PKCS#11 result with independent TTLV copies and wipeable correlation bytes."""

    response_ttlv: ttlv.Structure = field(repr=False)
    interface_ttlv: ttlv.Structure | None = field(default=None, repr=False)
    function_ttlv: ttlv.Structure | None = field(default=None, repr=False)
    _correlation_value: _SecretBytes | None = field(default=None, repr=False)
    output_parameters_ttlv: ttlv.Structure | None = field(default=None, repr=False)
    return_code_ttlv: ttlv.Structure | None = field(default=None, repr=False)
    _closed: bool = field(default=False, init=False, repr=False)

    def __init__(
        self,
        response_ttlv: ttlv.Structure,
        interface_ttlv: ttlv.Structure | None = None,
        function_ttlv: ttlv.Structure | None = None,
        correlation_value: bytes | _SecretBytes | None = None,
        output_parameters_ttlv: ttlv.Structure | None = None,
        return_code_ttlv: ttlv.Structure | None = None,
    ) -> None:
        self.response_ttlv = response_ttlv
        self.interface_ttlv = interface_ttlv
        self.function_ttlv = function_ttlv
        self._correlation_value = _owned_secret_bytes(correlation_value)
        self.output_parameters_ttlv = output_parameters_ttlv
        self.return_code_ttlv = return_code_ttlv
        self._closed = False

    @property
    def correlation_value(self) -> bytes | None:
        """Return caller-owned bytes.

        Copies retained by Python are outside KMIPKit's zeroization control.
        """
        self._ensure_open()
        return (
            None
            if self._correlation_value is None
            else self._correlation_value.to_bytes()
        )

    def close(self) -> None:
        """Wipe owned correlation bytes and release all response TTLV copies."""
        if self._closed:
            return
        if self._correlation_value is not None:
            self._correlation_value.wipe()
            self._correlation_value = None
        for value in (
            self.interface_ttlv,
            self.function_ttlv,
            self.output_parameters_ttlv,
            self.return_code_ttlv,
            self.response_ttlv,
        ):
            if value is not None:
                value.close()
        self._closed = True

    def _ensure_open(self) -> None:
        if self._closed:
            raise RuntimeError("PKCS#11 response is closed")

    def __repr__(self) -> str:
        return "Pkcs11Response(response_ttlv=redacted, fields=redacted)"


@dataclass(frozen=True, slots=True, repr=False)
class LogResponse:
    """Log result with an owned complete generic response TTLV."""

    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return "LogResponse(response_ttlv=redacted)"


@dataclass(frozen=True, slots=True)
class DeriveKeyResponse:
    """Successful Derive Key response and its owned complete generic TTLV."""

    unique_identifier: extensions.UniqueIdentifier = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()


@dataclass(frozen=True, slots=True, repr=False)
class ExportResponse:
    """Typed Export result with independently owned opaque response structures."""

    object_type_raw: int | None
    unique_identifier: extensions.UniqueIdentifier | None = field(repr=False)
    attributes: ttlv.Structure | None = field(repr=False)
    any_object: ttlv.Structure | None = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the owned response TTLV, Attributes, and Any Object copies."""
        for value in (self.attributes, self.any_object, self.response_ttlv):
            if value is not None:
                value.close()

    def __repr__(self) -> str:
        return (
            "ExportResponse(object_type_raw="
            f"{self.object_type_raw!r}, unique_identifier=redacted, "
            "attributes=redacted, any_object=redacted, response_ttlv=redacted)"
        )


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


@dataclass(slots=True, init=False, repr=False)
class RngRetrieveResponse:
    """Successful §6.1.49 output in wipeable storage plus complete generic response TTLV."""

    _data: _SecretBytes | None = field(repr=False)
    response_ttlv: ttlv.Structure = field(repr=False)
    _closed: bool = field(default=False, init=False, repr=False)

    def __init__(self, data: _SecretBytes, response_ttlv: ttlv.Structure) -> None:
        self._data = data
        self.response_ttlv = response_ttlv
        self._closed = False

    @property
    def data(self) -> bytes:
        """Return a caller-owned random-data copy; Python cannot guarantee its zeroization."""
        self._ensure_open()
        if self._data is None:
            errors.raise_for_status(4)
        return self._data.to_bytes()

    def close(self) -> None:
        """Wipe the KMIPKit-owned data buffer and release the generic response TTLV copy."""
        if self._closed:
            return
        if self._data is not None:
            self._data.wipe()
            self._data = None
        self.response_ttlv.close()
        self._closed = True

    def _ensure_open(self) -> None:
        if self._closed:
            raise RuntimeError("RNG Retrieve response is closed")

    def __repr__(self) -> str:
        return "RngRetrieveResponse(data=redacted, response_ttlv=preserved)"


@dataclass(frozen=True, slots=True, repr=False)
class RngSeedResponse:
    """Successful §6.1.50 server-reported Data Length and complete generic response TTLV."""

    data_length: int
    response_ttlv: ttlv.Structure = field(repr=False)

    def close(self) -> None:
        """Release the caller-owned complete generic response structure."""
        self.response_ttlv.close()

    def __repr__(self) -> str:
        return f"RngSeedResponse(data_length={self.data_length}, response_ttlv=preserved)"


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

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            for item in self.items:
                native_response = item._native_usage_allocation_response
                if native_response is not None:
                    native_response.close()
            super().close()


class UsageAllocationContext(_ClientHandle):
    """Single-owner local usage accounting; reservation does not prove remote consumption."""

    _release_name = "kmipkit_usage_allocation_context_release"

    @property
    def usage_limits_unit(self) -> int:
        return _client_scalar(
            "uint32_t", lib.kmipkit_usage_allocation_context_usage_limits_unit, self
        )

    @property
    def allocated_units(self) -> int:
        return _client_scalar(
            "int64_t", lib.kmipkit_usage_allocation_context_allocated_units, self
        )

    @property
    def available_units(self) -> int:
        return _client_scalar(
            "int64_t", lib.kmipkit_usage_allocation_context_available_units, self
        )

    @property
    def reserved_units(self) -> int:
        return _client_scalar(
            "int64_t", lib.kmipkit_usage_allocation_context_reserved_units, self
        )

    @property
    def unique_identifier(self) -> extensions.UniqueIdentifier:
        identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
        identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
        identifier.abi_version = 1
        with _NATIVE_CALL_LOCK:
            _raise_abi_status(
                lib.kmipkit_usage_allocation_context_unique_identifier(
                    self._pointer(), identifier
                )
            )
            kind = int(identifier.kind)
            if kind == int(extensions.UniqueIdentifierKind.TEXT_STRING):
                if identifier.text_length > _MAX_INPUT_BYTES or (
                    identifier.text_length and identifier.text_data == ffi.NULL
                ):
                    errors.raise_for_status(4)
                encoded = (
                    bytes(ffi.buffer(identifier.text_data, int(identifier.text_length)))
                    if identifier.text_length
                    else b""
                )
                try:
                    text = encoded.decode("utf-8", errors="strict")
                except UnicodeDecodeError:
                    errors.raise_for_status(4)
                return extensions.UniqueIdentifier.text_string(text)
            if kind == int(extensions.UniqueIdentifierKind.ENUMERATION):
                return extensions.UniqueIdentifier.enumeration(int(identifier.enumeration_value))
            if kind == int(extensions.UniqueIdentifierKind.INTEGER):
                return extensions.UniqueIdentifier.integer(int(identifier.integer_value))
            errors.raise_for_status(4)


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
            return _batch_result(response_output[0], limits)

    def _execute_async_exchange(
        self,
        limits: ttlv.CodecLimits,
        native_function: Callable[..., int],
        arguments: tuple[Any, ...],
    ) -> AsyncOperationOutcome:
        if not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        output = ffi.new("kmipkit_client_async_operation_outcome_t **")
        with _NATIVE_CALL_LOCK:
            try:
                status = native_function(
                    self._pointer(),
                    *arguments,
                    ffi.cast("kmipkit_codec_limits_t *", limits._pointer()),
                    output,
                )
            except (OverflowError, TypeError, ValueError):
                raise errors.InvalidInputError() from None
            _raise_client_status(status)
            if output[0] == ffi.NULL:
                errors.raise_for_status(7)
            return _wrap_owned_handle(AsyncOperationOutcome, output[0])

    def execute_poll(
        self,
        correlation_value: bytes | bytearray | memoryview,
        limits: ttlv.CodecLimits,
    ) -> AsyncOperationOutcome:
        """Send exactly one Poll and return its explicit result without auto-polling."""
        if not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        raw_correlation, staged_correlation = _stage_correlation_value(
            correlation_value
        )
        try:
            return self._execute_async_exchange(
                limits,
                lib.kmipkit_client_execute_poll,
                (staged_correlation, len(raw_correlation)),
            )
        finally:
            _wipe_staging_buffer(staged_correlation)

    def execute_cancel(
        self,
        correlation_value: bytes | bytearray | memoryview,
        limits: ttlv.CodecLimits,
    ) -> AsyncOperationOutcome:
        """Send exactly one Cancel and return its server-reported result."""
        raw_correlation, staged_correlation = _stage_correlation_value(
            correlation_value
        )
        try:
            return self._execute_async_exchange(
                limits,
                lib.kmipkit_client_execute_cancel,
                (staged_correlation, len(raw_correlation)),
            )
        finally:
            _wipe_staging_buffer(staged_correlation)

    def execute_process(
        self,
        correlation_value: bytes | bytearray | memoryview,
        limits: ttlv.CodecLimits,
        *,
        asynchronous_indicator: int | None = None,
    ) -> AsyncOperationOutcome:
        """Send one Process with its caller-selected asynchronous indicator."""
        has_indicator = int(asynchronous_indicator is not None)
        indicator = (
            _unsigned(asynchronous_indicator, 32)
            if asynchronous_indicator is not None
            else 0
        )
        raw_correlation, staged_correlation = _stage_correlation_value(
            correlation_value
        )
        try:
            return self._execute_async_exchange(
                limits,
                lib.kmipkit_client_execute_process,
                (
                    staged_correlation,
                    len(raw_correlation),
                    has_indicator,
                    indicator,
                ),
            )
        finally:
            _wipe_staging_buffer(staged_correlation)

    def execute_query_async_requests(
        self,
        limits: ttlv.CodecLimits,
        *,
        correlation_values: Sequence[bytes | bytearray | memoryview] | None = None,
        operations: Sequence[int] | None = None,
        asynchronous_indicator: int | None = None,
    ) -> AsyncOperationOutcome:
        """Send one Query Asynchronous Requests and retain generic response TTLV."""
        if correlation_values is not None and (
            not isinstance(correlation_values, Sequence)
            or isinstance(correlation_values, (str, bytes, bytearray, memoryview))
        ):
            errors.raise_for_status(4)
        if operations is not None and (
            not isinstance(operations, Sequence)
            or isinstance(operations, (str, bytes, bytearray, memoryview))
        ):
            errors.raise_for_status(4)
        has_correlations = int(correlation_values is not None)
        has_operations = int(operations is not None)
        has_indicator = int(asynchronous_indicator is not None)
        indicator = (
            _unsigned(asynchronous_indicator, 32)
            if asynchronous_indicator is not None
            else 0
        )
        staged_values: list[Any] = []
        span_array = ffi.NULL
        operation_array = ffi.NULL
        correlation_count = 0
        operation_count = 0
        try:
            if correlation_values is not None:
                if len(correlation_values) > 100_000:
                    errors.raise_for_status(6)
                correlations = tuple(correlation_values)
                correlation_count = len(correlations)
                if correlation_count > 100_000:
                    errors.raise_for_status(6)
                total_correlation_bytes = 0
                if correlation_count:
                    span_array = ffi.new("kmipkit_byte_span_v1_t[]", correlation_count)
                    for index, value in enumerate(correlations):
                        raw, staged = _stage_correlation_value(value)
                        total_correlation_bytes += len(raw)
                        if total_correlation_bytes > _MAX_INPUT_BYTES:
                            _wipe_staging_buffer(staged)
                            errors.raise_for_status(6)
                        staged_values.append(staged)
                        span_array[index].data = ffi.cast("const uint8_t *", staged)
                        span_array[index].length = len(raw)
            if operations is not None:
                if len(operations) > 100_000:
                    errors.raise_for_status(6)
                raw_operations = tuple(_unsigned(value, 32) for value in operations)
                operation_count = len(raw_operations)
                if operation_count > 100_000:
                    errors.raise_for_status(6)
                if operation_count:
                    operation_array = ffi.new("uint32_t[]", raw_operations)
            return self._execute_async_exchange(
                limits,
                lib.kmipkit_client_execute_query_async_requests,
                (
                    has_correlations,
                    span_array,
                    correlation_count,
                    has_operations,
                    operation_array,
                    operation_count,
                    has_indicator,
                    indicator,
                ),
            )
        finally:
            for staged in staged_values:
                _wipe_staging_buffer(staged)

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
            return _batch_result(response_output[0], limits)

    def execute_guarded_protection_item(
        self,
        context: UsageAllocationContext,
        item: extensions.ClientBatchItem,
        cost: int,
        limits: ttlv.CodecLimits,
    ) -> BatchResult:
        """Execute one typed Encrypt, MAC, or Sign item under local usage accounting."""
        return self._execute_guarded_protection_item(
            context, item, cost, None, limits
        )

    def execute_guarded_protection_item_with_asynchronous_indicator(
        self,
        context: UsageAllocationContext,
        item: extensions.ClientBatchItem,
        cost: int,
        asynchronous_indicator: int,
        limits: ttlv.CodecLimits,
    ) -> BatchResult:
        """Execute one guarded protection item with an explicit raw KMIP indicator."""
        indicator = _unsigned(asynchronous_indicator, 32)
        return self._execute_guarded_protection_item(
            context, item, cost, indicator, limits
        )

    def _execute_guarded_protection_item(
        self,
        context: UsageAllocationContext,
        item: extensions.ClientBatchItem,
        cost: int,
        asynchronous_indicator: int | None,
        limits: ttlv.CodecLimits,
    ) -> BatchResult:
        if (
            not isinstance(context, UsageAllocationContext)
            or not isinstance(item, extensions.ClientBatchItem)
            or not isinstance(limits, ttlv.CodecLimits)
        ):
            errors.raise_for_status(4)
        raw_cost = _integer_range(cost, 1, _MAX_I64)
        response_output = ffi.new("kmipkit_client_batch_response_t **")
        with _NATIVE_CALL_LOCK:
            item._acquire_ownership_lock()
            try:
                client_pointer = self._pointer()
                context_pointer = context._pointer()
                limits_pointer = ffi.cast(
                    "kmipkit_codec_limits_t *", limits._pointer()
                )
                original = item._pointer()
                item_slot = ffi.new("kmipkit_client_batch_item_t **")
                item_slot[0] = ffi.cast(
                    "kmipkit_client_batch_item_t *",
                    int(_base_ffi.cast("uintptr_t", original)),
                )
                original, owner = item._take()
                try:
                    if asynchronous_indicator is None:
                        status = lib.kmipkit_client_execute_guarded_protection_item(
                            client_pointer,
                            context_pointer,
                            item_slot,
                            raw_cost,
                            limits_pointer,
                            response_output,
                        )
                    else:
                        status = lib.kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
                            client_pointer,
                            context_pointer,
                            item_slot,
                            raw_cost,
                            asynchronous_indicator,
                            limits_pointer,
                            response_output,
                        )
                except (OverflowError, TypeError, ValueError):
                    item._restore(original, owner)
                    raise errors.InvalidInputError() from None
                if item_slot[0] != ffi.NULL:
                    item._restore(original, owner)
                if status == 0 and item_slot[0] != ffi.NULL:
                    errors.raise_for_status(7)
                _raise_client_status(status)
            finally:
                item._release_ownership_lock()
        if response_output[0] == ffi.NULL:
            errors.raise_for_status(7)
        return _batch_result(response_output[0], limits)


def _batch_result(
    handle: Any, limits: ttlv.CodecLimits | None = None
) -> BatchResult:
    response = _wrap_owned_handle(BatchResult, handle, ())
    max_elements = (
        sys.maxsize
        if limits is None
        else ttlv.codec_limits_max_elements(limits)
    )
    max_message_bytes = (
        _MAX_INPUT_BYTES
        if limits is None
        else ttlv.codec_limits_max_message_bytes(limits)
    )
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
            keep_item_handle = False
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
                recertify_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x07
                ):
                    recertify_response = _recertify_response(item)
                rekey_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x04
                ):
                    rekey_response = _rekey_response(item)
                rekey_key_pair_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x1D
                ):
                    rekey_key_pair_response = _rekey_key_pair_response(item)
                reprovision_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x35
                ):
                    reprovision_response = _reprovision_response(item)
                register_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x03
                ):
                    register_response = _register_response(item)
                revoke_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x13
                ):
                    revoke_response = _revoke_response(item)
                activate_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x12
                ):
                    activate_response = _lifecycle_response(item, "activate", ActivateResponse)
                archive_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x15
                ):
                    archive_response = _lifecycle_response(item, "archive", ArchiveResponse)
                destroy_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x14
                ):
                    destroy_response = _lifecycle_response(item, "destroy", DestroyResponse)
                recover_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x16
                ):
                    recover_response = _lifecycle_response(item, "recover", RecoverResponse)
                rng_retrieve_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x25
                ):
                    rng_retrieve_response = _rng_retrieve_response(item)
                rng_seed_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x26
                ):
                    rng_seed_response = _rng_seed_response(item)
                check_response = None
                if int(metadata.operation) == 0x09:
                    check_response = _check_response(item)
                delegated_login_response = None
                if int(metadata.operation) == 0x2F:
                    delegated_login_response = _delegated_login_response(item)
                login_response = None
                if int(metadata.operation) == 0x2D:
                    login_response = _login_response(item)
                logout_response = None
                if int(metadata.operation) == 0x2E:
                    logout_response = _logout_response(item)
                allocation_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x11
                ):
                    allocation_response = _get_usage_allocation_response(item)
                obtain_lease_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x10
                ):
                    obtain_lease_response = _obtain_lease_response(item)
                constraints_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x38
                ):
                    constraints_response = _get_constraints_response(item)
                attribute_list_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x0C
                ):
                    attribute_list_response = _get_attribute_list_response(
                        item, max_elements, max_message_bytes
                    )
                get_attributes_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x0B
                ):
                    get_attributes_response = _get_attributes_response(item)
                add_attribute_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x0D
                ):
                    add_attribute_response = _add_attribute_response(item)
                set_attribute_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x31
                ):
                    set_attribute_response = _set_attribute_response(item)
                adjust_attribute_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x30
                ):
                    adjust_attribute_response = _adjust_attribute_response(item)
                delete_attribute_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x0F
                ):
                    delete_attribute_response = _delete_attribute_response(item)
                modify_attribute_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x0E
                ):
                    modify_attribute_response = _modify_attribute_response(item)
                set_constraints_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x37
                ):
                    set_constraints_response = _set_constraints_response(item)
                set_defaults_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x36
                ):
                    set_defaults_response = _set_defaults_response(item)
                derive_key_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x05
                ):
                    derive_key_response = _derive_key_response(item)
                export_response = None
                if int(metadata.operation) == 0x2B:
                    export_response = _export_response(item)
                import_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x2A
                ):
                    import_response = _import_response(item)
                join_split_key_response = _join_split_key_response_for_result(
                    item,
                    outcome=outcome,
                    result_status=int(metadata.result_status),
                    operation=int(metadata.operation),
                )
                get_response = None
                if int(metadata.operation) == 0x0A:
                    get_response = _get_response(item)
                locate_response = None
                if int(metadata.operation) == 0x08:
                    locate_response = _locate_response(item)
                interop_response = None
                if int(metadata.operation) == 0x34:
                    interop_response = _interop_response(item)
                pkcs11_response = None
                if int(metadata.operation) == 0x33:
                    pkcs11_response = _pkcs11_response(item)
                log_response = None
                if int(metadata.operation) == 0x2C:
                    log_response = _log_response(item)
                validate_response = None
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x17
                ):
                    validate_response = _validate_response(item)
                ping_response = None
                if int(metadata.operation) == 0x3B:
                    ping_response = _ping_response(item)
                query_response = None
                if int(metadata.operation) == 0x18:
                    query_response = _query_response(item)
                cryptographic_responses = (None, None, None, None, None, None, None)
                if outcome == BatchOutcome.COMPLETED and int(metadata.result_status) == 0:
                    cryptographic_responses = _cryptographic_responses(
                        item, int(metadata.operation)
                    )
                (
                    encrypt_response,
                    decrypt_response,
                    hash_response,
                    mac_response,
                    mac_verify_response,
                    sign_response,
                    signature_verify_response,
                ) = cryptographic_responses
                result = BatchItemResult(
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
                    hash_response,
                    mac_response,
                    mac_verify_response,
                    sign_response,
                    signature_verify_response,
                    certify_response,
                    allocation_response,
                    check_response,
                    delegated_login_response,
                    derive_key_response,
                    export_response,
                    get_constraints_response=constraints_response,
                    set_constraints_response=set_constraints_response,
                    set_defaults_response=set_defaults_response,
                    get_attribute_list_response=attribute_list_response,
                    get_attributes_response=get_attributes_response,
                    add_attribute_response=add_attribute_response,
                    set_attribute_response=set_attribute_response,
                    adjust_attribute_response=adjust_attribute_response,
                    _get_response=get_response,
                    _locate_response=locate_response,
                    import_response=import_response,
                    join_split_key_response=join_split_key_response,
                    interop_response=interop_response,
                    pkcs11_response=pkcs11_response,
                    log_response=log_response,
                    validate_response=validate_response,
                    login_response=login_response,
                    logout_response=logout_response,
                    obtain_lease_response=obtain_lease_response,
                    encrypt_response=encrypt_response,
                    decrypt_response=decrypt_response,
                    ping_response=ping_response,
                    query_response=query_response,
                    recertify_response=recertify_response,
                    rekey_response=rekey_response,
                    rekey_key_pair_response=rekey_key_pair_response,
                    reprovision_response=reprovision_response,
                    register_response=register_response,
                    revoke_response=revoke_response,
                    activate_response=activate_response,
                    archive_response=archive_response,
                    destroy_response=destroy_response,
                    recover_response=recover_response,
                    delete_attribute_response=delete_attribute_response,
                    modify_attribute_response=modify_attribute_response,
                )
                object.__setattr__(result, "_rng_retrieve_response", rng_retrieve_response)
                object.__setattr__(result, "_rng_seed_response", rng_seed_response)
                items.append(result)
                if (
                    outcome == BatchOutcome.COMPLETED
                    and int(metadata.result_status) == 0
                    and int(metadata.operation) == 0x11
                ):
                    object.__setattr__(result, "_native_usage_allocation_response", item)
                    keep_item_handle = True
            finally:
                if not keep_item_handle:
                    item.close()
        response.items = tuple(items)
        return response
    except Exception:
        for item in items:
            native_response = item._native_usage_allocation_response
            if native_response is not None:
                native_response.close()
        response.close()
        raise


class _ClientBatchItemResponse(_ClientHandle):
    _release_name = "kmipkit_client_batch_item_response_release"


def _client_scalar(c_type: str, function: Any, handle: _ClientHandle) -> int:
    output = ffi.new(f"{c_type} *")
    with _NATIVE_CALL_LOCK:
        _raise_abi_status(function(handle._pointer(), output))
    return int(output[0])


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
        if output.text_length > sys.maxsize:
            errors.raise_for_status(6)
        if output.text_length != 0 and output.text_data == ffi.NULL:
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
    if present[0] != 1:
        errors.raise_for_status(4)
    if value.length > sys.maxsize:
        errors.raise_for_status(6)
    if value.length and value.data == ffi.NULL:
        errors.raise_for_status(4)
    return bytes(ffi.buffer(value.data, int(value.length))) if value.length else b""


def _optional_output_secret_span(present: Any, value: Any) -> _SecretBytes | None:
    if present[0] == 0:
        if value.data != ffi.NULL or value.length != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    if value.length > sys.maxsize:
        errors.raise_for_status(6)
    if value.length and value.data == ffi.NULL:
        errors.raise_for_status(4)
    view = ffi.buffer(value.data, int(value.length)) if value.length else memoryview(b"")
    return _SecretBytes(view)


def _owned_secret_bytes(value: bytes | _SecretBytes | None) -> _SecretBytes | None:
    if value is None or isinstance(value, _SecretBytes):
        return value
    return _SecretBytes(value)


def _optional_output_identifier(value: Any) -> extensions.UniqueIdentifier | None:
    if value.kind == 0:
        if value.text_data != ffi.NULL or value.text_length != 0:
            errors.raise_for_status(4)
        return None
    return _read_unique_identifier(value)


def _require_absent_unique_identifier(value: Any) -> None:
    if (
        value.struct_size != ffi.sizeof("kmipkit_unique_identifier_v1_t")
        or value.abi_version != 1
        or value.reserved != 0
        or value.kind != 0
        or value.text_data != ffi.NULL
        or value.text_length != 0
        or value.enumeration_value != 0
        or value.integer_value != 0
    ):
        errors.raise_for_status(4)


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


def _encrypt_response(item: _ClientBatchItemResponse) -> EncryptResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    data_present = ffi.new("uint8_t *")
    data = ffi.new("kmipkit_byte_span_v1_t *")
    iv_present = ffi.new("uint8_t *")
    iv = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    tag_present = ffi.new("uint8_t *")
    tag = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_encrypt(
            item._pointer(), present, identifier, data_present, data,
            iv_present, iv, correlation_present, correlation, tag_present, tag
        )
    )
    if present[0] == 0:
        if any(flag[0] != 0 for flag in (data_present, iv_present, correlation_present, tag_present)):
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        for flag, span in (
            (data_present, data), (iv_present, iv),
            (correlation_present, correlation), (tag_present, tag),
        ):
            _optional_output_span(flag, span)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    return EncryptResponse(
        unique_identifier,
        _optional_output_secret_span(data_present, data),
        _optional_output_secret_span(iv_present, iv),
        _optional_output_secret_span(correlation_present, correlation),
        _optional_output_secret_span(tag_present, tag),
    )


def _decrypt_response(item: _ClientBatchItemResponse) -> DecryptResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    data_present = ffi.new("uint8_t *")
    data = ffi.new("kmipkit_byte_span_v1_t *")
    correlation_present = ffi.new("uint8_t *")
    correlation = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_decrypt(
            item._pointer(), present, identifier, data_present, data,
            correlation_present, correlation
        )
    )
    if present[0] == 0:
        if data_present[0] != 0 or correlation_present[0] != 0:
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        _optional_output_span(data_present, data)
        _optional_output_span(correlation_present, correlation)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    return DecryptResponse(
        unique_identifier,
        _optional_output_secret_span(data_present, data),
        _optional_output_secret_span(correlation_present, correlation),
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
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_certify_ttlv(
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
    return CertifyResponse(unique_identifier, response_ttlv)


def _recertify_response(item: _ClientBatchItemResponse) -> RecertifyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_recertify(
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
        lib.kmipkit_client_batch_item_response_recertify_ttlv(
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
    return RecertifyResponse(unique_identifier, response_ttlv)


def _reprovision_response(
    item: _ClientBatchItemResponse,
) -> ReProvisionResponse | None:
    present = ffi.new("uint8_t *")
    identifier_present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_reprovision(
            item._pointer(), present, identifier_present, identifier
        )
    )
    if present[0] == 0:
        if identifier_present[0] != 0:
            errors.raise_for_status(4)
        _require_absent_unique_identifier(identifier)
        return None
    if present[0] != 1 or identifier_present[0] not in (0, 1):
        errors.raise_for_status(4)
    if identifier_present[0] == 1:
        unique_identifier = _read_unique_identifier(identifier)
    else:
        _require_absent_unique_identifier(identifier)
        unique_identifier = None

    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_reprovision_ttlv(
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
    return ReProvisionResponse(unique_identifier, response_ttlv)


def _register_response(
    item: _ClientBatchItemResponse,
) -> RegisterResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_register(
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
        lib.kmipkit_client_batch_item_response_register_ttlv(
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
    return RegisterResponse(unique_identifier, response_ttlv)


def _revoke_response(item: _ClientBatchItemResponse) -> RevokeResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_revoke(
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
        lib.kmipkit_client_batch_item_response_revoke_ttlv(
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
    return RevokeResponse(unique_identifier, response_ttlv)


def _lifecycle_response(
    item: _ClientBatchItemResponse,
    operation: str,
    response_type: type[ActivateResponse]
    | type[ArchiveResponse]
    | type[DestroyResponse]
    | type[RecoverResponse],
) -> ActivateResponse | ArchiveResponse | DestroyResponse | RecoverResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        getattr(lib, f"kmipkit_client_batch_item_response_{operation}")(
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
    return response_type(unique_identifier)


def _rekey_response(item: _ClientBatchItemResponse) -> RekeyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_rekey(
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
        lib.kmipkit_client_batch_item_response_rekey_ttlv(
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
    return RekeyResponse(unique_identifier, response_ttlv)


def _rekey_key_pair_response(
    item: _ClientBatchItemResponse,
) -> RekeyKeyPairResponse | None:
    present = ffi.new("uint8_t *")
    private_identifier = _initialize_unique_identifier_output()
    public_identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_rekey_key_pair(
            item._pointer(), present, private_identifier, public_identifier
        )
    )
    if present[0] == 0:
        _optional_output_identifier(private_identifier)
        _optional_output_identifier(public_identifier)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    private_value = _optional_output_identifier(private_identifier)
    public_value = _optional_output_identifier(public_identifier)
    if private_value is None or public_value is None:
        errors.raise_for_status(4)
    ttlv_present = ffi.new("uint8_t *")
    ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
            item._pointer(), ttlv_present, ttlv_output
        )
    )
    native = ttlv_output[0]
    if ttlv_present[0] != 1 or native == ffi.NULL:
        if native != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(native)
        errors.raise_for_status(4)
    try:
        response_ttlv = ttlv._structure_from_native_address(int(ffi.cast("uintptr_t", native)))
    except Exception:
        lib.kmipkit_ttlv_structure_release(native)
        raise
    return RekeyKeyPairResponse(private_value, public_value, response_ttlv)


def _check_response(item: _ClientBatchItemResponse) -> CheckResponse | None:
    present = ffi.new("uint8_t *")
    identifier_present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    count_present = ffi.new("uint8_t *")
    count = ffi.new("int64_t *")
    mask_present = ffi.new("uint8_t *")
    usage_mask = ffi.new("int32_t *")
    lease_present = ffi.new("uint8_t *")
    lease = ffi.new("uint32_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_check(
            item._pointer(), present, identifier_present, identifier,
            count_present, count, mask_present, usage_mask, lease_present, lease,
        )
    )
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if identifier_present[0] not in (0, 1) or bool(identifier_present[0]) != (
        unique_identifier is not None
    ):
        errors.raise_for_status(4)
    count_value = _optional_output_integer(count_present, count[0])
    mask_value = _optional_output_integer(mask_present, usage_mask[0])
    lease_value = _optional_output_integer(lease_present, lease[0])
    if present[0] == 0:
        if any(value is not None for value in (unique_identifier, count_value, mask_value, lease_value)):
            errors.raise_for_status(4)
        response_ttlv_present = ffi.new("uint8_t *")
        response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_check_ttlv(
                item._pointer(), response_ttlv_present, response_ttlv_output
            )
        )
        if response_ttlv_present[0] != 0 or response_ttlv_output[0] != ffi.NULL:
            if response_ttlv_output[0] != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
            errors.raise_for_status(4)
        return None
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_check_ttlv(
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
    return CheckResponse(
        unique_identifier, count_value, mask_value, lease_value, response_ttlv
    )


def _delegated_login_response(
    item: _ClientBatchItemResponse,
) -> DelegatedLoginResponse | None:
    present = ffi.new("uint8_t *")
    ticket_present = ffi.new("uint8_t *")
    ticket_type = ffi.new("uint32_t *")
    ticket_value = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_delegated_login(
            item._pointer(), present, ticket_present, ticket_type, ticket_value
        )
    )
    if present[0] not in (0, 1) or ticket_present[0] not in (0, 1):
        errors.raise_for_status(4)
    secret: _SecretBytes | None = None
    if ticket_present[0] == 0:
        if ticket_type[0] != 0:
            errors.raise_for_status(4)
        _optional_output_span(ffi.new("uint8_t *"), ticket_value)
    else:
        if ticket_value.length > _MAX_INPUT_BYTES or (
            ticket_value.length and ticket_value.data == ffi.NULL
        ):
            errors.raise_for_status(4)
        secret = _SecretBytes(
            ffi.buffer(ticket_value.data, int(ticket_value.length))
            if ticket_value.length
            else b""
        )
    ttlv_present = ffi.new("uint8_t *")
    ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    try:
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_delegated_login_ttlv(
                item._pointer(), ttlv_present, ttlv_output
            )
        )
        structure_handle = ttlv_output[0]
        expected_ttlv_presence = ticket_present[0]
        if (
            ttlv_present[0] != expected_ttlv_presence
            or (expected_ttlv_presence == 1 and structure_handle == ffi.NULL)
            or (expected_ttlv_presence == 0 and structure_handle != ffi.NULL)
        ):
            if structure_handle != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(structure_handle)
            errors.raise_for_status(4)
        ticket_structure = None
        if structure_handle != ffi.NULL:
            try:
                ticket_structure = ttlv._structure_from_native_address(
                    int(ffi.cast("uintptr_t", structure_handle))
                )
            except Exception:
                lib.kmipkit_ttlv_structure_release(structure_handle)
                raise
        if present[0] == 0:
            if ticket_present[0] != 0:
                if ticket_structure is not None:
                    ticket_structure.close()
                errors.raise_for_status(4)
            return None
        return DelegatedLoginResponse(
            int(ticket_type[0]) if ticket_present[0] == 1 else None,
            secret,
            ticket_structure,
        )
    except Exception:
        if secret is not None:
            secret.wipe()
        raise


def _login_response(item: _ClientBatchItemResponse) -> LoginResponse | None:
    present = ffi.new("uint8_t *")
    ticket_present = ffi.new("uint8_t *")
    ticket_type = ffi.new("uint32_t *")
    ticket_value = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_login(
            item._pointer(), present, ticket_present, ticket_type, ticket_value
        )
    )
    if present[0] not in (0, 1) or ticket_present[0] not in (0, 1):
        errors.raise_for_status(4)
    secret: _SecretBytes | None = None
    if ticket_present[0] == 0:
        if ticket_type[0] != 0:
            errors.raise_for_status(4)
        _optional_output_span(ffi.new("uint8_t *"), ticket_value)
    else:
        if ticket_value.length > _MAX_INPUT_BYTES or (
            ticket_value.length and ticket_value.data == ffi.NULL
        ):
            errors.raise_for_status(4)
        secret = _SecretBytes(
            ffi.buffer(ticket_value.data, int(ticket_value.length))
            if ticket_value.length
            else b""
        )
    ttlv_present = ffi.new("uint8_t *")
    ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    try:
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_login_ttlv(
                item._pointer(), ttlv_present, ttlv_output
            )
        )
        structure_handle = ttlv_output[0]
        expected_ttlv_presence = ticket_present[0]
        if (
            ttlv_present[0] != expected_ttlv_presence
            or (expected_ttlv_presence == 1 and structure_handle == ffi.NULL)
            or (expected_ttlv_presence == 0 and structure_handle != ffi.NULL)
        ):
            if structure_handle != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(structure_handle)
            errors.raise_for_status(4)
        ticket_structure = None
        if structure_handle != ffi.NULL:
            try:
                ticket_structure = ttlv._structure_from_native_address(
                    int(ffi.cast("uintptr_t", structure_handle))
                )
            except Exception:
                lib.kmipkit_ttlv_structure_release(structure_handle)
                raise
        if present[0] == 0:
            if ticket_present[0] != 0:
                if ticket_structure is not None:
                    ticket_structure.close()
                errors.raise_for_status(4)
            return None
        return LoginResponse(
            int(ticket_type[0]) if ticket_present[0] == 1 else None,
            secret,
            ticket_structure,
        )
    except Exception:
        if secret is not None:
            secret.wipe()
        raise


def _optional_output_integer(present: Any, value: int) -> int | None:
    if present[0] == 0:
        if value != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1:
        errors.raise_for_status(4)
    return int(value)


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


def _obtain_lease_response(
    item: _ClientBatchItemResponse,
) -> ObtainLeaseResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    lease_time = ffi.new("uint32_t *")
    last_change_date = ffi.new("int64_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_obtain_lease(
            item._pointer(), present, identifier, lease_time, last_change_date
        )
    )
    if present[0] == 0:
        if identifier.kind != 0 or lease_time[0] != 0 or last_change_date[0] != 0:
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
        lib.kmipkit_client_batch_item_response_obtain_lease_ttlv(
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
    return ObtainLeaseResponse(
        unique_identifier, int(lease_time[0]), int(last_change_date[0]), response_ttlv
    )


def _import_response(item: _ClientBatchItemResponse) -> ImportResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_import(
            item._pointer(), present, identifier
        )
    )
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    if present[0] == 0:
        if identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        return None
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)

    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_import_ttlv(
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
    return ImportResponse(unique_identifier, response_ttlv)


def _join_split_key_response(
    item: _ClientBatchItemResponse,
) -> JoinSplitKeyResponse | None:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_join_split_key(
            item._pointer(), present, identifier
        )
    )
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    if present[0] == 0:
        if _optional_output_identifier(identifier) is not None:
            errors.raise_for_status(4)
        return None
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_join_split_key_ttlv(
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
    return JoinSplitKeyResponse(unique_identifier, response_ttlv)


def _join_split_key_response_for_result(
    item: _ClientBatchItemResponse,
    *,
    outcome: BatchOutcome | int,
    result_status: int,
    operation: int,
) -> JoinSplitKeyResponse | None:
    """Map only successful KMIP v2.1 Join Split Key result items to the typed response."""
    if (
        outcome != BatchOutcome.COMPLETED
        or result_status != 0
        or operation != _JOIN_SPLIT_KEY_OPERATION
    ):
        return None
    return _join_split_key_response(item)


def _get_response(item: _ClientBatchItemResponse) -> GetResponse | None:
    present = ffi.new("uint8_t *")
    payload_present = ffi.new("uint8_t *")
    object_type = ffi.new("uint32_t *")
    identifier = _initialize_unique_identifier_output()
    object_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get(
            item._pointer(), present, payload_present, object_type, identifier, object_output
        )
    )
    object_handle = object_output[0]
    if present[0] not in (0, 1) or payload_present[0] not in (0, 1) or (
        present[0] == 0 and payload_present[0] != 0
    ):
        if object_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(object_handle)
        errors.raise_for_status(4)
    if present[0] == 0:
        if (
            object_type[0] != 0
            or _optional_output_identifier(identifier) is not None
            or object_handle != ffi.NULL
        ):
            if object_handle != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(object_handle)
            errors.raise_for_status(4)
        return None
    if payload_present[0] == 0:
        if (
            object_type[0] != 0
            or _optional_output_identifier(identifier) is not None
            or object_handle != ffi.NULL
        ):
            if object_handle != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(object_handle)
            errors.raise_for_status(4)
        return GetResponse(None, None, None)
    if object_handle == ffi.NULL:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        lib.kmipkit_ttlv_structure_release(object_handle)
        errors.raise_for_status(4)
    try:
        object_value = ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", object_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(object_handle)
        raise
    return GetResponse(int(object_type[0]), unique_identifier, object_value)


def _locate_response(item: _ClientBatchItemResponse) -> LocateResponse | None:
    present = ffi.new("uint8_t *")
    payload_present = ffi.new("uint8_t *")
    located_items_present = ffi.new("uint8_t *")
    located_items = ffi.new("int32_t *")
    identifier_count = ffi.new("uint64_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_locate(
            item._pointer(), present, payload_present, located_items_present,
            located_items, identifier_count,
        )
    )
    if (
        present[0] not in (0, 1)
        or payload_present[0] not in (0, 1)
        or located_items_present[0] not in (0, 1)
        or (present[0] == 0 and payload_present[0] != 0)
    ):
        errors.raise_for_status(4)
    if present[0] == 0:
        if (
            located_items_present[0] != 0
            or located_items[0] != 0
            or identifier_count[0] != 0
        ):
            errors.raise_for_status(4)
        return None
    if payload_present[0] == 0:
        if (
            located_items_present[0] != 0
            or located_items[0] != 0
            or identifier_count[0] != 0
        ):
            errors.raise_for_status(4)
        return LocateResponse(None, ())
    if identifier_count[0] > sys.maxsize:
        errors.raise_for_status(6)
    if located_items_present[0] == 0 and located_items[0] != 0:
        errors.raise_for_status(4)
    count = int(identifier_count[0])
    identifiers: list[extensions.UniqueIdentifier] = []
    for index in range(count):
        output = _initialize_unique_identifier_output()
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_locate_unique_identifier_at(
                item._pointer(), index, output
            )
        )
        identifier = _optional_output_identifier(output)
        if identifier is None:
            errors.raise_for_status(4)
        identifiers.append(identifier)
    return LocateResponse(
        int(located_items[0]) if located_items_present[0] == 1 else None,
        tuple(identifiers),
    )


def _get_constraints_response(
    item: _ClientBatchItemResponse,
) -> GetConstraintsResponse:
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_constraints_ttlv(
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
    return GetConstraintsResponse(response_ttlv)


def _get_attribute_list_response(
    item: _ClientBatchItemResponse,
    max_elements: int,
    max_message_bytes: int,
) -> GetAttributeListResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    count = ffi.new("uint64_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_attribute_list(
            item._pointer(), present, identifier, ffi.NULL, 0, count
        )
    )
    if present[0] != 1:
        errors.raise_for_status(4)
    maximum_reference_count = min(max_elements, max_message_bytes // 16, sys.maxsize)
    if count[0] > maximum_reference_count:
        errors.raise_for_status(6)
    if identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    identifier_length = int(identifier.text_length)
    if identifier_length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        identifier_text = (
            bytes(ffi.buffer(identifier.text_data, identifier_length)).decode("utf-8", "strict")
            if identifier_length
            else ""
        )
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    unique_identifier = extensions.UniqueIdentifier.text_string(identifier_text)

    references_native = ffi.new(
        "kmipkit_attribute_reference_v1_t[]", int(count[0])
    ) if count[0] else ffi.NULL
    for index in range(int(count[0])):
        references_native[index].struct_size = ffi.sizeof("kmipkit_attribute_reference_v1_t")
        references_native[index].abi_version = 1
    fill_present = ffi.new("uint8_t *")
    fill_count = ffi.new("uint64_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_attribute_list(
            item._pointer(), fill_present, identifier, references_native, count[0], fill_count
        )
    )
    if fill_present[0] != 1 or fill_count[0] != count[0]:
        errors.raise_for_status(4)

    def decode_span(data: Any, length: int) -> str:
        span_length = int(length)
        if span_length and data == ffi.NULL:
            errors.raise_for_status(4)
        try:
            value = bytes(ffi.buffer(data, span_length)) if span_length else b""
            return value.decode("utf-8", "strict")
        except UnicodeDecodeError:
            errors.raise_for_status(4)

    references: list[extensions.AttributeReference] = []
    for index in range(int(count[0])):
        value = references_native[index]
        if value.reserved != 0 or value.reserved2 != 0:
            errors.raise_for_status(4)
        if value.kind == 1 and value.vendor_identification_data == ffi.NULL \
                and value.vendor_identification_length == 0 \
                and value.attribute_name_data == ffi.NULL and value.attribute_name_length == 0:
            references.append(extensions.AttributeReference.tag(int(value.tag_value)))
        elif value.kind == 2 and value.tag_value == 0:
            vendor = decode_span(
                value.vendor_identification_data, value.vendor_identification_length
            )
            name = decode_span(value.attribute_name_data, value.attribute_name_length)
            references.append(extensions.AttributeReference.name(vendor, name))
        else:
            errors.raise_for_status(4)

    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
            item._pointer(), response_ttlv_present, response_ttlv_output
        )
    )
    response_ttlv_handle = response_ttlv_output[0]
    if response_ttlv_present[0] != 1 or response_ttlv_handle == ffi.NULL:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        errors.raise_for_status(4)
    try:
        response_payload_ttlv = ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", response_ttlv_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        raise
    return GetAttributeListResponse(
        unique_identifier, tuple(references), response_payload_ttlv
    )


def _get_attributes_response(
    item: _ClientBatchItemResponse,
) -> GetAttributesResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    attributes_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_get_attributes(
            item._pointer(), present, identifier, attributes_output
        )
    )
    attributes_handle = attributes_output[0]
    if present[0] != 1:
        if attributes_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(attributes_handle)
        errors.raise_for_status(4)
    if identifier.kind != 1 or identifier.reserved != 0:
        if attributes_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(attributes_handle)
        errors.raise_for_status(4)
    identifier_length = int(identifier.text_length)
    if identifier_length and identifier.text_data == ffi.NULL:
        if attributes_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(attributes_handle)
        errors.raise_for_status(4)
    try:
        unique_identifier = (
            bytes(ffi.buffer(identifier.text_data, identifier_length)).decode(
                "utf-8", "strict"
            )
            if identifier_length
            else ""
        )
    except UnicodeDecodeError:
        if attributes_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(attributes_handle)
        errors.raise_for_status(4)
    if attributes_handle == ffi.NULL:
        return GetAttributesResponse(unique_identifier, None)
    try:
        attributes = ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", attributes_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(attributes_handle)
        raise
    return GetAttributesResponse(unique_identifier, attributes)


def _add_attribute_response(item: _ClientBatchItemResponse) -> AddAttributeResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_add_attribute(
            item._pointer(), present, identifier
        )
    )
    if present[0] != 1 or identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    length = int(identifier.text_length)
    if length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        value = bytes(ffi.buffer(identifier.text_data, length)).decode("utf-8", "strict") \
            if length else ""
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    return AddAttributeResponse(value)


def _set_attribute_response(item: _ClientBatchItemResponse) -> SetAttributeResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_set_attribute(
            item._pointer(), present, identifier
        )
    )
    if present[0] != 1 or identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    length = int(identifier.text_length)
    if length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        value = (
            bytes(ffi.buffer(identifier.text_data, length)).decode("utf-8", "strict")
            if length
            else ""
        )
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    return SetAttributeResponse(value)


def _adjust_attribute_response(
    item: _ClientBatchItemResponse,
) -> AdjustAttributeResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_adjust_attribute(
            item._pointer(), present, identifier
        )
    )
    if present[0] != 1 or identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    length = int(identifier.text_length)
    if length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        value = (
            bytes(ffi.buffer(identifier.text_data, length)).decode("utf-8", "strict")
            if length
            else ""
        )
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    return AdjustAttributeResponse(value)


def _delete_attribute_response(
    item: _ClientBatchItemResponse,
) -> DeleteAttributeResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_delete_attribute(
            item._pointer(), present, identifier
        )
    )
    if present[0] != 1 or identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    length = int(identifier.text_length)
    if length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        value = (
            bytes(ffi.buffer(identifier.text_data, length)).decode("utf-8", "strict")
            if length
            else ""
        )
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    return DeleteAttributeResponse(value)


def _modify_attribute_response(
    item: _ClientBatchItemResponse,
) -> ModifyAttributeResponse:
    present = ffi.new("uint8_t *")
    identifier = ffi.new("kmipkit_unique_identifier_v1_t *")
    identifier.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    identifier.abi_version = 1
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_modify_attribute(
            item._pointer(), present, identifier
        )
    )
    if present[0] != 1 or identifier.kind != 1 or identifier.reserved != 0:
        errors.raise_for_status(4)
    length = int(identifier.text_length)
    if length and identifier.text_data == ffi.NULL:
        errors.raise_for_status(4)
    try:
        value = (
            bytes(ffi.buffer(identifier.text_data, length)).decode("utf-8", "strict")
            if length
            else ""
        )
    except UnicodeDecodeError:
        errors.raise_for_status(4)
    return ModifyAttributeResponse(value)


def _set_constraints_response(
    item: _ClientBatchItemResponse,
) -> SetConstraintsResponse:
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_set_constraints_ttlv(
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
    return SetConstraintsResponse(response_ttlv)


def _set_defaults_response(
    item: _ClientBatchItemResponse,
) -> SetDefaultsResponse:
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_set_defaults_ttlv(
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
    return SetDefaultsResponse(response_ttlv)


def _ping_response(item: _ClientBatchItemResponse) -> bool | None:
    present = ffi.new("uint8_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_ping(item._pointer(), present)
    )
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    return bool(present[0]) if present[0] == 1 else None


def _logout_response(item: _ClientBatchItemResponse) -> bool | None:
    present = ffi.new("uint8_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_logout(item._pointer(), present)
    )
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    return bool(present[0]) if present[0] == 1 else None


def _query_response(item: _ClientBatchItemResponse) -> ttlv.Structure | None:
    present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    status = lib.kmipkit_client_batch_item_response_query_ttlv(
        item._pointer(), present, response_ttlv_output
    )
    response_ttlv_handle = response_ttlv_output[0]
    if status != 0:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        _raise_abi_status(status)
    if present[0] not in (0, 1) or (present[0] == 1) != (
        response_ttlv_handle != ffi.NULL
    ):
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        errors.raise_for_status(4)
    if present[0] == 0:
        return None
    try:
        return ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", response_ttlv_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        raise


def _interop_response(item: _ClientBatchItemResponse) -> InteropResponse | None:
    present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    status = lib.kmipkit_client_batch_item_response_interop_ttlv(
        item._pointer(), present, response_ttlv_output
    )
    if status != 0:
        if response_ttlv_output[0] != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
        _raise_abi_status(status)
    response_ttlv_handle = response_ttlv_output[0]
    if present[0] not in (0, 1) or (present[0] == 1 and response_ttlv_handle == ffi.NULL):
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        errors.raise_for_status(4)
    if present[0] == 0:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
            errors.raise_for_status(4)
        return None
    try:
        response_ttlv = ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", response_ttlv_handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        raise
    return InteropResponse(response_ttlv)


def _pkcs11_owned_structure(
    item: _ClientBatchItemResponse, symbol: str
) -> ttlv.Structure | None:
    present = ffi.new("uint8_t *")
    output = ffi.new("kmipkit_ttlv_structure_t **")
    status = getattr(lib, symbol)(item._pointer(), present, output)
    handle = output[0]
    if status != 0:
        if handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(handle)
        _raise_abi_status(status)
    if present[0] not in (0, 1) or (present[0] == 1) != (handle != ffi.NULL):
        if handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(handle)
        errors.raise_for_status(4)
    if present[0] == 0:
        return None
    try:
        return ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(handle)
        raise


def _pkcs11_correlation_value(
    item: _ClientBatchItemResponse,
) -> _SecretBytes | None:
    present = ffi.new("uint8_t *")
    output = ffi.new("kmipkit_byte_span_v1_t *")
    status = lib.kmipkit_client_batch_item_response_pkcs11_correlation_value(
        item._pointer(), present, output
    )
    if status != 0:
        _raise_abi_status(status)
    span = output[0]
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    if present[0] == 0:
        if span.data != ffi.NULL or span.length != 0:
            errors.raise_for_status(4)
        return None
    if span.length > sys.maxsize or (span.length > 0 and span.data == ffi.NULL):
        errors.raise_for_status(4)
    if span.length == 0:
        return _SecretBytes(memoryview(b""))
    return _SecretBytes(ffi.buffer(span.data, int(span.length)))


def _pkcs11_response(item: _ClientBatchItemResponse) -> Pkcs11Response | None:
    response_ttlv = _pkcs11_owned_structure(
        item, "kmipkit_client_batch_item_response_pkcs11_ttlv"
    )
    if response_ttlv is None:
        return None
    owned = [response_ttlv]
    correlation_value: _SecretBytes | None = None
    try:
        interface_ttlv = _pkcs11_owned_structure(
            item, "kmipkit_client_batch_item_response_pkcs11_interface_ttlv"
        )
        if interface_ttlv is not None:
            owned.append(interface_ttlv)
        function_ttlv = _pkcs11_owned_structure(
            item, "kmipkit_client_batch_item_response_pkcs11_function_ttlv"
        )
        if function_ttlv is not None:
            owned.append(function_ttlv)
        correlation_value = _pkcs11_correlation_value(item)
        output_parameters_ttlv = _pkcs11_owned_structure(
            item, "kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv"
        )
        if output_parameters_ttlv is not None:
            owned.append(output_parameters_ttlv)
        return_code_ttlv = _pkcs11_owned_structure(
            item, "kmipkit_client_batch_item_response_pkcs11_return_code_ttlv"
        )
        if return_code_ttlv is not None:
            owned.append(return_code_ttlv)
        return Pkcs11Response(
            response_ttlv,
            interface_ttlv,
            function_ttlv,
            correlation_value,
            output_parameters_ttlv,
            return_code_ttlv,
        )
    except Exception:
        if correlation_value is not None:
            correlation_value.wipe()
        for value in reversed(owned):
            value.close()
        raise


def _log_response(item: _ClientBatchItemResponse) -> LogResponse | None:
    present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    status = lib.kmipkit_client_batch_item_response_log_ttlv(
        item._pointer(), present, response_ttlv_output
    )
    if status != 0:
        if response_ttlv_output[0] != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
        errors.raise_for_status(status)
    if present[0] > 1 or (present[0] == 1) != (response_ttlv_output[0] != ffi.NULL):
        if response_ttlv_output[0] != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
        errors.raise_for_status(4)
    if present[0] == 0:
        return None
    address = int(ffi.cast("uintptr_t", response_ttlv_output[0]))
    try:
        response_ttlv = ttlv._structure_from_native_address(address)
    except Exception:
        lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
        raise
    return LogResponse(response_ttlv)


def _validate_response(item: _ClientBatchItemResponse) -> ValidateResponse:
    present = ffi.new("uint8_t *")
    validity_indicator = ffi.new("uint32_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_validate(
            item._pointer(), present, validity_indicator
        )
    )
    if present[0] != 1:
        errors.raise_for_status(4)
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    status = lib.kmipkit_client_batch_item_response_validate_ttlv(
        item._pointer(), response_ttlv_present, response_ttlv_output
    )
    response_ttlv_handle = response_ttlv_output[0]
    if status != 0:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        _raise_abi_status(status)
    if response_ttlv_present[0] != 1 or response_ttlv_handle == ffi.NULL:
        if response_ttlv_handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_handle)
        errors.raise_for_status(4)
    address = int(ffi.cast("uintptr_t", response_ttlv_handle))
    try:
        response_ttlv = ttlv._structure_from_native_address(address)
    except Exception:
        if response_ttlv_output[0] != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(response_ttlv_output[0])
        raise
    return ValidateResponse(int(validity_indicator[0]), response_ttlv)


def _derive_key_response(
    item: _ClientBatchItemResponse,
) -> DeriveKeyResponse:
    present = ffi.new("uint8_t *")
    identifier = _initialize_unique_identifier_output()
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_derive_key(
            item._pointer(), present, identifier
        )
    )
    if present[0] == 0:
        if identifier.kind != 0:
            errors.raise_for_status(4)
        _optional_output_identifier(identifier)
        errors.raise_for_status(4)
    if present[0] != 1:
        errors.raise_for_status(4)
    unique_identifier = _optional_output_identifier(identifier)
    if unique_identifier is None:
        errors.raise_for_status(4)
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_derive_key_ttlv(
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
    return DeriveKeyResponse(unique_identifier, response_ttlv)


def _export_response(item: _ClientBatchItemResponse) -> ExportResponse | None:
    present = ffi.new("uint8_t *")
    payload_present = ffi.new("uint8_t *")
    object_type = ffi.new("uint32_t *")
    identifier = _initialize_unique_identifier_output()
    attributes_output = ffi.new("kmipkit_ttlv_structure_t **")
    any_object_output = ffi.new("kmipkit_ttlv_structure_t **")
    response_ttlv_present = ffi.new("uint8_t *")
    response_ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    owned_wrappers: list[ttlv.Structure] = []

    try:
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_export(
                item._pointer(),
                present,
                payload_present,
                object_type,
                identifier,
                attributes_output,
                any_object_output,
            )
        )
        if present[0] not in (0, 1) or payload_present[0] not in (0, 1):
            errors.raise_for_status(4)
        _raise_abi_status(
            lib.kmipkit_client_batch_item_response_export_ttlv(
                item._pointer(), response_ttlv_present, response_ttlv_output
            )
        )
        if response_ttlv_present[0] not in (0, 1) or response_ttlv_present[0] != present[0]:
            errors.raise_for_status(4)

        if present[0] == 0:
            if (
                payload_present[0] != 0
                or object_type[0] != 0
                or identifier.kind != 0
                or attributes_output[0] != ffi.NULL
                or any_object_output[0] != ffi.NULL
                or response_ttlv_output[0] != ffi.NULL
            ):
                errors.raise_for_status(4)
            return None

        if response_ttlv_output[0] == ffi.NULL:
            errors.raise_for_status(4)
        if payload_present[0] == 0:
            if (
                object_type[0] != 0
                or identifier.kind != 0
                or attributes_output[0] != ffi.NULL
                or any_object_output[0] != ffi.NULL
            ):
                errors.raise_for_status(4)
            unique_identifier = None
            raw_object_type = None
        else:
            if attributes_output[0] == ffi.NULL or any_object_output[0] == ffi.NULL:
                errors.raise_for_status(4)
            unique_identifier = _optional_output_identifier(identifier)
            if unique_identifier is None:
                errors.raise_for_status(4)
            raw_object_type = int(object_type[0])

        def take_structure(output: Any) -> ttlv.Structure | None:
            handle = output[0]
            if handle == ffi.NULL:
                return None
            value = ttlv._structure_from_native_address(
                int(ffi.cast("uintptr_t", handle))
            )
            output[0] = ffi.NULL
            owned_wrappers.append(value)
            return value

        attributes = take_structure(attributes_output)
        any_object = take_structure(any_object_output)
        response_ttlv = take_structure(response_ttlv_output)
        if response_ttlv is None or (
            payload_present[0] == 1 and (attributes is None or any_object is None)
        ):
            errors.raise_for_status(4)  # COVERAGE-EXCLUDE: each non-null handle wraps or raises.
        return ExportResponse(
            raw_object_type,
            unique_identifier,
            attributes,
            any_object,
            response_ttlv,
        )
    except Exception:
        for wrapper in owned_wrappers:
            wrapper.close()
        for output in (attributes_output, any_object_output, response_ttlv_output):
            if output[0] != ffi.NULL:
                lib.kmipkit_ttlv_structure_release(output[0])
        raise


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


def _successful_response_ttlv(
    item: _ClientBatchItemResponse, function: Any
) -> ttlv.Structure:
    present = ffi.new("uint8_t *")
    output = ffi.new("kmipkit_ttlv_structure_t **")
    _raise_abi_status(function(item._pointer(), present, output))
    handle = output[0]
    if present[0] != 1 or handle == ffi.NULL:
        if handle != ffi.NULL:
            lib.kmipkit_ttlv_structure_release(handle)
        errors.raise_for_status(4)
    try:
        return ttlv._structure_from_native_address(
            int(ffi.cast("uintptr_t", handle))
        )
    except Exception:
        lib.kmipkit_ttlv_structure_release(handle)
        raise


def _rng_retrieve_response(
    item: _ClientBatchItemResponse,
) -> RngRetrieveResponse | None:
    present = ffi.new("uint8_t *")
    data_present = ffi.new("uint8_t *")
    data = ffi.new("kmipkit_byte_span_v1_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_rng_retrieve(
            item._pointer(), present, data_present, data
        )
    )
    if present[0] == 0:
        if data_present[0] != 0:
            errors.raise_for_status(4)
        _optional_output_secret_span(data_present, data)
        return None
    if present[0] != 1 or data_present[0] != 1:
        errors.raise_for_status(4)
    secret_data = _optional_output_secret_span(data_present, data)
    if secret_data is None:
        errors.raise_for_status(4)
    try:
        response_ttlv = _successful_response_ttlv(
            item, lib.kmipkit_client_batch_item_response_rng_retrieve_ttlv
        )
    except Exception:
        secret_data.wipe()
        raise
    return RngRetrieveResponse(secret_data, response_ttlv)


def _rng_seed_response(item: _ClientBatchItemResponse) -> RngSeedResponse | None:
    present = ffi.new("uint8_t *")
    data_length_present = ffi.new("uint8_t *")
    data_length = ffi.new("int32_t *")
    _raise_abi_status(
        lib.kmipkit_client_batch_item_response_rng_seed(
            item._pointer(), present, data_length_present, data_length
        )
    )
    if present[0] == 0:
        if data_length_present[0] != 0 or data_length[0] != 0:
            errors.raise_for_status(4)
        return None
    if present[0] != 1 or data_length_present[0] != 1:
        errors.raise_for_status(4)
    response_ttlv = _successful_response_ttlv(
        item, lib.kmipkit_client_batch_item_response_rng_seed_ttlv
    )
    return RngSeedResponse(int(data_length[0]), response_ttlv)


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
    EncryptResponse | None,
    DecryptResponse | None,
    HashResponse | None,
    MacResponse | None,
    MacVerifyResponse | None,
    SignResponse | None,
    SignatureVerifyResponse | None,
]:
    if operation == 0x1F:
        return _encrypt_response(item), None, None, None, None, None, None
    if operation == 0x20:
        return None, _decrypt_response(item), None, None, None, None, None
    if operation == 0x27:
        return None, None, _hash_response(item), None, None, None, None
    if operation == 0x23:
        return None, None, None, _mac_response(item), None, None, None
    if operation == 0x24:
        return None, None, None, None, _mac_verify_response(item), None, None
    if operation == 0x21:
        return None, None, None, None, None, _sign_response(item), None
    if operation == 0x22:
        return None, None, None, None, None, None, _signature_verify_response(item)
    return None, None, None, None, None, None, None


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
    "ActivateResponse",
    "AddAttributeResponse",
    "AdjustAttributeResponse",
    "ArchiveResponse",
    "AsyncOperationMetadataV1",
    "AsyncOperationOutcome",
    "Batch",
    "BatchItemMetadataV2",
    "BatchItemResult",
    "BatchOutcome",
    "BatchResult",
    "CertifyResponse",
    "CheckResponse",
    "DelegatedLoginResponse",
    "LoginResponse",
    "DeriveKeyResponse",
    "DestroyResponse",
    "DeleteAttributeResponse",
    "ExportResponse",
    "Client",
    "CredentialEncoding",
    "Endpoint",
    "EndpointKind",
    "HashResponse",
    "GetUsageAllocationResponse",
    "ObtainLeaseResponse",
    "Pkcs11Response",
    "ImportResponse",
    "JoinSplitKeyResponse",
    "InteropResponse",
    "LogResponse",
    "GetConstraintsResponse",
    "GetAttributeListResponse",
    "GetAttributesResponse",
    "ModifyAttributeResponse",
    "SetConstraintsResponse",
    "SetAttributeResponse",
    "SetDefaultsResponse",
    "ValidateResponse",
    "GetResponse",
    "LocateResponse",
    "MacResponse",
    "MacVerifyResponse",
    "RequestOptions",
    "ReProvisionResponse",
    "RecoverResponse",
    "RegisterResponse",
    "RngRetrieveResponse",
    "RngSeedResponse",
    "SignResponse",
    "SignatureVerifyResponse",
    "TimeoutKind",
    "TimeoutLimit",
    "TransportConfig",
    "TransportInput",
    "TrustKind",
    "TrustSource",
    "UsageAllocationContext",
]
