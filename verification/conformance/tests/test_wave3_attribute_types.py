"""Wave 3 Red tests for KMIP 3.0 object attributes in generate_attribute_types.py."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from verification.conformance import generate_attribute_types

ROOT = Path(__file__).resolve().parents[3]


class Wave3AttributeTypesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(
            (ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        cls.versioned_tags = json.loads(
            (ROOT / "verification/conformance/catalog/kmip-versioned-tags.json").read_text(encoding="utf-8")
        )

    def test_attribute_encodings_include_kmip30_object_attributes(self) -> None:
        rendered = generate_attribute_types.render_rust(
            self.catalog,
            versioned_tags=self.versioned_tags,
        )
        for tag_hex in (
            "0x42_01_90",  # Certificate Link
            "0x42_01_9E",  # Object Class
            "0x42_01_AE",  # OTP Counter
            "0x42_01_B6",  # Split Key Polynomial
            "0x42_01_B8",  # Deactivation Reason
            "0x42_01_BD",  # Certify Counter
            "0x42_01_C2",  # NIST Security Category
        ):
            self.assertIn(tag_hex, rendered)

    def test_typed_attributes_retain_kmip21_baseline_count(self) -> None:
        definitions = generate_attribute_types.collect_typed_attributes(self.catalog)
        self.assertEqual(62, len(definitions))

    def test_typed_attributes_with_versioned_tags_emits_all_ninety_nine(self) -> None:
        definitions = generate_attribute_types.collect_typed_attributes(
            self.catalog, self.versioned_tags
        )
        self.assertEqual(99, len(definitions))



if __name__ == "__main__":
    unittest.main()
