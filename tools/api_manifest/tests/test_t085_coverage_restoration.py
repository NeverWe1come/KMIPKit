"""Guard against reintroducing the temporary branch coverage waiver."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
BRANCH = "feature/KMIPKIT-0043-t076-shared-client-mapping"


class CoverageRestorationTests(unittest.TestCase):
    def test_ci_enforces_coverage_without_a_kmipkit_0043_branch_exception(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")

        self.assertNotIn(BRANCH, workflow)
        self.assertNotIn("temporary coverage-threshold waiver", workflow)
        self.assertNotIn("--cov-fail-under=0", workflow)
        self.assertIn("python -m coverage report", workflow)
        self.assertIn("--fail-under=90", workflow)
        self.assertIn("steps.enforce.outcome == 'skipped'", workflow)


if __name__ == "__main__":
    unittest.main()
