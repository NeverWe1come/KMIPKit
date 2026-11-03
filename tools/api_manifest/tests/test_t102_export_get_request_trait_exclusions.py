"""Pin exact T102 Export and Get protocol request trait exclusions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = ROOT / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-11-01.json"
PACKAGE = "rust-kmipkit-protocol"
APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
DIRECT = {
    "ExportRequest": {
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::export::ExportRequest>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::export::ExportRequest>>::borrow_mut",
        "core::convert::From<kmipkit_protocol::export::ExportRequest>>::from",
        "core::default::Default>::default",
        "core::fmt::Debug>::fmt",
    },
    "GetRequest": {
        "core::any::Any>::type_id",
        "core::borrow::Borrow<kmipkit_protocol::get::GetRequest>>::borrow",
        "core::borrow::BorrowMut<kmipkit_protocol::get::GetRequest>>::borrow_mut",
        "core::convert::From<kmipkit_protocol::get::GetRequest>>::from",
        "core::default::Default>::default",
        "core::fmt::Debug>::fmt",
    },
}
OWNER = {
    "ExportRequest": {
        "requirements": (
            "KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004",
            "KMIPKIT-0037-FR-005", "KMIPKIT-0037-FR-008",
        ),
        "reference": "OASIS KMIP v2.1 Section 6.1.18 Tables 217-219",
        "tables": ("Table 217 Export Request Payload", "Table 218 Export Response Payload", "Table 219 Export Errors"),
    },
    "GetRequest": {
        "requirements": (
            "KMIPKIT-0017-FR-001", "KMIPKIT-0017-FR-002", "KMIPKIT-0017-FR-003",
        ),
        "reference": "OASIS KMIP v2.1 Section 6.1.19 Tables 220-222",
        "tables": ("Table 220 Get Request Payload", "Table 221 Get Response Payload", "Table 222 Get Errors"),
    },
}
FUNCTIONAL_STATES = {
    "ExportRequest": {
        "": "verified",
        "::into_ttlv_payload": "blocked",
        "::new": "blocked",
        "::unique_identifier": "blocked",
        "::with_unique_identifier": "blocked",
        "::with_key_format_type": "blocked",
        "::with_key_wrap_type": "blocked",
        "::with_key_compression_type": "blocked",
        "::with_key_wrapping_specification": "blocked",
    },
    "GetRequest": {
        "": "verified",
        "::unique_identifier": "blocked",
        "::key_format_type": "blocked",
        "::key_wrap_type": "blocked",
        "::key_compression_type": "blocked",
        "::key_wrapping_specification": "blocked",
        "::new": "blocked",
        "::to_ttlv_payload": "blocked",
        "::with_unique_identifier": "blocked",
        "::with_key_format_type": "blocked",
        "::with_key_wrap_type": "blocked",
        "::with_key_compression_type": "blocked",
        "::with_key_wrapping_specification": "blocked",
    },
}


def paths(type_name: str, suffixes: set[str]) -> set[str]:
    return {f"<kmipkit_protocol::{type_name} as {suffix}" for suffix in suffixes}


class T102ExportGetRequestTraitExclusionTests(unittest.TestCase):
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
                row["exportName"] for row in self.report["declarations"]
                if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(prefix)
            }
            conditional = {
                row["exportName"] for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(prefix)
            }
            excluded = {
                row["exportName"] for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(direct, expected_direct)
            self.assertEqual(conditional, expected_conditional)
            self.assertEqual(excluded, expected)
            self.assertEqual(len(expected), 11)
            all_pairs.update((PACKAGE, name) for name in expected)

            declarations = {
                row["exportName"] for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                row["exportName"]: row for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(declarations, set())
            self.assertEqual(set(exclusions), expected)
            self.assertEqual(len(exclusions), 11)

            owner = OWNER[type_name]
            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], APPROVAL)
                    for requirement_id in (*owner["requirements"], "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(owner["reference"], row["evidence"])
                    for table in owner["tables"]:
                        self.assertIn(table, row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(type_name, row["reason"])

        self.assertEqual(len(all_pairs), 22)

    def test_protocol_functional_states_and_absent_root_aliases_are_preserved(self) -> None:
        for type_name, suffix_states in FUNCTIONAL_STATES.items():
            protocol = f"kmipkit_protocol::{type_name}"
            expected = {f"{protocol}{suffix}": state for suffix, state in suffix_states.items()}
            actual = {
                row["exportName"]: row["declarationState"] for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and (row.get("exportName") == protocol or row.get("exportName", "").startswith(protocol + "::"))
                and not row.get("exportName", "").startswith("<")
            }
            self.assertEqual(actual, expected)
            api = next(row for row in self.manifest["publicDeclarations"]
                       if row.get("packageId") == PACKAGE and row.get("exportName") == protocol)
            capability = "KMIPKIT-ELEM-OP-C2S-EXPORT" if type_name == "ExportRequest" else "KMIPKIT-ELEM-OP-C2S-GET"
            self.assertEqual(api["capabilityId"], capability)

            root = f"kmipkit::{type_name}"
            root_rows = [
                row for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and (row.get("exportName") == root or row.get("exportName", "").startswith(root + "::")
                     or row.get("exportName", "").startswith(f"<{root} as "))
            ]
            self.assertEqual(root_rows, [])

    def test_candidate_count_moves_only_by_the_exact_twenty_two_paths(self) -> None:
        count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(count, 1008)


if __name__ == "__main__":
    unittest.main()
