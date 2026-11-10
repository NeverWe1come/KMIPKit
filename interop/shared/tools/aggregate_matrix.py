#!/usr/bin/env python3
"""Cross-Language and Multi-KMS Matrix Aggregator for KMIPKit Interop."""

from __future__ import annotations

import argparse
import html
import json
import re
import sys
from collections import defaultdict
from pathlib import Path


RESULT_STATUS_NAMES = {
    "passed": "PASSED",
    "failed": "FAILED",
    "server_unsupported": "UNSUPPORTED_BY_SERVER",
    "policy_restricted": "POLICY_RESTRICTED",
    "not_run": "NOT_RUN",
}
CASE_RESULT_FIELDS = (
    "library_correctness", "server_compatibility", "parity", "duration_ms",
)
SUMMARY_FIELDS = ("total", "passed", "failed", "unsupported", "policy_restricted", "not_run")
RESULT_ENUMS = {"result_status": "Result Status", "result_reason": "Result Reason"}
RESULT_ENUM_ALIASES = {"Result Status": {"operationsucceeded": 0}}


def parse_result_file(file_path: Path) -> dict | None:
    p = file_path.resolve()
    if not p.exists():
        return None
    try:
        with open(p, "r", encoding="utf-8") as f:  # NOSONAR
            return json.load(f)
    except Exception:
        return None


def _enum_key(value: str) -> str:
    value = value.strip().rsplit("::", 1)[-1].rsplit(".", 1)[-1]
    return re.sub(r"[^a-z0-9]", "", value.casefold())


def _numeric_enum_code(value: object) -> int | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    text = str(value).strip()
    if re.fullmatch(r"[0-9a-fA-F]{8}", text):
        return int(text, 16)
    try:
        return int(text, 0) if text.lower().startswith("0x") else int(text, 10)
    except ValueError:
        return None


def _load_enum_codes() -> dict[str, dict[str, int]]:
    source_path = Path(__file__).resolve().parents[3] / "specification" / "catalog" / "kmip-2.1.json"
    try:
        catalog = json.loads(source_path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {enum_name: {} for enum_name in RESULT_ENUMS.values()}
    enum_ids = {
        element["name"]: element["element_id"]
        for element in catalog.get("elements", [])
        if element.get("kind") == "enumeration" and element.get("name") in RESULT_ENUMS.values()
    }
    codes = {enum_name: {} for enum_name in RESULT_ENUMS.values()}
    for element in catalog.get("elements", []):
        if element.get("kind") != "enumeration_value" or not isinstance(element.get("wire_value"), str):
            continue
        wire_value = element["wire_value"]
        if not re.fullmatch(r"[0-9a-fA-F]{8}", wire_value):
            continue
        for enum_name in RESULT_ENUMS.values():
            if enum_ids.get(enum_name) not in element.get("parent_element_ids", []):
                continue
            code = _numeric_enum_code(wire_value)
            if code is not None:
                codes[enum_name][_enum_key(element["name"])] = code
    for enum_name, aliases in RESULT_ENUM_ALIASES.items():
        codes[enum_name].update(aliases)
    return codes


def _normalize_enum_code(value: object, enum_name: str, enum_codes: dict[str, dict[str, int]]) -> int | str | None:
    if value is None:
        return None
    code = _numeric_enum_code(value)
    if code is not None:
        return code
    key = _enum_key(str(value))
    known_code = enum_codes.get(enum_name, {}).get(key)
    return known_code if known_code is not None else f"unmapped:{key}"


def _case_for_report(case: dict, enum_codes: dict[str, dict[str, int]]) -> dict:
    """Keep result fields useful for comparison and omit free-form diagnostics/bodies."""
    safe_result = {field: case[field] for field in CASE_RESULT_FIELDS if field in case}
    safe_result["case_id"] = case["case_id"]
    safe_result["status"] = RESULT_STATUS_NAMES.get(case.get("status"), case.get("status", "NOT_RUN"))
    for field, enum_name in RESULT_ENUMS.items():
        raw_value = case.get(f"{field}_code", case.get(field))
        if raw_value is not None:
            safe_result[f"{field}_code"] = _normalize_enum_code(raw_value, enum_name, enum_codes)
    return safe_result


def _summary_for_report(summary: dict) -> dict:
    return {field: summary[field] for field in SUMMARY_FIELDS if field in summary}


def _evidence_signature(run: dict) -> tuple:
    server = run["server"]
    return (
        server["id"], server["version"], server["configuration_fingerprint"],
        run["catalog_sha256"], run["execution"]["library_commit"],
    )


def _case_result_for_run(run: dict, case_id: str) -> dict:
    case = run["cases"].get(case_id)
    return {
        "run_id": run["run_id"],
        "language": run["language"],
        "transport": run["execution"]["transport"],
        "encoding": run["execution"]["encoding"],
        "api_level": run["execution"]["api_level"],
        "status": case.get("status", "NOT_RUN") if case else "NOT_RUN",
        "library_correctness": case.get("library_correctness", "not_checked") if case else "not_checked",
        "server_compatibility": case.get("server_compatibility", "unknown") if case else "unknown",
        "parity": case.get("parity", "not_compared") if case else "not_compared",
        "result_status_code": case.get("result_status_code") if case else None,
        "result_reason_code": case.get("result_reason_code") if case else None,
    }


def _difference_dimensions(results: list[dict]) -> list[str]:
    compared_fields = (
        ("status", "status"),
        ("result_status_code", "result_status"),
        ("result_reason_code", "result_reason"),
        ("library_correctness", "library_correctness"),
        ("server_compatibility", "server_compatibility"),
        ("parity", "parity"),
    )
    return [
        dimension for field, dimension in compared_fields
        if len({result.get(field) for result in results}) > 1
    ]


def aggregate_results(result_files: list[Path]) -> dict:
    groups = defaultdict(list)
    cases_by_group = defaultdict(set)
    all_cases = set()
    all_servers = set()
    all_languages = set()
    case_metadata = {}
    enum_codes = _load_enum_codes()
    catalog_path = Path(__file__).resolve().parents[1] / "catalog" / "kmipkit-1.0-test-catalog.json"
    try:
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        case_metadata = {
            case["case_id"]: {"category": case.get("category", ""), "description": case.get("description", "")}
            for case in catalog.get("test_cases", [])
        }
    except (OSError, ValueError, KeyError):
        case_metadata = {}
    for file_path in result_files:
        payload = parse_result_file(file_path)
        if not payload:
            continue
        server = payload.get("server", {})
        server_id = payload.get("server_identifier") or server.get("id", "unknown_server")
        version = payload.get("protocol_version") or "unknown"
        language = payload.get("language", "unknown_lang")
        execution = payload.get("execution", {})
        run_id = payload.get("run_id") or file_path.stem
        all_servers.add(server_id)
        all_languages.add(language)
        run_cases = {}
        for case in payload.get("cases", payload.get("results", [])):
            case_id = case.get("case_id")
            if not case_id:
                continue
            all_cases.add(case_id)
            run_cases[case_id] = _case_for_report(case, enum_codes)
        run = {
            "run_id": run_id,
            "language": language,
            "server": {
                "id": server_id,
                "version": server.get("version"),
                "configuration_fingerprint": server.get("configuration_fingerprint"),
            },
            "protocol_version": version,
            "catalog_sha256": payload.get("catalog_sha256"),
            "execution": {
                "transport": execution.get("transport"),
                "encoding": execution.get("encoding"),
                "api_level": execution.get("api_level"),
                "library_commit": execution.get("library_commit"),
                "artifact_sha256": execution.get("artifact_sha256"),
            },
            "summary": _summary_for_report(payload.get("summary", {})),
            "cases": run_cases,
        }
        groups[(server_id, version)].append(run)
        cases_by_group[(server_id, version)].update(run_cases)

    matrices = []
    for (server_id, version), runs in sorted(groups.items()):
        runs.sort(key=lambda item: (item["language"], item["run_id"]))
        run_key_counts = defaultdict(int)
        for run in runs:
            run_key_counts[run["run_id"]] += 1
            count = run_key_counts[run["run_id"]]
            run["result_key"] = run["run_id"] if count == 1 else f"{run['run_id']}#{count}"
        evidence = {_evidence_signature(run) for run in runs}
        fields_complete = all(all(value is not None for value in item) for item in evidence)
        comparability = "UNVERIFIED" if not fields_complete else ("COMPARABLE" if len(evidence) == 1 else "INCOMPARABLE")
        rows = []
        correctness = defaultdict(int)
        compatibility = defaultdict(int)
        not_executed = 0
        for case_id in sorted(cases_by_group[(server_id, version)]):
            row = {
                "case_id": case_id,
                **case_metadata.get(case_id, {}),
                "results": {},
                "status": "NOT_RUN",
                "difference_dimensions": [],
            }
            for run in runs:
                result = _case_result_for_run(run, case_id)
                row["results"][run["result_key"]] = result
                correctness[result["library_correctness"]] += 1
                compatibility[result["server_compatibility"]] += 1
                if result["status"] == "NOT_RUN":
                    not_executed += 1
            if comparability == "INCOMPARABLE":
                row["status"] = "NOT_COMPARABLE"
            elif comparability == "UNVERIFIED":
                row["status"] = "UNVERIFIED"
            else:
                executed_results = [result for result in row["results"].values() if result["status"] != "NOT_RUN"]
                if len(executed_results) == 1:
                    row["status"] = "PARTIAL" if len(runs) > 1 else "NOT_COMPARED"
                elif len(executed_results) > 1:
                    row["difference_dimensions"] = _difference_dimensions(executed_results)
                    if row["difference_dimensions"]:
                        row["status"] = "PARITY_MISMATCH"
                    elif len(executed_results) < len(runs):
                        row["status"] = "PARTIAL"
                    else:
                        row["status"] = "PARITY_OK"
            rows.append(row)
        matrices.append({
            "server_id": server_id,
            "protocol_version": version,
            "comparability": comparability,
            "runs": runs,
            "cases": rows,
            "summary": {
                "library_correctness": dict(correctness),
                "server_compatibility": dict(compatibility),
                "not_executed": not_executed,
            },
        })

    return {
        "servers": sorted(all_servers),
        "languages": sorted(all_languages),
        "total_cases": len(all_cases),
        "matrices": matrices,
    }


def generate_markdown(aggregated: dict) -> str:
    md = ["# KMIPKit Interoperability Matrix\n"]
    md.extend([
        f"- **Servers:** {', '.join(aggregated['servers']) or 'None'}",
        f"- **Languages:** {', '.join(aggregated['languages']) or 'None'}",
        f"- **Catalog cases:** {aggregated['total_cases']}\n",
    ])
    for matrix in aggregated.get("matrices", []):
        server = matrix["runs"][0]["server"] if matrix["runs"] else {"id": matrix["server_id"]}
        md.append(f"## Server `{matrix['server_id']}` — KMIP {matrix['protocol_version']}\n")
        md.append(f"Evidence comparability: **{matrix['comparability']}**; server configuration: `{server.get('configuration_fingerprint') or 'unknown'}`.")
        md.append(f"Library correctness: `{matrix['summary']['library_correctness']}`")
        md.append(f"Server compatibility: `{matrix['summary']['server_compatibility']}`")
        md.append(f"Cases not executed (not_run): **{matrix['summary']['not_executed']}**\n")
        md.append("Execution runs: " + "; ".join(
            f"`{run.get('result_key', run['run_id'])}` ({run['language']}, {run['execution']['transport'] or 'unknown'}, "
            f"{run['execution']['encoding'] or 'unknown'}, {run['execution']['api_level'] or 'unknown'})"
            for run in matrix["runs"]
        ) + "\n")
        columns = ["Case", "Category"] + [run.get("result_key", run["run_id"]) for run in matrix["runs"]] + ["Language/route differences"]
        md.append("| " + " | ".join(columns) + " |")
        md.append("| " + " | ".join(["---"] * len(columns)) + " |")
        for row in matrix["cases"]:
            cells = [f"`{row['case_id']}`", row.get("category", "")]
            for run in matrix["runs"]:
                result = row["results"][run.get("result_key", run["run_id"])]
                status_code = result["result_status_code"]
                reason_code = result["result_reason_code"]
                cells.append(
                    f"{result['status']} / library:{result['library_correctness']} / "
                    f"server:{result['server_compatibility']} / parity:{result['parity']} / "
                    f"KMIP:{status_code if status_code is not None else '-'}:"
                    f"{reason_code if reason_code is not None else '-'}"
                )
            difference = ", ".join(row["difference_dimensions"])
            cells.append(f"{row['status']}" + (f" ({difference})" if difference else ""))
            md.append("| " + " | ".join(str(cell).replace("|", "\\|") for cell in cells) + " |")
        md.append("")
    return "\n".join(md)


def generate_html(aggregated: dict) -> str:
    matrices = aggregated.get("matrices", [])
    if not matrices:
        for server_id, rows in aggregated.get("parity_by_server", {}).items():
            languages = sorted({name for row in rows for name in row.get("results", {})})
            runs = [{
                "run_id": language, "language": language,
                "server": {"id": server_id, "version": None, "configuration_fingerprint": None},
                "execution": {"transport": None, "encoding": None, "api_level": None},
            } for language in languages]
            cases = []
            for row in rows:
                cases.append({**row, "results": {
                    language: {
                        "run_id": language, "language": language, "transport": None,
                        "encoding": None, "api_level": None,
                        "status": row.get("results", {}).get(language, "NOT_RUN"),
                        "library_correctness": "not_checked", "server_compatibility": "unknown",
                    } for language in languages
                }})
            matrices.append({
                "server_id": server_id, "protocol_version": "unknown", "comparability": "UNVERIFIED",
                "runs": runs, "cases": cases,
                "summary": {"library_correctness": {}, "server_compatibility": {}, "not_executed": 0},
            })

    all_rows = [row for matrix in matrices for row in matrix.get("cases", [])]
    total_cases = len(all_rows)
    passed = sum(row.get("status") == "PARITY_OK" for row in all_rows)
    mismatches = sum(row.get("status") == "PARITY_MISMATCH" for row in all_rows)
    unsupported = sum(result.get("status") == "UNSUPPORTED_BY_SERVER" for row in all_rows for result in row.get("results", {}).values())
    categories = sorted({row.get("category", "") for row in all_rows if row.get("category")})
    style = '<style>body{font-family:system-ui,sans-serif;background:#0f172a;color:#f8fafc;margin:2rem;line-height:1.5}h1,h2{color:#fff}.subtitle,small{color:#94a3b8}.kpi-grid{display:flex;gap:1rem;flex-wrap:wrap;margin:1.5rem 0}.kpi-card,.matrix{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:1rem;margin:1rem 0}.kpi-value{font-size:1.7rem}.kpi-label{color:#94a3b8}.search-input{padding:.7rem;width:min(40rem,90%);margin:.5rem 0 1rem}.table-container{overflow-x:auto}table{border-collapse:collapse;width:100%;font-size:.85rem}th,td{border-bottom:1px solid #334155;padding:.6rem;text-align:left;vertical-align:top}code{overflow-wrap:anywhere}.status{font-weight:650}</style>'
    out = [
        "<!DOCTYPE html>", '<html lang="en"><head><meta charset="utf-8">',
        '<meta name="viewport" content="width=device-width, initial-scale=1">',
        '<title>KMIPKit Interoperability Dashboard</title>', style, '</head><body>',
        '<h1>KMIPKit Interoperability Dashboard</h1>',
        '<p class="subtitle">Every report run is retained. Comparability is evaluated within one server configuration and KMIP protocol version.</p>',
        '<div class="kpi-grid">',
        f'<div class="kpi-card"><div class="kpi-value">{len(aggregated.get("servers", []))}</div><div class="kpi-label">Servers</div></div>',
        f'<div class="kpi-card"><div class="kpi-value">{total_cases}</div><div class="kpi-label">Case rows</div></div>',
        f'<div class="kpi-card"><div class="kpi-value">{passed}</div><div class="kpi-label">Parity OK</div></div>',
        f'<div class="kpi-card"><div class="kpi-value">{mismatches}</div><div class="kpi-label">Language/route differences</div></div>',
        f'<div class="kpi-card"><div class="kpi-value">{unsupported}</div><div class="kpi-label">Server Unsupported</div></div>',
        '</div>', '<label for="search-input">Filter cases</label><br>',
        '<input id="search-input" class="search-input" type="search" placeholder="Search case, language, route, or result">',
        '<div class="filter-group">Categories: ' + ', '.join(html.escape(category) for category in categories) + '</div>',
    ]
    for matrix in matrices:
        runs = matrix.get("runs", [])
        config_fp = runs[0].get("server", {}).get("configuration_fingerprint") if runs else None
        summary = matrix.get("summary", {})
        out.extend([
            '<section class="matrix">',
            f'<h2>Server <code>{html.escape(str(matrix.get("server_id", "unknown")))}</code> — KMIP {html.escape(str(matrix.get("protocol_version", "unknown")))}</h2>',
            f'<p>Evidence comparability: <strong>{html.escape(str(matrix.get("comparability", "UNVERIFIED")))}</strong> · Server configuration: <code>{html.escape(str(config_fp or "unknown"))}</code></p>',
            f'<p>Library correctness: <code>{html.escape(json.dumps(summary.get("library_correctness", {}), sort_keys=True))}</code> · Server compatibility: <code>{html.escape(json.dumps(summary.get("server_compatibility", {}), sort_keys=True))}</code> · Not executed (not_run): {summary.get("not_executed", 0)}</p>',
            '<div class="table-container"><table><thead><tr><th>Case</th><th>Category</th>',
        ])
        for run in runs:
            execution = run.get("execution", {})
            label = f"{run.get('result_key', run.get('run_id'))} · {run.get('language')}"
            axes = f"{execution.get('transport') or 'unknown'} / {execution.get('encoding') or 'unknown'} / {execution.get('api_level') or 'unknown'}"
            out.append(f'<th>{html.escape(str(label))}<br><small>{html.escape(axes)}</small></th>')
        out.append('<th>Language/route differences</th></tr></thead><tbody>')
        for row in matrix.get("cases", []):
            out.append(f'<tr class="matrix-row"><td><code>{html.escape(str(row.get("case_id", "")))}</code></td><td>{html.escape(str(row.get("category", "")))}</td>')
            for run in runs:
                result = row.get("results", {}).get(run.get("result_key", run.get("run_id")), {})
                status_code = result.get("result_status_code")
                reason_code = result.get("result_reason_code")
                details = (
                    f"{result.get('status', 'NOT_RUN')} · library {result.get('library_correctness', 'not_checked')} · "
                    f"server {result.get('server_compatibility', 'unknown')} · parity {result.get('parity', 'not_compared')} · "
                    f"KMIP {status_code if status_code is not None else '-'}:"
                    f"{reason_code if reason_code is not None else '-'}"
                )
                out.append(f'<td class="status">{html.escape(details)}</td>')
            difference = ", ".join(row.get("difference_dimensions", []))
            row_status = str(row.get("status", "PARITY_OK")) + (f" ({difference})" if difference else "")
            out.append(f'<td>{html.escape(row_status)}</td></tr>')
        out.append('</tbody></table></div></section>')
    out.extend([
        '<script>document.getElementById("search-input").addEventListener("input",function(){const q=this.value.toLowerCase();document.querySelectorAll(".matrix-row").forEach(r=>{r.style.display=r.innerText.toLowerCase().includes(q)?"":"none";});});</script>',
        '<footer>KMIPKit interoperability results · self-contained offline report</footer>', '</body></html>',
    ])
    return "\n".join(out)

def main():
    parser = argparse.ArgumentParser(description="Aggregate Multi-Language & Multi-KMS validation results")
    parser.add_argument("results", nargs="+", type=Path, help="Paths to results.json files")
    parser.add_argument("--format", choices=["json", "markdown", "html"], default="markdown", help="Output format")
    parser.add_argument("--output", type=Path, help="Optional output destination file")

    args = parser.parse_args()
    aggregated = aggregate_results(args.results)

    if args.format == "json":
        out = json.dumps(aggregated, indent=2)
    elif args.format == "html":
        out = generate_html(aggregated)
    else:
        out = generate_markdown(aggregated)

    if args.output:
        out_p = args.output.resolve()
        out_p.parent.mkdir(parents=True, exist_ok=True)
        with open(out_p, "w", encoding="utf-8") as f:  # NOSONAR
            f.write(out)
        print(f"Wrote master matrix to {out_p}")
    else:
        print(out)


if __name__ == "__main__":
    main()
