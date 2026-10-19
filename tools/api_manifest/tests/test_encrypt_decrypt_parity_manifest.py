"""Require four-language mapping for KMIP 2.1 Encrypt and Decrypt."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
LANGUAGES = {"rust", "c", "java", "python"}
OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-ENCRYPT": {
        "name": "Encrypt",
        "section": "6.1.17",
        "request": {
            "rust": "kmipkit_client::ClientRequest::Encrypt",
            "c": "kmipkit_client_batch_item_encrypt",
            "java": "org.kmipkit.extensions.ClientBatchItem.encrypt",
            "python": "kmipkit.extensions.client_batch_item_encrypt",
        },
        "response": {
            "rust": "kmipkit_client::ClientBatchOutcome::encrypt_response",
            "c": "kmipkit_client_batch_item_response_encrypt",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.encryptResponse",
            "python": "kmipkit.client.BatchItemResult.encrypt_response",
        },
    },
    "KMIPKIT-ELEM-OP-C2S-DECRYPT": {
        "name": "Decrypt",
        "section": "6.1.11",
        "request": {
            "rust": "kmipkit_client::ClientRequest::Decrypt",
            "c": "kmipkit_client_batch_item_decrypt",
            "java": "org.kmipkit.extensions.ClientBatchItem.decrypt",
            "python": "kmipkit.extensions.client_batch_item_decrypt",
        },
        "response": {
            "rust": "kmipkit_client::ClientBatchOutcome::decrypt_response",
            "c": "kmipkit_client_batch_item_response_decrypt",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.decryptResponse",
            "python": "kmipkit.client.BatchItemResult.decrypt_response",
        },
    },
}
OWNER_SPEC = "specs/019-cryptographic-operations/spec.md"
EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
    "bindings/python/tests/test_encrypt_decrypt_operations.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
}


class EncryptDecryptParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = {
            row["operationId"]: row
            for row in self.manifest["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.functions = {row["id"]: row for row in self.manifest["functions"]}

    def test_both_operations_are_verified_against_the_approved_owner(self) -> None:
        self.assertEqual(set(OPERATIONS), set(self.operations))
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                operation = self.operations[operation_id]
                self.assertEqual(expected["name"], operation["name"])
                self.assertEqual(expected["section"], operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0019", operation["featureSpecId"])
                self.assertEqual(
                    {"state": "approved", "specId": "KMIPKIT-0019"},
                    operation["ownerDependency"],
                )
                self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
                self.assertEqual("Approved", operation["approvalEvidence"]["status"])
                self.assertEqual(LANGUAGES, set(operation["declarations"]))
                self.assertTrue(EVIDENCE <= set(operation["parityEvidenceIds"]))
                for evidence in EVIDENCE:
                    self.assertTrue((ROOT / evidence).is_file(), evidence)

                mapped_functions = {
                    name
                    for function in self.functions.values()
                    if function["id"] in {
                        f"client_batch_item_{expected['name'].lower()}",
                        f"client_batch_item_response_{expected['name'].lower()}",
                    }
                    for name in (
                        function["c"]["symbol"],
                        function["java"].get("jniSymbol", ""),
                        function["python"].get("cffiSymbol", ""),
                    )
                }
                self.assertIn(expected["request"]["c"], mapped_functions)
                self.assertIn(expected["response"]["c"], mapped_functions)
                for function_id in (
                    f"client_batch_item_{expected['name'].lower()}",
                    f"client_batch_item_response_{expected['name'].lower()}",
                ):
                    function = self.functions[function_id]
                    self.assertTrue(
                        any(req.startswith("KMIPKIT-0019-FR-") for req in function["requirementIds"])
                    )
                    self.assertTrue(
                        any(req.startswith("KMIPKIT-0043-FR-") for req in function["requirementIds"])
                    )

    def test_generated_parity_inventory_contains_both_verified_operations(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = {
            row["operationId"]: row
            for row in generated["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.assertEqual(set(OPERATIONS), set(operations))
        for operation_id, operation in operations.items():
            self.assertEqual("verified", operation["apiState"], operation_id)
            self.assertEqual(LANGUAGES, set(operation["declarations"]), operation_id)


if __name__ == "__main__":
    unittest.main()
