"""Map the shared cryptographic parser errors to the full client outcome."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
MAPPING_SEMANTICS = "normalized-client-protocol-error-outcome"
OWNER_REQUIREMENTS = {
    "KMIPKIT-0019-FR-007",
    "KMIPKIT-0019-FR-009",
    "KMIPKIT-0021-FR-001",
    "KMIPKIT-0021-FR-010",
    "KMIPKIT-0021-FR-012",
    "KMIPKIT-0021-FR-015",
    "KMIPKIT-0041-FR-001",
    "KMIPKIT-0041-FR-003",
    "KMIPKIT-0041-FR-004",
    "KMIPKIT-0041-FR-005",
    "KMIPKIT-0041-FR-006",
}
SHARED_REQUIREMENTS = {
    "KMIPKIT-0007-FR-006",
    "KMIPKIT-0007-FR-008",
    "KMIPKIT-0007-FR-013",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
NORMATIVE_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.32-001",
    "KMIPKIT-REQ-SPEC-6.1.33-001-001",
    "KMIPKIT-REQ-SPEC-6.1.33-001-002",
    "KMIPKIT-REQ-SPEC-6.1.50-001",
    "KMIPKIT-REQ-SPEC-6.1.55-001",
    "KMIPKIT-REQ-SPEC-6.1.56-001-001",
    "KMIPKIT-REQ-SPEC-6.1.56-002",
}
ERROR_EXPORTS = {
    "kmipkit_protocol::CryptographicOperationError": "type",
    "kmipkit_protocol::CryptographicOperationErrorKind": "type",
    "kmipkit_protocol::CryptographicOperationErrorKind::UnexpectedOperation": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::MissingResultStatus": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::InvalidOperationResult": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::InvalidOperationResult::0": "field",
    "kmipkit_protocol::CryptographicOperationErrorKind::PendingOutcomeRequired": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::NotPendingOutcome": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::MissingSuccessPayload": "variant",
    "kmipkit_protocol::CryptographicOperationErrorKind::MalformedPayload": "variant",
}
EXPECTED_MAPPING = {
    "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::ClientError"},
    "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_PROTOCOL"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ProtocolException"},
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.errors.ClientExecutionError",
    },
}
DIRECT_PARSERS = {
    ("rust-kmipkit", "kmipkit::RngRetrieveResponse::try_from_response_item"),
    ("rust-kmipkit", "kmipkit::RngSeedResponse::try_from_response_item"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngRetrieveResponse::try_from_response_item"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngSeedResponse::try_from_response_item"),
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/src/execute.rs",
    "crates/kmipkit-client/tests/error_contract.rs",
    "crates/kmipkit-client/tests/unit/hash_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/mac_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/mac_verify_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/sign_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs",
    "crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "bindings/c/include/kmipkit.h",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/main/java/org/kmipkit/ProtocolException.java",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeContractTest.java",
    "bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/ClientCryptoResponseMappingCoverageTest.java",
    "bindings/python/src/kmipkit/errors.py",
    "bindings/python/tests/test_client.py",
    "bindings/python/tests/test_hash_mac_signature_operations.py",
    "bindings/python/tests/test_rng_operation.py",
    "tools/api_manifest/tests/test_t102_crypto_protocol_error_outcome_mapping.py",
}


class T102CryptoProtocolErrorOutcomeMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.client_execution = next(
            row
            for row in cls.manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

    def test_cryptographic_errors_map_to_the_complete_protocol_outcome_surface(self) -> None:
        self.assertEqual(len(ERROR_EXPORTS), 10)
        required = OWNER_REQUIREMENTS | SHARED_REQUIREMENTS | NORMATIVE_REQUIREMENTS
        for export_name, kind in ERROR_EXPORTS.items():
            with self.subTest(export=export_name):
                row = self.declarations[("rust-kmipkit-protocol", export_name)]
                self.assertEqual(row["kind"], kind)
                self.assertEqual(row["declarationState"], "declared")
                self.assertEqual(row["capabilityId"], "client_execution")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertEqual(row["mappingSemantics"], MAPPING_SEMANTICS)
                self.assertEqual(row["languageMappings"], EXPECTED_MAPPING)
                self.assertEqual(row["approvalSource"], self.client_execution["approvalEvidence"])
                self.assertTrue(required <= set(row["requirementIds"]))
                self.assertTrue(PARITY_EVIDENCE <= set(row["parityEvidenceIds"]))

    def test_kind_accessor_and_operation_specific_parsers_remain_blocked(self) -> None:
        kind = self.declarations[
            ("rust-kmipkit-protocol", "kmipkit_protocol::CryptographicOperationError::kind")
        ]
        self.assertEqual(kind["declarationState"], "blocked")
        self.assertEqual(kind["candidateSpecId"], "KMIPKIT-0043")

        for identity in DIRECT_PARSERS:
            with self.subTest(export=identity[1]):
                row = self.declarations[identity]
                self.assertEqual(row["declarationState"], "blocked")
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0043")

    def test_normalized_outcome_allowlist_rejects_kind_and_missing_owner_requirement(self) -> None:
        row = copy.deepcopy(
            self.declarations[
                ("rust-kmipkit-protocol", "kmipkit_protocol::CryptographicOperationError")
            ]
        )
        row["declarationState"] = "declared"
        row["mappingSemantics"] = MAPPING_SEMANTICS
        row["requirementIds"] = sorted(OWNER_REQUIREMENTS | SHARED_REQUIREMENTS | NORMATIVE_REQUIREMENTS)
        generate._validate_normalized_client_protocol_error_source(row)

        kind = copy.deepcopy(row)
        kind["exportName"] = "kmipkit_protocol::CryptographicOperationError::kind"
        kind["kind"] = "associated-item"
        with self.assertRaisesRegex(generate.ManifestError, "approved response error family"):
            generate._validate_normalized_client_protocol_error_source(kind)

        missing_owner = copy.deepcopy(row)
        missing_owner["requirementIds"].remove("KMIPKIT-0021-FR-015")
        with self.assertRaisesRegex(generate.ManifestError, "response owner requirements"):
            generate._validate_normalized_client_protocol_error_source(missing_owner)


if __name__ == "__main__":
    unittest.main()
