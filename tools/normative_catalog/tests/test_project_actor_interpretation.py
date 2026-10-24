"""TDD regression for project actor recommendations in conditional source scope."""

from __future__ import annotations

import unittest

from tools.normative_catalog.tests.test_validate import (
    _reviewed_profile_conditional_capability_sources,
    _reviewed_profile_optional_capabilities,
)


class ProjectActorInterpretationTests(unittest.TestCase):
    def test_explicit_project_actor_interpretation_preserves_source_actor(self) -> None:
        """An explicit project recommendation does not overwrite the pinned source actor."""
        requirement_id = "KMIPKIT-REQ-SPEC-TEST-PROJECT-ACTOR-MAY"
        source_clause_id = "KMIPKIT-CLAUSE-SPEC-TEST-PROJECT-ACTOR"
        obligation = {
            "source_keyword": "MAY",
            "scope_disposition": "profile_conditional",
            "candidate_requirement_id": requirement_id,
            "condition": "The capability applies when the relevant representation is used.",
            "actor": "unresolved",
            "project_actor_interpretation": "client",
        }
        oracle = {
            "source_requirement_reviews": [
                {"source_clause_id": source_clause_id, "source_obligations": [obligation]}
            ],
            "source_only_requirement_gaps": [],
        }

        capabilities, gaps = _reviewed_profile_optional_capabilities(
            oracle,
            {source_clause_id},
            set(),
        )
        conditional_sources = _reviewed_profile_conditional_capability_sources(
            oracle,
            {source_clause_id},
            set(),
        )

        self.assertEqual(obligation["actor"], "unresolved")
        self.assertIn(requirement_id, capabilities)
        self.assertFalse(any("no reviewed client actor" in gap for gap in gaps))
        self.assertEqual(conditional_sources[requirement_id], {source_clause_id})


if __name__ == "__main__":
    unittest.main()
