#!/usr/bin/env python3
"""Generate Rust consumer checks from the shared normative catalog."""

import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/normative_catalog"))
from generate_named_values import _rust_u32_literal, _snake_identifier

ELEMENTS = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text())["elements"]
CATALOG = json.loads(
    (ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text()
)
ELEMENT_BY_ID = {element["element_id"]: element for element in ELEMENTS}
TARGET_BY_ID = {target["target_id"]: target for target in CATALOG["normative_targets"]}

TYPE_MAP = {
    "Structure": "Structure",
    "Integer": "Integer",
    "Long Integer": "LongInteger",
    "Big Integer": "BigInteger",
    "Enumeration": "Enumeration",
    "Boolean": "Boolean",
    "Text String": "TextString",
    "Byte String": "ByteString",
    "Date Time": "DateTime",
    "Interval": "Interval",
    "Date Time Extended": "DateTimeExtended",
}
TYPE_NAMES = sorted(TYPE_MAP, key=len, reverse=True)
FIELD_TYPE_FALLBACKS = {
    "Any Object (Section 2)": {"Structure"},
    "All Objects": {"Boolean"},
    "Adjustment Value": {"Integer"},
    "Certificate Request Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Interop Identifier": {"Text String"},
    "Private Key Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Public Key Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Common Protection Storage Masks": {"Structure"},
    "Private Protection Storage Masks": {"Structure"},
    "Public Protection Storage Masks": {"Structure"},
    "Request Count": {"Integer"},
    "Replaced Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Time Stamp": {"Date Time"},
    "Ephemeral": {"Boolean"},
    "Request Payload": {"Structure"},
    "Response Payload": {"Structure"},
    "Server Hashed Password": {"Structure"},
    "Vendor-Specific": {"Structure"},
    "Credential Value": set(TYPE_MAP),
    "Attribute Value": set(TYPE_MAP),
    "Any attribute in §4 - Object Attributes": set(TYPE_MAP),
}


def raw_value(value: str) -> int:
    return int(value.removeprefix("0x").removeprefix("0X"), 16)


def pattern_raw_values(value: str) -> list[int]:
    """Select deterministic representatives from an OASIS value or range."""
    range_match = re.fullmatch(r"\s*([0-9a-fA-F]+)\s*-\s*([0-9a-fA-F]+)\s*", value)
    if range_match:
        return [int(part, 16) for part in range_match.groups()]
    if "X" in value.upper():
        return [int(value.upper().replace("X", "0"), 16)]
    return [raw_value(value)]


def mask_raw_value(element: dict, parent: dict) -> int:
    wire_value = element["wire_value"]
    if "X" not in wire_value.upper():
        return raw_value(wire_value)
    wildcard_mask = int(wire_value.upper().replace("X", "F"), 16)
    assigned_mask = 0
    for candidate in ELEMENTS:
        if (
            parent["element_id"] in candidate["parent_element_ids"]
            and candidate["kind"] == "bitmask_value"
            and candidate.get("allocation") == "assigned"
            and "X" not in candidate.get("wire_value", "").upper()
        ):
            assigned_mask |= raw_value(candidate["wire_value"])
    available = wildcard_mask & ~assigned_mask & 0xFFFF_FFFF
    if available == 0:
        raise ValueError(f"no concrete unassigned bit exists in {element['element_id']}")
    return available & -available


def type_names(value: str) -> set[str]:
    normalized = value.replace("-", " ")
    found = set()
    for name in TYPE_NAMES:
        pattern = re.escape(name).replace(r"\ ", r"[\s-]+")
        if re.search(rf"(?<![A-Za-z]){pattern}(?![A-Za-z])", normalized, re.IGNORECASE):
            found.add(name)
    return found


def element_type_names(element: dict) -> set[str]:
    names = set()
    for field in ("source_encoding", "value_type"):
        if isinstance(element.get(field), str):
            names.update(type_names(element[field]))
    for field in ("value_type_alternatives",):
        names.update(element.get(field, []))
    names.intersection_update(TYPE_MAP)
    if element["kind"] == "enumeration":
        names.add("Enumeration")
    elif element["kind"] == "bitmask":
        names.add("Integer")
    elif element["kind"] in {
        "attribute_structure",
        "object_structure",
        "operation_structure",
        "credential",
    }:
        names.add("Structure")
    return names


ELEMENT_TYPES_BY_NAME = {}
for element in ELEMENTS:
    ELEMENT_TYPES_BY_NAME.setdefault(element["name"], set()).update(element_type_names(element))


def value_type_expression(types: set[str]) -> str:
    ordered = [name for name in TYPE_MAP if name in types]
    if not ordered:
        raise ValueError("normative target has no TTLV type construction evidence")
    variants = ", ".join(f"ItemType::{TYPE_MAP[name]}" for name in ordered)
    return f"super::value_types_roundtrip(&[{variants}])"


def enum_parent(element: dict) -> dict:
    parents = [ELEMENT_BY_ID[parent] for parent in element["parent_element_ids"]]
    for parent in parents:
        if parent["kind"] == "enumeration":
            return parent
    raise ValueError(f"{element['element_id']} has no parent Enumeration")


def bitmask_parent(element: dict) -> dict:
    parents = [ELEMENT_BY_ID[parent] for parent in element["parent_element_ids"]]
    for parent in parents:
        if parent["kind"] == "bitmask":
            return parent
    raise ValueError(f"{element['element_id']} has no parent Bit Mask")


def allocation_expression(raw: int, allocation: str) -> str:
    allocated = allocation in {"assigned", "extension"}
    return f"super::tag_allocation_matches({_rust_u32_literal(raw)}, {str(allocated).lower()})"


def normative_expression(case: dict) -> str:
    requires = case["requires"]
    kind = requires["target_kind"]
    target_id = requires["normative_target_id"]
    target = TARGET_BY_ID.get(target_id)
    if target is None:
        raise ValueError(f"{case['case_id']} references unknown normative target {target_id}")

    if kind == "enumeration_value":
        element = ELEMENT_BY_ID[target_id]
        group = enum_parent(element)
        raw_values = pattern_raw_values(requires["wire_value"])
        assigned = requires.get("allocation") == "assigned"
        known = "is_some()" if assigned else "is_none()"
        return " && ".join(
            f"enumeration::{_snake_identifier(group['name'])}::name({_rust_u32_literal(raw)}).{known}"
            f" && super::generic_enumeration_check({_rust_u32_literal(raw)})"
            for raw in raw_values
        )

    if kind == "referenced_enum_value":
        return " && ".join(
            f"super::generic_enumeration_check({_rust_u32_literal(raw)})"
            for raw in pattern_raw_values(requires["wire_value"])
        )

    if kind == "bitmask_value":
        element = ELEMENT_BY_ID[target_id]
        group = bitmask_parent(element)
        module = _snake_identifier(group["name"])
        raw = mask_raw_value(element, group)
        if requires.get("allocation") == "assigned":
            named = f"bitmask::{module}::known_bits({_rust_u32_literal(raw)}) == {_rust_u32_literal(raw)} && bitmask::{module}::names({_rust_u32_literal(raw)}).count() == 1"
        else:
            named = f"bitmask::{module}::unknown_bits({_rust_u32_literal(raw)}) == {_rust_u32_literal(raw)} && bitmask::{module}::names({_rust_u32_literal(raw)}).next().is_none()"
        return f"{named} && super::generic_integer_check({_rust_u32_literal(raw)})"

    if kind == "tag":
        return allocation_expression(raw_value(requires["wire_value"]), requires.get("allocation", "assigned"))

    if kind == "tag_range":
        range_parts = [part.strip() for part in re.split(r"[-–—]", requires["wire_value"])]
        if len(range_parts) != 2 or any(not re.fullmatch(r"(?:0x)?[0-9a-fA-FXx]+", part) for part in range_parts):
            raise ValueError(f"{case['case_id']} has an unsupported tag range")
        allocation = requires.get("allocation", "unused")
        return " && ".join(
            allocation_expression(
                int(value.removeprefix("0x").replace("X", fill).replace("x", fill), 16),
                allocation,
            )
            for value, fill in zip(range_parts, ("0", "F"), strict=True)
        )

    if kind == "data_type":
        expected_name = target["name"]
        return f"super::ttlv_boundary_vectors_match(ItemType::{TYPE_MAP[expected_name]})"

    if kind == "operation_payload_value_variant":
        return value_type_expression({requires["value_type"]})

    if kind == "operation_payload_field":
        alternatives = set()
        for alternative in target.get("value_type_alternatives", []):
            alternatives.update(type_names(alternative))
        alternatives.update(ELEMENT_TYPES_BY_NAME.get(requires["field_name"], set()))
        alternatives.update(FIELD_TYPE_FALLBACKS.get(requires["field_name"], set()))
        return value_type_expression(alternatives)

    element = ELEMENT_BY_ID.get(target_id)
    if element is not None:
        types = element_type_names(element) | ELEMENT_TYPES_BY_NAME.get(element["name"], set())
        types.update(FIELD_TYPE_FALLBACKS.get(element["name"], set()))
    else:
        types = set()
    if kind == "attribute_structure":
        types.add("Structure")
    return value_type_expression(types)


def legacy_expression(case: dict) -> str:
    requires = case["requires"]
    if "enum_type" in requires:
        kind, name = "enumeration", requires["enum_type"]
    elif "bitmask_type" in requires:
        kind, name = "bitmask", requires["bitmask_type"]
    else:
        return "false"
    group = next(element for element in ELEMENTS if element["kind"] == kind and element["name"] == name)
    value_kind = "enumeration_value" if kind == "enumeration" else "bitmask_value"
    values = [
        element
        for element in ELEMENTS
        if group["element_id"] in element["parent_element_ids"]
        and element["kind"] == value_kind
        and element.get("allocation") == "assigned"
        and re.fullmatch(r"(?:0x)?[0-9a-fA-F]{1,8}", element.get("wire_value", ""))
    ]
    module = _snake_identifier(name)
    if kind == "enumeration":
        if not values:
            if name == "Tag":
                values = [
                    element
                    for element in ELEMENTS
                    if group["element_id"] in element["parent_element_ids"]
                    and element["kind"] == "tag"
                    and element.get("allocation") == "assigned"
                    and re.fullmatch(
                        r"(?:0x)?[0-9a-fA-F]{1,8}", element.get("wire_value", "")
                    )
                ]
            if values:
                checks = [
                    f"super::generic_enumeration_check({_rust_u32_literal(raw_value(element['wire_value']))})"
                    for element in values
                ]
                checks.append("super::generic_enumeration_check(0x8000_0000)")
                return " && ".join(checks)
            # These standard Enumeration tables contain no assigned values;
            # exercise the KMIP extension range without inventing known values.
            return "super::generic_enumeration_check(0x8000_0000)"
        checks = [
            f"enumeration::{module}::name({_rust_u32_literal(raw_value(element['wire_value']))}).is_some() && super::generic_enumeration_check({_rust_u32_literal(raw_value(element['wire_value']))})"
            for element in values
        ]
    else:
        checks = [
            f"bitmask::{module}::names({_rust_u32_literal(mask_raw_value(element, group))}).count() == 1 && super::generic_integer_check({_rust_u32_literal(mask_raw_value(element, group))})"
            for element in values
        ]
    if requires.get("exhaustive_bitmask_combinations"):
        raw_values = [raw_value(element["wire_value"]) for element in values]
        list_values = ", ".join(_rust_u32_literal(value) for value in raw_values)
        checks.append(
            "super::bitmask_combinations_roundtrip("
            f"&[{list_values}], |mask| bitmask::{module}::names(mask).count())"
        )
    return " && ".join(checks) if checks else "false"


lines = [
    "// Generated by tools/generate_local_cases.py; do not edit.",
    "use kmipkit::protocol::named_values_generated::values::{bitmask, enumeration};",
    "use kmipkit::ttlv::ItemType;",
    "#[allow(clippy::too_many_lines, clippy::match_same_arms)] // Exhaustive generated catalog dispatch.",
    "pub(crate) fn verify(name: &str) -> bool {",
    "    match name {",
]
for case in CATALOG["test_cases"]:
    if case["execution"]["mode"] != "local_model":
        continue
    requires = case["requires"]
    try:
        expression = (
            normative_expression(case)
            if "target_kind" in requires
            else legacy_expression(case)
        )
    except ValueError as error:
        raise ValueError(f"{case['case_id']}: {error}") from error
    lines.append(f'        "{case["case_id"]}" => {expression},')
lines.extend(["        _ => false,", "    }", "}", ""])

lines.extend(["pub(crate) fn attribute_tag(name: &str) -> Option<u32> {", "    match name {"])
for element in ELEMENTS:
    if element["kind"] == "attribute":
        tags = [parent for parent in element["parent_element_ids"] if parent.startswith("KMIPKIT-ELEM-TAG-")]
        if tags:
            raw = int(tags[0].split("-")[-1], 16)
            lines.append(f'        "{element["name"]}" => Some({_rust_u32_literal(raw)}),')
lines.extend(["        _ => None,", "    }", "}", ""])

output = ROOT / "interop/rust-kmip-validator/src/local_cases_generated.rs"
output.write_text("\n".join(lines))
subprocess.run(
    ["rustfmt", "+1.94", "--edition", "2024", str(output)],
    check=True,
)
