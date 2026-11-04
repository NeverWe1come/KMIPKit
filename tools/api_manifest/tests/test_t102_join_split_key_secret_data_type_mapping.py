"""Pin the T102 Secret Data Type request-field mapping for Join Split Key."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
OWNER = "specs/023-key-material-transfer/spec.md"
DECLARATION_ID = "rust-observed-7c003b4ec061b8499a9e"
EXPORT_NAME = "kmipkit_protocol::JoinSplitKeyRequest::with_secret_data_type"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY"
REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.27-002-002"
OWNER_REQUIREMENTS = {
    "KMIPKIT-0037-FR-001",
    "KMIPKIT-0037-FR-004",
    "KMIPKIT-0037-FR-006",
    "KMIPKIT-0037-FR-008",
}
REQUIREMENTS = OWNER_REQUIREMENTS | {REQUIREMENT_ID}
EXPECTED_MAPPINGS = {
    "rust": {"packageId": "rust-kmipkit-protocol", "name": EXPORT_NAME},
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_join_split_key"},
    "java": {
        "packageId": "java-kmipkit",
        "name": "org.kmipkit.extensions.ClientBatchItem.joinSplitKey",
    },
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.extensions.client_batch_item_join_split_key",
    },
}
JOIN_SPLIT_PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_join_split_key_operation.py",
    "tools/api_manifest/tests/test_t102_join_split_key_secret_data_type_mapping.py",
}
REKEY_KEY_PAIR_UID_ID = "rust-observed-3fc6c1f77db9dabbee97"
REKEY_KEY_PAIR_UID_NAME = "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_unique_identifier"
REKEY_KEY_PAIR_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
REKEY_KEY_PAIR_OWNER = "specs/022-object-lifecycle/spec.md"
REKEY_KEY_PAIR_UID_REQUIREMENTS = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-008",
}
REKEY_KEY_PAIR_UID_MAPPINGS = {
    "rust": {"packageId": "rust-kmipkit-protocol", "name": REKEY_KEY_PAIR_UID_NAME},
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_rekey_key_pair"},
    "java": {
        "packageId": "java-kmipkit",
        "name": "org.kmipkit.extensions.ClientBatchItem.rekeyKeyPair",
    },
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.extensions.client_batch_item_rekey_key_pair",
    },
}
REKEY_KEY_PAIR_UID_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_rekey_key_pair_native_integration.py",
    "tools/api_manifest/tests/test_t102_join_split_key_secret_data_type_mapping.py",
}


class T102JoinSplitKeySecretDataTypeMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {row["declarationId"]: row for row in cls.manifest["publicDeclarations"]}

    def test_exact_request_setter_maps_only_its_supported_four_language_builder(self) -> None:
        declaration = self.declarations[DECLARATION_ID]
        self.assertEqual(EXPORT_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertNotIn("blockedReason", declaration)
        self.assertEqual(OPERATION_ID, declaration["capabilityId"])
        self.assertEqual(REQUIREMENTS, set(declaration["requirementIds"]))
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(EXPECTED_MAPPINGS, declaration["languageMappings"])
        self.assertEqual(
            {"kind": "feature-specification", "path": OWNER, "status": "Approved",
             "evidence": "**Status**: Approved for implementation — 2026-10-11"},
            declaration["approvalSource"],
        )
        self.assertTrue(JOIN_SPLIT_PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))

        operation = next(row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID)
        self.assertIn(DECLARATION_ID, operation["declarations"]["rust"])
        self.assertTrue(JOIN_SPLIT_PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))

    def test_rekey_key_pair_uid_maps_to_its_builder_without_a_fabricated_normative_id(self) -> None:
        declaration = self.declarations[REKEY_KEY_PAIR_UID_ID]
        self.assertEqual(REKEY_KEY_PAIR_UID_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertNotIn("blockedReason", declaration)
        self.assertEqual(REKEY_KEY_PAIR_OPERATION_ID, declaration["capabilityId"])
        self.assertEqual(REKEY_KEY_PAIR_UID_REQUIREMENTS, set(declaration["requirementIds"]))
        self.assertFalse(
            any(value.startswith("KMIPKIT-REQ-SPEC-6.1.47-") for value in declaration["requirementIds"])
        )
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(REKEY_KEY_PAIR_UID_MAPPINGS, declaration["languageMappings"])
        self.assertEqual(
            {
                "kind": "feature-specification",
                "path": REKEY_KEY_PAIR_OWNER,
                "status": "Approved",
                "evidence": "**Status**: Approved for implementation — 2026-10-11",
            },
            declaration["approvalSource"],
        )
        self.assertTrue(REKEY_KEY_PAIR_UID_EVIDENCE <= set(declaration["parityEvidenceIds"]))
        operation = next(
            row for row in self.manifest["operations"]
            if row["operationId"] == REKEY_KEY_PAIR_OPERATION_ID
        )
        self.assertIn(REKEY_KEY_PAIR_UID_ID, operation["declarations"]["rust"])
        self.assertTrue(REKEY_KEY_PAIR_UID_EVIDENCE <= set(operation["parityEvidenceIds"]))

    def test_neighboring_ttlv_and_response_surfaces_remain_blocked(self) -> None:
        expected_blocked = {
            "kmipkit_protocol::JoinSplitKeyRequest::attributes",
            "kmipkit_protocol::JoinSplitKeyRequest::into_ttlv_payload",
            "kmipkit_protocol::JoinSplitKeyRequest::object_type",
            "kmipkit_protocol::JoinSplitKeyRequest::protection_storage_masks",
            "kmipkit_protocol::JoinSplitKeyRequest::references",
            "kmipkit_protocol::JoinSplitKeyRequest::secret_data_type",
            "kmipkit_protocol::RekeyKeyPairRequest::into_ttlv_payload",
            "kmipkit_protocol::RekeyKeyPairRequest::private_key_unique_identifier",
        }
        actual_blocked = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith((
                "kmipkit_protocol::JoinSplitKeyRequest::",
                "kmipkit_protocol::RekeyKeyPairRequest::",
            ))
            and row["declarationState"] == "blocked"
        }
        self.assertEqual(expected_blocked, actual_blocked)

    def test_oasis_tables_pin_the_optional_enumeration_field_and_known_value(self) -> None:
        html = OASIS_HTML.read_text(encoding="cp1252")
        text = " ".join(re.sub(r"<[^>]+>", " ", html).split())
        self.assertIn("Table 244: Join Split Key Request Payload", text)
        self.assertIn("Secret Data Type No Determines which Secret Data type the Split Keys form.", text)
        self.assertIn("11.52 Secret Data Type Enumeration", text)
        self.assertIn("Table 483: Secret Data Type Enumeration", text)
        self.assertRegex(text, r"Password\s+00000001")
        self.assertIn("Table 310: Re-key Key Pair Request Payload", text)
        self.assertIn("Private Key Unique Identifier No Determines the existing Asymmetric key pair to be re-keyed.", text)
        tag_catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        self.assertIn(
            "KMIPKIT-ELEM-TAG-420066",
            {row.get("element_id") for row in tag_catalog.get("elements", [])},
        )

    def test_t102_total_drops_by_exactly_both_mapped_candidates(self) -> None:
        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(659, blocked)
        self.assertEqual(6_470, len(self.manifest["publicDeclarations"]))
        self.assertEqual(8_554, len(self.manifest["approvedExclusions"]))


if __name__ == "__main__":
    unittest.main()
