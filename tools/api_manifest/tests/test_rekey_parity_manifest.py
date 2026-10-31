"""Manifest regressions for the KMIP 2.1 §6.1.46 Re-key mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
TRACEABILITY = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.46-001",
    "KMIPKIT-REQ-SPEC-6.1.46-004-001",
    "KMIPKIT-REQ-SPEC-6.1.46-004-002",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/RekeyMappingTest.java",
    "bindings/python/tests/test_rekey_native_integration.py",
    "bindings/python/tests/test_rekey_operation.py",
    "crates/kmipkit-client/tests/unit/rekey_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
    "tools/api_manifest/tests/test_rekey_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::rekey",
        "kmipkit_client::ClientRequest::Rekey",
        "kmipkit_client::ClientBatchOutcome::rekey_response",
        "kmipkit_protocol::RekeyRequest",
        "kmipkit_protocol::RekeyResponse",
        "kmipkit_protocol::RekeyResponse::unique_identifier",
        "kmipkit_protocol::RekeyResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_rekey",
        "kmipkit_client_batch_item_response_rekey",
        "kmipkit_client_batch_item_response_rekey_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.rekey",
        "org.kmipkit.extensions.ClientBatchItemResult.rekeyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RekeyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RekeyResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.RekeyResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.RekeyResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_rekey",
        "kmipkit.client.BatchItemResult.rekey_response",
        "kmipkit.client.RekeyResponse",
        "kmipkit.client.RekeyResponse.unique_identifier",
        "kmipkit.client.RekeyResponse.response_ttlv",
        "kmipkit.client.RekeyResponse.close",
    },
}


class RekeyParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_rekey_has_approved_four_language_mapping(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Re-key", operation["name"])
        self.assertEqual("6.1.46", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))

        mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Re-key has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
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

    def test_rekey_traceability_links_normative_and_api_requirements(self) -> None:
        rows = [line for line in TRACEABILITY.read_text(encoding="utf-8").splitlines()
                if line.startswith("| T059 |")]
        self.assertEqual(1, len(rows))
        linked_ids = set(re.findall(
            r"KMIPKIT-(?:REQ-SPEC-6\.1\.46-[0-9-]+|0036-FR-\d{3}|0043-FR-\d{3})",
            rows[0],
        ))
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_ids)

    def test_generated_parity_inventory_contains_rekey_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [row for row in generated["operations"] if row["operationId"] == OPERATION_ID]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_rekey_c_abi_contract_is_documented_in_manifest_and_generated_header(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        request_docs = " ".join(functions["client_batch_item_rekey"]["c"]["documentation"])
        response_docs = " ".join(
            functions["client_batch_item_response_rekey"]["c"]["documentation"]
        )
        self.assertIn("copied during this call", request_docs)
        self.assertIn("both copies complete before this function returns", request_docs)
        self.assertIn("borrowed from the response item", response_docs)
        self.assertIn("invalid when the response item is released", response_docs)

        header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(
            encoding="utf-8"
        )
        self.assertIn("copied during this call", header)
        self.assertIn("both copies complete before this function returns", header)
        self.assertIn("borrowed from the response item", header)
        self.assertIn("invalid when the response item is released", header)


if __name__ == "__main__":
    unittest.main()
