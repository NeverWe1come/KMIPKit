"""Pin four-language mappings for key-material transfer serializers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/023-key-material-transfer/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_transfer_request_serializer_mapping.py"
)
OPERATIONS = {
    "export": {
        "name": "Export",
        "capability": "KMIPKIT-ELEM-OP-C2S-EXPORT",
        "builder": "kmipkit_client_batch_item_export",
        "table_row": "| Export | KMIPKIT-ELEM-OP-C2S-EXPORT | §6.1.18 | 217 | 218 | 219 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/export_operation_tests.rs",
        "section": "§6.1.18",
        "target": (
            "rust-observed-5af367e03728bdd7b565",
            "kmipkit_protocol::ExportRequest::into_ttlv_payload",
            "kmipkit_protocol::ExportRequest::new",
        ),
    },
    "import": {
        "name": "Import",
        "capability": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        "builder": "kmipkit_client_batch_item_import",
        "table_row": "| Import | KMIPKIT-ELEM-OP-C2S-IMPORT | §6.1.25 | 238 | 239 | 240 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs",
        "section": "§6.1.25",
        "target": (
            "rust-observed-eef9d3b1dc43a901bd90",
            "kmipkit_protocol::ImportRequest::into_ttlv_payload",
            "kmipkit_protocol::ImportRequest::new",
        ),
    },
    "join_split_key": {
        "name": "Join Split Key",
        "capability": "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        "builder": "kmipkit_client_batch_item_join_split_key",
        "table_row": "| Join Split Key | KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY | §6.1.27 | 244 | 245 | 246 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs",
        "section": "§6.1.27",
        "target": (
            "rust-observed-1864399564d3d70f4bbb",
            "kmipkit_protocol::JoinSplitKeyRequest::into_ttlv_payload",
            "kmipkit_protocol::JoinSplitKeyRequest::new",
        ),
    },
}


class T102TransferRequestSerializerMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_package_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }

    def _operation_row(self, operation_id: str) -> dict[str, object]:
        return next(
            row
            for row in self.manifest["operations"]
            if row["operationId"] == operation_id
        )

    def test_three_transfer_request_serializers_map_to_existing_capabilities(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            builder = self.by_package_name[("c-kmipkit", operation_spec["builder"])]
            operation = self._operation_row(operation_spec["capability"])
            with self.subTest(operation=operation_name):
                self.assertEqual(operation_spec["name"], operation["name"])
                self.assertEqual(operation_spec["capability"], builder["capabilityId"])
                self.assertEqual(OWNER, builder["approvalSource"]["path"])
                declaration_id, rust_name, reference_name = operation_spec["target"]
                declaration = self.declarations[declaration_id]
                reference = self.by_package_name[("rust-kmipkit-protocol", reference_name)]
                self.assertEqual(rust_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(operation_spec["capability"], declaration["capabilityId"])
                self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                self.assertEqual(reference["requirementIds"], declaration["requirementIds"])

                expected_mappings = dict(builder["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": rust_name,
                }
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertTrue(
                    set(builder["parityEvidenceIds"])
                    <= set(declaration["parityEvidenceIds"])
                )
                self.assertIn(
                    PARITY_TEST_EVIDENCE_ID,
                    declaration["parityEvidenceIds"],
                )
                self.assertIn(declaration_id, operation["declarations"]["rust"])
                self.assertIn(PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"])

    def test_mapping_cites_the_approved_owner_and_exact_oasis_tables(self) -> None:
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "OASIS Key Management Interoperability Protocol Specification Version 2.1",
            specification,
        )
        for operation_name, operation_spec in OPERATIONS.items():
            with self.subTest(operation=operation_name):
                self.assertIn(operation_spec["table_row"], specification)
                source_test = (ROOT / operation_spec["source_test"]).read_text(
                    encoding="utf-8"
                )
                self.assertIn(operation_spec["section"], source_test)


if __name__ == "__main__":
    unittest.main()
