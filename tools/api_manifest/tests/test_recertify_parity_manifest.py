"""Manifest regressions for the KMIP 2.1 §6.1.45 Re-certify mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
TRACEABILITY = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.45-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-002",
    "KMIPKIT-REQ-SPEC-6.1.45-004",
    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
    "KMIPKIT-REQ-SPEC-6.1.45-006-002",
    "KMIPKIT-REQ-SPEC-6.1.45-008",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/RecertifyMappingTest.java",
    "bindings/python/tests/test_recertify_operation.py",
    "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
    "tools/api_manifest/tests/test_recertify_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::recertify",
        "kmipkit_client::ClientRequest::Recertify",
        "kmipkit_client::ClientBatchOutcome::recertify_response",
        "kmipkit_protocol::RecertifyRequest",
        "kmipkit_protocol::RecertifyResponse",
        "kmipkit_protocol::RecertifyResponse::unique_identifier",
        "kmipkit_protocol::RecertifyResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_recertify",
        "kmipkit_client_batch_item_response_recertify",
        "kmipkit_client_batch_item_response_recertify_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.recertify",
        "org.kmipkit.extensions.ClientBatchItemResult.recertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_recertify",
        "kmipkit.client.BatchItemResult.recertify_response",
        "kmipkit.client.RecertifyResponse",
        "kmipkit.client.RecertifyResponse.unique_identifier",
        "kmipkit.client.RecertifyResponse.response_ttlv",
        "kmipkit.client.RecertifyResponse.close",
    },
}


class RecertifyParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_recertify_has_approved_four_language_mapping(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Re-certify", operation["name"])
        self.assertEqual("6.1.45", operation["section"])
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

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Re-certify has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertIn(declaration["capabilityId"], {OPERATION_ID, "client_execution"})
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        missing = {
            language: sorted(expected - actual_mappings[language])
            for language, expected in EXPECTED_MAPPINGS.items()
            if expected - actual_mappings[language]
        }
        self.assertEqual({}, missing)
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_requirements)

    def test_recertify_jni_bridge_is_an_exact_approved_exclusion(self) -> None:
        bridge_names = {
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemRecertify",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseRecertify",
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit" and row["exportName"] in bridge_names
        }
        self.assertEqual(bridge_names, set(exclusions))
        for export_name, exclusion in exclusions.items():
            with self.subTest(export=export_name):
                self.assertIn("raw-handle JNI bridge", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("T058", exclusion["evidence"])

    def test_recertify_traceability_links_normative_and_api_requirements(self) -> None:
        rows = [line for line in TRACEABILITY.read_text(encoding="utf-8").splitlines()
                if line.startswith("| T058 |")]
        self.assertEqual(1, len(rows))
        linked_ids = set(re.findall(
            r"KMIPKIT-(?:REQ-SPEC-6\.1\.45-[0-9-]+|0036-FR-\d{3}|0043-FR-\d{3})",
            rows[0],
        ))
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_ids)

    def test_generated_parity_inventory_contains_recertify_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [row for row in generated["operations"] if row["operationId"] == OPERATION_ID]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
