"""Generate private TTLV tag allocation metadata from reviewed versioned catalogs.

The KMIP 2.1 catalog and the checksum-pinned KMIP 3.0 tag delta are merged
into the supported-version allocation union. Exact versioned assignments take
precedence over older aggregate ranges; version compatibility is checked by
the protocol layer. The generated output is
``crates/kmipkit-ttlv/src/generated/tag_allocations.rs`` relative to
``--repo-root``.
"""

from __future__ import annotations

import argparse
import hashlib
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
VERSIONED_TAG_CATALOG_PATH = "specification/catalog/kmip-versioned-tags.json"
VERSIONED_SOURCE_PATH = "specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html"
VERSIONED_SOURCE_SHA256 = "a7d5e58e63acd9be33fab08b76039fc8db4e072fdbdf4a280ea5b743bae5682b"
VERSIONED_SOURCE_MAX_BYTES = 24 * 1024 * 1024
OUTPUT_PATH = "crates/kmipkit-ttlv/src/generated/tag_allocations.rs"
_EXACT_VALUE = re.compile(r"(?:0x)?[0-9A-Fa-f]{6}\Z")
_RANGE_VALUE = re.compile(r"\s*([0-9A-Fa-fXx]{6})\s*[-\u2013]\s*([0-9A-Fa-fXx]{6})\s*\Z")
_RANGE_ENDPOINT = re.compile(r"(?:[0-9A-Fa-f]{6}|[0-9A-Fa-f]{3}[Xx]{3})\Z")
_EXACT_ALLOCATIONS = {"assigned", "reserved"}
_RANGE_ALLOCATIONS = {"unused", "reserved", "extension"}


def _parse_exact_value(value: Any) -> int:
    if not isinstance(value, str) or not _EXACT_VALUE.fullmatch(value):
        raise ValueError("catalog tag has an invalid numeric value")
    numeric_value = int(value.removeprefix("0x").removeprefix("0X"), 16)
    if numeric_value > 0xFF_FF_FF:
        raise ValueError("catalog tag is outside the 24-bit range")
    return numeric_value


def _exact_tag_allocations(catalog: dict[str, Any]) -> list[tuple[int, str]]:
    elements = catalog.get("elements")
    if not isinstance(elements, list):
        raise ValueError("catalog has no element list")

    allocations: list[tuple[int, str]] = []
    seen_values: set[int] = set()
    for element in elements:
        if not isinstance(element, dict) or element.get("kind") != "tag":
            continue
        numeric_value = _parse_exact_value(element.get("wire_value"))
        allocation = element.get("allocation")
        if allocation not in _EXACT_ALLOCATIONS:
            raise ValueError("catalog tag has an unsupported allocation classification")
        if numeric_value in seen_values:
            raise ValueError("catalog contains duplicate numeric tag values")
        seen_values.add(numeric_value)
        allocations.append((numeric_value, allocation))
    if not allocations:
        raise ValueError("catalog has no exact tag allocations")
    return sorted(allocations)


def load_versioned_tag_allocations(root: Path) -> list[tuple[int, str]]:
    """Load exact KMIP 3.0 Table 605 assignments after checking the pinned source."""
    raw = safe_read_bytes(root, VERSIONED_TAG_CATALOG_PATH, max_bytes=MAX_BYTES)
    document = json.loads(raw)
    if not isinstance(document, dict) or document.get("formatVersion") != 1:
        raise ValueError("versioned tag catalog has an unsupported format")

    source = document.get("source")
    if source != {
        "sourceId": "kmip-3.0",
        "path": VERSIONED_SOURCE_PATH,
        "sha256": VERSIONED_SOURCE_SHA256,
    }:
        raise ValueError("versioned tag catalog source is not the pinned KMIP 3.0 document")
    source_bytes = safe_read_bytes(root, VERSIONED_SOURCE_PATH, max_bytes=VERSIONED_SOURCE_MAX_BYTES)
    if hashlib.sha256(source_bytes).hexdigest() != VERSIONED_SOURCE_SHA256:
        raise ValueError("pinned KMIP 3.0 source checksum does not match")

    tag_enumeration = document.get("tagEnumeration")
    if (
        not isinstance(tag_enumeration, dict)
        or tag_enumeration.get("section") != "11.58"
        or tag_enumeration.get("table") != 605
    ):
        raise ValueError("versioned tag catalog has an invalid Tag Enumeration source")
    assignments = tag_enumeration.get("assignments")
    if not isinstance(assignments, list) or len(assignments) != 60:
        raise ValueError("versioned tag catalog must contain all 60 Table 605 assignments")

    allocations: list[tuple[int, str]] = []
    seen_values: set[int] = set()
    seen_names: set[str] = set()
    seen_sections: set[str] = set()
    for assignment in assignments:
        if not isinstance(assignment, dict) or set(assignment) not in (
            {"name", "wire_value"},
            {"name", "wire_value", "attribute_section"},
        ):
            raise ValueError("versioned tag catalog has a malformed assignment")
        name = assignment.get("name")
        attribute_section = assignment.get("attribute_section")
        if not isinstance(name, str) or not name.strip():
            raise ValueError("versioned tag catalog assignment has no name")
        if attribute_section is not None and (
            not isinstance(attribute_section, str)
            or not re.fullmatch(r"4\.35\.(?:[1-9]|1[0-9]|20)", attribute_section)
        ):
            raise ValueError("versioned tag catalog assignment has an invalid attribute section")
        numeric_value = _parse_exact_value(assignment.get("wire_value"))
        if (
            numeric_value in seen_values
            or name in seen_names
            or (
                attribute_section is not None
                and attribute_section in seen_sections
            )
        ):
            raise ValueError("versioned tag catalog contains duplicate assignments")
        seen_values.add(numeric_value)
        seen_names.add(name)
        if attribute_section is not None:
            seen_sections.add(attribute_section)
        allocations.append((numeric_value, "assigned"))
    return sorted(allocations)


def _merge_versioned_tag_allocations(
    catalog: dict[str, Any], versioned_tags: list[tuple[int, str]]
) -> list[tuple[int, str]]:
    """Merge exact assignments, with supported-version values overriding old gaps."""
    allocations = dict(_exact_tag_allocations(catalog))
    for value, allocation in versioned_tags:
        if allocation != "assigned":
            raise ValueError("versioned tag delta contains a non-assigned classification")
        allocations[value] = "assigned"
    return sorted(allocations.items())


def _numeric_range_endpoint(value: str, *, upper: bool) -> int:
    if not _RANGE_ENDPOINT.fullmatch(value):
        raise ValueError("catalog tag range has an invalid endpoint")
    if "X" in value.upper():
        value = value.upper().replace("X", "F" if upper else "0")
    numeric_value = int(value, 16)
    if numeric_value > 0xFF_FF_FF:
        raise ValueError("catalog tag range endpoint is outside the 24-bit range")
    return numeric_value


def _tag_allocation_ranges(catalog: dict[str, Any]) -> list[tuple[int, int, str]]:
    ranges = catalog.get("tag_ranges")
    if not isinstance(ranges, list):
        raise ValueError("catalog has no tag range list")

    allocations: list[tuple[int, int, str]] = []
    seen_ranges: set[tuple[int, int]] = set()
    for tag_range in ranges:
        if not isinstance(tag_range, dict):
            raise ValueError("catalog has a malformed tag range")
        range_text = tag_range.get("value_range")
        if not isinstance(range_text, str):
            raise ValueError("catalog tag range has no numeric endpoints")
        match = _RANGE_VALUE.fullmatch(range_text)
        if match is None:
            raise ValueError("catalog tag range has malformed numeric endpoints")
        lower = _numeric_range_endpoint(match.group(1), upper=False)
        upper = _numeric_range_endpoint(match.group(2), upper=True)
        if lower > upper:
            raise ValueError("catalog tag range endpoints are reversed")
        allocation = tag_range.get("allocation")
        if allocation not in _RANGE_ALLOCATIONS:
            raise ValueError("catalog tag range has an unsupported allocation classification")
        key = (lower, upper)
        if key in seen_ranges:
            raise ValueError("catalog contains duplicate numeric tag ranges")
        seen_ranges.add(key)
        allocations.append((lower, upper, allocation))
    if not allocations:
        raise ValueError("catalog has no tag allocation ranges")
    return sorted(allocations)


def _rust_number(value: int) -> str:
    return f"0x{value >> 16:02X}_{(value >> 8) & 0xFF:02X}_{value & 0xFF:02X}"


def _rust_allocation(allocation: str) -> str:
    return {
        "assigned": "Assigned",
        "reserved": "Reserved",
        "unused": "Unused",
        "extension": "Extension",
    }[allocation]


def render_rust(
    catalog: dict[str, Any], versioned_tags: list[tuple[int, str]] | None = None
) -> str:
    """Render deterministic private Rust metadata for exact tags and ranges."""
    exact_allocations = _merge_versioned_tag_allocations(catalog, versioned_tags or [])
    tag_ranges = _tag_allocation_ranges(catalog)
    lines = [
        "// @generated by tools/normative_catalog/generate_ttlv_tags.py; do not edit.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub(super) enum TagAllocationKind {",
        "    Assigned,",
        "    Reserved,",
        "    Unused,",
        "    Extension,",
        "}",
        "",
        "pub(super) const EXACT_TAG_ALLOCATIONS: &[(u32, TagAllocationKind)] = &[",
    ]
    for value, allocation in exact_allocations:
        lines.append(f"    ({_rust_number(value)}, TagAllocationKind::{_rust_allocation(allocation)}),")
    lines.extend([
        "];",
        "",
        "pub(super) const TAG_ALLOCATION_RANGES: &[(u32, u32, TagAllocationKind)] = &[",
    ])
    for lower, upper, allocation in tag_ranges:
        lines.append(
            f"    ({_rust_number(lower)}, {_rust_number(upper)}, "
            f"TagAllocationKind::{_rust_allocation(allocation)}),"
        )
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
        raw_catalog = safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES)
        catalog = load_validated_catalog(raw_catalog, root)
        versioned_tags = load_versioned_tag_allocations(root)
        generated = render_rust(catalog, versioned_tags).encode("utf-8")
        if arguments.write:
            atomic_write_bytes(root, OUTPUT_PATH, generated)
            print(f"generated TTLV tag allocations: {root / OUTPUT_PATH}")
            return 0
        existing = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_BYTES)
    except CatalogValidationError:
        print("TTLV tag generation failed: catalog validation failed", file=sys.stderr)
        return 2
    except PathSecurityError:
        print("TTLV tag generation failed: repository path validation failed", file=sys.stderr)
        return 2
    except OSError:
        print("TTLV tag generation failed: repository I/O failed", file=sys.stderr)
        return 2
    except ValueError:
        print("TTLV tag generation failed: catalog allocation data is invalid", file=sys.stderr)
        return 2

    if existing != generated:
        print("generated TTLV tag allocations are stale; run generate_ttlv_tags.py --write", file=sys.stderr)
        return 1
    print(f"generated TTLV tag allocations verified: {root / OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
