"""Pin T102 Rust operation-result mappings to shared batch metadata."""

from __future__ import annotations

import json
import unittest
from dataclasses import dataclass
from pathlib import Path

from tools.api_manifest.tests.test_t102_remaining_response_view_mappings import (
    EXPECTED as REMAINING_RESPONSE_VIEWS,
)
from tools.api_manifest.tests.test_t102_response_view_mappings import (
    EXPECTED as RESPONSE_VIEWS,
)
from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


@dataclass(frozen=True)
class ExpectedResult:
    declaration_id: str
    package_id: str
    rust_name: str
    operation_id: str
    owner_spec_id: str
    owner_spec_path: str
    section: str
    response_table: int
    response_view_method: str


OWNER_SPECS = {
    "KMIPKIT-0017": "specs/017-managed-object-retrieval/spec.md",
    "KMIPKIT-0036": "specs/022-object-lifecycle/spec.md",
    "KMIPKIT-0037": "specs/023-key-material-transfer/spec.md",
    "KMIPKIT-0038": "specs/024-credential-sessions/spec.md",
    "KMIPKIT-0040": "specs/026-pkcs11/spec.md",
    "KMIPKIT-0041": "specs/027-rng-operations/spec.md",
}


def _result(
    declaration_id: str,
    package_id: str,
    rust_name: str,
    operation: str,
    owner_spec_id: str,
    section: str,
    response_table: int,
    response_view_method: str,
) -> ExpectedResult:
    return ExpectedResult(
        declaration_id=declaration_id,
        package_id=package_id,
        rust_name=rust_name,
        operation_id=f"KMIPKIT-ELEM-OP-C2S-{operation}",
        owner_spec_id=owner_spec_id,
        owner_spec_path=OWNER_SPECS[owner_spec_id],
        section=section,
        response_table=response_table,
        response_view_method=response_view_method,
    )


EXPECTED = (
    _result("rust-observed-9bbedec79bf9b6bec442", "rust-kmipkit", "kmipkit::DelegatedLoginResponse::result", "DELEGATED-LOGIN", "KMIPKIT-0038", "6.1.12", 200, "delegated_login"),
    _result("rust-observed-be1bd6b12265801ef8c5", "rust-kmipkit-protocol", "kmipkit_protocol::DelegatedLoginResponse::result", "DELEGATED-LOGIN", "KMIPKIT-0038", "6.1.12", 200, "delegated_login"),
    _result("rust-observed-ec1e4d1d9792ab7d23a8", "rust-kmipkit", "kmipkit::LoginResponse::result", "LOGIN", "KMIPKIT-0038", "6.1.30", 254, "login"),
    _result("rust-observed-0b8bdb85a2e3cbcb302d", "rust-kmipkit-protocol", "kmipkit_protocol::LoginResponse::result", "LOGIN", "KMIPKIT-0038", "6.1.30", 254, "login"),
    _result("rust-observed-03821a69a0dc8c40d729", "rust-kmipkit", "kmipkit::RngRetrieveResponse::result", "RNG-RETRIEVE", "KMIPKIT-0041", "6.1.49", 317, "rng_retrieve"),
    _result("rust-observed-5dfeb990a29d05b8b221", "rust-kmipkit-protocol", "kmipkit_protocol::RngRetrieveResponse::result", "RNG-RETRIEVE", "KMIPKIT-0041", "6.1.49", 317, "rng_retrieve"),
    _result("rust-observed-85485afb14fc66be894a", "rust-kmipkit", "kmipkit::RngSeedResponse::result", "RNG-SEED", "KMIPKIT-0041", "6.1.50", 320, "rng_seed"),
    _result("rust-observed-9a3303b7cbab80e0ed92", "rust-kmipkit-protocol", "kmipkit_protocol::RngSeedResponse::result", "RNG-SEED", "KMIPKIT-0041", "6.1.50", 320, "rng_seed"),
    _result("rust-observed-922a653ef9602bf84f38", "rust-kmipkit-client", "kmipkit_client::Pkcs11Response::result", "PKCS-11", "KMIPKIT-0040", "6.1.37", 274, "pkcs11"),
    _result("rust-observed-ff2aa089c2b3f8f847f0", "rust-kmipkit-protocol", "kmipkit_protocol::Pkcs11Response::result", "PKCS-11", "KMIPKIT-0040", "6.1.37", 274, "pkcs11"),
    _result("rust-observed-dd0b5d9e792ef9f1fc91", "rust-kmipkit-protocol", "kmipkit_protocol::CertifyResponse::result", "CERTIFY", "KMIPKIT-0036", "6.1.6", 180, "certify"),
    _result("rust-observed-5b477fc5e9fb005daed7", "rust-kmipkit-protocol", "kmipkit_protocol::CheckResponse::result", "CHECK", "KMIPKIT-0036", "6.1.7", 184, "check"),
    _result("rust-observed-89221bf1ef6ac98cfe5c", "rust-kmipkit-protocol", "kmipkit_protocol::ObtainLeaseResponse::result", "OBTAIN-LEASE", "KMIPKIT-0036", "6.1.35", 269, "obtain_lease"),
    _result("rust-observed-d8b7f8bc246747f7d127", "rust-kmipkit-protocol", "kmipkit_protocol::ReProvisionResponse::result", "RE-PROVISION", "KMIPKIT-0036", "6.1.48", 314, "reprovision"),
    _result("rust-observed-1f59cf7cfe8fbb9bfe63", "rust-kmipkit-protocol", "kmipkit_protocol::RecertifyResponse::result", "RE-CERTIFY", "KMIPKIT-0036", "6.1.45", 301, "recertify"),
    _result("rust-observed-180183b70604b127e90d", "rust-kmipkit-protocol", "kmipkit_protocol::RegisterResponse::result", "REGISTER", "KMIPKIT-0036", "6.1.43", 292, "register"),
    _result("rust-observed-b67075ad68cf5a5efea5", "rust-kmipkit-protocol", "kmipkit_protocol::RekeyKeyPairResponse::result", "RE-KEY-KEY-PAIR", "KMIPKIT-0036", "6.1.47", 311, "rekey_key_pair"),
    _result("rust-observed-a0f8fb989a5a0c680204", "rust-kmipkit-protocol", "kmipkit_protocol::RekeyResponse::result", "RE-KEY", "KMIPKIT-0036", "6.1.46", 306, "rekey"),
    _result("rust-observed-07c66a28a21d97094f6f", "rust-kmipkit-protocol", "kmipkit_protocol::RevokeResponse::result", "REVOKE", "KMIPKIT-0036", "6.1.44", 296, "revoke"),
    _result("rust-observed-36c26bc438116837a515", "rust-kmipkit-protocol", "kmipkit_protocol::DeriveKeyResponse::result", "DERIVE-KEY", "KMIPKIT-0037", "6.1.14", 206, "derive_key"),
    _result("rust-observed-44828be694c580fcebc9", "rust-kmipkit-protocol", "kmipkit_protocol::ExportResponse::result", "EXPORT", "KMIPKIT-0037", "6.1.18", 218, "export"),
    _result("rust-observed-9c12c375463794695915", "rust-kmipkit-protocol", "kmipkit_protocol::ImportResponse::result", "IMPORT", "KMIPKIT-0037", "6.1.25", 239, "import"),
    _result("rust-observed-ae9728ab46ca4fe0b190", "rust-kmipkit-protocol", "kmipkit_protocol::JoinSplitKeyResponse::result", "JOIN-SPLIT-KEY", "KMIPKIT-0037", "6.1.27", 245, "join_split_key"),
    _result("rust-observed-54b32873b562455e7e98", "rust-kmipkit-protocol", "kmipkit_protocol::GetResponse::result", "GET", "KMIPKIT-0017", "6.1.19", 221, "get"),
    _result("rust-observed-3057f3cd6f25b9d165e9", "rust-kmipkit-protocol", "kmipkit_protocol::LocateResponse::result", "LOCATE", "KMIPKIT-0017", "6.1.28", 248, "locate"),
)


SHARED_RESULT_MAPPINGS = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_response_t"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ClientBatchItemResult"},
    "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.BatchItemResult"},
}

RESPONSE_VIEW_CITATIONS = {
    item.method: (item.section, item.response_table) for item in RESPONSE_VIEWS
}
RESPONSE_VIEW_CITATIONS.update(
    {
        item.method: (item.section, int(item.table_reference.split()[1].rstrip(":")))
        for item in REMAINING_RESPONSE_VIEWS
    }
)


class T102OperationResultMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.declarations_by_id = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.operations = {row["operationId"]: row for row in cls.manifest["operations"]}
        cls.common_result = cls.declarations[
            ("rust-kmipkit-client", "kmipkit_client::ClientResponseView::result")
        ]

    def test_shared_metadata_members_remain_available(self) -> None:
        for export in (
            ("c-kmipkit", "kmipkit_client_batch_item_response_metadata_v2"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultStatus"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultReason"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultMessage"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_status"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_reason"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_message"),
        ):
            with self.subTest(export=export):
                self.assertIn(export, self.declarations)

    def test_post_batch_manifest_totals(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(703, T102_BLOCKED_CANDIDATES)
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))

    def test_exact_operation_result_declarations_map_to_shared_metadata(self) -> None:
        self.assertEqual(25, len(EXPECTED))
        self.assertEqual(len(EXPECTED), len({item.declaration_id for item in EXPECTED}))

        for item in EXPECTED:
            operation = self.operations[item.operation_id]
            result = self.declarations_by_id[item.declaration_id]
            view = self.declarations[
                (
                    "rust-kmipkit-client",
                    f"kmipkit_client::ClientResponseView<'a>::{item.response_view_method}",
                )
            ]
            expected_mappings = {
                "rust": {"packageId": item.package_id, "name": item.rust_name},
                **SHARED_RESULT_MAPPINGS,
            }

            with self.subTest(operation=item.operation_id, declaration=item.declaration_id):
                self.assertEqual(
                    (item.section, item.response_table),
                    RESPONSE_VIEW_CITATIONS[item.response_view_method],
                )
                self.assertEqual(item.rust_name, result["exportName"])
                self.assertEqual("associated-item", result["kind"])
                self.assertEqual("verified", result["declarationState"])
                self.assertEqual("four-language", result["parityScope"])
                self.assertEqual(item.operation_id, result["capabilityId"])
                self.assertEqual(expected_mappings, result["languageMappings"])
                for language, mapping in SHARED_RESULT_MAPPINGS.items():
                    self.assertEqual(mapping, self.common_result["languageMappings"][language])
                self.assertEqual(view["requirementIds"], result["requirementIds"])
                self.assertEqual(view["approvalSource"], result["approvalSource"])
                self.assertEqual(view["parityEvidenceIds"], result["parityEvidenceIds"])
                self.assertEqual(item.owner_spec_path, result["approvalSource"]["path"])
                self.assertEqual("Approved", result["approvalSource"]["status"])
                self.assertEqual(item.owner_spec_id, operation["featureSpecId"])
                self.assertEqual(item.section, operation["section"])
                self.assertEqual(
                    {"specId": item.owner_spec_id, "state": "approved"},
                    operation["ownerDependency"],
                )
                self.assertEqual(1, operation["declarations"]["rust"].count(item.declaration_id))
                self.assertIn(view["declarationId"], operation["declarations"]["rust"])
                self.assertEqual("verified", view["declarationState"])


if __name__ == "__main__":
    unittest.main()
