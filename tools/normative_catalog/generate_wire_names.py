"""Generate the private catalog tag-name table used by wire codecs."""

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
OUTPUT_PATH = "crates/kmipkit-client/src/wire_names_generated.rs"
OUTPUT_DATA_PATH = "crates/kmipkit-client/src/wire_names_generated.tsv"
_TAG_VALUE = re.compile(r"(?:0[xX])?[0-9A-Fa-f]{6}\Z")
_TSV_DELIMITER = re.compile(r"[\t\r\n]")


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


def render_rust() -> str:
    """Render the small Rust include wrapper for generated TSV metadata."""
    return (
        "// @generated by tools/normative_catalog/generate_wire_names.py; do not edit.\n"
        'pub(crate) const WIRE_NAMES: &str = include_str!("wire_names_generated.tsv");\n'
    )


def render_tsv(catalog: dict[str, Any]) -> str:
    """Render deterministic assigned tag, Enumeration, and family records."""
    records: list[str] = []
    for value, name in _assigned_tag_names(catalog):
        records.append(f"T\t{value:06X}\t{_tsv_field(name)}")
    for tag, family in _tag_enumeration_families(catalog):
        records.append(f"F\t{tag:06X}\t{_tsv_field(family)}")
    for family, value, name in _assigned_enumeration_names(catalog):
        records.append(f"E\t{_tsv_field(family)}\t{value:08X}\t{_tsv_field(name)}")
    return "\n".join(records) + "\n"


def _tsv_field(value: str) -> str:
    if _TSV_DELIMITER.search(value):
        raise ValueError("catalog wire-name field contains a TSV delimiter")
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the generated Rust include and TSV metadata")
    mode.add_argument("--check", action="store_true", help="fail if generated wire-name metadata is stale")
    arguments = parser.parse_args(argv)

    try:
        root = arguments.repo_root.resolve(strict=True)
        catalog_bytes = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(catalog_bytes, root)
        generated_source = render_rust().encode("utf-8")
        generated_data = render_tsv(catalog).encode("utf-8")
        if arguments.write:
            atomic_write_bytes(root, OUTPUT_DATA_PATH, generated_data)
            atomic_write_bytes(root, OUTPUT_PATH, generated_source)
            print(f"generated wire-name metadata: {root / OUTPUT_DATA_PATH}")
            return 0
        existing_source = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_BYTES)
        existing_data = safe_read_bytes(root, OUTPUT_DATA_PATH, max_bytes=MAX_BYTES)
    except CatalogValidationError:
        print("wire-name generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except (PathSecurityError, OSError):
        print("wire-name generation failed: repository path or I/O failure", file=sys.stderr)
        return 2
    except ValueError:
        print("wire-name generation failed: catalog wire-name data is invalid", file=sys.stderr)
        return 2

    if existing_source != generated_source or existing_data != generated_data:
        print("generated wire names are stale; run generate_wire_names.py --write", file=sys.stderr)
        return 1
    print(f"generated wire-name metadata verified: {root / OUTPUT_DATA_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
