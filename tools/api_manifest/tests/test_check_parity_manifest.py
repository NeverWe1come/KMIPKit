"""Require complete four-language Check mappings before release readiness."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-CHECK"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.7-001",
    "KMIPKIT-REQ-SPEC-6.1.7-002-001",
    "KMIPKIT-REQ-SPEC-6.1.7-002-002",
    "KMIPKIT-REQ-SPEC-6.1.7-003",
    "KMIPKIT-REQ-SPEC-6.1.7-004-001",
    "KMIPKIT-REQ-SPEC-6.1.7-004-002",
    "KMIPKIT-REQ-SPEC-6.1.7-005",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/CheckResponseMappingTest.java",
    "bindings/python/tests/test_check_operation.py",
    "crates/kmipkit-client/tests/unit/check_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/check_operation_tests.rs",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::check",
        "kmipkit_client::ClientBatchOutcome::check_response",
        "kmipkit_protocol::CheckRequest",
        "kmipkit_protocol::CheckResponse",
        "kmipkit_protocol::CheckResponse::unique_identifier",
        "kmipkit_protocol::CheckResponse::usage_limits_count",
        "kmipkit_protocol::CheckResponse::cryptographic_usage_mask",
        "kmipkit_protocol::CheckResponse::lease_time",
        "kmipkit_protocol::CheckResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_check",
        "kmipkit_client_batch_item_response_check",
        "kmipkit_client_batch_item_response_check_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.check",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.checkResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.usageLimitsCount",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.cryptographicUsageMask",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.leaseTime",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.CheckResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_check",
        "kmipkit.client.BatchItemResult.check_response",
        "kmipkit.client.CheckResponse",
        "kmipkit.client.CheckResponse.unique_identifier",
        "kmipkit.client.CheckResponse.usage_limits_count",
        "kmipkit.client.CheckResponse.cryptographic_usage_mask",
        "kmipkit.client.CheckResponse.lease_time",
        "kmipkit.client.CheckResponse.response_ttlv",
        "kmipkit.client.CheckResponse.close",
    },
}


class CheckParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row
            for row in self.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_check_has_approved_four_language_mapping_and_owner_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Check", operation["name"])
        self.assertEqual("6.1.7", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Check has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertIn(declaration["capabilityId"], {OPERATION_ID, "client_execution"})
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(
                    declaration["languageMappings"][language]["name"]
                )
                linked_requirements.update(declaration["requirementIds"])

        missing = {
            language: sorted(expected - actual_mappings[language])
            for language, expected in EXPECTED_MAPPINGS.items()
            if expected - actual_mappings[language]
        }
        self.assertEqual({}, missing)
        self.assertTrue(OWNER_REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_check_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
