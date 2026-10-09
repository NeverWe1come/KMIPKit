"""Tests for catalog-derived Rust attribute TTLV type metadata."""

from __future__ import annotations

import unittest

from tools.normative_catalog.generate_attribute_types import render_rust


def small_catalog() -> dict[str, object]:
    return {
        "elements": [
            {
                "element_id": "tag-420028",
                "kind": "tag",
                "wire_value": "420028",
                "allocation": "assigned",
            },
            {
                "element_id": "tag-420094",
                "kind": "tag",
                "wire_value": "420094",
                "allocation": "assigned",
            },
            {
                "element_id": "attribute-algorithm",
                "kind": "attribute",
                "name": "Cryptographic Algorithm",
                "parent_element_ids": ["tag-420028"],
                "source_encoding": "Enumeration",
            },
            {
                "element_id": "attribute-id",
                "kind": "attribute",
                "name": "Unique Identifier",
                "parent_element_ids": ["tag-420094"],
                "source_encoding": "Text String, Enumeration or Integer",
            },
            {
                "element_id": "attribute-vendor",
                "kind": "attribute",
                "name": "Vendor Attribute",
                "parent_element_ids": [],
            },
        ]
    }


class AttributeTypeGeneratorTests(unittest.TestCase):
    def test_renders_sorted_catalog_types_and_all_permitted_wire_forms(self) -> None:
        rendered = render_rust(small_catalog())

        self.assertIn("(0x00_42_28, &[ItemType::Enumeration])", rendered)
        self.assertIn(
            "(0x00_42_94, &[ItemType::TextString, ItemType::Enumeration, ItemType::Integer])",
            rendered,
        )
        self.assertNotIn("Vendor Attribute", rendered)
        self.assertLess(rendered.index("0x00_42_28"), rendered.index("0x00_42_94"))

    def test_rejects_unrecognized_source_encoding(self) -> None:
        catalog = small_catalog()
        elements = catalog["elements"]
        assert isinstance(elements, list)
        elements[2]["source_encoding"] = "Unknown Wire Encoding"

        with self.assertRaisesRegex(ValueError, "encoding"):
            render_rust(catalog)

    def test_rejects_attribute_without_one_catalog_tag_parent(self) -> None:
        catalog = small_catalog()
        elements = catalog["elements"]
        assert isinstance(elements, list)
        elements[2]["parent_element_ids"] = []

        with self.assertRaisesRegex(ValueError, "parent"):
            render_rust(catalog)


if __name__ == "__main__":
    unittest.main()
