#!/usr/bin/env python3
"""Join planned KMIP coverage with evidence from independently installed consumers."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
import subprocess
import sys
from pathlib import Path

from jsonschema import Draft202012Validator

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[2]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.matrix.generator import selected_cells
from verification.interop.shared.tools.catalog_reports import load_catalog_reports
from verification.interop.shared.tools.oasis_ttlv import attach_decoder_evidence

LANGUAGES = ("rust", "c", "java", "python")
VERSIONS = ("2.0", "2.1", "3.0")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def identifiers(rows):
    """Use the complete canonical obligation so changed assertions invalidate old evidence."""
    for row in rows:
        yield hashlib.sha256(canonical(row).encode("ascii")).hexdigest(), row


def read_validated(path, schema):
    data = json.loads(path.read_text(encoding="utf-8"))
    definition = json.loads((SHARED / "schemas" / schema).read_text(encoding="utf-8"))
    error = next(Draft202012Validator(definition).iter_errors(data), None)
    if error is not None:
        location = ".".join(map(str, error.absolute_path)) or "document"
        raise ValueError(f"{path.name}: invalid {schema} at {location}")
    return data


def plan(rows):
    entries = {}
    digest = hashlib.sha256()
    for obligation_id, row in identifiers(rows):
        if obligation_id in entries:
            raise ValueError(f"duplicate obligation {obligation_id}")
        entries[obligation_id] = row
        digest.update(bytes.fromhex(obligation_id))
    return digest.hexdigest(), entries


def eligible_evidence(row, scope):
    if row.get("kind") == "inventory" or row.get("execution_status") == "not_applicable":
        return False
    if row.get("kind") == "scenario_item":
        return scope in ("controlled", "live")
    if row.get("mode") == "network" and row["action"] != "construct":
        return scope in ("controlled", "live")
    return scope == "local"


def declaration(profile, operation):
    if profile is None or operation is None:
        return "unknown"
    def normalize(value):
        return "".join(c for c in value.casefold() if c.isalnum())

    matches = {entry["supported"] for name, entry in profile["operations"].items()
               if normalize(name) == normalize(operation)}
    if len(matches) > 1:
        raise ValueError(f"conflicting capability declarations for {operation}")
    if not matches:
        return "unknown"
    return "supported" if matches.pop() else "unsupported"


def collect_events(matrix, matrix_sha256, runs, languages):
    server = None
    library_commit = None
    events = {}
    provenance = []
    parity_groups = {}
    seen_runs = set()
    for run in runs:
        if run["matrix_sha256"] != matrix_sha256:
            raise ValueError("evidence was produced for a different coverage matrix")
        if library_commit is not None and run["library_commit"] != library_commit:
            raise ValueError("campaign mixes KMIPKit source commits")
        library_commit = run["library_commit"]
        language, scope = run["language"], run["scope"]
        if language not in languages:
            raise ValueError(f"evidence language is outside campaign: {language}")
        if run["run_id"] in seen_runs:
            raise ValueError("duplicate run ID")
        seen_runs.add(run["run_id"])
        if scope == "live":
            if server is not None and run["server"] != server:
                raise ValueError("campaign mixes live server identities or configurations")
            server = run["server"]
        provenance.append({k: v for k, v in run.items() if k != "events"})
        for event in run["events"]:
            obligation_id = event["obligation_id"]
            row = matrix.get(obligation_id)
            if row is None or row.get("language", language) != language or not eligible_evidence(row, scope):
                raise ValueError(f"event outside its planned scope: {obligation_id}")
            # XML scenarios are language-neutral. Their evidence must still identify the actual public route.
            expected = row.get("public_api_symbol")
            if expected is not None and event.get("public_api_symbol") != expected:
                raise ValueError(f"unexpected public API symbol for {obligation_id}")
            if row.get("execution_api_symbol") != event.get("execution_api_symbol"):
                if row.get("kind") != "scenario_item":
                    raise ValueError(f"unexpected execution API symbol for {obligation_id}")
            if scope != "local":
                for axis in ("encoding", "transport"):
                    if axis not in event or row.get(axis) not in (None, event[axis]):
                        raise ValueError(f"unexpected {axis} for {obligation_id}")
            if "server_compatibility" in event and scope != "live":
                raise ValueError("server compatibility requires live evidence")
            key = obligation_id, language, scope
            combination = event.get("encoding"), event.get("transport")
            observation = {k: v for k, v in event.items() if k != "obligation_id"}
            by_combination = events.setdefault(key, {})
            if combination in by_combination and observation != by_combination[combination]:
                raise ValueError(f"conflicting observations for {obligation_id}, {language}, {scope}")
            by_combination[combination] = observation
            if event.get("parity_group") and event["status"] == "passed":
                parity_key = (row["protocol_version"], row["case_id"], row.get("action"),
                              row.get("workflow_step_index", row.get("step_index")),
                              row.get("workflow_item_index", row.get("item_index")), scope,
                              combination, event["parity_group"])
                group = parity_groups.setdefault(parity_key, {})
                group.setdefault(language, set()).add(event["semantic_sha256"])

    return server, library_commit, provenance, events, parity_groups


def report(matrix, matrix_sha256, runs, languages, profile=None, catalog=None):
    server, library_commit, provenance, events, parity_groups = collect_events(
        matrix, matrix_sha256, runs, languages,
    )
    by_language = {language: Counter() for language in languages}
    by_version = {}
    compatibility = Counter()
    route_coverage = {language: Counter() for language in languages}
    rows = []
    scenarios = {}
    declarations = {}
    for obligation_id, row in matrix.items():
        members = (row["language"],) if "language" in row else languages
        if not set(members).intersection(by_language):
            continue
        operation = (row.get("operation") or row.get("workflow_operation")
                     or (catalog or {}).get(row["case_id"]))
        if operation not in declarations:
            declarations[operation] = declaration(profile, operation)
        declared = declarations[operation]
        entry = {"obligation_id": obligation_id, **row, "operation": operation, "declared": declared, "checks": {}}
        for language in members:
            if language not in by_language or row.get("kind") == "inventory":
                continue
            observations = {scope: list(events[obligation_id, language, scope].values())
                            for scope in ("local", "controlled", "live")
                            if (obligation_id, language, scope) in events}
            qualifying_scope = "controlled" if eligible_evidence(row, "controlled") else "local"
            required = row.get("execution_status") != "not_applicable"
            outcomes = {event["status"] for event in observations.get(qualifying_scope, ())}
            state = next((status for status in ("failed", "blocked", "passed") if status in outcomes), "pending")
            if not required:
                state = "not_applicable"
            by_language[language][state] += 1
            version_counts = by_version.setdefault(row["protocol_version"], {})
            version_counts.setdefault(language, Counter())[state] += 1
            if required:
                for symbol in {row.get("public_api_symbol"), row.get("execution_api_symbol")} - {None}:
                    route_coverage[language][symbol] += int(state != "passed")
            if eligible_evidence(row, "live"):
                live_outcomes = observations.get("live", ())
                if not live_outcomes:
                    compatibility["unobserved"] += 1
                for event in live_outcomes:
                    live = event.get("server_compatibility", "unknown")
                    compatibility[live] += 1
                    if declared == "supported" and live == "unsupported":
                        event["declaration_check"] = "advertised_but_unsupported"
                    elif declared == "unsupported" and live == "supported":
                        event["declaration_check"] = "supported_despite_declaration"
                    elif declared == "unknown" and live == "supported":
                        event["declaration_check"] = "supported_without_declaration"
            entry["checks"][language] = {"status": state, **observations}
            if row.get("kind") == "scenario_item":
                scenarios.setdefault((row["protocol_version"], row["case_id"], language), []).append(state)
        rows.append(entry)
    parity = Counter()
    for group in parity_groups.values():
        if set(group) != set(languages) or len(group) < 2:
            parity["incomplete"] += 1
        elif len(set().union(*group.values())) == 1:
            parity["matched"] += 1
        else:
            parity["mismatched"] += 1
    return {"schema_version": "1.0", "matrix_sha256": matrix_sha256, "library_commit": library_commit,
            "server": server, "profile_id": profile["server"]["id"] if profile else None,
            "runs": provenance, "languages": list(languages),
            "summary": {name: dict(counts) for name, counts in by_language.items()},
            "by_version": by_version,
            "public_api": {name: {"total": len(symbols), "complete": sum(n == 0 for n in symbols.values())}
                           for name, symbols in route_coverage.items()},
            "server_compatibility": dict(compatibility),
            "parity": dict(parity),
            "unmapped_inventory": sum(row.get("kind") == "inventory" for row in matrix.values()),
            "xml_scenarios": dict(Counter("passed" if all(s == "passed" for s in states) else "incomplete"
                                          for states in scenarios.values())),
            "rows": rows}


def write_jsonl(path, records):
    with path.open("w", encoding="utf-8", newline="\n") as output:
        for record in records:
            output.write(canonical(record) + "\n")


def write_report(destination, result, catalog_runs=()):
    rows = result["rows"]
    overview = {key: value for key, value in result.items() if key != "rows"}
    if catalog_runs:
        overview["catalog_runs"] = [{k: v for k, v in run.items() if k != "cases"} for run in catalog_runs]
        write_jsonl(destination / "catalog-checks.jsonl", (
            {"run_id": run["run_id"], **case} for run in catalog_runs for case in run["cases"]
        ))
    lines = ["# KMIPKit interoperability checks", "", f"Matrix SHA-256: `{overview['matrix_sha256']}`", "",
             "| Language | Public routes complete | Passed | Failed | Blocked | Pending |",
             "| --- | ---: | ---: | ---: | ---: | ---: |"]
    for language, counts in overview["summary"].items():
        routes = overview["public_api"][language]
        counts_text = " | ".join(str(counts.get(state, 0)) for state in ("passed", "failed", "blocked", "pending"))
        lines.append(f"| {language} | {routes['complete']}/{routes['total']} | {counts_text} |")
    lines.extend(("", f"Normative inventory entries: {overview['unmapped_inventory']}", ""))
    for status, count in overview["parity"].items():
        lines.append(f"- Parity {status}: {count}")
    for status, count in overview["server_compatibility"].items():
        lines.append(f"- Live {status.replace('_', ' ')}: {count}")
    for language, counts in overview.get("oasis_ttlv", {}).get("message_checks", {}).items():
        lines.append(f"- {language} OASIS TTLV messages: " + ", ".join(f"{count} {status}" for status, count in counts.items()))
    if catalog_runs:
        lines.extend(("", "## Installed-validator execution catalog", "",
                      "| Language | KMIP | Scope | Encoding | Transport | API | Passed | Failed | Unsupported | Policy restricted | Not run |",
                      "| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |"))
        for run in catalog_runs:
            execution = run["execution"]
            counts = run["summary"]
            cells = [run["language"], run["protocol_version"], run["scope"], execution["encoding"],
                     execution["transport"], execution["api_level"]]
            cells.extend(str(counts.get(state, 0)) for state in (
                "passed", "failed", "server_unsupported", "policy_restricted", "not_run",
            ))
            lines.append("| " + " | ".join(cells) + " |")
    (destination / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (destination / "summary.json").write_text(json.dumps(overview, indent=2) + "\n", encoding="utf-8")
    write_jsonl(destination / "checks.jsonl", rows)
    # Embedded data makes the report browsable directly from disk. Escape markup before embedding JSON.
    display = [
        [r["protocol_version"], r.get("operation"), r["case_id"], r.get("public_api_symbol"),
         r.get("action"), r.get("encoding"), r.get("transport"), r["declared"], r["checks"],
         r.get("fixture", {}).get("path"), r.get("workflow_step_index", r.get("step_index")),
         r.get("workflow_item_index", r.get("item_index"))]
        for r in rows
    ]
    template = (SHARED / "tools" / "campaign.html").read_text(encoding="utf-8")
    data = canonical({"summary": overview, "checks": display, "catalog_runs": catalog_runs}).replace("<", "\\u003c")
    (destination / "index.html").write_text(template.replace("<!--CAMPAIGN_DATA-->", data), encoding="utf-8")


def launch_validators(runners, args, digest):
    """A runner receives a plan and writes evidence even when a test fails."""
    match args.scope:
        case "local":
            scope_options = ("--scope", "local")
        case "controlled":
            scope_options = ("--scope", "controlled")
        case "live":
            scope_options = ("--scope", "live")
        case _:
            raise ValueError("invalid campaign scope")
    output_dir = args.output_dir.resolve()
    runs = []
    failures = []
    connection = read_validated(args.connection, "server-connection.schema.json") if args.connection else None
    if args.scope == "live" and connection is None:
        raise ValueError("live runs require --connection")
    if connection is not None:
        if not {"server_product", "server_version"} <= connection.keys():
            raise ValueError("live runs require server_product and server_version")
        if tuple(args.version) != (connection["protocol_version"],):
            raise ValueError("the live plan must match the configured protocol version")
    if args.scope != "live" and connection is not None:
        raise ValueError("--connection requires --scope live")
    for language, executable in runners.items():
        output = output_dir / f"evidence-{language}-{scope_options[1]}.json"
        if output.exists():
            raise ValueError(f"evidence output already exists: {output.name}; use a fresh output directory")
        command = [str(executable), "--plan", str(output_dir / "plan.jsonl"),
                   "--matrix-sha256", digest, *scope_options, "--evidence", str(output)]
        if args.connection:
            command.extend(("--connection", str(args.connection.resolve())))
        if args.profile:
            command.extend(("--profile", str(args.profile.resolve())))
        completed = subprocess.run(command, check=False)
        if completed.returncode or not output.is_file():
            failures.append(language)
        if output.is_file():
            run = read_validated(output, "campaign-evidence.schema.json")
            if run["language"] != language or run["scope"] != args.scope:
                raise ValueError(f"runner for {language} produced evidence for a different language or scope")
            if connection is not None:
                expected = {"id": connection["server_id"], "product": connection["server_product"],
                            "version": connection["server_version"],
                            "configuration_fingerprint": connection["server_configuration_fingerprint"]}
                for event in run["events"]:
                    if event.get("transport") != connection["transport"] or event.get("encoding") != connection["encoding"]:
                        raise ValueError(f"runner for {language} used a different transport or encoding")
                if run["server"] != expected:
                    raise ValueError(f"runner for {language} used a different server configuration")
            runs.append(run)
    return runs, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", choices=VERSIONS, action="append")
    parser.add_argument("--language", choices=LANGUAGES, action="append")
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--require-complete", action="store_true")
    parser.add_argument("--runner", metavar="LANGUAGE EXECUTABLE", nargs=2, action="append", default=[])
    parser.add_argument("--scope", choices=("local", "controlled", "live"), default="local")
    parser.add_argument("--connection", type=Path)
    parser.add_argument("--library-commit", help="Require evidence from this exact source commit")
    parser.add_argument("--controlled-report", type=Path, action="append", default=[],
                        help="Import an installed validator's controlled execution-catalog report")
    parser.add_argument("--live-report", type=Path, action="append", default=[],
                        help="Import an installed validator's live execution-catalog report")
    parser.add_argument("--oasis-ttlv-report", type=Path,
                        help="Import decoder evidence for OASIS request and response shapes")
    parser.add_argument("evidence", nargs="*", type=Path)
    args = parser.parse_args()
    runners = {language: Path(executable).resolve() for language, executable in args.runner}
    languages = tuple(dict.fromkeys(args.language or runners or LANGUAGES))
    try:
        if len(runners) != len(args.runner) or not set(runners).issubset(languages) or not set(languages).issubset(LANGUAGES):
            raise ValueError("give one runner per selected, supported language")
        if args.connection and not runners:
            raise ValueError("--connection requires a runner")
        if args.version is not None:
            args.version = list(dict.fromkeys(args.version))
        if args.connection and args.version is None:
            connection = read_validated(args.connection, "server-connection.schema.json")
            args.version = [connection["protocol_version"]]
        digest, matrix = plan(selected_cells(ROOT, tuple(dict.fromkeys(args.version or VERSIONS)), None))
        args.output_dir.mkdir(parents=True, exist_ok=True)
        if runners or (not args.evidence and not args.require_complete):
            write_jsonl(args.output_dir / "plan.jsonl", ({"obligation_id": key, **row} for key, row in matrix.items()))
        profile = read_validated(args.profile, "server-capabilities.schema.json") if args.profile else None
        runs = [read_validated(path, "campaign-evidence.schema.json") for path in args.evidence]
        failures = []
        if runners:
            launched, failures = launch_validators(runners, args, digest)
            runs.extend(launched)
        if args.library_commit and any(run["library_commit"] != args.library_commit for run in runs):
            raise ValueError("evidence does not match the requested library commit")
        catalog = json.loads((SHARED / "catalog/kmipkit-2x-matrix-catalog.json").read_text(encoding="utf-8"))
        operations = {case["case_id"]: case.get("requires", {}).get("operation") for case in catalog["test_cases"]}
        result = report(matrix, digest, runs, languages, profile, operations)
        if args.oasis_ttlv_report:
            attach_decoder_evidence(args.oasis_ttlv_report, result, args.library_commit)
        result["runner_failures"] = failures
        catalog_runs = load_catalog_reports(
            [("controlled", path) for path in args.controlled_report] + [("live", path) for path in args.live_report],
            (SHARED / "catalog/kmipkit-1.0-test-catalog.json", SHARED / "catalog/kmipkit-2x-matrix-catalog.json"),
            languages, tuple(args.version or VERSIONS), read_validated,
        )
        if args.library_commit and any(run["execution"]["library_commit"] != args.library_commit for run in catalog_runs):
            raise ValueError("catalog evidence does not match the requested library commit")
        write_report(args.output_dir, result, catalog_runs)
    except (OSError, ValueError) as exc:
        parser.exit(2, f"Campaign failed: {exc}\n")
    print(json.dumps({k: result[k] for k in ("matrix_sha256", "summary", "public_api", "server_compatibility")}, indent=2))
    if failures or result["parity"].get("mismatched") or (
        args.require_complete and not coverage_complete(result, tuple(args.version or VERSIONS))
    ):
        return 1
    return 0


def coverage_complete(result, versions):
    if result["unmapped_inventory"]:
        return False
    return all(
        version in result["by_version"]
        and language in result["by_version"][version]
        and not any(result["by_version"][version][language].get(state) for state in ("pending", "blocked", "failed"))
        for version in versions for language in result["languages"]
    )


if __name__ == "__main__":
    raise SystemExit(main())
