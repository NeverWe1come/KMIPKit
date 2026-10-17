"""Regressions for actor and scope review in KMIP 2.1 section 6."""

from __future__ import annotations

import json
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.audit_sources import _decode_document


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SOURCE_PATH = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"

FALSE_CLIENT_CANDIDATES = {
    "KMIPKIT-REQ-SPEC-6.1.44-001": "KMIPKIT-CLAUSE-SPEC-6.1.44-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-002": "KMIPKIT-CLAUSE-SPEC-6.1.45-002",
    "KMIPKIT-REQ-SPEC-6.1.45-006-002": "KMIPKIT-CLAUSE-SPEC-6.1.45-006",
}

VALIDATE_CANDIDATES = {
    "KMIPKIT-REQ-SPEC-6.1.57-001": "KMIPKIT-CLAUSE-SPEC-6.1.57-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001": "KMIPKIT-CLAUSE-SPEC-6.1.57-002",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002": "KMIPKIT-CLAUSE-SPEC-6.1.57-002",
    "KMIPKIT-REQ-SPEC-6.1.57-004": "KMIPKIT-CLAUSE-SPEC-6.1.57-004",
    "KMIPKIT-REQ-SPEC-6.1.57-005": "KMIPKIT-CLAUSE-SPEC-6.1.57-005",
}

VALIDATE_LOCATORS = {
    "KMIPKIT-CLAUSE-SPEC-6.1.57-001": [36998, 37000],
    "KMIPKIT-CLAUSE-SPEC-6.1.57-002": [37002, 37006],
    "KMIPKIT-CLAUSE-SPEC-6.1.57-004": [37044, 37060],
    "KMIPKIT-CLAUSE-SPEC-6.1.57-005": [37062, 37079],
    "KMIPKIT-CLAUSE-SPEC-6.1.57-006": [37081, 37099],
}

VALIDATE_KEYWORDS = {
    "KMIPKIT-REQ-SPEC-6.1.57-001": "SHALL",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001": "MAY",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002": "MAY",
    "KMIPKIT-REQ-SPEC-6.1.57-004": "MAY",
    "KMIPKIT-REQ-SPEC-6.1.57-005": "MAY",
    "KMIPKIT-REQ-SPEC-6.1.57-006": "SHALL",
}


class _TextParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        self.parts.append(data)

    def normalized_text(self) -> str:
        return " ".join("".join(self.parts).split())


def pinned_excerpt(line_range: list[int]) -> str:
    lines = _decode_document(SOURCE_PATH.read_bytes()).splitlines()
    parser = _TextParser()
    parser.feed("\n".join(lines[line_range[0] - 1 : line_range[1]]))
    parser.close()
    return parser.normalized_text()


class SectionSixActorAndScopeRegressionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        self.fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        self.requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        self.reviews = {
            row["source_clause_id"]: row
            for row in self.fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        self.crosswalk_ids = {
            row["requirement_id"]
            for row in self.fixture["requirement_crosswalk"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        self.client_profile_ids = {
            requirement_id
            for profile in self.catalog["profiles"]
            if profile["role"] == "client"
            for requirement_id in profile["requirement_ids"]
        }

    def test_server_side_candidates_are_reviewed_but_never_client_mapped(self) -> None:
        for requirement_id, clause_id in FALSE_CLIENT_CANDIDATES.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                review = self.reviews[clause_id]
                rejections = {
                    candidate["requirement_id"]: candidate
                    for candidate in review["rejected_candidate_mappings"]
                }

                self.assertNotIn(requirement_id, self.crosswalk_ids)
                self.assertNotIn(requirement_id, self.client_profile_ids)
                self.assertNotIn(requirement_id, review["expected_client_requirement_ids"])
                self.assertNotEqual(requirement["role"], "client")
                self.assertNotEqual(requirement["scope_state"], "client_1_0")
                self.assertIn(requirement_id, rejections)
                self.assertIn("reject", rejections[requirement_id]["expected_disposition"])

    def test_import_and_encrypt_actor_dispositions_remain_explicit(self) -> None:
        import_id = "KMIPKIT-REQ-SPEC-6.1.25-001"
        import_requirement = self.requirements[import_id]
        import_review = self.reviews["KMIPKIT-CLAUSE-SPEC-6.1.25-001"]
        import_atoms = {
            atom["source_keyword"]: atom
            for atom in import_review["source_obligations"]
        }
        self.assertNotIn(import_id, self.crosswalk_ids)
        self.assertNotIn(import_id, self.client_profile_ids)
        self.assertEqual(import_requirement["role"], "unspecified")
        self.assertEqual(import_requirement["direction"], "unclear")
        self.assertEqual(import_requirement["scope_state"], "unclear")
        self.assertEqual(import_atoms["SHALL NOT"]["actor"], "server")
        self.assertEqual(import_atoms["SHALL NOT"]["scope_disposition"], "server_only")
        self.assertEqual(import_atoms["MUST"]["actor"], "server/actor-ambiguous")
        self.assertEqual(import_atoms["MUST"]["scope_disposition"], "unresolved")
        self.assertIn(
            "reject_as_client_requirement_actor_ambiguous",
            {
                candidate["expected_disposition"]
                for candidate in import_review["rejected_candidate_mappings"]
                if candidate["requirement_id"] == import_id
            },
        )

        encrypt_id = "KMIPKIT-REQ-SPEC-6.1.17-005-001"
        encrypt_requirement = self.requirements[encrypt_id]
        encrypt_review = self.reviews["KMIPKIT-CLAUSE-SPEC-6.1.17-005"]
        self.assertNotIn(encrypt_id, self.crosswalk_ids)
        self.assertNotIn(encrypt_id, self.client_profile_ids)
        self.assertEqual(encrypt_requirement["role"], "not_applicable")
        self.assertEqual(encrypt_requirement["scope_state"], "out_of_scope")
        self.assertEqual(encrypt_requirement["status"], "retired")
        self.assertIn(
            "reject_candidate_non_normative_client_description",
            {
                candidate["expected_disposition"]
                for candidate in encrypt_review["rejected_candidate_mappings"]
                if candidate["requirement_id"] == encrypt_id
            },
        )

    def test_validate_client_candidates_have_individual_source_dispositions(self) -> None:
        for requirement_id, clause_id in VALIDATE_CANDIDATES.items():
            with self.subTest(requirement_id=requirement_id):
                review = self.reviews.get(clause_id)
                self.assertIsNotNone(review, f"missing review for {requirement_id}")
                assert review is not None
                accepted = requirement_id in review["expected_client_requirement_ids"]
                rejected = any(
                    candidate["requirement_id"] == requirement_id
                    for candidate in review["rejected_candidate_mappings"]
                )
                atom = next(
                    (
                        obligation
                        for obligation in review["source_obligations"]
                        if obligation.get("candidate_requirement_id") == requirement_id
                    ),
                    None,
                )
                self.assertTrue(accepted or rejected or atom is not None)
                self.assertIsNotNone(atom, f"missing source atom for {requirement_id}")
                assert atom is not None
                self.assertEqual(atom["source_keyword"], VALIDATE_KEYWORDS[requirement_id])
                self.assertIn(requirement_id, self.crosswalk_ids)
                self.assertIn(requirement_id, review["expected_client_requirement_ids"])
                requirement = self.requirements[requirement_id]
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")

    def test_validate_feature_scope_does_not_attribute_passive_shall(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.57-006"
        clause_id = "KMIPKIT-CLAUSE-SPEC-6.1.57-006"
        requirement = self.requirements[requirement_id]
        clause = next(
            row for row in self.catalog["source_clauses"] if row["clause_id"] == clause_id
        )
        review = self.reviews[clause_id]
        atom = review["source_obligations"][0]

        self.assertEqual(clause["role"], "unclear")
        self.assertEqual(clause["direction"], "client_to_server")
        self.assertEqual(clause["scope_state"], "client_1_0")
        self.assertEqual(requirement["feature_spec"], "KMIPKIT-0042")
        self.assertEqual(requirement["role"], "unspecified")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertEqual(atom["source_keyword"], "SHALL")
        self.assertEqual(atom["candidate_requirement_id"], requirement_id)
        self.assertEqual(atom["actor"], "unresolved")
        self.assertEqual(atom["scope_disposition"], "client_1_0")
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertNotIn(requirement_id, self.crosswalk_ids)
        self.assertNotIn(requirement_id, self.client_profile_ids)
        self.assertEqual(requirement["profile_ids"], [])
        self.assertIn("feature/API serialization scope", review["review_note"])
        self.assertIn("does not establish server-side evaluation", review["review_note"])

    def test_validate_review_locators_match_the_pinned_html(self) -> None:
        clauses = {
            row["clause_id"]: row for row in self.catalog["source_clauses"]
        }
        for clause_id, line_range in VALIDATE_LOCATORS.items():
            with self.subTest(clause_id=clause_id):
                review = self.reviews.get(clause_id)
                self.assertIsNotNone(review, f"missing review for {clause_id}")
                assert review is not None
                self.assertEqual(review["html_line_range"], line_range)
                self.assertEqual(review["source_excerpt"], pinned_excerpt(line_range))
                self.assertEqual(review["source_locator"], clauses[clause_id]["locator"])
        self.assertIn("No, MAY be repeated", self.reviews["KMIPKIT-CLAUSE-SPEC-6.1.57-004"]["source_excerpt"])
        self.assertIn("No, MAY be repeated", self.reviews["KMIPKIT-CLAUSE-SPEC-6.1.57-005"]["source_excerpt"])
        self.assertIn("If omitted, the current date and time SHALL be assumed", self.reviews["KMIPKIT-CLAUSE-SPEC-6.1.57-006"]["source_excerpt"])

    def test_required_yes_payload_gaps_preserve_client_direction_and_scope(self) -> None:
        expected = {
            "KMIPKIT-REQ-SPEC-6.1.14-007-001": (
                "KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED",
                [25980, 26001],
                "include the desired Attributes in the Derive Key request",
            ),
        }
        gaps = {row["gap_id"]: row for row in self.fixture["source_only_requirement_gaps"]}
        for requirement_id, expected_data in expected.items():
            gap_id, line_range, direction = expected_data
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                gap = gaps[gap_id]
                self.assertEqual(requirement["normative_strength"], "mandatory")
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertIsNone(gap["source_clause_id"])
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(gap["source_excerpt"], pinned_excerpt(line_range))
                self.assertEqual(gap["source_keywords"], [])
                self.assertEqual(gap["source_force"], "required_table_body_value")
                self.assertEqual(gap["actor"], "client")
                self.assertEqual(gap["direction"], direction)
                self.assertEqual(gap["scope_disposition"], "client_1_0")
                self.assertEqual(gap["candidate_requirement_id"], requirement_id)

    def test_required_usage_count_table_contract_preserves_client_direction_and_scope(self) -> None:
        """The moved Table 232 row retains its actor, requiredness, and client scope."""
        gap_id = "KMIPKIT-SOURCE-GAP-SPEC-6.1.23-USAGE-LIMITS-COUNT-REQUIRED"
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.23-002-001"
        contract = next(
            row for row in self.fixture["source_table_contracts"] if row["gap_id"] == gap_id
        )
        requirement = self.requirements[requirement_id]

        self.assertEqual(requirement["source_gap_id"], gap_id)
        self.assertEqual(requirement["normative_strength"], "mandatory")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertIsNone(contract["source_clause_id"])
        self.assertEqual(contract["source_locator"]["kind"], "table_body_row")
        self.assertEqual(contract["source_locator"]["html_line_range"], [28080, 28097])
        self.assertEqual(contract["source_excerpt"], pinned_excerpt([28080, 28097]))
        self.assertEqual(contract["source_force"], "required_table_body_value")
        self.assertEqual(contract["actor"], "client")
        self.assertEqual(
            contract["direction"], "include Usage Limits Count in the client request"
        )
        self.assertEqual(contract["scope_disposition"], "client_1_0")
        self.assertEqual(contract["condition"], "A Get Usage Allocation request is sent.")
        self.assertEqual(contract["requiredness"], "Yes")
        self.assertEqual(contract["contract_kind"], "operation_request_table_field")
        self.assertEqual(
            contract["operation_element_id"], "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION"
        )


if __name__ == "__main__":
    unittest.main()
