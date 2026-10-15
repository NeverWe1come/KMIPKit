"""Source-gap decisions are checked against the independent profile oracle."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"


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


def _load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


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
            "Create, Create Key Pair, Create Split Key, Re-key, Re-key Key Pair, or Derive Key.",
        )
        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-4.26-001"])
        self.assertNotIn("Register", requirement["condition"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertEqual(
            requirement["verification_refs"],
            ["crates/kmipkit-protocol/tests/unit/create_tests.rs::request_encodes_raw_key_format_for_server_generated_create"],
        )
        self.assertIn("covers the Create request only", requirement["review_note"])
        self.assertIn("does not evidence the other operations", requirement["review_note"])

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

    def test_section_4_31_absent_link_may_is_a_client_capability_without_implementation_evidence(self) -> None:
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
        self.assertEqual(requirement["implementation_refs"], [])
        self.assertEqual(requirement["verification_refs"], [])
        self.assertEqual(requirement["status"], "unassigned")
        self.assertIn("inbound or outbound", requirement["review_note"])
        self.assertIn("does not assign a message direction", requirement["review_note"])
        self.assertIn("operation-specific Link requirements", requirement["review_note"])

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
                self.assertEqual(requirement["implementation_refs"], [])
                self.assertEqual(requirement["verification_refs"], [])
                self.assertEqual(requirement["status"], "unassigned")
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
                self.assertIn("no local typed Link", gap["review_note"])

        official_case = next(
            row for row in catalog["test_cases"] if row["test_id"] == "KMIPKIT-TEST-CN01-2-66"
        )
        self.assertEqual(official_case["official_case_id"], "TC-PGP-1-21")
        self.assertEqual(official_case["source_section"], "2.66")
        self.assertEqual(official_case["requirement_ids"], ["KMIPKIT-REQ-SPEC-4.31-004", "KMIPKIT-REQ-SPEC-4.31-005"])
        self.assertEqual(official_case["fixture_availability"], "unavailable")
        self.assertIsNone(official_case["fixture_path"])

        self.assertEqual(len(oracle["source_table_contracts"]), 66)
        self.assertEqual(oracle["counts"]["requirement_crosswalk_records"], 505)
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
        self.assertEqual("open", discrepancy["state"])
        conflict_text = " ".join(discrepancy["alternatives"] + [discrepancy["downstream_impact"]])
        self.assertIn("Tables 196, 214, 235, 259, and 334", conflict_text)
        self.assertIn("Digested Data", conflict_text)

    def test_section_6_1_procedure_clauses_preserve_unresolved_scope_and_conflict(self) -> None:
        catalog = _load_json(CATALOG_PATH)
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}

        for ordinal in (6, 7, 8):
            clause = clauses[f"KMIPKIT-CLAUSE-SPEC-6.1-{ordinal:03d}"]
            with self.subTest(clause=clause["clause_id"]):
                self.assertEqual(clause["source_keywords"], ["REQUIRED"])
                self.assertEqual(clause["disposition"], "informative_context")
                self.assertEqual(clause["requirement_ids"], [])
                self.assertEqual(clause["role"], "unclear")
                self.assertEqual(clause["direction"], "unclear")
                self.assertEqual(clause["scope_state"], "unclear")
                self.assertIn("procedure", clause["exclusion_rationale"].lower())
                self.assertIn("lowercase", clause["exclusion_rationale"].lower())

        data_clause = clauses["KMIPKIT-CLAUSE-SPEC-6.1-009"]
        self.assertEqual(data_clause["source_keywords"], ["REQUIRED"])
        self.assertEqual(data_clause["disposition"], "source_discrepancy")
        self.assertEqual(data_clause["requirement_ids"], [])
        self.assertEqual(data_clause["role"], "unclear")
        self.assertEqual(data_clause["direction"], "unclear")
        self.assertEqual(data_clause["scope_state"], "unclear")
        self.assertIn("lowercase", data_clause["exclusion_rationale"].lower())
        self.assertIn("table", data_clause["exclusion_rationale"].lower())

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


if __name__ == "__main__":
    unittest.main()
