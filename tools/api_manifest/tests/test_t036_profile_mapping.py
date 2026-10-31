"""Manifest contract for the accepted T036 profile-validation mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PROFILE_CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"
CONFORMANCE_DOC_PATH = ROOT / "docs" / "compliance" / "conformance.md"
USER_GUIDE_PATHS = (
    ROOT / "docs" / "user-guide" / "en" / "profile-conformance.md",
    ROOT / "docs" / "user-guide" / "es" / "profile-conformance.md",
)

RUST_MAPPINGS = {
    "kmipkit_client::ProfileValidationSelection",
    "kmipkit_client::ProfileValidationContext",
    "kmipkit_client::ProfileObjectState",
    "kmipkit_client::Client::execute_with_profiles",
    "kmipkit_client::Client::execute_with_profiles_and_options",
    "kmipkit_client::Client::execute_with_profiles_and_context",
    "kmipkit_client::Client::execute_with_profiles_and_context_and_options",
    "kmipkit::Client::execute_with_profiles",
    "kmipkit::Client::execute_with_profiles_and_options",
    "kmipkit::Client::execute_with_profiles_and_context",
    "kmipkit::Client::execute_with_profiles_and_context_and_options",
    "kmipkit_client::ProfileValidationSelection::new",
    "kmipkit_client::ProfileValidationSelection::profile_ids",
    "kmipkit_client::ProfileValidationSelection::validate",
    "kmipkit_client::ProfileValidationContext::new",
    "kmipkit_client::ProfileValidationContext::with_object_state",
    "kmipkit_client::ProfileObjectState::new",
    "kmipkit_client::ProfileObjectState::with_original_creation_date_present",
    "kmipkit_client::ProfileObjectState::with_usage_limits_present",
    "kmipkit_client::ProfileObjectState::with_protection_use_started",
    "kmipkit_client::ProfileObjectState::with_get_usage_allocation_performed",
    "kmipkit_client::ProfileObjectState::with_get_usage_allocation_changed_count",
    "kmipkit_client::ProfileObjectState::with_usage_limits_total",
    "kmipkit_client::ProfileObjectState::with_usage_limits_count",
    "kmipkit::ClientError::ProfileValidation",
    "kmipkit::ClientError::ProfileValidation::error",
    "kmipkit::ClientError::profile_validation_error",
    "kmipkit::ProfileValidationError",
    "kmipkit::ProfileValidationError::batch_item_index",
    "kmipkit::ProfileValidationError::code",
    "kmipkit::ProfileValidationError::profile_id",
    "kmipkit::ProfileValidationError::requirement_id",
    "kmipkit::ProfileValidationError::source_clause_id",
    "kmipkit::ProfileValidationError::source_gap_id",
    "kmipkit::ProfileValidationError::source_gap_locator",
    "kmipkit_client::ClientError::ProfileValidation",
    "kmipkit_client::ClientError::ProfileValidation::error",
    "kmipkit_client::ClientError::profile_validation_error",
    "kmipkit_client::ProfileValidationError",
    "kmipkit_client::ProfileValidationError::batch_item_index",
    "kmipkit_client::ProfileValidationError::code",
    "kmipkit_client::ProfileValidationError::profile_id",
    "kmipkit_client::ProfileValidationError::requirement_id",
    "kmipkit_client::ProfileValidationError::source_clause_id",
    "kmipkit_client::ProfileValidationError::source_gap_id",
    "kmipkit_client::ProfileValidationError::source_gap_locator",
    "kmipkit_protocol::ProfileValidationError",
    "kmipkit_protocol::ProfileValidationError::batch_item_index",
    "kmipkit_protocol::ProfileValidationError::code",
    "kmipkit_protocol::ProfileValidationError::profile_id",
    "kmipkit_protocol::ProfileValidationError::requirement_id",
    "kmipkit_protocol::ProfileValidationError::source_clause_id",
    "kmipkit_protocol::ProfileValidationError::source_gap_id",
    "kmipkit_protocol::ProfileValidationError::source_gap_locator",
}

C_MAPPINGS = {
    "kmipkit_profile_validation_selection_v1_t",
    "kmipkit_profile_object_state_v1_t",
    "kmipkit_profile_validation_context_v1_t",
    "kmipkit_client_execute_with_profiles",
    "kmipkit_client_execute_with_profiles_and_options_v1",
    "kmipkit_client_execute_with_profiles_and_context",
    "kmipkit_client_execute_with_profiles_and_context_and_options_v1",
}

JAVA_MAPPINGS = {
    "org.kmipkit.extensions.Client.executeWithProfiles",
    "org.kmipkit.extensions.Client.executeWithProfilesAndOptions",
    "org.kmipkit.extensions.Client.executeWithProfilesAndContext",
    "org.kmipkit.extensions.Client.executeWithProfilesAndContextAndOptions",
    "org.kmipkit.extensions.ProfileValidationSelection",
    "org.kmipkit.extensions.ProfileValidationContext",
    "org.kmipkit.extensions.ProfileObjectState",
    "org.kmipkit.extensions.ProfileValidationSelection.profileIds",
    "org.kmipkit.extensions.ProfileValidationContext.withObjectState",
    "org.kmipkit.extensions.ProfileObjectState.withOriginalCreationDatePresent",
    "org.kmipkit.extensions.ProfileObjectState.withUsageLimitsPresent",
    "org.kmipkit.extensions.ProfileObjectState.withProtectionUseStarted",
    "org.kmipkit.extensions.ProfileObjectState.withGetUsageAllocationPerformed",
    "org.kmipkit.extensions.ProfileObjectState.withGetUsageAllocationChangedCount",
    "org.kmipkit.extensions.ProfileObjectState.withUsageLimitsTotal",
    "org.kmipkit.extensions.ProfileObjectState.withUsageLimitsCount",
}

PYTHON_MAPPINGS = {
    "kmipkit.client.Client.execute_with_profiles",
    "kmipkit.client.Client.execute_with_profiles_and_options",
    "kmipkit.client.Client.execute_with_profiles_and_context",
    "kmipkit.client.Client.execute_with_profiles_and_context_and_options",
    "kmipkit.client.ProfileValidationSelection",
    "kmipkit.client.ProfileValidationContext",
    "kmipkit.client.ProfileObjectState",
    "kmipkit.client.ProfileValidationSelection.profile_ids",
    "kmipkit.client.ProfileValidationContext.with_object_state",
    "kmipkit.client.ProfileObjectState.with_original_creation_date_present",
    "kmipkit.client.ProfileObjectState.with_usage_limits_present",
    "kmipkit.client.ProfileObjectState.with_protection_use_started",
    "kmipkit.client.ProfileObjectState.with_get_usage_allocation_performed",
    "kmipkit.client.ProfileObjectState.with_get_usage_allocation_changed_count",
    "kmipkit.client.ProfileObjectState.with_usage_limits_total",
    "kmipkit.client.ProfileObjectState.with_usage_limits_count",
}

JAVA_PROFILE_EXPORTS = {
    "org.kmipkit.extensions.ProfileValidationSelection.<init>",
    "org.kmipkit.extensions.ProfileValidationSelection.equals",
    "org.kmipkit.extensions.ProfileValidationSelection.hashCode",
    "org.kmipkit.extensions.ProfileValidationSelection.toString",
    "org.kmipkit.extensions.ProfileObjectState.<init>",
    "org.kmipkit.extensions.ProfileObjectState.equals",
    "org.kmipkit.extensions.ProfileObjectState.hashCode",
    "org.kmipkit.extensions.ProfileObjectState.toString",
    "org.kmipkit.extensions.ProfileObjectState.uniqueIdentifier",
    "org.kmipkit.extensions.ProfileObjectState.originalCreationDatePresent",
    "org.kmipkit.extensions.ProfileObjectState.usageLimitsPresent",
    "org.kmipkit.extensions.ProfileObjectState.protectionUseStarted",
    "org.kmipkit.extensions.ProfileObjectState.getUsageAllocationPerformed",
    "org.kmipkit.extensions.ProfileObjectState.getUsageAllocationChangedCount",
    "org.kmipkit.extensions.ProfileObjectState.usageLimitsTotal",
    "org.kmipkit.extensions.ProfileObjectState.usageLimitsCount",
    "org.kmipkit.extensions.ProfileValidationContext.<init>",
    "org.kmipkit.extensions.ProfileValidationContext.equals",
    "org.kmipkit.extensions.ProfileValidationContext.hashCode",
    "org.kmipkit.extensions.ProfileValidationContext.toString",
    "org.kmipkit.extensions.ProfileValidationContext.objectStates",
    "org.kmipkit.extensions.ProfileValidationContext.Entry",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.<init>",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.batchItemIndex",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.state",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.equals",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.hashCode",
    "org.kmipkit.extensions.ProfileValidationContext.Entry.toString",
}
JAVA_PROFILE_RECORD_METHOD_EXCLUSIONS = {
    export
    for export in JAVA_PROFILE_EXPORTS
    if export.endswith((".equals", ".hashCode", ".toString"))
}
PYTHON_PROFILE_VALUE_FIELDS = {
    "kmipkit.client.ProfileObjectState.unique_identifier",
    "kmipkit.client.ProfileObjectState.original_creation_date_present",
    "kmipkit.client.ProfileObjectState.usage_limits_present",
    "kmipkit.client.ProfileObjectState.protection_use_started",
    "kmipkit.client.ProfileObjectState.get_usage_allocation_performed",
    "kmipkit.client.ProfileObjectState.get_usage_allocation_changed_count",
    "kmipkit.client.ProfileObjectState.usage_limits_total",
    "kmipkit.client.ProfileObjectState.usage_limits_count",
    "kmipkit.client.ProfileValidationContext.object_states",
}


class T036ProfileMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.capabilities = {
            row["capabilityId"]: row for row in cls.manifest["capabilities"]
        }

    def test_profile_validation_is_declared_as_four_language_mapping_only(self) -> None:
        capability = self.capabilities.get("client_profile_validation")
        self.assertIsNotNone(capability, "accepted profile validation mapping is missing")
        assert capability is not None

        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual("four-language", capability["parityScope"])
        self.assertEqual("KMIPKIT-0010", capability["ownerSpecId"])
        self.assertEqual("Accepted", capability["approvalEvidence"]["status"])
        self.assertEqual(
            "specs/010-profile-conformance/spec.md",
            capability["approvalEvidence"]["path"],
        )
        self.assertEqual(RUST_MAPPINGS, set(capability["mappings"]["rust"]))
        self.assertTrue(C_MAPPINGS <= set(capability["mappings"]["c"]))
        selection_type = next(
            row for row in self.manifest["types"]
            if row["id"] == "ProfileValidationSelection"
        )
        self.assertEqual(
            ["struct_size", "abi_version", "profile_id_data", "profile_id_data_length",
             "profile_id_offsets", "profile_id_lengths", "profile_id_count"],
            [field["name"] for field in selection_type["c"]["fields"]],
        )
        self.assertTrue(JAVA_MAPPINGS <= set(capability["mappings"]["java"]))
        self.assertTrue(PYTHON_MAPPINGS <= set(capability["mappings"]["python"]))
        self.assertIn("KMIPKIT-0010-FR-005", capability["requirementIds"])
        self.assertIn("KMIPKIT-0010-FR-013", capability["requirementIds"])
        self.assertIn("KMIPKIT-0010-FR-019", capability["requirementIds"])
        self.assertIn("KMIPKIT-0043-FR-021", capability["requirementIds"])

    def test_profile_validation_error_details_map_to_all_four_public_facades(self) -> None:
        capability = self.capabilities["client_profile_validation"]
        expected_mappings = {
            "rust": {
                "kmipkit::ClientError::profile_validation_error",
                "kmipkit_client::ClientError::profile_validation_error",
                "kmipkit::ProfileValidationError::code",
                "kmipkit_client::ProfileValidationError::code",
                "kmipkit_protocol::ProfileValidationError::code",
                "kmipkit::ProfileValidationError::source_gap_locator",
                "kmipkit_client::ProfileValidationError::source_gap_locator",
                "kmipkit_protocol::ProfileValidationError::source_gap_locator",
                "kmipkit::ProfileValidationError::batch_item_index",
                "kmipkit_client::ProfileValidationError::batch_item_index",
                "kmipkit_protocol::ProfileValidationError::batch_item_index",
            },
            "c": {
                "kmipkit_profile_validation_error_t",
                "kmipkit_profile_validation_error_view_v1_t",
                "kmipkit_last_error_profile_validation_error_get_v1",
                "kmipkit_profile_validation_error_view_get_v1",
                "kmipkit_profile_validation_error_release",
            },
            "java": {
                "org.kmipkit.ClientOperationException.profileValidation",
                "org.kmipkit.ProfileValidationDetails",
                "org.kmipkit.ProfileValidationDetails.errorCode",
                "org.kmipkit.ProfileValidationDetails.sourceGapLocator",
                "org.kmipkit.ProfileValidationDetails.batchItemIndex",
            },
            "python": {
                "kmipkit.errors.ClientExecutionError.profile_validation",
                "kmipkit.errors.ProfileValidationDetails",
                "kmipkit.errors.ProfileValidationDetails.error_code",
                "kmipkit.errors.ProfileValidationDetails.source_gap_locator",
                "kmipkit.errors.ProfileValidationDetails.batch_item_index",
            },
        }
        for language, expected in expected_mappings.items():
            with self.subTest(language=language):
                self.assertTrue(expected <= set(capability["mappings"][language]))

        for requirement_id in (
            "KMIPKIT-0010-FR-012",
            "KMIPKIT-0010-FR-019",
            "KMIPKIT-0043-FR-007",
            "KMIPKIT-0043-FR-013",
            "KMIPKIT-0043-FR-021",
        ):
            self.assertIn(requirement_id, capability["requirementIds"])

        declaration = next(
            row for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] == "kmipkit_client::ClientError::profile_validation_error"
        )
        self.assertEqual("client_profile_validation", declaration["capabilityId"])
        self.assertEqual("declared", declaration["declarationState"])
        self.assertEqual(
            {"rust", "c", "java", "python"},
            set(declaration["languageMappings"]),
        )

    def test_profile_readiness_remains_incomplete_and_unclaimed(self) -> None:
        catalog = json.loads(PROFILE_CATALOG_PATH.read_text(encoding="utf-8"))
        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        self.assertEqual("evidence_incomplete", baseline["claim_state"])
        self.assertFalse(baseline.get("support_claim", False))

    def test_all_observed_java_profile_exports_are_mapped_or_exactly_excluded(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "java-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit"
        }
        self.assertEqual(
            JAVA_PROFILE_EXPORTS,
            (JAVA_PROFILE_EXPORTS & declarations.keys())
            | (JAVA_PROFILE_EXPORTS & exclusions.keys()),
        )
        self.assertFalse(JAVA_PROFILE_EXPORTS & (declarations.keys() & exclusions.keys()))
        for export in JAVA_PROFILE_EXPORTS - JAVA_PROFILE_RECORD_METHOD_EXCLUSIONS:
            with self.subTest(export=export):
                declaration = declarations.get(export)
                self.assertIsNotNone(declaration)
                assert declaration is not None
                self.assertEqual("client_profile_validation", declaration["capabilityId"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(declaration["languageMappings"]),
                )
                self.assertEqual(
                    "specs/010-profile-conformance/spec.md",
                    declaration["approvalSource"]["path"],
                )
        for export in JAVA_PROFILE_RECORD_METHOD_EXCLUSIONS:
            with self.subTest(exclusion=export):
                exclusion = exclusions.get(export)
                self.assertIsNotNone(exclusion)
                assert exclusion is not None
                self.assertIn("record", exclusion["reason"].lower())
                self.assertEqual(
                    "specs/010-profile-conformance/spec.md",
                    exclusion["approvalSource"]["path"],
                )

    def test_python_profile_ids_export_uses_the_scanned_field_kind(self) -> None:
        row = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit.client.ProfileValidationSelection.profile_ids"
            and row["packageId"] == "python-kmipkit"
        )
        self.assertEqual("field", row["kind"])

    def test_profile_value_fields_are_mapped_as_four_language_records(self) -> None:
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"] in PYTHON_PROFILE_VALUE_FIELDS
        }
        self.assertEqual(PYTHON_PROFILE_VALUE_FIELDS, rows.keys())
        for export, row in rows.items():
            with self.subTest(export=export):
                self.assertEqual("field", row["kind"])
                self.assertEqual("client_profile_validation", row["capabilityId"])
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                self.assertNotIn(
                    export,
                    {
                        exclusion["exportName"]
                        for exclusion in self.manifest["approvedExclusions"]
                    },
                )

    def test_overloaded_profile_value_constructors_inventory_exact_java_signatures(self) -> None:
        expected = {
            "org.kmipkit.extensions.ProfileObjectState.<init>": {
                "public org.kmipkit.extensions.ProfileObjectState(java.lang.String)",
                "public org.kmipkit.extensions.ProfileObjectState(java.lang.String, java.lang.Boolean, java.lang.Boolean, java.lang.Boolean, java.lang.Boolean, java.lang.Boolean, java.lang.Long, java.lang.Long)",
            },
            "org.kmipkit.extensions.ProfileValidationContext.<init>": {
                "public org.kmipkit.extensions.ProfileValidationContext()",
                "public org.kmipkit.extensions.ProfileValidationContext(java.util.List<org.kmipkit.extensions.ProfileValidationContext$Entry>)",
            },
        }
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] in expected
        }
        self.assertEqual(expected.keys(), declarations.keys())
        for export, signatures in expected.items():
            with self.subTest(export=export):
                self.assertEqual(sorted(signatures), declarations[export]["javaSignatures"])

    def test_incomplete_baseline_evidence_is_not_described_as_runtime_available(self) -> None:
        conformance = " ".join(
            CONFORMANCE_DOC_PATH.read_text(encoding="utf-8").split()
        )

        self.assertNotIn("explicit runtime selection is available", conformance)
        self.assertNotIn("Runtime selection being available means", conformance)
        self.assertIn("runtime profile selection for Baseline remains unavailable", conformance)
        self.assertIn("per-exchange validation fails closed before transport", conformance)
        self.assertIn("evidence_incomplete", conformance)

        english_guide = " ".join(USER_GUIDE_PATHS[0].read_text(encoding="utf-8").split())
        spanish_guide = " ".join(USER_GUIDE_PATHS[1].read_text(encoding="utf-8").split())
        self.assertNotIn("Baseline Client runtime selection is available", english_guide)
        self.assertNotIn("Baseline Client currently meets this runtime gate", english_guide)
        self.assertIn(
            "Baseline Client runtime selection remains unavailable while evidence is incomplete",
            english_guide,
        )
        self.assertNotIn("Baseline Client está disponible", spanish_guide)
        self.assertNotIn("validación runtime está disponible", spanish_guide)
        self.assertNotIn("Baseline Client ya cumple este control runtime", spanish_guide)
        self.assertIn(
            "La selección runtime de Baseline Client no está disponible mientras su evidencia siga incompleta",
            spanish_guide,
        )


if __name__ == "__main__":
    unittest.main()
