"""Require the fluent Locate and Revoke binding declarations."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXPECTED_FUNCTIONS = {
    "client_locate_v1",
    "client_locate_with_options_v1",
    "client_revoke_v1",
    "client_revoke_with_options_v1",
}


class FluentLocateRevokeManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_direct_functions_are_declared_for_all_bindings(self) -> None:
        rows = [
            row for row in self.manifest["functions"]
            if row["id"] in EXPECTED_FUNCTIONS
        ]
        self.assertEqual(EXPECTED_FUNCTIONS, {row["id"] for row in rows})
        for row in rows:
            with self.subTest(function=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )

    def test_fluent_types_are_declared_for_all_bindings(self) -> None:
        expected = {"LocateBuilder", "RevokeBuilder"}
        rows = [row for row in self.manifest["types"] if row["id"] in expected]
        self.assertEqual(expected, {row["id"] for row in rows})
        for row in rows:
            with self.subTest(builder=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )


if __name__ == "__main__":
    unittest.main()
