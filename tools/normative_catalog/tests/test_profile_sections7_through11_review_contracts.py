"""Precise source dispositions for KMIP Specification sections 7 through 11."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog.tests.test_source_gap_context_resolutions import pinned_source_excerpt


ROOT = Path(__file__).resolve().parents[3]
SPEC_SOURCE_PATH = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"


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
        cls.clauses = {row["clause_id"]: row for row in cls.catalog["source_clauses"]}
        cls.gaps = {row["gap_id"]: row for row in cls.oracle["source_only_requirement_gaps"]}

    def assert_source_actor_unspecified(self, actor: object) -> None:
        self.assertEqual(actor, "unspecified")

    def assert_server_response_classification(
        self, clause_id: str, review: dict[str, object], gap: dict[str, object]
    ) -> None:
        clause = self.clauses[clause_id]
        self.assertEqual(clause["disposition"], "server_only")
        self.assertEqual(clause["scope_state"], "server_only")
        self.assertEqual(clause["role"], "unclear")
        self.assertEqual(clause["direction"], "server_to_client")
        self.assertEqual(clause["requirement_ids"], [])

        self.assertEqual(review["expected_client_requirement_ids"], [])
        for atom in review["source_obligations"]:
            self.assertEqual(atom["actor"], "server")
            self.assertEqual(atom["direction"], "server_to_client")
            self.assertEqual(atom["scope_disposition"], "server_only")
            self.assertIsNone(atom["candidate_requirement_id"])

        self.assertEqual(gap["source_clause_id"], clause_id)
        self.assertEqual(gap["source_locator"], clause["locator"])
        self.assertEqual(gap["actor"], "server")
        self.assertEqual(gap["direction"], "server_to_client")
        self.assertEqual(gap["scope_disposition"], "server_only")
        self.assertEqual(gap["status"], "server_actor_disposition_recorded")
        self.assertIsNone(gap["candidate_requirement_id"])
        self.assertIsNone(gap.get("reused_requirement_id"))
        self.assertIsNone(gap["proposed_requirement_id"])

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

    def test_result_message_source_atoms_keep_response_output_and_client_use_separate(self) -> None:
        response_gap = self.gaps["KMIPKIT-SOURCE-GAP-SPEC-8.4-002"]
        error_gap = self.gaps["KMIPKIT-SOURCE-GAP-SPEC-9.17-001"]

        self.assertEqual(error_gap["actor"], "unspecified")
        self.assertEqual(error_gap["direction"], "server_to_client")
        self.assertEqual(error_gap["scope_disposition"], "mixed")
        self.assertIsNone(error_gap["candidate_requirement_id"])
        self.assertIsNone(error_gap["proposed_requirement_id"])
        response_clause = self.clauses["KMIPKIT-CLAUSE-SPEC-8.4-002"]
        self.assertEqual(response_clause["direction"], "server_to_client")

        error_clause = self.clauses["KMIPKIT-CLAUSE-SPEC-9.17-001"]
        self.assertEqual(error_clause["disposition"], "source_discrepancy")
        self.assertEqual(error_clause["scope_state"], "mixed")
        self.assertEqual(error_clause["role"], "unclear")
        self.assertEqual(error_clause["direction"], "server_to_client")
        self.assertEqual(error_clause["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.17-001-002"])
        self.assertIsNone(error_clause["exclusion_rationale"])
        self.assertIn("second MAY is assigned", error_clause["condition"])
        self.assertEqual(error_clause["direction"], "server_to_client")

        response_review = self.reviews["KMIPKIT-CLAUSE-SPEC-8.4-002"]
        self.assert_server_response_classification(
            "KMIPKIT-CLAUSE-SPEC-8.4-002", response_review, response_gap
        )
        self.assertEqual(len(response_review["source_obligations"]), 1)
        self.assertEqual(response_review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertIn("client-initiated 1.0", response_review["review_note"])
        self.assertIn("does not literally name the sender", response_review["review_note"])

        error_review = self.reviews["KMIPKIT-CLAUSE-SPEC-9.17-001"]
        error_atoms = error_review["source_obligations"]
        self.assertEqual([atom["source_keyword"] for atom in error_atoms], ["MAY", "MAY"])
        self.assertEqual([atom["actor"] for atom in error_atoms], ["unspecified", "client"])
        self.assertEqual([atom["direction"] for atom in error_atoms], ["server_to_client", "server_to_client"])
        self.assertEqual([atom["scope_disposition"] for atom in error_atoms], ["server_only", "client_1_0"])
        self.assertIsNone(error_atoms[0]["candidate_requirement_id"])
        self.assertEqual(error_atoms[1]["candidate_requirement_id"], "KMIPKIT-REQ-SPEC-9.17-001-002")
        self.assertEqual(error_review["expected_client_requirement_ids"], ["KMIPKIT-REQ-SPEC-9.17-001-002"])

    def test_response_header_and_batch_rows_are_server_output_only(self) -> None:
        rows = {
            "KMIPKIT-CLAUSE-SPEC-8.5-002": {
                "line_range": [43202, 43221],
                "gap_id": "KMIPKIT-SOURCE-GAP-SPEC-8.5-002",
                "keywords": ["MAY", "REQUIRED"],
                "source_force": "MAY_and_conditional_REQUIRED",
                "conditions": [
                    "MAY be repeated",
                    "client set Attestation Capable Indicator to True",
                ],
                "separate_duty_refs": [],
            },
            "KMIPKIT-CLAUSE-SPEC-8.6-002": {
                "line_range": [43361, 43379],
                "gap_id": "KMIPKIT-SOURCE-GAP-SPEC-8.6-002",
                "keywords": ["REQUIRED"],
                "source_force": "conditional_REQUIRED",
                "conditions": ["REQUIRED if present in Request Batch Item"],
                "separate_duty_refs": ["§8.3"],
            },
            "KMIPKIT-CLAUSE-SPEC-8.6-003": {
                "line_range": [43399, 43419],
                "gap_id": "KMIPKIT-SOURCE-GAP-SPEC-8.6-003",
                "keywords": ["REQUIRED", "OPTIONAL"],
                "source_force": "conditional_REQUIRED_and_OPTIONAL",
                "conditions": ["REQUIRED if Result Status is Failure", "otherwise OPTIONAL"],
                "separate_duty_refs": [],
            },
            "KMIPKIT-CLAUSE-SPEC-8.6-004": {
                "line_range": [43420, 43439],
                "gap_id": "KMIPKIT-SOURCE-GAP-SPEC-8.6-004",
                "keywords": ["OPTIONAL"],
                "source_force": "OPTIONAL",
                "conditions": ["OPTIONAL if Result Status is not Pending or Success"],
                "separate_duty_refs": [],
            },
            "KMIPKIT-CLAUSE-SPEC-8.6-005": {
                "line_range": [43440, 43459],
                "gap_id": "KMIPKIT-SOURCE-GAP-SPEC-8.6-005",
                "keywords": ["REQUIRED"],
                "source_force": "conditional_REQUIRED",
                "conditions": ["REQUIRED if Result Status is Pending"],
                "separate_duty_refs": ["§9.1"],
            },
        }
        self.assertEqual(
            pinned_source_excerpt(SPEC_SOURCE_PATH, [43283, 43284]),
            "Table 398: Response Header Structure",
        )
        self.assertEqual(
            pinned_source_excerpt(SPEC_SOURCE_PATH, [43502, 43503]),
            "Table 399: Response Batch Item Structure",
        )

        for clause_id, expected in rows.items():
            with self.subTest(clause_id=clause_id):
                clause = self.clauses[clause_id]

                review = self.reviews[clause_id]
                self.assertEqual(review["html_line_range"], expected["line_range"])
                self.assertEqual(
                    review["source_excerpt"],
                    pinned_source_excerpt(SPEC_SOURCE_PATH, expected["line_range"]),
                )
                self.assertEqual(review["source_locator"], clause["locator"])
                atoms = review["source_obligations"]
                self.assertEqual([atom["source_keyword"] for atom in atoms], expected["keywords"])
                for condition in expected["conditions"]:
                    self.assertIn(condition, " ".join(atom["condition"] for atom in atoms))

                gap = self.gaps[expected["gap_id"]]
                self.assert_server_response_classification(clause_id, review, gap)
                self.assertEqual(gap["html_line_range"], expected["line_range"])
                self.assertEqual(gap["source_force"], expected["source_force"])
                for source_section in expected["separate_duty_refs"]:
                    self.assertIn(source_section, review["review_note"])

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

    def test_credential_reviews_separate_source_rules_from_approved_client_interpretation(self) -> None:
        device_clause_id = "KMIPKIT-CLAUSE-SPEC-9.11-004"
        device_review = self.reviews[device_clause_id]
        device_excerpt = [44184, 44189]
        self.assertEqual(
            device_review["source_excerpt"],
            pinned_source_excerpt(SPEC_SOURCE_PATH, device_excerpt),
        )
        device_atoms = device_review["source_obligations"]
        uniqueness = next(
            atom for atom in device_atoms
            if atom["candidate_requirement_id"] == "KMIPKIT-REQ-SPEC-9.11-004-002"
        )
        minimum = next(
            atom for atom in device_atoms
            if atom["candidate_requirement_id"] == "KMIPKIT-REQ-SPEC-9.11-004-001"
        )
        self.assertEqual(
            uniqueness["source_text"],
            "One or a combination of the Device Serial Number, Network Identifier, Machine Identifier, and Media Identifier SHALL be unique.",
        )
        self.assert_source_actor_unspecified(uniqueness["actor"])
        self.assertEqual(minimum["source_text"], "The client SHALL provide at least one field.")
        self.assertEqual(minimum["direction"], "client provides at least one Device Credential field")
        self.assertIn("KMIPKIT-0008", device_review["review_note"])
        self.assertIn("KMIPKIT-DISC-042", device_review["review_note"])
        self.assertIn("comparison scope is unspecified", device_review["review_note"])
        self.assertIn("caller remains responsible for actual uniqueness", device_review["review_note"])

        timestamp_clause_id = "KMIPKIT-CLAUSE-SPEC-9.11-010"
        timestamp_review = self.reviews[timestamp_clause_id]
        self.assertEqual(
            timestamp_review["source_excerpt"],
            pinned_source_excerpt(SPEC_SOURCE_PATH, [44598, 44602]),
        )
        for atom in timestamp_review["source_obligations"]:
            self.assert_source_actor_unspecified(atom["actor"])
        self.assertEqual(
            self.clauses[timestamp_clause_id]["condition"],
            "When Credential Type is Hashed Password.",
        )
        timestamp_requirement = self.requirements["KMIPKIT-REQ-SPEC-9.11-010-001"]
        self.assertEqual(timestamp_requirement["condition"], "When Credential Type is Hashed Password.")
        for source_gap in ("comparison scope", "state owner", "clock behavior"):
            self.assertIn(source_gap, timestamp_review["review_note"])
        self.assertIn("OD-003 is resolved", timestamp_review["review_note"])
        self.assertIn("strict increase in hash-production order", timestamp_review["review_note"])
        self.assertIn("strict", timestamp_requirement["review_note"])

        default_requirement = self.requirements["KMIPKIT-REQ-SPEC-9.11-010-002"]
        self.assertIn("no override", default_requirement["condition"])
        for default_fact in ("Table 415", "optional", "SHA-256", "omitted"):
            self.assertIn(default_fact.casefold(), timestamp_review["review_note"].casefold())

    def test_credential_crosswalk_qualifies_inferred_source_actors(self) -> None:
        crosswalk = {
            row["requirement_id"]: row
            for row in self.oracle["requirement_crosswalk"]
        }
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-9.11-004-002",
            "KMIPKIT-REQ-SPEC-9.11-010-001",
            "KMIPKIT-REQ-SPEC-9.11-010-002",
        ):
            with self.subTest(requirement_id=requirement_id):
                row = crosswalk[requirement_id]
                self.assert_source_actor_unspecified(row["source_actor"])
                self.assertIn("KMIPKIT-0008", row["review_note"])
                self.assertEqual(row["role"], "client")


if __name__ == "__main__":
    unittest.main()
