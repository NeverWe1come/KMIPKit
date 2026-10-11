"""Source-review regressions for Profiles v2.1 section 5 requirement atoms."""

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
    "5.3.1": "KMIPKIT-PROFILE-HTTPS-CLIENT",
    "5.6.1": "KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-CLIENT",
    "5.7.1": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-BASIC-CLIENT",
    "5.7.2": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
    "5.7.3": "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
    "5.8.1": "KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-CLIENT",
    "5.9.1": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-CLIENT",
    "5.9.2": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-CLIENT",
    "5.9.3": "KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-CLIENT",
    "5.10.1": "KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-CLIENT",
    "5.11.1": "KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-CLIENT",
    "5.12.2": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.12.3": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.12.4": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.12.6.1": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.12.6.2": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.12.6.3": "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT",
    "5.13.1": "KMIPKIT-PROFILE-AES-XTS-CLIENT",
    "5.15": "KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT",
    "5.17": "KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT",
    "5.17.1": "KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT",
    "5.17.2": "KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT",
    "5.18.1": "KMIPKIT-PROFILE-PKCS11-CLIENT",
    "5.18.3": "KMIPKIT-PROFILE-PKCS11-CLIENT",
}
OUT_OF_SCOPE_IDS = {
    "KMIPKIT-REQ-PROF-5.18.1-004",
    "KMIPKIT-REQ-PROF-5.18.1-005",
    "KMIPKIT-REQ-PROF-5.18.3-001",
    "KMIPKIT-REQ-PROF-5.18.3-002",
    "KMIPKIT-REQ-PROF-5.18.3-003",
    "KMIPKIT-REQ-PROF-5.18.3-005",
}
UNRESOLVED_ACTOR_IDS = {
    "KMIPKIT-REQ-PROF-5.12.2-002",
    "KMIPKIT-REQ-PROF-5.12.2-003",
    "KMIPKIT-REQ-PROF-5.12.6.2-002",
    "KMIPKIT-REQ-PROF-5.12.6.2-003",
    "KMIPKIT-REQ-PROF-5.12.6.3-001",
    "KMIPKIT-REQ-PROF-5.12.6.3-004",
    "KMIPKIT-REQ-PROF-5.17.1-001",
    "KMIPKIT-REQ-PROF-5.17.1-002",
    "KMIPKIT-REQ-PROF-5.17.2-002",
}
SHARED_ACTOR = {
    "KMIPKIT-REQ-PROF-5.12.6.1-001": "tape library clients and servers",
    "KMIPKIT-REQ-PROF-5.17-001": "a client or server conformant to this profile",
}
UNRESOLVED_SOURCE_ACTOR = {
    "KMIPKIT-REQ-PROF-5.12.2-002": "source does not assign a client or server actor",
    "KMIPKIT-REQ-PROF-5.12.2-003": "source does not assign a client or server actor",
    "KMIPKIT-REQ-PROF-5.12.6.2-002": "actor not explicit in this test-case statement",
    "KMIPKIT-REQ-PROF-5.12.6.2-003": "implementation actor not identified by this statement",
    "KMIPKIT-REQ-PROF-5.12.6.3-001": "source says Tape Library; client/server endpoint is not specified",
    "KMIPKIT-REQ-PROF-5.12.6.3-004": "Tape Library implementation; client/server endpoint is not specified",
    "KMIPKIT-REQ-PROF-5.17.1-001": "test-case endpoint is not specified by this clause",
    "KMIPKIT-REQ-PROF-5.17.1-002": "test-case endpoint is not specified by this clause",
    "KMIPKIT-REQ-PROF-5.17.2-002": "test-case endpoint is not specified by this clause",
}
HTML_LINES = {
    ("5.3.1", 1): [5131, 5132],
    ("5.3.1", 2): [5133, 5133],
    ("5.3.1", 3): [5134, 5134],
    ("5.3.1", 4): [5135, 5136],
    ("5.3.1", 5): [5137, 5138],
    ("5.3.1", 8): [5143, 5143],
    ("5.3.1", 9): [5144, 5144],
    ("5.3.1", 10): [5145, 5146],
    ("5.6.1", 1): [7049, 7049],
    ("5.6.1", 2): [7050, 7051],
    ("5.6.1", 3): [7052, 7054],
    ("5.7.1", 1): [7214, 7214],
    ("5.7.1", 2): [7215, 7216],
    ("5.7.1", 3): [7217, 7219],
    ("5.7.2", 1): [7231, 7231],
    ("5.7.2", 2): [7232, 7233],
    ("5.7.2", 3): [7234, 7236],
    ("5.7.3", 1): [7248, 7248],
    ("5.7.3", 2): [7249, 7250],
    ("5.7.3", 3): [7251, 7253],
    ("5.8.1", 1): [7497, 7497],
    ("5.8.1", 2): [7498, 7499],
    ("5.8.1", 3): [7500, 7502],
    ("5.9.1", 1): [7671, 7671],
    ("5.9.1", 2): [7672, 7673],
    ("5.9.1", 3): [7678, 7679],
    ("5.9.1", 4): [7680, 7682],
    ("5.9.2", 1): [7694, 7694],
    ("5.9.2", 2): [7695, 7696],
    ("5.9.2", 3): [7708, 7709],
    ("5.9.2", 4): [7710, 7712],
    ("5.9.3", 1): [7724, 7724],
    ("5.9.3", 2): [7725, 7726],
    ("5.9.3", 3): [7731, 7732],
    ("5.9.3", 4): [7733, 7735],
    ("5.10.1", 1): [8183, 8183],
    ("5.10.1", 2): [8184, 8185],
    ("5.10.1", 3): [8186, 8188],
    ("5.11.1", 1): [8296, 8296],
    ("5.11.1", 2): [8297, 8298],
    ("5.11.1", 3): [8299, 8300],
    ("5.11.1", 4): [8301, 8303],
    ("5.12.2", 2): [8548, 8550],
    ("5.12.2", 3): [8552, 8554],
    ("5.12.2", 4): [8593, 8598],
    ("5.12.2", 5): [8600, 8602],
    ("5.12.2", 7): [8630, 8633],
    ("5.12.3", 1): [8642, 8646],
    ("5.12.3", 3): [8665, 8669],
    ("5.12.4", 1): [8680, 8680],
    ("5.12.4", 2): [8681, 8683],
    ("5.12.4", 3): [8684, 8685],
    ("5.12.4", 4): [8686, 8687],
    ("5.12.4", 5): [8688, 8688],
    ("5.12.4", 6): [8699, 8700],
    ("5.12.4", 7): [8713, 8716],
    ("5.12.4", 8): [8717, 8718],
    ("5.12.4", 9): [8719, 8721],
    ("5.12.6.1", 1): [8899, 8904],
    ("5.12.6.2", 2): [8920, 8920],
    ("5.12.6.2", 3): [8930, 8931],
    ("5.12.6.2", 4): [8933, 8936],
    ("5.12.6.3", 1): [8945, 8953],
    ("5.12.6.3", 2): [8955, 8960],
    ("5.12.6.3", 3): [8962, 8965],
    ("5.12.6.3", 4): [8967, 8969],
    ("5.13.1", 1): [8989, 8989],
    ("5.13.1", 3): [8992, 8994],
    ("5.15", 1): [9106, 9106],
    ("5.15", 2): [9109, 9112],
    ("5.15", 3): [9115, 9116],
    ("5.15", 4): [9119, 9122],
    ("5.17", 1): [9380, 9381],
    ("5.17.1", 1): [9391, 9392],
    ("5.17.1", 2): [9394, 9395],
    ("5.17.2", 2): [9413, 9414],
    ("5.18.1", 4): [9468, 9475],
    ("5.18.1", 5): [9504, 9507],
    ("5.18.3", 1): [10404, 10404],
    ("5.18.3", 2): [10405, 10405],
    ("5.18.3", 3): [10406, 10407],
    ("5.18.3", 5): [10413, 10415],
}


def _load_json(relative_path: str) -> dict[str, object]:
    return json.loads((ROOT / relative_path).read_text(encoding="utf-8"))


class ProfileSectionFiveCrosswalkTests(unittest.TestCase):
    def test_catalogued_section_five_atoms_have_reviewed_actor_and_scope_dispositions(self) -> None:
        catalog = _load_json("specification/catalog/kmip-2.1.json")
        fixture = _load_json("tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json")
        clauses = {
            clause["clause_id"]: clause
            for clause in catalog["source_clauses"]
            if clause["source_id"] == SOURCE_ID and clause["section"].startswith("5.")
        }
        requirements = {
            requirement["requirement_id"]: requirement
            for requirement in catalog["requirements"]
            if any(clause_id in clauses for clause_id in requirement["source_clause_ids"])
        }
        self.assertEqual(len(requirements), 84)

        expected_mappable_ids = set(requirements) - OUT_OF_SCOPE_IDS - UNRESOLVED_ACTOR_IDS
        self.assertEqual(len(expected_mappable_ids), 69)
        crosswalk_rows = [
            row
            for row in fixture["requirement_crosswalk"]
            if row["source_id"] == SOURCE_ID and row["source_section"].startswith("5.")
        ]
        row_by_id = {row["requirement_id"]: row for row in crosswalk_rows}
        self.assertEqual(len(row_by_id), len(crosswalk_rows))
        self.assertEqual(
            set(row_by_id),
            expected_mappable_ids,
            "only the 69 reviewed client/shared §5 atoms belong in the crosswalk",
        )

        reviews = {
            review["source_clause_id"]: review
            for review in fixture["source_requirement_reviews"]
            if review["source_id"] == SOURCE_ID and review["source_section"].startswith("5.")
        }
        self.assertEqual(set(reviews), set(clauses) & {req_clause for req in requirements.values() for req_clause in req["source_clause_ids"]})
        candidates = {
            candidate["clause_id"]: candidate
            for candidate in audit_document_with_text((ROOT / SOURCE_PATH).read_bytes(), SOURCE_ID)
            if candidate["clause_id"] in clauses
        }
        self.assertEqual(set(candidates), set(clauses))

        requirements_by_clause: dict[str, list[dict[str, object]]] = {}
        for requirement in requirements.values():
            requirements_by_clause.setdefault(requirement["source_clause_ids"][0], []).append(requirement)

        for clause_id, clause_requirements in requirements_by_clause.items():
            clause = clauses[clause_id]
            section = clause["section"]
            candidate = candidates[clause_id]
            review = reviews[clause_id]
            line_range = HTML_LINES[(section, clause["locator"]["ordinal"])]
            requirement_ids = {requirement["requirement_id"] for requirement in clause_requirements}
            expected_clause_client_ids = sorted(requirement_ids & expected_mappable_ids)

            with self.subTest(clause_id=clause_id):
                self.assertEqual(candidate["section"], section)
                self.assertEqual(candidate["locator"], clause["locator"])
                self.assertEqual(review["source_path"], SOURCE_PATH)
                self.assertEqual(review["source_locator"], clause["locator"])
                self.assertEqual(review["html_line_range"], line_range)
                self.assertEqual(
                    pinned_source_excerpt(SOURCE_PATH, line_range),
                    review["source_excerpt"],
                )
                self.assertEqual(review["source_excerpt"], candidate["source_text"])
                self.assertEqual(review["source_keywords"], candidate["source_keywords"])
                self.assertEqual(review["applies_to"], PROFILE_BY_SECTION[section])
                self.assertEqual(review["expected_client_requirement_ids"], expected_clause_client_ids)

                reviewed_ids = {
                    atom.get("candidate_requirement_id")
                    for atom in review["source_obligations"]
                    if atom.get("candidate_requirement_id")
                }
                self.assertEqual(
                    reviewed_ids,
                    requirement_ids,
                    "source review must identify every and only catalogued atom for this clause",
                )

                for requirement in clause_requirements:
                    requirement_id = requirement["requirement_id"]
                    source_atoms = [
                        atom
                        for atom in review["source_obligations"]
                        if atom.get("candidate_requirement_id") == requirement_id
                    ]
                    self.assertTrue(source_atoms)
                    source_atom = source_atoms[0]
                    self.assertIn(source_atom["source_keyword"], candidate["source_keywords"])
                    if requirement_id in expected_mappable_ids:
                        row = row_by_id[requirement_id]
                        expected_role = "both" if requirement_id in SHARED_ACTOR else "client"
                        expected_actor = SHARED_ACTOR.get(requirement_id, "client")
                        self.assertEqual(row["source_clause_id"], clause_id)
                        self.assertEqual(row["source_section"], section)
                        self.assertEqual(row["source_locator"], clause["locator"])
                        self.assertEqual(row["source_keyword"], requirement["source_keyword"])
                        self.assertEqual(row["role"], expected_role)
                        self.assertEqual(row["scope_state"], "profile_conditional")
                        self.assertEqual(row["applies_to"], PROFILE_BY_SECTION[section])
                        self.assertEqual(row["source_obligation_order"], source_atom["source_order"])
                        self.assertEqual(row["source_actor"], expected_actor)
                        self.assertEqual(source_atom["actor"], expected_actor)
                        self.assertEqual(source_atom["scope_disposition"], "profile_conditional")
                        if requirement_id in SHARED_ACTOR:
                            self.assertEqual(row["source_direction"], "unspecified")
                    elif requirement_id in OUT_OF_SCOPE_IDS:
                        self.assertEqual(source_atom["scope_disposition"], "out_of_scope")
                        self.assertEqual(source_atom["actor"], "client")
                        self.assertNotIn(requirement_id, row_by_id)
                    else:
                        self.assertIn(requirement_id, UNRESOLVED_ACTOR_IDS)
                        self.assertEqual(source_atom["scope_disposition"], "unresolved")
                        self.assertEqual(source_atom["actor"], UNRESOLVED_SOURCE_ACTOR[requirement_id])
                        self.assertNotIn(requirement_id, row_by_id)

    def test_ttlv_http_atoms_and_literal_internal_reference_defects_remain_linked(self) -> None:
        fixture = _load_json("tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json")
        rows = {
            row["requirement_id"]: row
            for row in fixture["requirement_crosswalk"]
            if row["source_id"] == SOURCE_ID and row["source_section"] == "5.3.1"
        }
        self.assertEqual(
            set(rows),
            {
                "KMIPKIT-REQ-PROF-5.3.1-001",
                "KMIPKIT-REQ-PROF-5.3.1-002",
                "KMIPKIT-REQ-PROF-5.3.1-003",
                "KMIPKIT-REQ-PROF-5.3.1-004",
                "KMIPKIT-REQ-PROF-5.3.1-005",
                "KMIPKIT-REQ-PROF-5.3.1-008",
                "KMIPKIT-REQ-PROF-5.3.1-009",
                "KMIPKIT-REQ-PROF-5.3.1-010",
            },
        )
        self.assertEqual(rows["KMIPKIT-REQ-PROF-5.3.1-005"]["source_locator"]["ordinal"], 5)
        self.assertEqual(rows["KMIPKIT-REQ-PROF-5.3.1-010"]["source_locator"]["ordinal"], 10)
        self.assertEqual(rows["KMIPKIT-REQ-PROF-5.3.1-005"]["source_direction"], "client_to_server")
        self.assertEqual(rows["KMIPKIT-REQ-PROF-5.3.1-010"]["source_direction"], "client_to_server")

        defects = {
            "KMIPKIT-SOURCE-GAP-PROF-5.13.1-8990": ("KMIPKIT-DISC-007", "5.10.1"),
            "KMIPKIT-SOURCE-GAP-PROF-5.18.3-10411": ("KMIPKIT-DISC-008", "5.5.2"),
        }
        gaps = {gap["gap_id"]: gap for gap in fixture["source_only_requirement_gaps"]}
        for gap_id, (discrepancy_id, literal_reference) in defects.items():
            with self.subTest(gap_id=gap_id):
                self.assertIn(gap_id, gaps)
                gap = gaps[gap_id]
                self.assertIn(literal_reference, gap["source_excerpt"])
                self.assertIn(discrepancy_id, gap.get("discrepancy_ids", []))

        for section, discrepancy_id in (("5.13.1", "KMIPKIT-DISC-007"), ("5.18.3", "KMIPKIT-DISC-008")):
            section_rows = [
                row
                for row in fixture["requirement_crosswalk"]
                if row["source_id"] == SOURCE_ID and row["source_section"] == section
            ]
            for row in section_rows:
                self.assertIn(discrepancy_id, row.get("discrepancy_ids", []))


if __name__ == "__main__":
    unittest.main()
