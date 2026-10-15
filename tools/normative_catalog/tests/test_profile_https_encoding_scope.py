"""Source-backed scope checks for the HTTPS Client profile."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog.tests.test_source_gap_context_resolutions import pinned_source_excerpt


ROOT = Path(__file__).resolve().parents[3]
PROFILES_SOURCE_PATH = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"


class HttpsEncodingScopeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        cls.clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        cls.gaps = {row["gap_id"]: row for row in fixture["source_only_requirement_gaps"]}

    def test_content_type_clauses_keep_client_request_scope_and_encoding_conditions(self) -> None:
        cases = (
            ("KMIPKIT-CLAUSE-PROF-5.3.1-006", [5139, 5140], "XML"),
            ("KMIPKIT-CLAUSE-PROF-5.3.1-007", [5141, 5142], "JSON"),
        )
        for clause_id, line_range, encoding in cases:
            with self.subTest(clause_id=clause_id):
                clause = self.clauses[clause_id]
                self.assertEqual(clause["role"], "client")
                self.assertEqual(clause["direction"], "client_to_server")
                self.assertEqual(clause["condition"], f"When message encoding is {encoding}.")
                self.assertEqual(clause["disposition"], "non_applicable")
                self.assertEqual(clause["scope_state"], "out_of_scope")
                self.assertEqual(clause["requirement_ids"], [])
                excerpt = pinned_source_excerpt(PROFILES_SOURCE_PATH, line_range)
                self.assertIn(f"if the message encoding is {encoding}", excerpt)
                self.assertIn("SHALL specify a Content-Type", excerpt)

    def test_server_to_client_https_behavior_keeps_its_conditional_client_actor(self) -> None:
        clause_id = "KMIPKIT-CLAUSE-PROF-5.3.1-011"
        clause = self.clauses[clause_id]
        self.assertEqual(clause["role"], "client")
        self.assertEqual(clause["direction"], "server_to_client")
        self.assertEqual(
            clause["condition"],
            "When the client supports responding to server-to-client operations.",
        )
        self.assertEqual(clause["disposition"], "later_1_1")
        self.assertEqual(clause["scope_state"], "client_1_1")
        self.assertEqual(clause["requirement_ids"], [])
        self.assertEqual(
            pinned_source_excerpt(PROFILES_SOURCE_PATH, [5149, 5150]),
            "KMIP clients that support responding to server to client operations SHALL behave as a HTTPS server.",
        )

        gap = self.gaps["KMIPKIT-SOURCE-GAP-PROF-5.3.1-5149"]
        self.assertEqual(gap["source_clause_id"], clause_id)
        self.assertEqual(gap["source_locator"], clause["locator"])
        self.assertEqual(gap["scope_disposition"], "out_of_scope")
        self.assertIsNone(gap["candidate_requirement_id"])
        self.assertIsNone(gap["proposed_requirement_id"])

    def test_out_of_scope_https_gaps_link_to_their_catalog_clauses(self) -> None:
        for gap_id, clause_id in (
            ("KMIPKIT-SOURCE-GAP-PROF-5.3.1-5139", "KMIPKIT-CLAUSE-PROF-5.3.1-006"),
            ("KMIPKIT-SOURCE-GAP-PROF-5.3.1-5141", "KMIPKIT-CLAUSE-PROF-5.3.1-007"),
            ("KMIPKIT-SOURCE-GAP-PROF-5.3.1-5149", "KMIPKIT-CLAUSE-PROF-5.3.1-011"),
        ):
            with self.subTest(gap_id=gap_id):
                gap = self.gaps[gap_id]
                self.assertEqual(gap["source_clause_id"], clause_id)
                self.assertEqual(gap["source_locator"], self.clauses[clause_id]["locator"])


if __name__ == "__main__":
    unittest.main()
