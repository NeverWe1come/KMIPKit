"""Generate Rust TTLV type constraints for catalogued KMIP attributes."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from .validate import MAX_BYTES, CatalogValidationError, load_validated_catalog
else:
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from validate import MAX_BYTES, CatalogValidationError, load_validated_catalog


CATALOG_PATH = "verification/conformance/catalog/kmip-2.1.json"
VERSIONED_TAGS_PATH = "verification/conformance/catalog/kmip-versioned-tags.json"
OUTPUT_PATH = "crates/kmipkit-protocol/src/attribute_types_generated.rs"
TYPED_OUTPUT_PATH = "crates/kmipkit-protocol/src/typed_attributes_generated.rs"
TYPED_FFI_OUTPUT_PATH = "crates/kmipkit-ffi/src/extension_registry/typed_attributes_catalog_generated.rs"
TYPED_C_HEADER_OUTPUT_PATH = "bindings/c/include/kmipkit_standard_attributes_generated.h"
TYPED_JAVA_OUTPUT_PATH = "bindings/java/src/main/java/org/kmipkit/ttlv/StandardAttribute.java"
TYPED_PYTHON_OUTPUT_PATH = "bindings/python/src/kmipkit/_generated/standard_attribute.py"
CLIENT_ATTRIBUTE_TAG_OUTPUT_PATH = "crates/kmipkit-client/src/attribute_tags_generated.rs"
_VENDOR_ATTRIBUTE_ID = "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE"
_CERTIFICATE_ATTRIBUTES_ID = "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES"
_VENDOR_ATTRIBUTE_TAG = 0x0042_0008
_WIRE_VALUE = re.compile(r"(?:0x)?[0-9A-Fa-f]{6}\Z")
_TYPE_NAMES = {
    "Big Integer": "BigInteger",
    "Big-Integer": "BigInteger",
    "Boolean": "Boolean",
    "Byte String": "ByteString",
    "Date Time": "DateTime",
    "Date-Time": "DateTime",
    "Date Time Extended": "DateTimeExtended",
    "Enumeration": "Enumeration",
    "Integer": "Integer",
    "Interval": "Interval",
    "Long Integer": "LongInteger",
    "RNG Parameters": "Structure",
    "Structure": "Structure",
    "Text String": "TextString",
}
_ENCODING_NAMES = {
    value_kind: f"{value_kind}Encoding"
    for value_kind in (
        "BigInteger",
        "Boolean",
        "ByteString",
        "DateTime",
        "DateTimeExtended",
        "Enumeration",
        "Integer",
        "Interval",
        "LongInteger",
        "Structure",
        "TextString",
        "UniqueIdentifier",
    )
}


@dataclass(frozen=True, slots=True)
class TypedAttributeDefinition:
    """One catalogued direct attribute and its typed Rust marker metadata."""

    element_id: str
    source_name: str
    marker_name: str
    tag: int
    value_kind: str
    source_multiplicity: str | None
    min_version: str = "2.0"
    max_version: str | None = None


def _versioned_tag_attributes(versioned_tags: dict[str, Any] | None) -> list[dict[str, Any]]:
    if versioned_tags is None:
        return []
    entries: list[dict[str, Any]] = []
    for entry in versioned_tags.get("tagEnumeration", {}).get("assignments", []):
        if isinstance(entry, dict) and entry.get("is_object_attribute") is True:
            entries.append(entry)
    for entry in versioned_tags.get("v3_0_promoted_attributes", []):
        if isinstance(entry, dict):
            entries.append(entry)
    return entries


def _attribute_encodings(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> list[tuple[int, list[str]]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    tags: dict[str, tuple[int, str, bool]] = {}
    seen_tag_values: set[int] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "tag":
            continue
        element_id = element.get("element_id")
        wire_value = element.get("wire_value")
        if not isinstance(element_id, str) or not _WIRE_VALUE.fullmatch(str(wire_value)):
            raise ValueError("catalog tag has an invalid identity or wire value")
        numeric_value = int(str(wire_value).removeprefix("0x").removeprefix("0X"), 16)
        if element_id in tags or numeric_value in seen_tag_values:
            raise ValueError("catalog contains duplicate tag identities or values")
        allocation = element.get("allocation")
        if allocation not in {"assigned", "reserved"}:
            raise ValueError("catalog tag has an unsupported allocation")
        seen_tag_values.add(numeric_value)
        tags[element_id] = (numeric_value, str(element.get("name", "")), allocation == "assigned")

    attributes: list[tuple[int, list[str]]] = []
    seen_attribute_tags: set[int] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "attribute":
            continue
        source_encoding = element.get("source_encoding")
        if source_encoding is None:
            continue
        if not isinstance(source_encoding, str) or not source_encoding.strip():
            raise ValueError("catalog attribute has an invalid source encoding")
        parents = element.get("parent_element_ids")
        if element.get("element_id") == _VENDOR_ATTRIBUTE_ID:
            if source_encoding != "Structure" or parents != []:
                raise ValueError("Vendor Attribute must retain its separate Table 150 structure")
            continue
        if (
            not isinstance(parents, list)
            or len(parents) != 1
            or not isinstance(parents[0], str)
            or parents[0] not in tags
        ):
            raise ValueError("catalog attribute must have exactly one tag parent")
        raw_tag, _, assigned = tags[parents[0]]
        if not assigned:
            raise ValueError("attribute parent tag is not assigned")
        if raw_tag in seen_attribute_tags:
            raise ValueError("catalog attributes share a duplicate TTLV tag")
        seen_attribute_tags.add(raw_tag)

        types = _item_types_for_encoding(source_encoding)
        attributes.append((raw_tag, types))

    for entry in _versioned_tag_attributes(versioned_tags):
        wire_value = entry.get("wire_value")
        source_encoding = entry.get("source_encoding")
        if not _WIRE_VALUE.fullmatch(str(wire_value)):
            raise ValueError("versioned attribute tag has an invalid wire value")
        if not isinstance(source_encoding, str) or not source_encoding.strip():
            raise ValueError("versioned attribute has an invalid source encoding")
        raw_tag = int(str(wire_value).removeprefix("0x").removeprefix("0X"), 16)
        if raw_tag in seen_attribute_tags:
            raise ValueError("versioned attributes share a duplicate TTLV tag")
        seen_attribute_tags.add(raw_tag)
        types = _item_types_for_encoding(source_encoding)
        attributes.append((raw_tag, types))

    if not attributes:
        raise ValueError("catalog has no typed attribute encodings")
    return sorted(attributes)


def _rust_number(value: int) -> str:
    return f"0x{value >> 16:02X}_{(value >> 8) & 0xFF:02X}_{value & 0xFF:02X}"


def _item_types_for_encoding(source_encoding: str) -> list[str]:
    if source_encoding.strip() == "Unique Identifier":
        return ["TextString", "Enumeration", "Integer"]
    names = [part.strip() for part in re.split(r",|\s+or\s+", source_encoding) if part.strip()]
    if not names or any(name not in _TYPE_NAMES for name in names):
        raise ValueError("catalog attribute has an unsupported source encoding")
    return list(dict.fromkeys(_TYPE_NAMES[name] for name in names))


def _marker_from_words(words: list[str]) -> str:
    if not words:
        raise ValueError("catalog attribute has an invalid element identity")
    marker = "".join(
        word if word.isdigit() else word[0].upper() + word[1:].lower()
        for word in words
    )
    if not marker[0].isalpha():
        marker = "Attribute" + marker
    return marker


def _marker_identifier(element_id: str) -> str:
    prefix = "KMIPKIT-ELEM-ATTRIBUTE-"
    if not element_id.startswith(prefix):
        raise ValueError("catalog attribute has an invalid element identity")
    words = re.findall(r"[A-Za-z]+|[0-9]+", element_id[len(prefix) :])
    return _marker_from_words(words)


def _source_multiplicity(element: dict[str, Any]) -> str | None:
    value = element.get("source_multiple_instances_permitted")
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise ValueError("catalog attribute has invalid multiplicity metadata")
    return value


def collect_typed_attributes(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> list[TypedAttributeDefinition]:
    """Collect direct, typed object attributes from the validated catalog."""
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    if versioned_tags and "v2_1_tags" in versioned_tags:
        v21_tags = {
            int(e["wire_value"].removeprefix("0x").removeprefix("0X"), 16)
            for e in versioned_tags["v2_1_tags"]
            if e.get("is_object_attribute")
        }
    else:
        v21_tags = {
            0x42_01_6A,
            0x42_01_6B,
            0x42_01_6C,
            0x42_01_6D,
            0x42_01_6E,
            0x42_01_6F,
            0x42_01_72,
        }
    deprecated_in_v30 = {0x42_00_4A, 0x42_00_B5, 0x42_00_B6, 0x42_00_B7}

    # Reuse the existing allocation, parent, duplicate-tag, and encoding checks
    # so the new typed surface cannot develop separate catalog semantics.
    encodings = dict(_attribute_encodings(catalog, versioned_tags))
    tag_records: dict[str, tuple[int, bool]] = {}
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "tag":
            continue
        element_id = element.get("element_id")
        wire_value = element.get("wire_value")
        if not isinstance(element_id, str) or not _WIRE_VALUE.fullmatch(str(wire_value)):
            raise ValueError("catalog tag has an invalid identity or wire value")
        allocation = element.get("allocation")
        if allocation not in {"assigned", "reserved"}:
            raise ValueError("catalog tag has an unsupported allocation")
        tag_records[element_id] = (
            int(str(wire_value).removeprefix("0x").removeprefix("0X"), 16),
            allocation == "assigned",
        )

    definitions: list[TypedAttributeDefinition] = []
    markers: set[str] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "attribute":
            continue
        element_id = element.get("element_id")
        if not isinstance(element_id, str):
            raise ValueError("catalog attribute has an invalid element identity")
        scope_state = element.get("scope_state")
        if scope_state is not None and scope_state != "client_1_0":
            continue
        if element_id == _CERTIFICATE_ATTRIBUTES_ID:
            # §4.6/Table 39 describes certificate contents, not a direct TTLV
            # Attribute Item; it has no source tag or value encoding.
            if element.get("source_encoding") or element.get("parent_element_ids"):
                raise ValueError("Certificate Attributes unexpectedly has direct Item metadata")
            continue

        source_name = element.get("source_name") or element.get("name")
        if not isinstance(source_name, str) or not source_name.strip():
            raise ValueError("catalog attribute has an invalid source name")
        marker_name = _marker_identifier(element_id)
        if marker_name in markers:
            raise ValueError("catalog attributes have duplicate typed marker names")
        markers.add(marker_name)

        source_encoding = element.get("source_encoding")
        if not isinstance(source_encoding, str) or not source_encoding.strip():
            raise ValueError("catalog attribute has an invalid source encoding")
        types = _item_types_for_encoding(source_encoding)
        if types == ["TextString", "Enumeration", "Integer"]:
            value_kind = "UniqueIdentifier"
        elif len(types) == 1:
            value_kind = types[0]
        else:
            raise ValueError("catalog attribute has an unsupported typed value union")

        parents = element.get("parent_element_ids")
        if element_id == _VENDOR_ATTRIBUTE_ID:
            if source_encoding != "Structure" or parents != []:
                raise ValueError("Vendor Attribute must retain its separate Table 150 structure")
            if not any(
                raw_tag == _VENDOR_ATTRIBUTE_TAG and assigned
                for raw_tag, assigned in tag_records.values()
            ):
                raise ValueError("Vendor Attribute tag is not catalogued as assigned")
            raw_tag = _VENDOR_ATTRIBUTE_TAG
        else:
            if (
                not isinstance(parents, list)
                or len(parents) != 1
                or not isinstance(parents[0], str)
                or parents[0] not in tag_records
            ):
                raise ValueError("catalog attribute must have exactly one tag parent")
            raw_tag, assigned = tag_records[parents[0]]
            if not assigned:
                raise ValueError("catalog attribute parent tag is not assigned")
            if raw_tag not in encodings:
                raise ValueError("catalog attribute tag has no validated TTLV encoding")

        min_version = "2.1" if raw_tag in v21_tags else "2.0"
        max_version = "2.1" if raw_tag in deprecated_in_v30 else None
        definitions.append(
            TypedAttributeDefinition(
                element_id=element_id,
                source_name=source_name,
                marker_name=marker_name,
                tag=raw_tag,
                value_kind=value_kind,
                source_multiplicity=_source_multiplicity(element),
                min_version=min_version,
                max_version=max_version,
            )
        )

    for entry in _versioned_tag_attributes(versioned_tags):
        source_name = entry.get("name")
        wire_value = entry.get("wire_value")
        source_encoding = entry.get("source_encoding")
        if not isinstance(source_name, str) or not source_name.strip():
            raise ValueError("versioned attribute has an invalid source name")
        if not _WIRE_VALUE.fullmatch(str(wire_value)):
            raise ValueError("versioned attribute has an invalid wire value")
        if not isinstance(source_encoding, str) or not source_encoding.strip():
            raise ValueError("versioned attribute has an invalid source encoding")
        raw_tag = int(str(wire_value).removeprefix("0x").removeprefix("0X"), 16)
        if raw_tag not in encodings:
            raise ValueError("versioned attribute tag has no validated TTLV encoding")
        marker_name = _marker_from_words(re.findall(r"[A-Za-z]+|[0-9]+", source_name))
        if marker_name in markers:
            raise ValueError("versioned attributes have duplicate typed marker names")
        markers.add(marker_name)
        types = _item_types_for_encoding(source_encoding)
        if types == ["TextString", "Enumeration", "Integer"]:
            value_kind = "UniqueIdentifier"
        elif len(types) == 1:
            value_kind = types[0]
        else:
            raise ValueError("versioned attribute has an unsupported typed value union")
        multiplicity = entry.get("source_multiplicity")
        if multiplicity is not None and (not isinstance(multiplicity, str) or not multiplicity.strip()):
            raise ValueError("versioned attribute has invalid multiplicity metadata")
        element_id = f"KMIPKIT-ELEM-ATTRIBUTE-V30-{_constant_name(marker_name).replace('_', '-')}"
        definitions.append(
            TypedAttributeDefinition(
                element_id=element_id,
                source_name=source_name,
                marker_name=marker_name,
                tag=raw_tag,
                value_kind=value_kind,
                source_multiplicity=multiplicity,
                min_version="3.0",
                max_version=None,
            )
        )

    if not definitions:
        raise ValueError("catalog has no direct typed attributes")
    tags = [definition.tag for definition in definitions]
    if len(tags) != len(set(tags)):
        raise ValueError("catalog attributes have duplicate typed tags")
    return sorted(definitions, key=lambda definition: definition.tag)


def render_rust(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render a deterministic lookup table from catalog attribute encodings."""
    encodings = _attribute_encodings(catalog, versioned_tags)
    lines = [
        "// @generated by verification/conformance/generate_attribute_types.py; do not edit.",
        "use kmipkit_ttlv::ItemType;",
        "",
        "pub(super) const ATTRIBUTE_TYPES: &[(u32, &[ItemType])] = &[",
    ]
    for raw_tag, types in encodings:
        if len(types) == 1:
            lines.append(
                f"    ({_rust_number(raw_tag)}, &[ItemType::{types[0]}]),"
            )
        else:
            lines.extend([
                "    (",
                f"        {_rust_number(raw_tag)},",
                "        &[",
                *(f"            ItemType::{item_type}," for item_type in types),
                "        ],",
                "    ),",
            ])
    lines.extend([
        "];",
        "",
        "pub(super) fn expected_types(raw_tag: u32) -> Option<&'static [ItemType]> {",
        "    ATTRIBUTE_TYPES",
        "        .binary_search_by_key(&raw_tag, |(tag, _)| *tag)",
        "        .ok()",
        "        .map(|index| ATTRIBUTE_TYPES[index].1)",
        "}",
        "",
    ])
    return "\n".join(lines)


def render_client_attribute_tags_rust(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render the source-backed direct-attribute tag allowlist for fluent builders."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    tags = " |\n        ".join(_rust_number(definition.tag) for definition in definitions)
    rendered = "\n".join([
        "// @generated by verification/conformance/generate_attribute_types.py; do not edit.",
        "#[allow(clippy::too_many_lines)] // Generated exhaustive tag list.",
        "pub(crate) fn is_catalogued_attribute_tag(raw_tag: u32) -> bool {",
        "    matches!(raw_tag,",
        f"        {tags}",
        "    )",
        "}",
        "",
    ])
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=rendered,
        capture_output=True,
        check=False,
        text=True,
    )
    if formatted.returncode != 0:
        raise ValueError("generated fluent attribute-tag Rust source could not be formatted")
    return formatted.stdout


def _rust_string(value: str) -> str:
    escaped = []
    for character in value:
        if character == "\\":
            escaped.append("\\\\")
        elif character == '"':
            escaped.append('\\"')
        elif character == "\n":
            escaped.append("\\n")
        elif character == "\r":
            escaped.append("\\r")
        elif character == "\t":
            escaped.append("\\t")
        elif ord(character) < 0x20 or ord(character) == 0x7F:
            escaped.append(f"\\u{{{ord(character):X}}}")
        else:
            escaped.append(character)
    return '"' + "".join(escaped) + '"'


def _owned_type(value_kind: str) -> str:
    return {
        "BigInteger": "Vec<u8>",
        "Boolean": "bool",
        "ByteString": "Vec<u8>",
        "DateTime": "i64",
        "DateTimeExtended": "i64",
        "Enumeration": "u32",
        "Integer": "i32",
        "Interval": "u32",
        "LongInteger": "i64",
        "Structure": "kmipkit_ttlv::Structure",
        "TextString": "String",
        "UniqueIdentifier": "super::super::UniqueIdentifierValue",
    }[value_kind]


def _view_type(value_kind: str) -> str:
    return {
        "BigInteger": "&'a [u8]",
        "Boolean": "bool",
        "ByteString": "&'a [u8]",
        "DateTime": "i64",
        "DateTimeExtended": "i64",
        "Enumeration": "u32",
        "Integer": "i32",
        "Interval": "u32",
        "LongInteger": "i64",
        "Structure": "kmipkit_ttlv::StructureView<'a>",
        "TextString": "&'a str",
        "UniqueIdentifier": "super::super::UniqueIdentifierView<'a>",
    }[value_kind]


def render_typed_rust(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render typed markers and source metadata for direct KMIP attributes."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    lines = [
        "// @generated by verification/conformance/generate_attribute_types.py; do not edit.",
        "use super::AttributeDefinition;",
        "",
        "#[cfg(test)]",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(crate) struct AttributeMetadata {",
        "    pub(crate) element_id: &'static str,",
        "    pub(crate) source_name: &'static str,",
        "    pub(crate) marker_name: &'static str,",
        "    pub(crate) tag: u32,",
        "    pub(crate) value_kind: &'static str,",
        "    pub(crate) source_multiplicity: Option<&'static str>,",
        "    pub(crate) min_version: (u8, u8),",
        "    pub(crate) max_version: Option<(u8, u8)>,",
        "}",
        "",
        "#[cfg(test)]",
        "pub(crate) const ATTRIBUTE_DEFINITIONS: &[AttributeMetadata] = &[",
    ]
    for definition in definitions:
        min_ver = f"({definition.min_version.replace('.', ', ')})"
        max_ver = (
            f"Some(({definition.max_version.replace('.', ', ')}))"
            if definition.max_version
            else "None"
        )
        lines.append(
            "    AttributeMetadata { "
            f"element_id: {_rust_string(definition.element_id)}, "
            f"source_name: {_rust_string(definition.source_name)}, "
            f"marker_name: {_rust_string(definition.marker_name)}, "
            f"tag: {_rust_number(definition.tag)}, "
            f"value_kind: {_rust_string(definition.value_kind)}, "
            "source_multiplicity: "
            + (
                f"Some({_rust_string(definition.source_multiplicity)})"
                if definition.source_multiplicity is not None
                else "None"
            )
            + ", "
            f"min_version: {min_ver}, "
            f"max_version: {max_ver} "
            "},"
        )
    lines.extend(
        [
            "];",
            "",
            "pub mod typed {",
            "    use super::super::AttributeValueError;",
            "    use kmipkit_ttlv::{Value, ValueView};",
            "",
            "    macro_rules! define_typed_attribute {",
            "        ($marker:ident, $source_name:literal, $tag:literal, $min_version:literal, $encoding:ty, $owned:ty, $view:ty) => {",
            "            #[doc = concat!(\"Typed marker for the KMIP \", $min_version, \" \", $source_name, \" attribute.\")]",
            "            pub struct $marker;",
            "            impl super::super::sealed::Sealed for $marker {}",
            "            impl super::AttributeDefinition for $marker {",
            "                type Value = $owned;",
            "                type ValueView<'a> = $view where Self: 'a;",
            "                const TAG: u32 = $tag;",
            "",
            "                fn into_ttlv(value: Self::Value) -> Value {",
            "                    <$encoding as super::super::AttributeValueEncoding>::into_ttlv(value)",
            "                }",
            "",
            "                fn value_view<'a>(value: ValueView<'a>) -> Result<Self::ValueView<'a>, AttributeValueError>",
            "                where",
            "                    Self: 'a,",
            "                {",
            "                    <$encoding as super::super::AttributeValueEncoding>::value_view(value)",
            "                }",
            "            }",
            "        };",
            "    }",
            "",
        ]
    )
    for definition in definitions:
        encoding = _ENCODING_NAMES[definition.value_kind]
        lines.append(
            f"    define_typed_attribute!({definition.marker_name}, "
            f"{_rust_string(definition.source_name)}, {_rust_number(definition.tag)}, "
            f"{_rust_string(definition.min_version)}, "
            f"super::super::{encoding}, {_owned_type(definition.value_kind)}, "
            f"{_view_type(definition.value_kind)});"
        )
    lines.extend(
        [
            "}",
            "",
            "#[cfg(test)]",
            "pub(crate) const ATTRIBUTE_TAGS: &[u32] = &[",
        ]
    )
    for definition in definitions:
        lines.append(f"    typed::{definition.marker_name}::TAG,")
    lines.extend(["];", ""])
    rendered = "\n".join(lines)
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=rendered,
        capture_output=True,
        check=False,
        text=True,
    )
    if formatted.returncode != 0:
        raise ValueError("generated typed attribute Rust source could not be formatted")
    return formatted.stdout


def render_typed_ffi_rust(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render the direct-attribute tag and value-kind catalog for the C ABI."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    tags = " |\n        ".join(_rust_number(definition.tag) for definition in definitions)
    entries = "\n".join(
        f'    ({_rust_number(definition.tag)}, "{definition.value_kind}"),'
        for definition in definitions
    )
    rendered = "\n".join([
        "// @generated by verification/conformance/generate_attribute_types.py; do not edit.",
        "/// Returns whether the numeric tag belongs to the generated direct-attribute catalog.",
        "#[allow(clippy::too_many_lines)] // Generated exhaustive tag list.",
        "pub(super) fn is_catalogued_attribute_tag(raw_tag: u32) -> bool {",
        "    matches!(raw_tag,",
        f"        {tags}",
        "    )",
        "}",
        "",
        "/// Catalogued direct-attribute tag and source TTLV value kind.",
        "#[cfg(test)]",
        "pub(super) const ATTRIBUTE_DEFINITIONS: &[(u32, &str)] = &[",
        entries,
        "];",
        "",
    ])
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=rendered,
        capture_output=True,
        check=False,
        text=True,
    )
    if formatted.returncode != 0:
        raise ValueError("generated typed attribute FFI source could not be formatted")
    return formatted.stdout


def render_typed_c_header(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render named C tag constants for directly typed KMIP attributes."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    lines = [
        "/* @generated by verification/conformance/generate_attribute_types.py; do not edit. */",
        "#ifndef KMIPKIT_STANDARD_ATTRIBUTES_GENERATED_H",
        "#define KMIPKIT_STANDARD_ATTRIBUTES_GENERATED_H",
        "#include <stdint.h>",
        "",
        "#define KMIPKIT_STANDARD_ATTRIBUTE_TAG(attribute) ((uint32_t)(attribute))",
        "",
    ]
    for definition in definitions:
        lines.append(
            f"#define KMIPKIT_STANDARD_ATTRIBUTE_{_constant_name(definition.marker_name)} "
            f"((uint32_t)0x{definition.tag:08X})"
        )
    lines.extend(["", "#endif /* KMIPKIT_STANDARD_ATTRIBUTES_GENERATED_H */", ""])
    return "\n".join(lines)


def _constant_name(marker_name: str) -> str:
    """Convert a generated Rust marker to a stable foreign-language constant."""
    return re.sub(r"(?<!^)(?=[A-Z])", "_", marker_name).upper()


def render_typed_java(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render the Java catalog enum for typed direct attributes."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    lines = [
        "// @generated by verification/conformance/generate_attribute_types.py; do not edit.",
        "package org.kmipkit.ttlv;",
        "",
        "/** OASIS KMIP 2.0–3.0 direct object attributes with their assigned TTLV tags. */",
        "public enum StandardAttribute {",
    ]
    for index, definition in enumerate(definitions):
        suffix = ";" if index == len(definitions) - 1 else ","
        multiplicity = (
            "null"
            if definition.source_multiplicity is None
            else json.dumps(definition.source_multiplicity, ensure_ascii=True)
        )
        max_version = (
            "null"
            if definition.max_version is None
            else json.dumps(definition.max_version, ensure_ascii=True)
        )
        lines.append(
            f"    {_constant_name(definition.marker_name)}(0x{definition.tag:06X}, "
            f"\"{definition.value_kind}\", {multiplicity}, \"{definition.min_version}\", {max_version}){suffix}"
        )
    lines.extend(
        [
            "",
            "    private final int tag;",
            "    private final String valueKind;",
            "    private final String sourceMultiplicity;",
            "    private final String minVersion;",
            "    private final String maxVersion;",
            "",
            "    StandardAttribute(int tag, String valueKind, String sourceMultiplicity, String minVersion, String maxVersion) {",
            "        this.tag = tag;",
            "        this.valueKind = valueKind;",
            "        this.sourceMultiplicity = sourceMultiplicity;",
            "        this.minVersion = minVersion;",
            "        this.maxVersion = maxVersion;",
            "    }",
            "",
            "    /** Returns the assigned numeric KMIP tag. */",
            "    public int tag() { return tag; }",
            "",
            "    /** Returns the catalogued TTLV value kind or value union. */",
            "    public String valueKind() { return valueKind; }",
            "",
            "    /** Returns the exact source multiplicity text, or {@code null} when silent. */",
            "    public String sourceMultiplicity() { return sourceMultiplicity; }",
            "",
            "    /** Returns the minimum OASIS KMIP version where this attribute is valid. */",
            "    public String minVersion() { return minVersion; }",
            "",
            "    /** Returns the maximum OASIS KMIP version where this attribute is valid, or {@code null} if not deprecated. */",
            "    public String maxVersion() { return maxVersion; }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def render_typed_python(
    catalog: dict[str, Any],
    versioned_tags: dict[str, Any] | None = None,
) -> str:
    """Render the Python catalog enum for typed direct attributes."""
    definitions = collect_typed_attributes(catalog, versioned_tags)
    lines = [
        '"""Generated catalog of OASIS KMIP 2.0–3.0 direct object attributes."""',
        "",
        "from enum import Enum",
        "",
        "",
        "class StandardAttribute(Enum):",
        '    """Assigned tag, TTLV value kind, and source multiplicity for an attribute."""',
    ]
    for definition in definitions:
        multiplicity = (
            "None"
            if definition.source_multiplicity is None
            else json.dumps(definition.source_multiplicity, ensure_ascii=True)
        )
        max_ver = (
            "None"
            if definition.max_version is None
            else json.dumps(definition.max_version, ensure_ascii=True)
        )
        lines.append(
            f"    {_constant_name(definition.marker_name)} = "
            f"(0x{definition.tag:06X}, \"{definition.value_kind}\", {multiplicity}, \"{definition.min_version}\", {max_ver})"
        )
    lines.extend(
        [
            "",
            "    @property",
            "    def tag(self) -> int:",
            "        return self.value[0]",
            "",
            "    @property",
            "    def value_kind(self) -> str:",
            "        return self.value[1]",
            "",
            "    @property",
            "    def source_multiplicity(self) -> str | None:",
            "        return self.value[2]",
            "",
            "    @property",
            "    def min_version(self) -> str:",
            "        return self.value[3]",
            "",
            "    @property",
            "    def max_version(self) -> str | None:",
            "        return self.value[4]",
            "",
        ]
    )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the generated Rust source")
    mode.add_argument("--check", action="store_true", help="fail if generated Rust source is stale")
    arguments = parser.parse_args(argv)

    try:
        root = arguments.repo_root.resolve(strict=True)
        raw_catalog = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(raw_catalog, root)
        versioned_tags: dict[str, Any] | None = None
        if (root / VERSIONED_TAGS_PATH).is_file():
            raw_versioned_tags = safe_read_bytes(root, VERSIONED_TAGS_PATH, max_bytes=MAX_BYTES)
            versioned_tags = json.loads(raw_versioned_tags.decode("utf-8"))
        generated = {
            OUTPUT_PATH: render_rust(catalog, versioned_tags).encode("utf-8"),
            TYPED_OUTPUT_PATH: render_typed_rust(catalog, versioned_tags).encode("utf-8"),
            TYPED_FFI_OUTPUT_PATH: render_typed_ffi_rust(catalog, versioned_tags).encode("utf-8"),
            TYPED_C_HEADER_OUTPUT_PATH: render_typed_c_header(catalog, versioned_tags).encode("utf-8"),
            TYPED_JAVA_OUTPUT_PATH: render_typed_java(catalog, versioned_tags).encode("utf-8"),
            TYPED_PYTHON_OUTPUT_PATH: render_typed_python(catalog, versioned_tags).encode("utf-8"),
            CLIENT_ATTRIBUTE_TAG_OUTPUT_PATH: render_client_attribute_tags_rust(
                catalog, versioned_tags
            ).encode("utf-8"),
        }
        if arguments.write:
            for path, contents in generated.items():
                atomic_write_bytes(root, path, contents)
                print(f"generated attribute TTLV types: {root / path}")
            return 0
        existing = {
            path: safe_read_bytes(root, path, max_bytes=MAX_BYTES)
            for path in generated
        }
    except CatalogValidationError:
        print("attribute type generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except PathSecurityError:
        print("attribute type generation failed: repository path validation failed", file=sys.stderr)
        return 2
    except OSError:
        print("attribute type generation failed: repository I/O failed", file=sys.stderr)
        return 2
    except ValueError as error:
        print(f"attribute type generation failed: {error}", file=sys.stderr)
        return 2

    stale = False
    for path, contents in generated.items():
        if existing[path] != contents:
            print(
                f"generated attribute TTLV types are stale at {path}; "
                "run generate_attribute_types.py --write",
                file=sys.stderr,
            )
            stale = True
        else:
            print(f"generated attribute TTLV types verified: {root / path}")
    if stale:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
