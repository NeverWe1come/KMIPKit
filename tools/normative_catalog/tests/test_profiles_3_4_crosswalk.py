"""Source-scope regression tests for Profiles v2.1 sections 3 through 4.1.2."""

from __future__ import annotations

import json
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.audit_sources import _decode_document


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SOURCE_PATH = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"


class _TextParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        self.parts.append(data)

    def normalized_text(self) -> str:
        return " ".join("".join(self.parts).split())


def pinned_excerpt(line_range: list[int]) -> str:
    lines = _decode_document((ROOT / SOURCE_PATH).read_bytes()).splitlines()
    parser = _TextParser()
    parser.feed("\n".join(lines[line_range[0] - 1 : line_range[1]]))
    parser.close()
    return parser.normalized_text()


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


CLIENT_IDS = {
    "KMIPKIT-REQ-PROF-3.1-001",
    "KMIPKIT-REQ-PROF-3.1.1-002",
    "KMIPKIT-REQ-PROF-3.1.1-005",
    "KMIPKIT-REQ-PROF-3.1.2-002",
    "KMIPKIT-REQ-PROF-3.1.2-005",
    "KMIPKIT-REQ-PROF-3.2.1-001",
    "KMIPKIT-REQ-PROF-3.2.2-001",
    "KMIPKIT-REQ-PROF-3.2.3-001",
    "KMIPKIT-REQ-PROF-3.2.4-002",
    "KMIPKIT-REQ-PROF-3.2-001",
    "KMIPKIT-REQ-PROF-4.1.2-007",
}
UNRESOLVED_IDS = {
    "KMIPKIT-REQ-PROF-3-002",
    "KMIPKIT-REQ-PROF-3-003",
    "KMIPKIT-REQ-PROF-3-004",
    "KMIPKIT-REQ-PROF-3-006",
    "KMIPKIT-REQ-PROF-4.1.2-001",
    "KMIPKIT-REQ-PROF-4.1.2-005",
}
OUT_OF_SCOPE_IDS = {
    "KMIPKIT-REQ-PROF-3.1.1-004",
    "KMIPKIT-REQ-PROF-4.1-001",
    "KMIPKIT-REQ-PROF-4.1-003",
    "KMIPKIT-REQ-PROF-4.1.2-008",
    "KMIPKIT-REQ-PROF-4.1.2-009",
}
ALL_TARGET_IDS = CLIENT_IDS | UNRESOLVED_IDS | OUT_OF_SCOPE_IDS


class ProfileSectionsThreeAndFourCrosswalkTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = load_json(CATALOG_PATH)
        self.fixture = load_json(FIXTURE_PATH)
        self.requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        self.clauses = {row["clause_id"]: row for row in self.catalog["source_clauses"]}
        profile_reviews = {
            row["source_clause_id"]: row
            for row in self.fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-profiles"
        }
        profile_crosswalk = {
            row["requirement_id"]: row
            for row in self.fixture["requirement_crosswalk"]
            if row["source_id"] == "KMIPKIT-SRC-profiles"
        }
        self.reviews = {
            clause_id: row
            for clause_id, row in profile_reviews.items()
            if row["source_section"] == "3"
            or row["source_section"].startswith("3.")
            or row["source_section"] == "4"
            or (
                row["source_section"].startswith("4.")
                and int(row["source_section"].split(".")[1]) <= 1
            )
        }
        self.crosswalk = {
            requirement_id: row
            for requirement_id, row in profile_crosswalk.items()
            if row["source_section"] == "3"
            or row["source_section"].startswith("3.")
            or row["source_section"] == "4"
            or (
                row["source_section"].startswith("4.")
                and int(row["source_section"].split(".")[1]) <= 1
            )
        }

    def test_target_ids_have_audited_source_rows_and_only_client_mappings(self) -> None:
        self.assertEqual(len(ALL_TARGET_IDS), 22)
        self.assertTrue(ALL_TARGET_IDS.issubset(self.requirements))
        target_clause_ids = {
            self.requirements[requirement_id]["source_clause_ids"][0]
            for requirement_id in ALL_TARGET_IDS
        }
        self.assertTrue(target_clause_ids.issubset(self.reviews))
        self.assertEqual(set(self.crosswalk), CLIENT_IDS)
        self.assertEqual(set(self.reviews), target_clause_ids)

    def test_unresolved_and_out_of_scope_rows_are_not_client_obligations(self) -> None:
        self.assertEqual(len(UNRESOLVED_IDS), 6)
        self.assertEqual(len(OUT_OF_SCOPE_IDS), 5)
        self.assertFalse((UNRESOLVED_IDS | OUT_OF_SCOPE_IDS) & self.crosswalk.keys())
        reviewed_clause_ids = {
            self.requirements[requirement_id]["source_clause_ids"][0]
            for requirement_id in UNRESOLVED_IDS | OUT_OF_SCOPE_IDS
        }
        self.assertTrue(reviewed_clause_ids.issubset(self.reviews))
        for requirement_id in UNRESOLVED_IDS:
            requirement = self.requirements[requirement_id]
            review = self.reviews[requirement["source_clause_ids"][0]]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"][0]["actor"], "unresolved")
            self.assertEqual(review["source_obligations"][0]["scope_disposition"], "unresolved")
        for requirement_id in OUT_OF_SCOPE_IDS:
            requirement = self.requirements[requirement_id]
            review = self.reviews[requirement["source_clause_ids"][0]]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"][0]["scope_disposition"], "out_of_scope")

    def test_client_crosswalk_rows_preserve_profile_membership_and_source_actors(self) -> None:
        https_ids = {
            "KMIPKIT-REQ-PROF-3.2-001",
            "KMIPKIT-REQ-PROF-3.2.1-001",
            "KMIPKIT-REQ-PROF-3.2.2-001",
            "KMIPKIT-REQ-PROF-3.2.3-001",
            "KMIPKIT-REQ-PROF-3.2.4-002",
        }
        baseline_ids = {
            "KMIPKIT-REQ-PROF-3.1-001",
            "KMIPKIT-REQ-PROF-3.1.1-002",
            "KMIPKIT-REQ-PROF-3.1.1-005",
            "KMIPKIT-REQ-PROF-3.1.2-002",
            "KMIPKIT-REQ-PROF-3.1.2-005",
        }
        for requirement_id in CLIENT_IDS:
            requirement = self.requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            self.assertIn(clause_id, self.reviews)
            review = self.reviews[clause_id]
            row = self.crosswalk[requirement_id]
            expected_applies_to = (
                "KMIPKIT-PROFILE-HTTPS-CLIENT"
                if requirement_id in https_ids
                else "KMIPKIT-PROFILE-BASELINE-CLIENT"
                if requirement_id in baseline_ids
                else "all_client_profiles"
            )
            self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
            expected_actor = (
                "client and server"
                if requirement_id
                in {
                    "KMIPKIT-REQ-PROF-3.1.1-005",
                    "KMIPKIT-REQ-PROF-3.1.2-005",
                    "KMIPKIT-REQ-PROF-3.2.1-001",
                    "KMIPKIT-REQ-PROF-3.2.2-001",
                    "KMIPKIT-REQ-PROF-3.2.3-001",
                }
                else "client"
            )
            self.assertEqual(review["source_obligations"][0]["actor"], expected_actor)
            self.assertEqual(row["source_actor"], expected_actor)
            self.assertEqual(row["applies_to"], expected_applies_to)

    def test_single_item_batch_id_requirement_lists_all_client_profiles(self) -> None:
        """The reviewed §4.1.2 MAY applies across client profiles, not only Baseline."""
        requirement = self.requirements["KMIPKIT-REQ-PROF-4.1.2-007"]
        expected_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.catalog["profiles"]
            if profile["role"] == "client"
        )

        self.assertEqual(requirement["profile_ids"], expected_profile_ids)
        self.assertEqual(self.crosswalk[requirement["requirement_id"]]["applies_to"], "all_client_profiles")

    def test_https_profile_closure_includes_all_explicit_client_requirements(self) -> None:
        """Sections 3.2 and 3.2.1–3.2.4 assign conditions to the HTTPS profile."""
        https_profile_id = "KMIPKIT-PROFILE-HTTPS-CLIENT"
        https_profile = next(
            row for row in self.catalog["profiles"] if row["profile_id"] == https_profile_id
        )
        expected_ids = {
            "KMIPKIT-REQ-PROF-3.2-001",
            "KMIPKIT-REQ-PROF-3.2.1-001",
            "KMIPKIT-REQ-PROF-3.2.2-001",
            "KMIPKIT-REQ-PROF-3.2.3-001",
            "KMIPKIT-REQ-PROF-3.2.4-002",
        }

        for requirement_id in expected_ids:
            requirement = self.requirements[requirement_id]
            self.assertIn(https_profile_id, requirement["profile_ids"])
            if requirement_id == "KMIPKIT-REQ-PROF-3.2-001":
                self.assertNotIn(
                    "KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"]
                )
            else:
                self.assertEqual(requirement["profile_ids"], [https_profile_id])
            self.assertIn(requirement_id, https_profile["requirement_ids"])

    def test_https_authentication_suite_clause_is_client_applicable_without_direction(self) -> None:
        """Sections 3.2 and 6.4 establish client scope, while §3.2 leaves direction unstated."""
        requirement_id = "KMIPKIT-REQ-PROF-3.2-001"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.2-001"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]

        self.assertEqual(requirement["direction"], "unclear")
        self.assertTrue(
            {
                "KMIPKIT-PROFILE-HTTPS-CLIENT",
                "KMIPKIT-PROFILE-HTTPS-SERVER",
                "KMIPKIT-PROFILE-COMPLETE-SERVER",
            }.issubset(requirement["profile_ids"])
        )
        self.assertNotIn("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"])
        self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
        self.assertEqual(review["source_obligations"][0]["actor"], "client")
        self.assertEqual(review["source_obligations"][0]["direction"], "unresolved")
        self.assertEqual(review["html_line_range"], [4227, 4229])
        self.assertIn(
            "SHALL support the HTTPS Authentication Suite conditions (3.2)",
            pinned_excerpt([10532, 10534]),
        )
        self.assertIn(
            "SHALL support the HTTPS Authentication Suite conditions (3.2)",
            pinned_excerpt([10550, 10552]),
        )

    def test_basic_authentication_suite_requirement_is_in_baseline_client_closure(self) -> None:
        """Sections 3.1 and 6.1 make the conditional TLS requirement part of Baseline Client."""
        requirement_id = "KMIPKIT-REQ-PROF-3.1-001"
        baseline_profile_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        requirement = self.requirements[requirement_id]
        profile = next(
            row for row in self.catalog["profiles"] if row["profile_id"] == baseline_profile_id
        )

        self.assertIn(baseline_profile_id, requirement["profile_ids"])
        self.assertIn(requirement_id, profile["requirement_ids"])
        self.assertEqual(self.crosswalk[requirement_id]["applies_to"], baseline_profile_id)
        self.assertNotIn(requirement_id, UNRESOLVED_IDS)

    def test_tls12_client_may_is_preserved_but_outside_the_10_profile_scope(self) -> None:
        """Profiles §3.1.1's TLS 1.2 permission cannot expand the TLS 1.3-only ADR."""
        requirement_id = "KMIPKIT-REQ-PROF-3.1.1-004"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.1-004"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]
        baseline = next(
            row
            for row in self.catalog["profiles"]
            if row["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        source_excerpt = pinned_excerpt([4030, 4037])
        transport_decision = (ROOT / "docs/adr/0005-transport-and-tls.md").read_text(
            encoding="utf-8"
        )

        self.assertIn("Conformant KMIP clients MAY support:", source_excerpt)
        self.assertIn("TLS v1.2 [RFC5246]", source_excerpt)
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertEqual(review["source_obligations"][0]["scope_disposition"], "out_of_scope")
        self.assertEqual(requirement["scope_state"], "out_of_scope")
        self.assertIn("ADR-0005", requirement["review_note"])
        self.assertIn("TLS 1.3 only", transport_decision)
        self.assertIn(clause_id, baseline["source_clause_ids"])
        self.assertEqual(self.clauses[clause_id]["scope_state"], "out_of_scope")
        self.assertIn(requirement_id, self.clauses[clause_id]["requirement_ids"])
        self.assertIn("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"])
        self.assertNotIn(requirement_id, baseline["requirement_ids"])
        self.assertNotIn(requirement_id, self.crosswalk)

    def test_profile_test_variations_do_not_create_client_runtime_scope(self) -> None:
        """§§4.1 and 4.1.2 separate conformance-test variation from client obligations."""
        expected_scope = {
            "KMIPKIT-REQ-PROF-4.1-001": ("unclear", "unclear", "out_of_scope"),
            "KMIPKIT-REQ-PROF-4.1-003": ("unclear", "unclear", "out_of_scope"),
            "KMIPKIT-REQ-PROF-4.1.2-001": ("unclear", "unclear", "unclear"),
            "KMIPKIT-REQ-PROF-4.1.2-005": ("unclear", "unclear", "unclear"),
            "KMIPKIT-REQ-PROF-4.1.2-008": (
                "not_applicable",
                "not_applicable",
                "out_of_scope",
            ),
            "KMIPKIT-REQ-PROF-4.1.2-009": (
                "not_applicable",
                "not_applicable",
                "out_of_scope",
            ),
        }
        baseline_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        baseline = next(
            profile for profile in self.catalog["profiles"] if profile["profile_id"] == baseline_id
        )

        for requirement_id, (role, direction, scope) in expected_scope.items():
            requirement = self.requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            clause = self.clauses[clause_id]
            review = self.reviews[clause_id]

            self.assertEqual((requirement["role"], requirement["direction"], requirement["scope_state"]), (role, direction, scope))
            self.assertEqual((clause["role"], clause["direction"], clause["scope_state"]), (role, direction, scope))
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertNotIn(requirement_id, self.crosswalk)
            self.assertNotIn(requirement_id, baseline["requirement_ids"])
            self.assertNotIn(clause_id, baseline["source_clause_ids"])

        auth_requirement = self.requirements["KMIPKIT-REQ-PROF-4.1.2-005"]
        self.assertNotIn("client request", auth_requirement["summary"].lower())
        interop_text = pinned_excerpt([4494, 4502]).lower()
        self.assertIn("not expected that production kmip clients will support the interop operation", interop_text)

    def test_profile_4_1_2_007_is_may_without_a_false_must_keyword(self) -> None:
        requirement_id = "KMIPKIT-REQ-PROF-4.1.2-007"
        requirement = self.requirements[requirement_id]
        clause = self.clauses["KMIPKIT-CLAUSE-PROF-4.1.2-007"]
        self.assertIn("KMIPKIT-CLAUSE-PROF-4.1.2-007", self.reviews)
        review = self.reviews["KMIPKIT-CLAUSE-PROF-4.1.2-007"]
        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertIn("MAY", clause["source_keywords"])
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertEqual(review["html_line_range"], [4488, 4493])
        self.assertEqual(review["source_excerpt"], pinned_excerpt([4488, 4493]))

    def test_all_target_review_locators_and_excerpts_match_pinned_html(self) -> None:
        expected_lines = {
            "KMIPKIT-CLAUSE-PROF-3-002": [3967, 3970],
            "KMIPKIT-CLAUSE-PROF-3-003": [3972, 3975],
            "KMIPKIT-CLAUSE-PROF-3-004": [3977, 3980],
            "KMIPKIT-CLAUSE-PROF-3-006": [3987, 3990],
            "KMIPKIT-CLAUSE-PROF-3.1-001": [3999, 4001],
            "KMIPKIT-CLAUSE-PROF-3.1.1-002": [4016, 4016],
            "KMIPKIT-CLAUSE-PROF-3.1.1-004": [4030, 4030],
            "KMIPKIT-CLAUSE-PROF-3.1.1-005": [4037, 4037],
            "KMIPKIT-CLAUSE-PROF-3.1.2-002": [4068, 4069],
            "KMIPKIT-CLAUSE-PROF-3.1.2-005": [4187, 4188],
            "KMIPKIT-CLAUSE-PROF-3.2-001": [4227, 4229],
            "KMIPKIT-CLAUSE-PROF-3.2.1-001": [4236, 4237],
            "KMIPKIT-CLAUSE-PROF-3.2.2-001": [4244, 4245],
            "KMIPKIT-CLAUSE-PROF-3.2.3-001": [4252, 4253],
            "KMIPKIT-CLAUSE-PROF-3.2.4-002": [4264, 4266],
            "KMIPKIT-CLAUSE-PROF-4.1-001": [4312, 4314],
            "KMIPKIT-CLAUSE-PROF-4.1-003": [4320, 4321],
            "KMIPKIT-CLAUSE-PROF-4.1.2-001": [4464, 4465],
            "KMIPKIT-CLAUSE-PROF-4.1.2-005": [4482, 4483],
            "KMIPKIT-CLAUSE-PROF-4.1.2-007": [4488, 4493],
            "KMIPKIT-CLAUSE-PROF-4.1.2-008": [4494, 4495],
            "KMIPKIT-CLAUSE-PROF-4.1.2-009": [4496, 4497],
        }
        self.assertEqual(set(self.reviews), set(expected_lines))
        for requirement_id in ALL_TARGET_IDS:
            clause_id = self.requirements[requirement_id]["source_clause_ids"][0]
            review = self.reviews[clause_id]
            self.assertEqual(review["html_line_range"], expected_lines[clause_id])
            self.assertEqual(review["source_excerpt"], pinned_excerpt(expected_lines[clause_id]))
            self.assertEqual(review["source_locator"], self.clauses[clause_id]["locator"])


if __name__ == "__main__":
    unittest.main()
