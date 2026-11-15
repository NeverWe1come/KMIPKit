"""Exact T102 cross-language profile readiness and catalog mapping contract."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
READINESS_SHA256 = "ce0632e2dadf8ed79064c88becc07cf3f1bc1cfba76ce4ce3f5cf8cc3c5dff06"
CATALOG_SHA256 = "d876fc31f19b3b4b63a5b09765dfdac6925d97a2199aac6c88c10a9e5bcb47cb"
LANGUAGE_MAPPING_SHA256 = {
    "rust": "477d61fb1bf54ce7368fed40271ff6cdeec01113f0314dd3312382af3be477a7",
    "c": "64c6bf7ac45e050c128dc923e58bde968e54b8f187396332a36fd61825e2aa74",
    "java": "9d327c449f9cb9c9ac81d2f0c203632c11cef6c3e4388c8101782768358954e8",
    "python": "bb53f030895362fc20e8bf240822a8871f1c27967c786f2a85f282ee8e89f279",
}
OWNER_REQUIREMENTS = {
    "KMIPKIT-0010-FR-001", "KMIPKIT-0010-FR-002", "KMIPKIT-0010-FR-003",
    "KMIPKIT-0010-FR-010", "KMIPKIT-0010-FR-011", "KMIPKIT-0010-FR-012",
    "KMIPKIT-0010-FR-013", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017", "KMIPKIT-0043-FR-021", "KMIPKIT-0043-FR-022",
}


def stable_names_sha256(names: set[str]) -> str:
    return hashlib.sha256("\n".join(sorted(names)).encode("utf-8")).hexdigest()


class T102ProfileReadinessParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_exact_45_readiness_and_44_catalog_declarations_map_atomically(self) -> None:
        declarations = self.manifest["publicDeclarations"]
        readiness_names = {
            row["exportName"] for row in declarations
            if row["packageId"] == "rust-kmipkit-protocol"
            and (row["exportName"].startswith("kmipkit_protocol::ProfileEvidenceRecord")
                 or row["exportName"].startswith("kmipkit_protocol::ProfileReadiness")
                 or row["exportName"].startswith("kmipkit_protocol::ProfileEvidenceReadiness"))
        }
        catalog_names = {
            row["exportName"] for row in declarations
            if row["packageId"] == "rust-kmipkit-protocol"
            and (row["exportName"].startswith("kmipkit_protocol::profiles_generated::Generated")
                 or row["exportName"] in {
                    "kmipkit_protocol::profiles_generated::CLIENT_PROFILES",
                    "kmipkit_protocol::profiles_generated::CLIENT_REQUIREMENTS",
                    "kmipkit_protocol::profiles_generated::PROFILE_TEST_CASES",
                 })
        }
        self.assertEqual(len(readiness_names), 45)
        self.assertEqual(stable_names_sha256(readiness_names), READINESS_SHA256)
        self.assertEqual(len(catalog_names), 44)
        self.assertEqual(stable_names_sha256(catalog_names), CATALOG_SHA256)
        self.assertEqual(
            len([row for row in declarations if row.get("capabilityId") == "client_profile_readiness"
                 and row["packageId"] == "rust-kmipkit-protocol"
                 and row.get("candidateSpecId") in (None, "KMIPKIT-0043")
                 and (row["exportName"] in readiness_names | catalog_names)]),
            89,
        )
        for name in readiness_names | catalog_names:
            row = next(row for row in declarations
                       if row["packageId"] == "rust-kmipkit-protocol" and row["exportName"] == name)
            self.assertEqual(row["declarationState"], "declared", name)
            self.assertEqual(row["capabilityId"], "client_profile_readiness", name)
            self.assertTrue({"KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"}.issubset(
                set(row["requirementIds"])), name)
            self.assertTrue(set(row["requirementIds"]) & OWNER_REQUIREMENTS, name)

        capability = next(row for row in self.manifest["capabilities"]
                          if row["capabilityId"] == "client_profile_readiness")
        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["parityScope"], "four-language")
        self.assertEqual(capability["ownerSpecId"], "KMIPKIT-0010")
        self.assertEqual(set(capability["requirementIds"]), OWNER_REQUIREMENTS)
        self.assertTrue({"rust", "c", "java", "python"}.issubset(capability["mappings"]))
        for language in ("rust", "c", "java", "python"):
            self.assertTrue(capability["mappings"][language], language)

    def test_all_89_rust_exports_map_to_exact_declared_cross_language_targets(self) -> None:
        declarations = self.manifest["publicDeclarations"]
        readiness_names = {
            row["exportName"] for row in declarations
            if row["packageId"] == "rust-kmipkit-protocol"
            and (row["exportName"].startswith("kmipkit_protocol::ProfileEvidenceRecord")
                 or row["exportName"].startswith("kmipkit_protocol::ProfileReadiness")
                 or row["exportName"].startswith("kmipkit_protocol::ProfileEvidenceReadiness"))
        }
        catalog_names = {
            row["exportName"] for row in declarations
            if row["packageId"] == "rust-kmipkit-protocol"
            and (row["exportName"].startswith("kmipkit_protocol::profiles_generated::Generated")
                 or row["exportName"] in {
                    "kmipkit_protocol::profiles_generated::CLIENT_PROFILES",
                    "kmipkit_protocol::profiles_generated::CLIENT_REQUIREMENTS",
                    "kmipkit_protocol::profiles_generated::PROFILE_TEST_CASES",
                 })
        }
        rust_rows = [
            row for row in declarations
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"] in readiness_names | catalog_names
        ]
        self.assertEqual(len(rust_rows), 89)
        targets = {
            (row["packageId"], row["exportName"]): row
            for row in declarations
        }
        for language, expected_hash in LANGUAGE_MAPPING_SHA256.items():
            mapping_rows = []
            mapping_targets = set()
            for row in rust_rows:
                mapping = row["languageMappings"].get(language)
                self.assertIsNotNone(mapping, f"{row['exportName']} has no {language} mapping")
                mapping_rows.append(
                    f"{row['exportName']}\t{mapping['packageId']}:{mapping['name']}"
                )
                mapping_targets.add((mapping["packageId"], mapping["name"]))
                target = targets.get((mapping["packageId"], mapping["name"]))
                self.assertIsNotNone(target, f"{row['exportName']} target is not declared in {language}")
                self.assertEqual(target["declarationState"], "declared", row["exportName"])
                self.assertEqual(target["capabilityId"], "client_profile_readiness", row["exportName"])
            if language in {"java", "python"}:
                self.assertEqual(len(mapping_targets), 63, language)
            self.assertEqual(
                hashlib.sha256("\n".join(sorted(mapping_rows)).encode("utf-8")).hexdigest(),
                expected_hash,
                language,
            )

        evaluate = next(row for row in rust_rows
                        if row["exportName"] == "kmipkit_protocol::ProfileReadiness::evaluate")
        self.assertEqual(evaluate["languageMappings"]["java"]["name"],
                         "org.kmipkit.extensions.ProfileReadiness.evaluate")
        self.assertEqual(evaluate["languageMappings"]["python"]["name"],
                         "kmipkit.profile_readiness.ProfileReadiness.evaluate")
        blocker = next(row for row in rust_rows
                       if row["exportName"] == "kmipkit_protocol::ProfileReadinessBlocker::ProfileUnavailable")
        self.assertEqual(blocker["languageMappings"]["java"]["name"],
                         "org.kmipkit.extensions.ProfileReadinessBlocker.kind")
        self.assertEqual(blocker["languageMappings"]["python"]["name"],
                         "kmipkit.profile_readiness.ProfileReadinessBlocker.kind")
        unresolved_scope = next(row for row in rust_rows
                                if row["exportName"] == "kmipkit_protocol::profiles_generated::GeneratedClientProfile::unresolved_scope_obligations")
        self.assertEqual(unresolved_scope["languageMappings"]["java"]["name"],
                         "org.kmipkit.extensions.ProfileCatalog.ClientProfile.unresolvedScopeObligations")
        self.assertEqual(unresolved_scope["languageMappings"]["python"]["name"],
                         "kmipkit.profile_catalog.ClientProfile.unresolved_scope_obligations")

        verification_category_members = {
            "DiscouragedAbsenceOrDeviation": "DISCOURAGED_ABSENCE_OR_DEVIATION",
            "OptionalCapability": "OPTIONAL_CAPABILITY",
            "ProhibitedBehavior": "PROHIBITED_BEHAVIOR",
            "RecommendedBehaviorOrDeviation": "RECOMMENDED_BEHAVIOR_OR_DEVIATION",
            "RequiredBehavior": "REQUIRED_BEHAVIOR",
        }
        for rust_member, facade_member in verification_category_members.items():
            export_name = (
                "kmipkit_protocol::profiles_generated::GeneratedVerificationCategory::"
                f"{rust_member}"
            )
            row = next(row for row in rust_rows if row["exportName"] == export_name)
            self.assertEqual(
                "org.kmipkit.extensions.ProfileCatalog.VerificationCategory." + facade_member,
                row["languageMappings"]["java"]["name"],
            )
            self.assertEqual(
                "kmipkit.profile_catalog.VerificationCategory." + facade_member,
                row["languageMappings"]["python"]["name"],
            )




if __name__ == "__main__":
    unittest.main()
