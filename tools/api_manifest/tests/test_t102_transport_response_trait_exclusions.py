"""Exact T102 inventory contract for Rust-only TransportResponse trait artifacts."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = REPOSITORY_ROOT / "specification" / "api" / "public-api.json"
ADR_PATH = "docs/adr/0014-public-transport-exchange-contract.md"
EXPECTED_TRANSPORT_RESPONSE_TRAITS = {
    "<kmipkit_transport::TransportResponse as core::any::Any>::type_id",
    "<kmipkit_transport::TransportResponse as core::borrow::Borrow<kmipkit_transport::response::TransportResponse>>::borrow",
    "<kmipkit_transport::TransportResponse as core::borrow::BorrowMut<kmipkit_transport::response::TransportResponse>>::borrow_mut",
    "<kmipkit_transport::TransportResponse as core::convert::From<kmipkit_transport::response::TransportResponse>>::from",
    "<kmipkit_transport::TransportResponse as core::convert::Into<U>>::into",
    "<kmipkit_transport::TransportResponse as core::convert::TryFrom<U>>::Error",
    "<kmipkit_transport::TransportResponse as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_transport::TransportResponse as core::convert::TryInto<U>>::Error",
    "<kmipkit_transport::TransportResponse as core::convert::TryInto<U>>::try_into",
    "<kmipkit_transport::TransportResponse as core::fmt::Debug>::fmt",
    "<kmipkit_transport::TransportResponse as core::ops::drop::Drop>::drop",
}


class T102TransportResponseTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_exact_transport_response_trait_exports_have_adr_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-transport"
            and row.get("exportName") in EXPECTED_TRANSPORT_RESPONSE_TRAITS
        }
        exclusions = [
            row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-transport"
            and row.get("exportName") in EXPECTED_TRANSPORT_RESPONSE_TRAITS
        ]

        self.assertEqual(set(declarations), set())
        self.assertEqual(
            {row["exportName"] for row in exclusions},
            EXPECTED_TRANSPORT_RESPONSE_TRAITS,
        )
        self.assertEqual(len(exclusions), len(EXPECTED_TRANSPORT_RESPONSE_TRAITS))

        for row in exclusions:
            with self.subTest(export_name=row["exportName"]):
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("not a separate KMIPKit product capability", row["reason"])
                self.assertEqual(
                    row["approvalSource"],
                    {
                        "kind": "accepted-adr",
                        "path": ADR_PATH,
                        "status": "Accepted",
                        "evidence": "**Status**: Accepted under delegated maintainer authorization on 2026-10-06.",
                    },
                )
                self.assertIn("ADR-0014 Decision 1/6", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-020", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("redacted Debug", row["evidence"])
                self.assertIn("initialized byte range", row["evidence"])

    def test_rust_only_response_capability_and_security_contract_remain_declared(self) -> None:
        retained_exports = {
            "kmipkit_transport::Transport::exchange",
            "kmipkit_transport::TransportResponse",
            "kmipkit_transport::TransportResponse::new",
            "kmipkit_transport::TransportResponse::as_bytes",
        }
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-transport"
            and row.get("exportName") in retained_exports
        }
        exclusions = {
            row["exportName"]
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-transport"
        }

        self.assertEqual(set(declarations), retained_exports)
        self.assertTrue(all(row["declarationState"] == "declared" for row in declarations.values()))
        self.assertTrue(all(row["parityScope"] == "rust-only-adr" for row in declarations.values()))
        self.assertTrue(retained_exports.isdisjoint(exclusions))

        response_tests = (
            REPOSITORY_ROOT / "crates" / "kmipkit-transport" / "tests" / "unit" / "response_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("fn response_debug_redacts_initialized_response_bytes", response_tests)
        self.assertIn("fn successful_response_drop_zeroizes_initialized_bytes_before_release", response_tests)
        self.assertIn("fn partial_read_error_drops_zeroized_response_and_redacts_source", response_tests)
