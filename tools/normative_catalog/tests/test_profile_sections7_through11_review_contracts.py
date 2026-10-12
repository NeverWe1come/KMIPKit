"""Precise source dispositions for KMIP Specification sections 7 through 11."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class ProfileSectionsSevenThroughElevenReviewContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json").read_text(
                encoding="utf-8"
            )
        )
        cls.catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        cls.reviews = {
            row["source_clause_id"]: row for row in cls.oracle["source_requirement_reviews"]
        }
        cls.requirements = {
            row["requirement_id"]: row for row in cls.catalog["requirements"]
        }

    def test_titlecase_may_candidates_are_explicitly_rejected_without_client_crosswalk(self) -> None:
        expected = {
            "KMIPKIT-REQ-SPEC-7.12-002": "KMIPKIT-CLAUSE-SPEC-7.12-002",
            "KMIPKIT-REQ-SPEC-7.7-002": "KMIPKIT-CLAUSE-SPEC-7.7-002",
            "KMIPKIT-REQ-SPEC-7.35-002": "KMIPKIT-CLAUSE-SPEC-7.35-002",
        }
        crosswalk_ids = {row["requirement_id"] for row in self.oracle["requirement_crosswalk"]}

        for requirement_id, clause_id in expected.items():
            with self.subTest(requirement_id=requirement_id):
                review = self.reviews[clause_id]
                self.assertEqual(review["source_obligations"], [])
                self.assertIn("May be repeated", review["source_excerpt"])
                self.assertIn(
                    requirement_id,
                    {row["requirement_id"] for row in review["rejected_candidate_mappings"]},
                )
                self.assertNotIn(requirement_id, crosswalk_ids)
                requirement = self.requirements[requirement_id]
                self.assertEqual(requirement["scope_state"], "out_of_scope")
                self.assertEqual(requirement["status"], "retired")

    def test_server_predicates_keep_server_actor_and_stay_out_of_client_crosswalk(self) -> None:
        expected = {
            "KMIPKIT-REQ-SPEC-7.18-001-002",
            "KMIPKIT-REQ-SPEC-7.33-001",
            "KMIPKIT-REQ-SPEC-9.2-001",
            "KMIPKIT-REQ-SPEC-9.3-001-002",
        }
        crosswalk = [
            row for row in self.oracle["requirement_crosswalk"] if row["requirement_id"] in expected
        ]
        self.assertEqual({row["requirement_id"] for row in crosswalk}, expected)
        self.assertTrue(all(row["role"] == "server" for row in crosswalk))
        self.assertTrue(all(row["source_actor"] == "server" for row in crosswalk))
        self.assertTrue(all(row["scope_state"] == "server_only" for row in crosswalk))
        for requirement_id in expected:
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                self.assertEqual(requirement["role"], "server")
                self.assertEqual(requirement["scope_state"], "server_only")

        response_id = "KMIPKIT-REQ-SPEC-9.21-001-002"
        response_requirement = self.requirements[response_id]
        self.assertEqual(response_requirement["role"], "unspecified")
        self.assertEqual(response_requirement["scope_state"], "unclear")
        self.assertNotIn(response_id, {row["requirement_id"] for row in self.oracle["requirement_crosswalk"]})
        response_review = self.reviews["KMIPKIT-CLAUSE-SPEC-9.21-001"]
        response_atom = next(
            atom for atom in response_review["source_obligations"]
            if atom["candidate_requirement_id"] == response_id
        )
        self.assertEqual(response_atom["actor"], "response producer (endpoint not named)")
        self.assertEqual(response_atom["scope_disposition"], "unclear")

    def test_response_and_error_message_gaps_do_not_invent_client_actors_or_labels(self) -> None:
        gaps = {row["gap_id"]: row for row in self.oracle["source_only_requirement_gaps"]}
        response_gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-8.4-002"]
        error_gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-9.17-001"]

        for gap in (response_gap, error_gap):
            with self.subTest(gap_id=gap["gap_id"]):
                self.assertEqual(gap["actor"], "unspecified")
                self.assertEqual(gap["direction"], "server_to_client")
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        self.assertEqual(response_gap["scope_disposition"], "server_only")
        self.assertEqual(error_gap["scope_disposition"], "unresolved")
        clauses = {row["clause_id"]: row for row in self.catalog["source_clauses"]}
        for clause_id in ("KMIPKIT-CLAUSE-SPEC-8.4-002", "KMIPKIT-CLAUSE-SPEC-9.17-001"):
            with self.subTest(clause_id=clause_id):
                self.assertEqual(clauses[clause_id]["role"], "unclear")
                self.assertEqual(clauses[clause_id]["direction"], "server_to_client")

        response_review = self.reviews["KMIPKIT-CLAUSE-SPEC-8.4-002"]
        self.assertEqual(len(response_review["source_obligations"]), 1)
        self.assertEqual(response_review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertEqual(response_review["source_obligations"][0]["actor"], "unspecified")
        self.assertEqual(response_review["source_obligations"][0]["direction"], "server_to_client")
        self.assertEqual(response_review["source_obligations"][0]["scope_disposition"], "server_only")
        self.assertIsNone(response_review["source_obligations"][0]["candidate_requirement_id"])

        error_review = self.reviews["KMIPKIT-CLAUSE-SPEC-9.17-001"]
        error_atoms = error_review["source_obligations"]
        self.assertEqual([atom["source_keyword"] for atom in error_atoms], ["MAY", "MAY"])
        self.assertTrue(all(atom["actor"] == "unspecified" for atom in error_atoms))
        self.assertEqual([atom["direction"] for atom in error_atoms], ["server_to_client", "unspecified"])
        self.assertEqual([atom["scope_disposition"] for atom in error_atoms], ["server_only", "unresolved"])
        self.assertTrue(all(atom["candidate_requirement_id"] is None for atom in error_atoms))

    def test_time_stamp_may_atoms_keep_only_explicit_client_actor_mapped(self) -> None:
        review = self.reviews["KMIPKIT-CLAUSE-SPEC-9.20-001"]
        atoms = {atom["source_order"]: atom for atom in review["source_obligations"]}
        ambiguous = atoms[3]
        self.assertEqual(ambiguous["source_keyword"], "MAY")
        self.assertEqual(ambiguous["actor"], "unspecified")
        self.assertEqual(ambiguous["direction"], "unspecified")
        self.assertEqual(ambiguous["scope_disposition"], "unresolved")
        self.assertIsNone(ambiguous["candidate_requirement_id"])

        explicit_client = atoms[5]
        self.assertEqual(explicit_client["actor"], "client")
        self.assertEqual(explicit_client["direction"], "client_to_server")
        self.assertEqual(explicit_client["scope_disposition"], "client_1_0")
        self.assertEqual(explicit_client["candidate_requirement_id"], "KMIPKIT-REQ-SPEC-9.20-001-002")

        crosswalk_orders = [
            row["source_obligation_order"]
            for row in self.oracle["requirement_crosswalk"]
            if row["requirement_id"] == "KMIPKIT-REQ-SPEC-9.20-001-002"
        ]
        self.assertEqual(crosswalk_orders, [5])
        requirement = self.requirements["KMIPKIT-REQ-SPEC-9.20-001-002"]
        self.assertNotIn("enforce reasonable time usage", requirement["summary"])


if __name__ == "__main__":
    unittest.main()
