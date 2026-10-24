"""Regression tests for actor attribution in the pinned Import source clause."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog.generate_profiles import compute_profile_closure


ROOT = Path(__file__).resolve().parents[3]
REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.25-001"
SOURCE_CLAUSE_ID = "KMIPKIT-CLAUSE-SPEC-6.1.25-001"


class ImportActorScopeTests(unittest.TestCase):
    """Keep an actor-ambiguous MUST visible without applying it to clients."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        cls.oracle = json.loads(
            (
                ROOT
                / "tools"
                / "normative_catalog"
                / "tests"
                / "fixtures"
                / "profile-closure-v2.1.json"
            ).read_text(encoding="utf-8")
        )

    def test_actor_ambiguous_import_must_blocks_client_profile_readiness(self) -> None:
        requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        clauses = {row["clause_id"]: row for row in self.catalog["source_clauses"]}
        requirement = requirements[REQUIREMENT_ID]
        clause = clauses[SOURCE_CLAUSE_ID]

        self.assertEqual(requirement["role"], "unspecified")
        self.assertEqual(requirement["direction"], "unclear")
        self.assertEqual(requirement["scope_state"], "unclear")
        self.assertEqual(requirement["status"], "unassigned")
        self.assertIsNone(requirement["feature_spec"])
        self.assertEqual(requirement["implementation_refs"], [])
        self.assertEqual(requirement["verification_refs"], [])
        self.assertEqual(clause["role"], "unclear")
        self.assertEqual(clause["direction"], "unclear")
        self.assertEqual(clause["scope_state"], "unclear")

        paragraph_review = next(
            row
            for row in self.oracle["source_requirement_reviews"]
            if row["source_clause_id"] == SOURCE_CLAUSE_ID
        )
        rejected = [
            row
            for row in paragraph_review["rejected_candidate_mappings"]
            if row["requirement_id"] == REQUIREMENT_ID
        ]
        self.assertEqual(len(rejected), 1)
        self.assertEqual(
            rejected[0]["expected_disposition"],
            "reject_as_client_requirement_actor_ambiguous",
        )
        self.assertFalse(
            any(row["requirement_id"] == REQUIREMENT_ID for row in self.oracle["requirement_crosswalk"])
        )

        for profile in (row for row in self.catalog["profiles"] if row["role"] == "client"):
            closure = compute_profile_closure(
                self.catalog,
                profile["direct_source_clause_ids"],
                profile_id=profile["profile_id"],
            )
            with self.subTest(profile_id=profile["profile_id"]):
                self.assertNotIn(REQUIREMENT_ID, closure.requirement_ids)
                self.assertTrue(
                    any(
                        item.requirement_id == REQUIREMENT_ID
                        and item.source_clause_id == SOURCE_CLAUSE_ID
                        for item in closure.unresolved_scope_obligations
                    ),
                    "an actor-ambiguous source obligation must block profile readiness",
                )


if __name__ == "__main__":
    unittest.main()
