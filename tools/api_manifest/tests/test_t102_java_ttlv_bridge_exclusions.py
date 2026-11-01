"""Pin the generic TTLV Java JNI bridge as implementation-only inventory."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/043-public-api-bindings/spec.md"
PACKAGE = "java-kmipkit"
EXPECTED_BRIDGE_EXPORTS = frozenset(
    f"org.kmipkit.NativeExtensionRegistry.{name}"
    for name in (
        "RAW_TAG",
        "TAG",
        "TTLV_VALUE",
        "TTLV_STRUCTURE",
        "TTLV_ITEM",
        "TTLV_STRUCTURE_VIEW",
        "TTLV_ITEM_VIEW",
        "TTLV_VALUE_VIEW",
        "rawTagCreate",
        "rawTagValue",
        "rawTagChecked",
        "tagValue",
        "ttlvItemCreate",
        "ttlvItemViewTag",
        "ttlvItemViewType",
        "ttlvItemViewValue",
        "ttlvStructureCreate",
        "ttlvStructureWithItem",
        "ttlvStructureView",
        "ttlvStructureViewItemAt",
        "ttlvStructureViewItemCount",
        "ttlvValueInteger",
        "ttlvValueLongInteger",
        "ttlvValueEnumeration",
        "ttlvValueBoolean",
        "ttlvValueDateTime",
        "ttlvValueInterval",
        "ttlvValueDateTimeExtended",
        "ttlvValueBigInteger",
        "ttlvValueTextString",
        "ttlvValueByteString",
        "ttlvValueStructure",
        "ttlvValueView",
        "ttlvValueViewStructure",
        "ttlvValueViewType",
        "ttlvValueViewInteger",
        "ttlvValueViewLongInteger",
        "ttlvValueViewEnumeration",
        "ttlvValueViewBoolean",
        "ttlvValueViewDateTime",
        "ttlvValueViewInterval",
        "ttlvValueViewDateTimeExtended",
        "ttlvValueViewByteLength",
        "ttlvValueViewByteAt",
        "codecLimitValues",
    )
)


class JavaTtlvBridgeExclusionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        self.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }

    def test_exact_generic_ttlv_jni_bridge_exports_are_excluded(self) -> None:
        self.assertEqual(45, len(EXPECTED_BRIDGE_EXPORTS))
        actual_exclusions = {
            name
            for package, name in self.exclusions
            if package == PACKAGE and name in EXPECTED_BRIDGE_EXPORTS
        }
        self.assertEqual(EXPECTED_BRIDGE_EXPORTS, actual_exclusions)

        for export_name in sorted(EXPECTED_BRIDGE_EXPORTS):
            with self.subTest(export_name=export_name):
                exclusion = self.exclusions[(PACKAGE, export_name)]
                self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("JNI bridge", exclusion["reason"])
                self.assertIn("typed facade", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn("KMIPKIT-0004", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_neighboring_extension_and_codec_bridge_exports_stay_unresolved(self) -> None:
        for export_name in (
            "org.kmipkit.NativeExtensionRegistry.codecLimitsCreate",
            "org.kmipkit.NativeExtensionRegistry.ttlvPathCreate",
            "org.kmipkit.NativeExtensionRegistry.ttlvPathWithChildTag",
        ):
            with self.subTest(export_name=export_name):
                declaration = self.declarations[(PACKAGE, export_name)]
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertEqual("KMIPKIT-0043", declaration["candidateSpecId"])
                self.assertNotIn((PACKAGE, export_name), self.exclusions)


if __name__ == "__main__":
    unittest.main()