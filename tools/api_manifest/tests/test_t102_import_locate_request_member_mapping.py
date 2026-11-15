"""Pin four-language mappings for remaining Locate and Import request members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_import_locate_request_member_mapping.py"
)
OPERATIONS = {
    "locate": {
        "name": "Locate",
        "capability": "KMIPKIT-ELEM-OP-C2S-LOCATE",
        "owner": "specs/017-managed-object-retrieval/spec.md",
        "builder": "kmipkit_client_batch_item_locate",
        "table_row": "| Locate | KMIPKIT-ELEM-OP-C2S-LOCATE | OASIS KMIP Specification v2.1 §6.1.28 | Tables 247–249 | KMIPKIT-TEST-CN01-2-62; KMIPKIT-TEST-CN01-2-63; KMIPKIT-TEST-CN01-2-64; KMIPKIT-TEST-CN01-2-65 | TC-MDO-2-21; TC-MDO-3-21; TC-OFFSET-1-21; TC-OFFSET-2-21 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/locate_tests.rs",
        "section": "§6.1.28",
        "constructor": "kmipkit_protocol::LocateRequest::new",
        "targets": (
            ("rust-observed-f439a46d07fb7b09a304", "kmipkit_protocol::LocateRequest::attributes"),
            ("rust-observed-71e45ec7ed189cbf589f", "kmipkit_protocol::LocateRequest::maximum_items"),
            ("rust-observed-c545de4134a8f62c3871", "kmipkit_protocol::LocateRequest::object_group_member"),
            ("rust-observed-c0872e13783c072c07fb", "kmipkit_protocol::LocateRequest::offset_items"),
            ("rust-observed-6eb74f84ebb61fd70253", "kmipkit_protocol::LocateRequest::storage_status_mask"),
            ("rust-observed-843c51bfe3611b61603b", "kmipkit_protocol::LocateRequest::to_ttlv_payload"),
        ),
    },
    "import": {
        "name": "Import",
        "capability": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        "owner": "specs/023-key-material-transfer/spec.md",
        "builder": "kmipkit_client_batch_item_import",
        "table_row": "| Import | KMIPKIT-ELEM-OP-C2S-IMPORT | §6.1.25 | 238 | 239 | 240 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs",
        "section": "§6.1.25",
        "constructor": "kmipkit_protocol::ImportRequest::new",
        "targets": (
            ("rust-observed-fca2c150feb784fdd203", "kmipkit_protocol::ImportRequest::attributes"),
            ("rust-observed-f05defb53b753257aad1", "kmipkit_protocol::ImportRequest::key_wrap_type"),
            ("rust-observed-b012cd721c307c69208d", "kmipkit_protocol::ImportRequest::object"),
            ("rust-observed-61072f660e5d534e226c", "kmipkit_protocol::ImportRequest::object_type"),
            ("rust-observed-14d3acc960ccab00dbe3", "kmipkit_protocol::ImportRequest::replace_existing"),
            ("rust-observed-5e06cdb2edfa3494515d", "kmipkit_protocol::ImportRequest::unique_identifier"),
        ),
    },
}
FOREIGN_MAPPINGS = {
    "locate": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_locate"),
        "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItem.locate"),
        "python": ("python-kmipkit", "kmipkit.extensions.client_batch_item_locate"),
    },
    "import": {
        "c": ("c-kmipkit", "kmipkit_client_batch_item_import"),
        "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItem.importObject"),
        "python": ("python-kmipkit", "kmipkit.extensions.client_batch_item_import"),
    },
}


class T102ImportLocateRequestMemberMappingTests(unittest.TestCase):
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
        cls.operations = {
            row["operationId"]: row for row in cls.manifest["operations"]
        }

    def _assert_target_mapping(
        self,
        operation_spec: dict[str, object],
        builder: dict[str, object],
        constructor: dict[str, object],
        operation: dict[str, object],
        declaration_id: str,
        export_name: str,
    ) -> None:
        declaration = self.declarations[declaration_id]
        mappings = dict(builder["languageMappings"])
        mappings["rust"] = {
            "packageId": "rust-kmipkit-protocol",
            "name": export_name,
        }

        self.assertEqual(export_name, declaration["exportName"])
        self.assertEqual("rust-kmipkit-protocol", declaration["packageId"])
        self.assertEqual("associated-item", declaration["kind"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertNotIn("blockedReason", declaration)
        self.assertEqual(operation_spec["capability"], declaration["capabilityId"])
        self.assertEqual(constructor["approvalSource"], declaration["approvalSource"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
        self.assertEqual(mappings, declaration["languageMappings"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertTrue(
            set(builder["parityEvidenceIds"])
            <= set(declaration["parityEvidenceIds"])
        )
        self.assertIn(PARITY_TEST_EVIDENCE_ID, declaration["parityEvidenceIds"])
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_remaining_import_and_locate_request_members_map_to_existing_builders(self) -> None:
        for operation_key, operation_spec in OPERATIONS.items():
            with self.subTest(operation=operation_spec["name"]):
                operation = self.operations[operation_spec["capability"]]
                builder = self.by_package_name[("c-kmipkit", operation_spec["builder"])]
                constructor = self.by_package_name[
                    ("rust-kmipkit-protocol", operation_spec["constructor"])
                ]
                self.assertEqual(operation_spec["name"], operation["name"])
                self.assertEqual(operation_spec["capability"], builder["capabilityId"])
                self.assertEqual(operation_spec["owner"], builder["approvalSource"]["path"])
                self.assertEqual("Approved", builder["approvalSource"]["status"])

                for language, (package_id, name) in FOREIGN_MAPPINGS[operation_key].items():
                    self.assertEqual(
                        {"packageId": package_id, "name": name},
                        builder["languageMappings"][language],
                    )

                for declaration_id, export_name in operation_spec["targets"]:
                    with self.subTest(declaration=export_name):
                        self._assert_target_mapping(
                            operation_spec,
                            builder,
                            constructor,
                            operation,
                            declaration_id,
                            export_name,
                        )

                self.assertIn(PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"])

    def test_mappings_cite_each_approved_owner_and_exact_oasis_tables(self) -> None:
        for operation_spec in OPERATIONS.values():
            with self.subTest(operation=operation_spec["name"]):
                specification = (ROOT / operation_spec["owner"]).read_text(encoding="utf-8")
                self.assertIn(operation_spec["table_row"], specification)
                source_test = (ROOT / operation_spec["source_test"]).read_text(encoding="utf-8")
                self.assertIn(operation_spec["section"], source_test)


if __name__ == "__main__":
    unittest.main()
