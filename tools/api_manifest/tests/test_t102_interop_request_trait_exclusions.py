"""Pin the exact KMIPKIT-0043 exclusions for Interop request traits."""

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
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::InteropRequest as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::InteropRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::InteropRequest as core::borrow::Borrow<kmipkit_protocol::interop::InteropRequest>>::borrow",
    "<kmipkit_protocol::InteropRequest as core::borrow::BorrowMut<kmipkit_protocol::interop::InteropRequest>>::borrow_mut",
    "<kmipkit_protocol::InteropRequest as core::convert::From<kmipkit_protocol::interop::InteropRequest>>::from",
    "<kmipkit_protocol::InteropRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::InteropRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::InteropRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::InteropRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::InteropRequest as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::InteropRequest as core::fmt::Debug>::fmt",
}
EXPECTED_DIRECT_TRAITS = {
    "<kmipkit_protocol::InteropRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::InteropRequest as core::borrow::Borrow<kmipkit_protocol::interop::InteropRequest>>::borrow",
    "<kmipkit_protocol::InteropRequest as core::borrow::BorrowMut<kmipkit_protocol::interop::InteropRequest>>::borrow_mut",
    "<kmipkit_protocol::InteropRequest as core::convert::From<kmipkit_protocol::interop::InteropRequest>>::from",
    "<kmipkit_protocol::InteropRequest as core::fmt::Debug>::fmt",
}
EXPECTED_CONDITIONAL_TRAITS = EXPECTED_PROTOCOL_TRAIT_EXPORTS - EXPECTED_DIRECT_TRAITS
EXPECTED_INTEROP_RESPONSE_TRAITS = {
    "<kmipkit_protocol::InteropResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::InteropResponse as core::borrow::Borrow<kmipkit_protocol::interop::InteropResponse>>::borrow",
    "<kmipkit_protocol::InteropResponse as core::borrow::BorrowMut<kmipkit_protocol::interop::InteropResponse>>::borrow_mut",
    "<kmipkit_protocol::InteropResponse as core::convert::From<kmipkit_protocol::interop::InteropResponse>>::from",
    "<kmipkit_protocol::InteropResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::InteropResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::InteropResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::InteropResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::InteropResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::InteropResponse as core::fmt::Debug>::fmt",
}


def _root_interop_request_exports(rows: list[dict[str, str]]) -> set[str]:
    return {
        row["exportName"]
        for row in rows
        if row.get("packageId") == "rust-kmipkit"
        and row.get("exportName", "").startswith(
            ("kmipkit::InteropRequest", "<kmipkit::InteropRequest as ")
        )
    }


class T102InteropRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_root_export_filter_recognizes_compiler_trait_path_format(self) -> None:
        trait_path = "<kmipkit::InteropRequest as core::fmt::Debug>::fmt"
        self.assertEqual(
            _root_interop_request_exports(
                [{"packageId": "rust-kmipkit", "exportName": trait_path}]
            ),
            {trait_path},
        )

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
                    "KMIPKIT-0042-FR-001",
                    "KMIPKIT-0042-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                    "KMIP v2.1 §6.1.26 Tables 241–243",
                    "Table 241 request",
                    "No direct OASIS test case",
                    "does not claim OASIS conformance or interoperability",
                ):
                    self.assertIn(evidence, row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("InteropRequest", row["reason"])

    def test_rust_audit_reports_exact_direct_and_conditional_trait_paths(self) -> None:
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
        self.assertEqual(len(direct), 5)
        self.assertEqual(len(conditional), 5)
        self.assertEqual(excluded, EXPECTED_PROTOCOL_TRAIT_EXPORTS)

    def test_request_mapping_root_surface_and_response_exclusions_remain_unchanged(self) -> None:
        protocol_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::InteropRequest")
        }
        verified = {
            "kmipkit_protocol::InteropRequest",
            "kmipkit_protocol::InteropRequest::new",
            "kmipkit_protocol::InteropRequest::into_ttlv_payload",
        }
        blocked = {
            "kmipkit_protocol::InteropRequest::interop_function",
            "kmipkit_protocol::InteropRequest::with_extension",
            "kmipkit_protocol::InteropRequest::with_interop_identifier",
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
        self.assertEqual(
            _root_interop_request_exports(self.manifest["publicDeclarations"]), set()
        )

        response_traits = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "<kmipkit_protocol::InteropResponse as "
            )
        }
        self.assertEqual(response_traits, EXPECTED_INTEROP_RESPONSE_TRAITS)

    def test_blocked_candidate_count_decreases_by_exactly_ten(self) -> None:
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
