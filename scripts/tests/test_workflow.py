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


if __name__ == "__main__":
    unittest.main(verbosity=2)
