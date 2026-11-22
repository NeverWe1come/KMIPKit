import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "tools" / "install_artifact.py"
SPEC = importlib.util.spec_from_file_location("install_artifact", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
INSTALL_ARTIFACT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALL_ARTIFACT)


class InstallArtifactPathTests(unittest.TestCase):
    def test_git_archive_extraction_streams_from_a_temporary_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repository = root / "repository"
            repository.mkdir()
            subprocess.run(["git", "-C", str(repository), "init", "--quiet"], check=True)
            subprocess.run(["git", "-C", str(repository), "config", "user.name", "CI Test"], check=True)
            subprocess.run(
                ["git", "-C", str(repository), "config", "user.email", "ci@example.invalid"],
                check=True,
            )
            source_files = {
                "Cargo.toml": "[workspace]\n",
                "Cargo.lock": "# lock fixture\n",
                ".cargo/config.toml": "[net]\noffline = false\n",
                "crates/kmipkit-ffi/Cargo.toml": "[package]\nname = 'kmipkit-ffi'\n",
                "bindings/java/build.gradle.kts": "plugins {}\n",
                "bindings/c/include/kmipkit.h": "/* native ABI */\n",
                "specs/README.md": "excluded design source\n",
            }
            for relative_path, contents in source_files.items():
                source = repository / relative_path
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text(contents, encoding="utf-8")
            subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
            subprocess.run(["git", "-C", str(repository), "commit", "--quiet", "-m", "fixture"], check=True)
            commit = subprocess.run(
                ["git", "-C", str(repository), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            destination = root / "extracted"
            destination.mkdir()

            INSTALL_ARTIFACT.extract_git_archive(repository, commit, destination)

            self.assertEqual(
                (destination / "bindings/java/build.gradle.kts").read_text(encoding="utf-8"),
                "plugins {}\n",
            )
            self.assertFalse((destination / "specs").exists())

    def test_pin_file_must_be_inside_consumer_project(self):
        project = Path(__file__).parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            outside = Path(temporary) / "outside.json"
            outside.write_text("{}", encoding="utf-8")
            with self.assertRaises(ValueError):
                INSTALL_ARTIFACT.resolve_pin_file(outside, project)

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
