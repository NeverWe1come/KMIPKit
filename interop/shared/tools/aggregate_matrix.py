#!/usr/bin/env python3
"""Cross-Language and Multi-KMS Matrix Aggregator for KMIPKit Interop."""

from __future__ import annotations

import argparse
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
    data_by_server_lang = {}  # (server_id, language) -> dict of case_id -> case_res

    all_servers = set()
    all_languages = set()
    all_cases = set()
    case_meta = {}

    for rf in result_files:
        payload = parse_result_file(rf)
        if not payload:
            continue
        server_id = payload.get("server_identifier", "unknown_server")
        lang = payload.get("language", "unknown_lang")
        all_servers.add(server_id)
        all_languages.add(lang)

        case_map = {}
        for c in payload.get("cases", []):
            cid = c.get("case_id")
            if not cid:
                continue
            all_cases.add(cid)
            case_map[cid] = c
            if cid not in case_meta:
                case_meta[cid] = {
                    "category": c.get("category", ""),
                    "description": c.get("description", "")
                }
        data_by_server_lang[(server_id, lang)] = case_map

    # 1. Build Parity Matrix per Server
    parity_by_server = {}
    for server_id in sorted(all_servers):
        rows = []
        for cid in sorted(all_cases):
            row = {
                "case_id": cid,
                "category": case_meta.get(cid, {}).get("category", ""),
                "description": case_meta.get(cid, {}).get("description", ""),
                "results": {},
                "status": "PARITY_OK"
            }
            statuses = set()
            for lang in sorted(all_languages):
                cmap = data_by_server_lang.get((server_id, lang), {})
                c_res = cmap.get(cid)
                if c_res:
                    st = c_res.get("status")
                    row["results"][lang] = st
                    if st in ("PASSED", "FAILED"):
                        statuses.add(st)
                else:
                    row["results"][lang] = "NOT_RUN"

            if len(statuses) > 1:
                row["status"] = "PARITY_MISMATCH"
            rows.append(row)
        parity_by_server[server_id] = rows

    return {
        "servers": sorted(list(all_servers)),
        "languages": sorted(list(all_languages)),
        "total_cases": len(all_cases),
        "parity_by_server": parity_by_server
    }


def generate_markdown(aggregated: dict) -> str:
    md = []
    md.append("# KMIPKit Cross-Language & Multi-KMS Master Matrix\n")
    md.append(f"- **Servers Evaluated**: {', '.join(aggregated['servers']) or 'None'}")
    md.append(f"- **Languages Evaluated**: {', '.join(aggregated['languages']) or 'None'}")
    md.append(f"- **Total Test Cases**: {aggregated['total_cases']}\n")

    for s_id, rows in aggregated["parity_by_server"].items():
        md.append(f"## Server: `{s_id}`\n")
        langs = aggregated["languages"]
        header = ["Test Case", "Category"] + [l.capitalize() for l in langs] + ["Parity Status"]
        md.append("| " + " | ".join(header) + " |")
        md.append("| " + " | ".join(["---"] * len(header)) + " |")

        for r in rows:
            line = [f"`{r['case_id']}`", r["category"]]
            for l in langs:
                res = r["results"].get(l, "NOT_RUN")
                badge = "✅ PASS" if res == "PASSED" else ("❌ FAIL" if res == "FAILED" else ("➖ UNSUPP" if res == "UNSUPPORTED_BY_SERVER" else "⚪ N/A"))
                line.append(badge)
            status_badge = "✅ OK" if r["status"] == "PARITY_OK" else "⚠️ **MISMATCH**"
            line.append(status_badge)
            md.append("| " + " | ".join(line) + " |")
        md.append("\n")

    return "\n".join(md)


def main():
    parser = argparse.ArgumentParser(description="Aggregate Multi-Language & Multi-KMS validation results")
    parser.add_argument("results", nargs="+", type=Path, help="Paths to results.json files")
    parser.add_argument("--format", choices=["json", "markdown"], default="markdown", help="Output format")
    parser.add_argument("--output", type=Path, help="Optional output destination file")

    args = parser.parse_args()
    aggregated = aggregate_results(args.results)

    if args.format == "json":
        out = json.dumps(aggregated, indent=2)
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
