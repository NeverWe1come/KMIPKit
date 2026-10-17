"""Manifest regressions for the T043 Certify operation mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-CERTIFY"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.6-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-002",
    "KMIPKIT-REQ-SPEC-6.1.6-002-003",
    "KMIPKIT-REQ-SPEC-6.1.6-003",
    "KMIPKIT-REQ-SPEC-6.1.6-007",
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-003",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-005",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-008",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.6-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-002",
    "KMIPKIT-REQ-SPEC-6.1.6-002-003",
    "KMIPKIT-REQ-SPEC-6.1.6-003",
    "KMIPKIT-REQ-SPEC-6.1.6-007",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/CertifyJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/CertifyResponseMappingTest.java",
    "bindings/python/tests/test_certify_operation.py",
    "bindings/python/tests/test_client.py",
    "crates/kmipkit-client/tests/unit/certify_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-ffi/tests/support/cryptographic_operation_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::Certify",
        "kmipkit_client::ClientBatchOutcome::certify_response",
        "kmipkit_protocol::CertifyResponse",
        "kmipkit_protocol::CertifyResponse::unique_identifier",
        "kmipkit_protocol::CertifyResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_certify",
        "kmipkit_client_batch_item_response_certify",
        "kmipkit_client_batch_item_response_certify_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.certify",
        "org.kmipkit.extensions.ClientBatchItemResult.CertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.certifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.CertifyResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.CertifyResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_certify",
        "kmipkit.client.CertifyResponse",
        "kmipkit.client.BatchItemResult.certify_response",
        "kmipkit.client.CertifyResponse.response_ttlv",
        "kmipkit.client.CertifyResponse.close",
    },
}
EXPECTED_IMPLEMENTATION_EXCLUSIONS = {
    "org.kmipkit.NativeExtensionRegistry.clientBatchItemCertify",
    "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseCertify",
    "org.kmipkit.extensions.ClientBatchItemResult.CertifyResponse.toString",
    "kmipkit.client.CertifyResponse.__dataclass_fields__",
    "kmipkit.client.CertifyResponse.__dataclass_params__",
    "kmipkit.client.CertifyResponse.__delattr__",
    "kmipkit.client.CertifyResponse.__eq__",
    "kmipkit.client.CertifyResponse.__hash__",
    "kmipkit.client.CertifyResponse.__init__",
    "kmipkit.client.CertifyResponse.__match_args__",
    "kmipkit.client.CertifyResponse.__repr__",
    "kmipkit.client.CertifyResponse.__setattr__",
    "kmipkit.client.CertifyResponse.__slots__",
}


class CertifyParityManifestTests(unittest.TestCase):
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

    def test_certify_has_approved_four_language_mapping(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Certify", operation["name"])
        self.assertEqual("6.1.6", operation["section"])
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

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Certify has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertIn(declaration["capabilityId"], {OPERATION_ID, "client_execution"})
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(
                    PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]),
                    declaration_id,
                )
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(
                    declaration["languageMappings"][language]["name"]
                )
                linked_requirements.update(declaration["requirementIds"])

        missing = {
            language: sorted(expected - actual_mappings[language])
            for language, expected in EXPECTED_MAPPINGS.items()
            if expected - actual_mappings[language]
        }
        self.assertEqual({}, missing)
        self.assertTrue(REQUIREMENTS <= linked_requirements)

    def test_certify_public_declarations_link_owner_requirement_ids(self) -> None:
        operation = self.operations[0]
        declaration_ids = {
            declaration_id
            for language_ids in operation["declarations"].values()
            for declaration_id in language_ids
        }
        for declaration_id in declaration_ids:
            declaration = self.declarations[declaration_id]
            if declaration["capabilityId"] == OPERATION_ID:
                self.assertTrue(
                    OWNER_REQUIREMENTS <= set(declaration["requirementIds"]),
                    declaration_id,
                )

    def test_generated_parity_inventory_contains_certify_mappings(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row
            for row in generated["operations"]
            if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        operation = operations[0]
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual(LANGUAGES, set(operation["declarations"]))

    def test_certify_bridge_and_generated_helpers_have_exact_exclusions(self) -> None:
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["exportName"] in EXPECTED_IMPLEMENTATION_EXCLUSIONS
        }
        self.assertEqual(EXPECTED_IMPLEMENTATION_EXCLUSIONS, set(exclusions))
        for export_name in EXPECTED_IMPLEMENTATION_EXCLUSIONS:
            exclusion = exclusions[export_name]
            self.assertEqual("java-kmipkit" if export_name.startswith("org.") else "python-kmipkit",
                exclusion["packageId"])
            self.assertTrue(exclusion["reason"].strip(), export_name)
            self.assertTrue(exclusion["evidence"].strip(), export_name)

    def test_certify_c_response_documentation_states_borrowed_identifier_lifetime(self) -> None:
        response_function = next(
            row
            for row in self.manifest["functions"]
            if row["id"] == "client_batch_item_response_certify"
        )
        documentation = " ".join(response_function["c"]["documentation"])
        self.assertIn("borrowed from the response-item handle", documentation)
        self.assertIn("until that handle is released", documentation)


if __name__ == "__main__":
    unittest.main()
