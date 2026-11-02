"""Exact T102 exclusions for Get Constraints response trait artifacts.

The functional response contract is owned by KMIPKIT-0039 and grounded in
KMIP v2.1 §6.1.22 Tables 230–231 and §7.7. These compiler-visible Rust
traits are not separate KMIP capabilities.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PREFIX = "<kmipkit_protocol::GetConstraintsResponse as "
ROOT_PREFIX = "<kmipkit::GetConstraintsResponse as "
EXPECTED_TRAIT_EXPORTS = {
    "<kmipkit_protocol::GetConstraintsResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::GetConstraintsResponse as core::borrow::Borrow<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::borrow",
    "<kmipkit_protocol::GetConstraintsResponse as core::borrow::BorrowMut<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::borrow_mut",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::From<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::from",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::GetConstraintsResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::GetConstraintsResponse as core::fmt::Debug>::fmt",
}
EXPECTED_ROOT_TRAIT_EXPORTS = {
    "<kmipkit::GetConstraintsResponse as core::any::Any>::type_id",
    "<kmipkit::GetConstraintsResponse as core::borrow::Borrow<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::borrow",
    "<kmipkit::GetConstraintsResponse as core::borrow::BorrowMut<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::borrow_mut",
    "<kmipkit::GetConstraintsResponse as core::convert::From<kmipkit_protocol::get_constraints::GetConstraintsResponse>>::from",
    "<kmipkit::GetConstraintsResponse as core::convert::Into<U>>::into",
    "<kmipkit::GetConstraintsResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit::GetConstraintsResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit::GetConstraintsResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit::GetConstraintsResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit::GetConstraintsResponse as core::fmt::Debug>::fmt",
}
RETAINED_EXPORTS = {
    "kmipkit_protocol::GetConstraintsResponse",
    "kmipkit_protocol::GetConstraintsResponse::constraints",
    "kmipkit_protocol::GetConstraintsResponse::result",
    "kmipkit_protocol::GetConstraintsResponse::with_ttlv",
}


class T102GetConstraintsResponseTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_all_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0039-FR-001", row["evidence"])
                self.assertIn("KMIPKIT-0039-FR-004", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.22 Tables 229–231 and §7.7", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("GetConstraintsResponse", row["reason"])

    def test_export_audit_reports_the_exact_observed_and_excluded_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_TRAIT_EXPORTS)

    def test_response_type_and_payload_accessors_remain_verified(self) -> None:
        retained = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName") in RETAINED_EXPORTS
        }
        self.assertEqual(set(retained), RETAINED_EXPORTS)
        for export_name, row in retained.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["declarationState"], "verified")
                self.assertEqual(
                    row["capabilityId"], "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS"
                )
                self.assertIn("KMIPKIT-0039-FR-001", row["requirementIds"])

    def test_root_reexport_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(ROOT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(ROOT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_ROOT_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0039-FR-001", row["evidence"])
                self.assertIn("KMIPKIT-0039-FR-004", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.22 Tables 229–231 and §7.7", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("GetConstraintsResponse", row["reason"])

    def test_rust_audit_reports_exact_root_reexport_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(ROOT_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(ROOT_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_ROOT_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_ROOT_TRAIT_EXPORTS)

    def test_root_response_facade_remains_blocked_except_existing_ttlv_constructor(self) -> None:
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::GetConstraintsResponse")
        }
        blocked = {
            "kmipkit::GetConstraintsResponse",
            "kmipkit::GetConstraintsResponse::constraints",
            "kmipkit::GetConstraintsResponse::result",
            "kmipkit::GetConstraintsResponse::try_from_response_item",
        }
        ttlv_constructor = "kmipkit::GetConstraintsResponse::with_ttlv"
        self.assertEqual(set(rows), blocked | {ttlv_constructor})
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in blocked:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["candidateSpecId"], "KMIPKIT-0043")
        self.assertEqual(rows[ttlv_constructor]["declarationState"], "verified")
        self.assertEqual(
            rows[ttlv_constructor]["capabilityId"],
            "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS",
        )


if __name__ == "__main__":
    unittest.main()
