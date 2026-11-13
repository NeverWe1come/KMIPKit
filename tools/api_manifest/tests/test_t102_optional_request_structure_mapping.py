"""Pin exact optional request Structures for Re-key, Join Split Key, and Re-certify."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.test_t102_certify_request_builder_mapping import (
    OPERATION as CERTIFY_OPERATION_ID,
    OPERATION_ADDED_EVIDENCE as CERTIFY_OPERATION_ADDED_EVIDENCE,
    TARGETS as CERTIFY_TARGETS,
)
from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
    without_0047_versioned_values,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CERTIFY_TARGET_IDS = {target["id"] for target in CERTIFY_TARGETS}
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
MASK_WIRE_FIXTURE_SOURCES = (
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_rekey_native_integration.py",
    "bindings/python/tests/test_recertify_operation.py",
    "bindings/python/tests/test_join_split_key_operation.py",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/register_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/rekey_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/rekey_key_pair_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/register_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs",
)
OWNER = "specs/022-object-lifecycle/spec.md"
TARGETS = (
    {
        "id": "rust-observed-55695fec9108789be098",
        "name": "kmipkit_protocol::RekeyRequest::with_attributes",
        "operation": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "tag": 0x420125,
        "owner": OWNER,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-008"},
        "c": "kmipkit_client_batch_item_rekey",
        "java": "org.kmipkit.extensions.ClientBatchItem.rekey",
        "python": "kmipkit.extensions.client_batch_item_rekey",
        "evidence": {
            "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_rekey_native_integration.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
    {
        "id": "rust-observed-3c3be169b5ff8899e1c7",
        "name": "kmipkit_protocol::RekeyRequest::with_protection_storage_masks",
        "operation": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "tag": 0x42015F,
        "owner": OWNER,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-008"},
        "c": "kmipkit_client_batch_item_rekey",
        "java": "org.kmipkit.extensions.ClientBatchItem.rekey",
        "python": "kmipkit.extensions.client_batch_item_rekey",
        "evidence": {
            "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_rekey_native_integration.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
    {
        "id": "rust-observed-f894598b6998d1b416b1",
        "name": "kmipkit_protocol::JoinSplitKeyRequest::with_attributes",
        "operation": "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        "tag": 0x420125,
        "owner": "specs/023-key-material-transfer/spec.md",
        "fr": {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004", "KMIPKIT-0037-FR-006", "KMIPKIT-0037-FR-008"},
        "c": "kmipkit_client_batch_item_join_split_key",
        "java": "org.kmipkit.extensions.ClientBatchItem.joinSplitKey",
        "python": "kmipkit.extensions.client_batch_item_join_split_key",
        "evidence": {
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_join_split_key_operation.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
    {
        "id": "rust-observed-236134f1fc8e195442a0",
        "name": "kmipkit_protocol::JoinSplitKeyRequest::with_protection_storage_masks",
        "operation": "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        "tag": 0x42015F,
        "owner": "specs/023-key-material-transfer/spec.md",
        "fr": {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004", "KMIPKIT-0037-FR-006", "KMIPKIT-0037-FR-008"},
        "c": "kmipkit_client_batch_item_join_split_key",
        "java": "org.kmipkit.extensions.ClientBatchItem.joinSplitKey",
        "python": "kmipkit.extensions.client_batch_item_join_split_key",
        "evidence": {
            "crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs",
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_join_split_key_operation.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
    {
        "id": "rust-observed-0cad07b4f1010a7d9a55",
        "name": "kmipkit_protocol::RecertifyRequest::with_attributes",
        "operation": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        "tag": 0x420125,
        "owner": OWNER,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "c": "kmipkit_client_batch_item_recertify",
        "java": "org.kmipkit.extensions.ClientBatchItem.recertify",
        "python": "kmipkit.extensions.client_batch_item_recertify",
        "evidence": {
            "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_recertify_operation.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
    {
        "id": "rust-observed-6e9fb656f103c43fa2de",
        "name": "kmipkit_protocol::RecertifyRequest::with_protection_storage_masks",
        "operation": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        "tag": 0x42015F,
        "owner": OWNER,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "c": "kmipkit_client_batch_item_recertify",
        "java": "org.kmipkit.extensions.ClientBatchItem.recertify",
        "python": "kmipkit.extensions.client_batch_item_recertify",
        "evidence": {
            "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_recertify_operation.py",
            "tools/api_manifest/tests/test_t102_optional_request_structure_mapping.py",
        },
    },
)
BLOCKED_CANDIDATES_BEFORE_BATCH = 669
BLOCKED_CANDIDATES_AFTER_BATCH = 647
RECERTIFY_TYPE_ID = "rust-observed-4e379cb3b97bca658dbf"
WRAPPING_REQUEST_BUILDER_IDS = {
    "rust-observed-9cd4c6b6fd190aff211f",
    "rust-observed-506281d0cf8e3c41a90e",
    "rust-observed-61aecad439c9a84c7241",
    "rust-observed-ba1772cec998d92e5148",
}
WRAPPING_REQUEST_BUILDER_OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-IMPORT",
    "KMIPKIT-ELEM-OP-C2S-EXPORT",
    "KMIPKIT-ELEM-OP-C2S-GET",
}
RAW_RECERTIFY_DECLARATIONS = {
    "rust-observed-bd90b6e53d6c05c1a6b0",
    "rust-observed-4c37b205dc5bee05df9a",
}
RECERTIFY_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"
RECERTIFY_OPERATION_ADDED_EVIDENCE = {
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
}
PKCS11_REQUEST_SETTER_IDS = {
    "rust-observed-60273eccedf62e75a65e",
    "rust-observed-b1e7666a5a57f054a891",
    "rust-observed-37ff26bed9cff3029512",
    "rust-observed-837fc199aac73b1ffd00",
    "rust-observed-cc71aab290dc65b5928f",
    "rust-observed-573ae1d0de80ffa2884b",
}
# The non-target snapshot includes subsequent approved T102 manifest slices.
# Includes the reviewed 0047 release stack and complete 0048/0049 inventories.
# Snapshot includes final 0049 and exact 0050 constructor mappings.
# The snapshot includes the approved KMIPKIT-0052 async typed-reader routes.
# It also includes the 50 approved KMIPKIT-0099 fluent Create Split Key declarations.
OTHER_DECLARATIONS_SHA256 = "2262c64f426e388615f5f1ed48ca0a4d798bbaeb39c200c80a96b66832688ee1"
OTHER_OPERATIONS_SHA256 = "2ea94003f2877d0e6d84271d67d9add6f48a5c5b7a837c48b22fe40f41022d39"
def canonical_sha256(value: object) -> str:
    encoded = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


class T102OptionalRequestStructureMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }

    def test_exact_six_optional_request_structures_map_to_existing_builders(self) -> None:
        self.assertEqual(6, len(TARGETS))
        for target in TARGETS:
            with self.subTest(export_name=target["name"], tag=f"{target['tag']:06x}"):
                declaration = self.declarations[target["id"]]
                operation = next(
                    row for row in self.manifest["operations"]
                    if row["operationId"] == target["operation"]
                )
                self.assertEqual(target["name"], declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual(target["operation"], declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(target["owner"], declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                requirements = set(declaration["requirementIds"])
                self.assertEqual(target["fr"], requirements)
                self.assertFalse(any(value.startswith("KMIPKIT-REQ-SPEC-") for value in requirements))
                expected_mappings = {
                    "rust": {"packageId": "rust-kmipkit-protocol", "name": target["name"]},
                    "c": {"packageId": "c-kmipkit", "name": target["c"]},
                    "java": {"packageId": "java-kmipkit", "name": target["java"]},
                    "python": {"packageId": "python-kmipkit", "name": target["python"]},
                }
                self.assertEqual(
                    expected_mappings,
                    declaration["languageMappings"],
                )
                self.assertTrue(target["evidence"] <= set(declaration["parityEvidenceIds"]))
                self.assertIn(target["id"], operation["declarations"]["rust"])

    def test_oasis_request_tables_mark_fields_optional_and_tags_are_catalogued(self) -> None:
        raw_html = OASIS_HTML.read_bytes()
        self.assertEqual(
            "8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF",
            hashlib.sha256(raw_html).hexdigest().upper(),
        )
        text = " ".join(re.sub(r"<[^>]+>", " ", raw_html.decode("cp1252")).split())
        for table_title in (
            "Table 244: Join Split Key Request Payload",
            "Table 300: Re-certify Request Payload",
            "Table 305: Re-key Request Payload",
        ):
            with self.subTest(table=table_title):
                caption_offset = text.index(table_title)
                payload_header = text.rfind(
                    "Request Payload Item REQUIRED Description", 0, caption_offset
                )
                payload = text[payload_header:caption_offset]
                self.assertRegex(payload, r"Attributes\s+No\s+Specifies desired object attributes\.")
                self.assertRegex(
                    payload,
                    r"Protection Storage Masks\s+No\s+Specifies all permissible Protection Storage Mask selections for the new object",
                )

        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        elements = {row["element_id"]: row for row in catalog["elements"]}
        self.assertIn("Table 117: Protection Storage Mask", text)
        self.assertIn("Protection Storage Mask Integer", text)
        self.assertEqual("Protection Storage Masks", elements["KMIPKIT-ELEM-TAG-42015F"]["name"])
        self.assertEqual("Protection Storage Mask", elements["KMIPKIT-ELEM-TAG-42015E"]["name"])
        self.assertEqual("42015F", elements["KMIPKIT-ELEM-TAG-42015F"]["wire_value"])
        self.assertEqual("42015E", elements["KMIPKIT-ELEM-TAG-42015E"]["wire_value"])
        for target in TARGETS:
            tag_id = f"KMIPKIT-ELEM-TAG-{target['tag']:06X}"
            self.assertIn(tag_id, elements)
            self.assertEqual("assigned", elements[tag_id]["allocation"])
            self.assertEqual("client_1_0", elements[tag_id]["scope_state"])

    def test_structure_wire_fixtures_use_integer_mask_child_tag(self) -> None:
        for relative_path in MASK_WIRE_FIXTURE_SOURCES:
            with self.subTest(path=relative_path):
                source = (ROOT / relative_path).read_text(encoding="utf-8").lower().replace("_", "")
                self.assertIn("42015e", source)
                self.assertNotIn("420160", source)

    def test_only_these_six_rows_leave_the_t102_blocked_inventory(self) -> None:
        target_ids = {target["id"] for target in TARGETS}
        self.assertEqual(6, len(target_ids))
        verified_target_ids = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] in target_ids
            and row["declarationState"] == "verified"
        }
        self.assertEqual(target_ids, verified_target_ids)
        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(
            BLOCKED_CANDIDATES_BEFORE_BATCH
            - len(target_ids)
            - len(CERTIFY_TARGET_IDS)
            - 1
            - len(WRAPPING_REQUEST_BUILDER_IDS)
            - len(PKCS11_REQUEST_SETTER_IDS),
            BLOCKED_CANDIDATES_AFTER_BATCH,
        )
        self.assertEqual(
            "verified",
            self.declarations[RECERTIFY_TYPE_ID]["declarationState"],
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))

    def test_all_non_target_declarations_and_operations_match_the_pinned_snapshot(self) -> None:
        target_ids = {target["id"] for target in TARGETS}
        other_declarations = copy.deepcopy([
            row for row in self.manifest["publicDeclarations"]
            if row["declarationId"] not in target_ids
        ])
        blocked_reason = (
            "The public Rust declaration has no resolved owner in the approved dependency inventory; "
            "assign an owner before advertising it."
        )
        for row in other_declarations:
            if row["declarationId"] in {
                "rust-observed-e0a06a71290bee987cc7",
                "rust-observed-5722377217051635233d",
                *RAW_RECERTIFY_DECLARATIONS,
            }:
                row["parityEvidenceIds"].remove(
                    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
                )
            if row["declarationId"] == RECERTIFY_TYPE_ID:
                row["capabilityId"] = "public_binding_surface"
                row["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                row["declarationState"] = "blocked"
                row["candidateSpecId"] = "KMIPKIT-0043"
                row["blockedReason"] = blocked_reason
                row.pop("approvalSource", None)
                row.pop("languageMappings", None)
                row.pop("parityEvidenceIds", None)
            if row["declarationId"] in CERTIFY_TARGET_IDS:
                row["capabilityId"] = "public_binding_surface"
                row["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                row["declarationState"] = "blocked"
                row["candidateSpecId"] = "KMIPKIT-0043"
                row["blockedReason"] = blocked_reason
                row.pop("approvalSource", None)
                row.pop("languageMappings", None)
                row.pop("parityEvidenceIds", None)
            if row["declarationId"] in WRAPPING_REQUEST_BUILDER_IDS:
                row["capabilityId"] = "public_binding_surface"
                row["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                row["declarationState"] = "blocked"
                row["candidateSpecId"] = "KMIPKIT-0043"
                row["blockedReason"] = blocked_reason
                row.pop("approvalSource", None)
                row.pop("languageMappings", None)
                row.pop("parityEvidenceIds", None)
            if row["declarationId"] in PKCS11_REQUEST_SETTER_IDS:
                row["capabilityId"] = "public_binding_surface"
                row["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                row["declarationState"] = "blocked"
                row["candidateSpecId"] = "KMIPKIT-0043"
                row["blockedReason"] = blocked_reason
                row.pop("approvalSource", None)
                row.pop("languageMappings", None)
                row.pop("parityEvidenceIds", None)
        # Keep the non-target snapshot aligned with the reconciled 0053 routes;
        # the capability inventory includes verified routes and explicit open
        # candidates rather than treating the whole category as blocked.
        for capability, count in (
            ("typed_object_attributes", 544),
            ("typed_managed_objects_and_key_material", 762),
        ):
            with self.subTest(stack_capability=capability):
                typed_rows = [
                    row for row in other_declarations
                    if row["capabilityId"] == capability
                ]
                self.assertEqual(count, len(typed_rows))
                states = {row["declarationState"] for row in typed_rows}
                self.assertTrue(states.issubset({"verified", "blocked"}))
                self.assertEqual(
                    {"verified": 544}
                    if capability == "typed_object_attributes"
                    else {"verified": 762},
                    {
                        state: sum(row["declarationState"] == state for row in typed_rows)
                        for state in states
                    },
                )
        other_declarations = without_0047_versioned_values(other_declarations)
        self.assertEqual(OTHER_DECLARATIONS_SHA256, canonical_sha256(other_declarations))

        other_operations = copy.deepcopy(self.manifest["operations"])
        for operation in other_operations:
            operation_target_ids = {
                target["id"]
                for target in TARGETS
                if target["operation"] == operation["operationId"]
            }
            if operation["operationId"] == CERTIFY_OPERATION_ID:
                operation_target_ids.update(CERTIFY_TARGET_IDS)
            if operation["operationId"] == RECERTIFY_OPERATION_ID:
                operation_target_ids.add(RECERTIFY_TYPE_ID)
            if operation["operationId"] in WRAPPING_REQUEST_BUILDER_OPERATIONS:
                operation_target_ids.update(
                    WRAPPING_REQUEST_BUILDER_IDS
                    & set(operation["declarations"]["rust"])
                )
            if operation["operationId"] == "KMIPKIT-ELEM-OP-C2S-PKCS-11":
                operation_target_ids.update(PKCS11_REQUEST_SETTER_IDS)
            if operation_target_ids:
                operation["declarations"]["rust"] = [
                    value for value in operation["declarations"]["rust"]
                    if value not in operation_target_ids
                ]
            if operation["operationId"] == CERTIFY_OPERATION_ID:
                operation["parityEvidenceIds"] = [
                    evidence_id for evidence_id in operation["parityEvidenceIds"]
                    if evidence_id not in CERTIFY_OPERATION_ADDED_EVIDENCE
                ]
            if operation["operationId"] == RECERTIFY_OPERATION_ID:
                operation["parityEvidenceIds"] = [
                    evidence_id for evidence_id in operation["parityEvidenceIds"]
                    if evidence_id not in RECERTIFY_OPERATION_ADDED_EVIDENCE
                ]
        self.assertEqual(
            "a1e58fe7e76de32113a53d5ad14e6c40ca822a8e51a6cc53deb99f67b5486663",
            canonical_sha256(other_operations),
        )


if __name__ == "__main__":
    unittest.main()
