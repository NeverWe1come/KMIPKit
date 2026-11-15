"""Require a four-language KMIPKIT-0016 Get Attribute List mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST"
OWNER_APPROVAL_EVIDENCE = "specs/016-attribute-operations/traceability.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::get_attribute_list",
        "kmipkit_client::ClientRequest::get_attribute_list",
        "kmipkit_client::ClientBatchOutcome::GetAttributeList",
        "kmipkit_protocol::GetAttributeListRequest",
        "kmipkit_protocol::GetAttributeListResponse",
        "kmipkit_protocol::GetAttributeListResponse::unique_identifier",
        "kmipkit_protocol::GetAttributeListResponse::attribute_references",
        "kmipkit_protocol::GetAttributeListResponse::to_ttlv_payload",
        "kmipkit_protocol::AttributeReference::tag",
        "kmipkit_protocol::AttributeReference::name",
        "kmipkit_protocol::AttributeReference::tag_value",
        "kmipkit_protocol::AttributeReference::name_parts",
    },
    "c": {
        "kmipkit_client_batch_item_get_attribute_list",
        "kmipkit_client_batch_item_response_get_attribute_list",
        "kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv",
        "kmipkit_attribute_reference_v1_t",
        "kmipkit_attribute_reference_v1_t.tag_value",
        "kmipkit_attribute_reference_v1_t.vendor_identification_data",
        "kmipkit_attribute_reference_v1_t.attribute_name_data",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.getAttributeList",
        "org.kmipkit.extensions.ClientBatchItemResult.getAttributeListResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetAttributeListResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.GetAttributeListResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.GetAttributeListResponse.attributeReferences",
        "org.kmipkit.extensions.ClientBatchItemResult.GetAttributeListResponse.responsePayloadTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.GetAttributeListResponse.close",
        "org.kmipkit.extensions.AttributeReference",
        "org.kmipkit.extensions.AttributeReference.attributeName",
        "org.kmipkit.extensions.AttributeReference.equals",
        "org.kmipkit.extensions.AttributeReference.hashCode",
        "org.kmipkit.extensions.AttributeReference.name",
        "org.kmipkit.extensions.AttributeReference.tag",
        "org.kmipkit.extensions.AttributeReference.tagValue",
        "org.kmipkit.extensions.AttributeReference.toString",
        "org.kmipkit.extensions.AttributeReference.vendorIdentification",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_get_attribute_list",
        "kmipkit.client.BatchItemResult.get_attribute_list_response",
        "kmipkit.client.GetAttributeListResponse",
        "kmipkit.client.GetAttributeListResponse.unique_identifier",
        "kmipkit.client.GetAttributeListResponse.attribute_references",
        "kmipkit.client.GetAttributeListResponse.response_payload_ttlv",
        "kmipkit.client.GetAttributeListResponse.close",
        "kmipkit.extensions.AttributeReference",
        "kmipkit.extensions.AttributeReference.__dataclass_fields__",
        "kmipkit.extensions.AttributeReference.__dataclass_params__",
        "kmipkit.extensions.AttributeReference.__delattr__",
        "kmipkit.extensions.AttributeReference.__eq__",
        "kmipkit.extensions.AttributeReference.__hash__",
        "kmipkit.extensions.AttributeReference.__init__",
        "kmipkit.extensions.AttributeReference.__match_args__",
        "kmipkit.extensions.AttributeReference.__repr__",
        "kmipkit.extensions.AttributeReference.__setattr__",
        "kmipkit.extensions.AttributeReference.__slots__",
        "kmipkit.extensions.AttributeReference.attribute_name",
        "kmipkit.extensions.AttributeReference.name",
        "kmipkit.extensions.AttributeReference.tag",
        "kmipkit.extensions.AttributeReference.tag_value",
        "kmipkit.extensions.AttributeReference.vendor_identification",
    },
}
PARITY_EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs",
    "crates/kmipkit-protocol/src/attribute_reference.rs",
    "crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/get_attribute_list_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/GetAttributeListJavaFacadeTest.java",
    "bindings/python/tests/test_get_attribute_list_operation.py",
    "tools/api_manifest/tests/test_get_attribute_list_parity_manifest.py",
}
REQUIREMENTS = {
    "KMIPKIT-0016-FR-007",
    "KMIPKIT-0016-FR-010",
    "KMIPKIT-0016-FR-011",
    "KMIPKIT-0016-FR-012",
    *(f"KMIPKIT-0043-FR-{number:03}" for number in range(4, 14)),
    "KMIPKIT-0043-FR-015",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}


class GetAttributeListParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_get_attribute_list_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Get Attribute List", operation["name"])
        self.assertEqual("6.1.21", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0016", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual(
            OWNER_APPROVAL_EVIDENCE, operation["approvalEvidence"]["path"]
        )
        self.assertIn(
            "0fa129e1248e735343704fb662926a401585ed7c",
            operation["approvalEvidence"]["evidence"],
        )
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        java_constructor = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"]
            == "org.kmipkit.extensions.ClientBatchItem.getAttributeList"
        )
        self.assertEqual(
            [
                "public static org.kmipkit.extensions.ClientBatchItem getAttributeList()",
                "public static org.kmipkit.extensions.ClientBatchItem getAttributeList(org.kmipkit.extensions.UniqueIdentifier)",
            ],
            java_constructor["javaSignatures"],
        )

        assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_APPROVAL_EVIDENCE,
        )


if __name__ == "__main__":
    unittest.main()
