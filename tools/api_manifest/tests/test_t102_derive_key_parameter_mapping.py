"""Require Derive Key parameter types to map to the four-language API."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import T102_BLOCKED_CANDIDATES

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/023-key-material-transfer/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_derive_key_parameter_mapping.py"
DECLARATIONS = (
    ("rust-observed-6b4e7ba39cb4bd340234", "kmipkit_protocol::DerivationMethod", "type"),
    ("rust-observed-c9c07b2c46007f71efff", "kmipkit_protocol::DerivationMethod::from_raw", "associated-item"),
    ("rust-observed-26622f873c7ff432e698", "kmipkit_protocol::DerivationMethod::raw", "associated-item"),
    ("rust-observed-2a7e6d69030fffb3d878", "kmipkit_protocol::DerivationParameters", "type"),
    ("rust-observed-6180f713348716efda66", "kmipkit_protocol::DerivationParameters::new", "associated-item"),
    ("rust-observed-dc2961ed0d889736434a", "kmipkit_protocol::DerivationParameters::with_cryptographic_parameters", "associated-item"),
    ("rust-observed-03405735e5d8d2625d78", "kmipkit_protocol::DerivationParameters::with_derivation_data", "associated-item"),
    ("rust-observed-802e1ac0317dd6821593", "kmipkit_protocol::DerivationParameters::with_extension", "associated-item"),
    ("rust-observed-7e6811850f3485981f29", "kmipkit_protocol::DerivationParameters::with_initialization_vector", "associated-item"),
    ("rust-observed-27a55bcba38400fa89c6", "kmipkit_protocol::DerivationParameters::with_iteration_count", "associated-item"),
    ("rust-observed-c80c500a2247a0f2eeab", "kmipkit_protocol::DerivationParameters::with_salt", "associated-item"),
)


class T102DeriveKeyParameterMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.anchor = next(
            row
            for row in cls.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] == "kmipkit_client::Client::derive_key"
        )
        cls.operation = next(
            row
            for row in cls.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )

    def test_derivation_method_and_parameters_map_to_the_existing_builders(self) -> None:
        for declaration_id, name, kind in DECLARATIONS:
            with self.subTest(declaration=name):
                declaration = self.declarations[declaration_id]
                expected_mappings = copy.deepcopy(self.anchor["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": name,
                }

                self.assertEqual(name, declaration["exportName"])
                self.assertEqual(kind, declaration["kind"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual(self.anchor["requirementIds"], declaration["requirementIds"])
                self.assertEqual(self.anchor["approvalSource"], declaration["approvalSource"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(
                    self.anchor["parityEvidenceIds"], declaration["parityEvidenceIds"]
                )
                self.assertNotIn("blockedReason", declaration)
                self.assertNotIn("candidateSpecId", declaration)
                self.assertIn(declaration_id, self.operation["declarations"]["rust"])
                self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_foreign_builders_accept_the_generic_derivation_structure(self) -> None:
        owner = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Derive Key | KMIPKIT-ELEM-OP-C2S-DERIVE-KEY | §6.1.14 | 205 | 206 | 207 |",
            owner,
        )
        self.assertIn("§7.12, Table 364", owner)

        c_header = (ROOT / "bindings/c/include/kmipkit.h").read_text(encoding="utf-8")
        self.assertIn("uint32_t derivation_method", c_header)
        self.assertIn("kmipkit_ttlv_structure_t * derivation_parameters", c_header)

        java_source = (
            ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java"
        ).read_text(encoding="utf-8")
        self.assertIn("long derivationMethod", java_source)
        self.assertIn("TtlvStructure derivationParameters", java_source)

        python_source = (ROOT / "bindings/python/src/kmipkit/extensions.py").read_text(
            encoding="utf-8"
        )
        self.assertIn("derivation_method: int", python_source)
        self.assertIn("derivation_parameters: ttlv.Structure", python_source)
        self.assertIn("DERIVATION_DATA", (ROOT / "crates/kmipkit-protocol/src/derive_key.rs").read_text(encoding="utf-8"))
        self.assertIn("request_encodes_required_table_205_fields_in_order_and_repeats_identifiers", (ROOT / "crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs").read_text(encoding="utf-8"))
        self.assertEqual(
            T102_BLOCKED_CANDIDATES,
            sum(
                row.get("declarationState") == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in self.manifest["publicDeclarations"]
            ),
        )


if __name__ == "__main__":
    unittest.main()
