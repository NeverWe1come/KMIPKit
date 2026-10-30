"""Regression tests for bidirectional normative requirement references."""

from __future__ import annotations

import json
import unittest

from tools.normative_catalog.audit_sources import audit_document
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

    def test_retired_candidate_review_matches_current_catalog_disposition(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        fixture = json.loads(
            (
                ROOT
                / "tools"
                / "normative_catalog"
                / "tests"
                / "fixtures"
                / "profile-closure-v2.1.json"
            ).read_text(encoding="utf-8")
        )
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.17-005-001"
        requirement = next(
            item for item in catalog["requirements"] if item["requirement_id"] == requirement_id
        )
        source_review = next(
            item
            for item in fixture["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-6.1.17-005"
        )

        self.assertEqual(requirement["status"], "retired")
        self.assertEqual(requirement["scope_state"], "out_of_scope")
        self.assertNotIn("currently says client", source_review["review_note"])
        self.assertIn("retired", source_review["review_note"])

    def test_profile_clause_keyword_inventory_matches_pinned_source_without_changing_requirement(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        source_id = "KMIPKIT-SRC-profiles"
        clause_id = "KMIPKIT-CLAUSE-PROF-4.1.2-007"
        source_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        audited_clause = next(
            item for item in audit_document(source_path.read_bytes(), source_id) if item["clause_id"] == clause_id
        )
        catalog_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == clause_id
        )
        requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-PROF-4.1.2-007"
        )

        self.assertEqual(catalog_clause["source_keywords"], audited_clause["source_keywords"])
        self.assertEqual(requirement["source_keyword"], "MAY")

    def test_out_of_scope_profile_clause_keyword_inventory_matches_pinned_source(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        source_id = "KMIPKIT-SRC-profiles"
        clause_id = "KMIPKIT-CLAUSE-PROF-4.1.2-008"
        source_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        audited_clause = next(
            item for item in audit_document(source_path.read_bytes(), source_id) if item["clause_id"] == clause_id
        )
        catalog_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == clause_id
        )

        self.assertEqual(catalog_clause["source_keywords"], audited_clause["source_keywords"])


if __name__ == "__main__":
    unittest.main()
