"""Tests for deterministic KMIPKIT-0052 manifest reconciliation."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest.reconcile_0052_response_navigation import reconcile


class Reconcile0052ResponseNavigationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "requirementIds": [],
            "functions": [],
            "publicDeclarations": [],
            "capabilities": [],
            "approvedExclusions": [],
        }

    def test_adds_response_contract_routes_and_exact_bridge_exclusions(self) -> None:
        result = reconcile(self.manifest)

        self.assertEqual(len(result["functions"]), 2)
        self.assertEqual(len(result["publicDeclarations"]), 28)
        self.assertEqual(
            {row["exportName"] for row in result["approvedExclusions"]},
            {
                "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseRawTtlv",
                "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseHeaderTtlv",
            },
        )
        capability = result["capabilities"][0]
        self.assertEqual(capability["capabilityId"], "response_ttlv_navigation")
        self.assertIn("KMIPKIT-0052-FR-001", capability["requirementIds"])
        self.assertIn(
            "kmipkit::ClientBatchItemResponse::raw", capability["mappings"]["rust"]
        )
        self.assertIn(
            "kmipkit_client_batch_item_response_raw_ttlv", capability["mappings"]["c"]
        )

    def test_reconciliation_is_deterministic_and_does_not_mutate_input(self) -> None:
        original = copy.deepcopy(self.manifest)

        first = reconcile(self.manifest)
        second = reconcile(self.manifest)

        self.assertEqual(first, second)
        self.assertEqual(self.manifest, original)

    def test_rejects_an_existing_route_with_a_different_contract(self) -> None:
        self.manifest["publicDeclarations"] = [{
            "declarationId": "kmipkit-0052-response-payload-rust",
            "packageId": "rust-kmipkit-client",
            "kind": "associated-item",
            "exportName": "kmipkit_client::ClientBatchItemResponse::raw",
            "capabilityId": "unexpected-capability",
            "requirementIds": [],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/052-response-navigation/spec.md",
                "status": "Approved",
                "evidence": "Approved",
            },
            "languageMappings": {},
            "parityEvidenceIds": [],
        }]

        with self.assertRaisesRegex(ValueError, "unexpected existing declaration contract"):
            reconcile(self.manifest)


if __name__ == "__main__":
    unittest.main()
