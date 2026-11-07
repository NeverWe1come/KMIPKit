"""Pin the exact T102 mapping for KMIPKit-local profile diagnostics.

KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1 provide profile
context only. ``ProfileValidationErrorCode`` is a KMIPKit-defined diagnostic
enum, not an OASIS error code. Each Rust enum variant maps by its ``as_str``
output to the stable string exposed by the C, Java, and Python error details;
the managed-language APIs do not construct separate code enums.
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
TYPE_PATHS = {
    "rust-kmipkit": (
        "kmipkit::ProfileValidationErrorCode",
        "kmipkit::protocol::ProfileValidationErrorCode",
    ),
    "rust-kmipkit-client": ("kmipkit_client::ProfileValidationErrorCode",),
    "rust-kmipkit-protocol": ("kmipkit_protocol::ProfileValidationErrorCode",),
}
VARIANT_CODES = {
    "EmptySelection": "empty_selection",
    "DuplicateProfile": "duplicate_profile",
    "UnknownProfile": "unknown_profile",
    "ProfileUnavailable": "profile_unavailable",
    "ConditionalProfileUnsatisfied": "conditional_profile_unsatisfied",
    "DependencyCycle": "dependency_cycle",
    "MissingRequirementMetadata": "missing_requirement_metadata",
    "UnresolvedRequirementScope": "unresolved_requirement_scope",
    "MissingRuleMapping": "missing_rule_mapping",
    "IncompleteRuleMapping": "incomplete_rule_mapping",
    "RuleMappingMismatch": "rule_mapping_mismatch",
    "MissingCategoryTest": "missing_category_test",
    "ConflictingConstraints": "conflicting_constraints",
    "RequestRuleRejected": "request_rule_rejected",
    "ResponseRuleRejected": "response_rule_rejected",
    "TransportConstraintRejected": "transport_constraint_rejected",
    "ProfileStateContextRequired": "profile_state_context_required",
    "ProfileStateBindingMismatch": "profile_state_binding_mismatch",
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
            (package, f"{type_path}{suffix}")
            for package, type_paths in TYPE_PATHS.items()
            for type_path in type_paths
            for suffix in (
                "",
                "::as_str",
                *(f"::{variant}" for variant in VARIANT_CODES),
            )
        }
        self.assertEqual(expected, set(rows))
        self.assertEqual(80, len(rows))
        self.assertEqual(18, len(VARIANT_CODES))

        declarations = self.manifest["publicDeclarations"]
        for (package, export_name), row in rows.items():
            crate = PACKAGES[package]
            type_path = next(
                candidate
                for candidate in TYPE_PATHS[package]
                if export_name.startswith(candidate)
            )
            suffix = export_name.removeprefix(type_path)
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
        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual(
            {export_name for _, export_name in expected},
            {
                export_name
                for export_name in rust_mappings
                if "ProfileValidationErrorCode" in export_name
            },
        )
        for language, (_, target) in LANGUAGE_TARGETS.items():
            self.assertIn(target, capability["mappings"][language])

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
