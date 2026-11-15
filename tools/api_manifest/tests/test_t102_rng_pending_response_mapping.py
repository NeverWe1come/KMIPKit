"""Require four-language parity for the approved RNG Pending response parsers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
COMMON_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/RngOperationMappingTest.java",
    "bindings/python/tests/test_rng_native_integration.py",
    "bindings/python/tests/test_rng_operation.py",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "tools/api_manifest/tests/test_rng_parity_manifest.py",
    "tools/api_manifest/tests/test_t102_rng_pending_response_mapping.py",
}
MANIFEST_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
OPERATIONS = {
    "RngRetrieve": {
        "capability": "KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE",
        "owner_requirements": {
            "KMIPKIT-0041-FR-001",
            "KMIPKIT-0041-FR-003",
            "KMIPKIT-0041-FR-004",
            "KMIPKIT-0009-FR-002",
        },
        "c": "kmipkit_client_batch_item_response_rng_retrieve",
        "java": "org.kmipkit.extensions.ClientBatchItemResult.rngRetrieveResponse",
        "python": "kmipkit.client.BatchItemResult.rng_retrieve_response",
        "rust": "rng_retrieve_response",
        "evidence": COMMON_EVIDENCE | {
            "crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs",
            "crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs",
        },
    },
    "RngSeed": {
        "capability": "KMIPKIT-ELEM-OP-C2S-RNG-SEED",
        "owner_requirements": {
            "KMIPKIT-0041-FR-001",
            "KMIPKIT-0041-FR-003",
            "KMIPKIT-0041-FR-004",
            "KMIPKIT-0009-FR-002",
        },
        "c": "kmipkit_client_batch_item_response_rng_seed",
        "java": "org.kmipkit.extensions.ClientBatchItemResult.rngSeedResponse",
        "python": "kmipkit.client.BatchItemResult.rng_seed_response",
        "rust": "rng_seed_response",
        "evidence": COMMON_EVIDENCE | {
            "crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs",
            "crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs",
        },
    },
}


def _rust_pending_parsers(response: str) -> tuple[tuple[str, str], ...]:
    return (
        ("rust-kmipkit", f"kmipkit::{response}Response::try_from_pending_response_item"),
        (
            "rust-kmipkit-protocol",
            f"kmipkit_protocol::{response}Response::try_from_pending_response_item",
        ),
    )


class T102RngPendingResponseMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        cls.operations = {row["operationId"]: row for row in manifest["operations"]}

    def test_pending_response_parsers_map_to_the_four_language_rng_facade(self) -> None:
        for response, expected in OPERATIONS.items():
            operation = self.operations[expected["capability"]]
            self.assertEqual("approved", operation["ownerDependency"]["state"])
            self.assertEqual("specs/027-rng-operations/spec.md", operation["approvalEvidence"]["path"])

            for package, rust_path in _rust_pending_parsers(response):
                with self.subTest(response=response, package=package):
                    declaration = self.declarations[(package, rust_path)]
                    self.assertEqual("verified", declaration["declarationState"])
                    self.assertEqual(expected["capability"], declaration["capabilityId"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertEqual("specs/027-rng-operations/spec.md", declaration["approvalSource"]["path"])
                    self.assertEqual("Approved", declaration["approvalSource"]["status"])
                    self.assertTrue(
                        expected["owner_requirements"] | MANIFEST_REQUIREMENTS
                        <= set(declaration["requirementIds"])
                    )
                    mappings = declaration["languageMappings"]
                    self.assertEqual(package, mappings["rust"]["packageId"])
                    self.assertEqual(rust_path, mappings["rust"]["name"])
                    self.assertEqual(("c-kmipkit", expected["c"]), (
                        mappings["c"]["packageId"], mappings["c"]["name"]
                    ))
                    self.assertEqual(("java-kmipkit", expected["java"]), (
                        mappings["java"]["packageId"], mappings["java"]["name"]
                    ))
                    self.assertEqual(("python-kmipkit", expected["python"]), (
                        mappings["python"]["packageId"], mappings["python"]["name"]
                    ))
                    self.assertTrue(expected["evidence"] <= set(declaration["parityEvidenceIds"]))
                    for path in declaration["parityEvidenceIds"]:
                        self.assertTrue((ROOT / path).is_file(), path)

            rust_ids = set(operation["declarations"]["rust"])
            for package, rust_path in _rust_pending_parsers(response):
                self.assertIn(self.declarations[(package, rust_path)]["declarationId"], rust_ids)


if __name__ == "__main__":
    unittest.main()
