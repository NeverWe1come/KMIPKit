#!/usr/bin/env python3
"""Write a safe, actionable job summary from GitHub Actions step outcomes."""

from __future__ import annotations

import html
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Mapping


JOB_LABELS = {
    "core": "Core checks",
    "script-contracts": "Script contracts",
    "language-bindings": "Language bindings",
    "ffi-sanitizer": "FFI sanitizer",
    "fuzz-smoke": "Fuzz smoke",
    "normative-inventory": "Normative inventory",
    "coverage": "Coverage collection",
    "coverage-gate": "Coverage gate",
    "adapter-coverage": "Adapter coverage",
    "dependency-policy": "Dependency policy",
    "scheduled-dependency-policy": "Scheduled dependency policy",
    "branch-coverage": "Informational branch coverage",
    "run-summary": "CI run summary",
}
WORD_LABELS = {
    "api": "API",
    "asan": "ASAN",
    "c": "C",
    "ci": "CI",
    "ffi": "FFI",
    "jni": "JNI",
    "llvm": "LLVM",
    "ttlv": "TTLV",
    "ubsan": "UBSAN",
    "uv": "uv",
    "wsl": "WSL",
    "cmake": "CMake",
    "maven": "Maven",
    "msvc": "MSVC",
    "oasis": "OASIS",
    "rustsec": "RustSec",
}
STEP_LABELS = {
    "source": "Check production source eligibility",
    "enforce": "Enforce coverage thresholds",
    "download": "Download platform and adapter reports",
    "active-release": "Report active release ref",
    "scanned-commit": "Report scanned commit",
    "branch-report": "Attempt informational branch coverage",
    "fallback-summary": "Fallback summary when checkout or rendering fails",
    "write-summary": "Write run summary",
}
EXPLANATIONS = {
    "check-formatting": "Rust formatting differs from rustfmt output; run `cargo fmt --all` and commit the result.",
    "run-clippy": "Clippy or a target build failed with warnings denied; inspect its first compiler diagnostic in this step's log.",
    "run-workspace-tests": "Rust compilation or one or more workspace tests failed; inspect the first compiler error or failing test in this step's log.",
    "build-documentation": "Rust documentation failed to build; inspect the rustdoc diagnostic in this step's log.",
    "run-python-contracts": "A Python contract, catalog, fixture, or generated-file check failed; inspect the named failing test or generator message in this step's log.",
    "run-powershell-contracts": "A PowerShell contract failed; inspect the first failed assertion in this step's log.",
    "run-asan-consumer": "The AddressSanitizer-instrumented C consumer failed; inspect its sanitizer or assertion diagnostic in this step's log.",
    "run-the-c-consumer-against-the-instrumented-rust-ffi": "The C consumer failed against the AddressSanitizer-instrumented Rust FFI; inspect the sanitizer report or consumer assertion in this step's log.",
    "run-native-sanitizers": "The native sanitizer regression failed; inspect the AddressSanitizer or UndefinedBehaviorSanitizer diagnostic in this step's log.",
    "run-fuzz-target": "The bounded extension-schema fuzz target failed to build, crashed, or timed out; inspect libFuzzer's terminal output in this step's log.",
    "fuzz-the-bounded-extension-schema-target": "The bounded extension-schema fuzz target failed to build, crashed, or timed out; inspect libFuzzer's terminal output in this step's log.",
    "run-c-consumer": "The C ABI consumer reported a failure; inspect its failing operation and status in this step's log.",
    "run-the-c-consumer-and-example": "The C ABI consumer or example failed; inspect the failing operation and native exit status in this step's log.",
    "run-java-tests": "The Java/JNI tests or Maven verification failed; inspect the named Surefire test or compiler error in this step's log.",
    "run-python-tests": "The Python adapter tests or example failed; inspect the pytest failure or traceback in this step's log.",
    "run-java-coverage": "The Java tests or JaCoCo report collection failed; inspect the first Maven/Surefire error in this step's log.",
    "run-python-coverage": "The Python coverage tests or report generation failed; inspect the pytest failure or coverage error in this step's log.",
    "run-jni-coverage": "The JNI coverage suite or LLVM report generation failed; inspect the native test/LLVM diagnostic in this step's log.",
    "enforce-coverage-thresholds": "Coverage data is missing, malformed, or below a documented threshold; the Coverage gate summary contains the measured values and reason.",
    "run-dependency-policy": "The dependency, license, advisory, source, or lockfile policy check failed; inspect the specific cargo-deny finding in this step's log.",
    "run-scheduled-dependency-policy": "The scheduled dependency-policy scan failed; inspect the exact cargo-deny or advisory-refresh diagnostic in this step's log.",
    "run-root-and-fuzz-dependency-policy": "The root or fuzz dependency policy failed; inspect the exact cargo-deny advisory, license, source, or lockfile finding in this step's log.",
    "check-pinned-oasis-tree-against-exact-pr-base": "The pinned OASIS source tree differs from the exact PR base. Restore immutable upstream files; inspect the reported path in this step's log.",
    "compare-source-candidates-with-reviewed-clause-ledger": "Changed OASIS source candidates do not match the reviewed clause ledger. Review the candidate paths and update the approved inventory evidence.",
    "validate-complete-normative-catalog": "The normative catalog failed validation. Inspect the first reported source hash, clause, identifier, or completeness error in this step's log.",
    "verify-generated-ttlv-tag-allocations": "Generated TTLV tag allocations differ from the catalog inputs or are invalid. Inspect the reported tag and regenerate the committed output with the pinned generator.",
    "verify-generated-result-mappings": "Generated result mappings are stale or invalid. Inspect the reported value and regenerate the committed output with the pinned generator.",
    "add-versioned-llvm-tool-names": "The versioned LLVM tool links or PATH setup failed; inspect the missing binary or link target in this step's log.",
    "source": "The production source preflight failed while inspecting Rust files or writing its eligibility status; inspect the first file or parser error in this step's log.",
    "enforce": "The coverage gate rejected a missing, malformed, or below-threshold report; the Coverage gate Summary contains the measured values and exact reason.",
    "summarize-failed-platform-or-adapter-collection": "The upstream coverage collection failed; this step could not write its failure details, so inspect the platform and adapter job results listed immediately before it.",
    "summarize": "The workflow could not write the coverage status summary; inspect the source-preflight, tool-install, or report-generation step outcomes above.",
    "active-release": "The workflow could not report the configured active release ref; inspect the output file or shell error in this step's log.",
    "scanned-commit": "The workflow could not resolve or report the checked-out commit; inspect the checkout or Git error in this step's log.",
    "fallback-summary": "The fallback Summary could not be written after checkout or summary rendering failed; inspect the preceding step's error.",
    "write-summary": "The run-summary renderer failed or reported a required CI failure; inspect the Summary text and this step's log.",
    "attempt-informational-branch-coverage": "The optional nightly branch-coverage report failed; inspect the tool diagnostic in this step's log. This result remains informational.",
    "write-run-summary": "The run summary renderer failed or reported a required CI failure; inspect the summary output and this step's log.",
}


def _safe_text(value: Any) -> str:
    """Escape dynamic workflow values before including them in Markdown."""
    text = " ".join(str(value).split())
    return (
        html.escape(text, quote=False)
        .replace("|", "&#124;")
        .replace("`", "&#96;")
        .replace("[", "&#91;")
        .replace("]", "&#93;")
    )


def _display_step_id(step_id: str) -> str:
    known = STEP_LABELS.get(step_id)
    if known is not None:
        return known
    words = re.sub(r"[^A-Za-z0-9]+", " ", step_id).split()
    return " ".join(WORD_LABELS.get(word.lower(), word.capitalize()) for word in words)


def _failure_explanation(step_id: str) -> str:
    known = EXPLANATIONS.get(step_id)
    if known is not None:
        return known
    normalized = step_id.lower()
    if "install" in normalized or "setup" in normalized or "set-up" in normalized or "toolchain" in normalized:
        return "A required tool or dependency could not be installed or configured; inspect the version/download error in this step's log."
    if "test" in normalized or "contract" in normalized:
        return "A test or contract failed; inspect the failing test name and assertion in this step's log."
    if "coverage" in normalized or "report" in normalized:
        return "Coverage collection or validation failed; inspect the report/tool error in this step's log and Coverage gate summary."
    if "build" in normalized or "compile" in normalized or "configure" in normalized:
        return "A build or configure command failed; inspect the first compiler or configuration diagnostic in this step's log."
    if "upload" in normalized or "download" in normalized:
        return "A required workflow artifact could not be transferred; inspect the artifact path and action error in this step's log."
    if "checkout" in normalized:
        return "GitHub could not check out the commit; inspect the repository/ref/network error in this step's log."
    return "This command or action did not complete successfully; open the failed step's log for its native error details."


def _matrix_description(matrix: Mapping[str, Any]) -> str:
    parts = [f"{_safe_text(key)}: `{_safe_text(value)}`" for key, value in sorted(matrix.items())]
    return " · ".join(parts)


def _write_console(markdown: str) -> None:
    stream = sys.stdout
    if hasattr(stream, "reconfigure"):
        stream.reconfigure(encoding="utf-8", errors="backslashreplace")
    stream.write(markdown)


def build_diagnostics(
    *,
    job_id: str,
    matrix: Mapping[str, Any],
    steps: Mapping[str, Any],
) -> tuple[str, list[dict[str, str]]]:
    """Return a Markdown job diagnosis and records safe to pass to the run summary."""
    label = JOB_LABELS.get(job_id, _display_step_id(job_id))
    context = _matrix_description(matrix)
    records: list[dict[str, str]] = []
    for step_id, value in steps.items():
        if step_id == "ci-diagnostics" or not isinstance(value, Mapping):
            continue
        outcome = value.get("outcome")
        conclusion = value.get("conclusion")
        if outcome != "failure" and conclusion != "failure":
            continue
        records.append(
            {
                "step_id": str(step_id),
                "label": _display_step_id(str(step_id)),
                "explanation": _failure_explanation(str(step_id)),
            }
        )

    if records:
        lines = [f"### ❌ Failure diagnosis — {_safe_text(label)}", ""]
        lines.append(f"**Job:** `{_safe_text(job_id)}`")
        if context:
            lines.append(f"**Matrix:** {context}")
        lines.extend(("", "**Failed checks:**"))
        for record in records:
            lines.append(
                f"- **{_safe_text(record['label'])}**: {_safe_text(record['explanation'])}"
            )
        lines.extend(
            (
                "",
                "Open the failed step's log above for the full compiler, test, sanitizer, action, or policy output.",
                "",
            )
        )
    else:
        lines = [f"### ✅ Job checks passed — {_safe_text(label)}", ""]
        if context:
            lines.append(f"**Matrix:** {context}")
        lines.extend(("All recorded steps completed without a failure.", ""))

    return "\n".join(lines), records


def main() -> int:
    """Write the job summary and a compact output for the workflow run summary."""
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not summary_path or not output_path:
        print("GITHUB_STEP_SUMMARY and GITHUB_OUTPUT must be set.", file=sys.stderr)
        return 2

    try:
        steps = json.loads(os.environ.get("CI_STEPS_JSON", "{}"))
        matrix = json.loads(os.environ.get("CI_MATRIX_JSON", "{}"))
        if not isinstance(steps, dict) or (matrix is not None and not isinstance(matrix, dict)):
            raise ValueError("CI_STEPS_JSON and CI_MATRIX_JSON must contain JSON objects.")
        if matrix is None:
            matrix = {}
        markdown, records = build_diagnostics(
            job_id=os.environ.get("CI_JOB_ID", "unknown"),
            matrix=matrix,
            steps=steps,
        )
        Path(summary_path).write_text(markdown, encoding="utf-8", newline="\n")
        _write_console(markdown)
        encoded_records = json.dumps(records, ensure_ascii=True, separators=(",", ":"))
        with Path(output_path).open("a", encoding="utf-8", newline="\n") as output:
            output.write(f"diagnostic_details={encoded_records}\n")
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"CI job diagnostics could not be written: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
