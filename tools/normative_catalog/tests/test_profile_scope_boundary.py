"""Regression tests for client-profile applicability against the approved 1.0 scope."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog.generate_profiles import compute_profile_closure


ROOT = Path(__file__).resolve().parents[3]
PKCS11_REQUIREMENT_IDS = (
    "KMIPKIT-REQ-PROF-5.18.1-004",
    "KMIPKIT-REQ-PROF-5.18.1-005",
    "KMIPKIT-REQ-PROF-5.18.3-001",
    "KMIPKIT-REQ-PROF-5.18.3-002",
    "KMIPKIT-REQ-PROF-5.18.3-003",
    "KMIPKIT-REQ-PROF-5.18.3-005",
)


class ProfileScopeBoundaryTests(unittest.TestCase):
    def test_pkcs11_encoding_reference_expands_the_reviewed_profile_clause_closure(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(
                encoding="utf-8"
            )
        )
        oracle = json.loads(
            (
                ROOT
                / "tools"
                / "normative_catalog"
                / "tests"
                / "fixtures"
                / "profile-closure-v2.1.json"
            ).read_text(encoding="utf-8")
        )
        profile_id = "KMIPKIT-PROFILE-PKCS11-CLIENT"
        edge_id = "KMIPKIT-INCLUSION-PROF-PKCS11-ENCODING"
        included_clause_ids = {
            f"KMIPKIT-CLAUSE-PROF-5.18.1-{ordinal:03}"
            for ordinal in range(1, 6)
        }
        catalog_profile = next(
            row for row in catalog["profiles"] if row["profile_id"] == profile_id
        )
        oracle_profile = next(
            row for row in oracle["profiles"] if row["profile_id"] == profile_id
        )
        oracle_edge = next(
            (
                row
                for row in oracle["provenance"]["inclusion_edges"]
                if row["inclusion_id"] == edge_id
            ),
            None,
        )
        inclusions = {
            row["inclusion_id"]: row for row in catalog["normative_inclusions"]
        }
        source_clauses = {
            row["clause_id"]: row for row in oracle["source_clause_crosswalk"]
        }
        clauses = {
            row["clause_id"]: row for row in catalog["source_clauses"]
        }
        review = next(
            row
            for row in oracle["source_requirement_reviews"]
            if row["source_clause_id"] == "KMIPKIT-CLAUSE-PROF-5.18.3-002"
        )

        self.assertEqual(review["source_excerpt"], "SHALL conform with PKCS#11 Encoding (5.18.1)")
        self.assertEqual(review["html_line_range"], [10405, 10405])
        self.assertIn(edge_id, inclusions)
        self.assertEqual(
            inclusions[edge_id],
            {
                "inclusion_id": edge_id,
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-5.18.3-002",
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.18.1",
            },
        )
        self.assertIsNotNone(oracle_edge)
        self.assertEqual(
            {
                key: oracle_edge[key]
                for key in (
                    "inclusion_id",
                    "source_clause_id",
                    "included_source_id",
                    "included_section",
                )
            },
            inclusions[edge_id],
        )
        self.assertEqual(
            oracle_edge["source_locator"], "Profiles v2.1 section 5.18.3 item 2"
        )
        self.assertIn(edge_id, oracle_profile["normative_inclusion_ids"])
        self.assertTrue(included_clause_ids.issubset(oracle_profile["source_clause_ids"]))
        self.assertTrue(included_clause_ids.issubset(catalog_profile["source_clause_ids"]))
        self.assertEqual(
            {
                clause_id: (
                    source_clauses[clause_id]["locator"]["ordinal"],
                    source_clauses[clause_id]["locator"]["block_kind"],
                )
                for clause_id in included_clause_ids
            },
            {
                f"KMIPKIT-CLAUSE-PROF-5.18.1-{ordinal:03}": (ordinal, block_kind)
                for ordinal, block_kind in enumerate(
                    ("paragraph", "paragraph", "paragraph", "paragraph", "list_item"),
                    start=1,
                )
            },
        )
        self.assertTrue(
            any(
                "lowercase may/required" in boundary
                and "do not create normative requirements" in boundary
                for boundary in oracle["provenance"]["normative_boundaries"]
            )
        )
        for clause_id in included_clause_ids:
            self.assertEqual(clauses[clause_id]["scope_state"], "out_of_scope")
        for ordinal in range(1, 4):
            clause = clauses[f"KMIPKIT-CLAUSE-PROF-5.18.1-{ordinal:03}"]
            self.assertEqual(clause["requirement_ids"], [])
        for requirement_id in PKCS11_REQUIREMENT_IDS:
            clause_id = next(
                row["clause_id"]
                for row in catalog["source_clauses"]
                if requirement_id in row["requirement_ids"]
            )
            self.assertEqual(clauses[clause_id]["requirement_ids"], [requirement_id])

        closure = compute_profile_closure(
            catalog,
            catalog_profile["direct_source_clause_ids"],
            profile_id=profile_id,
        )
        self.assertTrue(included_clause_ids.issubset(closure.source_clause_ids))

    def test_pkcs11_encoding_requirements_remain_outside_the_ttlv_only_client_scope(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(
                encoding="utf-8"
            )
        )
        oracle = json.loads(
            (
                ROOT
                / "tools"
                / "normative_catalog"
                / "tests"
                / "fixtures"
                / "profile-closure-v2.1.json"
            ).read_text(encoding="utf-8")
        )
        requirements = {
            row["requirement_id"]: row for row in catalog["requirements"]
        }
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        reviews = {
            row["source_clause_id"]: row
            for row in oracle["source_requirement_reviews"]
        }
        pkcs11_profile_id = "KMIPKIT-PROFILE-PKCS11-CLIENT"
        profile = next(
            row for row in catalog["profiles"] if row["profile_id"] == pkcs11_profile_id
        )

        self.assertEqual(profile["applicability"], "out_of_scope")
        self.assertEqual(oracle["source_closure_status"], "complete")

        for requirement_id in PKCS11_REQUIREMENT_IDS:
            requirement = requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            review = reviews[clause_id]

            self.assertEqual(requirement["role"], "client")
            self.assertEqual(requirement["scope_state"], "out_of_scope")
            self.assertIn(pkcs11_profile_id, requirement["profile_ids"])
            self.assertEqual(clauses[clause_id]["scope_state"], "out_of_scope")
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertTrue(
                all(
                    obligation["scope_disposition"] == "out_of_scope"
                    for obligation in review["source_obligations"]
                )
            )
            self.assertNotIn(requirement_id, profile["requirement_ids"])


if __name__ == "__main__":
    unittest.main()
