"""Pin T102's exact Debug trait exclusions for caller-attested profile state."""

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
    "rust-kmipkit-protocol": ("kmipkit_protocol", "blocked"),
    "rust-kmipkit-client": ("kmipkit_client", "declared"),
    "rust-kmipkit": ("kmipkit", "blocked"),
}
TYPE_NAMES = ("ProfileValidationContext", "ProfileObjectState")
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
OWNER_REQUIREMENTS = (
    "KMIPKIT-0010-FR-012",
    "KMIPKIT-0010-FR-019",
)
T102_REQUIREMENTS = (
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-021",
)


def debug_path(crate: str, type_name: str) -> str:
    return f"<{crate}::{type_name} as core::fmt::Debug>::fmt"


class T102ProfileDebugTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))

    def test_only_six_exact_debug_paths_are_excluded_and_other_traits_stay_candidates(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        all_selected: set[tuple[str, str]] = set()
        retained_path_count = 0
        for package, (crate, _parent_state) in PACKAGES.items():
            for type_name in TYPE_NAMES:
                prefix = f"<{crate}::{type_name} as "
                direct = {
                    row["exportName"]
                    for row in self.report["declarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                conditional = {
                    row["exportName"]
                    for row in self.report["conditionalDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                expected_debug = {debug_path(crate, type_name)}
                observed = direct | conditional
                self.assertIn(expected_debug.pop(), direct)
                self.assertEqual(len(direct), 15 if type_name == "ProfileValidationContext" else 14)
                self.assertEqual(len(conditional), 5)

                declarations = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                exclusions = {
                    row["exportName"]: row
                    for row in self.manifest["approvedExclusions"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                self.assertEqual(set(declarations), observed - {debug_path(crate, type_name)})
                self.assertEqual(set(exclusions), {debug_path(crate, type_name)})

                for export_name, row in declarations.items():
                    with self.subTest(package=package, export_name=export_name):
                        self.assertEqual(row["declarationState"], "blocked")
                        self.assertEqual(row["candidateSpecId"], "KMIPKIT-0043")
                for export_name, row in exclusions.items():
                    with self.subTest(package=package, export_name=export_name):
                        self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                        for requirement_id in (*OWNER_REQUIREMENTS, *T102_REQUIREMENTS):
                            self.assertIn(requirement_id, row["evidence"])
                        citation = f"KMIP Specification v2.1 {chr(0x00A7)}4.38 and {chr(0x00A7)}4.59"
                        self.assertIn(citation, row["evidence"])
                        self.assertIn("compiler-generated Rust traits", row["evidence"])
                        self.assertIn("No OASIS requirement is assigned to these traits", row["evidence"])
                        self.assertIn("inventory-only", row["evidence"])
                        self.assertIn("redacted", row["reason"].lower())
                        self.assertIn("no separate kmip client capability", row["reason"].lower())

                observed_excluded = {
                    row["exportName"]
                    for row in self.report["excluded"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                self.assertEqual(observed_excluded, {debug_path(crate, type_name)})

                all_selected.update((package, path) for path in (observed - (observed - {debug_path(crate, type_name)})))
                retained_path_count += len(declarations)

        self.assertEqual(all_selected, {
            (package, debug_path(crate, type_name))
            for package, (crate, _parent_state) in PACKAGES.items()
            for type_name in TYPE_NAMES
        })
        self.assertEqual(retained_path_count, 111)

    def test_profile_parent_states_and_capability_readiness_are_preserved(self) -> None:
        for package, (crate, expected_state) in PACKAGES.items():
            for type_name in TYPE_NAMES:
                parents = [
                    row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName") == f"{crate}::{type_name}"
                ]
                self.assertEqual(len(parents), 1, (package, type_name))
                self.assertEqual(parents[0]["declarationState"], expected_state)

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        self.assertEqual(capability["capabilityState"], "declared")
        baseline = next(
            row
            for row in self.catalog["profiles"]
            if row["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        self.assertEqual(baseline["claim_state"], "evidence_incomplete")

    def test_manifest_totals_reflect_only_the_six_debug_exclusions(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 1588)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6786)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 7938)


if __name__ == "__main__":
    unittest.main()
