"""Smoke test for the runnable Python vendor-extension example."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
EXAMPLE = ROOT / "bindings" / "python" / "examples" / "vendor_extension_registry.py"


class VendorExtensionExampleTests(unittest.TestCase):
    def test_example_runs_and_demonstrates_preservation_and_ordered_attachments(self) -> None:
        environment = os.environ.copy()
        source = str(ROOT / "bindings" / "python" / "src")
        existing_pythonpath = environment.get("PYTHONPATH")
        environment["PYTHONPATH"] = (
            source if not existing_pythonpath else os.pathsep.join((source, existing_pythonpath))
        )

        result = subprocess.run(
            [sys.executable, str(EXAMPLE)],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            check=False,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout)
        self.assertEqual(
            summary["inbound"],
            {
                "identity": "example.vendor/alpha/1",
                "recognized": True,
                "typed_value_available": True,
                "preserved_tags": [
                    "0x420001",
                    "0x420002",
                    "0x420004",
                    "0x420006",
                    "0x540001",
                ],
            },
        )
        self.assertEqual(
            summary["discover_versions"],
            [
                {
                    "identity": "example.vendor/alpha/1",
                    "criticality_indicator": False,
                },
                {
                    "identity": "example.vendor/ambiguous-beta/1",
                    "criticality_indicator": True,
                },
            ],
        )


if __name__ == "__main__":
    unittest.main()
