"""Manifest acceptance tests for OASIS KMIP 2.1 §6.1.43 Register mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-REGISTER"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.43-001",
    "KMIPKIT-REQ-SPEC-6.1.43-005",
    "KMIPKIT-REQ-SPEC-6.1.43-007",
    "KMIPKIT-REQ-SPEC-6.1.43-009-001",
    "KMIPKIT-REQ-SPEC-6.1.43-009-002",
    "KMIPKIT-REQ-SPEC-6.1.43-010-001",
    "KMIPKIT-REQ-SPEC-6.1.43-010-002",
    "KMIPKIT-REQ-SPEC-6.1.43-011",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/RegisterMappingTest.java",
    "bindings/python/tests/test_register_operation.py",
    "crates/kmipkit-client/tests/unit/register_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs",
    "tools/api_manifest/tests/test_register_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::register",
        "kmipkit_client::ClientRequest::Register",
        "kmipkit_client::ClientBatchOutcome::register_response",
        "kmipkit_protocol::RegisterRequest",
        "kmipkit_protocol::RegisterResponse",
        "kmipkit_protocol::RegisterResponse::unique_identifier",
        "kmipkit_protocol::RegisterResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_register",
        "kmipkit_client_batch_item_response_register",
        "kmipkit_client_batch_item_response_register_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.register",
        "org.kmipkit.extensions.ClientBatchItemResult.registerResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RegisterResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RegisterResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.RegisterResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.RegisterResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_register",
        "kmipkit.client.BatchItemResult.register_response",
        "kmipkit.client.RegisterResponse",
        "kmipkit.client.RegisterResponse.unique_identifier",
        "kmipkit.client.RegisterResponse.response_ttlv",
        "kmipkit.client.RegisterResponse.close",
    },
}


class RegisterParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_register_has_approved_four_language_mapping_and_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Register", operation["name"])
        self.assertEqual("6.1.43", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0036", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        linked_ids = {item for ids in operation["declarations"].values() for item in ids}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Register has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                if declaration["capabilityId"] == OPERATION_ID:
                    self.assertTrue(PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                    self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                    self.assertEqual("Approved", declaration["approvalSource"]["status"])
                mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        missing = {
            language: sorted(expected - mappings[language])
            for language, expected in EXPECTED_MAPPINGS.items()
            if expected - mappings[language]
        }
        self.assertEqual({}, missing)
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_requirements)
        self.assertTrue(linked_ids)


if __name__ == "__main__":
    unittest.main()
