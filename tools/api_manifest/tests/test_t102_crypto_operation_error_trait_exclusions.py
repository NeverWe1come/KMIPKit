"""Pin the T102 cryptographic operation error trait inventory batch."""

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
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REPORT_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)

PACKAGE = "rust-kmipkit-protocol"
CRATE = "kmipkit_protocol"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/019-cryptographic-operations/spec.md",
    "status": "Approved",
    "evidence": (
        "**Status**: Approved for implementation under the maintainer's standing "
        "direct instruction to execute the complete KMIPKit plan autonomously "
        "(reaffirmed 2026-10-19)."
    ),
}
OWNER_REQUIREMENTS = ("KMIPKIT-0019-FR-007", "KMIPKIT-0019-FR-009")
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
DIRECT_SUFFIXES = {
    "CryptographicOperationError": {
        "alloc::borrow::ToOwned>::Owned",
        "alloc::borrow::ToOwned>::clone_into",
        "alloc::borrow::ToOwned>::to_owned",
        "alloc::string::ToString>::to_string",
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::borrow_mut",
        "core::clone::Clone>::clone",
        "core::clone::Clone>::clone_from",
        "core::clone::CloneToUninit>::clone_to_uninit",
        "core::cmp::Eq>::assert_receiver_is_total_eq",
        "core::cmp::PartialEq>::eq",
        "core::cmp::PartialEq>::ne",
        "core::convert::From<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::from",
        "core::error::Error>::cause",
        "core::error::Error>::description",
        "core::error::Error>::provide",
        "core::error::Error>::source",
        "core::error::Error>::type_id",
        "core::fmt::Debug>::fmt",
        "core::fmt::Display>::fmt",
    },
    "CryptographicOperationErrorKind": {
        "alloc::borrow::ToOwned>::Owned",
        "alloc::borrow::ToOwned>::clone_into",
        "alloc::borrow::ToOwned>::to_owned",
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::cryptographic_operation::CryptographicOperationErrorKind>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::cryptographic_operation::CryptographicOperationErrorKind>>::borrow_mut",
        "core::clone::Clone>::clone",
        "core::clone::Clone>::clone_from",
        "core::clone::CloneToUninit>::clone_to_uninit",
        "core::cmp::Eq>::assert_receiver_is_total_eq",
        "core::cmp::PartialEq>::eq",
        "core::cmp::PartialEq>::ne",
        "core::convert::From<kmipkit_protocol::cryptographic_operation::CryptographicOperationErrorKind>>::from",
        "core::fmt::Debug>::fmt",
    },
}
CONDITIONAL_SUFFIXES = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}


def expected_paths(type_name: str, suffixes: set[str]) -> set[str]:
    return {f"<{CRATE}::{type_name} as {suffix}" for suffix in suffixes}


class T102CryptoOperationErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_error_trait_paths_are_owner_backed_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        expected_total: set[str] = set()
        for type_name, direct_suffixes in DIRECT_SUFFIXES.items():
            direct = expected_paths(type_name, direct_suffixes)
            conditional = expected_paths(type_name, CONDITIONAL_SUFFIXES)
            expected = direct | conditional
            self.assertEqual(len(direct), 21 if type_name == "CryptographicOperationError" else 14)
            self.assertEqual(len(conditional), 5)

            for key, expected_paths_for_kind in (
                ("declarations", direct),
                ("conditionalDeclarations", conditional),
            ):
                observed = {
                    row["exportName"]
                    for row in self.report[key]
                    if row.get("packageId") == PACKAGE
                    and row.get("exportName", "").startswith(f"<{CRATE}::{type_name} as ")
                }
                self.assertEqual(observed, expected_paths_for_kind, (type_name, key))

            declarations = {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(f"<{CRATE}::{type_name} as ")
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(f"<{CRATE}::{type_name} as ")
            }
            self.assertEqual(set(declarations), set())
            self.assertEqual(set(exclusions), expected)

            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    for requirement_id in (*OWNER_REQUIREMENTS, *T102_REQUIREMENTS):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(
                        "KMIP Specification v2.1 §6.1.11/Table 198 (Decrypt Errors) and §6.1.17/Table 216 (Encrypt Errors)",
                        row["evidence"],
                    )
                    self.assertIn("compiler-observed Rust standard-trait paths", row["evidence"])
                    self.assertIn("No OASIS requirement is assigned to Rust traits", row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("no separate KMIP client capability", row["reason"])

            reported = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(f"<{CRATE}::{type_name} as ")
            }
            self.assertEqual(reported, expected)
            expected_total.update(expected)

        self.assertEqual(len(expected_total), 45)

    def test_error_type_declarations_are_mapped_to_shared_client_outcomes(self) -> None:
        for type_name in DIRECT_SUFFIXES:
            row = next(
                declaration
                for declaration in self.manifest["publicDeclarations"]
                if declaration.get("packageId") == PACKAGE
                and declaration.get("exportName") == f"{CRATE}::{type_name}"
            )
            self.assertEqual(row["declarationState"], "declared")
            self.assertNotIn("candidateSpecId", row)
            self.assertEqual(row.get("capabilityId"), "client_execution")

    def test_manifest_totals_reflect_the_exact_error_trait_batch(self) -> None:
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, T102_BLOCKED_CANDIDATES)
        self.assertEqual(len(self.manifest["publicDeclarations"]), T102_PUBLIC_DECLARATIONS)
        self.assertEqual(len(self.manifest["approvedExclusions"]), T102_APPROVED_EXCLUSIONS)


if __name__ == "__main__":
    unittest.main()
