"""Exact public-route reconciliation contracts for KMIPKIT-0049."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest import reconcile_0049_managed_objects as reconcile


class Reconcile0049Tests(unittest.TestCase):
    def test_effective_hashing_algorithm_reader_is_in_reviewed_manifest(self) -> None:
        repository_root = Path(__file__).resolve().parents[3]
        manifest = json.loads(
            (repository_root / "specification/api/public-api.json").read_text(encoding="utf-8")
        )
        expected_routes = {
            (
                "rust-kmipkit",
                "kmipkit::protocol::HashedPasswordCredentialObject::effective_hashing_algorithm_raw",
            ),
            (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::HashedPasswordCredentialObject::effective_hashing_algorithm_raw",
            ),
        }
        declared_routes = {
            (row["packageId"], row["exportName"])
            for row in manifest["publicDeclarations"]
        }

        self.assertTrue(expected_routes <= declared_routes)

    def test_adds_exact_blocked_routes_and_trait_exclusions_idempotently(self) -> None:
        manifest = {"requirementIds": [], "publicDeclarations": [], "approvedExclusions": [], "capabilities": []}
        report = {
            "missingFromManifest": [
                {"packageId": "rust-kmipkit-protocol", "kind": "type", "exportName": "kmipkit_protocol::KeyBlock"},
                {"packageId": "rust-kmipkit-protocol", "kind": "associated-item", "exportName": "kmipkit_protocol::KeyBlock::try_new"},
                {"packageId": "rust-kmipkit", "kind": "associated-item", "exportName": "<kmipkit::protocol::KeyBlock as core::fmt::Debug>::fmt"},
            ],
        }

        once = reconcile.reconcile(manifest, report)
        twice = reconcile.reconcile(once, report)

        self.assertEqual(once, twice)
        self.assertEqual(2, len(once["publicDeclarations"]))
        self.assertEqual(1, len(once["approvedExclusions"]))
        self.assertTrue(all(row["declarationState"] == "blocked" for row in once["publicDeclarations"]))
        self.assertEqual("KMIPKIT-0053", once["publicDeclarations"][0]["candidateSpecId"])

    def test_reconciles_kmip30_system_credential_object_routes(self) -> None:
        manifest = {
            "requirementIds": [],
            "publicDeclarations": [],
            "approvedExclusions": [],
            "capabilities": [{
                "capabilityId": reconcile.CAPABILITY_ID,
                "requirementIds": ["KMIPKIT-0049-FR-001"],
                "capabilityState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": "KMIPKIT-0053",
                "blockedReason": reconcile.BLOCKED_REASON,
            }],
        }
        report = {"missingFromManifest": [
            {
                "packageId": "rust-kmipkit-protocol",
                "kind": "type",
                "exportName": "kmipkit_protocol::PasswordCredentialObject",
            },
            {
                "packageId": "rust-kmipkit",
                "kind": "associated-item",
                "exportName": "kmipkit::protocol::PasswordCredentialObject::try_new",
            },
        ]}

        result = reconcile.reconcile(manifest, report)

        self.assertEqual(2, len(result["publicDeclarations"]))
        self.assertIn("KMIPKIT-0049-FR-012", result["requirementIds"])
        self.assertIn("KMIPKIT-0049-FR-013", result["requirementIds"])
        self.assertEqual(
            result["requirementIds"],
            result["capabilities"][0]["requirementIds"],
        )
        for declaration in result["publicDeclarations"]:
            self.assertIn("KMIPKIT-0049-FR-012", declaration["requirementIds"])
            self.assertIn("KMIPKIT-0049-FR-013", declaration["requirementIds"])

    def test_rejects_unowned_rust_routes_instead_of_guessing(self) -> None:
        manifest = {"requirementIds": [], "publicDeclarations": [], "approvedExclusions": [], "capabilities": []}
        report = {"missingFromManifest": [
            {"packageId": "rust-kmipkit", "kind": "type", "exportName": "kmipkit::Client"},
        ]}
        with self.assertRaises(reconcile.ReconciliationError):
            reconcile.reconcile(manifest, report)


if __name__ == "__main__":
    unittest.main()
