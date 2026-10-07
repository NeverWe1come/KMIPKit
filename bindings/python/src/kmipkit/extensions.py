"""Manifest-backed Python facade for the vendor extension registry C ABI.

KMIPKit owns and zeroizes its Rust buffers. Python necessarily creates and
retains its own copies of input strings, byte strings, and values returned by
TTLV accessors; Python's runtime does not provide deterministic zeroization of
those external copies.

Opaque native identity handles expose bounded field getters. Inspection keeps
generic TTLV intact; Python never infers identity by rescanning payload content.
"""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any, overload

from . import errors, ttlv
from ._ffi import ffi, lib
from ._handles import NativeHandle, _invoke, _invoke_consuming, _owned_bytes, _scalar


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


class ExtensionIdentity(NativeHandle):
    _release_name = "kmipkit_extension_identity_release"

    def __init__(
        self,
        handle: Any,
        vendor_identifier: str | None,
        name: str | None,
        version: str | None,
    ) -> None:
        super().__init__(handle)
        self.vendor_identifier = vendor_identifier
        self.name = name
        self.version = version


class Compatibility(NativeHandle):
    _release_name = "kmipkit_extension_compatibility_release"


class TtlvPath(NativeHandle):
    _release_name = "kmipkit_ttlv_path_release"

    def __init__(self, handle: Any, tags: tuple[int, ...]) -> None:
        super().__init__(handle)
        self.tags = tags


class Discriminator(NativeHandle):
    _release_name = "kmipkit_extension_discriminator_release"

    def __init__(
        self,
        handle: Any,
        path: TtlvPath,
        scalar_type: ttlv.ItemType,
        scalar_value: object,
    ) -> None:
        super().__init__(handle)
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
    ) -> None:
        super().__init__(handle)
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
        return ExtensionInformation(handle, **values)


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
    ) -> None:
        super().__init__(
            handle,
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
    ) -> None:
        ordered = tuple(sorted(definitions, key=_identity_key))
        super().__init__(handle, owner=ordered)
        self._definitions = ordered
        self.limits = limits


class ClientConfiguration(NativeHandle):
    _release_name = "kmipkit_client_configuration_release"

    def __init__(
        self,
        handle: Any,
        definitions: tuple[ExtensionDefinition, ...],
        limits: ExtensionRegistryLimits,
    ) -> None:
        super().__init__(handle, owner=definitions)
        self._definitions = definitions
        self.limits = limits


class ValidatedExtensionValue(NativeHandle):
    _release_name = "kmipkit_validated_extension_value_release"

    def __init__(
        self,
        handle: Any,
        identity: ExtensionIdentity | None = None,
        owner: object | None = None,
    ) -> None:
        super().__init__(handle, owner=(identity, owner))
        self.identity = identity


class RegisteredExtensionValue(NativeHandle):
    _release_name = "kmipkit_registered_extension_value_release"

    def __init__(self, handle: Any, identity: ExtensionIdentity) -> None:
        super().__init__(handle, owner=identity)
        self.identity = identity


class ClientRequestMessageExtension(NativeHandle):
    _release_name = "kmipkit_client_request_message_extension_release"


class ClientBatchItem(NativeHandle):
    _release_name = "kmipkit_client_batch_item_release"


class ExtensionRecognition(NativeHandle):
    _release_name = "kmipkit_extension_recognition_release"

    def __init__(
        self,
        handle: Any,
        registry: ClientExtensionRegistry,
        vendor_identifier: str,
        value: ttlv.Structure,
    ) -> None:
        super().__init__(handle, owner=(registry, value))
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
    output = ffi.new(f"{c_type} **")
    call_arguments = list(arguments)
    if out_index is None:
        call_arguments.append(output)
    else:
        call_arguments.insert(out_index, output)
    function = getattr(lib, function_name)
    status = (
        _invoke_consuming(function, tuple(call_arguments), consumed_indices)
        if consumed_indices
        else _invoke(function, *call_arguments)
    )
    errors.raise_for_status(status)
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    return output[0]


def _make_bytes(value: str) -> tuple[bytes, Any]:
    if not isinstance(value, str):
        errors.raise_for_status(4)
    return _owned_bytes(value)


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
    return ExtensionIdentity(handle, vendor_identifier, name, version)


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
    return Compatibility(handle)


def create_ttlv_path(first_tag: ttlv.Tag) -> TtlvPath:
    value = ttlv.tag_value(first_tag)
    handle = _make_handle("kmipkit_ttlv_path_t", "kmipkit_ttlv_path_create", (value,))
    return TtlvPath(handle, (value,))


def with_child_tag(path: TtlvPath, tag: ttlv.Tag) -> TtlvPath:
    value = ttlv.tag_value(tag)
    handle = _make_handle(
        "kmipkit_ttlv_path_t",
        "kmipkit_ttlv_path_with_child_tag",
        (path, value),
        consumed_indices=(0,),
    )
    return TtlvPath(handle, (*path.tags, value))


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
    return Discriminator(handle, path, scalar_type, scalar_value_copy)


def scalar_schema(item_type: ttlv.ItemType) -> ExtensionSchema:
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        "kmipkit_extension_schema_scalar",
        (int(item_type),),
    )
    return ExtensionSchema(handle)


def structure_schema(
    children: list[ExtensionChildRule] | tuple[ExtensionChildRule, ...],
    order_constraints: list[ExtensionOrderConstraint]
    | tuple[ExtensionOrderConstraint, ...],
    preserve_undeclared_children: bool,
) -> ExtensionSchema:
    if not isinstance(preserve_undeclared_children, bool):
        errors.raise_for_status(4)
    child_array = (
        ffi.NULL
        if not children
        else ffi.new(
            "kmipkit_extension_child_rule_t *[]",
            [child._pointer() for child in children],
        )
    )
    order_array = (
        ffi.NULL
        if not order_constraints
        else ffi.new(
            "kmipkit_extension_order_constraint_t *[]",
            [constraint._pointer() for constraint in order_constraints],
        )
    )
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        "kmipkit_extension_schema_structure",
        (
            child_array,
            len(children),
            order_array,
            len(order_constraints),
            int(preserve_undeclared_children),
        ),
    )
    return ExtensionSchema(handle, owner=(tuple(children), tuple(order_constraints)))


def _child_rule(
    tag: ttlv.Tag, schema: ExtensionSchema, mode: str
) -> ExtensionChildRule:
    tag_value = ttlv.tag_value(tag)
    handle = _make_handle(
        "kmipkit_extension_child_rule_t",
        f"kmipkit_extension_child_rule_{mode}",
        (tag_value, schema._pointer()),
    )
    return ExtensionChildRule(handle, owner=(tag, schema))


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
    return ExtensionOrderConstraint(handle, owner=(before_tag, after_tag))


def _schema_transform(
    schema: ExtensionSchema, symbol: str, *values: int
) -> ExtensionSchema:
    handle = _make_handle(
        "kmipkit_extension_schema_t",
        symbol,
        (schema, *values),
        consumed_indices=(0,),
    )
    return ExtensionSchema(handle)


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
    return ExtensionDefinition(
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
        return ValidatedExtensionValue(handle, target.identity, owner=target)
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
    return RegisteredExtensionValue(handle, identity)


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
    status = _invoke(lib.kmipkit_extension_registry_limits_validate, *values)
    errors.raise_for_status(status)
    return ExtensionRegistryLimits(**dict(zip(_LIMIT_FIELDS, values, strict=True)))


def default_extension_registry_limits() -> ExtensionRegistryLimits:
    outputs = [ffi.new("uint64_t *") for _ in _LIMIT_FIELDS]
    errors.raise_for_status(
        _invoke(lib.kmipkit_extension_registry_limits_default_values, *outputs)
    )
    return ExtensionRegistryLimits(
        **dict(zip(_LIMIT_FIELDS, (int(value[0]) for value in outputs), strict=True))
    )


def create_client_extension_registry(
    definitions: list[ExtensionDefinition] | tuple[ExtensionDefinition, ...],
    limits: ExtensionRegistryLimits,
) -> ClientExtensionRegistry:
    source = tuple(sorted(definitions, key=_identity_key))
    definition_array = (
        ffi.NULL
        if not source
        else ffi.new(
            "kmipkit_extension_definition_t *[]",
            [definition._pointer() for definition in source],
        )
    )
    limit_values = tuple(getattr(limits, field) for field in _LIMIT_FIELDS)
    handle = _make_handle(
        "kmipkit_client_extension_registry_t",
        "kmipkit_client_extension_registry_create",
        (definition_array, len(source), *limit_values),
    )
    return ClientExtensionRegistry(handle, source, limits)


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
    return ClientConfiguration(handle, definitions, limits)


def client_configuration_extension_registry(
    configuration: ClientConfiguration,
) -> ClientExtensionRegistry:
    handle = _make_handle(
        "kmipkit_client_extension_registry_t",
        "kmipkit_client_configuration_extension_registry",
        (configuration._pointer(),),
    )
    return ClientExtensionRegistry(
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
    return ExtensionDefinition(
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
        return ExtensionDefinition(output[0], identity)
    return ExtensionDefinition(
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
    return ExtensionIdentity(
        handle, identity.vendor_identifier, identity.name, identity.version
    )


def _identity_text(identity: ExtensionIdentity, symbol: str) -> str:
    value = ttlv.Value(
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
        return ExtensionInformation(output[0], None)
    return ExtensionInformation(
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
    return ExtensionInformation(handle, extension_name)


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
    return ttlv.Structure(handle, owner=information)


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
        source = ExtensionDefinition(
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
    return ExtensionDefinition(
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
    return ExtensionRecognition(handle, registry, vendor_identifier, value)


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
    return ValidatedExtensionValue(output[0], owner=recognition)


def extension_recognition_generic_value(
    recognition: ExtensionRecognition,
) -> ttlv.TtlvStructureView:
    handle = _make_handle(
        "kmipkit_ttlv_structure_view_t",
        "kmipkit_extension_recognition_generic_value",
        (recognition._pointer(),),
    )
    return ttlv.TtlvStructureView(handle, owner=recognition)


def validated_extension_identity(value: ValidatedExtensionValue) -> ExtensionIdentity:
    handle = _make_handle(
        "kmipkit_extension_identity_t",
        "kmipkit_validated_extension_value_identity",
        (value._pointer(),),
    )
    return _populate_identity_fields(ExtensionIdentity(handle, None, None, None))


def validated_extension_value_generic_value(
    value: ValidatedExtensionValue,
) -> ttlv.TtlvStructureView:
    handle = _make_handle(
        "kmipkit_ttlv_structure_view_t",
        "kmipkit_validated_extension_value_generic_value",
        (value._pointer(),),
    )
    return ttlv.TtlvStructureView(handle, owner=value)


def value_at(value: ValidatedExtensionValue, path: TtlvPath) -> ttlv.TtlvValueView:
    handle = _make_handle(
        "kmipkit_ttlv_value_view_t",
        "kmipkit_validated_extension_value_value_at",
        (value._pointer(), path._pointer()),
    )
    return ttlv.TtlvValueView(handle, owner=(value, path))


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
    return ClientRequestMessageExtension(handle)


def with_extension(
    item: ClientBatchItem, extension: ClientRequestMessageExtension
) -> ClientBatchItem:
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_with_extension",
        (item, extension),
        consumed_indices=(0, 1),
    )
    return ClientBatchItem(handle)


def client_batch_item_discover_versions() -> ClientBatchItem:
    """Create an empty Discover Versions request batch item."""
    handle = _make_handle(
        "kmipkit_client_batch_item_t",
        "kmipkit_client_batch_item_discover_versions",
        (),
    )
    return ClientBatchItem(handle)


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
    return _populate_identity_fields(ExtensionIdentity(handle, None, None, None))


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
    "RegisteredExtensionValue",
    "TtlvPath",
    "ValidatedExtensionValue",
    "client_batch_item_discover_versions",
    "client_batch_item_extension_count",
    "client_batch_item_extension_criticality_indicator_at",
    "client_batch_item_extension_identity_at",
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
