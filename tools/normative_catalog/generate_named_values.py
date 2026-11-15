"""Generate four-language named KMIP Enumeration and Bit Mask values.

The checked-in normative catalog is the sole source. `--check` performs no
writes and fails if any generated language surface is missing or stale.
"""

from __future__ import annotations

import argparse
import hashlib
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
VERSIONED_CATALOG_PATH = "specification/catalog/kmip-versioned-values.json"
VERSIONED_SOURCE_MAX_BYTES = 24 * 1024 * 1024
PINNED_VERSION_SOURCES = {
    "kmip-2.0": (
        "specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html",
        "41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2",
    ),
    "kmip-2.1": (
        "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html",
        "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf",
    ),
    "kmip-3.0": (
        "specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html",
        "a7d5e58e63acd9be33fab08b76039fc8db4e072fdbdf4a280ea5b743bae5682b",
    ),
}
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


@dataclass(frozen=True)
class VersionedNamedValue:
    name: str
    identifier: str
    value: int


@dataclass(frozen=True)
class VersionedNamedGroup:
    version: str
    kind: str
    name: str
    identifier: str
    source_id: str
    section: str
    table: str
    values: tuple[VersionedNamedValue, ...]


@dataclass(frozen=True)
class VersionedCatalog:
    groups: tuple[VersionedNamedGroup, ...]


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
    # Fixed-width octal byte escapes avoid C's greedy hex escapes, trigraphs,
    # source-encoding differences, and literal control characters.
    escaped_bytes: list[str] = []
    for byte in value.encode("utf-8"):
        if 0x20 <= byte <= 0x7E and byte not in {0x22, 0x3F, 0x5C}:
            escaped_bytes.append(chr(byte))
        else:
            escaped_bytes.append(f"\\{byte:03o}")
    return f'"{"".join(escaped_bytes)}"'


def _render_c_name_case(kind: str, group: NamedGroup, value: NamedValue) -> str:
    return (
        f"    case KMIPKIT_{kind}_{group.identifier}_{value.identifier}: "
        f"return {_c_string(value.name)};"
    )


def _source_provenance(source_refs: tuple[tuple[str, str], ...]) -> str:
    return ", ".join(f"{source_id} §{section}" for source_id, section in source_refs)


def _wire_integer(element: dict[str, Any]) -> int:
    raw = element.get("wire_value")
    if not isinstance(raw, str) or not re.fullmatch(r"(?:0x)?[0-9A-Fa-f]{1,8}", raw):
        raise ValueError("catalog value has malformed unsigned 32-bit wire value")
    value = int(raw.removeprefix("0x").removeprefix("0X"), 16)
    if not 0 <= value <= 0xFFFF_FFFF:
        raise ValueError("catalog value is outside the unsigned 32-bit range")
    return value


def load_versioned_values(root: Path) -> VersionedCatalog:
    """Load version deltas only after verifying their pinned local sources."""
    raw = safe_read_bytes(root, VERSIONED_CATALOG_PATH, max_bytes=MAX_BYTES)
    document = json.loads(raw)
    if not isinstance(document, dict) or document.get("formatVersion") != 1:
        raise ValueError("versioned values catalog has an unsupported format")
    sources = document.get("sources")
    if not isinstance(sources, dict) or set(sources) != set(PINNED_VERSION_SOURCES):
        raise ValueError("versioned values catalog source inventory is incomplete")
    for source_id, (expected_path, expected_hash) in PINNED_VERSION_SOURCES.items():
        entry = sources[source_id]
        if not isinstance(entry, dict) or entry.get("path") != expected_path:
            raise ValueError(f"versioned values catalog has an invalid {source_id} path")
        if entry.get("sha256") != expected_hash:
            raise ValueError(f"versioned values catalog has an invalid {source_id} checksum")
        source_bytes = safe_read_bytes(root, expected_path, max_bytes=VERSIONED_SOURCE_MAX_BYTES)
        if hashlib.sha256(source_bytes).hexdigest() != expected_hash:
            raise ValueError(f"pinned source checksum does not match for {source_id}")

    versions = document.get("versions")
    if not isinstance(versions, dict) or set(versions) != {"v2_1", "v3_0"}:
        raise ValueError("versioned values catalog must contain only the reviewed v2_1 and v3_0 deltas")
    groups: list[VersionedNamedGroup] = []
    for version in ("v2_1", "v3_0"):
        source_id = "kmip-2.1" if version == "v2_1" else "kmip-3.0"
        raw_groups = versions[version]
        if not isinstance(raw_groups, list):
            raise ValueError(f"{version} versioned groups must be an array")
        seen_groups: set[tuple[str, str]] = set()
        for raw_group in raw_groups:
            if not isinstance(raw_group, dict):
                raise ValueError(f"{version} versioned group must be an object")
            kind = raw_group.get("kind")
            name = raw_group.get("name")
            reference = raw_group.get("sourceId")
            section = raw_group.get("section")
            table = raw_group.get("table")
            raw_values = raw_group.get("values")
            if (
                kind != "enumeration"
                or not isinstance(name, str)
                or not name
                or reference != source_id
                or not isinstance(section, str)
                or not re.fullmatch(r"\d+(?:\.\d+)*", section)
                or not isinstance(table, str)
                or not table.isdigit()
                or not isinstance(raw_values, list)
                or not raw_values
            ):
                raise ValueError(f"{version} versioned group metadata is malformed")
            identity = (kind, name)
            if identity in seen_groups:
                raise ValueError(f"{version} contains a duplicate {name} group")
            seen_groups.add(identity)
            values: list[VersionedNamedValue] = []
            seen_names: set[str] = set()
            seen_numbers: set[int] = set()
            for raw_value in raw_values:
                if not isinstance(raw_value, dict):
                    raise ValueError(f"{version} {name} entry must be an object")
                value_name = raw_value.get("name")
                if not isinstance(value_name, str) or not value_name:
                    raise ValueError(f"{version} {name} value has no name")
                identifier = _identifier(value_name)
                numeric = _wire_integer(raw_value)
                if identifier in seen_names or numeric in seen_numbers:
                    raise ValueError(f"{version} {name} has a duplicate value name or assignment")
                seen_names.add(identifier)
                seen_numbers.add(numeric)
                values.append(VersionedNamedValue(value_name, identifier, numeric))
            values.sort(key=lambda item: (item.value, item.identifier))
            groups.append(
                VersionedNamedGroup(
                    version,
                    kind,
                    name,
                    _snake_identifier(name),
                    source_id,
                    section,
                    table,
                    tuple(values),
                )
            )
    return VersionedCatalog(tuple(groups))


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
                source = _source_provenance(value.source_refs)
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
                    lines.append(_render_c_name_case(kind, group, value))
                lines.extend(["    default: return 0;", "    }", "}", ""])
            else:
                lines.extend([
                    f"static inline uint32_t {prefix}_known_bits(uint32_t raw) {{ return raw & {prefix}_KNOWN_MASK; }}",
                    f"static inline uint32_t {prefix}_unknown_bits(uint32_t raw) {{ return raw & ~{prefix}_KNOWN_MASK; }}",
                    f"static inline const char *{prefix}_name(uint32_t bit) {{",
                    "    switch (bit) {",
                ])
                for value in group.values:
                    lines.append(_render_c_name_case(kind, group, value))
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
                source = _source_provenance(value.source_refs)
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
                lines.append(
                    f"    # {value.element_id}; {_source_provenance(value.source_refs)}"
                )
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


def render_versioned_outputs(versioned: VersionedCatalog) -> dict[str, str]:
    """Render exact version-delta constants and lookup helpers for four languages."""
    rust = ["    /// Exact assigned values added by later KMIP editions.", "    pub mod versions {"]
    java: list[str] = ["    // Exact assigned values added by later KMIP editions."]
    c: list[str] = ["/* Exact assigned values added by later KMIP editions. */"]
    python: list[str] = ["", "# Exact assigned values added by later KMIP editions."]
    for version in ("v2_1", "v3_0"):
        groups = [group for group in versioned.groups if group.version == version]
        rust.extend([f"        /// Values assigned in KMIP {version.replace('_', '.') }.", f"        pub mod {version} {{", "            pub mod enumeration {"])
        for group in groups:
            rust.extend([f"                /// KMIP {group.name} values from §{group.section}, Table {group.table}.", f"                pub mod {group.identifier} {{"])
            for value in group.values:
                rust.append(
                    f"                    /// `{value.name}`; {group.source_id} §{group.section}, Table {group.table}."
                )
                rust.append(
                    f"                    pub const {value.identifier}: u32 = {_rust_u32_literal(value.value)};"
                )
            rust.extend(["                    /// Returns the assigned name for a known value.", "                    #[must_use]", "                    pub const fn name(raw: u32) -> Option<&'static str> {", "                        match raw {"])
            for value in group.values:
                rust.append(
                    f"                            {_rust_u32_literal(value.value)} => Some({json.dumps(value.name, ensure_ascii=False)}),"
                )
            rust.extend(["                            _ => None,", "                        }", "                    }", "                }"])
        rust.extend(["            }", "        }"])
        for group in groups:
            prefix = f"{version.upper()}_{group.identifier.upper()}"
            c.extend(
                [
                    f"static inline const char *KMIPKIT_{prefix}_name(uint32_t raw) {{",
                    "    switch (raw) {",
                ]
            )
            c.extend(
                f"    case UINT32_C(0x{value.value:08X}): return {_c_string(value.name)};"
                for value in group.values
            )
            c.extend(["    default: return 0;", "    }", "}"])

            java_prefix = f"{version.upper()}_{group.identifier.upper()}"
            java.extend(
                f"    /** {value.name}; {group.source_id} §{group.section}, Table {group.table}. */\n    public static final long {java_prefix}_{value.identifier} = 0x{value.value:08X}L;"
                for value in group.values
            )
            java.extend(
                [
                    f"    /** Returns the KMIP {version.replace('_', '.')} {group.name} name, or null for an unassigned value. */",
                    f"    public static String {version}{_type_identifier(group.name)}Name(long raw) {{",
                    "        if (raw < 0 || raw > 0xFFFF_FFFFL) return null;",
                    "        return switch ((int) raw) {",
                ]
            )
            java.extend(
                f"            case 0x{value.value:08X} -> {json.dumps(value.name, ensure_ascii=False)};"
                for value in group.values
            )
            java.extend(["            default -> null;", "        };", "    }"])

            python_prefix = f"{version.upper()}_{group.identifier.upper()}"
            python.extend(
                f"{python_prefix}_{value.identifier} = 0x{value.value:08X}  # {group.source_id} §{group.section}, Table {group.table}"
                for value in group.values
            )
            python.extend(
                [
                    "",
                    f"def {version}_{group.identifier}_name(raw: int) -> str | None:",
                    f'    """Return the assigned KMIP {version.replace("_", ".")} {group.name} name."""',
                    "    return {",
                ]
            )
            python.extend(
                f"        0x{value.value:08X}: {value.name!r}," for value in group.values
            )
            python.extend(["    }.get(raw)", ""])
    rust.append("    }")
    return {
        "rust": "\n".join(rust) + "\n",
        "c": "\n".join(c) + "\n",
        "java": "\n".join(java) + "\n",
        "python": "\n".join(python),
    }


def render_outputs(
    values: CatalogValues, versioned: VersionedCatalog | None = None
) -> dict[str, str]:
    versioned = versioned or VersionedCatalog(())
    versioned_outputs = render_versioned_outputs(versioned)
    traceability = {
        "formatVersion": 1,
        "specId": "KMIPKIT-0047",
        "catalogPath": CATALOG_PATH,
        "versionedCatalogPath": VERSIONED_CATALOG_PATH,
        "versionedGroups": [
            {
                "version": group.version,
                "kind": group.kind,
                "name": group.name,
                "sourceId": group.source_id,
                "section": group.section,
                "table": group.table,
                "assignedValues": [
                    {"name": value.name, "identifier": value.identifier, "wireValue": f"0x{value.value:08X}"}
                    for value in group.values
                ],
            }
            for group in versioned.groups
        ],
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
    rust_base = _render_rust(values)
    c_base = _render_c(values)
    java_base = _render_java(values)
    python_base = _render_python(values)
    if versioned.groups:
        rust_base = rust_base.rsplit("}\n", 1)[0] + versioned_outputs["rust"] + "}\n"
        c_base = c_base.replace("#endif\n", versioned_outputs["c"] + "#endif\n")
        java_base = java_base.rsplit("}\n", 1)[0] + versioned_outputs["java"] + "}\n"
        python_base += versioned_outputs["python"]
    return {
        "rust": rust_base,
        "c": c_base,
        "java": java_base,
        "python": python_base,
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
        versioned = load_versioned_values(root)
        rendered = render_outputs(collect_values(catalog), versioned)
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
