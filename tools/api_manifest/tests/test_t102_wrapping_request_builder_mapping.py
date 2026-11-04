"""Pin the four Table 238, 217, and 220 wrapping request mappings for T102."""

from __future__ import annotations

import html
import hashlib
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OASIS = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
BASE_BLOCKED = 658
EXPECTED_BLOCKED = 654
COMMON_EVIDENCE = {
    "c": "crates/kmip-ffi/tests/c_api_coverage.rs",
    "c_consumer": "bindings/c/tests/client_abi_consumer.c",
    "java": "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "python": "bindings/python/tests/test_wrapping_request_wire.py",
    "contract": "tools/api_manifest/tests/test_t102_wrapping_request_builder_mapping.py",
}
TARGETS = {
    "rust-observed-9cd4c6b6fd190aff211f": {
        "name": "kmipkit_protocol::ImportRequest::with_key_wrap_type",
        "operation": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        "owner": "specs/023-key-material-transfer/spec.md",
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.25-006"},
        "fr": {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004", "KMIPKIT-0037-FR-006", "KMIPKIT-0037-FR-008"},
        "builder": "kmipkit_client_batch_item_import",
        "rust_test": "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs",
    },
    "rust-observed-506281d0cf8e3c41a90e": {
        "name": "kmipkit_protocol::ImportRequest::with_replace_existing",
        "operation": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        "owner": "specs/023-key-material-transfer/spec.md",
        "requirements": set(),
        "fr": {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004", "KMIPKIT-0037-FR-008"},
        "builder": "kmipkit_client_batch_item_import",
        "rust_test": "crates/kmipkit-protocol/tests/unit/import_operation_tests.rs",
    },
    "rust-observed-61aecad439c9a84c7241": {
        "name": "kmipkit_protocol::ExportRequest::with_key_wrapping_specification",
        "operation": "KMIPKIT-ELEM-OP-C2S-EXPORT",
        "owner": "specs/023-key-material-transfer/spec.md",
        "requirements": {
            "KMIPKIT-REQ-SPEC-6.1.18-001-001",
            "KMIPKIT-REQ-SPEC-6.1.18-008",
            "KMIPKIT-REQ-SPEC-7.18-001-001",
            "KMIPKIT-REQ-SPEC-7.18-004",
        },
        "fr": {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-002", "KMIPKIT-0037-FR-004", "KMIPKIT-0037-FR-006", "KMIPKIT-0037-FR-008"},
        "builder": "kmipkit_client_batch_item_export",
        "rust_test": "crates/kmipkit-protocol/tests/unit/export_operation_tests.rs",
    },
    "rust-observed-ba1772cec998d92e5148": {
        "name": "kmipkit_protocol::GetRequest::with_key_wrapping_specification",
        "operation": "KMIPKIT-ELEM-OP-C2S-GET",
        "owner": "specs/017-managed-object-retrieval/spec.md",
        "requirements": {
            "KMIPKIT-REQ-SPEC-7.18-001-001",
            "KMIPKIT-REQ-SPEC-7.18-004",
        },
        "fr": {"KMIPKIT-0017-FR-001", "KMIPKIT-0017-FR-002", "KMIPKIT-0017-FR-003", "KMIPKIT-0017-FR-005", "KMIPKIT-0017-FR-013"},
        "builder": "kmipkit_client_batch_item_get",
        "rust_test": "crates/kmipkit-protocol/tests/unit/get_tests.rs",
    },
}


class T102WrappingRequestBuilderMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_package_name = {
            (row.get("packageId"), row.get("exportName")): row
            for row in cls.manifest["publicDeclarations"]
        }

    def test_only_the_four_audited_wrapping_builders_are_mapped(self) -> None:
        self.assertEqual(len(TARGETS), 4)
        for declaration_id, expected in TARGETS.items():
            with self.subTest(declaration_id=declaration_id):
                declaration = self.declarations[declaration_id]
                self.assertEqual(expected["name"], declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual(expected["operation"], declaration["capabilityId"])
                self.assertEqual(expected["owner"], declaration["approvalSource"]["path"])
                linked_ids = set(declaration["requirementIds"])
                self.assertTrue(expected["fr"] <= linked_ids)
                self.assertTrue(expected["requirements"] <= linked_ids)
                expected_evidence = set(COMMON_EVIDENCE.values()) | {expected["rust_test"]}
                self.assertTrue(expected_evidence <= set(declaration["parityEvidenceIds"]))
                self.assertEqual("four-language", declaration["parityScope"])

                builder = self.by_package_name[("c-kmipkit", expected["builder"])]
                self.assertEqual(expected["operation"], builder["capabilityId"])
                self.assertEqual(expected["owner"], builder["approvalSource"]["path"])
                mappings = dict(builder["languageMappings"])
                mappings["rust"] = {"packageId": "rust-kmipkit-protocol", "name": expected["name"]}
                self.assertEqual(mappings, declaration["languageMappings"])
                operation = next(
                    row for row in self.manifest["operations"]
                    if row["operationId"] == expected["operation"]
                )
                self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_only_the_four_exact_rows_leave_the_blocked_inventory(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(EXPECTED_BLOCKED, blocked)
        before = blocked + len(TARGETS)
        self.assertEqual(BASE_BLOCKED, before)

    def test_client_and_server_requirement_scope_is_exact(self) -> None:
        requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        server_replace = requirements["KMIPKIT-REQ-SPEC-6.1.25-004"]
        self.assertEqual("server", server_replace["role"])
        self.assertEqual("server_to_client", server_replace["direction"])
        self.assertEqual("server_only", server_replace["scope_state"])
        for declaration_id in (
            "rust-observed-9cd4c6b6fd190aff211f",
            "rust-observed-506281d0cf8e3c41a90e",
        ):
            self.assertNotIn(server_replace["requirement_id"], self.declarations[declaration_id]["requirementIds"])
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-7.18-004",
            "KMIPKIT-REQ-SPEC-7.18-005",
            "KMIPKIT-REQ-SPEC-7.18-006",
        ):
            requirement = requirements[requirement_id]
            self.assertEqual("client", requirement["role"])
            self.assertEqual("client_to_server", requirement["direction"])
        get_req_ids = set(self.declarations["rust-observed-ba1772cec998d92e5148"]["requirementIds"])
        export_req_ids = set(self.declarations["rust-observed-61aecad439c9a84c7241"]["requirementIds"])
        self.assertIn("KMIPKIT-REQ-SPEC-7.18-004", get_req_ids)
        self.assertIn("KMIPKIT-REQ-SPEC-7.18-004", export_req_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-7.18-005", get_req_ids | export_req_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-7.18-006", get_req_ids | export_req_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-6.1.19-001", get_req_ids)

    def test_oasis_tables_pin_optional_fields_and_valid_wrapping_shape(self) -> None:
        raw = OASIS.read_bytes()
        self.assertEqual(
            "8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF",
            hashlib.sha256(raw).hexdigest().upper(),
        )
        source = raw.decode("cp1252")
        plain = html.unescape(re.sub(r"<[^>]+>", " ", source))
        plain = " ".join(plain.split())

        def table_payload(caption: str) -> str:
            caption_at = plain.index(caption)
            start = plain.rfind("Request Payload Item REQUIRED Description", 0, caption_at)
            return plain[start:caption_at]

        import_rows = table_payload("Table 238: Import Request Payload")
        self.assertRegex(import_rows, r"Replace Existing No A Boolean\.")
        self.assertRegex(import_rows, r"Key Wrap Type If and only if the key object is wrapped\.")
        self.assertIn("The object and attributes MAY be wrapped.", import_rows)
        export_rows = table_payload("Table 217: Export Request Payload")
        get_rows = table_payload("Table 220: Get Request Payload")
        self.assertRegex(export_rows, r"Key Wrapping Specification No")
        self.assertRegex(get_rows, r"Key Wrapping Specification No")

        table_370_at = plain.index("Table 370: Key Wrapping Specification Object Structure")
        table_370 = plain[max(0, table_370_at - 3500):table_370_at]
        self.assertRegex(table_370, r"Wrapping Method Enumeration Yes")
        self.assertRegex(
            table_370,
            r"Encryption Key Information Structure No, SHALL be present if MAC/Signature Key Information is omitted",
        )
        self.assertIn("An Encoding Option", plain[plain.index("7.18 Key Wrapping Specification"):table_370_at])


if __name__ == "__main__":
    unittest.main()
