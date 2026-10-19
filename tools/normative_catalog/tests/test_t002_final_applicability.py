"""Regression checks for T002's final source-applicability dispositions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = REPO_ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = REPO_ROOT / "specification/catalog/kmip-2.1.json"

SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-4.11-001"
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
    "KMIPKIT-REQ-SPEC-4.34-001-002",
    "KMIPKIT-REQ-SPEC-4.38-003",
    "KMIPKIT-REQ-SPEC-4.40-001-001",
    "KMIPKIT-REQ-SPEC-4.40-001-002",
    "KMIPKIT-REQ-SPEC-4.40-001-003",
    "KMIPKIT-REQ-SPEC-4.40-001-004",
    "KMIPKIT-REQ-SPEC-4.40-001-005",
    "KMIPKIT-REQ-SPEC-4.41-001-001",
    "KMIPKIT-REQ-SPEC-4.41-001-002",
    "KMIPKIT-REQ-SPEC-4.41-001-003",
    "KMIPKIT-REQ-SPEC-4.41-001-004",
    "KMIPKIT-REQ-SPEC-4.59-002",
    "KMIPKIT-REQ-SPEC-4.59-003-001",
    "KMIPKIT-REQ-SPEC-6.1.34-001-003",
}
FINAL_DISPOSITIONS = {
    **{requirement_id: "profile_conditional" for requirement_id in CONDITIONAL_REQUIREMENTS},
    SERVER_ONLY_REQUIREMENT: "server_only",
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

    def test_final_twenty_have_adjudicated_crosswalk_and_catalog_scope(self) -> None:
        self.assertEqual(len(FINAL_DISPOSITIONS), 20)
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
            expected_role = "server" if requirement_id == SERVER_ONLY_REQUIREMENT else "client"
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
                if requirement_id in BASELINE_SELECTED_ATTRIBUTE_REQUIREMENTS
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

    def test_client_actor_interpretation_is_explicit_for_conditional_rows(self) -> None:
        missing_project_rationale = [
            requirement_id
            for requirement_id in CONDITIONAL_REQUIREMENTS
            if "project" not in self.crosswalk[requirement_id]["review_note"].casefold()
            or "interpretation" not in self.crosswalk[requirement_id]["review_note"].casefold()
        ]
        self.assertEqual(missing_project_rationale, [])

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


if __name__ == "__main__":
    unittest.main()
