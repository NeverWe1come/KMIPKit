"""Regression tests for reviewed T076 shared-client public mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


class T076ClientMappingTests(unittest.TestCase):
    def test_client_batch_create_is_mapped_and_emitted_in_public_header(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declaration = next(
            item
            for item in manifest["publicDeclarations"]
            if item["packageId"] == "c-kmipkit"
            and item["exportName"] == "kmipkit_client_batch_create"
        )

        self.assertEqual(declaration["declarationState"], "declared")
        self.assertEqual(declaration["capabilityId"], "client_batch_execution")
        self.assertEqual(
            declaration["approvalSource"]["path"],
            "specs/006-message-batch-model/spec.md",
        )
        self.assertEqual(
            {
                language: mapping["name"]
                for language, mapping in declaration["languageMappings"].items()
            },
            {
                "rust": "kmipkit_client::ClientBatch::empty",
                "c": "kmipkit_client_batch_create",
                "java": "org.kmipkit.extensions.ClientBatch.create",
                "python": "kmipkit.client.Batch.create",
            },
        )

        header = generate.render_c_header(manifest)
        self.assertIn("typedef struct kmipkit_client_batch_t kmipkit_client_batch_t;", header)
        self.assertIn(
            "int32_t kmipkit_client_batch_create(kmipkit_client_batch_t ** out_batch);",
            header,
        )


if __name__ == "__main__":
    unittest.main()
