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
    """Keep the Import MUST visible as a documented server-side duty."""

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

    def test_import_must_is_server_only_by_accepted_operation_context(self) -> None:
        requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        clauses = {row["clause_id"]: row for row in self.catalog["source_clauses"]}
        requirement = requirements[REQUIREMENT_ID]
        clause = clauses[SOURCE_CLAUSE_ID]

        self.assertEqual(requirement["role"], "server")
        self.assertEqual(requirement["direction"], "not_applicable")
        self.assertEqual(requirement["scope_state"], "server_only")
        self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-033")
        self.assertEqual(requirement["status"], "unassigned")
        self.assertIsNone(requirement["feature_spec"])
        self.assertEqual(requirement["implementation_refs"], [])
        self.assertEqual(requirement["verification_refs"], [])
        self.assertEqual(clause["role"], "server")
        self.assertEqual(clause["direction"], "not_applicable")
        self.assertEqual(clause["scope_state"], "server_only")

        paragraph_review = next(
            row
            for row in self.oracle["source_requirement_reviews"]
            if row["source_clause_id"] == SOURCE_CLAUSE_ID
        )
        obligation = paragraph_review["source_obligations"][0]
        self.assertEqual(obligation["actor"], "unresolved")
        self.assertEqual(obligation["project_actor_interpretation"], "server")
        self.assertEqual(obligation["scope_disposition"], "server_only")
        self.assertEqual(paragraph_review["expected_client_requirement_ids"], [])
        self.assertFalse(
            any(row["requirement_id"] == REQUIREMENT_ID for row in paragraph_review["rejected_candidate_mappings"])
        )
        crosswalk = next(
            row for row in self.oracle["requirement_crosswalk"]
            if row["requirement_id"] == REQUIREMENT_ID
        )
        self.assertEqual(crosswalk["role"], "server")
        self.assertEqual(crosswalk["scope_state"], "server_only")
        self.assertEqual(crosswalk["decision_id"], "KMIPKIT-DEC-033")
        decision = next(
            row for row in self.catalog["decisions"]
            if row["decision_id"] == "KMIPKIT-DEC-033"
        )
        self.assertEqual(decision["status"], "accepted")
        self.assertIn(REQUIREMENT_ID, decision["requirement_ids"])

        for profile in (row for row in self.catalog["profiles"] if row["role"] == "client"):
            closure = compute_profile_closure(
                self.catalog,
                profile["direct_source_clause_ids"],
                profile_id=profile["profile_id"],
            )
            with self.subTest(profile_id=profile["profile_id"]):
                self.assertNotIn(REQUIREMENT_ID, closure.requirement_ids)
                self.assertFalse(
                    any(item.requirement_id == REQUIREMENT_ID for item in closure.unresolved_scope_obligations),
                    "the accepted server-only interpretation must not block client profile readiness",
                )


if __name__ == "__main__":
    unittest.main()
