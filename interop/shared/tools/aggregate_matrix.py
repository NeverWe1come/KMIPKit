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


def generate_html(aggregated: dict) -> str:
    """Generate a self-contained, air-gapped, offline HTML dashboard with zero CDN dependencies."""
    servers_str = ", ".join(aggregated["servers"]) or "None"
    langs = aggregated["languages"]
    total_cases = aggregated["total_cases"]

    # Calculate global KPIs
    total_parity_ok = 0
    total_mismatches = 0
    total_all_pass = 0
    total_unsupported = 0

    all_rows = []
    for s_id, rows in aggregated["parity_by_server"].items():
        for r in rows:
            all_rows.append(r)
            if r["status"] == "PARITY_OK":
                total_parity_ok += 1
            else:
                total_mismatches += 1

            lang_statuses = list(r["results"].values())
            if lang_statuses and all(st == "PASSED" for st in lang_statuses):
                total_all_pass += 1
            if any(st == "UNSUPPORTED_BY_SERVER" for st in lang_statuses):
                total_unsupported += 1

    categories = sorted(list({r["category"] for r in all_rows if r["category"]}))

    css = """
    :root {
      --bg: #0f172a;
      --card-bg: #1e293b;
      --border: #334155;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --primary: #38bdf8;
      --success-bg: #064e3b;
      --success-text: #34d399;
      --warn-bg: #78350f;
      --warn-text: #fde047;
      --fail-bg: #7f1d1d;
      --fail-text: #fca5a5;
      --na-bg: #334155;
      --na-text: #cbd5e1;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background-color: var(--bg);
      color: var(--text);
      padding: 24px;
      line-height: 1.5;
    }
    header { margin-bottom: 24px; border-bottom: 1px solid var(--border); padding-bottom: 16px; }
    h1 { font-size: 26px; font-weight: 700; color: #fff; margin-bottom: 8px; display: flex; align-items: center; gap: 10px; }
    .subtitle { color: var(--text-muted); font-size: 14px; }
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }
    .kpi-card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 16px;
      text-align: center;
    }
    .kpi-value { font-size: 32px; font-weight: 700; margin-bottom: 4px; }
    .kpi-label { font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
    .kpi-blue { color: var(--primary); }
    .kpi-green { color: var(--success-text); }
    .kpi-yellow { color: var(--warn-text); }
    .kpi-purple { color: #c084fc; }

    .controls-panel {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 16px;
      margin-bottom: 24px;
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    .search-row { display: flex; gap: 12px; }
    .search-input {
      flex: 1;
      padding: 10px 14px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: #0f172a;
      color: #fff;
      font-size: 14px;
      outline: none;
    }
    .search-input:focus { border-color: var(--primary); }
    .filter-group { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
    .filter-label { font-size: 13px; color: var(--text-muted); min-width: 90px; }
    .pill {
      background: #0f172a;
      border: 1px solid var(--border);
      color: var(--text-muted);
      padding: 4px 12px;
      border-radius: 16px;
      font-size: 12px;
      cursor: pointer;
      user-select: none;
      transition: all 0.15s ease;
    }
    .pill:hover { border-color: var(--primary); color: #fff; }
    .pill.active { background: var(--primary); color: #0f172a; border-color: var(--primary); font-weight: 600; }

    .table-container {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      overflow-x: auto;
      margin-bottom: 24px;
    }
    table { width: 100%; border-collapse: collapse; text-align: left; font-size: 13px; }
    th {
      background: #111827;
      color: var(--text-muted);
      padding: 12px 14px;
      font-weight: 600;
      border-bottom: 1px solid var(--border);
      white-space: nowrap;
    }
    td { padding: 10px 14px; border-bottom: 1px solid #1e293b; vertical-align: middle; }
    tr:hover td { background: rgba(255, 255, 255, 0.02); }
    .case-id { font-family: monospace; font-weight: 600; color: #38bdf8; }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 600;
      white-space: nowrap;
    }
    .badge-pass { background: var(--success-bg); color: var(--success-text); }
    .badge-fail { background: var(--fail-bg); color: var(--fail-text); }
    .badge-warn { background: var(--warn-bg); color: var(--warn-text); }
    .badge-unsupp { background: #374151; color: #9ca3af; }
    .badge-na { background: var(--na-bg); color: var(--na-text); }
    footer { text-align: center; color: var(--text-muted); font-size: 12px; margin-top: 32px; }
    """

    js = """
    function applyFilters() {
      const q = document.getElementById('search-input').value.toLowerCase().trim();
      const activeCat = document.querySelector('.cat-pill.active').dataset.cat;
      const activeStatus = document.querySelector('.status-pill.active').dataset.status;

      const rows = document.querySelectorAll('.matrix-row');
      let visible = 0;
      rows.forEach(r => {
        const text = r.innerText.toLowerCase();
        const cat = r.dataset.category;
        const status = r.dataset.status;

        const matchQ = (!q || text.includes(q));
        const matchCat = (activeCat === 'ALL' || cat === activeCat);
        const matchStatus = (activeStatus === 'ALL' || status === activeStatus);

        if (matchQ && matchCat && matchStatus) {
          r.style.display = '';
          visible++;
        } else {
          r.style.display = 'none';
        }
      });
      document.getElementById('visible-count').innerText = visible;
    }

    document.addEventListener('DOMContentLoaded', () => {
      document.getElementById('search-input').addEventListener('input', applyFilters);

      document.querySelectorAll('.cat-pill').forEach(btn => {
        btn.addEventListener('click', () => {
          document.querySelectorAll('.cat-pill').forEach(b => b.classList.remove('active'));
          btn.classList.add('active');
          applyFilters();
        });
      });

      document.querySelectorAll('.status-pill').forEach(btn => {
        btn.addEventListener('click', () => {
          document.querySelectorAll('.status-pill').forEach(b => b.classList.remove('active'));
          btn.classList.add('active');
          applyFilters();
        });
      });
    });
    """

    out = []
    out.append("<!DOCTYPE html>")
    out.append("<html lang=\"en\">")
    out.append("<head>")
    out.append("  <meta charset=\"utf-8\">")
    out.append("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")
    out.append("  <title>KMIPKit Multi-Language Master Matrix</title>")
    out.append(f"  <style>{css}</style>")
    out.append(f"  <script>{js}</script>")
    out.append("</head>")
    out.append("<body>")

    # Header
    out.append("  <header>")
    out.append("    <h1>🛡️ KMIPKit Interoperability Dashboard</h1>")
    out.append(f"    <div class=\"subtitle\">Servers: <strong>{html.escape(servers_str)}</strong> | Languages: <strong>{html.escape(', '.join(langs))}</strong></div>")
    out.append("  </header>")

    # KPI Grid
    out.append("  <div class=\"kpi-grid\">")
    out.append(f"    <div class=\"kpi-card\"><div class=\"kpi-value kpi-blue\">{total_cases}</div><div class=\"kpi-label\">Total Test Cases</div></div>")
    out.append(f"    <div class=\"kpi-card\"><div class=\"kpi-value kpi-green\">{total_parity_ok}</div><div class=\"kpi-label\">Parity OK</div></div>")
    out.append(f"    <div class=\"kpi-card\"><div class=\"kpi-value kpi-yellow\">{total_mismatches}</div><div class=\"kpi-label\">Parity Mismatches</div></div>")
    out.append(f"    <div class=\"kpi-card\"><div class=\"kpi-value kpi-purple\">{total_unsupported}</div><div class=\"kpi-label\">Server Unsupported</div></div>")
    out.append("  </div>")

    # Controls Panel
    out.append("  <div class=\"controls-panel\">")
    out.append("    <div class=\"search-row\">")
    out.append("      <input type=\"text\" id=\"search-input\" class=\"search-input\" placeholder=\"🔍 Search test case ID, description, or category...\">")
    out.append("    </div>")

    # Status Filters
    out.append("    <div class=\"filter-group\">")
    out.append("      <span class=\"filter-label\">Parity:</span>")
    out.append("      <button class=\"pill status-pill active\" data-status=\"ALL\">All</button>")
    out.append("      <button class=\"pill status-pill\" data-status=\"PARITY_OK\">Parity OK</button>")
    out.append("      <button class=\"pill status-pill\" data-status=\"PARITY_MISMATCH\">Mismatch Only</button>")
    out.append("    </div>")

    # Category Filters
    out.append("    <div class=\"filter-group\">")
    out.append("      <span class=\"filter-label\">Category:</span>")
    out.append("      <button class=\"pill cat-pill active\" data-cat=\"ALL\">All</button>")
    for c in categories:
        out.append(f"      <button class=\"pill cat-pill\" data-cat=\"{html.escape(c)}\">{html.escape(c)}</button>")
    out.append("    </div>")
    out.append(f"    <div style=\"font-size: 12px; color: var(--text-muted);\">Showing <span id=\"visible-count\">{total_cases}</span> test cases</div>")
    out.append("  </div>")

    # Tables by Server
    for s_id, rows in aggregated["parity_by_server"].items():
        out.append(f"  <h2 style=\"margin: 20px 0 12px; font-size: 18px;\">Server: <code>{html.escape(s_id)}</code></h2>")
        out.append("  <div class=\"table-container\">")
        out.append("    <table>")
        out.append("      <thead><tr>")
        out.append("        <th>Case ID</th>")
        out.append("        <th>Category</th>")
        out.append("        <th>Description</th>")
        for l in langs:
            out.append(f"        <th>{html.escape(l.capitalize())}</th>")
        out.append("        <th>Parity</th>")
        out.append("      </tr></thead>")
        out.append("      <tbody>")

        for r in rows:
            cid = r["case_id"]
            cat = r["category"]
            desc = r.get("description", "")
            status = r["status"]

            out.append(f"      <tr class=\"matrix-row\" data-category=\"{html.escape(cat)}\" data-status=\"{status}\">")
            out.append(f"        <td class=\"case-id\">{html.escape(cid)}</td>")
            out.append(f"        <td><span class=\"badge badge-na\">{html.escape(cat)}</span></td>")
            out.append(f"        <td style=\"color: var(--text-muted);\">{html.escape(desc)}</td>")

            for l in langs:
                res = r["results"].get(l, "NOT_RUN")
                if res == "PASSED":
                    badge = "<span class=\"badge badge-pass\">PASS</span>"
                elif res == "FAILED":
                    badge = "<span class=\"badge badge-fail\">FAIL</span>"
                elif res == "UNSUPPORTED_BY_SERVER":
                    badge = "<span class=\"badge badge-unsupp\">UNSUPP</span>"
                else:
                    badge = "<span class=\"badge badge-na\">N/A</span>"
                out.append(f"        <td>{badge}</td>")

            parity_badge = "<span class=\"badge badge-pass\">OK</span>" if status == "PARITY_OK" else "<span class=\"badge badge-warn\">MISMATCH</span>"
            out.append(f"        <td>{parity_badge}</td>")
            out.append("      </tr>")

        out.append("      </tbody>")
        out.append("    </table>")
        out.append("  </div>")

    out.append("  <footer>KMIPKit Interoperability Suite &bull; Self-Contained Air-Gapped Dashboard (Zero CDN)</footer>")
    out.append("</body>")
    out.append("</html>")
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
