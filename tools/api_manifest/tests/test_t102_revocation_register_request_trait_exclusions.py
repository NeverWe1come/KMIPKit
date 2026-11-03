"""Pin the exact T102 RevocationReason and RegisterRequest trait exclusions."""

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
OWNER_APPROVALS = {
    "RevocationReason": {
        "kind": "feature-specification",
        "path": "specs/022-object-lifecycle/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved for implementation — 2026-10-11",
    },
    "RegisterRequest": {
        "kind": "feature-specification",
        "path": "specs/029-opaque-object-context/spec.md",
        "status": "Approved",
        "evidence": (
            "**Status**: Approved for implementation under the maintainer's "
            "autonomous roadmap instruction dated 2026-10-13"
        ),
    },
}
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
DIRECT = {
    "RevocationReason": {
        "alloc::borrow::ToOwned>::Owned",
        "alloc::borrow::ToOwned>::clone_into",
        "alloc::borrow::ToOwned>::to_owned",
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::revoke::RevocationReason>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::revoke::RevocationReason>>::borrow_mut",
        "core::clone::Clone>::clone",
        "core::clone::Clone>::clone_from",
        "core::clone::CloneToUninit>::clone_to_uninit",
        "core::cmp::Eq>::assert_receiver_is_total_eq",
        "core::cmp::PartialEq>::eq",
        "core::cmp::PartialEq>::ne",
        "core::convert::From<kmipkit_protocol::revoke::RevocationReason>>::from",
        "core::fmt::Debug>::fmt",
    },
    "RegisterRequest": {
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::register::RegisterRequest>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::register::RegisterRequest>>::borrow_mut",
        "core::convert::From<kmipkit_protocol::register::RegisterRequest>>::from",
        "core::fmt::Debug>::fmt",
    },
}
OWNER = {
    "RevocationReason": {
        "requirements": (
            "KMIPKIT-0036-FR-001",
            "KMIPKIT-0036-FR-004",
            "KMIPKIT-0036-FR-006",
            "KMIPKIT-REQ-SPEC-6.1.44-001",
            "KMIPKIT-REQ-SPEC-6.1.44-003-001",
            "KMIPKIT-REQ-SPEC-6.1.44-003-002",
        ),
        "references": (
            "OASIS KMIP v2.1 Section 11.48, Table 481",
            "OASIS KMIP v2.1 Section 4.47, Table 123",
            "OASIS KMIP v2.1 Section 6.1.44, Tables 295-297",
        ),
        "tables": (
            "Table 481: Revocation Reason Code Enumeration",
            "Table 123: Revocation Reason Attribute Structure",
            "Table 295: Revoke Request Payload",
            "Table 296: Revoke Response Payload",
            "Table 297: Revoke Errors",
        ),
    },
    "RegisterRequest": {
        "requirements": (
            "KMIPKIT-0044-FR-001",
            "KMIPKIT-0044-FR-007",
            "KMIPKIT-REQ-SPEC-6.1.43-001",
        ),
        "references": (
            "OASIS KMIP v2.1 Section 2.3 (Opaque Object)",
            "Section 2.3, Table 4",
            "Section 11.34, Table 468",
            "Section 11.35, Table 469",
            "Section 6.1.43, Table 291",
        ),
        "tables": (
            "Table 4: Opaque Object Structure",
            "Table 468: Object Type Enumeration",
            "Table 469: Opaque Data Type Enumeration",
            "Table 291: Register Request Payload",
        ),
    },
}
FUNCTIONAL_STATES = {
    "RevocationReason": {
        "": "verified",
        "::AFFILIATION_CHANGED": "blocked",
        "::CA_COMPROMISE": "blocked",
        "::CESSATION_OF_OPERATION": "blocked",
        "::KEY_COMPROMISE": "blocked",
        "::PRIVILEGE_WITHDRAWN": "blocked",
        "::SUPERSEDED": "blocked",
        "::UNSPECIFIED": "blocked",
        "::from_raw": "verified",
        "::raw": "verified",
    },
    "RegisterRequest": {
        "": "verified",
        "::attributes": "blocked",
        "::into_ttlv_payload": "blocked",
        "::new": "declared",
        "::object": "blocked",
        "::object_type": "blocked",
        "::with_opaque_object_context": "verified",
        "::with_protection_storage_masks": "blocked",
    },
}


def paths(type_name: str, suffixes: set[str]) -> set[str]:
    return {f"<kmipkit_protocol::{type_name} as {suffix}" for suffix in suffixes}


class T102RevocationRegisterTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_trait_paths_are_owner_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        all_pairs: set[tuple[str, str]] = set()

        for type_name, direct_suffixes in DIRECT.items():
            expected_direct = paths(type_name, direct_suffixes)
            expected_conditional = paths(type_name, CONDITIONAL)
            expected = expected_direct | expected_conditional
            prefix = f"<kmipkit_protocol::{type_name} as "
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
            excluded = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(direct, expected_direct)
            self.assertEqual(conditional, expected_conditional)
            self.assertEqual(excluded, expected)
            self.assertEqual(len(expected), 19 if type_name == "RevocationReason" else 10)
            all_pairs.update((PACKAGE, name) for name in expected)

            declarations = {
                row["exportName"]
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
            self.assertEqual(declarations, set())
            self.assertEqual(set(exclusions), expected)
            self.assertEqual(len(exclusions), len(expected))

            owner = OWNER[type_name]
            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVALS[type_name])
                    for requirement_id in (
                        *owner["requirements"],
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    ):
                        self.assertIn(requirement_id, row["evidence"])
                    for reference in owner["references"]:
                        self.assertIn(reference, row["evidence"])
                    for table in owner["tables"]:
                        self.assertIn(table, row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(type_name, row["reason"])

        self.assertEqual(len(all_pairs), 29)

    def test_protocol_functional_states_and_absent_root_aliases_are_preserved(self) -> None:
        for type_name, suffix_states in FUNCTIONAL_STATES.items():
            protocol = f"kmipkit_protocol::{type_name}"
            expected = {f"{protocol}{suffix}": state for suffix, state in suffix_states.items()}
            actual = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and (
                    row.get("exportName") == protocol
                    or row.get("exportName", "").startswith(protocol + "::")
                )
                and not row.get("exportName", "").startswith("<")
            }
            self.assertEqual(actual, expected)

            root = f"kmipkit::{type_name}"
            root_rows = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and (
                    row.get("exportName") == root
                    or row.get("exportName", "").startswith(root + "::")
                    or row.get("exportName", "").startswith(f"<{root} as ")
                )
            ]
            self.assertEqual(root_rows, [])

    def test_candidate_and_manifest_totals_move_only_by_twenty_nine_paths(self) -> None:
        count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(count, 3182)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 8380)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 6344)


if __name__ == "__main__":
    unittest.main()
