"""Regression tests for source review coverage in KMIP §§2–5."""

from __future__ import annotations

import json
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.audit_sources import _decode_document


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SOURCE_IDS = {"KMIPKIT-SRC-spec", "KMIPKIT-SRC-profiles"}
TARGET_MAJOR_SECTIONS = {"2", "3", "4", "5"}
SOURCE_PATH = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
EXPECTED_LINES = {
    "KMIPKIT-CLAUSE-SPEC-2.1-001": [6700, 6716],
    "KMIPKIT-CLAUSE-SPEC-2.2-001": [6792, 6808],
    "KMIPKIT-CLAUSE-SPEC-2.3-003": [6891, 6907],
    "KMIPKIT-CLAUSE-SPEC-2.4-003": [6992, 7008],
    "KMIPKIT-CLAUSE-SPEC-2.5-001": [7084, 7100],
    "KMIPKIT-CLAUSE-SPEC-2.6-001": [7157, 7173],
    "KMIPKIT-CLAUSE-SPEC-2.7-002": [7232, 7248],
    "KMIPKIT-CLAUSE-SPEC-2.8-002": [7331, 7347],
    "KMIPKIT-CLAUSE-SPEC-2.9-001": [7499, 7515],
    "KMIPKIT-CLAUSE-SPEC-4-001": [9594, 9605],
    "KMIPKIT-CLAUSE-SPEC-4.57-001": [20200, 20204],
    "KMIPKIT-CLAUSE-SPEC-4.57-002": [20218, 20221],
    "KMIPKIT-CLAUSE-SPEC-4.57-003": [20225, 20231],
    "KMIPKIT-CLAUSE-SPEC-4.57-004": [20233, 20241],
    "KMIPKIT-CLAUSE-SPEC-4.57-005": [20243, 20251],
    "KMIPKIT-CLAUSE-SPEC-4.57-006": [20253, 20256],
    "KMIPKIT-CLAUSE-SPEC-4.57-007": [20258, 20262],
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


class SourceCoverageSectionsTwoThroughFiveTests(unittest.TestCase):
    def test_catalogued_requirements_have_review_rows_and_dispositions(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        reviews = {
            (row["source_id"], row["source_clause_id"]): row
            for row in fixture["source_requirement_reviews"]
        }

        missing_reviews: list[str] = []
        missing_dispositions: list[str] = []
        for requirement in catalog["requirements"]:
            requirement_id = requirement["requirement_id"]
            for clause_id in requirement["source_clause_ids"]:
                clause = clauses[clause_id]
                section = clause["section"]
                if (
                    clause["source_id"] not in SOURCE_IDS
                    or section.split(".")[0] not in TARGET_MAJOR_SECTIONS
                ):
                    continue

                review = reviews.get((clause["source_id"], clause_id))
                if review is None:
                    missing_reviews.append(f"{requirement_id} ({clause_id})")
                    continue
                has_disposition = bool(review.get("source_obligations")) and all(
                    obligation.get("scope_disposition")
                    for obligation in review["source_obligations"]
                )
                has_rejection = any(
                    candidate.get("requirement_id") == requirement_id
                    and candidate.get("expected_disposition")
                    for candidate in review.get("rejected_candidate_mappings", [])
                )
                if not (has_disposition or has_rejection):
                    missing_dispositions.append(f"{requirement_id} ({clause_id})")

        self.assertEqual(missing_reviews, [])
        self.assertEqual(missing_dispositions, [])

    def test_review_rows_match_pinned_section_and_locator_text(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        reviews = {
            row["source_clause_id"]: row
            for row in fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }

        self.assertTrue(set(EXPECTED_LINES).issubset(reviews))
        for clause_id, line_range in EXPECTED_LINES.items():
            review = reviews[clause_id]
            self.assertEqual(review["html_line_range"], line_range, clause_id)
            self.assertEqual(review["source_excerpt"], pinned_excerpt(line_range), clause_id)
            self.assertEqual(review["source_locator"], clauses[clause_id]["locator"], clause_id)

    def test_object_encoding_required_headers_do_not_create_client_obligations(self) -> None:
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        reviews = {
            row["source_clause_id"]: row
            for row in fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        candidate_req_ids_by_header_clause = {
            "KMIPKIT-CLAUSE-SPEC-2.1-001": "KMIPKIT-REQ-SPEC-2.1-001",
            "KMIPKIT-CLAUSE-SPEC-2.2-001": "KMIPKIT-REQ-SPEC-2.2-001",
            "KMIPKIT-CLAUSE-SPEC-2.3-003": "KMIPKIT-REQ-SPEC-2.3-003",
            "KMIPKIT-CLAUSE-SPEC-2.4-003": "KMIPKIT-REQ-SPEC-2.4-003",
            "KMIPKIT-CLAUSE-SPEC-2.5-001": "KMIPKIT-REQ-SPEC-2.5-001",
            "KMIPKIT-CLAUSE-SPEC-2.6-001": "KMIPKIT-REQ-SPEC-2.6-001",
            "KMIPKIT-CLAUSE-SPEC-2.7-002": "KMIPKIT-REQ-SPEC-2.7-002",
            "KMIPKIT-CLAUSE-SPEC-2.8-002": "KMIPKIT-REQ-SPEC-2.8-002",
            "KMIPKIT-CLAUSE-SPEC-2.9-001": "KMIPKIT-REQ-SPEC-2.9-001",
        }

        self.assertTrue(
            set(candidate_req_ids_by_header_clause).issubset(reviews)
        )
        for clause_id, candidate_id in candidate_req_ids_by_header_clause.items():
            review = reviews[clause_id]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"], [])
            self.assertEqual(
                review["rejected_candidate_mappings"],
                [
                    {
                        "requirement_id": candidate_id,
                        "claimed_keyword": "REQUIRED",
                        "source_review": review["review_note"],
                        "expected_disposition": "reject_candidate_non_normative_table_header",
                    }
                ],
            )

    def test_active_state_fourth_predicate_preserves_source_reference_without_actor_inference(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        reviews = {
            row["source_clause_id"]: row
            for row in fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        review = reviews["KMIPKIT-CLAUSE-SPEC-4.57-003"]
        fourth = review["source_obligations"][3]
        requirement_id = "KMIPKIT-REQ-SPEC-4.57-003-004"

        self.assertEqual(
            fourth["source_text"],
            "If a Protect Stop attribute is set, then the object SHALL NOT be used for cryptographic purposes after the Process Stop Date.",
        )
        self.assertEqual(fourth["source_keyword"], "SHALL NOT")
        self.assertEqual(fourth["actor"], "unresolved")
        self.assertEqual(fourth["scope_disposition"], "unresolved")
        self.assertEqual(fourth["candidate_requirement_id"], requirement_id)
        self.assertIn(requirement_id, requirements)
        self.assertEqual(requirements[requirement_id]["role"], "unspecified")
        self.assertEqual(requirements[requirement_id]["scope_state"], "unclear")
        self.assertNotIn(requirement_id, review["expected_client_requirement_ids"])
        self.assertIn("Process Stop Date", review["review_note"])
        self.assertIn("KMIPKIT-DISC-013", review["review_note"])

    def test_unresolved_object_state_atoms_are_not_client_obligations(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        # Keep only actor/scope atoms still unresolved by T002; §4.57-001 is client scope,
        # §4.57-002 is server-only by DEC-020, and §4.57-003-001 is server-only.
        unresolved_ids = {
            *(f"KMIPKIT-REQ-SPEC-4-001-{ordinal:03d}" for ordinal in range(1, 4)),
            *(f"KMIPKIT-REQ-SPEC-4.57-003-{ordinal:03d}" for ordinal in range(2, 5)),
            *(f"KMIPKIT-REQ-SPEC-4.57-004-{ordinal:03d}" for ordinal in range(1, 4)),
            *(f"KMIPKIT-REQ-SPEC-4.57-005-{ordinal:03d}" for ordinal in range(1, 4)),
            "KMIPKIT-REQ-SPEC-4.57-006",
            "KMIPKIT-REQ-SPEC-4.57-007",
        }
        review_by_requirement = {
            atom["candidate_requirement_id"]: (review, atom)
            for review in fixture["source_requirement_reviews"]
            for atom in review["source_obligations"]
            if atom["candidate_requirement_id"] is not None
        }
        crosswalk_ids = {
            row["requirement_id"] for row in fixture["requirement_crosswalk"]
        }
        client_profiles = [profile for profile in catalog["profiles"] if profile["role"] == "client"]

        self.assertTrue(unresolved_ids.issubset(review_by_requirement))
        self.assertFalse(unresolved_ids & crosswalk_ids)
        for requirement_id in unresolved_ids:
            requirement = requirements[requirement_id]
            review, atom = review_by_requirement[requirement_id]
            self.assertEqual(requirement["role"], "unspecified", requirement_id)
            self.assertEqual(requirement["scope_state"], "unclear", requirement_id)
            self.assertEqual(atom["actor"], "unresolved", requirement_id)
            self.assertEqual(atom["scope_disposition"], "unresolved", requirement_id)
            self.assertNotIn(requirement_id, review["expected_client_requirement_ids"])
            for profile in client_profiles:
                self.assertNotIn(requirement_id, profile["requirement_ids"], profile["profile_id"])

    def test_attribute_intro_maps_only_the_explicit_server_or_client_actors(self) -> None:
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        reviews = {
            row["source_clause_id"]: row
            for row in fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        crosswalk = {
            row["requirement_id"]: row
            for row in fixture["requirement_crosswalk"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
            and row.get("source_clause_id") == "KMIPKIT-CLAUSE-SPEC-4-001"
        }
        self.assertTrue("KMIPKIT-CLAUSE-SPEC-4-001" in reviews)
        review = reviews["KMIPKIT-CLAUSE-SPEC-4-001"]
        atoms = review["source_obligations"]

        self.assertEqual(
            [(row["source_order"], row["source_keyword"], row["actor"]) for row in atoms],
            [
                (1, "MAY", "unresolved"),
                (2, "SHOULD", "unresolved"),
                (3, "SHALL", "unresolved"),
                (4, "SHALL NOT", "either server or client"),
                (5, "SHALL NOT", "client"),
            ],
        )
        self.assertEqual(
            set(review["expected_client_requirement_ids"]),
            {"KMIPKIT-REQ-SPEC-4-001-004", "KMIPKIT-REQ-SPEC-4-001-005"},
        )
        self.assertEqual(
            {requirement_id: row["role"] for requirement_id, row in crosswalk.items()},
            {
                "KMIPKIT-REQ-SPEC-4-001-004": "both",
                "KMIPKIT-REQ-SPEC-4-001-005": "client",
            },
        )

    def test_existing_sections_three_to_five_header_notes_explain_source_and_case_gap(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        reviews = {
            row["source_clause_id"]: row
            for row in fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-spec"
        }
        header_dispositions = {
            "reject_candidate_non_normative_table_header",
            "table_header_only",
            "header_only_metadata",
            "header_only_keyword_label_not_normative_requirement",
        }
        headers: dict[str, dict[str, object]] = {}
        for review in reviews.values():
            for candidate in review["rejected_candidate_mappings"]:
                section = review["source_section"]
                if (
                    section.split(".")[0] in {"3", "4", "5"}
                    and candidate["expected_disposition"] in header_dispositions
                ):
                    headers[candidate["requirement_id"]] = review

        self.assertEqual(len(headers), 34)
        for requirement_id, review in headers.items():
            requirement = requirements[requirement_id]
            line_range = review["html_line_range"]
            note = requirement["review_note"]
            self.assertIn("kmip-spec-v2.1-os.html", note, requirement_id)
            self.assertIn(f"§{review['source_section']}", note, requirement_id)
            self.assertIn(f"lines {line_range[0]}-{line_range[1]}", note, requirement_id)
            self.assertIn("No requirement-specific official Test Cases ID is linked", note, requirement_id)
            self.assertEqual(requirement["test_case_ids"], [], requirement_id)


if __name__ == "__main__":
    unittest.main()
