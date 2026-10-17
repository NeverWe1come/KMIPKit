"""Require complete four-language Export mappings before release readiness."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-EXPORT"
OWNER_SPEC = "specs/023-key-material-transfer/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-0037-FR-001",
    "KMIPKIT-0037-FR-002",
    "KMIPKIT-0037-FR-003",
    "KMIPKIT-0037-FR-004",
    "KMIPKIT-0037-FR-005",
    "KMIPKIT-0037-FR-006",
    "KMIPKIT-0037-FR-007",
    "KMIPKIT-0037-FR-008",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ExportResponseMappingTest.java",
    "bindings/python/tests/test_export_operation.py",
    "crates/kmipkit-client/tests/unit/export_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/export_operation_tests.rs",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::export",
        "kmipkit_client::ClientRequest::Export",
        "kmipkit_client::ClientBatchOutcome::Export",
        "kmipkit_protocol::ExportRequest",
        "kmipkit_protocol::ExportResponse",
        "kmipkit_protocol::ExportResponse::object_type",
        "kmipkit_protocol::ExportResponse::unique_identifier",
        "kmipkit_protocol::ExportResponse::attributes",
        "kmipkit_protocol::ExportResponse::object",
        "kmipkit_protocol::ExportResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_export",
        "kmipkit_client_batch_item_response_export",
        "kmipkit_client_batch_item_response_export_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.export",
        "org.kmipkit.extensions.ClientBatchItemResult.exportResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.objectTypeRaw",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.attributes",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.anyObject",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.ExportResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_export",
        "kmipkit.client.BatchItemResult.export_response",
        "kmipkit.client.ExportResponse",
        "kmipkit.client.ExportResponse.object_type_raw",
        "kmipkit.client.ExportResponse.unique_identifier",
        "kmipkit.client.ExportResponse.attributes",
        "kmipkit.client.ExportResponse.any_object",
        "kmipkit.client.ExportResponse.response_ttlv",
        "kmipkit.client.ExportResponse.close",
    },
}


class ExportParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_export_has_approved_four_language_mapping_and_owner_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Export", operation["name"])
        self.assertEqual("6.1.18", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0037", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
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
            self.assertTrue(declaration_ids, f"Export has no {language} declarations")
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
        self.assertTrue(OWNER_REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_export_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
