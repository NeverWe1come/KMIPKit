"""Pin four-language capability mappings for PKCS#11 request serializers.

The payload is the client request structure in OASIS KMIP v2.1 §6.1.37,
Table 273. The approved owner records that the operation has no direct catalog
client clause, so this mapping does not invent a normative requirement.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/026-pkcs11/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_pkcs11_request_serializer_mapping.py"
)
CAPABILITY = "KMIPKIT-ELEM-OP-C2S-PKCS-11"
TARGETS = (
    (
        "rust-observed-dc0d9f8db2cc819081a5",
        "rust-kmipkit-client",
        "kmipkit_client::Pkcs11Request::into_ttlv_payload",
        "kmipkit_client::Pkcs11Request::new",
    ),
    (
        "rust-observed-0ca4edbce90ed942c680",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Pkcs11Request::into_ttlv_payload",
        "kmipkit_protocol::Pkcs11Request::new",
    ),
)


class T102Pkcs11RequestSerializerMappingTests(unittest.TestCase):
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

    def _assert_target_mapping(
        self,
        target: tuple[str, str, str, str],
        builder: dict[str, object],
        operation: dict[str, object],
    ) -> None:
        declaration_id, rust_package, rust_name, constructor_name = target
        declaration = self.declarations[declaration_id]
        constructor = self.by_package_name[(rust_package, constructor_name)]
        self.assertEqual(rust_name, declaration["exportName"])
        self.assertEqual(rust_package, declaration["packageId"])
        self.assertEqual("associated-item", declaration["kind"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertNotIn("blockedReason", declaration)
        self.assertEqual(CAPABILITY, declaration["capabilityId"])
        self.assertEqual(constructor["approvalSource"], declaration["approvalSource"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])

        expected_mappings = dict(builder["languageMappings"])
        expected_mappings["rust"] = {"packageId": rust_package, "name": rust_name}
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertTrue(
            set(builder["parityEvidenceIds"])
            <= set(declaration["parityEvidenceIds"])
        )
        self.assertIn(PARITY_TEST_EVIDENCE_ID, declaration["parityEvidenceIds"])
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_both_pkcs11_request_serializers_map_to_the_approved_capability(self) -> None:
        builder = self.by_package_name[("c-kmipkit", "kmipkit_client_batch_item_pkcs11")]
        operation = next(
            row
            for row in self.manifest["operations"]
            if row["operationId"] == CAPABILITY
        )
        self.assertEqual(OWNER, builder["approvalSource"]["path"])
        self.assertEqual("Approved", builder["approvalSource"]["status"])

        for target in TARGETS:
            _, _, rust_name, _ = target
            with self.subTest(declaration=rust_name):
                self._assert_target_mapping(target, builder, operation)

        self.assertIn(PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"])

    def test_mapping_cites_the_approved_owner_and_exact_oasis_table(self) -> None:
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "OASIS Key Management Interoperability Protocol Specification Version 2.1",
            specification,
        )
        self.assertIn(
            "| PKCS#11 | KMIPKIT-ELEM-OP-C2S-PKCS-11 | §6.1.37 | 273 | 274 | 275 |",
            specification,
        )
        source_test = (
            ROOT / "crates/kmipkit-protocol/tests/unit/pkcs11_operation_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("§6.1.37 Tables 273–275", source_test)


if __name__ == "__main__":
    unittest.main()
