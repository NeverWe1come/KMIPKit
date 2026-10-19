"""Regression checks for T002's final source-applicability dispositions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = REPO_ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = REPO_ROOT / "specification/catalog/kmip-2.1.json"

SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-4.11-001"
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

        mismatches = []
        for requirement_id, expected_scope in FINAL_DISPOSITIONS.items():
            crosswalk = self.crosswalk[requirement_id]
            requirement = self.requirements[requirement_id]
            expected_role = "server" if requirement_id == SERVER_ONLY_REQUIREMENT else "client"
            actual = (
                crosswalk["scope_state"],
                requirement["scope_state"],
                requirement["profile_ids"],
                crosswalk["role"],
                requirement["role"],
            )
            expected = (expected_scope, expected_scope, [], expected_role, expected_role)
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

    def test_actor_interpretation_and_specific_source_resolutions_are_recorded(self) -> None:
        missing_project_rationale = [
            requirement_id
            for requirement_id in CONDITIONAL_REQUIREMENTS
            if "project" not in self.crosswalk[requirement_id]["review_note"].casefold()
            or "interpretation" not in self.crosswalk[requirement_id]["review_note"].casefold()
        ]
        self.assertEqual(missing_project_rationale, [])

        deactivation_rows = [
            self.crosswalk[requirement_id]
            for requirement_id in (
                "KMIPKIT-REQ-SPEC-4.18-001-001",
                "KMIPKIT-REQ-SPEC-4.18-001-002",
            )
        ]
        self.assertTrue(all("deletable by client: no" in row["review_note"].casefold()
                            for row in deactivation_rows))
        self.assertTrue(all("pre-active" in row["review_note"].casefold()
                            and "active" in row["review_note"].casefold()
                            for row in deactivation_rows))

        modify_attribute = self.crosswalk["KMIPKIT-REQ-SPEC-6.1.34-001-003"]
        self.assertEqual(modify_attribute["source_section"], "6.1.34")
        self.assertEqual(modify_attribute["source_obligation_order"], 4)
        self.assertEqual(modify_attribute["source_direction"], "client_to_server")
        self.assertEqual(
            modify_attribute["scope_state"], "profile_conditional"
        )


if __name__ == "__main__":
    unittest.main()
