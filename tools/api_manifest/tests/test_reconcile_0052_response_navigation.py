"""Tests for deterministic KMIPKIT-0052 manifest reconciliation."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest.reconcile_0052_response_navigation import reconcile


class Reconcile0052ResponseNavigationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "requirementIds": [],
            "functions": [],
            "publicDeclarations": [],
            "capabilities": [],
            "approvedExclusions": [],
        }

    def test_adds_response_and_async_typed_contract_routes(self) -> None:
        result = reconcile(self.manifest)

        self.assertEqual(len(result["functions"]), 6)
        self.assertEqual(
            {row["id"] for row in result["functions"]},
            {
                "client_batch_item_response_raw_ttlv",
                "client_batch_item_response_header_ttlv",
                "kmipkit_client_async_operation_outcome_cancel_response_v1",
                "kmipkit_client_async_operation_outcome_poll_response_v1",
                "kmipkit_client_async_operation_outcome_process_response_v1",
                "kmipkit_client_async_operation_outcome_query_async_requests_response_v1",
            },
        )
        self.assertEqual(len(result["publicDeclarations"]), 78)
        declaration_ids = {row["declarationId"] for row in result["publicDeclarations"]}
        self.assertTrue(
            {
                "kmipkit-0052-async-cancel-c",
                "kmipkit-0052-async-cancel-java",
                "kmipkit-0052-async-cancel-python",
                "kmipkit-0052-async-poll-c",
                "kmipkit-0052-async-poll-java",
                "kmipkit-0052-async-poll-python",
                "kmipkit-0052-async-process-c",
                "kmipkit-0052-async-process-java",
                "kmipkit-0052-async-process-python",
                "kmipkit-0052-async-query_async_requests-c",
                "kmipkit-0052-async-query_async_requests-java",
                "kmipkit-0052-async-query_async_requests-python",
            } <= declaration_ids
        )
        self.assertEqual(
            {row["exportName"] for row in result["approvedExclusions"]},
            {
                "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseRawTtlv",
                "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseHeaderTtlv",
                "kmipkit_client_async_operation_response_v1_t.struct_size",
                "kmipkit_client_async_operation_response_v1_t.abi_version",
                "kmipkit_client_async_operation_response_v1_t.reserved",
                "kmipkit.client.AsyncOperationResponseV1.__init__",
                "kmipkit.client.AsyncOperationResponseV1.__slots__",
                "kmipkit.client.AsyncCancelResponse.__init__",
                "kmipkit.client.AsyncCancelResponse.__slots__",
                "kmipkit.client.AsyncCancelResponse.raw",
                "kmipkit.client.AsyncCancelResponse.result",
                "kmipkit.client.AsyncPollResponse.__init__",
                "kmipkit.client.AsyncPollResponse.__slots__",
                "kmipkit.client.AsyncPollResponse.raw",
                "kmipkit.client.AsyncPollResponse.result",
                "kmipkit.client.AsyncProcessResponse.__init__",
                "kmipkit.client.AsyncProcessResponse.__slots__",
                "kmipkit.client.AsyncProcessResponse.raw",
                "kmipkit.client.AsyncProcessResponse.result",
                "kmipkit.client.AsyncQueryAsyncRequestsResponse.__init__",
                "kmipkit.client.AsyncQueryAsyncRequestsResponse.raw",
                "kmipkit.client.AsyncQueryAsyncRequestsResponse.result",
            },
        )
        capability = result["capabilities"][0]
        self.assertEqual(capability["capabilityId"], "response_ttlv_navigation")
        self.assertIn("KMIPKIT-0052-FR-001", capability["requirementIds"])
        self.assertIn(
            "kmipkit::ClientBatchItemResponse::raw", capability["mappings"]["rust"]
        )
        self.assertIn(
            "kmipkit_client_batch_item_response_raw_ttlv", capability["mappings"]["c"]
        )
        self.assertIn(
            "crates/kmipkit-ffi/tests/support/async_operation_abi_tests.rs",
            capability["parityEvidenceIds"],
        )
        self.assertIn(
            "bindings/python/tests/test_async_operation_mapping.py",
            capability["parityEvidenceIds"],
        )

    def test_reconciliation_is_deterministic_and_does_not_mutate_input(self) -> None:
        original = copy.deepcopy(self.manifest)

        first = reconcile(self.manifest)
        second = reconcile(self.manifest)

        self.assertEqual(first, second)
        self.assertEqual(self.manifest, original)

    def test_rejects_an_existing_route_with_a_different_contract(self) -> None:
        self.manifest["publicDeclarations"] = [{
            "declarationId": "kmipkit-0052-response-payload-rust",
            "packageId": "rust-kmipkit-client",
            "kind": "associated-item",
            "exportName": "kmipkit_client::ClientBatchItemResponse::raw",
            "capabilityId": "unexpected-capability",
            "requirementIds": [],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/052-response-navigation/spec.md",
                "status": "Approved",
                "evidence": "Approved",
            },
            "languageMappings": {},
            "parityEvidenceIds": [],
        }]

        with self.assertRaisesRegex(ValueError, "unexpected existing declaration contract"):
            reconcile(self.manifest)


if __name__ == "__main__":
    unittest.main()
