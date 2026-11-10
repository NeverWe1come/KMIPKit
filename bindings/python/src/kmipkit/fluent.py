"""Task-oriented KMIP client builders backed by the shared Rust C ABI."""

from __future__ import annotations

from typing import Any

from . import errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._handles import _wrap_owned_handle
from .client import (
    Batch,
    BatchResult,
    Client,
    RequestOptions,
    _NATIVE_CALL_LOCK,
    _batch_result,
    _raise_client_status,
    _request_options_record,
    _unsigned,
)
from .typed_attributes import StandardAttribute


def _identifier(value: extensions.UniqueIdentifier) -> Any:
    if not isinstance(value, extensions.UniqueIdentifier):
        errors.raise_for_status(4)
    record = ffi.new("kmipkit_unique_identifier_v1_t *")
    record.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    record.abi_version = 1
    record.kind = int(value.kind)
    record.reserved = 0
    if value.kind is extensions.UniqueIdentifierKind.TEXT_STRING:
        encoded = value.text.encode("utf-8")
        owner = ffi.new("uint8_t[]", encoded)
        record.text_data, record.text_length = owner, len(encoded)
        return record, owner
    if value.kind is extensions.UniqueIdentifierKind.ENUMERATION:
        record.enumeration_value = value.number
    else:
        record.integer_value = value.number
    return record, None


def _execute_direct(client: Client, function: Any, identifier: Any,
                    limits: ttlv.CodecLimits, options: RequestOptions | None) -> BatchResult:
    if not isinstance(client, Client) or not isinstance(limits, ttlv.CodecLimits):
        errors.raise_for_status(4)
    record, owner = _identifier(identifier) if identifier is not None else (None, None)
    options_record = _request_options_record(options) if options is not None else None
    output = ffi.new("kmipkit_client_batch_response_t **")
    with _NATIVE_CALL_LOCK:
        args = [client._pointer()]
        if record is not None:
            args.append(record)
        args.extend([ffi.cast("kmipkit_codec_limits_t *", limits._pointer())])
        if options_record is not None:
            args.append(options_record)
        args.append(output)
        try:
            status = function(*args)
        except (OverflowError, TypeError, ValueError):
            errors.raise_for_status(4)
        _raise_client_status(status)
    if output[0] == ffi.NULL:
        errors.raise_for_status(7)
    return _batch_result(output[0], limits)


def _execute_direct_create(client: Client, request: Any, limits: ttlv.CodecLimits,
                           options: RequestOptions | None) -> BatchResult:
    if not isinstance(client, Client) or not isinstance(limits, ttlv.CodecLimits):
        errors.raise_for_status(4)
    output = ffi.new("kmipkit_client_batch_response_t **")
    function = (lib.kmipkit_client_create_symmetric_key_with_options_v1
                if options is not None else lib.kmipkit_client_create_symmetric_key_v1)
    args = [client._pointer(), request, ffi.cast("kmipkit_codec_limits_t *", limits._pointer())]
    if options is not None:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        args.append(_request_options_record(options))
    args.append(output)
    with _NATIVE_CALL_LOCK:
        try:
            status = function(*args)
        except (OverflowError, TypeError, ValueError):
            errors.raise_for_status(4)
        _raise_client_status(status)
    if output[0] == ffi.NULL:
        errors.raise_for_status(7)
    return _batch_result(output[0], limits)


class Request:
    """Scoped fluent namespace for the foundation lifecycle operations."""

    def __init__(self, client: Client) -> None:
        self._client = client

    def create_symmetric_key(self) -> SymmetricKeyBuilder:
        return SymmetricKeyBuilder(client=self._client)

    def get(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "get", unique_identifier)

    def destroy(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "destroy", unique_identifier)


class Operation:
    def __init__(self, client: Client, operation: str,
                 unique_identifier: extensions.UniqueIdentifier) -> None:
        _identifier(unique_identifier)
        self._client, self._operation, self._identifier = client, operation, unique_identifier
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def with_options(self, options: RequestOptions) -> Operation:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Operation:
        """Set the Request Header Asynchronous Indicator for this exchange."""
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def build(self) -> extensions.ClientBatchItem:
        """Build the existing typed Get or Destroy batch item without sending it."""
        if self._operation == "get":
            return extensions.client_batch_item_get(self._identifier)
        return extensions.client_batch_item_destroy(self._identifier)

    def to_batch_item(self) -> extensions.ClientBatchItem:
        """Alias for :meth:`build` for consistency with the Rust builder."""
        return self.build()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        with Batch.create() as batch:
            batch.add_item(self.build())
            if self._asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(self._asynchronous_indicator)
            if self._options is not None:
                return self._client.execute_with_options(batch, limits, self._options)
            return self._client.execute(batch, limits)


class SymmetricKeyBuilder:
    """Owns a native fluent builder until explicitly closed or context exit."""

    def __init__(self, *, client: Client | None = None) -> None:
        out = ffi.new("kmipkit_symmetric_key_builder_t **")
        with _NATIVE_CALL_LOCK:
            status = lib.kmipkit_symmetric_key_builder_create(out)
            _raise_client_status(status)
        if out[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = out[0]
        self._client = client
        self._algorithm = self._length = self._usage_mask = None
        self._name: str | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def _set(self, native_function: Any, value: int) -> SymmetricKeyBuilder:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(native_function(self._handle, value))
        return self

    def algorithm(self, value: int) -> SymmetricKeyBuilder:
        self._algorithm = value
        return self._set(lib.kmipkit_symmetric_key_builder_set_algorithm, value)

    def length(self, value: int) -> SymmetricKeyBuilder:
        self._length = value
        return self._set(lib.kmipkit_symmetric_key_builder_set_length, value)

    def usage_mask(self, value: int) -> SymmetricKeyBuilder:
        self._usage_mask = value
        return self._set(lib.kmipkit_symmetric_key_builder_set_usage_mask, value)

    def aes_256(self) -> SymmetricKeyBuilder:
        """Select AES with a 256-bit key; usage policy remains caller-selected."""
        from .protocol_values import CryptographicAlgorithm

        return self.algorithm(CryptographicAlgorithm.AES).length(256)

    def name(self, value: str) -> SymmetricKeyBuilder:
        self._ensure_open()
        if not isinstance(value, str):
            errors.raise_for_status(4)
        raw = value.encode("utf-8")
        staged = ffi.new("uint8_t[]", raw)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_set_name(self._handle, staged, len(raw)))
        self._name = value
        return self

    def add_attribute(self, attribute: StandardAttribute, value: ttlv.Value) -> SymmetricKeyBuilder:
        """Append a catalogued attribute after Rust checks its expected TTLV type."""
        if not isinstance(attribute, StandardAttribute) or not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_add_catalogued_attribute(
                self._handle, attribute.tag,
                ffi.cast("kmipkit_ttlv_value_t *", value._pointer()),
            ))
        return self

    def add_attribute_item(self, item: ttlv.TtlvItem) -> SymmetricKeyBuilder:
        """Append an explicitly typed expert Item through Rust validation."""
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_add_attribute_item(
                self._handle, ffi.cast("kmipkit_ttlv_item_t *", item._pointer())
            ))
        return self

    def add_vendor_attribute(
        self, vendor_identification: str, attribute_name: str, value: ttlv.TtlvItem,
    ) -> SymmetricKeyBuilder:
        """Add a structurally validated Vendor Attribute with an explicit value Item."""
        if (not isinstance(vendor_identification, str) or not isinstance(attribute_name, str)
                or not isinstance(value, ttlv.TtlvItem)):
            errors.raise_for_status(4)
        self._ensure_open()
        vendor, name = vendor_identification.encode("utf-8"), attribute_name.encode("utf-8")
        vendor_owner, name_owner = ffi.new("uint8_t[]", vendor), ffi.new("uint8_t[]", name)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_add_vendor_attribute(
                self._handle, vendor_owner, len(vendor), name_owner, len(name),
                ffi.cast("kmipkit_ttlv_item_t *", value._pointer()),
            ))
        return self

    def with_options(self, options: RequestOptions) -> SymmetricKeyBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> SymmetricKeyBuilder:
        """Set the KMIP Request Header indicator on the one-item batch."""
        self._ensure_open()
        if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        self._asynchronous_indicator = value
        return self

    def build(self) -> Any:
        self._ensure_open()
        if self._algorithm is None or self._length is None or self._usage_mask is None:
            errors.raise_for_status(4)
        output = ffi.new("kmipkit_client_batch_item_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_build_batch_item(self._handle, output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        return _wrap_owned_handle(extensions.ClientBatchItem, output[0])

    to_batch_item = build

    def execute(self, limits: ttlv.CodecLimits, client: Client | None = None) -> BatchResult:
        self._ensure_open()
        target = client or self._client
        if not isinstance(target, Client) or not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        batch = Batch.create()
        try:
            batch.add_item(self.build())
            if self._asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(self._asynchronous_indicator)
            if self._options is None:
                return target.execute(batch, limits)
            return target.execute_with_options(batch, limits, self._options)
        finally:
            batch.close()

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_symmetric_key_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> SymmetricKeyBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


__all__ = ["Operation", "Request", "SymmetricKeyBuilder"]
