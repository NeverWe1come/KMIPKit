"""Require the approved KMIP v2.1 §6.1.27 Join Split Key mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY"
OWNER_SPEC = "specs/023-key-material-transfer/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.27-001",
    "KMIPKIT-REQ-SPEC-6.1.27-002-001",
    "KMIPKIT-REQ-SPEC-6.1.27-002-002",
    "KMIPKIT-REQ-SPEC-6.1.27-005",
    *(f"KMIPKIT-0037-FR-{index:03}" for index in range(1, 9)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)),
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/join_split_key_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/JoinSplitKeyMappingTest.java",
    "bindings/python/tests/test_join_split_key_operation.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::join_split_key",
        "kmipkit_client::ClientRequest::JoinSplitKey",
        "kmipkit_client::ClientBatchOutcome::JoinSplitKeyCompleted",
        "kmipkit_client::ClientBatchOutcome::join_split_key_response",
        "kmipkit_protocol::JoinSplitKeyRequest",
        "kmipkit_protocol::JoinSplitKeyRequest::new",
        "kmipkit_protocol::SplitKeyReference",
        "kmipkit_protocol::SplitKeyReference::new",
        "kmipkit_protocol::SplitKeyReference::with_split_key_threshold",
        "kmipkit_protocol::JoinSplitKeyResponse",
        "kmipkit_protocol::JoinSplitKeyResponse::unique_identifier",
        "kmipkit_protocol::JoinSplitKeyResponse::with_ttlv",
    },
    "c": {
        "kmipkit_split_key_reference_create",
        "kmipkit_client_batch_item_join_split_key",
        "kmipkit_client_batch_item_response_join_split_key",
        "kmipkit_client_batch_item_response_join_split_key_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.SplitKeyReference",
        "org.kmipkit.extensions.ClientBatchItem.joinSplitKey",
        "org.kmipkit.extensions.ClientBatchItemResult.joinSplitKeyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.JoinSplitKeyResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.JoinSplitKeyResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.JoinSplitKeyResponse.close",
    },
    "python": {
        "kmipkit.extensions.SplitKeyReference",
        "kmipkit.extensions.client_batch_item_join_split_key",
        "kmipkit.client.BatchItemResult.join_split_key_response",
        "kmipkit.client.JoinSplitKeyResponse.unique_identifier",
        "kmipkit.client.JoinSplitKeyResponse.response_ttlv",
        "kmipkit.client.JoinSplitKeyResponse.close",
    },
}


class JoinSplitKeyParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_join_split_key_is_verified_against_approved_owner_and_section(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Join Split Key", operation["name"])
        self.assertEqual("6.1.27", operation["section"])
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
            self.assertTrue(declaration_ids, f"Join Split Key has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertEqual(REQUIREMENTS, linked_requirements)

    def test_generated_parity_inventory_contains_verified_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_foreign_reference_record_pairs_identifier_and_threshold(self) -> None:
        reference = next(
            (row for row in self.manifest["types"] if row["id"] == "SplitKeyReference"),
            None,
        )
        self.assertIsNotNone(reference, "the manifest must define the paired split-key reference")
        assert reference is not None
        self.assertEqual("opaque-handle", reference["c"]["kind"])
        self.assertEqual("kmipkit_split_key_reference_t", reference["c"]["name"])
        self.assertTrue(reference["c"]["releaseFunction"].endswith("_release"))
        self.assertEqual("SplitKeyReference", reference["java"]["name"])
        self.assertEqual("SplitKeyReference", reference["python"]["name"])

    def test_java_t052_inventory_records_overloads_kinds_and_exact_exclusions(self) -> None:
        request_factory = self.declarations["T052:request:constructor:java"]
        self.assertEqual(
            [
                "public static org.kmipkit.extensions.ClientBatchItem joinSplitKey(long, "
                "java.util.List<org.kmipkit.extensions.SplitKeyReference>)",
                "public static org.kmipkit.extensions.ClientBatchItem joinSplitKey(long, "
                "java.util.List<org.kmipkit.extensions.SplitKeyReference>, java.lang.Long, "
                "org.kmipkit.ttlv.TtlvStructure, org.kmipkit.ttlv.TtlvStructure)",
            ],
            request_factory["javaSignatures"],
        )
        self.assertEqual(
            "java-member", self.declarations["T052:response:ttlv:java"]["kind"]
        )
        self.assertEqual(
            "java-member", self.declarations["T052:response:unique-identifier:java"]["kind"]
        )

        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit"
        }
        expected = {
            "org.kmipkit.NativeExtensionRegistry.SPLIT_KEY_REFERENCE",
            "org.kmipkit.NativeExtensionRegistry.splitKeyReferenceCreate",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemJoinSplitKey",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseJoinSplitKey",
            "org.kmipkit.extensions.ClientBatchItemResult.JoinSplitKeyResponse.toString",
            "org.kmipkit.extensions.SplitKeyReference.<init>",
            "org.kmipkit.extensions.SplitKeyReference.uniqueIdentifier",
            "org.kmipkit.extensions.SplitKeyReference.splitKeyThreshold",
            "org.kmipkit.extensions.SplitKeyReference.toString",
        }
        self.assertTrue(expected <= set(exclusions))
        self.assertTrue(all(
            "KMIPKIT-0043-FR-021" in exclusions[name]["evidence"] for name in expected
        ))


if __name__ == "__main__":
    unittest.main()
