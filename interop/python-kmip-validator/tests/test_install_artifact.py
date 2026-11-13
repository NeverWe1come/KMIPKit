from pathlib import Path
import importlib.util
from unittest import TestCase, mock


ROOT = Path(__file__).resolve().parents[3]
INSTALLER = ROOT / "interop/python-kmip-validator/tools/install_artifact.py"
SPEC = importlib.util.spec_from_file_location("python_validator_installer", INSTALLER)
assert SPEC and SPEC.loader
installer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(installer)


class ArtifactInstallerTests(TestCase):
    def test_replaces_an_installed_package_with_the_exact_pinned_wheel(self):
        wheel = Path("kmipkit-1.0.0-py3-none-linux_x86_64.whl")
        with mock.patch.object(installer.subprocess, "run") as run:
            installer.install(wheel)

        command = run.call_args.args[0]
        self.assertIn("--force-reinstall", command)
        self.assertEqual(command[-1], str(wheel.resolve()))
