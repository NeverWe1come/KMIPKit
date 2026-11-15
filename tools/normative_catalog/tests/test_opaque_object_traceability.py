"""Normative traceability for the Opaque Object Register context feature."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CATALOG = ROOT / "specification/catalog/kmip-2.1.json"
TRACEABILITY = ROOT / "specs/029-opaque-object-context/traceability.md"

REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-2.3-002": "§2.3",
    "KMIPKIT-REQ-SPEC-2.3-003": "§2.3, Table 4; §6.1.43, Table 291",
    "KMIPKIT-REQ-SPEC-6.1.43-001": "§6.1.43, Table 291",
    "KMIPKIT-REQ-SPEC-6.1.43-007": "§6.1.43, Table 293",
    "KMIPKIT-REQ-SPEC-6.1.43-009-001": "§6.1.43, Table 293",
    "KMIPKIT-REQ-SPEC-6.1.43-009-002": "§6.1.43, Table 293",
    "KMIPKIT-REQ-SPEC-6.1.43-010-001": "§6.1.43, Table 293",
    "KMIPKIT-REQ-SPEC-6.1.43-010-002": "§6.1.43, Table 293",
}


class OpaqueObjectTraceabilityTests(unittest.TestCase):
    def test_table4_required_fields_are_traced_to_child_elements_not_heading(self) -> None:
        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        clauses = {record["clause_id"]: record for record in catalog["source_clauses"]}
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        elements = {record["element_id"]: record for record in catalog["elements"]}

        header = clauses["KMIPKIT-CLAUSE-SPEC-2.3-003"]
        self.assertEqual(
            header["locator"], {"ordinal": 3, "block_kind": "table_row"}
        )
        self.assertEqual(header["source_keywords"], ["REQUIRED"])
        self.assertEqual(header["disposition"], "non_applicable")
        self.assertEqual(header["requirement_ids"], [])

        requirement = requirements["KMIPKIT-REQ-SPEC-2.3-003"]
        self.assertEqual(requirement["feature_spec"], "KMIPKIT-0044")
        self.assertEqual(requirement["source_clause_ids"], [])
        self.assertEqual(requirement["source_keyword"], None)
        self.assertEqual(requirement["source_force"], "required_table_body_value")
        self.assertEqual(
            requirement["source_gap_id"],
            "KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED",
        )
        self.assertEqual(
            requirement["source_locator"],
            {
                "kind": "table_body_row",
                "section": "2.3",
                "row_label": "Opaque Data Type",
                "table": "object_definition",
                "html_line_range": [6927, 6945],
            },
        )
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertIn("Opaque Data Type", requirement["summary"])
        self.assertIn("Opaque Data Value", requirement["summary"])
        self.assertNotIn("heading", requirement["summary"].lower())
        self.assertIn(
            "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
            "opaque_object_requires_typed_table4_data_type_and_data_value_children",
            requirement["verification_refs"],
        )
        self.assertIn(
            "crates/kmipkit-client/tests/unit/register_execution_tests.rs::"
            "malformed_table_4_opaque_object_is_not_sent",
            requirement["verification_refs"],
        )

        required_children = {
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-TYPE": "Enumeration",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-VALUE": "Byte String",
        }
        for element_id, encoding in required_children.items():
            with self.subTest(element_id=element_id):
                element = elements[element_id]
                self.assertEqual(element["source_requiredness"], "Yes")
                self.assertEqual(element["source_encoding"], encoding)
                self.assertIn("KMIPKIT-REQ-SPEC-2.3-003", element["requirement_ids"])
                self.assertIn(
                    {"source_id": "KMIPKIT-SRC-spec", "section": "2.3"},
                    element["source_refs"],
                )

        register_evidence = {
            "KMIPKIT-REQ-SPEC-6.1.43-001": (
                "crates/kmipkit-client/tests/unit/register_execution_tests.rs::"
                "managed_opaque_register_preserves_standard_table_291_wire_shape_and_attribute_order"
            ),
            "KMIPKIT-REQ-SPEC-6.1.43-007": (
                "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
                "managed_opaque_context_accepts_table293_attributes_without_a_key_block_and_emits_no_marker"
            ),
            "KMIPKIT-REQ-SPEC-6.1.43-009-001": (
                "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
                "managed_opaque_context_accepts_table293_attributes_without_a_key_block_and_emits_no_marker"
            ),
            "KMIPKIT-REQ-SPEC-6.1.43-009-002": (
                "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
                "managed_opaque_context_requires_algorithm_and_length_as_a_pair"
            ),
            "KMIPKIT-REQ-SPEC-6.1.43-010-001": (
                "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
                "managed_opaque_context_accepts_table293_attributes_without_a_key_block_and_emits_no_marker"
            ),
            "KMIPKIT-REQ-SPEC-6.1.43-010-002": (
                "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs::"
                "managed_opaque_context_requires_algorithm_and_length_as_a_pair"
            ),
        }
        for requirement_id, verification_ref in register_evidence.items():
            with self.subTest(requirement_id=requirement_id):
                linked = requirements[requirement_id]
                self.assertEqual(linked["feature_spec"], "KMIPKIT-0036")
                self.assertIn("crates/kmipkit-protocol/src/register.rs", linked["implementation_refs"])
                self.assertIn(verification_ref, linked["verification_refs"])

    def test_table4_required_child_rows_have_a_body_gap_separate_from_header(self) -> None:
        oracle = json.loads(
            (ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json").read_text(
                encoding="utf-8"
            )
        )
        expected_rows = {
            "KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED": (
                "Opaque Data Type",
                [6927, 6945],
                "Opaque Data Type Enumeration Yes",
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-VALUE-REQUIRED": (
                "Opaque Data Value",
                [6946, 6964],
                "Opaque Data Value Byte String Yes",
            ),
        }
        gaps = {
            gap["gap_id"]: gap
            for gap in oracle["source_only_requirement_gaps"]
            if gap["gap_id"] in expected_rows
        }
        self.assertEqual(set(gaps), set(expected_rows))
        self.assertEqual(
            oracle["counts"]["source_only_requirement_gaps"],
            len(oracle["source_only_requirement_gaps"]),
        )
        for gap_id, (row_label, line_range, excerpt) in expected_rows.items():
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                self.assertEqual(gap["source_id"], "KMIPKIT-SRC-spec")
                self.assertEqual(
                    gap["source_path"],
                    "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html",
                )
                self.assertEqual(gap["source_section"], "2.3")
                self.assertIsNone(gap["source_clause_id"])
                self.assertEqual(
                    gap["source_locator"],
                    {
                        "kind": "table_body_row",
                        "section": "2.3",
                        "row_label": row_label,
                        "table": "object_definition",
                        "html_line_range": line_range,
                    },
                )
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(gap["source_excerpt"], excerpt)
                self.assertEqual(gap["source_keywords"], [])
                self.assertEqual(gap["source_force"], "required_table_body_value")
                self.assertEqual(gap["candidate_requirement_id"], "KMIPKIT-REQ-SPEC-2.3-003")
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertEqual(gap["reused_requirement_id"], "KMIPKIT-REQ-SPEC-2.3-003")
                self.assertEqual(gap["status"], "source_obligation_covered_by_existing_stable_label")
                self.assertIn(row_label, gap["audited_statement"])

        header = next(
            review
            for review in oracle["source_requirement_reviews"]
            if review["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-2.3-003"
        )
        self.assertEqual(header["source_obligations"], [])
        self.assertEqual(header["expected_client_requirement_ids"], [])
        self.assertEqual(
            header["rejected_candidate_mappings"][0]["expected_disposition"],
            "reject_candidate_non_normative_table_header",
        )

    def test_table4_body_requirement_remains_in_profile_closures(self) -> None:
        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json").read_text(
                encoding="utf-8"
            )
        )
        oracle_profiles = {
            profile["profile_id"]: profile for profile in oracle["profiles"]
        }
        requirement_id = "KMIPKIT-REQ-SPEC-2.3-003"
        relevant_profiles = [
            profile
            for profile in catalog["profiles"]
            if "KMIPKIT-CLAUSE-SPEC-2.3-003" in profile["source_clause_ids"]
        ]
        self.assertTrue(relevant_profiles)
        for profile in relevant_profiles:
            with self.subTest(profile_id=profile["profile_id"]):
                self.assertIn(requirement_id, profile["requirement_ids"])
                self.assertIn(
                    requirement_id,
                    oracle_profiles[profile["profile_id"]]["requirement_ids_audited_subset"],
                )

    def test_each_context_requirement_has_source_code_positive_and_negative_evidence(self) -> None:
        self.assertTrue(TRACEABILITY.is_file(), "feature traceability ledger must exist")
        lines = TRACEABILITY.read_text(encoding="utf-8").splitlines()
        rows: dict[str, list[str]] = {}
        for line in lines:
            if not line.startswith("|"):
                continue
            cells = [cell.strip() for cell in line.strip("|").split("|")]
            if len(cells) != 5 or not cells[0].strip("`").startswith("KMIPKIT-REQ-SPEC-"):
                continue
            requirement_id = cells[0].strip("`")
            self.assertNotIn(requirement_id, rows, f"duplicate traceability row: {requirement_id}")
            rows[requirement_id] = cells

        self.assertEqual(set(rows), set(REQUIREMENTS))
        for requirement_id, source_locator in REQUIREMENTS.items():
            with self.subTest(requirement_id=requirement_id):
                row = rows[requirement_id]
                source, implementation, positive, negative = row[1:]
                self.assertIn("OASIS KMIP 2.1", source)
                self.assertIn(source_locator, source)
                self._check_file_references(implementation)
                self._check_test_references(positive)
                self._check_test_references(negative)
                if requirement_id == "KMIPKIT-REQ-SPEC-6.1.43-001":
                    self.assertIn(
                        "tools/normative_catalog/tests/test_opaque_object_traceability.py::"
                        "test_register_request_constructor_requires_object_type_and_attributes",
                        negative,
                    )

    def test_register_request_constructor_requires_object_type_and_attributes(self) -> None:
        source = (ROOT / "crates/kmipkit-protocol/src/register.rs").read_text(
            encoding="utf-8"
        )
        constructor = re.search(
            r"pub const fn new\((?P<parameters>.*?)\)\s*->\s*Self",
            source,
            re.DOTALL,
        )
        self.assertIsNotNone(constructor)
        parameters = constructor.group("parameters")
        self.assertRegex(parameters, r"\bobject_type\s*:\s*ObjectType\b")
        self.assertRegex(parameters, r"\battributes\s*:\s*AttributeSet\b")
        self.assertRegex(parameters, r"\bobject\s*:\s*Item\b")

        register_request_fields = re.search(
            r"pub struct RegisterRequest\s*\{(?P<fields>[^}]*)\}",
            source,
            re.DOTALL,
        )
        self.assertIsNotNone(register_request_fields)
        self.assertNotRegex(
            register_request_fields.group("fields"),
            r"(?m)^\s*pub(?:\([^)]*\))?\s+\w+",
            "callers cannot construct an incomplete RegisterRequest literal",
        )

    def _check_file_references(self, cell: str) -> None:
        references = re.findall(r"`([^`]+)`", cell)
        self.assertTrue(references, "implementation evidence must name source files")
        for relative_path in references:
            self.assertTrue((ROOT / relative_path).is_file(), relative_path)

    def _check_test_references(self, cell: str) -> None:
        references = re.findall(r"`([^`]+)`", cell)
        self.assertTrue(references, "verification evidence must name exact test symbols")
        for reference in references:
            relative_path, separator, test_name = reference.partition("::")
            self.assertTrue(separator, reference)
            test_file = ROOT / relative_path
            self.assertTrue(test_file.is_file(), relative_path)
            source = test_file.read_text(encoding="utf-8")
            self.assertRegex(source, rf"\b(?:fn|def)\s+{re.escape(test_name)}\s*\(", reference)
