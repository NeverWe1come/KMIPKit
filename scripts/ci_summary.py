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
    ("Normative inventory", "normative-inventory"),
    ("Coverage collection (3 OS)", "coverage"),
    ("Coverage gate", "coverage-gate"),
    ("Dependency policy", "dependency-policy"),
)
SCHEDULED_JOBS = (
    ("Scheduled dependency policy", "scheduled-dependency-policy"),
)
PR_ONLY_JOBS = tuple(label for label, _ in PR_REQUIRED_JOBS)
KNOWN_RESULTS = {"success", "failure", "cancelled", "skipped"}
CANONICAL_REPOSITORY = "NeverWe1come/KMIPKit"
RELEASE_REF = "refs/heads/release/1.0.0"
EXPECTED_WORKFLOW_REF = (
    f"{CANONICAL_REPOSITORY}/.github/workflows/ci.yml@{RELEASE_REF}"
)
EVIDENCE_MANIFEST_SCHEMA_VERSION = 1
RESULT_LABELS = {
    "success": "✅ PASS",
    "failure": "❌ FAIL",
    "cancelled": "⏹️ CANCELLED",
    "skipped": "⚪ SKIPPED",
    "missing": "❓ MISSING",
    "unknown": "❓ UNKNOWN",
}


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    """Build an object while rejecting duplicate keys at every nesting level."""
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON object key")
        result[key] = value
    return result


def _positive_decimal(value: Any) -> bool:
    return isinstance(value, str) and value.isdecimal() and int(value) > 0


def _normalized_branch_protection(policy: Any) -> dict[str, Any]:
    if not isinstance(policy, Mapping) or set(policy) != {
        "protected",
        "required_pull_request_reviews",
        "enforce_admins",
        "allow_force_pushes",
        "allow_deletions",
    }:
        raise ValueError("branch-protection evidence is missing or malformed")
    if policy.get("protected") is not True:
        raise ValueError("branch-protection evidence is insufficient")

    reviews = policy.get("required_pull_request_reviews")
    if not isinstance(reviews, Mapping) or set(reviews) != {
        "required_approving_review_count",
        "dismiss_stale_reviews",
        "bypass_pull_request_allowances",
    }:
        raise ValueError("required review policy is missing or malformed")
    approvals = reviews.get("required_approving_review_count")
    if type(approvals) is not int or approvals < 1 or reviews.get("dismiss_stale_reviews") is not True:
        raise ValueError("required review policy is insufficient")
    allowances = reviews.get("bypass_pull_request_allowances")
    if not isinstance(allowances, Mapping) or set(allowances) != {"users", "teams", "apps"}:
        raise ValueError("pull-request bypass policy is missing or malformed")
    if any(type(allowances.get(name)) is not list or allowances[name] for name in ("users", "teams", "apps")):
        raise ValueError("pull-request bypass policy is insufficient")

    enforce_admins = policy.get("enforce_admins")
    allow_force_pushes = policy.get("allow_force_pushes")
    allow_deletions = policy.get("allow_deletions")
    if not isinstance(enforce_admins, Mapping) or set(enforce_admins) != {"enabled"}:
        raise ValueError("admin review policy is missing or malformed")
    if enforce_admins.get("enabled") is not True:
        raise ValueError("admin review policy is insufficient")
    if not isinstance(allow_force_pushes, Mapping) or set(allow_force_pushes) != {"enabled"}:
        raise ValueError("force-push policy is missing or malformed")
    if allow_force_pushes.get("enabled") is not False:
        raise ValueError("force-push policy is insufficient")
    if not isinstance(allow_deletions, Mapping) or set(allow_deletions) != {"enabled"}:
        raise ValueError("branch-deletion policy is missing or malformed")
    if allow_deletions.get("enabled") is not False:
        raise ValueError("branch-deletion policy is insufficient")

    return {
        "protected": True,
        "required_pull_request_reviews": {
            "required_approving_review_count": approvals,
            "dismiss_stale_reviews": True,
            "bypass_pull_request_allowances": {
                "users": [],
                "teams": [],
                "apps": [],
            },
        },
        "enforce_admins": {"enabled": True},
        "allow_force_pushes": {"enabled": False},
        "allow_deletions": {"enabled": False},
    }


def build_evidence_manifest(
    *,
    event_name: str,
    context: Mapping[str, Any],
    needs: Mapping[str, Any],
    branch_protection: Any,
) -> dict[str, Any]:
    """Create a strict ephemeral manifest from current run and job metadata."""
    if event_name != "push":
        raise ValueError("verification evidence requires a protected release push")
    if not isinstance(context, Mapping):
        raise ValueError("current workflow context is missing")
    repository = context.get("repository")
    ref = context.get("ref")
    ref_protected = context.get("ref_protected")
    sha = context.get("sha")
    workflow_ref = context.get("workflow_ref")
    run_id = context.get("run_id")
    run_attempt = context.get("run_attempt")
    if repository != CANONICAL_REPOSITORY:
        raise ValueError("verification evidence repository does not match")
    if ref != RELEASE_REF or ref_protected is not True:
        raise ValueError("verification evidence requires the protected release ref")
    if not isinstance(sha, str) or re.fullmatch(r"[0-9a-fA-F]{40,64}", sha) is None:
        raise ValueError("checked-out commit is missing or malformed")
    if workflow_ref != EXPECTED_WORKFLOW_REF:
        raise ValueError("verification evidence workflow does not match")
    if not _positive_decimal(run_id) or not _positive_decimal(run_attempt):
        raise ValueError("current workflow run identity is missing or malformed")
    if not isinstance(needs, Mapping):
        raise ValueError("required workflow checks are missing")

    checks: list[dict[str, str]] = []
    for _, job_id in PR_REQUIRED_JOBS:
        entry = needs.get(job_id)
        if not isinstance(entry, Mapping) or entry.get("result") != "success":
            raise ValueError(f"required workflow check {job_id} did not succeed")
        checks.append({"check_id": job_id, "job_id": job_id, "conclusion": "success"})
    check_ids = [check["check_id"] for check in checks]
    if len(check_ids) != len(set(check_ids)):
        raise ValueError("verification check identifiers are duplicated")

    return {
        "schema_version": EVIDENCE_MANIFEST_SCHEMA_VERSION,
        "repository": CANONICAL_REPOSITORY,
        "event_name": event_name,
        "ref": RELEASE_REF,
        "ref_protected": True,
        "branch_protection": _normalized_branch_protection(branch_protection),
        "checked_out_commit": sha.lower(),
        "workflow_ref": EXPECTED_WORKFLOW_REF,
        "run_id": run_id,
        "run_attempt": run_attempt,
        "checks": checks,
    }


def validate_evidence_manifest(
    manifest: Any,
    *,
    event_name: str,
    current_context: Mapping[str, Any],
    current_head: str,
) -> None:
    """Reject any manifest that is not bound to the current eligible run."""
    manifest_fields = {
        "schema_version",
        "repository",
        "event_name",
        "ref",
        "ref_protected",
        "branch_protection",
        "checked_out_commit",
        "workflow_ref",
        "run_id",
        "run_attempt",
        "checks",
    }
    if not isinstance(manifest, Mapping) or set(manifest) != manifest_fields:
        raise ValueError("verification manifest is missing or malformed")
    if event_name != "push" or manifest.get("event_name") != event_name:
        raise ValueError("verification manifest event does not match the current run")
    if not isinstance(current_context, Mapping):
        raise ValueError("current workflow context is missing")

    if (
        type(manifest.get("schema_version")) is not int
        or manifest.get("schema_version") != EVIDENCE_MANIFEST_SCHEMA_VERSION
    ):
        raise ValueError("verification manifest schema is unsupported")
    context_pairs = (
        ("repository", CANONICAL_REPOSITORY),
        ("ref", RELEASE_REF),
        ("ref_protected", True),
        ("workflow_ref", EXPECTED_WORKFLOW_REF),
    )
    for field, expected in context_pairs:
        if current_context.get(field) != expected or manifest.get(field) != expected:
            raise ValueError(f"verification manifest {field} does not match the current run")
    if (
        type(current_context.get("ref_protected")) is not bool
        or type(manifest.get("ref_protected")) is not bool
    ):
        raise ValueError("verification manifest protection flag is malformed")

    current_sha = current_context.get("sha")
    checked_out_commit = manifest.get("checked_out_commit")
    if (
        not isinstance(current_sha, str)
        or not isinstance(current_head, str)
        or not isinstance(checked_out_commit, str)
        or re.fullmatch(r"[0-9a-fA-F]{40,64}", current_sha) is None
        or re.fullmatch(r"[0-9a-fA-F]{40,64}", current_head) is None
        or checked_out_commit.lower() != current_sha.lower()
        or checked_out_commit.lower() != current_head.lower()
    ):
        raise ValueError("verification manifest commit does not match the current checkout")
    if manifest.get("run_id") != current_context.get("run_id"):
        raise ValueError("verification manifest run ID does not match the current run")
    if manifest.get("run_attempt") != current_context.get("run_attempt"):
        raise ValueError("verification manifest attempt does not match the current run")
    if not _positive_decimal(current_context.get("run_id")) or not _positive_decimal(
        current_context.get("run_attempt")
    ):
        raise ValueError("current workflow run identity is missing or malformed")

    checks = manifest.get("checks")
    if not isinstance(checks, list):
        raise ValueError("verification manifest checks are missing or malformed")
    expected_job_ids = [job_id for _, job_id in PR_REQUIRED_JOBS]
    observed_jobs: list[str] = []
    observed_check_ids: list[str] = []
    for check in checks:
        if not isinstance(check, Mapping) or set(check) != {"check_id", "job_id", "conclusion"}:
            raise ValueError("verification manifest check is malformed")
        check_id = check.get("check_id")
        job_id = check.get("job_id")
        if not isinstance(check_id, str) or not check_id.strip() or not isinstance(job_id, str):
            raise ValueError("verification manifest check identity is malformed")
        if check.get("conclusion") != "success":
            raise ValueError("a required verification check did not succeed")
        observed_check_ids.append(check_id)
        observed_jobs.append(job_id)
    if (
        len(observed_check_ids) != len(set(observed_check_ids))
        or len(observed_jobs) != len(set(observed_jobs))
        or set(observed_jobs) != set(expected_job_ids)
        or len(observed_jobs) != len(expected_job_ids)
    ):
        raise ValueError("verification manifest check IDs are missing, duplicated, or unexpected")

    normalized_policy = _normalized_branch_protection(manifest.get("branch_protection"))
    if normalized_policy != manifest.get("branch_protection"):
        raise ValueError("verification manifest branch policy is not normalized")


def _safe_text(value: Any) -> str:
    """Escape dynamic values before inserting them into Markdown text or tables."""
    text = " ".join(str(value).split())
    return html.escape(text, quote=False).replace("|", "&#124;").replace("`", "&#96;")


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
    elif (
        event_name == "push"
        and context.get("ref") == RELEASE_REF
        and context.get("ref_protected") is True
    ):
        rows, required_results = _render_required_rows(PR_REQUIRED_JOBS, needs)
        rows.extend(
            (
                "| Scheduled dependency policy | ⚪ Not applicable — schedule-triggered checks |",
                "| Branch coverage | ⚪ Not applicable — schedule-triggered checks |",
            )
        )
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

    lines.extend(
        (
            "",
            "Matrix details and failure logs remain in the individual jobs. Coverage measurements or an `unavailable` reason are shown in the Coverage gate summary.",
            "",
        )
    )
    return "\n".join(lines), 0 if overall_passed else 1


def _failure_summary(message: str) -> str:
    return "\n".join(("# ❌ CI summary could not be rendered", "", f"{_safe_text(message)}", ""))


def main() -> int:
    """Write the GitHub job summary and preserve the required check result."""
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        print("GITHUB_STEP_SUMMARY is not set.", file=sys.stderr)
        return 2

    try:
        needs = json.loads(os.environ.get("CI_NEEDS_JSON", "{}"), object_pairs_hook=_unique_json_object)
        if not isinstance(needs, dict):
            raise ValueError("CI_NEEDS_JSON must contain a JSON object.")
        event_name = os.environ.get("GITHUB_EVENT_NAME", "")
        context = {
            "repository": os.environ.get("GITHUB_REPOSITORY", ""),
            "ref": os.environ.get("GITHUB_REF", ""),
            "sha": os.environ.get("GITHUB_SHA", ""),
            "run_id": os.environ.get("GITHUB_RUN_ID", ""),
            "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", "1"),
            "ref_protected": os.environ.get("GITHUB_REF_PROTECTED", "").lower() == "true",
            "workflow_ref": os.environ.get("GITHUB_WORKFLOW_REF", ""),
            "server_url": os.environ.get("GITHUB_SERVER_URL", "https://github.com"),
        }
        if event_name == "push" and context["ref"] == RELEASE_REF:
            policy_json = os.environ.get("CI_BRANCH_PROTECTION_JSON", "")
            branch_protection = json.loads(policy_json, object_pairs_hook=_unique_json_object)
            manifest = build_evidence_manifest(
                event_name=event_name,
                context=context,
                needs=needs,
                branch_protection=branch_protection,
            )
            output_path = os.environ.get("GITHUB_OUTPUT")
            if not output_path:
                raise ValueError("manifest output is not configured")
            with Path(output_path).open("a", encoding="utf-8", newline="\n") as output_file:
                output_file.write(
                    "profile_evidence_manifest="
                    + json.dumps(manifest, separators=(",", ":"), sort_keys=True)
                    + "\n"
                )
        markdown, exit_code = build_summary(
            event_name=event_name,
            context=context,
            needs=needs,
        )
    except (json.JSONDecodeError, TypeError, ValueError):
        markdown = _failure_summary("Invalid workflow status input.")
        exit_code = 1

    Path(summary_path).write_text(markdown, encoding="utf-8", newline="\n")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
