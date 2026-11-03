"""Pin LocateRequest builder parity to the approved Locate operation contract."""

from __future__ import annotations

import json
import re
import unittest
from html import unescape
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OASIS = ROOT / "specification" / "oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
OWNER_SPEC = "specs/017-managed-object-retrieval/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-LOCATE"
CONTRACT_EVIDENCE = (
    "tools/api_manifest/tests/test_t102_locate_request_builder_mapping.py"
)

OWNER_REQUIREMENTS = {
    *(f"KMIPKIT-0017-FR-{number:03}" for number in (1, 2, 6, 7, 9, 13)),
    "KMIPKIT-REQ-SPEC-6.1.28-001",
    "KMIPKIT-REQ-SPEC-6.1.28-002",
    "KMIPKIT-REQ-SPEC-6.1.28-007",
    "KMIPKIT-REQ-SPEC-6.1.28-013-001",
    "KMIPKIT-REQ-SPEC-6.1.28-013-002",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{number:03}"
    for number in (4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 17, 18, 21, 22)
}

EXPECTED = (
    (
        "rust-observed-e07fce825be9d33278e1",
        "kmipkit_protocol::LocateRequest::new",
    ),
    (
        "rust-observed-8fff50bdd49a09ae00bf",
        "kmipkit_protocol::LocateRequest::with_maximum_items",
    ),
    (
        "rust-observed-aa9a474fd92d1503b86d",
        "kmipkit_protocol::LocateRequest::with_offset_items",
    ),
    (
        "rust-observed-8d60104e0b52b30f3496",
        "kmipkit_protocol::LocateRequest::with_storage_status_mask",
    ),
    (
        "rust-observed-feb2e57f6cbe856a8fa1",
        "kmipkit_protocol::LocateRequest::with_object_group_member",
    ),
)

FOREIGN_MAPPINGS = {
    "c": {
        "packageId": "c-kmipkit",
        "name": "kmipkit_client_batch_item_locate",
    },
    "java": {
        "packageId": "java-kmipkit",
        "name": "org.kmipkit.extensions.ClientBatchItem.locate",
    },
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.extensions.client_batch_item_locate",
    },
}


class T102LocateRequestBuilderMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in manifest["publicDeclarations"]
        }
        cls.operations = {row["operationId"]: row for row in manifest["operations"]}

    def test_exact_locate_request_constructors_map_to_existing_facades(self) -> None:
        operation = self.operations[OPERATION_ID]
        declarations = operation["declarations"]["rust"]
        expected_requirements = OWNER_REQUIREMENTS | API_REQUIREMENTS
        self.assertEqual(5, len(EXPECTED))
        self.assertEqual(5, len({declaration_id for declaration_id, _ in EXPECTED}))

        for declaration_id, export_name in EXPECTED:
            row = self.declarations[declaration_id]
            expected_mappings = {
                "rust": {"packageId": "rust-kmipkit-protocol", "name": export_name},
                **FOREIGN_MAPPINGS,
            }
            with self.subTest(export=export_name):
                self.assertIn(declaration_id, declarations)
                self.assertEqual("rust-kmipkit-protocol", row["packageId"])
                self.assertEqual(export_name, row["exportName"])
                self.assertEqual("associated-item", row["kind"])
                self.assertEqual("verified", row["declarationState"])
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(OPERATION_ID, row["capabilityId"])
                self.assertEqual(expected_requirements, set(row["requirementIds"]))
                self.assertEqual(OWNER_SPEC, row["approvalSource"]["path"])
                self.assertEqual("Approved", row["approvalSource"]["status"])
                self.assertEqual(expected_mappings, row["languageMappings"])
                self.assertIn(CONTRACT_EVIDENCE, row["parityEvidenceIds"])

    def test_table_247_fields_and_order_match_the_pinned_oasis_source(self) -> None:
        source = OASIS.read_text(encoding="cp1252")
        locate = source.split('name="_Toc57115660">6.1.28 Locate', 1)[1]
        table = locate.split("<div align=center>", 1)[1].split(
            'name="_Toc32239041">Table </a>247: Locate Request Payload', 1
        )[0]
        text = unescape(re.sub(r"<[^>]+>", " ", table))
        text = re.sub(r"\s+", " ", text)
        fields = (
            "Maximum Items",
            "Offset Items",
            "Storage Status Mask",
            "Object Group Member",
            "Attributes",
        )
        positions = [text.index(field) for field in fields]

        self.assertEqual(sorted(positions), positions)
        self.assertIn("An Integer object that indicates the maximum number", text)
        self.assertIn("An Integer object that indicates the number of object identifiers", text)
        self.assertIn("Integer object (used as a bit mask)", text)
        self.assertIn("An Enumeration object that indicates the object group member type", text)
        self.assertIn("Attributes structure MAY be empty", text)

    def test_java_locate_forwards_each_value_and_presence_flag_in_order(self) -> None:
        source = (
            ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java"
        ).read_text(encoding="utf-8")
        method = source.split("public static ClientBatchItem locate(", 1)[1].split(
            "/** Creates the empty Get Constraints", 1
        )[0]
        forwarding = re.compile(
            r"NativeExtensionRegistry\.clientBatchItemLocate\(\s*"
            r"maximumItems != null,\s*maximumItems == null \? 0 : maximumItems,\s*"
            r"offsetItems != null,\s*offsetItems == null \? 0 : offsetItems,\s*"
            r"storageStatusMask != null,\s*storageStatusMask == null \? 0 : storageStatusMask,\s*"
            r"objectGroupMember != null,\s*objectGroupMember == null \? 0 : objectGroupMember,\s*"
            r"handleOrNull\(attributes\)\s*\)",
        )
        self.assertRegex(method, forwarding)

        java_test = (
            ROOT / "bindings/java/src/test/java/org/kmipkit/extensions/GetLocateResponseMappingTest.java"
        ).read_text(encoding="utf-8")
        self.assertIn("ClientBatchItem.locate(\n                        -4, 0, 0x8000_0001L, 0xf123_4567L, attributes)", java_test)
        self.assertIn("ClientBatchItem.locate(null, null, null, null, attributes)", java_test)


if __name__ == "__main__":
    unittest.main()
