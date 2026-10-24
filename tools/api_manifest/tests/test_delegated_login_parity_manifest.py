"""Require complete four-language Delegated Login mappings and provenance."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN"
OWNER_SPEC = "specs/024-credential-sessions/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    "KMIPKIT-0038-FR-001",
    "KMIPKIT-0038-FR-002",
    "KMIPKIT-0038-FR-003",
    "KMIPKIT-0038-FR-004",
    "KMIPKIT-0038-FR-005",
    "KMIPKIT-0038-FR-006",
    "KMIPKIT-0038-FR-008",
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
TABLE_TRACEABILITY = {
    "KMIPKIT-024-T199-REQUEST",
    "KMIPKIT-024-T200-RESPONSE",
    "KMIPKIT-024-T201-ERROR",
    "KMIPKIT-ELEM-TAG-420049",
    "KMIPKIT-ELEM-TAG-42014C",
    "KMIPKIT-ELEM-TAG-420095",
    "KMIPKIT-ELEM-TAG-42014D",
    "KMIPKIT-ELEM-TAG-420149",
    "KMIPKIT-ELEM-TAG-42014A",
    "KMIPKIT-ELEM-TAG-42014B",
    "KMIPKIT-ELEM-TAG-42007F",
    "KMIPKIT-ELEM-TAG-42007E",
    "KMIPKIT-ELEM-TAG-42007D",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/DelegatedLoginMappingTest.java",
    "bindings/python/tests/test_delegated_login.py",
    "crates/kmipkit-client/tests/unit/delegated_login_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::delegated_login",
        "kmipkit_client::ClientBatchOutcome::delegated_login_response",
        "kmipkit_protocol::DelegatedLoginRequest",
        "kmipkit_protocol::DelegatedLoginRequest::new",
        "kmipkit_protocol::DelegatedLoginRequest::with_lease_time_interval",
        "kmipkit_protocol::DelegatedLoginRequest::with_lease_time_date_time",
        "kmipkit_protocol::DelegatedLoginRequest::with_request_count",
        "kmipkit_protocol::DelegatedLoginRequest::with_usage_limits",
        "kmipkit_protocol::DelegatedLoginResponse",
        "kmipkit_protocol::DelegatedLoginResponse::ticket",
        "kmipkit_protocol::Ticket::ticket_type",
        "kmipkit_protocol::Ticket::with_ticket_value",
        "kmipkit_protocol::Ticket::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_delegated_login",
        "kmipkit_client_batch_item_response_delegated_login",
        "kmipkit_client_batch_item_response_delegated_login_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.delegatedLogin",
        "org.kmipkit.extensions.ClientBatchItemResult.delegatedLoginResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse.ticketTypeRaw",
        "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse.ticketValue",
        "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse.ticketTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.DelegatedLoginResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_delegated_login",
        "kmipkit.client.BatchItemResult.delegated_login_response",
        "kmipkit.client.DelegatedLoginResponse",
        "kmipkit.client.DelegatedLoginResponse.ticket_type_raw",
        "kmipkit.client.DelegatedLoginResponse.ticket_value",
        "kmipkit.client.DelegatedLoginResponse.ticket_ttlv",
        "kmipkit.client.DelegatedLoginResponse.close",
    },
}


class DelegatedLoginParityManifestTests(unittest.TestCase):
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

    def test_delegated_login_has_approved_four_language_mapping_and_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Delegated Login", operation["name"])
        self.assertEqual("6.1.12", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0038", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0038", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        traceability = (ROOT / "specs/024-credential-sessions/traceability.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("§6.1.12; Tables 199–201", traceability)
        for trace_id in TABLE_TRACEABILITY:
            self.assertIn(trace_id, traceability)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Delegated Login has no {language} declarations")
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
        self.assertTrue(REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_java_ticket_ttlv_getter_does_not_claim_a_native_entrypoint(self) -> None:
        function = next(
            row
            for row in self.manifest["functions"]
            if row["id"] == "client_batch_item_response_delegated_login_ttlv"
        )

        self.assertNotIn("jniSymbol", function["java"])

    def test_python_ticket_value_property_is_inventoried_as_an_export(self) -> None:
        self.assertEqual(
            "python-export",
            self.declarations["T045:delegated-login:ticket-value:python"]["kind"],
        )


if __name__ == "__main__":
    unittest.main()
