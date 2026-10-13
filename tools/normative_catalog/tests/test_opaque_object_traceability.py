"""Normative traceability for the Opaque Object Register context feature."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
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
