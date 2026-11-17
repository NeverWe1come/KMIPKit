"""Require explicit manifest mappings for generic Create builder aliases."""

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
EXPECTED = {
    ("java-kmipkit", "org.kmipkit.fluent.CreateBuilder.close"): {
        "rust": "kmipkit_client::fluent::CreateBuilder",
        "c": "kmipkit_create_builder_release",
        "java": "org.kmipkit.fluent.CreateBuilder.close",
        "python": "kmipkit.fluent.CreateBuilder.close",
    },
    ("java-kmipkit", "org.kmipkit.fluent.CreateBuilder.toBatchItem"): {
        "rust": "kmipkit_client::fluent::CreateBuilder<'a>::to_batch_item",
        "c": "kmipkit_create_builder_build_batch_item",
        "java": "org.kmipkit.fluent.CreateBuilder.toBatchItem",
        "python": "kmipkit.fluent.CreateBuilder.build",
    },
    ("python-kmipkit", "kmipkit.fluent.CreateBuilder.close"): {
        "rust": "kmipkit_client::fluent::CreateBuilder",
        "c": "kmipkit_create_builder_release",
        "java": "org.kmipkit.fluent.CreateBuilder.close",
        "python": "kmipkit.fluent.CreateBuilder.close",
    },
    ("python-kmipkit", "kmipkit.fluent.CreateBuilder.to_batch_item"): {
        "rust": "kmipkit_client::fluent::CreateBuilder<'a>::to_batch_item",
        "c": "kmipkit_create_builder_build_batch_item",
        "java": "org.kmipkit.fluent.CreateBuilder.build",
        "python": "kmipkit.fluent.CreateBuilder.to_batch_item",
    },
}


class FluentCreateBuilderAliasManifestTests(unittest.TestCase):
    def test_all_public_aliases_are_mapped_to_the_four_language_contract(self) -> None:
        manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )
        rows = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        self.assertTrue(set(EXPECTED).issubset(rows))
        for key, expected_mappings in EXPECTED.items():
            with self.subTest(export=key[1]):
                row = rows[key]
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual("fluent_create_builder_parity", row["capabilityId"])
                self.assertEqual(
                    expected_mappings,
                    {language: mapping["name"] for language, mapping in row["languageMappings"].items()},
                )


if __name__ == "__main__":
    unittest.main()
