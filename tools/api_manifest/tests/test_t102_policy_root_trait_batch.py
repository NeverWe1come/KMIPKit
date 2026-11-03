"""Pin root policy-control trait exclusions and payload-sensitive holdouts."""

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
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/025-policy-controls/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation \u2014 2026-10-11",
}
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
ERROR_COMMON_DIRECT = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "alloc::string::ToString>::to_string",
    "core::any::Any>::type_id",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::error::Error>::cause",
    "core::error::Error>::description",
    "core::error::Error>::provide",
    "core::error::Error>::source",
    "core::error::Error>::type_id",
    "core::fmt::Debug>::fmt",
    "core::fmt::Display>::fmt",
}
ERRORS = {
    "GetConstraintsError": ("get_constraints", "6.1.22", "231", ("001", "003", "005", "006", "007")),
    "GetUsageAllocationError": ("get_usage_allocation", "6.1.23", "234", ("001", "003", "005", "006", "007")),
    "SetConstraintsError": ("set_constraints", "6.1.52", "327", ("001", "003", "005", "006", "007")),
    "SetDefaultsError": ("set_defaults", "6.1.53", "330", ("001", "003", "005", "006", "007")),
}
REQUESTS = {
    "GetConstraintsRequest": (
        "get_constraints",
        "6.1.22",
        "229",
        {
            "alloc::borrow::ToOwned>::Owned",
            "alloc::borrow::ToOwned>::clone_into",
            "alloc::borrow::ToOwned>::to_owned",
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::get_constraints::GetConstraintsRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::get_constraints::GetConstraintsRequest>>::borrow_mut",
            "core::clone::Clone>::clone",
            "core::clone::Clone>::clone_from",
            "core::clone::CloneToUninit>::clone_to_uninit",
            "core::cmp::Eq>::assert_receiver_is_total_eq",
            "core::cmp::PartialEq>::eq",
            "core::cmp::PartialEq>::ne",
            "core::convert::From<kmipkit_protocol::get_constraints::GetConstraintsRequest>>::from",
            "core::default::Default>::default",
            "core::fmt::Debug>::fmt",
        },
    ),
    "GetUsageAllocationRequest": (
        "get_usage_allocation",
        "6.1.23",
        "232",
        {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::get_usage_allocation::GetUsageAllocationRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::get_usage_allocation::GetUsageAllocationRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::get_usage_allocation::GetUsageAllocationRequest>>::from",
        },
    ),
    "SetConstraintsRequest": (
        "set_constraints",
        "6.1.52",
        "325",
        {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::set_constraints::SetConstraintsRequest>>::from",
            "core::fmt::Debug>::fmt",
        },
    ),
    "SetDefaultsRequest": (
        "set_defaults",
        "6.1.53",
        "328",
        {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::from",
            "core::default::Default>::default",
            "core::fmt::Debug>::fmt",
        },
    ),
}


def paths(public_type: str, suffixes: set[str]) -> set[str]:
    return {f"<{public_type} as {suffix}" for suffix in suffixes}


class T102PolicyRootTraitBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_exact_root_trait_paths_are_excluded_and_sensitive_debug_stays_blocked(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        expected: dict[tuple[str, str], tuple[str, str, str, tuple[str, ...]]] = {}
        for name, (module, section, table, owner_suffixes) in ERRORS.items():
            public_type = f"kmipkit::{name}"
            implementation_type = f"kmipkit_protocol::{module}::{name}"
            direct = ERROR_COMMON_DIRECT | {
                f"core::borrow::Borrow<{implementation_type}>>::borrow",
                f"core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
                f"core::convert::From<{implementation_type}>>::from",
            }
            for export_name in paths(public_type, direct | CONDITIONAL):
                expected[("rust-kmipkit", export_name)] = (
                    "error",
                    section,
                    table,
                    tuple(f"KMIPKIT-0039-FR-{suffix}" for suffix in owner_suffixes),
                )

        for name, (module, section, table, direct) in REQUESTS.items():
            public_type = f"kmipkit::{name}"
            selected = paths(public_type, direct | CONDITIONAL)
            self.assertEqual(len(direct), {"GetConstraintsRequest": 15, "GetUsageAllocationRequest": 4, "SetConstraintsRequest": 5, "SetDefaultsRequest": 6}[name])
            expected.update(
                {
                    ("rust-kmipkit", export_name): (
                        "request",
                        section,
                        table,
                        ("KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-002", "KMIPKIT-0039-FR-004", "KMIPKIT-0039-FR-008"),
                    )
                    for export_name in selected
                }
            )

        for package, public_type in (
            ("rust-kmipkit", "kmipkit::UsageAllocationContextError"),
            ("rust-kmipkit-client", "kmipkit_client::UsageAllocationContextError"),
        ):
            implementation_type = "kmipkit_client::usage_allocation::UsageAllocationContextError"
            direct = ERROR_COMMON_DIRECT | {
                f"core::borrow::Borrow<{implementation_type}>>::borrow",
                f"core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
                f"core::convert::From<{implementation_type}>>::from",
            }
            expected.update(
                {
                    (package, export_name): (
                        "context-error",
                        "6.1.23",
                        "234",
                        ("KMIPKIT-0039-FR-005", "KMIPKIT-0039-FR-009"),
                    )
                    for export_name in paths(public_type, direct | CONDITIONAL)
                }
            )

        self.assertEqual(len(expected), 206)
        observed_direct = {
            (row["packageId"], row["exportName"])
            for row in self.report["declarations"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        observed_conditional = {
            (row["packageId"], row["exportName"])
            for row in self.report["conditionalDeclarations"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        self.assertEqual(len(observed_direct), 156)
        self.assertEqual(len(observed_conditional), 50)
        self.assertEqual(observed_direct | observed_conditional, set(expected))

        declarations = {
            (row.get("packageId"), row.get("exportName")): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row.get("packageId"), row.get("exportName")): row
            for row in self.manifest["approvedExclusions"]
        }
        self.assertEqual(set(exclusions).intersection(expected), set(expected))
        for key, (category, section, table, owner_suffixes) in expected.items():
            with self.subTest(package=key[0], export_name=key[1]):
                self.assertNotIn(key, declarations)
                row = exclusions[key]
                self.assertEqual(row.get("candidateSpecId"), "KMIPKIT-0039")
                self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("not a separate KMIPKit client capability", row["reason"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                for requirement_id in owner_suffixes:
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn(f"Section {section}", row["evidence"])
                self.assertIn(f"Table {table}", row["evidence"])
                self.assertIn("inventory-only", row["evidence"])
                if category == "error" or "core::fmt::Debug>::fmt" in key[1] or "core::fmt::Display>::fmt" in key[1]:
                    self.assertIn("KMIPKIT-0043-FR-013", row["evidence"])

        uid_debug = ("rust-kmipkit", "<kmipkit::GetUsageAllocationRequest as core::fmt::Debug>::fmt")
        self.assertIn(uid_debug, declarations)
        self.assertEqual(declarations[uid_debug].get("candidateSpecId"), "KMIPKIT-0043")
        self.assertNotIn(uid_debug, exclusions)

        for package, public_type in (
            ("rust-kmipkit", "kmipkit::GetConstraintsError"),
            ("rust-kmipkit", "kmipkit::GetUsageAllocationError"),
            ("rust-kmipkit", "kmipkit::SetConstraintsError"),
            ("rust-kmipkit", "kmipkit::SetDefaultsError"),
            ("rust-kmipkit", "kmipkit::UsageAllocationContextError"),
            ("rust-kmipkit-client", "kmipkit_client::UsageAllocationContextError"),
        ):
            parent = declarations[(package, public_type)]
            self.assertEqual(parent.get("declarationState"), "blocked")
            self.assertEqual(parent.get("candidateSpecId"), "KMIPKIT-0043")

        for public_type in (
            "kmipkit::GetConstraintsRequest",
            "kmipkit::GetUsageAllocationRequest",
            "kmipkit::SetConstraintsRequest",
            "kmipkit::SetDefaultsRequest",
        ):
            parent = declarations[("rust-kmipkit", public_type)]
            self.assertEqual(parent.get("declarationState"), "verified")
            self.assertNotIn("candidateSpecId", parent)


if __name__ == "__main__":
    unittest.main()
