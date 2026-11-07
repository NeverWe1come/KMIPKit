"""Exact T102 inventory and preservation contract for key value selectors."""

from __future__ import annotations

from tools.api_manifest.tests.pre_send_validation_routes import is_pre_send_validation_trait

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
RUST_REPORT_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
API_SPEC_PATH = "specs/043-public-api-bindings/spec.md"
TRACEABILITY_PATH = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"

KEY_VALUE_TYPES = {
    "KeyFormatType": {
        "implementation": "kmipkit_protocol::get::KeyFormatType",
        "owner_requirements": (
            "KMIPKIT-0037-FR-001",
            "KMIPKIT-0037-FR-002",
            "KMIPKIT-0037-FR-004",
            "KMIPKIT-0037-FR-008",
            "KMIPKIT-0017-FR-003",
            "KMIPKIT-0017-FR-005",
        ),
        "source_context": ("§11.25/Table 461",),
        "get_related": True,
    },
    "KeyWrapType": {
        "implementation": "kmipkit_protocol::get::KeyWrapType",
        "owner_requirements": (
            "KMIPKIT-0037-FR-001",
            "KMIPKIT-0037-FR-002",
            "KMIPKIT-0037-FR-004",
            "KMIPKIT-0037-FR-008",
            "KMIPKIT-0017-FR-003",
            "KMIPKIT-0017-FR-005",
        ),
        "source_context": ("§11.29",),
        "get_related": True,
    },
    "KeyCompressionType": {
        "implementation": "kmipkit_protocol::get::KeyCompressionType",
        "owner_requirements": (
            "KMIPKIT-0037-FR-001",
            "KMIPKIT-0037-FR-002",
            "KMIPKIT-0037-FR-004",
            "KMIPKIT-0037-FR-008",
            "KMIPKIT-0017-FR-003",
            "KMIPKIT-0017-FR-005",
        ),
        "source_context": ("§11.24/Table 459",),
        "get_related": True,
    },
    "DerivationMethod": {
        "implementation": "kmipkit_protocol::derive_key::DerivationMethod",
        "owner_requirements": (
            "KMIPKIT-0037-FR-001",
            "KMIPKIT-0037-FR-002",
            "KMIPKIT-0037-FR-004",
            "KMIPKIT-0037-FR-008",
        ),
        "source_context": ("§11.14/Tables 445/446",),
        "get_related": False,
    },
}


def expected_direct_traits(public_name: str, implementation_name: str) -> set[str]:
    prefix = f"<{public_name} as "
    return {
        f"{prefix}alloc::borrow::ToOwned>::Owned",
        f"{prefix}alloc::borrow::ToOwned>::clone_into",
        f"{prefix}alloc::borrow::ToOwned>::to_owned",
        f"{prefix}core::any::Any>::type_id",
        f"{prefix}core::borrow::Borrow<{implementation_name}>>::borrow",
        f"{prefix}core::borrow::BorrowMut<{implementation_name}>>::borrow_mut",
        f"{prefix}core::clone::Clone>::clone",
        f"{prefix}core::clone::Clone>::clone_from",
        f"{prefix}core::clone::CloneToUninit>::clone_to_uninit",
        f"{prefix}core::cmp::Eq>::assert_receiver_is_total_eq",
        f"{prefix}core::cmp::PartialEq>::eq",
        f"{prefix}core::cmp::PartialEq>::ne",
        f"{prefix}core::convert::From<{implementation_name}>>::from",
        f"{prefix}core::fmt::Debug>::fmt",
        f"{prefix}core::hash::Hash>::hash",
        f"{prefix}core::hash::Hash>::hash_slice",
    }


def expected_conditional_traits(public_name: str) -> set[str]:
    prefix = f"<{public_name} as "
    return {
        f"{prefix}core::convert::Into<U>>::into",
        f"{prefix}core::convert::TryFrom<U>>::Error",
        f"{prefix}core::convert::TryFrom<U>>::try_from",
        f"{prefix}core::convert::TryInto<U>>::Error",
        f"{prefix}core::convert::TryInto<U>>::try_into",
    }


class T102KeyValueTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_compiler_paths_have_owner_backed_exclusions(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertEqual(self.rust_report["issues"], [])
        self.assertEqual(self.rust_report["missingFromManifest"], [])
        self.assertEqual(self.rust_report["notObserved"], [])
        self.assertEqual(self.rust_report["kindMismatches"], [])
        self.assertEqual(self.rust_report["conditionalConditionMismatches"], [])
        self.assertEqual(self.rust_report["conditionalStateMismatches"], [])

        expected_all: set[str] = set()
        for type_name, details in KEY_VALUE_TYPES.items():
            public_type = f"kmipkit_protocol::{type_name}"
            direct = expected_direct_traits(public_type, details["implementation"])
            conditional = expected_conditional_traits(public_type)
            expected = direct | conditional
            expected_all.update(expected)

            with self.subTest(type_name=type_name):
                observed_direct = {
                    row["exportName"]
                    for row in self.rust_report["declarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                }
                observed_conditional = {
                    row["exportName"]
                    for row in self.rust_report["conditionalDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                }
                self.assertEqual(observed_direct, direct)
                self.assertEqual(observed_conditional, conditional)
                self.assertEqual(len(direct), 16)
                self.assertEqual(len(conditional), 5)

                declarations = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName") in expected
                }
                exclusions = {
                    row["exportName"]: row
                    for row in self.manifest.get("approvedExclusions", [])
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName") in expected
                }
                self.assertEqual(set(declarations), set())
                self.assertEqual(set(exclusions), expected)

                for row in exclusions.values():
                    with self.subTest(export_name=row["exportName"]):
                        self.assertIn("standard-trait implementation", row["reason"])
                        self.assertIn("not a separate KMIPKit product capability", row["reason"])
                        self.assertEqual(
                            row["approvalSource"],
                            {
                                "kind": "feature-specification",
                                "path": API_SPEC_PATH,
                                "status": "Approved",
                                "evidence": "**Status**: Approved for implementation",
                            },
                        )
                        self.assertIn("KMIPKIT-0043-FR-010/021", row["evidence"])
                        evidence = row["evidence"]
                        self.assertLessEqual(len(evidence.encode("utf-8")), 300)
                        self.assertIn(
                            "Approved for implementation 2026-10-11; not Accepted",
                            evidence,
                        )
                        self.assertIn("KMIPKIT-0037-FR-001/002/004/008", evidence)
                        for source_context in details["source_context"]:
                            self.assertIn(source_context, evidence)
                        self.assertIn("Trait-only inventory", evidence)
                        self.assertIn("no OASIS trait requirement", evidence)
                        self.assertIn("new wire capability", evidence)
                        self.assertIn("Operation context only", evidence)
                        if details["get_related"]:
                            self.assertIn("KMIPKIT-0017-FR-003/005", evidence)

        self.assertEqual(len(expected_all), 84)
        self.assertEqual(len(self.manifest["publicDeclarations"]), T102_PUBLIC_DECLARATIONS)
        self.assertEqual(len(self.manifest["approvedExclusions"]), T102_APPROVED_EXCLUSIONS)

    def test_full_oasis_operation_context_is_recorded_separately(self) -> None:
        traceability = TRACEABILITY_PATH.read_text(encoding="utf-8")
        for context in (
            "§6.1.14, Table 205: Derive Key Request Payload",
            "§6.1.18, Table 217: Export Request Payload",
            "§6.1.19, Table 220: Get Request Payload",
            "§6.1.25, Table 238: Import Request Payload",
            "Approved for implementation by maintainer (2026-10-11), not Accepted",
            "DerivationParameters",
            "DeriveKeyRequest",
        ):
            with self.subTest(context=context):
                self.assertIn(context, traceability)

    def test_parent_raw_paths_are_verified_and_secret_traits_are_redacted_exclusions(self) -> None:
        expected_blocked = {
            f"kmipkit_protocol::{type_name}{suffix}": "verified"
            for type_name in KEY_VALUE_TYPES
            for suffix in ("", "::from_raw", "::raw")
        }
        expected_blocked.update(
            {
                "kmipkit_protocol::DerivationParameters": "verified",
                "kmipkit_protocol::DeriveKeyRequest": "verified",
            }
        )
        observed = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName") in expected_blocked
        }
        self.assertEqual(set(observed), set(expected_blocked))
        self.assertEqual(
            {name: row["declarationState"] for name, row in observed.items()},
            expected_blocked,
        )

        expected_trait_counts = {"DerivationParameters": 11, "DeriveKeyRequest": 10}
        for type_name, expected_count in expected_trait_counts.items():
            prefix = f"<kmipkit_protocol::{type_name} as "
            declarations = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
                and not is_pre_send_validation_trait(row)
            ]
            exclusions = [
                row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            ]
            with self.subTest(type_name=type_name):
                self.assertEqual(declarations, [])
                self.assertEqual(len(exclusions), expected_count)
                self.assertTrue(
                    all(
                        row.get("candidateSpecId") == "KMIPKIT-0037"
                        and row["approvalSource"]["path"]
                        == "specs/023-key-material-transfer/spec.md"
                        for row in exclusions
                    )
                )
                redaction_evidence = (
                    "Debug emits presence and count metadata only"
                    if type_name == "DerivationParameters"
                    else "Debug emits only type codes and collection counts, with nested secret parameters redacted"
                )
                self.assertTrue(all(redaction_evidence in row["evidence"] for row in exclusions))


if __name__ == "__main__":
    unittest.main()
