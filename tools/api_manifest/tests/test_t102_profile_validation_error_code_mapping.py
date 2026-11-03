"""Pin the exact T102 mapping for KMIPKit-local profile diagnostics.

KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1 provide profile
context only. ``ProfileValidationErrorCode`` is a KMIPKit-defined diagnostic
enum, not an OASIS error code.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PACKAGES = {
    "rust-kmipkit": "kmipkit",
    "rust-kmipkit-client": "kmipkit_client",
    "rust-kmipkit-protocol": "kmipkit_protocol",
}
VARIANTS = {
    "ConditionalProfileUnsatisfied",
    "ConflictingConstraints",
    "DependencyCycle",
    "DuplicateProfile",
    "EmptySelection",
    "IncompleteRuleMapping",
    "MissingCategoryTest",
    "MissingRequirementMetadata",
    "MissingRuleMapping",
    "ProfileStateBindingMismatch",
    "ProfileStateContextRequired",
    "ProfileUnavailable",
    "RequestRuleRejected",
    "ResponseRuleRejected",
    "RuleMappingMismatch",
    "TransportConstraintRejected",
    "UnknownProfile",
    "UnresolvedRequirementScope",
}
LANGUAGE_TARGETS = {
    "c": (
        "c-kmipkit",
        "kmipkit_profile_validation_error_view_v1_t.error_code_data",
    ),
    "java": (
        "java-kmipkit",
        "org.kmipkit.ProfileValidationDetails.errorCode",
    ),
    "python": (
        "python-kmipkit",
        "kmipkit.errors.ProfileValidationDetails.error_code",
    ),
}


class T102ProfileValidationErrorCodeMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_exact_code_declarations_map_to_existing_four_language_diagnostic(self) -> None:
        rows = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
            if "ProfileValidationErrorCode" in row["exportName"]
        }
        expected = {
            (package, f"{crate}::ProfileValidationErrorCode{suffix}")
            for package, crate in PACKAGES.items()
            for suffix in (
                "",
                "::as_str",
                *(f"::{variant}" for variant in VARIANTS),
            )
        }
        self.assertEqual(expected, set(rows))
        self.assertEqual(60, len(rows))

        declarations = self.manifest["publicDeclarations"]
        for (package, export_name), row in rows.items():
            crate = PACKAGES[package]
            suffix = export_name.removeprefix(f"{crate}::ProfileValidationErrorCode")
            expected_kind = "type" if not suffix else (
                "associated-item" if suffix == "::as_str" else "variant"
            )
            with self.subTest(export_name=export_name):
                self.assertEqual("client_profile_validation", row["capabilityId"])
                self.assertEqual(expected_kind, row["kind"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)
                self.assertEqual(
                    {
                        "rust": {"name": export_name, "packageId": package},
                        **{
                            language: {"name": target, "packageId": package_id}
                            for language, (package_id, target) in LANGUAGE_TARGETS.items()
                        },
                    },
                    row["languageMappings"],
                )

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        rust_mappings = set(capability["mappings"]["rust"])
        self.assertEqual(
            {export_name for _, export_name in expected},
            {
                export_name
                for export_name in rust_mappings
                if "ProfileValidationErrorCode" in export_name
            },
        )

        for package, crate in PACKAGES.items():
            code_accessor = next(
                row
                for row in declarations
                if row["packageId"] == package
                and row["exportName"] == f"{crate}::ProfileValidationError::code"
            )
            for (row_package, _), row in rows.items():
                if row_package == package:
                    self.assertEqual(row["requirementIds"], code_accessor["requirementIds"])
                    self.assertEqual(row["approvalSource"], code_accessor["approvalSource"])


if __name__ == "__main__":
    unittest.main()
