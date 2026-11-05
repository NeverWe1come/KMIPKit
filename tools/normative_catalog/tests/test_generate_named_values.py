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
        self.assertEqual(sum(len(value.values) for value in values.enumerations), 644)
        self.assertEqual(len(values.bitmasks), 3)
        self.assertEqual(sum(len(value.values) for value in values.bitmasks), 33)

        catalog_values = [
            value
            for value in catalog["elements"]
            if value.get("kind") in {"enumeration_value", "bitmask_value"}
        ]
        self.assertEqual(sum(value["kind"] == "enumeration_value" for value in catalog_values), 723)
        self.assertEqual(sum(value["kind"] == "bitmask_value" for value in catalog_values), 44)
        self.assertEqual(
            sum(value.get("allocation") == "assigned" for value in catalog_values),
            677,
        )

    def test_outputs_include_named_values_in_every_language(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

        outputs = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog)
        )

        self.assertEqual(
            set(outputs),
            {"rust", "c", "java", "python", "traceability"},
        )
        for language in ("rust", "c", "java", "python"):
            source = outputs[language]
            self.assertIn("AES", source)
            self.assertIn("ENCRYPT", source)
        self.assertIn("KMIPKIT-ELEM-ENUM", outputs["traceability"])

    def test_rust_output_is_clippy_clean_for_numeric_literals_and_pure_lookups(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        rust = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog)
        )["rust"]

        self.assertIn("pub const INCREMENT: u32 = 0x0000_0001;", rust)
        self.assertIn("#[must_use]\n            pub const fn name", rust)
        self.assertIn("#[must_use]\n            pub const fn known_bits", rust)
        self.assertIn("/// `AESKeyWrapPadding`;", rust)
        self.assertNotIn("match raw {\n                    _ => None", rust)
        self.assertNotIn("match raw {\n                None", rust)
        self.assertIn("pub const fn name(_raw: u32)", rust)

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
                    "source_refs": [{"source_id": "source", "section": "11.1"}],
                    "parent_element_ids": ["enum-1"],
                    "wire_value": "00000001",
                    "allocation": "assigned",
                },
                {
                    "element_id": "value-2",
                    "kind": "enumeration_value",
                    "name": "Same",
                    "source_refs": [{"source_id": "source", "section": "11.1"}],
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
