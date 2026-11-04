"""Pin four-language parity for the remaining Get Usage Allocation request members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
BUILDER = ("c-kmipkit", "kmipkit_client_batch_item_get_usage_allocation")
CAPABILITY = "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION"
OWNER = "specs/025-policy-controls/spec.md"
NORMATIVE_REQUIREMENTS = {"KMIPKIT-REQ-SPEC-6.1.23-002-001"}
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_get_usage_allocation_request_mapping.py"
)
REQUIRED_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/GetUsageAllocationJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetUsageAllocationResponseMappingTest.java",
    "bindings/python/tests/test_get_usage_allocation_operation.py",
    "crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-ffi/tests/support/get_usage_allocation_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs",
    "tools/api_manifest/tests/test_get_usage_allocation_parity_manifest.py",
    "tools/api_manifest/tests/test_t102_request_builder_mapping_batch.py",
    PARITY_TEST_EVIDENCE_ID,
}
TARGETS = {
    "rust-observed-d26a642304a3935ec483": (
        "kmipkit::GetUsageAllocationRequest::to_ttlv_payload",
        "kmipkit::GetUsageAllocationRequest::new",
    ),
    "rust-observed-fd1593dd30df34f1e29e": (
        "kmipkit::GetUsageAllocationRequest::unique_identifier",
        "kmipkit::GetUsageAllocationRequest::with_unique_identifier",
    ),
    "rust-observed-d9ddf7e087a1a8ab6fdf": (
        "kmipkit_protocol::GetUsageAllocationRequest::to_ttlv_payload",
        "kmipkit_protocol::GetUsageAllocationRequest::new",
    ),
    "rust-observed-2fc8b2a7575195beabba": (
        "kmipkit_protocol::GetUsageAllocationRequest::unique_identifier",
        "kmipkit_protocol::GetUsageAllocationRequest::with_unique_identifier",
    ),
}


class T102GetUsageAllocationRequestMappingTests(unittest.TestCase):
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

    def test_only_the_four_remaining_request_members_map_to_allocation_capability(self) -> None:
        builder = self.by_package_name[BUILDER]
        operation = next(
            row for row in self.manifest["operations"]
            if row["operationId"] == CAPABILITY
        )

        for declaration_id, (rust_name, reference_name) in TARGETS.items():
            with self.subTest(declaration_id=declaration_id):
                declaration = self.declarations[declaration_id]
                reference = self.by_package_name[
                    (declaration["packageId"], reference_name)
                ]
                self.assertEqual(rust_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(CAPABILITY, declaration["capabilityId"])
                self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual(reference["requirementIds"], declaration["requirementIds"])

                expected_mappings = dict(builder["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": rust_name,
                }
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertTrue(REQUIRED_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_mapping_cites_the_approved_owner_and_oasis_request_table(self) -> None:
        self.assertTrue(
            NORMATIVE_REQUIREMENTS
            <= set(self.manifest["requirementIds"])
        )
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn("OASIS Key Management Interoperability Protocol Specification Version 2.1", specification)
        self.assertIn("§6.1.23", specification)
        self.assertIn("232", specification)

        source_tests = (
            ROOT / "crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("§6.1.23", source_tests)
        self.assertIn("Table 232", source_tests)


if __name__ == "__main__":
    unittest.main()
