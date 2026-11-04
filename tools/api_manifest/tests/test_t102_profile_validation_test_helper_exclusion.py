"""Keep the test-support-only profile constructor blocked without an ADR boundary."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PROFILE_SOURCE = ROOT / "crates" / "kmipkit-protocol" / "src" / "profile.rs"
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

    def test_test_support_helpers_stay_blocked_without_an_accepted_rust_only_adr(self) -> None:
        profile_source = PROFILE_SOURCE.read_text(encoding="utf-8")
        self.assertRegex(
            profile_source,
            re.compile(
                r"(?ms)^\s*/// Constructs a stable validation error for cross-crate client tests\."
                r"\n\s*#\[cfg\(feature = \"test-support\"\)\]"
                r"\n\s*#\[doc\(hidden\)\]"
                r"\n\s*#\[must_use\]"
                r"\n\s*pub fn for_test\("
            ),
        )

        for identity in TEST_HELPERS:
            with self.subTest(identity=identity):
                self.assertTrue(
                    identity in self.declarations,
                    f"public test-support helper disappeared instead of staying blocked: {identity}",
                )
                declaration = self.declarations.get(identity)
                if declaration is None:
                    continue
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertEqual("KMIPKIT-0043", declaration["candidateSpecId"])
                self.assertNotIn("languageMappings", declaration)
                self.assertIn("accepted ADR", declaration["blockedReason"])
                self.assertNotIn(identity, self.exclusions)

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
