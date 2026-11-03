"""Pin the exact KMIPKIT-0043 RNG Retrieve request trait exclusions."""

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
PROTOCOL_TYPE = "kmipkit_protocol::RngRetrieveRequest"
ROOT_TYPE = "kmipkit::RngRetrieveRequest"
PROTOCOL_PREFIX = f"<{PROTOCOL_TYPE} as "
ROOT_PREFIX = f"<{ROOT_TYPE} as "
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
DIRECT = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "core::any::Any>::type_id",
    "core::borrow::Borrow<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::borrow",
    "core::borrow::BorrowMut<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::borrow_mut",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::convert::From<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::from",
    "core::fmt::Debug>::fmt",
}
EXPECTED = {f"<{PROTOCOL_TYPE} as {suffix}" for suffix in DIRECT | CONDITIONAL}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0041-FR-001",
    "KMIPKIT-0041-FR-002",
    "KMIPKIT-0041-FR-004",
)
NORMATIVE_REFERENCE = "OASIS KMIP v2.1 Section 6.1.49 Tables 316-318"


class T102RngRetrieveRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_trait_paths_are_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        direct = {
            row["exportName"] for row in self.report["declarations"]
            if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        conditional = {
            row["exportName"] for row in self.report["conditionalDeclarations"]
            if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        excluded = {
            row["exportName"] for row in self.report["excluded"]
            if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        self.assertEqual(direct, {f"<{PROTOCOL_TYPE} as {suffix}" for suffix in DIRECT})
        self.assertEqual(conditional, {f"<{PROTOCOL_TYPE} as {suffix}" for suffix in CONDITIONAL})
        self.assertEqual(excluded, EXPECTED)
        self.assertEqual(len(direct), 14)
        self.assertEqual(len(conditional), 5)

        declarations = {
            row["exportName"] for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        exclusions = {
            row["exportName"]: row for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == PACKAGE and row.get("exportName", "").startswith(PROTOCOL_PREFIX)
        }
        self.assertEqual(declarations, set())
        self.assertEqual(set(exclusions), EXPECTED)
        self.assertEqual(len(exclusions), 19)
        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], APPROVAL)
                for requirement_id in (*OWNER_REQUIREMENTS, "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"):
                    self.assertIn(requirement_id, row["evidence"])
                self.assertIn(NORMATIVE_REFERENCE, row["evidence"])
                self.assertIn("Table 316 required Data Length", row["evidence"])
                self.assertIn("Table 317 response Data", row["evidence"])
                self.assertIn("Table 318 RNG Retrieve Errors", row["evidence"])
                self.assertIn("inventory-only", row["evidence"])
                self.assertIn("makes no conformance claim", row["evidence"])
                self.assertIn("exact Rust standard-trait implementation", row["reason"])
                self.assertIn("not a separate KMIP client capability", row["reason"])

    def test_protocol_and_root_alias_functional_states_remain_blocked_or_verified(self) -> None:
        protocol_rows = {
            row["exportName"]: row["declarationState"] for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == PACKAGE
            and (row.get("exportName") == PROTOCOL_TYPE or row.get("exportName", "").startswith(PROTOCOL_TYPE + "::"))
        }
        self.assertEqual(protocol_rows, {
            PROTOCOL_TYPE: "verified",
            f"{PROTOCOL_TYPE}::new": "verified",
            f"{PROTOCOL_TYPE}::data_length": "blocked",
            f"{PROTOCOL_TYPE}::to_ttlv_payload": "blocked",
        })

        expected_root_traits = {path.replace(PROTOCOL_TYPE, ROOT_TYPE, 1) for path in EXPECTED}
        expected_root_api = {
            ROOT_TYPE,
            f"{ROOT_TYPE}::new",
            f"{ROOT_TYPE}::data_length",
            f"{ROOT_TYPE}::to_ttlv_payload",
        }
        root_rows = {
            row["exportName"]: row["declarationState"] for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and (row.get("exportName") in expected_root_api or row.get("exportName", "").startswith(ROOT_PREFIX))
        }
        self.assertEqual(set(root_rows), expected_root_traits | expected_root_api)
        self.assertEqual(len(expected_root_traits), 19)
        self.assertEqual(len(expected_root_api), 4)
        self.assertTrue(all(state == "blocked" for state in root_rows.values()))

    def test_candidate_count_moves_only_by_the_nineteen_protocol_paths(self) -> None:
        candidate_count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidate_count, 2213)


if __name__ == "__main__":
    unittest.main()
