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
            len([row for row in declarations if row.get("capabilityId") == "client_profile_readiness"]),
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




if __name__ == "__main__":
    unittest.main()
