"""Pin the exact T102 Locate value-trait inventory exclusions."""

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
PACKAGE = "rust-kmipkit-protocol"
API_SPEC = "specs/043-public-api-bindings/spec.md"
OWNER_SPEC = "specs/017-managed-object-retrieval/spec.md"
APPROVAL = {
    "kind": "feature-specification",
    "path": API_SPEC,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
CONDITIONAL_SUFFIXES = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
TYPES = {
    "ObjectGroupMember": "kmipkit_protocol::locate::ObjectGroupMember",
    "StorageStatusMask": "kmipkit_protocol::locate::StorageStatusMask",
}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0017-FR-001",
    "KMIPKIT-0017-FR-006",
    "KMIPKIT-0017-FR-009",
)


def expected_direct(public_type: str, implementation: str) -> set[str]:
    prefix = f"<{public_type} as "
    suffixes = {
        "alloc::borrow::ToOwned>::Owned",
        "alloc::borrow::ToOwned>::clone_into",
        "alloc::borrow::ToOwned>::to_owned",
        "core::any::Any>::type_id",
        f"core::borrow::Borrow<{implementation}>>::borrow",
        f"core::borrow::BorrowMut<{implementation}>>::borrow_mut",
        "core::clone::Clone>::clone",
        "core::clone::Clone>::clone_from",
        "core::clone::CloneToUninit>::clone_to_uninit",
        "core::cmp::Eq>::assert_receiver_is_total_eq",
        "core::cmp::PartialEq>::eq",
        "core::cmp::PartialEq>::ne",
        f"core::convert::From<{implementation}>>::from",
        "core::fmt::Debug>::fmt",
        "core::hash::Hash>::hash",
        "core::hash::Hash>::hash_slice",
    }
    return {f"{prefix}{suffix}" for suffix in suffixes}


def expected_conditional(public_type: str) -> set[str]:
    return {f"<{public_type} as {suffix}" for suffix in CONDITIONAL_SUFFIXES}


class T102LocateValueTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_exact_locate_value_trait_paths_are_owner_backed_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        expected_all: set[str] = set()
        for name, implementation in TYPES.items():
            public_type = f"kmipkit_protocol::{name}"
            expected_direct_paths = expected_direct(public_type, implementation)
            expected_conditional_paths = expected_conditional(public_type)
            expected_paths = expected_direct_paths | expected_conditional_paths
            expected_all.update(expected_paths)
            prefix = f"<{public_type} as "

            direct = {
                row["exportName"]
                for row in self.report["declarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            conditional = {
                row["exportName"]
                for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            audited_exclusions = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            with self.subTest(type_name=name, evidence="pinned Rust report"):
                self.assertEqual(direct, expected_direct_paths)
                self.assertEqual(conditional, expected_conditional_paths)
                self.assertEqual(len(direct), 16)
                self.assertEqual(len(conditional), 5)
                self.assertEqual(audited_exclusions, expected_paths)

            declarations = {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName") in expected_paths
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName") in expected_paths
            }
            with self.subTest(type_name=name, evidence="manifest exact set"):
                self.assertEqual(set(declarations), set())
                self.assertEqual(set(exclusions), expected_paths)
                self.assertEqual(len(exclusions), 21)

            for row in exclusions.values():
                with self.subTest(type_name=name, export_name=row["exportName"]):
                    self.assertEqual(row["approvalSource"], APPROVAL)
                    for requirement_id in (
                        *OWNER_REQUIREMENTS,
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    ):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(OWNER_SPEC, row["evidence"])
                    self.assertIn("§11.33, Table 467", row["evidence"])
                    self.assertIn("§12.3, Table 498", row["evidence"])
                    self.assertIn("§6.1.28, Table 247", row["evidence"])
                    self.assertIn("no OASIS clause applies directly to compiler-generated traits", row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])

        self.assertEqual(len(expected_all), 42)
        total_candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(total_candidates, 1_264)

    def test_locate_value_parents_and_raw_accessors_keep_their_states(self) -> None:
        rows = {
            row["exportName"]: row["declarationState"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == PACKAGE
            and any(
                row.get("exportName", "").startswith(f"kmipkit_protocol::{name}")
                for name in TYPES
            )
            and not row["exportName"].startswith("<")
        }
        expected = {
            f"kmipkit_protocol::{name}": "blocked"
            for name in TYPES
        }
        for name in TYPES:
            expected[f"kmipkit_protocol::{name}::from_raw"] = "verified"
            expected[f"kmipkit_protocol::{name}::raw"] = "verified"
        self.assertEqual(rows, expected)


if __name__ == "__main__":
    unittest.main()
