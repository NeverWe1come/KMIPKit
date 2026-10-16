"""Manifest regressions for the T032 creation operation mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CREATION_OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-CREATE",
    "KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR",
    "KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY",
}
CREATION_FUNCTIONS = {
    "kmipkit_client_batch_item_create",
    "kmipkit_client_batch_item_create_key_pair",
    "kmipkit_client_batch_item_create_split_key",
    "kmipkit_client_batch_item_response_metadata_v2",
    "kmipkit_client_batch_item_response_create",
    "kmipkit_client_batch_item_response_create_key_pair",
    "kmipkit_client_batch_item_response_create_split_key_count",
    "kmipkit_client_batch_item_response_create_split_key_unique_identifier_at",
}


class CreationParityManifestTests(unittest.TestCase):
    def test_creation_operations_have_approved_four_language_mappings(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operations = {
            row["operationId"]: row
            for row in manifest["operations"]
            if row["operationId"] in CREATION_OPERATIONS
        }
        declarations = {
            row["declarationId"]: row for row in manifest["publicDeclarations"]
        }

        self.assertEqual(CREATION_OPERATIONS, set(operations))
        for operation_id, operation in operations.items():
            with self.subTest(operation=operation_id):
                self.assertEqual("declared", operation["apiState"])
                self.assertEqual("KMIPKIT-0014", operation["featureSpecId"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(operation["declarations"]),
                )
                for language, declaration_ids in operation["declarations"].items():
                    self.assertTrue(declaration_ids)
                    for declaration_id in declaration_ids:
                        declaration = declarations[declaration_id]
                        self.assertIn(
                            declaration["capabilityId"],
                            {operation_id, "server_generated_creation", "client_execution"},
                        )
                        self.assertEqual("declared", declaration["declarationState"])
                        self.assertEqual(
                            {"rust", "c", "java", "python"},
                            set(declaration["languageMappings"]),
                        )
                        self.assertIn(language, declaration["languageMappings"])

    def test_creation_ffi_functions_and_versioned_carrier_types_are_manifested(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        functions = {
            row["c"]["symbol"] for row in manifest["functions"]
        }
        types = {row["id"] for row in manifest["types"]}

        self.assertTrue(CREATION_FUNCTIONS <= functions)
        self.assertTrue(
            {"UniqueIdentifierV1", "ClientBatchItemMetadataV2"} <= types
        )

    def test_creation_response_text_identifiers_document_borrowed_lifetime(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        symbols = {
            "kmipkit_client_batch_item_response_create",
            "kmipkit_client_batch_item_response_create_key_pair",
            "kmipkit_client_batch_item_response_create_split_key_unique_identifier_at",
        }
        functions = {
            row["c"]["symbol"]: row["c"].get("documentation", [])
            for row in manifest["functions"]
            if row.get("c", {}).get("symbol") in symbols
        }

        self.assertEqual(symbols, set(functions))
        for symbol, documentation in functions.items():
            with self.subTest(symbol=symbol):
                text = " ".join(documentation).lower()
                self.assertIn("borrowed", text)
                self.assertIn("response item handle", text)
                self.assertIn("copy", text)


if __name__ == "__main__":
    unittest.main()
