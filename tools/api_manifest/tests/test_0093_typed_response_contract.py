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


if __name__ == "__main__":
    unittest.main()
