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
                / "rust-export-report-2026-10-12.json"
            ).read_text(encoding="utf-8")
        )

    def test_current_inventory_links_every_mapped_rust_declaration_to_approved_owner_requirements(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)

        self.assertEqual(inventory["observedRustDeclarationCount"], 9497)
        self.assertEqual(inventory["mappedRustDeclarationCount"], 109)
        self.assertEqual(len(inventory["blockedUnreconciledDeclarations"]), 36)
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
        for language in ("rust", "c", "java", "python"):
            expected = sorted(
                {row["languageMappings"][language]["name"] for row in declarations}
            )
            self.assertEqual(capability["mappings"][language], expected)

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
