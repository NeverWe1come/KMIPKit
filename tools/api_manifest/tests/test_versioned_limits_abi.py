"""Public C ABI checks for versioned extension-registry limits."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"


class VersionedLimitsAbiTests(unittest.TestCase):
    def test_public_c_limit_entry_points_use_the_versioned_value_type(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        functions = {function["id"]: function for function in manifest["functions"]}
        expected = {
            "client_extension_registry_create": "limits",
            "extension_registry_limits_defaults": "out_limits",
            "extension_registry_limits_create": "limits",
        }

        for function_id, parameter_name in expected.items():
            with self.subTest(function=function_id):
                parameters = functions[function_id]["c"]["parameters"]
                matching = [
                    item for item in parameters if item["name"] == parameter_name
                ]
                self.assertEqual(len(matching), 1)
                if not matching:
                    continue
                self.assertEqual(
                    matching[0]["type"],
                    "kmipkit_extension_registry_limits_v1_t *",
                )


if __name__ == "__main__":
    unittest.main()
