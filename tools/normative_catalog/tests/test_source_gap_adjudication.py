"""Source-gap decisions are checked against the independent profile oracle."""

from __future__ import annotations

import html
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SPEC_SOURCE_PATH = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"


AUDITED_REQUIREMENTS = {
    "KMIPKIT-SOURCE-GAP-SPEC-3.1-KEY-FORMAT-TYPE": "KMIPKIT-REQ-SPEC-3.1-007",
    "KMIPKIT-SOURCE-GAP-SPEC-3.2-KEY-MATERIAL": "KMIPKIT-REQ-SPEC-3.2-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.3-WRAPPING-METHOD": "KMIPKIT-REQ-SPEC-3.3-010",
    "KMIPKIT-SOURCE-GAP-SPEC-3.3-UNIQUE-IDENTIFIER": "KMIPKIT-REQ-SPEC-3.3-011",
    "KMIPKIT-SOURCE-GAP-SPEC-3.4-KEY": "KMIPKIT-REQ-SPEC-3.4-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.5-P": "KMIPKIT-REQ-SPEC-3.5-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.5-Q": "KMIPKIT-REQ-SPEC-3.5-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.5-G": "KMIPKIT-REQ-SPEC-3.5-004",
    "KMIPKIT-SOURCE-GAP-SPEC-3.5-X": "KMIPKIT-REQ-SPEC-3.5-005",
    "KMIPKIT-SOURCE-GAP-SPEC-3.6-P": "KMIPKIT-REQ-SPEC-3.6-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.6-G": "KMIPKIT-REQ-SPEC-3.6-004",
    "KMIPKIT-SOURCE-GAP-SPEC-3.6-Y": "KMIPKIT-REQ-SPEC-3.6-005",
    "KMIPKIT-SOURCE-GAP-SPEC-3.7-MODULUS": "KMIPKIT-REQ-SPEC-3.7-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.8-MODULUS": "KMIPKIT-REQ-SPEC-3.8-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.8-PUBLIC-EXPONENT": "KMIPKIT-REQ-SPEC-3.8-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.9-P": "KMIPKIT-REQ-SPEC-3.9-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.9-G": "KMIPKIT-REQ-SPEC-3.9-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.9-X": "KMIPKIT-REQ-SPEC-3.9-004",
    "KMIPKIT-SOURCE-GAP-SPEC-3.10-P": "KMIPKIT-REQ-SPEC-3.10-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.10-G": "KMIPKIT-REQ-SPEC-3.10-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.10-Y": "KMIPKIT-REQ-SPEC-3.10-004",
    "KMIPKIT-SOURCE-GAP-SPEC-3.11-RECOMMENDED-CURVE": "KMIPKIT-REQ-SPEC-3.11-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.11-D": "KMIPKIT-REQ-SPEC-3.11-003",
    "KMIPKIT-SOURCE-GAP-SPEC-3.12-RECOMMENDED-CURVE": "KMIPKIT-REQ-SPEC-3.12-002",
    "KMIPKIT-SOURCE-GAP-SPEC-3.12-Q-STRING": "KMIPKIT-REQ-SPEC-3.12-003",
    "KMIPKIT-SOURCE-GAP-SPEC-5.5-VENDOR-IDENTIFICATION": "KMIPKIT-REQ-SPEC-5.5-003",
    "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-NAME": "KMIPKIT-REQ-SPEC-5.5-004",
    "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-REFERENCE-TAG": "KMIPKIT-REQ-SPEC-5.5-005",
    "KMIPKIT-SOURCE-GAP-SPEC-5.6-CURRENT-ATTRIBUTE": "KMIPKIT-REQ-SPEC-5.6-002",
    "KMIPKIT-SOURCE-GAP-SPEC-5.7-NEW-ATTRIBUTE": "KMIPKIT-REQ-SPEC-5.7-002",
}

PROCEDURE_GAP_CONTEXT = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-011": ([22429, 22431], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-012": ([22437, 22439], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-013": ([22441, 22443], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-014": ([22449, 22451], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-015": ([22457, 22459], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-016": ([22465, 22467], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-017": ([22469, 22471], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-018": ([22473, 22475], "multi-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-019": ([22485, 22488], "single-part"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-020": ([22494, 22496], "single-part"),
}
ADJUDICATED_SOURCE_GAPS = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-MULTIPART-OPTIONAL-PREFACE": ([22417, 22420], "reviewed_optional_client_capability_context"),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009": ([22498, 22499], "reviewed_table_precedence"),
    "KMIPKIT-SOURCE-GAP-SPEC-9.17-001": ([45156, 45158], "reviewed_mixed_response_and_client_access"),
}
OUTSIDE_CLIENT_1_0_SOURCE_GAPS = frozenset(
    {
        "KMIPKIT-SOURCE-GAP-PROF-3.1.1-4009",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.1-4023",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.2-4060",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.2-4076",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.2-4089",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4196",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4200",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4204",
        "KMIPKIT-SOURCE-GAP-PROF-3.1.4-4217",
        "KMIPKIT-SOURCE-GAP-PROF-3.2.4-4260",
        "KMIPKIT-SOURCE-GAP-PROF-5.3.1-5139",
        "KMIPKIT-SOURCE-GAP-PROF-5.3.1-5141",
        "KMIPKIT-SOURCE-GAP-PROF-5.3.1-5149",
        "KMIPKIT-SOURCE-GAP-PROF-5.4.2-5980",
        "KMIPKIT-SOURCE-GAP-PROF-5.5.2-6613",
        "KMIPKIT-SOURCE-GAP-PROF-5.12.3-8658",
        "KMIPKIT-SOURCE-GAP-PROF-5.18.3-10411",
        "KMIPKIT-SOURCE-GAP-PROF-6.6-10708",
        "KMIPKIT-SOURCE-GAP-PROF-6.8-10746",
        "KMIPKIT-SOURCE-GAP-SPEC-8.4-002",
        "KMIPKIT-SOURCE-GAP-SPEC-8.5-002",
        "KMIPKIT-SOURCE-GAP-SPEC-8.6-002",
        "KMIPKIT-SOURCE-GAP-SPEC-8.6-003",
        "KMIPKIT-SOURCE-GAP-SPEC-8.6-004",
        "KMIPKIT-SOURCE-GAP-SPEC-8.6-005",
        "KMIPKIT-SOURCE-GAP-SPEC-4.28-001-SERVER-NONMUTATION",
        "KMIPKIT-SOURCE-GAP-SPEC-6.1.42-RECOVER-AUTHENTICATION-SHOULD-ACTOR-UNRESOLVED",
        "KMIPKIT-SOURCE-GAP-SPEC-6.1.54-ENDPOINT-ROLE-REQUIRED",
    }
)


def _load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _normalized_html_source_span(lines: list[str], line_range: list[int]) -> str:
    source_span = " ".join(lines[line_range[0] - 1 : line_range[1]])
    return " ".join(html.unescape(re.sub(r"<[^>]+>", " ", source_span)).split())


class SourceGapAdjudicationTests(unittest.TestCase):
    def test_section_4_21_lowercase_request_reuses_only_the_section_4_26_capability(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        reviews = {row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        requirement_id = "KMIPKIT-REQ-SPEC-4.26-001"
        gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-4.21-CLIENT-LOWERCASE-MAY"]
        review = reviews["KMIPKIT-CLAUSE-SPEC-4.21-001"]
        client_atom, server_atom = review["source_obligations"][4:6]
        requirement = requirements[requirement_id]

        self.assertEqual(gap["html_line_range"], [13896, 13897])
        self.assertEqual(gap["source_force"], "lowercase_may_not_formal_keyword")
        self.assertEqual(gap["source_keywords"], [])
        self.assertEqual(
            gap["source_excerpt"],
            "object type and algorithm. Clients may request via supplying a non-default Key Format Value attribute on operations that create a Managed Object, and the",
        )
        self.assertEqual(gap["scope_disposition"], "client_1_0")
        self.assertEqual(gap["candidate_requirement_id"], requirement_id)
        self.assertEqual(gap["reused_requirement_id"], requirement_id)
        self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")
        self.assertIsNone(gap["proposed_requirement_id"])
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertEqual(client_atom["actor"], "client")
        self.assertIsNone(client_atom["candidate_requirement_id"])
        self.assertEqual(client_atom["source_gap_id"], gap["gap_id"])
        self.assertEqual(client_atom["source_keyword"], "may (lowercase; not a §1.2 formal keyword)")
        self.assertEqual(server_atom["actor"], "server")
        self.assertEqual(server_atom["source_keyword"], "SHALL")
        self.assertEqual(server_atom["scope_disposition"], "server_only")
        self.assertIsNone(server_atom["candidate_requirement_id"])
        self.assertEqual(
            requirement["condition"],
            "The server produces cryptographic material for the client using Create, Create Key Pair, Create Split Key, Re-key, Re-key Key Pair, or Derive Key.",
        )
        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-4.26-001"])
        self.assertNotIn("Register", requirement["condition"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertEqual(
            requirement["verification_refs"],
            [
                "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_requested_key_format_for_server_generated_material_operations",
                "crates/kmipkit-protocol/tests/unit/create_tests.rs::request_encodes_raw_key_format_for_server_generated_create",
            ],
        )
        self.assertIn(
            "The client evaluator checks the request-side Attribute or key-pair attribute groups",
            requirement["review_note"],
        )
        self.assertIn("no server acceptance", requirement["review_note"].lower())

        catalog_profiles = [row for row in catalog["profiles"] if row["role"] == "client"]
        oracle_profiles = [row for row in oracle["profiles"] if row["role"] == "client"]
        self.assertEqual(len(catalog_profiles), 18)
        self.assertEqual(len(oracle_profiles), 18)
        self.assertTrue(all(requirement_id in row["requirement_ids"] for row in catalog_profiles))
        self.assertTrue(all(requirement_id in row["requirement_ids"] for row in oracle_profiles))

    def test_section_4_24_fresh_initial_input_reuses_generic_attribute_may(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        reviews = {row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        gap_id = "KMIPKIT-SOURCE-GAP-SPEC-4.24-CLIENT-FRESH-INPUT-EXCEPTION"
        requirement_id = "KMIPKIT-REQ-SPEC-6.1-004"
        gap = gaps[gap_id]
        review = reviews["KMIPKIT-CLAUSE-SPEC-4.24-001"]
        client_atom = review["source_obligations"][1]
        requirement = requirements[requirement_id]

        self.assertEqual(gap["html_line_range"], [14444, 14449])
        self.assertEqual(gap["source_force"], "client_input_exception_without_formal_modal")
        self.assertEqual(gap["source_keywords"], ["SHALL"])
        self.assertEqual(gap["actor"], "client")
        self.assertEqual(gap["direction"], "Register or Import request supplies Fresh=False")
        self.assertEqual(gap["condition"], "new object creation through Register or Import")
        self.assertEqual(gap["scope_disposition"], "client_1_0")
        self.assertEqual(gap["candidate_requirement_id"], requirement_id)
        self.assertEqual(gap["reused_requirement_id"], requirement_id)
        self.assertEqual(gap["reuse_kind"], "contextual_capability")
        self.assertIsNone(gap["proposed_requirement_id"])
        self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertEqual(client_atom["source_order"], 2)
        self.assertEqual(client_atom["actor"], "client")
        self.assertEqual(client_atom["source_keyword"], "client exception to server SHALL")
        self.assertEqual(client_atom["scope_disposition"], "client_1_0")
        self.assertIsNone(client_atom["candidate_requirement_id"])
        self.assertEqual(client_atom["source_gap_id"], gap_id)
        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-6.1-004"])
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertIn("Create-only", requirement["review_note"])
        self.assertIn("Table 76", gap["review_note"])
        self.assertIn("initial", gap["review_note"].lower())

    def test_section_4_31_absent_link_may_has_scoped_register_evidence(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        reviews = {row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]}
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        requirement_id = "KMIPKIT-REQ-SPEC-4.31-002"
        gap_id = "KMIPKIT-SOURCE-GAP-SPEC-4.31-MAY-ABSENT-LINKS"
        gap = gaps[gap_id]
        review = reviews["KMIPKIT-CLAUSE-SPEC-4.31-002"]
        atom = review["source_obligations"][0]
        self.assertTrue(
            requirement_id in requirements,
            "the client absent-Link MAY needs its own stable requirement",
        )
        requirement = requirements[requirement_id]

        self.assertEqual(gap["source_clause_id"], "KMIPKIT-CLAUSE-SPEC-4.31-002")
        self.assertEqual(gap["html_line_range"], [15815, 15819])
        self.assertEqual(
            gap["source_excerpt"],
            "It is also possible that a Managed Object does not have links to associated cryptographic objects. "
            "This MAY occur in cases where the associated key material is not available to the server or client "
            "(e.g., the registration of a CA Signer certificate with a server, where the corresponding private "
            "key is held in a different manner).",
        )
        self.assertEqual(gap["source_keywords"], ["MAY"])
        self.assertEqual(gap["source_force"], "MAY")
        self.assertEqual(gap["actor"], "client")
        self.assertEqual(gap["direction"], "unclear")
        self.assertEqual(
            gap["condition"],
            "associated cryptographic key material is unavailable to the server or client",
        )
        self.assertEqual(gap["scope_disposition"], "client_1_0")
        self.assertEqual(gap["candidate_requirement_id"], requirement_id)
        self.assertIsNone(gap["proposed_requirement_id"])
        self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")

        self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
        self.assertEqual(atom["source_keyword"], "MAY")
        self.assertEqual(atom["actor"], "client")
        self.assertEqual(atom["scope_disposition"], "client_1_0")
        self.assertEqual(atom["candidate_requirement_id"], requirement_id)

        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertEqual(requirement["normative_strength"], "permission_or_optional")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "unclear")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertEqual(
            requirement["condition"],
            "associated cryptographic key material is unavailable to the server or client",
        )
        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-4.31-002"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertEqual(
            requirement["implementation_refs"],
            [
                "crates/kmipkit-protocol/src/register.rs",
                "crates/kmipkit-client/src/execute.rs",
                "crates/kmipkit-protocol/src/profile.rs::client_can_omit_link_when_associated_key_material_is_unavailable",
                "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
            ],
        )
        self.assertEqual(
            requirement["verification_refs"],
            [
                "crates/kmipkit-client/tests/unit/register_execution_tests.rs::"
                "register_client_omits_signature_algorithm_when_x509_certificate_encodes_it",
                "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                "profile_maps_optional_absent_link_for_unavailable_associated_key_material",
            ],
        )
        self.assertEqual(requirement["feature_spec"], "KMIPKIT-0036")
        self.assertEqual(requirement["status"], "verified")
        self.assertIn("Link remains absent", requirement["review_note"])
        self.assertIn("does not assign a message direction", requirement["review_note"])
        self.assertIn("every operation", requirement["review_note"])

        clause = clauses["KMIPKIT-CLAUSE-SPEC-4.31-002"]
        self.assertEqual(clause["disposition"], "requirement")
        self.assertEqual(clause["requirement_ids"], [requirement_id])
        self.assertEqual(clause["role"], "client")
        self.assertEqual(clause["direction"], "unclear")
        self.assertEqual(clause["scope_state"], "client_1_0")
        self.assertEqual(clauses["KMIPKIT-CLAUSE-SPEC-4.31-001"]["requirement_ids"], [])

        server_should = reviews["KMIPKIT-CLAUSE-SPEC-4.31-001"]["source_obligations"][0]
        self.assertEqual(server_should["source_keyword"], "SHOULD")
        self.assertEqual(server_should["actor"], "server")
        self.assertEqual(server_should["scope_disposition"], "server_only")
        self.assertIsNone(server_should["candidate_requirement_id"])

        header_requirement = requirements["KMIPKIT-REQ-SPEC-4.31-003"]
        self.assertEqual(header_requirement["role"], "not_applicable")
        self.assertEqual(header_requirement["scope_state"], "out_of_scope")
        self.assertEqual(header_requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-4.31-003"])

    def test_section_4_31_table_91_required_members_have_distinct_client_mappings(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        crosswalk = {row["requirement_id"]: row for row in oracle["requirement_crosswalk"]}
        client_profiles = [row for row in catalog["profiles"] if row["role"] == "client"]
        oracle_profiles = [row for row in oracle["profiles"] if row["role"] == "client"]
        condition = (
            "A complete Link attribute is represented as object state rather than as a structured Locate search criterion."
        )
        expected_rows = {
            "KMIPKIT-SOURCE-GAP-SPEC-4.31-LINK-TYPE-REQUIRED": (
                "KMIPKIT-REQ-SPEC-4.31-004",
                "Link Type",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINK-TYPE",
                [15923, 15940],
                "Link Type Enumeration Yes",
                "KMIPKIT-TEST-CN01-2-66",
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-4.31-LINKED-OBJECT-IDENTIFIER-REQUIRED": (
                "KMIPKIT-REQ-SPEC-4.31-005",
                "Linked Object Identifier",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINKED-OBJECT-IDENTIFIER",
                [15942, 15961],
                "Linked Object Identifier Text String/Enumeration/Integer Yes",
                "KMIPKIT-TEST-CN01-2-66",
            ),
        }

        self.assertEqual(len(client_profiles), 18)
        self.assertEqual(len(oracle_profiles), 18)
        for gap_id, (requirement_id, row_label, element_id, line_range, excerpt, test_case_id) in expected_rows.items():
            with self.subTest(gap_id=gap_id):
                self.assertTrue(gap_id in gaps, f"Table 91 source row is missing: {gap_id}")
                self.assertTrue(requirement_id in requirements, f"Table 91 stable requirement ID is missing: {requirement_id}")
                self.assertTrue(requirement_id in crosswalk, f"Table 91 independent crosswalk entry is missing: {requirement_id}")
                if gap_id not in gaps or requirement_id not in requirements or requirement_id not in crosswalk:
                    continue
                gap = gaps[gap_id]
                requirement = requirements[requirement_id]
                mapping = crosswalk[requirement_id]

                self.assertIsNone(gap["source_clause_id"])
                self.assertEqual(gap["source_section"], "4.31")
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(gap["source_excerpt"], excerpt)
                self.assertEqual(gap["source_keywords"], [])
                self.assertEqual(gap["source_force"], "required_table_body_value")
                self.assertEqual(gap["actor"], "client")
                self.assertEqual(gap["direction"], "both")
                self.assertEqual(gap["condition"], condition)
                self.assertEqual(gap["scope_disposition"], "client_1_0")
                self.assertEqual(gap["candidate_requirement_id"], requirement_id)
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")
                self.assertEqual(gap["source_locator"]["kind"], "table_body_row")
                self.assertEqual(gap["source_locator"]["row_label"], row_label)
                self.assertEqual(gap["source_locator"]["html_line_range"], line_range)

                self.assertEqual(requirement["source_clause_ids"], [])
                self.assertEqual(requirement["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "4.31"}])
                self.assertIsNone(requirement["source_keyword"])
                self.assertEqual(requirement["source_force"], "required_table_body_value")
                self.assertEqual(requirement["source_gap_id"], gap_id)
                self.assertEqual(requirement["source_locator"], gap["source_locator"])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "both")
                self.assertEqual(requirement["condition"], condition)
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["element_ids"], [element_id])
                self.assertEqual(requirement["profile_ids"], [])
                self.assertEqual(requirement["test_case_ids"], [test_case_id])
                self.assertIsNone(requirement["feature_spec"])
                expected_implementation_refs = {
                    "KMIPKIT-REQ-SPEC-4.31-004": [
                        "crates/kmipkit-protocol/src/profile.rs::client_link_type_is_required_in_object_state",
                        "crates/kmipkit-protocol/src/profile.rs::object_state_links_have_required_members",
                        "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
                    ],
                    "KMIPKIT-REQ-SPEC-4.31-005": [
                        "crates/kmipkit-protocol/src/profile.rs::client_linked_object_identifier_is_required_in_object_state",
                        "crates/kmipkit-protocol/src/profile.rs::object_state_links_have_required_members",
                        "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
                    ],
                }
                expected_verification_refs = [
                    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_accepts_complete_client_object_state_link_values",
                    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links",
                    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_incomplete_client_object_state_links_in_get_attributes_responses",
                    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_correlates_reordered_get_attributes_link_responses_by_batch_id",
                    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_allows_partial_link_locate_search_criteria",
                ]
                self.assertEqual(
                    requirement["implementation_refs"],
                    expected_implementation_refs[requirement_id],
                )
                self.assertEqual(requirement["verification_refs"], expected_verification_refs)
                self.assertEqual(requirement["status"], "verified")
                self.assertTrue(all(requirement_id in profile["requirement_ids"] for profile in client_profiles))
                self.assertTrue(all(requirement_id in profile["requirement_ids"] for profile in oracle_profiles))

                self.assertEqual(mapping["source_gap_id"], gap_id)
                self.assertEqual(mapping["source_id"], "KMIPKIT-SRC-spec")
                self.assertEqual(mapping["source_section"], "4.31")
                self.assertEqual(mapping["source_locator"], gap["source_locator"])
                self.assertEqual(mapping["source_force"], "required_table_body_value")
                self.assertEqual(mapping["role"], "client")
                self.assertEqual(mapping["scope_state"], "client_1_0")
                self.assertEqual(mapping["applies_to"], "all_client_profiles")
                self.assertEqual(mapping["source_actor"], "client")

                self.assertIn("Table 91", gap["review_note"])
                self.assertIn("Locate", gap["review_note"])
                self.assertIn("Get Attributes", gap["review_note"])
                self.assertIn("KMIPKIT-0010 provides request and response validation evidence", gap["review_note"])

        official_case = next(
            row for row in catalog["test_cases"] if row["test_id"] == "KMIPKIT-TEST-CN01-2-66"
        )
        self.assertEqual(official_case["official_case_id"], "TC-PGP-1-21")
        self.assertEqual(official_case["source_section"], "2.66")
        self.assertEqual(official_case["requirement_ids"], ["KMIPKIT-REQ-SPEC-4.31-004", "KMIPKIT-REQ-SPEC-4.31-005"])
        self.assertEqual(official_case["fixture_availability"], "unavailable")
        self.assertIsNone(official_case["fixture_path"])

        self.assertEqual(len(oracle["source_table_contracts"]), 68)
        self.assertEqual(oracle["counts"]["requirement_crosswalk_records"], 521)
        self.assertEqual(oracle["counts"]["source_only_requirement_gaps"], 112)

    def test_reviewed_structural_requirements_have_stable_catalog_mappings(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}

        for gap_id, requirement_id in AUDITED_REQUIREMENTS.items():
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                requirement = requirements[requirement_id]
                self.assertEqual(gap["candidate_requirement_id"], requirement_id)
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")
                self.assertEqual(requirement["source_gap_id"], gap_id)
                self.assertEqual(requirement["source_force"], "required_table_body_value")
                self.assertEqual(requirement["scope_state"], "profile_conditional")

    def test_optional_dsa_q_is_not_misclassified_as_a_required_source_gap(self) -> None:
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        self.assertNotIn("KMIPKIT-SOURCE-GAP-SPEC-3.6-Q", gaps)

    def test_current_and_new_attribute_structures_are_request_only(self) -> None:
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        for gap_id in (
            "KMIPKIT-SOURCE-GAP-SPEC-5.6-CURRENT-ATTRIBUTE",
            "KMIPKIT-SOURCE-GAP-SPEC-5.7-NEW-ATTRIBUTE",
        ):
            with self.subTest(gap_id=gap_id):
                self.assertEqual(gaps[gap_id]["actor"], "client")
                self.assertEqual(gaps[gap_id]["direction"], "client_to_server")

    def test_table_161_conditions_match_the_attribute_reference_form(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        expected_conditions = {
            "KMIPKIT-SOURCE-GAP-SPEC-5.5-VENDOR-IDENTIFICATION": (
                "KMIPKIT-REQ-SPEC-5.5-003",
                "vendor-name form of Attribute Reference",
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-NAME": (
                "KMIPKIT-REQ-SPEC-5.5-004",
                "vendor-name form of Attribute Reference",
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-REFERENCE-TAG": (
                "KMIPKIT-REQ-SPEC-5.5-005",
                "tag form of Attribute Reference",
            ),
        }

        for gap_id, (requirement_id, expected_condition) in expected_conditions.items():
            with self.subTest(gap_id=gap_id):
                self.assertEqual(gaps[gap_id]["condition"], expected_condition)
                self.assertEqual(requirements[requirement_id]["condition"], expected_condition)

    def test_attribute_structure_rows_have_scoped_feature_and_test_evidence(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        mappings = {
            "KMIPKIT-REQ-SPEC-5.5-003": (
                "KMIPKIT-SOURCE-GAP-SPEC-5.5-VENDOR-IDENTIFICATION",
                "crates/kmipkit-protocol/src/attribute_reference.rs::AttributeReference::to_ttlv_item",
                "crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs::request_preserves_name_reference_order_and_omits_identifier_and_adjustment_value",
            ),
            "KMIPKIT-REQ-SPEC-5.5-004": (
                "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-NAME",
                "crates/kmipkit-protocol/src/attribute_reference.rs::AttributeReference::to_ttlv_item",
                "crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs::request_preserves_name_reference_order_and_omits_identifier_and_adjustment_value",
            ),
            "KMIPKIT-REQ-SPEC-5.5-005": (
                "KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-REFERENCE-TAG",
                "crates/kmipkit-protocol/src/attribute_reference.rs::AttributeReference::to_ttlv_item",
                "crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs::request_follows_table_170_field_order_and_preserves_tag_reference_and_parameter",
            ),
            "KMIPKIT-REQ-SPEC-5.6-002": (
                "KMIPKIT-SOURCE-GAP-SPEC-5.6-CURRENT-ATTRIBUTE",
                "crates/kmipkit-protocol/src/modify_attribute.rs::ModifyAttributeRequest::to_ttlv_payload",
                "crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs::request_preserves_exact_current_and_new_items_in_table_265_order",
            ),
            "KMIPKIT-REQ-SPEC-5.7-002": (
                "KMIPKIT-SOURCE-GAP-SPEC-5.7-NEW-ATTRIBUTE",
                "crates/kmipkit-protocol/src/modify_attribute.rs::ModifyAttributeRequest::to_ttlv_payload",
                "crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs::request_preserves_exact_current_and_new_items_in_table_265_order",
            ),
        }

        for requirement_id, (gap_id, implementation_ref, verification_ref) in mappings.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                gap = gaps[gap_id]
                self.assertEqual(requirement["feature_spec"], "KMIPKIT-0016")
                self.assertEqual(requirement["status"], "scoped_verified")
                self.assertEqual(requirement["scope_state"], "profile_conditional")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertEqual(gap["scope_disposition"], "profile_conditional")
                self.assertIn("KMIPKIT-0016", gap["review_note"])
                self.assertNotIn("remain unassigned", gap["review_note"])
                self.assertIn(implementation_ref, requirement["implementation_refs"])
                self.assertIn(verification_ref, requirement["verification_refs"])

    def test_tag_reference_response_mapping_cites_the_decoder(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        requirement = requirements["KMIPKIT-REQ-SPEC-5.5-005"]

        self.assertIn(
            "crates/kmipkit-protocol/src/attribute_reference.rs::AttributeReference::try_from_ttlv_item",
            requirement["implementation_refs"],
        )
        self.assertIn(
            "crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs::response_preserves_tag_form_attribute_references",
            requirement["verification_refs"],
        )

    def test_data_paragraph_discrepancy_covers_all_matching_single_part_operations(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        discrepancy = next(
            row
            for row in catalog["discrepancies"]
            if row["discrepancy_id"] == "KMIPKIT-DISC-045"
        )

        source_sections = {reference["section"] for reference in discrepancy["source_refs"]}
        self.assertEqual(
            source_sections,
            {"6.1", "6.1.11", "6.1.17", "6.1.24", "6.1.32", "6.1.55"},
            "only the general Data rule and affected single-part operation tables belong",
        )
        affected_elements = set(discrepancy["affected_element_ids"])
        self.assertTrue(
            {
                "KMIPKIT-ELEM-OP-C2S-ENCRYPT",
                "KMIPKIT-ELEM-OP-C2S-DECRYPT",
                "KMIPKIT-ELEM-OP-C2S-HASH",
                "KMIPKIT-ELEM-OP-C2S-MAC",
                "KMIPKIT-ELEM-OP-C2S-SIGN",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR",
                "KMIPKIT-ELEM-TAG-420107",
            } <= affected_elements,
            "all disputed operations and request fields must be recorded",
        )
        self.assertNotIn("KMIPKIT-ELEM-OP-C2S-MAC-VERIFY", affected_elements)
        self.assertNotIn("KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY", affected_elements)
        self.assertEqual([], discrepancy["affected_requirement_ids"])
        self.assertEqual("normative_conflict", discrepancy["normative_status"])
        self.assertEqual("resolved_by_approved_decision", discrepancy["state"])
        self.assertEqual("KMIPKIT-DEC-015", discrepancy["decision_id"])
        decision = next(row for row in catalog["decisions"] if row["decision_id"] == discrepancy["decision_id"])
        self.assertEqual(decision["status"], "accepted")
        self.assertIn("Tables 196, 214, 235, 259, and 334", decision["interpretation"])
        self.assertIn("both true and Correlation Value is absent", decision["interpretation"])
        self.assertIn("both indicators are absent or false", decision["interpretation"])
        self.assertIn("omission is permitted when either indicator is true", decision["interpretation"])
        self.assertIn("table value of No does not make Data universally required", decision["interpretation"])
        self.assertIn("Digested Data", decision["interpretation"])
        conflict_text = " ".join(discrepancy["alternatives"] + [discrepancy["downstream_impact"]])
        self.assertIn("Tables 196, 214, 235, 259, and 334", conflict_text)
        self.assertIn("Digested Data", conflict_text)

    def test_section_6_1_procedure_clause_context_matches_reviewed_oracle(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        for ordinal in (6, 7, 8):
            clause = clauses[f"KMIPKIT-CLAUSE-SPEC-6.1-{ordinal:03d}"]
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{ordinal:03d}"]
            with self.subTest(clause=clause["clause_id"]):
                self.assertEqual(gap["actor"], "client")
                self.assertIn("request parameters", gap["direction"])
                self.assertEqual(gap["scope_disposition"], "client_1_0")
                self.assertEqual(clause["source_keywords"], ["REQUIRED"])
                self.assertEqual(clause["disposition"], "informative_context")
                self.assertEqual(clause["requirement_ids"], [])
                self.assertEqual(clause["role"], gap["actor"])
                self.assertEqual(clause["direction"], "client_to_server")
                self.assertEqual(clause["scope_state"], gap["scope_disposition"])
                self.assertIn("client-to-server request context", clause["exclusion_rationale"].lower())
                self.assertIn("procedure", clause["exclusion_rationale"].lower())
                self.assertIn("lowercase", clause["exclusion_rationale"].lower())
                self.assertIn("standalone requirement id", clause["exclusion_rationale"].lower())

        data_clause = clauses["KMIPKIT-CLAUSE-SPEC-6.1-009"]
        data_gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        self.assertEqual(data_gap["actor"], "client")
        self.assertIn("request parameters", data_gap["direction"])
        self.assertEqual(data_gap["scope_disposition"], "client_1_0")
        self.assertIsNone(data_gap["candidate_requirement_id"])
        self.assertIsNone(data_gap["proposed_requirement_id"])
        self.assertEqual(data_clause["source_keywords"], ["REQUIRED"])
        self.assertEqual(data_clause["disposition"], "source_discrepancy")
        self.assertEqual(data_clause["requirement_ids"], [])
        self.assertEqual(data_clause["role"], data_gap["actor"])
        self.assertEqual(data_clause["direction"], "client_to_server")
        self.assertEqual(data_clause["scope_state"], "client_1_0")
        self.assertIn("client-to-server request context", data_clause["exclusion_rationale"].lower())
        self.assertIn("lowercase", data_clause["exclusion_rationale"].lower())
        self.assertIn("decision kmipkit-dec-015", data_clause["exclusion_rationale"].lower())

    def test_profile_conditional_structural_rows_are_not_client_profile_requirements(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        crosswalk = {row["requirement_id"]: row for row in oracle["requirement_crosswalk"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        client_profiles = [row for row in catalog["profiles"] if row["role"] == "client"]
        oracle_profiles = [row for row in oracle["profiles"] if row["role"] == "client"]

        for gap_id, requirement_id in AUDITED_REQUIREMENTS.items():
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                mapping = crosswalk[requirement_id]
                requirement = requirements[requirement_id]
                self.assertEqual(gap["scope_disposition"], "profile_conditional")
                self.assertEqual(mapping["scope_state"], "profile_conditional")
                self.assertEqual(mapping["applies_to"], [])
                self.assertEqual(requirement["scope_state"], "profile_conditional")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertIn("Profiles v2.1 §5.1.1 item 7", gap["review_note"])
                self.assertIn("HTML lines 4628–4631", gap["review_note"])
                self.assertIn("Required=Yes applies only when its enclosing structure or representation is used", gap["review_note"])
                self.assertIn("does not require every client profile", gap["review_note"])
                table_locator = gap["source_locator"]
                row_range = table_locator["html_line_range"]
                table_by_section = {
                    "3.1": 12, "3.2": 13, "3.4": 18, "3.5": 19, "3.6": 20,
                    "3.7": 21, "3.8": 22, "3.9": 23, "3.10": 24, "3.11": 25,
                    "3.12": 26, "5.5": 161, "5.6": 162, "5.7": 163,
                }
                table_number = (
                    15 if gap["source_section"] == "3.3" and table_locator["row_label"] == "Wrapping Method"
                    else 16 if gap["source_section"] == "3.3"
                    else table_by_section[gap["source_section"]]
                )
                expected_row_citation = (
                    f"Specification v2.1 §{gap['source_section']}, "
                    f"Table {table_number}, pinned HTML lines {row_range[0]}–{row_range[1]}"
                )
                self.assertIn(expected_row_citation, gap["review_note"])
                self.assertIn("Profiles v2.1 §5.1.1 item 7", mapping["review_note"])
                self.assertFalse(
                    any(requirement_id in profile["requirement_ids"] for profile in client_profiles),
                    f"{requirement_id} must not be asserted for every client profile",
                )
                self.assertFalse(
                    any(
                        requirement_id in profile["requirement_ids"]
                        or requirement_id in profile["requirement_ids_audited_subset"]
                        for profile in oracle_profiles
                    ),
                    f"{requirement_id} must not appear in the independent client-profile oracle",
                )

        get_attributes_gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-REFERENCE-TAG"]
        self.assertIn("Table 161", get_attributes_gap["review_note"])
        self.assertIn("OR", get_attributes_gap["review_note"])
        self.assertIn("Table 223", get_attributes_gap["review_note"])
        self.assertIn("that operation permits omitting it", get_attributes_gap["review_note"])
        self.assertIn("§6.1.20", get_attributes_gap["review_note"])

    def test_disc_011_and_012_preserve_literal_normative_inclusions(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        decisions = {row["decision_id"]: row for row in catalog["decisions"]}
        discrepancies = {row["discrepancy_id"]: row for row in catalog["discrepancies"]}
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        decision = decisions.get("KMIPKIT-DEC-017")
        self.assertIsNotNone(decision, "literal cross-reference adjudication decision is missing")
        if decision is None:
            return
        self.assertEqual(decision["status"], "accepted")
        self.assertEqual(
            set(decision["discrepancy_ids"]),
            {"KMIPKIT-DISC-011", "KMIPKIT-DISC-012"},
        )
        self.assertIn("literal §5.7.1", decision["interpretation"])
        self.assertIn("possible authorial intent", decision["interpretation"])
        self.assertIn("does not change the normative inclusion", decision["interpretation"])

        for discrepancy_id in ("KMIPKIT-DISC-011", "KMIPKIT-DISC-012"):
            discrepancy = discrepancies.get(discrepancy_id)
            self.assertIsNotNone(discrepancy, f"{discrepancy_id} is missing")
            if discrepancy is None:
                continue
            self.assertEqual(discrepancy["normative_status"], "normative")
            self.assertEqual(discrepancy["state"], "resolved_by_approved_decision")
            self.assertEqual(discrepancy["decision_id"], "KMIPKIT-DEC-017")

        for gap_id, inclusion_id, expected_profile_id in (
            (
                "KMIPKIT-SOURCE-GAP-PROF-6.13-10848",
                "KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT",
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
            ),
            (
                "KMIPKIT-SOURCE-GAP-PROF-6.14-10867",
                "KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT",
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
            ),
        ):
            gap = gaps.get(gap_id)
            self.assertIsNotNone(gap, f"{gap_id} is missing")
            if gap is None:
                continue
            self.assertEqual(gap["source_resolution"]["inclusion_id"], inclusion_id)
            self.assertEqual(gap["source_resolution"]["included_section"], "5.7.1")
            self.assertEqual(gap["source_resolution"]["profile_id"], expected_profile_id)
            self.assertIn("possible authorial intent", gap["source_resolution"]["review_note"])
            self.assertNotIn("remains open", gap["source_resolution"]["review_note"])

    def test_t002_source_clause_crosswalk_and_baseline_inclusions_are_exact(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        source_clauses = {row["clause_id"]: row for row in oracle["source_clause_crosswalk"]}
        oracle_profiles = {row["profile_id"]: row for row in oracle["profiles"]}
        catalog_profiles = {
            row["profile_id"]: row
            for row in catalog["profiles"]
            if row["role"] == "client"
        }

        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(oracle_profiles), 18)
        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        for profile_id, expected in oracle_profiles.items():
            profile = catalog_profiles[profile_id]
            with self.subTest(profile_id=profile_id):
                self.assertEqual(
                    profile["direct_source_clause_ids"],
                    expected["direct_source_clause_ids"],
                )
                self.assertEqual(profile["source_clause_ids"], expected["source_clause_ids"])
                self.assertEqual(profile["requirement_ids"], expected["requirement_ids"])
                self.assertTrue(
                    {
                        source_clauses[clause_id]["section"]
                        for clause_id in expected["direct_source_clause_ids"]
                    }.issubset(set(expected["direct_profiles_sections"])),
                )

        baseline_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        baseline = oracle_profiles[baseline_id]
        self.assertEqual(
            baseline["direct_profiles_sections"],
            ["3.1", "4.1.2", "5.1.1", "6.1"],
        )
        self.assertEqual(len(baseline["source_clause_ids"]), 907)
        self.assertEqual(
            {
                source_id: sum(
                    source_clauses[clause_id]["source_id"] == source_id
                    for clause_id in baseline["source_clause_ids"]
                )
                for source_id in ("KMIPKIT-SRC-spec", "KMIPKIT-SRC-profiles")
            },
            {"KMIPKIT-SRC-spec": 877, "KMIPKIT-SRC-profiles": 30},
        )

        expected_edges = {
            "KMIPKIT-INCLUSION-PROF-BASELINE-SPEC": (
                "KMIPKIT-CLAUSE-PROF-6.1-001",
                "KMIPKIT-SRC-spec",
                None,
            ),
            "KMIPKIT-INCLUSION-PROF-BASELINE-BASIC-AUTH": (
                "KMIPKIT-CLAUSE-PROF-6.1-002",
                "KMIPKIT-SRC-profiles",
                "3.1",
            ),
            "KMIPKIT-INCLUSION-PROF-BASELINE-CONDITIONS": (
                "KMIPKIT-CLAUSE-PROF-6.1-003",
                "KMIPKIT-SRC-profiles",
                "5.1.1",
            ),
        }
        actual_edges = {
            edge["inclusion_id"]: (
                edge["source_clause_id"],
                edge["included_source_id"],
                edge["included_section"],
            )
            for edge in catalog["normative_inclusions"]
            if edge["inclusion_id"] in expected_edges
        }
        self.assertEqual(actual_edges, expected_edges)

    def test_t002_resolves_procedure_context_and_adjudicates_three_remaining_source_gaps(self) -> None:
        oracle = _load_json(ORACLE_PATH)
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        spec_lines = SPEC_SOURCE_PATH.read_text(encoding="cp1252").splitlines()
        for gap_id, (line_range, mode) in PROCEDURE_GAP_CONTEXT.items():
            gap = gaps[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(
                    gap["source_excerpt"],
                    _normalized_html_source_span(spec_lines, line_range),
                )
                self.assertEqual(gap["source_force"], "imperative_procedure_without_formal_modal")
                self.assertEqual(gap["actor"], "client")
                self.assertEqual(
                    gap["direction"],
                    f"client {mode} request and response processing",
                )
                self.assertEqual(
                    gap["condition"],
                    f"For a {mode} cryptographic operation.",
                )
                self.assertEqual(gap["scope_disposition"], "client_1_0")
                self.assertEqual(gap["status"], "reviewed_procedure_context")
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertIn("standalone requirement ID", gap["review_note"])

        unresolved_ids = {
            gap_id for gap_id, gap in gaps.items() if gap["scope_disposition"] == "unresolved"
        }
        self.assertFalse(set(ADJUDICATED_SOURCE_GAPS) & unresolved_ids)
        for gap_id, (line_range, status) in ADJUDICATED_SOURCE_GAPS.items():
            gap = gaps[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(
                    gap["source_excerpt"],
                    _normalized_html_source_span(spec_lines, line_range),
                )
                self.assertNotEqual(gap["scope_disposition"], "unresolved")
                self.assertEqual(gap["status"], status)
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        actual_outside_scope_ids = {
            gap_id
            for gap_id, gap in gaps.items()
            if gap["scope_disposition"]
            in {"server_only", "out_of_scope", "mixed_server_and_out_of_scope", "client_1_1"}
        }
        self.assertEqual(actual_outside_scope_ids, OUTSIDE_CLIENT_1_0_SOURCE_GAPS)

    def test_t002_maps_all_requirements_without_clause_ids_to_exact_table_rows(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        oracle = _load_json(ORACLE_PATH)
        requirements = [row for row in catalog["requirements"] if not row["source_clause_ids"]]
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        contracts = {row["gap_id"]: row for row in oracle["source_table_contracts"]}
        profile_requirements = {
            row["profile_id"]: set(row["requirement_ids"])
            for row in catalog["profiles"]
            if row["role"] == "client"
        }
        spec_lines = SPEC_SOURCE_PATH.read_text(encoding="cp1252").splitlines()

        self.assertEqual(len(requirements), 57)
        self.assertEqual(
            {state: sum(row["scope_state"] == state for row in requirements)
             for state in ("client_1_0", "profile_conditional")},
            {"client_1_0": 27, "profile_conditional": 30},
        )
        self.assertEqual(
            {status: sum(row["status"] == status for row in requirements)
             for status in ("verified", "scoped_verified", "unassigned")},
            {"verified": 25, "scoped_verified": 7, "unassigned": 25},
        )

        for requirement in requirements:
            requirement_id = requirement["requirement_id"]
            gap_id = requirement["source_gap_id"]
            source_rows = [row for row in (gaps.get(gap_id), contracts.get(gap_id)) if row is not None]
            with self.subTest(requirement_id=requirement_id):
                self.assertEqual(len(source_rows), 1)
                source_row = source_rows[0]
                self.assertEqual(source_row["source_id"], "KMIPKIT-SRC-spec")
                self.assertIn({
                    "source_id": "KMIPKIT-SRC-spec",
                    "section": source_row["source_section"],
                }, requirement["source_refs"])
                self.assertEqual(requirement["source_locator"], source_row["source_locator"])
                self.assertEqual(requirement["source_locator"]["kind"], "table_body_row")
                self.assertEqual(
                    source_row["source_excerpt"],
                    _normalized_html_source_span(
                        spec_lines,
                        requirement["source_locator"]["html_line_range"],
                    ),
                )
                self.assertEqual(requirement["source_clause_ids"], [])
                if requirement["scope_state"] == "client_1_0":
                    self.assertTrue(all(requirement_id in ids for ids in profile_requirements.values()))
                else:
                    self.assertTrue(all(requirement_id not in ids for ids in profile_requirements.values()))

        table_232_id = "KMIPKIT-REQ-SPEC-6.1.23-002-001"
        table_232_contract = contracts["KMIPKIT-SOURCE-GAP-SPEC-6.1.23-USAGE-LIMITS-COUNT-REQUIRED"]
        self.assertNotIn(table_232_contract["gap_id"], gaps)
        self.assertEqual(table_232_contract["table_number"], 232)
        self.assertEqual(table_232_contract["source_locator"]["row_label"], "Usage Limits Count")
        self.assertEqual(
            next(row for row in requirements if row["requirement_id"] == table_232_id)["source_locator"],
            table_232_contract["source_locator"],
        )


if __name__ == "__main__":
    unittest.main()
