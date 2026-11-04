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
EXPECTED_TIMESTAMP_SEQUENCE_BRIDGE_EXPORTS = frozenset(
    f"org.kmipkit.NativeExtensionRegistry.{name}"
    for name in (
        "HASHED_PASSWORD_TIMESTAMP_SEQUENCE",
        "hashedPasswordTimestampSequenceNew",
        "hashedPasswordTimestampSequenceRecord",
        "hashedPasswordTimestampSequenceLastTimestamp",
    )
)
EXPECTED_TTLV_PATH_BRIDGE_EXPORTS = frozenset(
    {
        "org.kmipkit.NativeExtensionRegistry.ttlvPathCreate",
        "org.kmipkit.NativeExtensionRegistry.ttlvPathWithChildTag",
    }
)
EXPECTED_VENDOR_EXTENSION_BRIDGE_MEMBERS = frozenset(
    {
        "compatibilityCreate",
        "discriminatorCreate",
        "extensionChildRuleOptional",
        "extensionChildRuleRepeated",
        "extensionChildRuleRequired",
        "extensionDefinitionCreate",
        "extensionDefinitionValidate",
        "extensionDefinitionWithInformation",
        "extensionIdentityCreate",
        "extensionIdentityName",
        "extensionIdentityVendorIdentifier",
        "extensionIdentityVersion",
        "extensionInformationCreate",
        "extensionInformationToTtlv",
        "extensionInformationWithAttribute",
        "extensionInformationWithDescription",
        "extensionInformationWithEnumeration",
        "extensionInformationWithParentStructureTag",
        "extensionInformationWithTag",
        "extensionInformationWithType",
        "extensionOrderConstraintCreate",
        "extensionRecognitionGenericValue",
        "extensionRecognitionValidatedValue",
        "extensionSchemaAllowedBitMask",
        "extensionSchemaAllowedEnumeration",
        "extensionSchemaMaximumLength",
        "extensionSchemaMinimumLength",
        "extensionSchemaRequiredBitMask",
        "extensionSchemaScalar",
        "extensionSchemaSignedRange",
        "extensionSchemaStructure",
        "extensionSchemaUnsignedRange",
        "validatedExtensionValueGenericValue",
        "validatedExtensionValueIdentity",
        "validatedExtensionValueValueAt",
        "clientConfigurationCreate",
        "clientConfigurationExtensionRegistry",
        "clientExtensionRegistryCreate",
        "clientExtensionRegistryDefinitionCount",
        "clientExtensionRegistryInspect",
        "clientExtensionRegistryValidate",
        "clientRequestMessageExtensionCreate",
        "clientBatchItemWithExtension",
        "clientBatchItemExtensionCriticalityIndicatorAt",
        "clientBatchItemExtensionIdentityAt",
        "CHILD_RULE",
        "COMPATIBILITY",
        "CONFIGURATION",
        "DEFINITION",
        "DISCRIMINATOR",
        "IDENTITY",
        "INFORMATION",
        "ORDER_CONSTRAINT",
        "RECOGNITION",
        "REGISTERED_VALUE",
        "REGISTRY",
        "REQUEST_EXTENSION",
        "SCHEMA",
        "TTLV_PATH",
        "VALIDATED_VALUE",
    }
)
EXPECTED_NON_FACADE_VENDOR_EXTENSION_BRIDGE_MEMBERS = frozenset(
    {
        "BATCH_ITEM",
        "PATH",
        "REGISTER_REQUEST",
        "clientBatchItemDiscoverVersions",
        "clientBatchItemExtensionCount",
        "clientBatchItemRegister",
        "clientExtensionRegistryDefinitionAt",
        "clientExtensionRegistryDefinitionForIdentity",
        "codecLimitsCreate",
        "ensureLoaded",
        "extensionDefinitionInformation",
        "extensionRecognitionIsRecognized",
        "registerRequestCreate",
        "registerRequestWithOpaqueObjectContext",
        "release",
    }
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
                self.assertEqual(PACKAGE, exclusion["packageId"])
                self.assertEqual(export_name, exclusion["exportName"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("JNI bridge", exclusion["reason"])
                self.assertIn("typed facade", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                if export_name.endswith(".codecLimitValues"):
                    self.assertIn("KMIPKIT-0005-FR-007", exclusion["evidence"])
                else:
                    self.assertIn("KMIPKIT-0004-FR-", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_unreferenced_codec_limits_bridge_export_has_exact_exclusion(self) -> None:
        for export_name in ("org.kmipkit.NativeExtensionRegistry.codecLimitsCreate",):
            with self.subTest(export_name=export_name):
                exclusion = self.exclusions[(PACKAGE, export_name)]
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("unreferenced JNI handle factory", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_timestamp_sequence_jni_bridge_is_excluded_under_approved_owner(self) -> None:
        actual_exclusions = {
            export_name
            for package, export_name in self.exclusions
            if package == PACKAGE and export_name in EXPECTED_TIMESTAMP_SEQUENCE_BRIDGE_EXPORTS
        }
        self.assertEqual(EXPECTED_TIMESTAMP_SEQUENCE_BRIDGE_EXPORTS, actual_exclusions)

        for export_name in sorted(EXPECTED_TIMESTAMP_SEQUENCE_BRIDGE_EXPORTS):
            with self.subTest(export_name=export_name):
                exclusion = self.exclusions[(PACKAGE, export_name)]
                self.assertEqual(PACKAGE, exclusion["packageId"])
                self.assertEqual(export_name, exclusion["exportName"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("timestamp-sequence JNI plumbing", exclusion["reason"])
                self.assertIn("KMIPKIT-0008-FR-007/010", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn("HashedPasswordTimestampSequence", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_ttlv_path_jni_bridge_is_excluded_under_approved_extension_owner(self) -> None:
        actual_exclusions = {
            export_name
            for package, export_name in self.exclusions
            if package == PACKAGE and export_name in EXPECTED_TTLV_PATH_BRIDGE_EXPORTS
        }
        self.assertEqual(EXPECTED_TTLV_PATH_BRIDGE_EXPORTS, actual_exclusions)

        for export_name in sorted(EXPECTED_TTLV_PATH_BRIDGE_EXPORTS):
            with self.subTest(export_name=export_name):
                exclusion = self.exclusions[(PACKAGE, export_name)]
                self.assertEqual(PACKAGE, exclusion["packageId"])
                self.assertEqual(export_name, exclusion["exportName"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("JNI bridge", exclusion["reason"])
                self.assertIn("TtlvPath facade", exclusion["reason"])
                self.assertIn("KMIPKIT-0012-FR-002/006/010/012", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_live_vendor_extension_bridge_exports_have_exact_owner_exclusions(self) -> None:
        expected_exports = frozenset(
            f"org.kmipkit.NativeExtensionRegistry.{member}"
            for member in EXPECTED_VENDOR_EXTENSION_BRIDGE_MEMBERS
        )
        self.assertEqual(60, len(expected_exports))
        actual_exclusions = {
            name
            for package, name in self.exclusions
            if package == PACKAGE and name in expected_exports
        }
        self.assertEqual(expected_exports, actual_exclusions)

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "vendor_extension_registry"
        )
        self.assertEqual("KMIPKIT-0012", capability["ownerSpecId"])
        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual("four-language", capability["parityScope"])
        self.assertEqual({"rust", "c", "java", "python"}, set(capability["mappings"]))

        java_sources = [
            path
            for path in (ROOT / "bindings" / "java" / "src" / "main" / "java").rglob("*.java")
            if path.name != "NativeExtensionRegistry.java"
        ]
        source_text = "\n".join(path.read_text(encoding="utf-8") for path in java_sources)
        for export_name in sorted(expected_exports):
            with self.subTest(export_name=export_name):
                member = export_name.rsplit(".", maxsplit=1)[-1]
                exclusion = self.exclusions[(PACKAGE, export_name)]
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("internal vendor-extension JNI plumbing", exclusion["reason"])
                self.assertIn("KMIPKIT-0012-FR-010", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn(f"NativeExtensionRegistry.{member}", source_text)
                self.assertNotIn((PACKAGE, export_name), self.declarations)

    def test_non_facade_vendor_extension_bridge_exports_have_exact_exclusions(self) -> None:
        prefix = "org.kmipkit.NativeExtensionRegistry."
        actual_blocked_members = {
            name.removeprefix(prefix)
            for package, name in self.declarations
            if package == PACKAGE
            and name.startswith(prefix)
            and self.declarations[(package, name)]["declarationState"] == "blocked"
            and self.declarations[(package, name)].get("candidateSpecId") == "KMIPKIT-0043"
        }
        self.assertEqual(frozenset(), actual_blocked_members)

        java_sources = [
            path
            for path in (ROOT / "bindings" / "java" / "src" / "main" / "java").rglob("*.java")
            if path.name != "NativeExtensionRegistry.java"
        ]
        source_text = "\n".join(path.read_text(encoding="utf-8") for path in java_sources)

        actual_exclusions = {
            name.removeprefix(prefix)
            for package, name in self.exclusions
            if package == PACKAGE
            and name.startswith(prefix)
            and name.removeprefix(prefix)
            in EXPECTED_NON_FACADE_VENDOR_EXTENSION_BRIDGE_MEMBERS
        }
        self.assertEqual(
            EXPECTED_NON_FACADE_VENDOR_EXTENSION_BRIDGE_MEMBERS,
            actual_exclusions,
        )
        for member in sorted(EXPECTED_NON_FACADE_VENDOR_EXTENSION_BRIDGE_MEMBERS):
            export_name = prefix + member
            exclusion = self.exclusions[(PACKAGE, export_name)]
            with self.subTest(export_name=export_name):
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("internal vendor-extension JNI plumbing", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertNotIn((PACKAGE, export_name), self.declarations)
                if member == "codecLimitsCreate":
                    self.assertNotIn(f"NativeExtensionRegistry.{member}", source_text)
                else:
                    self.assertIn(f"NativeExtensionRegistry.{member}", source_text)
        bridge_type = self.exclusions[(PACKAGE, "org.kmipkit.NativeExtensionRegistry")]
        self.assertEqual(OWNER_SPEC, bridge_type["approvalSource"]["path"])
        self.assertEqual("Approved", bridge_type["approvalSource"]["status"])
        self.assertIn("internal JNI bridge", bridge_type["reason"])
        self.assertIn("KMIPKIT-0043-FR-008", bridge_type["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", bridge_type["evidence"])


if __name__ == "__main__":
    unittest.main()
