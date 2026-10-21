"""Manifest acceptance tests for OASIS KMIP 2.1 §6.1.44 Revoke mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-REVOKE"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.44-001",
    "KMIPKIT-REQ-SPEC-6.1.44-003-001",
    "KMIPKIT-REQ-SPEC-6.1.44-003-002",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/RevokeMappingTest.java",
    "bindings/python/tests/test_revoke_operation.py",
    "crates/kmipkit-client/tests/unit/revoke_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs",
    "tools/api_manifest/tests/test_revoke_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::revoke",
        "kmipkit_client::ClientRequest::Revoke",
        "kmipkit_client::ClientBatchOutcome::revoke_response",
        "kmipkit_protocol::RevocationReason",
        "kmipkit_protocol::RevokeRequest",
        "kmipkit_protocol::RevokeResponse",
        "kmipkit_protocol::RevokeResponse::unique_identifier",
        "kmipkit_protocol::RevokeResponse::with_ttlv",
    },
    "c": {
        "kmipkit_revocation_reason_create",
        "kmipkit_revoke_request_create",
        "kmipkit_client_batch_item_revoke",
        "kmipkit_client_batch_item_response_revoke",
        "kmipkit_client_batch_item_response_revoke_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.RevocationReason.create",
        "org.kmipkit.extensions.RevocationReason.rawCode",
        "org.kmipkit.extensions.RevocationReason.message",
        "org.kmipkit.extensions.RevokeRequest.create",
        "org.kmipkit.extensions.ClientBatchItem.revoke",
        "org.kmipkit.extensions.ClientBatchItemResult.revokeResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RevokeResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.RevokeResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.RevokeResponse.close",
    },
    "python": {
        "kmipkit.extensions.RevocationReason.create",
        "kmipkit.extensions.RevokeRequest.create",
        "kmipkit.extensions.client_batch_item_revoke",
        "kmipkit.client.BatchItemResult.revoke_response",
        "kmipkit.client.RevokeResponse",
        "kmipkit.client.RevokeResponse.unique_identifier",
        "kmipkit.client.RevokeResponse.response_ttlv",
        "kmipkit.client.RevokeResponse.close",
    },
}


class RevokeParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_revoke_has_approved_four_language_mapping_and_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Revoke", operation["name"])
        self.assertEqual("6.1.44", operation["section"])
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
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Revoke has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                if declaration.get("capabilityId") == OPERATION_ID:
                    self.assertTrue(
                        PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"])
                    )
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

    def test_revoke_traceability_names_the_nested_reason_contract(self) -> None:
        traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        rows = [line for line in traceability.splitlines() if line.startswith("| T063 |")]
        self.assertEqual(1, len(rows))
        row = rows[0]
        self.assertIn("§6.1.44, Tables 295–297", row)
        self.assertIn("§4.47, Table 123", row)
        self.assertIn("§11.48, Table 481", row)
        self.assertIn("KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-MESSAGE", row)
        self.assertIn("request_encodes_optional_audit_message_inside_revocation_reason_structure", row)


if __name__ == "__main__":
    unittest.main()
