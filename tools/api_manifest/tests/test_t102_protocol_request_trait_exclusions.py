"""Pin the T102 standard-trait exclusions for four approved protocol types."""

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
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)

TYPES = {
    "DelegatedLoginRequest": {
        "module": "delegated_login",
        "owner_spec": "specs/024-credential-sessions/spec.md",
        "owner_spec_id": "KMIPKIT-0038",
        "owner_requirement_ids": ("KMIPKIT-0038-FR-001", "KMIPKIT-0038-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.12 Tables 199-201",
        "request_table": "Table 199 as request",
        "protocol_functional_states": {
            "": "verified",
            "::into_ttlv_payload": "verified",
            "::new": "verified",
            "::with_lease_time_date_time": "verified",
            "::with_lease_time_interval": "verified",
            "::with_request_count": "verified",
            "::with_usage_limits": "verified",
        },
        "root_functional_states": {
            "": "verified",
            "::into_ttlv_payload": "verified",
            "::new": "verified",
            "::with_lease_time_date_time": "verified",
            "::with_lease_time_interval": "verified",
            "::with_request_count": "verified",
            "::with_usage_limits": "verified",
        },
    },
    "LogoutRequest": {
        "module": "logout",
        "owner_spec": "specs/024-credential-sessions/spec.md",
        "owner_spec_id": "KMIPKIT-0038",
        "owner_requirement_ids": ("KMIPKIT-0038-FR-001", "KMIPKIT-0038-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.31 Tables 256-258",
        "request_table": "Table 256 as request",
        "protocol_functional_states": {
            "": "verified",
            "::into_ttlv_payload": "verified",
            "::new": "verified",
        },
        "root_functional_states": {
            "": "verified",
            "::into_ttlv_payload": "verified",
            "::new": "verified",
        },
    },
    "GetUsageAllocationRequest": {
        "module": "get_usage_allocation",
        "owner_spec": "specs/025-policy-controls/spec.md",
        "owner_spec_id": "KMIPKIT-0039",
        "owner_requirement_ids": ("KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.23 Tables 232-234",
        "request_table": "Table 232 as request",
        "protocol_functional_states": {
            "": "verified",
            "::new": "verified",
            "::to_ttlv_payload": "verified",
            "::unique_identifier": "verified",
            "::usage_limits_count": "verified",
            "::with_unique_identifier": "verified",
        },
        "root_functional_states": {
            "": "verified",
            "::new": "verified",
            "::to_ttlv_payload": "verified",
            "::unique_identifier": "verified",
            "::usage_limits_count": "verified",
            "::with_unique_identifier": "verified",
        },
    },
    "SplitKeyReference": {
        "module": "join_split_key",
        "owner_spec": "specs/023-key-material-transfer/spec.md",
        "owner_spec_id": "KMIPKIT-0037",
        "owner_requirement_ids": ("KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-004"),
        "section_tables": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.27 Tables 244-246",
        "request_table": "Table 244 as request",
        "protocol_functional_states": {
            "": "verified",
            "::new": "verified",
            "::with_split_key_threshold": "verified",
        },
        "root_functional_states": {},
    },
}


def expected_direct_traits(public_type: str, implementation_type: str) -> set[str]:
    return {
        f"<{public_type} as core::any::Any>::type_id",
        f"<{public_type} as core::borrow::Borrow<{implementation_type}>>::borrow",
        f"<{public_type} as core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
        f"<{public_type} as core::convert::From<{implementation_type}>>::from",
        f"<{public_type} as core::fmt::Debug>::fmt",
    }


def expected_conditional_traits(public_type: str) -> set[str]:
    return {
        f"<{public_type} as core::convert::Into<U>>::into",
        f"<{public_type} as core::convert::TryFrom<U>>::Error",
        f"<{public_type} as core::convert::TryFrom<U>>::try_from",
        f"<{public_type} as core::convert::TryInto<U>>::Error",
        f"<{public_type} as core::convert::TryInto<U>>::try_into",
    }


class T102ProtocolRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_exact_trait_paths_are_compiler_observed_and_excluded_by_owner(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(self.rust_report["issues"], [])

        all_expected_exclusions: set[tuple[str, str]] = set()
        for type_name, details in TYPES.items():
            public_type = f"kmipkit_protocol::{type_name}"
            implementation_type = f"kmipkit_protocol::{details['module']}::{type_name}"
            direct_paths = expected_direct_traits(public_type, implementation_type)
            conditional_paths = expected_conditional_traits(public_type)
            expected_paths = direct_paths | conditional_paths
            all_expected_exclusions.update(
                ("rust-kmipkit-protocol", export_name)
                for export_name in expected_paths
            )

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
            observed_excluded = {
                row["exportName"]
                for row in self.rust_report["excluded"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(f"<{public_type} as ")
            }
            with self.subTest(type_name=type_name, inventory="compiler audit"):
                self.assertEqual(observed_direct, direct_paths)
                self.assertEqual(observed_conditional, conditional_paths)
                self.assertEqual(observed_excluded, expected_paths)
                self.assertEqual(len(observed_direct), 5)
                self.assertEqual(len(observed_conditional), 5)

            declarations = {
                (row["packageId"], row["exportName"])
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(f"<{public_type} as ")
                and not is_pre_send_validation_trait(row)
            }
            exclusions = [
                row
                for row in self.manifest.get("approvedExclusions", [])
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(f"<{public_type} as ")
            ]
            with self.subTest(type_name=type_name, inventory="manifest"):
                self.assertEqual(declarations, set())
                self.assertEqual(
                    {(row["packageId"], row["exportName"]) for row in exclusions},
                    {("rust-kmipkit-protocol", name) for name in expected_paths},
                )
                self.assertEqual(len(exclusions), 10)
                for row in exclusions:
                    with self.subTest(export_name=row["exportName"]):
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
                        self.assertIn(details["request_table"], row["evidence"])
                        self.assertIn("inventory-only", row["evidence"])
                        self.assertIn("no OASIS conformance", row["evidence"])
                        self.assertIn("standard-trait implementation", row["reason"])
                        self.assertIn("not a separate KMIP", row["reason"])

        self.assertEqual(len(all_expected_exclusions), 40)
        manifest_exclusions = {
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
        self.assertEqual(manifest_exclusions, all_expected_exclusions)

    def test_protocol_and_root_functional_paths_and_states_remain_unchanged(self) -> None:
        for type_name, details in TYPES.items():
            protocol_type = f"kmipkit_protocol::{type_name}"
            root_type = f"kmipkit::{type_name}"
            protocol_functional_states = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName", "").startswith(protocol_type)
                and not row["exportName"].startswith("<")
            }
            expected_protocol_states = {
                f"{protocol_type}{suffix}": state
                for suffix, state in details["protocol_functional_states"].items()
            }
            root_states = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and row.get("exportName", "").startswith(root_type)
            }
            expected_root_states = {
                f"{root_type}{suffix}": state
                for suffix, state in details["root_functional_states"].items()
            }
            with self.subTest(type_name=type_name):
                self.assertEqual(protocol_functional_states, expected_protocol_states)
                self.assertEqual(root_states, expected_root_states)
            if type_name == "GetUsageAllocationRequest":
                root_trait_prefix = f"<{root_type} as "
                expected_root_traits = {
                    export_name.replace(protocol_type, root_type)
                    for export_name in (
                        expected_direct_traits(
                            protocol_type,
                            f"kmipkit_protocol::{details['module']}::{type_name}",
                        )
                        | expected_conditional_traits(protocol_type)
                    )
                }
                uid_debug = f"<{root_type} as core::fmt::Debug>::fmt"
                expected_root_traits.add(uid_debug)
                actual_root_traits = {
                    row["exportName"]
                    for row in self.manifest["approvedExclusions"]
                    if row.get("packageId") == "rust-kmipkit"
                    and row.get("exportName", "").startswith(root_trait_prefix)
                }
                self.assertEqual(actual_root_traits, expected_root_traits)

    def test_candidate_count_reflects_only_the_exact_forty_exclusions(self) -> None:
        self.assertEqual(
            sum(
                row.get("candidateSpecId") == "KMIPKIT-0043"
                and row["declarationState"] == "blocked"
                for row in self.manifest["publicDeclarations"]
            ),
            T102_BLOCKED_CANDIDATES,
        )


if __name__ == "__main__":
    unittest.main()
