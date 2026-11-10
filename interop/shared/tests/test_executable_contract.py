from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from interop.shared.tools.aggregate_matrix import aggregate_results


ROOT = Path(__file__).resolve().parents[3]
CATALOG = ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"


class ExecutableContractTests(unittest.TestCase):
    def test_catalog_declares_versions_and_case_execution_contracts(self) -> None:
        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))

        self.assertEqual(["2.0", "2.1"], catalog["target_versions"])
        self.assertGreaterEqual(catalog["statistics"]["total_cases"], 200)
        case_ids = [case["case_id"] for case in catalog["test_cases"]]
        self.assertEqual(len(case_ids), len(set(case_ids)))
        self.assertEqual(5, catalog["statistics"]["server_to_client_operations_covered"])
        versions_by_operation = {}
        for case in catalog["test_cases"]:
            with self.subTest(case_id=case["case_id"]):
                self.assertTrue(case["normative_refs"])
                execution = case["execution"]
                self.assertIn(execution["mode"], {"server_exchange", "local_model"})
                self.assertTrue(execution["steps"])
                self.assertTrue(execution["assertions"])
                self.assertTrue(execution["protocol_versions"])
                self.assertTrue(execution["mutation"] in {"read_only", "mutating"})
                if case["category"] != "version_boundaries":
                    versions_by_operation.setdefault(case["operation"], execution["protocol_versions"])
        self.assertEqual(["2.0", "2.1"], versions_by_operation["Export"])
        self.assertEqual(["2.0", "2.1"], versions_by_operation["Import"])
        self.assertEqual(6, sum(case["category"] == "version_boundaries" for case in catalog["test_cases"]))
        source_operations = {
            element["name"] for element in json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))["elements"]
            if element.get("kind") == "operation" and element.get("direction") == "client_to_server" and element.get("scope_state") in {"client_1_0", "client_1_1"}
        }
        catalog_operations = {
            case["operation"] for case in catalog["test_cases"]
            if case["category"] not in {"version_boundaries", "server_initiated_operations", "normative_inventory"}
        }
        self.assertEqual(source_operations, catalog_operations)

    def test_catalog_and_result_schemas_validate_generated_evidence(self) -> None:
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")

        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        catalog_schema = json.loads((ROOT / "interop/shared/schemas/test-catalog.schema.json").read_text(encoding="utf-8"))
        jsonschema.validate(catalog, catalog_schema)

        result_schema = json.loads((ROOT / "interop/shared/schemas/validation-results.schema.json").read_text(encoding="utf-8"))
        sample = {
            "schema_version": "2.0",
            "run_id": "offline-rust-001",
            "tested_at": "2026-11-10T12:00:00Z",
            "language": "rust",
            "server": {"id": "loopback", "product": "KMIPKit Test Server", "version": "test", "configuration_fingerprint": "a" * 64},
            "protocol_version": "2.1",
            "catalog_sha256": "c" * 64,
            "execution": {"transport": "raw_tls", "encoding": "TTLV", "api_level": "typed-protocol", "library_commit": "abcdef0", "artifact_sha256": "b" * 64},
            "summary": {"total": 1, "passed": 1, "failed": 0, "unsupported": 0, "policy_restricted": 0, "not_run": 0},
            "cases": [{"case_id": "test-get", "status": "passed", "library_correctness": "passed", "server_compatibility": "supported", "parity": "not_compared", "result_status_code": 0, "result_reason_code": 7}],
        }
        jsonschema.validate(sample, result_schema)
        sample["cases"][0]["result_reason_code"] = 4294967296
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(sample, result_schema)

    def test_live_configuration_requires_isolation_and_mutation_opt_in(self) -> None:
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")

        schema = json.loads((ROOT / "interop/shared/schemas/server-connection.schema.json").read_text(encoding="utf-8"))
        config = json.loads((ROOT / "interop/shared/profiles/connection.example.json").read_text(encoding="utf-8"))
        jsonschema.validate(config, schema)

        unsafe = json.loads(json.dumps(config))
        unsafe["environment"]["dedicated_test_environment_confirmed"] = False
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(unsafe, schema)

        plaintext_secret = json.loads(json.dumps(config))
        plaintext_secret["authentication"]["username_password"]["password"] = "do-not-store"
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(plaintext_secret, schema)

    def test_case_inventory_does_not_claim_server_results(self) -> None:
        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        self.assertTrue(all("observed_result" not in case for case in catalog["test_cases"]))

    def test_ciphertrust_profile_is_not_a_kmip_20_or_21_target(self) -> None:
        profile = json.loads((ROOT / "interop/shared/profiles/thales-ciphertrust.json").read_text(encoding="utf-8"))
        self.assertFalse(set(profile["server"]["kmip_versions"]) & {"2.0", "2.1"})

    def test_aggregator_compares_language_specific_artifacts_built_from_same_commit(self) -> None:
        def result(language: str, artifact: str, commit: str = "abcdef0") -> dict:
            return {
                "schema_version": "2.0",
                "language": language,
                "server": {"id": "test-kms", "version": "test-1", "configuration_fingerprint": "a" * 64},
                "protocol_version": "2.1",
                "catalog_sha256": "c" * 64,
                "execution": {"library_commit": commit, "artifact_sha256": artifact, "transport": "raw_tls", "encoding": "TTLV", "api_level": "typed-protocol"},
                "cases": [{"case_id": "test-get", "status": "passed", "library_correctness": "passed", "server_compatibility": "supported", "parity": "matched"}],
            }

        with tempfile.TemporaryDirectory() as tmp:
            paths = []
            for language, artifact in (("rust", "b" * 64), ("python", "d" * 64)):
                path = Path(tmp) / f"{language}.json"
                path.write_text(json.dumps(result(language, artifact)), encoding="utf-8")
                paths.append(path)
            matrix = aggregate_results(paths)

        self.assertEqual("COMPARABLE", matrix["matrices"][0]["comparability"])
        self.assertEqual({"passed": 2}, matrix["matrices"][0]["summary"]["library_correctness"])
        self.assertEqual({"supported": 2}, matrix["matrices"][0]["summary"]["server_compatibility"])

    def test_aggregator_marks_different_library_commits_incomparable(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            paths = []
            for language, commit in (("rust", "abcdef0"), ("python", "1234567")):
                payload = {
                    "language": language,
                    "server": {"id": "test-kms", "version": "test-1", "configuration_fingerprint": "a" * 64},
                    "protocol_version": "2.1",
                    "catalog_sha256": "c" * 64,
                    "execution": {"library_commit": commit, "artifact_sha256": "b" * 64, "transport": "raw_tls", "encoding": "TTLV", "api_level": "typed-protocol"},
                    "cases": [],
                }
                path = Path(tmp) / f"{language}.json"
                path.write_text(json.dumps(payload), encoding="utf-8")
                paths.append(path)
            matrix = aggregate_results(paths)

        self.assertEqual("INCOMPARABLE", matrix["matrices"][0]["comparability"])


if __name__ == "__main__":
    unittest.main()
