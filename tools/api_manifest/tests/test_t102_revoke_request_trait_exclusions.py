"""Pin the exact KMIPKIT-0043 exclusions for Revoke request trait paths."""

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
    "path": "specs/022-object-lifecycle/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::RevokeRequest as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::RevokeRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::RevokeRequest as core::borrow::Borrow<kmipkit_protocol::revoke::RevokeRequest>>::borrow",
    "<kmipkit_protocol::RevokeRequest as core::borrow::BorrowMut<kmipkit_protocol::revoke::RevokeRequest>>::borrow_mut",
    "<kmipkit_protocol::RevokeRequest as core::convert::From<kmipkit_protocol::revoke::RevokeRequest>>::from",
    "<kmipkit_protocol::RevokeRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::RevokeRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::RevokeRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::RevokeRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::RevokeRequest as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::RevokeRequest as core::fmt::Debug>::fmt",
}


class T102RevokeRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_protocol_request_traits_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }

        self.assertEqual(declarations, set())
        self.assertEqual(set(exclusions), EXPECTED_PROTOCOL_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0036-FR-001",
                    "KMIPKIT-0036-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn(
                    "KMIP v2.1 Section 6.1.44 Tables 295-297, including Table 295 request",
                    row["evidence"],
                )
                self.assertIn("no direct Revoke case ID", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("RevokeRequest", row["reason"])

    def test_rust_audit_reports_the_exact_direct_and_conditional_trait_paths(self) -> None:
        direct = {
            row["exportName"]
            for row in self.rust_report["declarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        conditional = {
            row["exportName"]
            for row in self.rust_report["conditionalDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        excluded = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        expected_direct = {
            name
            for name in EXPECTED_PROTOCOL_TRAIT_EXPORTS
            if " as core::convert::Into<U>" not in name
            and " as core::convert::TryFrom<U>" not in name
            and " as core::convert::TryInto<U>" not in name
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(direct, expected_direct)
        self.assertEqual(
            conditional,
            EXPECTED_PROTOCOL_TRAIT_EXPORTS - expected_direct,
        )
        self.assertEqual(len(direct), 5)
        self.assertEqual(len(conditional), 5)
        self.assertEqual(excluded, EXPECTED_PROTOCOL_TRAIT_EXPORTS)

    def test_request_mapping_and_unmapped_root_surface_remain_unchanged(self) -> None:
        protocol_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "kmipkit_protocol::RevokeRequest"
            )
        }
        verified = {
            "kmipkit_protocol::RevokeRequest",
            "kmipkit_protocol::RevokeRequest::new",
            "kmipkit_protocol::RevokeRequest::revocation_message",
            "kmipkit_protocol::RevokeRequest::with_compromise_occurrence_date",
            "kmipkit_protocol::RevokeRequest::with_revocation_message",
            "kmipkit_protocol::RevokeRequest::with_unique_identifier",
        }
        blocked = {
            "kmipkit_protocol::RevokeRequest::compromise_occurrence_date",
            "kmipkit_protocol::RevokeRequest::into_ttlv_payload",
            "kmipkit_protocol::RevokeRequest::revocation_reason",
            "kmipkit_protocol::RevokeRequest::unique_identifier",
        }
        self.assertEqual(set(protocol_rows), verified | blocked)
        self.assertEqual(
            {
                name
                for name, row in protocol_rows.items()
                if row["declarationState"] == "verified"
            },
            verified,
        )
        self.assertEqual(
            {
                name
                for name, row in protocol_rows.items()
                if row["declarationState"] == "blocked"
            },
            blocked,
        )
        root_rows = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::RevokeRequest")
        }
        self.assertEqual(root_rows, set())
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            3808,
        )


if __name__ == "__main__":
    unittest.main()
