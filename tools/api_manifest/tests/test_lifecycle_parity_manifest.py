"""Manifest acceptance tests for T039 managed-object lifecycle mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OWNER_SPEC = "specs/018-managed-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-ACTIVATE": {
        "name": "Activate",
        "section": "6.1.1",
        "rust": {
            "kmipkit_client::Client::activate",
            "kmipkit_client::ClientRequest::Activate",
            "kmipkit_client::ClientBatchOutcome::activate_response",
            "kmipkit_protocol::ActivateRequest",
            "kmipkit_protocol::ActivateRequest::new",
            "kmipkit_protocol::ActivateResponse",
            "kmipkit_protocol::ActivateResponse::result",
            "kmipkit_protocol::ActivateResponse::unique_identifier",
        },
        "c": {
            "kmipkit_client_batch_item_activate",
            "kmipkit_client_batch_item_response_activate",
        },
        "java": {
            "org.kmipkit.extensions.ClientBatchItem.activate",
            "org.kmipkit.extensions.ClientBatchItemResult.activateResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.ActivateResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.ActivateResponse.uniqueIdentifier",
        },
        "python": {
            "kmipkit.extensions.client_batch_item_activate",
            "kmipkit.client.BatchItemResult.activate_response",
            "kmipkit.client.ActivateResponse",
            "kmipkit.client.ActivateResponse.unique_identifier",
        },
    },
    "KMIPKIT-ELEM-OP-C2S-ARCHIVE": {
        "name": "Archive",
        "section": "6.1.4",
        "rust": {
            "kmipkit_client::Client::archive",
            "kmipkit_client::ClientRequest::Archive",
            "kmipkit_client::ClientBatchOutcome::archive_response",
            "kmipkit_protocol::ArchiveRequest",
            "kmipkit_protocol::ArchiveRequest::new",
            "kmipkit_protocol::ArchiveResponse",
            "kmipkit_protocol::ArchiveResponse::result",
            "kmipkit_protocol::ArchiveResponse::unique_identifier",
        },
        "c": {
            "kmipkit_client_batch_item_archive",
            "kmipkit_client_batch_item_response_archive",
        },
        "java": {
            "org.kmipkit.extensions.ClientBatchItem.archive",
            "org.kmipkit.extensions.ClientBatchItemResult.archiveResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.ArchiveResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.ArchiveResponse.uniqueIdentifier",
        },
        "python": {
            "kmipkit.extensions.client_batch_item_archive",
            "kmipkit.client.BatchItemResult.archive_response",
            "kmipkit.client.ArchiveResponse",
            "kmipkit.client.ArchiveResponse.unique_identifier",
        },
    },
    "KMIPKIT-ELEM-OP-C2S-DESTROY": {
        "name": "Destroy",
        "section": "6.1.15",
        "rust": {
            "kmipkit_client::Client::destroy",
            "kmipkit_client::ClientRequest::Destroy",
            "kmipkit_client::ClientBatchOutcome::destroy_response",
            "kmipkit_protocol::DestroyRequest",
            "kmipkit_protocol::DestroyRequest::new",
            "kmipkit_protocol::DestroyResponse",
            "kmipkit_protocol::DestroyResponse::result",
            "kmipkit_protocol::DestroyResponse::unique_identifier",
        },
        "c": {
            "kmipkit_client_batch_item_destroy",
            "kmipkit_client_batch_item_response_destroy",
        },
        "java": {
            "org.kmipkit.extensions.ClientBatchItem.destroy",
            "org.kmipkit.extensions.ClientBatchItemResult.destroyResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.DestroyResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.DestroyResponse.uniqueIdentifier",
        },
        "python": {
            "kmipkit.extensions.client_batch_item_destroy",
            "kmipkit.client.BatchItemResult.destroy_response",
            "kmipkit.client.DestroyResponse",
            "kmipkit.client.DestroyResponse.unique_identifier",
        },
    },
    "KMIPKIT-ELEM-OP-C2S-RECOVER": {
        "name": "Recover",
        "section": "6.1.42",
        "rust": {
            "kmipkit_client::Client::recover",
            "kmipkit_client::ClientRequest::Recover",
            "kmipkit_client::ClientBatchOutcome::recover_response",
            "kmipkit_protocol::RecoverRequest",
            "kmipkit_protocol::RecoverRequest::new",
            "kmipkit_protocol::RecoverResponse",
            "kmipkit_protocol::RecoverResponse::result",
            "kmipkit_protocol::RecoverResponse::unique_identifier",
        },
        "c": {
            "kmipkit_client_batch_item_recover",
            "kmipkit_client_batch_item_response_recover",
        },
        "java": {
            "org.kmipkit.extensions.ClientBatchItem.recover",
            "org.kmipkit.extensions.ClientBatchItemResult.recoverResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.RecoverResponse",
            "org.kmipkit.extensions.ClientBatchItemResult.RecoverResponse.uniqueIdentifier",
        },
        "python": {
            "kmipkit.extensions.client_batch_item_recover",
            "kmipkit.client.BatchItemResult.recover_response",
            "kmipkit.client.RecoverResponse",
            "kmipkit.client.RecoverResponse.unique_identifier",
        },
    },
}
OWNER_REQUIREMENTS = {
    f"KMIPKIT-0018-FR-{index:03}"
    for index in range(1, 11)
} | {
    "KMIPKIT-REQ-SPEC-6.1.4-001",
    "KMIPKIT-REQ-SPEC-6.1.42-001-001",
    "KMIPKIT-REQ-SPEC-6.1.42-001-002",
}
API_REQUIREMENTS = {
    "KMIPKIT-0043-FR-003",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-012",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/LifecycleOperationMappingTest.java",
    "bindings/python/tests/test_lifecycle_operations.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-client/tests/unit/activate_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/archive_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/destroy_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/recover_execution_tests.rs",
    "tools/api_manifest/tests/test_lifecycle_parity_manifest.py",
}


class LifecycleParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = {
            row["operationId"]: row
            for row in self.manifest["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_lifecycle_operations_have_approved_four_language_mappings(self) -> None:
        self.assertEqual(set(OPERATIONS), set(self.operations))
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                operation = self.operations[operation_id]
                self.assertEqual(expected["name"], operation["name"])
                self.assertEqual(expected["section"], operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0018", operation["featureSpecId"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("KMIPKIT-0018", operation["ownerDependency"]["specId"])
                approval = operation["approvalEvidence"]
                self.assertEqual(OWNER_SPEC, approval["path"])
                self.assertEqual("Approved", approval["status"])
                self.assertIn("Status**: Draft", approval["evidence"])
                approval_record = (
                    ROOT / "specs/043-public-api-bindings/implementation-evidence.md"
                ).read_text(encoding="utf-8")
                self.assertIn("PR #65", approval_record)
                self.assertIn("Draft for independent human review", approval["evidence"])
                self.assertEqual(LANGUAGES, set(operation["declarations"]))
                self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
                for evidence in PARITY_EVIDENCE:
                    self.assertTrue((ROOT / evidence).is_file(), evidence)

                actual = {language: set() for language in LANGUAGES}
                linked_requirements: set[str] = set()
                for language, declaration_ids in operation["declarations"].items():
                    self.assertTrue(declaration_ids, f"{expected['name']} has no {language} declarations")
                    for declaration_id in declaration_ids:
                        declaration = self.declarations[declaration_id]
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                        self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                        self.assertEqual("Approved", declaration["approvalSource"]["status"])
                        self.assertIn(
                            "Status**: Draft", declaration["approvalSource"]["evidence"]
                        )
                        actual[language].add(
                            declaration["languageMappings"][language]["name"]
                        )
                        linked_requirements.update(declaration["requirementIds"])

                for language in LANGUAGES:
                    self.assertTrue(expected[language] <= actual[language], language)
                self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_lifecycle_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = {
            row["operationId"]: row
            for row in generated["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.assertEqual(set(OPERATIONS), set(operations))
        for operation in operations.values():
            self.assertEqual("verified", operation["apiState"])
            self.assertEqual(LANGUAGES, set(operation["declarations"]))


if __name__ == "__main__":
    unittest.main()
