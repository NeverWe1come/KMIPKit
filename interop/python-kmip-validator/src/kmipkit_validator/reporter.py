from __future__ import annotations

import json
from html import escape
from pathlib import Path
from .models import CaseStatus, ValidationSummary


class MatrixReporter:
    @staticmethod
    def render_ansi(summary: ValidationSummary) -> None:
        print(f"\n=== KMIP 2.1 Validation Matrix: Server [{summary.server_id}] (Lang: {summary.language}) ===")
        print(
            f"Total: {summary.total_cases} | ✅ Passed: {summary.passed} | "
            f"❌ Failed: {summary.failed} | ➖ Unsupported: {summary.unsupported}"
        )
        print("-" * 80)
        print(f"{'Case ID':<35} | {'Operation':<25} | {'Status':<10}")
        print("-" * 80)

        for r in summary.results:
            if r.status == CaseStatus.PASSED:
                sym = "✅ PASS"
            elif r.status == CaseStatus.FAILED:
                sym = "❌ FAIL"
            elif r.status in (CaseStatus.UNSUPPORTED_SKIPPED, CaseStatus.UNSUPPORTED_VERIFIED_REJECTED):
                sym = "➖ UNSUPP"
            else:
                sym = "⚠️ UNEXP"
            print(f"{r.case_id:<35} | {r.operation:<25} | {sym:<10}")

        print("-" * 80)

    @staticmethod
    def render_markdown(summary: ValidationSummary, out_file: Path) -> None:
        lines = [
            f"# KMIP 2.1 Conformance Matrix: `{summary.server_id}`\n",
            f"- **Language**: `{summary.language}`",
            f"- **Total Cases**: {summary.total_cases}\n",
            "### Summary\n",
            "| Metric | Count |",
            "|---|---:|",
            f"| Total Cases | {summary.total_cases} |",
            f"| ✅ Passed | {summary.passed} |",
            f"| ❌ Failed | {summary.failed} |",
            f"| ➖ Unsupported | {summary.unsupported} |\n",
            "### Detailed Operations\n",
            "| Case ID | Operation | Status |",
            "|---|---|:---:|",
        ]
        for r in summary.results:
            sym = "✅" if r.status == CaseStatus.PASSED else "❌" if r.status == CaseStatus.FAILED else "➖"
            lines.append(f"| `{r.case_id}` | {r.operation} | {sym} |")

        out_file.parent.mkdir(parents=True, exist_ok=True)
        out_file.write_text("\n".join(lines) + "\n", encoding="utf-8")

    @staticmethod
    def render_html(summary: ValidationSummary, out_file: Path) -> None:
        rows = []
        for r in summary.results:
            cls = "pass" if r.status == CaseStatus.PASSED else "fail" if r.status == CaseStatus.FAILED else "unsupp"
            st = "✅ Passed" if r.status == CaseStatus.PASSED else "❌ Failed" if r.status == CaseStatus.FAILED else "➖ Unsupported"
            rows.append(f"<tr><td><code>{r.case_id}</code></td><td>{r.operation}</td><td class=\"{cls}\">{st}</td></tr>")

        html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>KMIP 2.1 Validation Report</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; background: #f8fafc; color: #1e293b; }}
.card {{ background: white; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 1rem; }}
th, td {{ text-align: left; padding: 0.75rem 1rem; border-bottom: 1px solid #e2e8f0; }}
th {{ background: #f1f5f9; font-weight: 600; }}
.pass {{ color: #16a34a; font-weight: bold; }}
.fail {{ color: #dc2626; font-weight: bold; }}
.unsupp {{ color: #64748b; font-weight: bold; }}
</style>
</head>
<body>
<div class="card">
<h1>KMIP 2.1 Validation Matrix: {summary.server_id}</h1>
<p><strong>Language:</strong> {summary.language}</p>
<p>✅ <strong>Passed:</strong> {summary.passed} | ❌ <strong>Failed:</strong> {summary.failed} | ➖ <strong>Unsupported:</strong> {summary.unsupported}</p>
</div>
<div class="card">
<table>
<thead>
<tr><th>Case ID</th><th>Operation</th><th>Status</th></tr>
</thead>
<tbody>
{"".join(rows)}
</tbody>
</table>
</div>
</body>
</html>
"""
        out_file.parent.mkdir(parents=True, exist_ok=True)
        out_file.write_text(html_content, encoding="utf-8")

    @staticmethod
    def render_json(summary: ValidationSummary, out_file: Path) -> None:
        data = {
            "run_id": "run-python",
            "language": summary.language,
            "server_id": summary.server_id,
            "summary": {
                "total_cases": summary.total_cases,
                "passed": summary.passed,
                "failed": summary.failed,
                "unsupported_by_server": summary.unsupported,
            },
            "results": [
                {
                    "case_id": r.case_id,
                    "operation": r.operation,
                    "status": r.status.value,
                    "detail": r.detail,
                }
                for r in summary.results
            ],
        }
        out_file.parent.mkdir(parents=True, exist_ok=True)
        out_file.write_text(json.dumps(data, indent=2), encoding="utf-8")

    @classmethod
    def write_reports(cls, summary: ValidationSummary, output_dir: Path) -> None:
        cls.render_markdown(summary, output_dir / "report.md")
        cls.render_html(summary, output_dir / "report.html")
        cls.render_json(summary, output_dir / "results.json")


def write_interop_reports(report: dict, output_dir: Path) -> None:
    """Write the shared schema-2.0 evidence in JSON, Markdown and HTML."""
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "results.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    summary = report["summary"]
    lines = [
        "# KMIPKit Python interoperability evidence",
        "",
        f"- Server: `{report['server']['id']}` ({report['server']['product']} {report['server']['version']})",
        f"- Protocol: KMIP {report['protocol_version']}",
        f"- Library commit: `{report['execution']['library_commit']}`",
        f"- Artifact SHA-256: `{report['execution']['artifact_sha256']}`",
        "",
        "| Outcome | Count |",
        "|---|---:|",
    ]
    for name in ("total", "passed", "failed", "unsupported", "policy_restricted", "not_run"):
        lines.append(f"| {name.replace('_', ' ').title()} | {summary[name]} |")
    lines.extend(["", "| Case | Status | KMIPKit | Server | Parity |", "|---|---|---|---|---|"])
    for case in report["cases"]:
        lines.append(
            f"| `{case['case_id']}` | {case['status']} | {case['library_correctness']} | "
            f"{case['server_compatibility']} | {case['parity']} |"
        )
    (output_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    rows = "".join(
        "<tr>"
        f"<td><code>{escape(case['case_id'])}</code></td>"
        f"<td>{escape(case['status'])}</td>"
        f"<td>{escape(case['library_correctness'])}</td>"
        f"<td>{escape(case['server_compatibility'])}</td>"
        f"<td>{escape(case['parity'])}</td>"
        "</tr>"
        for case in report["cases"]
    )
    table = "".join(
        f"<tr><th>{escape(name.replace('_', ' ').title())}</th><td>{summary[name]}</td></tr>"
        for name in ("total", "passed", "failed", "unsupported", "policy_restricted", "not_run")
    )
    html = (
        "<!doctype html><html lang='en'><meta charset='utf-8'><title>KMIPKit interoperability evidence</title>"
        "<h1>KMIPKit Python interoperability evidence</h1>"
        f"<p>Server {escape(report['server']['id'])}; KMIP {escape(report['protocol_version'])}; "
        f"library {escape(report['execution']['library_commit'])}</p>"
        f"<table><thead><tr><th>Outcome</th><th>Count</th></tr></thead><tbody>{table}</tbody></table>"
        "<table><thead><tr><th>Case</th><th>Status</th><th>KMIPKit</th><th>Server</th><th>Parity</th></tr></thead>"
        f"<tbody>{rows}</tbody></table></html>\n"
    )
    (output_dir / "report.html").write_text(html, encoding="utf-8")
