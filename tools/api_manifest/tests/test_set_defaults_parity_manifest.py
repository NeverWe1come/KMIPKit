"""Require a reviewed four-language Set Defaults mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS"
OWNER_SPEC = "specs/025-policy-controls/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::set_defaults",
        "kmipkit_client::ClientRequest::SetDefaults",
        "kmipkit_client::ClientBatchOutcome::SetDefaults",
        "kmipkit_protocol::SetDefaultsRequest",
        "kmipkit_protocol::SetDefaultsRequest::defaults_information",
        "kmipkit_protocol::SetDefaultsRequest::with_defaults_information",
        "kmipkit_protocol::SetDefaultsResponse",
        "kmipkit_protocol::SetDefaultsResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_set_defaults",
        "kmipkit_client_batch_item_response_set_defaults_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.setDefaults",
        "org.kmipkit.extensions.ClientBatchItemResult.setDefaultsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.SetDefaultsResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.SetDefaultsResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.SetDefaultsResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_set_defaults",
        "kmipkit.client.BatchItemResult.set_defaults_response",
        "kmipkit.client.SetDefaultsResponse",
        "kmipkit.client.SetDefaultsResponse.response_ttlv",
        "kmipkit.client.SetDefaultsResponse.close",
    },
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs",
    "crates/kmipkit-client/tests/unit/set_defaults_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/set_defaults_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/SetDefaultsJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/SetDefaultsResponseMappingTest.java",
    "bindings/python/tests/test_set_defaults_operation.py",
}
REQUIREMENTS = {
    *(f"KMIPKIT-0039-FR-{number:03}" for number in range(1, 7)),
    "KMIPKIT-0039-FR-008",
    *(f"KMIPKIT-0043-FR-{number:03}" for number in range(4, 14)),
    "KMIPKIT-0043-FR-015",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}


class SetDefaultsParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_set_defaults_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Set Defaults", operation["name"])
        self.assertEqual("6.1.53", operation["section"])
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
            self.assertTrue(declaration_ids, f"Set Defaults has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertTrue(REQUIREMENTS <= linked_requirements)


if __name__ == "__main__":
    unittest.main()
