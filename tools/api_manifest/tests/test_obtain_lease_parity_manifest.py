"""Require the approved KMIP v2.1 §6.1.35 Obtain Lease mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
REQUIREMENTS = {
    *(f"KMIPKIT-0036-FR-{index:03}" for index in range(1, 9)),
    *(f"KMIPKIT-0043-FR-{index:03}" for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)),
    "KMIPKIT-REQ-SPEC-6.1.35-001-001",
    "KMIPKIT-REQ-SPEC-6.1.35-001-002",
    "KMIPKIT-REQ-SPEC-6.1.35-002",
    "KMIPKIT-REQ-SPEC-6.1.35-005",
}
TRACEABILITY_IDS = {
    "KMIPKIT-REQ-SPEC-6.1.35-001-001",
    "KMIPKIT-REQ-SPEC-6.1.35-001-002",
    "KMIPKIT-REQ-SPEC-6.1.35-002",
    "KMIPKIT-REQ-SPEC-6.1.35-005",
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ObtainLeaseMappingTest.java",
    "bindings/python/tests/test_obtain_lease_operation.py",
    "crates/kmipkit-client/tests/unit/obtain_lease_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs",
    "tools/api_manifest/tests/test_obtain_lease_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::obtain_lease",
        "kmipkit_client::ClientRequest::ObtainLease",
        "kmipkit_client::ClientBatchOutcome::obtain_lease_response",
        "kmipkit_protocol::ObtainLeaseRequest",
        "kmipkit_protocol::ObtainLeaseRequest::new",
        "kmipkit_protocol::ObtainLeaseRequest::with_unique_identifier",
        "kmipkit_protocol::ObtainLeaseResponse",
        "kmipkit_protocol::ObtainLeaseResponse::unique_identifier",
        "kmipkit_protocol::ObtainLeaseResponse::lease_time",
        "kmipkit_protocol::ObtainLeaseResponse::last_change_date",
        "kmipkit_protocol::ObtainLeaseResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_obtain_lease",
        "kmipkit_client_batch_item_response_obtain_lease",
        "kmipkit_client_batch_item_response_obtain_lease_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.obtainLease",
        "org.kmipkit.extensions.ClientBatchItemResult.obtainLeaseResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse.leaseTime",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse.lastChangeDate",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.ObtainLeaseResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_obtain_lease",
        "kmipkit.client.BatchItemResult.obtain_lease_response",
        "kmipkit.client.ObtainLeaseResponse",
        "kmipkit.client.ObtainLeaseResponse.unique_identifier",
        "kmipkit.client.ObtainLeaseResponse.lease_time",
        "kmipkit.client.ObtainLeaseResponse.last_change_date",
        "kmipkit.client.ObtainLeaseResponse.response_ttlv",
        "kmipkit.client.ObtainLeaseResponse.close",
    },
}


class ObtainLeaseParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_obtain_lease_has_approved_four_language_mapping_and_owner_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Obtain Lease", operation["name"])
        self.assertEqual("6.1.35", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0036", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
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

        owner_traceability = (ROOT / "specs/022-object-lifecycle/traceability.md").read_text(
            encoding="utf-8"
        )
        for trace_id in TRACEABILITY_IDS:
            self.assertIn(trace_id, owner_traceability)
        mapping_traceability = (ROOT / "specs/043-public-api-bindings/traceability.md").read_text(
            encoding="utf-8"
        )
        t056_row = next(line for line in mapping_traceability.splitlines() if line.startswith("| T056 |"))
        self.assertTrue(set(re.findall(r"KMIPKIT-0036-FR-\d{3}", t056_row)) >= {
            f"KMIPKIT-0036-FR-{index:03}" for index in range(1, 9)
        })
        self.assertIn("KMIPKIT-0043-FR-007", t056_row)

    def test_generated_parity_inventory_contains_verified_obtain_lease_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
