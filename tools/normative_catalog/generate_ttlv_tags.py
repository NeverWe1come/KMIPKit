"""Generate private TTLV tag allocation metadata from the reviewed catalog.

The local input is ``specification/catalog/kmip-2.1.json`` and the generated
output is ``crates/kmipkit-ttlv/src/generated/tag_allocations.rs``, both
relative to ``--repo-root`` (the current directory by default). ``--write``
validates the catalog and atomically writes the output. ``--check`` safely
reads the existing output and reports whether it matches without writing or
creating files.
"""

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


def render_rust(catalog: dict[str, Any]) -> str:
    """Render deterministic private Rust metadata for exact tags and ranges."""
    exact_allocations = _exact_tag_allocations(catalog)
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
        generated = render_rust(catalog).encode("utf-8")
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
