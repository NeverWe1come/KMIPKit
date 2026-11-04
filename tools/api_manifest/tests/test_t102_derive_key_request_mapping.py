"""Require Derive Key request declarations to map to the four-language API."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/023-key-material-transfer/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_derive_key_request_mapping.py"
DECLARATIONS = (
    (
        "rust-observed-e1c0811fd3085d61d37e",
        "kmipkit_protocol::DeriveKeyRequest",
        "type",
    ),
    (
        "rust-observed-b0a921a3e52d117b586e",
        "kmipkit_protocol::DeriveKeyRequest::new",
        "associated-item",
    ),
    (
        "rust-observed-616ed68521fb27aad6bb",
        "kmipkit_protocol::DeriveKeyRequest::into_ttlv_payload",
        "associated-item",
    ),
)


class T102DeriveKeyRequestMappingTests(unittest.TestCase):
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

    def test_request_type_constructor_and_serializer_map_to_existing_builders(self) -> None:
        anchor = self.by_name[("rust-kmipkit-client", "kmipkit_client::Client::derive_key")]
        for declaration_id, name, kind in DECLARATIONS:
            with self.subTest(declaration=name):
                declaration = self.declarations[declaration_id]
                expected_mappings = dict(anchor["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": name,
                }

                self.assertEqual(name, declaration["exportName"])
                self.assertEqual(kind, declaration["kind"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual(anchor["requirementIds"], declaration["requirementIds"])
                self.assertEqual(anchor["approvalSource"], declaration["approvalSource"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertNotIn("blockedReason", declaration)
                self.assertNotIn("candidateSpecId", declaration)
                self.assertIn(declaration_id, self.operation["declarations"]["rust"])
                self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_mapping_is_grounded_in_the_approved_table_and_existing_wire_test(self) -> None:
        owner = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Derive Key | KMIPKIT-ELEM-OP-C2S-DERIVE-KEY | §6.1.14 | 205 | 206 | 207 |",
            owner,
        )
        request_source = (ROOT / "crates/kmipkit-protocol/src/derive_key.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub struct DerivationParameters", request_source)
        self.assertIn("pub struct DeriveKeyRequest", request_source)
        self.assertIn("pub fn into_ttlv_payload(self)", request_source)
        operation_test = (
            ROOT / "crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("§6.1.14 Table 205", operation_test)
        self.assertIn(
            "request_encodes_required_table_205_fields_in_order_and_repeats_identifiers",
            operation_test,
        )


if __name__ == "__main__":
    unittest.main()
