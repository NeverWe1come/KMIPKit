"""Pin four-language capability mappings for lifecycle request serializers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/022-object-lifecycle/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_lifecycle_request_serializer_mapping.py"
)
OPERATIONS = {
    "certify": {
        "name": "Certify",
        "capability": "KMIPKIT-ELEM-OP-C2S-CERTIFY",
        "builder": "kmipkit_client_batch_item_certify",
        "table_row": "| Certify | KMIPKIT-ELEM-OP-C2S-CERTIFY | §6.1.6 | 179 | 180 | 181 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs",
        "section": "§6.1.6",
        "target": (
            "rust-observed-455bf70fd49c2d1c0584",
            "kmipkit_protocol::CertifyRequest::into_ttlv_payload",
            "kmipkit_protocol::CertifyRequest::new",
        ),
    },
    "check": {
        "name": "Check",
        "capability": "KMIPKIT-ELEM-OP-C2S-CHECK",
        "builder": "kmipkit_client_batch_item_check",
        "table_row": "| Check | KMIPKIT-ELEM-OP-C2S-CHECK | §6.1.7 | 183 | 184 | 185 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/check_operation_tests.rs",
        "section": "§6.1.7",
        "target": (
            "rust-observed-1aa8855f67251843003b",
            "kmipkit_protocol::CheckRequest::into_ttlv_payload",
            "kmipkit_protocol::CheckRequest::new",
        ),
    },
    "obtain_lease": {
        "name": "Obtain Lease",
        "capability": "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
        "builder": "kmipkit_client_batch_item_obtain_lease",
        "table_row": "| Obtain Lease | KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE | §6.1.35 | 268 | 269 | 270 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs",
        "section": "§6.1.35",
        "target": (
            "rust-observed-3f7281ffc6f28316cdb2",
            "kmipkit_protocol::ObtainLeaseRequest::into_ttlv_payload",
            "kmipkit_protocol::ObtainLeaseRequest::new",
        ),
    },
    "recertify": {
        "name": "Re-certify",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        "builder": "kmipkit_client_batch_item_recertify",
        "table_row": "| Re-certify | KMIPKIT-ELEM-OP-C2S-RE-CERTIFY | §6.1.45 | 300 | 301 | 302 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
        "section": "§6.1.45",
        "target": (
            "rust-observed-d7bf84b1a5fd4c705597",
            "kmipkit_protocol::RecertifyRequest::into_ttlv_payload",
            "kmipkit_protocol::RecertifyRequest::new",
        ),
    },
    "rekey": {
        "name": "Re-key",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "builder": "kmipkit_client_batch_item_rekey",
        "table_row": "| Re-key | KMIPKIT-ELEM-OP-C2S-RE-KEY | §6.1.46 | 305 | 306 | 307 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
        "section": "§6.1.46",
        "target": (
            "rust-observed-1477c49c457fdc10fa5b",
            "kmipkit_protocol::RekeyRequest::into_ttlv_payload",
            "kmipkit_protocol::RekeyRequest::new",
        ),
    },
    "rekey_key_pair": {
        "name": "Re-key Key Pair",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
        "builder": "kmipkit_client_batch_item_rekey_key_pair",
        "table_row": "| Re-key Key Pair | KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR | §6.1.47 | 310 | 311 | 312 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs",
        "section": "§6.1.47",
        "target": (
            "rust-observed-92442b7d3422ffe465c3",
            "kmipkit_protocol::RekeyKeyPairRequest::into_ttlv_payload",
            "kmipkit_protocol::RekeyKeyPairRequest::new",
        ),
    },
    "reprovision": {
        "name": "Re-Provision",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-PROVISION",
        "builder": "kmipkit_client_batch_item_reprovision",
        "table_row": "| Re-Provision | KMIPKIT-ELEM-OP-C2S-RE-PROVISION | §6.1.48 | 313 | 314 | 315 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/reprovision_operation_tests.rs",
        "section": "§6.1.48",
        "target": (
            "rust-observed-d1446684e4f2641b3c96",
            "kmipkit_protocol::ReProvisionRequest::into_ttlv_payload",
            "kmipkit_protocol::ReProvisionRequest::new",
        ),
    },
    "revoke": {
        "name": "Revoke",
        "capability": "KMIPKIT-ELEM-OP-C2S-REVOKE",
        "builder": "kmipkit_client_batch_item_revoke",
        "table_row": "| Revoke | KMIPKIT-ELEM-OP-C2S-REVOKE | §6.1.44 | 295 | 296 | 297 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs",
        "section": "§6.1.44",
        "target": (
            "rust-observed-3110caf46cc1e36cbcc9",
            "kmipkit_protocol::RevokeRequest::into_ttlv_payload",
            "kmipkit_protocol::RevokeRequest::new",
        ),
    },
}


class T102LifecycleRequestSerializerMappingTests(unittest.TestCase):
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

    def test_eight_lifecycle_request_serializers_map_to_existing_capabilities(self) -> None:
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
                self.assertIn(PARITY_TEST_EVIDENCE_ID, declaration["parityEvidenceIds"])
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
