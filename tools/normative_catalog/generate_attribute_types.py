"""Generate Rust TTLV type constraints for catalogued KMIP attributes."""

from __future__ import annotations

import argparse
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


CATALOG_PATH = "specification/catalog/kmip-2.1.json"
OUTPUT_PATH = "crates/kmipkit-protocol/src/attribute_types_generated.rs"
TYPED_OUTPUT_PATH = "crates/kmipkit-protocol/src/typed_attributes_generated.rs"
_VENDOR_ATTRIBUTE_ID = "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE"
_CERTIFICATE_ATTRIBUTES_ID = "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES"
_VENDOR_ATTRIBUTE_TAG = 0x0042_0008
_WIRE_VALUE = re.compile(r"(?:0x)?[0-9A-Fa-f]{6}\Z")
_TYPE_NAMES = {
    "Big Integer": "BigInteger",
    "Boolean": "Boolean",
    "Byte String": "ByteString",
    "Date Time": "DateTime",
    "Date-Time": "DateTime",
    "Date Time Extended": "DateTimeExtended",
    "Enumeration": "Enumeration",
    "Integer": "Integer",
    "Interval": "Interval",
    "RNG Parameters": "Structure",
    "Structure": "Structure",
    "Text String": "TextString",
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


def _attribute_encodings(catalog: dict[str, Any]) -> list[tuple[int, list[str]]]:
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

    if not attributes:
        raise ValueError("catalog has no typed attribute encodings")
    return sorted(attributes)


def _rust_number(value: int) -> str:
    return f"0x{value >> 16:02X}_{(value >> 8) & 0xFF:02X}_{value & 0xFF:02X}"


def _item_types_for_encoding(source_encoding: str) -> list[str]:
    names = [part.strip() for part in re.split(r",|\s+or\s+", source_encoding) if part.strip()]
    if not names or any(name not in _TYPE_NAMES for name in names):
        raise ValueError("catalog attribute has an unsupported source encoding")
    return list(dict.fromkeys(_TYPE_NAMES[name] for name in names))


def _marker_identifier(element_id: str) -> str:
    prefix = "KMIPKIT-ELEM-ATTRIBUTE-"
    if not element_id.startswith(prefix):
        raise ValueError("catalog attribute has an invalid element identity")
    words = re.findall(r"[A-Za-z]+|[0-9]+", element_id[len(prefix) :])
    if not words:
        raise ValueError("catalog attribute has an invalid element identity")
    marker = "".join(
        word if word.isdigit() else word[0].upper() + word[1:].lower()
        for word in words
    )
    if not marker[0].isalpha():
        marker = "Attribute" + marker
    return marker


def _source_multiplicity(element: dict[str, Any]) -> str | None:
    value = element.get("source_multiple_instances_permitted")
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise ValueError("catalog attribute has invalid multiplicity metadata")
    return value


def collect_typed_attributes(catalog: dict[str, Any]) -> list[TypedAttributeDefinition]:
    """Collect direct, typed object attributes from the validated catalog."""
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    # Reuse the existing allocation, parent, duplicate-tag, and encoding checks
    # so the new typed surface cannot develop separate catalog semantics.
    encodings = dict(_attribute_encodings(catalog))
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

        definitions.append(
            TypedAttributeDefinition(
                element_id=element_id,
                source_name=source_name,
                marker_name=marker_name,
                tag=raw_tag,
                value_kind=value_kind,
                source_multiplicity=_source_multiplicity(element),
            )
        )

    if not definitions:
        raise ValueError("catalog has no direct typed attributes")
    tags = [definition.tag for definition in definitions]
    if len(tags) != len(set(tags)):
        raise ValueError("catalog attributes have duplicate typed tags")
    return sorted(definitions, key=lambda definition: definition.tag)


def render_rust(catalog: dict[str, Any]) -> str:
    """Render a deterministic lookup table from catalog attribute encodings."""
    encodings = _attribute_encodings(catalog)
    lines = [
        "// @generated by tools/normative_catalog/generate_attribute_types.py; do not edit.",
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


def _render_typed_impl(definition: TypedAttributeDefinition) -> list[str]:
    marker = definition.marker_name
    tag = _rust_number(definition.tag)
    lines = [
        f"    /// Typed marker for the KMIP 2.1 {definition.source_name} attribute.",
        f"    pub struct {marker};",
        f"    impl super::super::sealed::Sealed for {marker} {{}}",
        f"    impl super::AttributeDefinition for {marker} {{",
        f"        type Value = {_owned_type(definition.value_kind)};",
        f"        type ValueView<'a> = {_view_type(definition.value_kind)} where Self: 'a;",
        f"        const TAG: u32 = {tag};",
        "",
        "        fn into_ttlv(value: Self::Value) -> Value {",
    ]
    if definition.value_kind == "UniqueIdentifier":
        lines.extend(
            [
                "            match value {",
                "                UniqueIdentifierValue::TextString(value) => Value::text_string(value),",
                "                UniqueIdentifierValue::Enumeration(value) => Value::enumeration(value),",
                "                UniqueIdentifierValue::Integer(value) => Value::integer(value),",
                "            }",
            ]
        )
    else:
        constructor = {
            "BigInteger": "big_integer",
            "Boolean": "boolean",
            "ByteString": "byte_string",
            "DateTime": "date_time",
            "DateTimeExtended": "date_time_extended",
            "Enumeration": "enumeration",
            "Integer": "integer",
            "Interval": "interval",
            "Structure": "structure",
            "TextString": "text_string",
        }[definition.value_kind]
        lines.append(f"            Value::{constructor}(value)")
    lines.extend(
        [
            "        }",
            "",
            "        fn value_view<'a>(",
            "            value: ValueView<'a>,",
            "        ) -> Result<Self::ValueView<'a>, AttributeValueError>\n        where\n            Self: 'a,\n        {",
        ]
    )
    if definition.value_kind == "UniqueIdentifier":
        lines.extend(
            [
                "            match value {",
                "                ValueView::TextString(value) => Ok(UniqueIdentifierView::TextString(value)),",
                "                ValueView::Enumeration(value) => Ok(UniqueIdentifierView::Enumeration(*value)),",
                "                ValueView::Integer(value) => Ok(UniqueIdentifierView::Integer(*value)),",
                "                _ => Err(AttributeValueError::ValueTypeMismatch),",
                "            }",
            ]
        )
    else:
        view_variant = {
            "BigInteger": ("BigInteger", "value"),
            "Boolean": ("Boolean", "*value"),
            "ByteString": ("ByteString", "value"),
            "DateTime": ("DateTime", "*value"),
            "DateTimeExtended": ("DateTimeExtended", "*value"),
            "Enumeration": ("Enumeration", "*value"),
            "Integer": ("Integer", "*value"),
            "Interval": ("Interval", "*value"),
            "Structure": ("Structure", "value"),
            "TextString": ("TextString", "value"),
        }[definition.value_kind]
        variant, expression = view_variant
        lines.extend(
            [
                "            match value {",
                f"                ValueView::{variant}(value) => Ok({expression}),",
                "                _ => Err(AttributeValueError::ValueTypeMismatch),",
                "            }",
            ]
        )
    lines.extend(["        }", "    }", ""])
    return lines


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
        "Structure": "Structure",
        "TextString": "String",
        "UniqueIdentifier": "UniqueIdentifierValue",
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
        "Structure": "StructureView<'a>",
        "TextString": "&'a str",
        "UniqueIdentifier": "UniqueIdentifierView<'a>",
    }[value_kind]


def render_typed_rust(catalog: dict[str, Any]) -> str:
    """Render typed markers and source metadata for direct KMIP attributes."""
    definitions = collect_typed_attributes(catalog)
    lines = [
        "// @generated by tools/normative_catalog/generate_attribute_types.py; do not edit.",
        "use super::{",
        "    AttributeDefinition, AttributeValueError, UniqueIdentifierValue,",
        "    UniqueIdentifierView,",
        "};",
        "use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};",
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
        "}",
        "",
        "#[cfg(test)]",
        "pub(crate) const ATTRIBUTE_DEFINITIONS: &[AttributeMetadata] = &[",
    ]
    for definition in definitions:
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
            + " "
            "},"
        )
    lines.extend(["];", "", "pub mod typed {", "    use super::*;", ""])
    for definition in definitions:
        lines.extend(_render_typed_impl(definition))
    lines.extend(["}", ""])
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
        generated = {
            OUTPUT_PATH: render_rust(catalog).encode("utf-8"),
            TYPED_OUTPUT_PATH: render_typed_rust(catalog).encode("utf-8"),
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
