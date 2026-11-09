"""Exact Rust API reconciliation contracts for KMIPKIT-0050."""

from __future__ import annotations

import unittest
import re
from pathlib import Path

from tools.api_manifest import reconcile_0050_parameters_auth_client as reconcile


def _manifest() -> dict:
    declarations = [
        {
            "declarationId": "get-c",
            "packageId": "c-kmipkit",
            "exportName": "kmipkit_client_batch_item_get",
            "capabilityId": "KMIPKIT-ELEM-OP-C2S-GET",
            "languageMappings": {"c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_get"}},
        },
        {
            "declarationId": "get-java",
            "packageId": "java-kmipkit",
            "exportName": "org.kmipkit.extensions.ClientBatchItem.get",
            "capabilityId": "KMIPKIT-ELEM-OP-C2S-GET",
            "languageMappings": {"java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ClientBatchItem.get"}},
        },
        {
            "declarationId": "get-python",
            "packageId": "python-kmipkit",
            "exportName": "kmipkit.extensions.client_batch_item_get",
            "capabilityId": "KMIPKIT-ELEM-OP-C2S-GET",
            "languageMappings": {"python": {"packageId": "python-kmipkit", "name": "kmipkit.extensions.client_batch_item_get"}},
        },
    ]
    return {
        "requirementIds": [],
        "operations": [
            {
                "operationId": "KMIPKIT-ELEM-OP-C2S-GET",
                "name": "Get",
                "apiState": "verified",
                "approvalEvidence": {
                    "kind": "feature-specification",
                    "path": "specs/017-managed-object-retrieval/spec.md",
                    "status": "Approved",
                    "evidence": "Approved for implementation by the maintainer.",
                },
                "declarations": {
                    "rust": [],
                    "c": ["get-c"],
                    "java": ["get-java"],
                    "python": ["get-python"],
                },
                "parityEvidenceIds": ["tests/get.rs"],
            }
        ],
        "publicDeclarations": declarations,
        "approvedExclusions": [],
        "capabilities": [],
    }


class Reconcile0050Tests(unittest.TestCase):
    def test_maps_direct_methods_blocks_cross_language_gaps_and_excludes_exact_traits(self) -> None:
        report = {
            "missingFromManifest": [
                {"packageId": "rust-kmipkit-client", "kind": "associated-item", "exportName": "kmipkit_client::Client::get"},
                {"packageId": "rust-kmipkit", "kind": "associated-item", "exportName": "kmipkit::Client::get_with_options"},
                {"packageId": "rust-kmipkit-protocol", "kind": "type", "exportName": "kmipkit_protocol::CryptographicParameters"},
                {"packageId": "rust-kmipkit", "kind": "associated-item", "exportName": "kmipkit::protocol::EncryptRequest::with_typed_cryptographic_parameters"},
                {"packageId": "rust-kmipkit-client", "kind": "associated-item", "exportName": "kmipkit_client::ClientBatch::with_authentication"},
                {"packageId": "rust-kmipkit-protocol", "kind": "associated-item", "exportName": "<kmipkit_protocol::CryptographicParameters as core::fmt::Debug>::fmt"},
            ],
        }

        result = reconcile.reconcile(_manifest(), report)
        rows = {row["exportName"]: row for row in result["publicDeclarations"]}
        exclusions = {row["exportName"]: row for row in result["approvedExclusions"]}

        direct = rows["kmipkit_client::Client::get"]
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-GET", direct["capabilityId"])
        self.assertEqual("verified", direct["declarationState"])
        self.assertEqual("org.kmipkit.extensions.ClientBatchItem.get", direct["languageMappings"]["java"]["name"])
        self.assertEqual("kmipkit.extensions.client_batch_item_get", direct["languageMappings"]["python"]["name"])
        self.assertIn("KMIPKIT-0050-FR-011", direct["requirementIds"])

        typed = rows["kmipkit_protocol::CryptographicParameters"]
        self.assertEqual("typed_cryptographic_parameters", typed["capabilityId"])
        self.assertEqual("blocked", typed["declarationState"])
        self.assertEqual("KMIPKIT-0053", typed["candidateSpecId"])

        auth = rows["kmipkit_client::ClientBatch::with_authentication"]
        self.assertEqual("request_message_authentication", auth["capabilityId"])
        self.assertEqual("blocked", auth["declarationState"])

        trait = exclusions["<kmipkit_protocol::CryptographicParameters as core::fmt::Debug>::fmt"]
        self.assertEqual("KMIPKIT-0050", trait["candidateSpecId"])
        self.assertNotIn(trait["exportName"], rows)

    def test_reconciliation_is_idempotent_and_rejects_unknown_exports(self) -> None:
        report = {"missingFromManifest": [
            {"packageId": "rust-kmipkit-client", "kind": "associated-item", "exportName": "kmipkit_client::Client::get"},
        ]}
        once = reconcile.reconcile(_manifest(), report)
        self.assertEqual(once, reconcile.reconcile(once, report))

        invalid = {"missingFromManifest": [
            {"packageId": "rust-kmipkit", "kind": "type", "exportName": "kmipkit::Client::new_unknown"},
        ]}
        with self.assertRaises(reconcile.ReconciliationError):
            reconcile.reconcile(_manifest(), invalid)

    def test_direct_mapping_ignores_unlisted_metadata_and_prefix_lookalikes(self) -> None:
        manifest = _manifest()
        lookalikes = {
            "c": "kmipkit_client_batch_item_metadata_v2_t.operation",
            "java": "org.kmipkit.extensions.ClientBatchItem.unrelated",
            "python": "kmipkit.extensions.client_batch_item_unrelated",
        }
        for language, name in lookalikes.items():
            manifest["publicDeclarations"].insert(0, {
                "declarationId": f"unlisted-{language}",
                "capabilityId": "KMIPKIT-ELEM-OP-C2S-GET",
                "languageMappings": {language: {"packageId": f"{language}-kmipkit", "name": name}},
            })
        report = {"missingFromManifest": [{
            "packageId": "rust-kmipkit-client",
            "kind": "associated-item",
            "exportName": "kmipkit_client::Client::get",
        }]}

        result = reconcile.reconcile(manifest, report)
        direct = result["publicDeclarations"][-1]

        for language, expected in {
            "c": "kmipkit_client_batch_item_get",
            "java": "org.kmipkit.extensions.ClientBatchItem.get",
            "python": "kmipkit.extensions.client_batch_item_get",
        }.items():
            with self.subTest(language=language):
                self.assertEqual(expected, direct["languageMappings"][language]["name"])

    def test_direct_mapping_rejects_missing_exact_batch_constructor(self) -> None:
        manifest = _manifest()
        manifest["publicDeclarations"][0]["languageMappings"]["c"]["name"] = "kmipkit_client_batch_item_metadata_v2_t.operation"
        report = {"missingFromManifest": [{
            "packageId": "rust-kmipkit-client",
            "kind": "associated-item",
            "exportName": "kmipkit_client::Client::get",
        }]}

        with self.assertRaises(reconcile.ReconciliationError):
            reconcile.reconcile(manifest, report)

    def test_reconciliation_repairs_its_stale_approval_evidence(self) -> None:
        manifest = _manifest()
        stale_source = {
            "kind": "feature-specification",
            "path": "specs/050-parameters-auth-client/spec.md",
            "status": "Approved",
            "evidence": "The maintainer directly approved the KMIPKIT-0050 implementation plan.",
        }
        manifest["publicDeclarations"][0]["approvalSource"] = stale_source
        result = reconcile.reconcile(manifest, {"missingFromManifest": []})
        self.assertEqual(
            manifest["operations"][0]["approvalEvidence"],
            result["publicDeclarations"][0]["approvalSource"],
        )


    def test_kmip30_operation_references_use_operation_enumeration_table_584(self) -> None:
        root = Path(__file__).resolve().parents[3]
        paths = (
            "crates/kmipkit-client/src/execute.rs",
            "crates/kmipkit-protocol/tests/unit/kmip30_operations_tests.rs",
            "crates/kmipkit-client/tests/unit/kmip30_operation_surface_tests.rs",
            "specs/050-parameters-auth-client/spec.md",
            "specs/050-parameters-auth-client/traceability.md",
            "specs/050-parameters-auth-client/implementation-evidence.md",
        )
        for relative_path in paths:
            with self.subTest(path=relative_path):
                content = (root / relative_path).read_text(encoding="utf-8")
                self.assertIsNone(re.search(r"§11\.37,?\s+Table 583", content), relative_path)
                self.assertTrue("Table 584" in content, relative_path)


if __name__ == "__main__":
    unittest.main()
