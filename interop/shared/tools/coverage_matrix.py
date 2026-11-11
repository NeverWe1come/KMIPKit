#!/usr/bin/env python3
"""Expand normative cases into the installed public construction-route matrix."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import tempfile
from pathlib import Path
from typing import Iterator

from cli_paths import resolve_within

ROOT = Path(__file__).resolve().parents[3]
LANGUAGES = ("rust", "c", "java", "python")


def _norm(value: str) -> str:
    return re.sub(r"[^a-z0-9]", "", value.casefold())


def _operation_ids(catalog: dict) -> dict[str, str]:
    return {
        _norm(target["name"]): target["target_id"]
        for target in catalog["normative_targets"]
        if target["kind"] == "operation"
    }


def _case_targets(case: dict, operation_ids: dict[str, str]) -> set[str]:
    return {item["target_id"] for item in _case_obligations(case, operation_ids)}


def _case_obligations(case: dict, operation_ids: dict[str, str]) -> list[dict]:
    """Preserve each normative target and each ordered scenario operation/item."""
    requires = case.get("requires", {})
    obligations = []

    def add(target_id: str, *, step_index: int | None = None,
            operation: str | None = None, item_index: int | None = None) -> None:
        identity = (target_id, step_index, operation, item_index)
        if not any((item["target_id"], item.get("workflow_step_index"),
                    item.get("workflow_operation"), item.get("workflow_item_index")) == identity
                   for item in obligations):
            item = {"target_id": target_id}
            if step_index is not None:
                item["workflow_step_index"] = step_index
            if operation is not None:
                item["workflow_operation"] = operation
            if item_index is not None:
                item["workflow_item_index"] = item_index
            obligations.append(item)

    plain_targets = []
    for key in ("normative_target_id", "operation_id"):
        value = requires.get(key)
        if isinstance(value, str):
            plain_targets.append(value)
    protocol_scenario = requires.get("protocol_scenario", {})
    if isinstance(protocol_scenario, dict):
        scenario_targets = [value for value in protocol_scenario.get("target_ids", []) if isinstance(value, str)]
        plain_targets.extend(scenario_targets)
        operation_names = protocol_scenario.get("operations", [])
        if not isinstance(operation_names, list):
            raise ValueError(f"scenario operations must be an ordered array: {case['case_id']}")
        if any(not isinstance(operation, str) or not operation for operation in operation_names):
            raise ValueError(f"scenario operation names must be non-empty strings: {case['case_id']}")
        scenario_versions = protocol_scenario.get("protocol_versions")
        execution_versions = case.get("execution", {}).get("protocol_versions", [])
        if scenario_versions is not None:
            if not isinstance(scenario_versions, list) or set(scenario_versions) != set(execution_versions):
                raise ValueError(f"scenario protocol versions disagree with the case: {case['case_id']}")
        parameters = protocol_scenario.get("parameters", {})
        if not isinstance(parameters, dict):
            raise ValueError(f"scenario parameters must be an object: {case['case_id']}")
        mapped_operation_ids = [operation_ids.get(_norm(operation)) for operation in operation_names]
        if any(target is None for target in mapped_operation_ids):
            raise ValueError(f"scenario operation lacks a normative target: {case['case_id']}")
        scenario_operation_ids = [target for target in scenario_targets if target in set(operation_ids.values())]
        if mapped_operation_ids != scenario_operation_ids:
            raise ValueError(f"scenario operation order disagrees with its normative targets: {case['case_id']}")
    else:
        operation_names = []
        scenario_targets = []
        parameters = {}
    plain_targets.extend(value for value in requires.get("normative_target_ids", []) if isinstance(value, str))

    item_targets = set()
    scenario_items = parameters.get("items", [])
    if not isinstance(scenario_items, list):
        raise ValueError(f"scenario batch items must be an ordered array: {case['case_id']}")
    for item_index, item in enumerate(scenario_items):
        if isinstance(item, str) and item:
            operation = item.split("(", 1)[0].strip()
        elif isinstance(item, dict) and isinstance(item.get("operation"), str) and item["operation"]:
            operation = item["operation"]
        else:
            raise ValueError(f"scenario batch items require an operation name: {case['case_id']}")
        target_id = operation_ids.get(_norm(operation))
        if not target_id:
            raise ValueError(f"scenario batch item lacks a normative operation target: {case['case_id']}")
        add(target_id, operation=operation, item_index=item_index)
        item_targets.add(target_id)

    operation_sequence = requires.get("operation_sequence", [])
    if not isinstance(operation_sequence, list):
        raise ValueError(f"operation_sequence must be an ordered array: {case['case_id']}")
    if any(not isinstance(operation, str) or not operation for operation in operation_sequence):
        raise ValueError(f"operation_sequence names must be non-empty strings: {case['case_id']}")
    operations = [*operation_names, *operation_sequence]
    scenario_step_targets = set()
    for step_index, operation in enumerate(operations):
        operation_id = operation_ids.get(_norm(operation))
        if not operation_id:
            raise ValueError(f"scenario operation lacks a normative target: {case['case_id']}")
        if operation_id in item_targets:
            continue
        add(operation_id, step_index=step_index, operation=operation)
        scenario_step_targets.add(operation_id)

    # Keep every non-operation field/value target as its own obligation.
    for target_id in plain_targets:
        if target_id not in item_targets and target_id not in scenario_step_targets:
            add(target_id)
    # Cases authored before individual operation target IDs were attached can
    # still be matched to the normative operation by its exact catalog name.
    operation_id = operation_ids.get(_norm(case.get("operation", "")))
    if operation_id and operation_id not in item_targets and operation_id not in scenario_step_targets:
        add(operation_id)
    return obligations


def _unavailable_for_target(manifest: dict, language: str, target_id: str) -> dict | None:
    for item in manifest["unavailable"]:
        if item["language"] != language:
            continue
        if target_id in item.get("operation_ids", []) or target_id in item.get("normative_target_ids", []):
            return item
    return None


def _unavailable_for_case(manifest: dict, language: str, targets: set[str]) -> dict | None:
    for item in manifest["unavailable"]:
        if item["language"] != language:
            continue
        operation_ids = set(item.get("operation_ids", []))
        if operation_ids and operation_ids.intersection(targets):
            return item
        capability = item.get("capability")
        if capability == "generic-send" and any(target.startswith("KMIPKIT-PAYLOAD-") for target in targets):
            return item
    return None


def applicable_routes(case: dict, manifest: dict, language: str, operation_ids: dict[str, str]) -> list[dict]:
    targets = _case_targets(case, operation_ids)
    case_version = set(case["execution"]["protocol_versions"])
    candidates = []
    for route in manifest["routes"]:
        if route["language"] != language or case["category"] not in route["case_categories"]:
            continue
        if not case_version.intersection(route["protocol_versions"]):
            continue
        declared_targets = set(route["normative_target_ids"])
        if "*" not in declared_targets and not targets.intersection(declared_targets):
            continue
        candidates.append(route)
    return candidates


def iter_cells(catalog: dict, manifest: dict, *, languages: tuple[str, ...] = LANGUAGES) -> Iterator[dict]:
    """Yield construction, execution, and response-decode obligations separately."""
    operation_ids = _operation_ids(catalog)
    execution_routes = {route["execution_route_id"]: route for route in manifest.get("execution_routes", [])}
    for case in catalog["test_cases"]:
        versions = case["execution"]["protocol_versions"]
        for language in languages:
            routes = applicable_routes(case, manifest, language, operation_ids)
            obligations = _case_obligations(case, operation_ids)
            if not obligations:
                unavailable = _unavailable_for_case(manifest, language, set())
                if unavailable is None:
                    raise ValueError(f"case has no normative target obligation: {case['case_id']}")
            for version in versions:
                for obligation in obligations:
                    target_id = obligation["target_id"]
                    target_routes = [route for route in routes
                                     if version in route["protocol_versions"]
                                     and ("*" in route["normative_target_ids"]
                                          or target_id in route["normative_target_ids"])]
                    if not target_routes and _unavailable_for_target(manifest, language, target_id) is None:
                        raise ValueError(
                            f"no public construction route or source-backed limitation for "
                            f"{case['case_id']} target {target_id} ({language}, KMIP {version})"
                        )
            for route in routes:
                for version in versions:
                    if version not in route["protocol_versions"]:
                        continue
                    route_obligations = [obligation for obligation in obligations
                                         if "*" in route["normative_target_ids"]
                                         or obligation["target_id"] in route["normative_target_ids"]]
                    if not route_obligations:
                        continue
                    for combination in route["combinations"]:
                        for obligation in route_obligations:
                            base = {
                                "case_id": case["case_id"],
                                "language": language,
                                "route_id": route["route_id"],
                                **obligation,
                                "protocol_version": version,
                                "encoding": combination["encoding"],
                                "transport": combination["transport"],
                                "mode": route["mode"],
                                "public_api_symbol": route["public_api_symbol"],
                                "artifact_commit": route["artifact_commit"],
                                "execution_status": "required",
                                "evidence_collected": False,
                            }
                            if route["mode"] == "local":
                                for action in route["actions"]:
                                    yield {**base, "action": action, "execution_route_id": None,
                                           "execution_api_symbol": None}
                                continue

                            execution_route_id = route.get("execution_route_id")
                            execution = execution_routes.get(execution_route_id)
                            if execution is None or execution.get("language") != language:
                                raise ValueError(
                                    f"network construction route has no matching public executor: {route['route_id']}"
                                )
                            for action in route["actions"]:
                                if action != "construct":
                                    raise ValueError(f"network construction route has an invalid action: {action}")
                                yield {**base, "action": "construct", "execution_route_id": execution_route_id,
                                       "execution_api_symbol": execution["public_api_symbol"]}
                            for action in ("send", "decode"):
                                yield {**base, "action": action, "execution_route_id": execution_route_id,
                                       "execution_api_symbol": execution["public_api_symbol"]}


def build_matrix(catalog: dict, manifest: dict) -> dict:
    """Return grouped routes and coverage counts without duplicating every cell."""
    cell_count = 0
    seen = set()
    case_language_coverage = set()
    construction_route_cell_counts: dict[str, int] = {}
    execution_route_cell_counts: dict[str, int] = {}
    digest = hashlib.sha256()
    for cell in iter_cells(catalog, manifest):
        key = (cell["case_id"], cell["language"], cell["route_id"], cell.get("target_id"),
               cell.get("workflow_step_index"), cell.get("workflow_item_index"),
               cell["execution_route_id"], cell["action"], cell["protocol_version"],
               cell["encoding"], cell["transport"])
        if key in seen:
            raise ValueError(f"duplicate coverage matrix cell: {key}")
        seen.add(key)
        digest.update(json.dumps(cell, sort_keys=True, separators=(",", ":")).encode("utf-8"))
        digest.update(b"\n")
        cell_count += 1
        case_language_coverage.add((cell["case_id"], cell["language"]))
        construction_route_cell_counts[cell["route_id"]] = construction_route_cell_counts.get(cell["route_id"], 0) + 1
        if cell["execution_route_id"]:
            route_id = cell["execution_route_id"]
            execution_route_cell_counts[route_id] = execution_route_cell_counts.get(route_id, 0) + 1
    required = {(case["case_id"], language) for case in catalog["test_cases"] for language in LANGUAGES}
    missing = sorted(required - case_language_coverage)
    if missing:
        raise ValueError(f"cases lack applicable public API coverage: {missing[:20]!r}")
    empty_construction_routes = sorted(route["route_id"] for route in manifest["routes"]
                                       if route["route_id"] not in construction_route_cell_counts)
    if empty_construction_routes:
        raise ValueError(f"public construction routes lack an executable normative case: {empty_construction_routes[:20]!r}")
    empty_execution_routes = sorted(route["execution_route_id"] for route in manifest.get("execution_routes", [])
                                    if route["execution_route_id"] not in execution_route_cell_counts)
    if empty_execution_routes:
        raise ValueError(f"public execution routes lack a required call: {empty_execution_routes[:20]!r}")
    return {
        "schema_version": "1.0",
        "matrix_kind": "planned_coverage_obligations",
        "execution_evidence_collected": False,
        "catalog_version": catalog["version"],
        "library_commit": manifest["library_commit"],
        "matrix_sha256": digest.hexdigest(),
        "case_count": len(catalog["test_cases"]),
        "route_count": len(manifest["routes"]) + len(manifest.get("execution_routes", [])),
        "public_api_route_count": len(manifest["routes"]),
        "execution_route_count": len(manifest.get("execution_routes", [])),
        "cell_count": cell_count,
        "case_language_count": len(case_language_coverage),
        "coverage_by_language": {
            language: sum(1 for case_id, case_language in case_language_coverage if case_language == language)
            for language in LANGUAGES
        },
        "public_api_routes_without_cells": [],
        "execution_routes_without_cells": [],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--jsonl", action="store_true", help="stream every case/route/version/encoding/transport cell")
    parser.add_argument("--output", type=Path, help="write deterministic JSONL rows atomically (requires --jsonl)")
    args = parser.parse_args()
    if args.output is not None and not args.jsonl:
        parser.error("--output requires --jsonl")
    output_path = (resolve_within(args.output, (args.root, Path(tempfile.gettempdir())))
                   if args.output is not None else None)
    catalog = json.loads((args.root / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
    manifest = json.loads((args.root / "interop/shared/coverage/route-capabilities.json").read_text())
    if args.jsonl:
        if output_path is not None:
            summary = build_matrix(catalog, manifest)
            try:
                import jsonschema
            except ImportError as error:
                raise SystemExit("writing coverage JSONL requires the CI jsonschema dependency") from error
            row_schema = json.loads((args.root / "interop/shared/schemas/coverage-matrix-row.schema.json").read_text())
            validator = jsonschema.Draft202012Validator(row_schema)
            validator.check_schema(row_schema)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            descriptor, temporary = tempfile.mkstemp(prefix=f".{output_path.name}.", dir=output_path.parent)
            try:
                emitted_digest = hashlib.sha256()
                emitted_count = 0
                with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as output:
                    for cell in iter_cells(catalog, manifest):
                        validator.validate(cell)
                        line = json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
                        output.write(line)
                        emitted_digest.update(line.encode("utf-8"))
                        emitted_count += 1
                    output.flush()
                    os.fsync(output.fileno())
                if emitted_count != summary["cell_count"] or emitted_digest.hexdigest() != summary["matrix_sha256"]:
                    raise ValueError("emitted coverage JSONL does not match the deterministic matrix summary")
                os.replace(temporary, output_path)  # NOSONAR: normalized to the repository or system temp directory above
            except BaseException:
                try:
                    os.unlink(temporary)
                except FileNotFoundError:
                    pass
                raise
            print(json.dumps(summary, indent=2, sort_keys=True))
            return
        for cell in iter_cells(catalog, manifest):
            print(json.dumps(cell, sort_keys=True, separators=(",", ":")))
    else:
        print(json.dumps(build_matrix(catalog, manifest), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
