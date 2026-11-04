"""Pin exactly four optional Certify request-builder mappings for T102."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
OWNER = "specs/022-object-lifecycle/spec.md"
OPERATION = "KMIPKIT-ELEM-OP-C2S-CERTIFY"
C_BUILDER = "kmipkit_client_batch_item_certify"
JAVA_BUILDER = "org.kmipkit.extensions.ClientBatchItem.certify"
PYTHON_BUILDER = "kmipkit.extensions.client_batch_item_certify"
EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_certify_operation.py",
    "tools/api_manifest/tests/test_t102_certify_request_builder_mapping.py",
}
TARGETS = (
    {
        "id": "rust-observed-ff892d7f1e1585911b9c",
        "name": "kmipkit_protocol::CertifyRequest::with_unique_identifier",
        "field": "Unique Identifier",
        "tag": 0x420094,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-002-001", "KMIPKIT-REQ-SPEC-6.1.6-002-002"},
    },
    {
        "id": "rust-observed-25fb2f73d24e03d2ab13",
        "name": "kmipkit_protocol::CertifyRequest::with_certificate_request_type",
        "field": "Certificate Request Type",
        "tag": 0x420019,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-007"},
    },
    {
        "id": "rust-observed-75b1748bcbc69c385a66",
        "name": "kmipkit_protocol::CertifyRequest::with_attributes",
        "field": "Attributes",
        "tag": 0x420125,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-002-003"},
    },
    {
        "id": "rust-observed-ddcac6317bf0ae830eab",
        "name": "kmipkit_protocol::CertifyRequest::with_protection_storage_masks",
        "field": "Protection Storage Masks",
        "tag": 0x42015F,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-008"},
        "requirements": set(),
    },
)
BLOCKED_NEIGHBORS = {
    "rust-observed-f928aa3b375fa686e506": "kmipkit_protocol::CertifyRequest",
    "rust-observed-9a4cd073497d09724f7e": "kmipkit_protocol::CertifyRequest::new",
    "rust-observed-3620d8e9bb89e3bbe251": "kmipkit_protocol::CertifyRequest::with_certificate_request_value",
    "rust-observed-d0317408cb64ea049eaf": "kmipkit_protocol::CertifyRequest::unique_identifier",
    "rust-observed-455bf70fd49c2d1c0584": "kmipkit_protocol::CertifyRequest::into_ttlv_payload",
}
BLOCKED_BEFORE = 663
BLOCKED_AFTER = 659
DECLARATIONS_SHA256 = "8e003e145a03c1a585e3bd49bb6dd7599ed19342ce762fd185007e623babdcda"
OPERATIONS_SHA256 = "8ee7f0c367d2125be38c05d42159b515900b868ec1a898206492bc3c2fad969a"


def canonical_sha256(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


class T102CertifyRequestBuilderMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {row["declarationId"]: row for row in cls.manifest["publicDeclarations"]}
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.elements = {row["element_id"]: row for row in cls.catalog["elements"]}

    def test_exact_four_certify_request_setters_map_to_existing_builders(self) -> None:
        self.assertEqual(4, len(TARGETS))
        for target in TARGETS:
            with self.subTest(declaration=target["id"], field=target["field"]):
                declaration = self.declarations[target["id"]]
                self.assertEqual(target["name"], declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual(OPERATION, declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual(target["fr"] | target["requirements"], set(declaration["requirementIds"]))
                self.assertEqual(
                    {
                        "rust": {"packageId": "rust-kmipkit-protocol", "name": target["name"]},
                        "c": {"packageId": "c-kmipkit", "name": C_BUILDER},
                        "java": {"packageId": "java-kmipkit", "name": JAVA_BUILDER},
                        "python": {"packageId": "python-kmipkit", "name": PYTHON_BUILDER},
                    },
                    declaration["languageMappings"],
                )
                self.assertTrue(EVIDENCE <= set(declaration["parityEvidenceIds"]))
                tag_id = f"KMIPKIT-ELEM-TAG-{target['tag']:06X}"
                self.assertIn(tag_id, self.elements)
                self.assertEqual("assigned", self.elements[tag_id]["allocation"])
                self.assertEqual("client_1_0", self.elements[tag_id]["scope_state"])
                self.assertIn(target["id"], self.operation["declarations"]["rust"])

    def test_only_the_four_audited_setters_move_from_blocked_and_neighbors_stay_blocked(self) -> None:
        target_ids = {target["id"] for target in TARGETS}
        self.assertEqual(4, len(target_ids))
        verified = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] in target_ids and row["declarationState"] == "verified"
        }
        self.assertEqual(target_ids, verified)
        for declaration_id, name in BLOCKED_NEIGHBORS.items():
            with self.subTest(blocked_neighbor=name):
                row = self.declarations[declaration_id]
                self.assertEqual(name, row["exportName"])
                self.assertEqual("blocked", row["declarationState"])
                self.assertEqual("KMIPKIT-0043", row["candidateSpecId"])
        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(BLOCKED_BEFORE - len(target_ids), BLOCKED_AFTER)
        self.assertEqual(BLOCKED_AFTER, blocked)
        self.assertEqual(6_470, len(self.manifest["publicDeclarations"]))
        self.assertEqual(8_554, len(self.manifest["approvedExclusions"]))

    def test_oasis_table_179_fields_and_child_mask_tag_are_pinned(self) -> None:
        raw_html = OASIS_HTML.read_bytes()
        self.assertEqual(
            "8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF",
            hashlib.sha256(raw_html).hexdigest().upper(),
        )
        text = " ".join(re.sub(r"<[^>]+>", " ", raw_html.decode("cp1252")).split())
        caption = text.index("Table 179: Certify Request Payload")
        header = text.rfind("Request Payload Item REQUIRED Description", 0, caption)
        payload = text[header:caption]
        for field in ("Unique Identifier", "Certificate Request Type", "Attributes", "Protection Storage Masks"):
            with self.subTest(field=field):
                self.assertRegex(payload, rf"{re.escape(field)}\s+No\b")
        self.assertIn("Table 117: Protection Storage Mask", text)
        self.assertIn("Protection Storage Mask Integer", text)
        self.assertEqual("Protection Storage Masks", self.elements["KMIPKIT-ELEM-TAG-42015F"]["name"])
        self.assertEqual("Protection Storage Mask", self.elements["KMIPKIT-ELEM-TAG-42015E"]["name"])

    def test_non_target_declarations_and_operation_rows_match_snapshot(self) -> None:
        target_ids = {target["id"] for target in TARGETS}
        other_declarations = [row for row in self.manifest["publicDeclarations"] if row["declarationId"] not in target_ids]
        self.assertEqual(DECLARATIONS_SHA256, canonical_sha256(other_declarations))
        other_operations = copy.deepcopy(self.manifest["operations"])
        for operation in other_operations:
            if operation["operationId"] == OPERATION:
                operation["declarations"]["rust"] = [
                    declaration_id for declaration_id in operation["declarations"]["rust"]
                    if declaration_id not in target_ids
                ]
        self.assertEqual(OPERATIONS_SHA256, canonical_sha256(other_operations))

    @property
    def operation(self) -> dict[str, object]:
        return next(row for row in self.manifest["operations"] if row["operationId"] == OPERATION)


if __name__ == "__main__":
    unittest.main()