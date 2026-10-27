"""Require the approved KMIP v2.1 §6.1.31 Logout mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-LOGOUT"
OWNER_SPEC = "specs/024-credential-sessions/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    *(f"KMIPKIT-0038-FR-{index:03}" for index in (1, 2, 3, 4, 5, 6, 8)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)),
}
TRACEABILITY_IDS = {
    "KMIPKIT-024-T256-REQUEST",
    "KMIPKIT-024-T257-RESPONSE",
    "KMIPKIT-024-T258-ERROR",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/LogoutMappingTest.java",
    "bindings/python/tests/test_logout_operation.py",
    "crates/kmipkit-client/tests/unit/logout_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/logout_operation_tests.rs",
    "tools/api_manifest/tests/test_logout_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::logout",
        "kmipkit_client::ClientBatchOutcome::logout_response",
        "kmipkit_protocol::LogoutRequest",
        "kmipkit_protocol::LogoutRequest::new",
        "kmipkit_protocol::LogoutResponse",
        "kmipkit_protocol::LogoutResponse::result",
    },
    "c": {
        "kmipkit_client_batch_item_logout",
        "kmipkit_client_batch_item_response_logout",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.logout",
        "org.kmipkit.extensions.ClientBatchItemResult.logoutResponse",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_logout",
        "kmipkit.client.BatchItemResult.logout_response",
    },
}


class LogoutParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_logout_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Logout", operation["name"])
        self.assertEqual("6.1.31", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0038", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0038", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        actual_mappings, linked_requirements = assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
        )
        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertEqual(expected, actual_mappings[language], language)
        self.assertEqual(REQUIREMENTS, linked_requirements)

        owner_traceability = (ROOT / "specs/024-credential-sessions/traceability.md").read_text(
            encoding="utf-8"
        )
        for trace_id in TRACEABILITY_IDS:
            self.assertIn(trace_id, owner_traceability)
        mapping_traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        t055_row = next(
            line for line in mapping_traceability.splitlines() if line.startswith("| T055 |")
        )
        self.assertTrue(
            set(re.findall(r"KMIPKIT-0038-FR-\d{3}", t055_row))
            >= {f"KMIPKIT-0038-FR-{index:03}" for index in (1, 2, 3, 4, 5, 6, 8)}
        )
        self.assertTrue(
            {value for value in REQUIREMENTS if value.startswith("KMIPKIT-0043-")}
            <= set(re.findall(r"KMIPKIT-0043-FR-\d{3}", t055_row))
        )

    def test_generated_parity_inventory_contains_verified_logout_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
