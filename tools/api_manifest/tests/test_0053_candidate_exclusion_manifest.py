"""Manifest validation for exact KMIPKIT-0053 type-system exclusions."""

from __future__ import annotations

import copy
import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]


class CandidateExclusionManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = generate._load_json(
            ROOT, generate.MANIFEST_PATH, generate.MAX_MANIFEST_BYTES
        )
        cls.schema = generate._load_json(ROOT, generate.SCHEMA_PATH, generate.MAX_SCHEMA_BYTES)
        cls.catalog = generate._load_json(ROOT, generate.CATALOG_PATH, generate.MAX_CATALOG_BYTES)

    def test_exact_type_system_exclusions_are_accepted_by_the_manifest_schema(self) -> None:
        generate._validate_schema(self.manifest, self.schema)

    def test_candidate_exclusions_require_an_exact_verified_counterpart(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        exclusion = next(
            row
            for row in manifest["approvedExclusions"]
            if row.get("candidateSpecId") == "KMIPKIT-0053"
        )
        exclusion["counterpartDeclarationIds"] = ["missing-exact-counterpart"]

        with self.assertRaises(generate.ManifestError):
            generate._validate_approved_exclusions([exclusion], {}, ROOT)

    def test_candidate_exclusions_require_verified_same_capability_counterparts(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        exclusion = next(
            row
            for row in manifest["approvedExclusions"]
            if row.get("candidateSpecId") == "KMIPKIT-0053"
            and row.get("artifactKind")
        )
        counterpart_id = exclusion["counterpartDeclarationIds"][0]
        counterpart = next(
            row
            for row in manifest["publicDeclarations"]
            if row["declarationId"] == counterpart_id
        )
        counterpart["declarationState"] = "declared"

        with self.assertRaises(generate.ManifestError):
            generate._validate_approved_exclusions([exclusion], {}, ROOT)


if __name__ == "__main__":
    unittest.main()
