"""Task-oriented KMIP client builders backed by the shared Rust C ABI."""

from __future__ import annotations

from collections.abc import Callable, Sequence
from typing import TYPE_CHECKING, Any, Self

from . import attribute_operations, errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import NativeHandle, _wrap_owned_handle
from ._protocol_version import ProtocolVersion
from .client import (
    _NATIVE_CALL_LOCK,
    Batch,
    BatchItemResult,
    BatchResult,
    Client,
    RequestOptions,
    _batch_result,
    _raise_client_status,
    _request_options_record,
    _unsigned,
)
from .credentials import SecretBytes, SecretText
from .cryptographic_parameters import CryptographicParameters
from .typed_attributes import StandardAttribute

_MAX_CRYPTO_INPUT_BYTES = 16 * 1024 * 1024

if TYPE_CHECKING:
    from .attribute_operations import AddAttributeRequest, DeleteAttributeRequest, SetAttributeRequest


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


def _signed_i32_bits(value: int) -> int:
    """Accept signed integers or raw uint32 bit patterns for KMIP bitmasks."""
    if not isinstance(value, int) or isinstance(value, bool):
        errors.raise_for_status(4)
    if -(1 << 31) <= value <= (1 << 31) - 1:
        return value
    if 0 <= value <= 0xFFFF_FFFF:
        return value - (1 << 32) if value >= (1 << 31) else value
    errors.raise_for_status(4)


def _identifier(value: extensions.UniqueIdentifier) -> Any:
    if not isinstance(value, extensions.UniqueIdentifier):
        errors.raise_for_status(4)
    record = ffi.new("kmipkit_unique_identifier_v1_t *")
    record.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    record.abi_version = 1
    record.kind = int(value.kind)
    record.reserved = 0
    if value.kind in (
        extensions.UniqueIdentifierKind.TEXT_STRING,
        extensions.UniqueIdentifierKind.IDENTIFIER,
    ):
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
    record, owner = _identifier(identifier)
    options_record = _request_options_record(options) if options is not None else None
    output = ffi.new("kmipkit_client_batch_response_t **")
    with _NATIVE_CALL_LOCK:
        args = [_client_abi_pointer("kmipkit_client_t *", client._pointer())]
        args.append(record if record is not None else ffi.NULL)
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


def _execute_direct_attributes(
    client: Client,
    unique_identifier: extensions.UniqueIdentifier,
    attribute_references: Sequence[extensions.AttributeReference],
    limits: ttlv.CodecLimits,
    options: RequestOptions | None,
) -> BatchResult:
    if not isinstance(client, Client) or not isinstance(limits, ttlv.CodecLimits):
        errors.raise_for_status(4)
    if options is not None and not isinstance(options, RequestOptions):
        errors.raise_for_status(4)
    if (
        not isinstance(unique_identifier, extensions.UniqueIdentifier)
        or unique_identifier.kind
        not in (
            extensions.UniqueIdentifierKind.TEXT_STRING,
            extensions.UniqueIdentifierKind.IDENTIFIER,
        )
    ):
        errors.raise_for_status(4)
    if not isinstance(attribute_references, Sequence) or isinstance(
        attribute_references, (str, bytes, bytearray)
    ):
        errors.raise_for_status(4)
    references = extensions._bounded_snapshot(
        attribute_references, extensions._MAX_ATTRIBUTE_REFERENCES
    )
    if any(
        not isinstance(reference, extensions.AttributeReference)
        for reference in references
    ):
        errors.raise_for_status(4)

    with extensions._staging_buffer_scope() as staging_buffers:
        identifier_record, identifier_owner = _identifier(unique_identifier)
        staging_buffers.append(identifier_owner)
        native_references, reference_count = (
            extensions._client_native_attribute_reference_array(
                references, staging_buffers
            )
        )
        options_record = (
            _request_options_record(options) if options is not None else None
        )
        output = ffi.new("kmipkit_client_batch_response_t **")
        function = (
            lib.kmipkit_client_get_attributes_with_options_v1
            if options is not None
            else lib.kmipkit_client_get_attributes_v1
        )
        arguments = [
            _client_abi_pointer("kmipkit_client_t *", client._pointer()),
            identifier_record if identifier_record is not None else ffi.NULL,
            native_references if native_references is not None else ffi.NULL,
            reference_count,
            _client_abi_pointer("kmipkit_codec_limits_t *", limits._pointer()),
        ]
        if options_record is not None:
            arguments.append(options_record)
        arguments.append(output)
        with _NATIVE_CALL_LOCK:
            try:
                status = function(*arguments)
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


def _resolve_limits(
    client: Client | None, limits: ttlv.CodecLimits | None
) -> ttlv.CodecLimits:
    if not isinstance(client, Client):
        errors.raise_for_status(4)
    if limits is not None:
        if not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        return limits
    return client.codec_limits


def _check_result_or_raise(item: BatchItemResult) -> BatchItemResult:
    if item.result_status != 0:
        err = errors.ServerResultError.from_item_result(item)
        item.close()
        raise err
    return item


def _execute_single_batch(
    client: Client,
    build: Callable[[], extensions.ClientBatchItem],
    limits: ttlv.CodecLimits | None,
    configured_options: RequestOptions | None,
    asynchronous_indicator: int | None,
    options: RequestOptions | None = None,
) -> BatchResult:
    resolved_limits = _resolve_limits(client, limits)
    if options is not None and not isinstance(options, RequestOptions):
        errors.raise_for_status(4)
    with Batch.create() as batch:
        with build() as item:
            batch.add_item(item)
        if asynchronous_indicator is not None:
            batch.set_asynchronous_indicator(asynchronous_indicator)
        selected_options = options if options is not None else configured_options
        return (
            client.execute_with_options(batch, resolved_limits, selected_options)
            if selected_options is not None
            else client.execute(batch, resolved_limits)
        )


def _execute_single_item(
    client: Client,
    build: Callable[[], extensions.ClientBatchItem],
    limits: ttlv.CodecLimits | None,
    configured_options: RequestOptions | None,
    asynchronous_indicator: int | None,
    options: RequestOptions | None = None,
) -> Any:
    """Execute one typed item, returning ``BatchResult`` when ``limits`` is passed or ``BatchItemResult`` otherwise."""
    batch_result = _execute_single_batch(
        client, build, limits, configured_options, asynchronous_indicator, options
    )
    if limits is not None:
        return batch_result
    with batch_result:
        return _check_result_or_raise(batch_result._take_single_item())


def _execute_single_outcome(
    client: Client,
    build: Callable[[], extensions.ClientBatchItem],
    limits: ttlv.CodecLimits | None,
    configured_options: RequestOptions | None,
    asynchronous_indicator: int | None,
    options: RequestOptions | None = None,
) -> BatchItemResult:
    """Execute one typed item and return its single ``BatchItemResult`` without raising on server rejection/Pending."""
    batch_result = _execute_single_batch(
        client, build, limits, configured_options, asynchronous_indicator, options
    )
    with batch_result:
        return batch_result._take_single_item()


def _build_fluent_batch_item(
    handle: Any, native_function: Any
) -> extensions.ClientBatchItem:
    output = ffi.new("kmipkit_client_batch_item_t **")
    with _NATIVE_CALL_LOCK:
        _raise_client_status(native_function(handle, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(7)
    # The fluent facade and shared handle wrapper use separate CFFI declarations.
    native_handle = _base_ffi.cast(
        "kmipkit_client_batch_item_t *", int(ffi.cast("uintptr_t", output[0]))
    )
    return _wrap_owned_handle(extensions.ClientBatchItem, native_handle)


def _execute_fluent_builder(
    builder: Any,
    limits: ttlv.CodecLimits | None,
    client: Client | None,
) -> Any:
    builder._ensure_open()
    target = client or builder._client
    if not isinstance(target, Client):
        errors.raise_for_status(4)
    resolved_limits = _resolve_limits(target, limits)
    batch = Batch.create()
    try:
        batch.add_item(builder.build())
        if builder._asynchronous_indicator is not None:
            batch.set_asynchronous_indicator(builder._asynchronous_indicator)
        batch_result = (
            target.execute(batch, resolved_limits)
            if builder._options is None
            else target.execute_with_options(batch, resolved_limits, builder._options)
        )
    finally:
        batch.close()
    if limits is not None:
        return batch_result
    with batch_result:
        return _check_result_or_raise(batch_result._take_single_item())


def _execute_fluent_builder_outcome(
    builder: Any,
    limits: ttlv.CodecLimits | None,
    client: Client | None,
) -> BatchItemResult:
    builder._ensure_open()
    target = client or builder._client
    if not isinstance(target, Client):
        errors.raise_for_status(4)
    resolved_limits = _resolve_limits(target, limits)
    batch = Batch.create()
    try:
        batch.add_item(builder.build())
        if builder._asynchronous_indicator is not None:
            batch.set_asynchronous_indicator(builder._asynchronous_indicator)
        batch_result = (
            target.execute(batch, resolved_limits)
            if builder._options is None
            else target.execute_with_options(batch, resolved_limits, builder._options)
        )
    finally:
        batch.close()
    with batch_result:
        return batch_result._take_single_item()


def _normalize_identifier(
    value: extensions.UniqueIdentifier | str | None,
    protocol_version: ProtocolVersion | None = None,
) -> extensions.UniqueIdentifier | None:
    if value is None:
        return None
    if isinstance(value, str):
        if protocol_version is not None and protocol_version.major >= 3:
            return extensions.UniqueIdentifier.identifier(value)
        return extensions.UniqueIdentifier.text_string(value)
    if isinstance(value, extensions.UniqueIdentifier):
        return value
    errors.raise_for_status(4)


def _validated_item(item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
    try:
        report = item.validation_report()
        try:
            if not report.is_valid:
                errors.raise_for_status(4)
            return item
        finally:
            report.close()
    except BaseException:
        item.close()
        raise


class Request:
    """Scoped fluent namespace for the foundation lifecycle operations."""

    def __init__(self, client: Client | None = None) -> None:
        self._client = client

    def create_symmetric_key(self) -> SymmetricKeyBuilder:
        return SymmetricKeyBuilder(client=self._client)

    def create(self) -> CreateBuilder:
        return CreateBuilder(client=self._client)

    def ping(self) -> PingBuilder:
        return PingBuilder(self._client)

    def create_key_pair(self) -> CreateKeyPairBuilder:
        return CreateKeyPairBuilder(client=self._client)

    def create_group(self, attributes: Any) -> CreateGroupBuilder:
        return CreateGroupBuilder(self._client, attributes)

    def create_credential(
        self, attributes: Any, credential_object: CreateCredentialObject,
    ) -> CreateCredentialBuilder:
        return CreateCredentialBuilder(self._client, attributes, credential_object)

    def create_user(self, attributes: Any) -> CreateUserBuilder:
        return CreateUserBuilder(self._client, attributes)

    def deactivate(self, unique_identifier: extensions.UniqueIdentifier) -> DeactivateBuilder:
        return DeactivateBuilder(self._client, unique_identifier)

    def decapsulate(
        self, unique_identifier: extensions.UniqueIdentifier,
        ciphertext: bytes | bytearray | memoryview,
    ) -> DecapsulateBuilder:
        return DecapsulateBuilder(self._client, unique_identifier, ciphertext)

    def encapsulate(self, unique_identifier: extensions.UniqueIdentifier) -> EncapsulateBuilder:
        return EncapsulateBuilder(self._client, unique_identifier)

    def obliterate(self, unique_identifier: extensions.UniqueIdentifier) -> ObliterateBuilder:
        return ObliterateBuilder(self._client, unique_identifier)

    def create_split_key(
        self,
        object_type: int | None = None,
        parts: int | None = None,
        threshold: int | None = None,
        method: int | None = None,
    ) -> CreateSplitKeyBuilder:
        return CreateSplitKeyBuilder(
            self._client,
            object_type=object_type,
            parts=parts,
            threshold=threshold,
            method=method,
        )

    def register(
        self,
        object: Any = None,
        object_type: int | None = None,
    ) -> RegisterBuilder:
        return RegisterBuilder(self._client, object=object, object_type=object_type)

    def activate(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> ActivateBuilder:
        return ActivateBuilder(self._client, unique_identifier)

    def archive(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> ArchiveBuilder:
        return ArchiveBuilder(self._client, unique_identifier)

    def recover(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> RecoverBuilder:
        return RecoverBuilder(self._client, unique_identifier)

    def rekey(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> RekeyBuilder:
        return RekeyBuilder(self._client, unique_identifier)

    def rekey_key_pair(
        self,
        private_key_unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> RekeyKeyPairBuilder:
        return RekeyKeyPairBuilder(self._client, private_key_unique_identifier)

    def recertify(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> RecertifyBuilder:
        return RecertifyBuilder(self._client, unique_identifier)

    def set_attribute(self) -> SetAttributeBuilder:
        return SetAttributeBuilder(client=self._client)

    def add_attribute(self) -> AddAttributeBuilder:
        return AddAttributeBuilder(client=self._client)

    def delete_attribute(self) -> DeleteAttributeBuilder:
        return DeleteAttributeBuilder(client=self._client)

    def locate(
        self,
        attributes: Any,
    ) -> LocateBuilder:
        return LocateBuilder(self._client, attributes)

    def revoke(
        self,
        first: Any,
        second: Any = None,
    ) -> RevokeBuilder:
        if isinstance(first, extensions.UniqueIdentifier):
            if isinstance(second, extensions.RevocationReason):
                builder = RevokeBuilder(self._client, second.raw_code, second.message)
            elif isinstance(second, int):
                builder = RevokeBuilder(self._client, second, None)
            else:
                errors.raise_for_status(4)
            return builder.unique_identifier(first)
        if isinstance(first, extensions.RevocationReason):
            return RevokeBuilder(self._client, first.raw_code, first.message)
        return RevokeBuilder(self._client, first, second)

    def get(self, unique_identifier: extensions.UniqueIdentifier | str) -> Operation:
        return Operation(self._client, "get", unique_identifier)

    def get_request(self) -> GetBuilder:
        """Start a Get request with all selectors initially unset."""
        return GetBuilder(self._client)

    def destroy(self, unique_identifier: extensions.UniqueIdentifier | str) -> Operation:
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

    def query(self) -> QueryBuilder:
        return QueryBuilder(self._client)

    def discover_versions(self) -> DiscoverVersionsBuilder:
        return DiscoverVersionsBuilder(self._client)

    def check(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> CheckBuilder:
        return CheckBuilder(self._client, unique_identifier)

    def export(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> ExportBuilder:
        return ExportBuilder(self._client, unique_identifier)

    def import_object(self) -> ImportBuilder:
        return ImportBuilder(self._client)

    def rng_retrieve(self, data_length: int | None = None) -> RngRetrieveBuilder:
        return RngRetrieveBuilder(self._client, data_length)

    def rng_seed(self, data: bytes | bytearray | memoryview | None = None) -> RngSeedBuilder:
        return RngSeedBuilder(self._client, data)

    def certify(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> CertifyBuilder:
        return CertifyBuilder(self._client, unique_identifier)

    def derive_key(self) -> DeriveKeyBuilder:
        return DeriveKeyBuilder(self._client)

    def validate(self) -> ValidateBuilder:
        return ValidateBuilder(self._client)

    def join_split_key(self) -> JoinSplitKeyBuilder:
        return JoinSplitKeyBuilder(self._client)

    def obtain_lease(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> ObtainLeaseBuilder:
        return ObtainLeaseBuilder(self._client, unique_identifier)

    def get_usage_allocation(
        self,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
        usage_limits_count: int | None = None,
    ) -> GetUsageAllocationBuilder:
        return GetUsageAllocationBuilder(self._client, unique_identifier, usage_limits_count)

    def modify_attribute(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> ModifyAttributeBuilder:
        return ModifyAttributeBuilder(self._client, unique_identifier)

    def adjust_attribute(
        self, unique_identifier: extensions.UniqueIdentifier | str | None = None
    ) -> AdjustAttributeBuilder:
        return AdjustAttributeBuilder(self._client, unique_identifier)

    def get_constraints(self) -> GetConstraintsBuilder:
        return GetConstraintsBuilder(self._client)

    def cancel(
        self, correlation_value: bytes | bytearray | memoryview | None = None
    ) -> CancelBuilder:
        return CancelBuilder(self._client, correlation_value)

    def poll(
        self, correlation_value: bytes | bytearray | memoryview | None = None
    ) -> PollBuilder:
        return PollBuilder(self._client, correlation_value)

    def process(
        self, correlation_value: bytes | bytearray | memoryview | None = None
    ) -> ProcessBuilder:
        return ProcessBuilder(self._client, correlation_value)

    def query_async_requests(self) -> QueryAsyncRequestsBuilder:
        return QueryAsyncRequestsBuilder(self._client)

    def login(self) -> LoginBuilder:
        return LoginBuilder(self._client)

    def logout(self, ticket: ttlv.Structure | None = None) -> LogoutBuilder:
        return LogoutBuilder(self._client, ticket)

    def delegated_login(
        self, rights: ttlv.Structure | None = None
    ) -> DelegatedLoginBuilder:
        return DelegatedLoginBuilder(self._client, rights)

    def set_endpoint_role(
        self, endpoint_role: int | None = None, vendor_identification: str | None = None
    ) -> SetEndpointRoleBuilder:
        builder = SetEndpointRoleBuilder(self._client, endpoint_role)
        if vendor_identification is not None:
            builder.vendor_identification(vendor_identification)
        return builder

    def pkcs11(self, function: ttlv.Structure | None = None) -> Pkcs11Builder:
        return Pkcs11Builder(self._client, function)

    def interop(
        self,
        interop_function: int | None = None,
        interop_identifier: str | bytes | bytearray | memoryview | None = None,
    ) -> InteropBuilder:
        return InteropBuilder(self._client, interop_function, interop_identifier)

    def reprovision(self) -> ReProvisionBuilder:
        return ReProvisionBuilder(self._client)

    def set_defaults(
        self, defaults_information: ttlv.Structure | None = None
    ) -> SetDefaultsBuilder:
        builder = SetDefaultsBuilder(self._client)
        if defaults_information is not None:
            builder.defaults_information(defaults_information)
        return builder

    def set_constraints(
        self, constraints: ttlv.Structure | None = None
    ) -> SetConstraintsBuilder:
        return SetConstraintsBuilder(self._client, constraints)

    def log(
        self, log_message: str | bytes | bytearray | memoryview | None = None
    ) -> LogBuilder:
        return LogBuilder(self._client, log_message)


class LocateBuilder:
    """Typed KMIP 2.1 Locate request with an explicit validated Attributes filter."""

    def __init__(
        self,
        client: Client,
        attributes: attribute_operations.AttributeSet,
    ) -> None:
        if not isinstance(attributes, attribute_operations.AttributeSet):
            errors.raise_for_status(4)
        self._client = client
        self._attributes: ttlv.Structure | None = attributes.as_items()
        self._maximum_items: int | None = None
        self._offset_items: int | None = None
        self._storage_status_mask: int | None = None
        self._object_group_member: int | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def maximum_items(self, value: int) -> LocateBuilder:
        self._maximum_items = extensions._integer_range(
            value, -(1 << 31), (1 << 31) - 1
        )
        return self

    def offset_items(self, value: int) -> LocateBuilder:
        self._offset_items = extensions._integer_range(
            value, -(1 << 31), (1 << 31) - 1
        )
        return self

    def storage_status_mask(self, value: int) -> LocateBuilder:
        self._storage_status_mask = _unsigned(value, 32)
        return self

    def object_group_member(self, value: int) -> LocateBuilder:
        self._object_group_member = _unsigned(value, 32)
        return self

    def with_options(self, options: RequestOptions) -> LocateBuilder:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> LocateBuilder:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        with self.build():
            pass

    def build(self) -> extensions.ClientBatchItem:
        if not isinstance(self._attributes, ttlv.Structure):
            errors.raise_for_status(4)
        return extensions.client_batch_item_locate(
            self._attributes,
            maximum_items=self._maximum_items,
            offset_items=self._offset_items,
            storage_status_mask=self._storage_status_mask,
            object_group_member=self._object_group_member,
        )

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def execute(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> Any:
        return _execute_single_item(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_outcome(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> BatchItemResult:
        return _execute_single_outcome(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_with_options(
        self,
        limits: ttlv.CodecLimits,
        options: RequestOptions,
    ) -> BatchResult:
        return self.execute(limits, options)

    def close(self) -> None:
        if self._attributes is not None:
            self._attributes.close()
            self._attributes = None

    def __enter__(self) -> LocateBuilder:
        if self._attributes is None:
            errors.raise_for_status(4)
        return self

    def __exit__(self, exc_type: Any, exc: Any, traceback: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


class RevokeBuilder:
    """Typed KMIP 2.1 Revoke request preserving raw reason and optional fields."""

    def __init__(
        self,
        client: Client,
        reason_code: int,
        revocation_message: str | None = None,
    ) -> None:
        self._client = client
        self._reason_code = _unsigned(reason_code, 32)
        if revocation_message is not None and not isinstance(revocation_message, str):
            errors.raise_for_status(4)
        self._revocation_message = revocation_message
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._compromise_occurrence_date: int | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def unique_identifier(
        self,
        value: extensions.UniqueIdentifier | None,
    ) -> RevokeBuilder:
        if value is not None and not isinstance(value, extensions.UniqueIdentifier):
            errors.raise_for_status(4)
        self._unique_identifier = value
        return self

    def revocation_message(self, value: str | None) -> RevokeBuilder:
        if value is not None and not isinstance(value, str):
            errors.raise_for_status(4)
        self._revocation_message = value
        return self

    def compromise_occurrence_date(self, value: int | None) -> RevokeBuilder:
        self._compromise_occurrence_date = (
            None if value is None else extensions._integer_range(
                value, -(1 << 63), (1 << 63) - 1
            )
        )
        return self

    def with_options(self, options: RequestOptions) -> RevokeBuilder:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> RevokeBuilder:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        with self.build():
            pass

    def build(self) -> extensions.ClientBatchItem:
        reason = extensions.RevocationReason.create(
            self._reason_code, self._revocation_message
        )
        try:
            request = extensions.RevokeRequest.create(
                self._unique_identifier,
                reason,
                self._compromise_occurrence_date,
            )
            try:
                return extensions.client_batch_item_revoke(request)
            finally:
                request.close()
        finally:
            reason.close()

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def execute(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> Any:
        return _execute_single_item(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_outcome(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> BatchItemResult:
        return _execute_single_outcome(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_with_options(
        self,
        limits: ttlv.CodecLimits,
        options: RequestOptions,
    ) -> BatchResult:
        return self.execute(limits, options)


class PingBuilder:
    """Build and execute the empty client-initiated KMIP 2.1 Ping request."""

    def __init__(self, client: Client | None = None) -> None:
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def with_options(self, options: RequestOptions) -> PingBuilder:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> PingBuilder:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        with self.build():
            pass

    def build(self) -> extensions.ClientBatchItem:
        return extensions.client_batch_item_ping()

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def execute(
        self,
        limits: ttlv.CodecLimits | None = None,
        *,
        client: Client | None = None,
        options: RequestOptions | None = None,
    ) -> Any:
        target = client or self._client
        return _execute_single_item(
            target,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_outcome(
        self,
        limits: ttlv.CodecLimits | None = None,
        *,
        client: Client | None = None,
        options: RequestOptions | None = None,
    ) -> BatchItemResult:
        target = client or self._client
        return _execute_single_outcome(
            target,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )


class CreateBuilder:
    """Owns a Rust-backed builder for one typed KMIP Create request."""

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_create_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_builder_create(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._object_type: int | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def object_type(self, value: int) -> CreateBuilder:
        self._ensure_open()
        object_type = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_create_builder_set_object_type(self._handle, object_type)
            )
        self._object_type = object_type
        return self

    def attributes(self, value: attribute_operations.AttributeSet) -> CreateBuilder:
        if not isinstance(value, attribute_operations.AttributeSet):
            errors.raise_for_status(4)
        self._ensure_open()
        with value.as_items() as items:
            with _NATIVE_CALL_LOCK:
                _raise_client_status(lib.kmipkit_create_builder_set_attributes(
                    self._handle,
                    _client_abi_pointer("kmipkit_ttlv_structure_t *", items._pointer()),
                ))
        return self

    def protection_storage_masks(self, masks: ttlv.Structure | None) -> CreateBuilder:
        if masks is not None and not isinstance(masks, ttlv.Structure):
            errors.raise_for_status(4)
        self._ensure_open()
        pointer = (
            ffi.NULL if masks is None else
            _client_abi_pointer("kmipkit_ttlv_structure_t *", masks._pointer())
        )
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_builder_set_protection_storage_masks(
                self._handle, pointer
            ))
        return self

    def with_options(self, options: RequestOptions) -> CreateBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def with_asynchronous_indicator(self, value: int) -> CreateBuilder:
        self._ensure_open()
        indicator = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_builder_set_asynchronous_indicator(
                self._handle, indicator
            ))
        self._asynchronous_indicator = indicator
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        if self._object_type is None:
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_builder_validate(self._handle))

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        self.validate()
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_create_builder_build_batch_item
        )

    to_batch_item = build

    def execute(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def execute_one(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> BatchItemResult:
        """Execute Create and return the owned result, including rejection or Pending."""
        return _execute_fluent_builder_outcome(self, limits, client)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_create_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> CreateBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


def _require_kmip30_client(client: Any) -> None:
    if not isinstance(client, Client):
        errors.raise_for_status(4)


class _Kmip30OperationBuilder:
    """Shared ownership and execution for opt-in KMIP 3.0 typed operations."""

    def __init__(self, client: Client, handle: Any) -> None:
        if not isinstance(client, Client) or handle == ffi.NULL:
            errors.raise_for_status(4)
        self._client = client
        self._handle = handle
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            errors.raise_for_status(4)

    def with_options(self, options: RequestOptions) -> Self:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            status = lib.kmipkit_kmip30_operation_builder_set_asynchronous_indicator(
                self._handle, self._asynchronous_indicator
            )
            _raise_client_status(status)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_kmip30_operation_builder_validate(self._handle))

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        output = ffi.new("kmipkit_client_batch_item_t **")
        with _NATIVE_CALL_LOCK:
            status = lib.kmipkit_kmip30_operation_builder_build_batch_item(self._handle, output)
            _raise_client_status(status)
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        handle = _base_ffi.cast(
            "kmipkit_client_batch_item_t *", int(ffi.cast("uintptr_t", output[0]))
        )
        return _wrap_owned_handle(extensions.ClientBatchItem, handle)

    to_batch_item = build

    def execute(self, limits: ttlv.CodecLimits | None = None) -> Any:
        self._ensure_open()
        return _execute_single_item(
            self._client, self.build, limits, self._options,
            self._asynchronous_indicator,
        )

    def execute_outcome(self, limits: ttlv.CodecLimits | None = None) -> BatchItemResult:
        self._ensure_open()
        return _execute_single_outcome(
            self._client, self.build, limits, self._options,
            self._asynchronous_indicator,
        )

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_kmip30_operation_builder_release(self._handle)
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


def _attribute_set_handle(attributes: Any) -> Any:
    from .attribute_operations import AttributeSet

    if not isinstance(attributes, AttributeSet):
        errors.raise_for_status(4)
    return _client_abi_pointer("kmipkit_attribute_set_t *", attributes._pointer())


def _new_create_credential_object(function: Any, *arguments: Any) -> CreateCredentialObject:
    output = ffi.new("kmipkit_create_credential_object_t **")
    with _NATIVE_CALL_LOCK:
        _raise_client_status(function(*arguments, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(7)
    handle = _base_ffi.cast(
        "kmipkit_create_credential_object_t *", int(ffi.cast("uintptr_t", output[0]))
    )
    return _wrap_owned_handle(CreateCredentialObject, handle)


def _credential_span(value: bytes | bytearray | memoryview) -> tuple[Any, Any, bytearray]:
    staged, pointer = _stage_crypto_bytes(value)
    span = ffi.new("kmipkit_byte_span_v1_t *", {"data": pointer, "length": len(staged)})
    return span, pointer, staged


def _credential_text_span(value: str) -> tuple[Any, Any, bytearray]:
    if not isinstance(value, str):
        errors.raise_for_status(4)
    try:
        staged = bytearray(value.encode("utf-8", "strict"))
    except UnicodeError:
        errors.raise_for_status(4)
    if len(staged) > _MAX_CRYPTO_INPUT_BYTES:
        _wipe_crypto_stage(staged)
        errors.raise_for_status(6)
    pointer = ffi.from_buffer("const uint8_t[]", staged) if staged else ffi.NULL
    span = ffi.new("kmipkit_byte_span_v1_t *", {"data": pointer, "length": len(staged)})
    return span, pointer, staged


def _secret_bytes_copy(value: SecretBytes | None) -> bytearray | None:
    if value is None:
        return None
    if not isinstance(value, SecretBytes):
        errors.raise_for_status(4)
    return value.with_bytes(lambda view: bytearray(view))


def _secret_text_copy(value: SecretText | None) -> bytearray | None:
    if value is None:
        return None
    if not isinstance(value, SecretText):
        errors.raise_for_status(4)
    return value.with_str(lambda text: bytearray(text.encode("utf-8", "strict")))


def _staged_secret_span(staged: bytearray) -> Any:
    pointer = ffi.from_buffer("const uint8_t[]", staged) if staged else ffi.NULL
    return ffi.new("kmipkit_byte_span_v1_t *", {"data": pointer, "length": len(staged)})


class CreateCredentialObject(NativeHandle):
    """One closed, zeroizing KMIP Create Credential object alternative."""

    _release_name = "kmipkit_create_credential_object_release"

    @classmethod
    def certificate(cls, certificate: ttlv.TtlvItem) -> CreateCredentialObject:
        if not isinstance(certificate, ttlv.TtlvItem):
            errors.raise_for_status(4)
        pointer = extensions._client_native_item(certificate)
        return _new_create_credential_object(
            lib.kmipkit_create_credential_object_certificate, pointer
        )

    @classmethod
    def password(
        cls, *, password: SecretText | None = None,
        password_salt: SecretBytes | None = None,
        password_salt_algorithm: int | None = None,
        salted_password: SecretBytes | None = None,
        iteration_count: int | None = None,
    ) -> CreateCredentialObject:
        staged: list[bytearray] = []
        spans: list[Any] = []
        try:
            for value, is_text in ((password, True), (password_salt, False),
                                   (salted_password, False)):
                if value is None:
                    spans.append(ffi.NULL)
                    continue
                v = _secret_text_copy(value) if is_text else _secret_bytes_copy(value)
                assert v is not None
                staged.append(v)
                spans.append(_staged_secret_span(v))
            algorithm = None if password_salt_algorithm is None else _unsigned(password_salt_algorithm, 32)
            iterations = None if iteration_count is None else _unsigned(iteration_count, 31)
            return _new_create_credential_object(
                lib.kmipkit_create_credential_object_password,
                spans[0], spans[1], int(algorithm is not None), algorithm or 0,
                spans[2], int(iterations is not None), iterations or 0,
            )
        finally:
            for owner in staged:
                _wipe_crypto_stage(owner)

    @classmethod
    def device(
        cls, *, serial_number: str | None = None, device_identifier: str | None = None,
        network_identifier: str | None = None, machine_identifier: str | None = None,
        media_identifier: str | None = None,
    ) -> CreateCredentialObject:
        staged: list[bytearray] = []
        spans: list[Any] = []
        try:
            for value in (serial_number, device_identifier, network_identifier,
                          machine_identifier, media_identifier):
                if value is None:
                    spans.append(ffi.NULL)
                else:
                    span, _pointer, owner = _credential_text_span(value)
                    spans.append(span)
                    staged.append(owner)
            return _new_create_credential_object(
                lib.kmipkit_create_credential_object_device, *spans
            )
        finally:
            for owner in staged:
                _wipe_crypto_stage(owner)

    @classmethod
    def hashed_password(
        cls, hashed_username_password: SecretBytes,
        hashed_password_username: SecretBytes, *,
        hashing_algorithm: int | None = None,
    ) -> CreateCredentialObject:
        first_owner: bytearray | None = None
        second_owner: bytearray | None = None
        try:
            first_owner = _secret_bytes_copy(hashed_username_password)
            second_owner = _secret_bytes_copy(hashed_password_username)
            if first_owner is None or second_owner is None:
                errors.raise_for_status(4)
            first = _staged_secret_span(first_owner)
            second = _staged_secret_span(second_owner)
            algorithm = None if hashing_algorithm is None else _unsigned(hashing_algorithm, 32)
            return _new_create_credential_object(
                lib.kmipkit_create_credential_object_hashed_password,
                int(algorithm is not None), algorithm or 0, first, second,
            )
        finally:
            if first_owner is not None:
                _wipe_crypto_stage(first_owner)
            if second_owner is not None:
                _wipe_crypto_stage(second_owner)

    @classmethod
    def one_time_password(
        cls, algorithm: int, *, digest: int | None = None,
        serial: str | None = None,
        seed: SecretBytes | None = None,
        interval: int | None = None,
        digits: int | None = None,
    ) -> CreateCredentialObject:
        staged: list[bytearray] = []
        try:
            serial_span = ffi.NULL
            if serial is not None:
                span, _pointer, owner = _credential_text_span(serial)
                serial_span = span
                staged.append(owner)
            seed_span = ffi.NULL
            if seed is not None:
                owner = _secret_bytes_copy(seed)
                if owner is None:
                    errors.raise_for_status(4)
                staged.append(owner)
                seed_span = _staged_secret_span(owner)
            algo = _unsigned(algorithm, 32)
            digest_val = None if digest is None else _unsigned(digest, 32)
            interval_value = None if interval is None else _unsigned(interval, 32)
            digits_value = None if digits is None else _unsigned(digits, 31)
            return _new_create_credential_object(
                lib.kmipkit_create_credential_object_one_time_password,
                algo, int(digest_val is not None), digest_val or 0,
                serial_span, seed_span,
                int(interval_value is not None), interval_value or 0,
                int(digits_value is not None), digits_value or 0,
            )
        finally:
            for owner in staged:
                _wipe_crypto_stage(owner)


class CreateCredentialBuilder(_Kmip30OperationBuilder):
    """Build a typed Create Credential from its closed object alternative."""

    def __init__(self, client: Client, attributes: Any,
                 credential_object: CreateCredentialObject) -> None:
        _require_kmip30_client(client)
        if not isinstance(credential_object, CreateCredentialObject):
            errors.raise_for_status(4)
        output = ffi.new("kmipkit_kmip30_operation_builder_t **")
        attributes_pointer = _attribute_set_handle(attributes)
        object_pointer = _client_abi_pointer(
            "kmipkit_create_credential_object_t *", credential_object._pointer()
        )
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_credential_builder_create(
                attributes_pointer, object_pointer, output
            ))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        super().__init__(client, output[0])


def _kmip30_builder(create_function: Any, *arguments: Any) -> Any:
    output = ffi.new("kmipkit_kmip30_operation_builder_t **")
    with _NATIVE_CALL_LOCK:
        _raise_client_status(create_function(*arguments, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(7)
    return output[0]


class CreateGroupBuilder(_Kmip30OperationBuilder):
    """Create a KMIP 3.0 Group with an ordered AttributeSet."""

    def __init__(self, client: Client, attributes: Any) -> None:
        _require_kmip30_client(client)
        handle = _kmip30_builder(
            lib.kmipkit_create_group_builder_create, _attribute_set_handle(attributes)
        )
        super().__init__(client, handle)


class CreateUserBuilder(_Kmip30OperationBuilder):
    """Create a KMIP 3.0 User with an ordered AttributeSet."""

    def __init__(self, client: Client, attributes: Any) -> None:
        _require_kmip30_client(client)
        handle = _kmip30_builder(
            lib.kmipkit_create_user_builder_create, _attribute_set_handle(attributes)
        )
        super().__init__(client, handle)


def _kmip30_identifier(value: extensions.UniqueIdentifier) -> tuple[Any, Any]:
    if not isinstance(value, extensions.UniqueIdentifier):
        errors.raise_for_status(4)
    return _identifier(value)


class DeactivateBuilder(_Kmip30OperationBuilder):
    """Deactivate one object, with optional typed reason and date fields."""

    def __init__(self, client: Client, unique_identifier: extensions.UniqueIdentifier) -> None:
        _require_kmip30_client(client)
        record, owner = _kmip30_identifier(unique_identifier)
        handle = _kmip30_builder(lib.kmipkit_deactivate_builder_create, record)
        del owner
        super().__init__(client, handle)

    def deactivation_reason(self, reason: ttlv.Structure) -> Self:
        self._ensure_open()
        if not isinstance(reason, ttlv.Structure):
            errors.raise_for_status(4)
        pointer = _client_abi_pointer("kmipkit_ttlv_structure_t *", reason._pointer())
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_kmip30_operation_builder_set_deactivation_reason(
                self._handle, pointer
            ))
        return self

    def deactivation_date(self, unix_seconds: int) -> Self:
        self._ensure_open()
        if not isinstance(unix_seconds, int) or isinstance(unix_seconds, bool):
            errors.raise_for_status(4)
        if not -(1 << 63) <= unix_seconds < (1 << 63):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_kmip30_operation_builder_set_deactivation_date(
                self._handle, unix_seconds
            ))
        return self


class _KemBuilder(_Kmip30OperationBuilder):
    def attributes(self, attributes: Any) -> Self:
        self._ensure_open()
        pointer = _attribute_set_handle(attributes)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_kmip30_operation_builder_set_attributes(
                self._handle, pointer
            ))
        return self

    def cryptographic_parameters(self, parameters: CryptographicParameters) -> Self:
        self._ensure_open()
        if not isinstance(parameters, CryptographicParameters):
            errors.raise_for_status(4)
        pointer = _client_abi_pointer(
            "kmipkit_cryptographic_parameters_t *", parameters._pointer()
        )
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_kmip30_operation_builder_set_cryptographic_parameters(
                self._handle, pointer
            ))
        return self


class DecapsulateBuilder(_KemBuilder):
    """Decapsulate public ciphertext without exposing the server-managed secret."""

    def __init__(
        self, client: Client, unique_identifier: extensions.UniqueIdentifier,
        ciphertext: bytes | bytearray | memoryview,
    ) -> None:
        _require_kmip30_client(client)
        record, identifier_owner = _kmip30_identifier(unique_identifier)
        staged, data_pointer = _stage_crypto_bytes(ciphertext)
        span = ffi.new("kmipkit_byte_span_v1_t *", {"data": data_pointer, "length": len(staged)})
        try:
            handle = _kmip30_builder(
                lib.kmipkit_decapsulate_builder_create, record, span
            )
        finally:
            _wipe_crypto_stage(staged)
            del identifier_owner
        super().__init__(client, handle)


class EncapsulateBuilder(_KemBuilder):
    """Encapsulate to the supplied key, returning only its identifier and public Data."""

    def __init__(self, client: Client, unique_identifier: extensions.UniqueIdentifier) -> None:
        _require_kmip30_client(client)
        record, owner = _kmip30_identifier(unique_identifier)
        handle = _kmip30_builder(lib.kmipkit_encapsulate_builder_create, record)
        del owner
        super().__init__(client, handle)


class ObliterateBuilder(_Kmip30OperationBuilder):
    """Obliterate the object identified by an explicit KMIP Unique Identifier."""

    def __init__(self, client: Client, unique_identifier: extensions.UniqueIdentifier) -> None:
        _require_kmip30_client(client)
        record, owner = _kmip30_identifier(unique_identifier)
        handle = _kmip30_builder(lib.kmipkit_obliterate_builder_create, record)
        del owner
        super().__init__(client, handle)


class SetAttributeBuilder:
    """Owns a native builder for one typed KMIP Set Attribute request."""

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_set_attribute_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_create(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None
        self._unique_identifier: extensions.UniqueIdentifier | None = None

    def unique_identifier(self, value: str) -> SetAttributeBuilder:
        self._ensure_open()
        if not isinstance(value, str):
            errors.raise_for_status(4)
        encoded = value.encode("utf-8")
        owner = ffi.new("uint8_t[]", encoded)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_set_unique_identifier(
                self._handle, owner, len(encoded)
            ))
        self._unique_identifier = extensions.UniqueIdentifier.text_string(value)
        return self

    def attribute_item(self, item: ttlv.TtlvItem) -> SetAttributeBuilder:
        self._ensure_open()
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_add_attribute_item(
                self._handle, ffi.cast("kmipkit_ttlv_item_t *", item._pointer())
            ))
        return self

    def catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> SetAttributeBuilder:
        self._ensure_open()
        if not isinstance(attribute, StandardAttribute) or not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_add_catalogued_attribute(
                self._handle, _unsigned(attribute.tag, 32),
                ffi.cast("kmipkit_ttlv_value_t *", value._pointer()),
            ))
        return self

    def vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> SetAttributeBuilder:
        self._ensure_open()
        if (not isinstance(vendor, str) or not isinstance(name, str)
                or not isinstance(value, ttlv.TtlvItem)):
            errors.raise_for_status(4)
        vendor_bytes, name_bytes = vendor.encode("utf-8"), name.encode("utf-8")
        vendor_owner, name_owner = ffi.new("uint8_t[]", vendor_bytes), ffi.new("uint8_t[]", name_bytes)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_add_vendor_attribute(
                self._handle, vendor_owner, len(vendor_bytes), name_owner, len(name_bytes),
                ffi.cast("kmipkit_ttlv_item_t *", value._pointer()),
            ))
        return self

    def with_options(self, options: RequestOptions) -> SetAttributeBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> SetAttributeBuilder:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_set_asynchronous_indicator(
                self._handle, self._asynchronous_indicator
            ))
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_set_attribute_builder_validate(self._handle))

    def build_request(self) -> SetAttributeRequest:
        """Return an independently owned typed Set Attribute request."""
        self._ensure_open()
        output = ffi.new("kmipkit_set_attribute_request_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_set_attribute_builder_build_request(self._handle, output)
            )
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        from .attribute_operations import SetAttributeRequest

        # The fluent bridge and the shared NativeHandle wrapper use distinct
        # CFFI declarations, so transfer the owned pointer through its address.
        native_handle = _base_ffi.cast(
            "kmipkit_set_attribute_request_t *",
            int(ffi.cast("uintptr_t", output[0])),
        )
        return SetAttributeRequest._from_fluent_builder(
            native_handle, self._unique_identifier
        )

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_set_attribute_builder_build_batch_item
        )

    to_batch_item = build

    def execute(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_set_attribute_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> SetAttributeBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


class AddAttributeBuilder:
    """Owns a native builder for one typed KMIP Add Attribute request."""

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_add_attribute_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_create(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None
        self._unique_identifier: extensions.UniqueIdentifier | None = None

    def unique_identifier(self, value: str) -> AddAttributeBuilder:
        self._ensure_open()
        if not isinstance(value, str):
            errors.raise_for_status(4)
        encoded = value.encode("utf-8")
        owner = ffi.new("uint8_t[]", encoded)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_set_unique_identifier(
                self._handle, owner, len(encoded)
            ))
        self._unique_identifier = extensions.UniqueIdentifier.text_string(value)
        return self

    def attribute_item(self, item: ttlv.TtlvItem) -> AddAttributeBuilder:
        self._ensure_open()
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_add_attribute_item(
                self._handle, ffi.cast("kmipkit_ttlv_item_t *", item._pointer())
            ))
        return self

    def catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> AddAttributeBuilder:
        self._ensure_open()
        if not isinstance(attribute, StandardAttribute) or not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_add_catalogued_attribute(
                self._handle, _unsigned(attribute.tag, 32),
                ffi.cast("kmipkit_ttlv_value_t *", value._pointer()),
            ))
        return self

    def vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> AddAttributeBuilder:
        self._ensure_open()
        if (not isinstance(vendor, str) or not isinstance(name, str)
                or not isinstance(value, ttlv.TtlvItem)):
            errors.raise_for_status(4)
        vendor_bytes, name_bytes = vendor.encode("utf-8"), name.encode("utf-8")
        vendor_owner, name_owner = ffi.new("uint8_t[]", vendor_bytes), ffi.new("uint8_t[]", name_bytes)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_add_vendor_attribute(
                self._handle, vendor_owner, len(vendor_bytes), name_owner, len(name_bytes),
                ffi.cast("kmipkit_ttlv_item_t *", value._pointer()),
            ))
        return self

    def with_options(self, options: RequestOptions) -> AddAttributeBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> AddAttributeBuilder:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_set_asynchronous_indicator(
                self._handle, self._asynchronous_indicator
            ))
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_add_attribute_builder_validate(self._handle))

    def build_request(self) -> AddAttributeRequest:
        """Return an independently owned typed Add Attribute request."""
        self._ensure_open()
        output = ffi.new("kmipkit_add_attribute_request_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_add_attribute_builder_build_request(self._handle, output)
            )
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        from .attribute_operations import AddAttributeRequest

        # The fluent bridge and the shared NativeHandle wrapper use distinct
        # CFFI declarations, so transfer the owned pointer through its address.
        native_handle = _base_ffi.cast(
            "kmipkit_add_attribute_request_t *",
            int(ffi.cast("uintptr_t", output[0])),
        )
        return AddAttributeRequest._from_fluent_builder(
            native_handle, self._unique_identifier
        )

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_add_attribute_builder_build_batch_item
        )

    to_batch_item = build

    def execute(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_add_attribute_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> AddAttributeBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


class DeleteAttributeBuilder:
    """Owns a native builder for one typed KMIP Delete Attribute request."""

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_delete_attribute_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_create(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._attribute_reference: extensions.AttributeReference | None = None

    def unique_identifier(self, value: str) -> DeleteAttributeBuilder:
        self._ensure_open()
        if not isinstance(value, str):
            errors.raise_for_status(4)
        encoded = value.encode("utf-8")
        owner = ffi.new("uint8_t[]", encoded)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_set_unique_identifier(
                self._handle, owner, len(encoded)
            ))
        self._unique_identifier = extensions.UniqueIdentifier.text_string(value)
        return self

    def current_attribute_item(self, item: ttlv.TtlvItem) -> DeleteAttributeBuilder:
        self._ensure_open()
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_set_current_attribute_item(
                self._handle, ffi.cast("kmipkit_ttlv_item_t *", item._pointer())
            ))
        return self

    def catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> DeleteAttributeBuilder:
        self._ensure_open()
        if not isinstance(attribute, StandardAttribute) or not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_set_catalogued_attribute(
                self._handle, _unsigned(attribute.tag, 32),
                ffi.cast("kmipkit_ttlv_value_t *", value._pointer()),
            ))
        return self

    def vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> DeleteAttributeBuilder:
        self._ensure_open()
        if (not isinstance(vendor, str) or not isinstance(name, str)
                or not isinstance(value, ttlv.TtlvItem)):
            errors.raise_for_status(4)
        vendor_bytes, name_bytes = vendor.encode("utf-8"), name.encode("utf-8")
        vendor_owner, name_owner = ffi.new("uint8_t[]", vendor_bytes), ffi.new("uint8_t[]", name_bytes)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_set_vendor_attribute(
                self._handle, vendor_owner, len(vendor_bytes), name_owner, len(name_bytes),
                ffi.cast("kmipkit_ttlv_item_t *", value._pointer()),
            ))
        return self

    def current_vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> DeleteAttributeBuilder:
        """Select one structurally valid KMIP Vendor Attribute."""
        return self.vendor_attribute(vendor, name, value)

    def attribute_reference(
        self, value: extensions.AttributeReference,
    ) -> DeleteAttributeBuilder:
        self._ensure_open()
        if not isinstance(value, extensions.AttributeReference):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            reference = extensions._client_native_attribute_reference(value, staging_buffers)
            with _NATIVE_CALL_LOCK:
                _raise_client_status(lib.kmipkit_delete_attribute_builder_set_attribute_reference(
                    self._handle, reference
                ))
        self._attribute_reference = value
        return self

    def with_options(self, options: RequestOptions) -> DeleteAttributeBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> DeleteAttributeBuilder:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_set_asynchronous_indicator(
                self._handle, self._asynchronous_indicator
            ))
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_delete_attribute_builder_validate(self._handle))

    def build_request(self) -> DeleteAttributeRequest:
        """Return an independently owned typed Delete Attribute request."""
        self._ensure_open()
        output = ffi.new("kmipkit_delete_attribute_request_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_delete_attribute_builder_build_request(self._handle, output)
            )
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        native_handle = _base_ffi.cast(
            "kmipkit_delete_attribute_request_t *",
            int(ffi.cast("uintptr_t", output[0])),
        )
        from .attribute_operations import DeleteAttributeRequest

        return _wrap_owned_handle(
            DeleteAttributeRequest, native_handle, self._unique_identifier,
            self._attribute_reference, None,
        )

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_delete_attribute_builder_build_batch_item
        )

    to_batch_item = build

    def execute(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None,
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_delete_attribute_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> DeleteAttributeBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - finalizers must not raise during shutdown.
            pass


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
            in (
                extensions.UniqueIdentifierKind.TEXT_STRING,
                extensions.UniqueIdentifierKind.IDENTIFIER,
            )
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

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        with self.build():
            pass

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def execute(self, limits: ttlv.CodecLimits | None = None) -> Any:
        resolved_limits = _resolve_limits(self._client, limits)
        with Batch.create() as batch:
            with self.build() as item:
                batch.add_item(item)
            if self._asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(self._asynchronous_indicator)
            batch_result = (
                self._client.execute_with_options(batch, resolved_limits, self._options)
                if self._options is not None
                else self._client.execute(batch, resolved_limits)
            )
        if limits is not None:
            return batch_result
        with batch_result:
            return _check_result_or_raise(batch_result._take_single_item())

    def execute_outcome(self, limits: ttlv.CodecLimits | None = None) -> BatchItemResult:
        resolved_limits = _resolve_limits(self._client, limits)
        with Batch.create() as batch:
            with self.build() as item:
                batch.add_item(item)
            if self._asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(self._asynchronous_indicator)
            batch_result = (
                self._client.execute_with_options(batch, resolved_limits, self._options)
                if self._options is not None
                else self._client.execute(batch, resolved_limits)
            )
        with batch_result:
            return batch_result._take_single_item()


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
                 unique_identifier: extensions.UniqueIdentifier | str) -> None:
        normalized = _normalize_identifier(
            unique_identifier,
            client.protocol_version if isinstance(client, Client) else None,
        )
        _identifier(normalized)
        self._client, self._operation, self._identifier = client, operation, normalized
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
        """Build an existing typed lifecycle item without sending it."""
        constructors = {
            "get": extensions.client_batch_item_get,
            "destroy": extensions.client_batch_item_destroy,
            "activate": extensions.client_batch_item_activate,
            "archive": extensions.client_batch_item_archive,
            "recover": extensions.client_batch_item_recover,
        }
        try:
            constructor = constructors[self._operation]
        except KeyError:
            errors.raise_for_status(4)
        return constructor(self._identifier)

    def to_batch_item(self) -> extensions.ClientBatchItem:
        """Alias for :meth:`build` for consistency with the Rust builder."""
        return self.build()

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        """Validate locally without sending a request."""
        self.build().close()

    def execute(self, limits: ttlv.CodecLimits | None = None) -> Any:
        return _execute_single_item(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
        )

    def execute_outcome(self, limits: ttlv.CodecLimits | None = None) -> BatchItemResult:
        return _execute_single_outcome(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
        )


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
        if isinstance(client, Client):
            self.with_protocol_version(client.protocol_version)

    def with_protocol_version(self, version: ProtocolVersion) -> SymmetricKeyBuilder:
        """Select KMIP version used to emit name attributes (`Name` vs `AlternativeName`)."""
        self._ensure_open()
        if not isinstance(version, ProtocolVersion):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_symmetric_key_builder_set_protocol_version(
                    self._handle, version.major, version.minor
                )
            )
        return self

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
        bits = _signed_i32_bits(value)
        self._set(lib.kmipkit_symmetric_key_builder_set_usage_mask, bits)
        self._usage_mask = bits
        return self

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
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_symmetric_key_builder_build_batch_item
        )

    to_batch_item = build

    def execute(self, limits: ttlv.CodecLimits | None = None, client: Client | None = None) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def execute_one(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> BatchItemResult:
        """Execute Create and return the owned result, including rejection or Pending."""
        return _execute_fluent_builder_outcome(self, limits, client)

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


class CreateKeyPairBuilder:
    """Owns a native builder for a typed KMIP Create Key Pair request."""

    def __init__(self, *, client: Client | None = None) -> None:
        output = ffi.new("kmipkit_create_key_pair_builder_t **")
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_key_pair_builder_create(output))
        if output[0] == ffi.NULL:
            errors.raise_for_status(7)
        self._handle = output[0]
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None
        if isinstance(client, Client):
            self.with_protocol_version(client.protocol_version)

    def with_protocol_version(self, version: ProtocolVersion) -> CreateKeyPairBuilder:
        """Select KMIP version used to emit name attributes (`Name` vs `AlternativeName`)."""
        self._ensure_open()
        if not isinstance(version, ProtocolVersion):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(
                lib.kmipkit_create_key_pair_builder_set_protocol_version(
                    self._handle, version.major, version.minor
                )
            )
        return self

    def _set(self, native_function: Any, value: int) -> CreateKeyPairBuilder:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(native_function(self._handle, value))
        return self

    def algorithm(self, value: int) -> CreateKeyPairBuilder:
        return self._set(
            lib.kmipkit_create_key_pair_builder_set_algorithm, _unsigned(value, 32)
        )

    def length(self, bits: int) -> CreateKeyPairBuilder:
        if (not isinstance(bits, int) or isinstance(bits, bool)
                or not -(1 << 31) <= bits < (1 << 31)):
            errors.raise_for_status(4)
        return self._set(lib.kmipkit_create_key_pair_builder_set_length, bits)

    def private_usage_mask(self, value: int) -> CreateKeyPairBuilder:
        return self._set(
            lib.kmipkit_create_key_pair_builder_set_private_usage_mask,
            _signed_i32_bits(value),
        )

    def public_usage_mask(self, value: int) -> CreateKeyPairBuilder:
        return self._set(
            lib.kmipkit_create_key_pair_builder_set_public_usage_mask,
            _signed_i32_bits(value),
        )

    def _set_name(self, function: Any, value: str) -> CreateKeyPairBuilder:
        self._ensure_open()
        if not isinstance(value, str):
            errors.raise_for_status(4)
        encoded = value.encode("utf-8")
        staged = ffi.new("uint8_t[]", encoded)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(function(self._handle, staged, len(encoded)))
        return self

    def private_name(self, value: str) -> CreateKeyPairBuilder:
        return self._set_name(lib.kmipkit_create_key_pair_builder_set_private_name, value)

    def public_name(self, value: str) -> CreateKeyPairBuilder:
        return self._set_name(lib.kmipkit_create_key_pair_builder_set_public_name, value)

    def _add_item(self, function: Any, item: ttlv.TtlvItem) -> CreateKeyPairBuilder:
        self._ensure_open()
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(function(
                self._handle, ffi.cast("kmipkit_ttlv_item_t *", item._pointer())
            ))
        return self

    def add_common_attribute_item(self, item: ttlv.TtlvItem) -> CreateKeyPairBuilder:
        return self._add_item(
            lib.kmipkit_create_key_pair_builder_add_common_attribute_item, item
        )

    def add_private_attribute_item(self, item: ttlv.TtlvItem) -> CreateKeyPairBuilder:
        return self._add_item(
            lib.kmipkit_create_key_pair_builder_add_private_attribute_item, item
        )

    def add_public_attribute_item(self, item: ttlv.TtlvItem) -> CreateKeyPairBuilder:
        return self._add_item(
            lib.kmipkit_create_key_pair_builder_add_public_attribute_item, item
        )

    def _add_catalogued_attribute(
        self, function: Any, attribute: StandardAttribute, value: ttlv.Value,
    ) -> CreateKeyPairBuilder:
        self._ensure_open()
        if not isinstance(attribute, StandardAttribute) or not isinstance(value, ttlv.Value):
            errors.raise_for_status(4)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(function(
                self._handle, _unsigned(attribute.tag, 32),
                ffi.cast("kmipkit_ttlv_value_t *", value._pointer()),
            ))
        return self

    def add_common_catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> CreateKeyPairBuilder:
        return self._add_catalogued_attribute(
            lib.kmipkit_create_key_pair_builder_add_common_catalogued_attribute,
            attribute, value,
        )

    def add_private_catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> CreateKeyPairBuilder:
        return self._add_catalogued_attribute(
            lib.kmipkit_create_key_pair_builder_add_private_catalogued_attribute,
            attribute, value,
        )

    def add_public_catalogued_attribute(
        self, attribute: StandardAttribute, value: ttlv.Value,
    ) -> CreateKeyPairBuilder:
        return self._add_catalogued_attribute(
            lib.kmipkit_create_key_pair_builder_add_public_catalogued_attribute,
            attribute, value,
        )

    def _add_vendor_attribute(
        self, function: Any, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> CreateKeyPairBuilder:
        self._ensure_open()
        if (not isinstance(vendor, str) or not isinstance(name, str)
                or not isinstance(value, ttlv.TtlvItem)):
            errors.raise_for_status(4)
        vendor_bytes = vendor.encode("utf-8")
        name_bytes = name.encode("utf-8")
        vendor_owner = ffi.new("uint8_t[]", vendor_bytes)
        name_owner = ffi.new("uint8_t[]", name_bytes)
        with _NATIVE_CALL_LOCK:
            _raise_client_status(function(
                self._handle, vendor_owner, len(vendor_bytes), name_owner, len(name_bytes),
                ffi.cast("kmipkit_ttlv_item_t *", value._pointer()),
            ))
        return self

    def add_common_vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> CreateKeyPairBuilder:
        return self._add_vendor_attribute(
            lib.kmipkit_create_key_pair_builder_add_common_vendor_attribute,
            vendor, name, value,
        )

    def add_private_vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> CreateKeyPairBuilder:
        return self._add_vendor_attribute(
            lib.kmipkit_create_key_pair_builder_add_private_vendor_attribute,
            vendor, name, value,
        )

    def add_public_vendor_attribute(
        self, vendor: str, name: str, value: ttlv.TtlvItem,
    ) -> CreateKeyPairBuilder:
        return self._add_vendor_attribute(
            lib.kmipkit_create_key_pair_builder_add_public_vendor_attribute,
            vendor, name, value,
        )

    def with_options(self, options: RequestOptions) -> CreateKeyPairBuilder:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> CreateKeyPairBuilder:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        self._ensure_open()
        with _NATIVE_CALL_LOCK:
            _raise_client_status(lib.kmipkit_create_key_pair_builder_validate(self._handle))

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _build_fluent_batch_item(
            self._handle, lib.kmipkit_create_key_pair_builder_build_batch_item
        )

    to_batch_item = build

    def execute(self, limits: ttlv.CodecLimits | None = None, client: Client | None = None) -> Any:
        return _execute_fluent_builder(self, limits, client)

    def execute_outcome(
        self, limits: ttlv.CodecLimits | None = None, client: Client | None = None
    ) -> BatchItemResult:
        return _execute_fluent_builder_outcome(self, limits, client)

    def close(self) -> None:
        if self._handle != ffi.NULL:
            with _NATIVE_CALL_LOCK:
                lib.kmipkit_create_key_pair_builder_release(self._handle)
            self._handle = ffi.NULL

    def _ensure_open(self) -> None:
        if self._handle == ffi.NULL:
            raise RuntimeError("fluent builder is closed")

    def __enter__(self) -> CreateKeyPairBuilder:
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

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        """Validate the typed request locally without sending it."""
        item = self.build()
        item.close()

    def _execute_raw(self, resolved_limits: ttlv.CodecLimits) -> BatchResult:
        output = ffi.new("kmipkit_client_batch_response_t **")
        client_pointer = _client_abi_pointer(
            "kmipkit_client_t *", self._client._pointer()
        )
        limits_pointer = _client_abi_pointer(
            "kmipkit_codec_limits_t *", resolved_limits._pointer()
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
        return _batch_result(output[0], resolved_limits)

    def execute(self, limits: ttlv.CodecLimits | None = None) -> Any:
        self._ensure_open()
        resolved_limits = _resolve_limits(self._client, limits)
        batch_result = self._execute_raw(resolved_limits)
        if limits is not None:
            return batch_result
        with batch_result:
            return _check_result_or_raise(batch_result._take_single_item())

    def execute_outcome(self, limits: ttlv.CodecLimits | None = None) -> BatchItemResult:
        self._ensure_open()
        resolved_limits = _resolve_limits(self._client, limits)
        with self._execute_raw(resolved_limits) as batch_result:
            return batch_result._take_single_item()

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


class _TypedOperationBuilder:
    """Common execution and ownership for typed lifecycle builders."""

    def __init__(self, client: Client | None = None) -> None:
        self._client = client
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None
        self._closed = False
        self._owned: list[Any] = []

    def _ensure_open(self) -> None:
        if self._closed:
            errors.raise_for_status(4)

    def _own(self, resource: Any) -> Any:
        self._ensure_open()
        if resource is not None:
            self._owned.append(resource)
        return resource

    def _discard(self, resource: Any) -> None:
        if resource is None:
            return
        if resource in self._owned:
            self._owned.remove(resource)
        try:
            resource.close()
        except Exception:
            pass

    def _replace_structure(
        self, previous: ttlv.Structure | None, value: ttlv.Structure | None
    ) -> ttlv.Structure | None:
        self._ensure_open()
        if value is not None and not isinstance(value, ttlv.Structure):
            errors.raise_for_status(4)
        copy = value.copy() if value is not None else None
        self._discard(previous)
        return self._own(copy) if copy is not None else None

    def with_options(self, options: RequestOptions) -> Self:
        self._ensure_open()
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._ensure_open()
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def _validated(self, item: extensions.ClientBatchItem) -> extensions.ClientBatchItem:
        report = item.validation_report()
        try:
            if not report.is_valid:
                item.close()
                errors.raise_for_status(4)
            return item
        finally:
            report.close()

    def validate(self) -> None:
        with self.build():
            pass

    def to_batch_item(self) -> extensions.ClientBatchItem:
        return self.build()

    def build(self) -> extensions.ClientBatchItem:
        raise NotImplementedError

    def execute(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> Any:
        self._ensure_open()
        if self._client is None:
            errors.raise_for_status(4)
        return _execute_single_item(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_outcome(
        self,
        limits: ttlv.CodecLimits | None = None,
        options: RequestOptions | None = None,
    ) -> BatchItemResult:
        self._ensure_open()
        if self._client is None:
            errors.raise_for_status(4)
        return _execute_single_outcome(
            self._client,
            self.build,
            limits,
            self._options,
            self._asynchronous_indicator,
            options,
        )

    def execute_with_options(
        self,
        limits: ttlv.CodecLimits | None,
        options: RequestOptions,
    ) -> Any:
        return self.execute(limits, options)

    def close(self) -> None:
        if not self._closed:
            self._closed = True
            for res in reversed(self._owned):
                try:
                    res.close()
                except Exception:
                    pass
            self._owned.clear()

    def __enter__(self) -> Self:
        self._ensure_open()
        return self

    def __exit__(self, exc_type: Any, exc: Any, traceback: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass


class GetBuilder(_TypedOperationBuilder):
    """Build a Get request with all optional KMIP selectors."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._identifier: extensions.UniqueIdentifier | None = None
        self._format: int | None = None
        self._wrap: int | None = None
        self._compression: int | None = None
        self._wrapping: ttlv.Structure | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier) -> GetBuilder:
        self._ensure_open()
        _identifier(value)
        self._identifier = value
        return self

    def key_format_type(self, value: int) -> GetBuilder:
        self._ensure_open()
        self._format = _unsigned(value, 32)
        return self

    def key_wrap_type(self, value: int) -> GetBuilder:
        self._ensure_open()
        self._wrap = _unsigned(value, 32)
        return self

    def key_compression_type(self, value: int) -> GetBuilder:
        self._ensure_open()
        self._compression = _unsigned(value, 32)
        return self

    def key_wrapping_specification(self, value: ttlv.Structure) -> GetBuilder:
        self._wrapping = self._replace_structure(self._wrapping, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        item = extensions.client_batch_item_get(
            self._identifier, self._format, self._wrap, self._compression, self._wrapping,
        )
        try:
            with item.validation_report() as report:
                if not report.is_valid:
                    errors.raise_for_status(4)
        except BaseException:
            item.close()
            raise
        return item


class _AttributeCollectionBuilder(_TypedOperationBuilder):
    """Owns ordered attribute copies for Register and Create Split Key."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._attributes: list[ttlv.TtlvItem] = []
        self._positioned: dict[int, ttlv.TtlvItem] = {}

    def _validate_attribute(self, value: ttlv.TtlvItem) -> None:
        if not isinstance(value, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with ttlv.ttlv_structure_create() as empty:
            with attribute_operations.AttributeSet.try_new(empty) as s:
                s.try_push(value)

    def attribute(self, value: Any) -> Self:
        from .typed_attributes import TypedAttribute

        if isinstance(value, TypedAttribute):
            with value.as_item() as item:
                return self.attribute_item(item)
        if isinstance(value, ttlv.TtlvItem):
            return self.attribute_item(value)
        errors.raise_for_status(4)

    def attribute_item(self, value: ttlv.TtlvItem) -> Self:
        self._ensure_open()
        self._validate_attribute(value)
        copy = value.copy()
        self._attributes.append(self._own(copy))
        return self

    def attributes(self, values: attribute_operations.AttributeSet) -> Self:
        self._ensure_open()
        if not isinstance(values, attribute_operations.AttributeSet):
            errors.raise_for_status(4)
        copies: list[ttlv.TtlvItem] = []
        try:
            with values.as_items() as items:
                with ttlv.ttlv_structure_view(items) as view:
                    for idx in range(ttlv.ttlv_structure_view_item_count(view)):
                        with ttlv.ttlv_structure_view_item_at(view, idx) as item:
                            copies.append(item.copy())
        except BaseException:
            for item in copies:
                item.close()
            raise
        for item in self._attributes:
            self._discard(item)
        self._attributes.clear()
        for item in copies:
            self._attributes.append(self._own(item))
        return self

    def attribute_at(self, position: int, value: ttlv.TtlvItem) -> Self:
        self._ensure_open()
        if (
            not isinstance(position, int)
            or isinstance(position, bool)
            or position < 0
            or position in self._positioned
        ):
            errors.raise_for_status(4)
        self._validate_attribute(value)
        copy = value.copy()
        self._positioned[position] = self._own(copy)
        return self

    def vendor_attribute(self, vendor: str, name: str, value: ttlv.TtlvItem) -> Self:
        self._ensure_open()
        if (
            not isinstance(vendor, str)
            or not isinstance(name, str)
            or not isinstance(value, ttlv.TtlvItem)
        ):
            errors.raise_for_status(4)
        vendor_bytes = vendor.encode("utf-8")
        name_bytes = name.encode("utf-8")
        with ttlv.codec_limits_defaults() as limits:
            fields = ttlv.ttlv_structure_create()
            try:
                with (
                    ttlv.raw_tag_from_raw(0x42009D) as raw_tag,
                    ttlv.raw_tag_try_checked(raw_tag) as tag,
                    ttlv.ttlv_value_text_string(vendor_bytes, limits) as v,
                    ttlv.ttlv_item_create(tag, v, limits) as item,
                ):
                    fields = ttlv.ttlv_structure_with_item(fields, item, limits)
                with (
                    ttlv.raw_tag_from_raw(0x42000A) as raw_tag,
                    ttlv.raw_tag_try_checked(raw_tag) as tag,
                    ttlv.ttlv_value_text_string(name_bytes, limits) as v,
                    ttlv.ttlv_item_create(tag, v, limits) as item,
                ):
                    fields = ttlv.ttlv_structure_with_item(fields, item, limits)
                with (
                    ttlv.raw_tag_from_raw(0x42000B) as raw_tag,
                    ttlv.raw_tag_try_checked(raw_tag) as tag,
                    value.value_copy() as copy_val,
                    ttlv.ttlv_item_create(tag, copy_val, limits) as item,
                ):
                    fields = ttlv.ttlv_structure_with_item(fields, item, limits)
                with (
                    ttlv.raw_tag_from_raw(0x420008) as raw_tag,
                    ttlv.raw_tag_try_checked(raw_tag) as tag,
                    ttlv.ttlv_value_structure(fields, limits) as struct_val,
                    ttlv.ttlv_item_create(tag, struct_val, limits) as attr_item,
                ):
                    return self.attribute_item(attr_item)
            finally:
                fields.close()

    def _build_attributes(self) -> ttlv.Structure:
        self._ensure_open()
        ordered = list(self._attributes)
        for pos in sorted(self._positioned.keys()):
            if pos > len(ordered):
                errors.raise_for_status(4)
            ordered.insert(pos, self._positioned[pos])
        with ttlv.codec_limits_defaults() as limits:
            items = ttlv.ttlv_structure_create()
            try:
                for it in ordered:
                    with it.copy() as cp:
                        items = ttlv.ttlv_structure_with_item(items, cp, limits)
                return items
            except BaseException:
                items.close()
                raise

    def close(self) -> None:
        self._attributes.clear()
        self._positioned.clear()
        super().close()


class ActivateBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Activate request with optional identifier and exchange options."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        if unique_identifier is not None:
            self.unique_identifier(unique_identifier)

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_activate(self._unique_identifier)
        )


class ArchiveBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Archive request with optional identifier and exchange options."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        if unique_identifier is not None:
            self.unique_identifier(unique_identifier)

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_archive(self._unique_identifier)
        )


class RecoverBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Recover request with optional identifier and exchange options."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        if unique_identifier is not None:
            self.unique_identifier(unique_identifier)

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_recover(self._unique_identifier)
        )


class CreateSplitKeyBuilder(_AttributeCollectionBuilder):
    """Fluent typed KMIP Create Split Key request with ordered attributes."""

    def __init__(
        self,
        client: Client | None = None,
        object_type: int | None = None,
        parts: int | None = None,
        threshold: int | None = None,
        method: int | None = None,
    ) -> None:
        super().__init__(client)
        self._object_type: int | None = None
        self._method: int | None = None
        self._parts: int | None = None
        self._threshold: int | None = None
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._prime_field_size: bytearray | None = None
        self._protection_storage_masks: ttlv.Structure | None = None

        if object_type is not None:
            self.object_type(object_type)
        if method is not None:
            self.method(method)
        if parts is not None:
            self.parts(parts)
        if threshold is not None:
            self.threshold(threshold)

    def object_type(self, value: int) -> Self:
        self._ensure_open()
        self._object_type = _unsigned(value, 32)
        return self

    def method(self, value: int) -> Self:
        self._ensure_open()
        self._method = _unsigned(value, 32)
        return self

    def parts(self, value: int) -> Self:
        self._ensure_open()
        self._parts = _signed_i32_bits(value)
        return self

    def threshold(self, value: int) -> Self:
        self._ensure_open()
        self._threshold = _signed_i32_bits(value)
        return self

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def prime_field_size(
        self, value: bytes | bytearray | memoryview | None
    ) -> Self:
        self._ensure_open()
        if self._prime_field_size is not None:
            _wipe_crypto_stage(self._prime_field_size)
            self._prime_field_size = None
        if value is not None:
            self._prime_field_size = bytearray(value)
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if (
            self._object_type is None
            or self._method is None
            or self._parts is None
            or self._threshold is None
        ):
            errors.raise_for_status(4)
        with self._build_attributes() as attributes:
            return _validated_item(
                extensions.client_batch_item_create_split_key(
                    self._object_type,
                    self._parts,
                    self._threshold,
                    self._method,
                    attributes,
                    unique_identifier=self._unique_identifier,
                    prime_field_size=self._prime_field_size,
                    protection_storage_masks=self._protection_storage_masks,
                )
            )

    def close(self) -> None:
        if self._prime_field_size is not None:
            _wipe_crypto_stage(self._prime_field_size)
            self._prime_field_size = None
        super().close()


class RegisterBuilder(_AttributeCollectionBuilder):
    """Fluent typed KMIP Register request retaining copies of the object and ordered attributes."""

    def __init__(
        self,
        client: Client | None = None,
        object: Any = None,
        object_type: int | None = None,
    ) -> None:
        super().__init__(client)
        self._object_type: int | None = None
        self._object: ttlv.TtlvItem | None = None
        self._protection_storage_masks: ttlv.Structure | None = None
        self._opaque_object_context: extensions.OpaqueObjectContext | None = None
        if object is not None:
            if object_type is not None:
                self.object_item(object_type, object)
            else:
                self.object(object)

    def object(self, value: Any) -> Self:
        self._ensure_open()
        if value is None:
            errors.raise_for_status(4)
        from .managed_objects import ManagedObject

        if isinstance(value, ManagedObject):
            return self.object_item(value.object_type, value)
        if isinstance(value, ttlv.TtlvItem) and self._object_type is not None:
            return self.object_item(self._object_type, value)
        errors.raise_for_status(4)

    def object_item(self, object_type: int, value: ttlv.TtlvItem) -> Self:
        self._ensure_open()
        raw_type = _unsigned(object_type, 32)
        if not isinstance(value, ttlv.TtlvItem):
            errors.raise_for_status(4)
        copy = value.copy()
        if self._object is not None:
            self._discard(self._object)
        self._object = self._own(copy)
        self._object_type = raw_type
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def opaque_object_context(
        self, value: extensions.OpaqueObjectContext | None
    ) -> Self:
        self._ensure_open()
        if value is not None and not isinstance(value, extensions.OpaqueObjectContext):
            errors.raise_for_status(4)
        self._opaque_object_context = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._object is None or self._object_type is None:
            errors.raise_for_status(4)
        with self._build_attributes() as attributes:
            with self._object.copy() as copy:
                request = extensions.RegisterRequest.create(
                    self._object_type, attributes, copy
                )
                try:
                    if self._opaque_object_context is not None:
                        request = request.with_opaque_object_context(
                            self._opaque_object_context
                        )
                    if self._protection_storage_masks is not None:
                        request = request.with_protection_storage_masks(
                            self._protection_storage_masks
                        )
                    return _validated_item(
                        extensions.client_batch_item_register(request)
                    )
                finally:
                    request.close()


class RekeyBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Rekey request preserving identifier, offset and attributes."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._offset: int | None = None
        self._attributes: ttlv.Structure | None = None
        self._protection_storage_masks: ttlv.Structure | None = None
        if unique_identifier is not None:
            self.unique_identifier(unique_identifier)

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def offset(self, value: int | None) -> Self:
        self._ensure_open()
        self._offset = None if value is None else _unsigned(value, 32)
        return self

    def attributes(self, value: ttlv.Structure | None) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_rekey(
                self._unique_identifier,
                offset=self._offset,
                attributes=self._attributes,
                protection_storage_masks=self._protection_storage_masks,
            )
        )


class RekeyKeyPairBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Re-key Key Pair request retaining all attribute and protection-mask fields."""

    def __init__(
        self,
        client: Client | None = None,
        private_key_unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._private_key_unique_identifier: extensions.UniqueIdentifier | None = None
        self._offset: int | None = None
        self._common_attributes: ttlv.Structure | None = None
        self._private_key_attributes: ttlv.Structure | None = None
        self._public_key_attributes: ttlv.Structure | None = None
        self._common_protection_storage_masks: ttlv.Structure | None = None
        self._private_protection_storage_masks: ttlv.Structure | None = None
        self._public_protection_storage_masks: ttlv.Structure | None = None
        if private_key_unique_identifier is not None:
            self.private_key_unique_identifier(private_key_unique_identifier)

    def private_key_unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._private_key_unique_identifier = _normalize_identifier(value)
        return self

    def offset(self, value: int | None) -> Self:
        self._ensure_open()
        self._offset = None if value is None else _unsigned(value, 32)
        return self

    def _convert_attributes(
        self, value: ttlv.Structure | attribute_operations.AttributeSet | None
    ) -> ttlv.Structure | None:
        if value is None:
            return None
        if isinstance(value, attribute_operations.AttributeSet):
            with value.as_items() as items:
                return items.copy()
        if isinstance(value, ttlv.Structure):
            return value.copy()
        errors.raise_for_status(4)

    def common_attributes(
        self, value: ttlv.Structure | attribute_operations.AttributeSet | None
    ) -> Self:
        self._ensure_open()
        converted = self._convert_attributes(value)
        self._discard(self._common_attributes)
        self._common_attributes = self._own(converted) if converted is not None else None
        return self

    def private_key_attributes(
        self, value: ttlv.Structure | attribute_operations.AttributeSet | None
    ) -> Self:
        self._ensure_open()
        converted = self._convert_attributes(value)
        self._discard(self._private_key_attributes)
        self._private_key_attributes = self._own(converted) if converted is not None else None
        return self

    def public_key_attributes(
        self, value: ttlv.Structure | attribute_operations.AttributeSet | None
    ) -> Self:
        self._ensure_open()
        converted = self._convert_attributes(value)
        self._discard(self._public_key_attributes)
        self._public_key_attributes = self._own(converted) if converted is not None else None
        return self

    def common_protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._common_protection_storage_masks = self._replace_structure(
            self._common_protection_storage_masks, value
        )
        return self

    def private_protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._private_protection_storage_masks = self._replace_structure(
            self._private_protection_storage_masks, value
        )
        return self

    def public_protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._public_protection_storage_masks = self._replace_structure(
            self._public_protection_storage_masks, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_rekey_key_pair(
                self._private_key_unique_identifier,
                offset=self._offset,
                common_attributes=self._common_attributes,
                private_key_attributes=self._private_key_attributes,
                public_key_attributes=self._public_key_attributes,
                common_protection_storage_masks=self._common_protection_storage_masks,
                private_protection_storage_masks=self._private_protection_storage_masks,
                public_protection_storage_masks=self._public_protection_storage_masks,
            )
        )


class RecertifyBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Recertify request. Structure setters retain independent copies."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._offset: int | None = None
        self._attributes: ttlv.Structure | None = None
        self._protection_storage_masks: ttlv.Structure | None = None
        self._certificate_request_unique_identifier: extensions.UniqueIdentifier | None = None
        self._certificate_request_type: int | None = None
        self._certificate_request_value: bytearray | None = None
        if unique_identifier is not None:
            self.unique_identifier(unique_identifier)

    def unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def offset(self, value: int | None) -> Self:
        self._ensure_open()
        self._offset = None if value is None else _unsigned(value, 32)
        return self

    def attributes(self, value: ttlv.Structure | None) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def certificate_request_unique_identifier(
        self, value: extensions.UniqueIdentifier | str | None
    ) -> Self:
        self._ensure_open()
        self._certificate_request_unique_identifier = _normalize_identifier(value)
        return self

    def certificate_request_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._certificate_request_type = None if value is None else _unsigned(value, 32)
        return self

    def certificate_request_value(
        self, value: bytes | bytearray | memoryview | None
    ) -> Self:
        self._ensure_open()
        if self._certificate_request_value is not None:
            _wipe_crypto_stage(self._certificate_request_value)
            self._certificate_request_value = None
        if value is not None:
            self._certificate_request_value = bytearray(value)
        return self

    def close(self) -> None:
        if self._certificate_request_value is not None:
            _wipe_crypto_stage(self._certificate_request_value)
            self._certificate_request_value = None
        super().close()

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return _validated_item(
            extensions.client_batch_item_recertify(
                self._unique_identifier,
                self._certificate_request_unique_identifier,
                certificate_request_type=self._certificate_request_type,
                certificate_request_value=self._certificate_request_value,
                offset=self._offset,
                attributes=self._attributes,
                protection_storage_masks=self._protection_storage_masks,
            )
        )


class QueryBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Query request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._functions: list[int] = []
        self._object_groups: list[str] | None = None

    def function(self, value: int) -> Self:
        self._ensure_open()
        self._functions.append(_unsigned(value, 32))
        return self

    def functions(self, values: Sequence[int]) -> Self:
        self._ensure_open()
        for value in values:
            self._functions.append(_unsigned(value, 32))
        return self

    def object_groups(self, groups: Sequence[str] | None) -> Self:
        self._ensure_open()
        self._object_groups = None if groups is None else list(groups)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        fns = self._functions if self._functions else [1]
        return self._validated(
            extensions.client_batch_item_query(fns, self._object_groups)
        )


class DiscoverVersionsBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Discover Versions request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._versions: list[Any] = []

    def protocol_version(self, version: Any) -> Self:
        self._ensure_open()
        self._versions.append(version)
        return self

    def protocol_versions(self, versions: Sequence[Any]) -> Self:
        self._ensure_open()
        self._versions.extend(versions)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_discover_versions(
                self._versions if self._versions else None
            )
        )


class CheckBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Check request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._usage_limits_count: int | None = None
        self._cryptographic_usage_mask: int | None = None
        self._lease_time: int | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def usage_limits_count(self, value: int | None) -> Self:
        self._ensure_open()
        self._usage_limits_count = value
        return self

    def cryptographic_usage_mask(self, value: int | None) -> Self:
        self._ensure_open()
        self._cryptographic_usage_mask = value
        return self

    def lease_time(self, value: int | None) -> Self:
        self._ensure_open()
        self._lease_time = None if value is None else _unsigned(value, 32)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_check(
                self._unique_identifier,
                usage_limits_count=self._usage_limits_count,
                cryptographic_usage_mask=self._cryptographic_usage_mask,
                lease_time=self._lease_time,
            )
        )


class ExportBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Export request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._key_format_type: int | None = None
        self._key_wrap_type: int | None = None
        self._key_compression_type: int | None = None
        self._key_wrapping_specification: ttlv.Structure | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def key_format_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._key_format_type = None if value is None else _unsigned(value, 32)
        return self

    def key_wrap_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._key_wrap_type = None if value is None else _unsigned(value, 32)
        return self

    def key_compression_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._key_compression_type = None if value is None else _unsigned(value, 32)
        return self

    def key_wrapping_specification(self, value: ttlv.Structure | None) -> Self:
        self._key_wrapping_specification = self._replace_structure(
            self._key_wrapping_specification, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_export(
                self._unique_identifier,
                key_format_type=self._key_format_type,
                key_wrap_type=self._key_wrap_type,
                key_compression_type=self._key_compression_type,
                key_wrapping_specification=self._key_wrapping_specification,
            )
        )


class ImportBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Import request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._unique_identifier: extensions.UniqueIdentifier | None = None
        self._object_type: int | None = None
        self._replace_existing: bool | None = None
        self._key_wrap_type: int | None = None
        self._attributes: ttlv.Structure | None = None
        self._object: ttlv.Structure | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def object_type(self, value: int) -> Self:
        self._ensure_open()
        self._object_type = _unsigned(value, 32)
        return self

    def replace_existing(self, value: bool | None) -> Self:
        self._ensure_open()
        self._replace_existing = value
        return self

    def key_wrap_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._key_wrap_type = None if value is None else _unsigned(value, 32)
        return self

    def attributes(self, value: ttlv.Structure) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def object(self, value: ttlv.Structure) -> Self:
        self._object = self._replace_structure(self._object, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if (
            self._unique_identifier is None
            or self._object_type is None
            or self._attributes is None
            or self._object is None
        ):
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_import(
                self._unique_identifier,
                self._object_type,
                self._attributes,
                self._object,
                replace_existing=self._replace_existing,
                key_wrap_type=self._key_wrap_type,
            )
        )


class RngRetrieveBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP RNG Retrieve request."""

    def __init__(
        self, client: Client | None = None, data_length: int | None = None
    ) -> None:
        super().__init__(client)
        self._data_length = data_length

    def data_length(self, value: int) -> Self:
        self._ensure_open()
        self._data_length = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._data_length is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_rng_retrieve(self._data_length)
        )


class RngSeedBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP RNG Seed request."""

    def __init__(
        self,
        client: Client | None = None,
        data: bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._data = None if data is None else bytearray(data)

    def data(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._data = bytearray(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._data is None:
            errors.raise_for_status(4)
        return self._validated(extensions.client_batch_item_rng_seed(bytes(self._data)))


class CertifyBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Certify request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._certificate_request_type: int | None = None
        self._certificate_request_value: bytearray | None = None
        self._attributes: ttlv.Structure | None = None
        self._protection_storage_masks: ttlv.Structure | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def certificate_request_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._certificate_request_type = None if value is None else _unsigned(value, 32)
        return self

    def certificate_request_value(
        self, value: bytes | bytearray | memoryview | None
    ) -> Self:
        self._ensure_open()
        self._certificate_request_value = None if value is None else bytearray(value)
        return self

    def attributes(self, value: ttlv.Structure | None) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_certify(
                self._unique_identifier,
                certificate_request_type=self._certificate_request_type,
                certificate_request_value=(
                    None
                    if self._certificate_request_value is None
                    else bytes(self._certificate_request_value)
                ),
                attributes=self._attributes,
                protection_storage_masks=self._protection_storage_masks,
            )
        )


class DeriveKeyBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Derive Key request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._object_type: int | None = None
        self._unique_identifiers: list[extensions.UniqueIdentifier] = []
        self._derivation_method: int | None = None
        self._derivation_parameters: ttlv.Structure | None = None
        self._attributes: ttlv.Structure | None = None

    def object_type(self, value: int) -> Self:
        self._ensure_open()
        self._object_type = _unsigned(value, 32)
        return self

    def unique_identifier(self, value: extensions.UniqueIdentifier | str) -> Self:
        self._ensure_open()
        self._unique_identifiers.append(_normalize_identifier(value))
        return self

    def unique_identifiers(
        self, values: Sequence[extensions.UniqueIdentifier | str]
    ) -> Self:
        self._ensure_open()
        for value in values:
            self._unique_identifiers.append(_normalize_identifier(value))
        return self

    def derivation_method(self, value: int) -> Self:
        self._ensure_open()
        self._derivation_method = _unsigned(value, 32)
        return self

    def derivation_parameters(self, value: ttlv.Structure) -> Self:
        self._derivation_parameters = self._replace_structure(
            self._derivation_parameters, value
        )
        return self

    def attributes(self, value: ttlv.Structure) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if (
            self._object_type is None
            or self._derivation_method is None
            or self._derivation_parameters is None
            or self._attributes is None
        ):
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_derive_key(
                self._object_type,
                self._unique_identifiers,
                self._derivation_method,
                self._derivation_parameters,
                self._attributes,
            )
        )


class ValidateBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Validate request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._certificates: list[ttlv.Structure] = []
        self._identifiers: list[extensions.UniqueIdentifier] = []
        self._validity_date: int | None = None

    def certificate(self, cert: ttlv.Structure) -> Self:
        self._ensure_open()
        self._certificates.append(self._own(ttlv.ttlv_structure_copy(cert)))
        return self

    def unique_identifier(self, value: extensions.UniqueIdentifier | str) -> Self:
        self._ensure_open()
        self._identifiers.append(_normalize_identifier(value))
        return self

    def validity_date(self, value: int | None) -> Self:
        self._ensure_open()
        self._validity_date = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_validate(
                self._certificates,
                self._identifiers,
                validity_date=self._validity_date,
            )
        )


class JoinSplitKeyBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Join Split Key request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._object_type: int | None = None
        self._references: list[Any] = []
        self._secret_data_type: int | None = None
        self._attributes: ttlv.Structure | None = None
        self._protection_storage_masks: ttlv.Structure | None = None

    def object_type(self, value: int) -> Self:
        self._ensure_open()
        self._object_type = _unsigned(value, 32)
        return self

    def reference(self, ref: Any) -> Self:
        self._ensure_open()
        self._references.append(ref)
        return self

    def references(self, refs: Sequence[Any]) -> Self:
        self._ensure_open()
        self._references.extend(refs)
        return self

    def secret_data_type(self, value: int | None) -> Self:
        self._ensure_open()
        self._secret_data_type = None if value is None else _unsigned(value, 32)
        return self

    def attributes(self, value: ttlv.Structure | None) -> Self:
        self._attributes = self._replace_structure(self._attributes, value)
        return self

    def protection_storage_masks(self, value: ttlv.Structure | None) -> Self:
        self._protection_storage_masks = self._replace_structure(
            self._protection_storage_masks, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._object_type is None or not self._references:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_join_split_key(
                self._object_type,
                self._references,
                secret_data_type=self._secret_data_type,
                attributes=self._attributes,
                protection_storage_masks=self._protection_storage_masks,
            )
        )


class ObtainLeaseBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Obtain Lease request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_obtain_lease(self._unique_identifier)
        )


class GetUsageAllocationBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Get Usage Allocation request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
        usage_limits_count: int | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._usage_limits_count = usage_limits_count

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def usage_limits_count(self, value: int) -> Self:
        self._ensure_open()
        self._usage_limits_count = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._usage_limits_count is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_get_usage_allocation(
                self._unique_identifier, self._usage_limits_count
            )
        )


class ModifyAttributeBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Modify Attribute request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._current_attribute: ttlv.Item | None = None
        self._new_attribute: ttlv.Item | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def current_attribute(self, value: ttlv.Item | None) -> Self:
        self._ensure_open()
        self._current_attribute = value
        return self

    def new_attribute(self, value: ttlv.Item) -> Self:
        self._ensure_open()
        self._new_attribute = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._new_attribute is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_modify_attribute(
                self._unique_identifier, self._current_attribute, self._new_attribute
            )
        )


class AdjustAttributeBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Adjust Attribute request."""

    def __init__(
        self,
        client: Client | None = None,
        unique_identifier: extensions.UniqueIdentifier | str | None = None,
    ) -> None:
        super().__init__(client)
        self._unique_identifier = (
            None if unique_identifier is None else _normalize_identifier(unique_identifier)
        )
        self._attribute_reference: extensions.AttributeReference | None = None
        self._adjustment_type: int | None = None
        self._adjustment_value: ttlv.Value | None = None

    def unique_identifier(self, value: extensions.UniqueIdentifier | str | None) -> Self:
        self._ensure_open()
        self._unique_identifier = _normalize_identifier(value)
        return self

    def attribute_reference(self, value: extensions.AttributeReference) -> Self:
        self._ensure_open()
        self._attribute_reference = value
        return self

    def adjustment_type(self, value: int) -> Self:
        self._ensure_open()
        self._adjustment_type = _unsigned(value, 32)
        return self

    def adjustment_value(self, value: ttlv.Value | None) -> Self:
        self._ensure_open()
        self._adjustment_value = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._attribute_reference is None or self._adjustment_type is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_adjust_attribute(
                self._unique_identifier,
                self._attribute_reference,
                self._adjustment_type,
                self._adjustment_value,
            )
        )


class GetConstraintsBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Get Constraints request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(extensions.client_batch_item_get_constraints())


class CancelBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Cancel request."""

    def __init__(
        self,
        client: Client | None = None,
        correlation_value: bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._correlation_value: bytes | None = (
            None if correlation_value is None else bytes(correlation_value)
        )

    def correlation_value(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._correlation_value = bytes(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._correlation_value is None:
            errors.raise_for_status(4)
        with extensions.CancelRequest.create(self._correlation_value) as req:
            return self._validated(extensions.client_batch_item_cancel_from_request(req))


class PollBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Poll request."""

    def __init__(
        self,
        client: Client | None = None,
        correlation_value: bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._correlation_value: bytes | None = (
            None if correlation_value is None else bytes(correlation_value)
        )

    def correlation_value(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._correlation_value = bytes(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._correlation_value is None:
            errors.raise_for_status(4)
        with extensions.PollRequest.create(self._correlation_value) as req:
            return self._validated(extensions.client_batch_item_poll_from_request(req))


class ProcessBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Process request."""

    def __init__(
        self,
        client: Client | None = None,
        correlation_value: bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._correlation_value: bytes | None = (
            None if correlation_value is None else bytes(correlation_value)
        )

    def correlation_value(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._correlation_value = bytes(value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._correlation_value is None:
            errors.raise_for_status(4)
        with extensions.ProcessRequest.create(self._correlation_value) as req:
            return self._validated(extensions.client_batch_item_process_from_request(req))


class QueryAsyncRequestsBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Query Asynchronous Requests request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._correlation_values: list[bytes] | None = None
        self._operations: list[int] | None = None

    def correlation_value(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        if self._correlation_values is None:
            self._correlation_values = []
        self._correlation_values.append(bytes(value))
        return self

    def correlation_values(self, values: Sequence[bytes | bytearray | memoryview]) -> Self:
        self._ensure_open()
        self._correlation_values = [bytes(v) for v in values]
        return self

    def operation(self, operation: int) -> Self:
        self._ensure_open()
        if self._operations is None:
            self._operations = []
        self._operations.append(_unsigned(operation, 32))
        return self

    def operations(self, operations: Sequence[int]) -> Self:
        self._ensure_open()
        self._operations = [_unsigned(op, 32) for op in operations]
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        req = extensions.QueryAsyncRequestsRequest.create()
        try:
            if self._correlation_values is not None:
                next_req = req.with_correlation_values(self._correlation_values)
                req.close()
                req = next_req
            if self._operations is not None:
                next_req = req.with_operations(self._operations)
                req.close()
                req = next_req
            return self._validated(
                extensions.client_batch_item_query_async_requests_from_request(req)
            )
        finally:
            req.close()


class LoginBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Login request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._lease_time_interval: int | None = None
        self._lease_time_date_time: int | None = None
        self._request_count: int | None = None
        self._usage_limits: ttlv.Structure | None = None

    def lease_time_interval(self, seconds: int | None) -> Self:
        self._ensure_open()
        self._lease_time_interval = None if seconds is None else _unsigned(seconds, 32)
        self._lease_time_date_time = None
        return self

    def lease_time_date_time(self, unix_seconds: int | None) -> Self:
        self._ensure_open()
        self._lease_time_date_time = unix_seconds
        self._lease_time_interval = None
        return self

    def request_count(self, count: int | None) -> Self:
        self._ensure_open()
        self._request_count = count
        return self

    def usage_limits(self, value: ttlv.Structure | None) -> Self:
        self._usage_limits = self._replace_structure(self._usage_limits, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_login(
                lease_time_interval=self._lease_time_interval,
                lease_time_date_time=self._lease_time_date_time,
                request_count=self._request_count,
                usage_limits=self._usage_limits,
            )
        )


class LogoutBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Logout request."""

    def __init__(
        self, client: Client | None = None, ticket: ttlv.Structure | None = None
    ) -> None:
        super().__init__(client)
        self._ticket: ttlv.Structure | None = None
        if ticket is not None:
            self.ticket(ticket)

    def ticket(self, value: ttlv.Structure) -> Self:
        self._ticket = self._replace_structure(self._ticket, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._ticket is None:
            errors.raise_for_status(4)
        return self._validated(extensions.client_batch_item_logout(self._ticket))


class DelegatedLoginBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Delegated Login request."""

    def __init__(
        self, client: Client | None = None, rights: ttlv.Structure | None = None
    ) -> None:
        super().__init__(client)
        self._rights: ttlv.Structure | None = None
        if rights is not None:
            self.rights(rights)
        self._lease_time_interval: int | None = None
        self._lease_time_date_time: int | None = None
        self._request_count: int | None = None
        self._usage_limits: ttlv.Structure | None = None

    def rights(self, value: ttlv.Structure) -> Self:
        self._rights = self._replace_structure(self._rights, value)
        return self

    def lease_time_interval(self, seconds: int | None) -> Self:
        self._ensure_open()
        self._lease_time_interval = None if seconds is None else _unsigned(seconds, 32)
        self._lease_time_date_time = None
        return self

    def lease_time_date_time(self, unix_seconds: int | None) -> Self:
        self._ensure_open()
        self._lease_time_date_time = unix_seconds
        self._lease_time_interval = None
        return self

    def request_count(self, count: int | None) -> Self:
        self._ensure_open()
        self._request_count = count
        return self

    def usage_limits(self, value: ttlv.Structure | None) -> Self:
        self._usage_limits = self._replace_structure(self._usage_limits, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._rights is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_delegated_login(
                self._rights,
                lease_time_interval=self._lease_time_interval,
                lease_time_date_time=self._lease_time_date_time,
                request_count=self._request_count,
                usage_limits=self._usage_limits,
            )
        )


class SetEndpointRoleBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Set Endpoint Role request."""

    def __init__(
        self, client: Client | None = None, endpoint_role: int | None = None
    ) -> None:
        super().__init__(client)
        self._endpoint_role: int | None = (
            None if endpoint_role is None else _unsigned(endpoint_role, 32)
        )
        self._vendor_identification: str | bytes | bytearray | memoryview | None = None

    def endpoint_role(self, value: int) -> Self:
        self._ensure_open()
        self._endpoint_role = _unsigned(value, 32)
        return self

    def vendor_identification(
        self, value: str | bytes | bytearray | memoryview | None
    ) -> Self:
        self._ensure_open()
        self._vendor_identification = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._endpoint_role is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_set_endpoint_role(
                self._endpoint_role, self._vendor_identification
            )
        )


class Pkcs11Builder(_TypedOperationBuilder):
    """Fluent typed KMIP PKCS#11 request."""

    def __init__(
        self, client: Client | None = None, function: ttlv.Structure | None = None
    ) -> None:
        super().__init__(client)
        self._function: ttlv.Structure | None = None
        if function is not None:
            self.function(function)
        self._interface: ttlv.Structure | None = None
        self._correlation_value: bytes | None = None
        self._input_parameters: ttlv.Structure | None = None
        self._extensions: list[ttlv.TtlvItem] = []

    def function(self, value: ttlv.Structure) -> Self:
        self._function = self._replace_structure(self._function, value)
        return self

    def interface(self, value: ttlv.Structure | None) -> Self:
        self._interface = self._replace_structure(self._interface, value)
        return self

    def correlation_value(self, value: bytes | bytearray | memoryview | None) -> Self:
        self._ensure_open()
        self._correlation_value = None if value is None else bytes(value)
        return self

    def input_parameters(self, value: ttlv.Structure | None) -> Self:
        self._input_parameters = self._replace_structure(self._input_parameters, value)
        return self

    def extension(self, item: ttlv.TtlvItem) -> Self:
        self._ensure_open()
        self._extensions.append(item)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._function is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_pkcs11(
                self._function,
                self._interface,
                self._correlation_value,
                self._input_parameters,
                payload_extensions=self._extensions,
            )
        )


class InteropBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Interop request."""

    def __init__(
        self,
        client: Client | None = None,
        interop_function: int | None = None,
        interop_identifier: str | bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._interop_function: int | None = (
            None if interop_function is None else _unsigned(interop_function, 32)
        )
        self._interop_identifier = interop_identifier

    def interop_function(self, value: int) -> Self:
        self._ensure_open()
        self._interop_function = _unsigned(value, 32)
        return self

    def interop_identifier(self, value: str | bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._interop_identifier = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._interop_function is None or self._interop_identifier is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_interop(
                self._interop_function, self._interop_identifier
            )
        )


class ReProvisionBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Re-Provision request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._certificate_request: ttlv.Structure | None = None
        self._certificate: ttlv.Structure | None = None

    def certificate_request(self, value: ttlv.Structure | None) -> Self:
        self._certificate_request = self._replace_structure(self._certificate_request, value)
        return self

    def certificate(self, value: ttlv.Structure | None) -> Self:
        self._certificate = self._replace_structure(self._certificate, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_reprovision(
                self._certificate_request, self._certificate
            )
        )


class SetDefaultsBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Set Defaults request."""

    def __init__(self, client: Client | None = None) -> None:
        super().__init__(client)
        self._defaults_information: ttlv.Structure | None = None

    def defaults_information(self, value: ttlv.Structure | None) -> Self:
        self._defaults_information = self._replace_structure(
            self._defaults_information, value
        )
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        return self._validated(
            extensions.client_batch_item_set_defaults(self._defaults_information)
        )


class SetConstraintsBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Set Constraints request."""

    def __init__(
        self, client: Client | None = None, constraints: ttlv.Structure | None = None
    ) -> None:
        super().__init__(client)
        self._constraints: ttlv.Structure | None = None
        if constraints is not None:
            self.constraints(constraints)

    def constraints(self, value: ttlv.Structure) -> Self:
        self._constraints = self._replace_structure(self._constraints, value)
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._constraints is None:
            errors.raise_for_status(4)
        return self._validated(
            extensions.client_batch_item_set_constraints(self._constraints)
        )


class LogBuilder(_TypedOperationBuilder):
    """Fluent typed KMIP Log request."""

    def __init__(
        self,
        client: Client | None = None,
        log_message: str | bytes | bytearray | memoryview | None = None,
    ) -> None:
        super().__init__(client)
        self._log_message = log_message

    def log_message(self, value: str | bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        self._log_message = value
        return self

    def build(self) -> extensions.ClientBatchItem:
        self._ensure_open()
        if self._log_message is None:
            errors.raise_for_status(4)
        return self._validated(extensions.client_batch_item_log(self._log_message))


__all__ = [
    "ActivateBuilder",
    "AddAttributeBuilder",
    "AdjustAttributeBuilder",
    "ArchiveBuilder",
    "CancelBuilder",
    "CertifyBuilder",
    "CheckBuilder",
    "CreateBuilder",
    "CreateCredentialBuilder",
    "CreateCredentialObject",
    "CreateGroupBuilder",
    "CreateKeyPairBuilder",
    "CreateSplitKeyBuilder",
    "CreateUserBuilder",
    "DeactivateBuilder",
    "DecapsulateBuilder",
    "DecryptBuilder",
    "DelegatedLoginBuilder",
    "DeleteAttributeBuilder",
    "DeriveKeyBuilder",
    "DiscoverVersionsBuilder",
    "EncapsulateBuilder",
    "EncryptBuilder",
    "ExportBuilder",
    "GetAttributeListBuilder",
    "GetAttributesBuilder",
    "GetBuilder",
    "GetConstraintsBuilder",
    "GetUsageAllocationBuilder",
    "HashBuilder",
    "ImportBuilder",
    "InteropBuilder",
    "JoinSplitKeyBuilder",
    "LocateBuilder",
    "LogBuilder",
    "LoginBuilder",
    "LogoutBuilder",
    "MacBuilder",
    "MacVerifyBuilder",
    "ModifyAttributeBuilder",
    "ObliterateBuilder",
    "ObtainLeaseBuilder",
    "Operation",
    "PingBuilder",
    "Pkcs11Builder",
    "PollBuilder",
    "ProcessBuilder",
    "QueryAsyncRequestsBuilder",
    "QueryBuilder",
    "ReProvisionBuilder",
    "RecertifyBuilder",
    "RecoverBuilder",
    "RegisterBuilder",
    "RekeyBuilder",
    "RekeyKeyPairBuilder",
    "Request",
    "RevokeBuilder",
    "RngRetrieveBuilder",
    "RngSeedBuilder",
    "SetAttributeBuilder",
    "SetConstraintsBuilder",
    "SetDefaultsBuilder",
    "SetEndpointRoleBuilder",
    "SignBuilder",
    "SignatureVerifyBuilder",
    "SymmetricKeyBuilder",
    "ValidateBuilder",
]
