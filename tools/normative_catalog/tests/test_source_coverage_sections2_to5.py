"""Regression tests for source review coverage in KMIP §§2–5."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SOURCE_IDS = {"KMIPKIT-SRC-spec", "KMIPKIT-SRC-profiles"}
TARGET_MAJOR_SECTIONS = {"2", "3", "4", "5"}


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
                if not review.get("source_obligations") or any(
                    not obligation.get("scope_disposition")
                    for obligation in review["source_obligations"]
                ):
                    missing_dispositions.append(f"{requirement_id} ({clause_id})")

        self.assertEqual(missing_reviews, [])
        self.assertEqual(missing_dispositions, [])


if __name__ == "__main__":
    unittest.main()
