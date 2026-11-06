"""Exact T102 exclusions for local runtime-metadata error trait artifacts."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = REPOSITORY_ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT_PATH = (
    REPOSITORY_ROOT
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
TRAIT_PREFIX = "<kmipkit::RuntimeMetadataError as "
EXPECTED_DIRECT_TRAIT_EXPORTS = {
    "<kmipkit::RuntimeMetadataError as alloc::borrow::ToOwned>::Owned",
    "<kmipkit::RuntimeMetadataError as alloc::borrow::ToOwned>::clone_into",
    "<kmipkit::RuntimeMetadataError as alloc::borrow::ToOwned>::to_owned",
    "<kmipkit::RuntimeMetadataError as alloc::string::ToString>::to_string",
    "<kmipkit::RuntimeMetadataError as core::any::Any>::type_id",
    "<kmipkit::RuntimeMetadataError as core::borrow::Borrow<kmipkit::RuntimeMetadataError>>::borrow",
    "<kmipkit::RuntimeMetadataError as core::borrow::BorrowMut<kmipkit::RuntimeMetadataError>>::borrow_mut",
    "<kmipkit::RuntimeMetadataError as core::clone::Clone>::clone",
    "<kmipkit::RuntimeMetadataError as core::clone::Clone>::clone_from",
    "<kmipkit::RuntimeMetadataError as core::clone::CloneToUninit>::clone_to_uninit",
    "<kmipkit::RuntimeMetadataError as core::cmp::Eq>::assert_receiver_is_total_eq",
    "<kmipkit::RuntimeMetadataError as core::cmp::PartialEq>::eq",
    "<kmipkit::RuntimeMetadataError as core::cmp::PartialEq>::ne",
    "<kmipkit::RuntimeMetadataError as core::convert::From<kmipkit::RuntimeMetadataError>>::from",
    "<kmipkit::RuntimeMetadataError as core::error::Error>::cause",
    "<kmipkit::RuntimeMetadataError as core::error::Error>::description",
    "<kmipkit::RuntimeMetadataError as core::error::Error>::provide",
    "<kmipkit::RuntimeMetadataError as core::error::Error>::source",
    "<kmipkit::RuntimeMetadataError as core::error::Error>::type_id",
    "<kmipkit::RuntimeMetadataError as core::fmt::Debug>::fmt",
    "<kmipkit::RuntimeMetadataError as core::fmt::Display>::fmt",
}
EXPECTED_CONDITIONAL_TRAIT_EXPORTS = {
    "<kmipkit::RuntimeMetadataError as core::convert::Into<U>>::into",
    "<kmipkit::RuntimeMetadataError as core::convert::TryFrom<U>>::Error",
    "<kmipkit::RuntimeMetadataError as core::convert::TryFrom<U>>::try_from",
    "<kmipkit::RuntimeMetadataError as core::convert::TryInto<U>>::Error",
    "<kmipkit::RuntimeMetadataError as core::convert::TryInto<U>>::try_into",
}
EXPECTED_TRAIT_EXPORTS = EXPECTED_DIRECT_TRAIT_EXPORTS | EXPECTED_CONDITIONAL_TRAIT_EXPORTS


class T102RuntimeMetadataErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))

    def test_only_exact_compiler_trait_paths_are_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }

        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), EXPECTED_TRAIT_EXPORTS)
        self.assertEqual(len(EXPECTED_DIRECT_TRAIT_EXPORTS), 21)
        self.assertEqual(len(EXPECTED_CONDITIONAL_TRAIT_EXPORTS), 5)

        reported_direct = {
            row["exportName"]
            for row in self.rust_report["declarations"]
            if row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        reported_conditional = {
            row["exportName"]
            for row in self.rust_report["conditionalDeclarations"]
            if row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        self.assertEqual(reported_direct, EXPECTED_DIRECT_TRAIT_EXPORTS)
        self.assertEqual(reported_conditional, EXPECTED_CONDITIONAL_TRAIT_EXPORTS)

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("standard-trait implementation", row["reason"])
                self.assertIn("local runtime metadata error", row["reason"])
                self.assertIn("KMIPKIT-0043-FR-007", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-013", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertIn("rust-export-report-2026-11-01.json", row["evidence"])
                self.assertIn("21 direct and five conditional", row["evidence"])
                self.assertIn("No OASIS clause applies", row["evidence"])

    def test_runtime_error_type_and_resource_limit_variant_remain_mapped(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName")
            in {
                "kmipkit::RuntimeMetadataError",
                "kmipkit::RuntimeMetadataError::ResourceLimit",
            }
        }
        self.assertEqual(
            set(declarations),
            {
                "kmipkit::RuntimeMetadataError",
                "kmipkit::RuntimeMetadataError::ResourceLimit",
            },
        )
        for export_name, row in declarations.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["declarationState"], "declared")
                self.assertEqual(row["capabilityId"], "ffi_runtime_metadata")
                self.assertEqual(
                    set(row["requirementIds"]),
                    {"KMIPKIT-0043-FR-007", "KMIPKIT-0043-FR-013"},
                )
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)

        limit_error = declarations["kmipkit::RuntimeMetadataError::ResourceLimit"]
        self.assertEqual(
            {
                language: mapping["name"]
                for language, mapping in limit_error["languageMappings"].items()
            },
            {
                "rust": "kmipkit::RuntimeMetadataError::ResourceLimit",
                "c": "KMIPKIT_ERROR_RESOURCE_LIMIT",
                "java": "org.kmipkit.ResourceLimitException",
                "python": "kmipkit.errors.ResourceLimitError",
            },
        )


if __name__ == "__main__":
    unittest.main()
