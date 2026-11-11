#!/usr/bin/env python3
"""Validate that a consumer report contains real calls for every planned route action."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import tempfile

from cli_paths import resolve_within
from coverage_matrix import iter_cells


LANGUAGES = ("rust", "c", "java", "python")
VERSIONS = {"2.0", "2.1"}
ENCODINGS = {"TTLV", "JSON", "XML"}
TRANSPORTS = {"raw_tls", "https"}


def _key(record: dict) -> tuple:
    return (
        record["case_id"], record["route_id"], record["target_id"],
        record.get("workflow_step_index"), record.get("workflow_item_index"),
        record.get("execution_route_id"), record["action"], record["protocol_version"],
        record["encoding"], record["transport"],
    )


def _event_outcome(record: dict) -> dict:
    """Fields that must match when the same route cell appears in two runs."""
    return {field: value for field, value in record.items() if field != "invocation_id"}


def _expected_batch_operations(case: dict) -> list[str] | None:
    scenario = case.get("requires", {}).get("protocol_scenario", {})
    if not isinstance(scenario, dict):
        return None
    parameters = scenario.get("parameters", {})
    items = parameters.get("items", []) if isinstance(parameters, dict) else []
    if items:
        operations = []
        for item in items:
            if isinstance(item, str) and item:
                operations.append(item.split("(", 1)[0].strip())
            elif isinstance(item, dict) and isinstance(item.get("operation"), str) and item["operation"]:
                operations.append(item["operation"])
            else:
                raise ValueError("catalog batch items must contain an operation name")
        return operations
    ordered = parameters.get("operations_in_request_order", []) if isinstance(parameters, dict) else []
    if not ordered:
        expected = scenario.get("expected", {})
        ordered = expected.get("operation_order", []) if isinstance(expected, dict) else []
    if not ordered and len(scenario.get("operations", [])) > 1:
        ordered = scenario["operations"]
    return list(ordered) if len(ordered) > 1 else None


def _ordinal(index: int) -> str:
    return {1: "first", 2: "second", 3: "third", 4: "fourth", 5: "fifth"}.get(index, f"item_{index}")


def _expected_batch_result_codes(case: dict, catalog: dict, result_field: str) -> list[int] | None:
    """Resolve ordered scenario outcomes through the catalog's KMIP enum values."""
    scenario = case.get("requires", {}).get("protocol_scenario", {})
    expected = scenario.get("expected", {}) if isinstance(scenario, dict) else {}
    if not isinstance(expected, dict):
        return None
    names = []
    index = 1
    while True:
        base = _ordinal(index)
        key = base if result_field == "status" else f"{base}_reason"
        if expected.get(key) is None:
            break
        names.append(expected[key])
        index += 1
    if not names:
        return None

    enum_name = "RESULT-STATUS" if result_field == "status" else "RESULT-REASON"
    enum_values = {}
    for target in catalog.get("normative_targets", []):
        if target.get("kind") != "enumeration_value":
            continue
        if not any(enum_name in parent for parent in target.get("parent_element_ids", [])):
            continue
        normalized_name = re.sub(r"[^a-z0-9]", "", target.get("name", "").casefold())
        try:
            enum_values[normalized_name] = int(target["wire_value"], 16)
        except (KeyError, TypeError, ValueError):
            continue
    values = []
    for name in names:
        if not isinstance(name, str):
            raise ValueError(f"catalog batch expected {result_field} values must be names")
        normalized_name = re.sub(r"[^a-z0-9]", "", name.casefold())
        if normalized_name not in enum_values:
            raise ValueError(f"catalog batch expected {result_field} is not a known KMIP enum value: {name}")
        values.append(enum_values[normalized_name])
    return values


def _validate_batch_event(event: dict, case: dict, catalog: dict, key: tuple) -> None:
    expected_operations = _expected_batch_operations(case)
    if expected_operations is None:
        return
    if event.get("batch_operation_sequence") != expected_operations:
        raise ValueError(f"batch request operation order does not match the catalog: {key}")
    if event.get("batch_item_count") != len(expected_operations):
        raise ValueError(f"batch item count does not match the catalog: {key}")
    if event.get("status", "passed") != "passed":
        return
    if event.get("peer_observed_operation_sequence") != expected_operations:
        raise ValueError(f"controlled peer did not observe the catalog batch order: {key}")
    scenario = case.get("requires", {}).get("protocol_scenario", {})
    expected = scenario.get("expected", {}) if isinstance(scenario, dict) else {}
    expected_responses = expected.get("response_count", len(expected_operations))
    if event.get("peer_observed_response_count") != expected_responses:
        raise ValueError(f"controlled peer response count does not match the catalog: {key}")
    for field, evidence_field in (
        ("status", "peer_observed_result_status_codes"),
        ("reason", "peer_observed_result_reason_codes"),
    ):
        expected_codes = _expected_batch_result_codes(case, catalog, field)
        if expected_codes is not None and event.get(evidence_field) != expected_codes:
            raise ValueError(f"controlled peer per-item {field} outcomes do not match the catalog: {key}")
    expected_effects = expected.get("effects")
    if expected_effects is not None and event.get("peer_observed_effects") != expected_effects:
        raise ValueError(f"controlled peer per-item effects do not match the catalog: {key}")


def _expected_cells(report: dict, catalog: dict, manifest: dict) -> dict[tuple, dict]:
    language = report["language"]
    selected_versions = set(report["protocol_versions"])
    network_combinations = {
        (item["encoding"], item["transport"]) for item in report["network_combinations"]
    }
    cells = {}
    for cell in iter_cells(catalog, manifest, languages=(language,)):
        if cell["language"] != language or cell["protocol_version"] not in selected_versions:
            continue
        if cell["transport"] is not None and (cell["encoding"], cell["transport"]) not in network_combinations:
            continue
        cells[_key(cell)] = cell
    return cells


def validate_report(report: dict, catalog: dict, manifest: dict, *,
                    catalog_sha256: str, route_manifest_sha256: str,
                    expected_artifact_sha256: str) -> dict:
    """Validate one runner-emitted evidence report against installed provenance."""
    if report.get("schema_version") != "1.0":
        raise ValueError("route evidence schema_version must be 1.0")
    language = report.get("language")
    if language not in LANGUAGES:
        raise ValueError("route evidence language is invalid")
    server = report.get("server")
    if not isinstance(server, dict) or any(not isinstance(server.get(field), str) or not server[field]
                                           for field in ("id", "product", "version")):
        raise ValueError("route evidence requires a stable server identity and version")
    if not re.fullmatch(r"[0-9a-f]{64}", str(server.get("configuration_fingerprint", ""))):
        raise ValueError("route evidence server configuration fingerprint is invalid")
    if report.get("library_commit") != manifest.get("library_commit"):
        raise ValueError("route evidence library commit does not match the route manifest")
    artifact_sha = report.get("artifact_sha256", "")
    if not re.fullmatch(r"[0-9a-f]{64}", expected_artifact_sha256):
        raise ValueError("installed artifact SHA-256 must be a lowercase SHA-256")
    if artifact_sha != expected_artifact_sha256:
        raise ValueError("route evidence artifact does not match the installed artifact")
    if report.get("catalog_sha256") != catalog_sha256:
        raise ValueError("route evidence catalog SHA-256 does not match the catalog")
    if report.get("route_manifest_sha256") != route_manifest_sha256:
        raise ValueError("route evidence manifest SHA-256 does not match the route manifest")

    selected_versions = report.get("protocol_versions")
    if not isinstance(selected_versions, list) or not selected_versions or not set(selected_versions) <= VERSIONS:
        raise ValueError("route evidence protocol_versions must select KMIP 2.0 and/or 2.1")
    if len(set(selected_versions)) != len(selected_versions):
        raise ValueError("route evidence protocol_versions contains duplicates")
    if not set(selected_versions) <= set(catalog.get("target_versions", [])):
        raise ValueError("route evidence selects a version absent from the catalog")

    combinations = report.get("network_combinations")
    if not isinstance(combinations, list) or not combinations:
        raise ValueError("route evidence network_combinations must select at least one supported pair")
    seen_combinations = set()
    for combination in combinations:
        if not isinstance(combination, dict):
            raise ValueError("network combination must be an object")
        key = combination.get("encoding"), combination.get("transport")
        if key[0] not in ENCODINGS or key[1] not in TRANSPORTS:
            raise ValueError("network combination has an invalid encoding or transport")
        if key in seen_combinations:
            raise ValueError("network_combinations contains duplicates")
        seen_combinations.add(key)
    supported_combinations = {
        (combination["encoding"], combination["transport"])
        for route in manifest["routes"]
        if route["language"] == language and route["mode"] == "network"
        for combination in route["combinations"]
    }
    if not seen_combinations <= supported_combinations:
        raise ValueError("route evidence selects an encoding/transport pair absent from the public API")

    expected = _expected_cells(report, catalog, manifest)
    if not expected:
        raise ValueError("route evidence selection has no applicable route cells")
    events = report.get("events")
    if not isinstance(events, list):
        raise ValueError("route evidence events must be an array")
    observed = {}
    case_by_id = {case["case_id"]: case for case in catalog["test_cases"]}
    for event in events:
        if not isinstance(event, dict):
            raise ValueError("route evidence event must be an object")
        try:
            key = _key(event)
        except KeyError as error:
            raise ValueError(f"route evidence event is missing {error.args[0]}") from error
        if key not in expected:
            raise ValueError(f"unplanned route execution cell: {key}")
        if key in observed:
            raise ValueError(f"duplicate route execution cell: {key}")
        status = event.get("status")
        if status not in {"passed", "failed", "not_run"}:
            raise ValueError(f"route execution status is invalid for {key}")
        count = event.get("invocation_count")
        if not isinstance(count, int) or isinstance(count, bool) or count < 0:
            raise ValueError(f"route execution invocation_count is invalid for {key}")
        if status == "not_run" or count == 0:
            raise ValueError(f"route execution cell was not actually invoked: {key}")
        if count != 1:
            raise ValueError(f"route execution invocation_count is invalid for {key}")
        if status == "failed" and (not isinstance(event.get("failure_code"), str) or not event["failure_code"]):
            raise ValueError(f"failed route execution requires a sanitized failure_code: {key}")
        planned = expected[key]
        if event.get("target_id") != planned["target_id"]:
            raise ValueError(f"normative target does not match the planned route cell: {key}")
        for field in ("workflow_step_index", "workflow_operation", "workflow_item_index"):
            if event.get(field) != planned.get(field):
                raise ValueError(f"workflow identity does not match the planned route cell: {key}")
        if event.get("public_api_symbol") != planned["public_api_symbol"]:
            raise ValueError(f"public API symbol does not match the planned route: {key}")
        if event.get("execution_api_symbol") != planned.get("execution_api_symbol"):
            raise ValueError(f"execution API symbol does not match the public executor: {key}")
        if event.get("invocation_id") is None or not isinstance(event["invocation_id"], str) or not event["invocation_id"]:
            raise ValueError(f"route execution event requires an invocation_id: {key}")
        if status == "passed":
            hash_field = "response_tree_sha256" if event["action"] == "decode" else "request_tree_sha256"
            tree_hash = event.get(hash_field)
            if not isinstance(tree_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", tree_hash):
                expected_hash = "response tree hash" if event["action"] == "decode" else "request tree hash"
                raise ValueError(f"passed {event['action']} event requires a valid {expected_hash}: {key}")
        if "wire_bytes_deterministic" in event and not isinstance(event["wire_bytes_deterministic"], bool):
            raise ValueError(f"wire_bytes_deterministic must be a boolean: {key}")
        if event.get("wire_bytes_deterministic") is True:
            wire_hash = event.get("wire_bytes_sha256")
            if not isinstance(wire_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", wire_hash):
                raise ValueError(f"deterministic wire bytes require a valid SHA-256: {key}")
        if event["action"] == "send":
            _validate_batch_event(event, case_by_id[event["case_id"]], catalog, key)
        observed[key] = event

    missing = sorted(set(expected) - set(observed))
    if missing:
        raise ValueError(f"missing route execution cells: {missing[:10]!r}")
    failed = sum(event["status"] == "failed" for event in observed.values())
    all_versions = set(catalog.get("target_versions", []))
    all_network_combinations = {
        (combination["encoding"], combination["transport"])
        for route in manifest["routes"] if route["mode"] == "network"
        for combination in route["combinations"]
    }
    complete_matrix = (set(selected_versions) == all_versions and seen_combinations == all_network_combinations)
    return {
        "run_id": report["run_id"],
        "language": language,
        "expected_cells": len(expected),
        "invoked_cells": sum(event["invocation_count"] > 0 for event in observed.values()),
        "failed_cells": failed,
        "selected_scope_complete": True,
        "complete_matrix": complete_matrix,
        "all_invocations_passed": failed == 0,
    }


def validate_collection(reports: list[dict], catalog: dict, manifest: dict, *,
                        catalog_sha256: str, route_manifest_sha256: str,
                        expected_artifact_sha256: str) -> dict:
    """Require selected run reports collectively to cover the complete matrix."""
    if not reports:
        raise ValueError("route evidence collection must include at least one report")
    summaries = [validate_report(
        report, catalog, manifest, catalog_sha256=catalog_sha256,
        route_manifest_sha256=route_manifest_sha256,
        expected_artifact_sha256=expected_artifact_sha256,
    ) for report in reports]
    languages = {summary["language"] for summary in summaries}
    run_ids = [summary["run_id"] for summary in summaries]
    if len(languages) != 1:
        raise ValueError("route evidence collection must contain one language")
    if len(run_ids) != len(set(run_ids)):
        raise ValueError("route evidence collection contains duplicate run_id values")
    server_identities = {
        (report["server"]["id"], report["server"]["product"], report["server"]["version"],
         report["server"]["configuration_fingerprint"])
        for report in reports
    }
    if len(server_identities) != 1:
        identities = {(item[0], item[1], item[2]) for item in server_identities}
        if len(identities) == 1:
            raise ValueError("route evidence collection contains different server configurations")
        raise ValueError("route evidence collection contains different KMIP servers or server versions")

    language = next(iter(languages))
    all_versions = set(catalog.get("target_versions", []))
    all_network_combinations = {
        (combination["encoding"], combination["transport"])
        for route in manifest["routes"] if route["mode"] == "network"
        for combination in route["combinations"]
    }
    full_selection = {
        "schema_version": "1.0", "run_id": "full-matrix", "language": language,
        "library_commit": manifest["library_commit"], "artifact_sha256": expected_artifact_sha256,
        "catalog_sha256": catalog_sha256, "route_manifest_sha256": route_manifest_sha256,
        "protocol_versions": sorted(all_versions),
        "network_combinations": [
            {"encoding": encoding, "transport": transport}
            for encoding, transport in sorted(all_network_combinations)
        ],
        "events": [],
    }
    expected = _expected_cells(full_selection, catalog, manifest)
    observations: dict[tuple, dict] = {}
    duplicate_identical = 0
    for report in reports:
        for event in report["events"]:
            key = _key(event)
            previous = observations.get(key)
            if previous is None:
                observations[key] = event
                continue
            if _event_outcome(previous) != _event_outcome(event):
                raise ValueError(f"conflicting repeated route execution cell: {key}")
            duplicate_identical += 1
    observed = set(observations)
    missing = sorted(set(expected) - observed)
    if missing:
        raise ValueError(f"route evidence collection misses full matrix cells: {missing[:10]!r}")
    return {
        "language": language,
        "server": {"id": reports[0]["server"]["id"],
                   "product": reports[0]["server"]["product"],
                   "version": reports[0]["server"]["version"],
                   "configuration_fingerprint": reports[0]["server"]["configuration_fingerprint"]},
        "library_commit": manifest["library_commit"],
        "artifact_sha256": expected_artifact_sha256,
        "catalog_sha256": catalog_sha256,
        "route_manifest_sha256": route_manifest_sha256,
        "run_ids": run_ids,
        "expected_cells": len(expected),
        "observed_cells": len(observed),
        "duplicate_identical_cell_events": duplicate_identical,
        "selected_scope_complete": True,
        "complete_matrix": True,
        "all_invocations_passed": all(summary["all_invocations_passed"] for summary in summaries),
    }


def validate_parity_collection(reports: list[dict], catalog: dict, manifest: dict, *,
                               catalog_sha256: str, route_manifest_sha256: str,
                               expected_artifact_sha256_by_language: dict[str, str]) -> dict:
    """Require full per-language matrices and compare normalized trees across routes/languages."""
    by_language: dict[str, list[dict]] = {language: [] for language in LANGUAGES}
    for report in reports:
        language = report.get("language")
        if language not in by_language:
            raise ValueError("route parity collection contains an invalid language")
        by_language[language].append(report)
    if any(not values for values in by_language.values()):
        raise ValueError("route parity collection requires Rust, C, Java, and Python reports")

    summaries = {}
    for language, language_reports in by_language.items():
        expected_artifact = expected_artifact_sha256_by_language.get(language)
        if expected_artifact is None:
            raise ValueError(f"route parity collection is missing the installed {language} artifact digest")
        summaries[language] = validate_collection(
            language_reports, catalog, manifest, catalog_sha256=catalog_sha256,
            route_manifest_sha256=route_manifest_sha256,
            expected_artifact_sha256=expected_artifact,
        )

    identities = {
        (report["server"]["id"], report["server"]["product"], report["server"]["version"],
         report["server"]["configuration_fingerprint"], report["library_commit"],
         report["catalog_sha256"], report["route_manifest_sha256"])
        for report in reports
    }
    if len(identities) != 1:
        raise ValueError("route parity reports use different server configurations or contract inputs")

    constructed: dict[tuple, set[str]] = {}
    sent: dict[tuple, set[str]] = {}
    for report in reports:
        for event in report["events"]:
            if event.get("status") != "passed" or event.get("action") not in {"construct", "send"}:
                continue
            identity = (
                report["language"], event["case_id"], event["route_id"], event["target_id"],
                event.get("workflow_step_index"), event.get("workflow_operation"),
                event.get("workflow_item_index"), event["protocol_version"],
                event["encoding"], event["transport"],
            )
            destination = constructed if event["action"] == "construct" else sent
            destination.setdefault(identity, set()).add(event["request_tree_sha256"])
    for identity in constructed.keys() & sent.keys():
        if constructed[identity] != sent[identity]:
            raise ValueError(f"constructed request tree differs from the request sent: {identity}")

    grouped: dict[tuple, dict[str, list[dict]]] = {}
    for language, language_reports in by_language.items():
        for report in language_reports:
            for event in report["events"]:
                tree_field = "response_tree_sha256" if event["action"] == "decode" else "request_tree_sha256"
                if event.get("status") != "passed" or not event.get(tree_field):
                    continue
                key = (
                    event["case_id"], event["target_id"], event.get("workflow_step_index"),
                    event.get("workflow_operation"), event.get("workflow_item_index"),
                    event["action"], event["protocol_version"], event["encoding"], event["transport"],
                )
                grouped.setdefault(key, {}).setdefault(language, []).append(event)

    compared = 0
    fully_compared = 0
    byte_compared = 0
    groups_without_cross_language_comparison = 0
    for key, language_events in grouped.items():
        per_language_hashes = {
            language: {event["response_tree_sha256"] if key[5] == "decode" else event["request_tree_sha256"]
                       for event in events}
            for language, events in language_events.items()
        }
        if any(len(hashes) > 1 for hashes in per_language_hashes.values()):
            raise ValueError(f"constructor routes disagree on normalized KMIP trees: {key}")
        if len(per_language_hashes) > 1:
            compared += 1
            flattened = set().union(*per_language_hashes.values())
            if len(flattened) > 1:
                raise ValueError(f"languages disagree on normalized KMIP trees: {key}")
            if len(per_language_hashes) == len(LANGUAGES):
                fully_compared += 1
            else:
                groups_without_cross_language_comparison += 1
        else:
            groups_without_cross_language_comparison += 1

        events = [event for group in language_events.values() for event in group]
        if events and all(event.get("wire_bytes_deterministic") is True
                          and isinstance(event.get("wire_bytes_sha256"), str) for event in events):
            if len({event["wire_bytes_sha256"] for event in events}) > 1:
                raise ValueError(f"deterministic wire encodings disagree for identical inputs: {key}")
            byte_compared += 1

    if not compared:
        raise ValueError("route parity collection has no matching normalized tree groups")
    return {
        "languages": list(LANGUAGES),
        "server": {"id": reports[0]["server"]["id"], "product": reports[0]["server"]["product"],
                   "version": reports[0]["server"]["version"],
                   "configuration_fingerprint": reports[0]["server"]["configuration_fingerprint"]},
        "language_matrices": summaries,
        "parity_groups_compared": compared,
        "parity_groups_all_languages": fully_compared,
        "parity_groups_without_all_languages": groups_without_cross_language_comparison,
        "deterministic_wire_groups_compared": byte_compared,
        "all_invocations_passed": all(summary["all_invocations_passed"] for summary in summaries.values()),
        "parity_coverage_complete": groups_without_cross_language_comparison == 0,
        "tree_parity_passed": True,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--report", type=Path, action="append", required=True,
                        help="runner-emitted JSON report; repeat to combine protocol/encoding/transport runs")
    parser.add_argument("--artifact-sha256", required=True, action="append",
                        help="installed package digest; use language=SHA256 pairs with --parity")
    parser.add_argument("--parity", action="store_true",
                        help="validate all four full language matrices and compare normalized trees")
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    catalog_path = root / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
    manifest_path = root / "interop/shared/coverage/route-capabilities.json"
    catalog_bytes = catalog_path.read_bytes()
    manifest_bytes = manifest_path.read_bytes()
    report_paths = [resolve_within(path, (root, Path(tempfile.gettempdir())), must_exist=True)
                    for path in args.report]
    reports = [json.loads(path.read_text(encoding="utf-8")) for path in report_paths]  # NOSONAR: report paths are resolved under the repository or system temp directory
    catalog = json.loads(catalog_bytes)
    manifest = json.loads(manifest_bytes)
    catalog_sha = hashlib.sha256(catalog_bytes).hexdigest()
    route_sha = hashlib.sha256(manifest_bytes).hexdigest()
    if args.parity:
        artifacts = {}
        for value in args.artifact_sha256:
            if "=" not in value:
                parser.error("--parity requires --artifact-sha256 language=SHA256 for each language")
            language, digest = value.split("=", 1)
            if language in artifacts:
                parser.error(f"duplicate artifact digest for {language}")
            artifacts[language] = digest
        summary = validate_parity_collection(
            reports, catalog, manifest, catalog_sha256=catalog_sha, route_manifest_sha256=route_sha,
            expected_artifact_sha256_by_language=artifacts,
        )
    else:
        if len(args.artifact_sha256) != 1 or "=" in args.artifact_sha256[0]:
            parser.error("single-language validation requires one --artifact-sha256 SHA256 value")
        summary = validate_collection(
            reports, catalog, manifest, catalog_sha256=catalog_sha,
            route_manifest_sha256=route_sha,
            expected_artifact_sha256=args.artifact_sha256[0],
        )
    print(json.dumps(summary, sort_keys=True))
    if (not summary["all_invocations_passed"]
            or (args.parity and not summary["parity_coverage_complete"])):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
