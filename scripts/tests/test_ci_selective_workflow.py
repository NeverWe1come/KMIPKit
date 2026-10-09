"""Workflow-level contracts for selective CI routing."""

from __future__ import annotations

import re
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ci.yml"


class SelectiveWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contents = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else ""

    def job(self, job_id: str) -> str:
        match = re.search(
            rf"(?ms)^  {re.escape(job_id)}:\n(.*?)(?=^  [a-z][\w-]*:|\Z)",
            self.contents,
        )
        self.assertIsNotNone(match, f"Workflow must define job `{job_id}`.")
        return match.group(1) if match else ""

    def test_pull_request_workflow_has_no_workflow_level_path_filter(self) -> None:
        self.assertRegex(self.contents, r"(?ms)^on:\n\s+pull_request:")
        pull_request = re.search(r"(?ms)^  pull_request:\n(.*?)(?=^  [a-z][\w-]*:|^permissions:)", self.contents)
        self.assertIsNotNone(pull_request)
        if pull_request:
            self.assertNotRegex(pull_request.group(1), r"(?m)^\s+(?:paths|paths-ignore):")

    def test_classifier_is_a_current_sha_required_job_and_publishes_the_plan(self) -> None:
        classifier = self.job("impact-plan")
        self.assertRegex(classifier, r"(?m)^    if: github\.event_name == 'pull_request'$|^    if: github\.event_name == 'pull_request'$")
        self.assertIn("scripts/ci_impact.py classify", classifier)
        self.assertIn("github.event.pull_request.base.sha", classifier)
        self.assertIn("github.sha", classifier)
        self.assertRegex(classifier, r"(?m)^    outputs:")
        self.assertIn("plan_json", classifier)

    def test_independent_component_jobs_keep_three_platform_matrices(self) -> None:
        for job_id in ("language-c", "language-java", "language-python"):
            with self.subTest(job=job_id):
                job = self.job(job_id)
                self.assertIn("needs:", job)
                self.assertIn("impact-plan", job)
                self.assertRegex(job, r"(?m)^        os: \[ubuntu-latest, windows-latest, macos-latest\]")
        self.assertNotIn("  language-bindings:\n", self.contents)

    def test_sanitizer_and_adapter_coverage_producers_are_independently_selectable(self) -> None:
        for job_id in (
            "ffi-sanitizer-c",
            "ffi-sanitizer-jni",
            "coverage-java",
            "coverage-python",
            "coverage-jni",
        ):
            with self.subTest(job=job_id):
                job = self.job(job_id)
                self.assertIn("impact-plan", job)
        self.assertNotIn("  adapter-coverage:\n", self.contents)

    def test_coverage_gate_consumes_only_plan_selected_scopes(self) -> None:
        gate = self.job("coverage-gate")
        self.assertIn("needs.impact-plan.outputs.coverage_scopes", gate)
        self.assertIn("--scopes-json", gate)
        self.assertIn("coverage-java", gate)
        self.assertIn("coverage-python", gate)
        self.assertIn("coverage-jni", gate)

    def test_summary_waits_for_classifier_and_every_possible_pr_job(self) -> None:
        summary = self.job("run-summary")
        for job_id in (
            "impact-plan",
            "docs-contracts",
            "core",
            "script-contracts",
            "language-c",
            "language-java",
            "language-python",
            "ffi-sanitizer-c",
            "ffi-sanitizer-jni",
            "fuzz-smoke",
            "normative-inventory",
            "coverage",
            "coverage-java",
            "coverage-python",
            "coverage-jni",
            "coverage-gate",
            "dependency-policy",
            "scheduled-dependency-policy",
            "branch-coverage",
        ):
            with self.subTest(job=job_id):
                self.assertRegex(summary, rf"(?m)^      - {re.escape(job_id)}$")
        self.assertIn("CI_NEEDS_JSON", summary)

    def test_schedule_jobs_do_not_depend_on_the_pr_impact_plan(self) -> None:
        for job_id in ("scheduled-dependency-policy", "branch-coverage"):
            with self.subTest(job=job_id):
                job = self.job(job_id)
                self.assertRegex(job, r"(?m)^    if: github\.event_name == 'schedule'$|^    if: github\.event_name == 'schedule'$")
                self.assertNotIn("needs:", job)

    def test_docs_only_validator_is_lightweight_and_checks_traceability(self) -> None:
        docs = self.job("docs-contracts")
        self.assertIn("test_requirement_traceability.py", docs)
        self.assertIn("test_feature_traceability.py", docs)
        self.assertNotIn("cargo", docs)
        self.assertNotIn("mvn", docs)
        self.assertNotIn("pytest", docs)


if __name__ == "__main__":
    unittest.main(verbosity=2)
