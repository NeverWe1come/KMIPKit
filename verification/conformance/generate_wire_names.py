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
VERSIONED_TAGS_PATH = "specification/catalog/kmip-versioned-tags.json"
VERSIONED_VALUES_PATH = "specification/catalog/kmip-versioned-values.json"
OUTPUT_PATH = "crates/kmipkit-client/src/wire_names_generated.rs"
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


def _assigned_bitmask_names(catalog: dict[str, Any]) -> list[tuple[str, int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    bitmasks: dict[str, str] = {}
    seen_families: set[str] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "bitmask":
            continue
        identifier = element.get("element_id")
        name = element.get("name")
        if (
            not isinstance(identifier, str)
            or not identifier
            or not isinstance(name, str)
            or not name
        ):
            raise ValueError("catalog bitmask family name or identifier is invalid")
        if identifier in bitmasks or name in seen_families:
            raise ValueError("catalog contains duplicate bitmask family names or identifiers")
        bitmasks[identifier] = name
        seen_families.add(name)

    names: list[tuple[str, int, str]] = []
    seen_values: set[tuple[str, int]] = set()
    seen_names: set[tuple[str, str]] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "bitmask_value":
            continue
        if element.get("allocation") != "assigned":
            continue
        parents = element.get("parent_element_ids")
        raw_value = element.get("wire_value")
        name = element.get("name")
        if (
            not isinstance(parents, list)
            or len(parents) != 1
            or parents[0] not in bitmasks
        ):
            raise ValueError("catalog bitmask value has no unique bitmask family")
        if not isinstance(raw_value, str) or not re.fullmatch(r"[0-9A-Fa-f]{8}", raw_value):
            raise ValueError("catalog bitmask value is not an assigned 32-bit value")
        if not isinstance(name, str) or not name:
            raise ValueError("catalog bitmask value name is invalid")
        family = bitmasks[parents[0]]
        value = int(raw_value, 16)
        if (family, value) in seen_values or (family, name) in seen_names:
            raise ValueError("catalog contains ambiguous bitmask names or values")
        seen_values.add((family, value))
        seen_names.add((family, name))
        names.append((family, value, name))
    return sorted(names)


def _record_enumeration_family(
    family_by_tag: dict[int, str], tag: int, family: str
) -> None:
    existing_family = family_by_tag.setdefault(tag, family)
    if existing_family != family:
        raise ValueError("catalog maps one Enumeration tag to multiple families")


def _tag_enumeration_families(catalog: dict[str, Any]) -> list[tuple[int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    enumeration_elements = [
        element
        for element in elements
        if isinstance(element, dict) and element.get("kind") == "enumeration"
    ]
    families = [element.get("name") for element in enumeration_elements]
    if not all(isinstance(name, str) and name for name in families):
        raise ValueError("catalog enumeration family is invalid")
    tags_by_name = dict((name, value) for value, name in _assigned_tag_names(catalog))
    if len(set(families)) != len(families):
        raise ValueError("catalog contains ambiguous Enumeration family names")
    family_by_tag: dict[int, str] = {}

    # Some KMIP fields are Enumeration-typed tags represented directly by a
    # same-named enumeration family, rather than by a structure_member row.
    # The catalog records their type through the generic `Tag` enumeration
    # parent. Use that explicit relationship; do not infer from names alone.
    generic_tag_parents = {
        element.get("element_id")
        for element in enumeration_elements
        if element.get("name") == "Tag"
        and isinstance(element.get("element_id"), str)
        and element.get("element_id")
    }
    if len(generic_tag_parents) > 1:
        raise ValueError("catalog has ambiguous generic Enumeration tag identifiers")
    generic_tag_parent = next(iter(generic_tag_parents), None)
    family_names = set(families)
    for tag_element in elements:
        parents = tag_element.get("parent_element_ids") if isinstance(tag_element, dict) else None
        if (
            not isinstance(tag_element, dict)
            or tag_element.get("kind") != "tag"
            or tag_element.get("allocation") != "assigned"
            or generic_tag_parent is None
            or not isinstance(parents, list)
            or generic_tag_parent not in parents
        ):
            continue
        tag_name = tag_element.get("name")
        raw_tag = tag_element.get("wire_value")
        if not isinstance(tag_name, str) or tag_name not in family_names:
            continue
        if not isinstance(raw_tag, str) or not _TAG_VALUE.fullmatch(raw_tag):
            raise ValueError("catalog Enumeration tag has an invalid assigned tag value")
        tag = int(raw_tag.removeprefix("0x").removeprefix("0X"), 16)
        _record_enumeration_family(family_by_tag, tag, tag_name)

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
        _record_enumeration_family(family_by_tag, tag, family)
    return sorted((tag, family) for tag, family in family_by_tag.items())


def _tag_bitmask_families(catalog: dict[str, Any]) -> list[tuple[int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    bitmask_families = [
        element.get("name")
        for element in elements
        if isinstance(element, dict) and element.get("kind") == "bitmask"
    ]
    if not all(isinstance(name, str) and name for name in bitmask_families):
        raise ValueError("catalog bitmask family name is invalid")
    if len(set(bitmask_families)) != len(bitmask_families):
        raise ValueError("catalog contains ambiguous bitmask family names")

    tag_by_name = {name: value for value, name in _assigned_tag_names(catalog)}
    family_by_tag: list[tuple[int, str]] = []
    for family in bitmask_families:
        if family not in tag_by_name:
            raise ValueError("catalog bitmask family has no matching assigned tag")
        family_by_tag.append((tag_by_name[family], family))
    return sorted(family_by_tag)


def _merge_versioned_wire_data(
    tag_names: list[tuple[int, str]],
    enum_families: list[tuple[int, str]],
    enum_values: list[tuple[str, int, str]],
    mask_families: list[tuple[int, str]],
    mask_values: list[tuple[str, int, str]],
    versioned_tags: dict[str, Any] | None,
    versioned_values: dict[str, Any] | None,
) -> tuple[
    list[tuple[int, str]],
    list[tuple[int, str]],
    list[tuple[str, int, str]],
    list[tuple[int, str]],
    list[tuple[str, int, str]],
]:
    tag_map = dict(tag_names)
    tag_name_set = set(tag_map.values())
    enum_fam_map = dict(enum_families)
    mask_fam_map = dict(mask_families)

    if versioned_tags is not None:
        assignments = list(versioned_tags.get("tagEnumeration", {}).get("assignments", []))
        assignments.extend(versioned_tags.get("v3_0_promoted_attributes", []))
        for a in assignments:
            raw = a["wire_value"]
            val = int(raw.removeprefix("0x").removeprefix("0X"), 16)
            name = a["name"]
            if val not in tag_map and name not in tag_name_set:
                tag_map[val] = name
                tag_name_set.add(name)
            if a.get("source_encoding") == "Enumeration" or "enumeration_family" in a:
                fam = a.get("enumeration_family")
                if not isinstance(fam, str) or not fam.strip():
                    raise ValueError(f"Enumeration tag {name} ({raw}) has no enumeration_family")
                if val not in enum_fam_map:
                    enum_fam_map[val] = fam
            if a.get("source_encoding") == "Bitmask" or a.get("is_bitmask") or "bitmask_family" in a:
                fam = a.get("bitmask_family")
                if not isinstance(fam, str) or not fam.strip():
                    raise ValueError(f"Bitmask tag {name} ({raw}) has no bitmask_family")
                if val not in mask_fam_map:
                    mask_fam_map[val] = fam

    enum_val_seen = {(f, v) for f, v, _ in enum_values}
    enum_name_seen = {(f, n) for f, _, n in enum_values}
    mask_val_seen = {(f, v) for f, v, _ in mask_values}
    mask_name_seen = {(f, n) for f, _, n in mask_values}
    merged_enum_vals = list(enum_values)
    merged_mask_vals = list(mask_values)

    if versioned_values is not None:
        for _ver, groups in versioned_values.get("versions", {}).items():
            for group in groups:
                kind = group.get("kind")
                family = group.get("name")
                for entry in group.get("values", []):
                    val = int(entry["wire_value"].removeprefix("0x").removeprefix("0X"), 16)
                    vname = entry["name"]
                    if kind == "enumeration":
                        if (family, val) not in enum_val_seen and (family, vname) not in enum_name_seen:
                            enum_val_seen.add((family, val))
                            enum_name_seen.add((family, vname))
                            merged_enum_vals.append((family, val, vname))
                    elif kind == "bitmask":
                        if (family, val) not in mask_val_seen and (family, vname) not in mask_name_seen:
                            mask_val_seen.add((family, val))
                            mask_name_seen.add((family, vname))
                            merged_mask_vals.append((family, val, vname))

    return (
        sorted(tag_map.items()),
        sorted(enum_fam_map.items()),
        sorted(merged_enum_vals),
        sorted(mask_fam_map.items()),
        sorted(merged_mask_vals),
    )


def render_tsv(
    catalog: dict[str, Any],
    *,
    versioned_tags: dict[str, Any] | None = None,
    versioned_values: dict[str, Any] | None = None,
) -> str:
    """Render deterministic assigned tag, Enumeration, and Bit Mask records."""
    tag_names, enum_families, enum_values, mask_families, mask_values = _merge_versioned_wire_data(
        _assigned_tag_names(catalog),
        _tag_enumeration_families(catalog),
        _assigned_enumeration_names(catalog),
        _tag_bitmask_families(catalog),
        _assigned_bitmask_names(catalog),
        versioned_tags,
        versioned_values,
    )
    records: list[str] = []
    for value, name in tag_names:
        records.append(f"T\t{value:06X}\t{_tsv_field(name)}")
    for tag, family in enum_families:
        records.append(f"F\t{tag:06X}\t{_tsv_field(family)}")
    for family, value, name in enum_values:
        records.append(f"E\t{_tsv_field(family)}\t{value:08X}\t{_tsv_field(name)}")
    for tag, family in mask_families:
        records.append(f"M\t{tag:06X}\t{_tsv_field(family)}")
    for family, value, name in mask_values:
        records.append(f"B\t{_tsv_field(family)}\t{value:08X}\t{_tsv_field(name)}")
    return "\n".join(records) + "\n"


def render_rust(
    catalog: dict[str, Any],
    *,
    versioned_tags: dict[str, Any] | None = None,
    versioned_values: dict[str, Any] | None = None,
) -> str:
    """Embed deterministic TSV metadata in one generated Rust string literal."""
    metadata = render_tsv(
        catalog, versioned_tags=versioned_tags, versioned_values=versioned_values
    )
    hashes = ""
    while f'"{hashes}' in metadata:
        hashes += "#"
    if len(hashes) > 255:
        raise ValueError("catalog wire-name data needs an unsupported Rust string delimiter")
    return (
        "// @generated by verification/conformance/generate_wire_names.py; do not edit.\n"
        f'pub(crate) const WIRE_NAMES: &str = r{hashes}"{metadata}"{hashes};\n'
    )


def _tsv_field(value: str) -> str:
    if _TSV_DELIMITER.search(value):
        raise ValueError("catalog wire-name field contains a TSV delimiter")
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the generated Rust metadata source")
    mode.add_argument("--check", action="store_true", help="fail if generated wire-name metadata is stale")
    arguments = parser.parse_args(argv)

    try:
        root = arguments.repo_root.resolve(strict=True)
        catalog_bytes = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(catalog_bytes, root)
        versioned_tags = json.loads(
            safe_read_bytes(root, VERSIONED_TAGS_PATH, max_bytes=MAX_BYTES)
        )
        versioned_values = json.loads(
            safe_read_bytes(root, VERSIONED_VALUES_PATH, max_bytes=MAX_BYTES)
        )
        generated_source = render_rust(
            catalog, versioned_tags=versioned_tags, versioned_values=versioned_values
        ).encode("utf-8")
        if arguments.write:
            atomic_write_bytes(root, OUTPUT_PATH, generated_source)
            print(f"generated wire-name metadata: {root / OUTPUT_PATH}")
            return 0
        existing_source = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_BYTES)
    except CatalogValidationError:
        print("wire-name generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except (PathSecurityError, OSError):
        print("wire-name generation failed: repository path or I/O failure", file=sys.stderr)
        return 2
    except ValueError:
        print("wire-name generation failed: catalog wire-name data is invalid", file=sys.stderr)
        return 2

    if existing_source != generated_source:
        print("generated wire names are stale; run generate_wire_names.py --write", file=sys.stderr)
        return 1
    print(f"generated wire-name metadata verified: {root / OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
