"""Tests for catalog-derived Rust result-value tables."""

from __future__ import annotations

import unittest

from tools.normative_catalog.generate_result_values import render_rust


STATUS_ID = "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS"
REASON_ID = "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON"


def small_catalog() -> dict[str, object]:
    return {
        "elements": [
            {"element_id": STATUS_ID, "kind": "enumeration", "name": "Result Status"},
            {"element_id": REASON_ID, "kind": "enumeration", "name": "Result Reason"},
            {
                "element_id": "status-success",
                "kind": "enumeration_value",
                "name": "Success",
                "parent_element_ids": [STATUS_ID],
                "allocation": "assigned",
                "wire_value": "00000000",
            },
            {
                "element_id": "status-extension",
                "kind": "enumeration_value",
                "name": "Extensions",
                "parent_element_ids": [STATUS_ID],
                "allocation": "extension",
                "wire_value": "8XXXXXXX",
            },
            {
                "element_id": "reason-invalid-message",
                "kind": "enumeration_value",
                "name": "Invalid Message",
                "parent_element_ids": [REASON_ID],
                "allocation": "assigned",
                "wire_value": "00000004",
            },
            {
                "element_id": "reason-reserved",
                "kind": "enumeration_value",
                "name": "(Reserved)",
                "parent_element_ids": [REASON_ID],
                "allocation": "reserved",
                "wire_value": "00000002",
            },
        ]
    }


class ResultValueGeneratorTests(unittest.TestCase):
    def test_renders_only_assigned_numeric_values_in_stable_order(self) -> None:
        rendered = render_rust(small_catalog())

        self.assertIn('(ResultStatus::from_raw(0x00000000), "Success")', rendered)
        self.assertIn('(ResultReason::from_raw(0x00000004), "Invalid Message")', rendered)
        self.assertNotIn("Extensions", rendered)
        self.assertNotIn("Reserved", rendered)
        self.assertLess(rendered.index("RESULT_STATUSES"), rendered.index("RESULT_REASONS"))

    def test_rejects_duplicate_numeric_values(self) -> None:
        catalog = small_catalog()
        elements = catalog["elements"]
        assert isinstance(elements, list)
        elements.append(
            {
                "element_id": "status-duplicate",
                "kind": "enumeration_value",
                "name": "Different Name",
                "parent_element_ids": [STATUS_ID],
                "allocation": "assigned",
                "wire_value": "00000000",
            }
        )

        with self.assertRaisesRegex(ValueError, "duplicate"):
            render_rust(catalog)


if __name__ == "__main__":
    unittest.main()
