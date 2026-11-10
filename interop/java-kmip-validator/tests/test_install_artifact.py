import importlib.util
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "tools" / "install_artifact.py"
SPEC = importlib.util.spec_from_file_location("install_artifact", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
INSTALL_ARTIFACT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALL_ARTIFACT)


class InstallArtifactPathTests(unittest.TestCase):
    def test_pin_file_must_be_inside_consumer_project(self):
        project = Path(__file__).parents[1]
        with self.assertRaises(ValueError):
            INSTALL_ARTIFACT.resolve_pin_file(project.parent / "outside.json", project)

    def test_pin_file_symlink_cannot_escape_consumer_project(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            project = root / "consumer"
            project.mkdir()
            outside = root / "outside.json"
            outside.write_text("{}", encoding="utf-8")
            link = project / "outside-link.json"
            link.symlink_to(outside)
            with self.assertRaises(ValueError):
                INSTALL_ARTIFACT.resolve_pin_file(link, project)

    def test_pin_file_inside_consumer_project_is_accepted(self):
        project = Path(__file__).parents[1]
        self.assertEqual(
            INSTALL_ARTIFACT.resolve_pin_file(project / "library-pin.json", project),
            (project / "library-pin.json").resolve(),
        )


if __name__ == "__main__":
    unittest.main()
