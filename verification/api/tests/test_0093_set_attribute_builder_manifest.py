"""Require complete four-language parity for the fluent Set Attribute builder."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BUILDERS = {
    "SetAttributeBuilder": {
        "attribute",
        "attribute_item",
        "build",
        "build_request",
        "execute",
        "to_batch_item",
        "unique_identifier",
        "validate",
        "vendor_attribute",
        "with_asynchronous_indicator",
        "with_options",
    }
}


class FluentSetAttributeBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "bindings/shared/public-api.json").read_text(encoding="utf-8")
        )

    def test_every_public_rust_path_is_traced_and_four_language_mapped(self) -> None:
        expected: dict[str, str] = {}
        for prefix in ("kmipkit_client::", "kmipkit_client::fluent::"):
            for builder, methods in BUILDERS.items():
                expected[f"{prefix}{builder}"] = "type"
                for method in methods:
                    expected[f"{prefix}{builder}<'a>::{method}"] = "associated-item"
            expected[f"{prefix}RequestBuilder<'a>::set_attribute"] = "associated-item"
        expected["kmipkit_client::SetAttributeBuilder::detached"] = "associated-item"

        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("declarationState") == "declared"
            and (
                "SetAttributeBuilder" in row["exportName"]
                or "RequestBuilder<'a>::set_attribute" in row["exportName"]
            )
        ]
        self.assertEqual(expected, {row["exportName"]: row["kind"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row.get("languageMappings", {})),
                )
                self.assertNotIn("blockedReason", row)

    def test_python_builder_lifecycle_exports_are_mapped(self) -> None:
        expected = {
            "kmipkit.fluent.SetAttributeBuilder.__enter__",
            "kmipkit.fluent.SetAttributeBuilder.__exit__",
            "kmipkit.fluent.SetAttributeBuilder.__init__",
        }
        expected_targets = {
            "kmipkit.fluent.SetAttributeBuilder.__init__": {
                "rust": "kmipkit_client::SetAttributeBuilder::detached",
                "c": "kmipkit_set_attribute_builder_create",
                "java": "org.kmipkit.fluent.FluentRequest.setAttribute",
            },
            "kmipkit.fluent.SetAttributeBuilder.__enter__": {
                "rust": "kmipkit_client::SetAttributeBuilder",
                "c": "kmipkit_set_attribute_builder_t",
                "java": "org.kmipkit.fluent.SetAttributeBuilder",
            },
            "kmipkit.fluent.SetAttributeBuilder.__exit__": {
                "rust": "kmipkit_client::SetAttributeBuilder",
                "c": "kmipkit_set_attribute_builder_release",
                "java": "org.kmipkit.fluent.SetAttributeBuilder.close",
            },
        }
        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "python-kmipkit"
            and row.get("exportName") in expected
        ]
        self.assertEqual(expected, {row["exportName"] for row in rows})
        for row in rows:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("fluent_set_attribute_parity", row["capabilityId"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                for language, target in expected_targets[row["exportName"]].items():
                    self.assertEqual(
                        target, row["languageMappings"][language]["name"]
                    )

    def test_fluent_builder_operations_are_declared_in_all_four_languages(self) -> None:
        functions = [
            row
            for row in self.manifest["functions"]
            if row["id"].startswith("set_attribute_builder_")
        ]
        names = {row["id"] for row in functions}
        self.assertEqual(
            {
                "set_attribute_builder_create",
                "set_attribute_builder_set_unique_identifier",
                "set_attribute_builder_add_attribute_item",
                "set_attribute_builder_add_catalogued_attribute",
                "set_attribute_builder_add_vendor_attribute",
                "set_attribute_builder_set_asynchronous_indicator",
                "set_attribute_builder_validate",
                "set_attribute_builder_build_request",
                "set_attribute_builder_build_batch_item",
                "set_attribute_builder_execute_v1",
                "set_attribute_builder_execute_with_options_v1",
            },
            names,
        )
        for row in functions:
            with self.subTest(function=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )

    def test_every_c_builder_function_is_declared_in_the_public_header(self) -> None:
        expected_symbols = {
            row["c"]["symbol"]
            for row in self.manifest["functions"]
            if row["id"].startswith("set_attribute_builder_")
        }
        declared_symbols = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "c-kmipkit"
            and row.get("kind") == "c-symbol"
            and row.get("declarationState") in {"declared", "verified"}
        }
        self.assertEqual(expected_symbols, expected_symbols & declared_symbols)


if __name__ == "__main__":
    unittest.main()
