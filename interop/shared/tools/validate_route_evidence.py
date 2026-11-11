#!/usr/bin/env python3
"""Validate that a consumer report contains real calls for every planned route action."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import tempfile
from collections import defaultdict

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


def _expects_tag_decode_rejection(cell: dict, case: dict) -> bool:
    requires = case.get("requires", {})
    if (cell.get("action") != "decode" or cell.get("mode") != "local"
            or ".generic-ttlv." not in f".{cell.get('route_id', '')}."):
        return False
    if requires.get("target_kind") not in {"tag", "tag_range"}:
        return False
    if requires.get("allocation") not in {"reserved", "unused"}:
        return False
    symbol = re.sub(r"[^a-z0-9]", "", cell.get("public_api_symbol", "").casefold())
    return "codecdecode" in symbol or symbol.endswith("ttlvdecode")


def _request_identity(record: dict) -> tuple:
    """Identify a request across its construction and send action rows."""
    return (
        record["case_id"], record["route_id"], record["target_id"],
        record.get("workflow_step_index"), record.get("workflow_operation"),
        record.get("workflow_item_index"), record.get("execution_route_id"),
        record["protocol_version"], record["encoding"], record["transport"],
    )


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


def _selected_case_ids(report: dict, catalog: dict) -> set[str] | None:
    selected_case_ids = report.get("case_ids")
    if selected_case_ids is None:
        return None
    if not isinstance(selected_case_ids, list) or not selected_case_ids or any(
            not isinstance(case_id, str) or not case_id for case_id in selected_case_ids):
        raise ValueError("route evidence case_ids must be a non-empty array of case IDs")
    if len(set(selected_case_ids)) != len(selected_case_ids):
        raise ValueError("route evidence case_ids contains duplicates")
    all_case_ids = {case["case_id"] for case in catalog.get("test_cases", [])}
    unknown_case_ids = set(selected_case_ids) - all_case_ids
    if unknown_case_ids:
        raise ValueError(f"route evidence contains unknown case_ids: {sorted(unknown_case_ids)!r}")
    return set(selected_case_ids)


def _expected_cells(report: dict, catalog: dict, manifest: dict) -> dict[tuple, dict]:
    language = report["language"]
    selected_versions = set(report["protocol_versions"])
    selected_case_ids = _selected_case_ids(report, catalog)
    network_combinations = {
        (item["encoding"], item["transport"]) for item in report["network_combinations"]
    }
    cells = {}
    for cell in iter_cells(catalog, manifest, languages=(language,)):
        if cell["language"] != language or cell["protocol_version"] not in selected_versions:
            continue
        if selected_case_ids is not None and cell["case_id"] not in selected_case_ids:
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

    all_case_ids = {case["case_id"] for case in catalog.get("test_cases", [])}
    selected_case_ids = _selected_case_ids(report, catalog)

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
            case = case_by_id[event["case_id"]]
            if event["action"] == "decode":
                expects_rejection = _expects_tag_decode_rejection(planned, case)
                outcome = event.get("decode_outcome")
                if outcome == "rejected" or expects_rejection:
                    if not expects_rejection:
                        raise ValueError(f"decode rejection is not expected for this route cell: {key}")
                    if event.get("decode_outcome") != "rejected":
                        raise ValueError(f"reserved or unused tag decoder did not report its expected rejection: {key}")
                    if event.get("decode_rejection_code") != "tag_not_allocated":
                        raise ValueError(f"reserved or unused tag decoder reported the wrong rejection code: {key}")
                    input_hash = event.get("decode_input_sha256")
                    if not isinstance(input_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", input_hash):
                        raise ValueError(f"expected decode rejection requires a valid input SHA-256: {key}")
                    if "response_tree_sha256" in event:
                        raise ValueError(f"rejected TTLV input cannot have a decoded response tree: {key}")
                else:
                    if outcome not in {None, "decoded"}:
                        raise ValueError(f"decode_outcome is invalid for this route cell: {key}")
                    tree_hash = event.get("response_tree_sha256")
                    if not isinstance(tree_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", tree_hash):
                        raise ValueError(f"passed decode event requires a valid response tree hash: {key}")
            else:
                tree_hash = event.get("request_tree_sha256")
                if not isinstance(tree_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", tree_hash):
                    raise ValueError(f"passed {event['action']} event requires a valid request tree hash: {key}")
        if "wire_bytes_deterministic" in event and not isinstance(event["wire_bytes_deterministic"], bool):
            raise ValueError(f"wire_bytes_deterministic must be a boolean: {key}")
        if event.get("wire_bytes_deterministic") is True:
            wire_hash = event.get("wire_bytes_sha256")
            if not isinstance(wire_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", wire_hash):
                raise ValueError(f"deterministic wire bytes require a valid SHA-256: {key}")
        if event["action"] == "send":
            case = case_by_id[event["case_id"]]
            expected_status = str(case.get("requires", {}).get("expected_status", "")).casefold()
            if expected_status.replace("_", "") == "notsent":
                if event.get("request_delivery_state") != "not_sent":
                    raise ValueError(f"version-boundary send did not prove the request remained unsent: {key}")
                if event.get("transport_invocation_count") != 0:
                    raise ValueError(f"version-boundary request invoked the transport: {key}")
            elif event.get("request_delivery_state") == "not_sent":
                raise ValueError(f"unexpected not-sent result for a network route cell: {key}")
            _validate_batch_event(event, case, catalog, key)
        observed[key] = event

    constructed = {
        _request_identity(event): event["request_tree_sha256"]
        for event in observed.values() if event["action"] == "construct" and event.get("status") == "passed"
    }
    for event in observed.values():
        if event["action"] != "send" or event.get("status") != "passed":
            continue
        constructed_hash = constructed.get(_request_identity(event))
        if constructed_hash is not None and constructed_hash != event.get("request_tree_sha256"):
            raise ValueError(f"constructed request tree differs from sent request: {_request_identity(event)}")

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
    complete_matrix = (set(selected_versions) == all_versions and seen_combinations == all_network_combinations
                       and (selected_case_ids is None or selected_case_ids == all_case_ids))
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


def build_evidence_ledger(report: dict, catalog: dict, manifest: dict, *, catalog_sha256: str,
                          route_manifest_sha256: str, expected_artifact_sha256: str) -> dict:
    """Validate one selected run and summarize all planned cells as executed or pending."""
    summary = validate_report(
        report, catalog, manifest, catalog_sha256=catalog_sha256,
        route_manifest_sha256=route_manifest_sha256,
        expected_artifact_sha256=expected_artifact_sha256,
    )
    language = report["language"]
    expected = list(iter_cells(catalog, manifest, languages=(language,)))
    expected_by_key = {_key(cell): cell for cell in expected}
    observed_by_key = {_key(event): event for event in report["events"]}
    unexpected = set(observed_by_key) - set(expected_by_key)
    if unexpected:
        raise ValueError(f"route evidence ledger contains unplanned cells: {sorted(unexpected)[:10]!r}")

    route_counts: dict[str, dict[str, int]] = defaultdict(lambda: {"required_cells": 0, "executed_cells": 0})
    shape_counts: dict[tuple, dict[str, int]] = defaultdict(lambda: {"required_cells": 0, "executed_cells": 0})
    pending_by_case: dict[str, int] = defaultdict(int)
    for key, cell in expected_by_key.items():
        route_counts[cell["route_id"]]["required_cells"] += 1
        shape = (cell["route_id"], cell["action"], cell["protocol_version"],
                 cell["encoding"], cell["transport"])
        shape_counts[shape]["required_cells"] += 1
        if key in observed_by_key:
            route_counts[cell["route_id"]]["executed_cells"] += 1
            shape_counts[shape]["executed_cells"] += 1
        else:
            pending_by_case[cell["case_id"]] += 1

    route_summary = []
    for route_id, counts in sorted(route_counts.items()):
        route_summary.append({**counts, "route_id": route_id,
                              "pending_cells": counts["required_cells"] - counts["executed_cells"]})
    shape_summary = []
    for (route_id, action, version, encoding, transport), counts in sorted(
            shape_counts.items(), key=lambda item: tuple("" if value is None else str(value)
                                                         for value in item[0])):
        shape_summary.append({
            "route_id": route_id, "action": action, "protocol_version": version,
            "encoding": encoding, "transport": transport, **counts,
            "pending_cells": counts["required_cells"] - counts["executed_cells"],
        })

    pending_case_ids = sorted(pending_by_case)
    executed = len(observed_by_key)
    pending = len(expected_by_key) - executed
    return {
        "schema_version": "1.0",
        "language": language,
        "library_commit": report["library_commit"],
        "artifact_sha256": report["artifact_sha256"],
        "catalog_sha256": catalog_sha256,
        "route_manifest_sha256": route_manifest_sha256,
        "evidence_case_ids": report.get("case_ids", sorted({event["case_id"] for event in report["events"]})),
        "selected_scope_cells": summary["expected_cells"],
        "selected_scope_complete": summary["selected_scope_complete"],
        "all_invocations_passed": summary["all_invocations_passed"],
        "planned_cells": len(expected_by_key),
        "executed_cells": executed,
        "pending_cells": pending,
        "complete_matrix": pending == 0 and summary["all_invocations_passed"],
        "pending_reason_code": "route_evidence_not_emitted",
        "pending_reason": "No validated per-cell route execution evidence is recorded for this cell.",
        "pending_case_ids": pending_case_ids,
        "pending_cases": [{"case_id": case_id, "pending_cells": pending_by_case[case_id]}
                           for case_id in pending_case_ids],
        "route_summary": route_summary,
        "route_action_axis_summary": shape_summary,
        "controlled_peer_cells": sum(event["action"] == "send" for event in observed_by_key.values()),
        "vendor_interoperability_claim": False,
    }


def write_selected_scope_ledger(ledger: dict) -> Path:
    """Write the Java route ledger to its fixed, repository-owned build path."""
    repository = Path(__file__).resolve(strict=True).parents[3]
    output_path = repository / "interop/java-kmip-validator/build/route-evidence-ledger.json"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(ledger, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return output_path


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
                if event["action"] == "decode" and event.get("decode_outcome") == "rejected":
                    tree_field = "decode_input_sha256"
                else:
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
        per_language_hashes = {}
        for language, events in language_events.items():
            per_language_hashes[language] = {
                event["decode_input_sha256"] if key[5] == "decode" and event.get("decode_outcome") == "rejected"
                else event["response_tree_sha256"] if key[5] == "decode"
                else event["request_tree_sha256"]
                for event in events
            }
        if key[5] == "decode" and any(
                len({(event.get("decode_outcome", "decoded"), event.get("decode_rejection_code"))
                     for event in events}) > 1
                for events in language_events.values()):
            raise ValueError(f"decoder routes disagree on the normalized decode outcome: {key}")
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
        if key[5] == "decode":
            decode_outcomes = {event.get("decode_outcome", "decoded") for event in events}
            rejection_codes = {event.get("decode_rejection_code") for event in events}
            if len(decode_outcomes) > 1 or len(rejection_codes) > 1:
                raise ValueError(f"languages disagree on the normalized decode outcome: {key}")
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
    parser.add_argument("--selected-scope", action="store_true",
                        help="validate one complete report scope and retain global pending cells separately")
    parser.add_argument("--write-ledger", action="store_true",
                        help="write the deterministic ledger to the Java validator's fixed build path")
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    allowed_file_roots = (root, Path(tempfile.gettempdir()))
    catalog_path = root / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
    manifest_path = root / "interop/shared/coverage/route-capabilities.json"
    catalog_bytes = catalog_path.read_bytes()
    manifest_bytes = manifest_path.read_bytes()
    report_paths = [resolve_within(path, allowed_file_roots, must_exist=True) for path in args.report]
    reports = [json.loads(path.read_text(encoding="utf-8")) for path in report_paths]  # NOSONAR: report paths are resolved under the repository or system temp directory
    catalog = json.loads(catalog_bytes)
    manifest = json.loads(manifest_bytes)
    catalog_sha = hashlib.sha256(catalog_bytes).hexdigest()
    route_sha = hashlib.sha256(manifest_bytes).hexdigest()
    if args.selected_scope:
        if args.parity or len(reports) != 1 or len(args.artifact_sha256) != 1 or "=" in args.artifact_sha256[0]:
            parser.error("--selected-scope requires one report and one non-parity artifact digest")
        summary = validate_report(
            reports[0], catalog, manifest, catalog_sha256=catalog_sha,
            route_manifest_sha256=route_sha, expected_artifact_sha256=args.artifact_sha256[0],
        )
        if args.write_ledger:
            ledger = build_evidence_ledger(
                reports[0], catalog, manifest, catalog_sha256=catalog_sha,
                route_manifest_sha256=route_sha, expected_artifact_sha256=args.artifact_sha256[0],
            )
            ledger_path = write_selected_scope_ledger(ledger)
            summary["ledger"] = {
                "path": str(ledger_path), "planned_cells": ledger["planned_cells"],
                "executed_cells": ledger["executed_cells"], "pending_cells": ledger["pending_cells"],
                "complete_matrix": ledger["complete_matrix"],
            }
    elif args.write_ledger:
        parser.error("--write-ledger requires --selected-scope")
    elif args.parity:
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
