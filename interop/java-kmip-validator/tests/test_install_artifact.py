import importlib.util
from pathlib import Path
import shlex
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


class ReproducibleArtifactBuildEnvironmentTests(unittest.TestCase):
    def test_cargo_flags_remap_temp_root_and_preserve_caller_rustflags(self):
        temporary_root = Path("/tmp/kmipkit-java-source-with-spaces")

        environment = INSTALL_ARTIFACT.build_environment_with_path_remapping(
            {"RUSTFLAGS": "-C debuginfo=1"}, temporary_root
        )

        self.assertEqual(environment["RUSTFLAGS"], "-C debuginfo=1")
        self.assertEqual(
            environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
            [
                "-C",
                "debuginfo=1",
                f"--remap-path-prefix={temporary_root.resolve()}=/kmipkit-build",
            ],
        )

    def test_existing_encoded_rustflags_keep_precedence_and_are_preserved(self):
        environment = INSTALL_ARTIFACT.build_environment_with_path_remapping(
            {
                "RUSTFLAGS": "--cfg ignored_when_encoded_flags_are_set",
                "CARGO_ENCODED_RUSTFLAGS": "-C\x1fdebuginfo=0",
            },
            Path("/tmp/kmipkit-java-source-encoded"),
        )

        self.assertEqual(
            environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
            [
                "-C",
                "debuginfo=0",
                "--remap-path-prefix=/tmp/kmipkit-java-source-encoded=/kmipkit-build",
            ],
        )
        self.assertEqual(
            environment["RUSTFLAGS"], "--cfg ignored_when_encoded_flags_are_set"
        )

    def test_native_compiler_flags_remap_temp_root_and_preserve_caller_flags(self):
        temporary_root = Path("/tmp/kmipkit-java-source-with-spaces")

        environment = INSTALL_ARTIFACT.build_environment_with_path_remapping(
            {
                "CFLAGS": "-O2 -DNAME='two words'",
                "HOST_CFLAGS": "-O1",
                "CXXFLAGS": "-O0",
            },
            temporary_root,
        )

        for name, optimization in (
            ("CFLAGS", "-O2"),
            ("HOST_CFLAGS", "-O1"),
            ("CXXFLAGS", "-O0"),
        ):
            with self.subTest(name=name):
                flags = shlex.split(environment[name])
                self.assertIn(optimization, flags)
                self.assertIn(
                    "-ffile-prefix-map="
                    f"{temporary_root.resolve()}=/kmipkit-build",
                    flags,
                )
                self.assertIn(
                    "-fdebug-prefix-map="
                    f"{temporary_root.resolve()}=/kmipkit-build",
                    flags,
                )
        self.assertIn("-DNAME=two words", shlex.split(environment["CFLAGS"]))

    def test_empty_encoded_rustflags_do_not_create_empty_arguments(self):
        environment = INSTALL_ARTIFACT.build_environment_with_path_remapping(
            {"CARGO_ENCODED_RUSTFLAGS": "\x1f-C\x1fdebuginfo=0\x1f"},
            Path("/tmp/kmipkit-java-source-empty-flags"),
        )

        self.assertEqual(
            environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
            [
                "-C",
                "debuginfo=0",
                "--remap-path-prefix=/tmp/kmipkit-java-source-empty-flags=/kmipkit-build",
            ],
        )


class InstalledRouteProbeWorkflowTests(unittest.TestCase):
    def test_java_workflow_probes_and_persists_installed_package_routes(self):
        workflow = Path(__file__).parents[3] / ".github" / "workflows" / "interop-java.yml"
        content = workflow.read_text(encoding="utf-8")
        required_fragments = (
            "name: Probe installed Java package public routes",
            "../../interop/shared/tools/probe_route_artifact.py",
            "--language java",
            "--artifact build/maven-repository/org/kmipkit/kmipkit-java/1.0.0/kmipkit-java-1.0.0-linux-glibc-x86_64.jar",
            "--provenance build/artifact-manifest.json",
            "| tee build/route-capability-probe.json",
            "tools/compare_route_ledger.py",
            "name: Upload Java route capability evidence",
            "name: java-route-capability-evidence",
            "path: interop/java-kmip-validator/build/route-capability-probe.json",
        )
        for fragment in required_fragments:
            with self.subTest(fragment=fragment):
                self.assertIn(fragment, content)
        self.assertLess(
            content.index("name: Build and install the pinned JNI package"),
            content.index("name: Probe installed Java package public routes"),
        )
        self.assertLess(
            content.index("name: Probe installed Java package public routes"),
            content.index("name: Upload Java route capability evidence"),
        )
        self.assertIn("python3 -B -m unittest discover -s tests -p 'test_*.py' -v", content)

    def test_java_workflow_does_not_supply_cli_controlled_ledger_paths(self):
        workflow = Path(__file__).parents[3] / ".github" / "workflows" / "interop-java.yml"
        content = workflow.read_text(encoding="utf-8")
        compare_command = content.split("python3 tools/compare_route_ledger.py", maxsplit=1)[1].splitlines()[0]
        self.assertEqual(compare_command.strip(), "")
        self.assertNotIn("--reference", content)
        self.assertNotIn("--generated", content)


if __name__ == "__main__":
    unittest.main()
