"""Regression tests for client-profile applicability against the approved 1.0 scope."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


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
        self.assertEqual(oracle["oracle_status"], "partial-source-reviewed; not complete T003 oracle")

        for requirement_id in PKCS11_REQUIREMENT_IDS:
            requirement = requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            review = reviews[clause_id]

            self.assertEqual(requirement["role"], "client")
            self.assertEqual(requirement["scope_state"], "out_of_scope")
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
