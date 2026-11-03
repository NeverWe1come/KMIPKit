"""Pin the client-only Usage Allocation context trait inventory exclusions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REPORT_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
CLIENT_PACKAGE = "rust-kmipkit-client"
CLIENT_PUBLIC_TYPE = "kmipkit_client::UsageAllocationContext"
CLIENT_IMPLEMENTATION_TYPE = "kmipkit_client::usage_allocation::UsageAllocationContext"
ROOT_PACKAGE = "rust-kmipkit"
ROOT_PUBLIC_TYPE = "kmipkit::UsageAllocationContext"
ROOT_IMPLEMENTATION_TYPE = CLIENT_IMPLEMENTATION_TYPE
CLIENT_ERROR_PUBLIC_TYPE = "kmipkit_client::UsageAllocationContextError"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/025-policy-controls/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}


def expected_direct_traits(public_type: str, implementation_type: str) -> set[str]:
    return {
        f"<{public_type} as core::any::Any>::type_id",
        f"<{public_type} as core::borrow::Borrow<{implementation_type}>>::borrow",
        f"<{public_type} as core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
        f"<{public_type} as core::convert::From<{implementation_type}>>::from",
        f"<{public_type} as core::fmt::Debug>::fmt",
    }


def expected_conditional_traits(public_type: str) -> set[str]:
    return {
        f"<{public_type} as core::convert::Into<U>>::into",
        f"<{public_type} as core::convert::TryFrom<U>>::Error",
        f"<{public_type} as core::convert::TryFrom<U>>::try_from",
        f"<{public_type} as core::convert::TryInto<U>>::Error",
        f"<{public_type} as core::convert::TryInto<U>>::try_into",
    }


class T102UsageAllocationContextTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_client_trait_paths_are_excluded_and_root_alias_stays_blocked(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertEqual(self.report["issues"], [])

        client_direct = expected_direct_traits(
            CLIENT_PUBLIC_TYPE, CLIENT_IMPLEMENTATION_TYPE
        )
        client_conditional = expected_conditional_traits(CLIENT_PUBLIC_TYPE)
        observed_direct = {
            row["exportName"]
            for row in self.report["declarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{CLIENT_PUBLIC_TYPE} as ")
        }
        observed_conditional = {
            row["exportName"]
            for row in self.report["conditionalDeclarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{CLIENT_PUBLIC_TYPE} as ")
        }
        self.assertEqual(observed_direct, client_direct)
        self.assertEqual(observed_conditional, client_conditional)
        self.assertEqual(len(observed_direct), 5)
        self.assertEqual(len(observed_conditional), 5)

        selected = client_direct | client_conditional
        client_declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{CLIENT_PUBLIC_TYPE} as ")
        }
        client_exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{CLIENT_PUBLIC_TYPE} as ")
        }
        self.assertEqual(set(client_declarations), set())
        self.assertEqual(set(client_exclusions), selected)

        for export_name, row in client_exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row.get("candidateSpecId"), "KMIPKIT-0039")
                self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("not a separate KMIPKit client capability", row["reason"])
                for requirement_id in (
                    "KMIPKIT-0039-FR-009",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("§6.1.23, Table 234", row["evidence"])
                self.assertIn("compiler-observed Rust trait paths", row["evidence"])
                self.assertIn(
                    "no oasis requirement requires these rust traits",
                    row["evidence"].lower(),
                )
                self.assertIn("inventory-only", row["evidence"])
                self.assertIn("Borrow", row["evidence"])
                self.assertIn(
                    "identity conversions move the same non-Clone context", row["evidence"]
                )
                self.assertIn(
                    "cannot duplicate, reset, or replenish reservation state", row["evidence"]
                )
                if export_name == f"<{CLIENT_PUBLIC_TYPE} as core::fmt::Debug>::fmt":
                    self.assertIn("KMIPKIT-0039-FR-005", row["evidence"])
                    self.assertIn("KMIPKIT-0043-FR-013", row["evidence"])
                    self.assertIn("redacted", row["evidence"].lower())

        report_exclusions = {
            row["exportName"]
            for row in self.report["excluded"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{CLIENT_PUBLIC_TYPE} as ")
        }
        self.assertEqual(report_exclusions, selected)

        client_parent = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName") == CLIENT_PUBLIC_TYPE
        )
        self.assertEqual(client_parent["declarationState"], "declared")
        self.assertEqual(client_parent["capabilityId"], "usage_allocation_guard")
        self.assertIn("KMIPKIT-0039-FR-009", client_parent["requirementIds"])

        root_direct = expected_direct_traits(ROOT_PUBLIC_TYPE, ROOT_IMPLEMENTATION_TYPE)
        root_conditional = expected_conditional_traits(ROOT_PUBLIC_TYPE)
        root_selected = root_direct | root_conditional
        root_declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == ROOT_PACKAGE
            and row.get("exportName", "").startswith(f"<{ROOT_PUBLIC_TYPE} as ")
        }
        self.assertEqual(set(root_declarations), root_selected)
        self.assertTrue(
            all(
                row["declarationState"] == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in root_declarations.values()
            )
        )
        root_parent = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == ROOT_PACKAGE
            and row.get("exportName") == ROOT_PUBLIC_TYPE
        )
        self.assertEqual(root_parent["declarationState"], "blocked")
        self.assertEqual(root_parent.get("candidateSpecId"), "KMIPKIT-0043")
        root_exclusions = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == ROOT_PACKAGE
            and row.get("exportName", "").startswith(f"<{ROOT_PUBLIC_TYPE} as ")
        }
        self.assertEqual(root_exclusions, set())

    def test_error_sibling_traits_are_exactly_excluded_from_client_inventory(self) -> None:
        error_public_type = CLIENT_ERROR_PUBLIC_TYPE
        error_parent = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName") == error_public_type
        )
        self.assertEqual(error_parent["declarationState"], "blocked")
        self.assertEqual(error_parent.get("candidateSpecId"), "KMIPKIT-0043")
        error_trait_declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{error_public_type} as ")
        ]
        self.assertEqual(error_trait_declarations, [])
        error_exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == CLIENT_PACKAGE
            and row.get("exportName", "").startswith(f"<{error_public_type} as ")
        ]
        self.assertEqual(
            len({row["exportName"] for row in error_exclusions}), 26
        )
        self.assertTrue(
            all(row.get("candidateSpecId") == "KMIPKIT-0039" for row in error_exclusions)
        )

    def test_manifest_and_rust_inventory_totals_include_both_policy_trait_batches(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 1432)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6630)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 8094)
        self.assertEqual(self.report["counts"]["manifest"], 3715)
        self.assertEqual(self.report["counts"]["matched"], 3715)
        self.assertEqual(self.report["counts"]["excluded"], 6748)


if __name__ == "__main__":
    unittest.main()
