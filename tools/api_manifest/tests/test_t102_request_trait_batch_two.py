"""Pin exact T102 exclusions for approved Validate and policy request traits."""

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

CONDITIONAL_TRAIT_SUFFIXES = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}

TYPES = {
    "ValidateError": {
        "owner_spec": "specs/028-diagnostics-interop/spec.md",
        "owner_spec_id": "KMIPKIT-0042",
        "owner_requirement_ids": ("KMIPKIT-0042-FR-001", "KMIPKIT-0042-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.57 Tables 340-342",
        "related_tables": "Tables 341-342 as Validate response and error tables",
        "direct_suffixes": {
            "alloc::borrow::ToOwned>::Owned",
            "alloc::borrow::ToOwned>::clone_into",
            "alloc::borrow::ToOwned>::to_owned",
            "alloc::string::ToString>::to_string",
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::validate::ValidateError>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::validate::ValidateError>>::borrow_mut",
            "core::clone::Clone>::clone",
            "core::clone::Clone>::clone_from",
            "core::clone::CloneToUninit>::clone_to_uninit",
            "core::cmp::Eq>::assert_receiver_is_total_eq",
            "core::cmp::PartialEq>::eq",
            "core::cmp::PartialEq>::ne",
            "core::convert::From<kmipkit_protocol::validate::ValidateError>>::from",
            "core::error::Error>::cause",
            "core::error::Error>::description",
            "core::error::Error>::provide",
            "core::error::Error>::source",
            "core::error::Error>::type_id",
            "core::fmt::Debug>::fmt",
            "core::fmt::Display>::fmt",
        },
        "protocol_states": {
            "": "verified",
            "::InvalidOperationResult": "verified",
            "::InvalidOperationResult::0": "verified",
            "::MalformedResponseItem": "verified",
            "::MalformedResponsePayload": "verified",
            "::MissingResponsePayload": "verified",
            "::MissingResultStatus": "verified",
            "::UnexpectedOperation": "verified",
        },
        "root_states": {},
    },
    "ValidateRequest": {
        "owner_spec": "specs/028-diagnostics-interop/spec.md",
        "owner_spec_id": "KMIPKIT-0042",
        "owner_requirement_ids": ("KMIPKIT-0042-FR-001", "KMIPKIT-0042-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.57 Tables 340-342",
        "related_tables": "Table 340 as request",
        "direct_suffixes": {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::validate::ValidateRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::validate::ValidateRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::validate::ValidateRequest>>::from",
            "core::default::Default>::default",
            "core::fmt::Debug>::fmt",
        },
        "protocol_states": {
            "": "verified",
            "::chain_len": "verified",
            "::into_ttlv_payload": "verified",
            "::new": "verified",
            "::validity_date": "verified",
            "::with_certificate": "verified",
            "::with_extension": "verified",
            "::with_unique_identifier": "verified",
            "::with_validity_date": "verified",
        },
        "root_states": {},
    },
    "GetConstraintsRequest": {
        "owner_spec": "specs/025-policy-controls/spec.md",
        "owner_spec_id": "KMIPKIT-0039",
        "owner_requirement_ids": ("KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.22 Tables 229-231",
        "related_tables": "Table 229 as request",
        "direct_suffixes": {
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
        "protocol_states": {
            "": "verified",
            "::new": "verified",
            "::to_ttlv_payload": "blocked",
        },
        "root_states": {
            "": "blocked",
            "::new": "blocked",
            "::to_ttlv_payload": "blocked",
        },
    },
    "SetDefaultsRequest": {
        "owner_spec": "specs/025-policy-controls/spec.md",
        "owner_spec_id": "KMIPKIT-0039",
        "owner_requirement_ids": ("KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.53 Tables 328-330",
        "related_tables": "Table 328 as request",
        "direct_suffixes": {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::set_defaults::SetDefaultsRequest>>::from",
            "core::default::Default>::default",
            "core::fmt::Debug>::fmt",
        },
        "protocol_states": {
            "": "verified",
            "::defaults_information": "verified",
            "::new": "verified",
            "::to_ttlv_payload": "blocked",
            "::with_defaults_information": "verified",
        },
        "root_states": {
            "": "blocked",
            "::defaults_information": "blocked",
            "::new": "blocked",
            "::to_ttlv_payload": "blocked",
            "::with_defaults_information": "blocked",
        },
    },
}


def trait_paths(public_type: str, suffixes: set[str]) -> set[str]:
    return {f"<{public_type} as {suffix}" for suffix in suffixes}


class T102RequestTraitBatchTwoTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_exact_compiler_observed_traits_are_excluded_under_approved_owners(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(self.rust_report["issues"], [])

        all_expected_pairs: set[tuple[str, str]] = set()
        for type_name, details in TYPES.items():
            public_type = f"kmipkit_protocol::{type_name}"
            direct_paths = trait_paths(public_type, details["direct_suffixes"])
            conditional_paths = trait_paths(public_type, CONDITIONAL_TRAIT_SUFFIXES)
            expected_paths = direct_paths | conditional_paths
            expected_pairs = {
                ("rust-kmipkit-protocol", export_name)
                for export_name in expected_paths
            }
            all_expected_pairs.update(expected_pairs)
            prefix = f"<{public_type} as "

            observed_direct = {
                row["exportName"]
                for row in self.rust_report["declarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            }
            observed_conditional = {
                row["exportName"]
                for row in self.rust_report["conditionalDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            }
            observed_excluded = {
                (row["packageId"], row["exportName"])
                for row in self.rust_report["excluded"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            }
            with self.subTest(type_name=type_name, evidence="pinned Rust audit"):
                self.assertEqual(observed_direct, direct_paths)
                self.assertEqual(observed_conditional, conditional_paths)
                self.assertEqual(observed_excluded, expected_pairs)

            declarations = {
                (row["packageId"], row["exportName"])
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                (row["packageId"], row["exportName"]): row
                for row in self.manifest.get("approvedExclusions", [])
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(prefix)
            }
            with self.subTest(type_name=type_name, evidence="manifest exact set"):
                self.assertEqual(declarations, set())
                self.assertEqual(set(exclusions), expected_pairs)
                self.assertEqual(len(exclusions), len(expected_paths))

            for row in exclusions.values():
                with self.subTest(type_name=type_name, export_name=row["exportName"]):
                    self.assertEqual(
                        row["approvalSource"],
                        {
                            "kind": "feature-specification",
                            "path": details["owner_spec"],
                            "status": "Approved",
                            "evidence": "**Status**: Approved for implementation",
                        },
                    )
                    for requirement_id in (
                        *details["owner_requirement_ids"],
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    ):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(details["section_tables"], row["evidence"])
                    self.assertIn(details["related_tables"], row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("no OASIS conformance", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])

        self.assertEqual(len(all_expected_pairs), 68)
        manifest_target_exclusions = {
            (row["packageId"], row["exportName"])
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and any(
                row.get("exportName", "").startswith(
                    f"<kmipkit_protocol::{type_name} as "
                )
                for type_name in TYPES
            )
        }
        self.assertEqual(manifest_target_exclusions, all_expected_pairs)

    def test_protocol_and_root_functional_declarations_keep_their_states(self) -> None:
        for type_name, details in TYPES.items():
            protocol_type = f"kmipkit_protocol::{type_name}"
            expected_protocol = {
                f"{protocol_type}{suffix}": state
                for suffix, state in details["protocol_states"].items()
            }
            protocol_states = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(protocol_type)
                and not row["exportName"].startswith("<")
            }
            root_type = f"kmipkit::{type_name}"
            expected_root = {
                f"{root_type}{suffix}": state
                for suffix, state in details["root_states"].items()
            }
            root_states = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and row.get("exportName", "").startswith(root_type)
            }
            with self.subTest(type_name=type_name):
                self.assertEqual(protocol_states, expected_protocol)
                self.assertEqual(root_states, expected_root)

    def test_candidate_count_moves_by_only_the_exact_sixty_eight_paths(self) -> None:
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            1264,
        )


if __name__ == "__main__":
    unittest.main()
