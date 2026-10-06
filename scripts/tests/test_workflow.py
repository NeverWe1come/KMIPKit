"""Static security and trigger contract checks for the pull-request workflow."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"
TESTING_GUIDE = REPOSITORY_ROOT / "docs" / "development" / "testing.md"
POLICY_RUNNER = REPOSITORY_ROOT / "scripts" / "Test-DependencyPolicy.ps1"


class WorkflowContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contents = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else None

    def require_workflow(self) -> str:
        self.assertIsNotNone(self.contents, "The CI workflow must exist before its contract can pass.")
        return self.contents

    def require_job(self, contents: str, job: str) -> str:
        match = re.search(rf"(?ms)^  {re.escape(job)}:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
        self.assertIsNotNone(match, f"The {job} job must exist.")
        return match.group(1)

    def assert_pi_runner_with_hosted_fallback(self, body: str, fallback: str) -> None:
        self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", body)
        self.assertIn("fromJSON('[\"self-hosted\",\"Linux\",\"ARM64\"]')", body)
        self.assertIn(fallback, body)

    def require_policy_job(self, contents: str, job: str) -> str:
        return self.require_job(contents, job)

    def require_policy_runner(self) -> str:
        self.assertTrue(
            POLICY_RUNNER.is_file(),
            "The pinned dependency-policy runner must exist before its workflow contract can pass.",
        )
        return POLICY_RUNNER.read_text(encoding="utf-8")

    def scheduled_policy_job(self, contents: str) -> str:
        jobs = re.findall(r"(?ms)^  ([a-z][\w-]*):\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
        for _, body in jobs:
            if "Test-DependencyPolicy.ps1" in body and re.search(r"(?i)github\.event_name.*schedule", body):
                return body
        self.fail("A scheduled job must invoke the dependency-policy runner.")

    @staticmethod
    def inline_run_scalars(contents: str) -> list[tuple[int, str]]:
        scalars: list[tuple[int, str]] = []
        for line_number, line in enumerate(contents.splitlines(), start=1):
            match = re.match(r"^\s*run:\s*(.*?)\s*$", line)
            if match is not None:
                scalars.append((line_number, match.group(1)))
        return scalars

    def test_pull_request_targets_only_supported_integration_branches(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?m)^\s*pull_request\s*:")
        self.assertIn("master", contents)
        self.assertRegex(contents, r"release/\*\*")
        self.assertNotRegex(contents, r"(?m)^\s*pull_request_target\s*:")
        self.assertNotRegex(contents, r"(?m)^\s*workflow_run\s*:")

    def test_workflow_permissions_are_read_only(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^permissions:\s*\n\s*contents:\s*read\b")
        self.assertNotRegex(contents, r"(?m)^\s*(?:write-all|packages:\s*write|contents:\s*write)\b")

    def test_external_actions_use_full_commit_sha_and_release_comment(self) -> None:
        contents = self.require_workflow()
        action_lines = [line for line in contents.splitlines() if " uses:" in line]
        self.assertTrue(action_lines, "The workflow must list its action dependencies.")
        for line in action_lines:
            with self.subTest(line=line):
                self.assertRegex(line, r"@[0-9a-f]{40}\s+#\s+.+v\d")
                self.assertNotRegex(line, r"@(?:v\d|main|master)\b")

    def test_workflow_has_no_secrets_or_repository_writes(self) -> None:
        contents = self.require_workflow()
        self.assertNotIn("secrets.", contents)
        self.assertNotRegex(contents, r"(?i)gh pr (?:create|merge)|git push|cargo publish")

    def test_required_platform_toolchain_and_informational_branch_matrices_exist(self) -> None:
        contents = self.require_workflow()
        self.assertIn("os: [ubuntu-latest, windows-latest, macos-latest]", contents)
        self.assertIn("rust: ['1.94', stable]", contents)
        self.assertIn("platform: ubuntu", contents)
        self.assertIn("platform: windows", contents)
        self.assertIn("platform: macos", contents)
        self.assertRegex(contents, r"(?ms)^  branch-coverage:.*?continue-on-error:\s*true")
        self.assertIn("--branch --json --summary-only", contents)

    def test_linux_jobs_route_to_pi_only_for_same_repository_pull_requests(self) -> None:
        contents = self.require_workflow()

        for job in ("core", "script-contracts", "coverage"):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assertIn("matrix.os == 'ubuntu-latest'", body)
                self.assert_pi_runner_with_hosted_fallback(body, "|| matrix.os")

        for job in ("normative-inventory", "coverage-gate"):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assert_pi_runner_with_hosted_fallback(body, "|| 'ubuntu-latest'")

        branch_coverage = self.require_job(contents, "branch-coverage")
        self.assertRegex(branch_coverage, r"(?m)^    runs-on: \[self-hosted, Linux, ARM64\]$")

    def test_nightly_schedule_runs_only_the_informational_branch_job(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^on:\s*\n(?:(?!^jobs:).)*?^\s+schedule:")
        self.assertIn("cron: '17 3 * * *'", contents)
        for job in ("core", "script-contracts", "coverage"):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assertRegex(body, r"(?m)^    if: github\.event_name == 'pull_request'$")

        branch_coverage = self.require_job(contents, "branch-coverage")
        self.assertRegex(branch_coverage, r"(?m)^    if: github\.event_name == 'schedule'$")
        gate = self.require_job(contents, "coverage-gate")
        self.assertRegex(gate, r"(?m)^    if: always\(\) && github\.event_name == 'pull_request'$")
        self.assertIn("if: always() && needs.coverage.result != 'success'", gate)

    def test_branch_coverage_documentation_matches_schedule_only_workflow(self) -> None:
        guide = TESTING_GUIDE.read_text(encoding="utf-8")
        paragraph = re.search(r"(?ms)^CI attempts branch coverage separately.*?(?=\n\n|\Z)", guide)
        self.assertIsNotNone(paragraph, "The branch-coverage documentation must exist.")
        normalized_paragraph = " ".join(paragraph.group(0).split())
        self.assertIn("only on a daily schedule", normalized_paragraph)
        self.assertIn("does not run on pull requests", normalized_paragraph)

    def test_every_supported_pull_request_runs_the_dependency_policy_job(self) -> None:
        contents = self.require_workflow()
        trigger = re.search(r"(?ms)^on:\s*\n(.*?)(?=^permissions:)", contents)
        self.assertIsNotNone(trigger, "The workflow must declare pull-request triggers.")
        pull_request = re.search(r"(?ms)^  pull_request:\s*\n(.*?)(?=^  schedule:|\Z)", trigger.group(1))
        self.assertIsNotNone(pull_request, "Dependency policy requires pull_request events.")
        self.assertIn("master", pull_request.group(1))
        self.assertRegex(pull_request.group(1), r"release/\*\*")
        self.assertNotRegex(pull_request.group(1), r"(?m)^\s*paths(?:-ignore)?:")

        job = self.require_policy_job(contents, "dependency-policy")
        self.assertNotRegex(job, r"(?m)^\s*if:.*(?:paths|changed-files)")
        self.assertRegex(job, r"(?i)Test-DependencyPolicy\.ps1")

    def test_dependency_policy_job_is_read_only_and_routes_fork_runs_to_hosted_linux(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^permissions:\s*\n\s*contents:\s*read\b")
        self.assertNotRegex(contents, r"(?m)^\s*(?:contents|pull-requests|packages):\s*write\b")
        self.assertNotIn("secrets.", contents)
        self.assertNotRegex(contents, r"(?m)^\s*pull_request_target\s*:")

        job = self.require_policy_job(contents, "dependency-policy")
        self.assert_pi_runner_with_hosted_fallback(job, "|| 'ubuntu-latest'")

    def test_dependency_policy_job_pins_tool_and_checks_both_workspaces(self) -> None:
        contents = self.require_workflow()
        job = self.require_policy_job(contents, "dependency-policy")
        self.assertRegex(job, r"(?i)Test-DependencyPolicy\.ps1")
        runner = self.require_policy_runner()
        self.assertIn("$expectedDenyVersion = '0.20.2'", runner)
        self.assertRegex(
            runner,
            r"(?is)-Arguments\s+@\(\s*'install',\s*'--locked',\s*'--version',\s*\$expectedDenyVersion,\s*'cargo-deny'\s*\)",
        )
        for workspace in ("Cargo.toml", "fuzz/Cargo.toml"):
            with self.subTest(workspace=workspace):
                self.assertIn(workspace, runner)
        self.assertIn("--all-features", runner)
        self.assertIn("--locked", runner)

    def test_daily_policy_schedule_checks_out_the_configured_active_release_ref(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?m)^\s+schedule:\s*$")
        self.assertRegex(contents, r"(?m)^\s+- cron:\s*['\"](?:\d+\s+\d+\s+\*\s+\*\s+\*|@daily)['\"]")

        job = self.scheduled_policy_job(contents)
        self.assertRegex(job, r"(?i)github\.event_name.*schedule")
        self.assertRegex(job, r"(?i)actions/checkout")

        active_ref = re.search(
            r"(?im)^\s*(?:ACTIVE_RELEASE_REF|active-release-ref):\s*([^\s#]+)", contents
        )
        self.assertIsNotNone(active_ref, "The active release ref must be an explicit workflow setting.")
        self.assertRegex(
            job,
            r"(?i)ref:\s*\$\{\{\s*(?:env|vars)\.(?:ACTIVE_RELEASE_REF|active-release-ref)\s*\}\}",
        )
        self.assertNotIn("ref: ${{ github.sha }}", job)

    def test_scheduled_policy_reports_scanned_commit_and_each_rustsec_revision(self) -> None:
        contents = self.require_workflow()
        job = self.scheduled_policy_job(contents)
        active_ref_run = next(
            (scalar for _, scalar in self.inline_run_scalars(job) if "Active release ref:" in scalar),
            None,
        )
        self.assertEqual('echo "Active release ref: $ACTIVE_RELEASE_REF"', active_ref_run.strip("'"))
        self.assertIn("$ACTIVE_RELEASE_REF", job)
        runner = self.require_policy_runner()
        output = job + "\n" + runner
        for required in ("scanned commit", "RustSec", "root", "fuzz", "SHA", "timestamp"):
            with self.subTest(required=required):
                self.assertIn(required.lower(), output.lower())

    def test_inline_run_commands_with_yaml_colons_are_quoted(self) -> None:
        contents = self.require_workflow()
        unsafe_scalars = [
            (line_number, scalar)
            for line_number, scalar in self.inline_run_scalars(contents)
            if ": " in scalar and not scalar.startswith(("'", '"'))
        ]
        self.assertEqual(
            [],
            unsafe_scalars,
            f"Inline run scalars containing ': ' must be quoted: {unsafe_scalars}",
        )

    def test_testing_guide_explains_schedule_default_branch_activation(self) -> None:
        guide = TESTING_GUIDE.read_text(encoding="utf-8").lower()
        self.assertIn("default branch", guide)
        self.assertRegex(guide, r"(?s)schedul.{0,180}(?:integrat|default branch)")
        self.assertRegex(guide, r"(?s)(?:integrat|merged).{0,180}(?:default branch|scheduled)")

    def test_dependency_policy_local_command_and_review_process_are_documented(self) -> None:
        testing_guide = TESTING_GUIDE.read_text(encoding="utf-8")
        policy_guide = (REPOSITORY_ROOT / "docs" / "security" / "dependency-policy.md").read_text(encoding="utf-8")
        self.assertIn("pwsh -File .\\scripts\\Test-DependencyPolicy.ps1", testing_guide)
        self.assertIn("0.20.2", testing_guide)
        self.assertIn("root and fuzz", testing_guide)
        normalized_policy_guide = " ".join(policy_guide.lower().split())
        self.assertRegex(normalized_policy_guide, r"tool upgrades?.{0,160}(?:review|version|engineering)")
        self.assertRegex(normalized_policy_guide, r"(?s)## exception review lifecycle.*?before expiry")
        for required in (
            "exact crate name and resolved version",
            "rationale",
            "mitigation",
            "accountable owner",
            "reviewer different from that owner",
            "expiry",
            "approval reference",
            "renewal is a new human decision",
            "remove the dependency/finding and both exception entries",
            "remove the exception from both files",
        ):
            with self.subTest(exception_review=required):
                self.assertIn(required, normalized_policy_guide)

    def test_run_summary_waits_for_all_jobs_and_runs_after_failures(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "run-summary")
        self.assertRegex(job, r"(?m)^    if: always\(\)$")
        for dependency in (
            "core",
            "script-contracts",
            "normative-inventory",
            "coverage",
            "coverage-gate",
            "dependency-policy",
            "scheduled-dependency-policy",
            "branch-coverage",
        ):
            with self.subTest(dependency=dependency):
                self.assertRegex(job, rf"(?m)^      - {re.escape(dependency)}$")
        self.assertIn("scripts/ci_summary.py", job)
        self.assertIn("${{ toJSON(needs) }}", job)
        self.assertIn("GITHUB_STEP_SUMMARY", job)
        self.assert_pi_runner_with_hosted_fallback(job, "|| fromJSON('[\"self-hosted\",\"Linux\",\"ARM64\"]')")

    def test_coverage_gate_adds_its_result_and_metrics_to_the_job_summary(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "coverage-gate")
        self.assertIn("--summary-file", job)
        self.assertIn("$GITHUB_STEP_SUMMARY", job)


if __name__ == "__main__":
    unittest.main(verbosity=2)
