"""The approved KMIP 3.0 fluent operation set must exist in the API matrix."""

from __future__ import annotations

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification/api/public-api.json"
KMIP30_DRAFT_API = ROOT / "specification/api/kmip-3.0-draft-api.json"
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
    def test_kmip30_fluent_capability_maps_all_routes_without_changing_21_inventory(self) -> None:
        document = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operations_21 = [
            row for row in document.get("operations", [])
            if row.get("scope") == "client_1_0"
        ]
        self.assertEqual(len(operations_21), 56)

        capabilities = {
            row.get("capabilityId"): row
            for row in document.get("capabilities", [])
        }
        capability = capabilities.get("kmip30_fluent_operations")
        self.assertIsNotNone(capability)
        assert capability is not None
        self.assertEqual(capability.get("ownerSpecId"), "KMIPKIT-0116")
        self.assertEqual(capability.get("capabilityState"), "verified")
        self.assertEqual(capability.get("parityScope"), "four-language")
        self.assertTrue(capability.get("parityEvidenceIds"))
        language_mappings = capability.get("mappings", {})
        self.assertEqual(set(language_mappings), {"rust", "c", "java", "python"})

        draft = json.loads(KMIP30_DRAFT_API.read_text(encoding="utf-8"))
        routes = {row["operation"]: row for row in draft.get("operations", [])}
        self.assertEqual(set(routes), OPERATIONS)
        for name, route in routes.items():
            with self.subTest(operation=name):
                mappings = route.get("fluentApi", {})
                self.assertEqual(set(mappings), {"rust", "c", "java", "python"})
                for language, mapping in mappings.items():
                    self.assertIsInstance(mapping, str)
                    self.assertIn(mapping, language_mappings[language])


if __name__ == "__main__":
    unittest.main()
