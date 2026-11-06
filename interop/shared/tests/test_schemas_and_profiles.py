"""Automated unit tests for interop shared schemas, profiles, and catalog."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

try:
    import jsonschema
    HAS_JSONSCHEMA = True
except ImportError:
    HAS_JSONSCHEMA = False


SHARED_DIR = Path(__file__).resolve().parents[1]
SCHEMAS_DIR = SHARED_DIR / "schemas"
PROFILES_DIR = SHARED_DIR / "profiles"
CATALOG_DIR = SHARED_DIR / "catalog"


class InteropSharedTests(unittest.TestCase):
    def test_schemas_are_valid_json(self) -> None:
        schema_files = list(SCHEMAS_DIR.glob("*.json"))
        self.assertGreaterEqual(len(schema_files), 4)
        for schema_path in schema_files:
            with self.subTest(schema=schema_path.name):
                with open(schema_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                self.assertIn("$schema", data)
                self.assertIn("$id", data)

    def test_profiles_conform_to_capabilities_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "server-capabilities.schema.json", "r", encoding="utf-8") as f:
            schema = json.load(f)

        profile_files = list(PROFILES_DIR.glob("*.json"))
        # Exclude connection example
        profile_files = [p for p in profile_files if "connection" not in p.name]
        self.assertGreaterEqual(len(profile_files), 4)

        for profile_path in profile_files:
            with self.subTest(profile=profile_path.name):
                with open(profile_path, "r", encoding="utf-8") as f:
                    profile = json.load(f)
                jsonschema.validate(instance=profile, schema=schema)

    def test_connection_example_conforms_to_connection_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "server-connection.schema.json", "r", encoding="utf-8") as f:
            schema = json.load(f)

        conn_path = PROFILES_DIR / "connection.example.json"
        with open(conn_path, "r", encoding="utf-8") as f:
            conn = json.load(f)
        jsonschema.validate(instance=conn, schema=schema)

    def test_catalog_conforms_to_catalog_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "test-catalog.schema.json", "r", encoding="utf-8") as f:
            schema = json.load(f)

        catalog_path = CATALOG_DIR / "kmipkit-1.0-test-catalog.json"
        with open(catalog_path, "r", encoding="utf-8") as f:
            catalog = json.load(f)
        jsonschema.validate(instance=catalog, schema=schema)

    def test_catalog_covers_all_56_c2s_operations(self) -> None:
        catalog_path = CATALOG_DIR / "kmipkit-1.0-test-catalog.json"
        with open(catalog_path, "r", encoding="utf-8") as f:
            catalog = json.load(f)

        stats = catalog.get("statistics", {})
        self.assertEqual(56, stats.get("operations_covered"))
        self.assertEqual(9, stats.get("managed_objects_covered"))
        self.assertEqual(63, stats.get("attributes_covered"))
        self.assertEqual(123, stats.get("total_cases"))


if __name__ == "__main__":
    unittest.main()
