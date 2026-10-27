"""Require complete four-language Get and Locate mappings before release readiness."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
C_HEADER = ROOT / "bindings" / "c" / "include" / "kmipkit.h"
OWNER_SPEC = "specs/017-managed-object-retrieval/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}

OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-GET": {
        "name": "Get",
        "section": "6.1.19",
        "expected": {
            "rust": {
                "kmipkit_client::ClientRequest::Get",
                "kmipkit_client::ClientBatchOutcome::GetCompleted",
                "kmipkit_protocol::GetRequest",
                "kmipkit_protocol::GetResponse",
                "kmipkit_protocol::GetResponse::object_type",
                "kmipkit_protocol::GetResponse::unique_identifier",
                "kmipkit_protocol::GetResponse::object",
            },
            "c": {
                "kmipkit_client_batch_item_get",
                "kmipkit_client_batch_item_response_get",
            },
            "java": {
                "org.kmipkit.extensions.ClientBatchItem.get",
                "org.kmipkit.extensions.ClientBatchItemResult.getResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.GetResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.GetResponse.objectTypeRaw",
                "org.kmipkit.extensions.ClientBatchItemResult.GetResponse.uniqueIdentifier",
                "org.kmipkit.extensions.ClientBatchItemResult.GetResponse.object",
                "org.kmipkit.extensions.ClientBatchItemResult.GetResponse.close",
            },
            "python": {
                "kmipkit.extensions.client_batch_item_get",
                "kmipkit.client.BatchItemResult.get_response",
                "kmipkit.client.GetResponse",
                "kmipkit.client.GetResponse.object_type_raw",
                "kmipkit.client.GetResponse.unique_identifier",
                "kmipkit.client.GetResponse.object",
                "kmipkit.client.GetResponse.close",
            },
        },
    },
    "KMIPKIT-ELEM-OP-C2S-LOCATE": {
        "name": "Locate",
        "section": "6.1.28",
        "expected": {
            "rust": {
                "kmipkit_client::ClientRequest::Locate",
                "kmipkit_client::ClientBatchOutcome::LocateCompleted",
                "kmipkit_protocol::LocateRequest",
                "kmipkit_protocol::LocateResponse",
                "kmipkit_protocol::LocateResponse::located_items",
                "kmipkit_protocol::LocateResponse::unique_identifiers",
                "kmipkit_protocol::StorageStatusMask::from_raw",
                "kmipkit_protocol::StorageStatusMask::raw",
                "kmipkit_protocol::ObjectGroupMember::from_raw",
                "kmipkit_protocol::ObjectGroupMember::raw",
            },
            "c": {
                "kmipkit_client_batch_item_locate",
                "kmipkit_client_batch_item_response_locate",
                "kmipkit_client_batch_item_response_locate_unique_identifier_at",
            },
            "java": {
                "org.kmipkit.extensions.ClientBatchItem.locate",
                "org.kmipkit.extensions.ClientBatchItemResult.locateResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.LocateResponse",
                "org.kmipkit.extensions.ClientBatchItemResult.LocateResponse.locatedItems",
                "org.kmipkit.extensions.ClientBatchItemResult.LocateResponse.uniqueIdentifiers",
            },
            "python": {
                "kmipkit.extensions.client_batch_item_locate",
                "kmipkit.client.BatchItemResult.locate_response",
                "kmipkit.client.LocateResponse",
                "kmipkit.client.LocateResponse.located_items",
                "kmipkit.client.LocateResponse.unique_identifiers",
            },
        },
    },
}
OWNER_REQUIREMENTS = {f"KMIPKIT-0017-FR-{index:03}" for index in range(1, 14)}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 17, 18, 21, 22)
}


class GetLocateParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_get_and_locate_have_approved_four_language_mappings_and_traceability(self) -> None:
        for operation_id, expected in OPERATIONS.items():
            with self.subTest(operation=operation_id):
                operations = [
                    row for row in self.manifest["operations"]
                    if row["operationId"] == operation_id
                ]
                self.assertEqual(1, len(operations))
                operation = operations[0]
                self.assertEqual(expected["name"], operation["name"])
                self.assertEqual(expected["section"], operation["section"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0017", operation["featureSpecId"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
                self.assertEqual("Approved", operation["approvalEvidence"]["status"])
                self.assertNotIn("blockedReason", operation)
                self.assertEqual(LANGUAGES, set(operation["declarations"]))
                self.assertTrue(operation["parityEvidenceIds"])

                assert_declaration_mappings(
                    self,
                    operation,
                    self.declarations,
                    operation_id=operation_id,
                    languages=LANGUAGES,
                    expected_mappings=expected["expected"],
                    required_requirements=OWNER_REQUIREMENTS | API_REQUIREMENTS,
                    expected_approval_source=OWNER_SPEC,
                )

    def test_generated_parity_inventory_contains_verified_get_and_locate(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = {row["operationId"]: row for row in generated["operations"]}
        for operation_id in OPERATIONS:
            with self.subTest(operation=operation_id):
                self.assertEqual("verified", operations[operation_id]["apiState"])
                self.assertEqual(LANGUAGES, set(operations[operation_id]["declarations"]))

    def test_generated_c_header_exposes_get_and_locate_symbols(self) -> None:
        header = C_HEADER.read_text(encoding="utf-8")
        for operation in OPERATIONS.values():
            for symbol in operation["expected"]["c"]:
                with self.subTest(symbol=symbol):
                    self.assertTrue(f"{symbol}(" in header, symbol)


if __name__ == "__main__":
    unittest.main()
