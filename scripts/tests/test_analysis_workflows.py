"""Security contracts for supplemental code analysis workflows."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"


class AnalysisWorkflowTests(unittest.TestCase):
    def read_workflow(self, filename: str) -> str:
        path = WORKFLOWS / filename
        self.assertTrue(path.is_file(), f"The {filename} workflow must exist.")
        return path.read_text(encoding="utf-8")

    def test_codeql_covers_all_shipped_languages(self) -> None:
        workflow = self.read_workflow("codeql.yml")
        for language in ("rust", "c-cpp", "java-kotlin", "python"):
            with self.subTest(language=language):
                self.assertRegex(workflow, rf"(?m)^\s+- language: {language}$")
        self.assertRegex(workflow, r"(?m)^\s+- language: rust\n\s+build-mode: none$")

    def test_codeql_actions_are_pinned_to_reviewed_commits(self) -> None:
        workflow = self.read_workflow("codeql.yml")
        action_lines = [line for line in workflow.splitlines() if re.search(r"\buses:\s", line)]
        self.assertTrue(action_lines)
        for line in action_lines:
            self.assertRegex(line, r"@[0-9a-f]{40}\s+#\s+.+v\d")

    def test_codeql_upload_does_not_run_on_fork_requests(self) -> None:
        workflow = self.read_workflow("codeql.yml")
        self.assertNotIn("pull_request_target:", workflow)
        self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", workflow)
        self.assertIn("persist-credentials: false", workflow)

    def test_sonar_automatic_analysis_is_not_duplicated_by_actions(self) -> None:
        self.assertFalse((WORKFLOWS / "sonar.yml").exists())


if __name__ == "__main__":
    unittest.main()
