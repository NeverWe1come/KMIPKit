"""Resolve every required coverage object from Cargo's artifact messages."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FfiCoverageObjectTests(unittest.TestCase):
    def run_resolver(self, layout: str, missing: str | None = None):
        temporary = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        paths = [root / layout / filename for filename in (
            "libkmipkit_ffi.so", "libkmipkit_ffi.a", "kmipkit_ffi-abc", "c_api_coverage-def"
        )]
        for path in paths:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()
        messages = [
            {"reason": "compiler-artifact", "target": {"name": "kmipkit_ffi"},
             "profile": {"test": False}, "filenames": [str(p) for p in paths[:2]],
             "executable": None},
            *({"reason": "compiler-artifact", "target": {"name": name},
               "profile": {"test": True}, "filenames": [str(path)],
               "executable": str(path)}
              for name, path in zip(("kmipkit_ffi", "c_api_coverage"), paths[2:])),
        ]
        if missing:
            paths[{"dso": 0, "staticlib": 1, "unit": 2, "consumer": 3}[missing]].unlink()
        manifest = root / "cargo.jsonl"
        manifest.write_text("\n".join(json.dumps(m) for m in messages), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/ffi_coverage_objects.py"), str(manifest)],
            capture_output=True, check=False,
        )
        return result, paths

    def test_collects_all_four_objects_from_legacy_cargo_layout(self):
        result, paths = self.run_resolver("debug/deps")
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(result.stdout.split(b"\0")[:-1], [bytes(p) for p in paths])

    def test_collects_all_four_objects_from_cargo_1_100_layout_with_spaces(self):
        result, paths = self.run_resolver("debug/build/kmipkit-ffi/1234/out with spaces")
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(result.stdout.split(b"\0")[:-1], [bytes(p) for p in paths])

    def test_missing_object_fails_without_emitting_partial_coverage_inputs(self):
        for missing in ("dso", "staticlib", "unit", "consumer"):
            with self.subTest(missing=missing):
                result, _ = self.run_resolver("debug/build/kmipkit-ffi/1234/out", missing)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, b"")
                self.assertIn(b"missing", result.stderr.lower())


if __name__ == "__main__":
    unittest.main()
