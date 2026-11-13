"""The approved KMIP 3.0 fluent operation set must exist in the API matrix."""

from __future__ import annotations

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification/api/public-api.json"
OPERATIONS = {
    "Create Credential",
    "Create Group",
    "Create User",
    "Deactivate",
    "Decapsulate",
    "Encapsulate",
    "Obliterate",
}


class Kmip30FluentManifestTests(unittest.TestCase):
    def test_all_assigned_kmip30_operations_publish_verified_four_language_api(self) -> None:
        document = json.loads(MANIFEST.read_text(encoding="utf-8"))
        rows = {
            row["name"]: row
            for row in document.get("operations", [])
            if row.get("version") == "3.0" or row.get("scope") == "client_3_0_draft"
        }
        self.assertEqual(set(rows), OPERATIONS)
        for name, row in rows.items():
            with self.subTest(operation=name):
                self.assertEqual(row.get("scope"), "client_3_0_draft")
                self.assertEqual(row.get("apiState"), "verified")
                self.assertEqual(set(row.get("languageParity", [])),
                                 {"rust", "c", "java", "python"})
                self.assertTrue(row.get("parityEvidenceIds"))


if __name__ == "__main__":
    unittest.main()
