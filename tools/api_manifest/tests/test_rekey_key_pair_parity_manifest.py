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

    def test_table_310_fields_and_abi_ownership_are_exact(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {row["id"]: row for row in manifest["functions"]}
        builder = declarations["client_batch_item_rekey_key_pair"]
        expected_fields = [
            "private_key_unique_identifier",
            "offset",
            "common_attributes",
            "private_key_attributes",
            "public_key_attributes",
            "common_protection_storage_masks",
            "private_protection_storage_masks",
            "public_protection_storage_masks",
        ]
        self.assertEqual(expected_fields, [item["name"] for item in builder["python"]["parameters"]])
        self.assertEqual(
            [
                "privateKeyUniqueIdentifier",
                "offset",
                "commonAttributes",
                "privateKeyAttributes",
                "publicKeyAttributes",
                "commonProtectionStorageMasks",
                "privateProtectionStorageMasks",
                "publicProtectionStorageMasks",
            ],
            [item["name"] for item in builder["java"]["parameters"]],
        )
        self.assertIn("copied before the constructor returns", builder["c"]["documentation"][1])
        typed = declarations["client_batch_item_response_rekey_key_pair"]
        self.assertIn("borrowed from the response item", typed["c"]["documentation"][1])
        ttlv = declarations["client_batch_item_response_rekey_key_pair_ttlv"]
        self.assertEqual("kmipkit_client_batch_item_response_rekey_key_pair_ttlv", ttlv["rust"]["name"])
        self.assertIn("caller-owned copy", ttlv["c"]["documentation"][0])

    def test_operation_evidence_names_pair_binding_tests(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operation = next(row for row in manifest["operations"] if row["operationId"] == OPERATION_ID)
        evidence = set(operation["parityEvidenceIds"])
        self.assertIn("bindings/java/src/test/java/org/kmipkit/extensions/RekeyKeyPairMappingTest.java", evidence)
        self.assertIn("bindings/python/tests/test_rekey_key_pair_native_integration.py", evidence)
        self.assertIn("bindings/python/tests/test_rekey_key_pair_operation.py", evidence)

if __name__ == "__main__":
    unittest.main()
