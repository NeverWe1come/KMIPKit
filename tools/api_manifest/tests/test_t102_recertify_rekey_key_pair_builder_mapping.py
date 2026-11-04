"""Pin the exact six KMIPKit request builders mapped in the current T102 batch."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
OWNER = "specs/022-object-lifecycle/spec.md"
RECERTIFY = "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"
REKEY_KEY_PAIR = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
JAVA_WIRE = "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
C_WIRE = "crates/kmipkit-ffi/tests/c_api_coverage.rs"
WIRE_EVIDENCE = {JAVA_WIRE, C_WIRE}
NEW_EVIDENCE = WIRE_EVIDENCE | {"tools/api_manifest/tests/test_t102_recertify_rekey_key_pair_builder_mapping.py"}
RECERTIFY_TYPE_ID = "rust-observed-4e379cb3b97bca658dbf"
RECERTIFY_TYPE_NAME = "kmipkit_protocol::RecertifyRequest::with_certificate_request_type"
RECERTIFY_VALUE_ID = "rust-observed-be7b44dc6c8bae24ad53"
RECERTIFY_VALUE_NAME = "kmipkit_protocol::RecertifyRequest::with_certificate_request_value"
RECERTIFY_TYPE_REQUIREMENTS = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-008",
    "KMIPKIT-REQ-SPEC-6.1.45-008",
}
RECERTIFY_VALUE_REQUIREMENTS = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-005",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-008",
    "KMIPKIT-REQ-SPEC-6.1.45-008",
}
RECERTIFY_TYPE_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    JAVA_WIRE,
    "bindings/python/tests/test_recertify_operation.py",
    "tools/api_manifest/tests/test_t102_recertify_rekey_key_pair_builder_mapping.py",
}
RECERTIFY_VALUE_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    JAVA_WIRE,
    "bindings/python/tests/test_recertify_operation.py",
    C_WIRE,
    "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
    "tools/api_manifest/tests/test_t102_recertify_rekey_key_pair_builder_mapping.py",
}
RAW_RECERTIFY_DECLARATIONS = {
    "rust-observed-bd90b6e53d6c05c1a6b0",
    "rust-observed-4c37b205dc5bee05df9a",
}
RAW_RECERTIFY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    JAVA_WIRE,
    "bindings/python/tests/test_recertify_operation.py",
}

TARGETS = {
    "rust-observed-e98ebdedfd3bd6dc5178": (
        "kmipkit_protocol::RecertifyRequest::new",
        RECERTIFY,
        {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004", "KMIPKIT-REQ-SPEC-6.1.45-001"},
        "kmipkit_client_batch_item_recertify",
    ),
    "rust-observed-3b7ec1d8b422ce2f856a": (
        "kmipkit_protocol::RecertifyRequest::with_unique_identifier",
        RECERTIFY,
        {
            "KMIPKIT-0036-FR-001",
            "KMIPKIT-0036-FR-002",
            "KMIPKIT-0036-FR-004",
            "KMIPKIT-0036-FR-006",
            "KMIPKIT-0036-FR-008",
            "KMIPKIT-REQ-SPEC-6.1.45-002-001",
            "KMIPKIT-REQ-SPEC-6.1.45-002-003",
        },
        "kmipkit_client_batch_item_recertify",
    ),
    "rust-observed-a305050686931a657f67": (
        "kmipkit_protocol::RecertifyRequest::with_certificate_request_unique_identifier",
        RECERTIFY,
        {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-008"},
        "kmipkit_client_batch_item_recertify",
    ),
    "rust-observed-18147c6b3bc743f37b88": (
        "kmipkit_protocol::RecertifyRequest::with_offset",
        RECERTIFY,
        {
            "KMIPKIT-0036-FR-001",
            "KMIPKIT-0036-FR-004",
            "KMIPKIT-0036-FR-006",
            "KMIPKIT-0036-FR-008",
            "KMIPKIT-REQ-SPEC-6.1.45-006-001",
        },
        "kmipkit_client_batch_item_recertify",
    ),
    "rust-observed-87194cce123787c123e6": (
        "kmipkit_protocol::RekeyKeyPairRequest::new",
        REKEY_KEY_PAIR,
        {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004", "KMIPKIT-REQ-SPEC-6.1.47-001"},
        "kmipkit_client_batch_item_rekey_key_pair",
    ),
    "rust-observed-c48c50a3aba9f3320543": (
        "kmipkit_protocol::RekeyKeyPairRequest::with_offset",
        REKEY_KEY_PAIR,
        {
            "KMIPKIT-0036-FR-001",
            "KMIPKIT-0036-FR-004",
            "KMIPKIT-0036-FR-006",
            "KMIPKIT-0036-FR-008",
            "KMIPKIT-REQ-SPEC-6.1.47-004-001",
        },
        "kmipkit_client_batch_item_rekey_key_pair",
    ),
}


class T102RecertifyRekeyKeyPairBuilderMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {row["declarationId"]: row for row in cls.manifest["publicDeclarations"]}
        cls.by_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.elements = {row["element_id"]: row for row in cls.catalog["elements"]}

    def test_only_the_exact_six_rows_map_with_their_direct_requirement_subsets(self) -> None:
        self.assertEqual(6, len(TARGETS))
        for declaration_id, (export_name, capability, requirements, builder_name) in TARGETS.items():
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                builder = self.by_name[("c-kmipkit", builder_name)]
                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(capability, declaration["capabilityId"])
                self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                self.assertEqual(requirements, set(declaration["requirementIds"]))
                self.assertNotIn("KMIPKIT-REQ-SPEC-6.1.45-008", declaration["requirementIds"])
                self.assertEqual("four-language", declaration["parityScope"])
                expected_mappings = dict(builder["languageMappings"])
                expected_mappings["rust"] = {"packageId": "rust-kmipkit-protocol", "name": export_name}
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertTrue(NEW_EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))

        for row in self.manifest["publicDeclarations"]:
            if row.get("capabilityId") == RECERTIFY and row.get("declarationState") == "verified":
                if row["declarationId"] in TARGETS:
                    self.assertNotIn("KMIPKIT-REQ-SPEC-6.1.45-008", row["requirementIds"])

    def test_selected_ids_are_added_to_exact_operation_rust_sets(self) -> None:
        operations = {row["operationId"]: row for row in self.manifest["operations"]}
        expected = {
            RECERTIFY: {key for key, (_, operation, _, _) in TARGETS.items() if operation == RECERTIFY},
            REKEY_KEY_PAIR: {
                key for key, (_, operation, _, _) in TARGETS.items() if operation == REKEY_KEY_PAIR
            },
        }
        for operation_id, declaration_ids in expected.items():
            self.assertTrue(declaration_ids.issubset(set(operations[operation_id]["declarations"]["rust"])))

    def test_t102_candidate_total_after_recertify_builder_mapping(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))

    def test_recertify_type_is_a_seventh_exact_mapping_with_value_absent(self) -> None:
        declaration = self.declarations[RECERTIFY_TYPE_ID]
        self.assertEqual(RECERTIFY_TYPE_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(RECERTIFY, declaration["capabilityId"])
        self.assertEqual(RECERTIFY_TYPE_REQUIREMENTS, set(declaration["requirementIds"]))
        self.assertEqual("four-language", declaration["parityScope"])
        builder = self.by_name[("c-kmipkit", "kmipkit_client_batch_item_recertify")]
        expected_mappings = dict(builder["languageMappings"])
        expected_mappings["rust"] = {"packageId": "rust-kmipkit-protocol", "name": RECERTIFY_TYPE_NAME}
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertTrue(RECERTIFY_TYPE_EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))

        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))
        self.assertEqual(
            "verified",
            self.declarations["rust-observed-e98ebdedfd3bd6dc5178"]["declarationState"],
        )
        for blocked_id in (
            "rust-observed-1f4d5ae1f197c06d274f",
            "rust-observed-d7bf84b1a5fd4c705597",
            "rust-observed-e1413e6ad346eed590c5",
        ):
            self.assertEqual("blocked", self.declarations[blocked_id]["declarationState"])

    def test_recertify_type_and_value_condition_and_description_are_source_pinned(self) -> None:
        oasis = " ".join(re.sub(
            r"<[^>]+>", " ", OASIS_HTML.read_text(encoding="cp1252")
        ).split())
        self.assertIn("Table 300: Re-certify Request Payload", oasis)
        self.assertRegex(oasis, r"Certificate Request Type No An Enumeration object.*?It is REQUIRED if the Certificate Request is present\.")
        self.assertRegex(oasis, r"Certificate Request Value No A Byte String object with the certificate request\.")
        self.assertEqual("Certificate Request Type", self.elements["KMIPKIT-ELEM-TAG-420019"]["name"])
        self.assertIn("KMIPKIT-ELEM-ENUMERATION-TAG", self.elements["KMIPKIT-ELEM-TAG-420019"]["parent_element_ids"])
        source = MANIFEST.read_text(encoding="utf-8")
        self.assertTrue("Certificate Request Type is optional and may appear without Certificate Request Value" in source)
        self.assertFalse("Certificate Request Type is present only with Certificate Request Value" in source)

    def test_recertify_request_value_maps_to_the_sensitive_four_language_builder(self) -> None:
        declaration = self.declarations[RECERTIFY_VALUE_ID]
        self.assertEqual(RECERTIFY_VALUE_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(RECERTIFY, declaration["capabilityId"])
        self.assertEqual(RECERTIFY_VALUE_REQUIREMENTS, set(declaration["requirementIds"]))
        self.assertEqual("four-language", declaration["parityScope"])
        builder = self.by_name[("c-kmipkit", "kmipkit_client_batch_item_recertify")]
        expected_mappings = dict(builder["languageMappings"])
        expected_mappings["rust"] = {
            "packageId": "rust-kmipkit-protocol",
            "name": RECERTIFY_VALUE_NAME,
        }
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertTrue(RECERTIFY_VALUE_EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))

        operations = {row["operationId"]: row for row in self.manifest["operations"]}
        self.assertIn(RECERTIFY_VALUE_ID, operations[RECERTIFY]["declarations"]["rust"])

        binding_evidence = {
            "bindings/c/tests/client_abi_consumer.c": "recertify_request_sentinel",
            JAVA_WIRE: "recertify-request-sentinel",
            "bindings/python/tests/test_recertify_operation.py": "recertify-request-sentinel",
            C_WIRE: "recertify-request-sentinel",
        }
        for path, proof in binding_evidence.items():
            self.assertIn(proof, (ROOT / path).read_text(encoding="utf-8"), path)

    def test_raw_recertify_operation_has_direct_metadata_evidence_in_each_binding(self) -> None:
        self.assertEqual(2, len(RAW_RECERTIFY_DECLARATIONS))
        operations = {row["operationId"]: row for row in self.manifest["operations"]}
        for declaration_id in RAW_RECERTIFY_DECLARATIONS:
            declaration = self.declarations[declaration_id]
            self.assertIn("ClientOperation::Recertify", declaration["exportName"])
            self.assertTrue(RAW_RECERTIFY_EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))
        self.assertIn(JAVA_WIRE, operations[RECERTIFY]["parityEvidenceIds"])
        proofs = {
            "bindings/c/tests/client_abi_consumer.c": "recertify_metadata.operation != UINT32_C(7)",
            JAVA_WIRE: "assertEquals(0x07, item.operation());",
            "bindings/python/tests/test_recertify_operation.py": "assert item.operation == 0x07",
        }
        for path, proof in proofs.items():
            self.assertTrue(proof in (ROOT / path).read_text(encoding="utf-8"), path)

    def test_approved_tables_define_the_wire_fields_and_placeholder_behavior(self) -> None:
        html = OASIS_HTML.read_text(encoding="cp1252")
        text = " ".join(re.sub(r"<[^>]+>", " ", html).split())
        self.assertIn("Table 189: Create Key Pair Request Payload", text)
        self.assertIn("Table 190: Create Key Pair Response Payload", text)
        self.assertIn("The ID Placeholder value SHALL be set to the Unique Identifier of the Private Key.", text)
        self.assertRegex(
            text,
            r"The key management server SHALL implement a temporary variable called the ID Placeholder\. "
            r"This value consists of a single Unique Identifier\..*?If the Batch Order Option is set to true \(or unspecified\), "
            r"then subsequent operations in the batched request MAY make use of the ID Placeholder by omitting the Unique Identifier field",
        )
        self.assertIn("Table 300: Re-certify Request Payload", text)
        self.assertRegex(
            text,
            r"Certificate Request Unique Identifier No The Unique Identifier of the Certificate Request\."
            r".*?Offset No An Interval object indicating the difference between the Initial Date of the new certificate",
        )
        self.assertIn("Table 310: Re-key Key Pair Request Payload", text)
        self.assertRegex(
            text,
            r"Private Key Unique Identifier No Determines the existing Asymmetric key pair to be re-keyed\. "
            r"If omitted, then the ID Placeholder is substituted by the server\."
            r".*?Offset No An Interval object indicating the difference between the Initial Date and the Activation Date",
        )


if __name__ == "__main__":
    unittest.main()
