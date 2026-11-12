"""Task-oriented KMIP client builders backed by the shared Rust C ABI."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any, Self

from . import errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import _wrap_owned_handle
from .client import (
    _NATIVE_CALL_LOCK,
    Batch,
    BatchResult,
    Client,
    RequestOptions,
    _batch_result,
    _raise_client_status,
    _request_options_record,
    _unsigned,
)
from .cryptographic_parameters import CryptographicParameters
from .typed_attributes import StandardAttribute

_MAX_CRYPTO_INPUT_BYTES = 16 * 1024 * 1024


def _wipe_crypto_stage(staged: bytearray) -> None:
    """Overwrite a mutable Python buffer owned by this binding."""
    staged[:] = bytes(len(staged))


def _client_abi_pointer(c_type: str, pointer: Any) -> Any:
    """Bridge an opaque handle into the client CFFI declaration set by address."""
    try:
        address = int(ffi.cast("uintptr_t", pointer))
    except (TypeError, ValueError):
        address = int(_base_ffi.cast("uintptr_t", pointer))
    return ffi.cast(c_type, address)


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
        args = [_client_abi_pointer("kmipkit_client_t *", client._pointer())]
        if record is not None:
            args.append(record)
        args.append(
            _client_abi_pointer("kmipkit_codec_limits_t *", limits._pointer())
        )
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
    args = [
        _client_abi_pointer("kmipkit_client_t *", client._pointer()),
        request,
        _client_abi_pointer("kmipkit_codec_limits_t *", limits._pointer()),
    ]
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

    def encrypt(self) -> EncryptBuilder:
        return EncryptBuilder(client=self._client)

    def decrypt(self) -> DecryptBuilder:
        return DecryptBuilder(client=self._client)

    def sign(self) -> SignBuilder:
        return SignBuilder(client=self._client)

    def signature_verify(self) -> SignatureVerifyBuilder:
        return SignatureVerifyBuilder(client=self._client)

    def mac(self) -> MacBuilder:
        return MacBuilder(client=self._client)

    def mac_verify(self) -> MacVerifyBuilder:
        return MacVerifyBuilder(client=self._client)

    def hash(self) -> HashBuilder:
        return HashBuilder(client=self._client)

    def get_attributes(
        self,
        unique_identifier: extensions.UniqueIdentifier | None = None,
        attribute_references: Sequence[extensions.AttributeReference] = (),
    ) -> GetAttributesBuilder:
        return GetAttributesBuilder(
            self._client, unique_identifier, attribute_references
        )

    def get_attribute_list(
        self,
        unique_identifier: extensions.UniqueIdentifier | None = None,
    ) -> GetAttributeListBuilder:
        return GetAttributeListBuilder(self._client, unique_identifier)


class _AttributeReadBuilder:
    """Shared one-exchange lifecycle for typed attribute read requests."""

    def __init__(
        self,
        client: Client,
        unique_identifier: extensions.UniqueIdentifier | None,
    ) -> None:
        if unique_identifier is not None and not (
            isinstance(unique_identifier, extensions.UniqueIdentifier)
            and unique_identifier.kind
            is extensions.UniqueIdentifierKind.TEXT_STRING
        ):
            errors.raise_for_status(4)
        self._client = client
        self._unique_identifier = unique_identifier
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def with_options(self, options: RequestOptions) -> Self:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def validate(self) -> None:
        with self.build():
            pass

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        if not isinstance(self._client, Client) or not isinstance(
            limits, ttlv.CodecLimits
        ):
            errors.raise_for_status(4)
        with Batch.create() as batch:
            with self.build() as item:
                batch.add_item(item)
            if self._asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(self._asynchronous_indicator)
            if self._options is not None:
                return self._client.execute_with_options(
                    batch, limits, self._options
                )
            return self._client.execute(batch, limits)


class GetAttributesBuilder(_AttributeReadBuilder):
    """Typed Get Attributes request preserving reference order."""

    def __init__(
        self,
        client: Client,
        unique_identifier: extensions.UniqueIdentifier | None,
        attribute_references: Sequence[extensions.AttributeReference],
    ) -> None:
        if not isinstance(attribute_references, Sequence) or isinstance(
            attribute_references, (str, bytes, bytearray)
        ):
            errors.raise_for_status(4)
        if len(attribute_references) > 100_000:
            errors.raise_for_status(5)
        references = tuple(attribute_references)
        if any(not isinstance(reference, extensions.AttributeReference)
               for reference in references):
            errors.raise_for_status(4)
        super().__init__(client, unique_identifier)
        self._attribute_references = references

    def build(self) -> extensions.ClientBatchItem:
        return extensions.client_batch_item_get_attributes(
            self._unique_identifier, self._attribute_references
        )


class GetAttributeListBuilder(_AttributeReadBuilder):
    """Typed Get Attribute List request with an optional object identifier."""

    def build(self) -> extensions.ClientBatchItem:
        return extensions.client_batch_item_get_attribute_list(
            self._unique_identifier
        )


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

    def attribute_item_at(self, index: int, item: ttlv.TtlvItem) -> SymmetricKeyBuilder:
        """Copy an explicit Item into the final Object Attributes at ``index``.

        The zero-based index includes semantic and appended attributes. Rust
        validates catalogued types and Vendor Attribute structure, and rejects
        duplicate or out-of-range positions when building the typed request.
        This method only addresses Object Attributes and never sends a request.
        """
        index = _unsigned(index, 64)
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_symmetric_key_builder_insert_attribute_item(
                self._handle, index, ffi.cast("kmipkit_ttlv_item_t *", item._pointer()),
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
        # The fluent facade uses the client CFFI declarations while the shared
        # handle wrapper is owned by the base ABI declarations. Bridge the
        # opaque pointer by address, as Client.Batch.item_at does.
        handle = _base_ffi.cast(
            "kmipkit_client_batch_item_t *", int(ffi.cast("uintptr_t", output[0]))
        )
        return _wrap_owned_handle(extensions.ClientBatchItem, handle)

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


def _stage_crypto_bytes(value: bytes | bytearray | memoryview) -> tuple[bytearray, Any]:
    if not isinstance(value, (bytes, bytearray, memoryview)):
        errors.raise_for_status(4)
    try:
        view = memoryview(value)
    except (TypeError, ValueError):
        errors.raise_for_status(4)
    if view.nbytes > _MAX_CRYPTO_INPUT_BYTES:
        errors.raise_for_status(6)
    try:
        staged = bytearray(view.cast("B"))
    except (BufferError, TypeError, ValueError):
        errors.raise_for_status(4)
    if len(staged) > _MAX_CRYPTO_INPUT_BYTES:
        _wipe_crypto_stage(staged)
        errors.raise_for_status(6)
    try:
        pointer = ffi.from_buffer("const uint8_t[]", staged) if staged else ffi.NULL
    except BaseException:
        _wipe_crypto_stage(staged)
        raise
    return staged, pointer


class _CryptoOperationBuilder:
    """Shared Python ownership and execution for Rust-owned crypto builders."""

    _creator: Any

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_crypto_operation_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(self._creator(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._options: RequestOptions | None = None

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def _set_status(self, function: Any, *args: Any) -> Self:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(function(self._handle, *args))
        return self

    def _set_bytes(self, function: Any, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        staged, pointer = _stage_crypto_bytes(value)
        try:
            span = ffi.new("kmipkit_byte_span_v1_t *")
            span.data = pointer
            span.length = len(staged)
            with _NATIVE_CALL_LOCK:
                _raise_client_status(function(self._handle, span))
        finally:
            _wipe_crypto_stage(staged)
        return self

    def _set_ttlv(self, function: Any, c_type: str, value: Any) -> Self:
        self._ensure_open()
        pointer = _client_abi_pointer(c_type, value._pointer())
        return self._set_status(function, pointer)

    def cryptographic_parameters(self, value: CryptographicParameters) -> Self:
        if not isinstance(value, CryptographicParameters):
            errors.raise_for_status(4)
        self._ensure_open()
        self._set_status(
            lib.kmipkit_crypto_operation_builder_set_cryptographic_parameters,
            _client_abi_pointer(
                "kmipkit_cryptographic_parameters_t *", value._pointer()
            ),
        )
        return self

    def cryptographic_parameters_structure(self, value: ttlv.Structure) -> Self:
        if not isinstance(value, ttlv.Structure):
            errors.raise_for_status(4)
        return self._set_ttlv(
            lib.kmipkit_crypto_operation_builder_set_cryptographic_parameters_structure,
            "kmipkit_ttlv_structure_t *",
            value,
        )

    def data(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(lib.kmipkit_crypto_operation_builder_set_data, value)

    def data_value(self, value: ttlv.Value) -> Self:
        if not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        return self._set_ttlv(
            lib.kmipkit_crypto_operation_builder_set_data_value,
            "kmipkit_ttlv_value_t *",
            value,
        )

    def correlation_value(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_correlation_value, value
        )

    def init_indicator(self, value: bool) -> Self:
        if not isinstance(value, bool):
            errors.raise_for_status(4)
        return self._set_status(
            lib.kmipkit_crypto_operation_builder_set_init_indicator,
            2 if value else 1,
        )

    def final_indicator(self, value: bool) -> Self:
        if not isinstance(value, bool):
            errors.raise_for_status(4)
        return self._set_status(
            lib.kmipkit_crypto_operation_builder_set_final_indicator,
            2 if value else 1,
        )

    def with_options(self, options: RequestOptions) -> Self:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._ensure_open()
        return self._set_status(
            lib.kmipkit_crypto_operation_builder_set_asynchronous_indicator,
            _unsigned(value, 32),
        )

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        output = ffi.new("kmipkit_client_batch_item_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_crypto_operation_builder_build_batch_item(
                    self._handle, output
                )
            )
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        handle = _base_ffi.cast(
            "kmipkit_client_batch_item_t *",
            int(ffi.cast("uintptr_t", output[0])),
        )
        return _wrap_owned_handle(extensions.ClientBatchItem, handle)

    to_batch_item = build

    def validate(self) -> None:
        """Validate the typed request locally without sending it."""
        item = self.build()
        item.close()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        self._ensure_open()
        if not isinstance(self._client, Client) or not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        output = ffi.new("kmipkit_client_batch_response_t **")
        client_pointer = _client_abi_pointer(
            "kmipkit_client_t *", self._client._pointer()
        )
        limits_pointer = _client_abi_pointer(
            "kmipkit_codec_limits_t *", limits._pointer()
        )
        with _NATIVE_CALL_LOCK:
            if self._options is None:
                status = lib.kmipkit_crypto_operation_builder_execute_v1(
                    self._handle, client_pointer, limits_pointer, output
                )
            else:
                options_record = _request_options_record(self._options)
                status = lib.kmipkit_crypto_operation_builder_execute_with_options_v1(
                    self._handle,
                    client_pointer,
                    limits_pointer,
                    options_record,
                    output,
                )
            _raise_client_status(status)
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        return _batch_result(output[0], limits)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_crypto_operation_builder_release(self._handle)
            self._handle = ffi.NULL

    def __enter__(self) -> Self:
        self._ensure_open()
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


class _KeyIdentifierInput:
    def unique_identifier(self, value: extensions.UniqueIdentifier) -> Self:
        record, owner = _identifier(value)
        self._set_status(lib.kmipkit_crypto_operation_builder_set_unique_identifier, record)
        del owner
        return self


class _CipherInput:
    def iv_counter_nonce(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_iv_counter_nonce, value
        )

    def authenticated_encryption_additional_data(
        self, value: bytes | bytearray | memoryview
    ) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data,
            value,
        )


class _DecryptInput:
    def authenticated_encryption_tag(
        self, value: bytes | bytearray | memoryview
    ) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_authenticated_encryption_tag,
            value,
        )


class _DigestInput:
    def digested_data(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_digested_data, value
        )


class _SignatureInput:
    def signature_data(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_signature_data, value
        )


class _MacInput:
    def mac_data(self, value: bytes | bytearray | memoryview) -> Self:
        return self._set_bytes(
            lib.kmipkit_crypto_operation_builder_set_mac_data, value
        )


class EncryptBuilder(_CryptoOperationBuilder, _KeyIdentifierInput, _CipherInput):
    """Rust-backed fluent builder for a typed KMIP Encrypt request."""

    _creator = staticmethod(lib.kmipkit_encrypt_builder_create)


class DecryptBuilder(_CryptoOperationBuilder, _KeyIdentifierInput, _CipherInput, _DecryptInput):
    """Rust-backed fluent builder for a typed KMIP Decrypt request."""

    _creator = staticmethod(lib.kmipkit_decrypt_builder_create)


class SignBuilder(_CryptoOperationBuilder, _KeyIdentifierInput, _DigestInput):
    """Rust-backed fluent builder for a typed KMIP Sign request."""

    _creator = staticmethod(lib.kmipkit_sign_builder_create)


class SignatureVerifyBuilder(
    _CryptoOperationBuilder, _KeyIdentifierInput, _DigestInput, _SignatureInput
):
    """Rust-backed fluent builder for a typed KMIP Signature Verify request."""

    _creator = staticmethod(lib.kmipkit_signature_verify_builder_create)


class MacBuilder(_CryptoOperationBuilder, _KeyIdentifierInput):
    """Rust-backed fluent builder for a typed KMIP MAC request."""

    _creator = staticmethod(lib.kmipkit_mac_builder_create)


class MacVerifyBuilder(_CryptoOperationBuilder, _KeyIdentifierInput, _MacInput):
    """Rust-backed fluent builder for a typed KMIP MAC Verify request."""

    _creator = staticmethod(lib.kmipkit_mac_verify_builder_create)


class HashBuilder(_CryptoOperationBuilder):
    """Rust-backed fluent builder for a typed KMIP Hash request."""

    _creator = staticmethod(lib.kmipkit_hash_builder_create)


__all__ = [
    "DecryptBuilder",
    "EncryptBuilder",
    "GetAttributeListBuilder",
    "GetAttributesBuilder",
    "HashBuilder",
    "MacBuilder",
    "MacVerifyBuilder",
    "Operation",
    "Request",
    "SignBuilder",
    "SignatureVerifyBuilder",
    "SymmetricKeyBuilder",
]
