import importlib.util
from pathlib import Path
import tempfile
import unittest

from kmipkit_validator.reporter import resolve_report_directory


PROJECT = Path(__file__).resolve().parents[1]
INSTALLER_PATH = PROJECT / "tools" / "install_artifact.py"
SPEC = importlib.util.spec_from_file_location("install_artifact", INSTALLER_PATH)
assert SPEC is not None and SPEC.loader is not None
INSTALLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALLER)


class SafeOutputPathTests(unittest.TestCase):
    def test_report_directory_stays_under_the_invocation_directory(self):
        self.assertEqual(
            (Path.cwd() / "results").resolve(),
            resolve_report_directory(Path("results")),
        )

    def test_report_directory_rejects_parent_traversal_and_absolute_paths(self):
        with self.assertRaises(ValueError):
            resolve_report_directory(Path("../outside"))
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ValueError):
                resolve_report_directory(Path(temporary))

    def test_artifact_outputs_stay_under_the_validator_project(self):
        candidate = PROJECT / "build" / "artifact-manifest.json"
        self.assertEqual(candidate.resolve(), INSTALLER.resolve_project_output(candidate))
        with self.assertRaises(ValueError):
            INSTALLER.resolve_project_output(PROJECT.parent / "outside.json")


if __name__ == "__main__":
    unittest.main()
