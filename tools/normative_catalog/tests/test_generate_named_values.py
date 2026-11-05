"""Tests for the deterministic four-language named-value generator."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog import generate_named_values


ROOT = Path(__file__).resolve().parents[3]


class NamedValueGeneratorTests(unittest.TestCase):
    def test_catalog_contains_the_approved_named_value_surface(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

        values = generate_named_values.collect_values(catalog)

        self.assertEqual(len(values.enumerations), 64)
        self.assertEqual(sum(len(value.values) for value in values.enumerations), 723)
        self.assertEqual(len(values.bitmasks), 3)
        self.assertEqual(sum(len(value.values) for value in values.bitmasks), 44)

    def test_outputs_include_named_values_in_every_language(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

        outputs = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog)
        )

        self.assertEqual(
            set(outputs),
            {"rust", "c", "java", "python"},
        )
        for source in outputs.values():
            self.assertIn("AES", source)
            self.assertIn("ENCRYPT", source)

    def test_duplicate_names_and_values_are_rejected(self) -> None:
        catalog: dict[str, Any] = {
            "elements": [
                {
                    "element_id": "enum-1",
                    "kind": "enumeration",
                    "name": "Example Kind",
                },
                {
                    "element_id": "value-1",
                    "kind": "enumeration_value",
                    "name": "Same",
                    "parent_element_ids": ["enum-1"],
                    "wire_value": "00000001",
                    "allocation": "assigned",
                },
                {
                    "element_id": "value-2",
                    "kind": "enumeration_value",
                    "name": "Same",
                    "parent_element_ids": ["enum-1"],
                    "wire_value": "00000002",
                    "allocation": "assigned",
                },
            ]
        }

        with self.assertRaisesRegex(ValueError, "duplicate.*name"):
            generate_named_values.collect_values(catalog)


if __name__ == "__main__":
    unittest.main()
