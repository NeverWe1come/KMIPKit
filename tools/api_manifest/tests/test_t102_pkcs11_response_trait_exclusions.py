"""Pin the exact KMIPKIT-0043 exclusions for PKCS#11 response trait paths."""

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
    "path": "specs/026-pkcs11/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::Pkcs11Response as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::Pkcs11Response as core::any::Any>::type_id",
    "<kmipkit_protocol::Pkcs11Response as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Response>>::borrow",
    "<kmipkit_protocol::Pkcs11Response as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Response>>::borrow_mut",
    "<kmipkit_protocol::Pkcs11Response as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Response>>::from",
    "<kmipkit_protocol::Pkcs11Response as core::convert::Into<U>>::into",
    "<kmipkit_protocol::Pkcs11Response as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::Pkcs11Response as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::Pkcs11Response as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::Pkcs11Response as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::Pkcs11Response as core::fmt::Debug>::fmt",
}


def _root_pkcs11_response_exports(rows: list[dict[str, str]]) -> set[str]:
    return {
        row["exportName"]
        for row in rows
        if row.get("packageId") == "rust-kmipkit"
        and row.get("exportName", "").startswith(
            ("kmipkit::Pkcs11Response", "<kmipkit::Pkcs11Response as ")
        )
    }


class T102Pkcs11ResponseTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_root_export_filter_covers_compiler_trait_path_shape(self) -> None:
        trait_path = "<kmipkit::Pkcs11Response as core::fmt::Debug>::fmt"
        self.assertEqual(
            _root_pkcs11_response_exports(
                [{"packageId": "rust-kmipkit", "exportName": trait_path}]
            ),
            {trait_path},
        )

    def test_protocol_response_traits_are_exact_approved_exclusions(self) -> None:
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
                    "KMIPKIT-0040-FR-001",
                    "KMIPKIT-0040-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn(
                    "KMIP v2.1 Section 6.1.37 Tables 273-275, including Table 274 response",
                    row["evidence"],
                )
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("Pkcs11Response", row["reason"])

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
        expected_direct = {
            name
            for name in EXPECTED_PROTOCOL_TRAIT_EXPORTS
            if " as core::convert::Into<U>" not in name
            and " as core::convert::TryFrom<U>" not in name
            and " as core::convert::TryInto<U>" not in name
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(direct, expected_direct)
        self.assertEqual(conditional, EXPECTED_PROTOCOL_TRAIT_EXPORTS - expected_direct)
        self.assertEqual(len(direct), 5)
        self.assertEqual(len(conditional), 5)
        self.assertEqual(excluded, EXPECTED_PROTOCOL_TRAIT_EXPORTS)

    def test_response_mapping_and_unmapped_root_surface_remain_unchanged(self) -> None:
        protocol_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "kmipkit_protocol::Pkcs11Response"
            )
        }
        verified = {
            "kmipkit_protocol::Pkcs11Response",
            "kmipkit_protocol::Pkcs11Response::with_correlation_value",
            "kmipkit_protocol::Pkcs11Response::with_function",
            "kmipkit_protocol::Pkcs11Response::with_interface",
            "kmipkit_protocol::Pkcs11Response::with_output_parameters",
            "kmipkit_protocol::Pkcs11Response::with_return_code",
            "kmipkit_protocol::Pkcs11Response::with_ttlv",
            "kmipkit_protocol::Pkcs11Response::with_ttlv_payload",
        }
        blocked = {
            "kmipkit_protocol::Pkcs11Response::result",
            "kmipkit_protocol::Pkcs11Response::try_from_response_item",
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
        root_rows = _root_pkcs11_response_exports(self.manifest["publicDeclarations"])
        self.assertEqual(root_rows, set())
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            3078,
        )


if __name__ == "__main__":
    unittest.main()
