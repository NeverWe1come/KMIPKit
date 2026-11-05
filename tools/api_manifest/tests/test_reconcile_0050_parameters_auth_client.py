"""Exact Rust API reconciliation contracts for KMIPKIT-0050."""

from __future__ import annotations

import unittest

from tools.api_manifest import reconcile_0050_parameters_auth_client as reconcile


def _manifest() -> dict:
    declarations = [
        {
            "declarationId": "get-c",
            "packageId": "c-kmipkit",
            "exportName": "kmipkit_client_batch_item_get",
            "languageMappings": {"c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_get"}},
        },
        {
            "declarationId": "get-java",
            "packageId": "java-kmipkit",
            "exportName": "org.kmipkit.extensions.ClientBatchItem.get",
            "languageMappings": {"java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ClientBatchItem.get"}},
        },
        {
            "declarationId": "get-python",
            "packageId": "python-kmipkit",
            "exportName": "kmipkit.extensions.client_batch_item_get",
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


if __name__ == "__main__":
    unittest.main()
