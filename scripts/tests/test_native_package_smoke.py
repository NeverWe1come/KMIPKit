"""Contract tests for installed native-package smoke helpers."""

from __future__ import annotations

import argparse
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.native_package_smoke import python_smoke


class PythonNativePackageSmokeTests(unittest.TestCase):
    def test_wheel_smoke_requires_public_pkcs11_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "kmipkit"
            package_root.mkdir()
            (package_root / "client.py").write_text(
                "class Pkcs11Response: pass\nclass BatchItemResult: pkcs11_response = None\n",
                encoding="utf-8",
            )
            (package_root / "extensions.py").write_text(
                "def client_batch_item_pkcs11(): pass\n", encoding="utf-8"
            )
            python_smoke.verify_public_pkcs11_api_present(package_root)

    def test_wheel_smoke_rejects_missing_public_pkcs11_response_api(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            package_root = Path(temporary_directory) / "kmipkit"
            package_root.mkdir()
            (package_root / "client.py").write_text("class BatchItemResult: pass\n", encoding="utf-8")
            (package_root / "extensions.py").write_text(
                "def client_batch_item_pkcs11(): pass\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(RuntimeError, "public PKCS#11 response API"):
                python_smoke.verify_public_pkcs11_api_present(package_root)

    def test_main_preserves_consumer_virtualenv_interpreter_path(self) -> None:
        consumer_environment = Path.cwd() / "consumer-env"
        consumer_python = consumer_environment / (
            "Scripts/python.exe" if os.name == "nt" else "bin/python"
        )
        resolved_system_python = Path.cwd() / "system-python" / "python3.12"
        wrong_abi_library = Path.cwd() / "wrong-abi.so"

        with tempfile.TemporaryDirectory() as temporary_directory:
            smoke_root = Path(temporary_directory) / "smoke"
            arguments = argparse.Namespace(
                consumer_python=consumer_python,
                wrong_abi_library=wrong_abi_library,
                smoke_root=smoke_root,
            )
            original_resolve = type(consumer_python).resolve

            def resolve_venv_symlink(path: Path, *args: object, **kwargs: object) -> Path:
                if path == consumer_python:
                    return resolved_system_python
                return original_resolve(path, *args, **kwargs)

            with (
                patch.object(
                    python_smoke.argparse.ArgumentParser,
                    "parse_args",
                    return_value=arguments,
                ),
                patch.object(python_smoke, "run_smokes") as run_smokes,
                patch.object(
                    type(consumer_python),
                    "resolve",
                    autospec=True,
                    side_effect=resolve_venv_symlink,
                ),
            ):
                python_smoke.main()

        self.assertEqual(run_smokes.call_args.args[0], consumer_python.absolute())


if __name__ == "__main__":
    unittest.main()
