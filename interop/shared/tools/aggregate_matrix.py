#!/usr/bin/env python3
"""Cross-Language and Multi-KMS Matrix Aggregator for KMIPKit Interop."""

from __future__ import annotations

import argparse
import html
import json
import sys
from collections import defaultdict
from pathlib import Path


def parse_result_file(file_path: Path) -> dict | None:
    p = file_path.resolve()
    if not p.exists():
        return None
    try:
        with open(p, "r", encoding="utf-8") as f:  # NOSONAR
            return json.load(f)
    except Exception:
        return None


def aggregate_results(result_files: list[Path]) -> dict:
    groups = defaultdict(list)
    cases_by_group = defaultdict(set)
    all_cases = set()
    all_servers = set()
    all_languages = set()
    case_metadata = {}
    catalog_path = Path(__file__).resolve().parents[1] / "catalog" / "kmipkit-1.0-test-catalog.json"
    try:
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        case_metadata = {
            case["case_id"]: {"category": case.get("category", ""), "description": case.get("description", "")}
            for case in catalog.get("test_cases", [])
        }
    except (OSError, ValueError, KeyError):
        case_metadata = {}
    status_names = {
        "passed": "PASSED", "failed": "FAILED",
        "server_unsupported": "UNSUPPORTED_BY_SERVER",
        "policy_restricted": "POLICY_RESTRICTED", "not_run": "NOT_RUN",
    }

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
            case_status = status_names.get(case.get("status"), case.get("status", "NOT_RUN"))
            safe_fields = (
                "library_correctness", "server_compatibility",
                "parity", "duration_ms", "result_status", "result_reason",
            )
            run_cases[case_id] = {
                **{key: case[key] for key in safe_fields if key in case},
                "case_id": case_id,
                "status": case_status,
            }
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
            "summary": payload.get("summary", {}),
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
        evidence = {
            (
                run["server"]["id"], run["server"]["version"],
                run["server"]["configuration_fingerprint"], run["catalog_sha256"],
                run["execution"]["library_commit"],
            )
            for run in runs
        }
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
                "status": "PARITY_OK",
            }
            observed_statuses = set()
            for run in runs:
                case = run["cases"].get(case_id)
                result = {
                    "run_id": run["run_id"],
                    "language": run["language"],
                    "transport": run["execution"]["transport"],
                    "encoding": run["execution"]["encoding"],
                    "api_level": run["execution"]["api_level"],
                    "status": case.get("status", "NOT_RUN") if case else "NOT_RUN",
                    "library_correctness": case.get("library_correctness", "not_checked") if case else "not_checked",
                    "server_compatibility": case.get("server_compatibility", "unknown") if case else "unknown",
                    "parity": case.get("parity", "not_compared") if case else "not_compared",
                }
                row["results"][run["result_key"]] = result
                if result["status"] != "NOT_RUN":
                    observed_statuses.add(result["status"])
                correctness[result["library_correctness"]] += 1
                compatibility[result["server_compatibility"]] += 1
                if result["status"] == "NOT_RUN":
                    not_executed += 1
            if len(observed_statuses) > 1:
                row["status"] = "PARITY_MISMATCH"
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
                cells.append(
                    f"{result['status']} / library:{result['library_correctness']} / server:{result['server_compatibility']}"
                )
            cells.append(row["status"])
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
                details = f"{result.get('status', 'NOT_RUN')} · library {result.get('library_correctness', 'not_checked')} · server {result.get('server_compatibility', 'unknown')}"
                out.append(f'<td class="status">{html.escape(details)}</td>')
            out.append(f'<td>{html.escape(str(row.get("status", "PARITY_OK")))}</td></tr>')
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
