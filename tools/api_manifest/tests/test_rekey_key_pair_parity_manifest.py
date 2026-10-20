"""Manifest acceptance tests for the KMIP 2.1 §6.1.47 binding mapping."""
from __future__ import annotations
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"

class RekeyKeyPairManifestTests(unittest.TestCase):
    def test_operation_is_gated_on_approved_four_language_mapping(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operation = next(row for row in manifest["operations"] if row["operationId"] == OPERATION_ID)
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual({"rust", "c", "java", "python"}, set(operation["declarations"]))
        self.assertTrue(operation["parityEvidenceIds"])
        linked = {identifier for values in operation["declarations"].values() for identifier in values}
        declarations = {row["declarationId"]: row for row in manifest["publicDeclarations"]}
        self.assertTrue(linked <= declarations.keys())
        self.assertTrue(all(declarations[item]["declarationState"] == "verified" for item in linked))

    def test_pair_response_retains_both_identifiers_as_public_api(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operation = next(row for row in manifest["operations"] if row["operationId"] == OPERATION_ID)
        names = {row["languageMappings"][language]["name"] for language, ids in operation["declarations"].items() for item in ids for row in manifest["publicDeclarations"] if row["declarationId"] == item}
        self.assertTrue(any("RekeyKeyPair" in name or "rekey_key_pair" in name or "rekeyKeyPair" in name for name in names))
        self.assertIn("KMIPKIT-REQ-SPEC-6.1.47-001", {item for row in manifest["publicDeclarations"] if row["declarationId"] in {x for ids in operation["declarations"].values() for x in ids} for item in row["requirementIds"]})

if __name__ == "__main__":
    unittest.main()
