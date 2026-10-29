"""T013 traceability and parity gates for mapped shared Rust declarations."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = REPO_ROOT / "tools" / "api_manifest" / "audit_shared_rust_inventory.py"
SPEC = importlib.util.spec_from_file_location("audit_shared_rust_inventory", MODULE_PATH)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class SharedRustInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(
            (REPO_ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8")
        )
        self.rust_report = json.loads(
            (
                REPO_ROOT
                / "specs"
                / "043-public-api-bindings"
                / "reports"
                / "rust-export-report-2026-10-13.json"
            ).read_text(encoding="utf-8")
        )

    def test_approved_owner_lookup_uses_feature_requirements_when_directory_number_differs(self) -> None:
        owner_requirements = AUDIT._read_approved_owner("KMIPKIT-0044", REPO_ROOT)

        self.assertIn("KMIPKIT-0044-FR-001", owner_requirements)
        self.assertIn("KMIPKIT-0044-FR-007", owner_requirements)

    def test_inventory_accounts_for_exact_approved_rust_exclusions_separately(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)

        self.assertEqual(inventory["approvedExcludedRustDeclarationCount"], 22)
        self.assertEqual(
            sum(record["excludedDeclarationCount"] for record in inventory["crates"]),
            22,
        )
        self.assertEqual(
            sum(record["blockedDeclarationCount"] for record in inventory["crates"]),
            inventory["observedRustDeclarationCount"]
            - inventory["mappedRustDeclarationCount"]
            - inventory["approvedExcludedRustDeclarationCount"],
        )

    def test_current_inventory_links_every_mapped_rust_declaration_to_approved_owner_requirements(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)

        self.assertEqual(inventory["observedRustDeclarationCount"], 9525)
        self.assertEqual(inventory["mappedRustDeclarationCount"], 143)
        self.assertEqual(len(inventory["blockedUnreconciledDeclarations"]), 36)
        self.assertNotIn("KMIPKIT-0009", inventory["draftOwnerGates"])
        self.assertEqual(inventory["draftOwnerGates"]["KMIPKIT-0009"]["blockedOperations"], 4)
        self.assertEqual(
            {record["packageId"] for record in inventory["crates"]},
            {
                "rust-kmipkit-ttlv",
                "rust-kmipkit-protocol",
                "rust-kmipkit-transport",
                "rust-kmipkit-client",
                "rust-kmipkit",
            },
        )
        for declaration in inventory["mappedDeclarations"]:
            self.assertTrue(declaration["requirementIds"])
            if declaration["ownerSpecId"] is not None:
                self.assertTrue(declaration["ownerRequirementIds"])
                self.assertTrue(
                    set(declaration["ownerRequirementIds"]) <= set(declaration["requirementIds"])
                )
            else:
                self.assertEqual(declaration["ownerAdrId"], "ADR-0014")
                self.assertFalse(declaration["ownerRequirementIds"])
                self.assertIn("KMIPKIT-0043-FR-020", declaration["requirementIds"])

    def test_draft_owned_asynchronous_and_profile_surfaces_remain_blocked(self) -> None:
        for record in self.manifest["publicDeclarations"]:
            if record.get("candidateSpecId") in {"KMIPKIT-0009", "KMIPKIT-0010"}:
                self.assertEqual(record["declarationState"], "blocked")
                self.assertNotIn("approvalSource", record)
                self.assertNotIn("languageMappings", record)
        for operation in self.manifest["operations"]:
            dependency = operation.get("ownerDependency", {})
            if dependency.get("state") == "candidate" and dependency.get("specId") in {
                "KMIPKIT-0009",
                "KMIPKIT-0010",
            }:
                self.assertEqual(operation["apiState"], "blocked")

    def test_transport_is_the_only_accepted_rust_only_capability(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        rows = [row for row in self.manifest["publicDeclarations"] if row["parityScope"] == "rust-only-adr"]

        self.assertEqual(len(rows), 5)
        self.assertEqual({row["capabilityId"] for row in rows}, {"rust_transport_exchange"})
        self.assertEqual({row["packageId"] for row in rows}, {"rust-kmipkit-transport"})
        self.assertEqual(len(inventory["rustOnlyDeclarations"]), 5)

    def test_approved_registry_capability_matches_its_existing_four_language_declarations(self) -> None:
        capability = next(
            row for row in self.manifest["capabilities"] if row["capabilityId"] == "vendor_extension_registry"
        )
        declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["capabilityId"] == "vendor_extension_registry"
            and row["declarationState"] in {"declared", "verified"}
        ]

        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["ownerSpecId"], "KMIPKIT-0012")
        self.assertEqual(capability["approvalEvidence"]["status"], "Approved")
        language_packages = {
            "rust": {
                package["id"]
                for package in self.manifest["publicPackages"]
                if package["language"] == "rust"
            },
            "c": {"c-kmipkit"},
            "java": {"java-kmipkit"},
            "python": {"python-kmipkit"},
        }
        for language, packages in language_packages.items():
            expected = sorted(
                {
                    row["exportName"]
                    for row in declarations
                    if row["packageId"] in packages
                }
            )
            self.assertEqual(capability["mappings"][language], expected)

    def test_typed_view_and_discover_versions_exports_use_their_semantic_owner_requirements(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        declarations = {row["exportName"]: row for row in inventory["mappedDeclarations"]}

        structure_view = declarations["kmipkit_ttlv::StructureView"]
        self.assertEqual(structure_view["capabilityId"], "generic_ttlv_model")
        self.assertEqual(structure_view["ownerSpecId"], "KMIPKIT-0004")
        self.assertTrue(
            {
                "KMIPKIT-0004-FR-009",
                "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            }
            <= set(structure_view["requirementIds"])
        )

        batch_item = declarations["kmipkit_client::ClientBatchItem"]
        self.assertEqual(batch_item["ownerSpecId"], "KMIPKIT-0012")
        self.assertIn("KMIPKIT-0007-FR-001", batch_item["requirementIds"])

        discover_versions = declarations["kmipkit_client::ClientBatchItem::discover_versions"]
        self.assertEqual(discover_versions["capabilityId"], "discover_versions_request_item")
        self.assertEqual(discover_versions["ownerSpecId"], "KMIPKIT-0007")
        self.assertTrue(
            {
                "KMIPKIT-0007-FR-001",
                "KMIPKIT-0007-FR-002",
            }
            <= set(discover_versions["ownerRequirementIds"])
        )

        operation = next(row for row in self.manifest["operations"] if row["name"] == "Discover Versions")
        self.assertEqual(operation["apiState"], "blocked")

    def test_all_ready_capability_mappings_match_exact_ready_exports(self) -> None:
        language_packages = {
            language: {
                package["id"]
                for package in self.manifest["publicPackages"]
                if package["language"] == language
            }
            for language in ("rust", "c", "java", "python")
        }
        for capability in self.manifest["capabilities"]:
            if "mappings" not in capability:
                continue
            for language, packages in language_packages.items():
                if capability["parityScope"] == "rust-only-adr" and language != "rust":
                    continue
                expected = sorted(
                    {
                        row["exportName"]
                        for row in self.manifest["publicDeclarations"]
                        if row["capabilityId"] == capability["capabilityId"]
                        and row["declarationState"] in {"declared", "verified"}
                        and row["packageId"] in packages
                    }
                )
                self.assertEqual(
                    capability["mappings"].get(language, []),
                    expected,
                    f"{capability['capabilityId']} {language} mappings must name exact ready exports",
                )

    def test_unrepresented_runtime_error_enum_stays_blocked_but_other_variant_maps(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit"
        }
        error_type = declarations["kmipkit::ClientCauseCategory"]
        other_variant = declarations["kmipkit::ClientCauseCategory::Other"]

        self.assertEqual(error_type["kind"], "type")
        self.assertEqual(error_type["declarationState"], "blocked")
        self.assertEqual(error_type["candidateSpecId"], "KMIPKIT-0043")
        for missing_category in ("invalid-input", "protocol", "transport"):
            with self.subTest(category=missing_category):
                self.assertIn(missing_category, error_type["blockedReason"])
        self.assertNotIn("languageMappings", error_type)
        self.assertNotIn("approvalSource", error_type)
        self.assertEqual(other_variant["kind"], "variant")
        self.assertEqual(other_variant["declarationState"], "declared")
        self.assertEqual(
            other_variant["languageMappings"]["rust"],
            {"packageId": "rust-kmipkit", "name": "kmipkit::ClientCauseCategory::Other"},
        )

    def test_every_ready_language_mapping_resolves_to_an_exact_ready_export(self) -> None:
        declarations_by_export = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        for declaration in self.manifest["publicDeclarations"]:
            if declaration["declarationState"] not in {"declared", "verified"}:
                continue
            for language, mapping in declaration["languageMappings"].items():
                target = declarations_by_export.get((mapping["packageId"], mapping["name"]))
                self.assertIsNotNone(
                    target,
                    f"{declaration['declarationId']} maps {language} to an unlisted export",
                )
                self.assertIn(
                    target["declarationState"],
                    {"declared", "verified"},
                    f"{declaration['declarationId']} maps {language} to a blocked export",
                )

    def test_unreconciled_destructors_and_helpers_are_blocked_without_mappings(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        blocked = inventory["blockedUnreconciledDeclarations"]

        self.assertEqual(len(blocked), 36)
        self.assertEqual(
            sum("callable public declaration" in row["blockedReason"] for row in blocked),
            22,
        )
        self.assertEqual(
            sum("No exact equivalent public Rust declaration" in row["blockedReason"] for row in blocked),
            12,
        )
        for row in blocked:
            source = next(
                declaration
                for declaration in self.manifest["publicDeclarations"]
                if declaration["declarationId"] == row["declarationId"]
            )
            self.assertEqual(source["declarationState"], "blocked")
            self.assertNotIn("languageMappings", source)
            self.assertNotIn("approvalSource", source)

    def test_runtime_metadata_namespaces_keep_their_candidate_in_blocked_report(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        blocked = {
            row["declarationId"]: row
            for row in inventory["blockedUnreconciledDeclarations"]
        }
        expected = {
            "org.kmipkit.RuntimeMetadata:java": "org.kmipkit.RuntimeMetadata",
            "kmipkit.metadata:python": "kmipkit.metadata",
        }

        for declaration_id, export_name in expected.items():
            with self.subTest(declaration=declaration_id):
                row = blocked[declaration_id]
                self.assertEqual(row["exportName"], export_name)
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0043")
                self.assertIn("crate root is not an exported declaration", row["blockedReason"])

                source = next(
                    declaration
                    for declaration in self.manifest["publicDeclarations"]
                    if declaration["declarationId"] == declaration_id
                )
                self.assertEqual(source["declarationState"], "blocked")
                self.assertNotIn("languageMappings", source)
                self.assertNotIn("approvalSource", source)

    def test_ready_declaration_with_an_unobserved_rust_mapping_is_rejected(self) -> None:
        altered = json.loads(json.dumps(self.manifest))
        declaration = next(
            row
            for row in altered["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit"
            and row.get("declarationState") == "declared"
        )
        declaration["languageMappings"]["rust"]["name"] = "kmipkit::invented_public_item"

        with self.assertRaises(AUDIT.InventoryError):
            AUDIT.build_inventory(altered, self.rust_report, REPO_ROOT)

    def test_unapproved_rust_only_owner_is_rejected(self) -> None:
        altered = json.loads(json.dumps(self.manifest))
        declaration = next(row for row in altered["publicDeclarations"] if row["parityScope"] == "rust-only-adr")
        declaration["capabilityId"] = "public_binding_surface"

        with self.assertRaises(AUDIT.InventoryError):
            AUDIT.build_inventory(altered, self.rust_report, REPO_ROOT)


if __name__ == "__main__":
    unittest.main()
