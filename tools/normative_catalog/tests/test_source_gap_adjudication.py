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
        self.assertTrue(
            {"6.1", "6.1.11", "6.1.17", "6.1.24", "6.1.32", "6.1.55"} <= source_sections,
            "the general Data rule and every affected operation table must be linked",
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
