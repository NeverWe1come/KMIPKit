#!/usr/bin/env python3
"""Select KMIP 2.x interoperability cases for the published Cosmian/Eviden support table."""

from __future__ import annotations

import argparse
from collections import defaultdict
from copy import deepcopy
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.operation_catalog import build_normative_catalog, direction, with_cases
from verification.interop.shared.tools.matrix.generator import LANGUAGES

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
FIELD_EXCHANGES = {
    "Data": ("test-op-hash", "test-multipart-hash-sha256"),
    "Data Length": ("test-crypto-rng-retrieve-rng-32",),
    "Signature Data": ("test-crypto-sign-rsa-2048-pkcs1_v1_5",
                       "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5"),
    "MAC Data": ("test-crypto-mac-hmac_sha256-256", "test-crypto-mac-verify-hmac_sha256-256"),
    "Correlation Value": ("test-multipart-hash-sha256",),
    "Init Indicator": ("test-multipart-hash-sha256",),
    "Final Indicator": ("test-multipart-hash-sha256",),
    "Lease Time": ("test-op-obtain-lease",),
    "Last Change Date": ("test-op-obtain-lease",),
}

CONSTRUCTION_APIS = ("typed-protocol", "fluent", "generic-ttlv")
LEGACY_BOUNDARIES = {
    "Template": "legacy managed object",
    "Template-Attribute Structures": "legacy attribute container",
    "Transparent Key Structures": "legacy umbrella row; individual formats are tracked separately",
    **{f"{algorithm} {visibility} Key": "legacy algorithm-specific transparent key format"
       for algorithm in ("ECDSA", "ECDH", "ECMQV") for visibility in ("Private", "Public")},
}
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
        matched = {target["target_id"] for target in targets if matches(target, kind, names)}
        if status == "not_applicable":
            if matched:
                raise ValueError(f"{name} is listed as N/A for {version} but has a normative target")
            if name in LEGACY_BOUNDARIES:
                boundary = {"kind": "legacy", "context": LEGACY_BOUNDARIES[name]}
            else:
                other_version = next(v for v in ALLOWED_VERSIONS if v != version)
                if entry[other_version] != "supported":
                    raise ValueError(f"{name} has no documented KMIP version boundary")
                boundary = {"kind": "introduced", "version": other_version}
            rows.append({
                "kind": kind, "name": name, "declared_support": status, "normative_targets": [],
                "server_cases": [], "local_cases": [], "disposition": "version_boundary_required",
                "version_boundary": boundary,
            })
            continue
        if kind == "base_object" and not matched:
            matched = {target["target_id"] for target in targets
                       if target["kind"] == "tag" and normal(target["name"]) in names}
        if kind == "operation":
            matched = {target_id for target_id in matched
                       if ("-OP-S2C-" in target_id) == (operation_direction(name) == "server_to_client")}
        ids = set().union(*(normative_by_target[reference] for reference in matched))
        if kind == "operation":
            ids.update(case["case_id"] for case in catalog["test_cases"]
                       if case["category"] in SERVER_SCOPE and normal(case["operation"]) in names
                       and direction(case) == operation_direction(name))
        if kind in {"managed_object", "attribute"}:
            key = "object_type" if kind == "managed_object" else "attribute"
            ids.update(case["case_id"] for case in catalog["test_cases"] if normal(case["requires"].get(key, "")) in names)
        if kind in {"base_object", "attribute"}:
            ids.update(case_id for case_id in FIELD_EXCHANGES.get(name, ()) if case_id in candidates)
        if kind == "message":
            ids.update(case["case_id"] for case in catalog["test_cases"]
                       if case["case_id"] == "test-op-query")
        ids = {case_id for case_id in ids if candidates[case_id]["execution"]["status"] != "not_applicable"}
        used.update(ids)
        live = sorted(case_id for case_id in ids if candidates[case_id]["execution"]["mode"] == "server_exchange")
        local = sorted(case_id for case_id in ids if candidates[case_id]["execution"]["mode"] == "local_model")
        if status == "vendor_extension" and not live:
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

    for row in rows:
        row["obligations"] = [
            {"language": language, "api_level": api,
             "expectation": {
                 "supported": "verify_result",
                 "not_implemented": "verify_unsupported_response",
                 "vendor_extension": "verify_provider_fixture",
                 "not_applicable": "verify_version_boundary",
             }[row["declared_support"]]}
            for language in LANGUAGES for api in CONSTRUCTION_APIS
        ]

    selection = with_cases(catalog,
                           [case for case_id, case in candidates.items() if case_id in used])
    return selection, {
        "source": snapshot["source"], "source_sha256": snapshot["source_sha256"],
        "protocol_version": version, "rows": rows,
    }


def with_execution_evidence(inventory, catalog, catalog_runs):
    """Attach compatible case results without claiming full coverage of a row."""
    report = deepcopy(inventory)
    expected = {case["case_id"]: case["execution"]["assertions"] for case in catalog["test_cases"]}
    by_route = defaultdict(lambda: defaultdict(list))
    for run in catalog_runs:
        if run["protocol_version"] != report["protocol_version"]:
            continue
        execution = run["execution"]
        route = by_route[run["language"], execution["api_level"]]
        for case in run["cases"]:
            case_id = case["case_id"]
            if case_id not in expected or case["assertions"] != expected[case_id]:
                continue
            route[case_id].append({
                **case, "run_id": run["run_id"], "scope": run["scope"],
                "catalog_sha256": run["catalog_sha256"],
                "library_commit": execution["library_commit"],
                "artifact_sha256": execution["artifact_sha256"],
            })

    for row in report["rows"]:
        case_ids = sorted(set(row["server_cases"]) | set(row["local_cases"]))
        for obligation in row["obligations"]:
            route = by_route[obligation["language"], obligation["api_level"]]
            obligation["case_results"] = [result for case_id in case_ids for result in route.get(case_id, [])]
    return report


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
