"""Static checks for the installed C consumer workflow."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github" / "workflows" / "interop-c.yml"


class CWorkflowContractTests(unittest.TestCase):
    def test_workflow_uses_preinstalled_c_toolchain_without_apt_network(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("cmake --version", workflow)
        self.assertIn("gcc --version", workflow)
        self.assertNotIn("apt-get install", workflow)


    def test_schema_dependency_precedes_ctest_and_uses_its_python(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertLess(workflow.index('jsonschema==4.23.0'),
                        workflow.index('ctest --test-dir'))
        configure = next(line for line in workflow.splitlines() if 'cmake -S interop/' in line)
        self.assertIn('Python3_EXECUTABLE', configure)


if __name__ == "__main__":
    unittest.main()
