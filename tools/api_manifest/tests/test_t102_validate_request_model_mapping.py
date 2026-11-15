"""Require four-language parity for the complete Validate request/result model."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-VALIDATE"
OWNER_SPEC = "specs/028-diagnostics-interop/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUEST_EXPORTS = {
    "kmipkit_protocol::ValidateRequest::chain_len",
    "kmipkit_protocol::ValidateRequest::into_ttlv_payload",
    "kmipkit_protocol::ValidateRequest::validity_date",
    "kmipkit_protocol::ValidateRequest::with_extension",
}
RESULT_EXPORT = "kmipkit_protocol::ValidateResponse::result"
EXPECTED_MAPPINGS = {language: set() for language in LANGUAGES}
REQUIREMENTS = {
    *(f"KMIPKIT-0042-FR-{index:03}" for index in range(1, 9)),
    *(
        f"KMIPKIT-0043-FR-{index:03}"
        for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
    ),
    "KMIPKIT-REQ-SPEC-6.1.57-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002",
    "KMIPKIT-REQ-SPEC-6.1.57-004",
    "KMIPKIT-REQ-SPEC-6.1.57-005",
    "KMIPKIT-REQ-SPEC-6.1.57-006",
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/validate_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/validate_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ValidateJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/ValidateResponseMappingTest.java",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_validate_operation.py",
    "bindings/python/tests/test_typed_messages.py",
    "tools/api_manifest/tests/test_validate_parity_manifest.py",
    "tools/api_manifest/tests/test_t102_validate_request_model_mapping.py",
}
REQUEST_MAPPING = {
    "c": "kmipkit_client_batch_item_validate",
    "java": "org.kmipkit.extensions.ClientBatchItem.validate",
    "python": "kmipkit.extensions.client_batch_item_validate",
}
RESULT_MAPPING = {
    "c": "kmipkit_client_batch_item_response_metadata_v2",
    "java": "org.kmipkit.extensions.ClientBatchItemResult.metadataV2",
    "python": "kmipkit.client.BatchItemResult.metadata_v2",
}


class ValidateRequestModelMappingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operation = next(
            row
            for row in self.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        self.declarations_by_name = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_validate_request_helpers_and_result_map_to_existing_facades(self) -> None:
        assert_declaration_mappings(
            self,
            self.operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
        )

        mapped_exports = REQUEST_EXPORTS | {RESULT_EXPORT}
        operation_exports = {
            self.declarations[declaration_id]["exportName"]
            for declaration_id in self.operation["declarations"]["rust"]
        }
        self.assertTrue(mapped_exports <= operation_exports)

        for export_name in REQUEST_EXPORTS:
            declaration = self.declarations_by_name[export_name]
            self.assertEqual("verified", declaration["declarationState"])
            self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
            self.assertEqual(
                "kmipkit_client_batch_item_validate",
                declaration["languageMappings"]["c"]["name"],
            )
            self.assertEqual(
                "org.kmipkit.extensions.ClientBatchItem.validate",
                declaration["languageMappings"]["java"]["name"],
            )
            self.assertEqual(
                "kmipkit.extensions.client_batch_item_validate",
                declaration["languageMappings"]["python"]["name"],
            )
            self.assertTrue(
                PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]),
                export_name,
            )

        result = self.declarations_by_name[RESULT_EXPORT]
        self.assertEqual("verified", result["declarationState"])
        self.assertEqual(OWNER_SPEC, result["approvalSource"]["path"])
        for language, expected_name in RESULT_MAPPING.items():
            self.assertEqual(expected_name, result["languageMappings"][language]["name"])
        self.assertTrue(PARITY_EVIDENCE <= set(result["parityEvidenceIds"]))


if __name__ == "__main__":
    unittest.main()
