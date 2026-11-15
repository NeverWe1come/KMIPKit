"""Generate the source-backed operation inventory for the fluent API roadmap."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from .validate import MAX_BYTES
else:
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from validate import MAX_BYTES

BASE_PATH = "specification/catalog/kmip-2.1.json"
VERSIONED_PATH = "specification/catalog/kmip-versioned-values.json"
MANIFEST_PATH = "specification/api/public-api.json"
OUTPUT_PATH = "specification/catalog/fluent-operation-inventory.json"
MAX_SOURCE_BYTES = 64 * 1024 * 1024
V3_OPERATION_SECTIONS = {
    "Create Credential": "6.1.9",
    "Create Group": "6.1.10",
    "Create User": "6.1.13",
    "Deactivate": "6.1.14",
    "Decapsulate": "6.1.15",
    "Encapsulate": "6.1.22",
    "Obliterate": "6.1.41",
}


def _read_json(root: Path, relative_path: str, *, max_bytes: int = MAX_BYTES) -> Any:
    return json.loads(safe_read_bytes(root, relative_path, max_bytes=max_bytes))


def _v3_operations(document: dict[str, Any]) -> list[dict[str, Any]]:
    versions = document.get("versions")
    groups = versions.get("v3_0") if isinstance(versions, dict) else None
    if not isinstance(groups, list):
        raise ValueError("versioned values catalog has no KMIP 3.0 groups")
    operation_groups = [
        group for group in groups
        if isinstance(group, dict)
        and group.get("kind") == "enumeration"
        and group.get("name") == "Operation"
    ]
    if len(operation_groups) != 1:
        raise ValueError("versioned values catalog must have one KMIP 3.0 Operation group")
    values = operation_groups[0].get("values")
    if not isinstance(values, list) or len(values) != 7:
        raise ValueError("KMIP 3.0 operation delta must contain seven assigned values")
    return values


def build_inventory(
    base_catalog: dict[str, Any],
    versioned_catalog: dict[str, Any],
    manifest: dict[str, Any],
) -> dict[str, Any]:
    """Join assigned operations with direction, owner clause and API scope."""
    base_rows = [
        element for element in base_catalog.get("elements", [])
        if isinstance(element, dict) and element.get("kind") == "operation"
    ]
    operation_values = {
        element.get("name"): element.get("wire_value")
        for element in base_catalog.get("elements", [])
        if isinstance(element, dict)
        and element.get("kind") == "enumeration_value"
        and "KMIPKIT-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", [])
    }
    route_keys: set[tuple[str, str, str]] = set()
    seen_values: set[str] = set()
    rows: list[dict[str, Any]] = []
    for element in base_rows:
        name = element.get("name")
        refs = element.get("source_refs")
        if not isinstance(name, str):
            raise ValueError("KMIP 2.1 operation name is missing")
        if not isinstance(refs, list) or not refs:
            raise ValueError(f"KMIP 2.1 operation {name} has no source reference")
        section = refs[0].get("section")
        wire_value = operation_values.get(name)
        if not isinstance(wire_value, str) or len(wire_value) != 8:
            raise ValueError(f"KMIP 2.1 operation {name} has no assigned wire value")
        if wire_value in seen_values and element.get("direction") not in {
            "client_to_server", "server_to_client"
        }:
            raise ValueError(f"duplicate KMIP 2.1 operation wire value: {wire_value}")
        route_key = (name, wire_value, element.get("direction"))
        if route_key in route_keys:
            raise ValueError(f"duplicate KMIP 2.1 operation route: {route_key}")
        route_keys.add(route_key)
        seen_values.add(wire_value)
        rows.append({
            "name": name,
            "version": "2.1",
            "wireValue": wire_value.upper(),
            "direction": element.get("direction"),
            "scopeState": element.get("scope_state"),
            "sourceRefs": [{"sourceId": ref.get("source_id"), "section": ref.get("section")}
                           for ref in refs],
            "featureSpec": element.get("feature_spec"),
        })

    manifest_operations = manifest.get("operations")
    if not isinstance(manifest_operations, list):
        raise ValueError("public API manifest has no operation inventory")
    public_by_name = {
        row.get("name"): row for row in manifest_operations
        if isinstance(row, dict) and row.get("scope") == "client_1_0"
    }
    for value in _v3_operations(versioned_catalog):
        name = value.get("name")
        wire_value = value.get("wire_value")
        if not isinstance(name, str) or not isinstance(wire_value, str):
            raise ValueError("malformed KMIP 3.0 operation assignment")
        section = V3_OPERATION_SECTIONS.get(name)
        if section is None:
            raise ValueError(f"KMIP 3.0 operation has no reviewed source section: {name}")
        wire_value = wire_value.removeprefix("0x").upper().zfill(8)
        if wire_value in seen_values:
            raise ValueError(f"duplicate operation assignment across versions: {wire_value}")
        seen_values.add(wire_value)
        rows.append({
            "name": name,
            "version": "3.0",
            "wireValue": wire_value,
            "direction": "client_to_server",
            "scopeState": "client_3_0_draft",
            "sourceRefs": [{"sourceId": "kmip-3.0", "section": section}],
            "featureSpec": "KMIPKIT-0071",
        })

    for row in rows:
        public = public_by_name.get(row["name"]) if row["direction"] == "client_to_server" else None
        row["publicApiScope"] = public.get("scope") if public else None
        row["publicApiState"] = public.get("apiState") if public else None
        row["clientSend"] = row["direction"] == "client_to_server"
        row["listenerRoute"] = row["direction"] == "server_to_client"

    rows.sort(key=lambda row: (row["version"], int(row["wireValue"], 16)))
    return {
        "formatVersion": 1,
        "sources": [BASE_PATH, VERSIONED_PATH, MANIFEST_PATH],
        "operations": rows,
    }


def render(root: Path) -> bytes:
    inventory = build_inventory(
        _read_json(root, BASE_PATH),
        _read_json(root, VERSIONED_PATH, max_bytes=MAX_SOURCE_BYTES),
        _read_json(root, MANIFEST_PATH, max_bytes=MAX_SOURCE_BYTES),
    )
    return (json.dumps(inventory, indent=2, ensure_ascii=False) + "\n").encode()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        root = args.repo_root.resolve(strict=True)
        rendered = render(root)
        if args.check:
            actual = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_SOURCE_BYTES)
            if actual != rendered:
                print(f"{OUTPUT_PATH} is stale", file=sys.stderr)
                return 1
        else:
            atomic_write_bytes(root, OUTPUT_PATH, rendered)
    except (OSError, ValueError, json.JSONDecodeError, PathSecurityError) as error:
        print(f"fluent operation inventory: {error}", file=sys.stderr)
        return 1
    if not args.check:
        print(f"wrote {OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
