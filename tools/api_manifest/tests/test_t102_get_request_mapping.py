"""Pin four-language parity for the remaining typed Get request members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GET_BUILDER = ("c-kmipkit", "kmipkit_client_batch_item_get")
GET_CAPABILITY = "KMIPKIT-ELEM-OP-C2S-GET"
GET_OWNER = "specs/017-managed-object-retrieval/spec.md"
REQUIRED_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetLocateResponseMappingTest.java",
    "bindings/python/tests/test_get_locate_operation.py",
    "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/get_tests.rs",
    "tools/api_manifest/tests/test_get_locate_parity_manifest.py",
}
TARGETS = {
    "rust-observed-f6bec1eb3ed6ee5dd803": (
        "kmipkit_protocol::GetRequest::key_compression_type",
        "kmipkit_protocol::GetRequest::with_key_compression_type",
    ),
    "rust-observed-182124c45d14c3d5fa16": (
        "kmipkit_protocol::GetRequest::key_format_type",
        "kmipkit_protocol::GetRequest::with_key_format_type",
    ),
    "rust-observed-ce5becedf774f5eb0306": (
        "kmipkit_protocol::GetRequest::key_wrap_type",
        "kmipkit_protocol::GetRequest::with_key_wrap_type",
    ),
    "rust-observed-8171844c0205eb46b437": (
        "kmipkit_protocol::GetRequest::key_wrapping_specification",
        "kmipkit_protocol::GetRequest::with_key_wrapping_specification",
    ),
    "rust-observed-fdef1e88a66b70a4e60d": (
        "kmipkit_protocol::GetRequest::to_ttlv_payload",
        "kmipkit_protocol::GetRequest::new",
    ),
    "rust-observed-423f4b8bdf2a26f55332": (
        "kmipkit_protocol::GetRequest::unique_identifier",
        "kmipkit_protocol::GetRequest::with_unique_identifier",
    ),
}


class T102GetRequestMappingTests(unittest.TestCase):
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

    def test_only_the_six_remaining_get_request_members_map_to_get_capability(self) -> None:
        builder = self.by_package_name[GET_BUILDER]
        operation = next(
            row for row in self.manifest["operations"]
            if row["operationId"] == GET_CAPABILITY
        )

        for declaration_id, (rust_name, reference_name) in TARGETS.items():
            with self.subTest(declaration_id=declaration_id):
                declaration = self.declarations[declaration_id]
                reference = self.by_package_name[("rust-kmipkit-protocol", reference_name)]
                self.assertEqual(rust_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(GET_CAPABILITY, declaration["capabilityId"])
                self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                self.assertEqual(GET_OWNER, declaration["approvalSource"]["path"])
                self.assertEqual(reference["requirementIds"], declaration["requirementIds"])

                expected_mappings = dict(builder["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": rust_name,
                }
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertTrue(REQUIRED_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertIn(
                    "tools/api_manifest/tests/test_t102_get_request_mapping.py",
                    declaration["parityEvidenceIds"],
                )
                self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_get_request_mapping_cites_its_oasis_tables_and_approved_requirements(self) -> None:
        self.assertTrue(
            {
                "KMIPKIT-REQ-SPEC-6.1.19-001",
                "KMIPKIT-REQ-SPEC-6.1.19-002",
                "KMIPKIT-REQ-SPEC-7.18-001-001",
                "KMIPKIT-REQ-SPEC-7.18-004",
            }
            <= set(self.manifest["requirementIds"])
        )
        specification = (ROOT / GET_OWNER).read_text(encoding="utf-8")
        self.assertIn("OASIS KMIP Specification v2.1 §6.1.19", specification)
        self.assertIn("Tables 220–222", specification)

        source_tests = (ROOT / "crates/kmipkit-protocol/tests/unit/get_tests.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("§6.1.19, Tables 220–221", source_tests)
        self.assertIn("§7.18, Table 370", source_tests)


if __name__ == "__main__":
    unittest.main()
