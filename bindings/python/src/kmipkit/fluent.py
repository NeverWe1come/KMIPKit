"""Task-oriented KMIP client builders backed by the shared Rust C ABI."""

from __future__ import annotations

from typing import Any, Self

from . import errors, extensions, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import _wrap_owned_handle
from ._handles import NativeHandle
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
from .cryptographic_parameters import CryptographicParameters
from .credentials import SecretBytes, SecretText
from .typed_attributes import StandardAttribute, TypedAttribute

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


def _execute_single_item(
    client: Client,
    item: extensions.ClientBatchItem,
    limits: ttlv.CodecLimits,
    options: RequestOptions | None,
    asynchronous_indicator: int | None,
) -> BatchResult:
    """Transfer one typed item into a batch and execute exactly one exchange."""
    with Batch.create() as batch:
        batch.add_item(item)
        if asynchronous_indicator is not None:
            batch.set_asynchronous_indicator(asynchronous_indicator)
        if options is None:
            return client.execute(batch, limits)
        return client.execute_with_options(batch, limits, options)


def _require_kmip30_client(client: Any) -> None:
    if not isinstance(client, Client):
        errors.raise_for_status(4)


class Request:
    """Scoped fluent namespace for the foundation lifecycle operations."""

    def __init__(self, client: Client) -> None:
        if client is None:
            errors.raise_for_status(4)
        self._client = client

    def create_symmetric_key(self) -> SymmetricKeyBuilder:
        return SymmetricKeyBuilder(client=self._client)

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

    def get(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "get", unique_identifier)

    def destroy(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "destroy", unique_identifier)

    def activate(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "activate", unique_identifier)

    def archive(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "archive", unique_identifier)

    def recover(self, unique_identifier: extensions.UniqueIdentifier) -> Operation:
        return Operation(self._client, "recover", unique_identifier)

    def locate(self, attributes: Any) -> LocateBuilder:
        from .attribute_operations import AttributeSet

        if not isinstance(attributes, AttributeSet):
            errors.raise_for_status(4)
        return LocateBuilder(self._client, attributes)

    def revoke(
        self,
        unique_identifier: extensions.UniqueIdentifier,
        reason: extensions.RevocationReason,
    ) -> RevokeBuilder:
        return RevokeBuilder(self._client, unique_identifier, reason)

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

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        self._ensure_open()
        if not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        return _execute_single_item(
            self._client, self.build(), limits, self._options,
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
    # The runtime's decoded str copy cannot be reliably erased; encoded staging is.
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
                value = _secret_text_copy(value) if is_text else _secret_bytes_copy(value)
                assert value is not None
                staged.append(value)
                spans.append(_staged_secret_span(value))
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
        interval: int | None = None, digits: int | None = None,
    ) -> CreateCredentialObject:
        serial_span: Any = ffi.NULL
        seed_span: Any = ffi.NULL
        staged: list[bytearray] = []
        try:
            if serial is not None:
                serial_span, _pointer, owner = _credential_text_span(serial)
                staged.append(owner)
            if seed is not None:
                secret_seed = _secret_bytes_copy(seed)
                assert secret_seed is not None
                staged.append(secret_seed)
                seed_span = _staged_secret_span(secret_seed)
            algorithm_value = _unsigned(algorithm, 32)
            digest_value = None if digest is None else _unsigned(digest, 32)
            interval_value = None if interval is None else _unsigned(interval, 32)
            digits_value = None if digits is None else _unsigned(digits, 31)
            return _new_create_credential_object(
                lib.kmipkit_create_credential_object_one_time_password,
                algorithm_value, int(digest_value is not None), digest_value or 0,
                serial_span, seed_span, int(interval_value is not None), interval_value or 0,
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


class CreateKeyPairBuilder:
    """Build a typed Create Key Pair request from semantic key parameters."""

    def __init__(self, *, client: Client | None = None) -> None:
        self._client = client
        self._algorithm: int | None = None
        self._length: int | None = None
        self._private_usage_mask: int | None = None
        self._public_usage_mask: int | None = None
        self._common_attributes: list[TypedAttribute] = []
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def algorithm(self, value: int) -> Self:
        self._algorithm = _unsigned(value, 32)
        return self

    def length(self, value: int) -> Self:
        if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= 0x7FFF_FFFF:
            errors.raise_for_status(4)
        self._length = value
        return self

    def private_usage_mask(self, value: int) -> Self:
        if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value <= 0x7FFF_FFFF:
            errors.raise_for_status(4)
        self._private_usage_mask = value
        return self

    def public_usage_mask(self, value: int) -> Self:
        if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value <= 0x7FFF_FFFF:
            errors.raise_for_status(4)
        self._public_usage_mask = value
        return self

    def with_options(self, options: RequestOptions) -> Self:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def common_attribute(self, value: TypedAttribute) -> Self:
        """Add a typed common attribute borrowed until build or execute completes."""
        if not isinstance(value, TypedAttribute):
            errors.raise_for_status(4)
        self._common_attributes.append(value)
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    @staticmethod
    def _attribute_structure(
        attributes: list[tuple[StandardAttribute, int, bool]],
        limits: ttlv.CodecLimits,
    ) -> ttlv.Structure:
        structure = ttlv.ttlv_structure_create()
        try:
            for attribute, raw_value, enumeration in attributes:
                value = (
                    ttlv.ttlv_value_enumeration(raw_value)
                    if enumeration
                    else ttlv.ttlv_value_integer(raw_value)
                )
                typed: TypedAttribute | None = None
                item: ttlv.TtlvItem | None = None
                try:
                    typed = TypedAttribute.create(attribute, value)
                    item = typed.as_item()
                    value.close()
                    value = None
                    typed.close()
                    typed = None
                    structure = ttlv.ttlv_structure_with_item(structure, item, limits)
                    item = None
                finally:
                    if value is not None:
                        value.close()
                    if typed is not None:
                        typed.close()
                    if item is not None:
                        item.close()
            return structure
        except Exception:
            structure.close()
            raise

    def _common_attribute_structure(self, limits: ttlv.CodecLimits) -> ttlv.Structure:
        structure = self._attribute_structure(
            [
                (StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, self._algorithm, True),
                (StandardAttribute.CRYPTOGRAPHIC_LENGTH, self._length, False),
            ],
            limits,
        )
        try:
            for attribute in self._common_attributes:
                with attribute.as_item() as item:
                    structure = ttlv.ttlv_structure_with_item(structure, item, limits)
            return structure
        except Exception:
            structure.close()
            raise

    def build(self) -> extensions.ClientBatchItem:
        """Create a local typed request; sending still requires explicit execute."""
        if self._algorithm is None or self._length is None:
            errors.raise_for_status(4)
        with (
            ttlv.codec_limits_defaults() as limits,
            self._common_attribute_structure(limits) as common,
            self._attribute_structure(
                [(StandardAttribute.CRYPTOGRAPHIC_USAGE_MASK, self._private_usage_mask, False)]
                if self._private_usage_mask is not None
                else [],
                limits,
            ) as private,
            self._attribute_structure(
                [(StandardAttribute.CRYPTOGRAPHIC_USAGE_MASK, self._public_usage_mask, False)]
                if self._public_usage_mask is not None
                else [],
                limits,
            ) as public,
        ):
            return extensions.client_batch_item_create_key_pair(common, private, public)

    to_batch_item = build

    def validate(self) -> None:
        item = self.build()
        item.close()

    def execute(
        self,
        limits: ttlv.CodecLimits,
        client: Client | None = None,
    ) -> BatchResult:
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

    def validate(self) -> None:
        """Validate locally without sending a request."""
        self.build().close()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        return _execute_single_item(
            self._client, self.build(), limits, self._options,
            self._asynchronous_indicator,
        )


class LocateBuilder:
    """Build and execute a typed Locate request with explicit filters."""

    def __init__(self, client: Client, attributes: Any) -> None:
        self._client = client
        self._attributes = attributes
        self._maximum_items: int | None = None
        self._offset_items: int | None = None
        self._storage_status_mask: int | None = None
        self._object_group_member: int | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def maximum_items(self, value: int) -> Self:
        self._maximum_items = value
        return self

    def offset_items(self, value: int) -> Self:
        self._offset_items = value
        return self

    def storage_status_mask(self, value: int) -> Self:
        self._storage_status_mask = value
        return self

    def object_group_member(self, value: int) -> Self:
        self._object_group_member = value
        return self

    def with_options(self, options: RequestOptions) -> Self:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def build(self) -> extensions.ClientBatchItem:
        with self._attributes.as_items() as attributes:
            return extensions.client_batch_item_locate(
                attributes,
                maximum_items=self._maximum_items,
                offset_items=self._offset_items,
                storage_status_mask=self._storage_status_mask,
                object_group_member=self._object_group_member,
            )

    to_batch_item = build

    def validate(self) -> None:
        self.build().close()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        if not isinstance(self._client, Client) or not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        return _execute_single_item(
            self._client, self.build(), limits, self._options,
            self._asynchronous_indicator,
        )


class RevokeBuilder:
    """Build and execute a typed Revoke request with a redacted reason message."""

    def __init__(
        self,
        client: Client,
        unique_identifier: extensions.UniqueIdentifier,
        reason: extensions.RevocationReason,
    ) -> None:
        if not isinstance(reason, extensions.RevocationReason):
            errors.raise_for_status(4)
        _identifier(unique_identifier)
        self._client = client
        self._identifier = unique_identifier
        self._reason_code = reason.raw_code
        self._reason_message = reason.message
        self._compromise_occurrence_date: int | None = None
        self._options: RequestOptions | None = None
        self._asynchronous_indicator: int | None = None

    def compromise_occurrence_date(self, value: int) -> Self:
        self._compromise_occurrence_date = value
        return self

    def with_options(self, options: RequestOptions) -> Self:
        if not isinstance(options, RequestOptions):
            errors.raise_for_status(4)
        self._options = options
        return self

    def asynchronous_indicator(self, value: int) -> Self:
        self._asynchronous_indicator = _unsigned(value, 32)
        return self

    def build(self) -> extensions.ClientBatchItem:
        with extensions.RevocationReason.create(
            self._reason_code, self._reason_message
        ) as reason:
            request = extensions.RevokeRequest.create(
                self._identifier, reason, self._compromise_occurrence_date
            )
            return extensions.client_batch_item_revoke(request)

    to_batch_item = build

    def validate(self) -> None:
        self.build().close()

    def execute(self, limits: ttlv.CodecLimits) -> BatchResult:
        if not isinstance(self._client, Client) or not isinstance(limits, ttlv.CodecLimits):
            errors.raise_for_status(4)
        return _execute_single_item(
            self._client, self.build(), limits, self._options,
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
    "CreateKeyPairBuilder",
    "DecryptBuilder",
    "EncryptBuilder",
    "HashBuilder",
    "MacBuilder",
    "MacVerifyBuilder",
    "Operation",
    "Request",
    "SignBuilder",
    "SignatureVerifyBuilder",
    "SymmetricKeyBuilder",
]
