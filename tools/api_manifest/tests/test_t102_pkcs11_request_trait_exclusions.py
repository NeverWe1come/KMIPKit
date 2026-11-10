"""Pin the exact KMIPKIT-0043 exclusions for PKCS#11 request trait paths."""

from __future__ import annotations

import json
import unittest

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
    T102_RUST_AUDIT_EXCLUDED,
    T102_RUST_AUDIT_MANIFEST,
    T102_RUST_AUDIT_OBSERVED,
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
PKCS11_RESEARCH = ROOT / "specs" / "026-pkcs11" / "research.md"
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/026-pkcs11/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
PROTOCOL_PREFIX = "<kmipkit_protocol::Pkcs11Request as "
EXPECTED_PROTOCOL_TRAIT_EXPORTS = {
    "<kmipkit_protocol::Pkcs11Request as core::any::Any>::type_id",
    "<kmipkit_protocol::Pkcs11Request as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow",
    "<kmipkit_protocol::Pkcs11Request as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow_mut",
    "<kmipkit_protocol::Pkcs11Request as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Request>>::from",
    "<kmipkit_protocol::Pkcs11Request as core::convert::Into<U>>::into",
    "<kmipkit_protocol::Pkcs11Request as core::convert::TryFrom<U>>::Error",
    "<kmipkit_protocol::Pkcs11Request as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_protocol::Pkcs11Request as core::convert::TryInto<U>>::Error",
    "<kmipkit_protocol::Pkcs11Request as core::convert::TryInto<U>>::try_into",
    "<kmipkit_protocol::Pkcs11Request as core::fmt::Debug>::fmt",
}
EXPECTED_DIRECT_TRAITS = {
    "<kmipkit_protocol::Pkcs11Request as core::any::Any>::type_id",
    "<kmipkit_protocol::Pkcs11Request as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow",
    "<kmipkit_protocol::Pkcs11Request as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow_mut",
    "<kmipkit_protocol::Pkcs11Request as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Request>>::from",
    "<kmipkit_protocol::Pkcs11Request as core::fmt::Debug>::fmt",
}
EXPECTED_CONDITIONAL_TRAITS = EXPECTED_PROTOCOL_TRAIT_EXPORTS - EXPECTED_DIRECT_TRAITS
EXPECTED_CLIENT_REQUEST_TRAITS = {
    "<kmipkit_client::Pkcs11Request as core::any::Any>::type_id",
    "<kmipkit_client::Pkcs11Request as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow",
    "<kmipkit_client::Pkcs11Request as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow_mut",
    "<kmipkit_client::Pkcs11Request as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Request>>::from",
    "<kmipkit_client::Pkcs11Request as core::convert::Into<U>>::into",
    "<kmipkit_client::Pkcs11Request as core::convert::TryFrom<U>>::Error",
    "<kmipkit_client::Pkcs11Request as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_client::Pkcs11Request as core::convert::TryInto<U>>::Error",
    "<kmipkit_client::Pkcs11Request as core::convert::TryInto<U>>::try_into",
    "<kmipkit_client::Pkcs11Request as core::fmt::Debug>::fmt",
}
EXPECTED_PKCS11_RESPONSE_TRAITS = {
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


class T102Pkcs11RequestTraitExclusionTests(unittest.TestCase):
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
                    "KMIPKIT-0040-FR-001",
                    "KMIPKIT-0040-FR-002",
                    "KMIPKIT-0040-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                    "OASIS Key Management Interoperability Protocol Specification v2.1 §6.1.37 Tables 273–275",
                    "Table 273 as request",
                    "No direct operation Test Case exists in the approved catalog",
                    "the official fixture and profile case PKCS11-M-1-21 are unavailable",
                    "makes no conformance, interoperability, profile-support, or certification claim",
                ):
                    self.assertIn(evidence, row["evidence"])
                self.assertNotIn(chr(0xFFFD), row["evidence"])
                self.assertNotIn("KMIPKIT-0040-FR-005", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("Pkcs11Request", row["reason"])

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
        self.assertEqual(len(direct), 5)
        self.assertEqual(len(conditional), 5)
        self.assertEqual(excluded, EXPECTED_PROTOCOL_TRAIT_EXPORTS)

    def test_request_methods_root_alias_and_separate_client_request_remain_unchanged(self) -> None:
        protocol_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith("kmipkit_protocol::Pkcs11Request")
        }
        verified = {
            "kmipkit_protocol::Pkcs11Request",
            "kmipkit_protocol::Pkcs11Request::new",
            "kmipkit_protocol::Pkcs11Request::into_ttlv_payload",
            "kmipkit_protocol::Pkcs11Request::with_correlation_value",
            "kmipkit_protocol::Pkcs11Request::with_extension",
            "kmipkit_protocol::Pkcs11Request::with_input_parameters",
            "kmipkit_protocol::Pkcs11Request::with_interface",
        }
        blocked: set[str] = set()
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
            {
                row["exportName"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and row.get("exportName", "").startswith("kmipkit::Pkcs11Request")
            },
            set(),
        )

        client_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and "Pkcs11Request" in row.get("exportName", "")
        }
        self.assertIn("kmipkit_client::Pkcs11Request", client_rows)
        self.assertFalse(
            {
                name
                for name in client_rows
                if name.startswith("<kmipkit_client::Pkcs11Request as ")
            }
        )
        client_trait_exclusions = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName", "").startswith(
                "<kmipkit_client::Pkcs11Request as "
            )
        }
        self.assertEqual(client_trait_exclusions, EXPECTED_CLIENT_REQUEST_TRAITS)
        expected_client_request_declarations = {
            "kmipkit_client::Pkcs11Request",
            "kmipkit_client::Pkcs11Request::new",
            "kmipkit_client::Pkcs11Request::into_ttlv_payload",
            "kmipkit_client::Pkcs11Request::with_correlation_value",
            "kmipkit_client::Pkcs11Request::with_extension",
            "kmipkit_client::Pkcs11Request::with_input_parameters",
            "kmipkit_client::Pkcs11Request::with_interface",
        }
        client_functional_rows = {
            name: row
            for name, row in client_rows.items()
            if name.startswith("kmipkit_client::Pkcs11Request")
        }
        self.assertEqual(
            set(client_functional_rows), expected_client_request_declarations
        )
        self.assertEqual(
            {
                name
                for name, row in client_functional_rows.items()
                if row["declarationState"] == "verified"
            },
            {
                "kmipkit_client::Pkcs11Request",
                "kmipkit_client::Pkcs11Request::new",
                "kmipkit_client::Pkcs11Request::into_ttlv_payload",
                "kmipkit_client::Pkcs11Request::with_correlation_value",
                "kmipkit_client::Pkcs11Request::with_extension",
                "kmipkit_client::Pkcs11Request::with_input_parameters",
                "kmipkit_client::Pkcs11Request::with_interface",
            },
        )
        self.assertEqual(
            {
                name
                for name, row in client_functional_rows.items()
                if row["declarationState"] == "blocked"
            },
            expected_client_request_declarations
            - {
                "kmipkit_client::Pkcs11Request",
                "kmipkit_client::Pkcs11Request::new",
                "kmipkit_client::Pkcs11Request::into_ttlv_payload",
                "kmipkit_client::Pkcs11Request::with_correlation_value",
                "kmipkit_client::Pkcs11Request::with_extension",
                "kmipkit_client::Pkcs11Request::with_input_parameters",
                "kmipkit_client::Pkcs11Request::with_interface",
            },
        )

    def test_existing_protocol_response_trait_exclusions_remain_exact(self) -> None:
        response_traits = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "<kmipkit_protocol::Pkcs11Response as "
            )
        }
        self.assertEqual(response_traits, EXPECTED_PKCS11_RESPONSE_TRAITS)

    def test_vendor_function_source_interpretation_remains_recorded(self) -> None:
        research = PKCS11_RESEARCH.read_text(encoding="utf-8")
        self.assertIn("KMIPKIT-026-SOURCE-001", research)
        self.assertIn("§7.26 Table 378", research)
        self.assertIn("vendor-defined functions", research)

    def test_t102_slice_and_manifest_snapshots_remain_consistent(self) -> None:
        # KMIPKIT-0047 adds 143 exact versioned value declarations to the
        # checked-in API inventory; keep the aggregate snapshot current.
        self.assertEqual(T102_PUBLIC_DECLARATIONS, 12_017)
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            T102_BLOCKED_CANDIDATES,
        )
        self.assertEqual(len(self.manifest["publicDeclarations"]), T102_PUBLIC_DECLARATIONS)
        self.assertEqual(len(self.manifest["approvedExclusions"]), T102_APPROVED_EXCLUSIONS)
        self.assertEqual(self.rust_report["counts"]["manifest"], T102_RUST_AUDIT_MANIFEST)
        self.assertEqual(self.rust_report["counts"]["excluded"], T102_RUST_AUDIT_EXCLUDED)
        self.assertEqual(self.rust_report["counts"]["observed"], T102_RUST_AUDIT_OBSERVED)


if __name__ == "__main__":
    unittest.main()
