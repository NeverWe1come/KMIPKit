"""Generate the protocol crate's source-backed attribute mutation policies."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from .validate import MAX_BYTES, CatalogValidationError, load_validated_catalog
else:
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from validate import MAX_BYTES, CatalogValidationError, load_validated_catalog


CATALOG_PATH = "specification/catalog/kmip-2.1.json"
OUTPUT_PATH = "crates/kmipkit-protocol/src/generated/attribute_policy.rs"
_TAG_VALUE = re.compile(r"(?:0[xX])?([0-9A-Fa-f]{6})\Z")
_SOURCE_POLICY_COUNT = 62
_RUNTIME_POLICY_COUNT = 61
_CERTIFICATE_ATTRIBUTES_ID = "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES"


def _required_text(record: dict[str, Any], field: str) -> str:
    value = record.get(field)
    if not isinstance(value, str):
        raise ValueError(f"catalog policy field {field} must be text")
    return value


def _rust_string(value: str) -> str:
    """Encode text as a valid deterministic Rust string literal."""
    escaped: list[str] = ['"']
    for character in value:
        codepoint = ord(character)
        if character == '"':
            escaped.append('\\"')
        elif character == "\\":
            escaped.append("\\\\")
        elif character == "\n":
            escaped.append("\\n")
        elif character == "\r":
            escaped.append("\\r")
        elif character == "\t":
            escaped.append("\\t")
        elif codepoint < 0x20 or codepoint == 0x7F:
            escaped.append(f"\\u{{{codepoint:x}}}")
        else:
            escaped.append(character)
    escaped.append('"')
    return "".join(escaped)


def _source_refs(record: dict[str, Any], field: str = "source_refs") -> list[dict[str, str]]:
    refs = record.get(field)
    if not isinstance(refs, list) or not refs:
        raise ValueError(f"catalog policy {field} must be a non-empty list")
    checked: list[dict[str, str]] = []
    for source_ref in refs:
        if not isinstance(source_ref, dict):
            raise ValueError(f"catalog policy {field} contains a malformed reference")
        checked.append(
            {
                "source_id": _required_text(source_ref, "source_id"),
                "section": _required_text(source_ref, "section"),
            }
        )
    return checked


def _policy_rules(attribute: dict[str, Any], field: str) -> list[dict[str, Any]]:
    rules = attribute.get(field)
    if not isinstance(rules, list):
        raise ValueError(f"catalog attribute {field} must be a list")
    checked: list[dict[str, Any]] = []
    for rule in rules:
        if not isinstance(rule, dict):
            raise ValueError(f"catalog attribute {field} contains a malformed rule")
        checked.append(
            {
                "source_text": _required_text(rule, "source_text"),
                "source_refs": _source_refs(rule),
            }
        )
    return checked


def _tag_value(tag: dict[str, Any]) -> int:
    value = tag.get("wire_value")
    match = _TAG_VALUE.fullmatch(value) if isinstance(value, str) else None
    if match is None or tag.get("allocation") != "assigned":
        raise ValueError("standard attribute does not reference an assigned numeric tag")
    return int(match.group(1), 16)


def _standard_policies(catalog: dict[str, Any]) -> list[dict[str, Any]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    tags: dict[str, dict[str, Any]] = {}
    attributes: list[dict[str, Any]] = []
    vendor_attributes: list[dict[str, Any]] = []
    for element in elements:
        if not isinstance(element, dict):
            raise ValueError("catalog element is malformed")
        if element.get("kind") == "tag":
            element_id = element.get("element_id")
            if isinstance(element_id, str):
                if element_id in tags:
                    raise ValueError("catalog contains a duplicate tag identifier")
                tags[element_id] = element
        elif element.get("kind") == "attribute":
            if element.get("name") == "Vendor Attribute":
                vendor_attributes.append(element)
            elif "source_policy_table" in element:
                attributes.append(element)

    if len(attributes) != _SOURCE_POLICY_COUNT:
        raise ValueError("catalog must contain exactly 62 standard attribute policies")
    if len(vendor_attributes) != 1:
        raise ValueError("catalog must contain exactly one Vendor Attribute policy")

    policies: list[dict[str, Any]] = []
    seen_values: set[int] = set()
    for attribute in attributes:
        parents = attribute.get("parent_element_ids")
        if attribute.get("element_id") == _CERTIFICATE_ATTRIBUTES_ID:
            if (
                parents != []
                or attribute.get("source_policy_table") != "Table 40"
                or not any(ref.get("section") == "4.6" for ref in attribute.get("source_refs", []))
            ):
                raise ValueError("Certificate Attributes catalog-only metadata is inconsistent")
            _required_text(attribute, "source_initially_set_by")
            _required_text(attribute, "source_modifiable_by_client")
            _required_text(attribute, "source_deletable_by_client")
            _required_text(attribute, "source_always_required")
            _policy_rules(attribute, "source_operation_restrictions")
            _policy_rules(attribute, "source_conditional_rules")
            continue
        if not isinstance(parents, list) or len(parents) != 1 or not isinstance(parents[0], str):
            raise ValueError("standard attribute must reference exactly one assigned Item tag")
        tag = tags.get(parents[0])
        if tag is None or tag.get("kind") != "tag":
            raise ValueError("standard attribute references a missing Item tag")
        numeric_tag = _tag_value(tag)
        if numeric_tag in seen_values:
            raise ValueError("standard attribute policies contain a duplicate numeric tag")
        seen_values.add(numeric_tag)

        policy = {
            "tag": numeric_tag,
            "element_id": _required_text(attribute, "element_id"),
            "name": _required_text(attribute, "name"),
            "source_refs": _source_refs(attribute),
            "source_policy_table": _required_text(attribute, "source_policy_table"),
            "source_initially_set_by": _required_text(attribute, "source_initially_set_by"),
            "source_modifiable_by_client": _required_text(attribute, "source_modifiable_by_client"),
            "source_deletable_by_client": _required_text(attribute, "source_deletable_by_client"),
            "source_always_required": _required_text(attribute, "source_always_required"),
            "source_operation_restrictions": _policy_rules(attribute, "source_operation_restrictions"),
            "source_conditional_rules": _policy_rules(attribute, "source_conditional_rules"),
        }
        policies.append(policy)

    if len(policies) != _RUNTIME_POLICY_COUNT:
        raise ValueError("catalog must contain exactly 61 tag-addressable standard attribute policies")
    policies.sort(key=lambda policy: policy["tag"])
    return policies


def _vendor_policy(catalog: dict[str, Any]) -> dict[str, Any]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")
    matches = [
        element
        for element in elements
        if isinstance(element, dict)
        and element.get("kind") == "attribute"
        and element.get("name") == "Vendor Attribute"
    ]
    if len(matches) != 1:
        raise ValueError("catalog must contain exactly one Vendor Attribute policy")
    source = matches[0].get("source_value_policies")
    if not isinstance(source, list) or len(source) != 1 or not isinstance(source[0], dict):
        raise ValueError("Vendor Attribute must contain exactly one value policy")
    source = source[0]
    predicate = source.get("value_predicate")
    if not isinstance(predicate, dict):
        raise ValueError("Vendor Attribute value predicate is malformed")
    operations = source.get("prohibited_client_operations")
    if not isinstance(operations, list) or not operations or any(not isinstance(item, str) for item in operations):
        raise ValueError("Vendor Attribute prohibited operations must be a non-empty text list")
    equals = _required_text(predicate, "equals")
    if equals != "y":
        raise ValueError("Vendor Attribute policy must retain the source-backed y predicate")
    return {
        "source_refs": _source_refs(source),
        "structure_table": _required_text(source, "structure_table"),
        "source_text": _required_text(source, "source_text"),
        "member_element_id": _required_text(predicate, "member_element_id"),
        "equals": equals,
        "source_indicates_origin": _required_text(predicate, "source_indicates_origin"),
        "prohibited_client_operations": operations,
    }


def _render_source_refs(refs: list[dict[str, str]]) -> list[str]:
    return [
        "SourceRef { source_id: "
        f"{_rust_string(ref['source_id'])}, section: {_rust_string(ref['section'])} }},"
        for ref in refs
    ]


def _render_rules(field: str, rules: list[dict[str, Any]]) -> list[str]:
    lines = [f"        {field}: &["]
    for rule in rules:
        lines.append("            SourceRule {")
        lines.append(f"                source_text: {_rust_string(rule['source_text'])},")
        lines.append("                source_refs: &[")
        lines.extend(f"                    {line}" for line in _render_source_refs(rule["source_refs"]))
        lines.append("                ],")
        lines.append("            },")
    lines.append("        ],")
    return lines


def render_rust(catalog: dict[str, Any]) -> str:
    """Render deterministic, source-preserving private policy metadata."""
    policies = _standard_policies(catalog)
    vendor = _vendor_policy(catalog)
    lines = [
        "// @generated by tools/normative_catalog/generate_attribute_policy.py; do not edit.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(super) struct SourceRef {",
        "    pub(super) source_id: &'static str,",
        "    pub(super) section: &'static str,",
        "}",
        "",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(super) struct SourceRule {",
        "    pub(super) source_text: &'static str,",
        "    pub(super) source_refs: &'static [SourceRef],",
        "}",
        "",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(super) struct AttributePolicy {",
        "    pub(super) tag: u32,",
        "    pub(super) element_id: &'static str,",
        "    pub(super) name: &'static str,",
        "    pub(super) source_refs: &'static [SourceRef],",
        "    pub(super) source_policy_table: &'static str,",
        "    pub(super) source_initially_set_by: &'static str,",
        "    pub(super) source_modifiable_by_client: &'static str,",
        "    pub(super) source_deletable_by_client: &'static str,",
        "    pub(super) source_always_required: &'static str,",
        "    pub(super) source_operation_restrictions: &'static [SourceRule],",
        "    pub(super) source_conditional_rules: &'static [SourceRule],",
        "}",
        "",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(super) struct VendorAttributePolicy {",
        "    pub(super) source_refs: &'static [SourceRef],",
        "    pub(super) structure_table: &'static str,",
        "    pub(super) source_text: &'static str,",
        "    pub(super) member_element_id: &'static str,",
        "    pub(super) equals: &'static str,",
        "    pub(super) source_indicates_origin: &'static str,",
        "    pub(super) prohibited_client_operations: &'static [&'static str],",
        "}",
        "",
        "impl VendorAttributePolicy {",
        "    pub(super) fn matches_vendor_identification(self, value: &str) -> bool {",
        "        value == self.equals",
        "    }",
        "}",
        "",
        "#[rustfmt::skip]",
        "pub(super) const ATTRIBUTE_POLICIES: &[AttributePolicy] = &[",
    ]
    for policy in policies:
        lines.extend(
            [
                "    AttributePolicy {",
                f"        tag: 0x{policy['tag'] >> 16:02X}_{(policy['tag'] >> 8) & 0xFF:02X}_{policy['tag'] & 0xFF:02X},",
                f"        element_id: {_rust_string(policy['element_id'])},",
                f"        name: {_rust_string(policy['name'])},",
                "        source_refs: &[",
            ]
        )
        lines.extend(f"            {line}" for line in _render_source_refs(policy["source_refs"]))
        lines.extend(
            [
                "        ],",
                f"        source_policy_table: {_rust_string(policy['source_policy_table'])},",
                f"        source_initially_set_by: {_rust_string(policy['source_initially_set_by'])},",
                f"        source_modifiable_by_client: {_rust_string(policy['source_modifiable_by_client'])},",
                f"        source_deletable_by_client: {_rust_string(policy['source_deletable_by_client'])},",
                f"        source_always_required: {_rust_string(policy['source_always_required'])},",
            ]
        )
        lines.extend(_render_rules("source_operation_restrictions", policy["source_operation_restrictions"]))
        lines.extend(_render_rules("source_conditional_rules", policy["source_conditional_rules"]))
        lines.append("    },")
    lines.extend(
        [
            "];",
            "",
            "#[rustfmt::skip]",
            "pub(super) const VENDOR_ATTRIBUTE_POLICY: VendorAttributePolicy = VendorAttributePolicy {",
        ]
    )
    lines.extend(["    source_refs: &["])
    lines.extend(f"        {line}" for line in _render_source_refs(vendor["source_refs"]))
    lines.extend(
        [
            "    ],",
            f"    structure_table: {_rust_string(vendor['structure_table'])},",
            f"    source_text: {_rust_string(vendor['source_text'])},",
            f"    member_element_id: {_rust_string(vendor['member_element_id'])},",
            f"    equals: {_rust_string(vendor['equals'])},",
            f"    source_indicates_origin: {_rust_string(vendor['source_indicates_origin'])},",
            "    prohibited_client_operations: &[",
        ]
    )
    lines.extend(f"        {_rust_string(operation)}," for operation in vendor["prohibited_client_operations"])
    lines.extend(["    ],", "};", ""])
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
        catalog = load_validated_catalog(raw_catalog, root, require_complete=True)
        generated = render_rust(catalog).encode("utf-8")
        if arguments.write:
            atomic_write_bytes(root, OUTPUT_PATH, generated)
            print(f"generated attribute policies: {root / OUTPUT_PATH}")
            return 0
        existing = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_BYTES)
    except CatalogValidationError:
        print("attribute-policy generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except (PathSecurityError, OSError):
        print("attribute-policy generation failed: repository path or I/O validation failed", file=sys.stderr)
        return 2
    except (TypeError, ValueError):
        print("attribute-policy generation failed: catalog policy data is invalid", file=sys.stderr)
        return 2

    if existing != generated:
        print("generated attribute policies are stale; run generate_attribute_policy.py --write", file=sys.stderr)
        return 1
    print(f"generated attribute policies verified: {root / OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
