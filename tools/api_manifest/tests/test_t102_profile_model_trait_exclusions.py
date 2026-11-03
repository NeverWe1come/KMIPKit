"""Pin T102's exact Rust profile-model standard-trait exclusions."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"
REPORTS_PATH = ROOT / "specs" / "043-public-api-bindings" / "reports"
REPORT_PATH = REPORTS_PATH / "rust-export-report-2026-11-01.json"

TYPE_CASES = (
    ("rust-kmipkit", "kmipkit::ProfileTransport", "kmipkit_protocol::profile::ProfileTransport", "blocked", 14),
    ("rust-kmipkit", "kmipkit::ProfileValidationResult", "kmipkit_protocol::profile::ProfileValidationResult", "blocked", 14),
    ("rust-kmipkit", "kmipkit::ProfileValidationSelection", "kmipkit_protocol::profile::ProfileValidationSelection", "blocked", 14),
    ("rust-kmipkit-client", "kmipkit_client::ProfileTransport", "kmipkit_protocol::profile::ProfileTransport", "blocked", 14),
    ("rust-kmipkit-client", "kmipkit_client::ProfileValidationSelection", "kmipkit_protocol::profile::ProfileValidationSelection", "declared", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileEvidenceReadiness", "kmipkit_protocol::profile::ProfileReadiness", "blocked", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileEvidenceRecord", "kmipkit_protocol::profile::ProfileEvidenceRecord", "blocked", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileReadiness", "kmipkit_protocol::profile::ProfileReadiness", "blocked", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileReadinessBlocker", "kmipkit_protocol::profile::ProfileReadinessBlocker", "blocked", 27),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileTransport", "kmipkit_protocol::profile::ProfileTransport", "blocked", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileValidationResult", "kmipkit_protocol::profile::ProfileValidationResult", "blocked", 14),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileValidationSelection", "kmipkit_protocol::profile::ProfileValidationSelection", "blocked", 14),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::profiles_generated::GeneratedClientProfile",
        "kmipkit_protocol::profiles_generated::GeneratedClientProfile",
        "blocked",
        14,
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::profiles_generated::GeneratedClientRequirement",
        "kmipkit_protocol::profiles_generated::GeneratedClientRequirement",
        "blocked",
        14,
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::profiles_generated::GeneratedProfileScopeBlocker",
        "kmipkit_protocol::profiles_generated::GeneratedProfileScopeBlocker",
        "blocked",
        14,
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::profiles_generated::GeneratedProfileTestCase",
        "kmipkit_protocol::profiles_generated::GeneratedProfileTestCase",
        "blocked",
        14,
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::profiles_generated::GeneratedVerificationCategory",
        "kmipkit_protocol::profiles_generated::GeneratedVerificationCategory",
        "blocked",
        14,
    ),
)

COMMON_DIRECT_SUFFIXES = (
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "core::any::Any>::type_id",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::fmt::Debug>::fmt",
)
CONDITIONAL_SUFFIXES = (
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
)
READINESS_BLOCKER_ORDER_SUFFIXES = (
    "core::cmp::Ord>::clamp",
    "core::cmp::Ord>::cmp",
    "core::cmp::Ord>::max",
    "core::cmp::Ord>::min",
    "core::cmp::PartialOrd>::__chaining_ge",
    "core::cmp::PartialOrd>::__chaining_gt",
    "core::cmp::PartialOrd>::__chaining_le",
    "core::cmp::PartialOrd>::__chaining_lt",
    "core::cmp::PartialOrd>::ge",
    "core::cmp::PartialOrd>::gt",
    "core::cmp::PartialOrd>::le",
    "core::cmp::PartialOrd>::lt",
    "core::cmp::PartialOrd>::partial_cmp",
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
)
CATALOG_REQUIREMENTS = (
    "KMIPKIT-0010-FR-001",
    "KMIPKIT-0010-FR-002",
    "KMIPKIT-0010-FR-004",
    "KMIPKIT-0010-FR-007",
    "KMIPKIT-0010-FR-008",
    "KMIPKIT-0010-FR-009",
    "KMIPKIT-0010-FR-010",
    "KMIPKIT-0010-FR-011",
    "KMIPKIT-0010-FR-017",
    "KMIPKIT-0010-FR-018",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")

# Stable digests pin the non-trait API records for these exact owner paths and
# the out-of-scope state/context models while the trait inventory is reconciled.
PROTECTED_API_SHA256 = "087b1988d8dbaf2d828104681eead5f4c78dae27210797df92ac4a361d483b45"
OUT_OF_SCOPE_API_SHA256 = "7d6d514495b15f781d689c1bea7f8e1227c5a9f92517cf99aaa33f73265f07c9"
PROFILE_CATALOG_SHA256 = "034aa2e2456e9ac28a24314ee99f00005903978da8d24c5958e4833e748840fb"


def trait_paths(owner: str, suffixes: tuple[str, ...]) -> set[str]:
    return {f"<{owner} as {suffix}" for suffix in suffixes}


def expected_direct_paths(owner: str, target: str, direct_count: int) -> set[str]:
    suffixes = set(COMMON_DIRECT_SUFFIXES)
    suffixes.update(
        {
            f"core::borrow::Borrow<{target}>>::borrow",
            f"core::borrow::BorrowMut<{target}>>::borrow_mut",
            f"core::convert::From<{target}>>::from",
        }
    )
    if direct_count == 27:
        suffixes.update(READINESS_BLOCKER_ORDER_SUFFIXES)
    if len(suffixes) != direct_count:
        raise AssertionError(f"Unexpected direct-trait definition for {owner}.")
    return trait_paths(owner, tuple(sorted(suffixes)))


def canonical_sha256(rows: object) -> str:
    payload = json.dumps(rows, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


class T102ProfileModelTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_profile_model_trait_paths_are_compiler_observed_and_owner_approved(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        all_paths: set[tuple[str, str]] = set()
        for package, owner, target, _parent_state, direct_count in TYPE_CASES:
            expected_direct = expected_direct_paths(owner, target, direct_count)
            expected_conditional = trait_paths(owner, CONDITIONAL_SUFFIXES)
            expected = expected_direct | expected_conditional

            for report_key, expected_paths in (
                ("declarations", expected_direct),
                ("conditionalDeclarations", expected_conditional),
            ):
                observed = {
                    row["exportName"]
                    for row in self.report[report_key]
                    if row.get("packageId") == package
                    and row.get("exportName", "").startswith(f"<{owner} as ")
                }
                self.assertEqual(observed, expected_paths, (package, owner, report_key))

            observed_excluded = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(f"<{owner} as ")
            }
            self.assertEqual(observed_excluded, expected, (package, owner, "report excluded"))

            declarations = {
                row["exportName"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(f"<{owner} as ")
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(f"<{owner} as ")
            }
            self.assertEqual(declarations, set(), (package, owner, "manifest declarations"))
            self.assertEqual(set(exclusions), expected, (package, owner, "manifest exclusions"))
            self.assertEqual(len(exclusions), direct_count + 5)

            for export_name, row in exclusions.items():
                with self.subTest(package=package, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    for requirement_id in (*OWNER_REQUIREMENTS, *CATALOG_REQUIREMENTS, *T102_REQUIREMENTS):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertNotIn("KMIPKIT-0010-FR-019", row["evidence"])
                    self.assertIn("KMIP Specification v2.1 §14.1", row["evidence"])
                    self.assertIn(
                        "KMIP Profiles v2.1 §§3.1, 5.1.1, 5.1.3, 5.6.3, and 6.1",
                        row["evidence"],
                    )
                    self.assertIn("no OASIS error-table reference applies", row["evidence"])
                    self.assertIn("compiler-generated Rust standard-trait paths", row["evidence"])
                    self.assertIn("inventory-only slice makes no conformance", row["evidence"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(owner.rsplit("::", 1)[-1], row["reason"])
            all_paths.update((package, export_name) for export_name in expected)

        self.assertEqual(len(all_paths), 336)

    def test_parent_members_capability_catalog_and_out_of_scope_models_are_preserved(self) -> None:
        for package, owner, _target, expected_state, _direct_count in TYPE_CASES:
            parent = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package and row.get("exportName") == owner
            ]
            self.assertEqual(len(parent), 1, owner)
            self.assertEqual(parent[0]["declarationState"], expected_state, owner)

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        self.assertEqual(capability["capabilityState"], "declared")

        self.assertEqual(
            canonical_sha256(self.catalog["profiles"]),
            PROFILE_CATALOG_SHA256,
            "Profile readiness records must remain unchanged.",
        )
        self.assertEqual(
            canonical_sha256(self._protected_rows(TYPE_CASES)),
            PROTECTED_API_SHA256,
            "Parent, method, field, variant, and reexport records must remain unchanged.",
        )

        out_of_scope = (
            ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileObjectState"),
            ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileValidationContext"),
        )
        for package, owner in out_of_scope:
            parent = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package and row.get("exportName") == owner
            ]
            self.assertEqual(len(parent), 1, owner)
            self.assertEqual(parent[0]["declarationState"], "blocked", owner)
        self.assertEqual(
            canonical_sha256(self._protected_rows(out_of_scope)),
            OUT_OF_SCOPE_API_SHA256,
            "ProfileObjectState and ProfileValidationContext must remain out of scope.",
        )
        self.assertFalse(
            any(
                row.get("packageId") == "rust-kmipkit-protocol"
                and any(
                    row.get("exportName", "").startswith(f"<{owner} as ")
                    for _package, owner in out_of_scope
                )
                for row in self.manifest["approvedExclusions"]
            )
        )

    def test_manifest_totals_change_only_by_the_exact_profile_model_batch(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 2429)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 7627)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 7097)

    def _protected_rows(self, owners: object) -> list[dict[str, object]]:
        return sorted(
            (
                row
                for row in self.manifest["publicDeclarations"]
                if not row.get("exportName", "").startswith("<")
                and any(
                    row.get("packageId") == package
                    and (
                        row.get("exportName") == owner
                        or row.get("exportName", "").startswith(f"{owner}::")
                    )
                    for package, owner, *_rest in owners
                )
            ),
            key=lambda row: (row.get("packageId", ""), row.get("exportName", "")),
        )


if __name__ == "__main__":
    unittest.main()
