"""KMIPKIT-0095 bindings retain explicit ownership and codec parity."""
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class ResponseViewBindingsManifestTests(unittest.TestCase):
    def test_three_renderers_map_the_same_owned_full_response_in_all_languages(self):
        manifest = json.loads((ROOT / "specification/api/public-api.json").read_text())
        functions = {row["id"]: row for row in manifest["functions"]}
        for method, java in [("ttlv_hex", "ttlvHex"), ("json", "json"), ("xml", "xml")]:
            record = functions["client_batch_item_response_" + method]
            self.assertEqual(record["c"]["parameters"][1]["name"], "limits")
            self.assertEqual(record["c"]["parameters"][-1]["type"], "kmipkit_secret_bytes_t **")
            self.assertEqual(record["java"]["method"], java)
            self.assertEqual(record["java"]["returnType"], "SecretBytes")
            self.assertEqual(record["python"]["returnType"], "credentials.SecretBytes")
            self.assertEqual(record["ordering"], "preserves-response-field-wire-order")
            self.assertEqual(record["rust"]["requestBoundary"], "not-applicable")
        for row in manifest["publicDeclarations"]:
            if row["exportName"].startswith(("kmipkit::ClientBatchItemResponse::", "kmipkit_client::ClientBatchItemResponse::")) and row["exportName"].split("::")[-1] in {"ttlv_hex", "json", "xml"}:
                self.assertEqual(row["declarationState"], "verified")
                self.assertEqual(set(row["languageMappings"]), {"rust", "c", "java", "python"})
