"""Pin the remaining typed client response-view operation mappings."""

from __future__ import annotations

import json
import unittest
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


@dataclass(frozen=True)
class ExpectedResponseView:
    declaration_id: str
    method: str
    operation_id: str
    section: str
    owner_spec_id: str
    owner_spec_path: str
    table_reference: str
    c_name: str
    java_name: str
    python_name: str


EXPECTED = (
    ExpectedResponseView(
        "rust-observed-4812c1bbb95571e37e86", "delegated_login",
        "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN", "6.1.12", "KMIPKIT-0038",
        "specs/024-credential-sessions/spec.md", "Table 200",
        "kmipkit_client_batch_item_response_delegated_login",
        "org.kmipkit.extensions.ClientBatchItemResult.delegatedLoginResponse",
        "kmipkit.client.BatchItemResult.delegated_login_response",
    ),
    ExpectedResponseView(
        "rust-observed-8c336787323fbacf2a2d", "derive_key",
        "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY", "6.1.14", "KMIPKIT-0037",
        "specs/023-key-material-transfer/spec.md", "Table 206",
        "kmipkit_client_batch_item_response_derive_key",
        "org.kmipkit.extensions.ClientBatchItemResult.deriveKeyResponse",
        "kmipkit.client.BatchItemResult.derive_key_response",
    ),
    ExpectedResponseView(
        "rust-observed-22ecd2fc5338404d1d38", "export",
        "KMIPKIT-ELEM-OP-C2S-EXPORT", "6.1.18", "KMIPKIT-0037",
        "specs/023-key-material-transfer/spec.md", "Table 218",
        "kmipkit_client_batch_item_response_export",
        "org.kmipkit.extensions.ClientBatchItemResult.exportResponse",
        "kmipkit.client.BatchItemResult.export_response",
    ),
    ExpectedResponseView(
        "rust-observed-56d69424436b5bc52713", "get",
        "KMIPKIT-ELEM-OP-C2S-GET", "6.1.19", "KMIPKIT-0017",
        "specs/017-managed-object-retrieval/spec.md", "Table 221",
        "kmipkit_client_batch_item_response_get",
        "org.kmipkit.extensions.ClientBatchItemResult.getResponse",
        "kmipkit.client.BatchItemResult.get_response",
    ),
    ExpectedResponseView(
        "rust-observed-0c30b411196334de84c2", "get_constraints",
        "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS", "6.1.22", "KMIPKIT-0039",
        "specs/025-policy-controls/spec.md", "Table 230",
        "kmipkit_client_batch_item_response_get_constraints_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.getConstraintsResponse",
        "kmipkit.client.BatchItemResult.get_constraints_response",
    ),
    ExpectedResponseView(
        "rust-observed-95f9f5171659f51e66fd", "get_usage_allocation",
        "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION", "6.1.23", "KMIPKIT-0039",
        "specs/025-policy-controls/spec.md", "Table 233",
        "kmipkit_client_batch_item_response_get_usage_allocation",
        "org.kmipkit.extensions.ClientBatchItemResult.getUsageAllocationResponse",
        "kmipkit.client.BatchItemResult.get_usage_allocation_response",
    ),
    ExpectedResponseView(
        "rust-observed-69cf1abee3b40197e4bd", "import",
        "KMIPKIT-ELEM-OP-C2S-IMPORT", "6.1.25", "KMIPKIT-0037",
        "specs/023-key-material-transfer/spec.md", "Table 239",
        "kmipkit_client_batch_item_response_import",
        "org.kmipkit.extensions.ClientBatchItemResult.importResponse",
        "kmipkit.client.BatchItemResult.import_response",
    ),
    ExpectedResponseView(
        "rust-observed-0f0e98b3c48648ff81ca", "interop",
        "KMIPKIT-ELEM-OP-C2S-INTEROP", "6.1.26", "KMIPKIT-0042",
        "specs/028-diagnostics-interop/spec.md", "Table 242",
        "kmipkit_client_batch_item_response_interop_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.interopResponse",
        "kmipkit.client.BatchItemResult.interop_response",
    ),
    ExpectedResponseView(
        "rust-observed-bbf36b0243e4b1d9d72c", "join_split_key",
        "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY", "6.1.27", "KMIPKIT-0037",
        "specs/023-key-material-transfer/spec.md", "Table 245",
        "kmipkit_client_batch_item_response_join_split_key",
        "org.kmipkit.extensions.ClientBatchItemResult.joinSplitKeyResponse",
        "kmipkit.client.BatchItemResult.join_split_key_response",
    ),
    ExpectedResponseView(
        "rust-observed-3af9b2ecd05ddeeb0b4e", "locate",
        "KMIPKIT-ELEM-OP-C2S-LOCATE", "6.1.28", "KMIPKIT-0017",
        "specs/017-managed-object-retrieval/spec.md", "Table 248: Locate Response Payload",
        "kmipkit_client_batch_item_response_locate",
        "org.kmipkit.extensions.ClientBatchItemResult.locateResponse",
        "kmipkit.client.BatchItemResult.locate_response",
    ),
    ExpectedResponseView(
        "rust-observed-141e4937511dd08558c7", "log",
        "KMIPKIT-ELEM-OP-C2S-LOG", "6.1.29", "KMIPKIT-0042",
        "specs/028-diagnostics-interop/spec.md", "Table 251",
        "kmipkit_client_batch_item_response_log_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.logResponse",
        "kmipkit.client.BatchItemResult.log_response",
    ),
    ExpectedResponseView(
        "rust-observed-f3be85c48bde445fdb91", "login",
        "KMIPKIT-ELEM-OP-C2S-LOGIN", "6.1.30", "KMIPKIT-0038",
        "specs/024-credential-sessions/spec.md", "Table 254",
        "kmipkit_client_batch_item_response_login",
        "org.kmipkit.extensions.ClientBatchItemResult.loginResponse",
        "kmipkit.client.BatchItemResult.login_response",
    ),
    ExpectedResponseView(
        "rust-observed-8fcba59b0e2f25ed8415", "logout",
        "KMIPKIT-ELEM-OP-C2S-LOGOUT", "6.1.31", "KMIPKIT-0038",
        "specs/024-credential-sessions/spec.md", "Table 257",
        "kmipkit_client_batch_item_response_logout",
        "org.kmipkit.extensions.ClientBatchItemResult.logoutResponse",
        "kmipkit.client.BatchItemResult.logout_response",
    ),
    ExpectedResponseView(
        "rust-observed-a4870be478a8122547b4", "pkcs11",
        "KMIPKIT-ELEM-OP-C2S-PKCS-11", "6.1.37", "KMIPKIT-0040",
        "specs/026-pkcs11/spec.md", "Table 274",
        "kmipkit_client_batch_item_response_pkcs11_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.pkcs11Response",
        "kmipkit.client.BatchItemResult.pkcs11_response",
    ),
    ExpectedResponseView(
        "rust-observed-3efd3c9084931a73f361", "rng_retrieve",
        "KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE", "6.1.49", "KMIPKIT-0041",
        "specs/027-rng-operations/spec.md", "Table 317",
        "kmipkit_client_batch_item_response_rng_retrieve",
        "org.kmipkit.extensions.ClientBatchItemResult.rngRetrieveResponse",
        "kmipkit.client.BatchItemResult.rng_retrieve_response",
    ),
    ExpectedResponseView(
        "rust-observed-3c2fb9520a6cdfb64f0b", "rng_seed",
        "KMIPKIT-ELEM-OP-C2S-RNG-SEED", "6.1.50", "KMIPKIT-0041",
        "specs/027-rng-operations/spec.md", "Table 320",
        "kmipkit_client_batch_item_response_rng_seed",
        "org.kmipkit.extensions.ClientBatchItemResult.rngSeedResponse",
        "kmipkit.client.BatchItemResult.rng_seed_response",
    ),
    ExpectedResponseView(
        "rust-observed-af5926f653d5286e1739", "set_constraints",
        "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS", "6.1.52", "KMIPKIT-0039",
        "specs/025-policy-controls/spec.md", "Table 326",
        "kmipkit_client_batch_item_response_set_constraints_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.setConstraintsResponse",
        "kmipkit.client.BatchItemResult.set_constraints_response",
    ),
    ExpectedResponseView(
        "rust-observed-bc81c7514e2474dea773", "set_defaults",
        "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS", "6.1.53", "KMIPKIT-0039",
        "specs/025-policy-controls/spec.md", "Table 329",
        "kmipkit_client_batch_item_response_set_defaults_ttlv",
        "org.kmipkit.extensions.ClientBatchItemResult.setDefaultsResponse",
        "kmipkit.client.BatchItemResult.set_defaults_response",
    ),
    ExpectedResponseView(
        "rust-observed-a0692c89fb668b47105f", "validate",
        "KMIPKIT-ELEM-OP-C2S-VALIDATE", "6.1.57", "KMIPKIT-0042",
        "specs/028-diagnostics-interop/spec.md", "Table 341",
        "kmipkit_client_batch_item_response_validate",
        "org.kmipkit.extensions.ClientBatchItemResult.validateResponse",
        "kmipkit.client.BatchItemResult.validate_response",
    ),
)


class T102RemainingResponseViewMappingTests(unittest.TestCase):
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

    def test_exact_response_view_declaration_set(self) -> None:
        expected_names = {
            f"kmipkit_client::ClientResponseView<'a>::{item.method}" for item in EXPECTED
        }
        expected_ids = {item.declaration_id for item in EXPECTED}
        actual_ids = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] in expected_names
        }
        self.assertEqual(expected_ids, actual_ids)

    def test_response_views_map_to_approved_operation_surfaces(self) -> None:
        for item in EXPECTED:
            rust_name = f"kmipkit_client::ClientResponseView<'a>::{item.method}"
            target = self.declarations_by_id[item.declaration_id]
            operation = self.operations[item.operation_id]
            foreign = {
                "c": self.declarations[("c-kmipkit", item.c_name)],
                "java": self.declarations[("java-kmipkit", item.java_name)],
                "python": self.declarations[("python-kmipkit", item.python_name)],
            }
            owner_spec = (ROOT / item.owner_spec_path).read_text(encoding="utf-8")
            mappings = {
                "rust": {"packageId": "rust-kmipkit-client", "name": rust_name},
                "c": {"packageId": "c-kmipkit", "name": item.c_name},
                "java": {"packageId": "java-kmipkit", "name": item.java_name},
                "python": {"packageId": "python-kmipkit", "name": item.python_name},
            }
            with self.subTest(operation=item.operation_id):
                self.assertEqual("verified", target["declarationState"])
                self.assertEqual("associated-item", target["kind"])
                self.assertEqual(item.operation_id, target["capabilityId"])
                self.assertEqual("four-language", target["parityScope"])
                self.assertEqual(mappings, target["languageMappings"])
                self.assertEqual(item.section, operation["section"])
                self.assertEqual(item.owner_spec_id, operation["featureSpecId"])
                self.assertEqual(
                    {"specId": item.owner_spec_id, "state": "approved"},
                    operation["ownerDependency"],
                )
                self.assertEqual(1, operation["declarations"]["rust"].count(item.declaration_id))
                if item.method == "locate":
                    self.assertIn("Tables 247–249", owner_spec)
                else:
                    self.assertIn(item.table_reference, owner_spec)
                self.assertTrue(
                    {"KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"}
                    <= set(target["requirementIds"])
                )
                source = foreign["c"]
                inherited_requirements = set()
                for language, source_row in foreign.items():
                    self.assertEqual("verified", source_row["declarationState"])
                    self.assertEqual(item.operation_id, source_row["capabilityId"])
                    self.assertEqual(source["approvalSource"], source_row["approvalSource"])
                    self.assertEqual(source["parityEvidenceIds"], source_row["parityEvidenceIds"])
                    inherited_requirements.update(source_row["requirementIds"])
                inherited_requirements.update({"KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"})
                self.assertEqual(sorted(inherited_requirements), target["requirementIds"])
                self.assertEqual(source["approvalSource"], target["approvalSource"])
                self.assertEqual(source["parityEvidenceIds"], target["parityEvidenceIds"])

    def test_locate_mapping_cites_only_the_normative_response_payload_table(self) -> None:
        locate = next(item for item in EXPECTED if item.method == "locate")
        owner_spec = (ROOT / locate.owner_spec_path).read_text(encoding="utf-8")
        oasis_source = (
            ROOT
            / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        ).read_text(encoding="latin-1")

        self.assertEqual("Table 248: Locate Response Payload", locate.table_reference)
        self.assertIn("Tables 247–249", owner_spec)
        self.assertIn(locate.table_reference, oasis_source)


if __name__ == "__main__":
    unittest.main()
