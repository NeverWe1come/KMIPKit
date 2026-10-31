"""T091 contract for immutable ClientBatch item readback parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REQUIREMENTS = [
    "KMIPKIT-0006-FR-005",
    "KMIPKIT-0006-FR-008",
    "KMIPKIT-0006-FR-014",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
]


class T091BatchItemReadbackManifestTests(unittest.TestCase):
    def test_client_batch_items_map_to_owned_indexed_readback_in_every_facade(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        self.assertTrue(set(REQUIREMENTS) <= set(manifest["requirementIds"]))

        rust = declarations[("rust-kmipkit-client", "kmipkit_client::ClientBatch::items")]
        self.assertEqual("declared", rust["declarationState"])
        self.assertEqual(REQUIREMENTS, rust["requirementIds"])
        self.assertEqual(
            {
                "packageId": "rust-kmipkit-client",
                "name": "kmipkit_client::ClientBatch::items",
            },
            rust["languageMappings"]["rust"],
        )
        self.assertEqual(
            {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_at"},
            rust["languageMappings"]["c"],
        )
        self.assertEqual(
            {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.ClientBatch.itemAt",
            },
            rust["languageMappings"]["java"],
        )
        self.assertEqual(
            {"packageId": "python-kmipkit", "name": "kmipkit.client.Batch.item_at"},
            rust["languageMappings"]["python"],
        )

        for package_id, export_name in (
            ("c-kmipkit", "kmipkit_client_batch_item_count"),
            ("c-kmipkit", "kmipkit_client_batch_item_at"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatch.itemCount"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatch.itemAt"),
            ("python-kmipkit", "kmipkit.client.Batch.item_count"),
            ("python-kmipkit", "kmipkit.client.Batch.item_at"),
        ):
            with self.subTest(export=export_name):
                declaration = declarations[(package_id, export_name)]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("client_batch_model", declaration["capabilityId"])
                self.assertEqual(REQUIREMENTS, declaration["requirementIds"])


if __name__ == "__main__":
    unittest.main()
