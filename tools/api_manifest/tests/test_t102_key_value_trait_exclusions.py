"""Exact T102 inventory and preservation contract for key value selectors."""

from __future__ import annotations

import json
import unittest
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
OWNER_SPEC_PATH = "specs/023-key-material-transfer/spec.md"
GET_OWNER_SPEC_PATH = "specs/017-managed-object-retrieval/spec.md"

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
        "source_context": (
            "§11.25, Table 461: Key Format Type Enumeration",
            "§6.1.18, Table 217: Export Request Payload",
            "§6.1.19, Table 220: Get Request Payload",
        ),
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
        "source_context": (
            "§11.29: Key Wrap Type Enumeration",
            "§6.1.18, Table 217: Export Request Payload",
            "§6.1.19, Table 220: Get Request Payload",
            "§6.1.25, Table 238: Import Request Payload",
        ),
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
        "source_context": (
            "§11.24, Table 459: Key Compression Type Enumeration values",
            "§6.1.18, Table 217: Export Request Payload",
            "§6.1.19, Table 220: Get Request Payload",
        ),
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
        "source_context": (
            "§11.14, Table 445: Derivation Method Enumeration Descriptions",
            "§11.14, Table 446: Derivation Method Enumeration",
            "§6.1.14, Table 205: Derive Key Request Payload",
        ),
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
                        self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                        self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                        self.assertIn(OWNER_SPEC_PATH, row["evidence"])
                        self.assertIn("Approved for implementation on 2026-10-11", row["evidence"])
                        self.assertIn("not Accepted", row["evidence"])
                        for requirement_id in details["owner_requirements"]:
                            self.assertIn(requirement_id, row["evidence"])
                        for source_context in details["source_context"]:
                            self.assertIn(source_context, row["evidence"])
                        self.assertIn(
                            "no OASIS clause applies to compiler-generated Rust traits",
                            row["evidence"],
                        )
                        self.assertIn("operation context only", row["evidence"])
                        if details["get_related"]:
                            self.assertIn(GET_OWNER_SPEC_PATH, row["evidence"])

        self.assertEqual(len(expected_all), 84)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 7501)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 7223)

    def test_parent_raw_paths_and_secret_bearing_models_remain_blocked(self) -> None:
        expected_blocked = {
            f"kmipkit_protocol::{type_name}{suffix}": "blocked"
            for type_name in KEY_VALUE_TYPES
            for suffix in ("", "::from_raw", "::raw")
        }
        expected_blocked.update(
            {
                "kmipkit_protocol::DerivationParameters": "blocked",
                "kmipkit_protocol::DeriveKeyRequest": "blocked",
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

        sensitive_prefixes = (
            "<kmipkit_protocol::DerivationParameters as ",
            "<kmipkit_protocol::DeriveKeyRequest as ",
        )
        sensitive_declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(sensitive_prefixes)
        ]
        self.assertTrue(sensitive_declarations)
        self.assertTrue(
            all(
                row["declarationState"] == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in sensitive_declarations
            )
        )
        self.assertFalse(
            any(
                row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(sensitive_prefixes)
                for row in self.manifest.get("approvedExclusions", [])
            )
        )


if __name__ == "__main__":
    unittest.main()
