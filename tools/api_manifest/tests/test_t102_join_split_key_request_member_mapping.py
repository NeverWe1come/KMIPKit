"""Require four-language mappings for Join Split Key request members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
MAPPING_EVIDENCE = (
    "tools/api_manifest/tests/test_t102_join_split_key_request_member_mapping.py"
)
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY"
OWNER = "specs/023-key-material-transfer/spec.md"
BUILDER = "kmipkit_client_batch_item_join_split_key"
SOURCE_TEST = "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs"
TARGETS = (
    ("rust-observed-0b291199b700388a5ee9", "kmipkit_protocol::JoinSplitKeyRequest::attributes"),
    ("rust-observed-77928c7308f9e390f8f6", "kmipkit_protocol::JoinSplitKeyRequest::object_type"),
    ("rust-observed-4fef0ce2a15f87ce8d12", "kmipkit_protocol::JoinSplitKeyRequest::protection_storage_masks"),
    ("rust-observed-dbc03ccef090eedbdbd9", "kmipkit_protocol::JoinSplitKeyRequest::references"),
    ("rust-observed-28b2674184eadc2239c5", "kmipkit_protocol::JoinSplitKeyRequest::secret_data_type"),
)


class T102JoinSplitKeyRequestMemberMappingTests(unittest.TestCase):
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
        cls.operations = {
            row["operationId"]: row for row in cls.manifest["operations"]
        }

    def _assert_target_mapping(
        self,
        declaration: dict[str, object],
        constructor: dict[str, object],
        mappings: dict[str, object],
        declaration_id: str,
        export_name: str,
        operation: dict[str, object],
    ) -> None:
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(OPERATION_ID, declaration["capabilityId"])
        self.assertEqual(constructor["approvalSource"], declaration["approvalSource"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(export_name, declaration["exportName"])
        self.assertEqual(mappings, declaration["languageMappings"])
        self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_request_members_map_to_the_existing_four_language_builder(self) -> None:
        operation = self.operations[OPERATION_ID]
        builder = self.by_name[("c-kmipkit", BUILDER)]
        constructor = self.by_name[
            ("rust-kmipkit-protocol", "kmipkit_protocol::JoinSplitKeyRequest::new")
        ]

        self.assertEqual(OWNER, builder["approvalSource"]["path"])
        self.assertEqual("Approved", builder["approvalSource"]["status"])
        for declaration_id, export_name in TARGETS:
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                mappings = dict(builder["languageMappings"])
                mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": export_name,
                }
                self._assert_target_mapping(
                    declaration,
                    constructor,
                    mappings,
                    declaration_id,
                    export_name,
                    operation,
                )

    def test_mapping_pins_approved_owner_and_oasis_request_source(self) -> None:
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Join Split Key | KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY | §6.1.27 | 244 | 245 | 246 |",
            specification,
        )
        source_test = (ROOT / SOURCE_TEST).read_text(encoding="utf-8")
        self.assertIn("OASIS KMIP v2.1 §6.1.27, Tables 244–246", source_test)


if __name__ == "__main__":
    unittest.main()
