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


def observed_scope(obligation, scope, recipe_by_id):
    counts = Counter()
    for record in obligation.get("case_results", ()):
        if record["scope"] != scope or record["status"] == "not_run":
            continue
        if record.get("library_correctness") == "passed":
            recipe = recipe_by_id[record["case_id"]]
            status, code, reason = record["status"], record.get("result_status_code"), record.get("result_reason_code")
            required_status = recipe["requires"].get("expected_status")
            if required_status == "NotSent" and status == "passed" and code is None:
                counts["boundary"] += 1
            elif required_status == "OperationFailed" and status == "passed" and code == 1:
                reasons = recipe["requires"].get("expected_result_reasons", ())
                counts["negative" if not reasons or reason in reasons else "incomplete"] += 1
            elif status == "server_unsupported" and code == 1 and reason in {5, 8}:
                counts["unsupported"] += 1
            elif status == "passed" and (code == 0 or (
                code is None and recipe["requires"].get("workflow_type") == "server_to_client"
            )):
                counts["success"] += 1
            else:
                counts["incomplete"] += 1
        else:
            counts["failed" if record["status"] == "failed" else "incomplete"] += 1
    return dict(counts)


def summarize(inventory, catalog):
    """Keep local, controlled peer and provider evidence visible on each cell."""
    recipes = {case["case_id"]: case for case in catalog["test_cases"]}
    counts = defaultdict(Counter)
    for row in inventory["rows"]:
        for cell in row["obligations"]:
            local = Counter(local_checks(row, cell))
            controlled = observed_scope(cell, "controlled", recipes)
            live = observed_scope(cell, "live", recipes)
            cell["evidence_summary"] = {
                "local": dict(local), "controlled": controlled, "live": live,
            }
            evidence = (*local.keys(), *controlled.keys(), *live.keys())
            aggregate = counts[cell["language"], cell["api_level"]]
            aggregate["total"] += 1
            aggregate["local_cells"] += bool(local["passed"])
            result_kinds = {"boundary", "negative", "unsupported", "success"}
            aggregate["controlled_cells"] += bool(result_kinds & controlled.keys())
            aggregate["live_cells"] += bool(result_kinds & live.keys())
            aggregate["no_evidence"] += not evidence
            aggregate["failed_cells"] += "failed" in evidence or "rejected" in local
    return {
        "protocol_version": inventory["protocol_version"],
        "rows": len(inventory["rows"]),
        "routes": [{"language": language, "api_level": api, **values}
                   for (language, api), values in sorted(counts.items())],
    }
