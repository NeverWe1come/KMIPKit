"""Keep FFI-only internal fallback wrappers out of product error parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
C_HEADER = ROOT / "bindings" / "c" / "include" / "kmipkit.h"
ARCHITECTURE = "docs/architecture/ffi-and-bindings.md"
OWNER_SPEC = "specs/043-public-api-bindings/spec.md"

EXCLUDED_EXPORTS = {
    ("c-kmipkit", "KMIPKIT_ERROR_INTERNAL"),
    ("java-kmipkit", "org.kmipkit.KmipException"),
    ("java-kmipkit", "org.kmipkit.KmipException.<init>"),
    ("python-kmipkit", "kmipkit.errors.KmipKitError"),
    ("python-kmipkit", "kmipkit.errors.KmipKitError.category"),
}


class T102BindingInternalErrorExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["approvedExclusions"]
        }

    def test_generic_internal_fallback_exports_have_specific_exclusions(self) -> None:
        for key in sorted(EXCLUDED_EXPORTS):
            with self.subTest(export=key[1]):
                self.assertFalse(key in self.declarations, f"candidate declaration remains: {key}")
                row = self.exclusions[key]
                self.assertEqual(OWNER_SPEC, row["approvalSource"]["path"])
                self.assertEqual("Approved", row["approvalSource"]["status"])
                self.assertIn("generic FFI", row["reason"])
                self.assertIn("no equivalent public Rust error category", row["reason"])
                self.assertIn(ARCHITECTURE, row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])

    def test_internal_status_remains_a_blocked_mapping_but_keeps_stable_c_code(self) -> None:
        internal = next(
            row for row in self.manifest["errorCategories"] if row["id"] == "internal"
        )
        self.assertEqual("Other", internal["rustVariant"])
        self.assertEqual("KMIPKIT_ERROR_INTERNAL", internal["cCode"])
        self.assertEqual(7, internal["cValue"])
        self.assertTrue(internal["stableCHeaderCode"])
        self.assertEqual("blocked", internal["mappingState"])
        header = C_HEADER.read_text(encoding="utf-8")
        self.assertIn("#define KMIPKIT_ERROR_INTERNAL ((int32_t)7)", header)

    def test_language_specific_error_helpers_and_constructor_are_exact_exclusions(self) -> None:
        expected = {
            ("python-kmipkit", "kmipkit.errors.raise_for_status"),
            ("java-kmipkit", "org.kmipkit.ResourceLimitException.<init>"),
        }
        missing = expected - self.exclusions.keys()
        self.assertFalse(missing, f"expected exact exclusions are missing: {sorted(missing)}")
        if missing:
            return
        for key in sorted(expected):
            with self.subTest(export=key[1]):
                self.assertFalse(key in self.declarations, f"blocked declaration remains: {key}")
                exclusion = self.exclusions[key]
                self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
                self.assertEqual(OWNER_SPEC, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

        helper = self.exclusions[("python-kmipkit", "kmipkit.errors.raise_for_status")]
        self.assertIn("binding-only adapter plumbing", helper["reason"])
        self.assertIn("test_internal_status_raises_redacted_generic_error", helper["evidence"])
        for requirement_id in (
            "KMIPKIT-0043-FR-009",
            "KMIPKIT-0043-FR-013",
        ):
            self.assertIn(requirement_id, helper["evidence"])

        constructor = self.exclusions[("java-kmipkit", "org.kmipkit.ResourceLimitException.<init>")]
        self.assertIn("caller-supplied String message", constructor["reason"])
        self.assertIn("fieldless RuntimeMetadataError::ResourceLimit", constructor["reason"])
        for requirement_id in (
            "KMIPKIT-0043-FR-007",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-013",
        ):
            self.assertIn(requirement_id, constructor["evidence"])

        for key in (
            ("java-kmipkit", "org.kmipkit.ResourceLimitException"),
            ("python-kmipkit", "kmipkit.errors.ResourceLimitError"),
        ):
            with self.subTest(mapped_error_type=key[1]):
                declaration = self.declarations[key]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("ffi_runtime_metadata", declaration["capabilityId"])


if __name__ == "__main__":
    unittest.main()
