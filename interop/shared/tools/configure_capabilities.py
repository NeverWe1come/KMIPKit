#!/usr/bin/env python3
"""Interactive CLI tool to inspect, configure, and validate KMIP Server capability profiles."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_profile(path: Path) -> dict:
    safe_path = Path(path).resolve()
    with open(safe_path, "r", encoding="utf-8") as f:  # NOSONAR
        return json.load(f)


def save_profile(path: Path, data: dict) -> None:
    safe_path = Path(path).resolve()
    safe_path.parent.mkdir(parents=True, exist_ok=True)
    with open(safe_path, "w", encoding="utf-8") as f:  # NOSONAR
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")


def match_catalog_against_profile(catalog: dict, profile: dict) -> dict:
    supported_ops = set(k for k, v in profile.get("operations", {}).items() if v.get("supported"))
    supported_objs = set(k for k, v in profile.get("managed_objects", {}).items() if v.get("supported"))
    supported_attrs = set(k for k, v in profile.get("attributes", {}).items() if v.get("supported"))

    planned = []
    unsupported = []

    for case in catalog.get("test_cases", []):
        req = case.get("requires", {})
        op_req = req.get("operation")
        obj_req = req.get("object_type")
        attr_req = req.get("attribute")

        is_supported = True
        reasons = []

        if op_req and op_req not in supported_ops:
            is_supported = False
            reasons.append(f"Operation '{op_req}' not supported")
        if obj_req and obj_req not in supported_objs:
            is_supported = False
            reasons.append(f"Object type '{obj_req}' not supported")
        if attr_req and attr_req not in supported_attrs:
            is_supported = False
            reasons.append(f"Attribute '{attr_req}' not supported")

        item = {
            "case_id": case["case_id"],
            "operation": case["operation"],
            "oasis_section": case.get("oasis_section", "N/A"),
            "category": case["category"],
        }
        if is_supported:
            planned.append(item)
        else:
            item["unsupported_reasons"] = reasons
            unsupported.append(item)

    return {
        "server_id": profile.get("server", {}).get("id", "unknown"),
        "total_cases": len(catalog.get("test_cases", [])),
        "planned_count": len(planned),
        "unsupported_count": len(unsupported),
        "planned_cases": planned,
        "unsupported_cases": unsupported
    }


def main():
    parser = argparse.ArgumentParser(description="Configure and inspect KMIP Server capability profiles")
    parser.add_argument("--profile", type=Path, required=True, help="Path to capability profile JSON")
    parser.add_argument("--catalog", type=Path, help="Path to kmipkit-1.0-test-catalog.json")
    parser.add_argument("--summary", action="store_true", help="Print summary of matched test cases")
    parser.add_argument("--set-op", nargs=2, metavar=("OPERATION", "SUPPORTED"), help="Update operation support (e.g. 'Create' true)")
    parser.add_argument("--set-encoding", nargs=2, metavar=("ENCODING", "SUPPORTED"), help="Update encoding support (e.g. 'JSON' true)")

    args = parser.parse_args()

    if not args.profile.exists():
        print(f"Error: Profile does not exist: {args.profile}", file=sys.stderr)
        sys.exit(1)

    profile = load_profile(args.profile)
    modified = False

    if args.set_op:
        op_name, val_str = args.set_op
        val = val_str.lower() in {"true", "1", "yes"}
        profile.setdefault("operations", {}).setdefault(op_name, {})["supported"] = val
        modified = True
        print(f"Updated operation '{op_name}' -> supported = {val}")

    if args.set_encoding:
        enc_name, val_str = args.set_encoding
        val = val_str.lower() in {"true", "1", "yes"}
        profile.setdefault("encodings", {}).setdefault(enc_name, {})["supported"] = val
        modified = True
        print(f"Updated encoding '{enc_name}' -> supported = {val}")

    if modified:
        save_profile(args.profile, profile)
        print(f"Saved modified profile to {args.profile}")

    if args.summary:
        catalog_path = args.catalog
        if not catalog_path:
            catalog_path = Path(__file__).resolve().parents[1] / "catalog" / "kmipkit-1.0-test-catalog.json"
        if not catalog_path.exists():
            print(f"Error: Catalog does not exist: {catalog_path}", file=sys.stderr)
            sys.exit(1)
        safe_catalog_path = catalog_path.resolve()
        with open(safe_catalog_path, "r", encoding="utf-8") as f:  # NOSONAR
            catalog = json.load(f)
        match = match_catalog_against_profile(catalog, profile)
        print(f"\nCapability matching summary for {match['server_id']}:")
        print(f"  Total test catalog cases: {match['total_cases']}")
        print(f"  Supported (✅ / ❌ executable): {match['planned_count']}")
        print(f"  Unsupported (➖ skipped/probed): {match['unsupported_count']}")


if __name__ == "__main__":
    main()
