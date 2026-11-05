"""Exact public-route reconciliation contracts for KMIPKIT-0049."""

from __future__ import annotations

import unittest

from tools.api_manifest import reconcile_0049_managed_objects as reconcile


class Reconcile0049Tests(unittest.TestCase):
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

    def test_rejects_unowned_rust_routes_instead_of_guessing(self) -> None:
        manifest = {"requirementIds": [], "publicDeclarations": [], "approvedExclusions": [], "capabilities": []}
        report = {"missingFromManifest": [
            {"packageId": "rust-kmipkit", "kind": "type", "exportName": "kmipkit::Client"},
        ]}
        with self.assertRaises(reconcile.ReconciliationError):
            reconcile.reconcile(manifest, report)


if __name__ == "__main__":
    unittest.main()
