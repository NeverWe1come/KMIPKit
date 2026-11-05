"""Generate the private catalog tag-name table used by wire codecs."""

from __future__ import annotations

import argparse
import json
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
OUTPUT_PATH = "crates/kmipkit-client/src/wire_names_generated.rs"
_TAG_VALUE = re.compile(r"(?:0[xX])?[0-9A-Fa-f]{6}\Z")


def _assigned_tag_names(catalog: dict[str, Any]) -> list[tuple[int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    names: list[tuple[int, str]] = []
    seen_values: set[int] = set()
    seen_names: set[str] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "tag":
            continue
        if element.get("allocation") != "assigned":
            continue
        raw_value = element.get("wire_value")
        name = element.get("name")
        if not isinstance(raw_value, str) or not _TAG_VALUE.fullmatch(raw_value):
            raise ValueError("catalog tag has an invalid numeric value")
        value = int(raw_value.removeprefix("0x").removeprefix("0X"), 16)
        if not isinstance(name, str) or not name:
            raise ValueError("catalog tag name or value is invalid")
        if value in seen_values or name in seen_names:
            raise ValueError("catalog contains duplicate assigned tag names or values")
        seen_values.add(value)
        seen_names.add(name)
        names.append((value, name))
    if not names:
        raise ValueError("catalog has no assigned tag names")
    return sorted(names)


def _assigned_enumeration_names(catalog: dict[str, Any]) -> list[tuple[str, int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    enumerations: dict[str, str] = {}
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "enumeration":
            continue
        identifier = element.get("element_id")
        name = element.get("name")
        if (
            not isinstance(identifier, str)
            or not identifier
            or not isinstance(name, str)
            or not name
        ):
            raise ValueError("catalog enumeration name or identifier is invalid")
        if identifier in enumerations:
            raise ValueError("catalog contains duplicate enumeration identifiers")
        enumerations[identifier] = name

    names: list[tuple[str, int, str]] = []
    seen_values: set[tuple[str, int]] = set()
    seen_names: set[tuple[str, str]] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "enumeration_value":
            continue
        if element.get("allocation") != "assigned":
            continue
        parents = element.get("parent_element_ids")
        raw_value = element.get("wire_value")
        name = element.get("name")
        if (
            not isinstance(parents, list)
            or len(parents) != 1
            or parents[0] not in enumerations
        ):
            raise ValueError("catalog enumeration value has no unique enumeration family")
        if not isinstance(raw_value, str) or not re.fullmatch(
            r"[0-9A-Fa-f]{8}", raw_value
        ):
            raise ValueError("catalog enumeration value is not an assigned 32-bit value")
        if not isinstance(name, str) or not name:
            raise ValueError("catalog enumeration value name is invalid")
        family = enumerations[parents[0]]
        value = int(raw_value, 16)
        if (family, value) in seen_values or (family, name) in seen_names:
            raise ValueError("catalog contains ambiguous enumeration names or values")
        seen_values.add((family, value))
        seen_names.add((family, name))
        names.append((family, value, name))
    return sorted(names)


def _tag_enumeration_families(catalog: dict[str, Any]) -> list[tuple[int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    families = [
        element.get("name")
        for element in elements
        if isinstance(element, dict) and element.get("kind") == "enumeration"
    ]
    if not all(isinstance(name, str) and name for name in families):
        raise ValueError("catalog enumeration family is invalid")
    tags_by_name = dict((name, value) for value, name in _assigned_tag_names(catalog))
    if len(set(families)) != len(families):
        raise ValueError("catalog contains ambiguous Enumeration family names")
    family_by_tag: dict[int, str] = {}
    for member in elements:
        if (
            not isinstance(member, dict)
            or member.get("kind") not in {"structure_member", "message_field", "operation_structure"}
        ):
            continue
        if member.get("source_encoding") != "Enumeration":
            continue
        member_name = member.get("name")
        if not isinstance(member_name, str) or member_name not in tags_by_name:
            raise ValueError("catalog Enumeration member has no matching assigned tag")
        matching_families = [name for name in families if member_name.endswith(name)]
        if not matching_families:
            raise ValueError("catalog Enumeration member has no matching family")
        family = max(matching_families, key=len)
        tag = tags_by_name[member_name]
        family_by_tag.setdefault(tag, family)
    return sorted((tag, family) for tag, family in family_by_tag.items())


def _rust_tuple(*fields: str) -> list[str]:
    inline = f"    ({', '.join(fields)}),"
    # Match rustfmt's default tuple-call width after punctuation overhead.
    if sum(map(len, fields)) <= 56:
        return [inline]
    return ["    (", *(f"        {field}," for field in fields), "    ),"]


def render_rust(catalog: dict[str, Any]) -> str:
    """Render deterministic assigned tag and Enumeration names."""
    lines = [
        "// @generated by tools/normative_catalog/generate_wire_names.py; do not edit.",
        "pub(crate) const TAG_NAMES: &[(u32, &str)] = &[",
    ]
    for value, name in _assigned_tag_names(catalog):
        tag = f"0x{value >> 16:02X}_{(value >> 8) & 0xFF:02X}_{value & 0xFF:02X}"
        escaped_name = json.dumps(name, ensure_ascii=False)
        lines.extend(_rust_tuple(tag, escaped_name))
    lines.extend(["];", "", "pub(crate) const TAG_ENUMERATION_FAMILIES: &[(u32, &str)] = &["])
    for tag, family in _tag_enumeration_families(catalog):
        escaped_family = json.dumps(family, ensure_ascii=False)
        tag_value = f"0x{tag >> 16:02X}_{(tag >> 8) & 0xFF:02X}_{tag & 0xFF:02X}"
        lines.extend(_rust_tuple(tag_value, escaped_family))
    lines.extend(["];", "", "pub(crate) const ENUMERATION_NAMES: &[(&str, u32, &str)] = &["])
    for family, value, name in _assigned_enumeration_names(catalog):
        escaped_family = json.dumps(family, ensure_ascii=False)
        escaped_name = json.dumps(name, ensure_ascii=False)
        enum_value = (
            f"0x{value >> 24:02X}_{(value >> 16) & 0xFF:02X}_"
            f"{(value >> 8) & 0xFF:02X}_{value & 0xFF:02X}"
        )
        lines.extend(_rust_tuple(escaped_family, enum_value, escaped_name))
    lines.extend(["];", ""])
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
        catalog_bytes = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(catalog_bytes, root)
        generated = render_rust(catalog).encode("utf-8")
        if arguments.write:
            atomic_write_bytes(root, OUTPUT_PATH, generated)
            print(f"generated wire names: {root / OUTPUT_PATH}")
            return 0
        existing = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_BYTES)
    except CatalogValidationError:
        print("wire-name generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except (PathSecurityError, OSError):
        print("wire-name generation failed: repository path or I/O failure", file=sys.stderr)
        return 2
    except ValueError:
        print("wire-name generation failed: catalog wire-name data is invalid", file=sys.stderr)
        return 2

    if existing != generated:
        print("generated wire names are stale; run generate_wire_names.py --write", file=sys.stderr)
        return 1
    print(f"generated wire names verified: {root / OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
