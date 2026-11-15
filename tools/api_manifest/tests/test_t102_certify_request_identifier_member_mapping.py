"""Require the Certify request identifier accessor in four-language parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/022-object-lifecycle/spec.md"
DECLARATION_ID = "rust-observed-d0317408cb64ea049eaf"
EXPORT_NAME = "kmipkit_protocol::CertifyRequest::unique_identifier"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-CERTIFY"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_certify_request_identifier_member_mapping.py"


class T102CertifyRequestIdentifierMemberMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.operation = next(
            row
            for row in cls.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )

    def test_identifier_member_maps_to_the_existing_four_language_builder(self) -> None:
        declaration = self.declarations[DECLARATION_ID]
        constructor = self.by_name[
            ("rust-kmipkit-protocol", "kmipkit_protocol::CertifyRequest::new")
        ]
        builder = self.by_name[("c-kmipkit", "kmipkit_client_batch_item_certify")]
        expected_mappings = dict(builder["languageMappings"])
        expected_mappings["rust"] = {
            "packageId": "rust-kmipkit-protocol",
            "name": EXPORT_NAME,
        }

        self.assertEqual(EXPORT_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(OPERATION_ID, declaration["capabilityId"])
        self.assertEqual(OWNER, declaration["approvalSource"]["path"])
        self.assertEqual("Approved", declaration["approvalSource"]["status"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertTrue(
            set(builder["parityEvidenceIds"])
            <= set(declaration["parityEvidenceIds"])
        )
        self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
        self.assertIn(DECLARATION_ID, self.operation["declarations"]["rust"])
        self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_mapping_uses_the_approved_certify_source_and_existing_wire_case(self) -> None:
        owner = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Certify | KMIPKIT-ELEM-OP-C2S-CERTIFY | §6.1.6 | 179 | 180 | 181 |",
            owner,
        )
        request_source = (ROOT / "crates/kmipkit-protocol/src/certify.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub const fn unique_identifier(&self)", request_source)
        operation_test = (
            ROOT / "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("OASIS KMIP v2.1 §6.1.6", operation_test)
        self.assertIn("request_emits_table_179_fields_once_and_in_order", operation_test)
        self.assertIn("UNIQUE_IDENTIFIER", operation_test)


if __name__ == "__main__":
    unittest.main()
