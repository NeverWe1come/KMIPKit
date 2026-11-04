"""Pin exact Java/Python implementation declarations outside the typed facade."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/043-public-api-bindings/spec.md"
PACKAGE = "java-kmipkit"

EXPECTED_GROUPS = {
    "generated metadata": {
        "requirementIds": {"KMIPKIT-0043-FR-015", "KMIPKIT-0043-FR-021"},
        "reason": "generated public API metadata",
        "exports": {
            "org.kmipkit.generated.ExtensionRegistryApi",
            "org.kmipkit.generated.ExtensionRegistryApi.FORMAT_VERSION",
            "org.kmipkit.generated.ExtensionRegistryApi.FUNCTION_METADATA_JSON",
            "org.kmipkit.generated.PublicApi",
            "org.kmipkit.generated.PublicApi.DECLARATIONS_JSON",
            "org.kmipkit.generated.PublicApi.FORMAT_VERSION",
        },
    },
    "native handle lifecycle": {
        "requirementIds": {"KMIPKIT-0043-FR-008", "KMIPKIT-0043-FR-021"},
        "reason": "internal native-handle lifecycle plumbing",
        "exports": {
            "org.kmipkit.internal.NativeHandle",
            "org.kmipkit.internal.NativeHandle.close",
            "org.kmipkit.internal.NativeHandle.get",
            "org.kmipkit.internal.NativeHandle.owned",
            "org.kmipkit.internal.NativeHandle.transfer",
            "org.kmipkit.internal.NativeHandle.transferAll",
            "org.kmipkit.internal.NativeHandle.withNativeHandles",
            "org.kmipkit.internal.NativeHandle.withValue",
            "org.kmipkit.internal.NativeHandle.withValues",
            "org.kmipkit.ttlv.TtlvItem.nativeHandleOwner",
            "org.kmipkit.ttlv.TtlvStructure.nativeHandleOwner",
        },
    },
    "Java enum boilerplate": {
        "requirementIds": {"KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"},
        "reason": "compiler-generated enum convenience methods",
        "exports": {
            "org.kmipkit.extensions.OpaqueObjectContext.valueOf",
            "org.kmipkit.extensions.OpaqueObjectContext.values",
        },
    },
}

PYTHON_NAMESPACE = "kmipkit"


class BindingInternalDeclarationExclusionTests(unittest.TestCase):
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

    def _assert_exact_exclusion_group(self, group_name: str) -> None:
        group = EXPECTED_GROUPS[group_name]
        expected = {
            (PACKAGE, export_name) for export_name in group["exports"]
        }
        actual = {
            key for key in self.exclusions if key[0] == PACKAGE and key[1] in group["exports"]
        }
        self.assertEqual(expected, actual)

        for key in sorted(expected):
            with self.subTest(group=group_name, export_name=key[1]):
                exclusion = self.exclusions[key]
                self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn(group["reason"], exclusion["reason"])
                self.assertEqual(
                    group["requirementIds"],
                    set(exclusion["evidence"].replace(".", "").split("; "))
                    & group["requirementIds"],
                )
                self.assertNotIn(key, self.declarations)

    def test_generated_metadata_declarations_are_exact_exclusions(self) -> None:
        self._assert_exact_exclusion_group("generated metadata")

    def test_native_handle_lifecycle_declarations_are_exact_exclusions(self) -> None:
        self._assert_exact_exclusion_group("native handle lifecycle")

    def test_java_enum_convenience_methods_are_exact_exclusions(self) -> None:
        self._assert_exact_exclusion_group("Java enum boilerplate")

    def test_python_root_package_is_only_an_inventoried_namespace(self) -> None:
        key = ("python-kmipkit", PYTHON_NAMESPACE)
        exclusion = self.exclusions[key]
        self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
        self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
        self.assertEqual("Approved", exclusion["approvalSource"]["status"])
        self.assertIn("package namespace", exclusion["reason"])
        self.assertIn("KMIPKIT-0043-FR-020", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
        self.assertNotIn(key, self.declarations)


if __name__ == "__main__":
    unittest.main()
