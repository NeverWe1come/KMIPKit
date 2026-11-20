"""T091 exclusion for Python wrapper storage metadata, not its public API."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
EXPORTED_PATH = "kmipkit.extensions.ClientBatchItem.__slots__"


class T091ClientBatchItemStorageExclusionTests(unittest.TestCase):
    def test_python_slot_descriptor_is_excluded_without_excluding_the_wrapper(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        key = ("python-kmipkit", EXPORTED_PATH)

        exclusion = exclusions.get(key)
        self.assertIsNotNone(exclusion, f"missing exact exclusion for {EXPORTED_PATH}")
        self.assertNotIn(key, declarations)
        self.assertIn("storage metadata", exclusion["reason"])
        self.assertIn(
            ("python-kmipkit", "kmipkit.extensions.ClientBatchItem"), declarations
        )
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
        self.assertEqual(
            {
                "kind": "feature-specification",
                "path": "specs/043-public-api-bindings/spec.md",
                "status": "Approved",
            },
            {
                field: exclusion["approvalSource"][field]
                for field in ("kind", "path", "status")
            },
        )


if __name__ == "__main__":
    unittest.main()
