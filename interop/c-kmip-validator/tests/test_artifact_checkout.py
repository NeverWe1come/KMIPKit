from pathlib import Path
import re
import subprocess
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

    def test_pinned_c_artifact_is_consistent_and_in_release_history(self):
        installer = (ROOT / "interop/c-kmip-validator/tools/install_artifact.py").read_text(
            encoding="utf-8"
        )
        verifier = (ROOT / "interop/c-kmip-validator/tools/verify_artifact.py").read_text(
            encoding="utf-8"
        )
        match = re.search(r'^PIN = "([0-9a-f]{40})"$', installer, re.MULTILINE)
        self.assertIsNotNone(match)
        pin = match.group(1)
        self.assertIn(f'PIN = "{pin}"', verifier)

        cmake = (ROOT / "interop/c-kmip-validator/CMakeLists.txt").read_text(encoding="utf-8")
        workflow = WORKFLOW.read_text(encoding="utf-8")
        readme = (ROOT / "interop/c-kmip-validator/README.md").read_text(encoding="utf-8")
        spec = (ROOT / "specs/085-interop-c-live/spec.md").read_text(encoding="utf-8")
        self.assertIn(f'set(KMIPKIT_LIBRARY_COMMIT "{pin}")', cmake)
        self.assertIn(f"/install/{pin}", workflow)
        self.assertIn(pin, readme)
        self.assertIn(f'**Base**: release `{pin}`', spec)

        subprocess.run(
            ["git", "cat-file", "-e", f"{pin}^{{tree}}"], cwd=ROOT, check=True
        )
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", pin, "origin/release/1.0.0"],
            cwd=ROOT,
            check=True,
        )


if __name__ == "__main__":
    unittest.main()
