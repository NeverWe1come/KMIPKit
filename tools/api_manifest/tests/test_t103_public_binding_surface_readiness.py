"""Require a resolved inventory owner for every shared binding declaration."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


class PublicBindingSurfaceReadinessTests(unittest.TestCase):
    def test_inventory_capability_is_declared_with_exact_mapping_inventory(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        capability = next(
            item
            for item in manifest["capabilities"]
            if item.get("capabilityId") == "public_binding_surface"
        )
        declarations = [
            item
            for item in manifest["publicDeclarations"]
            if item.get("capabilityId") == "public_binding_surface"
            and item.get("declarationState") in {"declared", "verified"}
        ]

        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual("KMIPKIT-0043", capability["ownerSpecId"])
        self.assertNotIn("candidateSpecId", capability)
        self.assertNotIn("blockedReason", capability)
        generate._validate_approval_source(
            capability["approvalEvidence"], ROOT, feature_id="KMIPKIT-0043"
        )
        for language in ("rust", "c", "java", "python"):
            expected = sorted(
                {
                    row["languageMappings"][language]["name"]
                    for row in declarations
                    if language in row.get("languageMappings", {})
                }
            )
            self.assertEqual(expected, capability["mappings"][language])


if __name__ == "__main__":
    unittest.main()
