"""Manifest acceptance tests for KMIP 2.1 §§6.1.49–6.1.50 RNG mappings."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/027-rng-operations/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE": {
        "name": "RNG Retrieve",
        "section": "6.1.49",
        "functions": {
            "client_batch_item_rng_retrieve",
            "client_batch_item_response_rng_retrieve",
            "client_batch_item_response_rng_retrieve_ttlv",
        },
        "mappings": {
            "rust": {
                "kmipkit_client::Client::rng_retrieve",
                "kmipkit_protocol::RngRetrieveRequest::new",
                "kmipkit_protocol::RngRetrieveResponse::data",
                "kmipkit_protocol::RngRetrieveResponse::with_ttlv",
            },
            "c": {
                "kmipkit_client_batch_item_rng_retrieve",
                "kmipkit_client_batch_item_response_rng_retrieve",
                "kmipkit_client_batch_item_response_rng_retrieve_ttlv",
            },
            "java": {
                "org.kmipkit.extensions.ClientBatchItem.rngRetrieve",
                "org.kmipkit.extensions.ClientBatchItemResult.rngRetrieveResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.RngRetrieveResponse.data",
                "org.kmipkit.extensions.ClientBatchItemResult.RngRetrieveResponse.responseTtlv",
            },
            "python": {
                "kmipkit.extensions.client_batch_item_rng_retrieve",
                "kmipkit.client.BatchItemResult.rng_retrieve_response",
                "kmipkit.client.RngRetrieveResponse.data",
                "kmipkit.client.RngRetrieveResponse.response_ttlv",
            },
        },
    },
    "KMIPKIT-ELEM-OP-C2S-RNG-SEED": {
        "name": "RNG Seed",
        "section": "6.1.50",
        "functions": {
            "client_batch_item_rng_seed",
            "client_batch_item_response_rng_seed",
            "client_batch_item_response_rng_seed_ttlv",
        },
        "mappings": {
            "rust": {
                "kmipkit_client::Client::rng_seed",
                "kmipkit_protocol::RngSeedRequest::new",
                "kmipkit_protocol::RngSeedResponse::data_length",
                "kmipkit_protocol::RngSeedResponse::with_ttlv",
            },
            "c": {
                "kmipkit_client_batch_item_rng_seed",
                "kmipkit_client_batch_item_response_rng_seed",
                "kmipkit_client_batch_item_response_rng_seed_ttlv",
            },
            "java": {
                "org.kmipkit.extensions.ClientBatchItem.rngSeed",
                "org.kmipkit.extensions.ClientBatchItemResult.rngSeedResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.RngSeedResponse.dataLength",
                "org.kmipkit.extensions.ClientBatchItemResult.RngSeedResponse.responseTtlv",
            },
            "python": {
                "kmipkit.extensions.client_batch_item_rng_seed",
                "kmipkit.client.BatchItemResult.rng_seed_response",
                "kmipkit.client.RngSeedResponse.data_length",
                "kmipkit.client.RngSeedResponse.response_ttlv",
            },
        },
    },
}


class RngParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = {row["operationId"]: row for row in self.manifest["operations"]}
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        self.functions = {row["id"]: row for row in self.manifest["functions"]}

    def test_rng_operations_are_approved_and_have_four_language_parity(self) -> None:
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                operation = self.operations[operation_id]
                self.assertEqual(expected["name"], operation["name"])
                self.assertEqual(expected["section"], operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0041", operation["featureSpecId"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("KMIPKIT-0041", operation["ownerDependency"]["specId"])
                self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
                self.assertEqual("Approved for implementation", operation["approvalEvidence"]["status"])
                self.assertEqual(LANGUAGES, set(operation["declarations"]))

                linked = {
                    declaration_id
                    for ids in operation["declarations"].values()
                    for declaration_id in ids
                }
                self.assertTrue(linked)
                mapped_names = {language: set() for language in LANGUAGES}
                for language, declaration_ids in operation["declarations"].items():
                    self.assertTrue(declaration_ids, f"{expected['name']} has no {language} declarations")
                    for declaration_id in declaration_ids:
                        declaration = self.declarations[declaration_id]
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                        self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                        mapped_names[language].add(
                            declaration["languageMappings"][language]["name"]
                        )

                missing = {
                    language: sorted(names - mapped_names[language])
                    for language, names in expected["mappings"].items()
                    if names - mapped_names[language]
                }
                self.assertEqual({}, missing)

                evidence = set(operation["parityEvidenceIds"])
                self.assertIn("tools/api_manifest/tests/test_rng_parity_manifest.py", evidence)
                for evidence_path in evidence:
                    self.assertTrue((ROOT / evidence_path).is_file(), evidence_path)

    def test_request_and_response_functions_have_explicit_secret_and_ttlv_contracts(self) -> None:
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                missing = expected["functions"] - self.functions.keys()
                self.assertEqual(set(), missing)
                request = self.functions[
                    "client_batch_item_rng_retrieve"
                    if expected["name"] == "RNG Retrieve"
                    else "client_batch_item_rng_seed"
                ]
                self.assertTrue(
                    any(value.startswith("KMIPKIT-REQ-SPEC-") for value in request["requirementIds"])
                )
                self.assertIn("out_item", [item["name"] for item in request["c"]["parameters"]])
                if expected["name"] == "RNG Seed":
                    data = request["c"]["parameters"][0]
                    self.assertEqual("kmipkit_byte_span_v1_t *", data["type"])
                    self.assertEqual("borrowed", data["ownership"])
                    self.assertIn("zeroiz", " ".join(request["c"]["documentation"]).lower())

                response_ttlv = self.functions[
                    f"client_batch_item_response_{'rng_retrieve' if expected['name'] == 'RNG Retrieve' else 'rng_seed'}_ttlv"
                ]
                self.assertTrue(
                    any(value.startswith("KMIPKIT-REQ-SPEC-") for value in response_ttlv["requirementIds"])
                )
                self.assertIn("caller-owned copy", response_ttlv["c"]["documentation"][0])


if __name__ == "__main__":
    unittest.main()
