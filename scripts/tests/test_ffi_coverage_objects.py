"""Resolve the ABI libraries and every executed FFI coverage test from Cargo."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FfiCoverageObjectTests(unittest.TestCase):
    def run_resolver(
        self,
        layout: str,
        missing: str | None = None,
        ambiguous: bool = False,
        target_root: Path | None = None,
        external_test_role: str | None = None,
    ) -> tuple[subprocess.CompletedProcess[bytes], list[Path]]:
        temporary_root = ROOT / "target"
        temporary_root.mkdir(exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=temporary_root)
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        paths = [
            root / layout / filename
            for filename in (
                "libkmipkit_ffi.so",
                "libkmipkit_ffi.a",
                "kmipkit_ffi-abc",
                "c_api_coverage-def",
                "operation_parity_0053-fed",
            )
        ]
        for path in paths:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()
        messages = [
            {
                "reason": "compiler-artifact",
                "target": {"name": "kmipkit_ffi"},
                "profile": {"test": False},
                "filenames": [str(p) for p in paths[:2]],
                "executable": None,
            }
        ]
        for name, path in zip(
            ("kmipkit_ffi", "c_api_coverage", "operation_parity_0053"), paths[2:]
        ):
            messages.append({
                "reason": "compiler-artifact",
                "target": {"name": name},
                "profile": {"test": True},
                "filenames": [str(path)],
                "executable": str(path),
            })
        if external_test_role:
            role_index = {"unit": 1, "consumer": 2, "parity": 3}[external_test_role]
            messages[role_index]["executable"] = str(ROOT / "scripts" / "ffi_coverage_objects.py")
        # Cargo also reports build scripts, unrelated crates, and completion.
        messages.extend([
            {"reason": "build-finished", "success": True},
            {"reason": "compiler-artifact", "target": {"name": "other_crate"},
             "profile": {"test": False}, "filenames": [], "executable": None},
        ])
        if ambiguous:
            second_dso = root / "other-build" / "libkmipkit_ffi.so"
            second_dso.parent.mkdir()
            second_dso.touch()
            messages[0]["filenames"].append(str(second_dso))
        if missing:
            paths[{"dso": 0, "staticlib": 1, "unit": 2, "consumer": 3, "parity": 4}[missing]].unlink()
        manifest = root / "cargo.jsonl"
        manifest.write_text("\n".join(json.dumps(m) for m in messages), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/ffi_coverage_objects.py")],
            input=manifest.read_bytes(),
            capture_output=True,
            check=False,
            env={**os.environ, "CARGO_TARGET_DIR": str(target_root or root)},
        )
        return result, paths

    def test_collects_libraries_and_all_executed_tests_from_legacy_cargo_layout(self):
        result, paths = self.run_resolver("debug/deps")
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(result.stdout.split(b"\0")[:-1], [bytes(p) for p in paths])

    def test_collects_libraries_and_all_executed_tests_from_cargo_1_100_layout_with_spaces(self):
        result, paths = self.run_resolver("debug/build/kmipkit-ffi/1234/out with spaces")
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(result.stdout.split(b"\0")[:-1], [bytes(p) for p in paths])

    def test_ambiguous_artifact_fails_without_emitting_partial_coverage_inputs(self):
        result, _ = self.run_resolver("debug/deps", ambiguous=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")
        self.assertIn(b"found 2", result.stderr)

    def test_missing_object_fails_without_emitting_partial_coverage_inputs(self):
        for missing in ("dso", "staticlib", "unit", "consumer", "parity"):
            with self.subTest(missing=missing):
                result, _ = self.run_resolver("debug/build/kmipkit-ffi/1234/out", missing)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b"")
                self.assertIn(b"missing", result.stderr.lower())

    def test_rejects_filesystem_path_arguments(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/ffi_coverage_objects.py"), "../secrets"],
            input=b"",
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"usage", result.stderr.lower())

    def test_rejects_artifacts_outside_the_cargo_target_directory(self):
        target_root = ROOT / "target" / "unexpected-cargo-target"
        target_root.mkdir(exist_ok=True)
        result, _ = self.run_resolver("debug/deps", target_root=target_root)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")
        self.assertIn(b"outside", result.stderr.lower())

    def test_rejects_test_executables_outside_the_cargo_target_directory(self):
        for role in ("unit", "consumer", "parity"):
            with self.subTest(role=role):
                result, _ = self.run_resolver("debug/deps", external_test_role=role)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b"")
                self.assertIn(b"outside", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()
