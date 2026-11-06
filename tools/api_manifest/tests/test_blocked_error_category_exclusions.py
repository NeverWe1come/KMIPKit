"""Blocked error categories may point to specifically excluded exports."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.generate import ManifestError, _validate_references

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
INTERNAL_EXPORTS = (
    ("c-kmipkit", "KMIPKIT_ERROR_INTERNAL"),
    ("java-kmipkit", "org.kmipkit.KmipException"),
    ("python-kmipkit", "kmipkit.errors.KmipKitError"),
)


class BlockedErrorCategoryExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_blocked_category_can_reference_exact_excluded_language_exports(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        excluded = set(INTERNAL_EXPORTS)
        manifest["publicDeclarations"] = [
            row
            for row in manifest["publicDeclarations"]
            if (row["packageId"], row["exportName"]) not in excluded
        ]
        manifest["approvedExclusions"].extend(
            {"packageId": package_id, "exportName": export_name}
            for package_id, export_name in INTERNAL_EXPORTS
        )

        try:
            _validate_references(manifest)
        except ManifestError as error:
            self.fail(f"blocked exclusions should satisfy error references: {error}")

    def test_ready_category_cannot_reference_an_excluded_export(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        excluded = set(INTERNAL_EXPORTS)
        manifest["publicDeclarations"] = [
            row
            for row in manifest["publicDeclarations"]
            if (row["packageId"], row["exportName"]) not in excluded
        ]
        manifest["approvedExclusions"].extend(
            {"packageId": package_id, "exportName": export_name}
            for package_id, export_name in INTERNAL_EXPORTS
        )
        internal = next(
            row for row in manifest["errorCategories"] if row["id"] == "internal"
        )
        internal["mappingState"] = "ready"
        internal.pop("stableCHeaderCode", None)

        with self.assertRaisesRegex(
            ManifestError, "error category references an undeclared language export"
        ):
            _validate_references(manifest)


if __name__ == "__main__":
    unittest.main()
