"""Manifest regressions for the T043 Certify operation mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-CERTIFY"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-003",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-005",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-008",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::Certify",
        "kmipkit_client::ClientBatchOutcome::certify_response",
        "kmipkit_protocol::CertifyResponse",
        "kmipkit_protocol::CertifyResponse::unique_identifier",
    },
    "c": {
        "kmipkit_client_batch_item_certify",
        "kmipkit_client_batch_item_response_certify",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.certify",
        "org.kmipkit.extensions.ClientBatchItemResult.CertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.certifyResponse",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_certify",
        "kmipkit.client.CertifyResponse",
        "kmipkit.client.BatchItemResult.certify_response",
    },
}


class CertifyParityManifestTests(unittest.TestCase):
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

    def test_certify_has_approved_four_language_mapping(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Certify", operation["name"])
        self.assertEqual("6.1.6", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("declared", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0036", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Certify has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertIn(declaration["capabilityId"], {OPERATION_ID, "client_execution"})
                self.assertEqual("declared", declaration["declarationState"])
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

    def test_generated_parity_inventory_contains_certify_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row
            for row in generated["operations"]
            if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        operation = operations[0]
        self.assertEqual("declared", operation["apiState"])
        self.assertEqual(LANGUAGES, set(operation["declarations"]))


if __name__ == "__main__":
    unittest.main()
