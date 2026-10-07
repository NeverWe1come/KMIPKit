"""Behavioral tests for the public API manifest generator CLI."""

from __future__ import annotations

import errno
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
GENERATOR = ROOT / "tools" / "api_manifest" / "generate.py"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
SCHEMA = ROOT / "specification" / "api" / "public-api.schema.json"
GENERATED_OUTPUTS = (
    "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "bindings/c/include/kmipkit.h",
    "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "tests/fixtures/extensions/generated/registry_parity.json",
)
SYMLINK_DENIAL_ERRNOS = {
    errno.EACCES,
    errno.EPERM,
    getattr(errno, "ENOTSUP", errno.EPERM),
    getattr(errno, "EOPNOTSUPP", errno.EPERM),
}
SYMLINK_DENIAL_WINERRORS = {5, 50, 1314}


def _create_repo(parent: Path) -> Path:
    """Copy the checked-in generation inputs into an isolated temporary root."""
    root = parent / "repo"
    api_dir = root / "specification" / "api"
    api_dir.mkdir(parents=True)
    shutil.copy2(MANIFEST, api_dir / MANIFEST.name)
    shutil.copy2(SCHEMA, api_dir / SCHEMA.name)
    return root


class ManifestGeneratorCliTests(unittest.TestCase):
    def _run_generator(self, root: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        self.assertTrue(
            GENERATOR.is_file(),
            "manifest generator CLI must be implemented before its behavior can be exercised",
        )
        return subprocess.run(
            [sys.executable, "-B", str(GENERATOR), *arguments],
            cwd=root,
            capture_output=True,
            check=False,
            text=True,
        )

    def _load_manifest(self, root: Path) -> dict[str, object]:
        path = root / "specification" / "api" / "public-api.json"
        return json.loads(path.read_text(encoding="utf-8"))

    def _write_manifest(self, root: Path, manifest: dict[str, object]) -> None:
        path = root / "specification" / "api" / "public-api.json"
        path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def _assert_symlink_creation_or_skip(self, link: Path, target: Path, *, directory: bool) -> None:
        try:
            link.symlink_to(target, target_is_directory=directory)
        except OSError as error:
            denied = error.errno in SYMLINK_DENIAL_ERRNOS or getattr(error, "winerror", None) in SYMLINK_DENIAL_WINERRORS
            if denied:
                self.skipTest(f"the operating system denied symlink creation: {error}")
            raise

    def test_write_is_deterministic_across_repeated_runs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))

            first = self._run_generator(root, "--write")
            self.assertEqual(first.returncode, 0, first.stderr)
            first_outputs = {
                relative_path: (root / relative_path).read_bytes()
                for relative_path in GENERATED_OUTPUTS
            }
            self.assertTrue(all(first_outputs.values()))

            second = self._run_generator(root, "--write")

            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(
                {relative_path: (root / relative_path).read_bytes() for relative_path in GENERATED_OUTPUTS},
                first_outputs,
            )

    def test_check_mode_compares_outputs_without_rewriting_them(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            write_result = self._run_generator(root, "--write")
            self.assertEqual(write_result.returncode, 0, write_result.stderr)
            output_paths = [root / path for path in GENERATED_OUTPUTS]
            before = {path: path.read_bytes() for path in output_paths}
            timestamp_ns = 946_684_800_000_000_000
            for path in output_paths:
                os.utime(path, ns=(timestamp_ns, timestamp_ns))
            timestamps = {path: path.stat().st_mtime_ns for path in output_paths}

            result = self._run_generator(root, "--check")

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual({path: path.read_bytes() for path in output_paths}, before)
            self.assertEqual({path: path.stat().st_mtime_ns for path in output_paths}, timestamps)

    def test_check_mode_reports_stale_output_without_rewriting_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            write_result = self._run_generator(root, "--write")
            self.assertEqual(write_result.returncode, 0, write_result.stderr)
            stale_output = root / GENERATED_OUTPUTS[0]
            sentinel = b"stale generated output\n"
            stale_output.write_bytes(sentinel)
            timestamp_ns = 946_684_800_000_000_000
            os.utime(stale_output, ns=(timestamp_ns, timestamp_ns))
            expected_mtime_ns = stale_output.stat().st_mtime_ns

            result = self._run_generator(root, "--check")

            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(stale_output.read_bytes(), sentinel)
            self.assertEqual(stale_output.stat().st_mtime_ns, expected_mtime_ns)

    def test_rejects_malformed_manifest_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            (root / "specification" / "api" / "public-api.json").write_text("{broken\n", encoding="utf-8")

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_rejects_unknown_required_manifest_field_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            manifest["generatorRequiredShortcut"] = True
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_rejects_traversal_output_destination_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            manifest = self._load_manifest(root)
            outputs = manifest["generatedOutputs"]
            self.assertIsInstance(outputs, dict)
            outputs["cHeader"] = "../escaped-kmipkit.h"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((parent / "escaped-kmipkit.h").exists())
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_rejects_symlinked_output_file_without_modifying_its_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            external_file = parent / "outside.h"
            sentinel = b"external sentinel\n"
            external_file.write_bytes(sentinel)
            link = root / GENERATED_OUTPUTS[1]
            link.parent.mkdir(parents=True)
            self._assert_symlink_creation_or_skip(link, external_file, directory=False)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(external_file.read_bytes(), sentinel)
            self.assertFalse(
                any((root / path).exists() for path in GENERATED_OUTPUTS if path != GENERATED_OUTPUTS[1])
            )

    def test_rejects_symlinked_output_parent_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            external_directory = parent / "outside-bindings"
            external_directory.mkdir()
            bindings = root / "bindings"
            self._assert_symlink_creation_or_skip(bindings, external_directory, directory=True)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            external_outputs = (
                external_directory / Path(path).relative_to("bindings")
                for path in GENERATED_OUTPUTS
                if path.startswith("bindings/")
            )
            self.assertFalse(any(path.exists() for path in external_outputs))
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))


if __name__ == "__main__":
    unittest.main()
