"""Pin the exact T102 Get and Locate error trait-artifact exclusions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
PACKAGE = "rust-kmipkit-protocol"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/017-managed-object-retrieval/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0017-FR-001",
    "KMIPKIT-0017-FR-011",
    "KMIPKIT-0017-FR-012",
    "KMIPKIT-0017-FR-013",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")

COMMON_DIRECT = {
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
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
OPERATIONS = {
    "GetError": {
        "module": "get",
        "section": "\u00a76.1.19.1",
        "caption": "Table 222: Get Errors",
    },
    "LocateError": {
        "module": "locate",
        "section": "\u00a76.1.28.1",
        "caption": "Table 249: Locate Errors",
    },
}
ERROR_VARIANTS = (
    "UnexpectedOperation",
    "MissingResultStatus",
    "InvalidOperationResult",
    "InvalidOperationResult::0",
    "MissingSuccessPayload",
    "MalformedSuccessPayload",
)
RESPONSE_STATES = {
    "GetResponse": {
        "": "verified",
        "::object": "verified",
        "::object_type": "verified",
        "::result": "blocked",
        "::try_from_response_item": "blocked",
        "::unique_identifier": "verified",
    },
    "LocateResponse": {
        "": "verified",
        "::located_items": "verified",
        "::result": "blocked",
        "::try_from_response_item": "blocked",
        "::unique_identifiers": "verified",
    },
}


def exact_paths(public_type: str, suffixes: set[str]) -> set[str]:
    return {f"<{public_type} as {suffix}" for suffix in suffixes}


def direct_suffixes(type_name: str, module: str) -> set[str]:
    implementation_type = f"kmipkit_protocol::{module}::{type_name}"
    return COMMON_DIRECT | {
        f"core::borrow::Borrow<{implementation_type}>>::borrow",
        f"core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
        f"core::convert::From<{implementation_type}>>::from",
    }


class T102GetLocateErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_error_trait_paths_are_compiler_observed_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        self.assertEqual(self.report["missingFromManifest"], [])
        self.assertEqual(self.report["notObserved"], [])
        self.assertEqual(self.report["kindMismatches"], [])
        self.assertEqual(self.report["conditionalConditionMismatches"], [])
        self.assertEqual(self.report["conditionalStateMismatches"], [])
        total_paths: set[str] = set()

        for type_name, operation in OPERATIONS.items():
            public_type = f"kmipkit_protocol::{type_name}"
            prefix = f"<{public_type} as "
            expected_direct = exact_paths(
                public_type, direct_suffixes(type_name, operation["module"])
            )
            expected_conditional = exact_paths(public_type, CONDITIONAL)
            expected = expected_direct | expected_conditional
            observed_direct = {
                row["exportName"]
                for row in self.report["declarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_conditional = {
                row["exportName"]
                for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_excluded = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(observed_direct, expected_direct)
            self.assertEqual(observed_conditional, expected_conditional)
            self.assertEqual(observed_excluded, expected)
            self.assertEqual(len(expected_direct), 21)
            self.assertEqual(len(expected_conditional), 5)

            declarations = {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(declarations, {})
            self.assertEqual(set(exclusions), expected)
            self.assertEqual(len(exclusions), 26)

            expected_evidence = (
                "KMIPKIT-0043-FR-010/021; "
                "KMIPKIT-0017-FR-001/011/012/013; "
                f"OASIS KMIP Specification v2.1 {operation['section']}, "
                f"{operation['caption']}; payload-free conversion error."
            )
            self.assertLessEqual(len(expected_evidence.encode("utf-8")), 250)
            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    self.assertEqual(row["evidence"], expected_evidence)
                    self.assertIn("compiler-observed Rust standard-trait path", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(type_name, row["reason"])
            total_paths.update(expected)

        self.assertEqual(len(total_paths), 52)

    def test_error_variants_and_related_response_rows_keep_their_states(self) -> None:
        for type_name in OPERATIONS:
            protocol_name = f"kmipkit_protocol::{type_name}"
            expected_errors = {
                protocol_name: "blocked",
                **{
                    f"{protocol_name}::{suffix}": "blocked"
                    for suffix in ERROR_VARIANTS
                },
            }
            actual_errors = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and (
                    row.get("exportName") == protocol_name
                    or row.get("exportName", "").startswith(protocol_name + "::")
                )
                and not row.get("exportName", "").startswith("<")
            }
            self.assertEqual(actual_errors, expected_errors)

            root_name = f"kmipkit::{type_name}"
            root_rows = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and (
                    row.get("exportName") == root_name
                    or row.get("exportName", "").startswith(root_name + "::")
                    or row.get("exportName", "").startswith(f"<{root_name} as ")
                )
            ]
            self.assertEqual(root_rows, [])

        for response_name, states in RESPONSE_STATES.items():
            expected = {
                f"kmipkit_protocol::{response_name}{suffix}": state
                for suffix, state in states.items()
            }
            actual = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and (
                    row.get("exportName") == f"kmipkit_protocol::{response_name}"
                    or row.get("exportName", "").startswith(
                        f"kmipkit_protocol::{response_name}::"
                    )
                )
            }
            self.assertEqual(actual, expected)

    def test_manifest_and_candidate_totals_decrease_by_52(self) -> None:
        candidate_count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidate_count, 1098)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6296)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 8428)


if __name__ == "__main__":
    unittest.main()
