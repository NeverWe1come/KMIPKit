"""Require the four-language fluent Delete Attribute ABI inventory."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FUNCTION_IDS = {
    "create",
    "set_unique_identifier",
    "set_current_attribute_item",
    "set_catalogued_attribute",
    "set_vendor_attribute",
    "set_attribute_reference",
    "set_asynchronous_indicator",
    "validate",
    "build_request",
    "build_batch_item",
    "execute_v1",
    "execute_with_options_v1",
}


class FluentDeleteAttributeBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_functions_are_declared_for_all_four_languages(self) -> None:
        rows = [
            row for row in self.manifest["functions"]
            if row["id"].startswith("delete_attribute_builder_")
        ]
        self.assertEqual(
            {f"delete_attribute_builder_{suffix}" for suffix in FUNCTION_IDS},
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
            if row["id"] == "DeleteAttributeBuilder"
        ]
        self.assertEqual(1, len(rows))
        self.assertEqual(
            {"rust", "c", "java", "python"},
            {language for language in ("rust", "c", "java", "python")
             if rows[0].get(language)},
        )

    def test_java_vendor_attribute_alias_is_declared_in_manifest(self) -> None:
        export_name = "org.kmipkit.fluent.DeleteAttributeBuilder.vendorAttribute"
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit"
        }
        self.assertTrue(export_name in declarations, f"missing Java declaration: {export_name}")
        declaration = declarations[export_name]
        self.assertEqual("DeleteAttributeBuilder.vendorAttribute:java", declaration["declarationId"])
        self.assertEqual("fluent_delete_attribute_parity", declaration["capabilityId"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(
            {
                "rust": "kmipkit_client::fluent::DeleteAttributeBuilder<'a>::current_vendor_attribute",
                "c": "kmipkit_delete_attribute_builder_set_vendor_attribute",
                "java": export_name,
                "python": "kmipkit.fluent.DeleteAttributeBuilder.current_vendor_attribute",
            },
            {language: mapping["name"] for language, mapping in declaration["languageMappings"].items()},
        )
        capability = next(
            row for row in self.manifest["capabilities"]
            if row["capabilityId"] == "fluent_delete_attribute_parity"
        )
        self.assertIn(export_name, capability["mappings"]["java"])

    def test_attribute_reference_has_no_unrelated_java_overload_signatures(self) -> None:
        export_name = "org.kmipkit.fluent.DeleteAttributeBuilder.attributeReference"
        declaration = next(
            row for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit" and row.get("exportName") == export_name
        )
        self.assertNotIn("javaSignatures", declaration)


if __name__ == "__main__":
    unittest.main()
