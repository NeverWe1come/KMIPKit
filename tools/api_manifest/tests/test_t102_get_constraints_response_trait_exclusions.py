"""Exact T102 operation-trait exclusion and preservation contracts.

The tests cover Constraints, Check, Certify, Derive Key, Export, Get, Import,
Join Split Key, Interop, Locate, Log, and Validate operations. Each operation's
approved owner and OASIS citation are asserted by its focused case;
compiler-visible Rust traits are not separate KMIP capabilities.
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
SET_PREFIX = "<kmipkit_protocol::SetConstraintsResponse as "
SET_ROOT_PREFIX = "<kmipkit::SetConstraintsResponse as "
CERTIFY_PREFIX = "<kmipkit_protocol::CertifyResponse as "
VALIDATE_PREFIX = "<kmipkit_protocol::ValidateResponse as "
CHECK_PREFIX = "<kmipkit_protocol::CheckResponse as "
DERIVE_KEY_PREFIX = "<kmipkit_protocol::DeriveKeyResponse as "
EXPORT_PREFIX = "<kmipkit_protocol::ExportResponse as "
GET_PREFIX = "<kmipkit_protocol::GetResponse as "
LOCATE_PREFIX = "<kmipkit_protocol::LocateResponse as "
INTEROP_PREFIX = "<kmipkit_protocol::InteropResponse as "
LOCATE_REQUEST_PREFIX = "<kmipkit_protocol::LocateRequest as "
LOG_PREFIX = "<kmipkit_protocol::LogResponse as "
IMPORT_PREFIX = "<kmipkit_protocol::ImportResponse as "
JOIN_SPLIT_RESPONSE_PREFIX = "<kmipkit_protocol::JoinSplitKeyResponse as "
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
EXPECTED_SET_TRAIT_EXPORTS = {
    "<kmipkit_protocol::SetConstraintsResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::SetConstraintsResponse as core::borrow::Borrow<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::borrow",
    "<kmipkit_protocol::SetConstraintsResponse as core::borrow::BorrowMut<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::borrow_mut",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::From<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::from",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::SetConstraintsResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::SetConstraintsResponse as core::fmt::Debug>::fmt",
}
EXPECTED_SET_ROOT_TRAIT_EXPORTS = {
    "<kmipkit::SetConstraintsResponse as core::any::Any>::type_id",
    "<kmipkit::SetConstraintsResponse as core::borrow::Borrow<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::borrow",
    "<kmipkit::SetConstraintsResponse as core::borrow::BorrowMut<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::borrow_mut",
    "<kmipkit::SetConstraintsResponse as core::convert::From<kmipkit_protocol::set_constraints::SetConstraintsResponse>>::from",
    "<kmipkit::SetConstraintsResponse as core::convert::Into<U>>::into",
    "<kmipkit::SetConstraintsResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit::SetConstraintsResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit::SetConstraintsResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit::SetConstraintsResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit::SetConstraintsResponse as core::fmt::Debug>::fmt",
}
EXPECTED_CERTIFY_TRAIT_EXPORTS = {
    "<kmipkit_protocol::CertifyResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::CertifyResponse as core::borrow::Borrow<kmipkit_protocol::certify::CertifyResponse>>::borrow",
    "<kmipkit_protocol::CertifyResponse as core::borrow::BorrowMut<kmipkit_protocol::certify::CertifyResponse>>::borrow_mut",
    "<kmipkit_protocol::CertifyResponse as core::convert::From<kmipkit_protocol::certify::CertifyResponse>>::from",
    "<kmipkit_protocol::CertifyResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::CertifyResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::CertifyResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::CertifyResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::CertifyResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::CertifyResponse as core::fmt::Debug>::fmt",
}
EXPECTED_VALIDATE_TRAIT_EXPORTS = {
    "<kmipkit_protocol::ValidateResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::ValidateResponse as core::borrow::Borrow<kmipkit_protocol::validate::ValidateResponse>>::borrow",
    "<kmipkit_protocol::ValidateResponse as core::borrow::BorrowMut<kmipkit_protocol::validate::ValidateResponse>>::borrow_mut",
    "<kmipkit_protocol::ValidateResponse as core::convert::From<kmipkit_protocol::validate::ValidateResponse>>::from",
    "<kmipkit_protocol::ValidateResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::ValidateResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::ValidateResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::ValidateResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::ValidateResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::ValidateResponse as core::fmt::Debug>::fmt",
}
EXPECTED_CHECK_TRAIT_EXPORTS = {
    "<kmipkit_protocol::CheckResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::CheckResponse as core::borrow::Borrow<kmipkit_protocol::check::CheckResponse>>::borrow",
    "<kmipkit_protocol::CheckResponse as core::borrow::BorrowMut<kmipkit_protocol::check::CheckResponse>>::borrow_mut",
    "<kmipkit_protocol::CheckResponse as core::convert::From<kmipkit_protocol::check::CheckResponse>>::from",
    "<kmipkit_protocol::CheckResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::CheckResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::CheckResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::CheckResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::CheckResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::CheckResponse as core::fmt::Debug>::fmt",
}
EXPECTED_DERIVE_KEY_TRAIT_EXPORTS = {
    "<kmipkit_protocol::DeriveKeyResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::DeriveKeyResponse as core::borrow::Borrow<kmipkit_protocol::derive_key::DeriveKeyResponse>>::borrow",
    "<kmipkit_protocol::DeriveKeyResponse as core::borrow::BorrowMut<kmipkit_protocol::derive_key::DeriveKeyResponse>>::borrow_mut",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::From<kmipkit_protocol::derive_key::DeriveKeyResponse>>::from",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::DeriveKeyResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::DeriveKeyResponse as core::fmt::Debug>::fmt",
}
EXPECTED_EXPORT_TRAIT_EXPORTS = {
    "<kmipkit_protocol::ExportResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::ExportResponse as core::borrow::Borrow<kmipkit_protocol::export::ExportResponse>>::borrow",
    "<kmipkit_protocol::ExportResponse as core::borrow::BorrowMut<kmipkit_protocol::export::ExportResponse>>::borrow_mut",
    "<kmipkit_protocol::ExportResponse as core::convert::From<kmipkit_protocol::export::ExportResponse>>::from",
    "<kmipkit_protocol::ExportResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::ExportResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::ExportResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::ExportResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::ExportResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::ExportResponse as core::fmt::Debug>::fmt",
}
EXPECTED_GET_TRAIT_EXPORTS = {
    "<kmipkit_protocol::GetResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::GetResponse as core::borrow::Borrow<kmipkit_protocol::get::GetResponse>>::borrow",
    "<kmipkit_protocol::GetResponse as core::borrow::BorrowMut<kmipkit_protocol::get::GetResponse>>::borrow_mut",
    "<kmipkit_protocol::GetResponse as core::convert::From<kmipkit_protocol::get::GetResponse>>::from",
    "<kmipkit_protocol::GetResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::GetResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::GetResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::GetResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::GetResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::GetResponse as core::fmt::Debug>::fmt",
}
EXPECTED_LOCATE_TRAIT_EXPORTS = {
    "<kmipkit_protocol::LocateResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::LocateResponse as core::borrow::Borrow<kmipkit_protocol::locate::LocateResponse>>::borrow",
    "<kmipkit_protocol::LocateResponse as core::borrow::BorrowMut<kmipkit_protocol::locate::LocateResponse>>::borrow_mut",
    "<kmipkit_protocol::LocateResponse as core::convert::From<kmipkit_protocol::locate::LocateResponse>>::from",
    "<kmipkit_protocol::LocateResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::LocateResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::LocateResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::LocateResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::LocateResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::LocateResponse as core::fmt::Debug>::fmt",
}
EXPECTED_INTEROP_TRAIT_EXPORTS = {
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
EXPECTED_LOCATE_REQUEST_TRAIT_EXPORTS = {
    "<kmipkit_protocol::LocateRequest as core::any::Any>::type_id",
    "<kmipkit_protocol::LocateRequest as core::borrow::Borrow<kmipkit_protocol::locate::LocateRequest>>::borrow",
    "<kmipkit_protocol::LocateRequest as core::borrow::BorrowMut<kmipkit_protocol::locate::LocateRequest>>::borrow_mut",
    "<kmipkit_protocol::LocateRequest as core::convert::From<kmipkit_protocol::locate::LocateRequest>>::from",
    "<kmipkit_protocol::LocateRequest as core::convert::Into<U>>::into",
    "<kmipkit_protocol::LocateRequest as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::LocateRequest as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::LocateRequest as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::LocateRequest as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::LocateRequest as core::fmt::Debug>::fmt",
}
EXPECTED_LOG_TRAIT_EXPORTS = {
    "<kmipkit_protocol::LogResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::LogResponse as core::borrow::Borrow<kmipkit_protocol::log::LogResponse>>::borrow",
    "<kmipkit_protocol::LogResponse as core::borrow::BorrowMut<kmipkit_protocol::log::LogResponse>>::borrow_mut",
    "<kmipkit_protocol::LogResponse as core::convert::From<kmipkit_protocol::log::LogResponse>>::from",
    "<kmipkit_protocol::LogResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::LogResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::LogResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::LogResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::LogResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::LogResponse as core::fmt::Debug>::fmt",
}
EXPECTED_IMPORT_TRAIT_EXPORTS = {
    "<kmipkit_protocol::ImportResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::ImportResponse as core::borrow::Borrow<kmipkit_protocol::import::ImportResponse>>::borrow",
    "<kmipkit_protocol::ImportResponse as core::borrow::BorrowMut<kmipkit_protocol::import::ImportResponse>>::borrow_mut",
    "<kmipkit_protocol::ImportResponse as core::convert::From<kmipkit_protocol::import::ImportResponse>>::from",
    "<kmipkit_protocol::ImportResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::ImportResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::ImportResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::ImportResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::ImportResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::ImportResponse as core::fmt::Debug>::fmt",
}
EXPECTED_JOIN_SPLIT_RESPONSE_TRAIT_EXPORTS = {
    "<kmipkit_protocol::JoinSplitKeyResponse as core::any::Any>::type_id",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::borrow::Borrow<kmipkit_protocol::join_split_key::JoinSplitKeyResponse>>::borrow",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::borrow::BorrowMut<kmipkit_protocol::join_split_key::JoinSplitKeyResponse>>::borrow_mut",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::From<kmipkit_protocol::join_split_key::JoinSplitKeyResponse>>::from",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::Into<U>>::into",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::JoinSplitKeyResponse as core::fmt::Debug>::fmt",
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

    def test_set_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(SET_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(SET_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_SET_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0039-FR-001", row["evidence"])
                self.assertIn("KMIPKIT-0039-FR-004", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.52 Tables 325–327", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("SetConstraintsResponse", row["reason"])

    def test_rust_audit_reports_exact_set_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(SET_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(SET_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_SET_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_SET_TRAIT_EXPORTS)

    def test_set_response_type_and_ttlv_members_remain_verified(self) -> None:
        verified = {
            "kmipkit_protocol::SetConstraintsResponse",
            "kmipkit_protocol::SetConstraintsResponse::result",
            "kmipkit_protocol::SetConstraintsResponse::with_ttlv",
        }
        blocked = {"kmipkit_protocol::SetConstraintsResponse::try_from_response_item"}
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::SetConstraintsResponse")
        }
        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS")
                self.assertIn("KMIPKIT-0039-FR-001", rows[name]["requirementIds"])


    def test_set_root_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(SET_ROOT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(SET_ROOT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_SET_ROOT_TRAIT_EXPORTS)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0039-FR-001",
                    "KMIPKIT-0039-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.52 Tables 325–327", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("SetConstraintsResponse", row["reason"])

    def test_rust_audit_reports_exact_set_root_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(SET_ROOT_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(SET_ROOT_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_SET_ROOT_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_SET_ROOT_TRAIT_EXPORTS)

    def test_set_root_response_facade_remains_blocked_except_existing_ttlv_constructor(self) -> None:
        blocked = {
            "kmipkit::SetConstraintsResponse",
            "kmipkit::SetConstraintsResponse::result",
            "kmipkit::SetConstraintsResponse::try_from_response_item",
        }
        ttlv_constructor = "kmipkit::SetConstraintsResponse::with_ttlv"
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::SetConstraintsResponse")
        }
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
            "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS",
        )


    def test_certify_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CERTIFY_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CERTIFY_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_CERTIFY_TRAIT_EXPORTS)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0036-FR-001", row["evidence"])
                self.assertIn("KMIPKIT-0036-FR-004", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.6 Tables 179–181", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("CertifyResponse", row["reason"])

    def test_rust_audit_reports_exact_certify_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CERTIFY_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CERTIFY_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_CERTIFY_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_CERTIFY_TRAIT_EXPORTS)

    def test_certify_response_functional_declarations_remain_in_their_current_states(self) -> None:
        verified = {
            "kmipkit_protocol::CertifyResponse",
            "kmipkit_protocol::CertifyResponse::unique_identifier",
            "kmipkit_protocol::CertifyResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::CertifyResponse::result",
            "kmipkit_protocol::CertifyResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::CertifyResponse")
        }
        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::CertifyResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_validate_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(VALIDATE_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(VALIDATE_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_VALIDATE_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0042-FR-001",
                    "KMIPKIT-0042-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.57 Tables 340–342", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("ValidateResponse", row["reason"])

    def test_rust_audit_reports_exact_validate_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(VALIDATE_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(VALIDATE_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_VALIDATE_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_VALIDATE_TRAIT_EXPORTS)

    def test_validate_response_functional_declarations_remain_verified(self) -> None:
        expected = {
            "kmipkit_protocol::ValidateResponse",
            "kmipkit_protocol::ValidateResponse::result",
            "kmipkit_protocol::ValidateResponse::try_from_response_item",
            "kmipkit_protocol::ValidateResponse::validity_indicator",
            "kmipkit_protocol::ValidateResponse::with_ttlv",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::ValidateResponse")
        }

        self.assertEqual(set(rows), expected)
        for export_name, row in rows.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["declarationState"], "verified")
                self.assertEqual(row["capabilityId"], "KMIPKIT-ELEM-OP-C2S-VALIDATE")
                self.assertIn("KMIPKIT-0042-FR-001", row["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::ValidateResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_check_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CHECK_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CHECK_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_CHECK_TRAIT_EXPORTS)
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
                self.assertIn("KMIP v2.1 §6.1.7 Tables 183–185", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("CheckResponse", row["reason"])

    def test_rust_audit_reports_exact_check_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CHECK_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(CHECK_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_CHECK_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_CHECK_TRAIT_EXPORTS)

    def test_check_response_preserves_verified_accessors_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::CheckResponse",
            "kmipkit_protocol::CheckResponse::cryptographic_usage_mask",
            "kmipkit_protocol::CheckResponse::lease_time",
            "kmipkit_protocol::CheckResponse::unique_identifier",
            "kmipkit_protocol::CheckResponse::usage_limits_count",
            "kmipkit_protocol::CheckResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::CheckResponse::result",
            "kmipkit_protocol::CheckResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::CheckResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-CHECK")
                self.assertIn("KMIPKIT-0036-FR-001", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::CheckResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_derive_key_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(DERIVE_KEY_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(DERIVE_KEY_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_DERIVE_KEY_TRAIT_EXPORTS)
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
                self.assertIn("KMIP v2.1 §6.1.14 Tables 205–207", row["evidence"])
                self.assertIn("§7.12 Table 364", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("DeriveKeyResponse", row["reason"])

    def test_rust_audit_reports_exact_derive_key_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(DERIVE_KEY_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(DERIVE_KEY_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_DERIVE_KEY_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_DERIVE_KEY_TRAIT_EXPORTS)

    def test_derive_key_response_preserves_verified_accessor_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::DeriveKeyResponse",
            "kmipkit_protocol::DeriveKeyResponse::unique_identifier",
            "kmipkit_protocol::DeriveKeyResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::DeriveKeyResponse::result",
            "kmipkit_protocol::DeriveKeyResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::DeriveKeyResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY")
                self.assertIn("KMIPKIT-0037-FR-001", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::DeriveKeyResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_export_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(EXPORT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(EXPORT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_EXPORT_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
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
                self.assertIn("KMIP v2.1 §6.1.18 Tables 217–219", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("ExportResponse", row["reason"])

    def test_rust_audit_reports_exact_export_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(EXPORT_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(EXPORT_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_EXPORT_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_EXPORT_TRAIT_EXPORTS)

    def test_export_response_preserves_verified_accessors_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::ExportResponse",
            "kmipkit_protocol::ExportResponse::attributes",
            "kmipkit_protocol::ExportResponse::object",
            "kmipkit_protocol::ExportResponse::object_type",
            "kmipkit_protocol::ExportResponse::unique_identifier",
            "kmipkit_protocol::ExportResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::ExportResponse::result",
            "kmipkit_protocol::ExportResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::ExportResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-EXPORT")
                self.assertIn("KMIPKIT-0037-FR-001", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::ExportResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_get_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(GET_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(GET_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_GET_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0017-FR-001",
                    "KMIPKIT-0017-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.19 Tables 220–222", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("GetResponse", row["reason"])

    def test_rust_audit_reports_exact_get_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(GET_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(GET_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_GET_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_GET_TRAIT_EXPORTS)

    def test_get_response_preserves_verified_accessors_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::GetResponse",
            "kmipkit_protocol::GetResponse::object",
            "kmipkit_protocol::GetResponse::object_type",
            "kmipkit_protocol::GetResponse::unique_identifier",
        }
        blocked = {
            "kmipkit_protocol::GetResponse::result",
            "kmipkit_protocol::GetResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::GetResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-GET")
                self.assertIn("KMIPKIT-0017-FR-001", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::GetResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_locate_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_LOCATE_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0017-FR-001",
                    "KMIPKIT-0017-FR-008",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.28 Tables 247–249", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("LocateResponse", row["reason"])

    def test_rust_audit_reports_exact_locate_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_LOCATE_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_LOCATE_TRAIT_EXPORTS)

    def test_locate_response_preserves_verified_accessors_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::LocateResponse",
            "kmipkit_protocol::LocateResponse::located_items",
            "kmipkit_protocol::LocateResponse::unique_identifiers",
        }
        blocked = {
            "kmipkit_protocol::LocateResponse::result",
            "kmipkit_protocol::LocateResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::LocateResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-LOCATE")
                self.assertIn("KMIPKIT-0017-FR-001", rows[name]["requirementIds"])
                self.assertIn("KMIPKIT-0017-FR-008", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::LocateResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_interop_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(INTEROP_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(INTEROP_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_INTEROP_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0042-FR-001",
                    "KMIPKIT-0042-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.26 Tables 241–243", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("InteropResponse", row["reason"])

    def test_rust_audit_reports_exact_interop_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(INTEROP_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(INTEROP_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_INTEROP_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_INTEROP_TRAIT_EXPORTS)

    def test_interop_response_preserves_verified_members_and_blocked_parser(self) -> None:
        verified = {
            "kmipkit_protocol::InteropResponse",
            "kmipkit_protocol::InteropResponse::result",
            "kmipkit_protocol::InteropResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::InteropResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::InteropResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-INTEROP")
                self.assertIn("KMIPKIT-0042-FR-001", rows[name]["requirementIds"])
                self.assertIn("KMIPKIT-0042-FR-008", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::InteropResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_locate_request_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_REQUEST_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_REQUEST_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_LOCATE_REQUEST_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0017-FR-001",
                    "KMIPKIT-0017-FR-006",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.28 Tables 247–249", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("LocateRequest", row["reason"])

    def test_rust_audit_reports_exact_locate_request_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_REQUEST_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOCATE_REQUEST_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_LOCATE_REQUEST_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_LOCATE_REQUEST_TRAIT_EXPORTS)

    def test_locate_request_preserves_verified_type_and_blocked_helpers(self) -> None:
        verified = {"kmipkit_protocol::LocateRequest"}
        blocked = {
            "kmipkit_protocol::LocateRequest::attributes",
            "kmipkit_protocol::LocateRequest::maximum_items",
            "kmipkit_protocol::LocateRequest::new",
            "kmipkit_protocol::LocateRequest::object_group_member",
            "kmipkit_protocol::LocateRequest::offset_items",
            "kmipkit_protocol::LocateRequest::storage_status_mask",
            "kmipkit_protocol::LocateRequest::to_ttlv_payload",
            "kmipkit_protocol::LocateRequest::with_maximum_items",
            "kmipkit_protocol::LocateRequest::with_object_group_member",
            "kmipkit_protocol::LocateRequest::with_offset_items",
            "kmipkit_protocol::LocateRequest::with_storage_status_mask",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::LocateRequest")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        self.assertEqual(
            rows["kmipkit_protocol::LocateRequest"]["capabilityId"],
            "KMIPKIT-ELEM-OP-C2S-LOCATE",
        )
        self.assertIn(
            "KMIPKIT-0017-FR-006",
            rows["kmipkit_protocol::LocateRequest"]["requirementIds"],
        )
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::LocateRequest")
        ]
        self.assertEqual(root_rows, [])

    def test_log_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOG_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOG_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_LOG_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                for requirement_id in (
                    "KMIPKIT-0042-FR-001",
                    "KMIPKIT-0042-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn("KMIP v2.1 §6.1.29 Tables 250–252", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("LogResponse", row["reason"])

    def test_rust_audit_reports_exact_log_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOG_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(LOG_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_LOG_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_LOG_TRAIT_EXPORTS)

    def test_log_response_preserves_verified_members_and_blocked_parser(self) -> None:
        verified = {
            "kmipkit_protocol::LogResponse",
            "kmipkit_protocol::LogResponse::result",
            "kmipkit_protocol::LogResponse::with_ttlv",
        }
        blocked = {"kmipkit_protocol::LogResponse::try_from_response_item"}
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::LogResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-LOG")
                self.assertIn("KMIPKIT-0042-FR-001", rows[name]["requirementIds"])
                self.assertIn("KMIPKIT-0042-FR-004", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::LogResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_import_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(IMPORT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(IMPORT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_IMPORT_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 10)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )
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
                self.assertIn("KMIP v2.1 §6.1.25 Tables 238–240", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("ImportResponse", row["reason"])

    def test_rust_audit_reports_exact_import_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(IMPORT_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(IMPORT_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_IMPORT_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_IMPORT_TRAIT_EXPORTS)

    def test_import_response_preserves_verified_accessors_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::ImportResponse",
            "kmipkit_protocol::ImportResponse::unique_identifier",
            "kmipkit_protocol::ImportResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::ImportResponse::result",
            "kmipkit_protocol::ImportResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::ImportResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-IMPORT")
                self.assertIn("KMIPKIT-0037-FR-001", rows[name]["requirementIds"])
                self.assertIn("KMIPKIT-0037-FR-004", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::ImportResponse")
        ]
        self.assertEqual(root_rows, [])


    def test_join_split_key_response_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(JOIN_SPLIT_RESPONSE_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(JOIN_SPLIT_RESPONSE_PREFIX)
        }

        self.assertEqual(declarations, set())
        self.assertEqual(set(exclusions), EXPECTED_JOIN_SPLIT_RESPONSE_TRAIT_EXPORTS)
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
                self.assertIn("KMIP v2.1 §6.1.27 Tables 244–246", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("JoinSplitKeyResponse", row["reason"])

    def test_rust_audit_reports_exact_join_split_key_response_trait_paths(self) -> None:
        observed = [
            *self.rust_report["declarations"],
            *self.rust_report["conditionalDeclarations"],
        ]
        observed_trait_paths = {
            row["exportName"]
            for row in observed
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(JOIN_SPLIT_RESPONSE_PREFIX)
        }
        audited_exclusions = {
            row["exportName"]
            for row in self.rust_report["excluded"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(JOIN_SPLIT_RESPONSE_PREFIX)
        }

        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(observed_trait_paths, EXPECTED_JOIN_SPLIT_RESPONSE_TRAIT_EXPORTS)
        self.assertEqual(audited_exclusions, EXPECTED_JOIN_SPLIT_RESPONSE_TRAIT_EXPORTS)

    def test_join_split_key_response_preserves_verified_accessor_and_blocked_helpers(self) -> None:
        verified = {
            "kmipkit_protocol::JoinSplitKeyResponse",
            "kmipkit_protocol::JoinSplitKeyResponse::unique_identifier",
            "kmipkit_protocol::JoinSplitKeyResponse::with_ttlv",
        }
        blocked = {
            "kmipkit_protocol::JoinSplitKeyResponse::result",
            "kmipkit_protocol::JoinSplitKeyResponse::try_from_response_item",
        }
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::JoinSplitKeyResponse")
        }

        self.assertEqual(set(rows), verified | blocked)
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "verified"},
            verified,
        )
        self.assertEqual(
            {name for name, row in rows.items() if row["declarationState"] == "blocked"},
            blocked,
        )
        for name in verified:
            with self.subTest(export_name=name):
                self.assertEqual(rows[name]["capabilityId"], "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY")
                self.assertIn("KMIPKIT-0037-FR-001", rows[name]["requirementIds"])
                self.assertIn("KMIPKIT-0037-FR-004", rows[name]["requirementIds"])
        root_rows = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith("kmipkit::JoinSplitKeyResponse")
        ]
        self.assertEqual(root_rows, [])

    def test_join_split_key_response_slice_has_expected_candidate_count(self) -> None:
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row.get("declarationState") == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            4048,
        )


if __name__ == "__main__":
    unittest.main()
