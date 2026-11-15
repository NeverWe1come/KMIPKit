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

    def _assert_capability_mappings(
        self, capability_id: str, expected: dict[str, set[str]], label: str
    ) -> None:
        capability = next(
            row for row in self.manifest["capabilities"] if row["capabilityId"] == capability_id
        )
        for language, names in expected.items():
            with self.subTest(capability=capability_id, language=language):
                actual = set(capability["mappings"][language])
                self.assertTrue(
                    names <= actual,
                    f"missing {language} async {label}: {sorted(names - actual)}",
                )

    def test_async_capability_maps_operation_checked_typed_response_readers(self) -> None:
        readers = {
            "rust": {
                "kmipkit_client::ClientOperationOutcome::cancel_response",
                "kmipkit_client::ClientOperationOutcome::poll_response",
                "kmipkit_client::ClientOperationOutcome::process_response",
                "kmipkit_client::ClientOperationOutcome::query_async_requests_response",
            },
            "c": {
                "kmipkit_client_async_operation_outcome_cancel_response_v1",
                "kmipkit_client_async_operation_outcome_poll_response_v1",
                "kmipkit_client_async_operation_outcome_process_response_v1",
                "kmipkit_client_async_operation_outcome_query_async_requests_response_v1",
            },
            "java": {
                "org.kmipkit.extensions.AsyncOperationOutcome.cancelResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.pollResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.processResponse",
                "org.kmipkit.extensions.AsyncOperationOutcome.queryAsyncRequestsResponse",
            },
            "python": {
                "kmipkit.client.AsyncOperationOutcome.cancel_response",
                "kmipkit.client.AsyncOperationOutcome.poll_response",
                "kmipkit.client.AsyncOperationOutcome.process_response",
                "kmipkit.client.AsyncOperationOutcome.query_async_requests_response",
            },
        }
        response_fields = {
            "c": {
                "kmipkit_client_async_operation_response_v1_t.operation",
                "kmipkit_client_async_operation_response_v1_t.is_pending",
                "kmipkit_client_async_operation_response_v1_t.cancellation_result",
                "kmipkit_client_async_operation_response_v1_t.asynchronous_correlation_value",
                "kmipkit_client_async_operation_response_v1_t.response_payload_member_count",
            },
            "java": {
                "org.kmipkit.extensions.AsyncOperationResponseV1.result",
                "org.kmipkit.extensions.AsyncOperationResponseV1.raw",
                "org.kmipkit.extensions.AsyncCancelResponse.cancellationResult",
                "org.kmipkit.extensions.AsyncPollResponse.isPending",
                "org.kmipkit.extensions.AsyncProcessResponse.payloadMemberCount",
                "org.kmipkit.extensions.AsyncQueryAsyncRequestsResponse.isPending",
            },
            "python": {
                "kmipkit.client.AsyncOperationResponseV1.result",
                "kmipkit.client.AsyncOperationResponseV1.raw",
                "kmipkit.client.AsyncCancelResponse.cancellation_result",
                "kmipkit.client.AsyncPollResponse.is_pending",
                "kmipkit.client.AsyncProcessResponse.payload_member_count",
                "kmipkit.client.AsyncQueryAsyncRequestsResponse.is_pending",
            },
        }

        self._assert_capability_mappings(
            "typed_async_response_readers", readers, "typed response readers"
        )
        self._assert_capability_mappings(
            "response_ttlv_navigation", response_fields, "response fields"
        )


if __name__ == "__main__":
    unittest.main()
