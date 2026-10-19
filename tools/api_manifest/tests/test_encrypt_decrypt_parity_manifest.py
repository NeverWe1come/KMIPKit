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
        "normative_requirements": {
            "KMIPKIT-REQ-SPEC-4.16-001-001",
            "KMIPKIT-REQ-SPEC-4.16-001-002",
            "KMIPKIT-REQ-SPEC-4.16-002",
            "KMIPKIT-REQ-SPEC-4.16-003",
            "KMIPKIT-REQ-SPEC-6.1.17-001-001",
            "KMIPKIT-REQ-SPEC-6.1.17-001-002",
            "KMIPKIT-REQ-SPEC-6.1.17-005-001",
            "KMIPKIT-REQ-SPEC-6.1.17-005-002",
            "KMIPKIT-REQ-SPEC-6.1.17-006",
        },
        "request_requirements": {
            "KMIPKIT-REQ-SPEC-4.16-001-001",
            "KMIPKIT-REQ-SPEC-4.16-001-002",
            "KMIPKIT-REQ-SPEC-4.16-002",
            "KMIPKIT-REQ-SPEC-4.16-003",
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
        "normative_requirements": {
            "KMIPKIT-REQ-SPEC-4.16-001-001",
            "KMIPKIT-REQ-SPEC-4.16-001-002",
            "KMIPKIT-REQ-SPEC-4.16-002",
            "KMIPKIT-REQ-SPEC-4.16-003",
            "KMIPKIT-REQ-SPEC-6.1.11-001-001",
            "KMIPKIT-REQ-SPEC-6.1.11-001-002",
            "KMIPKIT-REQ-SPEC-6.1.11-004-001",
            "KMIPKIT-REQ-SPEC-6.1.11-004-002",
            "KMIPKIT-REQ-SPEC-6.1.11-005",
            "KMIPKIT-REQ-SPEC-6.1.11-006",
        },
        "request_requirements": {
            "KMIPKIT-REQ-SPEC-4.16-001-001",
            "KMIPKIT-REQ-SPEC-4.16-001-002",
            "KMIPKIT-REQ-SPEC-4.16-002",
            "KMIPKIT-REQ-SPEC-4.16-003",
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

                linked_requirements: set[str] = set()
                operation_name = expected["name"].lower()
                for endpoint in ("request", "response"):
                    function_id = (
                        f"client_batch_item_{operation_name}"
                        if endpoint == "request"
                        else f"client_batch_item_response_{operation_name}"
                    )
                    function = self.functions[function_id]
                    actual_mappings = {
                        "rust": f"{function['rust']['module']}::{function['rust']['name']}",
                        "c": function["c"]["symbol"],
                        "java": (
                            f"org.kmipkit.{function['java']['package'].split('.')[-1]}"
                            f".{function['java']['class']}.{function['java']['method']}"
                        ),
                        "python": function["python"]["function"],
                    }
                    self.assertEqual(expected[endpoint], actual_mappings)
                    for language in LANGUAGES:
                        declaration_ids = operation["declarations"][language]
                        self.assertEqual(2, len(declaration_ids), language)
                        declaration = next(
                            row for row in self.manifest["publicDeclarations"]
                            if row["declarationId"] == declaration_ids[0 if endpoint == "request" else 1]
                        )
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                        self.assertEqual(expected[endpoint][language],
                                         declaration["languageMappings"][language]["name"])
                        self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                        self.assertEqual("Approved", declaration["approvalSource"]["status"])
                        linked_requirements.update(declaration["requirementIds"])
                        if endpoint == "request":
                            self.assertTrue(
                                expected["request_requirements"]
                                <= set(declaration["requirementIds"]),
                                f"request declaration omits shared §4.16 requirements: {language}",
                            )
                    self.assertTrue(
                        any(req.startswith("KMIPKIT-0019-FR-") for req in function["requirementIds"])
                    )
                    self.assertTrue(
                        any(req.startswith("KMIPKIT-0043-FR-") for req in function["requirementIds"])
                    )
                self.assertTrue(expected["normative_requirements"] <= linked_requirements)

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

    def test_typed_response_close_is_traced_as_secret_buffer_wipe(self) -> None:
        declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        expected_lifecycle = {
            "rust": "wipe-on-final-owner-drop",
            "c": "release-shared-reference-wipe-on-final-owner-drop",
            "java": "wipe-owned-copy-on-close",
            "python": "wipe-owned-copy-on-close",
        }
        for operation, section in (("Encrypt", "6.1.17"), ("Decrypt", "6.1.11")):
            operation_id = f"KMIPKIT-ELEM-OP-C2S-{operation.upper()}"
            with self.subTest(operation=operation):
                for language in ("java", "python"):
                    declaration_id = f"T040:{operation.lower()}:response-close:{language}"
                    declaration = declarations[declaration_id]
                    self.assertEqual("verified", declaration["declarationState"])
                    self.assertEqual("secret-buffer-wipe", declaration["mappingSemantics"])
                    self.assertEqual(
                        expected_lifecycle,
                        declaration["lifecycleSemanticsByLanguage"],
                    )
                    self.assertEqual(operation_id, declaration["capabilityId"])
                    self.assertIn("KMIPKIT-0019-FR-009", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0043-FR-013", declaration["requirementIds"])
                    self.assertEqual(section, self.operations[operation_id]["section"])
                    self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                    expected_names = {
                        "rust": "kmipkit_client::ClientBatchItemResponse",
                        "c": "kmipkit_client_batch_item_response_release",
                        "java": (
                            "org.kmipkit.extensions.ClientBatchItemResult."
                            f"{operation}Response.close"
                        ),
                        "python": f"kmipkit.client.{operation}Response.close",
                    }
                    self.assertEqual(
                        expected_names,
                        {
                            language: mapping["name"]
                            for language, mapping in declaration["languageMappings"].items()
                        },
                    )
                self.assertIn(
                    f"org.kmipkit.extensions.ClientBatchItemResult.{operation}Response.close",
                    declarations[f"T040:{operation.lower()}:response-close:java"]["exportName"],
                )

    def test_python_typed_response_types_and_fields_are_declared(self) -> None:
        declarations_by_export = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        for operation, fields in (
            (
                "Encrypt",
                (
                    "unique_identifier",
                    "data",
                    "iv_counter_nonce",
                    "correlation_value",
                    "authenticated_encryption_tag",
                ),
            ),
            ("Decrypt", ("unique_identifier", "data", "correlation_value")),
        ):
            type_name = f"kmipkit.client.{operation}Response"
            with self.subTest(response=operation):
                self.assertEqual("type", declarations_by_export[type_name]["kind"])
                self.assertEqual(
                    "verified",
                    declarations_by_export[type_name]["declarationState"],
                )
                for field_name in fields:
                    declaration_name = f"{type_name}.{field_name}"
                    self.assertEqual(
                        "python-export", declarations_by_export[declaration_name]["kind"]
                    )
                    self.assertEqual(
                        "verified",
                        declarations_by_export[declaration_name]["declarationState"],
                    )
                self.assertEqual(
                    f"kmipkit.client.{operation}Response.close",
                    declarations_by_id[
                        f"T040:{operation.lower()}:response-close:python"
                    ]["exportName"],
                )


if __name__ == "__main__":
    unittest.main()
