"""Exact T102 mappings for approved scalar request builders across five operations."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
T102_CANDIDATES_AFTER_BATCH = 665
DEFERRED_TTLV = {
    "rust-observed-ba1772cec998d92e5148": "kmipkit_protocol::GetRequest::with_key_wrapping_specification",
    "rust-observed-61aecad439c9a84c7241": "kmipkit_protocol::ExportRequest::with_key_wrapping_specification",
}
NEW_EVIDENCE = {
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "tools/api_manifest/tests/test_t102_request_builder_mapping_batch.py",
}
OPERATIONS = {
    "check": {
        "owner": "specs/022-object-lifecycle/spec.md",
        "reference": "OASIS KMIP v2.1 §6.1.7 Tables 183–185",
        "capability": "KMIPKIT-ELEM-OP-C2S-CHECK",
        "builder": "kmipkit_client_batch_item_check",
        "rust_package": "rust-kmipkit-protocol",
        "normative": {
            "KMIPKIT-REQ-SPEC-6.1.7-001",
            "KMIPKIT-REQ-SPEC-6.1.7-002-001",
            "KMIPKIT-REQ-SPEC-6.1.7-002-002",
            "KMIPKIT-REQ-SPEC-6.1.7-003",
            "KMIPKIT-REQ-SPEC-6.1.7-004-001",
            "KMIPKIT-REQ-SPEC-6.1.7-004-002",
            "KMIPKIT-REQ-SPEC-6.1.7-005",
        },
        "targets": (
            ("rust-observed-3beea67907cb67493c50", "kmipkit_protocol::CheckRequest::new"),
            ("rust-observed-7ef0a3fa4f7211edc2d4", "kmipkit_protocol::CheckRequest::with_unique_identifier"),
            ("rust-observed-2117b9df5e767118ae4c", "kmipkit_protocol::CheckRequest::with_usage_limits_count"),
            ("rust-observed-ad3438f583c6a1a34512", "kmipkit_protocol::CheckRequest::with_cryptographic_usage_mask"),
            ("rust-observed-4d485551dac41dbd1ef3", "kmipkit_protocol::CheckRequest::with_lease_time"),
        ),
    },
    "get": {
        "owner": "specs/017-managed-object-retrieval/spec.md",
        "reference": "OASIS KMIP v2.1 §6.1.19 Tables 220–222",
        "capability": "KMIPKIT-ELEM-OP-C2S-GET",
        "builder": "kmipkit_client_batch_item_get",
        "rust_package": "rust-kmipkit-protocol",
        "normative": {
            "KMIPKIT-REQ-SPEC-6.1.19-001",
            "KMIPKIT-REQ-SPEC-6.1.19-002",
        },
        "targets": (
            ("rust-observed-66a8b011f8175b8c04f5", "kmipkit_protocol::GetRequest::new"),
            ("rust-observed-684a647c13f5225359fa", "kmipkit_protocol::GetRequest::with_unique_identifier"),
            ("rust-observed-c2da75d2d210d98b7ea5", "kmipkit_protocol::GetRequest::with_key_format_type"),
            ("rust-observed-6611a6ba852bddfeb702", "kmipkit_protocol::GetRequest::with_key_wrap_type"),
            ("rust-observed-e0c241319e5da88830dd", "kmipkit_protocol::GetRequest::with_key_compression_type"),
        ),
    },
    "export": {
        "owner": "specs/023-key-material-transfer/spec.md",
        "reference": "OASIS KMIP v2.1 §6.1.18 Tables 217–219",
        "capability": "KMIPKIT-ELEM-OP-C2S-EXPORT",
        "builder": "kmipkit_client_batch_item_export",
        "rust_package": "rust-kmipkit-protocol",
        "normative": set(),
        "targets": (
            ("rust-observed-e4656d1822c4f833d1ba", "kmipkit_protocol::ExportRequest::new"),
            ("rust-observed-27c208d3d7f52f071e4c", "kmipkit_protocol::ExportRequest::with_unique_identifier"),
            ("rust-observed-bc4b3a9f02b6b38524fa", "kmipkit_protocol::ExportRequest::with_key_format_type"),
            ("rust-observed-ea651cfc24b62abdda90", "kmipkit_protocol::ExportRequest::with_key_wrap_type"),
            ("rust-observed-5cf48b4b17981987d2f7", "kmipkit_protocol::ExportRequest::with_key_compression_type"),
        ),
    },
    "rekey": {
        "owner": "specs/022-object-lifecycle/spec.md",
        "reference": "OASIS KMIP v2.1 §6.1.46 Tables 305–307",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "builder": "kmipkit_client_batch_item_rekey",
        "rust_package": "rust-kmipkit-protocol",
        "normative": {
            "KMIPKIT-REQ-SPEC-6.1.46-001",
            "KMIPKIT-REQ-SPEC-6.1.46-004-001",
        },
        "targets": (
            ("rust-observed-4781288042e981f6d3bd", "kmipkit_protocol::RekeyRequest::new"),
            ("rust-observed-43be97b668d6f5efb547", "kmipkit_protocol::RekeyRequest::with_unique_identifier"),
            ("rust-observed-e04ecae89493ca7e5fbf", "kmipkit_protocol::RekeyRequest::with_offset"),
        ),
    },
    "get_usage_allocation": {
        "owner": "specs/025-policy-controls/spec.md",
        "reference": "OASIS KMIP v2.1 §6.1.23 Tables 232–234",
        "capability": "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
        "builder": "kmipkit_client_batch_item_get_usage_allocation",
        "rust_package": "rust-kmipkit-protocol",
        "normative": {"KMIPKIT-REQ-SPEC-6.1.23-002-001"},
        "targets": (
            ("rust-observed-51a60237c466a9503394", "kmipkit_protocol::GetUsageAllocationRequest::with_unique_identifier"),
            ("rust-observed-543f908844c7bb7b67bb", "kmipkit::GetUsageAllocationRequest::with_unique_identifier"),
        ),
    },
}


class T102RequestBuilderMappingBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }

    def test_only_the_exact_twenty_scalar_builder_rows_map_to_existing_parity(self) -> None:
        target_ids = {
            declaration_id
            for operation in OPERATIONS.values()
            for declaration_id, _ in operation["targets"]
        }
        self.assertEqual(20, len(target_ids))
        mapped_targets = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] in target_ids
            and row.get("declarationState") == "verified"
        }
        self.assertEqual(target_ids, mapped_targets)

        for operation_name, operation in OPERATIONS.items():
            builder = self.by_name[("c-kmipkit", operation["builder"])]
            self.assertEqual(operation["capability"], builder["capabilityId"])
            self.assertEqual(operation["owner"], builder["approvalSource"]["path"])
            self.assertTrue(operation["normative"].issubset(set(builder["requirementIds"])))
            operation_row = next(
                row for row in self.manifest["operations"]
                if row["operationId"] == operation["capability"]
            )

            for declaration_id, export_name in operation["targets"]:
                with self.subTest(operation=operation_name, declaration=export_name):
                    declaration = self.declarations[declaration_id]
                    self.assertEqual(export_name, declaration["exportName"])
                    self.assertEqual("verified", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertNotIn("blockedReason", declaration)
                    self.assertEqual("associated-item", declaration["kind"])
                    self.assertEqual(operation["capability"], declaration["capabilityId"])
                    self.assertEqual(builder["approvalSource"], declaration["approvalSource"])
                    self.assertEqual(builder["requirementIds"], declaration["requirementIds"])
                    if operation_name == "rekey":
                        self.assertNotIn(
                            "KMIPKIT-REQ-SPEC-6.1.46-004-002",
                            declaration["requirementIds"],
                        )
                    expected_mappings = dict(builder["languageMappings"])
                    expected_mappings["rust"] = {
                        "packageId": (
                            "rust-kmipkit"
                            if export_name.startswith("kmipkit::")
                            else operation["rust_package"]
                        ),
                        "name": export_name,
                    }
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertTrue(NEW_EVIDENCE.issubset(set(declaration["parityEvidenceIds"])))
                    self.assertIn(declaration_id, operation_row["declarations"]["rust"])

    def test_only_the_twenty_target_rows_leave_the_t102_blocked_inventory(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_CANDIDATES_AFTER_BATCH, blocked)

    def test_two_deferred_ttlv_rows_remain_blocked(self) -> None:
        for declaration_id, export_name in DEFERRED_TTLV.items():
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertEqual("KMIPKIT-0043", declaration["candidateSpecId"])

    def test_rekey_table_305_request_offset_is_interval_and_uid_is_optional(self) -> None:
        html = OASIS_HTML.read_text(encoding="cp1252")
        text = re.sub(r"<[^>]+>", " ", html)
        text = " ".join(text.split())
        request_caption = "Table 305: Re-key Request Payload"
        caption_offset = text.index(request_caption)
        payload_start = text.rfind("Request Payload Item REQUIRED Description", 0, caption_offset)
        payload_text = text[payload_start:caption_offset]
        unique_identifier_start = payload_text.index("Unique Identifier")
        offset_start = payload_text.index("Offset")
        self.assertRegex(
            payload_text[unique_identifier_start:offset_start],
            r"Unique Identifier\s+No\s+Determines the existing Symmetric Key being re-keyed\. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier\.",
        )
        self.assertRegex(
            payload_text[offset_start:],
            r"Offset\s+No\s+An Interval object indicating the difference between the Initial Date and the Activation Date of the replacement key to be created\.",
        )
        self.assertIn("Response Payload", text[caption_offset:])
        self.assertIn("Table 306: Re-key Response Payload", text[caption_offset:])


if __name__ == "__main__":
    unittest.main()
