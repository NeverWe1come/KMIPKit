"""Pin the exact KMIPKIT-0043 exclusions for Import request traits."""

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
    "path": "specs/023-key-material-transfer/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::ImportRequest as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::ImportRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::ImportRequest as core::borrow::Borrow<kmipkit_protocol::import::ImportRequest>>::borrow",
    "<kmipkit_protocol::ImportRequest as core::borrow::BorrowMut<kmipkit_protocol::import::ImportRequest>>::borrow_mut",
    "<kmipkit_protocol::ImportRequest as core::convert::From<kmipkit_protocol::import::ImportRequest>>::from",
    "<kmipkit_protocol::ImportRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::ImportRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::ImportRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::ImportRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::ImportRequest as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::ImportRequest as core::fmt::Debug>::fmt",
}


def _root_import_request_exports(rows: list[dict[str, str]]) -> set[str]:
    return {
        row["exportName"]
        for row in rows
        if row.get("packageId") == "rust-kmipkit"
        and row.get("exportName", "").startswith(
            ("kmipkit::ImportRequest", "<kmipkit::ImportRequest as ")
        )
    }


class T102ImportRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_root_export_filter_recognizes_compiler_trait_path_format(self) -> None:
        trait_path = "<kmipkit::ImportRequest as core::fmt::Debug>::fmt"
        self.assertEqual(
            _root_import_request_exports(
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
                for requirement_id in (
                    "KMIPKIT-0037-FR-001",
                    "KMIPKIT-0037-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn(
                    "KMIP v2.1 Section 6.1.25 Tables 238-240, including the Table 238 request",
                    row["evidence"],
                )
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("ImportRequest", row["reason"])

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

    def test_request_mapping_and_unmapped_root_surface_remain_unchanged(self) -> None:
        protocol_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "kmipkit_protocol::ImportRequest"
            )
        }
        verified = {
            "kmipkit_protocol::ImportRequest",
            "kmipkit_protocol::ImportRequest::new",
        }
        blocked = {
            "kmipkit_protocol::ImportRequest::attributes",
            "kmipkit_protocol::ImportRequest::into_ttlv_payload",
            "kmipkit_protocol::ImportRequest::key_wrap_type",
            "kmipkit_protocol::ImportRequest::object",
            "kmipkit_protocol::ImportRequest::object_type",
            "kmipkit_protocol::ImportRequest::replace_existing",
            "kmipkit_protocol::ImportRequest::unique_identifier",
            "kmipkit_protocol::ImportRequest::with_key_wrap_type",
            "kmipkit_protocol::ImportRequest::with_replace_existing",
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
            _root_import_request_exports(self.manifest["publicDeclarations"]),
            set(),
        )
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            3520,
        )


if __name__ == "__main__":
    unittest.main()
