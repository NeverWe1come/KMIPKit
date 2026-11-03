"""Pin the exact KMIPKIT-0043 exclusions for Set Constraints request traits."""

from __future__ import annotations

import json
import unittest

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)
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
    "path": "specs/025-policy-controls/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::SetConstraintsRequest as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::SetConstraintsRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::SetConstraintsRequest as core::borrow::Borrow<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow",
    "<kmipkit_protocol::SetConstraintsRequest as core::borrow::BorrowMut<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow_mut",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::From<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::from",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::SetConstraintsRequest as core::fmt::Debug>::fmt",
}
EXPECTED_DIRECT_TRAITS = {
    "<kmipkit_protocol::SetConstraintsRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::SetConstraintsRequest as core::borrow::Borrow<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow",
    "<kmipkit_protocol::SetConstraintsRequest as core::borrow::BorrowMut<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow_mut",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::From<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::from",
    "<kmipkit_protocol::SetConstraintsRequest as core::fmt::Debug>::fmt",
}
EXPECTED_CONDITIONAL_TRAITS = {
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::SetConstraintsRequest as core::convert::TryInto<U>>::try_into",
}
EXPECTED_PROTOCOL_FUNCTIONAL_STATES = {
    "kmipkit_protocol::SetConstraintsRequest": "verified",
    "kmipkit_protocol::SetConstraintsRequest::constraints": "verified",
    "kmipkit_protocol::SetConstraintsRequest::new": "verified",
    "kmipkit_protocol::SetConstraintsRequest::to_ttlv_payload": "blocked",
}
EXPECTED_ROOT_FACADE_STATES = {
    "kmipkit::SetConstraintsRequest": "verified",
    "kmipkit::SetConstraintsRequest::constraints": "verified",
    "kmipkit::SetConstraintsRequest::new": "verified",
    "kmipkit::SetConstraintsRequest::to_ttlv_payload": "blocked",
}


class T102SetConstraintsRequestTraitExclusionTests(unittest.TestCase):
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
                for evidence in (
                    "KMIPKIT-0039-FR-001",
                    "KMIPKIT-0039-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                    "OASIS Key Management Interoperability Protocol Specification v2.1 §6.1.52 Tables 325–327",
                    "Table 325 as request",
                    "No direct Set Constraints test-case ID appears in the approved catalog",
                    "inventory-only checkpoint does not claim OASIS conformance or interoperability",
                ):
                    self.assertIn(evidence, row["evidence"])
                self.assertNotIn(chr(0xFFFD), row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("SetConstraintsRequest", row["reason"])

    def test_rust_audit_reports_exact_five_direct_and_five_conditional_paths(self) -> None:
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

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(direct, EXPECTED_DIRECT_TRAITS)
        self.assertEqual(conditional, EXPECTED_CONDITIONAL_TRAITS)
        self.assertEqual(excluded, EXPECTED_PROTOCOL_TRAIT_EXPORTS)

    def test_functional_protocol_and_root_facade_states_remain_unchanged(self) -> None:
        protocol_states = {
            row["exportName"]: row["declarationState"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "kmipkit_protocol::SetConstraintsRequest"
            )
            and not row["exportName"].startswith("<")
        }
        root_states = {
            row["exportName"]: row["declarationState"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(
                "kmipkit::SetConstraintsRequest"
            )
        }

        self.assertEqual(protocol_states, EXPECTED_PROTOCOL_FUNCTIONAL_STATES)
        self.assertEqual(root_states, EXPECTED_ROOT_FACADE_STATES)

    def test_candidate_count_reflects_only_these_ten_exclusions(self) -> None:
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            T102_BLOCKED_CANDIDATES,
        )


if __name__ == "__main__":
    unittest.main()
