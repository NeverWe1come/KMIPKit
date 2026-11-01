"""Manifest regressions for the T033 cryptographic operation mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-HASH": ("Hash", "6.1.24"),
    "KMIPKIT-ELEM-OP-C2S-MAC": ("MAC", "6.1.32"),
    "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": ("MAC Verify", "6.1.33"),
    "KMIPKIT-ELEM-OP-C2S-SIGN": ("Sign", "6.1.55"),
    "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": ("Signature Verify", "6.1.56"),
}
LANGUAGES = {"rust", "c", "java", "python"}
NORMATIVE_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.32-001",
    "KMIPKIT-REQ-SPEC-6.1.33-001-001",
    "KMIPKIT-REQ-SPEC-6.1.33-001-002",
    "KMIPKIT-REQ-SPEC-6.1.55-001",
    "KMIPKIT-REQ-SPEC-6.1.56-001-001",
    "KMIPKIT-REQ-SPEC-6.1.56-002",
}
PUBLIC_FUNCTIONS = {
    "client_batch_item_hash": (
        "kmipkit_client_batch_item_hash",
        "ClientBatchItem",
        "hash",
        "kmipkit.extensions.client_batch_item_hash",
    ),
    "client_batch_item_mac": (
        "kmipkit_client_batch_item_mac",
        "ClientBatchItem",
        "mac",
        "kmipkit.extensions.client_batch_item_mac",
    ),
    "client_batch_item_mac_verify": (
        "kmipkit_client_batch_item_mac_verify",
        "ClientBatchItem",
        "macVerify",
        "kmipkit.extensions.client_batch_item_mac_verify",
    ),
    "client_batch_item_sign": (
        "kmipkit_client_batch_item_sign",
        "ClientBatchItem",
        "sign",
        "kmipkit.extensions.client_batch_item_sign",
    ),
    "client_batch_item_signature_verify": (
        "kmipkit_client_batch_item_signature_verify",
        "ClientBatchItem",
        "signatureVerify",
        "kmipkit.extensions.client_batch_item_signature_verify",
    ),
    "client_batch_item_response_hash": (
        "kmipkit_client_batch_item_response_hash",
        "ClientBatchItemResult",
        "hashResponse",
        "kmipkit.client.BatchItemResult.hash_response",
    ),
    "client_batch_item_response_mac": (
        "kmipkit_client_batch_item_response_mac",
        "ClientBatchItemResult",
        "macResponse",
        "kmipkit.client.BatchItemResult.mac_response",
    ),
    "client_batch_item_response_mac_verify": (
        "kmipkit_client_batch_item_response_mac_verify",
        "ClientBatchItemResult",
        "macVerifyResponse",
        "kmipkit.client.BatchItemResult.mac_verify_response",
    ),
    "client_batch_item_response_sign": (
        "kmipkit_client_batch_item_response_sign",
        "ClientBatchItemResult",
        "signResponse",
        "kmipkit.client.BatchItemResult.sign_response",
    ),
    "client_batch_item_response_signature_verify": (
        "kmipkit_client_batch_item_response_signature_verify",
        "ClientBatchItemResult",
        "signatureVerifyResponse",
        "kmipkit.client.BatchItemResult.signature_verify_response",
    ),
    "client_batch_item_response_hash_ttlv": (
        "kmipkit_client_batch_item_response_hash_ttlv",
        "HashResponse",
        "responseTtlv",
        "kmipkit.client.HashResponse.response_ttlv",
    ),
    "client_batch_item_response_mac_ttlv": (
        "kmipkit_client_batch_item_response_mac_ttlv",
        "MacResponse",
        "responseTtlv",
        "kmipkit.client.MacResponse.response_ttlv",
    ),
    "client_batch_item_response_mac_verify_ttlv": (
        "kmipkit_client_batch_item_response_mac_verify_ttlv",
        "MacVerifyResponse",
        "responseTtlv",
        "kmipkit.client.MacVerifyResponse.response_ttlv",
    ),
    "client_batch_item_response_sign_ttlv": (
        "kmipkit_client_batch_item_response_sign_ttlv",
        "SignResponse",
        "responseTtlv",
        "kmipkit.client.SignResponse.response_ttlv",
    ),
    "client_batch_item_response_signature_verify_ttlv": (
        "kmipkit_client_batch_item_response_signature_verify_ttlv",
        "SignatureVerifyResponse",
        "responseTtlv",
        "kmipkit.client.SignatureVerifyResponse.response_ttlv",
    ),
}


class HashMacSignatureParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = {
            row["operationId"]: row
            for row in self.manifest["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.declarations = {
            row["declarationId"]: row
            for row in self.manifest["publicDeclarations"]
        }

    def test_five_operations_have_approved_four_language_mappings(self) -> None:
        self.assertEqual(set(OPERATIONS), set(self.operations))

        linked_requirements: set[str] = set()
        for operation_id, (name, section) in OPERATIONS.items():
            with self.subTest(operation=name):
                operation = self.operations[operation_id]
                self.assertEqual(name, operation["name"])
                self.assertEqual(section, operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("declared", operation["apiState"])
                self.assertEqual("KMIPKIT-0021", operation["featureSpecId"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("KMIPKIT-0021", operation["ownerDependency"]["specId"])
                self.assertEqual(
                    "specs/021-hash-mac-signature-operations/spec.md",
                    operation["approvalEvidence"]["path"],
                )
                self.assertEqual("Approved", operation["approvalEvidence"]["status"])
                self.assertNotIn("blockedReason", operation)
                self.assertEqual(LANGUAGES, set(operation["declarations"]))

                for language, declaration_ids in operation["declarations"].items():
                    self.assertTrue(declaration_ids, f"{name} has no {language} declarations")
                    for declaration_id in declaration_ids:
                        declaration = self.declarations[declaration_id]
                        expected_declaration_state = (
                            "verified"
                            if declaration_id.startswith("T101:")
                            or declaration["exportName"].endswith("Response::with_ttlv")
                            or declaration["exportName"]
                            in {
                                "kmipkit_protocol::MacVerifyResponse::validity_indicator",
                                "kmipkit_protocol::SignatureVerifyResponse::validity_indicator",
                            }
                            else "declared"
                        )
                        self.assertIn(
                            declaration["capabilityId"],
                            {operation_id, "client_execution"},
                        )
                        self.assertEqual(expected_declaration_state, declaration["declarationState"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                        self.assertIn(language, declaration["languageMappings"])
                        self.assertEqual(
                            "specs/021-hash-mac-signature-operations/spec.md",
                            declaration["approvalSource"]["path"],
                        )
                        linked_requirements.update(declaration["requirementIds"])

        self.assertTrue(NORMATIVE_REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_the_same_five_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        generated_operations = {
            row["operationId"]: row
            for row in generated["operations"]
            if row["operationId"] in OPERATIONS
        }

        self.assertEqual(set(OPERATIONS), set(generated_operations))
        for operation_id in OPERATIONS:
            with self.subTest(operation=operation_id):
                self.assertEqual("declared", generated_operations[operation_id]["apiState"])
                self.assertEqual(LANGUAGES, set(generated_operations[operation_id]["declarations"]))

    def test_generated_parity_inventory_contains_t033_and_t101_functions(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        generated_functions = {
            row["id"]: row
            for row in generated["functions"]
            if row["id"] in PUBLIC_FUNCTIONS
        }

        self.assertEqual(set(PUBLIC_FUNCTIONS), set(generated_functions))
        for function_id, (c_symbol, java_class, java_method, python_name) in PUBLIC_FUNCTIONS.items():
            with self.subTest(function=function_id):
                row = generated_functions[function_id]
                self.assertEqual(c_symbol, row["c"]["symbol"])
                self.assertEqual(java_class, row["java"]["class"])
                self.assertEqual(java_method, row["java"]["method"])
                self.assertEqual(python_name, row["python"]["function"])

    def test_t101_success_response_ttlv_is_inventoried_under_each_operation(self) -> None:
        expected = {
            "KMIPKIT-ELEM-OP-C2S-HASH": "T101:hash:response-ttlv",
            "KMIPKIT-ELEM-OP-C2S-MAC": "T101:mac:response-ttlv",
            "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": "T101:mac_verify:response-ttlv",
            "KMIPKIT-ELEM-OP-C2S-SIGN": "T101:sign:response-ttlv",
            "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": "T101:signature_verify:response-ttlv",
        }
        for operation_id, prefix in expected.items():
            with self.subTest(operation=operation_id):
                operation = self.operations[operation_id]
                for language in ("c", "java", "python"):
                    self.assertEqual(
                        [f"{prefix}:{language}"],
                        [item for item in operation["declarations"][language]
                         if item.startswith("T101:")],
                    )

    def test_mac_verify_and_signature_verify_map_unknown_validity_indicators(self) -> None:
        expected = {
            "kmipkit_protocol::MacVerifyResponse::validity_indicator",
            "kmipkit_protocol::SignatureVerifyResponse::validity_indicator",
        }
        rows = [
            row for row in self.manifest["publicDeclarations"]
            if row["exportName"] in expected
        ]
        self.assertEqual(expected, {row["exportName"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertIn("KMIPKIT-0021-FR-008", row["requirementIds"])
                self.assertIn(
                    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
                    row["parityEvidenceIds"],
                )
                self.assertIn(
                    "bindings/python/tests/test_hash_mac_signature_operations.py",
                    row["parityEvidenceIds"],
                )

    def test_hash_client_request_convenience_method_has_four_language_mapping(self) -> None:
        declaration_id = "rust-observed-7cc46b70a35514bf0bf6"
        declaration = self.declarations[declaration_id]

        self.assertEqual("KMIPKIT-ELEM-OP-C2S-HASH", declaration["capabilityId"])
        self.assertEqual("declared", declaration["declarationState"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(
            "specs/021-hash-mac-signature-operations/spec.md",
            declaration["approvalSource"]["path"],
        )
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::ClientRequest::hash",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_hash",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.hash",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.client_batch_item_hash",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(
            declaration_id,
            self.operations["KMIPKIT-ELEM-OP-C2S-HASH"]["declarations"]["rust"],
        )


if __name__ == "__main__":
    unittest.main()
