"""Public declaration contract for the approved Python insertion adapter."""

import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class PythonExpertAttributePositionManifestTests(unittest.TestCase):
    def test_positioned_item_has_four_language_mappings_and_requirements(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        rows = [row for row in manifest["publicDeclarations"] if row["exportName"]
                == "kmipkit.fluent.SymmetricKeyBuilder.attribute_item_at"]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertEqual(row["kind"], "python-export")
        self.assertEqual(row["declarationState"], "declared")
        self.assertEqual(row["parityScope"], "four-language")
        self.assertEqual(set(row["languageMappings"]), {"rust", "c", "java", "python"})
        self.assertEqual(row["languageMappings"]["c"]["name"],
                         "kmipkit_symmetric_key_builder_insert_attribute_item")
        self.assertEqual(row["languageMappings"]["java"]["name"],
                         "org.kmipkit.fluent.SymmetricKeyBuilder.attributeItemAt")
        self.assertEqual(row["requirementIds"], [f"KMIPKIT-0096-FR-{index:03d}" for index in range(1, 5)])
        self.assertEqual(row["approvalSource"]["path"], "specs/096-fluent-expert-attributes/spec.md")


if __name__ == "__main__":
    unittest.main()
