"""Reconcile the final Java and Python profile binding inventory."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"

PYTHON_PROFILE_FIELD_EXPORTS = {
    *(f"kmipkit.profile_catalog.ClientProfile.{name}" for name in (
        "applicability", "claim_state", "dependency_profile_ids", "direct_source_clause_ids",
        "encoding_requirements", "name", "profile_id", "requirement_ids", "selected_target",
        "source_clause_ids", "test_cases", "transport_requirements", "unresolved_scope_obligations",
    )),
    *(f"kmipkit.profile_catalog.ClientRequirement.{name}" for name in (
        "has_accepted_deviation", "official_test_case_ids", "profile_ids", "requirement_id",
        "scope_state", "source_clause_ids", "source_force", "source_gap_id", "source_keyword",
        "source_locator", "verification_category",
    )),
    "kmipkit.profile_catalog.ProfileScopeBlocker.requirement_id",
    "kmipkit.profile_catalog.ProfileScopeBlocker.scope_state",
    "kmipkit.profile_catalog.ProfileScopeBlocker.source_clause_id",
    *(f"kmipkit.profile_catalog.ProfileTestCase.{name}" for name in (
        "fixture_availability", "mandatory_status", "official_case_id", "test_id",
    )),
    *(f"kmipkit.profile_readiness.ProfileReadiness.{name}" for name in (
        "blockers", "effective_state", "profile_id", "published_claim",
        "requirement_records_complete", "runtime_validation_available", "target_selected",
    )),
    *(f"kmipkit.profile_readiness.ProfileReadinessBlocker.{name}" for name in (
        "kind", "message", "profile_id", "requirement_id", "source_clause_id", "test_id",
    )),
}

JAVA_INTERNAL_EXPORTS = {
    "org.kmipkit.NativeExtensionRegistry.PROFILE_CATALOG_CLIENT_PROFILES",
    "org.kmipkit.NativeExtensionRegistry.PROFILE_CATALOG_CLIENT_REQUIREMENTS",
    "org.kmipkit.NativeExtensionRegistry.PROFILE_CATALOG_PROFILE_TEST_CASES",
    "org.kmipkit.NativeExtensionRegistry.profileCatalogRelationCount",
    "org.kmipkit.NativeExtensionRegistry.profileCatalogRelationRow",
    "org.kmipkit.NativeExtensionRegistry.profileCatalogRow",
    "org.kmipkit.NativeExtensionRegistry.profileCatalogTableCount",
    "org.kmipkit.NativeExtensionRegistry.profileReadinessBlockerAt",
    "org.kmipkit.NativeExtensionRegistry.profileReadinessCreate",
    "org.kmipkit.NativeExtensionRegistry.profileReadinessRelease",
    "org.kmipkit.NativeExtensionRegistry.profileReadinessView",
    "org.kmipkit.NativeExtensionRegistry.registerRequestWithProtectionStorageMasks",
    "org.kmipkit.extensions.ProfileCatalog",
    *{
        f"org.kmipkit.extensions.ProfileCatalog.{record}.{member}"
        for record in (
            "ClientProfile",
            "ClientRequirement",
            "ProfileScopeBlocker",
            "ProfileTestCase",
        )
        for member in ("<init>", "equals", "hashCode", "toString")
    },
    "org.kmipkit.extensions.ProfileCatalog.VerificationCategory.valueOf",
    "org.kmipkit.extensions.ProfileCatalog.VerificationCategory.values",
    *{
        f"org.kmipkit.extensions.ProfileEvidenceRecord.{member}"
        for member in ("<init>", "equals", "hashCode", "toString")
    },
    *{
        f"org.kmipkit.extensions.ProfileReadinessBlocker.{member}"
        for member in ("<init>", "equals", "hashCode", "toString")
    },
}

JAVA_PROFILE_VALUE_EXPORTS = {
    "org.kmipkit.extensions.ProfileCatalog.VerificationCategory.value",
    "org.kmipkit.extensions.ProfileEvidenceRecord.requirementId",
}


class T109BindingExportAuditReconciliationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_python_profile_fields_match_the_audited_field_kind(self) -> None:
        manifest_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "python-kmipkit"
            and row.get("exportName") in PYTHON_PROFILE_FIELD_EXPORTS
        }
        self.assertEqual(PYTHON_PROFILE_FIELD_EXPORTS, set(manifest_rows))
        for export_name, row in manifest_rows.items():
            with self.subTest(export=export_name):
                self.assertEqual("field", row["kind"])

    def test_java_profile_value_members_are_declared_with_four_language_mappings(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit"
        }
        for export_name in JAVA_PROFILE_VALUE_EXPORTS:
            with self.subTest(export=export_name):
                self.assertIn(export_name, declarations)
                row = declarations[export_name]
                self.assertEqual("java-member", row["kind"])
                self.assertEqual("client_profile_readiness", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(
                    {"c", "java", "python", "rust"},
                    set(row["languageMappings"]),
                )

    def test_java_internal_bridges_and_generated_members_have_exact_exclusions(self) -> None:
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "java-kmipkit"
        }
        missing = JAVA_INTERNAL_EXPORTS - set(exclusions)
        self.assertFalse(missing, f"Java profile exclusions missing: {sorted(missing)}")
        for export_name in JAVA_INTERNAL_EXPORTS:
            row = exclusions[export_name]
            with self.subTest(export=export_name):
                self.assertTrue(row.get("reason"))
                self.assertIn("KMIPKIT-0043-FR-021", row.get("evidence", ""))
                self.assertEqual(
                    "specs/010-profile-conformance/spec.md",
                    row["approvalSource"]["path"],
                )

    def test_java_overloads_declare_all_audited_signatures(self) -> None:
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit"
        }
        for export_name in (
            "org.kmipkit.extensions.ClientBatchItem.pkcs11",
            "org.kmipkit.extensions.ProfileEvidenceRecord.passed",
        ):
            with self.subTest(export=export_name):
                self.assertIn(export_name, rows)
                signatures = rows[export_name].get("javaSignatures", [])
                self.assertEqual(2, len(signatures))
                self.assertEqual(signatures, sorted(set(signatures)))


if __name__ == "__main__":
    unittest.main()
