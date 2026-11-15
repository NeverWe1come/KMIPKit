"""Keep runtime metadata namespaces distinct from their callable capabilities."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/043-public-api-bindings/spec.md"
NAMESPACES = {
    ("java-kmipkit", "org.kmipkit.RuntimeMetadata"),
    ("python-kmipkit", "kmipkit.metadata"),
}
CHILD_CAPABILITIES = {
    ("java-kmipkit", "org.kmipkit.RuntimeMetadata.version"),
    ("java-kmipkit", "org.kmipkit.RuntimeMetadata.capabilityIsAvailable"),
    ("java-kmipkit", "org.kmipkit.RuntimeMetadata.apiLevels"),
    ("python-kmipkit", "kmipkit.metadata.get_version"),
    ("python-kmipkit", "kmipkit.metadata.capability_is_available"),
    ("python-kmipkit", "kmipkit.metadata.get_api_levels"),
}


class RuntimeMetadataNamespaceExclusionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        self.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }

    def test_namespace_containers_are_exact_exclusions(self) -> None:
        for key in sorted(NAMESPACES):
            with self.subTest(export_name=key[1]):
                exclusion = self.exclusions[key]
                self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("metadata namespace", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-007", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertNotIn(key, self.declarations)

    def test_callable_runtime_metadata_and_api_level_capabilities_remain_declared(self) -> None:
        for key in sorted(CHILD_CAPABILITIES):
            with self.subTest(export_name=key[1]):
                declaration = self.declarations[key]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertTrue(declaration.get("languageMappings"))


if __name__ == "__main__":
    unittest.main()
