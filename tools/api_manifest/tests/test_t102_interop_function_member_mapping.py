"""Require the Interop Function field to map across the existing operation API."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/028-diagnostics-interop/spec.md"
DECLARATION_ID = "rust-observed-326a5a1e346eb2a8787e"
EXPORT_NAME = "kmipkit_protocol::InteropRequest::interop_function"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-INTEROP"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_interop_function_member_mapping.py"


class T102InteropFunctionMemberMappingTests(unittest.TestCase):
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
            row for row in cls.manifest["operations"] if row["operationId"] == OPERATION_ID
        )

    def _assert_capability_mapping(self, declaration: dict[str, object]) -> None:
        constructor = self.by_name[
            ("rust-kmipkit-protocol", "kmipkit_protocol::InteropRequest::new")
        ]
        builder = self.by_name[("c-kmipkit", "kmipkit_client_batch_item_interop")]
        expected_mappings = dict(builder["languageMappings"])
        expected_mappings["rust"] = {
            "packageId": "rust-kmipkit-protocol",
            "name": EXPORT_NAME,
        }

        self.assertEqual(OWNER, declaration["approvalSource"]["path"])
        self.assertEqual("Approved", declaration["approvalSource"]["status"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
        self.assertIn(DECLARATION_ID, self.operation["declarations"]["rust"])
        self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_interop_function_maps_to_the_existing_four_language_builder(self) -> None:
        declaration = self.declarations[DECLARATION_ID]

        self.assertEqual(EXPORT_NAME, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(OPERATION_ID, declaration["capabilityId"])
        self._assert_capability_mapping(declaration)

    def test_mapping_is_grounded_in_the_approved_table_and_open_value_test(self) -> None:
        owner = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Interop | KMIPKIT-ELEM-OP-C2S-INTEROP | §6.1.26 | 241 | 242 | 243 |",
            owner,
        )
        protocol_source = (ROOT / "crates/kmipkit-protocol/src/interop.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("OASIS KMIP Specification v2.1 §6.1.26, Tables 241–243", protocol_source)
        operation_test = (
            ROOT / "crates/kmipkit-protocol/tests/unit/interop_operation_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("interop_function_standard_vendor_and_future_values_are_open", operation_test)


if __name__ == "__main__":
    unittest.main()
