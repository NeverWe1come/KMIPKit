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
        "fast-preflight",
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
            "base_sha": "b" * 40,
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
        self.assertIn("Rust and FFI coverage", markdown)
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

    def test_fast_preflight_failure_is_visible_and_blocks_script_contract_matrix(self) -> None:
        summary_module = self.require_summary()
        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=self.pull_request_needs(**{"fast-preflight": "failure", "script-contracts": "skipped"}),
        )

        self.assertEqual(1, exit_code)
        self.assertIn("Fast preflight", markdown)
        self.assertIn("Script contracts", markdown)
        self.assertIn("CI result: FAIL", markdown)

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

    def test_java_coverage_matrix_failure_does_not_use_an_arbitrary_leg_diagnosis(self) -> None:
        summary_module = self.require_summary()
        needs = self.pull_request_needs(**{"coverage-java": "failure"})
        needs["coverage-java"]["outputs"] = {
            "diagnostic_details": json.dumps(
                [{"label": "Windows setup", "explanation": "This leg alone reports a test failure."}]
            )
        }

        markdown, exit_code = summary_module.build_summary(
            event_name="pull_request",
            context=self.context(),
            needs=needs,
        )

        self.assertEqual(1, exit_code)
        self.assertIn("One or more matrix legs failed or were omitted", markdown)
        self.assertNotIn("This leg alone reports a test failure", markdown)

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
        self.assertIn("**Impact:** Selective CI", markdown)
        self.assertIn("Components:** documentation", markdown)
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

    def test_protected_release_push_summary_renders_required_jobs(self) -> None:
        summary_module = self.require_summary()

        markdown, exit_code = summary_module.build_summary(
            event_name="push",
            context=self.release_context(),
            needs=self.successful_release_needs(),
        )

        self.assertEqual(0, exit_code)
        self.assertIn("CI result: PASS", markdown)
        self.assertIn("Selected checks passed: 6/6", markdown)
        self.assertIn("Core matrix (3 OS × Rust 1.94 and stable)", markdown)
        self.assertIn("Dependency policy", markdown)
        self.assertNotIn("Documentation contracts (`docs-contracts`)", markdown)
        self.assertNotIn("C consumer (3 OS) (`language-c`)", markdown)

    def test_protected_release_push_reports_failed_required_job(self) -> None:
        summary_module = self.require_summary()
        needs = self.successful_release_needs()
        needs["core"] = {"result": "failure"}

        markdown, exit_code = summary_module.build_summary(
            event_name="push",
            context=self.release_context(),
            needs=needs,
        )

        self.assertEqual(1, exit_code)
        self.assertIn("CI result: FAIL", markdown)
        self.assertIn("Core matrix", markdown)
        self.assertIn("❌ FAIL", markdown)

    def test_release_manifest_is_not_written_before_summary_rendering_succeeds(self) -> None:
        summary_module = self.require_summary()
        context = self.release_context()
        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "step-summary.md"
            output_path = Path(directory) / "step-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_OUTPUT": str(output_path),
                "CI_NEEDS_JSON": json.dumps(self.successful_release_needs()),
                "CI_BRANCH_PROTECTION_JSON": json.dumps(self.branch_protection()),
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REPOSITORY": context["repository"],
                "GITHUB_REF": context["ref"],
                "GITHUB_REF_PROTECTED": "true",
                "GITHUB_WORKFLOW_REF": context["workflow_ref"],
                "GITHUB_SHA": context["sha"],
                "GITHUB_RUN_ID": context["run_id"],
                "GITHUB_RUN_ATTEMPT": context["run_attempt"],
                "GITHUB_SERVER_URL": "https://github.com",
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                with mock.patch.object(
                    summary_module,
                    "build_summary",
                    side_effect=RuntimeError("summary rendering failed"),
                ):
                    with self.assertRaisesRegex(RuntimeError, "summary rendering failed"):
                        summary_module.main()

            self.assertFalse(output_path.exists())

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
            github_output = Path(directory) / "github-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(output),
                "GITHUB_OUTPUT": str(github_output),
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
            self.assertEqual("summary_written=false", github_output.read_text(encoding="utf-8").strip())

    def test_release_evidence_manifest_binds_exact_current_run_checks(self) -> None:
        summary_module = self.require_summary()
        manifest = summary_module.build_evidence_manifest(
            event_name="push",
            context=self.release_context(),
            needs=self.successful_release_needs(),
            branch_protection=self.branch_protection(),
        )

        expected_job_ids = list(summary_module.EVIDENCE_REQUIRED_JOB_IDS)
        self.assertEqual(
            (
                "core",
                "script-contracts",
                "normative-inventory",
                "coverage",
                "coverage-gate",
                "dependency-policy",
            ),
            tuple(expected_job_ids),
        )
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
        self.assertIsNone(
            summary_module.validate_evidence_manifest(
                manifest,
                event_name="push",
                current_context=self.release_context(),
                current_head="a" * 40,
            )
        )

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
        required_jobs = list(summary_module.EVIDENCE_REQUIRED_JOB_IDS)
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

    def test_release_evidence_manifest_validation_rejects_stale_and_duplicate_evidence(self) -> None:
        summary_module = self.require_summary()
        manifest = summary_module.build_evidence_manifest(
            event_name="push",
            context=self.release_context(),
            needs=self.successful_release_needs(),
            branch_protection=self.branch_protection(),
        )
        invalid_manifests = []
        for field, value in (
            ("repository", "someone-else/KMIPKit"),
            ("event_name", "pull_request"),
            ("ref", "refs/heads/main"),
            ("ref_protected", False),
            ("workflow_ref", "untrusted-workflow"),
            ("run_id", "987654321"),
            ("run_attempt", "2"),
        ):
            invalid_manifests.append(manifest | {field: value})
        invalid_manifests.append(manifest | {"checked_out_commit": "b" * 40})
        invalid_manifests.append(manifest | {"schema_version": True})
        invalid_manifests.append(manifest | {"ref_protected": 1})
        invalid_manifests.append(manifest | {"checks": manifest["checks"][:-1]})
        invalid_manifests.append(
            manifest
            | {
                "checks": [
                    *manifest["checks"][:-1],
                    manifest["checks"][0],
                ]
            }
        )
        invalid_manifests.append(
            manifest
            | {
                "checks": [
                    *manifest["checks"][:-1],
                    manifest["checks"][-1] | {"conclusion": "failure"},
                ]
            }
        )
        invalid_manifests.append(
            manifest
            | {
                "checks": [
                    manifest["checks"][0] | {"check_id": "unrelated-check"},
                    *manifest["checks"][1:],
                ]
            }
        )
        invalid_manifests.append(manifest | {"unknown": "field"})

        for invalid_manifest in invalid_manifests:
            with self.subTest(manifest=invalid_manifest):
                with self.assertRaises(ValueError):
                    summary_module.validate_evidence_manifest(
                        invalid_manifest,
                        event_name="push",
                        current_context=self.release_context(),
                        current_head="a" * 40,
                    )

        for current_context, current_head in (
            (self.release_context(run_id="987654321"), "a" * 40),
            (self.release_context(run_attempt="2"), "a" * 40),
            (self.release_context(sha="b" * 40), "a" * 40),
            (self.release_context(ref_protected=1), "a" * 40),
            (self.release_context(), "b" * 40),
        ):
            with self.subTest(current_context=current_context, current_head=current_head):
                with self.assertRaises(ValueError):
                    summary_module.validate_evidence_manifest(
                        manifest,
                        event_name="push",
                        current_context=current_context,
                        current_head=current_head,
                    )

    def test_main_writes_release_manifest_and_summary_status(self) -> None:
        summary_module = self.require_summary()
        context = self.release_context()
        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "step-summary.md"
            output_path = Path(directory) / "step-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_OUTPUT": str(output_path),
                "CI_NEEDS_JSON": json.dumps(self.successful_release_needs()),
                "CI_BRANCH_PROTECTION_JSON": json.dumps(self.branch_protection()),
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REPOSITORY": context["repository"],
                "GITHUB_REF": context["ref"],
                "GITHUB_REF_PROTECTED": "true",
                "GITHUB_WORKFLOW_REF": context["workflow_ref"],
                "GITHUB_SHA": context["sha"],
                "GITHUB_RUN_ID": context["run_id"],
                "GITHUB_RUN_ATTEMPT": context["run_attempt"],
                "GITHUB_SERVER_URL": "https://github.com",
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(0, exit_code)
            output = output_path.read_text(encoding="utf-8")
            lines = output.splitlines()
            self.assertEqual(2, len(lines))
            key, encoded_manifest = lines[0].split("=", 1)
            self.assertEqual("profile_evidence_manifest", key)
            self.assertEqual("summary_written=true", lines[1])
            manifest = json.loads(encoded_manifest)
            self.assertEqual(len(summary_module.EVIDENCE_REQUIRED_JOB_IDS), len(manifest["checks"]))
            self.assertNotIn("CI_BRANCH_PROTECTION_JSON", output)
            self.assertNotIn("private-key", output.lower())

    def test_main_fails_closed_when_release_policy_evidence_is_missing(self) -> None:
        summary_module = self.require_summary()
        context = self.release_context()
        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "step-summary.md"
            output_path = Path(directory) / "step-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_OUTPUT": str(output_path),
                "CI_NEEDS_JSON": json.dumps(self.successful_release_needs()),
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REPOSITORY": context["repository"],
                "GITHUB_REF": context["ref"],
                "GITHUB_REF_PROTECTED": "true",
                "GITHUB_WORKFLOW_REF": context["workflow_ref"],
                "GITHUB_SHA": context["sha"],
                "GITHUB_RUN_ID": context["run_id"],
                "GITHUB_RUN_ATTEMPT": context["run_attempt"],
                "GITHUB_SERVER_URL": "https://github.com",
            }
            with mock.patch.dict(os.environ, environment, clear=False):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            self.assertEqual("summary_written=false", output_path.read_text(encoding="utf-8").strip())
            self.assertIn(
                "CI summary could not be rendered",
                summary_path.read_text(encoding="utf-8"),
            )

    def test_main_does_not_invent_missing_release_run_attempt(self) -> None:
        summary_module = self.require_summary()
        context = self.release_context()
        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "step-summary.md"
            output_path = Path(directory) / "step-output.txt"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_OUTPUT": str(output_path),
                "CI_NEEDS_JSON": json.dumps(self.successful_release_needs()),
                "CI_BRANCH_PROTECTION_JSON": json.dumps(self.branch_protection()),
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REPOSITORY": context["repository"],
                "GITHUB_REF": context["ref"],
                "GITHUB_REF_PROTECTED": "true",
                "GITHUB_WORKFLOW_REF": context["workflow_ref"],
                "GITHUB_SHA": context["sha"],
                "GITHUB_RUN_ID": context["run_id"],
                "GITHUB_SERVER_URL": "https://github.com",
            }
            with mock.patch.dict(os.environ, environment, clear=True):
                exit_code = summary_module.main()

            self.assertEqual(1, exit_code)
            self.assertIn("CI summary could not be rendered", summary_path.read_text(encoding="utf-8"))
            output = output_path.read_text(encoding="utf-8")
            self.assertNotIn("profile_evidence_manifest=", output)
            self.assertEqual("summary_written=false", output.strip())


if __name__ == "__main__":
    unittest.main(verbosity=2)
