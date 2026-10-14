"""Contract tests for installed native-package smoke helpers."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.native_package_smoke import python_smoke


class PythonNativePackageSmokeTests(unittest.TestCase):
    def test_package_native_directory_uses_consumer_venv_site_packages(self) -> None:
        consumer_python = Path("/tmp/consumer-env/bin/python")
        venv_site_packages = Path("/tmp/consumer-env/lib/python3.12/site-packages")
        global_site_packages = Path("/usr/local/lib/python3.12/dist-packages")

        def run_python(arguments: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            purelib = global_site_packages if "-I" in arguments else venv_site_packages
            return subprocess.CompletedProcess(
                args=arguments,
                returncode=0,
                stdout=f"{purelib}\n",
                stderr="",
            )

        with patch.object(python_smoke.subprocess, "run", side_effect=run_python):
            result = python_smoke.package_native_directory(consumer_python)

        self.assertEqual(result, venv_site_packages / "kmipkit" / "_native_ffi")


if __name__ == "__main__":
    unittest.main()
