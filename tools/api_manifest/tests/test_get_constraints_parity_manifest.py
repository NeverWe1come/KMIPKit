"""Require an approved four-language Get Constraints mapping before release readiness."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS"
OWNER_SPEC = "specs/025-policy-controls/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::get_constraints",
        "kmipkit_client::ClientRequest::GetConstraints",
        "kmipkit_client::ClientBatchOutcome::GetConstraints",
        "kmipkit_protocol::GetConstraintsRequest",
        "kmipkit_protocol::GetConstraintsResponse",
        "kmipkit_protocol::GetConstraintsResponse::constraints",
        "kmipkit_protocol::GetConstraintsResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_get_constraints",
        "kmipkit_client_batch_item_response_get_constraints_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.getConstraints",
        "org.kmipkit.extensions.ClientBatchItemResult.getConstraintsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetConstraintsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetConstraintsResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.GetConstraintsResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_get_constraints",
        "kmipkit.client.BatchItemResult.get_constraints_response",
        "kmipkit.client.GetConstraintsResponse",
        "kmipkit.client.GetConstraintsResponse.response_ttlv",
        "kmipkit.client.GetConstraintsResponse.close",
    },
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/get_constraints_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/get_constraints_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/GetConstraintsJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetConstraintsResponseMappingTest.java",
    "bindings/python/tests/test_get_constraints_operation.py",
}


class GetConstraintsParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_get_constraints_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Get Constraints", operation["name"])
        self.assertEqual("6.1.22", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0039", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Get Constraints has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(
                    declaration["languageMappings"][language]["name"]
                )
                linked_requirements.update(declaration["requirementIds"])

        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertTrue({"KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-002", "KMIPKIT-0039-FR-004"} <= linked_requirements)
        self.assertTrue({"KMIPKIT-0043-FR-004", "KMIPKIT-0043-FR-005", "KMIPKIT-0043-FR-011", "KMIPKIT-0043-FR-021"} <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
