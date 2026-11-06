"""Exclude the generated Rust profile namespace while mapping all its children."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
MODULE_ID = "rust-observed-048f3160831863109c3f"
MODULE_NAME = "kmipkit_protocol::profiles_generated"
OWNER = "specs/043-public-api-bindings/spec.md"
EVIDENCE = "tools/api_manifest/tests/test_t102_profiles_generated_module_mapping.py"


class T102GeneratedProfileModuleMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_module_namespace_has_one_exact_approved_exclusion(self) -> None:
        module_rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] == MODULE_ID
        ]
        exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            and row.get("exportName") == MODULE_NAME
        ]

        self.assertEqual([], module_rows)
        self.assertEqual(1, len(exclusions))
        exclusion = exclusions[0]
        self.assertIn("namespace container", exclusion["reason"])
        self.assertEqual(OWNER, exclusion["approvalSource"]["path"])
        self.assertEqual("Approved", exclusion["approvalSource"]["status"])
        self.assertIn("44 child declarations", exclusion["evidence"])
        self.assertIn("client_profile_readiness", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

        children = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName", "").startswith(MODULE_NAME + "::")
        ]
        self.assertEqual(44, len(children))
        self.assertTrue(
            all(
                row["declarationState"] == "declared"
                and row["capabilityId"] == "client_profile_readiness"
                and row["parityScope"] == "four-language"
                for row in children
            )
        )
        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "client_profile_readiness"
        )
        self.assertNotIn(MODULE_NAME, capability["mappings"]["rust"])
        self.assertEqual(
            T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"])
        )
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)

    def test_profile_catalog_sources_exist_in_all_four_languages(self) -> None:
        sources = (
            ROOT / "crates/kmipkit-protocol/src/profiles_generated.rs",
            ROOT / "bindings/c/include/kmipkit.h",
            ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/ProfileCatalog.java",
            ROOT / "bindings/python/src/kmipkit/profile_catalog.py",
        )
        for source in sources:
            self.assertTrue(source.is_file(), str(source))


if __name__ == "__main__":
    unittest.main()
