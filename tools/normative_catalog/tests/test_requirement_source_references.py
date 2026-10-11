"""Regression tests for bidirectional normative requirement references."""

from __future__ import annotations

import json
import unittest

from tools.normative_catalog.tests.test_validate import ROOT


class RequirementSourceReferenceTests(unittest.TestCase):
    def test_active_state_fourth_predicate_is_listed_by_its_source_clause(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        requirement_id = "KMIPKIT-REQ-SPEC-4.57-003-004"
        requirement = next(
            item for item in catalog["requirements"] if item["requirement_id"] == requirement_id
        )
        clause_id = "KMIPKIT-CLAUSE-SPEC-4.57-003"
        source_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == clause_id
        )

        self.assertIn(clause_id, requirement["source_clause_ids"])
        self.assertIn(requirement_id, source_clause["requirement_ids"])


if __name__ == "__main__":
    unittest.main()
