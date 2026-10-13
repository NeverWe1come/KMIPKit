"""Regressions for the reviewed T076 shared-client public API mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
T076_SYMBOLS = {
    "kmipkit_client_batch_add_error_continuation_value",
    "kmipkit_client_batch_add_item",
    "kmipkit_client_batch_create",
    "kmipkit_client_batch_item_with_unique_batch_item_id",
    "kmipkit_client_batch_item_response_metadata_v1",
    "kmipkit_client_batch_item_response_pending_correlation_value",
    "kmipkit_client_batch_item_response_release",
    "kmipkit_client_batch_item_response_t",
    "kmipkit_client_batch_item_response_unique_batch_item_id",
    "kmipkit_client_batch_release",
    "kmipkit_client_batch_response_item_at",
    "kmipkit_client_batch_response_item_count",
    "kmipkit_client_batch_response_release",
    "kmipkit_client_batch_response_t",
    "kmipkit_client_batch_set_asynchronous_indicator",
    "kmipkit_client_batch_set_client_correlation_value",
    "kmipkit_client_batch_set_order_option",
    "kmipkit_client_batch_set_request_time_stamp",
    "kmipkit_client_batch_t",
    "kmipkit_client_create",
    "kmipkit_client_execute",
    "kmipkit_client_release",
    "kmipkit_client_t",
    "kmipkit_transport_config_create_v1",
    "kmipkit_transport_config_release",
    "kmipkit_transport_config_t",
    "kmipkit_client_execute_with_error_ttlv_v1",
}
T076_BLOCKED = {
    "kmipkit_client_batch_item_response_metadata_v1",
    "kmipkit_client_execute_with_error_ttlv_v1",
}
T076_READY = T076_SYMBOLS - T076_BLOCKED


class T076ClientMappingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("declarationId", "").startswith("T076:")
        ]

    def test_all_27_exports_are_mapped_or_remain_exactly_blocked(self) -> None:
        c_rows = [row for row in self.declarations if row["packageId"] == "c-kmipkit"]
        self.assertEqual(T076_SYMBOLS, {row["exportName"] for row in c_rows})
        self.assertEqual(27, len(c_rows))

        ready = {
            row["exportName"]
            for row in c_rows
            if row["declarationState"] in {"declared", "verified"}
        }
        blocked = {
            row["exportName"]
            for row in c_rows
            if row["declarationState"] == "blocked"
        }
        self.assertEqual(T076_READY, ready)
        self.assertEqual(T076_BLOCKED, blocked)

        for row in c_rows:
            if row["declarationState"] == "blocked":
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("approvalSource", row)
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("signature", row)
            else:
                self.assertIn("approvalSource", row)
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )

    def test_ready_function_mappings_resolve_to_real_four_language_apis(self) -> None:
        expected = {
            "client_batch_add_error_continuation_value": (
                "kmipkit_client::ClientBatch::with_batch_error_continuation_option",
                "kmipkit_client_batch_add_error_continuation_value",
                "org.kmipkit.extensions.ClientBatch.addErrorContinuationValue",
                "kmipkit.client.Batch.add_error_continuation_value",
            ),
            "client_batch_add_item": (
                "kmipkit_client::ClientBatch::push",
                "kmipkit_client_batch_add_item",
                "org.kmipkit.extensions.ClientBatch.add",
                "kmipkit.client.Batch.add_item",
            ),
            "client_batch_create": (
                "kmipkit_client::ClientBatch::empty",
                "kmipkit_client_batch_create",
                "org.kmipkit.extensions.ClientBatch.create",
                "kmipkit.client.Batch.create",
            ),
            "client_batch_item_with_unique_batch_item_id": (
                "kmipkit_client::ClientBatchItem::with_unique_batch_item_id",
                "kmipkit_client_batch_item_with_unique_batch_item_id",
                "org.kmipkit.extensions.ClientBatchItem.withUniqueBatchItemId",
                "kmipkit.extensions.with_unique_batch_item_id",
            ),
            "client_batch_item_response_pending_correlation_value": (
                "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value",
                "kmipkit_client_batch_item_response_pending_correlation_value",
                "org.kmipkit.extensions.ClientBatchItemResult.pendingCorrelationValue",
                "kmipkit.client.BatchItemResult.pending_correlation_value",
            ),
            "client_batch_item_response_unique_batch_item_id": (
                "kmipkit_client::ClientBatchItemResponse::unique_batch_item_id",
                "kmipkit_client_batch_item_response_unique_batch_item_id",
                "org.kmipkit.extensions.ClientBatchItemResult.uniqueBatchItemId",
                "kmipkit.client.BatchItemResult.unique_batch_item_id",
            ),
            "client_batch_response_item_at": (
                "kmipkit_client::ClientBatchResponse::get",
                "kmipkit_client_batch_response_item_at",
                "org.kmipkit.extensions.ClientBatchResponse.itemAt",
                "kmipkit.client.BatchResult.items",
            ),
            "client_batch_response_item_count": (
                "kmipkit_client::ClientBatchResponse::len",
                "kmipkit_client_batch_response_item_count",
                "org.kmipkit.extensions.ClientBatchResponse.itemCount",
                "kmipkit.client.BatchResult.items",
            ),
            "client_batch_set_asynchronous_indicator": (
                "kmipkit_client::ClientBatch::with_asynchronous_indicator",
                "kmipkit_client_batch_set_asynchronous_indicator",
                "org.kmipkit.extensions.ClientBatch.asynchronousIndicator",
                "kmipkit.client.Batch.set_asynchronous_indicator",
            ),
            "client_batch_set_client_correlation_value": (
                "kmipkit_client::ClientBatch::with_client_correlation_value",
                "kmipkit_client_batch_set_client_correlation_value",
                "org.kmipkit.extensions.ClientBatch.clientCorrelationValue",
                "kmipkit.client.Batch.set_client_correlation_value",
            ),
            "client_batch_set_order_option": (
                "kmipkit_client::ClientBatch::with_batch_order_option",
                "kmipkit_client_batch_set_order_option",
                "org.kmipkit.extensions.ClientBatch.ordered",
                "kmipkit.client.Batch.set_ordered",
            ),
            "client_batch_set_request_time_stamp": (
                "kmipkit_client::ClientBatch::with_request_time_stamp",
                "kmipkit_client_batch_set_request_time_stamp",
                "org.kmipkit.extensions.ClientBatch.requestTimestamp",
                "kmipkit.client.Batch.set_request_timestamp",
            ),
            "client_create": (
                "kmipkit_client::Client::new",
                "kmipkit_client_create",
                "org.kmipkit.extensions.Client.create",
                "kmipkit.client.Client.create",
            ),
            "client_execute": (
                "kmipkit_client::Client::execute",
                "kmipkit_client_execute",
                "org.kmipkit.extensions.Client.execute",
                "kmipkit.client.Client.execute",
            ),
            "transport_config_create_v1": (
                "kmipkit_transport::TransportConfigBuilder::build",
                "kmipkit_transport_config_create_v1",
                "org.kmipkit.extensions.TransportConfiguration.Builder.build",
                "kmipkit.client.TransportConfig",
            ),
        }
        functions = {record["id"]: record for record in self.manifest["functions"]}
        runtime_sources = {
            "rust": (
                ROOT / "crates" / "kmipkit-client" / "src" / "execute.rs",
                ROOT / "crates" / "kmipkit-transport" / "src" / "config.rs",
            ),
            "c": (
                ROOT / "crates" / "kmipkit-ffi" / "src" / "extension_registry" / "client_abi.rs",
                ROOT / "crates" / "kmipkit-ffi" / "src" / "extension_registry.rs",
            ),
            "java": tuple((ROOT / "bindings" / "java" / "src" / "main" / "java" / "org" / "kmipkit" / "extensions").glob("*.java")),
            "python": (ROOT / "bindings" / "python" / "src" / "kmipkit" / "client.py", ROOT / "bindings" / "python" / "src" / "kmipkit" / "extensions.py"),
        }

        for function_id, names in expected.items():
            self.assertIn(function_id, functions)
            function = functions[function_id]
            self.assertEqual(names[0], f"{function['rust']['module']}::{function['rust']['name']}")
            self.assertEqual(names[1], function["c"]["symbol"])
            self.assertEqual(names[2], f"{function['java']['package']}.{function['java']['class']}.{function['java']['method']}")
            self.assertEqual(names[3], f"{function['python']['module']}.{function['python']['function']}")
            for language, expected_name in zip(("rust", "c", "java", "python"), names):
                source_text = "\n".join(path.read_text(encoding="utf-8") for path in runtime_sources[language])
                member = expected_name.rsplit("::", 1)[-1].rsplit(".", 1)[-1]
                self.assertRegex(source_text, re.compile(rf"\b{re.escape(member)}\b"))

    def test_generated_public_header_and_c_consumer_use_the_same_t076_surface(self) -> None:
        header = generate.render_c_header(self.manifest)
        consumer = (ROOT / "bindings" / "c" / "tests" / "client_abi_consumer.c").read_text(
            encoding="utf-8"
        )
        self.assertIn('#include "kmipkit.h"', consumer)

        for export_name in T076_READY:
            declaration = next(row for row in self.declarations if row["exportName"] == export_name and row["packageId"] == "c-kmipkit")
            if declaration["kind"] == "type":
                self.assertIn(f"typedef struct {export_name} {export_name};", header)
            else:
                self.assertIn(f"{export_name}(", header)
            self.assertNotRegex(consumer, re.compile(rf"^.*\b{re.escape(export_name)}\s*\(", re.MULTILINE))

        for export_name in T076_BLOCKED:
            self.assertNotIn(f"{export_name}(", header)


if __name__ == "__main__":
    unittest.main()
