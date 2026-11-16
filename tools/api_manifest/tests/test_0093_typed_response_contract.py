"""KMIPKIT-0093 response adapters preserve typed results and representations."""

import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class TypedResponseContractManifestTests(unittest.TestCase):
    def test_typed_batch_result_maps_status_operation_and_pending_correlation(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        required = {
            "KMIPKIT-0093-FR-005",
            "KMIPKIT-0093-FR-008",
        }
        expected = {
            "kmipkit_client::ClientBatchItemResponse::outcome",
            "kmipkit_client::ClientBatchOutcome::operation",
            "kmipkit_client::ClientBatchOutcome::result",
            "kmipkit_client::ClientBatchOutcome::response",
            "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value",
        }
        declarations = {
            row["exportName"]: row
            for row in manifest["publicDeclarations"]
            if row["exportName"] in expected
        }
        self.assertEqual(set(declarations), expected)
        for export_name, row in declarations.items():
            with self.subTest(export=export_name):
                self.assertTrue(required.issubset(set(row["requirementIds"])))
                self.assertEqual(set(row["languageMappings"]), {"rust", "c", "java", "python"})

    def test_full_response_encodings_map_to_the_master_contract_in_all_languages(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        required = {"KMIPKIT-0093-FR-009"}
        functions = {row["id"]: row for row in manifest["functions"]}
        for function_id in (
            "client_batch_item_response_ttlv_hex",
            "client_batch_item_response_json",
            "client_batch_item_response_xml",
        ):
            with self.subTest(function=function_id):
                row = functions[function_id]
                self.assertTrue(required.issubset(set(row["requirementIds"])))
                self.assertEqual(row["java"]["returnType"], "SecretBytes")
                self.assertEqual(row["python"]["returnType"], "credentials.SecretBytes")
                self.assertEqual(row["ordering"], "preserves-response-field-wire-order")

    def test_c_response_renderers_document_caller_managed_copies(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        functions = {row["id"]: row for row in manifest["functions"]}
        for function_id in (
            "client_batch_item_response_ttlv_hex",
            "client_batch_item_response_json",
            "client_batch_item_response_xml",
        ):
            with self.subTest(function=function_id):
                documentation = " ".join(
                    functions[function_id]["c"]["documentation"]
                ).lower()
                self.assertIn("caller", documentation)
                self.assertIn("cannot be zeroized", documentation)


    def test_operation_response_view_requirement_mapping_is_four_language_parity(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        declarations = manifest["publicDeclarations"]
        by_package_and_name = {
            (row["packageId"], row["exportName"]): row for row in declarations
        }
        response_views = [
            row
            for row in declarations
            if "ClientResponseView::" in row["exportName"]
            and "KMIPKIT-0093-FR-008" in row["requirementIds"]
        ]
        self.assertGreater(len(response_views), 0)
        for row in response_views:
            with self.subTest(export=row["exportName"]):
                self.assertNotIn("KMIPKIT-0093-FR-005", row["requirementIds"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                for language, mapping in row["languageMappings"].items():
                    target = by_package_and_name[
                        (mapping["packageId"], mapping["name"])
                    ]
                    with self.subTest(language=language):
                        self.assertIn("KMIPKIT-0093-FR-008", target["requirementIds"])


if __name__ == "__main__":
    unittest.main()
