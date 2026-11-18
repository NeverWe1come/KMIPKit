"""Automated unit tests for KMIPKit Interop Operational Tools:
- live_discover_capabilities.py
- kms_garbage_collector.py
- aggregate_matrix.py (HTML Dashboard)
- run_all.py (Cross-language Orchestrator)
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

try:
    import jsonschema
    HAS_JSONSCHEMA = True
except ImportError:
    HAS_JSONSCHEMA = False

from interop.shared.tools.live_discover_capabilities import (
    build_capabilities_from_query_data,
    discover_server_capabilities,
    parse_mock_response,
)
from interop.shared.tools.kms_garbage_collector import simulate_object_cleanup
from interop.shared.tools.aggregate_matrix import aggregate_results, generate_html
from interop.shared.tools.run_all import (
    build_command_for_language,
    check_toolchain_prerequisites,
    orchestrate_interop_run,
)

BASE_DIR = Path(__file__).resolve().parents[3]
SHARED_DIR = Path(__file__).resolve().parents[1]
SCHEMAS_DIR = SHARED_DIR / "schemas"


class OperationalToolsTests(unittest.TestCase):
    def test_live_discover_capabilities_schema_compliance(self) -> None:
        caps = build_capabilities_from_query_data(
            server_info={"vendor": "MockVendor", "product": "MockKMS", "version": "2.1.5"},
            supported_operations=["Discover Versions", "Query", "Create", "Get", "Destroy"],
            supported_objects=["SymmetricKey", "PublicKey"],
            supported_versions=["2.1"],
        )

        self.assertEqual(caps["server"]["vendor"], "MockVendor")
        self.assertEqual(caps["server"]["product"], "MockKMS")
        self.assertTrue(caps["operations"]["Create"]["supported"])
        self.assertFalse(caps["operations"]["Archive"]["supported"])
        self.assertTrue(caps["managed_objects"]["SymmetricKey"]["supported"])
        self.assertFalse(caps["managed_objects"]["Certificate"]["supported"])

        if HAS_JSONSCHEMA:
            with open(SCHEMAS_DIR / "server-capabilities.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
                schema = json.load(f)
            jsonschema.validate(instance=caps, schema=schema)

    def test_live_discover_capabilities_custom_overrides(self) -> None:
        caps = discover_server_capabilities(
            vendor="CustomCorp",
            product="CloudKMS",
            server_id="custom-kms-01",
        )
        self.assertEqual(caps["server"]["vendor"], "CustomCorp")
        self.assertEqual(caps["server"]["product"], "CloudKMS")
        self.assertEqual(caps["server"]["id"], "custom-kms-01")

        if HAS_JSONSCHEMA:
            with open(SCHEMAS_DIR / "server-capabilities.schema.json", "r", encoding="utf-8") as f:  # NOSONAR
                schema = json.load(f)
            jsonschema.validate(instance=caps, schema=schema)

    def test_kms_garbage_collector_dry_run(self) -> None:
        inventory = [
            {"id": "k1", "name": "kmipkit-test-aes-1", "type": "SymmetricKey", "state": "Active"},
            {"id": "k2", "name": "kmipkit-test-rsa-2", "type": "PrivateKey", "state": "PreActive"},
            {"id": "p1", "name": "prod-master-key", "type": "SymmetricKey", "state": "Active"},
        ]
        res = simulate_object_cleanup(inventory, prefix="kmipkit-", dry_run=True)
        self.assertTrue(res["dry_run"])
        self.assertEqual(res["total_scanned"], 3)
        self.assertEqual(res["matched_count"], 2)
        self.assertEqual(res["skipped_count"], 1)
        self.assertEqual(res["revoked_count"], 0)
        self.assertEqual(res["destroyed_count"], 0)

        actions = res["actions"]
        self.assertEqual(len(actions), 2)
        self.assertEqual(actions[0]["action"], "WOULD_REVOKE_AND_DESTROY")
        self.assertEqual(actions[1]["action"], "WOULD_DESTROY")

    def test_kms_garbage_collector_active_cleanup(self) -> None:
        inventory = [
            {"id": "k1", "name": "kmipkit-test-aes-1", "type": "SymmetricKey", "state": "Active"},
            {"id": "k2", "name": "kmipkit-test-rsa-2", "type": "PrivateKey", "state": "PreActive"},
            {"id": "k3", "name": "kmipkit-failing-obj", "type": "SecretData", "state": "Active", "simulate_error": True},
        ]
        res = simulate_object_cleanup(inventory, prefix="kmipkit-", dry_run=False)
        self.assertFalse(res["dry_run"])
        self.assertEqual(res["matched_count"], 3)
        self.assertEqual(res["revoked_count"], 1)  # Only k1 succeeded revoke
        self.assertEqual(res["destroyed_count"], 2)  # k1 and k2 succeeded destroy
        self.assertEqual(res["failed_count"], 1)  # k3 failed

    def test_aggregate_matrix_html_dashboard_offline(self) -> None:
        sample_results = {
            "servers": ["test-kms-1"],
            "languages": ["rust", "python"],
            "total_cases": 2,
            "parity_by_server": {
                "test-kms-1": [
                    {
                        "case_id": "KMIP-AES-GCM-256",
                        "category": "SymmetricCrypto",
                        "description": "AES GCM 256 encrypt",
                        "results": {"rust": "PASSED", "python": "PASSED"},
                        "status": "PARITY_OK",
                    },
                    {
                        "case_id": "KMIP-RSA-4096-SIGN",
                        "category": "AsymmetricCrypto",
                        "description": "RSA 4096 sign verify",
                        "results": {"rust": "PASSED", "python": "FAILED"},
                        "status": "PARITY_MISMATCH",
                    },
                ]
            },
        }
        html_out = generate_html(sample_results)

        # 1. Structure check
        self.assertTrue(html_out.startswith("<!DOCTYPE html>"))
        self.assertIn("KMIPKit Interoperability Dashboard", html_out)
        self.assertIn("KMIP-AES-GCM-256", html_out)
        self.assertIn("KMIP-RSA-4096-SIGN", html_out)
        self.assertIn("kpi-grid", html_out)
        self.assertIn("search-input", html_out)

        # 2. Strict Offline / Air-Gapped Check: Zero external URLs in script or link
        external_urls = re.findall(r'(?:href|src)=["\']https?://[^"\']+["\']', html_out, re.IGNORECASE)
        self.assertEqual(len(external_urls), 0, f"Found external CDN or remote URLs: {external_urls}")

    def test_run_all_orchestrator_dry_run(self) -> None:
        tools = check_toolchain_prerequisites()
        self.assertIn("rust", tools)
        self.assertIn("python", tools)

        # Test command building for all 4 languages
        dummy_dir = BASE_DIR
        prof = SHARED_DIR / "profiles/generic-kmip-2.1.json"
        cat = SHARED_DIR / "catalog/kmipkit-1.0-test-catalog.json"
        vec = SHARED_DIR / "vectors/canonical-vectors.json"
        out = SHARED_DIR / "test-out.json"

        cmd_rust = build_command_for_language("rust", dummy_dir, prof, None, cat, vec, out)
        cmd_c = build_command_for_language("c", dummy_dir, prof, None, cat, vec, out)
        cmd_java = build_command_for_language("java", dummy_dir, prof, None, cat, vec, out)
        cmd_py = build_command_for_language("python", dummy_dir, prof, None, cat, vec, out)

        self.assertIn("cargo", cmd_rust[0])
        self.assertIn("kmip_validator_c", cmd_c[0])
        self.assertEqual(cmd_c[1], "run")
        self.assertNotIn("--vectors", cmd_c)
        self.assertTrue("gradle" in cmd_java[0] or "java" in cmd_java[0])
        self.assertIn("python", cmd_py[0])

        # Test dry-run execution
        import tempfile
        with tempfile.TemporaryDirectory() as tmp_dir:
            tmp_out_dir = Path(tmp_dir)
            res = orchestrate_interop_run(
                languages=["rust", "python"],
                base_dir=dummy_dir,
                profile_path=prof,
                catalog_path=cat,
                vectors_path=vec,
                output_dir=tmp_out_dir,
                dry_run=True,
            )
            self.assertTrue(res["dry_run"])
            self.assertEqual(len(res["runs"]), 2)
            self.assertEqual(res["runs"][0]["status"], "PLANNED")
            self.assertEqual(res["runs"][1]["status"], "PLANNED")
        self.assertEqual(res["runs"][1]["status"], "PLANNED")


if __name__ == "__main__":
    unittest.main()
