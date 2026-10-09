"""Executable contracts for the concise GitHub Actions run summary."""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SUMMARY_PATH = REPOSITORY_ROOT / "scripts" / "ci_summary.py"
SUMMARY = None
SUMMARY_LOAD_ERROR = None
if SUMMARY_PATH.is_file():
    try:
        SPEC = importlib.util.spec_from_file_location("ci_summary", SUMMARY_PATH)
        if SPEC is None or SPEC.loader is None:
            raise ImportError("CI summary has no importable module loader")
        SUMMARY = importlib.util.module_from_spec(SPEC)
        SPEC.loader.exec_module(SUMMARY)
    except Exception as error:  # The tests must run and explain the missing implementation in RED.
        SUMMARY_LOAD_ERROR = error


class CiSummaryTests(unittest.TestCase):
    PR_JOBS = (
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
    )

    def require_summary(self):
        self.assertIsNotNone(
            SUMMARY,
            f"ci_summary.py must provide the tested summary contract; load error: {SUMMARY_LOAD_ERROR}",
        )
        return SUMMARY

    @staticmethod
    def pull_request_needs(plan=None, **overrides):
        plan = plan or {
            "schema_version": 1,
            "base_sha": "b" * 40,
            "merge_sha": "a" * 40,
            "classes": ["full"],
            "paths": [],
            "selected_jobs": list(CiSummaryTests.PR_JOBS),
            "coverage_scopes": ["rust", "ffi-c", "java", "python", "jni"],
            "full": True,
            "reason": "A full-CI trigger was detected.",
            "fallback": False,
        }
        results = {job: "success" for job in CiSummaryTests.PR_JOBS}
        results.update(
            {
                "impact-plan": "success",
                "scheduled-dependency-policy": "skipped",
                "branch-coverage": "skipped",
            }
        )
        results.update(overrides)
        needs = {job: {"result": result} for job, result in results.items()}
        needs["impact-plan"]["outputs"] = {"plan_json": json.dumps(plan)}
        return needs

    @staticmethod
    def documentation_plan():
        return {
            "schema_version": 1,
            "base_sha": "b" * 40,
            "merge_sha": "a" * 40,
            "classes": ["documentation"],
            "paths": [{"path": "docs/development/testing.md", "class": "documentation"}],
            "selected_jobs": ["docs-contracts"],
            "coverage_scopes": [],
            "full": False,
            "reason": "Only documentation changed.",
            "fallback": False,
        }

    @classmethod
    def documentation_needs(cls, **overrides):
        results = {job: "skipped" for job in cls.PR_JOBS}
        results["docs-contracts"] = "success"
        results.update(
            {
                "impact-plan": "success",
                "scheduled-dependency-policy": "skipped",
                "branch-coverage": "skipped",
            }
        )
        results.update(overrides)
        needs = {job: {"result": result} for job, result in results.items()}
        needs["impact-plan"]["outputs"] = {"plan_json": json.dumps(cls.documentation_plan())}
        return needs

    @staticmethod
    def context():
        return {
            "repository": "NeverWe1come/KMIPKit",
            "ref": "refs/pull/42/merge",
            "sha": "a" * 40,
            "run_id": "123456789",
            "run_attempt": "2",
        }

    def test_pull_request_summary_shows_required_groups_and_expected_schedule_skips(self) -> None:
        summary_module = self.require_summary()
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=self.pull_request_needs(),
        )

        self.assertEqual(0, exit_code)
        self.assertIn("CI result: PASS", markdown)
        self.assertIn("Core matrix", markdown)
        self.assertIn("Script contracts", markdown)
        self.assertIn("Normative inventory", markdown)
        self.assertIn("Coverage collection", markdown)
        self.assertIn("Coverage gate", markdown)
        self.assertIn("Dependency policy", markdown)
        self.assertIn("Not applicable", markdown)
        self.assertIn("schedule-triggered checks", markdown)
        self.assertIn("https://github.com/NeverWe1come/KMIPKit/actions/runs/123456789/attempts/2", markdown)
        self.assertIn("`aaaaaaaa`", markdown)

    def test_failed_required_pull_request_group_makes_summary_fail(self) -> None:
        summary_module = self.require_summary()
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=self.pull_request_needs(core="failure", coverage="cancelled"),
        )

        self.assertEqual(1, exit_code)
        self.assertIn("CI result: FAIL", markdown)
        self.assertIn("Core matrix", markdown)
        self.assertIn("FAIL", markdown)
        self.assertIn("CANCELLED", markdown)

    def test_every_pull_request_job_failure_is_reported_and_fails_the_summary(self) -> None:
        summary_module = self.require_summary()
        expected_labels = {
            "language-c": "C consumer",
            "language-java": "Java and JNI",
            "language-python": "Python bindings",
            "ffi-sanitizer-c": "C FFI sanitizer",
            "ffi-sanitizer-jni": "JNI sanitizer",
            "fuzz-smoke": "Fuzz smoke",
            "coverage-python": "Python coverage",
            "coverage-java": "Java coverage",
            "coverage-jni": "JNI coverage",
        }

        for job_id, label in expected_labels.items():
            with self.subTest(job=job_id):
                markdown, exit_code = summary_module.build_summary(
                    event_name="pull_request",
                    context=self.context(),
                    needs=self.pull_request_needs(**{job_id: "failure"}),
                )

                self.assertEqual(1, exit_code)
                self.assertIn(label, markdown)
                self.assertIn("CI result: FAIL", markdown)
                self.assertIn("Failures to fix", markdown)

    def test_required_skipped_and_missing_jobs_cannot_produce_a_passing_summary(self) -> None:
        summary_module = self.require_summary()
        skipped = self.pull_request_needs(**{"language-java": "skipped"})
        missing = self.pull_request_needs()
        del missing["fuzz-smoke"]

        for needs in (skipped, missing):
            with self.subTest(needs=needs):
                markdown, exit_code = summary_module.build_summary(
                    event_name="pull_request",
                    context=self.context(),
                    needs=needs,
                )

                self.assertEqual(1, exit_code)
                self.assertIn("CI result: FAIL", markdown)

    def test_summary_includes_the_job_level_failure_diagnosis(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(**{"ffi-sanitizer-c": "failure"})
        needs["ffi-sanitizer-c"]["outputs"] = {
            "diagnostic_details": json.dumps(
                [
                    {
                        "step_id": "run-asan-consumer",
                        "label": "Run ASAN consumer",
                        "explanation": "AddressSanitizer-instrumented C consumer failed.",
                    }
                ]
            )
        }

        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(1, exit_code)
        self.assertIn("Run ASAN consumer", markdown)
        self.assertIn("AddressSanitizer-instrumented C consumer failed", markdown)
        self.assertIn("C FFI sanitizer", markdown)

    def test_documentation_plan_accepts_only_justified_component_skips(self) -> None:
        summary_module = self.require_summary()
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=self.documentation_needs(),
        )

        self.assertEqual(0, exit_code)
        self.assertIn("CI result: PASS", markdown)
        self.assertIn("Impact: documentation", markdown)
        self.assertIn("Only documentation changed", markdown)
        self.assertIn("Not affected", markdown)
        self.assertIn("Rust", markdown)

    def test_selected_job_skip_fails_even_when_every_other_job_is_authorized_skipped(self) -> None:
        summary_module = self.require_summary()
        needs = self.documentation_needs(**{"docs-contracts": "skipped"})
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(1, exit_code)
        self.assertIn("docs-contracts", markdown)
        self.assertIn("unexpectedly skipped", markdown.lower())

    def test_missing_invalid_or_sha_mismatched_impact_plan_fails_summary(self) -> None:
        summary_module = self.require_summary()
        missing = self.documentation_needs()
        missing["impact-plan"]["outputs"] = {}
        malformed = self.documentation_needs()
        malformed["impact-plan"]["outputs"]["plan_json"] = "{not json"
        mismatched = self.documentation_needs()
        plan = self.documentation_plan() | {"merge_sha": "c" * 40}
        mismatched["impact-plan"]["outputs"]["plan_json"] = json.dumps(plan)

        for needs in (missing, malformed, mismatched):
            with self.subTest(needs=needs):
                markdown, exit_code = summary_module.build_summary(
                    event_name="pull_request",
                    context=self.context(),
                    needs=needs,
                )
                self.assertEqual(1, exit_code)
                self.assertIn("CI result: FAIL", markdown)
                self.assertIn("impact plan", markdown.lower())

    def test_classifier_failure_cannot_be_hidden_by_successful_full_jobs(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(**{"impact-plan": "failure"})
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(1, exit_code)
        self.assertIn("Impact classifier", markdown)
        self.assertIn("CI result: FAIL", markdown)

    def test_schedule_summary_requires_dependency_policy_but_keeps_branch_coverage_informational(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(
            core="skipped",
            **{
                "script-contracts": "skipped",
                "normative-inventory": "skipped",
                "coverage": "skipped",
                "coverage-gate": "skipped",
                "dependency-policy": "skipped",
                "scheduled-dependency-policy": "success",
                "branch-coverage": "failure",
            },
        )
        needs["scheduled-dependency-policy"]["outputs"] = {
            "release_ref": "release/1.0.0",
            "scanned_commit": "b" * 40,
        }
        needs["branch-coverage"]["outputs"] = {"summary_status": "failed"}
        markdown, exit_code = summary_module.build_summary(
            event_name="schedule",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(0, exit_code)
        self.assertIn("CI result: PASS", markdown)
        self.assertIn("Scheduled dependency policy", markdown)
        self.assertIn("Branch coverage", markdown)
        self.assertIn("informational", markdown.lower())
        self.assertIn("Attempt failed", markdown)
        self.assertIn("Release scan:", markdown)
        self.assertIn("release/1.0.0", markdown)
        self.assertIn("Not applicable", markdown)

    def test_schedule_without_branch_coverage_status_does_not_claim_it_passed(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(
            **{
                "core": "skipped",
                "script-contracts": "skipped",
                "normative-inventory": "skipped",
                "coverage": "skipped",
                "coverage-gate": "skipped",
                "dependency-policy": "skipped",
                "scheduled-dependency-policy": "success",
                "branch-coverage": "success",
            }
        )
        markdown, exit_code = summary_module.build_summary(
            event_name="schedule",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(0, exit_code)
        self.assertIn("Unavailable — informational", markdown)
        self.assertNotIn("Attempt completed", markdown)

    def test_untrusted_ref_is_escaped_before_markdown_rendering(self) -> None:
        summary_module = self.require_summary()
        context = self.context() | {"ref": "refs/pull/42/merge|oops\nInjected heading"}
        markdown, _ = summary_module.build_summary(
            event_name="pull_request",
            context=context,
            needs=self.pull_request_needs(),
        )

        self.assertNotIn("|oops", markdown)
        self.assertNotIn("\nInjected heading", markdown)

    def test_main_writes_summary_even_when_required_checks_fail(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(core="failure")
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "step-summary.md"
            github_output = Path(directory) / "github-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(output),
                "CI_NEEDS_JSON": json.dumps(needs),
                "GITHUB_EVENT_NAME": "pull_request",
                "GITHUB_REPOSITORY": "NeverWe1come/KMIPKit",
                "GITHUB_REF": "refs/pull/42/merge",
                "GITHUB_SHA": "a" * 40,
                "GITHUB_RUN_ID": "123456789",
                "GITHUB_RUN_ATTEMPT": "1",
                "GITHUB_SERVER_URL": "https://github.com",
                "GITHUB_OUTPUT": str(github_output),
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            markdown = output.read_text(encoding="utf-8")
            self.assertIn("CI result: FAIL", markdown)
            self.assertIn("Core matrix", markdown)
            self.assertIn("summary_written=true", github_output.read_text(encoding="utf-8"))

    def test_main_marks_the_fallback_needed_when_status_input_cannot_be_rendered(self) -> None:
        summary_module = self.require_summary()
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "step-summary.md"
            github_output = Path(directory) / "github-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(output),
                "GITHUB_OUTPUT": str(github_output),
                "CI_NEEDS_JSON": "not-json",
                "GITHUB_EVENT_NAME": "pull_request",
            }

            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            self.assertIn("could not be rendered", output.read_text(encoding="utf-8"))
            self.assertIn("summary_written=false", github_output.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
