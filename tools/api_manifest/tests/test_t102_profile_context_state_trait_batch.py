"""Pin the T102 profile context and object-state trait inventory batch."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"
REPORT_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)

PACKAGES = {
    "rust-kmipkit": "kmipkit",
    "rust-kmipkit-client": "kmipkit_client",
    "rust-kmipkit-protocol": "kmipkit_protocol",
}
TYPES = {
    "ProfileValidationContext": (14, "kmipkit_protocol::profile::ProfileValidationContext"),
    "ProfileObjectState": (13, "kmipkit_protocol::profile::ProfileObjectState"),
}
DIRECT_SUFFIXES = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "core::any::Any>::type_id",
    "core::borrow::Borrow<{}>>::borrow",
    "core::borrow::BorrowMut<{}>>::borrow_mut",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::convert::From<{}>>::from",
}
CONDITIONAL_SUFFIXES = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/010-profile-conformance/spec.md",
    "status": "Accepted",
    "evidence": (
        "**Status**: Accepted for implementation by merge of [PR #49]"
        "(https://github.com/NeverWe1come/KMIPKit/pull/49) into `release/1.0.0` "
        "(merge commit `045b97dde40e54efbc264d95c1d827c4f7d40415`, 2026-10-07). "
        "No separate GitHub review event is claimed."
    ),
}
OWNER_REQUIREMENTS = ("KMIPKIT-0010-FR-012", "KMIPKIT-0010-FR-019")
T102_REQUIREMENTS = (
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-021",
)


def trait_paths(crate: str, type_name: str) -> tuple[set[str], set[str]]:
    _direct_count, target = TYPES[type_name]
    direct = {
        f"<{crate}::{type_name} as {suffix.format(target)}"
        for suffix in DIRECT_SUFFIXES
    }
    if type_name == "ProfileValidationContext":
        direct.add(f"<{crate}::{type_name} as core::default::Default>::default")
    conditional = {
        f"<{crate}::{type_name} as {suffix}" for suffix in CONDITIONAL_SUFFIXES
    }
    return direct, conditional


class T102ProfileContextStateTraitBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_context_and_state_traits_are_approved_inventory_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        expected_total: set[tuple[str, str]] = set()
        for package, crate in PACKAGES.items():
            for type_name, (direct_count, _target) in TYPES.items():
                direct, conditional = trait_paths(crate, type_name)
                self.assertEqual(len(direct), direct_count)
                self.assertEqual(len(conditional), 5)
                debug_path = f"<{crate}::{type_name} as core::fmt::Debug>::fmt"
                expected = direct | conditional | {debug_path}

                for key, expected_paths in (
                    ("declarations", direct | {debug_path}),
                    ("conditionalDeclarations", conditional),
                ):
                    observed = {
                        row["exportName"]
                        for row in self.report[key]
                        if row.get("packageId") == package
                        and row.get("exportName", "").startswith(f"<{crate}::{type_name} as ")
                    }
                    self.assertEqual(observed, expected_paths, (package, type_name, key))

                declarations = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(f"<{crate}::{type_name} as ")
                }
                exclusions = {
                    row["exportName"]: row
                    for row in self.manifest["approvedExclusions"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(f"<{crate}::{type_name} as ")
                }
                self.assertEqual(set(declarations), set())
                self.assertEqual(set(exclusions), expected)

                for export_name, row in exclusions.items():
                    with self.subTest(package=package, export_name=export_name):
                        self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                        if export_name == debug_path:
                            continue
                        for requirement_id in (*OWNER_REQUIREMENTS, *T102_REQUIREMENTS):
                            self.assertIn(requirement_id, row["evidence"])
                        self.assertIn("KMIP Specification v2.1 §4.38 and §4.59", row["evidence"])
                        self.assertIn("compiler-observed Rust standard-trait paths", row["evidence"])
                        self.assertIn("No OASIS requirement is assigned to these traits", row["evidence"])
                        self.assertIn("inventory-only", row["evidence"])
                        self.assertIn("no separate KMIP client capability", row["reason"])

                reported_exclusions = {
                    row["exportName"]
                    for row in self.report["excluded"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(f"<{crate}::{type_name} as ")
                }
                self.assertEqual(reported_exclusions, expected)
                expected_total.update((package, path) for path in direct | conditional)

        self.assertEqual(len(expected_total), 111)

    def test_parent_states_and_profile_claims_remain_unchanged(self) -> None:
        for package, crate in PACKAGES.items():
            for type_name in TYPES:
                parents = [
                    row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName") == f"{crate}::{type_name}"
                ]
                self.assertEqual(len(parents), 1)
                self.assertEqual(parents[0]["declarationState"], "declared")

        capability = next(
            row for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        self.assertEqual(capability["capabilityState"], "declared")
        baseline = next(
            row for row in self.catalog["profiles"]
            if row["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        self.assertEqual(baseline["claim_state"], "evidence_incomplete")

    def test_manifest_totals_reflect_current_t102_trait_batches(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 841)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6470)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 8480)


if __name__ == "__main__":
    unittest.main()
