"""Pin policy-control protocol error traits and blocked root aliases for T102."""

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
PROTOCOL_PACKAGE = "rust-kmipkit-protocol"
ROOT_PACKAGE = "rust-kmipkit"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/025-policy-controls/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation \u2014 2026-10-11",
}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0039-FR-001",
    "KMIPKIT-0039-FR-003",
    "KMIPKIT-0039-FR-004",
    "KMIPKIT-0039-FR-006",
    "KMIPKIT-0039-FR-007",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")

# Exact common direct Rust paths. The concrete Borrow/BorrowMut/From targets
# are pinned separately for each module-qualified protocol error type.
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
MODULES = {
    "GetConstraintsError": "get_constraints",
    "GetUsageAllocationError": "get_usage_allocation",
    "SetConstraintsError": "set_constraints",
    "SetDefaultsError": "set_defaults",
}
OPERATIONS = {
    "GetConstraintsError": {
        "element": "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS",
        "reference": "OASIS KMIP v2.1 Section 6.1.22, Table 231",
        "caption": "Table 231: Get Constraints Errors",
        "method": "get_constraints",
        "request": "GetConstraintsRequest",
        "response": "GetConstraintsResponse",
    },
    "GetUsageAllocationError": {
        "element": "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
        "reference": "OASIS KMIP v2.1 Section 6.1.23, Table 234",
        "caption": "Table 234: Get Usage Allocation Errors",
        "method": "get_usage_allocation",
        "request": "GetUsageAllocationRequest",
        "response": "GetUsageAllocationResponse",
    },
    "SetConstraintsError": {
        "element": "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS",
        "reference": "OASIS KMIP v2.1 Section 6.1.52, Table 327",
        "caption": "Table 327: Set Constraints Errors",
        "method": "set_constraints",
        "request": "SetConstraintsRequest",
        "response": "SetConstraintsResponse",
    },
    "SetDefaultsError": {
        "element": "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS",
        "reference": "OASIS KMIP v2.1 Section 6.1.53, Table 330",
        "caption": "Table 330: Set Defaults Errors",
        "method": "set_defaults",
        "request": "SetDefaultsRequest",
        "response": "SetDefaultsResponse",
    },
}
FUNCTIONAL_STATES = {
    "GetConstraintsError": {
        "": "blocked",
        "::InvalidOperationResult": "blocked",
        "::InvalidOperationResult::0": "blocked",
        "::MalformedResponseItem": "blocked",
        "::MalformedSuccessPayload": "blocked",
        "::MissingResultStatus": "blocked",
        "::MissingSuccessPayload": "blocked",
        "::TtlvModel": "blocked",
        "::TtlvModel::0": "blocked",
        "::UnexpectedOperation": "blocked",
    },
    "GetUsageAllocationError": {
        "": "blocked",
        "::InvalidOperationResult": "blocked",
        "::InvalidOperationResult::0": "blocked",
        "::MalformedResponseItem": "blocked",
        "::MalformedSuccessPayload": "blocked",
        "::MissingResultStatus": "blocked",
        "::MissingSuccessPayload": "blocked",
        "::UnexpectedOperation": "blocked",
    },
    "SetConstraintsError": {
        "": "blocked",
        "::InvalidOperationResult": "blocked",
        "::InvalidOperationResult::0": "blocked",
        "::MalformedResponseItem": "blocked",
        "::MissingResultStatus": "blocked",
        "::MissingSuccessPayload": "blocked",
        "::UnexpectedOperation": "blocked",
    },
    "SetDefaultsError": {
        "": "blocked",
        "::InvalidOperationResult": "blocked",
        "::InvalidOperationResult::0": "blocked",
        "::MalformedResponseItem": "blocked",
        "::MissingResultStatus": "blocked",
        "::MissingSuccessPayload": "blocked",
        "::UnexpectedOperation": "blocked",
    },
}


def trait_paths(type_name: str, suffixes: set[str], *, root: bool = False) -> set[str]:
    facade = "kmipkit" if root else "kmipkit_protocol"
    return {f"<{facade}::{type_name} as {suffix}" for suffix in suffixes}


def direct_suffixes(type_name: str) -> set[str]:
    module = MODULES[type_name]
    self_type = f"kmipkit_protocol::{module}::{type_name}"
    return COMMON_DIRECT | {
        f"core::borrow::Borrow<{self_type}>>::borrow",
        f"core::borrow::BorrowMut<{self_type}>>::borrow_mut",
        f"core::convert::From<{self_type}>>::from",
    }


class T102PolicyControlErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_error_trait_paths_are_owner_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        all_pairs: set[tuple[str, str]] = set()

        for type_name, operation in OPERATIONS.items():
            expected_direct = trait_paths(type_name, direct_suffixes(type_name))
            expected_conditional = trait_paths(type_name, CONDITIONAL)
            expected = expected_direct | expected_conditional
            prefix = f"<kmipkit_protocol::{type_name} as "
            observed_direct = {
                row["exportName"]
                for row in self.report["declarations"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_conditional = {
                row["exportName"]
                for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_excluded = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(observed_direct, expected_direct)
            self.assertEqual(observed_conditional, expected_conditional)
            self.assertEqual(observed_excluded, expected)
            self.assertEqual(len(expected_direct), 21)
            self.assertEqual(len(expected_conditional), 5)

            declarations = {
                row["exportName"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(declarations, set())
            self.assertEqual(set(exclusions), expected)
            self.assertEqual(len(exclusions), 26)

            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    for requirement_id in (
                        *OWNER_REQUIREMENTS,
                        operation["element"],
                        *T102_REQUIREMENTS,
                    ):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(operation["reference"], row["evidence"])
                    self.assertIn(operation["caption"], row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(type_name, row["reason"])
            all_pairs.update((PROTOCOL_PACKAGE, name) for name in expected)

        self.assertEqual(len(all_pairs), 104)

    def test_protocol_and_root_error_declarations_and_alias_traits_remain_blocked(self) -> None:
        for type_name, suffix_states in FUNCTIONAL_STATES.items():
            protocol_name = f"kmipkit_protocol::{type_name}"
            expected_protocol = {
                f"{protocol_name}{suffix}": state for suffix, state in suffix_states.items()
            }
            actual_protocol = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PROTOCOL_PACKAGE
                and (
                    row.get("exportName") == protocol_name
                    or row.get("exportName", "").startswith(protocol_name + "::")
                )
                and not row.get("exportName", "").startswith("<")
            }
            self.assertEqual(actual_protocol, expected_protocol)

            expected_root = {
                f"kmipkit::{type_name}{suffix}": state
                for suffix, state in suffix_states.items()
            }
            root_trait_paths = trait_paths(
                type_name,
                direct_suffixes(type_name) | CONDITIONAL,
                root=True,
            )
            expected_root.update({name: "blocked" for name in root_trait_paths})
            actual_root = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == ROOT_PACKAGE
                and (
                    row.get("exportName") == f"kmipkit::{type_name}"
                    or row.get("exportName", "").startswith(f"kmipkit::{type_name}::")
                    or row.get("exportName", "").startswith(f"<kmipkit::{type_name} as ")
                )
            }
            self.assertEqual(actual_root, expected_root)
            self.assertEqual(set(root_trait_paths) & {
                row.get("exportName")
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == ROOT_PACKAGE
            }, set())

    def test_related_request_response_and_operation_rows_retain_their_states(self) -> None:
        for operation in OPERATIONS.values():
            for type_name in (operation["request"], operation["response"]):
                protocol = f"kmipkit_protocol::{type_name}"
                root = f"kmipkit::{type_name}"
                protocol_rows = [
                    row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == PROTOCOL_PACKAGE
                    and row.get("exportName") == protocol
                ]
                root_rows = [
                    row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == ROOT_PACKAGE
                    and row.get("exportName") == root
                ]
                self.assertEqual(len(protocol_rows), 1)
                self.assertEqual(protocol_rows[0]["declarationState"], "verified")
                self.assertEqual(len(root_rows), 1)
                self.assertEqual(root_rows[0]["declarationState"], "blocked")

            method = operation["method"]
            expected = {
                (ROOT_PACKAGE, f"kmipkit::Client::{method}"): "verified",
                (ROOT_PACKAGE, f"kmipkit::ClientRequest::{method}"): "verified",
                ("rust-kmipkit-client", f"kmipkit_client::Client::{method}"): "verified",
                ("rust-kmipkit-client", f"kmipkit_client::ClientRequest::{method}"): "verified",
                (
                    "rust-kmipkit-client",
                    f"kmipkit_client::ClientResponseView<'a>::{method}",
                ): "blocked",
            }
            actual = {
                (row.get("packageId"), row.get("exportName")): row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if (row.get("packageId"), row.get("exportName")) in expected
            }
            self.assertEqual(actual, expected)

    def test_manifest_and_t102_candidate_totals_decrease_by_104_protocol_paths(self) -> None:
        count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(count, 3026)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 8224)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 6500)


if __name__ == "__main__":
    unittest.main()
