"""Reconcile Python native-handle convenience exports for T102."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PYTHON_TYPES = (
    "kmipkit.extensions.ClientBatchItem",
    "kmipkit.extensions.ClientConfiguration",
    "kmipkit.extensions.ClientExtensionRegistry",
    "kmipkit.extensions.ClientRequestMessageExtension",
    "kmipkit.extensions.Compatibility",
    "kmipkit.extensions.Discriminator",
    "kmipkit.extensions.ExtensionChildRule",
    "kmipkit.extensions.ExtensionDefinition",
    "kmipkit.extensions.ExtensionIdentity",
    "kmipkit.extensions.ExtensionInformation",
    "kmipkit.extensions.ExtensionOrderConstraint",
    "kmipkit.extensions.ExtensionRecognition",
    "kmipkit.extensions.ExtensionSchema",
    "kmipkit.extensions.RegisteredExtensionValue",
    "kmipkit.extensions.RegisterRequest",
    "kmipkit.extensions.TtlvPath",
    "kmipkit.extensions.ValidatedExtensionValue",
    "kmipkit.ttlv.CodecLimits",
    "kmipkit.ttlv.RawTag",
    "kmipkit.ttlv.Structure",
    "kmipkit.ttlv.Tag",
    "kmipkit.ttlv.TtlvItem",
    "kmipkit.ttlv.TtlvItemView",
    "kmipkit.ttlv.TtlvStructureView",
    "kmipkit.ttlv.TtlvValueView",
    "kmipkit.ttlv.Value",
)
APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
LANGUAGES = {"rust", "c", "java", "python"}


class T102PythonNativeHandleLifecycleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = cls.manifest["publicDeclarations"]
        cls.by_package_and_name = {
            (row["packageId"], row["exportName"]): row for row in cls.declarations
        }
        cls.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["approvedExclusions"]
        }

    def test_handle_exit_maps_to_the_existing_four_language_release(self) -> None:
        expected_names = {f"{python_type}.__exit__" for python_type in PYTHON_TYPES}
        rows = {
            row["exportName"]: row
            for row in self.declarations
            if row["packageId"] == "python-kmipkit" and row["exportName"] in expected_names
        }
        self.assertEqual(expected_names, set(rows))
        self.assertEqual(
            [],
            [name for name, row in rows.items() if row["declarationState"] == "blocked"],
        )

        for python_type in PYTHON_TYPES:
            close = self.by_package_and_name[("python-kmipkit", f"{python_type}.close")]
            exit_name = f"{python_type}.__exit__"
            row = rows[exit_name]
            self.assertNotIn("candidateSpecId", row)
            self.assertNotIn("blockedReason", row)
            self.assertEqual(close["capabilityId"], row["capabilityId"])
            self.assertEqual("four-language", row["parityScope"])
            self.assertEqual(LANGUAGES, set(row["languageMappings"]))
            for language in ("rust", "c", "java"):
                self.assertEqual(
                    close["languageMappings"][language],
                    row["languageMappings"][language],
                )
            self.assertEqual(exit_name, row["languageMappings"]["python"]["name"])
            self.assertEqual(close["approvalSource"], row["approvalSource"])
            self.assertTrue(
                {
                    "KMIPKIT-0043-FR-005",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                }
                <= set(row["requirementIds"])
            )

    def test_enter_and_closed_are_exact_python_adapter_exclusions(self) -> None:
        expected_names = {
            f"{python_type}.{member}"
            for python_type in PYTHON_TYPES
            for member in ("__enter__", "closed")
        }
        public_names = {
            row["exportName"]
            for row in self.declarations
            if row["packageId"] == "python-kmipkit"
        }
        self.assertEqual(set(), expected_names & public_names)
        self.assertEqual(
            expected_names,
            {
                name
                for package, name in self.exclusions
                if package == "python-kmipkit" and name in expected_names
            },
        )

        for python_type in PYTHON_TYPES:
            for member in ("__enter__", "closed"):
                name = f"{python_type}.{member}"
                exclusion = self.exclusions[("python-kmipkit", name)]
                self.assertEqual(APPROVAL, exclusion["approvalSource"])
                self.assertIn("python-only native-handle convenience", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-010", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn("bindings/python/src/kmipkit/_handles.py", exclusion["evidence"])


if __name__ == "__main__":
    unittest.main()
