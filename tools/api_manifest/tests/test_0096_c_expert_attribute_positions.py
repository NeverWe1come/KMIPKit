"""Contract checks for the C fluent expert-attribute insertion API."""

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


class ExpertAttributePositionManifestTests(unittest.TestCase):
    def test_c_builder_exposes_explicit_object_attribute_position(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declaration = next(
            (
                entry
                for entry in manifest["publicDeclarations"]
                if entry.get("exportName")
                == "kmipkit_symmetric_key_builder_insert_attribute_item"
            ),
            None,
        )

        self.assertIsNotNone(declaration)
        assert declaration is not None
        self.assertEqual(declaration["kind"], "c-symbol")
        self.assertIn("KMIPKIT-0096-FR-002", declaration["requirementIds"])
        self.assertEqual(declaration["declarationState"], "declared")
        self.assertIn("c", declaration["languageMappings"])
        signature = declaration["cSignature"]
        self.assertEqual(signature["returnType"], "int32_t")
        parameters = signature["parameters"]
        self.assertEqual(
            [parameter["name"] for parameter in parameters],
            ["builder", "index", "item"],
        )
        self.assertEqual(parameters[1]["type"], "uint64_t")


if __name__ == "__main__":
    unittest.main()
