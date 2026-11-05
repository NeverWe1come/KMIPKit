"""Generate four-language named KMIP Enumeration and Bit Mask values.

The checked-in normative catalog is the sole source. `--check` performs no
writes and fails if any generated language surface is missing or stale.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
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
OUTPUTS = {
    "rust": "crates/kmipkit-protocol/src/named_values_generated.rs",
    "c": "bindings/c/include/kmipkit_named_value_helpers_generated.h",
    "java": "bindings/java/src/main/java/org/kmipkit/protocol/KmipValues.java",
    "python": "bindings/python/src/kmipkit/protocol_values.py",
    "traceability": "specs/047-named-protocol-values/reports/named-value-traceability.json",
}
EXPECTED_CATALOG_COUNTS = (64, 723, 3, 44)
EXPECTED_ASSIGNED_COUNTS = (644, 33)


@dataclass(frozen=True)
class NamedValue:
    element_id: str
    name: str
    identifier: str
    value: int
    source_refs: tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class NamedGroup:
    element_id: str
    name: str
    identifier: str
    values: tuple[NamedValue, ...]


@dataclass(frozen=True)
class CatalogValues:
    enumerations: tuple[NamedGroup, ...]
    bitmasks: tuple[NamedGroup, ...]


def _identifier(value: str) -> str:
    # KMIP names include superscript numerals (for example, 2⁸ and 2¹⁶).
    # Compatibility normalization makes their numeric identity part of the
    # generated symbol instead of collapsing distinct names to the same one.
    normalized = unicodedata.normalize("NFKD", value)
    ascii_name = normalized.encode("ascii", "ignore").decode("ascii")
    words = re.findall(r"[A-Za-z0-9]+", ascii_name)
    if not words:
        raise ValueError("catalog name has no identifier characters")
    result = "_".join(word.upper() for word in words)
    if result[0].isdigit():
        result = f"VALUE_{result}"
    return result


def _snake_identifier(value: str) -> str:
    return _identifier(value).lower()


def _type_identifier(value: str) -> str:
    words = _identifier(value).split("_")
    result = "".join(word.title() for word in words)
    return f"Value{result}" if result[0].isdigit() else result


def _rust_u32_literal(value: int) -> str:
    digits = f"{value:08X}"
    return f"0x{digits[:4]}_{digits[4:]}"


def _c_string(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def _wire_integer(element: dict[str, Any]) -> int:
    raw = element.get("wire_value")
    if not isinstance(raw, str) or not re.fullmatch(r"(?:0x)?[0-9A-Fa-f]{1,8}", raw):
        raise ValueError("catalog value has malformed unsigned 32-bit wire value")
    value = int(raw.removeprefix("0x").removeprefix("0X"), 16)
    if not 0 <= value <= 0xFFFF_FFFF:
        raise ValueError("catalog value is outside the unsigned 32-bit range")
    return value


def collect_values(catalog: dict[str, Any]) -> CatalogValues:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")
    groups: dict[str, dict[str, Any]] = {}
    for element in elements:
        if isinstance(element, dict) and element.get("kind") in {"enumeration", "bitmask"}:
            element_id = element.get("element_id")
            if not isinstance(element_id, str) or element_id in groups:
                raise ValueError("catalog has a missing or duplicate named-value group identity")
            groups[element_id] = element

    children: dict[str, list[dict[str, Any]]] = {key: [] for key in groups}
    catalog_value_counts = {"enumeration": 0, "bitmask": 0}
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") not in {"enumeration_value", "bitmask_value"}:
            continue
        parents = element.get("parent_element_ids")
        if not isinstance(parents, list) or len(parents) != 1 or parents[0] not in groups:
            raise ValueError("catalog named value has an invalid parent identity")
        parent = groups[parents[0]]
        expected_kind = "enumeration_value" if parent["kind"] == "enumeration" else "bitmask_value"
        if element["kind"] != expected_kind:
            raise ValueError("catalog named value kind disagrees with its parent")
        catalog_value_counts[parent["kind"]] += 1
        if element.get("allocation") != "assigned":
            continue
        children[parents[0]].append(element)

    result: dict[str, list[NamedGroup]] = {"enumeration": [], "bitmask": []}
    group_identifier_owners: dict[str, str] = {}
    for group_id, group in groups.items():
        group_name = group.get("name")
        if not isinstance(group_name, str) or not group_name:
            raise ValueError("catalog named-value group has no name")
        group_identifier = _identifier(group_name)
        if group_identifier in group_identifier_owners:
            raise ValueError(
                "duplicate generated group identifier for "
                f"{group_identifier_owners[group_identifier]!r} and {group_name!r}"
            )
        group_identifier_owners[group_identifier] = group_name
        _snake_identifier(group_name)
        _type_identifier(group_name)
        seen_names: set[str] = set()
        seen_values: set[int] = set()
        named: list[NamedValue] = []
        for element in children[group_id]:
            name = element.get("name")
            element_id = element.get("element_id")
            source_refs = element.get("source_refs")
            if (
                not isinstance(name, str)
                or not name
                or not isinstance(element_id, str)
                or not isinstance(source_refs, list)
                or not source_refs
                or any(
                    not isinstance(reference, dict)
                    or not isinstance(reference.get("source_id"), str)
                    or not isinstance(reference.get("section"), str)
                    for reference in source_refs
                )
            ):
                raise ValueError("catalog assigned named value lacks a stable name or identity")
            identifier = _identifier(name)
            numeric = _wire_integer(element)
            if identifier in seen_names:
                raise ValueError(f"catalog contains duplicate assigned name in group {group_name}")
            if numeric in seen_values:
                raise ValueError(f"catalog contains duplicate assigned numeric value in group {group_name}")
            if group["kind"] == "bitmask" and numeric == 0:
                raise ValueError("catalog assigned bit mask value must contain at least one bit")
            seen_names.add(identifier)
            seen_values.add(numeric)
            named.append(
                NamedValue(
                    element_id,
                    name,
                    identifier,
                    numeric,
                    tuple(
                        (reference["source_id"], reference["section"])
                        for reference in source_refs
                    ),
                )
            )
        named.sort(key=lambda item: (item.value, item.identifier, item.element_id))
        result[group["kind"]].append(NamedGroup(group_id, group_name, group_identifier, tuple(named)))

    for values in result.values():
        values.sort(key=lambda item: (item.identifier, item.element_id))
    values = CatalogValues(tuple(result["enumeration"]), tuple(result["bitmask"]))
    catalog_counts = (
        len(values.enumerations),
        catalog_value_counts["enumeration"],
        len(values.bitmasks),
        catalog_value_counts["bitmask"],
    )
    if catalog_counts != EXPECTED_CATALOG_COUNTS:
        raise ValueError(f"catalog named-value counts differ from audited catalog: {catalog_counts}")
    assigned_counts = (
        sum(len(group.values) for group in values.enumerations),
        sum(len(group.values) for group in values.bitmasks),
    )
    if assigned_counts != EXPECTED_ASSIGNED_COUNTS:
        raise ValueError(f"assigned named-value counts differ from normative assignments: {assigned_counts}")
    return values


def _render_rust(values: CatalogValues) -> str:
    lines = ["//! Generated catalog-backed KMIP 2.1 named values.", "", "/// KMIP protocol Enumeration and Bit Mask values.", "pub mod values {"]
    kinds = (("enumeration", values.enumerations), ("bitmask", values.bitmasks))
    for kind_index, (kind, groups) in enumerate(kinds):
        lines.append(f"    /// Assigned KMIP 2.1 {kind.title()} values.")
        lines.extend([f"    pub mod {kind} {{"])
        for group_index, group in enumerate(groups):
            lines.append(f"        /// Values for the KMIP {group.name} {kind.title()}.")
            lines.extend([f"        pub mod {_snake_identifier(group.name)} {{"])
            for value in group.values:
                source = ", ".join(
                    f"{source_id} §{section}" for source_id, section in value.source_refs
                )
                lines.append(f"            /// `{value.name}`; {value.element_id}; {source}.")
                lines.append(
                    f"            pub const {value.identifier}: u32 = {_rust_u32_literal(value.value)};"
                )
            if kind == "enumeration":
                raw_parameter = "raw" if group.values else "_raw"
                lines.extend([
                    "            /// Returns the assigned catalog name for a known raw value.",
                    "            #[must_use]",
                    f"            pub const fn name({raw_parameter}: u32) -> Option<&'static str> {{",
                ])
                if group.values:
                    lines.append("                match raw {")
                    for value in group.values:
                        lines.append(
                            f"                    {_rust_u32_literal(value.value)} => Some({json.dumps(value.name, ensure_ascii=False)}),"
                        )
                    lines.extend(["                    _ => None,", "                }"])
                else:
                    lines.append("                None")
                lines.append("            }")
            else:
                mask = 0
                for value in group.values:
                    mask |= value.value
                lines.extend([
                    "            /// Mask containing every assigned bit in this group.",
                    f"            pub const KNOWN_MASK: u32 = {_rust_u32_literal(mask)};",
                    "            /// Returns only the bits assigned by KMIP 2.1.",
                    "            #[must_use]",
                    "            pub const fn known_bits(raw: u32) -> u32 {",
                    "                raw & KNOWN_MASK",
                    "            }",
                    "            /// Returns all raw bits not assigned by KMIP 2.1.",
                    "            #[must_use]",
                    "            pub const fn unknown_bits(raw: u32) -> u32 {",
                    "                raw & !KNOWN_MASK",
                    "            }",
                    "            /// Returns assigned bit names in ascending numeric order.",
                    "            pub fn names(raw: u32) -> impl Iterator<Item = &'static str> {",
                    "                [",
                    *(
                        f"                    ({_rust_u32_literal(value.value)}, {json.dumps(value.name, ensure_ascii=False)}),"
                        for value in group.values
                    ),
                    "                ]",
                    "                .into_iter()",
                    "                .filter_map(move |(bit, name)| (raw & bit != 0).then_some(name))",
                    "            }",
                ])
            lines.append("        }")
            if group_index + 1 < len(groups):
                lines.append("")
        lines.append("    }")
        if kind_index + 1 < len(kinds):
            lines.append("")
    lines.extend(["}", ""])
    return "\n".join(lines)


def _render_c(values: CatalogValues) -> str:
    lines = [
        "/* Generated KMIP 2.1 named-value lookup helpers. */",
        "#ifndef KMIPKIT_NAMED_VALUE_HELPERS_GENERATED_H",
        "#define KMIPKIT_NAMED_VALUE_HELPERS_GENERATED_H",
        "",
        "#include <stdint.h>",
        "",
        "/* Opaque namespace marker for the KMIP named-value header surface. */",
        "typedef struct kmipkit_named_values_namespace_t kmipkit_named_values_namespace_t;",
    ]
    for kind, groups in (("enumeration", values.enumerations), ("bitmask", values.bitmasks)):
        for group in groups:
            lines.append(
                f"typedef uint32_t kmipkit_{kind}_{_snake_identifier(group.name)}_t;"
            )
    lines.append("")
    for kind, groups in (("ENUMERATION", values.enumerations), ("BITMASK", values.bitmasks)):
        for group in groups:
            prefix = f"KMIPKIT_{kind}_{group.identifier}"
            if kind == "ENUMERATION":
                lines.extend([f"static inline const char *{prefix}_name(uint32_t raw) {{", "    switch (raw) {"])
                for value in group.values:
                    escaped = value.name.replace("\\", "\\\\").replace('"', '\\"')
                    lines.append(f'    case KMIPKIT_ENUMERATION_{group.identifier}_{value.identifier}: return "{escaped}";')
                lines.extend(["    default: return 0;", "    }", "}", ""])
            else:
                lines.extend([
                    f"static inline uint32_t {prefix}_known_bits(uint32_t raw) {{ return raw & {prefix}_KNOWN_MASK; }}",
                    f"static inline uint32_t {prefix}_unknown_bits(uint32_t raw) {{ return raw & ~{prefix}_KNOWN_MASK; }}",
                    f"static inline const char *{prefix}_name(uint32_t bit) {{",
                    "    switch (bit) {",
                ])
                for value in group.values:
                    escaped = value.name.replace("\\", "\\\\").replace('"', '\\"')
                    lines.append(f'    case KMIPKIT_BITMASK_{group.identifier}_{value.identifier}: return "{escaped}";')
                lines.extend([
                    "    default: return 0;",
                    "    }",
                    "}",
                    f"static inline uint32_t {prefix}_names(uint32_t raw, const char **out_names, uint32_t capacity) {{",
                    "    uint32_t count = 0;",
                    "    if (out_names == 0 && capacity != 0) return 0;",
                ])
                for value in group.values:
                    lines.extend([
                        f"    if ((raw & KMIPKIT_BITMASK_{group.identifier}_{value.identifier}) != 0) {{",
                        f"        if (out_names != 0 && count < capacity) out_names[count] = {_c_string(value.name)};",
                        "        ++count;",
                        "    }",
                    ])
                lines.extend(["    return count;", "}", ""])
    lines.extend(["#endif", ""])
    return "\n".join(lines)


def _render_java(values: CatalogValues) -> str:
    lines = ["package org.kmipkit.protocol;", "", "/** Generated KMIP 2.1 named values. */", "public final class KmipValues {", "    private KmipValues() {}", ""]
    for kind, groups in (("Enumeration", values.enumerations), ("BitMask", values.bitmasks)):
        for group in groups:
            type_name = _type_identifier(group.name)
            lines.extend([f"    /** Named KMIP 2.1 values for {group.name}. */", f"    public static final class {type_name} {{", f"        private {type_name}() {{}}"])
            for value in group.values:
                source = ", ".join(
                    f"{source_id} §{section}" for source_id, section in value.source_refs
                )
                lines.append(f"        /** {value.name}; {value.element_id}; {source}. */")
                lines.append(f"        public static final long {value.identifier} = 0x{value.value:08X}L;")
            if kind == "Enumeration":
                lines.extend([
                    "        /** Returns the assigned catalog name, or null for an unknown raw value. */",
                    "        public static String name(long raw) {",
                    "            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;",
                    "            return switch ((int) raw) {",
                ])
                for value in group.values:
                    escaped = value.name.replace("\\", "\\\\").replace('"', '\\"')
                    lines.append(f'                case 0x{value.value:08X} -> "{escaped}";')
                lines.extend(["                default -> null;", "            };", "        }"])
            else:
                mask = 0
                for value in group.values:
                    mask |= value.value
                lines.extend([
                    f"        public static final long KNOWN_MASK = 0x{mask:08X}L;",
                    "        /** Returns the assigned bits without changing the original raw value. */",
                    "        public static long knownBits(long raw) { return raw & KNOWN_MASK; }",
                    "        /** Returns all raw bits not assigned by KMIP 2.1. */",
                    "        public static long unknownBits(long raw) { return raw & ~KNOWN_MASK & 0xFFFF_FFFFL; }",
                    "        /** Returns names for set assigned bits in ascending numeric order. */",
                    "        public static java.util.List<String> names(long raw) {",
                    "            var names = new java.util.ArrayList<String>();",
                ])
                for value in group.values:
                    lines.append(
                        f"            if ((raw & {value.identifier}) != 0) names.add({json.dumps(value.name)});"
                    )
                lines.extend(["            return java.util.List.copyOf(names);", "        }"])
            lines.extend(["    }", ""])
    lines.extend(["}", ""])
    return "\n".join(lines)


def _render_python(values: CatalogValues) -> str:
    lines = ['"""Generated KMIP 2.1 named values."""', "", "from __future__ import annotations", ""]
    for kind, groups in (("enumeration", values.enumerations), ("bitmask", values.bitmasks)):
        for group in groups:
            type_name = _type_identifier(group.name)
            lines.extend([f"class {type_name}:", f'    """Named values for {group.name}."""'])
            for value in group.values:
                lines.append(f"    # {value.element_id}; " + ", ".join(
                    f"{source_id} §{section}" for source_id, section in value.source_refs
                ))
                lines.append(f"    {value.identifier} = 0x{value.value:08X}")
            if kind == "enumeration":
                lines.extend(["", "    @staticmethod", "    def name(raw: int) -> str | None:", '        """Return the assigned catalog name, or None for an unknown raw value."""', "        return {"])
                for value in group.values:
                    lines.append(f"            0x{value.value:08X}: {value.name!r},")
                lines.extend(["        }.get(raw)"])
            else:
                mask = 0
                for value in group.values:
                    mask |= value.value
                lines.extend([
                    f"    KNOWN_MASK = 0x{mask:08X}",
                    "",
                    "    @staticmethod",
                    "    def known_bits(raw: int) -> int:",
                    '        """Return only the assigned bits without changing the raw mask."""',
                    f"        return raw & {type_name}.KNOWN_MASK",
                    "",
                    "    @staticmethod",
                    "    def unknown_bits(raw: int) -> int:",
                    '        """Return all bits not assigned by KMIP 2.1."""',
                    f"        return raw & ~{type_name}.KNOWN_MASK & 0xFFFFFFFF",
                    "",
                    "    @staticmethod",
                    "    def names(raw: int) -> tuple[str, ...]:",
                    '        """Return names for set assigned bits in ascending numeric order."""',
                    "        return tuple(",
                    "            name for bit, name in (",
                    *(f"                (0x{value.value:08X}, {value.name!r})," for value in group.values),
                    "            ) if raw & bit",
                    "        )",
                ])
            lines.append("")
    return "\n".join(lines)


def render_outputs(values: CatalogValues) -> dict[str, str]:
    traceability = {
        "formatVersion": 1,
        "specId": "KMIPKIT-0047",
        "catalogPath": CATALOG_PATH,
        "groups": [
            {
                "kind": kind,
                "elementId": group.element_id,
                "name": group.name,
                "identifier": group.identifier,
                "assignedValues": [
                    {
                        "elementId": value.element_id,
                        "name": value.name,
                        "identifier": value.identifier,
                        "wireValue": f"0x{value.value:08X}",
                        "sourceRefs": [
                            {"sourceId": source_id, "section": section}
                            for source_id, section in value.source_refs
                        ],
                    }
                    for value in group.values
                ],
            }
            for kind, groups in (("enumeration", values.enumerations), ("bitmask", values.bitmasks))
            for group in groups
        ],
    }
    return {
        "rust": _render_rust(values),
        "c": _render_c(values),
        "java": _render_java(values),
        "python": _render_python(values),
        "traceability": json.dumps(traceability, ensure_ascii=False, indent=2) + "\n",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args(argv)
    try:
        root = args.repo_root.resolve(strict=True)
        raw = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(raw, root, require_complete=True)
        rendered = render_outputs(collect_values(catalog))
        for language, path in OUTPUTS.items():
            expected = rendered[language].encode("utf-8")
            if args.check:
                current = safe_read_bytes(root, path, max_bytes=MAX_BYTES)
                if current != expected:
                    print(f"generated {language} values are stale: {path}", file=sys.stderr)
                    return 1
            else:
                atomic_write_bytes(root, path, expected)
    except (OSError, PathSecurityError, CatalogValidationError, ValueError) as error:
        print(f"named value generation failed: {error}", file=sys.stderr)
        return 1
    print(f"Named protocol values {'verified' if args.check else 'written'}: 4 language outputs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
