import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import reconcile_0053_api_parity


class Reconcile0053ApiParityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "requirementIds": [],
            "functions": [],
            "publicDeclarations": [
                {
                    "declarationId": "rust-attribute-type",
                    "packageId": "rust-kmipkit",
                    "kind": "type",
                    "exportName": "kmipkit::protocol::Attribute",
                    "capabilityId": "typed_object_attributes",
                    "requirementIds": ["KMIPKIT-0048-FR-001"],
                    "declarationState": "blocked",
                    "parityScope": "four-language",
                    "candidateSpecId": "KMIPKIT-0053",
                    "blockedReason": "foreign parity is pending",
                },
                {
                    "declarationId": "rust-authentication",
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "kmipkit::ClientBatch::with_authentication",
                    "capabilityId": "request_message_authentication",
                    "requirementIds": ["KMIPKIT-0050-FR-004"],
                    "declarationState": "blocked",
                    "parityScope": "four-language",
                    "candidateSpecId": "KMIPKIT-0053",
                    "blockedReason": "foreign parity is pending",
                },
            ],
            "approvedExclusions": [],
            "capabilities": [
                {
                    "capabilityId": "typed_object_attributes",
                    "capabilityState": "blocked",
                    "parityScope": "four-language",
                    "ownerSpecId": "KMIPKIT-0048",
                },
                {
                    "capabilityId": "request_message_authentication",
                    "capabilityState": "blocked",
                    "parityScope": "four-language",
                    "ownerSpecId": "KMIPKIT-0050",
                },
            ],
        }
        self.routes = {
            "rust-attribute-type": {
                "rust": {"packageId": "rust-kmipkit", "name": "Attribute"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_typed_attribute"},
                "java": {"packageId": "java-kmipkit", "name": "TypedAttribute"},
                "python": {"packageId": "python-kmipkit", "name": "TypedAttribute"},
                "parityEvidenceIds": ["bindings/c/tests/typed_attribute_consumer.c"],
                "approvalSource": {
                    "kind": "feature-specification",
                    "path": "specs/048-typed-attributes/approval-record.md",
                    "status": "Approved",
                    "evidence": "direct approval of the Phase 5 plan",
                },
            },
            "rust-authentication": {
                "rust": {"packageId": "rust-kmipkit", "name": "ClientBatch::with_authentication"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_with_authentication"},
                "java": {"packageId": "java-kmipkit", "name": "ClientBatch.withAuthentication"},
                "python": {"packageId": "python-kmipkit", "name": "ClientBatch.with_authentication"},
                "parityEvidenceIds": ["bindings/c/tests/authentication_consumer.c"],
                "approvalSource": {
                    "kind": "feature-specification",
                    "path": "specs/050-cryptographic-parameters/approval-record.md",
                    "status": "Approved",
                    "evidence": "direct approval of the Phase 5 plan",
                },
            },
        }

    def test_reconciles_exact_four_language_routes_and_capabilities(self) -> None:
        result = reconcile_0053_api_parity.reconcile_manifest(self.manifest, self.routes)

        self.assertEqual("blocked", self.manifest["publicDeclarations"][0]["declarationState"])
        for declaration in result["publicDeclarations"]:
            self.assertEqual("verified", declaration["declarationState"])
            self.assertNotIn("candidateSpecId", declaration)
            self.assertNotIn("blockedReason", declaration)
            self.assertEqual({"rust", "c", "java", "python"}, set(declaration["languageMappings"]))
        self.assertTrue(all(item["capabilityState"] == "verified" for item in result["capabilities"]))

    def test_rejects_missing_language_or_evidence_mapping(self) -> None:
        routes = copy.deepcopy(self.routes)
        del routes["rust-attribute-type"]["python"]

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

    def test_reconciles_one_audited_candidate_without_unblocking_unrelated_routes(self) -> None:
        routes = {"rust-attribute-type": self.routes["rust-attribute-type"]}

        result = reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

        states = {row["declarationId"]: row["declarationState"]
                  for row in result["publicDeclarations"]}
        self.assertEqual("verified", states["rust-attribute-type"])
        self.assertEqual("blocked", states["rust-authentication"])
        capabilities = {row["capabilityId"]: row["capabilityState"]
                        for row in result["capabilities"]}
        self.assertEqual("verified", capabilities["typed_object_attributes"])
        self.assertEqual("blocked", capabilities["request_message_authentication"])

    def test_rejects_a_general_exclusion_for_typed_capabilities(self) -> None:
        routes = copy.deepcopy(self.routes)
        routes["rust-attribute-type"] = {
            "exclusion": {
                "reason": "foreign APIs are not supported",
                "evidence": ["specs/053-api-parity-facade/spec.md"],
            }
        }

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

    def test_repository_inventory_contains_only_the_reviewed_0053_candidate_groups(self) -> None:
        manifest_path = Path("specification/api/public-api.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        candidates = [
            declaration
            for declaration in manifest["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0053"
        ]
        self.assertEqual(744, len(candidates))
        self.assertEqual(
            {
                "typed_object_attributes": 285,
                "typed_managed_objects_and_key_material": 351,
                "typed_cryptographic_parameters": 106,
                "request_message_authentication": 2,
            },
            {
                capability_id: sum(
                    declaration["capabilityId"] == capability_id
                    for declaration in candidates
                )
                for capability_id in {
                    declaration["capabilityId"] for declaration in candidates
                }
            },
        )
        resolved = [
            declaration for declaration in manifest["publicDeclarations"]
            if declaration.get("declarationId") == "kmipkit-0048-kmipkit-7ff36f3a6017bfcf719b"
        ]
        self.assertEqual(1, len(resolved))
        self.assertEqual("verified", resolved[0]["declarationState"])

    def test_additional_foreign_route_is_added_once_with_its_requirement_trace(self) -> None:
        route_path = Path("specs/053-api-parity-facade/additional-api-routes.json")
        additions = json.loads(route_path.read_text(encoding="utf-8"))["functions"]

        first = reconcile_0053_api_parity.add_additional_functions(self.manifest, additions)
        second = reconcile_0053_api_parity.add_additional_functions(first, additions)

        self.assertEqual(1, sum(item["id"] == "typed_attribute_from_item" for item in first["functions"]))
        self.assertEqual(first, second)
        self.assertTrue(
            set(additions[0]["requirementIds"]).issubset(first["requirementIds"])
        )

    def test_real_typed_attribute_from_item_route_unblocks_generated_c_surface(self) -> None:
        route_path = Path("specs/053-api-parity-facade/additional-api-routes.json")
        route_data = json.loads(route_path.read_text(encoding="utf-8"))
        candidate_id = "kmipkit-0048-kmipkit-7ff36f3a6017bfcf719b"
        self.assertIn(candidate_id, route_data["candidateRoutes"])

        manifest = reconcile_0053_api_parity.reconcile_manifest(
            self.manifest_for_repository(), route_data["candidateRoutes"]
        )
        manifest = reconcile_0053_api_parity.add_additional_functions(
            manifest, route_data["functions"]
        )
        manifest = reconcile_0053_api_parity.add_public_declarations(
            manifest, route_data["publicDeclarations"]
        )
        mapped = next(row for row in manifest["publicDeclarations"]
                      if row["declarationId"] == candidate_id)
        self.assertEqual("verified", mapped["declarationState"])
        self.assertEqual(len(manifest["publicDeclarations"]),
                         len({row["declarationId"] for row in manifest["publicDeclarations"]}))
        self.assertTrue(any(row["exportName"] == "kmipkit_typed_attribute_from_item"
                            and row["declarationState"] == "verified"
                            for row in manifest["publicDeclarations"]))

    @staticmethod
    def manifest_for_repository() -> dict:
        return json.loads(Path("specification/api/public-api.json").read_text(encoding="utf-8"))

    def test_public_four_language_export_is_added_idempotently(self) -> None:
        declarations = [
            {
                "declarationId": "four-language-test-export",
                "packageId": "c-kmipkit",
                "kind": "c-symbol",
                "exportName": "kmipkit_test_export",
                "capabilityId": "typed_object_attributes",
                "requirementIds": ["KMIPKIT-0053-FR-002"],
                "declarationState": "verified",
                "parityScope": "four-language",
                "languageMappings": {
                    "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                    "c": {"packageId": "c-kmipkit", "name": "kmipkit_test_export"},
                    "java": {"packageId": "java-kmipkit", "name": "Test.export"},
                    "python": {"packageId": "python-kmipkit", "name": "kmipkit.test.export"},
                },
                "parityEvidenceIds": ["tests/consumer.c"],
            }
        ]

        first = reconcile_0053_api_parity.add_public_declarations(self.manifest, declarations)
        second = reconcile_0053_api_parity.add_public_declarations(first, declarations)

        self.assertEqual(first, second)
        self.assertEqual(
            "verified",
            next(
                row["declarationState"]
                for row in first["publicDeclarations"]
                if row["declarationId"] == "four-language-test-export"
            ),
        )

    def test_public_export_cannot_claim_parity_with_a_missing_language(self) -> None:
        declarations = [
            {
                "declarationId": "four-language-test-export",
                "packageId": "c-kmipkit",
                "exportName": "kmipkit_test_export",
                "declarationState": "verified",
                "parityScope": "four-language",
                "languageMappings": {
                    "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                    "c": {"packageId": "c-kmipkit", "name": "kmipkit_test_export"},
                    "java": {"packageId": "java-kmipkit", "name": "Test.export"},
                },
                "parityEvidenceIds": ["tests/consumer.c"],
                "requirementIds": ["KMIPKIT-0053-FR-002"],
            }
        ]

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_public_declarations(self.manifest, declarations)


if __name__ == "__main__":
    unittest.main()
