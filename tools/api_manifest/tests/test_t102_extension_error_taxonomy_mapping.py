"""Pin four-language mappings for approved extension error categories."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
C_HEADER = ROOT / "bindings" / "c" / "include" / "kmipkit.h"
OWNER_SPEC = "specs/012-vendor-extension-registry/spec.md"

CATEGORIES = {
    "compatibility_mismatch": {
        "rust": "CompatibilityMismatch",
        "c": "KMIPKIT_ERROR_COMPATIBILITY_MISMATCH",
        "cValue": 1,
        "java": "org.kmipkit.extensions.ExtensionCompatibilityException",
        "python": "kmipkit.errors.CompatibilityMismatchError",
    },
    "duplicate_key": {
        "rust": "DuplicateKey",
        "c": "KMIPKIT_ERROR_DUPLICATE_KEY",
        "cValue": 2,
        "java": "org.kmipkit.extensions.DuplicateExtensionKeyException",
        "python": "kmipkit.errors.DuplicateKeyError",
    },
    "invalid_identity": {
        "rust": "InvalidIdentity",
        "c": "KMIPKIT_ERROR_INVALID_IDENTITY",
        "cValue": 3,
        "java": "org.kmipkit.extensions.InvalidExtensionIdentityException",
        "python": "kmipkit.errors.InvalidIdentityError",
    },
    "invalid_schema": {
        "rust": "InvalidSchema",
        "c": "KMIPKIT_ERROR_INVALID_SCHEMA",
        "cValue": 5,
        "java": "org.kmipkit.extensions.InvalidExtensionSchemaException",
        "python": "kmipkit.errors.InvalidExtensionSchemaError",
    },
}


class T102ExtensionErrorTaxonomyMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.error_categories = {
            row["id"]: row for row in cls.manifest["errorCategories"]
        }
        cls.header = C_HEADER.read_text(encoding="utf-8")

    def test_extension_error_categories_are_stable_in_all_adapters(self) -> None:
        for category_id, expected in CATEGORIES.items():
            with self.subTest(category=category_id):
                category = self.error_categories[category_id]
                self.assertEqual(expected["rust"], category["rustVariant"])
                self.assertEqual(expected["c"], category["cCode"])
                self.assertEqual(expected["cValue"], category["cValue"])
                self.assertEqual(expected["java"], category["javaException"])
                self.assertEqual(expected["python"], category["pythonException"])
                self.assertEqual("ready", category["mappingState"])
                self.assertNotIn("blockedReason", category)
                self.assertIn(
                    f'#define {expected["c"]} ((int32_t){expected["cValue"]})',
                    self.header,
                )
                c_row = self.declarations[("c-kmipkit", expected["c"])]
                self.assertEqual("declared", c_row["declarationState"])
                self.assertNotIn("candidateSpecId", c_row)
                self.assertEqual("vendor_extension_registry", c_row["capabilityId"])
                self.assertIn("KMIPKIT-0012-FR-010", c_row["requirementIds"])
                self.assertEqual(
                    "rust-kmipkit-protocol",
                    c_row["languageMappings"]["rust"]["packageId"],
                )

    def test_observed_exception_exports_have_approved_owner_and_parity(self) -> None:
        requirements = {
            "KMIPKIT-0012-FR-010",
            "KMIPKIT-0012-FR-011",
            "KMIPKIT-0043-FR-021",
        }
        for category_id, expected in CATEGORIES.items():
            with self.subTest(category=category_id):
                mappings = {
                    "rust": {
                        "packageId": "rust-kmipkit-protocol",
                        "name": f"kmipkit_protocol::ProtocolErrorKind::{expected['rust']}",
                    },
                    "c": {
                        "packageId": "c-kmipkit",
                        "name": expected["c"],
                    },
                    "java": {
                        "packageId": "java-kmipkit",
                        "name": expected["java"],
                    },
                    "python": {
                        "packageId": "python-kmipkit",
                        "name": expected["python"],
                    },
                }
                exports = (
                    ("java-kmipkit", expected["java"]),
                    ("java-kmipkit", f"{expected['java']}.<init>"),
                    ("python-kmipkit", expected["python"]),
                    ("python-kmipkit", f"{expected['python']}.category"),
                )
                for key in exports:
                    with self.subTest(export=key[1]):
                        row = self.declarations[key]
                        self.assertEqual("declared", row["declarationState"])
                        self.assertNotIn("candidateSpecId", row)
                        self.assertEqual("vendor_extension_registry", row["capabilityId"])
                        self.assertTrue(requirements <= set(row["requirementIds"]))
                        self.assertEqual(
                            OWNER_SPEC, row["approvalSource"]["path"]
                        )
                        self.assertEqual("Approved", row["approvalSource"]["status"])
                        self.assertEqual(mappings, row["languageMappings"])

                if category_id == "duplicate_key":
                    duplicate = self.declarations[("python-kmipkit", expected["python"])]
                    self.assertIn("KMIPKIT-0012-FR-003", duplicate["requirementIds"])

    def test_rust_error_variants_are_exactly_mapped_from_both_public_crates(self) -> None:
        for category_id, expected in CATEGORIES.items():
            mappings = {
                "rust": {
                    "packageId": "rust-kmipkit-protocol",
                    "name": f"kmipkit_protocol::ProtocolErrorKind::{expected['rust']}",
                },
                "c": {"packageId": "c-kmipkit", "name": expected["c"]},
                "java": {"packageId": "java-kmipkit", "name": expected["java"]},
                "python": {"packageId": "python-kmipkit", "name": expected["python"]},
            }
            for package_id, prefix in (
                ("rust-kmipkit-protocol", "kmipkit_protocol::ProtocolErrorKind"),
                ("rust-kmipkit", "kmipkit::ProtocolErrorKind"),
            ):
                key = (package_id, f"{prefix}::{expected['rust']}")
                with self.subTest(category=category_id, rust_export=key[1]):
                    row = self.declarations[key]
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual("vendor_extension_registry", row["capabilityId"])
                    self.assertEqual(mappings, row["languageMappings"])

    def test_error_kind_container_has_specific_owner_backed_exclusions(self) -> None:
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        for package_id, export_name in (
            ("rust-kmipkit-protocol", "kmipkit_protocol::ProtocolErrorKind"),
            ("rust-kmipkit", "kmipkit::ProtocolErrorKind"),
        ):
            with self.subTest(export=export_name):
                row = exclusions[(package_id, export_name)]
                self.assertIn("no distinct cross-language enum type", row["reason"])
                self.assertIn("each named error variant is mapped", row["evidence"])
                self.assertEqual("specs/003-core-types-errors/spec.md", row["approvalSource"]["path"])
                self.assertEqual("Approved", row["approvalSource"]["status"])
                self.assertIn("KMIPKIT-0012-FR-010/011", row["evidence"])

    def test_duplicate_key_documentation_covers_both_duplicate_rules(self) -> None:
        errors_source = (ROOT / "bindings/python/src/kmipkit/errors.py").read_text(
            encoding="utf-8"
        )
        duplicate_doc = errors_source.split("class DuplicateKeyError", 1)[1].split(
            "class ", 1
        )[0]
        self.assertIn("discriminator", duplicate_doc.lower())


if __name__ == "__main__":
    unittest.main()
