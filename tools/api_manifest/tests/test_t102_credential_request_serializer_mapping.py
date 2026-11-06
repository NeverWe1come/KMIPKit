"""Pin four-language capability mappings for credential request serializers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/024-credential-sessions/spec.md"
PARITY_TEST_EVIDENCE_ID = (
    "tools/api_manifest/tests/test_t102_credential_request_serializer_mapping.py"
)
OPERATIONS = {
    "delegated_login": {
        "name": "Delegated Login",
        "capability": "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN",
        "builder": "kmipkit_client_batch_item_delegated_login",
        "table_row": "| Delegated Login | KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN | §6.1.12 | 199 | 200 | 201 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs",
        "citation": ("§6.1.12", "Tables 199–201"),
        "targets": (
            (
                "rust-observed-6058f94568d4e15fa53e",
                "rust-kmipkit",
                "kmipkit::DelegatedLoginRequest::into_ttlv_payload",
                "kmipkit::DelegatedLoginRequest::new",
            ),
            (
                "rust-observed-923757de0192b8de546c",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::DelegatedLoginRequest::into_ttlv_payload",
                "kmipkit_protocol::DelegatedLoginRequest::new",
            ),
        ),
    },
    "login": {
        "name": "Login",
        "capability": "KMIPKIT-ELEM-OP-C2S-LOGIN",
        "builder": "kmipkit_client_batch_item_login",
        "table_row": "| Login | KMIPKIT-ELEM-OP-C2S-LOGIN | §6.1.30 | 253 | 254 | 255 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/login_operation_tests.rs",
        "citation": ("§6.1.30", "Tables 253–255"),
        "targets": (
            (
                "rust-observed-96756151b5761cf5ed38",
                "rust-kmipkit",
                "kmipkit::LoginRequest::into_ttlv_payload",
                "kmipkit::LoginRequest::new",
            ),
            (
                "rust-observed-ab8fa6fdfc3d8c9d625b",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::LoginRequest::into_ttlv_payload",
                "kmipkit_protocol::LoginRequest::new",
            ),
        ),
    },
    "logout": {
        "name": "Logout",
        "capability": "KMIPKIT-ELEM-OP-C2S-LOGOUT",
        "builder": "kmipkit_client_batch_item_logout",
        "table_row": "| Logout | KMIPKIT-ELEM-OP-C2S-LOGOUT | §6.1.31 | 256 | 257 | 258 |",
        "source_test": "crates/kmipkit-protocol/tests/unit/logout_operation_tests.rs",
        "citation": ("§6.1.31", "Tables 256–258"),
        "targets": (
            (
                "rust-observed-c16d06347ca5d62a3686",
                "rust-kmipkit",
                "kmipkit::LogoutRequest::into_ttlv_payload",
                "kmipkit::LogoutRequest::new",
            ),
            (
                "rust-observed-e3c50df8a6beb061cb4c",
                "rust-kmipkit-protocol",
                "kmipkit_protocol::LogoutRequest::into_ttlv_payload",
                "kmipkit_protocol::LogoutRequest::new",
            ),
        ),
    },
}


class T102CredentialRequestSerializerMappingTests(unittest.TestCase):
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

    def _assert_mapping(
        self,
        operation_spec: dict[str, object],
        builder: dict[str, object],
        declaration_id: str,
        package_id: str,
        rust_name: str,
        reference_name: str,
    ) -> None:
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
        expected_mappings["rust"] = {"packageId": package_id, "name": rust_name}
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertTrue(
            set(builder["parityEvidenceIds"])
            <= set(declaration["parityEvidenceIds"])
        )
        self.assertIn(PARITY_TEST_EVIDENCE_ID, declaration["parityEvidenceIds"])

    def test_six_credential_request_serializers_map_to_existing_operation_capabilities(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            builder = self.by_package_name[("c-kmipkit", operation_spec["builder"])]
            operation = self._operation_row(operation_spec["capability"])
            with self.subTest(operation=operation_name):
                self.assertEqual(operation_spec["name"], operation["name"])
                self.assertEqual(operation_spec["capability"], builder["capabilityId"])
                self.assertEqual(OWNER, builder["approvalSource"]["path"])

                for declaration_id, package_id, rust_name, reference_name in operation_spec[
                    "targets"
                ]:
                    with self.subTest(declaration_id=declaration_id):
                        self._assert_mapping(
                            operation_spec,
                            builder,
                            declaration_id,
                            package_id,
                            rust_name,
                            reference_name,
                        )

    def test_operation_inventories_include_all_six_serializer_declarations(self) -> None:
        for operation_name, operation_spec in OPERATIONS.items():
            operation = self._operation_row(operation_spec["capability"])
            expected_ids = {target[0] for target in operation_spec["targets"]}
            with self.subTest(operation=operation_name):
                self.assertTrue(
                    expected_ids <= set(operation["declarations"]["rust"])
                )
                self.assertIn(
                    PARITY_TEST_EVIDENCE_ID, operation["parityEvidenceIds"]
                )

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
                for citation in operation_spec["citation"]:
                    self.assertIn(citation, source_test)


if __name__ == "__main__":
    unittest.main()
