from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github/workflows/interop-c.yml"


class ArtifactCheckoutContractTests(unittest.TestCase):
    def test_ci_checkout_includes_the_pinned_release_commit_history(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        marker = "      - name: Checkout repository\n"
        start = workflow.index(marker) + len(marker)
        next_step = workflow.find("      - name:", start)
        checkout_step = workflow[start:next_step]

        self.assertIn("fetch-depth: 0", checkout_step)


if __name__ == "__main__":
    unittest.main()
