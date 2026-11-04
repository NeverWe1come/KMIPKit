"""T102 mapping for PKCS#11 request setters.

The mapped payload is defined by OASIS KMIP v2.1 §6.1.37 Table 273,
§7.8, §7.27 Table 379, and §7.28 Table 380. The owner specification
records that §6.1.37 has no separate catalog client clause; these tests
therefore verify the approved operation and binding contract without
inventing a normative requirement.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
T102_CANDIDATES_AFTER_BATCH = 648
OWNER = "specs/026-pkcs11/spec.md"
CAPABILITY = "KMIPKIT-ELEM-OP-C2S-PKCS-11"
EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/Pkcs11MappingTest.java",
    "bindings/python/tests/test_pkcs11_operation.py",
    "crates/kmipkit-client/tests/unit/pkcs11_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/pkcs11_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs",
    "tools/api_manifest/tests/test_t102_pkcs11_request_builder_mapping_batch.py",
}
OWNER_REQUIREMENTS = [
    "KMIPKIT-0040-FR-001",
    "KMIPKIT-0040-FR-004",
    "KMIPKIT-0040-FR-005",
    "KMIPKIT-0040-FR-008",
]
TARGETS = (
    (
        "rust-observed-60273eccedf62e75a65e",
        "rust-kmipkit-client",
        "kmipkit_client::Pkcs11Request::with_interface",
    ),
    (
        "rust-observed-b1e7666a5a57f054a891",
        "rust-kmipkit-client",
        "kmipkit_client::Pkcs11Request::with_correlation_value",
    ),
    (
        "rust-observed-37ff26bed9cff3029512",
        "rust-kmipkit-client",
        "kmipkit_client::Pkcs11Request::with_input_parameters",
    ),
    (
        "rust-observed-837fc199aac73b1ffd00",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Pkcs11Request::with_interface",
    ),
    (
        "rust-observed-cc71aab290dc65b5928f",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Pkcs11Request::with_correlation_value",
    ),
    (
        "rust-observed-573ae1d0de80ffa2884b",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Pkcs11Request::with_input_parameters",
    ),
)


class T102Pkcs11RequestBuilderMappingBatchTests(unittest.TestCase):
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

    def test_only_the_six_pkcs11_request_setters_map_to_the_approved_four_language_api(self) -> None:
        target_ids = {declaration_id for declaration_id, _, _ in TARGETS}
        self.assertEqual(6, len(target_ids))

        operation = next(
            row for row in self.manifest["operations"]
            if row["operationId"] == CAPABILITY
        )
        builder = self.by_name[("c-kmipkit", "kmipkit_client_batch_item_pkcs11")]
        self.assertEqual(CAPABILITY, builder["capabilityId"])
        self.assertEqual(OWNER, builder["approvalSource"]["path"])
        self.assertEqual("Approved", builder["approvalSource"]["status"])

        for declaration_id, rust_package, export_name in TARGETS:
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual(rust_package, declaration["packageId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(CAPABILITY, declaration["capabilityId"])
                self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                api_requirements = [
                    requirement_id
                    for requirement_id in builder["requirementIds"]
                    if requirement_id.startswith("KMIPKIT-0043-")
                ]
                self.assertEqual(
                    OWNER_REQUIREMENTS + api_requirements,
                    declaration["requirementIds"],
                )
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)

                expected_mappings = dict(builder["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": rust_package,
                    "name": export_name,
                }
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertTrue(EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))
                self.assertIn(declaration_id, operation["declarations"]["rust"])

        mapped_targets = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] in target_ids
            and row.get("declarationState") == "verified"
        }
        self.assertEqual(target_ids, mapped_targets)

    def test_only_the_six_pkcs11_setters_leave_the_t102_blocked_inventory(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_CANDIDATES_AFTER_BATCH, blocked)


if __name__ == "__main__":
    unittest.main()
