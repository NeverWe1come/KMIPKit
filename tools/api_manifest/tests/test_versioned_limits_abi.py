"""Public C ABI compatibility checks for extension-registry limits."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
V1_FIXTURE_PATH = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "public-api-v1.json"


class VersionedLimitsAbiTests(unittest.TestCase):
    def test_legacy_c_entry_points_keep_their_original_signatures(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        v1 = json.loads(V1_FIXTURE_PATH.read_text(encoding="utf-8"))
        actual = {function["id"]: function for function in manifest["functions"]}
        expected = {function["id"]: function for function in v1["functions"]}

        for function_id in (
            "client_extension_registry_create",
            "extension_registry_limits_defaults",
            "extension_registry_limits_create",
        ):
            with self.subTest(function=function_id):
                self.assertEqual(actual[function_id]["c"], expected[function_id]["c"])

    def test_versioned_c_entry_points_are_additive_and_validate_v1_layout(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        functions = {function["id"]: function for function in manifest["functions"]}
        expected = {
            "client_extension_registry_create_v1": (
                "kmipkit_client_extension_registry_create_v1",
                "limits",
                "const kmipkit_extension_registry_limits_v1_t *",
            ),
            "extension_registry_limits_v1_defaults": (
                "kmipkit_extension_registry_limits_v1_default_values",
                "out_limits",
                "kmipkit_extension_registry_limits_v1_t *",
            ),
            "extension_registry_limits_v1_validate": (
                "kmipkit_extension_registry_limits_v1_validate",
                "limits",
                "const kmipkit_extension_registry_limits_v1_t *",
            ),
        }

        for function_id, (symbol, parameter_name, parameter_type) in expected.items():
            with self.subTest(function=function_id):
                function = functions[function_id]
                self.assertEqual(function["c"]["symbol"], symbol)
                matching = [
                    item for item in function["c"]["parameters"]
                    if item["name"] == parameter_name
                ]
                self.assertEqual(len(matching), 1)
                if matching:
                    self.assertEqual(matching[0]["type"], parameter_type)

        versioned_type = next(
            row for row in manifest["types"] if row["id"] == "ExtensionRegistryLimitsV1"
        )
        self.assertEqual(versioned_type["c"]["name"], "kmipkit_extension_registry_limits_v1_t")
        self.assertTrue(versioned_type["c"]["versioned"])


if __name__ == "__main__":
    unittest.main()
