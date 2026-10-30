"""Source-review regressions for Profiles v2.1 section 6 client clauses."""

from __future__ import annotations

import json
import unittest

from tools.normative_catalog.tests.test_validate import (
    ROOT,
    audit_document_with_text,
    pinned_source_excerpt,
)


SOURCE_ID = "KMIPKIT-SRC-profiles"
SOURCE_PATH = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
PROFILE_BY_SECTION = {
    "6.1": "all_client_profiles",
    "6.4": "KMIPKIT-PROFILE-HTTPS-CLIENT",
    "6.10": "KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-CLIENT",
    "6.12": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-BASIC-CLIENT",
    "6.13": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
    "6.14": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
    "6.16": "KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-CLIENT",
    "6.18": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-CLIENT",
    "6.19": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-CLIENT",
    "6.20": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-CLIENT",
    "6.24": "KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-CLIENT",
    "6.26": "KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-CLIENT",
    "6.28": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "6.30": "KMIPKIT-PROFILE-AES-XTS-CLIENT",
    "6.32": "KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT",
    "6.34": "KMIPKIT-PROFILE-PKCS11-CLIENT",
}
PROFILE_SECTIONS = {"6", *PROFILE_BY_SECTION}


def _load_json(relative_path: str) -> dict[str, object]:
    return json.loads((ROOT / relative_path).read_text(encoding="utf-8"))


class ProfileSectionSixCrosswalkTests(unittest.TestCase):
    def test_catalogued_section_six_atoms_have_exact_profile_crosswalks_and_reviews(self) -> None:
        catalog = _load_json("specification/catalog/kmip-2.1.json")
        fixture = _load_json("tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json")
        clauses = {
            clause["clause_id"]: clause
            for clause in catalog["source_clauses"]
            if clause["source_id"] == SOURCE_ID
            and clause["section"] in PROFILE_SECTIONS
            and clause["requirement_ids"]
        }
        requirements = {
            requirement["requirement_id"]: requirement
            for requirement in catalog["requirements"]
            if any(clause_id in clauses for clause_id in requirement["source_clause_ids"])
        }
        self.assertEqual(len(requirements), 80)

        crosswalk_rows = [
            row
            for row in fixture["requirement_crosswalk"]
            if row["source_id"] == SOURCE_ID and row["source_section"] in PROFILE_SECTIONS
        ]
        crosswalk_ids = [row["requirement_id"] for row in crosswalk_rows]
        self.assertEqual(len(crosswalk_ids), len(set(crosswalk_ids)))
        self.assertEqual(
            set(crosswalk_ids),
            set(requirements),
            "all and only the 80 catalogued Profiles §6 requirement IDs need a reviewed row",
        )

        candidates = {
            candidate["clause_id"]: candidate
            for candidate in audit_document_with_text((ROOT / SOURCE_PATH).read_bytes(), SOURCE_ID)
            if candidate["clause_id"] in clauses
        }
        reviews = {
            review["source_clause_id"]: review
            for review in fixture["source_requirement_reviews"]
            if review["source_id"] == SOURCE_ID and review["source_section"] in PROFILE_SECTIONS
        }
        self.assertEqual(set(reviews), set(clauses))

        row_by_id = {row["requirement_id"]: row for row in crosswalk_rows}
        for requirement_id, requirement in requirements.items():
            clause_id = requirement["source_clause_ids"][0]
            clause = clauses[clause_id]
            section = clause["section"]
            expected_role = "both" if section == "6" else "client"
            expected_actor = (
                "KMIP implementation conformant to this specification"
                if section == "6"
                else "client"
            )
            expected_applies_to = "all_client_profiles" if section == "6" else PROFILE_BY_SECTION[section]
            expected_discrepancy = {
                "6.13": "KMIPKIT-DISC-011",
                "6.14": "KMIPKIT-DISC-012",
            }.get(section)
            expected_client_ids = [requirement_id]
            candidate = candidates[clause_id]
            row = row_by_id[requirement_id]
            review = reviews[clause_id]

            with self.subTest(requirement_id=requirement_id):
                self.assertEqual(row["source_clause_id"], clause_id)
                self.assertEqual(row["source_section"], section)
                self.assertEqual(row["source_locator"], clause["locator"])
                self.assertEqual(row["source_obligation_order"], 1)
                self.assertEqual(row["source_keyword"], "SHALL")
                self.assertEqual(row["role"], expected_role)
                self.assertEqual(row["source_actor"], expected_actor)
                self.assertEqual(row["source_direction"], "unspecified")
                self.assertEqual(row["scope_state"], "profile_conditional")
                self.assertEqual(row["applies_to"], expected_applies_to)
                if section == "6":
                    self.assertEqual(row["role"], "both")
                    self.assertIn("does not name endpoints", row["review_note"])
                    self.assertIn("eligibility interpretation", row["review_note"])
                self.assertEqual(
                    row.get("discrepancy_ids"),
                    [expected_discrepancy] if expected_discrepancy else None,
                )

                self.assertEqual(review["source_path"], SOURCE_PATH)
                self.assertEqual(review["source_locator"], clause["locator"])
                if section == "6":
                    expected_excerpt = pinned_source_excerpt(SOURCE_PATH, [10467, 10469])
                    self.assertEqual(review["source_excerpt"], expected_excerpt)
                    self.assertIn(review["source_excerpt"], candidate["source_text"])
                else:
                    self.assertEqual(review["source_excerpt"], candidate["source_text"])
                self.assertEqual(
                    pinned_source_excerpt(SOURCE_PATH, review["html_line_range"]),
                    review["source_excerpt"],
                )
                self.assertEqual(review["source_keywords"], ["SHALL"])
                self.assertEqual(review["applies_to"], expected_applies_to)
                self.assertEqual(review["expected_client_requirement_ids"], expected_client_ids)
                self.assertEqual(
                    review.get("discrepancy_ids"),
                    [expected_discrepancy] if expected_discrepancy else None,
                )
                self.assertEqual(len(review["source_obligations"]), 1)
                atom = review["source_obligations"][0]
                expected_text = (
                    "A KMIP implementation conformant to this specification (the Key Management "
                    "Interoperability Protocol Profiles) SHALL meet all the conditions documented "
                    "in one or more of the following sections."
                    if section == "6"
                    else candidate["source_text"]
                )
                self.assertEqual(atom["source_order"], 1)
                self.assertEqual(atom["source_text"], expected_text)
                self.assertEqual(atom["source_keyword"], "SHALL")
                self.assertEqual(atom["actor"], expected_actor)
                self.assertEqual(atom["direction"], "unspecified")
                self.assertEqual(atom["condition"], clause["condition"])
                self.assertEqual(atom["scope_disposition"], "profile_conditional")
                self.assertEqual(atom["candidate_requirement_id"], requirement_id)
                if section != "6":
                    self.assertEqual(review["source_context"], "KMIP client implementations conformant to this profile:")
                else:
                    self.assertIn("does not name endpoints", review["review_note"])
                    self.assertIn("eligibility interpretation", review["review_note"])

    def test_foundry_source_reference_defects_keep_literal_conditional_scope_and_no_label(self) -> None:
        fixture = _load_json("tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json")
        expected = {
            "KMIPKIT-SOURCE-GAP-PROF-6.13-10848": (
                "6.13",
                [10848, 10849],
                "KMIPKIT-DISC-011",
                "5.7.1",
            ),
            "KMIPKIT-SOURCE-GAP-PROF-6.14-10867": (
                "6.14",
                [10867, 10868],
                "KMIPKIT-DISC-012",
                "5.7.1",
            ),
        }
        gaps = {
            gap["gap_id"]: gap
            for gap in fixture["source_only_requirement_gaps"]
            if gap["gap_id"] in expected
        }
        self.assertEqual(set(gaps), set(expected))
        for gap_id, (section, line_range, discrepancy_id, cited_section) in expected.items():
            gap = gaps[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(gap["source_id"], SOURCE_ID)
                self.assertEqual(gap["source_section"], section)
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(gap["actor"], "client")
                self.assertEqual(gap["scope_disposition"], "profile_conditional")
                self.assertIsNone(gap["source_clause_id"])
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertEqual(gap["source_excerpt"], "SHALL support the Basic Symmetric Key Foundry Client conditions (5.7.1) and;")
                self.assertIn(cited_section, gap["source_excerpt"])
                self.assertIn(discrepancy_id, gap["actor_scope_review"])


if __name__ == "__main__":
    unittest.main()
