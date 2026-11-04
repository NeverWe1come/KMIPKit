"""Keep profile-validation test constructors outside the product API."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PROFILE_SOURCE = ROOT / "crates" / "kmipkit-protocol" / "src" / "profile.rs"
PROFILE_OWNER = {
    "kind": "feature-specification",
    "path": "specs/010-profile-conformance/spec.md",
    "status": "Accepted",
    "evidence": "**Status**: Accepted for implementation by merge of [PR #49](https://github.com/NeverWe1come/KMIPKit/pull/49) into `release/1.0.0` (merge commit `045b97dde40e54efbc264d95c1d827c4f7d40415`, 2026-10-07). No separate GitHub review event is claimed.",
}
TEST_HELPERS = {
    ("rust-kmipkit", "kmipkit::ProfileValidationError::for_test"),
    ("rust-kmipkit-client", "kmipkit_client::ProfileValidationError::for_test"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileValidationError::for_test"),
}


class T102ProfileValidationTestHelperExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["approvedExclusions"]
        }

    def test_exact_test_support_helpers_are_excluded_under_profile_owner(self) -> None:
        profile_source = PROFILE_SOURCE.read_text(encoding="utf-8")
        self.assertIn('#[cfg(feature = "test-support")]', profile_source)

        for identity in TEST_HELPERS:
            with self.subTest(identity=identity):
                self.assertFalse(
                    identity in self.declarations,
                    f"test-support helper is still a public declaration: {identity}",
                )
                exclusion = self.exclusions[identity]
                self.assertEqual(PROFILE_OWNER, exclusion["approvalSource"])
                self.assertIn("test-support", exclusion["reason"])
                self.assertIn("cross-crate client tests", exclusion["evidence"])
                self.assertIn("crates/kmipkit-protocol/src/profile.rs", exclusion["evidence"])
                for requirement_id in (
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ):
                    self.assertIn(requirement_id, exclusion["evidence"])

    def test_profile_error_models_remain_product_declarations(self) -> None:
        for package_id, export_name in (
            ("rust-kmipkit", "kmipkit::ProfileValidationError"),
            ("rust-kmipkit-client", "kmipkit_client::ProfileValidationError"),
            ("rust-kmipkit-protocol", "kmipkit_protocol::ProfileValidationError"),
        ):
            with self.subTest(package_id=package_id):
                self.assertEqual(
                    "declared",
                    self.declarations[(package_id, export_name)]["declarationState"],
                )


if __name__ == "__main__":
    unittest.main()
