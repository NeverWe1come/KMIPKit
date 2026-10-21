"""Require the approved KMIP v2.1 §6.1.57 Validate mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-VALIDATE"
OWNER_SPEC = "specs/028-diagnostics-interop/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
TRACEABILITY_IDS = {
    "KMIPKIT-REQ-SPEC-6.1.57-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002",
    "KMIPKIT-REQ-SPEC-6.1.57-004",
    "KMIPKIT-REQ-SPEC-6.1.57-005",
    "KMIPKIT-REQ-SPEC-6.1.57-006",
}
REQUIREMENTS = {
    *(f"KMIPKIT-0042-FR-{index:03}" for index in range(1, 9)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)),
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::validate",
        "kmipkit_client::ClientRequest::Validate",
        "kmipkit_client::ClientBatchOutcome::Validate",
        "kmipkit_protocol::ValidateRequest",
        "kmipkit_protocol::ValidateRequest::with_certificate",
        "kmipkit_protocol::ValidateRequest::with_unique_identifier",
        "kmipkit_protocol::ValidateRequest::with_validity_date",
        "kmipkit_protocol::ValidateResponse",
        "kmipkit_protocol::ValidateResponse::validity_indicator",
        "kmipkit_protocol::ValidateResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_validate",
        "kmipkit_client_batch_item_response_validate",
        "kmipkit_client_batch_item_response_validate_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.validate",
        "org.kmipkit.extensions.ClientBatchItemResult.validateResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse.validityIndicatorRaw",
        "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_validate",
        "kmipkit.client.BatchItemResult.validate_response",
        "kmipkit.client.ValidateResponse",
        "kmipkit.client.ValidateResponse.validity_indicator_raw",
        "kmipkit.client.ValidateResponse.response_ttlv",
        "kmipkit.client.ValidateResponse.close",
    },
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/validate_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/validate_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ValidateJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/ValidateResponseMappingTest.java",
    "bindings/python/tests/test_validate_operation.py",
    "tools/api_manifest/tests/test_validate_parity_manifest.py",
}


class ValidateParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_validate_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Validate", operation["name"])
        self.assertEqual("6.1.57", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0042", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0042", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Validate has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertTrue(TRACEABILITY_IDS <= linked_requirements)
        self.assertTrue(REQUIREMENTS <= linked_requirements)

    def test_validate_traceability_requirements_are_registered_at_manifest_root(self) -> None:
        self.assertTrue(TRACEABILITY_IDS <= set(self.manifest["requirementIds"]))


if __name__ == "__main__":
    unittest.main()
