"""Table-requiredness mappings remain separate from formal prose obligations."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"


def _read(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


class AttributeScopeAdjudicationTests(unittest.TestCase):
    def test_reused_table_fields_are_in_the_reviewed_profile_membership(self) -> None:
        oracle = _read(ORACLE_PATH)
        reviews = {row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]}

        self.assertIn(
            "KMIPKIT-REQ-SPEC-6.1.12-003",
            reviews["KMIPKIT-CLAUSE-SPEC-6.1.12-003"]["expected_client_requirement_ids"],
        )
        derive_key_members = reviews["KMIPKIT-CLAUSE-SPEC-6.1.14-006"]["expected_client_requirement_ids"]
        self.assertIn("KMIPKIT-REQ-SPEC-6.1.14-010", derive_key_members)
        self.assertNotIn("KMIPKIT-REQ-SPEC-6.1.14-002", derive_key_members)

    def test_derive_key_attributes_requiredness_has_a_table_gap_crosswalk(self) -> None:
        catalog = _read(CATALOG_PATH)
        oracle = _read(ORACLE_PATH)
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.14-007-001"
        requirement = next(row for row in catalog["requirements"] if row["requirement_id"] == requirement_id)
        crosswalk = [row for row in oracle["requirement_crosswalk"] if row["requirement_id"] == requirement_id]

        self.assertEqual(requirement["source_clause_ids"], [])
        self.assertIsNone(requirement["source_keyword"])
        self.assertEqual(requirement["source_force"], "required_table_body_value")
        self.assertEqual(requirement["source_gap_id"], "KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED")
        self.assertTrue(any(
            row.get("source_gap_id") == requirement["source_gap_id"]
            and row.get("source_force") == "required_table_body_value"
            and row.get("source_locator", {}).get("html_line_range") == [25980, 26001]
            for row in crosswalk
        ))

    def test_derive_key_method_multiplicity_uses_the_accepted_client_interpretation(self) -> None:
        catalog = _read(CATALOG_PATH)
        oracle = _read(ORACLE_PATH)
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.14-002"
        requirement = next(row for row in catalog["requirements"] if row["requirement_id"] == requirement_id)
        review = next(
            row for row in oracle["source_requirement_reviews"]
            if row["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-6.1.14-002"
        )

        crosswalk = next(
            row for row in oracle["requirement_crosswalk"]
            if row["requirement_id"] == requirement_id
        )
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["scope_state"], "profile_conditional")
        self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-026")
        self.assertEqual(crosswalk["decision_id"], "KMIPKIT-DEC-026")
        self.assertIn(requirement_id, review["expected_client_requirement_ids"])
        self.assertEqual(review["source_obligations"][0]["actor"], "unresolved")
        self.assertEqual(
            review["source_obligations"][0]["project_actor_interpretation"],
            "client",
        )
        self.assertEqual(review["source_obligations"][0]["scope_disposition"], "profile_conditional")
        self.assertFalse(
            any(
                row.get("requirement_id") == requirement_id
                for row in review["rejected_candidate_mappings"]
            )
        )

    def test_derive_key_method_evidence_stays_with_the_baseline_profile(self) -> None:
        catalog = _read(CATALOG_PATH)
        oracle = _read(ORACLE_PATH)
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.14-002"
        requirement = next(row for row in catalog["requirements"] if row["requirement_id"] == requirement_id)
        review = next(
            row for row in oracle["source_requirement_reviews"]
            if row["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-6.1.14-002"
        )

        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["scope_state"], "profile_conditional")
        self.assertEqual(requirement["feature_spec"], "KMIPKIT-0037")
        self.assertEqual(requirement["status"], "scoped_verified")
        self.assertTrue(requirement["implementation_refs"])
        self.assertTrue(requirement["verification_refs"])
        self.assertIn(requirement_id, review["expected_client_requirement_ids"])
        for profile in catalog["profiles"]:
            with self.subTest(profile_id=profile["profile_id"]):
                if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT":
                    self.assertIn(requirement_id, profile["requirement_ids"])
                else:
                    self.assertNotIn(requirement_id, profile["requirement_ids"])


if __name__ == "__main__":
    unittest.main()
