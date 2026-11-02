"""Exact T102 inventory and state-preservation contract for credential responses."""

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
API_SPEC_PATH = "specs/043-public-api-bindings/spec.md"

RESPONSES = {
    "DelegatedLoginResponse": {
        "module": "delegated_login",
        "section": "§6.1.12",
        "response_table": "Table 200",
        "cases": "Delegated Login cases 2.21–2.33",
        "protocol_states": {
            "type": "verified",
            "result": "blocked",
            "ticket": "verified",
            "try_from_response_item": "blocked",
        },
        "root_members": ("result", "ticket", "try_from_response_item"),
    },
    "LoginResponse": {
        "module": "login",
        "section": "§6.1.30",
        "response_table": "Table 254",
        "cases": "Login cases 2.48–2.50",
        "protocol_states": {
            "type": "verified",
            "result": "blocked",
            "ticket": "verified",
            "try_from_response_item": "blocked",
        },
        "root_members": ("result", "ticket", "try_from_response_item"),
    },
    "LogoutResponse": {
        "module": "logout",
        "section": "§6.1.31",
        "response_table": "Table 257",
        "cases": "no direct Logout case",
        "protocol_states": {
            "type": "verified",
            "result": "verified",
            "try_from_response_item": "blocked",
        },
        "root_members": ("result", "try_from_response_item"),
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


class T102CredentialResponseTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_protocol_trait_paths_are_compiler_observed_and_excluded(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertEqual(self.rust_report["issues"], [])

        for response_type, details in RESPONSES.items():
            public_type = f"kmipkit_protocol::{response_type}"
            implementation_type = f"kmipkit_protocol::{details['module']}::{response_type}"
            direct_paths = expected_direct_traits(public_type, implementation_type)
            conditional_paths = expected_conditional_traits(public_type)

            with self.subTest(response_type=response_type):
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
                self.assertEqual(observed_direct, direct_paths)
                self.assertEqual(observed_conditional, conditional_paths)
                self.assertEqual(len(observed_direct), 5)
                self.assertEqual(len(observed_conditional), 5)

                protocol_declarations = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                }
                protocol_exclusions = [
                    row
                    for row in self.manifest.get("approvedExclusions", [])
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                ]
                self.assertEqual(protocol_declarations, {})
                self.assertEqual(
                    {row["exportName"] for row in protocol_exclusions},
                    direct_paths | conditional_paths,
                )
                self.assertEqual(len(protocol_exclusions), 10)

                for row in protocol_exclusions:
                    with self.subTest(response_type=response_type, export_name=row["exportName"]):
                        self.assertIn("standard-trait implementation", row["reason"])
                        self.assertIn("not a separate KMIP", row["reason"])
                        self.assertEqual(
                            row["approvalSource"],
                            {
                                "kind": "feature-specification",
                                "path": API_SPEC_PATH,
                                "status": "Approved",
                                "evidence": "**Status**: Approved for implementation",
                            },
                        )
                        self.assertIn("KMIPKIT-0038-FR-001", row["evidence"])
                        self.assertIn("KMIPKIT-0038-FR-004", row["evidence"])
                        self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                        self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                        self.assertIn(details["section"], row["evidence"])
                        self.assertIn(details["response_table"], row["evidence"])
                        self.assertIn(details["cases"], row["evidence"])
                        self.assertIn("inventory-only", row["evidence"])

    def test_functional_protocol_and_root_facade_rows_keep_their_states(self) -> None:
        for response_type, details in RESPONSES.items():
            protocol_type = f"kmipkit_protocol::{response_type}"
            root_type = f"kmipkit::{response_type}"

            with self.subTest(response_type=response_type):
                protocol_rows = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and (
                        row.get("exportName") == protocol_type
                        or row.get("exportName", "").startswith(f"{protocol_type}::")
                    )
                }
                expected_protocol_states = {
                    protocol_type if member == "type" else f"{protocol_type}::{member}": state
                    for member, state in details["protocol_states"].items()
                }
                self.assertEqual(
                    {name: row["declarationState"] for name, row in protocol_rows.items()},
                    expected_protocol_states,
                )
                for row in protocol_rows.values():
                    self.assertEqual(row["parityScope"], "four-language")

                root_members = [f"{root_type}::{member}" for member in details["root_members"]]
                expected_root_paths = {root_type, *root_members}
                root_direct = expected_direct_traits(
                    root_type,
                    f"kmipkit_protocol::{details['module']}::{response_type}",
                )
                root_conditional = expected_conditional_traits(root_type)
                expected_root_paths.update(root_direct | root_conditional)
                root_rows = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == "rust-kmipkit"
                    and (
                        row.get("exportName") == root_type
                        or row.get("exportName", "").startswith(f"{root_type}::")
                        or row.get("exportName", "").startswith(f"<{root_type} as ")
                    )
                }
                self.assertEqual(set(root_rows), expected_root_paths)
                self.assertTrue(
                    all(row["declarationState"] == "blocked" for row in root_rows.values())
                )
                self.assertTrue(
                    all(row["candidateSpecId"] == "KMIPKIT-0043" for row in root_rows.values())
                )
                self.assertTrue(
                    all(row["parityScope"] == "four-language" for row in root_rows.values())
                )
                root_exclusions = {
                    row["exportName"]
                    for row in self.manifest.get("approvedExclusions", [])
                    if row.get("packageId") == "rust-kmipkit"
                    and row.get("exportName", "").startswith(f"<{root_type} as ")
                }
                self.assertEqual(root_exclusions, set())


if __name__ == "__main__":
    unittest.main()
