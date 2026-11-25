#!/usr/bin/env python3
"""Build one server-exchange case for every operation and direction in a KMIP version."""

from __future__ import annotations

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.conformance.fluent_field_inventory import inventory

VERSIONS = ("2.0", "2.1", "3.0")
DOCUMENTS = {
    "2.0": "OASIS KMIP 2.0 Specification",
    "2.1": "OASIS KMIP 2.1 Specification",
    "3.0": "OASIS KMIP 3.0 Working Draft",
}


def direction(case):
    return ("server_to_client" if case["requires"].get("workflow_type") == "server_to_client"
            else "client_to_server")


def candidate_recipes(cases, version):
    candidates = {}
    for case in cases:
        execution = case["execution"]
        if execution["mode"] != "server_exchange" or execution["status"] == "not_applicable":
            continue
        if version not in execution["protocol_versions"]:
            continue
        if not case["case_id"].startswith(("test-op-", "test-crypto-", "test-s2c-")):
            continue
        candidates.setdefault((case["operation"], direction(case)), case)
    return candidates


def new_recipe(name, flow):
    slug = re.sub(r"[^a-z0-9]+", "-", name.casefold()).strip("-")
    return {
        "case_id": f"test-{'s2c' if flow == 'server_to_client' else 'op'}-{slug}",
        "category": "server_initiated_operations" if flow == "server_to_client" else "operations",
        "operation": name,
        "requires": {"operation": name, "workflow_type": flow},
        "isolation_recipe": {
            "unique_name_prefix": f"kmipkit-{slug}",
            "setup_steps": ["provide_isolated_prerequisites"],
            "target_step": name,
            "teardown_steps": ["remove_only_owned_resources"],
        },
        "execution": {
            "mode": "server_exchange",
            "steps": ["prepare prerequisites", "exchange target operation", "clean up fixtures"],
            "assertions": ["response operation and result match the target"],
            "mutation": "mutating",
            "status": "planned",
        },
    }


def build_catalog(root: Path, version: str, *, include_variants: bool = False) -> dict:
    source = inventory(root)["versions"][version]
    original = json.loads((root / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
    templates = candidate_recipes(original["test_cases"], "2.1" if version == "3.0" else version)
    cases = []
    for operation in source["operations"]:
        name, flow, section = (operation[key] for key in ("name", "direction", "section"))
        template = templates.get((name, flow)) or new_recipe(name, flow)
        execution = {**template["execution"], "protocol_versions": [version], "status": "planned"}
        # These operations change server state even though the legacy catalog used read_only.
        if name in {"Login", "Delegated Login", "Logout", "Log", "Interop", "Set Endpoint Role", "Notify", "Put"}:
            execution["mutation"] = "mutating"
        cases.append({
            **template, "oasis_section": section, "execution": execution,
            "normative_refs": [{"document": DOCUMENTS[version], "section": section}],
        })
    if include_variants:
        operation_keys = {(case["operation"], direction(case)) for case in cases}
        existing_ids = {case["case_id"] for case in cases}
        for case in original["test_cases"]:
            execution = case["execution"]
            if case["case_id"] in existing_ids or version not in execution["protocol_versions"]:
                continue
            if (execution["mode"] == "server_exchange"
                    and case["category"] != "version_boundaries"
                    and (case["operation"], direction(case)) not in operation_keys):
                continue
            cases.append({**case, "execution": {**execution, "protocol_versions": [version]}})
    if len({case["case_id"] for case in cases}) != len(cases):
        raise ValueError("operation catalog has duplicate case identifiers")
    return with_cases({**original, "catalog_source": str(source["source"]), "target_versions": [version]}, cases)


def select_cases(catalog: dict, *, operations: list[str], case_ids: list[str]) -> dict:
    """Select exact operations and case IDs without changing their protocol or recipes."""
    cases = catalog["test_cases"]
    operation_names = {case["operation"].casefold() for case in cases}
    requested_operations = {name.casefold() for name in operations}
    requested_cases = set(case_ids)
    unknown = (requested_operations - operation_names) | (
        requested_cases - {case["case_id"] for case in cases}
    )
    if unknown:
        raise ValueError("unknown operation or case in the selected version: " + ", ".join(sorted(unknown)))
    if not requested_operations and not requested_cases:
        return catalog
    selected = [
        case for case in cases
        if case["operation"].casefold() in requested_operations or case["case_id"] in requested_cases
    ]
    return with_cases(catalog, selected)


def with_cases(catalog: dict, cases: list[dict]) -> dict:
    operations = {
        (case["operation"], direction(case))
        for case in cases
        if case["execution"]["mode"] == "server_exchange"
        and case["execution"]["status"] != "not_applicable"
        and case["category"] != "version_boundaries"
    }
    categories = Counter(case["category"] for case in cases)
    return {
        **catalog,
        "test_cases": cases,
        "statistics": {
            "operations_covered": sum(flow == "client_to_server" for _, flow in operations),
            "server_to_client_operations_covered": sum(flow == "server_to_client" for _, flow in operations),
            "managed_objects_covered": categories["managed_objects"],
            "attributes_covered": categories["attributes"],
            "enumerations_covered": sum(case["case_id"].startswith("test-enum-") for case in cases),
            "bitmasks_covered": sum(case["case_id"].startswith("test-bitmask-") for case in cases),
            "total_cases": len(cases),
        },
    }


def build_normative_catalog(root: Path, version: str) -> tuple[dict, list[dict]]:
    """Combine the existing executable recipes and version-specific normative obligations."""
    if version not in ("2.0", "2.1"):
        raise ValueError(f"normative KMIP 2.x catalog does not include {version}")
    path = root / "verification/interop/shared/catalog/kmipkit-2x-matrix-catalog.json"
    normative = json.loads(path.read_text(encoding="utf-8"))
    targets = [target for target in normative["normative_targets"]
               if version in target.get("protocol_versions", ("2.0", "2.1"))]
    catalog = build_catalog(root, version, include_variants=True)
    cases = {case["case_id"]: case for case in catalog["test_cases"]}
    obligations = []
    for case in normative["test_cases"]:
        execution = case["execution"]
        if version not in execution["protocol_versions"]:
            continue
        case_id = case["case_id"]
        requires = case["requires"]
        if case_id in cases:
            # A baseline exchange cannot supply evidence for the broader specification assertions.
            if execution["assertions"] == cases[case_id]["execution"]["assertions"]:
                obligations.append(case)
                continue
            requires = {**requires, "baseline_case_id": case_id}
            case_id = f"{case_id}-normative"
            if case_id in cases:
                raise ValueError(f"duplicate normative case identifier: {case_id}")
        if "protocol_scenario" in requires:
            scenario = {**requires["protocol_scenario"], "protocol_versions": [version]}
            requires = {**requires, "protocol_scenario": scenario}
        obligation = {**case, "case_id": case_id, "execution": {**execution, "protocol_versions": [version]},
                      "requires": requires}
        cases[case_id] = obligation
        obligations.append(obligation)
    return with_cases({**catalog, "normative_targets": targets}, list(cases.values())), obligations


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", choices=VERSIONS, required=True)
    parser.add_argument("--with-variants", action="store_true",
                        help="Include version-specific attribute, object, and workflow cases")
    parser.add_argument("--operation", action="append", default=[], help="Select an operation by exact name")
    parser.add_argument("--case", action="append", default=[], help="Select a case by exact ID")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    catalog = build_catalog(ROOT, args.version, include_variants=args.with_variants)
    try:
        catalog = select_cases(catalog, operations=args.operation, case_ids=args.case)
    except ValueError as error:
        parser.error(str(error))
    result = json.dumps(catalog, indent=2, ensure_ascii=False) + "\n"
    if args.output is None:
        print(result, end="")
    else:
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(result)


if __name__ == "__main__":
    main()
