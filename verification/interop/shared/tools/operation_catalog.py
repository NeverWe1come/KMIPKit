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


def build_catalog(root: Path, version: str) -> dict:
    source = inventory(root)["versions"][version]
    original = json.loads((root / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
    templates = candidate_recipes(original["test_cases"], "2.1" if version == "3.0" else version)
    cases = []
    counts = Counter()
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
        counts[flow] += 1
    if len({case["case_id"] for case in cases}) != len(cases):
        raise ValueError("operation catalog has duplicate case identifiers")
    return {
        **original,
        "catalog_source": str(source["source"]),
        "target_versions": [version],
        "test_cases": cases,
        "statistics": {
            "operations_covered": counts["client_to_server"],
            "server_to_client_operations_covered": counts["server_to_client"],
            "managed_objects_covered": 0,
            "attributes_covered": 0,
            "total_cases": len(cases),
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", choices=VERSIONS, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = json.dumps(build_catalog(ROOT, args.version), indent=2, ensure_ascii=False) + "\n"
    if args.output is None:
        print(result, end="")
    else:
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(result)


if __name__ == "__main__":
    main()
