#!/usr/bin/env python3
"""Write a concise, event-aware summary for a GitHub Actions CI run."""

from __future__ import annotations

import html
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Mapping


PR_REQUIRED_JOBS = (
    ("Core matrix (3 OS × Rust 1.94 and stable)", "core"),
    ("Script contracts (3 OS)", "script-contracts"),
    ("Language bindings (3 OS)", "language-bindings"),
    ("FFI sanitizer", "ffi-sanitizer"),
    ("Fuzz smoke", "fuzz-smoke"),
    ("Normative inventory", "normative-inventory"),
    ("Coverage collection (3 OS)", "coverage"),
    ("Coverage gate", "coverage-gate"),
    ("Adapter coverage", "adapter-coverage"),
    ("Dependency policy", "dependency-policy"),
)
SCHEDULED_JOBS = (
    ("Scheduled dependency policy", "scheduled-dependency-policy"),
)
PR_ONLY_JOBS = tuple(label for label, _ in PR_REQUIRED_JOBS)
KNOWN_RESULTS = {"success", "failure", "cancelled", "skipped"}
RESULT_LABELS = {
    "success": "✅ PASS",
    "failure": "❌ FAIL",
    "cancelled": "⏹️ CANCELLED",
    "skipped": "⚪ SKIPPED",
    "missing": "❓ MISSING",
    "unknown": "❓ UNKNOWN",
}
MATRIX_JOB_IDS = {"core", "script-contracts", "language-bindings", "coverage"}


def _safe_text(value: Any) -> str:
    """Escape dynamic values before inserting them into Markdown text or tables."""
    text = " ".join(str(value).split())
    return (
        html.escape(text, quote=False)
        .replace("|", "&#124;")
        .replace("`", "&#96;")
        .replace("[", "&#91;")
        .replace("]", "&#93;")
    )


def _job_entry(needs: Mapping[str, Any], job_id: str) -> Mapping[str, Any]:
    value = needs.get(job_id)
    return value if isinstance(value, Mapping) else {}


def _job_result(needs: Mapping[str, Any], job_id: str) -> str:
    entry = _job_entry(needs, job_id)
    result = entry.get("result")
    if not isinstance(result, str):
        return "missing"
    normalized = result.lower()
    return normalized if normalized in KNOWN_RESULTS else "unknown"


def _branch_coverage_summary_status(needs: Mapping[str, Any]) -> str:
    outputs = _job_entry(needs, "branch-coverage").get("outputs")
    if isinstance(outputs, Mapping):
        status = outputs.get("summary_status")
        if status in {"passed", "failed", "unavailable"}:
            return str(status)
    result = _job_result(needs, "branch-coverage")
    if result == "failure":
        return "failed"
    return "unavailable"


def _render_required_row(label: str, job_id: str, needs: Mapping[str, Any]) -> tuple[str, bool]:
    result = _job_result(needs, job_id)
    return f"| {_safe_text(label)} | {RESULT_LABELS[result]} |", result == "success"


def _render_required_rows(
    job_group: tuple[tuple[str, str], ...], needs: Mapping[str, Any]
) -> tuple[list[str], list[bool]]:
    """Render one required group and return its pass flags in display order."""
    rendered = [_render_required_row(label, job_id, needs) for label, job_id in job_group]
    return [row for row, _ in rendered], [passed for _, passed in rendered]


def _diagnostic_records(needs: Mapping[str, Any], job_id: str) -> list[Mapping[str, str]]:
    if job_id in MATRIX_JOB_IDS:
        return []
    outputs = _job_entry(needs, job_id).get("outputs")
    if not isinstance(outputs, Mapping):
        return []
    serialized = outputs.get("diagnostic_details")
    if not isinstance(serialized, str):
        return []
    try:
        records = json.loads(serialized)
    except json.JSONDecodeError:
        return []
    if not isinstance(records, list):
        return []
    return [
        record
        for record in records
        if isinstance(record, Mapping)
        and isinstance(record.get("label"), str)
        and isinstance(record.get("explanation"), str)
    ]


def _failure_details(
    job_group: tuple[tuple[str, str], ...], needs: Mapping[str, Any]
) -> list[str]:
    details: list[str] = []
    for label, job_id in job_group:
        result = _job_result(needs, job_id)
        if result == "success":
            continue
        details.append(f"- **{_safe_text(label)} — {RESULT_LABELS[result]}**")
        records = _diagnostic_records(needs, job_id)
        if records:
            details.extend(
                f"  - **{_safe_text(record['label'])}:** {_safe_text(record['explanation'])}"
                for record in records
            )
        elif job_id in MATRIX_JOB_IDS:
            details.append(
                "  - One or more matrix legs failed; open the failed OS/toolchain leg and its job Summary for the exact step diagnosis."
            )
        elif result in {"missing", "skipped"}:
            details.append(
                "  - This required check did not run or did not return a result. Check the listed upstream failures or cancellation."
            )
        else:
            details.append(
                "  - Open this job's Summary for its failed step and the native error output in that step's log."
            )
    return details


def _commit_link(context: Mapping[str, str], server_url: str) -> str:
    repository = context.get("repository", "")
    sha = context.get("sha", "")
    if re.fullmatch(r"[0-9a-fA-F]{40,64}", sha) and re.fullmatch(r"[^/\s]+/[^/\s]+", repository):
        short_sha = _safe_text(sha[:8])
        url = f"{server_url}/{repository}/commit/{sha}"
        return f"[`{short_sha}`]({html.escape(url, quote=True)})"
    return f"`{_safe_text(sha or 'unknown')}`"


def build_summary(
    *,
    event_name: str,
    context: Mapping[str, str],
    needs: Mapping[str, Any],
) -> tuple[str, int]:
    """Return the run-summary Markdown and a failing exit code when required jobs fail."""
    server_url = context.get("server_url", "https://github.com").rstrip("/")
    repository = context.get("repository", "")
    run_id = context.get("run_id", "")
    attempt = context.get("run_attempt", "1")
    run_url = ""
    if re.fullmatch(r"[^/\s]+/[^/\s]+", repository) and run_id.isdecimal() and attempt.isdecimal():
        run_url = f"{server_url}/{repository}/actions/runs/{run_id}/attempts/{attempt}"

    rows: list[str]
    required_results: list[bool]

    if event_name == "pull_request":
        rows, required_results = _render_required_rows(PR_REQUIRED_JOBS, needs)
        rows.extend(
            (
                "| Scheduled dependency policy | ⚪ Not applicable — schedule-triggered checks |",
                "| Branch coverage | ⚪ Not applicable — schedule-triggered checks |",
            )
        )
    elif event_name == "schedule":
        rows = [
            f"| {_safe_text(label)} | ⚪ Not applicable — pull-request check |"
            for label in PR_ONLY_JOBS
        ]
        scheduled_rows, required_results = _render_required_rows(SCHEDULED_JOBS, needs)
        rows.extend(scheduled_rows)

        branch_status = _branch_coverage_summary_status(needs)
        branch_text = {
            "passed": "ℹ️ Attempt completed — informational, does not gate CI",
            "failed": "⚠️ Attempt failed — informational, does not gate CI",
            "unavailable": "⚪ Unavailable — informational, does not gate CI",
        }[branch_status]
        rows.append(f"| Branch coverage | {branch_text} |")
    else:
        rows = ["| CI event | ❌ Unsupported event |"]
        required_results = [False]

    passed_count = sum(required_results)
    required_count = len(required_results)
    overall_passed = required_count > 0 and passed_count == required_count
    result_label = "PASS" if overall_passed else "FAIL"
    headline = "✅ CI passed" if overall_passed else "❌ CI needs attention"
    event_label = "Pull request" if event_name == "pull_request" else "Scheduled" if event_name == "schedule" else event_name
    run_reference = f"[#{_safe_text(run_id)} (attempt {_safe_text(attempt)})]({html.escape(run_url, quote=True)})" if run_url else "unavailable"

    lines = [
        f"# {headline}",
        "",
        f"**CI result: {result_label}** · Required groups passed: {passed_count}/{required_count}.",
        "",
        f"**Event:** {_safe_text(event_label)} · **Ref:** `{_safe_text(context.get('ref', 'unknown'))}` · **Commit:** {_commit_link(context, server_url)} · **Run:** {run_reference}",
        "",
        "| Check group | Result |",
        "| --- | --- |",
        *rows,
    ]

    if not overall_passed:
        if event_name == "pull_request":
            failure_details = _failure_details(PR_REQUIRED_JOBS, needs)
        elif event_name == "schedule":
            failure_details = _failure_details(SCHEDULED_JOBS, needs)
        else:
            failure_details = ["- The workflow event is unsupported; no required-check policy is defined for it."]
        lines.extend(("", "## Failures to fix", "", *failure_details))

    if event_name == "schedule":
        scheduled_outputs = _job_entry(needs, "scheduled-dependency-policy").get("outputs")
        if isinstance(scheduled_outputs, Mapping):
            release_ref = scheduled_outputs.get("release_ref")
            scanned_commit = scheduled_outputs.get("scanned_commit")
            if isinstance(release_ref, str) and isinstance(scanned_commit, str):
                lines.extend(
                    (
                        "",
                        f"**Release scan:** `{_safe_text(release_ref)}` at `{_safe_text(scanned_commit[:12])}`.",
                    )
                )

        if _branch_coverage_summary_status(needs) == "failed":
            branch_diagnostics = _diagnostic_records(needs, "branch-coverage")
            lines.extend(("", "## Informational branch-coverage failure", ""))
            if branch_diagnostics:
                lines.extend(
                    f"- **{_safe_text(record['label'])}:** {_safe_text(record['explanation'])}"
                    for record in branch_diagnostics
                )
            else:
                lines.append("- Open the branch-coverage job Summary for its failed step; this check remains informational.")

    lines.extend(
        (
            "",
            "Each job Summary names its failed steps and explains where to find the native error. Full test, compiler, sanitizer, policy, and action output remains in the original step logs. Coverage measurements or an `unavailable` reason are shown in the Coverage gate summary.",
            "",
        )
    )
    return "\n".join(lines), 0 if overall_passed else 1


def _failure_summary(message: str) -> str:
    return "\n".join(("# ❌ CI summary could not be rendered", "", f"{_safe_text(message)}", ""))


def _write_console(markdown: str) -> None:
    stream = sys.stdout
    if hasattr(stream, "reconfigure"):
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    stream.write(markdown)


def main() -> int:
    """Write the GitHub job summary and preserve the required check result."""
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not summary_path or not output_path:
        print("GITHUB_STEP_SUMMARY and GITHUB_OUTPUT must be set.", file=sys.stderr)
        return 2

    renderer_succeeded = True
    try:
        needs = json.loads(os.environ.get("CI_NEEDS_JSON", "{}"))
        if not isinstance(needs, dict):
            raise ValueError("CI_NEEDS_JSON must contain a JSON object.")
        context = {
            "repository": os.environ.get("GITHUB_REPOSITORY", ""),
            "ref": os.environ.get("GITHUB_REF", ""),
            "sha": os.environ.get("GITHUB_SHA", ""),
            "run_id": os.environ.get("GITHUB_RUN_ID", ""),
            "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", "1"),
            "server_url": os.environ.get("GITHUB_SERVER_URL", "https://github.com"),
        }
        markdown, exit_code = build_summary(
            event_name=os.environ.get("GITHUB_EVENT_NAME", ""),
            context=context,
            needs=needs,
        )
    except (json.JSONDecodeError, TypeError, ValueError) as error:
        markdown = _failure_summary(f"Invalid workflow status input: {error}")
        exit_code = 1
        renderer_succeeded = False

    summary_written = False
    try:
        Path(summary_path).write_text(markdown, encoding="utf-8", newline="\n")
        _write_console(markdown)
        with Path(output_path).open("a", encoding="utf-8", newline="\n") as output:
            output.write(f"summary_written={'true' if renderer_succeeded else 'false'}\n")
    except OSError as error:
        print(f"CI run summary could not be written: {error}", file=sys.stderr)
        return 1
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
