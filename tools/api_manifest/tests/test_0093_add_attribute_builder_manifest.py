"""Require the four-language fluent Add Attribute ABI inventory."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FUNCTION_IDS = {
    "create",
    "set_unique_identifier",
    "add_attribute_item",
    "add_catalogued_attribute",
    "add_vendor_attribute",
    "set_asynchronous_indicator",
    "validate",
    "build_request",
    "build_batch_item",
    "execute_v1",
    "execute_with_options_v1",
}


class FluentAddAttributeBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_functions_are_declared_for_all_four_languages(self) -> None:
        rows = [
            row for row in self.manifest["functions"]
            if row["id"].startswith("add_attribute_builder_")
        ]
        self.assertEqual(
            {f"add_attribute_builder_{suffix}" for suffix in FUNCTION_IDS},
            {row["id"] for row in rows},
        )
        for row in rows:
            with self.subTest(function=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )

    def test_opaque_builder_type_is_declared_for_all_four_languages(self) -> None:
        rows = [
            row for row in self.manifest["types"]
            if row["id"] == "add_attribute_builder"
        ]
        self.assertEqual(1, len(rows))
        self.assertEqual(
            {"rust", "c", "java", "python"},
            {language for language in ("rust", "c", "java", "python")
             if rows[0].get(language)},
        )


if __name__ == "__main__":
    unittest.main()
