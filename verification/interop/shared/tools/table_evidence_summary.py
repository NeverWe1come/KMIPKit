"""Summarize verified and outstanding evidence for each KMIP table cell."""

from collections import Counter, defaultdict


def fixture_matches(result):
    checks = result["checks"]
    if "expected_checks" in result:
        return checks == result["expected_checks"]
    return bool(checks) and set(checks.values()) == {result["expected"]}


def local_checks(row, obligation):
    for field in (
        "local_fixture_results", "local_key_material_results",
        "local_typed_attribute_results", "local_managed_object_results",
    ):
        for result in obligation.get(field, ()):
            yield "passed" if fixture_matches(result) else "failed"
    for result in obligation.get("version_gate_results", ()):
        yield result["status"]
    if obligation["api_level"] == "generic-ttlv":
        for result in row.get("named_value_results", ()):
            if result["language"] == obligation["language"]:
                yield result["status"]
    for result in obligation.get("oasis_fixture_results", ()):
        yield from result["checks"].values()


def observation_kind(record, recipe):
    if record["status"] == "not_run":
        return None
    if record.get("library_correctness") != "passed":
        return "failed" if record["status"] == "failed" else "incomplete"

    status, code, reason = record["status"], record.get("result_status_code"), record.get("result_reason_code")
    required_status = recipe["requires"].get("expected_status")
    if required_status == "NotSent" and status == "passed" and code is None:
        return "boundary"
    if required_status == "OperationFailed" and status == "passed" and code == 1:
        reasons = recipe["requires"].get("expected_result_reasons", ())
        return "negative" if not reasons or reason in reasons else "incomplete"
    if status == "server_unsupported" and code == 1 and reason in {5, 8}:
        return "unsupported"
    if status == "passed" and (code == 0 or (
        code is None and recipe["requires"].get("workflow_type") == "server_to_client"
    )):
        return "success"
    return "incomplete"


def required_observation(expectation, recipe, scope):
    requirements = recipe["requires"]
    if expectation == "verify_provider_fixture" and not requirements.get("provider_fixture"):
        return None
    if scope == "controlled" and requirements.get("controlled_expected_status") == "Success":
        return "success"
    required_status = requirements.get("expected_status")
    if required_status == "NotSent":
        return "boundary"
    if required_status == "OperationFailed":
        return "negative"
    return {
        "verify_result": "success",
        "verify_provider_fixture": "success",
        "verify_unsupported_response": "unsupported",
    }.get(expectation)


def summarize_cases(row, cell, recipes):
    selected = cell.get("case_ids", row.get("server_cases", []) + row.get("boundary_cases", []))
    observations = {"controlled": Counter(), "live": Counter()}
    requirements = {
        scope: {case_id: required_observation(cell["expectation"], recipes[case_id], scope)
                for case_id in selected}
        for scope in observations
    }
    matching = {"controlled": set(), "live": set()}
    for record in cell.get("case_results", ()):
        scope, case_id = record["scope"], record["case_id"]
        if scope not in observations:
            continue
        observed = observation_kind(record, recipes[case_id])
        if observed is None:
            continue
        observations[scope][observed] += 1
        if observed == requirements[scope].get(case_id):
            matching[scope].add(case_id)
    case_results = {}
    for scope, matched in matching.items():
        required = {case_id for case_id, expected in requirements[scope].items()
                    if expected is not None and (scope != "live" or expected != "boundary")}
        case_results[scope] = {
            "matched": sorted(required & matched),
            "missing": sorted(required - matched),
        }
    return observations, case_results


def local_boundary_verified(row, cell):
    for fixture in cell.get("local_fixture_results", ()):
        if fixture.get("expected") == "rejected" and fixture_matches(fixture):
            return True
    for label in ("local_managed_object_results", "local_key_material_results"):
        for fixture in cell.get(label, ()):
            if fixture.get("scenario", "").startswith("reserved-") and fixture_matches(fixture):
                return True
    named_values = row.get("named_value_results", ()) if cell["api_level"] == "generic-ttlv" else ()
    return any(
        result["status"] == "passed" and result["accepted"] is False
        for result in cell.get("version_gate_results", ())
    ) or any(
        result["language"] == cell["language"] and result["status"] == "passed"
        for result in named_values
    )


def completion_status(row, cell, required_cases, local):
    controlled, live = required_cases["controlled"], required_cases["live"]
    if local["failed"]:
        return "failed_local_assertion"
    if cell["expectation"] == "verify_version_boundary":
        if not controlled["missing"] and controlled["matched"]:
            return "complete"
        if not row.get("boundary_cases") and local_boundary_verified(row, cell):
            return "complete"
        return "missing_version_boundary"
    if cell["expectation"] == "verify_provider_fixture" and not any(
        controlled[key] for key in ("missing", "matched")
    ):
        return "missing_provider_fixture"
    if not row.get("server_cases"):
        return "missing_exchange_recipe"
    if controlled["missing"] or not controlled["matched"]:
        return "missing_controlled_evidence"
    if live["missing"] or not live["matched"]:
        observed_at_provider = {
            result["case_id"] for result in cell.get("case_results", ())
            if result["scope"] == "live" and result["status"] != "not_run"
            and result.get("result_status_code") is not None
        }
        if live["missing"] and set(live["missing"]) <= observed_at_provider:
            return "provider_behavior_differs"
        return "missing_provider_evidence"
    return "complete"


def summarize(inventory, catalog, languages=None):
    """Keep local, controlled peer and provider evidence visible on each cell."""
    recipes = {case["case_id"]: case for case in catalog["test_cases"]}
    counts = defaultdict(Counter)
    completeness = Counter()
    for row in inventory["rows"]:
        for cell in row["obligations"]:
            local = Counter(local_checks(row, cell))
            observations, required_cases = summarize_cases(row, cell, recipes)
            controlled, live = observations["controlled"], observations["live"]
            cell["expected_case_outcomes"] = required_cases
            completion = completion_status(row, cell, required_cases, local)
            cell["completion_status"] = completion
            cell["evidence_summary"] = {
                "local": dict(local), "controlled": dict(controlled), "live": dict(live),
            }
            if languages is not None and cell["language"] not in languages:
                continue
            completeness[completion] += 1
            evidence = (*local.keys(), *controlled.keys(), *live.keys())
            aggregate = counts[cell["language"], cell["api_level"]]
            aggregate["total"] += 1
            aggregate["complete_cells"] += completion == "complete"
            aggregate["local_cells"] += bool(local["passed"])
            result_kinds = {"boundary", "negative", "unsupported", "success"}
            aggregate["controlled_cells"] += bool(result_kinds & controlled.keys())
            aggregate["live_cells"] += bool(result_kinds & live.keys())
            for scope, entries in required_cases.items():
                aggregate[f"{scope}_expected_cases_met_cells"] += bool(entries["matched"]) and not entries["missing"]
            aggregate["no_evidence"] += not evidence
            aggregate["failed_cells"] += "failed" in evidence or "rejected" in local
    return {
        "protocol_version": inventory["protocol_version"],
        "rows": len(inventory["rows"]),
        "completion": {
            "complete": bool(completeness) and set(completeness) == {"complete"},
            "evidence_complete": bool(completeness) and set(completeness) <= {"complete", "provider_behavior_differs"},
            "cells_by_status": dict(completeness),
        },
        "routes": [{"language": language, "api_level": api, **values}
                   for (language, api), values in sorted(counts.items())],
    }
