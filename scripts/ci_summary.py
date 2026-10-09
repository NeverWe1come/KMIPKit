#!/usr/bin/env python3
"""Write a concise, event-aware summary for a GitHub Actions CI run."""

from __future__ import annotations

import html
import importlib.util
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Mapping


PR_REQUIRED_JOBS = (
    ("Documentation contracts", "docs-contracts"),
    ("Core matrix (3 OS × Rust 1.94 and stable)", "core"),
    ("Script contracts (3 OS)", "script-contracts"),
    ("C consumer (3 OS)", "language-c"),
    ("Java and JNI integration (3 OS)", "language-java"),
    ("Python bindings (3 OS)", "language-python"),
    ("C FFI sanitizer", "ffi-sanitizer-c"),
    ("JNI sanitizer", "ffi-sanitizer-jni"),
    ("Fuzz smoke", "fuzz-smoke"),
    ("Normative inventory", "normative-inventory"),
    ("Rust and FFI coverage (3 OS)", "coverage"),
    ("Java coverage", "coverage-java"),
    ("Python coverage", "coverage-python"),
    ("JNI coverage", "coverage-jni"),
    ("Coverage gate", "coverage-gate"),
    ("Dependency policy", "dependency-policy"),
)
SCHEDULED_JOBS = (("Scheduled dependency policy", "scheduled-dependency-policy"),)
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
MATRIX_JOB_IDS = {
    "core",
    "script-contracts",
    "language-c",
    "language-java",
    "language-python",
    "coverage",
}
IMPACT_API = None
IMPACT_API_ERROR: Exception | None = None


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


def _impact_api():
    global IMPACT_API, IMPACT_API_ERROR
    if IMPACT_API is not None:
        return IMPACT_API
    if IMPACT_API_ERROR is not None:
        raise ValueError(f"Impact-plan validator is unavailable: {IMPACT_API_ERROR}")
    path = Path(__file__).with_name("ci_impact.py")
    try:
        spec = importlib.util.spec_from_file_location("ci_impact_for_summary", path)
        if spec is None or spec.loader is None:
            raise ImportError("ci_impact.py has no importable module loader")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        IMPACT_API = module
        return module
    except Exception as error:
        IMPACT_API_ERROR = error
        raise ValueError(f"Impact-plan validator is unavailable: {error}") from error


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


def _impact_plan(
    needs: Mapping[str, Any], context: Mapping[str, str]
) -> tuple[Mapping[str, Any] | None, str, bool]:
    """Return a validated plan, an error reason, and classifier-result status."""
    classifier_result = _job_result(needs, "impact-plan")
    outputs = _job_entry(needs, "impact-plan").get("outputs")
    serialized = outputs.get("plan_json") if isinstance(outputs, Mapping) else None
    if not isinstance(serialized, str):
        return None, "impact plan output is missing", classifier_result == "success"
    try:
        plan = json.loads(serialized)
    except json.JSONDecodeError as error:
        return None, f"impact plan JSON is invalid: {error}", classifier_result == "success"
    try:
        valid, reason = _impact_api().validate_plan(
            plan,
            expected_base_sha=context.get("base_sha") or None,
            expected_merge_sha=context.get("sha") or None,
        )
    except (TypeError, ValueError) as error:
        return None, str(error), classifier_result == "success"
    if not valid:
        return None, reason, classifier_result == "success"
    if classifier_result != "success":
        return None, f"impact classifier result is {classifier_result}", False
    return plan, "", True


def _skip_reason(job_id: str, plan: Mapping[str, Any]) -> str:
    classes = ", ".join(str(value) for value in plan.get("classes", [])) or "no classified component"
    return f"impact plan selected {classes}; this job is not affected"


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
    job_group: tuple[tuple[str, str], ...], needs: Mapping[str, Any], selected_jobs: set[str]
) -> list[str]:
    details: list[str] = []
    for label, job_id in job_group:
        result = _job_result(needs, job_id)
        if result == "success" or (result == "skipped" and job_id not in selected_jobs):
            continue
        details.append(f"- **{_safe_text(label)} (`{job_id}`) — {RESULT_LABELS[result]}**")
        records = _diagnostic_records(needs, job_id)
        if records:
            details.extend(
                f"  - **{_safe_text(record['label'])}:** {_safe_text(record['explanation'])}"
                for record in records
            )
        elif job_id in MATRIX_JOB_IDS:
            details.append(
                "  - One or more matrix legs failed or were omitted; open the affected OS/toolchain leg and its job Summary."
            )
        elif result == "skipped":
            details.append("  - This job was selected by the impact plan but was unexpectedly skipped.")
        elif result == "missing":
            details.append("  - The workflow did not report this job in the current run.")
        else:
            details.append("  - Open this job's Summary and native step log for the failure diagnosis.")
    return details


def _commit_link(context: Mapping[str, str], server_url: str) -> str:
    repository = context.get("repository", "")
    sha = context.get("sha", "")
    if re.fullmatch(r"[0-9a-fA-F]{40,64}", sha) and re.fullmatch(r"[^/\s]+/[^/\s]+", repository):
        short_sha = _safe_text(sha[:8])
        url = f"{server_url}/{repository}/commit/{sha}"
        return f"[`{short_sha}`]({html.escape(url, quote=True)})"
    return f"`{_safe_text(sha or 'unknown')}`"


def _pull_request_rows(
    needs: Mapping[str, Any], context: Mapping[str, str]
) -> tuple[list[str], list[bool], list[tuple[str, str]], set[str], str]:
    plan, plan_error, classifier_ok = _impact_plan(needs, context)
    selected_jobs = set(plan["selected_jobs"]) if plan is not None else {
        job_id for _, job_id in PR_REQUIRED_JOBS
    }
    rows: list[str] = []
    required_results: list[bool] = []

    classifier_result = _job_result(needs, "impact-plan")
    plan_passed = plan is not None and classifier_ok
    if plan_passed:
        mode = "full CI" if plan["full"] else "selective CI"
        row_result = f"✅ VALID — {_safe_text(mode)}"
    else:
        row_result = f"❌ INVALID — {_safe_text(plan_error or f'classifier {classifier_result}')}; full checks required"
    rows.append(f"| Impact classifier | {row_result} |")
    required_results.append(plan_passed)

    for label, job_id in PR_REQUIRED_JOBS:
        result = _job_result(needs, job_id)
        selected = job_id in selected_jobs
        if result == "missing":
            rendered = "❓ MISSING — no current-run result"
            passed = False
        elif result == "skipped" and not selected and plan is not None:
            rendered = f"⚪ Not affected — {_safe_text(_skip_reason(job_id, plan))}"
            passed = True
        elif result == "skipped":
            rendered = "❌ SKIPPED unexpectedly — selected by the impact plan"
            passed = False
        elif result == "success" and selected:
            rendered = RESULT_LABELS[result]
            passed = True
        elif result == "success":
            rendered = "✅ PASS — job ran although impact plan did not require it"
            passed = True
        else:
            rendered = RESULT_LABELS[result]
            passed = False
        rows.append(f"| {_safe_text(label)} (`{job_id}`) | {rendered} |")
        required_results.append(passed)

    rows.extend(
        (
            "| Scheduled dependency policy | ⚪ Not applicable — schedule-triggered checks |",
            "| Branch coverage | ⚪ Not applicable — schedule-triggered checks |",
        )
    )
    return rows, required_results, list(PR_REQUIRED_JOBS), selected_jobs, plan_error


def build_summary(
    *,
    event_name: str,
    context: Mapping[str, str],
    needs: Mapping[str, Any],
) -> tuple[str, int]:
    """Render event status and fail when selected checks or plan validation fail."""
    server_url = context.get("server_url", "https://github.com").rstrip("/")
    repository = context.get("repository", "")
    run_id = context.get("run_id", "")
    attempt = context.get("run_attempt", "1")
    run_url = ""
    if re.fullmatch(r"[^/\s]+/[^/\s]+", repository) and run_id.isdecimal() and attempt.isdecimal():
        run_url = f"{server_url}/{repository}/actions/runs/{run_id}/attempts/{attempt}"

    failures: list[tuple[str, str]] = []
    if event_name == "pull_request":
        rows, required_results, job_group, selected_jobs, plan_error = _pull_request_rows(needs, context)
        if plan_error:
            failures.append(("Impact classifier", plan_error))
    elif event_name == "schedule":
        rows = [
            f"| {_safe_text(label)} | ⚪ Not applicable — pull-request check |"
            for label in ("Impact classifier", *PR_ONLY_JOBS)
        ]
        scheduled_rows = []
        required_results = []
        job_group = SCHEDULED_JOBS
        selected_jobs = {job_id for _, job_id in SCHEDULED_JOBS}
        for label, job_id in SCHEDULED_JOBS:
            result = _job_result(needs, job_id)
            scheduled_rows.append(f"| {_safe_text(label)} (`{job_id}`) | {RESULT_LABELS[result]} |")
            required_results.append(result == "success")
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
        job_group = ()
        selected_jobs = set()

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
        f"**CI result: {result_label}** · Selected checks passed: {passed_count}/{required_count}.",
        "",
        f"**Event:** {_safe_text(event_label)} · **Ref:** `{_safe_text(context.get('ref', 'unknown'))}` · **Commit:** {_commit_link(context, server_url)} · **Run:** {run_reference}",
        "",
        "| Check group | Result |",
        "| --- | --- |",
        *rows,
    ]

    if not overall_passed:
        failure_details = _failure_details(job_group, needs, selected_jobs)
        if event_name == "pull_request" and failures:
            failure_details = [f"- **Impact classifier:** {_safe_text(failures[0][1])}", *failure_details]
        elif event_name not in {"pull_request", "schedule"}:
            failure_details = ["- The workflow event is unsupported; no required-check policy is defined for it."]
        lines.extend(("", "## Failures to fix", "", *failure_details))

    if event_name == "pull_request":
        plan, plan_error, _ = _impact_plan(needs, context)
        if plan is not None:
            mode = "Full CI" if plan["full"] else "Selective CI"
            lines.extend(
                (
                    "",
                    f"**Impact:** {_safe_text(mode)} · **Components:** {_safe_text(', '.join(plan['classes']) or 'none')} · **Reason:** {_safe_text(plan['reason'])}",
                )
            )
        elif plan_error:
            lines.extend(("", f"**Impact plan:** invalid — {_safe_text(plan_error)}; full checks were required."))

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
    """Write the GitHub job Summary and preserve the required check result."""
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
            "base_sha": os.environ.get("CI_PULL_REQUEST_BASE_SHA", ""),
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
