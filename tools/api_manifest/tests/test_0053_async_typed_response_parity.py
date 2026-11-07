"""Require operation-checked asynchronous response routes in every binding."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]


class AsyncTypedResponseParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = generate._load_json(
            ROOT, generate.MANIFEST_PATH, generate.MAX_MANIFEST_BYTES
        )

    def test_async_capability_maps_operation_checked_typed_response_readers(self) -> None:
        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_asynchronous_operations"
        )
        required = {
            "rust": {
                "kmipkit::ClientOperationOutcome::cancel_response",
                "kmipkit::ClientOperationOutcome::poll_response",
                "kmipkit::ClientOperationOutcome::process_response",
                "kmipkit::ClientOperationOutcome::query_async_requests_response",
            },
            "c": {
                "kmipkit_client_async_operation_outcome_cancel_response_v1",
                "kmipkit_client_async_operation_outcome_poll_response_v1",
                "kmipkit_client_async_operation_outcome_process_response_v1",
                "kmipkit_client_async_operation_outcome_query_async_requests_response_v1",
                "kmipkit_client_async_operation_response_v1_t.operation",
                "kmipkit_client_async_operation_response_v1_t.is_pending",
                "kmipkit_client_async_operation_response_v1_t.response_payload_member_count",
            },
            "java": {
                "org.kmipkit.extensions.AsyncOperationOutcome.cancelResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.pollResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.processResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.queryAsyncRequestsResponse",
                "org.kmipkit.extensions.AsyncOperationResponseV1.result",
                "org.kmipkit.extensions.AsyncCancelResponse.cancellationResult",
                "org.kmipkit.extensions.AsyncPollResponse.isPending",
                "org.kmipkit.extensions.AsyncProcessResponse.payloadMemberCount",
                "org.kmipkit.extensions.AsyncQueryAsyncRequestsResponse.isPending",
            },
            "python": {
                "kmipkit.client.AsyncOperationOutcome.cancel_response",
                "kmipkit.client.AsyncOperationOutcome.poll_response",
                "kmipkit.client.AsyncOperationOutcome.process_response",
                "kmipkit.client.AsyncOperationOutcome.query_async_requests_response",
                "kmipkit.client.AsyncOperationResponseV1.result",
                "kmipkit.client.AsyncCancelResponse.cancellation_result",
                "kmipkit.client.AsyncPollResponse.is_pending",
                "kmipkit.client.AsyncProcessResponse.payload_member_count",
                "kmipkit.client.AsyncQueryAsyncRequestsResponse.is_pending",
            },
        }

        for language, names in required.items():
            with self.subTest(language=language):
                self.assertTrue(
                    names <= set(capability["mappings"][language]),
                    f"missing {language} async response mappings: "
                    f"{sorted(names - set(capability['mappings'][language]))}",
                )


if __name__ == "__main__":
    unittest.main()
