"""Regression checks for T002's final source-applicability dispositions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = REPO_ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = REPO_ROOT / "specification/catalog/kmip-2.1.json"

SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-4.11-001"
PRE_ACTIVE_SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-4.57-002"
STATE_SERVER_ONLY_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-4.57-003-001",
    "KMIPKIT-REQ-SPEC-4.57-003-002",
    "KMIPKIT-REQ-SPEC-4.57-003-003",
    "KMIPKIT-REQ-SPEC-4.57-003-004",
    "KMIPKIT-REQ-SPEC-4.57-004-001",
    "KMIPKIT-REQ-SPEC-4.57-004-002",
    "KMIPKIT-REQ-SPEC-4.57-004-003",
    "KMIPKIT-REQ-SPEC-4.57-005-001",
    "KMIPKIT-REQ-SPEC-4.57-005-003",
    "KMIPKIT-REQ-SPEC-4.57-006",
    "KMIPKIT-REQ-SPEC-4.57-007",
    "KMIPKIT-REQ-SPEC-4.57-007-002",
}
BASELINE_SELECTED_ATTRIBUTE_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-4.1-001-001",
    "KMIPKIT-REQ-SPEC-4.1-001-002",
    "KMIPKIT-REQ-SPEC-4.1-001-003",
    "KMIPKIT-REQ-SPEC-4.18-001-001",
    "KMIPKIT-REQ-SPEC-4.18-001-002",
}
CONDITIONAL_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-4.1-001-001",
    "KMIPKIT-REQ-SPEC-4.1-001-002",
    "KMIPKIT-REQ-SPEC-4.1-001-003",
    "KMIPKIT-REQ-SPEC-4.18-001-001",
    "KMIPKIT-REQ-SPEC-4.18-001-002",
    "KMIPKIT-REQ-SPEC-4.38-003",
    "KMIPKIT-REQ-SPEC-4.59-002",
    "KMIPKIT-REQ-SPEC-4.59-003-001",
    "KMIPKIT-REQ-SPEC-6.1.34-001-003",
}
CLIENT_SCOPE_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-4.34-001-002",
    "KMIPKIT-REQ-SPEC-4.40-001-001",
    "KMIPKIT-REQ-SPEC-4.40-001-002",
    "KMIPKIT-REQ-SPEC-4.40-001-003",
    "KMIPKIT-REQ-SPEC-4.40-001-004",
    "KMIPKIT-REQ-SPEC-4.40-001-005",
    "KMIPKIT-REQ-SPEC-4.41-001-001",
    "KMIPKIT-REQ-SPEC-4.41-001-002",
    "KMIPKIT-REQ-SPEC-4.41-001-003",
    "KMIPKIT-REQ-SPEC-4.41-001-004",
    "KMIPKIT-REQ-SPEC-4.57-001",
    "KMIPKIT-REQ-SPEC-8-001",
    "KMIPKIT-REQ-SPEC-8-002",
    "KMIPKIT-REQ-SPEC-9.7-001",
}
MESSAGE_ENVELOPE_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-8-001": "KMIPKIT-CLAUSE-SPEC-8-001",
    "KMIPKIT-REQ-SPEC-8-002": "KMIPKIT-CLAUSE-SPEC-8-002",
    "KMIPKIT-REQ-SPEC-9.7-001": "KMIPKIT-CLAUSE-SPEC-9.7-001",
}
ALL_CLIENT_PROFILE_SCOPE_REQUIREMENTS = (
    BASELINE_SELECTED_ATTRIBUTE_REQUIREMENTS | CLIENT_SCOPE_REQUIREMENTS
)
FINAL_DISPOSITIONS = {
    **{requirement_id: "profile_conditional" for requirement_id in CONDITIONAL_REQUIREMENTS},
    **{requirement_id: "client_1_0" for requirement_id in CLIENT_SCOPE_REQUIREMENTS},
    SERVER_ONLY_REQUIREMENT: "server_only",
    PRE_ACTIVE_SERVER_ONLY_REQUIREMENT: "server_only",
    **{requirement_id: "server_only" for requirement_id in STATE_SERVER_ONLY_REQUIREMENTS},
}
SERVER_ONLY_REQUIREMENTS = {
    SERVER_ONLY_REQUIREMENT,
    PRE_ACTIVE_SERVER_ONLY_REQUIREMENT,
    *STATE_SERVER_ONLY_REQUIREMENTS,
}
TTLV_CLIENT_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-10.1.2-001",
    "KMIPKIT-REQ-SPEC-10.1.2-002-001",
    "KMIPKIT-REQ-SPEC-10.1.2-002-002",
    "KMIPKIT-REQ-SPEC-10.1.5-001-001",
    "KMIPKIT-REQ-SPEC-10.1.5-001-002",
    "KMIPKIT-REQ-SPEC-11.56-001",
}
PROFILE_VARIATION_REQUIREMENT = "KMIPKIT-REQ-PROF-4.1.2-001"
T040_BASELINE_PROFILE_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"
T040_REMAINING_APPLICABILITY = {
    "KMIPKIT-REQ-PROF-4.1.2-005": ("profile_conditional", "client", "client_to_server", "all_client_profiles"),
    "KMIPKIT-REQ-SPEC-4-001-001": ("mixed", "client", "both", "all_client_profiles"),
    "KMIPKIT-REQ-SPEC-4-001-002": ("mixed", "client", "both", "all_client_profiles"),
    "KMIPKIT-REQ-SPEC-4-001-003": ("mixed", "client", "both", "all_client_profiles"),
    "KMIPKIT-REQ-SPEC-4.38-003": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-4.57-005-002": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-4.59-002": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-4.59-003-001": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-6.1.14-002": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-6.1.34-001-003": ("profile_conditional", "client", "client_to_server", "baseline_client"),
    "KMIPKIT-REQ-SPEC-7.31-001": ("profile_conditional", "client", "server_to_client", "all_client_profiles"),
    "KMIPKIT-REQ-SPEC-7.32-001": ("profile_conditional", "client", "server_to_client", "all_client_profiles"),
}


def _load_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as source_file:
        return json.load(source_file)


class T002FinalApplicabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.oracle = _load_json(ORACLE_PATH)
        cls.catalog = _load_json(CATALOG_PATH)
        cls.crosswalk = {
            row["requirement_id"]: row for row in cls.oracle["requirement_crosswalk"]
        }
        cls.requirements = {
            row["requirement_id"]: row for row in cls.catalog["requirements"]
        }

    def test_final_scope_rows_have_adjudicated_crosswalk_and_catalog_scope(self) -> None:
        self.assertEqual(len(FINAL_DISPOSITIONS), 37)
        self.assertTrue(FINAL_DISPOSITIONS.keys() <= self.crosswalk.keys())
        self.assertTrue(FINAL_DISPOSITIONS.keys() <= self.requirements.keys())
        self.assertEqual(
            [row["requirement_id"] for row in self.oracle["requirement_crosswalk"]
             if row["scope_state"] == "unresolved"],
            [],
        )
        client_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.oracle["profiles"]
            if profile["role"] == "client"
        )

        mismatches = []
        for requirement_id, expected_scope in FINAL_DISPOSITIONS.items():
            crosswalk = self.crosswalk[requirement_id]
            requirement = self.requirements[requirement_id]
            expected_role = "server" if requirement_id in SERVER_ONLY_REQUIREMENTS else "client"
            expected_profiles = (
                client_profile_ids
                if requirement_id in BASELINE_SELECTED_ATTRIBUTE_REQUIREMENTS
                else []
            )
            actual = (
                crosswalk["scope_state"],
                crosswalk["applies_to"],
                requirement["scope_state"],
                requirement["profile_ids"],
                crosswalk["role"],
                requirement["role"],
            )
            expected_applies_to = (
                "all_client_profiles"
                if requirement_id in ALL_CLIENT_PROFILE_SCOPE_REQUIREMENTS
                else []
            )
            expected = (
                expected_scope,
                expected_applies_to,
                expected_scope,
                expected_profiles,
                expected_role,
                expected_role,
            )
            if actual != expected:
                mismatches.append((requirement_id, expected, actual))
        self.assertEqual(mismatches, [])

    def test_source_obligation_review_matches_adjudicated_scope(self) -> None:
        obligations = {
            obligation.get("candidate_requirement_id"): obligation
            for review in self.oracle["source_requirement_reviews"]
            for obligation in review["source_obligations"]
            if obligation.get("candidate_requirement_id") in FINAL_DISPOSITIONS
        }
        self.assertEqual(set(obligations), set(FINAL_DISPOSITIONS))

        mismatches = [
            (requirement_id, expected_scope, obligations[requirement_id]["scope_disposition"])
            for requirement_id, expected_scope in FINAL_DISPOSITIONS.items()
            if obligations[requirement_id]["scope_disposition"] != expected_scope
        ]
        self.assertEqual(mismatches, [])

        actor_mismatches = []
        for requirement_id, expected_scope in FINAL_DISPOSITIONS.items():
            expected_interpretation = "server" if expected_scope == "server_only" else "client"
            obligation = obligations[requirement_id]
            crosswalk = self.crosswalk[requirement_id]
            if (
                obligation.get("project_actor_interpretation") != expected_interpretation
                or obligation["actor"] != crosswalk["source_actor"]
            ):
                actor_mismatches.append((
                    requirement_id,
                    expected_interpretation,
                    obligation.get("project_actor_interpretation"),
                    crosswalk["source_actor"],
                    obligation["actor"],
                ))
        self.assertEqual(actor_mismatches, [])

    def test_client_actor_interpretation_is_explicit_for_adjudicated_client_rows(self) -> None:
        client_rows = CONDITIONAL_REQUIREMENTS | CLIENT_SCOPE_REQUIREMENTS
        source_obligations = {
            obligation.get("candidate_requirement_id"): obligation
            for review in self.oracle["source_requirement_reviews"]
            for obligation in review["source_obligations"]
            if obligation.get("candidate_requirement_id") in client_rows
        }
        missing_project_rationale = [
            requirement_id
            for requirement_id in client_rows
            if "project" not in self.crosswalk[requirement_id]["review_note"].casefold()
            or "interpretation" not in self.crosswalk[requirement_id]["review_note"].casefold()
        ]
        self.assertEqual(missing_project_rationale, [])
        for requirement_id in CLIENT_SCOPE_REQUIREMENTS:
            with self.subTest(requirement_id=requirement_id):
                expected_direction = (
                    "client_to_server"
                    if requirement_id == "KMIPKIT-REQ-SPEC-4.57-001"
                    else "both"
                    if requirement_id in MESSAGE_ENVELOPE_REQUIREMENTS
                    else "unclear"
                )
                expected_source_actor = (
                    "unspecified"
                    if requirement_id in MESSAGE_ENVELOPE_REQUIREMENTS
                    else "unresolved"
                )
                self.assertEqual(
                    self.crosswalk[requirement_id]["source_actor"], expected_source_actor
                )
                self.assertEqual(self.requirements[requirement_id]["direction"], expected_direction)
                self.assertEqual(self.crosswalk[requirement_id]["role"], "client")
                self.assertEqual(
                    source_obligations[requirement_id]["project_actor_interpretation"],
                    "client",
                )

    def test_message_envelope_rules_apply_to_client_requests_and_responses(self) -> None:
        """The client profile interpretation covers both message directions, not server behavior."""
        all_client_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.oracle["profiles"]
            if profile["role"] == "client"
        )
        catalog_profiles = {
            profile["profile_id"]: profile for profile in self.catalog["profiles"]
        }
        oracle_profiles = {
            profile["profile_id"]: profile for profile in self.oracle["profiles"]
        }
        reviews = {
            review["source_clause_id"]: review
            for review in self.oracle["source_requirement_reviews"]
        }

        for requirement_id, clause_id in MESSAGE_ENVELOPE_REQUIREMENTS.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                crosswalk = self.crosswalk[requirement_id]
                review = reviews[clause_id]
                obligations = [
                    obligation
                    for obligation in review["source_obligations"]
                    if obligation.get("candidate_requirement_id") == requirement_id
                ]

                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "both")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["test_case_ids"], [])
                self.assertEqual(crosswalk["role"], "client")
                self.assertEqual(crosswalk["scope_state"], "client_1_0")
                self.assertEqual(crosswalk["applies_to"], "all_client_profiles")
                self.assertEqual(crosswalk["source_actor"], "unspecified")
                self.assertEqual(crosswalk["source_direction"], "unspecified")
                self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
                self.assertEqual(review["applies_to"], "all_client_profiles")
                self.assertTrue(obligations)
                self.assertTrue(all(item["actor"] == "unspecified" for item in obligations))
                self.assertTrue(all(item["direction"] == "unspecified" for item in obligations))
                self.assertTrue(
                    all(item["scope_disposition"] == "client_1_0" for item in obligations)
                )
                self.assertTrue(
                    all(
                        item["project_actor_interpretation"] == "client"
                        for item in obligations
                    )
                )
                note = crosswalk["review_note"].casefold()
                self.assertIn("project interpretation", note)
                self.assertIn("request", note)
                self.assertIn("response", note)
                self.assertIn("does not assert server behavior", note)

                for profile_id in all_client_profile_ids:
                    self.assertIn(requirement_id, catalog_profiles[profile_id]["requirement_ids"])
                    self.assertIn(requirement_id, oracle_profiles[profile_id]["requirement_ids"])

    def test_state_attribute_modify_prohibition_is_client_scope(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-4.57-001"
        requirement = self.requirements[requirement_id]
        crosswalk = self.crosswalk[requirement_id]
        obligation = next(
            obligation
            for review in self.oracle["source_requirement_reviews"]
            for obligation in review["source_obligations"]
            if obligation.get("candidate_requirement_id") == requirement_id
        )

        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(crosswalk["scope_state"], "client_1_0")
        self.assertEqual(crosswalk["applies_to"], "all_client_profiles")
        self.assertEqual(crosswalk["source_actor"], "unresolved")
        self.assertEqual(crosswalk["source_direction"], "client_to_server")
        self.assertEqual(obligation["actor"], "unresolved")
        self.assertEqual(obligation["scope_disposition"], "client_1_0")
        self.assertEqual(obligation["project_actor_interpretation"], "client")

    def test_pre_active_use_prohibition_is_server_only(self) -> None:
        requirement_id = PRE_ACTIVE_SERVER_ONLY_REQUIREMENT
        clause_id = "KMIPKIT-CLAUSE-SPEC-4.57-002"
        requirement = self.requirements[requirement_id]
        clause = next(
            row for row in self.catalog["source_clauses"] if row["clause_id"] == clause_id
        )
        crosswalk = self.crosswalk[requirement_id]
        obligation = next(
            obligation
            for review in self.oracle["source_requirement_reviews"]
            for obligation in review["source_obligations"]
            if obligation.get("candidate_requirement_id") == requirement_id
        )
        decision = next(
            (
                row
                for row in self.catalog["decisions"]
                if row["decision_id"] == "KMIPKIT-DEC-020"
            ),
            None,
        )
        self.assertIsNotNone(decision, "the server-only interpretation needs a decision record")
        discrepancy = next(
            row
            for row in self.catalog["discrepancies"]
            if row["discrepancy_id"] == "KMIPKIT-DISC-013"
        )

        self.assertEqual(clause["scope_state"], "server_only")
        self.assertEqual(clause["role"], "server")
        self.assertEqual(clause["requirement_ids"], [requirement_id])
        self.assertIsNone(clause["exclusion_rationale"])
        self.assertEqual(requirement["role"], "server")
        self.assertEqual(requirement["scope_state"], "server_only")
        self.assertEqual(requirement["direction"], "unclear")
        self.assertEqual(requirement["profile_ids"], [])
        self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-020")
        self.assertEqual(crosswalk["role"], "server")
        self.assertEqual(crosswalk["scope_state"], "server_only")
        self.assertEqual(crosswalk["applies_to"], [])
        self.assertEqual(crosswalk["source_actor"], "unresolved")
        self.assertEqual(crosswalk["source_direction"], "unclear")
        self.assertEqual(obligation["actor"], "unresolved")
        self.assertEqual(obligation["scope_disposition"], "server_only")
        self.assertEqual(obligation["project_actor_interpretation"], "server")
        self.assertEqual(obligation["candidate_requirement_id"], requirement_id)
        self.assertEqual(decision["status"], "accepted")
        self.assertIn(requirement_id, decision["requirement_ids"])
        self.assertNotIn(requirement_id, discrepancy["affected_requirement_ids"])

        for profile in self.catalog["profiles"]:
            if profile["role"] == "client":
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertNotIn(requirement_id, profile["requirement_ids"])
        for profile in self.oracle["profiles"]:
            if profile["role"] == "client":
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertNotIn(requirement_id, profile["requirement_ids"])

    def test_state_use_restrictions_follow_server_owned_object_state(self) -> None:
        decision = next(
            (
                row
                for row in self.catalog["decisions"]
                if row["decision_id"] == "KMIPKIT-DEC-021"
            ),
            None,
        )
        self.assertIsNotNone(decision, "the server-owned state interpretation needs a decision")
        self.assertEqual(set(decision["requirement_ids"]), STATE_SERVER_ONLY_REQUIREMENTS)

        missing_requirements = STATE_SERVER_ONLY_REQUIREMENTS - self.requirements.keys()
        self.assertEqual(missing_requirements, set())
        missing_crosswalk = STATE_SERVER_ONLY_REQUIREMENTS - self.crosswalk.keys()
        self.assertEqual(missing_crosswalk, set())
        if missing_requirements or missing_crosswalk:
            return

        source_obligations = {
            obligation.get("candidate_requirement_id"): obligation
            for review in self.oracle["source_requirement_reviews"]
            for obligation in review["source_obligations"]
            if obligation.get("candidate_requirement_id") in STATE_SERVER_ONLY_REQUIREMENTS
        }
        self.assertEqual(set(source_obligations), STATE_SERVER_ONLY_REQUIREMENTS)
        for requirement_id in STATE_SERVER_ONLY_REQUIREMENTS:
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                crosswalk = self.crosswalk[requirement_id]
                obligation = source_obligations[requirement_id]
                self.assertEqual(requirement["role"], "server")
                self.assertEqual(requirement["scope_state"], "server_only")
                self.assertEqual(requirement["direction"], "unclear")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-021")
                self.assertEqual(crosswalk["role"], "server")
                self.assertEqual(crosswalk["scope_state"], "server_only")
                self.assertEqual(crosswalk["applies_to"], [])
                self.assertEqual(crosswalk["source_actor"], "unresolved")
                self.assertEqual(crosswalk["source_direction"], "unclear")
                self.assertEqual(obligation["actor"], "unresolved")
                self.assertEqual(obligation["scope_disposition"], "server_only")
                self.assertEqual(obligation["project_actor_interpretation"], "server")
                self.assertIn("project interpretation", crosswalk["review_note"].lower())

        for profile in self.catalog["profiles"]:
            if profile["role"] == "client":
                for requirement_id in STATE_SERVER_ONLY_REQUIREMENTS:
                    clause_id = self.requirements[requirement_id]["source_clause_ids"][0]
                    self.assertIn(clause_id, profile["source_clause_ids"])
                    self.assertNotIn(requirement_id, profile["requirement_ids"])
        for profile in self.oracle["profiles"]:
            if profile["role"] == "client":
                for requirement_id in STATE_SERVER_ONLY_REQUIREMENTS:
                    clause_id = self.requirements[requirement_id]["source_clause_ids"][0]
                    self.assertIn(clause_id, profile["source_clause_ids"])
                    self.assertNotIn(requirement_id, profile["requirement_ids"])

    def test_deactivation_date_can_only_be_modified_in_named_states_and_never_deleted(self) -> None:
        use_after_deactivation = self.crosswalk["KMIPKIT-REQ-SPEC-4.18-001-001"]
        self.assertIn(
            "Pinned OASIS KMIP Specification v2.1",
            use_after_deactivation["review_note"],
        )
        self.assertIn("extraordinary", use_after_deactivation["review_note"].casefold())
        self.assertIn("special permission", use_after_deactivation["review_note"].casefold())

        deactivation_date_mutation = self.crosswalk["KMIPKIT-REQ-SPEC-4.18-001-002"]
        mutation_note = deactivation_date_mutation["review_note"].casefold()
        self.assertIn("deletable by client: no", mutation_note)
        self.assertIn("table 64", mutation_note)
        self.assertNotIn("table 63", mutation_note)
        self.assertIn("pre-active", mutation_note)
        self.assertIn("active", mutation_note)

    def test_modify_attribute_keeps_existing_id_and_client_request_context(self) -> None:
        modify_attribute = self.crosswalk["KMIPKIT-REQ-SPEC-6.1.34-001-003"]
        self.assertEqual(modify_attribute["source_section"], "6.1.34")
        self.assertEqual(modify_attribute["source_obligation_order"], 4)
        self.assertEqual(
            self.requirements["KMIPKIT-REQ-SPEC-6.1.34-001-003"]["direction"],
            "client_to_server",
        )
        modify_review = next(
            review for review in self.oracle["source_requirement_reviews"]
            if review["source_section"] == "6.1.34"
        )
        modify_candidate_ids = {
            obligation["candidate_requirement_id"]
            for obligation in modify_review["source_obligations"]
            if obligation["candidate_requirement_id"] is not None
        }
        self.assertIn("KMIPKIT-REQ-SPEC-6.1.34-001-003", modify_candidate_ids)
        self.assertFalse(
            any("6.1.34" in requirement_id and requirement_id.endswith("001-005")
                for requirement_id in self.requirements)
        )
        self.assertEqual(
            modify_attribute["scope_state"], "profile_conditional"
        )

    def test_normative_ttlv_rules_map_to_each_in_scope_client_profile(self) -> None:
        """KMIP v2.1 TTLV encoding rules apply to every in-scope TTLV client profile."""
        applicable_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.oracle["profiles"]
            if profile["role"] == "client"
            and profile["applicability"] in {"client_1_0", "conditional"}
        )
        self.assertEqual(len(applicable_profile_ids), 15)

        reviews = {
            review["source_clause_id"]: review
            for review in self.oracle["source_requirement_reviews"]
        }
        for requirement_id in TTLV_CLIENT_REQUIREMENTS:
            requirement = self.requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            review = reviews[clause_id]
            obligation = next(
                item
                for item in review["source_obligations"]
                if item["candidate_requirement_id"] == requirement_id
            )
            crosswalk = self.crosswalk[requirement_id]

            self.assertEqual(requirement["scope_state"], "profile_conditional")
            self.assertEqual(requirement["role"], "client")
            self.assertEqual(requirement["direction"], "client_to_server")
            self.assertEqual(requirement["profile_ids"], applicable_profile_ids)
            self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-018")
            self.assertIn(requirement_id, review["expected_client_requirement_ids"])
            self.assertEqual(review["applies_to"], applicable_profile_ids)
            self.assertEqual(obligation["scope_disposition"], "profile_conditional")
            self.assertEqual(obligation["project_actor_interpretation"], "client")
            self.assertEqual(crosswalk["source_actor"], obligation["actor"])
            self.assertEqual(crosswalk["source_direction"], obligation["direction"])
            self.assertEqual(obligation["actor"], "unspecified")
            self.assertEqual(obligation["direction"], "unspecified")
            self.assertEqual(crosswalk["scope_state"], "profile_conditional")
            self.assertEqual(crosswalk["applies_to"], applicable_profile_ids)

    def test_profile_section_4_1_2_is_a_shared_claimed_profile_evidence_gate(self) -> None:
        """The Profile SHALL is conditional on a profile claim, not a new runtime rule."""
        all_client_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.oracle["profiles"]
            if profile["role"] == "client"
        )
        requirement = self.requirements[PROFILE_VARIATION_REQUIREMENT]
        crosswalk = self.crosswalk[PROFILE_VARIATION_REQUIREMENT]
        review = next(
            item
            for item in self.oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-PROF-4.1.2-001"
        )
        obligation = review["source_obligations"][0]

        self.assertEqual(requirement["scope_state"], "profile_conditional")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "not_applicable")
        self.assertEqual(requirement["profile_ids"], all_client_profile_ids)
        self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-018")
        self.assertEqual(review["expected_client_requirement_ids"], [PROFILE_VARIATION_REQUIREMENT])
        self.assertEqual(review["applies_to"], "all_client_profiles")
        self.assertEqual(obligation["scope_disposition"], "profile_conditional")
        self.assertEqual(obligation["project_actor_interpretation"], "client")
        self.assertEqual(crosswalk["source_actor"], obligation["actor"])
        self.assertEqual(crosswalk["source_direction"], obligation["direction"])
        self.assertEqual(obligation["actor"], "unresolved")
        self.assertEqual(obligation["direction"], "unresolved")
        self.assertEqual(crosswalk["scope_state"], "profile_conditional")
        self.assertEqual(crosswalk["applies_to"], "all_client_profiles")
        self.assertIn("shared", crosswalk["review_note"].casefold())
        self.assertIn("evidence", crosswalk["review_note"].casefold())

    def test_t040_remaining_applicability_rows_have_adjudicated_crosswalks(self) -> None:
        """Resolve the final Baseline Client source-scope rows without rewriting OASIS facts."""
        all_client_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.oracle["profiles"]
            if profile["role"] == "client"
        )
        source_reviews = {
            review["source_clause_id"]: review
            for review in self.oracle["source_requirement_reviews"]
        }
        accepted_decisions = {
            decision["decision_id"]: decision
            for decision in self.catalog["decisions"]
            if decision["status"] == "accepted"
        }
        catalog_profiles = {
            profile["profile_id"]: profile for profile in self.catalog["profiles"]
        }

        self.assertEqual(len(T040_REMAINING_APPLICABILITY), 12)
        for requirement_id, expected in T040_REMAINING_APPLICABILITY.items():
            with self.subTest(requirement_id=requirement_id):
                expected_scope, expected_role, expected_direction, target = expected
                requirement = self.requirements[requirement_id]
                crosswalk = self.crosswalk[requirement_id]
                clause_id = requirement["source_clause_ids"][0]
                review = source_reviews[clause_id]
                obligation = next(
                    item
                    for item in review["source_obligations"]
                    if item["candidate_requirement_id"] == requirement_id
                )
                expected_profiles = (
                    all_client_profile_ids
                    if target == "all_client_profiles"
                    else [T040_BASELINE_PROFILE_ID]
                )
                expected_applies_to = (
                    "all_client_profiles"
                    if target == "all_client_profiles"
                    else expected_profiles
                )

                self.assertEqual(requirement["scope_state"], expected_scope)
                self.assertEqual(requirement["role"], expected_role)
                self.assertEqual(requirement["direction"], expected_direction)
                self.assertEqual(requirement["profile_ids"], expected_profiles)
                self.assertEqual(requirement["decision_id"], crosswalk["decision_id"])
                self.assertEqual(crosswalk["scope_state"], expected_scope)
                self.assertEqual(crosswalk["role"], expected_role)
                self.assertEqual(crosswalk["applies_to"], expected_applies_to)
                self.assertEqual(crosswalk["source_actor"], obligation["actor"])
                self.assertEqual(crosswalk["source_direction"], obligation["direction"])
                self.assertEqual(obligation["scope_disposition"], expected_scope)
                self.assertEqual(obligation["project_actor_interpretation"], "client")
                self.assertIn(requirement_id, review["expected_client_requirement_ids"])
                self.assertEqual(review["applies_to"], expected_applies_to)

                decision = accepted_decisions[requirement["decision_id"]]
                self.assertIn(requirement_id, decision["requirement_ids"])
                self.assertIn("KMIP Specification v2.1", decision["interpretation"])
                self.assertIn(T040_BASELINE_PROFILE_ID, catalog_profiles)
                self.assertIn("project", crosswalk["review_note"].casefold())


if __name__ == "__main__":
    unittest.main()
