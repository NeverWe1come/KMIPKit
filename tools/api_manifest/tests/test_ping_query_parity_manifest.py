"""Require verified four-language mappings for Ping and Query."""

from __future__ import annotations

import ast
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OWNER_SPEC = "specs/020-query-ping/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}

OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-PING": {
        "name": "Ping",
        "section": "6.1.36",
        "request": {
            "rust": "kmipkit_client::ClientBatchItem::ping",
            "c": "kmipkit_client_batch_item_ping",
            "java": "org.kmipkit.extensions.ClientBatchItem.ping",
            "python": "kmipkit.extensions.client_batch_item_ping",
        },
        "response": {
            "rust": "kmipkit_client::ClientResponseView::ping",
            "c": "kmipkit_client_batch_item_response_ping",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.pingResponse",
            "python": "kmipkit.client.BatchItemResult.ping_response",
        },
        "normative_requirements": set(),
    },
    "KMIPKIT-ELEM-OP-C2S-QUERY": {
        "name": "Query",
        "section": "6.1.40",
        "request": {
            "rust": "kmipkit_client::ClientBatchItem::query",
            "c": "kmipkit_client_batch_item_query",
            "java": "org.kmipkit.extensions.ClientBatchItem.query",
            "python": "kmipkit.extensions.client_batch_item_query",
        },
        "response": {
            "rust": "kmipkit_protocol::QueryResponse::to_ttlv_payload",
            "c": "kmipkit_client_batch_item_response_query_ttlv",
            "java": "org.kmipkit.extensions.ClientBatchItemResult.queryResponse",
            "python": "kmipkit.client.BatchItemResult.query_response",
        },
        "normative_requirements": {
            "KMIPKIT-REQ-SPEC-6.1.40-014",
            "KMIPKIT-REQ-SPEC-6.1.40-016",
        },
    },
}
OWNER_REQUIREMENTS = {f"KMIPKIT-0020-FR-{index:03}" for index in range(1, 13)}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ClientBatchItemPingQueryMappingTest.java",
    "bindings/python/tests/test_query_ping_operation.py",
    "crates/kmipkit-client/tests/unit/ping_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/query_execution_tests.rs",
}


class PingQueryParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        self.functions = {row["id"]: row for row in self.manifest["functions"]}

    def test_ping_and_query_are_verified_with_complete_four_language_mappings(self) -> None:
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                operation = next(
                    row for row in self.manifest["operations"]
                    if row["operationId"] == operation_id
                )
                self.assertEqual(expected["name"], operation["name"])
                self.assertEqual(expected["section"], operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0020", operation["featureSpecId"])
                self.assertEqual(
                    {"state": "approved", "specId": "KMIPKIT-0020"},
                    operation["ownerDependency"],
                )
                self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
                self.assertEqual("Approved", operation["approvalEvidence"]["status"])
                self.assertEqual(LANGUAGES, set(operation["declarations"]))
                self.assertTrue(EVIDENCE <= set(operation["parityEvidenceIds"]))
                for path in EVIDENCE:
                    self.assertTrue((ROOT / path).is_file(), path)

                linked_requirements: set[str] = set()
                for endpoint in ("request", "response"):
                    function_id = (
                        f"client_batch_item_{expected['name'].lower()}"
                        if endpoint == "request"
                        else f"client_batch_item_response_{expected['name'].lower()}"
                    )
                    function = self.functions[function_id]
                    actual_mappings = {
                        "rust": f"{function['rust']['module']}::{function['rust']['name']}",
                        "c": function["c"]["symbol"],
                        "java": (
                            f"org.kmipkit.{function['java']['package'].split('.')[-1]}"
                            f".{function['java']['class']}.{function['java']['method']}"
                        ),
                        "python": function["python"]["function"],
                    }
                    self.assertEqual(expected[endpoint], actual_mappings)
                    for language in LANGUAGES:
                        declaration_ids = operation["declarations"][language]
                        self.assertEqual(2, len(declaration_ids), language)
                        declaration = self.declarations[declaration_ids[0 if endpoint == "request" else 1]]
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                        self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                        self.assertEqual("Approved", declaration["approvalSource"]["status"])
                        self.assertEqual(
                            expected[endpoint][language],
                            declaration["languageMappings"][language]["name"],
                        )
                        linked_requirements.update(declaration["requirementIds"])
                        if expected["name"] == "Query" and endpoint == "request":
                            self.assertTrue(
                                expected["normative_requirements"]
                                <= set(declaration["requirementIds"])
                            )
                self.assertTrue(OWNER_REQUIREMENTS <= linked_requirements)
                self.assertTrue(API_REQUIREMENTS <= linked_requirements)

    def test_query_with_limits_has_a_separate_verified_four_language_mapping(self) -> None:
        function = self.functions["client_batch_item_query_with_limits"]
        expected = {
            "rust": "kmipkit_client::ClientBatchItem::query_with_limits",
            "c": "kmipkit_client_batch_item_query_with_limits",
            "java": "org.kmipkit.extensions.ClientBatchItem.queryWithLimits",
            "python": "kmipkit.extensions.client_batch_item_query_with_limits",
        }
        actual = {
            "rust": f"{function['rust']['module']}::{function['rust']['name']}",
            "c": function["c"]["symbol"],
            "java": (
                f"org.kmipkit.{function['java']['package'].split('.')[-1]}"
                f".{function['java']['class']}.{function['java']['method']}"
            ),
            "python": function["python"]["function"],
        }
        self.assertEqual(expected, actual)
        self.assertEqual("CodecLimits", function["java"]["parameters"][-1]["type"])
        self.assertEqual("CodecLimits", function["python"]["parameters"][-1]["type"])
        for language, export_name in expected.items():
            declaration = self.declarations[
                f"T041:query:request-with-limits:{language}"
            ]
            self.assertEqual("verified", declaration["declarationState"])
            self.assertEqual("four-language", declaration["parityScope"])
            self.assertEqual(export_name, declaration["exportName"])

        root_alias = next(
            row for row in self.manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit::ClientBatchItem::query_with_limits"
        )
        self.assertEqual("verified", root_alias["declarationState"])
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-QUERY", root_alias["capabilityId"])
        self.assertEqual("four-language", root_alias["parityScope"])
        self.assertEqual(
            {
                **expected,
                "rust": "kmipkit::ClientBatchItem::query_with_limits",
            },
            {
                language: mapping["name"]
                for language, mapping in root_alias["languageMappings"].items()
            },
        )
        self.assertIn(
            "bindings/python/tests/test_query_input_snapshot_security.py",
            root_alias["parityEvidenceIds"],
        )
        batch_result_fields = {
            row["exportName"]: row["kind"]
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"] in {
                "kmipkit.client.BatchItemResult.decrypt_response",
                "kmipkit.client.BatchItemResult.encrypt_response",
                "kmipkit.client.BatchItemResult.ping_response",
                "kmipkit.client.BatchItemResult.query_response",
            }
        }
        self.assertEqual(
            {
                "kmipkit.client.BatchItemResult.decrypt_response": "field",
                "kmipkit.client.BatchItemResult.encrypt_response": "field",
                "kmipkit.client.BatchItemResult.ping_response": "field",
                "kmipkit.client.BatchItemResult.query_response": "field",
            },
            batch_result_fields,
        )

    def test_default_query_constructors_document_payload_and_message_limits(self) -> None:
        function = self.functions["client_batch_item_query"]
        c_documentation = " ".join(function["c"]["documentation"])
        self.assertIn("payload against the default codec limits", c_documentation)
        self.assertIn("complete Request Message", c_documentation)

        python_source = (ROOT / "bindings/python/src/kmipkit/extensions.py").read_text(
            encoding="utf-8"
        )
        python_tree = ast.parse(python_source)
        python_constructor = next(
            node
            for node in ast.walk(python_tree)
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == "client_batch_item_query"
        )
        python_documentation = ast.get_docstring(python_constructor) or ""
        self.assertIn("payload against the default codec limits", python_documentation)
        self.assertIn("complete Request Message", python_documentation)

    def test_generated_parity_inventory_contains_verified_ping_and_query(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = {
            row["operationId"]: row
            for row in generated["operations"]
            if row["operationId"] in OPERATIONS
        }
        self.assertEqual(set(OPERATIONS), set(operations))
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=expected["name"]):
                self.assertEqual("verified", operations[operation_id]["apiState"])
                self.assertEqual(
                    LANGUAGES,
                    set(operations[operation_id]["declarations"]),
                )


if __name__ == "__main__":
    unittest.main()
