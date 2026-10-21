"""Manifest-backed Python facade for the vendor extension registry C ABI.

Sensitive operation-input builders clear the CFFI staging arrays they own
after synchronous ABI calls. The variable TTLV BigInteger, TextString, and
ByteString constructors also clear their staging arrays. Immutable Python
``bytes`` and ``str`` values cannot be overwritten in place, and Python's
runtime may retain external copies of input and accessor values beyond
KMIPKit's control.

Opaque native identity handles expose bounded field getters. Inspection keeps
generic TTLV intact; Python never infers identity by rescanning payload content.
"""

from __future__ import annotations

import sys
from collections.abc import Iterable, Iterator
from contextlib import contextmanager
from dataclasses import dataclass
from enum import IntEnum
from typing import Any, overload

from . import errors, ttlv
from ._ffi import ffi, lib
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._handles import (
    NativeHandle,
    _NATIVE_CALL_LOCK,
    _NATIVE_HANDLE_TOKEN,
    _invoke,
    _invoke_consuming,
    _integer_range,
    _owned_bytes,
    _scalar,
    _wipe_staging_buffer,
    _wrap_owned_handle,
    synchronize_public_functions,
)


@dataclass(frozen=True, slots=True)
class ExtensionRegistryLimits:
    """Per-registry resource ceilings, validated by the native hard bounds."""

    max_definitions: int
    max_schema_nodes: int
    max_child_rules_per_structure: int
    max_text_bytes_per_field: int
    max_registry_text_bytes: int
    max_discriminator_scalar_bytes: int
    max_total_discriminator_scalar_bytes: int
    max_constraint_members_per_rule: int
    max_total_constraint_members: int
    max_payload_index_records: int
    max_lookup_comparisons: int
    max_depth: int


_LIMIT_FIELDS = (
    "max_definitions",
    "max_schema_nodes",
    "max_child_rules_per_structure",
    "max_text_bytes_per_field",
    "max_registry_text_bytes",
    "max_discriminator_scalar_bytes",
    "max_total_discriminator_scalar_bytes",
    "max_constraint_members_per_rule",
    "max_total_constraint_members",
    "max_payload_index_records",
    "max_lookup_comparisons",
    "max_depth",
)

_VERSIONED_LIMITS_TYPE = "kmipkit_extension_registry_limits_v1_t"


def _native_limits(values: tuple[int, ...]) -> Any:
    native = ffi.new(f"{_VERSIONED_LIMITS_TYPE} *")
    native.struct_size = ffi.sizeof(_VERSIONED_LIMITS_TYPE)
    native.abi_version = 1
    for field, value in zip(_LIMIT_FIELDS, values, strict=True):
        setattr(native, field, value)
    return native

_MAX_TEXT_BYTES_PER_FIELD = 4_096
_MAX_UNIQUE_BATCH_ITEM_ID_BYTES = 16 * 1024 * 1024
_MAX_CHILD_RULES_PER_STRUCTURE = 4_096
_MAX_OPERATION_INPUT_BYTES = 16 * 1024 * 1024
_CLIENT_ABI_HANDLE_FUNCTIONS = frozenset(
    {
        "kmipkit_client_batch_item_pkcs11",
        "kmipkit_client_batch_item_ping",
        "kmipkit_client_batch_item_query",
        "kmipkit_client_batch_item_query_with_limits",
        "kmipkit_client_batch_item_rekey",
        "kmipkit_client_batch_item_rekey_key_pair",
        "kmipkit_client_batch_item_reprovision",
        "kmipkit_client_batch_item_rng_retrieve",
        "kmipkit_client_batch_item_rng_seed",
        "kmipkit_client_batch_item_set_constraints",
    }
)


class ExtensionIdentity(NativeHandle):
    _release_name = "kmipkit_extension_identity_release"

    def __init__(
        self,
        handle: Any,
        vendor_identifier: str | None,
        name: str | None,
        version: str | None,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token)
        self.vendor_identifier = vendor_identifier
        self.name = name
        self.version = version


class Compatibility(NativeHandle):
    _release_name = "kmipkit_extension_compatibility_release"


class TtlvPath(NativeHandle):
    _release_name = "kmipkit_ttlv_path_release"

    def __init__(self, handle: Any, tags: tuple[int, ...], *, _token: object) -> None:
        super().__init__(handle, _token=_token)
        self.tags = tags


class Discriminator(NativeHandle):
    _release_name = "kmipkit_extension_discriminator_release"

    def __init__(
        self,
        handle: Any,
        path: TtlvPath,
        scalar_type: ttlv.ItemType,
        scalar_value: object,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token)
        self.path_tags = path.tags
        self.scalar_type = scalar_type
        self.scalar_value = scalar_value


class ExtensionSchema(NativeHandle):
    _release_name = "kmipkit_extension_schema_release"


class ExtensionChildRule(NativeHandle):
    _release_name = "kmipkit_extension_child_rule_release"


class ExtensionOrderConstraint(NativeHandle):
    _release_name = "kmipkit_extension_order_constraint_release"


class ExtensionInformation(NativeHandle):
    _release_name = "kmipkit_extension_information_release"

    def __init__(
        self,
        handle: Any,
        extension_name: str | None,
        tag: int | None = None,
        item_type: ttlv.ItemType | None = None,
        enumeration: int | None = None,
        attribute: bool | None = None,
        parent_structure_tag: int | None = None,
        description: str | None = None,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token)
        self.extension_name = extension_name
        self.tag = tag
        self.item_type = item_type
        self.enumeration = enumeration
        self.attribute = attribute
        self.parent_structure_tag = parent_structure_tag
        self.description = description

    def _replace(self, handle: Any, **changes: object) -> ExtensionInformation:
        values = {
            "extension_name": self.extension_name,
            "tag": self.tag,
            "item_type": self.item_type,
            "enumeration": self.enumeration,
            "attribute": self.attribute,
            "parent_structure_tag": self.parent_structure_tag,
            "description": self.description,
        }
        values.update(changes)
        return _wrap_owned_handle(ExtensionInformation, handle, **values)


class ExtensionDefinition(NativeHandle):
    _release_name = "kmipkit_extension_definition_release"

    def __init__(
        self,
        handle: Any,
        identity: ExtensionIdentity,
        discriminator: Discriminator | None = None,
        information: ExtensionInformation | None = None,
        compatibility: Compatibility | None = None,
        schema: ExtensionSchema | None = None,
        *,
        _token: object,
    ) -> None:
        super().__init__(
            handle,
            _token=_token,
            owner=(identity, discriminator, information, compatibility, schema),
        )
        self.identity = identity
        self.discriminator = discriminator
        self.information = information
        self.compatibility = compatibility
        self.schema = schema


class ClientExtensionRegistry(NativeHandle):
    _release_name = "kmipkit_client_extension_registry_release"

    def __init__(
        self,
        handle: Any,
        definitions: Iterable[ExtensionDefinition],
        limits: ExtensionRegistryLimits,
        *,
        _token: object,
    ) -> None:
        ordered = tuple(sorted(definitions, key=_identity_key))
        super().__init__(handle, _token=_token, owner=ordered)
        self._definitions = ordered
        self.limits = limits


class ClientConfiguration(NativeHandle):
    _release_name = "kmipkit_client_configuration_release"

    def __init__(
        self,
        handle: Any,
        definitions: tuple[ExtensionDefinition, ...],
        limits: ExtensionRegistryLimits,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token, owner=definitions)
        self._definitions = definitions
        self.limits = limits


class ValidatedExtensionValue(NativeHandle):
    _release_name = "kmipkit_validated_extension_value_release"

    def __init__(
        self,
        handle: Any,
        identity: ExtensionIdentity | None = None,
        owner: object | None = None,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token, owner=(identity, owner))
        self.identity = identity


class RegisteredExtensionValue(NativeHandle):
    _release_name = "kmipkit_registered_extension_value_release"

    def __init__(self, handle: Any, identity: ExtensionIdentity, *, _token: object) -> None:
        super().__init__(handle, _token=_token, owner=identity)
        self.identity = identity


class ClientRequestMessageExtension(NativeHandle):
    _release_name = "kmipkit_client_request_message_extension_release"


class ClientBatchItem(NativeHandle):
    _release_name = "kmipkit_client_batch_item_release"


class RevocationReason(NativeHandle):
    """Owned raw-preserving Revocation Reason Code and optional audit message."""

    _release_name = "kmipkit_revocation_reason_release"

    def __init__(
        self, handle: Any, raw_code: int, message: str | None, *, _token: object
    ) -> None:
        super().__init__(handle, _token=_token)
        self.raw_code = raw_code
        self.message = message

    @classmethod
    def create(cls, raw_code: int, message: str | None = None) -> RevocationReason:
        raw = _integer_range(raw_code, 0, 0xFFFF_FFFF)
        if message is not None and not isinstance(message, str):
            errors.raise_for_status(4)
        if message is None:
            encoded = b""
            buffer = ffi.NULL
        else:
            encoded, buffer = _owned_bytes(message, _MAX_OPERATION_INPUT_BYTES)
        try:
            handle = _make_handle(
                "kmipkit_revocation_reason_t",
                "kmipkit_revocation_reason_create",
                (raw, int(message is not None), buffer, len(encoded)),
            )
        finally:
            _wipe_staging_buffer(buffer)
        return _wrap_owned_handle(cls, handle, raw, message)

    def __repr__(self) -> str:
        return (
            f"RevocationReason(raw_code={self.raw_code}, "
            f"message={'redacted' if self.message is not None else 'None'})"
        )


class RevokeRequest(NativeHandle):
    """Owned typed Revoke request with optional identifier and compromise date."""

    _release_name = "kmipkit_revoke_request_release"

    @classmethod
    def create(
        cls,
        unique_identifier: UniqueIdentifier | None,
        reason: RevocationReason,
        compromise_occurrence_date: int | None,
    ) -> RevokeRequest:
        if not isinstance(reason, RevocationReason):
            errors.raise_for_status(4)
        if compromise_occurrence_date is None:
            has_date = 0
            date = 0
        else:
            has_date = 1
            date = _integer_range(compromise_occurrence_date, -(1 << 63), (1 << 63) - 1)
        with _staging_buffer_scope() as staging_buffers:
            identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
            handle = _make_handle(
                "kmipkit_revoke_request_t",
                "kmipkit_revoke_request_create",
                (identifier, reason, has_date, date),
                consumed_indices=(1,),
            )
        return _wrap_owned_handle(cls, handle)


class OpaqueObjectContext(IntEnum):
    """Local Register attribute-handling context for an Opaque Object."""

    GENERIC = 1
    MANAGED_CRYPTOGRAPHIC_OBJECT = 2


class UniqueIdentifierKind(IntEnum):
    """Exact TTLV wire type used by a KMIP Unique Identifier."""

    TEXT_STRING = 1
    ENUMERATION = 2
    INTEGER = 3


@dataclass(frozen=True, slots=True, repr=False)
class UniqueIdentifier:
    """Exact KMIP Unique Identifier value with its original wire type."""

    kind: UniqueIdentifierKind
    text: str | None = None
    number: int | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.kind, UniqueIdentifierKind):
            errors.raise_for_status(4)
        if self.kind is UniqueIdentifierKind.TEXT_STRING:
            if self.text is None or self.number is not None:
                errors.raise_for_status(4)
            _owned_bytes(self.text, _MAX_UNIQUE_BATCH_ITEM_ID_BYTES)
        elif self.kind is UniqueIdentifierKind.ENUMERATION:
            if self.text is not None or self.number is None:
                errors.raise_for_status(4)
            _integer_range(self.number, 0, 0xFFFF_FFFF)
        else:
            if self.text is not None or self.number is None:
                errors.raise_for_status(4)
            _integer_range(self.number, -(1 << 31), (1 << 31) - 1)

    @classmethod
    def text_string(cls, value: str) -> UniqueIdentifier:
        return cls(UniqueIdentifierKind.TEXT_STRING, text=value)

    @classmethod
    def enumeration(cls, value: int) -> UniqueIdentifier:
        return cls(UniqueIdentifierKind.ENUMERATION, number=value)

    @classmethod
    def integer(cls, value: int) -> UniqueIdentifier:
        return cls(UniqueIdentifierKind.INTEGER, number=value)

    def __repr__(self) -> str:
        return f"UniqueIdentifier(kind={self.kind.name}, value=redacted)"


class SplitKeyReference(NativeHandle):
    """Owned per-occurrence Join Split Key identifier and caller-known threshold."""

    _release_name = "kmipkit_split_key_reference_release"
    __slots__ = ("_unique_identifier", "_split_key_threshold")

    def __new__(cls, *arguments: object, **keywords: object) -> SplitKeyReference:
        del arguments, keywords
        return object.__new__(cls)

    def __init__(self, unique_identifier: UniqueIdentifier, split_key_threshold: int) -> None:
        if not isinstance(unique_identifier, UniqueIdentifier):
            errors.raise_for_status(4)
        threshold = _integer_range(split_key_threshold, 0, 0xFFFF_FFFF)
        with _staging_buffer_scope() as staging_buffers:
            native_identifier, _owner = _native_unique_identifier(
                unique_identifier, staging_buffers
            )
            handle = _make_handle(
                "kmipkit_split_key_reference_t",
                "kmipkit_split_key_reference_create",
                (native_identifier, threshold),
            )
        super().__init__(handle, _token=_NATIVE_HANDLE_TOKEN)
        self._unique_identifier = unique_identifier
        self._split_key_threshold = threshold

    @property
    def unique_identifier(self) -> UniqueIdentifier:
        """Return the identifier paired with this reference's local threshold."""
        return self._unique_identifier

    @property
    def split_key_threshold(self) -> int:
        """Return this occurrence's immutable local threshold context."""
        return self._split_key_threshold

    def __repr__(self) -> str:
        return f"SplitKeyReference(unique_identifier=redacted, split_key_threshold={self.split_key_threshold})"


class RegisterRequest(NativeHandle):
    """Owned typed Register request with an optional local Opaque Object context."""

    _release_name = "kmipkit_register_request_release"

    @classmethod
    def create(
        cls, object_type: int, attributes: ttlv.Structure, object: ttlv.TtlvItem
    ) -> RegisterRequest:
        if (
            not isinstance(object_type, int)
            or isinstance(object_type, bool)
            or not 0 <= object_type <= 0xFFFF_FFFF
        ):
            errors.raise_for_status(4)
        handle = _make_handle(
            "kmipkit_register_request_t",
            "kmipkit_register_request_create",
            (object_type, attributes, object),
            consumed_indices=(1, 2),
        )
        return _wrap_owned_handle(cls, handle)

    def with_opaque_object_context(
        self, context: OpaqueObjectContext
    ) -> RegisterRequest:
        if not isinstance(context, OpaqueObjectContext):
            errors.raise_for_status(4)
        handle = _make_handle(
            "kmipkit_register_request_t",
            "kmipkit_register_request_with_opaque_object_context",
            (self, int(context)),
            consumed_indices=(0,),
        )
        return _wrap_owned_handle(RegisterRequest, handle)


class ExtensionRecognition(NativeHandle):
    _release_name = "kmipkit_extension_recognition_release"

    def __init__(
        self,
        handle: Any,
        registry: ClientExtensionRegistry,
        vendor_identifier: str,
        value: ttlv.Structure,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token, owner=(registry, value))
        self._registry = registry
        self._vendor_identifier = vendor_identifier
        self._value = value


def _identity_key(definition: ExtensionDefinition) -> tuple[str, str, str]:
    if (
        definition.identity.vendor_identifier is None
        or definition.identity.name is None
        or definition.identity.version is None
    ):
        errors.raise_for_status(4)
    return (
        definition.identity.vendor_identifier,
        definition.identity.name,
        definition.identity.version,
    )


def _identity_tuple(identity: ExtensionIdentity) -> tuple[str, str, str]:
    if (
        identity.vendor_identifier is None
        or identity.name is None
        or identity.version is None
    ):
        errors.raise_for_status(4)
    return identity.vendor_identifier, identity.name, identity.version


def _make_handle(
    c_type: str,
    function_name: str,
    arguments: tuple[object, ...],
    out_index: int | None = None,
    consumed_indices: tuple[int, ...] = (),
) -> Any:
    native_ffi = (
        _client_ffi
        if function_name in _CLIENT_ABI_HANDLE_FUNCTIONS
        else ffi
    )
    native_lib = (
        _client_lib
        if function_name in _CLIENT_ABI_HANDLE_FUNCTIONS
        else lib
    )
    output = native_ffi.new(f"{c_type} **")
    call_arguments = list(arguments)
    if out_index is None:
        call_arguments.append(output)
    else:
        call_arguments.insert(out_index, output)
    function = getattr(native_lib, function_name)
    status = (
        _invoke_consuming(function, tuple(call_arguments), consumed_indices)
        if consumed_indices
        else _invoke(function, *call_arguments)
    )
    errors.raise_for_status(status)
    if output[0] == native_ffi.NULL:
        errors.raise_for_status(4)
    if native_ffi is ffi:
        return output[0]
    return ffi.cast(c_type + " *", int(native_ffi.cast("uintptr_t", output[0])))


@contextmanager
def _staging_buffer_scope() -> Iterator[list[Any]]:
    """Keep staged inputs registered until the builder completes or fails."""
    buffers: list[Any] = []
    try:
        yield buffers
    finally:
        for buffer in buffers:
            _wipe_staging_buffer(buffer)


def _make_bytes(value: str) -> tuple[bytes, Any]:
    if not isinstance(value, str):
        errors.raise_for_status(4)
    return _owned_bytes(value, _MAX_TEXT_BYTES_PER_FIELD)


def _bounded_snapshot(values: object, maximum: int | None) -> tuple[Any, ...]:
    """Snapshot a declared list or tuple only after checking its true size."""
    if isinstance(values, list):
        count = list.__len__(values)
        if maximum is not None and count > maximum:
            errors.raise_for_status(6)
        try:
            return tuple(list.__iter__(values))
        except MemoryError:
            errors.raise_for_status(6)
    if isinstance(values, tuple):
        count = tuple.__len__(values)
        if maximum is not None and count > maximum:
            errors.raise_for_status(6)
        try:
            return tuple(tuple.__iter__(values))
        except MemoryError:
            errors.raise_for_status(6)
    errors.raise_for_status(4)


def _sequence_length(values: list[Any] | tuple[Any, ...]) -> int:
    """Read the backing list or tuple length without invoking subclass overrides."""
    if isinstance(values, list):
        return list.__len__(values)
    return tuple.__len__(values)


def create_extension_identity(
    vendor_identifier: str, name: str, version: str
) -> ExtensionIdentity:
    vendor, vendor_data = _make_bytes(vendor_identifier)
    raw_name, name_data = _make_bytes(name)
    raw_version, version_data = _make_bytes(version)
    handle = _make_handle(
        "kmipkit_extension_identity_t",
        "kmipkit_extension_identity_create",
        (
            vendor_data,
            len(vendor),
            name_data,
            len(raw_name),
            version_data,
            len(raw_version),
        ),
    )
    return _wrap_owned_handle(
        ExtensionIdentity, handle, vendor_identifier, name, version
    )


def create_compatibility(
    kmip_min_major: int,
    kmip_min_minor: int,
    kmip_max_major: int,
    kmip_max_minor: int,
    kmipkit_minimum: str,
    kmipkit_maximum: str,
) -> Compatibility:
    minimum, minimum_data = _make_bytes(kmipkit_minimum)
    maximum, maximum_data = _make_bytes(kmipkit_maximum)
    handle = _make_handle(
        "kmipkit_extension_compatibility_t",
        "kmipkit_extension_compatibility_create",
        (
            kmip_min_major,
            kmip_min_minor,
            kmip_max_major,
            kmip_max_minor,
            minimum_data,
            len(minimum),
            maximum_data,
            len(maximum),
        ),
    )
    return _wrap_owned_handle(Compatibility, handle)


def create_ttlv_path(first_tag: ttlv.Tag) -> TtlvPath:
    value = ttlv.tag_value(first_tag)
    handle = _make_handle("kmipkit_ttlv_path_t", "kmipkit_ttlv_path_create", (value,))
    return _wrap_owned_handle(TtlvPath, handle, (value,))


def with_child_tag(path: TtlvPath, tag: ttlv.Tag) -> TtlvPath:
    value = ttlv.tag_value(tag)
    handle = _make_handle(
        "kmipkit_ttlv_path_t",
        "kmipkit_ttlv_path_with_child_tag",
        (path, value),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(TtlvPath, handle, (*path.tags, value))


def create_discriminator(path: TtlvPath, scalar_value: ttlv.Value) -> Discriminator:
    scalar_view = ttlv.ttlv_value_view(scalar_value)
    try:
        scalar_type = ttlv.ttlv_value_view_type(scalar_view)
        scalar_value_copy = _view_scalar(scalar_view, scalar_type)
    finally:
        scalar_view.close()
    handle = _make_handle(
        "kmipkit_extension_discriminator_t",
        "kmipkit_extension_discriminator_create",
        (path, scalar_value),
        consumed_indices=(0, 1),
    )
    return _wrap_owned_handle(
        Discriminator, handle, path, scalar_type, scalar_value_copy
    )


def scalar_schema(item_type: ttlv.ItemType) -> ExtensionSchema:
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        "kmipkit_extension_schema_scalar",
        (int(item_type),),
    )
    return _wrap_owned_handle(ExtensionSchema, handle)


def structure_schema(
    children: list[ExtensionChildRule] | tuple[ExtensionChildRule, ...],
    order_constraints: list[ExtensionOrderConstraint]
    | tuple[ExtensionOrderConstraint, ...],
    preserve_undeclared_children: bool,
) -> ExtensionSchema:
    if not isinstance(preserve_undeclared_children, bool):
        errors.raise_for_status(4)
    child_snapshot = _bounded_snapshot(children, _MAX_CHILD_RULES_PER_STRUCTURE)
    order_snapshot = _bounded_snapshot(order_constraints, _MAX_CHILD_RULES_PER_STRUCTURE)
    child_array = (
        ffi.NULL
        if not child_snapshot
        else ffi.new(
            "kmipkit_extension_child_rule_t *[]",
            [child._pointer() for child in child_snapshot],
        )
    )
    order_array = (
        ffi.NULL
        if not order_snapshot
        else ffi.new(
            "kmipkit_extension_order_constraint_t *[]",
            [constraint._pointer() for constraint in order_snapshot],
        )
    )
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        "kmipkit_extension_schema_structure",
        (
            child_array,
            len(child_snapshot),
            order_array,
            len(order_snapshot),
            int(preserve_undeclared_children),
        ),
    )
    return _wrap_owned_handle(
        ExtensionSchema, handle, owner=(child_snapshot, order_snapshot)
    )


def _child_rule(
    tag: ttlv.Tag, schema: ExtensionSchema, mode: str
) -> ExtensionChildRule:
    tag_value = ttlv.tag_value(tag)
    handle = _make_handle(
        "kmipkit_extension_child_rule_t",
        f"kmipkit_extension_child_rule_{mode}",
        (tag_value, schema._pointer()),
    )
    return _wrap_owned_handle(ExtensionChildRule, handle, owner=(tag, schema))


def required_child_rule(tag: ttlv.Tag, schema: ExtensionSchema) -> ExtensionChildRule:
    return _child_rule(tag, schema, "required")


def optional_child_rule(tag: ttlv.Tag, schema: ExtensionSchema) -> ExtensionChildRule:
    return _child_rule(tag, schema, "optional")


def repeated_child_rule(tag: ttlv.Tag, schema: ExtensionSchema) -> ExtensionChildRule:
    return _child_rule(tag, schema, "repeated")


def create_extension_order_constraint(
    before_tag: ttlv.Tag, after_tag: ttlv.Tag
) -> ExtensionOrderConstraint:
    handle = _make_handle(
        "kmipkit_extension_order_constraint_t",
        "kmipkit_extension_order_constraint_create",
        (ttlv.tag_value(before_tag), ttlv.tag_value(after_tag)),
    )
    return _wrap_owned_handle(
        ExtensionOrderConstraint, handle, owner=(before_tag, after_tag)
    )


def _schema_transform(
    schema: ExtensionSchema, symbol: str, *values: int
) -> ExtensionSchema:
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        symbol,
        (schema, *values),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(ExtensionSchema, handle)


def with_minimum_length(schema: ExtensionSchema, value: int) -> ExtensionSchema:
    return _schema_transform(schema, "kmipkit_extension_schema_minimum_length", value)


def with_maximum_length(schema: ExtensionSchema, value: int) -> ExtensionSchema:
    return _schema_transform(schema, "kmipkit_extension_schema_maximum_length", value)


def with_signed_range(
    schema: ExtensionSchema, minimum: int, maximum: int
) -> ExtensionSchema:
    return _schema_transform(
        schema, "kmipkit_extension_schema_signed_numeric_range", minimum, maximum
    )


def with_unsigned_range(
    schema: ExtensionSchema, minimum: int, maximum: int
) -> ExtensionSchema:
    return _schema_transform(
        schema, "kmipkit_extension_schema_unsigned_numeric_range", minimum, maximum
    )


def with_allowed_enumeration(schema: ExtensionSchema, value: int) -> ExtensionSchema:
    return _schema_transform(
        schema, "kmipkit_extension_schema_allowed_enumeration", value
    )


def with_allowed_bit_mask(schema: ExtensionSchema, value: int) -> ExtensionSchema:
    return _schema_transform(schema, "kmipkit_extension_schema_allowed_bit_mask", value)


def with_required_bit_mask(schema: ExtensionSchema, value: int) -> ExtensionSchema:
    return _schema_transform(
        schema, "kmipkit_extension_schema_required_bit_mask", value
    )


def create_extension_definition(
    identity: ExtensionIdentity,
    compatibility: Compatibility,
    discriminator: Discriminator,
    schema: ExtensionSchema,
) -> ExtensionDefinition:
    handle = _make_handle(
        "kmipkit_extension_definition_t",
        "kmipkit_extension_definition_create",
        (
            identity._pointer(),
            compatibility._pointer(),
            discriminator._pointer(),
            schema._pointer(),
        ),
    )
    return _wrap_owned_handle(
        ExtensionDefinition,
        handle, identity, discriminator, compatibility=compatibility, schema=schema
    )


@overload
def validate_extension_value(
    definition: ExtensionDefinition,
    value: ttlv.Structure,
    limits: ttlv.CodecLimits,
) -> ValidatedExtensionValue: ...


@overload
def validate_extension_value(
    registry: ClientExtensionRegistry,
    identity: ExtensionIdentity,
    value: ttlv.Structure,
    limits: ttlv.CodecLimits,
) -> RegisteredExtensionValue: ...


def validate_extension_value(
    target: ExtensionDefinition | ClientExtensionRegistry,
    *arguments: object,
) -> ValidatedExtensionValue | RegisteredExtensionValue:
    if isinstance(target, ExtensionDefinition):
        if (
            len(arguments) != 2
            or not isinstance(arguments[0], ttlv.Structure)
            or not isinstance(arguments[1], ttlv.CodecLimits)
        ):
            errors.raise_for_status(4)
        value, limits = arguments
        handle = _make_handle(
            "kmipkit_validated_extension_value_t",
            "kmipkit_extension_definition_validate",
            (target._pointer(), value._pointer(), limits._pointer()),
            out_index=2,
        )
        return _wrap_owned_handle(
            ValidatedExtensionValue, handle, target.identity, owner=target
        )
    if not isinstance(target, ClientExtensionRegistry):
        errors.raise_for_status(4)
    if (
        len(arguments) != 3
        or not isinstance(arguments[0], ExtensionIdentity)
        or not isinstance(arguments[1], ttlv.Structure)
        or not isinstance(arguments[2], ttlv.CodecLimits)
    ):
        errors.raise_for_status(4)
    identity, value, limits = arguments
    handle = _make_handle(
        "kmipkit_registered_extension_value_t",
        "kmipkit_client_extension_registry_validate",
        (target._pointer(), identity._pointer(), value._pointer(), limits._pointer()),
        out_index=4,
    )
    return _wrap_owned_handle(RegisteredExtensionValue, handle, identity)


def create_extension_registry_limits(*values: int) -> ExtensionRegistryLimits:
    if len(values) != len(_LIMIT_FIELDS):
        errors.raise_for_status(4)
    if any(
        not isinstance(value, int) or isinstance(value, bool) or value < 0
        for value in values
    ):
        errors.raise_for_status(4)
    if any(value > 0xFFFF_FFFF_FFFF_FFFF for value in values):
        errors.raise_for_status(6)
    status = _invoke(
        lib.kmipkit_extension_registry_limits_v1_validate, _native_limits(values)
    )
    errors.raise_for_status(status)
    return ExtensionRegistryLimits(**dict(zip(_LIMIT_FIELDS, values, strict=True)))


def default_extension_registry_limits() -> ExtensionRegistryLimits:
    output = _native_limits((0,) * len(_LIMIT_FIELDS))
    errors.raise_for_status(
        _invoke(lib.kmipkit_extension_registry_limits_v1_default_values, output)
    )
    return ExtensionRegistryLimits(
        **{field: int(getattr(output, field)) for field in _LIMIT_FIELDS}
    )


def create_client_extension_registry(
    definitions: list[ExtensionDefinition] | tuple[ExtensionDefinition, ...],
    limits: ExtensionRegistryLimits,
) -> ClientExtensionRegistry:
    if not isinstance(limits, ExtensionRegistryLimits):
        errors.raise_for_status(4)
    limit_values = tuple(getattr(limits, field) for field in _LIMIT_FIELDS)
    if any(
        not isinstance(value, int) or isinstance(value, bool) or value < 0
        for value in limit_values
    ):
        errors.raise_for_status(4)
    if any(value > 0xFFFF_FFFF_FFFF_FFFF for value in limit_values):
        errors.raise_for_status(6)
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_extension_registry_limits_v1_validate,
            _native_limits(limit_values),
        )
    )
    bounded_definitions = _bounded_snapshot(definitions, limit_values[0])
    source = tuple(sorted(bounded_definitions, key=_identity_key))
    definition_array = (
        ffi.NULL
        if not source
        else ffi.new(
            "kmipkit_extension_definition_t *[]",
            [definition._pointer() for definition in source],
        )
    )
    handle = _make_handle(
        "kmipkit_client_extension_registry_t",
        "kmipkit_client_extension_registry_create_v1",
        (definition_array, len(source), _native_limits(limit_values)),
    )
    return _wrap_owned_handle(ClientExtensionRegistry, handle, source, limits)


def create_client_configuration(
    registry: ClientExtensionRegistry,
) -> ClientConfiguration:
    definitions = registry._definitions
    limits = registry.limits
    handle = _make_handle(
        "kmipkit_client_configuration_t",
        "kmipkit_client_configuration_create",
        (registry,),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(ClientConfiguration, handle, definitions, limits)


def client_configuration_extension_registry(
    configuration: ClientConfiguration,
) -> ClientExtensionRegistry:
    handle = _make_handle(
        "kmipkit_client_extension_registry_t",
        "kmipkit_client_configuration_extension_registry",
        (configuration._pointer(),),
    )
    return _wrap_owned_handle(
        ClientExtensionRegistry,
        handle, configuration._definitions, configuration.limits
    )


def definition_count(registry: ClientExtensionRegistry) -> int:
    return _scalar(
        "uint64_t",
        lib.kmipkit_client_extension_registry_definition_count,
        registry._pointer(),
    )


def definition_at(
    registry: ClientExtensionRegistry, index: int
) -> ExtensionDefinition | None:
    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
        errors.raise_for_status(4)
    registry_pointer = registry._pointer()
    if index >= len(registry._definitions):
        return None
    output = ffi.new("kmipkit_extension_definition_t **")
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_client_extension_registry_definition_at,
            registry_pointer,
            index,
            output,
        )
    )
    if output[0] == ffi.NULL:
        return None
    metadata = registry._definitions[index]
    return _wrap_owned_handle(
        ExtensionDefinition,
        output[0],
        metadata.identity,
        metadata.discriminator,
        metadata.information,
        metadata.compatibility,
        metadata.schema,
    )


def definition_for_identity(
    registry: ClientExtensionRegistry, identity: ExtensionIdentity
) -> ExtensionDefinition | None:
    output = ffi.new("kmipkit_extension_definition_t **")
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_client_extension_registry_definition_for_identity,
            registry._pointer(),
            identity._pointer(),
            output,
        )
    )
    if output[0] == ffi.NULL:
        return None
    metadata_by_key = {_identity_key(item): item for item in registry._definitions}
    metadata = metadata_by_key.get(_identity_tuple(identity))
    if metadata is None:
        return _wrap_owned_handle(ExtensionDefinition, output[0], identity)
    return _wrap_owned_handle(
        ExtensionDefinition,
        output[0],
        metadata.identity,
        metadata.discriminator,
        metadata.information,
        metadata.compatibility,
        metadata.schema,
    )


def extension_definition_identity(definition: ExtensionDefinition) -> ExtensionIdentity:
    handle = _make_handle(
        "kmipkit_extension_identity_t",
        "kmipkit_extension_definition_identity",
        (definition._pointer(),),
    )
    identity = definition.identity
    return _wrap_owned_handle(
        ExtensionIdentity,
        handle, identity.vendor_identifier, identity.name, identity.version
    )


def _identity_text(identity: ExtensionIdentity, symbol: str) -> str:
    value = _wrap_owned_handle(
        ttlv.Value,
        _make_handle(
            "kmipkit_ttlv_value_t",
            symbol,
            (identity._pointer(),),
        )
    )
    try:
        with ttlv.ttlv_value_view(value) as view:
            if ttlv.ttlv_value_view_type(view) is not ttlv.ItemType.TextString:
                errors.raise_for_status(4)
            try:
                return _view_bytes(view).decode("utf-8")
            except UnicodeDecodeError:
                raise errors.InvalidInputError() from None
    finally:
        value.close()


def extension_identity_vendor_identifier(identity: ExtensionIdentity) -> str:
    """Return a copied vendor identifier from an opaque native identity."""
    return _identity_text(identity, "kmipkit_extension_identity_vendor_identifier")


def extension_identity_name(identity: ExtensionIdentity) -> str:
    """Return a copied extension name from an opaque native identity."""
    return _identity_text(identity, "kmipkit_extension_identity_name")


def extension_identity_version(identity: ExtensionIdentity) -> str:
    """Return a copied version from an opaque native identity."""
    return _identity_text(identity, "kmipkit_extension_identity_version")


def _populate_identity_fields(identity: ExtensionIdentity) -> ExtensionIdentity:
    identity.vendor_identifier = extension_identity_vendor_identifier(identity)
    identity.name = extension_identity_name(identity)
    identity.version = extension_identity_version(identity)
    return identity


def extension_definition_information(
    definition: ExtensionDefinition,
) -> ExtensionInformation | None:
    output = ffi.new("kmipkit_extension_information_t **")
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_extension_definition_information, definition._pointer(), output
        )
    )
    if output[0] == ffi.NULL:
        return None
    metadata = definition.information
    if metadata is None:
        return _wrap_owned_handle(ExtensionInformation, output[0], None)
    return _wrap_owned_handle(
        ExtensionInformation,
        output[0],
        metadata.extension_name,
        metadata.tag,
        metadata.item_type,
        metadata.enumeration,
        metadata.attribute,
        metadata.parent_structure_tag,
        metadata.description,
    )


def create_extension_information(extension_name: str) -> ExtensionInformation:
    raw, data = _make_bytes(extension_name)
    handle = _make_handle(
        "kmipkit_extension_information_t",
        "kmipkit_extension_information_create",
        (data, len(raw)),
    )
    return _wrap_owned_handle(ExtensionInformation, handle, extension_name)


def _with_information(
    information: ExtensionInformation, symbol: str, value: object, **metadata: object
) -> ExtensionInformation:
    if isinstance(value, str):
        raw, data = _make_bytes(value)
        arguments = (information, data, len(raw))
    else:
        arguments = (information, value)
    handle = _make_handle(
        "kmipkit_extension_information_t",
        symbol,
        arguments,
        consumed_indices=(0,),
    )
    return information._replace(handle, **metadata)


def with_tag(information: ExtensionInformation, tag: int) -> ExtensionInformation:
    return _with_information(
        information, "kmipkit_extension_information_tag", tag, tag=tag
    )


def with_type(
    information: ExtensionInformation, item_type: ttlv.ItemType
) -> ExtensionInformation:
    if not isinstance(item_type, ttlv.ItemType) or not 1 <= int(item_type) <= 11:
        errors.raise_for_status(4)
    return _with_information(
        information,
        "kmipkit_extension_information_type",
        int(item_type),
        item_type=item_type,
    )


def with_enumeration(
    information: ExtensionInformation, enumeration: int
) -> ExtensionInformation:
    return _with_information(
        information,
        "kmipkit_extension_information_enumeration",
        enumeration,
        enumeration=enumeration,
    )


def with_attribute(
    information: ExtensionInformation, attribute: bool
) -> ExtensionInformation:
    if not isinstance(attribute, bool):
        errors.raise_for_status(4)
    return _with_information(
        information,
        "kmipkit_extension_information_attribute",
        int(attribute),
        attribute=attribute,
    )


def with_parent_structure_tag(
    information: ExtensionInformation, parent_structure_tag: int
) -> ExtensionInformation:
    return _with_information(
        information,
        "kmipkit_extension_information_parent_structure_tag",
        parent_structure_tag,
        parent_structure_tag=parent_structure_tag,
    )


def with_description(
    information: ExtensionInformation, description: str
) -> ExtensionInformation:
    return _with_information(
        information,
        "kmipkit_extension_information_description",
        description,
        description=description,
    )


def extension_information_to_ttlv(information: ExtensionInformation) -> ttlv.Structure:
    handle = _make_handle(
        "kmipkit_ttlv_structure_t",
        "kmipkit_extension_information_to_ttlv",
        (information._pointer(),),
    )
    return _wrap_owned_handle(ttlv.Structure, handle, owner=information)


def with_extension_information(
    definition: ExtensionDefinition, information: ExtensionInformation
) -> ExtensionDefinition:
    source = definition
    if (
        definition.compatibility is not None
        and definition.discriminator is not None
        and definition.schema is not None
    ):
        cloned_handle = _make_handle(
            "kmipkit_extension_definition_t",
            "kmipkit_extension_definition_create",
            (
                definition.identity._pointer(),
                definition.compatibility._pointer(),
                definition.discriminator._pointer(),
                definition.schema._pointer(),
            ),
        )
        source = _wrap_owned_handle(
            ExtensionDefinition,
            cloned_handle,
            definition.identity,
            definition.discriminator,
            definition.information,
            definition.compatibility,
            definition.schema,
        )
    handle = _make_handle(
        "kmipkit_extension_definition_t",
        "kmipkit_extension_definition_with_information",
        (source, information._pointer()),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(
        ExtensionDefinition,
        handle,
        definition.identity,
        definition.discriminator,
        information,
        definition.compatibility,
        definition.schema,
    )


def inspect_extension(
    registry: ClientExtensionRegistry,
    vendor_identifier: str,
    value: ttlv.Structure,
    limits: ttlv.CodecLimits,
) -> ExtensionRecognition:
    vendor, vendor_data = _make_bytes(vendor_identifier)
    handle = _make_handle(
        "kmipkit_extension_recognition_t",
        "kmipkit_client_extension_registry_inspect",
        (
            registry._pointer(),
            vendor_data,
            len(vendor),
            value._pointer(),
            limits._pointer(),
        ),
        out_index=4,
    )
    return _wrap_owned_handle(
        ExtensionRecognition, handle, registry, vendor_identifier, value
    )


def is_recognized(recognition: ExtensionRecognition) -> bool:
    recognized = bool(
        _scalar(
            "uint32_t",
            lib.kmipkit_extension_recognition_is_recognized,
            recognition._pointer(),
        )
    )
    return recognized


def _view_bytes(view: ttlv.TtlvValueView) -> bytes:
    return bytes(
        ttlv.ttlv_value_view_byte_at(view, index)
        for index in range(ttlv.ttlv_value_view_byte_length(view))
    )


def _view_scalar(view: ttlv.TtlvValueView, item_type: ttlv.ItemType) -> object:
    functions = {
        ttlv.ItemType.Integer: ttlv.ttlv_value_view_integer,
        ttlv.ItemType.LongInteger: ttlv.ttlv_value_view_long_integer,
        ttlv.ItemType.Enumeration: ttlv.ttlv_value_view_enumeration,
        ttlv.ItemType.Boolean: ttlv.ttlv_value_view_boolean,
        ttlv.ItemType.DateTime: ttlv.ttlv_value_view_date_time,
        ttlv.ItemType.Interval: ttlv.ttlv_value_view_interval,
        ttlv.ItemType.DateTimeExtended: ttlv.ttlv_value_view_date_time_extended,
    }
    if item_type in (
        ttlv.ItemType.TextString,
        ttlv.ItemType.ByteString,
        ttlv.ItemType.BigInteger,
    ):
        return _view_bytes(view)
    getter = functions.get(item_type)
    return getter(view) if getter is not None else None


def validated_value(
    recognition: ExtensionRecognition,
) -> ValidatedExtensionValue | None:
    output = ffi.new("kmipkit_validated_extension_value_t **")
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_extension_recognition_validated_value,
            recognition._pointer(),
            output,
        )
    )
    if output[0] == ffi.NULL:
        return None
    return _wrap_owned_handle(ValidatedExtensionValue, output[0], owner=recognition)


def extension_recognition_generic_value(
    recognition: ExtensionRecognition,
) -> ttlv.TtlvStructureView:
    handle = _make_handle(
        "kmipkit_ttlv_structure_view_t",
        "kmipkit_extension_recognition_generic_value",
        (recognition._pointer(),),
    )
    return _wrap_owned_handle(ttlv.TtlvStructureView, handle, owner=recognition)


def validated_extension_identity(value: ValidatedExtensionValue) -> ExtensionIdentity:
    handle = _make_handle(
        "kmipkit_extension_identity_t",
        "kmipkit_validated_extension_value_identity",
        (value._pointer(),),
    )
    return _populate_identity_fields(
        _wrap_owned_handle(ExtensionIdentity, handle, None, None, None)
    )


def validated_extension_value_generic_value(
    value: ValidatedExtensionValue,
) -> ttlv.TtlvStructureView:
    handle = _make_handle(
        "kmipkit_ttlv_structure_view_t",
        "kmipkit_validated_extension_value_generic_value",
        (value._pointer(),),
    )
    return _wrap_owned_handle(ttlv.TtlvStructureView, handle, owner=value)


def value_at(value: ValidatedExtensionValue, path: TtlvPath) -> ttlv.TtlvValueView:
    handle = _make_handle(
        "kmipkit_ttlv_value_view_t",
        "kmipkit_validated_extension_value_value_at",
        (value._pointer(), path._pointer()),
    )
    return _wrap_owned_handle(ttlv.TtlvValueView, handle, owner=(value, path))


def create_client_request_message_extension(
    value: RegisteredExtensionValue, criticality_indicator: bool
) -> ClientRequestMessageExtension:
    if not isinstance(criticality_indicator, bool):
        errors.raise_for_status(4)
    handle = _make_handle(
        "kmipkit_client_request_message_extension_t",
        "kmipkit_client_request_message_extension_create",
        (value, int(criticality_indicator)),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(ClientRequestMessageExtension, handle)


def with_extension(
    item: ClientBatchItem, extension: ClientRequestMessageExtension
) -> ClientBatchItem:
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_with_extension",
        (item, extension),
        consumed_indices=(0, 1),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def with_unique_batch_item_id(
    item: ClientBatchItem, identifier: bytes | bytearray | memoryview
) -> ClientBatchItem:
    """Return a new request item with the caller-selected opaque identity bytes."""
    if not isinstance(item, ClientBatchItem) or not isinstance(
        identifier, (bytes, bytearray, memoryview)
    ):
        errors.raise_for_status(4)
    with _staging_buffer_scope() as staging_buffers:
        raw, data = _owned_bytes(identifier, _MAX_UNIQUE_BATCH_ITEM_ID_BYTES)
        staging_buffers.append(data)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_with_unique_batch_item_id",
            (item, data, len(raw)),
            consumed_indices=(0,),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_discover_versions() -> ClientBatchItem:
    """Create an empty Discover Versions request batch item."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_discover_versions",
        (),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_ping() -> ClientBatchItem:
    """Create an empty client-initiated KMIP Ping request batch item."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_ping",
        (),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_logout(ticket: ttlv.Structure) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.31 Logout item from its required Ticket Structure."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_logout",
        (_native_structure(ticket, required=True),),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def _validate_query_ttlv_text_length(length: int) -> int:
    if length < 0 or length > 0xFFFF_FFFF:
        errors.raise_for_status(6)
    return length


def _validate_query_ttlv_structure_length(length: int) -> int:
    if length < 0 or length > 0xFFFF_FFFF:
        errors.raise_for_status(6)
    return length


def _client_batch_item_query(
    functions: list[int] | tuple[int, ...],
    object_groups: list[str] | tuple[str, ...] | None = None,
    *,
    limits: ttlv.CodecLimits | None = None,
) -> ClientBatchItem:
    """Create Query while preserving order and duplicates, within explicit codec limits."""
    if limits is not None and not isinstance(limits, ttlv.CodecLimits):
        errors.raise_for_status(4)
    if not isinstance(functions, (list, tuple)):
        errors.raise_for_status(4)
    function_count = _sequence_length(functions)
    if function_count == 0:
        errors.raise_for_status(4)
    if function_count > sys.maxsize // _client_ffi.sizeof("uint32_t"):
        errors.raise_for_status(6)

    max_elements = (
        100_000 if limits is None else ttlv.codec_limits_max_elements(limits)
    )
    max_message_bytes = (
        16 * 1024 * 1024
        if limits is None
        else ttlv.codec_limits_max_message_bytes(limits)
    )
    if object_groups is not None and not isinstance(object_groups, (list, tuple)):
        errors.raise_for_status(4)
    max_structure_depth = (
        64 if limits is None else ttlv.codec_limits_max_structure_depth(limits)
    )
    group_count = 0 if object_groups is None else _sequence_length(object_groups)
    required_elements = function_count + 1
    estimated_bytes = 8 + function_count * 16
    required_depth = 3 if object_groups is None else 4
    if required_depth > max_structure_depth:
        errors.raise_for_status(6)
    if object_groups is not None:
        required_elements += 1 + group_count
        if group_count > sys.maxsize // _client_ffi.sizeof(
            "kmipkit_byte_span_v1_t"
        ):
            errors.raise_for_status(6)
        estimated_bytes += 8
    if required_elements > max_elements or estimated_bytes > max_message_bytes:
        errors.raise_for_status(6)

    function_values = _bounded_snapshot(functions, max_elements)
    group_values = (
        None
        if object_groups is None
        else _bounded_snapshot(object_groups, max_elements)
    )
    function_count = len(function_values)
    group_count = 0 if group_values is None else len(group_values)
    required_elements = function_count + 1
    estimated_bytes = 8 + function_count * 16
    object_groups_value_length = 0
    if group_values is not None:
        required_elements += 1 + group_count
        estimated_bytes += 8
        if group_count > sys.maxsize // _client_ffi.sizeof(
            "kmipkit_byte_span_v1_t"
        ):
            errors.raise_for_status(6)
        for group in group_values:
            if not isinstance(group, str):
                errors.raise_for_status(4)
            utf8_length = 0
            character_count = str.__len__(group)
            if character_count > max_message_bytes - min(estimated_bytes, max_message_bytes):
                errors.raise_for_status(6)
            for index in range(character_count):
                codepoint = ord(str.__getitem__(group, index))
                if 0xD800 <= codepoint <= 0xDFFF:
                    errors.raise_for_status(4)
                utf8_length += (
                    1 if codepoint <= 0x7F else
                    2 if codepoint <= 0x7FF else
                    3 if codepoint <= 0xFFFF else 4
                )
            _validate_query_ttlv_text_length(utf8_length)
            item_bytes = 8 + ((utf8_length + 7) & ~7)
            object_groups_value_length += item_bytes
            _validate_query_ttlv_structure_length(object_groups_value_length)
            estimated_bytes += item_bytes
            if estimated_bytes > max_message_bytes:
                errors.raise_for_status(6)
    _validate_query_ttlv_structure_length(estimated_bytes - 8)
    if required_elements > max_elements or estimated_bytes > max_message_bytes:
        errors.raise_for_status(6)

    raw_functions = [
        _integer_range(value, 0, 0xFFFF_FFFF) for value in function_values
    ]
    staged_buffers: list[Any] = []
    try:
        function_array = _client_ffi.new("uint32_t[]", raw_functions)
        if group_values is None:
            has_groups = 0
            group_array = _client_ffi.NULL
            group_count = 0
        else:
            has_groups = 1
            group_spans = _client_ffi.new(
                "kmipkit_byte_span_v1_t[]", len(group_values)
            ) if group_values else _client_ffi.NULL
            total_bytes = 0
            for index, group in enumerate(group_values):
                # The immutable snapshot was type-checked and UTF-8 validated above.
                encoded = str.encode(group, "utf-8")
                _validate_query_ttlv_text_length(len(encoded))
                if len(encoded) > sys.maxsize - total_bytes:
                    errors.raise_for_status(6)
                total_bytes += len(encoded)
                data = _client_ffi.new("uint8_t[]", encoded) if encoded else _client_ffi.NULL
                staged_buffers.append(data)
                group_spans[index].data = data
                group_spans[index].length = len(encoded)
            group_array = group_spans
            group_count = len(group_values)

        if limits is None:
            handle = _make_handle(
                "kmipkit_client_batch_item_t",
                "kmipkit_client_batch_item_query",
                (
                    function_array,
                    len(raw_functions),
                    has_groups,
                    group_array,
                    group_count,
                ),
            )
        else:
            handle = _make_handle(
                "kmipkit_client_batch_item_t",
                "kmipkit_client_batch_item_query_with_limits",
                (
                    _client_ffi.cast(
                        "kmipkit_codec_limits_t *",
                        int(ffi.cast("uintptr_t", limits._pointer())),
                    ),
                    function_array,
                    len(raw_functions),
                    has_groups,
                    group_array,
                    group_count,
                ),
            )
    finally:
        for buffer in staged_buffers:
            if buffer != _client_ffi.NULL:
                length = _client_ffi.sizeof(buffer)
                if length:
                    _client_ffi.buffer(buffer, length)[:] = b"\x00" * length
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_query(
    functions: list[int] | tuple[int, ...],
    object_groups: list[str] | tuple[str, ...] | None = None,
) -> ClientBatchItem:
    """Create a Query batch item after checking its payload against the default codec limits; execution rechecks the complete Request Message."""
    return _client_batch_item_query(functions, object_groups)


def client_batch_item_query_with_limits(
    functions: list[int] | tuple[int, ...],
    object_groups: list[str] | tuple[str, ...] | None,
    limits: ttlv.CodecLimits,
) -> ClientBatchItem:
    """Create Query within payload budgets; execution checks the complete Request Message."""
    return _client_batch_item_query(functions, object_groups, limits=limits)


def client_batch_item_register(request: RegisterRequest) -> ClientBatchItem:
    """Create a Register batch item after local request validation."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_register",
        (request,),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_revoke(request: RevokeRequest) -> ClientBatchItem:
    """Create a Revoke batch item and consume the owned request."""
    if not isinstance(request, RevokeRequest):
        errors.raise_for_status(4)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_revoke",
        (request,),
        consumed_indices=(0,),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def _native_structure(value: ttlv.Structure | None, *, required: bool = False) -> Any:
    if value is None:
        if required:
            errors.raise_for_status(4)
        return ffi.NULL
    if not isinstance(value, ttlv.Structure):
        errors.raise_for_status(4)
    return value._pointer()


def _client_native_structure(
    value: ttlv.Structure | None, *, required: bool = False
) -> Any:
    """Cast a public TTLV handle into the private client ABI's CFFI context."""
    if value is None:
        if required:
            errors.raise_for_status(4)
        return _client_ffi.NULL
    if not isinstance(value, ttlv.Structure):
        errors.raise_for_status(4)
    pointer = _native_structure(value)
    address = int(ffi.cast("uintptr_t", pointer))
    return _client_ffi.cast("kmipkit_ttlv_structure_t *", address)


def _native_value(value: ttlv.Value | None) -> Any:
    if value is None:
        return ffi.NULL
    if not isinstance(value, ttlv.Value):
        errors.raise_for_status(4)
    return value._pointer()


def _native_optional_span(
    value: bytes | bytearray | memoryview | None,
    staging_buffers: list[Any] | None = None,
) -> tuple[Any, Any]:
    """Create a borrowed span record while preserving omitted versus empty input."""
    if value is None:
        return ffi.NULL, None
    raw, data = _owned_bytes(value, _MAX_OPERATION_INPUT_BYTES)
    if staging_buffers is not None:
        staging_buffers.append(data)
    span = ffi.new(
        "kmipkit_byte_span_v1_t *",
        {"data": data if raw else ffi.NULL, "length": len(raw)},
    )
    return span, data


def _native_client_optional_span(
    value: bytes | bytearray | memoryview | None,
    staging_buffers: list[Any] | None = None,
) -> tuple[Any, Any]:
    """Create a byte span in the private client ABI's CFFI context."""
    if value is None:
        return _client_ffi.NULL, None
    raw, data = _owned_bytes(value, _MAX_OPERATION_INPUT_BYTES)
    if staging_buffers is not None:
        staging_buffers.append(data)
    native_data = (
        _client_ffi.cast("const uint8_t *", int(ffi.cast("uintptr_t", data)))
        if raw
        else _client_ffi.NULL
    )
    span = _client_ffi.new(
        "kmipkit_byte_span_v1_t *",
        {"data": native_data, "length": len(raw)},
    )
    return span, data


def _native_optional_indicator(value: bool | None) -> int:
    if value is None:
        return 0
    if not isinstance(value, bool):
        errors.raise_for_status(4)
    return 2 if value else 1


def _native_unique_identifier(
    value: UniqueIdentifier | None,
    staging_buffers: list[Any] | None = None,
) -> tuple[Any, Any]:
    if value is None:
        return ffi.NULL, None
    if not isinstance(value, UniqueIdentifier):
        errors.raise_for_status(4)
    record = ffi.new("kmipkit_unique_identifier_v1_t *")
    record.struct_size = ffi.sizeof("kmipkit_unique_identifier_v1_t")
    record.abi_version = 1
    record.kind = int(value.kind)
    record.reserved = 0
    record.enumeration_value = 0
    record.integer_value = 0
    owner = None
    if value.kind is UniqueIdentifierKind.TEXT_STRING:
        raw, owner = _owned_bytes(value.text, _MAX_UNIQUE_BATCH_ITEM_ID_BYTES)
        if staging_buffers is not None:
            staging_buffers.append(owner)
        data = owner if raw else ffi.NULL
        record.text_data = data
        record.text_length = len(raw)
    else:
        record.text_data = ffi.NULL
        record.text_length = 0
        if value.kind is UniqueIdentifierKind.ENUMERATION:
            record.enumeration_value = int(value.number)
        else:
            record.integer_value = int(value.number)
    return record, owner


def client_batch_item_create(
    object_type: int,
    attributes: ttlv.Structure,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a Create batch item with required Attributes and optional masks."""
    raw_object_type = _integer_range(object_type, 0, 0xFFFF_FFFF)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_create",
        (
            raw_object_type,
            _native_structure(attributes, required=True),
            _native_structure(protection_storage_masks),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_create_key_pair(
    common_attributes: ttlv.Structure | None = None,
    private_key_attributes: ttlv.Structure | None = None,
    public_key_attributes: ttlv.Structure | None = None,
    common_protection_storage_masks: ttlv.Structure | None = None,
    private_protection_storage_masks: ttlv.Structure | None = None,
    public_protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a Create Key Pair batch item preserving each optional group separately."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_create_key_pair",
        (
            _native_structure(common_attributes),
            _native_structure(private_key_attributes),
            _native_structure(public_key_attributes),
            _native_structure(common_protection_storage_masks),
            _native_structure(private_protection_storage_masks),
            _native_structure(public_protection_storage_masks),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_create_split_key(
    object_type: int,
    split_key_parts: int,
    split_key_threshold: int,
    split_key_method: int,
    attributes: ttlv.Structure,
    *,
    unique_identifier: UniqueIdentifier | None = None,
    prime_field_size: bytes | bytearray | memoryview | None = None,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a Create Split Key item with signed fields and explicit optional values."""
    raw_object_type = _integer_range(object_type, 0, 0xFFFF_FFFF)
    parts = _integer_range(split_key_parts, -(1 << 31), (1 << 31) - 1)
    threshold = _integer_range(split_key_threshold, -(1 << 31), (1 << 31) - 1)
    method = _integer_range(split_key_method, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(
            unique_identifier, staging_buffers
        )
        if prime_field_size is None:
            has_prime = 0
            prime_data = ffi.NULL
            prime_length = 0
        else:
            if not isinstance(prime_field_size, (bytes, bytearray, memoryview)):
                errors.raise_for_status(4)
            raw_prime, prime_owner = _owned_bytes(
                prime_field_size, _MAX_UNIQUE_BATCH_ITEM_ID_BYTES
            )
            staging_buffers.append(prime_owner)
            prime_data = prime_owner if raw_prime else ffi.NULL
            has_prime = 1
            prime_length = len(raw_prime)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_create_split_key",
            (
                raw_object_type,
                identifier,
                parts,
                threshold,
                method,
                has_prime,
                prime_data,
                prime_length,
                _native_structure(attributes, required=True),
                _native_structure(protection_storage_masks),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_certify(
    unique_identifier: UniqueIdentifier | None = None,
    *,
    certificate_request_type: int | None = None,
    certificate_request_value: bytes | bytearray | memoryview | None = None,
    attributes: ttlv.Structure | None = None,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create one typed Certify item, copying its opaque request value as sensitive input."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        if certificate_request_type is None:
            has_request_type = 0
            request_type = 0
        else:
            has_request_type = 1
            request_type = _integer_range(certificate_request_type, 0, 0xFFFF_FFFF)
        request_value, _ = _native_optional_span(
            certificate_request_value, staging_buffers
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_certify",
            (
                identifier,
                has_request_type,
                request_type,
                request_value,
                _native_structure(attributes),
                _native_structure(protection_storage_masks),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_recertify(
    unique_identifier: UniqueIdentifier | None = None,
    certificate_request_unique_identifier: UniqueIdentifier | None = None,
    *,
    certificate_request_type: int | None = None,
    certificate_request_value: bytes | bytearray | memoryview | None = None,
    offset: int | None = None,
    attributes: ttlv.Structure | None = None,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create one typed Re-certify item from KMIP 2.1 §6.1.45, Table 300.

    Certificate Request Value is copied into a temporary native staging buffer
    and that buffer is wiped after the synchronous ABI call. Python-owned input
    and runtime copies cannot be deterministically zeroized.
    """
    if certificate_request_type is None:
        has_request_type = 0
        request_type = 0
    else:
        has_request_type = 1
        request_type = _integer_range(certificate_request_type, 0, 0xFFFF_FFFF)
    if offset is None:
        has_offset = 0
        raw_offset = 0
    else:
        has_offset = 1
        raw_offset = _integer_range(offset, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        request_identifier, _ = _native_unique_identifier(
            certificate_request_unique_identifier, staging_buffers
        )
        request_value, _ = _native_optional_span(
            certificate_request_value, staging_buffers
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_recertify",
            (
                identifier,
                request_identifier,
                has_request_type,
                request_type,
                request_value,
                has_offset,
                raw_offset,
                _native_structure(attributes),
                _native_structure(protection_storage_masks),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_reprovision(
    certificate_request: ttlv.Structure | None = None,
    certificate: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.48 Re-Provision item from Table 313.

    Supply at most one of Certificate Request and Certificate. When neither is
    supplied, the server may create the replacement credential.
    """
    if certificate_request is not None and certificate is not None:
        errors.raise_for_status(4)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_reprovision",
        (
            _client_native_structure(certificate_request),
            _client_native_structure(certificate),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_rng_retrieve(data_length: int) -> ClientBatchItem:
    """Create §6.1.49 RNG Retrieve with the caller's signed KMIP Integer length."""
    length = _integer_range(data_length, -(1 << 31), (1 << 31) - 1)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_rng_retrieve",
        (length,),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_rng_seed(
    data: bytes | bytearray | memoryview,
) -> ClientBatchItem:
    """Create §6.1.50 RNG Seed from required seed bytes using a wiped staging buffer."""
    if data is None:
        errors.raise_for_status(4)
    with _staging_buffer_scope() as staging_buffers:
        seed_span, _ = _native_client_optional_span(data, staging_buffers)
        if seed_span == _client_ffi.NULL:
            errors.raise_for_status(4)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_rng_seed",
            (seed_span,),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_rekey(
    unique_identifier: UniqueIdentifier | None = None,
    *,
    offset: int | None = None,
    attributes: ttlv.Structure | None = None,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create one typed Re-key item from KMIP 2.1 §6.1.46, Table 305."""
    if offset is None:
        has_offset = 0
        raw_offset = 0
    else:
        has_offset = 1
        raw_offset = _integer_range(offset, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        client_identifier = _client_ffi.cast(
            "kmipkit_unique_identifier_v1_t *",
            int(ffi.cast("uintptr_t", identifier)),
        )
        attributes_pointer = _native_structure(attributes)
        masks_pointer = _native_structure(protection_storage_masks)
        client_attributes = _client_ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ffi.cast("uintptr_t", attributes_pointer)),
        )
        client_masks = _client_ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ffi.cast("uintptr_t", masks_pointer)),
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_rekey",
            (
                client_identifier,
                has_offset,
                raw_offset,
                client_attributes,
                client_masks,
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_rekey_key_pair(
    private_key_unique_identifier: UniqueIdentifier | None = None,
    *,
    offset: int | None = None,
    common_attributes: ttlv.Structure | None = None,
    private_key_attributes: ttlv.Structure | None = None,
    public_key_attributes: ttlv.Structure | None = None,
    common_protection_storage_masks: ttlv.Structure | None = None,
    private_protection_storage_masks: ttlv.Structure | None = None,
    public_protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a typed Re-key Key Pair item from KMIP 2.1 §6.1.47, Table 310."""
    if offset is None:
        has_offset, raw_offset = 0, 0
    else:
        has_offset, raw_offset = 1, _integer_range(offset, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(
            private_key_unique_identifier, staging_buffers
        )
        client_identifier = _client_ffi.cast(
            "kmipkit_unique_identifier_v1_t *",
            int(_client_ffi.cast("uintptr_t", identifier)),
        )
        structures = tuple(
            _native_structure(value)
            for value in (
                common_attributes,
                private_key_attributes,
                public_key_attributes,
                common_protection_storage_masks,
                private_protection_storage_masks,
                public_protection_storage_masks,
            )
        )
        args = (
            client_identifier,
            has_offset,
            raw_offset,
            *(
                _client_ffi.cast(
                    "kmipkit_ttlv_structure_t *",
                    int(_client_ffi.cast("uintptr_t", value)),
                )
                for value in structures
            ),
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_rekey_key_pair",
            args,
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_check(
    unique_identifier: UniqueIdentifier | None = None,
    *,
    usage_limits_count: int | None = None,
    cryptographic_usage_mask: int | None = None,
    lease_time: int | None = None,
) -> ClientBatchItem:
    """Create one typed Check item with optional KMIP 2.1 §6.1.7 fields."""
    if usage_limits_count is None:
        has_usage_limits_count = 0
        count = 0
    else:
        has_usage_limits_count = 1
        count = _integer_range(usage_limits_count, -(1 << 63), (1 << 63) - 1)
    if cryptographic_usage_mask is None:
        has_cryptographic_usage_mask = 0
        usage_mask = 0
    else:
        has_cryptographic_usage_mask = 1
        usage_mask = _integer_range(cryptographic_usage_mask, -(1 << 31), (1 << 31) - 1)
    if lease_time is None:
        has_lease_time = 0
        lease = 0
    else:
        has_lease_time = 1
        lease = _integer_range(lease_time, 0, (1 << 32) - 1)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_check",
            (
                identifier,
                has_usage_limits_count,
                count,
                has_cryptographic_usage_mask,
                usage_mask,
                has_lease_time,
                lease,
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_get_constraints() -> ClientBatchItem:
    """Create the empty Get Constraints request from KMIP 2.1 §6.1.22, Table 229."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_get_constraints",
        (),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_set_constraints(constraints: ttlv.Structure) -> ClientBatchItem:
    """Create a Set Constraints item from KMIP 2.1 §6.1.52, Table 325."""
    native_constraints = _client_native_structure(constraints, required=True)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_set_constraints",
        (native_constraints,),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_interop(
    interop_function: int,
    interop_identifier: str | bytes | bytearray | memoryview,
) -> ClientBatchItem:
    """Create a §6.1.26 Interop item with an open function and required identifier."""
    raw_function = _integer_range(interop_function, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        raw_identifier, identifier = _owned_bytes(
            interop_identifier, _MAX_OPERATION_INPUT_BYTES
        )
        staging_buffers.append(identifier)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_interop",
            (raw_function, identifier, len(raw_identifier)),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_pkcs11(
    function: ttlv.Structure,
    interface: ttlv.Structure | None = None,
    correlation_value: bytes | bytearray | memoryview | None = None,
    input_parameters: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.37 PKCS#11 item, preserving optional field presence."""
    native_function = _native_structure(function, required=True)
    native_interface = _client_ffi.cast(
        "kmipkit_ttlv_structure_t *",
        int(ffi.cast("uintptr_t", _native_structure(interface)))
        if interface is not None
        else 0,
    )
    native_input = _client_ffi.cast(
        "kmipkit_ttlv_structure_t *",
        int(ffi.cast("uintptr_t", _native_structure(input_parameters)))
        if input_parameters is not None
        else 0,
    )
    with _staging_buffer_scope() as staging_buffers:
        if correlation_value is None:
            correlation_span = _client_ffi.NULL
        else:
            raw_correlation, staged_correlation = _owned_bytes(
                correlation_value, _MAX_OPERATION_INPUT_BYTES
            )
            staging_buffers.append(staged_correlation)
            correlation_span = _client_ffi.new("kmipkit_byte_span_v1_t *")
            correlation_span.data = (
                _client_ffi.cast(
                    "const uint8_t *",
                    int(ffi.cast("uintptr_t", staged_correlation)),
                )
                if raw_correlation
                else _client_ffi.NULL
            )
            correlation_span.length = len(raw_correlation)
        function_pointer = _client_ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ffi.cast("uintptr_t", native_function)),
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_pkcs11",
            (function_pointer, native_interface, correlation_span, native_input),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_log(
    log_message: str | bytes | bytearray | memoryview,
) -> ClientBatchItem:
    """Create a §6.1.29 Log item with a required UTF-8 message."""
    with _staging_buffer_scope() as staging_buffers:
        raw_message, message = _owned_bytes(log_message, _MAX_OPERATION_INPUT_BYTES)
        staging_buffers.append(message)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_log",
            (message, len(raw_message)),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_delegated_login(
    rights: ttlv.Structure,
    *,
    lease_time_interval: int | None = None,
    lease_time_date_time: int | None = None,
    request_count: int | None = None,
    usage_limits: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.12 Delegated Login item from generic Rights TTLV."""
    if rights is None or not isinstance(rights, ttlv.Structure):
        errors.raise_for_status(4)
    if lease_time_interval is not None and lease_time_date_time is not None:
        errors.raise_for_status(4)
    if lease_time_interval is not None:
        lease_time_kind = 1
        lease_time = _integer_range(lease_time_interval, 0, 0xFFFF_FFFF)
    elif lease_time_date_time is not None:
        lease_time_kind = 2
        lease_time = _integer_range(lease_time_date_time, -(1 << 63), (1 << 63) - 1)
    else:
        lease_time_kind = 0
        lease_time = 0
    if request_count is None:
        has_request_count = 0
        count = 0
    else:
        has_request_count = 1
        count = _integer_range(request_count, -(1 << 31), (1 << 31) - 1)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_delegated_login",
        (
            _native_structure(rights, required=True),
            lease_time_kind,
            lease_time,
            has_request_count,
            count,
            _native_structure(usage_limits),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_login(
    *,
    lease_time_interval: int | None = None,
    lease_time_date_time: int | None = None,
    request_count: int | None = None,
    usage_limits: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.30 Login item from optional Table 253 fields."""
    if lease_time_interval is not None and lease_time_date_time is not None:
        errors.raise_for_status(4)
    if lease_time_interval is not None:
        lease_time_kind = 1
        lease_time = _integer_range(lease_time_interval, 0, 0xFFFF_FFFF)
    elif lease_time_date_time is not None:
        lease_time_kind = 2
        lease_time = _integer_range(lease_time_date_time, -(1 << 63), (1 << 63) - 1)
    else:
        lease_time_kind = 0
        lease_time = 0
    if request_count is None:
        has_request_count = 0
        count = 0
    else:
        has_request_count = 1
        count = _integer_range(request_count, -(1 << 31), (1 << 31) - 1)
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_login",
        (
            lease_time_kind,
            lease_time,
            has_request_count,
            count,
            _native_structure(usage_limits),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_derive_key(
    object_type: int,
    unique_identifiers: list[UniqueIdentifier] | tuple[UniqueIdentifier, ...],
    derivation_method: int,
    derivation_parameters: ttlv.Structure,
    attributes: ttlv.Structure,
) -> ClientBatchItem:
    """Create one typed KMIP 2.1 §6.1.14 Derive Key item."""
    raw_object_type = _integer_range(object_type, 0, 0xFFFF_FFFF)
    raw_method = _integer_range(derivation_method, 0, 0xFFFF_FFFF)
    identifiers = _bounded_snapshot(unique_identifiers, 100_000)
    if not identifiers or any(not isinstance(value, UniqueIdentifier) for value in identifiers):
        errors.raise_for_status(4)
    if not isinstance(derivation_parameters, ttlv.Structure) or not isinstance(
        attributes, ttlv.Structure
    ):
        errors.raise_for_status(4)

    with _staging_buffer_scope() as staging_buffers:
        native_identifiers = ffi.new("kmipkit_unique_identifier_v1_t[]", len(identifiers))
        total_identifier_bytes = 0
        for index, identifier in enumerate(identifiers):
            native_identifier, _owner = _native_unique_identifier(identifier, staging_buffers)
            total_identifier_bytes += native_identifier.text_length
            if total_identifier_bytes > _MAX_OPERATION_INPUT_BYTES:
                errors.raise_for_status(6)
            native_identifiers[index] = native_identifier[0]
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_derive_key",
            (
                raw_object_type,
                native_identifiers,
                len(identifiers),
                raw_method,
                _native_structure(derivation_parameters, required=True),
                _native_structure(attributes, required=True),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)

def client_batch_item_get_usage_allocation(
    unique_identifier: UniqueIdentifier | None,
    usage_limits_count: int,
) -> ClientBatchItem:
    """Create a Get Usage Allocation item with the exact signed 64-bit unit count."""
    count = _integer_range(usage_limits_count, -(1 << 63), (1 << 63) - 1)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_get_usage_allocation",
            (identifier, count),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_obtain_lease(
    unique_identifier: UniqueIdentifier | None,
) -> ClientBatchItem:
    """Create an Obtain Lease item with an optional §6.1.35 object identifier."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_obtain_lease",
            (identifier,),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_import(
    unique_identifier: UniqueIdentifier,
    object_type: int,
    attributes: ttlv.Structure,
    any_object: ttlv.Structure,
    *,
    replace_existing: bool | None = None,
    key_wrap_type: int | None = None,
) -> ClientBatchItem:
    """Create an Import item from KMIP 2.1 §6.1.25, Table 238.

    Attributes and Any Object are copied by the native ABI. Any Object must
    contain exactly one outer TTLV item. Key Wrap Type remains a raw value and
    is present only when the caller explicitly selects it.
    """
    if not isinstance(unique_identifier, UniqueIdentifier):
        errors.raise_for_status(4)
    raw_object_type = _integer_range(object_type, 0, 0xFFFF_FFFF)
    if replace_existing is not None and not isinstance(replace_existing, bool):
        errors.raise_for_status(4)
    has_replace_existing = int(replace_existing is not None)
    raw_replace_existing = int(replace_existing is True)
    has_key_wrap_type = int(key_wrap_type is not None)
    raw_key_wrap_type = 0 if key_wrap_type is None else _integer_range(
        key_wrap_type, 0, 0xFFFF_FFFF
    )
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_import",
            (
                identifier,
                raw_object_type,
                has_replace_existing,
                raw_replace_existing,
                has_key_wrap_type,
                raw_key_wrap_type,
                _native_structure(attributes, required=True),
                _native_structure(any_object, required=True),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_join_split_key(
    object_type: int,
    references: list[SplitKeyReference] | tuple[SplitKeyReference, ...],
    *,
    secret_data_type: int | None = None,
    attributes: ttlv.Structure | None = None,
    protection_storage_masks: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create KMIP 2.1 §6.1.27 Join Split Key with paired local thresholds."""
    raw_object_type = _integer_range(object_type, 0, 0xFFFF_FFFF)
    snapshot = _bounded_snapshot(references, 100_000)
    if not snapshot or any(not isinstance(reference, SplitKeyReference) for reference in snapshot):
        errors.raise_for_status(4)
    if len(snapshot) < max(reference.split_key_threshold for reference in snapshot):
        errors.raise_for_status(4)
    raw_secret_type = 0 if secret_data_type is None else _integer_range(
        secret_data_type, 0, 0xFFFF_FFFF
    )
    if attributes is not None and not isinstance(attributes, ttlv.Structure):
        errors.raise_for_status(4)
    if protection_storage_masks is not None and not isinstance(
        protection_storage_masks, ttlv.Structure
    ):
        errors.raise_for_status(4)
    with _NATIVE_CALL_LOCK:
        native_references = ffi.new(
            "kmipkit_split_key_reference_t *[]",
            [reference._pointer() for reference in snapshot],
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_join_split_key",
            (
                raw_object_type,
                native_references,
                len(snapshot),
                int(secret_data_type is not None),
                raw_secret_type,
                _native_structure(attributes),
                _native_structure(protection_storage_masks),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_get(
    unique_identifier: UniqueIdentifier | None = None,
    key_format_type: int | None = None,
    key_wrap_type: int | None = None,
    key_compression_type: int | None = None,
    key_wrapping_specification: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.19 Get item with optional Table 220 fields."""
    has_format = int(key_format_type is not None)
    raw_format = 0 if key_format_type is None else _integer_range(
        key_format_type, 0, 0xFFFF_FFFF
    )
    has_wrap = int(key_wrap_type is not None)
    raw_wrap = 0 if key_wrap_type is None else _integer_range(key_wrap_type, 0, 0xFFFF_FFFF)
    has_compression = int(key_compression_type is not None)
    raw_compression = 0 if key_compression_type is None else _integer_range(
        key_compression_type, 0, 0xFFFF_FFFF
    )
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_get",
            (
                identifier,
                has_format,
                raw_format,
                has_wrap,
                raw_wrap,
                has_compression,
                raw_compression,
                _native_structure(key_wrapping_specification),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_locate(
    attributes: ttlv.Structure,
    *,
    maximum_items: int | None = None,
    offset_items: int | None = None,
    storage_status_mask: int | None = None,
    object_group_member: int | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.28 Locate item with required Attributes."""
    has_maximum = int(maximum_items is not None)
    raw_maximum = 0 if maximum_items is None else _integer_range(
        maximum_items, -(1 << 31), (1 << 31) - 1
    )
    has_offset = int(offset_items is not None)
    raw_offset = 0 if offset_items is None else _integer_range(
        offset_items, -(1 << 31), (1 << 31) - 1
    )
    has_storage_mask = int(storage_status_mask is not None)
    raw_storage_mask = 0 if storage_status_mask is None else _integer_range(
        storage_status_mask, 0, 0xFFFF_FFFF
    )
    has_group_member = int(object_group_member is not None)
    raw_group_member = 0 if object_group_member is None else _integer_range(
        object_group_member, 0, 0xFFFF_FFFF
    )
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_locate",
        (
            has_maximum,
            raw_maximum,
            has_offset,
            raw_offset,
            has_storage_mask,
            raw_storage_mask,
            has_group_member,
            raw_group_member,
            _native_structure(attributes, required=True),
        ),
    )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_export(
    unique_identifier: UniqueIdentifier | None = None,
    key_format_type: int | None = None,
    key_wrap_type: int | None = None,
    key_compression_type: int | None = None,
    key_wrapping_specification: ttlv.Structure | None = None,
) -> ClientBatchItem:
    """Create a KMIP 2.1 §6.1.18 Export item with optional Table 217 selectors."""
    if key_format_type is None:
        has_key_format_type = 0
        raw_key_format_type = 0
    else:
        has_key_format_type = 1
        raw_key_format_type = _integer_range(key_format_type, 0, 0xFFFF_FFFF)
    if key_wrap_type is None:
        has_key_wrap_type = 0
        raw_key_wrap_type = 0
    else:
        has_key_wrap_type = 1
        raw_key_wrap_type = _integer_range(key_wrap_type, 0, 0xFFFF_FFFF)
    if key_compression_type is None:
        has_key_compression_type = 0
        raw_key_compression_type = 0
    else:
        has_key_compression_type = 1
        raw_key_compression_type = _integer_range(key_compression_type, 0, 0xFFFF_FFFF)
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_export",
            (
                identifier,
                has_key_format_type,
                raw_key_format_type,
                has_key_wrap_type,
                raw_key_wrap_type,
                has_key_compression_type,
                raw_key_compression_type,
                _native_structure(key_wrapping_specification),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_encrypt(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    iv_counter_nonce: bytes | bytearray | memoryview | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
    authenticated_encryption_additional_data: bytes | bytearray | memoryview | None = None,
) -> ClientBatchItem:
    """Create an Encrypt request item without performing local cryptography."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        iv_counter_nonce_span, _ = _native_optional_span(iv_counter_nonce, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        additional_data, _ = _native_optional_span(
            authenticated_encryption_additional_data, staging_buffers
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_encrypt",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                iv_counter_nonce_span,
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
                additional_data,
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_decrypt(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    iv_counter_nonce: bytes | bytearray | memoryview | None = None,
    authenticated_encryption_tag: bytes | bytearray | memoryview | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
    authenticated_encryption_additional_data: bytes | bytearray | memoryview | None = None,
) -> ClientBatchItem:
    """Create a Decrypt request item without performing local cryptography."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        iv_counter_nonce_span, _ = _native_optional_span(iv_counter_nonce, staging_buffers)
        tag, _ = _native_optional_span(authenticated_encryption_tag, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        additional_data, _ = _native_optional_span(
            authenticated_encryption_additional_data, staging_buffers
        )
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_decrypt",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                iv_counter_nonce_span,
                tag,
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
                additional_data,
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_hash(
    parameters: ttlv.Structure,
    data: ttlv.Value | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> ClientBatchItem:
    """Create a Hash request item without performing local cryptography."""
    with _staging_buffer_scope() as staging_buffers:
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_hash",
            (
                _native_structure(parameters, required=True),
                _native_value(data),
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_mac(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> ClientBatchItem:
    """Create a MAC request item while preserving optional wire fields."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_mac",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_mac_verify(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    mac_data: bytes | bytearray | memoryview | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> ClientBatchItem:
    """Create a MAC Verify request item and copy opaque MAC bytes at the ABI boundary."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        mac_span, _ = _native_optional_span(mac_data, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_mac_verify",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                mac_span,
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_sign(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    digested_data: bytes | bytearray | memoryview | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> ClientBatchItem:
    """Create a Sign request item; KMIPKit never computes a digest or signature locally."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        digest_span, _ = _native_optional_span(digested_data, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_sign",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                digest_span,
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_signature_verify(
    unique_identifier: UniqueIdentifier | None = None,
    parameters: ttlv.Structure | None = None,
    data: ttlv.Value | None = None,
    digested_data: bytes | bytearray | memoryview | None = None,
    signature_data: bytes | bytearray | memoryview | None = None,
    *,
    correlation_value: bytes | bytearray | memoryview | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> ClientBatchItem:
    """Create a Signature Verify request item using opaque signature inputs."""
    with _staging_buffer_scope() as staging_buffers:
        identifier, _ = _native_unique_identifier(unique_identifier, staging_buffers)
        digest_span, _ = _native_optional_span(digested_data, staging_buffers)
        signature_span, _ = _native_optional_span(signature_data, staging_buffers)
        correlation, _ = _native_optional_span(correlation_value, staging_buffers)
        handle = _make_handle(
            "kmipkit_client_batch_item_t",
            "kmipkit_client_batch_item_signature_verify",
            (
                identifier,
                _native_structure(parameters),
                _native_value(data),
                digest_span,
                signature_span,
                correlation,
                _native_optional_indicator(init_indicator),
                _native_optional_indicator(final_indicator),
            ),
        )
    return _wrap_owned_handle(ClientBatchItem, handle)


def client_batch_item_extension_count(item: ClientBatchItem) -> int:
    """Return the number of attached request extensions."""
    return _scalar(
        "uint64_t",
        lib.kmipkit_client_batch_item_extension_count,
        item._pointer(),
    )


def client_batch_item_extension_identity_at(
    item: ClientBatchItem, index: int
) -> ExtensionIdentity:
    """Return an extension identity at its caller-selected position."""
    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
        errors.raise_for_status(4)
    handle = _make_handle(
        "kmipkit_extension_identity_t",
        "kmipkit_client_batch_item_extension_identity_at",
        (item._pointer(), index),
    )
    return _populate_identity_fields(
        _wrap_owned_handle(ExtensionIdentity, handle, None, None, None)
    )


def client_batch_item_extension_criticality_indicator_at(
    item: ClientBatchItem, index: int
) -> bool:
    """Return the explicit criticality flag at its caller-selected position."""
    if not isinstance(index, int) or isinstance(index, bool) or index < 0:
        errors.raise_for_status(4)
    output = ffi.new("uint8_t *")
    errors.raise_for_status(
        _invoke(
            lib.kmipkit_client_batch_item_extension_criticality_indicator_at,
            item._pointer(),
            index,
            output,
        )
    )
    if output[0] not in (0, 1):
        errors.raise_for_status(4)
    return bool(output[0])


__all__ = [
    "ClientBatchItem",
    "ClientConfiguration",
    "ClientExtensionRegistry",
    "ClientRequestMessageExtension",
    "Compatibility",
    "Discriminator",
    "ExtensionChildRule",
    "ExtensionDefinition",
    "ExtensionIdentity",
    "ExtensionInformation",
    "ExtensionOrderConstraint",
    "ExtensionRecognition",
    "ExtensionRegistryLimits",
    "ExtensionSchema",
    "OpaqueObjectContext",
    "RegisteredExtensionValue",
    "RegisterRequest",
    "RevocationReason",
    "RevokeRequest",
    "SplitKeyReference",
    "UniqueIdentifier",
    "UniqueIdentifierKind",
    "TtlvPath",
    "ValidatedExtensionValue",
    "client_batch_item_discover_versions",
    "client_batch_item_ping",
    "client_batch_item_query",
    "client_batch_item_query_with_limits",
    "client_batch_item_encrypt",
    "client_batch_item_decrypt",
    "client_batch_item_hash",
    "client_batch_item_mac",
    "client_batch_item_mac_verify",
    "client_batch_item_sign",
    "client_batch_item_signature_verify",
    "client_batch_item_register",
    "client_batch_item_revoke",
    "client_batch_item_create",
    "client_batch_item_create_key_pair",
    "client_batch_item_create_split_key",
    "client_batch_item_certify",
    "client_batch_item_recertify",
    "client_batch_item_reprovision",
    "client_batch_item_rng_retrieve",
    "client_batch_item_rng_seed",
    "client_batch_item_rekey",
    "client_batch_item_rekey_key_pair",
    "client_batch_item_check",
    "client_batch_item_get_constraints",
    "client_batch_item_set_constraints",
    "client_batch_item_interop",
    "client_batch_item_pkcs11",
    "client_batch_item_log",
    "client_batch_item_delegated_login",
    "client_batch_item_login",
    "client_batch_item_logout",
    "client_batch_item_get_usage_allocation",
    "client_batch_item_obtain_lease",
    "client_batch_item_import",
    "client_batch_item_join_split_key",
    "client_batch_item_get",
    "client_batch_item_locate",
    "client_batch_item_export",
    "client_batch_item_extension_count",
    "client_batch_item_extension_criticality_indicator_at",
    "client_batch_item_extension_identity_at",
    "with_unique_batch_item_id",
    "client_configuration_extension_registry",
    "create_client_configuration",
    "create_client_extension_registry",
    "create_client_request_message_extension",
    "create_compatibility",
    "create_discriminator",
    "create_extension_definition",
    "create_extension_identity",
    "create_extension_information",
    "create_extension_order_constraint",
    "create_extension_registry_limits",
    "create_ttlv_path",
    "default_extension_registry_limits",
    "definition_at",
    "definition_count",
    "definition_for_identity",
    "extension_definition_identity",
    "extension_definition_information",
    "extension_identity_name",
    "extension_identity_vendor_identifier",
    "extension_identity_version",
    "extension_information_to_ttlv",
    "extension_recognition_generic_value",
    "inspect_extension",
    "is_recognized",
    "optional_child_rule",
    "repeated_child_rule",
    "required_child_rule",
    "scalar_schema",
    "structure_schema",
    "validate_extension_value",
    "validated_extension_identity",
    "validated_extension_value_generic_value",
    "validated_value",
    "value_at",
    "with_allowed_bit_mask",
    "with_allowed_enumeration",
    "with_attribute",
    "with_child_tag",
    "with_description",
    "with_enumeration",
    "with_extension",
    "with_extension_information",
    "with_maximum_length",
    "with_minimum_length",
    "with_parent_structure_tag",
    "with_required_bit_mask",
    "with_signed_range",
    "with_tag",
    "with_type",
    "with_unsigned_range",
]

synchronize_public_functions(globals(), __all__)
