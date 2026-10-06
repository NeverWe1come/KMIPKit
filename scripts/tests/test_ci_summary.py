"""Executable contracts for the concise GitHub Actions run summary."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


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
    def require_summary(self):
        self.assertIsNotNone(
            SUMMARY,
            f"ci_summary.py must provide the tested summary contract; load error: {SUMMARY_LOAD_ERROR}",
        )
        return SUMMARY

    @staticmethod
    def pull_request_needs(**overrides):
        results = {
            "core": "success",
            "script-contracts": "success",
            "normative-inventory": "success",
            "coverage": "success",
            "coverage-gate": "success",
            "dependency-policy": "success",
            "scheduled-dependency-policy": "skipped",
            "branch-coverage": "skipped",
        }
        results.update(overrides)
        return {job: {"result": result} for job, result in results.items()}

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
        self.assertIn("Not applicable", markdown)

    def test_untrusted_ref_is_escaped_before_markdown_rendering(self) -> None:
        summary_module = self.require_summary()
        context = self.context() | {"ref": "refs/pull/42/merge|oops\nInjected heading"}
        markdown, _ = summary_module.build_summary(
            event_name="pull_request",
            context=context,
            needs=self.pull_request_needs(),
        )

        self.assertNotIn("|oops", markdown)
        self.assertNotIn("Injected heading", markdown)


if __name__ == "__main__":
    unittest.main(verbosity=2)
