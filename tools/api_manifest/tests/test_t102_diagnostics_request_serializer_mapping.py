"""Pin four-language mappings for Interop and Log request serializers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/028-diagnostics-interop/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_diagnostics_request_serializer_mapping.py"
)
OPERATIONS = {
    "interop": {
        "name": "Interop",
        "capability": "KMIPKIT-ELEM-OP-C2S-INTEROP",
        "builder": "kmipkit_client_batch_item_interop",
        "table_row": "| Interop | KMIPKIT-ELEM-OP-C2S-INTEROP | §6.1.26 | 241 | 242 | 243 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/interop_operation_tests.rs",
        "section": "§6.1.26",
        "target": (
            "rust-observed-df96e7ac424e101c5d7e",
            "kmipkit_protocol::InteropRequest::into_ttlv_payload",
            "kmipkit_protocol::InteropRequest::new",
        ),
    },
    "log": {
        "name": "Log",
        "capability": "KMIPKIT-ELEM-OP-C2S-LOG",
        "builder": "kmipkit_client_batch_item_log",
        "table_row": "| Log | KMIPKIT-ELEM-OP-C2S-LOG | §6.1.29 | 250 | 251 | 252 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/log_operation_tests.rs",
        "section": "§6.1.29",
        "target": (
            "rust-observed-15c579db14e795367130",
            "kmipkit_protocol::LogRequest::into_ttlv_payload",
            "kmipkit_protocol::LogRequest::new",
        ),
    },
}


class T102DiagnosticsRequestSerializerMappingTests(unittest.TestCase):
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

    def _assert_operation_mapping(self, operation_spec: dict[str, object]) -> None:
        builder = self.by_package_name[("c-kmipkit", operation_spec["builder"])]
        operation = self._operation_row(operation_spec["capability"])
        declaration_id, rust_name, reference_name = operation_spec["target"]
        declaration = self.declarations[declaration_id]
        reference = self.by_package_name[("rust-kmipkit-protocol", reference_name)]

        self.assertEqual(operation_spec["name"], operation["name"])
        self.assertEqual(operation_spec["capability"], builder["capabilityId"])
        self.assertEqual(OWNER, builder["approvalSource"]["path"])
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
        self.assertIn(PARITY_TEST_EVIDENCE_ID, declaration["parityEvidenceIds"])
        self.assertIn(declaration_id, operation["declarations"]["rust"])
        self.assertIn(PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"])

    def test_two_diagnostics_request_serializers_map_to_existing_capabilities(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            with self.subTest(operation=operation_name):
                self._assert_operation_mapping(operation_spec)

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
