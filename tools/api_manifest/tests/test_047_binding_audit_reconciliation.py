"""Pin the exact cross-language inventory corrections exposed by release sync."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"


class BindingAuditReconciliationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_wire_encoding_jni_and_enum_helpers_have_exact_exclusions(self) -> None:
        expected = {
            "org.kmipkit.NativeExtensionRegistry.clientConfigurationWireEncoding",
            "org.kmipkit.NativeExtensionRegistry.clientConfigurationWithWireEncoding",
            "org.kmipkit.NativeExtensionRegistry.clientExecuteWithEncoding",
            "org.kmipkit.extensions.WireEncoding.valueOf",
            "org.kmipkit.extensions.WireEncoding.values",
        }
        actual = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit"
        }
        self.assertTrue(expected <= actual)

        records = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit"
            and row["exportName"] in expected
        }
        for export_name, record in records.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(
                    "specs/057-multi-encoding-client-integration/spec.md",
                    record["approvalSource"]["path"],
                )
                self.assertEqual("Approved", record["approvalSource"]["status"])
                self.assertTrue(record["reason"])
                self.assertTrue(record["evidence"])

    def test_execute_with_options_declares_both_java_overloads(self) -> None:
        declaration = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "java-kmipkit"
            and row["exportName"] == "org.kmipkit.extensions.Client.executeWithOptions"
        )
        self.assertEqual(
            [
                "public org.kmipkit.extensions.ClientBatchResponse executeWithOptions(org.kmipkit.extensions.ClientBatch, org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.RequestOptions) throws org.kmipkit.ClientOperationException",
                "public org.kmipkit.extensions.ClientBatchResponse executeWithOptions(org.kmipkit.extensions.ClientBatch, org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.WireEncoding) throws org.kmipkit.ClientOperationException",
            ],
            declaration["javaSignatures"],
        )

    def test_python_wire_encoding_property_uses_the_audited_export_kind(self) -> None:
        declaration = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"] == "kmipkit.extensions.ClientConfiguration.wire_encoding"
        )
        self.assertEqual("python-export", declaration["kind"])

    def test_named_value_manifest_has_no_stale_snake_case_java_mask_helpers(self) -> None:
        stale = [
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("capabilityId") == "named_protocol_values"
            and row.get("packageId") == "java-kmipkit"
            and row.get("exportName", "").endswith((".known_bits", ".unknown_bits"))
        ]
        self.assertEqual([], stale)

    def test_bitmask_single_bit_name_lookup_is_c_only(self) -> None:
        rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if "KMIPKIT-0047-FR-004" in row.get("requirementIds", [])
            and row.get("exportName", "").endswith("name")
        ]
        self.assertEqual(3, len(rows))
        self.assertEqual({"c-kmipkit"}, {row["packageId"] for row in rows})
        self.assertEqual(
            {"named_protocol_mask_single_bit_lookup"},
            {row["capabilityId"] for row in rows},
        )
        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "named_protocol_mask_single_bit_lookup"
        )
        self.assertEqual("c-only", capability["parityScope"])


if __name__ == "__main__":
    unittest.main()
