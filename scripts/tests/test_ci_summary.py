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

    @classmethod
    def release_context(cls, **overrides):
        context = cls.context() | {
            "ref": "refs/heads/release/1.0.0",
            "ref_protected": True,
            "workflow_ref": (
                "NeverWe1come/KMIPKit/.github/workflows/ci.yml@refs/heads/release/1.0.0"
            ),
            "run_attempt": "1",
        }
        context.update(overrides)
        return context

    @staticmethod
    def branch_protection(**overrides):
        policy = {
            "protected": True,
            "required_pull_request_reviews": {
                "required_approving_review_count": 1,
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
        policy.update(overrides)
        return policy

    def successful_release_needs(self):
        module = self.require_summary()
        return {
            job_id: {"result": "success"}
            for _, job_id in module.PR_REQUIRED_JOBS
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
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            markdown = output.read_text(encoding="utf-8")
            self.assertIn("CI result: FAIL", markdown)
            self.assertIn("Core matrix", markdown)

    def test_main_rejects_duplicate_job_ids_in_needs_json(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs()
        encoded_needs = json.dumps(needs)
        encoded_needs = encoded_needs.replace(
            '"core": {"result": "success"}',
            '"core": {"result": "failure"}, "core": {"result": "success"}',
            1,
        )
        self.assertGreater(encoded_needs.count('"core"'), 1)
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "step-summary.md"
            environment = {
                "GITHUB_STEP_SUMMARY": str(output),
                "CI_NEEDS_JSON": encoded_needs,
                "GITHUB_EVENT_NAME": "pull_request",
                "GITHUB_REPOSITORY": "NeverWe1come/KMIPKit",
                "GITHUB_REF": "refs/pull/42/merge",
                "GITHUB_SHA": "a" * 40,
                "GITHUB_RUN_ID": "123456789",
                "GITHUB_RUN_ATTEMPT": "1",
                "GITHUB_SERVER_URL": "https://github.com",
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            markdown = output.read_text(encoding="utf-8")
            self.assertIn("CI summary could not be rendered", markdown)
            self.assertNotIn("CI result: PASS", markdown)

    def test_release_evidence_manifest_binds_exact_current_run_checks(self) -> None:
        summary_module = self.require_summary()
        manifest = summary_module.build_evidence_manifest(
            event_name="push",
            context=self.release_context(),
            needs=self.successful_release_needs(),
            branch_protection=self.branch_protection(),
        )

        expected_job_ids = [job_id for _, job_id in summary_module.PR_REQUIRED_JOBS]
        self.assertEqual(1, manifest["schema_version"])
        self.assertEqual("NeverWe1come/KMIPKit", manifest["repository"])
        self.assertEqual("push", manifest["event_name"])
        self.assertEqual("refs/heads/release/1.0.0", manifest["ref"])
        self.assertIs(manifest["ref_protected"], True)
        self.assertEqual("a" * 40, manifest["checked_out_commit"])
        self.assertEqual(self.release_context()["workflow_ref"], manifest["workflow_ref"])
        self.assertEqual("123456789", manifest["run_id"])
        self.assertEqual("1", manifest["run_attempt"])
        self.assertEqual(
            [
                {"check_id": job_id, "job_id": job_id, "conclusion": "success"}
                for job_id in expected_job_ids
            ],
            manifest["checks"],
        )
        self.assertEqual(self.branch_protection(), manifest["branch_protection"])

    def test_release_evidence_manifest_rejects_ineligible_contexts(self) -> None:
        summary_module = self.require_summary()
        invalid_contexts = (
            ("pull_request", self.release_context()),
            ("schedule", self.release_context()),
            ("workflow_dispatch", self.release_context()),
            ("push", self.release_context(ref="refs/heads/main")),
            ("push", self.release_context(ref_protected=False)),
            ("push", self.release_context(repository="someone-else/KMIPKit")),
            ("push", self.release_context(sha="not-a-commit")),
            ("push", self.release_context(workflow_ref=".github/workflows/untrusted.yml@refs/heads/release/1.0.0")),
            ("push", self.release_context(run_id="not-numeric")),
            ("push", self.release_context(run_attempt="0")),
            ("push", self.release_context(repository="")),
            ("push", self.release_context(ref_protected="true")),
            ("push", self.release_context(workflow_ref="")),
            ("push", self.release_context(run_id="")),
        )
        for event_name, context in invalid_contexts:
            with self.subTest(event_name=event_name, context=context):
                with self.assertRaises(ValueError):
                    summary_module.build_evidence_manifest(
                        event_name=event_name,
                        context=context,
                        needs=self.successful_release_needs(),
                        branch_protection=self.branch_protection(),
                    )

    def test_release_evidence_manifest_requires_every_exact_job_to_succeed(self) -> None:
        summary_module = self.require_summary()
        required_jobs = [job_id for _, job_id in summary_module.PR_REQUIRED_JOBS]
        for job_id in required_jobs:
            for result in (None, "failure", "cancelled", "skipped", "unknown"):
                needs = self.successful_release_needs()
                if result is None:
                    needs.pop(job_id)
                else:
                    needs[job_id] = {"result": result}
                with self.subTest(job_id=job_id, result=result):
                    with self.assertRaises(ValueError):
                        summary_module.build_evidence_manifest(
                            event_name="push",
                            context=self.release_context(),
                            needs=needs,
                            branch_protection=self.branch_protection(),
                        )

    def test_release_evidence_manifest_rejects_insufficient_branch_protection(self) -> None:
        summary_module = self.require_summary()
        baseline = self.branch_protection()
        invalid_policies = []
        for field, value in (
            ("protected", False),
            ("required_pull_request_reviews", None),
            ("enforce_admins", {"enabled": False}),
            ("allow_force_pushes", {"enabled": True}),
            ("allow_deletions", {"enabled": True}),
        ):
            invalid_policies.append(baseline | {field: value})

        invalid_review_policies = (
            {"required_approving_review_count": 0},
            {"dismiss_stale_reviews": False},
            {"bypass_pull_request_allowances": None},
            {"bypass_pull_request_allowances": {"teams": [], "apps": []}},
            {"bypass_pull_request_allowances": {"users": [], "teams": []}},
            {"bypass_pull_request_allowances": {"users": [], "apps": []}},
            {"bypass_pull_request_allowances": {"users": [1], "teams": [], "apps": []}},
            {"bypass_pull_request_allowances": {"users": [], "teams": [2], "apps": []}},
            {"bypass_pull_request_allowances": {"users": [], "teams": [], "apps": [3]}},
            {"dismiss_stale_reviews": 1},
        )
        for review_override in invalid_review_policies:
            invalid_policies.append(
                baseline
                | {
                    "required_pull_request_reviews": (
                        baseline["required_pull_request_reviews"] | review_override
                    )
                }
            )

        for branch_policy in invalid_policies:
            with self.subTest(branch_policy=branch_policy):
                with self.assertRaises(ValueError):
                    summary_module.build_evidence_manifest(
                        event_name="push",
                        context=self.release_context(),
                        needs=self.successful_release_needs(),
                        branch_protection=branch_policy,
                    )


if __name__ == "__main__":
    unittest.main(verbosity=2)
