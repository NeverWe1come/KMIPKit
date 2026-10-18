"""Require the approved KMIP v2.1 §6.1.29 Log mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-LOG"
OWNER_SPEC = "specs/028-diagnostics-interop/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    *(f"KMIPKIT-0042-FR-{index:03}" for index in (1, 2, 3, 4, 5, 6, 8)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 15, 17, 18, 21, 22)),
}
TRACEABILITY_IDS = {
    "KMIPKIT-028-T250-MESSAGE",
    "KMIPKIT-028-T251-EMPTY",
    "KMIPKIT-028-T252-ERROR",
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/log_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/log_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/log_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/LogJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/LogResponseMappingTest.java",
    "bindings/python/tests/test_log_operation.py",
    "tools/api_manifest/tests/test_log_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::Log",
        "kmipkit_client::ClientBatchOutcome::Log",
        "kmipkit_protocol::LogRequest",
        "kmipkit_protocol::LogRequest::new",
        "kmipkit_protocol::LogResponse",
        "kmipkit_protocol::LogResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_log",
        "kmipkit_client_batch_item_response_log_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.log",
        "org.kmipkit.extensions.ClientBatchItemResult.logResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.LogResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.LogResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.LogResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_log",
        "kmipkit.client.BatchItemResult.log_response",
        "kmipkit.client.LogResponse",
        "kmipkit.client.LogResponse.response_ttlv",
        "kmipkit.client.LogResponse.close",
    },
}


class LogParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_log_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Log", operation["name"])
        self.assertEqual("6.1.29", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0042", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0042", operation["ownerDependency"]["specId"])
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
            self.assertTrue(declaration_ids, f"Log has no {language} declarations")
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

        traceability = (ROOT / "specs/028-diagnostics-interop/traceability.md").read_text(
            encoding="utf-8"
        )
        self.assertTrue(TRACEABILITY_IDS <= set(
            token for token in traceability.replace("|", " ").split()
            if token.startswith("KMIPKIT-028-")
        ))
        mapping_traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        t053_row = next(line for line in mapping_traceability.splitlines() if line.startswith("| T053 |"))
        linked_api_requirements = {
            value for value in REQUIREMENTS if value.startswith("KMIPKIT-0043-")
        }
        self.assertTrue(
            linked_api_requirements <= set(re.findall(r"KMIPKIT-0043-FR-\d{3}", t053_row))
        )

    def test_generated_parity_inventory_contains_verified_log_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
