"""Manifest regressions for the T049 Get Usage Allocation mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION"
OWNER_SPEC = "specs/025-policy-controls/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::GetUsageAllocation",
        "kmipkit_client::ClientBatchOutcome::GetUsageAllocation",
        "kmipkit_protocol::GetUsageAllocationRequest",
        "kmipkit_protocol::GetUsageAllocationRequest::new",
        "kmipkit_protocol::GetUsageAllocationRequest::usage_limits_count",
        "kmipkit_protocol::GetUsageAllocationResponse",
        "kmipkit::GetUsageAllocationResponse::with_ttlv",
        "kmipkit_protocol::GetUsageAllocationResponse::with_ttlv",
        "kmipkit_protocol::GetUsageAllocationResponse::unique_identifier",
    },
    "c": {
        "kmipkit_client_batch_item_get_usage_allocation",
        "kmipkit_client_batch_item_response_get_usage_allocation",
        "kmipkit_client_batch_item_response_get_usage_allocation_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.getUsageAllocation",
        "org.kmipkit.extensions.ClientBatchItemResult.getUsageAllocationResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetUsageAllocationResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetUsageAllocationResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.GetUsageAllocationResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.GetUsageAllocationResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_get_usage_allocation",
        "kmipkit.client.BatchItemResult.get_usage_allocation_response",
        "kmipkit.client.GetUsageAllocationResponse",
        "kmipkit.client.GetUsageAllocationResponse.unique_identifier",
        "kmipkit.client.GetUsageAllocationResponse.response_ttlv",
        "kmipkit.client.GetUsageAllocationResponse.close",
    },
}
REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.23-002-001",
    "KMIPKIT-0039-FR-001",
    "KMIPKIT-0039-FR-002",
    "KMIPKIT-0039-FR-003",
    "KMIPKIT-0039-FR-004",
    "KMIPKIT-0039-FR-005",
    "KMIPKIT-0039-FR-006",
    "KMIPKIT-0039-FR-008",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-012",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-015",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/get_usage_allocation_abi_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/GetUsageAllocationJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetUsageAllocationResponseMappingTest.java",
    "bindings/python/tests/test_get_usage_allocation_operation.py",
}
RESPONSE_TTLV_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetUsageAllocationResponseMappingTest.java",
    "bindings/python/tests/test_get_usage_allocation_operation.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs",
}


class GetUsageAllocationParityManifestTests(unittest.TestCase):
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

    def test_get_usage_allocation_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Get Usage Allocation", operation["name"])
        self.assertEqual("6.1.23", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0039", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0039", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Get Usage Allocation has no {language} declarations")
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
        self.assertTrue(REQUIREMENTS <= linked_requirements)
        self.assertNotIn("KMIPKIT-0039-FR-009", linked_requirements)

    def test_generated_parity_inventory_contains_verified_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row
            for row in generated["operations"]
            if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_complete_success_response_ttlv_has_owned_four_language_mapping(self) -> None:
        function = next(
            row for row in self.manifest["functions"]
            if row["id"] == "client_batch_item_response_get_usage_allocation_ttlv"
        )
        self.assertEqual("kmipkit_client_batch_item_response_get_usage_allocation_ttlv", function["c"]["symbol"])
        self.assertEqual("ffi", function["rust"]["owner"])
        self.assertEqual(
            "kmipkit_ffi::extension_registry::client_abi", function["rust"]["module"]
        )
        self.assertEqual(
            "kmipkit_client_batch_item_response_get_usage_allocation_ttlv",
            function["rust"]["name"],
        )
        self.assertEqual(
            "*mut *mut kmipkit_ttlv_structure_t",
            function["rust"]["parameters"][2]["type"],
        )
        self.assertEqual("getUsageAllocationResponse", function["java"]["method"])
        self.assertEqual(
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseGetUsageAllocation",
            function["java"]["jniSymbol"],
        )
        self.assertEqual("kmipkit_ttlv_structure_t **", function["c"]["parameters"][2]["type"])
        self.assertEqual("library-owned", function["c"]["parameters"][2]["ownership"])
        self.assertTrue(function["c"]["parameters"][2]["nullable"])
        self.assertTrue(any("only for a successful Get Usage Allocation result" in line for line in function["c"]["documentation"]))

        for declaration_id in (
            "T049:result:responseTtlv:c",
            "T049:result:responseTtlv:java",
            "T049:result:responseTtlv:javaClose",
            "T049:result:responseTtlv:python",
            "T049:result:responseTtlv:pythonClose",
            "rust-observed-ac6bfe0a5b7ff3e02a9a",
            "rust-observed-13dd78ed4ea678db8508",
        ):
            declaration = self.declarations[declaration_id]
            self.assertEqual("verified", declaration["declarationState"])
            self.assertIn("KMIPKIT-0039-FR-004", declaration["requirementIds"])
            self.assertIn("KMIPKIT-0043-FR-011", declaration["requirementIds"])
            self.assertTrue(
                RESPONSE_TTLV_EVIDENCE <= set(declaration["parityEvidenceIds"])
            )


if __name__ == "__main__":
    unittest.main()
