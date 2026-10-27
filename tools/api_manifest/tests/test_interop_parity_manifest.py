"""Manifest regressions for the T051 Interop mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-INTEROP"
OWNER_SPEC = "specs/028-diagnostics-interop/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::Interop",
        "kmipkit_client::ClientBatchOutcome::Interop",
        "kmipkit_protocol::InteropRequest",
        "kmipkit_protocol::InteropRequest::new",
        "kmipkit_protocol::InteropResponse",
        "kmipkit_protocol::InteropResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_interop",
        "kmipkit_client_batch_item_response_interop_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.interop",
        "org.kmipkit.extensions.ClientBatchItemResult.interopResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.InteropResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.InteropResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.InteropResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_interop",
        "kmipkit.client.BatchItemResult.interop_response",
        "kmipkit.client.InteropResponse",
        "kmipkit.client.InteropResponse.response_ttlv",
        "kmipkit.client.InteropResponse.close",
    },
}
REQUIREMENTS = {
    "KMIPKIT-0042-FR-001",
    "KMIPKIT-0042-FR-002",
    "KMIPKIT-0042-FR-003",
    "KMIPKIT-0042-FR-004",
    "KMIPKIT-0042-FR-005",
    "KMIPKIT-0042-FR-006",
    "KMIPKIT-0042-FR-008",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-012",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-015",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
API_REQUIREMENTS = {value for value in REQUIREMENTS if value.startswith("KMIPKIT-0043-")}
TRACEABILITY_IDS = {
    "KMIPKIT-028-T241-FUNCTION",
    "KMIPKIT-028-T241-IDENTIFIER",
    "KMIPKIT-028-T242-EMPTY",
    "KMIPKIT-028-T243-ERROR",
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/interop_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/interop_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/interop_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/InteropJavaFacadeTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/InteropResponseMappingTest.java",
    "bindings/python/tests/test_interop_operation.py",
}


class InteropParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_interop_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Interop", operation["name"])
        self.assertEqual("6.1.26", operation["section"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0042", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0042", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        actual_mappings, linked_requirements = assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
        )
        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertEqual(REQUIREMENTS, linked_requirements)
        self.assertFalse(any(value.startswith("KMIPKIT-0037-") for value in linked_requirements))
        self.assertEqual("type", self.declarations["T051:result:type:java"]["kind"])
        self.assertEqual("type", self.declarations["T051:result:type:python"]["kind"])
        self.assertEqual("field", self.declarations["T051:result:responseTtlv:python"]["kind"])
        self.assertEqual("field", self.declarations["T051:result:operation:python"]["kind"])
        exclusions = {item["exportName"] for item in self.manifest["approvedExclusions"]}
        self.assertTrue({
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemInterop",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseInterop",
            "org.kmipkit.extensions.ClientBatchItemResult.InteropResponse.toString",
            "kmipkit.client.InteropResponse.__repr__",
        } <= exclusions)
        traceability = (ROOT / "specs/028-diagnostics-interop/traceability.md").read_text(
            encoding="utf-8"
        )
        self.assertTrue(TRACEABILITY_IDS <= set(
            token for token in traceability.replace("|", " ").split() if token.startswith("KMIPKIT-028-")
        ))
        mapping_traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        t051_row = next(line for line in mapping_traceability.splitlines() if line.startswith("| T051 |"))
        self.assertTrue(API_REQUIREMENTS <= set(re.findall(r"KMIPKIT-0043-FR-\d{3}", t051_row)))

    def test_generated_parity_inventory_contains_verified_interop_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
