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
SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.28-011"
LOCATE_ATTRIBUTE_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.28-007"

LOCATE_EXISTING_OPERATION_DECLARATIONS = {
    "rust-observed-b11bbc68601ea288d5de",
    "rust-observed-c05d72bdbbb49325b3a9",
    "rust-observed-b4671df45a077a12336e",
    "rust-observed-7b06e44301523f96da88",
    "rust-observed-e64d93dbb39c85df4a83",
    "rust-observed-cfe296342a701e9d4e86",
    "rust-observed-8024779493eb6adf2f22",
    "rust-observed-10b65e56acbd55bdb961",
    "rust-observed-6769324f3e3f6bf80c00",
    "rust-observed-a5471ced86f7164975a8",
    "rust-observed-61509db45d5bbd9c64d9",
    "rust-observed-75c78b92458f67a2d3d3",
    "rust-observed-91a9559b313a7752bf47",
    "rust-observed-d9c65fa5caa0d0edfecc",
    "rust-observed-3af9b2ecd05ddeeb0b4e",
    "rust-observed-efcf6361220877783e8c",
    "rust-observed-dc477ae5a2b269c63724",
    "rust-observed-bec15365e44c2fb79fc9",
    "rust-observed-3057f3cd6f25b9d165e9",
    "rust-observed-e86907f9bc4668abb4f3",
    "rust-observed-d8e26ff8ff859475a2cd",
    "rust-observed-a4937291b5b6e595e042",
    "rust-observed-f81808570c0e448ffebc",
    "rust-observed-b5918d4a07217a75f879",
    "rust-observed-a27e6fe2eb857da91200",
    "rust-observed-ddea2a06a9c5a084209d",
    "rust-observed-40b23b68dd8b437e73db",
    "T038:locate:c:request:01",
    "T038:locate:c:response:02",
    "T038:locate:c:unique_identifiers:03",
    "T038:locate:java:request:01",
    "T038:locate:java:response:02",
    "T038:locate:java:type:03",
    "T038:locate:java:located_items:04",
    "T038:locate:java:unique_identifiers:05",
    "T038:locate:python:request:01",
    "T038:locate:python:response:02",
    "T038:locate:python:type:03",
    "T038:locate:python:located_items:04",
    "T038:locate:python:unique_identifiers:05",
}

# Exact approved facade aliases from the public protocol namespace.
LOCATE_PROTOCOL_ALIASES = {
    "rust-observed-0ed7f834e1e763c2bd24": "kmipkit::protocol::LocateRequest::new",
    "rust-observed-1fb2f0ef8c85083f7aa2": "kmipkit::protocol::LocateRequest::with_object_group_member",
    "rust-observed-2d35ba3aa643b1999e00": "kmipkit::protocol::LocateRequest::offset_items",
    "rust-observed-49d03cfe024d950280d2": "kmipkit::protocol::LocateResponse::try_from_response_item",
    "rust-observed-4b4b00d4c5e187b32c38": "kmipkit::protocol::ObjectGroupMember::from_raw",
    "rust-observed-5343a4e9de6bebc5507b": "kmipkit::protocol::StorageStatusMask::raw",
    "rust-observed-61a01d54f9ea83b9ae99": "kmipkit::protocol::LocateResponse",
    "rust-observed-6aebffd1646efc60f126": "kmipkit::protocol::StorageStatusMask",
    "rust-observed-756de058d68ae474dfba": "kmipkit::protocol::LocateRequest::object_group_member",
    "rust-observed-7af3e273e3fd323acf39": "kmipkit::protocol::LocateRequest::attributes",
    "rust-observed-7cb50a47a88a215f19f2": "kmipkit::protocol::LocateResponse::unique_identifiers",
    "rust-observed-7e54c3deafa02329aaaa": "kmipkit::protocol::LocateResponse::result",
    "rust-observed-823a64ae9538c5523f9d": "kmipkit::protocol::StorageStatusMask::from_raw",
    "rust-observed-9bb50225ddd5f1e28052": "kmipkit::protocol::LocateRequest::to_ttlv_payload",
    "rust-observed-a723201e26375e01d907": "kmipkit::protocol::LocateRequest::with_maximum_items",
    "rust-observed-a9946cf80e765b5b504e": "kmipkit::protocol::LocateRequest::with_offset_items",
    "rust-observed-cc1ac744ce26b9a9814b": "kmipkit::protocol::LocateRequest",
    "rust-observed-d7c4979851ede798a073": "kmipkit::protocol::LocateRequest::maximum_items",
    "rust-observed-d8189aa94d1d4a30f11e": "kmipkit::protocol::ObjectGroupMember::raw",
    "rust-observed-dc67a63a9f318f844a4c": "kmipkit::protocol::LocateRequest::storage_status_mask",
    "rust-observed-ddecc4abd5246e6facfd": "kmipkit::protocol::LocateRequest::with_storage_status_mask",
    "rust-observed-f09e32ad434e77d86642": "kmipkit::protocol::ObjectGroupMember",
    "rust-observed-fabbb8c6a23f584c3668": "kmipkit::protocol::LocateResponse::located_items",
}

LOCATE_ATTRIBUTE_REQUEST_DECLARATIONS = {
    "rust-observed-e64d93dbb39c85df4a83",
    "rust-observed-cfe296342a701e9d4e86",
    "rust-observed-8024779493eb6adf2f22",
    "rust-observed-75c78b92458f67a2d3d3",
    "rust-observed-91a9559b313a7752bf47",
    "rust-observed-d9c65fa5caa0d0edfecc",
    "rust-observed-efcf6361220877783e8c",
    "T038:locate:c:request:01",
    "T038:locate:java:request:01",
    "T038:locate:python:request:01",
    "rust-observed-e07fce825be9d33278e1",
    "rust-observed-8fff50bdd49a09ae00bf",
    "rust-observed-aa9a474fd92d1503b86d",
    "rust-observed-8d60104e0b52b30f3496",
    "rust-observed-feb2e57f6cbe856a8fa1",
    "rust-observed-f439a46d07fb7b09a304",
    "rust-observed-71e45ec7ed189cbf589f",
    "rust-observed-c545de4134a8f62c3871",
    "rust-observed-c0872e13783c072c07fb",
    "rust-observed-6eb74f84ebb61fd70253",
    "rust-observed-843c51bfe3611b61603b",
}

LOCATE_FUNCTION_IDS = {
    "client_batch_item_locate",
    "client_batch_item_response_locate",
    "client_batch_item_response_locate_unique_identifier_at",
}

# The KMIP v2.1 §6.1.28-011 server-only clause is not attached to client API rows.
LOCATE_DECLARATIONS_WITH_MISATTRIBUTED_SERVER_REQUIREMENT: set[str] = set()

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
    (
        "rust-observed-f439a46d07fb7b09a304",
        "kmipkit_protocol::LocateRequest::attributes",
    ),
    (
        "rust-observed-71e45ec7ed189cbf589f",
        "kmipkit_protocol::LocateRequest::maximum_items",
    ),
    (
        "rust-observed-c545de4134a8f62c3871",
        "kmipkit_protocol::LocateRequest::object_group_member",
    ),
    (
        "rust-observed-c0872e13783c072c07fb",
        "kmipkit_protocol::LocateRequest::offset_items",
    ),
    (
        "rust-observed-6eb74f84ebb61fd70253",
        "kmipkit_protocol::LocateRequest::storage_status_mask",
    ),
    (
        "rust-observed-843c51bfe3611b61603b",
        "kmipkit_protocol::LocateRequest::to_ttlv_payload",
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
        cls.functions = {row["id"]: row for row in manifest["functions"]}

    def test_locate_client_declarations_exclude_server_only_size_requirement(self) -> None:
        operation = self.operations[OPERATION_ID]
        locate_ids = {
            declaration_id
            for declarations in operation["declarations"].values()
            for declaration_id in declarations
        }
        self.assertEqual(74, len(locate_ids))
        self.assertEqual(
            LOCATE_EXISTING_OPERATION_DECLARATIONS | set(LOCATE_PROTOCOL_ALIASES) | {
                declaration_id for declaration_id, _ in EXPECTED
            },
            locate_ids,
        )
        self.assertEqual(23, len(LOCATE_PROTOCOL_ALIASES))
        for declaration_id, export_name in LOCATE_PROTOCOL_ALIASES.items():
            with self.subTest(protocol_alias=export_name):
                alias = self.declarations[declaration_id]
                self.assertEqual("rust-kmipkit", alias["packageId"])
                self.assertEqual(export_name, alias["exportName"])
                canonical_name = export_name.replace("kmipkit::protocol::", "kmipkit_protocol::", 1)
                canonical = next(
                    row for row in self.declarations.values()
                    if row["packageId"] == "rust-kmipkit-protocol"
                    and row["exportName"] == canonical_name
                )
                for field in ("capabilityId", "declarationState", "requirementIds", "approvalSource"):
                    self.assertEqual(canonical[field], alias[field])
        self.assertIn(SERVER_ONLY_REQUIREMENT, json.loads(MANIFEST.read_text(encoding="utf-8"))["requirementIds"])
        self.assertFalse(
            any(
                SERVER_ONLY_REQUIREMENT in self.declarations[declaration_id]["requirementIds"]
                for declaration_id in locate_ids
            )
        )
        self.assertTrue(LOCATE_FUNCTION_IDS <= set(self.functions))
        self.assertFalse(
            any(
                SERVER_ONLY_REQUIREMENT in self.functions[function_id]["requirementIds"]
                for function_id in LOCATE_FUNCTION_IDS
            )
        )

    def test_only_locate_attribute_request_surfaces_claim_client_requirement(self) -> None:
        operation = self.operations[OPERATION_ID]
        locate_ids = {
            declaration_id
            for declarations in operation["declarations"].values()
            for declaration_id in declarations
        }
        self.assertEqual(
            LOCATE_ATTRIBUTE_REQUEST_DECLARATIONS | {
                declaration_id
                for declaration_id, name in LOCATE_PROTOCOL_ALIASES.items()
                if name.startswith("kmipkit::protocol::LocateRequest")
            },
            {
                declaration_id
                for declaration_id in locate_ids
                if LOCATE_ATTRIBUTE_REQUIREMENT
                in self.declarations[declaration_id]["requirementIds"]
            },
        )
        self.assertIn(
            LOCATE_ATTRIBUTE_REQUIREMENT,
            self.functions["client_batch_item_locate"]["requirementIds"],
        )
        self.assertNotIn(
            LOCATE_ATTRIBUTE_REQUIREMENT,
            self.functions["client_batch_item_response_locate"]["requirementIds"],
        )
        self.assertNotIn(
            LOCATE_ATTRIBUTE_REQUIREMENT,
            self.functions["client_batch_item_response_locate_unique_identifier_at"]["requirementIds"],
        )

    def test_exact_locate_request_constructors_map_to_existing_facades(self) -> None:
        operation = self.operations[OPERATION_ID]
        declarations = operation["declarations"]["rust"]
        expected_requirements = OWNER_REQUIREMENTS | API_REQUIREMENTS
        self.assertEqual(11, len(EXPECTED))
        self.assertEqual(11, len({declaration_id for declaration_id, _ in EXPECTED}))

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
