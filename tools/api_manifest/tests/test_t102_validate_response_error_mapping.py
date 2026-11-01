"""Require four-language mapping for Validate response parsing errors."""

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
REQUIREMENTS = {
    *(f"KMIPKIT-0042-FR-{index:03}" for index in range(1, 9)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)),
    "KMIPKIT-REQ-SPEC-6.1.57-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002",
    "KMIPKIT-REQ-SPEC-6.1.57-004",
    "KMIPKIT-REQ-SPEC-6.1.57-005",
    "KMIPKIT-REQ-SPEC-6.1.57-006",
}
ERROR_EXPORTS = {
    "kmipkit_protocol::ValidateError",
    "kmipkit_protocol::ValidateError::InvalidOperationResult",
    "kmipkit_protocol::ValidateError::InvalidOperationResult::0",
    "kmipkit_protocol::ValidateError::MalformedResponseItem",
    "kmipkit_protocol::ValidateError::MalformedResponsePayload",
    "kmipkit_protocol::ValidateError::MissingResponsePayload",
    "kmipkit_protocol::ValidateError::MissingResultStatus",
    "kmipkit_protocol::ValidateError::UnexpectedOperation",
}
PARSER_EXPORT = "kmipkit_protocol::ValidateResponse::try_from_response_item"
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/validate_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/validate_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ValidateJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/ValidateResponseMappingTest.java",
    "bindings/python/tests/test_validate_operation.py",
    "tools/api_manifest/tests/test_validate_parity_manifest.py",
    "tools/api_manifest/tests/test_t102_validate_response_error_mapping.py",
}
EXPECTED_MAPPINGS = {
    "rust": ERROR_EXPORTS | {PARSER_EXPORT},
    "c": {"kmipkit_client_batch_item_response_validate"},
    "java": {
        "org.kmipkit.extensions.ClientBatchItemResult.validateResponse",
    },
    "python": {
        "kmipkit.client.BatchItemResult.validate_response",
    },
}
ERROR_MAPPINGS = {
    "c": "KMIPKIT_ERROR_PROTOCOL",
    "java": "org.kmipkit.ProtocolException",
    "python": "kmipkit.errors.ClientExecutionError",
}
PARSER_MAPPINGS = {
    "c": "kmipkit_client_batch_item_response_validate",
    "java": "org.kmipkit.extensions.ClientBatchItemResult.validateResponse",
    "python": "kmipkit.client.BatchItemResult.validate_response",
}


class ValidateResponseErrorMappingTests(unittest.TestCase):
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

    def test_validate_parser_and_error_taxonomy_map_to_existing_facades(self) -> None:
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

        exports = ERROR_EXPORTS | {PARSER_EXPORT}
        mapped_exports = {
            declaration["exportName"]
            for declaration_id in self.operation["declarations"]["rust"]
            if (declaration := self.declarations[declaration_id])
        }
        self.assertTrue(exports <= mapped_exports)
        for declaration in self.manifest["publicDeclarations"]:
            if declaration["packageId"] == "rust-kmipkit-protocol" and declaration["exportName"] in exports:
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertTrue(
                    PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]),
                    declaration["exportName"],
                )
                expected = PARSER_MAPPINGS if declaration["exportName"] == PARSER_EXPORT else ERROR_MAPPINGS
                actual = {
                    language: declaration["languageMappings"][language]["name"]
                    for language in ("c", "java", "python")
                }
                self.assertEqual(expected, actual, declaration["exportName"])


if __name__ == "__main__":
    unittest.main()
