"""Automated unit tests for interop shared schemas, profiles, vectors, tools, and catalog."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

try:
    import jsonschema
    HAS_JSONSCHEMA = True
except ImportError:
    HAS_JSONSCHEMA = False

from interop.shared.tools.preflight_and_audit import audit_profile_diff, validate_connection_config
from interop.shared.tools.aggregate_matrix import aggregate_results, generate_markdown


SHARED_DIR = Path(__file__).resolve().parents[1]
SCHEMAS_DIR = SHARED_DIR / "schemas"
PROFILES_DIR = SHARED_DIR / "profiles"
CATALOG_DIR = SHARED_DIR / "catalog"
VECTORS_DIR = SHARED_DIR / "vectors"


class InteropSharedTests(unittest.TestCase):
    def test_schemas_are_valid_json(self) -> None:
        schema_files = list(SCHEMAS_DIR.glob("*.json"))
        self.assertGreaterEqual(len(schema_files), 5)
        for schema_path in schema_files:
            with self.subTest(schema=schema_path.name):
                with open(schema_path, "r", encoding="utf-8") as f:  # NOSONAR
                    data = json.load(f)
                self.assertIn("$schema", data)
                self.assertIn("$id", data)

    def test_profiles_conform_to_capabilities_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "server-capabilities.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
            schema = json.load(f)

        profile_files = list(PROFILES_DIR.glob("*.json"))
        profile_files = [p for p in profile_files if "connection" not in p.name]
        self.assertGreaterEqual(len(profile_files), 4)

        for profile_path in profile_files:
            with self.subTest(profile=profile_path.name):
                with open(profile_path, "r", encoding="utf-8") as f:  # NOSONAR
                    profile = json.load(f)
                jsonschema.validate(instance=profile, schema=schema)

    def test_connection_example_conforms_to_connection_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "server-connection.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
            schema = json.load(f)

        conn_path = PROFILES_DIR / "connection.example.json"
        with open(conn_path, "r", encoding="utf-8") as f:  # NOSONAR
            conn = json.load(f)
        jsonschema.validate(instance=conn, schema=schema)

    def test_vectors_conform_to_vectors_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "test-vectors.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
            schema = json.load(f)

        vectors_path = VECTORS_DIR / "canonical-vectors.json"
        with open(vectors_path, "r", encoding="utf-8") as f:  # NOSONAR
            vec = json.load(f)
        jsonschema.validate(instance=vec, schema=schema)
        self.assertIn("aes-256", vec["symmetric_keys"])
        self.assertIn("rsa-2048", vec["asymmetric_keys"])
        self.assertIn("x509_certificate", vec["managed_objects"])

    def test_catalog_conforms_to_catalog_schema(self) -> None:
        if not HAS_JSONSCHEMA:
            self.skipTest("jsonschema library not installed")
        with open(SCHEMAS_DIR / "test-catalog.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
            schema = json.load(f)

        catalog_path = CATALOG_DIR / "kmipkit-1.0-test-catalog.json"
        with open(catalog_path, "r", encoding="utf-8") as f:  # NOSONAR
            catalog = json.load(f)
        jsonschema.validate(instance=catalog, schema=schema)

    def test_catalog_coverage_and_extended_categories(self) -> None:
        catalog_path = CATALOG_DIR / "kmipkit-1.0-test-catalog.json"
        with open(catalog_path, "r", encoding="utf-8") as f:  # NOSONAR
            catalog = json.load(f)

        stats = catalog.get("statistics", {})
        self.assertEqual(57, stats.get("operations_covered"))
        self.assertEqual(9, stats.get("managed_objects_covered"))
        self.assertEqual(63, stats.get("attributes_covered"))
        self.assertEqual(64, stats.get("enumerations_covered"))
        self.assertEqual(17, stats.get("batch_scenarios"))
        self.assertEqual(17, stats.get("asynchronous_scenarios"))
        self.assertEqual(3, stats.get("exhaustive_bitmask_families"))
        self.assertEqual(2560, stats.get("normative_targets"))
        self.assertEqual(2689, stats.get("total_cases"))

        categories = {tc.get("category") for tc in catalog.get("test_cases", [])}
        self.assertIn("protocol_and_session", categories)
        self.assertIn("object_lifecycle", categories)
        self.assertIn("managed_objects", categories)
        self.assertIn("attributes", categories)
        self.assertIn("cryptography_and_rng", categories)
        self.assertIn("enums_and_bitmasks", categories)
        self.assertIn("normative_inventory", categories)
        self.assertIn("batch_workflows", categories)
        self.assertIn("multipart_crypto", categories)
        self.assertIn("asynchronous_workflows", categories)
        self.assertIn("negative_and_boundary", categories)

    def test_preflight_and_profile_diff_tool(self) -> None:
        # 1. validate connection
        conn_path = PROFILES_DIR / "connection.example.json"
        res = validate_connection_config(conn_path, check_network=False)
        self.assertTrue(res["valid"])
        self.assertEqual("example-kms", res["server_id"])
        self.assertIn("client_cert_path", res["file_checks"])

        # 2. profile diff
        diff = audit_profile_diff(PROFILES_DIR / "cosmian-kms.json", PROFILES_DIR / "generic-kmip-2.1.json")
        self.assertIn("total_discrepancies", diff)
        self.assertGreater(diff["total_discrepancies"], 0)

    def test_aggregate_matrix_tool(self) -> None:
        # Create mock result data
        mock_res1 = {
            "server_identifier": "mock-kms",
            "server": {"id": "mock-kms", "version": "test-1", "configuration_fingerprint": "a" * 64},
            "protocol_version": "2.1",
            "catalog_sha256": "c" * 64,
            "execution": {"library_commit": "abcdef0", "transport": "raw_tls", "encoding": "TTLV", "api_level": "typed-protocol"},
            "language": "rust",
            "cases": [
                {"case_id": "test-op-create", "category": "object_lifecycle", "status": "PASSED"},
                {"case_id": "test-op-destroy", "category": "object_lifecycle", "status": "PASSED"}
            ]
        }
        mock_res2 = {
            "server_identifier": "mock-kms",
            "server": {"id": "mock-kms", "version": "test-1", "configuration_fingerprint": "a" * 64},
            "protocol_version": "2.1",
            "catalog_sha256": "c" * 64,
            "execution": {"library_commit": "abcdef0", "transport": "https", "encoding": "JSON", "api_level": "high-level"},
            "language": "python",
            "cases": [
                {"case_id": "test-op-create", "category": "object_lifecycle", "status": "PASSED"},
                {"case_id": "test-op-destroy", "category": "object_lifecycle", "status": "FAILED"}
            ]
        }
        tmp_dir = SHARED_DIR / "tests" / ".tmp_test_results"
        tmp_dir.mkdir(parents=True, exist_ok=True)
        try:
            p1 = tmp_dir / "rust_res.json"
            p2 = tmp_dir / "py_res.json"
            with open(p1, "w", encoding="utf-8") as f:  # NOSONAR
                json.dump(mock_res1, f)
            with open(p2, "w", encoding="utf-8") as f:  # NOSONAR
                json.dump(mock_res2, f)

            agg = aggregate_results([p1, p2])
            self.assertIn("mock-kms", agg["servers"])
            self.assertIn("rust", agg["languages"])
            self.assertIn("python", agg["languages"])
            self.assertEqual(2, agg["total_cases"])

            md = generate_markdown(agg)
            self.assertIn("mock-kms", md)
            self.assertIn("MISMATCH", md)
        finally:
            for child in tmp_dir.iterdir():
                child.unlink()
            tmp_dir.rmdir()


if __name__ == "__main__":
    unittest.main()
