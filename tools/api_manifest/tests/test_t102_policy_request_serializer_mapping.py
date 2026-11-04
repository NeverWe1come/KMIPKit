"""Pin four-language capability mappings for policy request serializers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/025-policy-controls/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_policy_request_serializer_mapping.py"
)
OPERATIONS = {
    "get_constraints": {
        "name": "Get Constraints",
        "capability": "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS",
        "builder": "kmipkit_client_batch_item_get_constraints",
        "table_row": "| Get Constraints | KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS | §6.1.22 | 229 | 230 | 231 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/get_constraints_tests.rs",
        "citation": ("§6.1.22", "Tables 229–231"),
        "targets": (
            (
                "rust-observed-9f1aa2332742556e5b24",
                "rust-kmipkit",
                "kmipkit::GetConstraintsRequest::to_ttlv_payload",
                "kmipkit::GetConstraintsRequest::new",
            ),
            (
                "rust-observed-0acd1d324fb7e26bfe6e",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::GetConstraintsRequest::to_ttlv_payload",
                "kmipkit_protocol::GetConstraintsRequest::new",
            ),
        ),
    },
    "set_constraints": {
        "name": "Set Constraints",
        "capability": "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS",
        "builder": "kmipkit_client_batch_item_set_constraints",
        "table_row": "| Set Constraints | KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS | §6.1.52 | 325 | 326 | 327 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/set_constraints_tests.rs",
        "citation": ("§6.1.52", "Tables 325–327"),
        "targets": (
            (
                "rust-observed-480a7b8eada6ab8078aa",
                "rust-kmipkit",
                "kmipkit::SetConstraintsRequest::to_ttlv_payload",
                "kmipkit::SetConstraintsRequest::new",
            ),
            (
                "rust-observed-c7d39ab287f72cc407c3",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::SetConstraintsRequest::to_ttlv_payload",
                "kmipkit_protocol::SetConstraintsRequest::new",
            ),
        ),
    },
    "set_defaults": {
        "name": "Set Defaults",
        "capability": "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS",
        "builder": "kmipkit_client_batch_item_set_defaults",
        "table_row": "| Set Defaults | KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS | §6.1.53 | 328 | 329 | 330 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs",
        "citation": ("§6.1.53", "Tables 328–330"),
        "targets": (
            (
                "rust-observed-406f1ab5b147b43150b0",
                "rust-kmipkit",
                "kmipkit::SetDefaultsRequest::to_ttlv_payload",
                "kmipkit::SetDefaultsRequest::new",
            ),
            (
                "rust-observed-d3c9f0f386e25d092f21",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::SetDefaultsRequest::to_ttlv_payload",
                "kmipkit_protocol::SetDefaultsRequest::new",
            ),
        ),
    },
}


class T102PolicyRequestSerializerMappingTests(unittest.TestCase):
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

    def test_only_the_six_policy_request_serializers_map_to_their_operation(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            builder = self.by_package_name[("c-kmipkit", operation_spec["builder"])]
            with self.subTest(operation=operation_name):
                self.assertEqual(operation_spec["capability"], builder["capabilityId"])
                self.assertEqual(OWNER, builder["approvalSource"]["path"])

                for declaration_id, package_id, rust_name, reference_name in operation_spec[
                    "targets"
                ]:
                    with self.subTest(declaration_id=declaration_id):
                        declaration = self.declarations[declaration_id]
                        reference = self.by_package_name[(package_id, reference_name)]
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
                            "packageId": package_id,
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

    def test_operation_inventories_include_all_six_serializer_declarations(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            operation = next(
                row
                for row in self.manifest["operations"]
                if row["operationId"] == operation_spec["capability"]
            )
            expected_ids = {
                target[0] for target in operation_spec["targets"]
            }
            with self.subTest(operation=operation_name):
                self.assertTrue(
                    expected_ids <= set(operation["declarations"]["rust"])
                )
                self.assertIn(
                    PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"]
                )

    def test_mapping_contract_cites_the_approved_owner_and_exact_oasis_tables(self) -> None:
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        source = "OASIS Key Management Interoperability Protocol Specification Version 2.1"
        self.assertIn(source, specification)
        for operation_name, operation_spec in OPERATIONS.items():
            with self.subTest(operation=operation_name):
                self.assertIn(operation_spec["table_row"], specification)
                source_test = (ROOT / operation_spec["source_test"]).read_text(
                    encoding="utf-8"
                )
                for citation in operation_spec["citation"]:
                    self.assertIn(citation, source_test)


if __name__ == "__main__":
    unittest.main()
