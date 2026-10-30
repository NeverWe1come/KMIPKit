"""Manifest contract for the accepted T036 profile-validation mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PROFILE_CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"

RUST_MAPPINGS = {
    "kmipkit_client::Client::execute_with_profiles",
    "kmipkit_client::Client::execute_with_profiles_and_options",
    "kmipkit_client::Client::execute_with_profiles_and_context",
    "kmipkit_client::Client::execute_with_profiles_and_context_and_options",
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
}

C_MAPPINGS = {
    "kmipkit_profile_validation_selection_create",
    "kmipkit_profile_validation_selection_release",
    "kmipkit_profile_validation_context_create",
    "kmipkit_profile_validation_context_add_object_state_v1",
    "kmipkit_profile_validation_context_release",
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
}

PYTHON_MAPPINGS = {
    "kmipkit.client.Client.execute_with_profiles",
    "kmipkit.client.Client.execute_with_profiles_and_options",
    "kmipkit.client.Client.execute_with_profiles_and_context",
    "kmipkit.client.Client.execute_with_profiles_and_context_and_options",
    "kmipkit.client.ProfileValidationSelection",
    "kmipkit.client.ProfileValidationContext",
    "kmipkit.client.ProfileObjectState",
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
        self.assertTrue(JAVA_MAPPINGS <= set(capability["mappings"]["java"]))
        self.assertTrue(PYTHON_MAPPINGS <= set(capability["mappings"]["python"]))
        self.assertIn("KMIPKIT-0010-FR-005", capability["requirementIds"])
        self.assertIn("KMIPKIT-0010-FR-013", capability["requirementIds"])
        self.assertIn("KMIPKIT-0010-FR-019", capability["requirementIds"])
        self.assertIn("KMIPKIT-0043-FR-021", capability["requirementIds"])

    def test_profile_readiness_remains_incomplete_and_unclaimed(self) -> None:
        catalog = json.loads(PROFILE_CATALOG_PATH.read_text(encoding="utf-8"))
        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        self.assertEqual("evidence_incomplete", baseline["claim_state"])
        self.assertFalse(baseline.get("support_claim", False))


if __name__ == "__main__":
    unittest.main()
