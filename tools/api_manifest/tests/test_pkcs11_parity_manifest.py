"""Manifest regressions for the T057 KMIP 2.1 PKCS#11 mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PARITY_FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-PKCS-11"
OWNER_SPEC = "specs/026-pkcs11/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    f"KMIPKIT-0040-FR-{number:03d}" for number in range(1, 9)
}
OASIS_ELEMENTS = {
    "KMIPKIT-ELEM-OP-C2S-PKCS-11",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS",
    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::pkcs11",
        "kmipkit_client::ClientRequest::Pkcs11",
        "kmipkit_client::ClientBatchOutcome::Pkcs11",
        "kmipkit_protocol::Pkcs11Request",
        "kmipkit_protocol::Pkcs11Request::new",
        "kmipkit_protocol::Pkcs11Response",
        "kmipkit_protocol::Pkcs11Response::with_ttlv",
        "kmipkit_protocol::Pkcs11Response::with_ttlv_payload",
    },
    "c": {
        "kmipkit_client_batch_item_pkcs11",
        "kmipkit_client_batch_item_response_pkcs11_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.pkcs11",
        "org.kmipkit.extensions.ClientBatchItemResult.pkcs11Response",
        "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response",
        "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.Pkcs11Response.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_pkcs11",
        "kmipkit.client.BatchItemResult.pkcs11_response",
        "kmipkit.client.Pkcs11Response",
        "kmipkit.client.Pkcs11Response.response_ttlv",
        "kmipkit.client.Pkcs11Response.close",
    },
}
API_REQUIREMENTS = {
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
    "tests/fixtures/api/pkcs11-parity.json",
    "crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/pkcs11_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/Pkcs11MappingTest.java",
    "bindings/python/tests/test_pkcs11_operation.py",
}


class Pkcs11ParityManifestTests(unittest.TestCase):
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

    def test_pkcs11_has_verified_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("PKCS#11", operation["name"])
        self.assertEqual("6.1.37", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0040", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0040", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertIn("Approved for implementation", operation["approvalEvidence"]["evidence"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"PKCS#11 has no {language} declarations")
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
        self.assertTrue(OWNER_REQUIREMENTS <= linked_requirements)
        self.assertTrue(API_REQUIREMENTS <= linked_requirements)

    def test_pkcs11_shared_fixture_links_normative_source_and_all_consumers(self) -> None:
        self.assertTrue(PARITY_FIXTURE.is_file(), "shared PKCS#11 parity fixture is missing")
        fixture = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-PKCS-11", fixture["capability"])
        self.assertEqual("OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275", fixture["normativeSource"])
        self.assertEqual({"rust", "c", "java", "python"}, set(fixture["consumerTests"]))
        self.assertTrue(set(fixture["requirementIds"]) >= OWNER_REQUIREMENTS)
        self.assertTrue(set(fixture["oasisElementIds"]) >= OASIS_ELEMENTS)
        self.assertNotIn("profileSupport", fixture["expectedOutput"])
        self.assertFalse(fixture["expectedOutput"]["automaticRetry"])
        self.assertFalse(fixture["expectedOutput"]["automaticFollowUp"])


if __name__ == "__main__":
    unittest.main()
