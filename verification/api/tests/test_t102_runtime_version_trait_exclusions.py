"""Exact T102 exclusions for compiler-visible RuntimeVersion trait artifacts."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = REPOSITORY_ROOT / "specification" / "api" / "public-api.json"
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
RUNTIME_VERSION_TRAIT_PREFIX = "<kmipkit::RuntimeVersion as "
EXPECTED_TRAIT_EXPORTS = {
    "<kmipkit::RuntimeVersion as alloc::borrow::ToOwned>::Owned",
    "<kmipkit::RuntimeVersion as alloc::borrow::ToOwned>::clone_into",
    "<kmipkit::RuntimeVersion as alloc::borrow::ToOwned>::to_owned",
    "<kmipkit::RuntimeVersion as core::any::Any>::type_id",
    "<kmipkit::RuntimeVersion as core::borrow::Borrow<kmipkit::RuntimeVersion>>::borrow",
    "<kmipkit::RuntimeVersion as core::borrow::BorrowMut<kmipkit::RuntimeVersion>>::borrow_mut",
    "<kmipkit::RuntimeVersion as core::clone::Clone>::clone",
    "<kmipkit::RuntimeVersion as core::clone::Clone>::clone_from",
    "<kmipkit::RuntimeVersion as core::clone::CloneToUninit>::clone_to_uninit",
    "<kmipkit::RuntimeVersion as core::cmp::Eq>::assert_receiver_is_total_eq",
    "<kmipkit::RuntimeVersion as core::cmp::PartialEq>::eq",
    "<kmipkit::RuntimeVersion as core::cmp::PartialEq>::ne",
    "<kmipkit::RuntimeVersion as core::convert::From<kmipkit::RuntimeVersion>>::from",
    "<kmipkit::RuntimeVersion as core::convert::Into<U>>::into",
    "<kmipkit::RuntimeVersion as core::convert::TryFrom<U>>::Error",
    "<kmipkit::RuntimeVersion as core::convert::TryFrom<U>>::try_from",
    "<kmipkit::RuntimeVersion as core::convert::TryInto<U>>::Error",
    "<kmipkit::RuntimeVersion as core::convert::TryInto<U>>::try_into",
    "<kmipkit::RuntimeVersion as core::fmt::Debug>::fmt",
}


class T102RuntimeVersionTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_all_runtime_version_trait_paths_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(RUNTIME_VERSION_TRAIT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(RUNTIME_VERSION_TRAIT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_TRAIT_EXPORTS)
        self.assertEqual(len(exclusions), 19)

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("KMIPKIT-0043-FR-007", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("runtime version", row["reason"].lower())


if __name__ == "__main__":
    unittest.main()
