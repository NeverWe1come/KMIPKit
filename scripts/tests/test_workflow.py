"""Static security and trigger contract checks for the pull-request workflow."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"


class WorkflowContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.contents = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else None

    def require_workflow(self) -> str:
        self.assertIsNotNone(self.contents, "The CI workflow must exist before its contract can pass.")
        return self.contents

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

    def test_nightly_schedule_runs_only_the_informational_branch_job(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^on:\s*\n(?:(?!^jobs:).)*?^\s+schedule:")
        self.assertIn("cron: '17 3 * * *'", contents)
        for job in ("core", "script-contracts", "coverage"):
            with self.subTest(job=job):
                match = re.search(rf"(?ms)^  {job}:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
                self.assertIsNotNone(match, f"The {job} job must exist.")
                self.assertRegex(match.group(1), r"(?m)^    if: github\.event_name == 'pull_request'$")

        branch_match = re.search(r"(?ms)^  branch-coverage:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
        self.assertIsNotNone(branch_match, "The branch-coverage job must exist.")
        self.assertRegex(branch_match.group(1), r"(?m)^    if: github\.event_name == 'schedule'$" )
        gate = re.search(r"(?ms)^  coverage-gate:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
        self.assertIsNotNone(gate)
        self.assertRegex(gate.group(1), r"(?m)^    if: always\(\) && github\.event_name == 'pull_request'$")
        self.assertIn("if: always() && needs.coverage.result != 'success'", gate.group(1))


if __name__ == "__main__":
    unittest.main(verbosity=2)
