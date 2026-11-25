#!/usr/bin/env python3
"""Select KMIP 2.x interoperability cases for the published Cosmian/Eviden support table."""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.operation_catalog import direction, with_cases
from verification.interop.shared.tools.operation_catalog import build_normative_catalog

ALLOWED_VERSIONS = ("2.0", "2.1")
SUPPORT_TABLE = Path("verification/interop/shared/catalog/cosmian-kmip2x-support.json")
ALIASES = {
    ("operation", "Set Attribute (Modify)"): ("Set Attribute", "Modify Attribute"),
    ("operation", "DeriveKey"): ("Derive Key",),
    ("managed_object", "Opaque Data"): ("Opaque Object",),
    ("attribute", "Protection Storage Masks"): ("Protection Storage Mask",),
}
KINDS = {
    "message": {"message_field"},
    "operation": {"operation"},
    "managed_object": {"object_type", "object_structure"},
    "base_object": {"object_structure", "operation_structure", "attribute_structure", "message_field", "credential",
                    "operation_payload_field"},
    "transparent_key": {"enumeration_value"},
    "attribute": {"attribute"},
}
# These workflow recipes assert or supply the listed base field within a real exchange.
BASE_OBJECT_EXCHANGES = {
    "Data": ("test-op-hash", "test-multipart-hash-sha256"),
    "Data Length": ("test-crypto-rng-retrieve-rng-32",),
    "Signature Data": ("test-crypto-sign-rsa-2048-pkcs1_v1_5",
                       "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5"),
    "MAC Data": ("test-crypto-mac-hmac_sha256-256", "test-crypto-mac-verify-hmac_sha256-256"),
    "Correlation Value": ("test-multipart-hash-sha256",),
    "Init Indicator": ("test-multipart-hash-sha256",),
    "Final Indicator": ("test-multipart-hash-sha256",),
}

EXCLUDED_STATUSES = {"not_applicable", "deprecated"}
SERVER_SCOPE = {"operations", "object_lifecycle", "cryptography_and_rng", "server_initiated_operations"}


def normal(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "", name.casefold())


def matches(target: dict, kind: str, names: set[str]) -> bool:
    if target["kind"] not in KINDS[kind]:
        return False
    if kind == "transparent_key":
        return "KEY-FORMAT-TYPE-TRANSPARENT-" in target["target_id"] and normal(
            target["name"].removeprefix("Transparent ")
        ) in names
    return normal(target["name"]) in names


def case_target_ids(case: dict) -> set[str]:
    requires = case.get("requires", {})
    identifiers = set(requires.get("normative_target_ids", []))
    for key in ("normative_target_id", "operation_id", "parent_target_id"):
        if requires.get(key):
            identifiers.add(requires[key])
    identifiers.update(requires.get("protocol_scenario", {}).get("target_ids", []))
    return identifiers


def operation_direction(name: str) -> str:
    return "server_to_client" if name in {"Notify", "Put"} else "client_to_server"


def build_selection(root: Path, version: str) -> tuple[dict, dict]:
    if version not in ALLOWED_VERSIONS:
        raise ValueError(f"Cosmian minimum requires KMIP 2.x, got {version}")
    snapshot = json.loads((root / SUPPORT_TABLE).read_text(encoding="utf-8"))
    catalog, normative_cases = build_normative_catalog(root, version)
    targets = catalog["normative_targets"]
    candidates = {case["case_id"]: case for case in catalog["test_cases"]}
    normative_by_target = defaultdict(set)
    used = set()
    for case in normative_cases:
        for target_id in case_target_ids(case):
            normative_by_target[target_id].add(case["case_id"])

    rows = []
    for entry in snapshot["rows"]:
        kind, name, status = entry["kind"], entry["name"], entry[version]
        names = {normal(alias) for alias in ALIASES.get((kind, name), (name,))}
        checked = status not in EXCLUDED_STATUSES
        matched = {target["target_id"] for target in targets if checked and matches(target, kind, names)}
        if kind == "base_object" and checked and not matched:
            matched = {target["target_id"] for target in targets
                       if target["kind"] == "tag" and normal(target["name"]) in names}
        if kind == "operation":
            matched = {target_id for target_id in matched
                       if ("-OP-S2C-" in target_id) == (operation_direction(name) == "server_to_client")}
        ids = set().union(*(normative_by_target[reference] for reference in matched))
        if kind == "operation" and checked:
            ids.update(case["case_id"] for case in catalog["test_cases"]
                       if case["category"] in SERVER_SCOPE and normal(case["operation"]) in names
                       and direction(case) == operation_direction(name))
        if kind in {"managed_object", "attribute"} and checked:
            key = "object_type" if kind == "managed_object" else "attribute"
            ids.update(case["case_id"] for case in catalog["test_cases"] if normal(case["requires"].get(key, "")) in names)
        if kind == "base_object" and checked:
            ids.update(case_id for case_id in BASE_OBJECT_EXCHANGES.get(name, ()) if case_id in candidates)
        if kind == "message" and checked:
            ids.update(case["case_id"] for case in catalog["test_cases"]
                       if case["case_id"] == "test-op-query")
        ids = {case_id for case_id in ids if candidates[case_id]["execution"]["status"] != "not_applicable"}
        used.update(ids)
        live = sorted(case_id for case_id in ids if candidates[case_id]["execution"]["mode"] == "server_exchange")
        local = sorted(case_id for case_id in ids if candidates[case_id]["execution"]["mode"] == "local_model")
        if not checked:
            disposition = "out_of_scope"
        elif status == "vendor_extension" and not live:
            disposition = "vendor_fixture_required"
        elif not matched:
            disposition = "missing_normative_target"
        elif live:
            disposition = "server_cases_selected"
        else:
            disposition = "local_only" if local else "missing_case"
        rows.append({
            "kind": kind, "name": name, "declared_support": status, "normative_targets": sorted(matched),
            "server_cases": live, "local_cases": local, "disposition": disposition,
        })

    selection = with_cases(catalog,
                           [case for case_id, case in candidates.items() if case_id in used])
    return selection, {"source": snapshot["source"], "protocol_version": version, "rows": rows}


def write_json(path: Path, value: dict) -> None:
    with path.open("x", encoding="utf-8") as output:
        json.dump(value, output, indent=2, ensure_ascii=False)
        output.write("\n")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", choices=ALLOWED_VERSIONS, required=True)
    parser.add_argument("--catalog-output", type=Path, required=True)
    parser.add_argument("--inventory-output", type=Path, required=True)
    args = parser.parse_args()
    try:
        catalog, inventory = build_selection(ROOT, args.version)
        write_json(args.catalog_output, catalog)
        write_json(args.inventory_output, inventory)
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
