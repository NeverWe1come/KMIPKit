"""Pin T102's exact Rust profile-validation error trait exclusions."""

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
    "KMIPKIT-0010-FR-003",
    "KMIPKIT-0010-FR-005",
    "KMIPKIT-0010-FR-013",
    "KMIPKIT-0010-FR-014",
    "KMIPKIT-0010-FR-019",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
PACKAGES = {
    "rust-kmipkit": "kmipkit",
    "rust-kmipkit-client": "kmipkit_client",
    "rust-kmipkit-protocol": "kmipkit_protocol",
}
ERROR_DIRECT_SUFFIXES = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "alloc::string::ToString>::to_string",
    "core::any::Any>::type_id",
    "core::borrow::Borrow<kmipkit_protocol::profile::ProfileValidationError>>::borrow",
    "core::borrow::BorrowMut<kmipkit_protocol::profile::ProfileValidationError>>::borrow_mut",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::convert::From<kmipkit_protocol::profile::ProfileValidationError>>::from",
    "core::error::Error>::cause",
    "core::error::Error>::description",
    "core::error::Error>::provide",
    "core::error::Error>::source",
    "core::error::Error>::type_id",
    "core::fmt::Debug>::fmt",
    "core::fmt::Display>::fmt",
}
CODE_DIRECT_SUFFIXES = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "core::any::Any>::type_id",
    "core::borrow::Borrow<kmipkit_protocol::profile::ProfileValidationErrorCode>>::borrow",
    "core::borrow::BorrowMut<kmipkit_protocol::profile::ProfileValidationErrorCode>>::borrow_mut",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::convert::From<kmipkit_protocol::profile::ProfileValidationErrorCode>>::from",
    "core::fmt::Debug>::fmt",
}
CONDITIONAL_SUFFIXES = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
BLOCKED_CODE_EXPORTS = {
    "ProfileValidationErrorCode",
    "ProfileValidationErrorCode::ConditionalProfileUnsatisfied",
    "ProfileValidationErrorCode::ConflictingConstraints",
    "ProfileValidationErrorCode::DependencyCycle",
    "ProfileValidationErrorCode::DuplicateProfile",
    "ProfileValidationErrorCode::EmptySelection",
    "ProfileValidationErrorCode::IncompleteRuleMapping",
    "ProfileValidationErrorCode::MissingCategoryTest",
    "ProfileValidationErrorCode::MissingRequirementMetadata",
    "ProfileValidationErrorCode::MissingRuleMapping",
    "ProfileValidationErrorCode::ProfileStateBindingMismatch",
    "ProfileValidationErrorCode::ProfileStateContextRequired",
    "ProfileValidationErrorCode::ProfileUnavailable",
    "ProfileValidationErrorCode::RequestRuleRejected",
    "ProfileValidationErrorCode::ResponseRuleRejected",
    "ProfileValidationErrorCode::RuleMappingMismatch",
    "ProfileValidationErrorCode::TransportConstraintRejected",
    "ProfileValidationErrorCode::UnknownProfile",
    "ProfileValidationErrorCode::UnresolvedRequirementScope",
    "ProfileValidationErrorCode::as_str",
}
ERROR_FUNCTIONAL_EXPORTS = {
    "ProfileValidationError",
    "ProfileValidationError::batch_item_index",
    "ProfileValidationError::code",
    "ProfileValidationError::profile_id",
    "ProfileValidationError::requirement_id",
    "ProfileValidationError::source_clause_id",
    "ProfileValidationError::source_gap_id",
    "ProfileValidationError::source_gap_locator",
}


def trait_paths(crate: str, type_name: str, suffixes: set[str]) -> set[str]:
    return {f"<{crate}::{type_name} as {suffix}" for suffix in suffixes}


class T102ProfileValidationErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))

    def test_exact_profile_error_trait_paths_are_compiler_observed_and_owner_approved(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        all_paths: set[tuple[str, str]] = set()

        for package, crate in PACKAGES.items():
            for type_name, direct_suffixes in (
                ("ProfileValidationError", ERROR_DIRECT_SUFFIXES),
                ("ProfileValidationErrorCode", CODE_DIRECT_SUFFIXES),
            ):
                prefix = f"<{crate}::{type_name} as "
                expected_direct = trait_paths(crate, type_name, direct_suffixes)
                expected_conditional = trait_paths(crate, type_name, CONDITIONAL_SUFFIXES)
                expected = expected_direct | expected_conditional
                observed_direct = {
                    row["exportName"]
                    for row in self.report["declarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                observed_conditional = {
                    row["exportName"]
                    for row in self.report["conditionalDeclarations"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                observed_excluded = {
                    row["exportName"]
                    for row in self.report["excluded"]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(prefix)
                }
                self.assertEqual(observed_direct, expected_direct)
                self.assertEqual(observed_conditional, expected_conditional)
                self.assertEqual(observed_excluded, expected)
                self.assertEqual(len(expected_direct), len(direct_suffixes))
                self.assertEqual(len(expected_conditional), 5)

                declarations = {
                    row["exportName"]
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
                self.assertEqual(declarations, set())
                self.assertEqual(set(exclusions), expected)
                self.assertEqual(len(exclusions), len(expected))

                for export_name, row in exclusions.items():
                    with self.subTest(package=package, export_name=export_name):
                        self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                        for requirement_id in (*OWNER_REQUIREMENTS, *T102_REQUIREMENTS):
                            self.assertIn(requirement_id, row["evidence"])
                        self.assertIn("KMIP Specification v2.1 §14.1", row["evidence"])
                        self.assertIn(
                            "Profiles v2.1 §§3.1, 5.1.1, 5.1.3, 5.6.3, and 6.1",
                            row["evidence"],
                        )
                        self.assertIn("compiler-generated Rust traits", row["evidence"])
                        self.assertIn("no OASIS error-table reference applies", row["evidence"])
                        self.assertIn("Exact compiler-observed Rust standard-trait", row["reason"])
                        self.assertIn("not a separate KMIP client capability", row["reason"])
                        self.assertIn(type_name, row["reason"])
                all_paths.update((package, export_name) for export_name in expected)

        self.assertEqual(len(all_paths), 135)

    def test_functional_profile_error_and_blocked_code_states_remain_unchanged(self) -> None:
        for package, crate in PACKAGES.items():
            rows = {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package
                and not row.get("exportName", "").startswith("<")
                and row["exportName"].startswith(f"{crate}::ProfileValidationError")
            }
            expected_error = {
                f"{crate}::{name}": "declared" for name in ERROR_FUNCTIONAL_EXPORTS
            }
            expected_error[f"{crate}::ProfileValidationError::for_test"] = "blocked"
            expected_code = {
                f"{crate}::{name}": "blocked" for name in BLOCKED_CODE_EXPORTS
            }
            self.assertEqual(
                {name: row["declarationState"] for name, row in rows.items()},
                expected_error | expected_code,
            )
            for name in expected_code:
                self.assertEqual(rows[name]["candidateSpecId"], "KMIPKIT-0043")

    def test_profile_capability_and_baseline_evidence_state_are_preserved(self) -> None:
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

    def test_manifest_and_candidate_totals_change_only_by_this_exact_batch(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 1721)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6919)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 7805)


if __name__ == "__main__":
    unittest.main()
