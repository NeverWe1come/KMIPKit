"""Map only RNG response error aliases through the shared client outcome."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
MAPPING_SEMANTICS = "normalized-client-protocol-error"
OWNER_REQUIREMENTS = {
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
ERROR_EXPORTS = {
    ("rust-kmipkit", "kmipkit::RngRetrieveError"): "kmipkit_protocol::RngRetrieveResponse",
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngRetrieveError"): "kmipkit_protocol::RngRetrieveResponse",
    ("rust-kmipkit", "kmipkit::RngSeedError"): "kmipkit_protocol::RngSeedResponse",
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngSeedError"): "kmipkit_protocol::RngSeedResponse",
}
EXPECTED_MAPPING = {
    "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::ClientError::Protocol"},
    "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_PROTOCOL"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ProtocolException"},
    "python": {"packageId": "python-kmipkit", "name": "kmipkit.errors.ClientExecutionError"},
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs",
    "crates/kmipkit-client/tests/error_contract.rs",
    "crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeContractTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/RngOperationMappingTest.java",
    "bindings/python/tests/test_client.py",
    "bindings/python/tests/test_rng_operation.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "tools/api_manifest/tests/test_t102_rng_error_mapping.py",
}


class T102RngErrorMappingTests(unittest.TestCase):
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

    def test_rng_error_aliases_map_to_the_shared_execution_protocol_error(self) -> None:
        self.assertEqual(len(ERROR_EXPORTS), 4)
        for identity, response_name in ERROR_EXPORTS.items():
            with self.subTest(identity=identity):
                row = self.declarations[identity]
                response = self.declarations[("rust-kmipkit-protocol", response_name)]
                requirements = set(row["requirementIds"])
                self.assertEqual(row["declarationState"], "declared")
                self.assertEqual(row["capabilityId"], "client_execution")
                self.assertEqual(row["parityScope"], "four-language")
                self.assertEqual(row["mappingSemantics"], MAPPING_SEMANTICS)
                self.assertEqual(row["languageMappings"], EXPECTED_MAPPING)
                self.assertEqual(row["approvalSource"], self.client_execution["approvalEvidence"])
                self.assertTrue(OWNER_REQUIREMENTS <= requirements)
                self.assertTrue(SHARED_REQUIREMENTS <= requirements)
                self.assertTrue(
                    {item for item in response["requirementIds"] if item.startswith("KMIPKIT-REQ-SPEC-")}
                    <= requirements
                )
                self.assertTrue(PARITY_EVIDENCE <= set(row["parityEvidenceIds"]))

    def test_direct_response_parsers_remain_blocked(self) -> None:
        for response_name in {response for response in ERROR_EXPORTS.values()}:
            parser = self.declarations[("rust-kmipkit-protocol", f"{response_name}::try_from_response_item")]
            self.assertEqual(parser["declarationState"], "blocked")
            self.assertEqual(parser["candidateSpecId"], "KMIPKIT-0043")

    def test_normalized_error_allowlist_rejects_rng_kind_and_missing_owner_requirement(self) -> None:
        row = copy.deepcopy(
            self.declarations[("rust-kmipkit-protocol", "kmipkit_protocol::RngRetrieveError")]
        )
        row["declarationState"] = "declared"
        row["mappingSemantics"] = MAPPING_SEMANTICS
        row["requirementIds"] = sorted(OWNER_REQUIREMENTS | SHARED_REQUIREMENTS)
        generate._validate_normalized_client_protocol_error_source(row)

        kind = copy.deepcopy(row)
        kind["exportName"] = "kmipkit_protocol::RngRetrieveError::kind"
        kind["kind"] = "associated-item"
        with self.assertRaisesRegex(generate.ManifestError, "approved response error family"):
            generate._validate_normalized_client_protocol_error_source(kind)

        missing_owner = copy.deepcopy(row)
        missing_owner["requirementIds"].remove("KMIPKIT-0041-FR-001")
        with self.assertRaisesRegex(generate.ManifestError, "response owner requirements"):
            generate._validate_normalized_client_protocol_error_source(missing_owner)

if __name__ == "__main__":
    unittest.main()
