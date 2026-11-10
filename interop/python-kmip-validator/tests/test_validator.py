from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import unittest

from kmipkit_validator.cli import _make_report, _plan, _run, main
from kmipkit_validator.contract import applicable_cases, load_catalog, load_connection
from kmipkit_validator.driver import EquivalenceVerifier, NormalizedResponse, PythonKmipDriver, RecipeRunner
from kmipkit_validator.reporter import write_interop_reports


ROOT = Path(__file__).resolve().parents[3]
SHARED = ROOT / "interop" / "shared"
CONNECTION = SHARED / "profiles" / "connection.example.json"
CATALOG = SHARED / "catalog" / "kmipkit-1.0-test-catalog.json"


class ValidatorContractTests(unittest.TestCase):
    def test_plan_is_offline_and_accounts_for_each_protocol_version(self):
        connection, _ = load_connection(CONNECTION)
        catalog, catalog_bytes = load_catalog(CATALOG)
        plan = _plan(connection, catalog, catalog_bytes)
        self.assertFalse(plan["network_attempted"])
        self.assertEqual(len(applicable_cases(catalog, "2.1")), len(plan["cases"]))

        connection["protocol_version"] = "2.0"
        plan_v2 = _plan(connection, catalog, catalog_bytes)
        self.assertEqual(len(applicable_cases(catalog, "2.0")), len(plan_v2["cases"]))
        self.assertNotEqual(len(plan["cases"]), len(plan_v2["cases"]))

    def test_python_driver_advertises_only_live_typed_recipes(self):
        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        self.assertTrue(PythonKmipDriver.supports(by_id["test-op-query"], "typed-protocol"))
        self.assertTrue(PythonKmipDriver.supports(by_id["test-op-ping"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-s2c-query"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-enum-cryptographic-algorithm"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-op-query"], "high-level"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-op-create"], "typed-protocol"))

    def test_python_driver_has_a_recipe_for_each_synchronous_catalog_operation(self):
        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        for case_id in (
            "test-op-create",
            "test-op-get",
            "test-op-register",
            "test-op-activate",
            "test-op-destroy",
            "test-op-locate",
            "test-obj-symmetric-key",
            "test-attr-comment",
            "test-crypto-encrypt-aes-256-cbc",
        ):
            with self.subTest(case_id=case_id):
                self.assertTrue(PythonKmipDriver.supports(by_id[case_id], "typed-protocol"))

    def test_python_mutation_gate_includes_required_setup_and_cleanup(self):
        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-op-get"]))
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-attr-comment"]))
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-op-ping"]) is False)

    def test_mutating_cases_are_policy_restricted_without_both_opt_ins(self):
        connection, connection_bytes = load_connection(CONNECTION)
        connection["api_level"] = "high-level"  # No route is eligible to create a client.
        catalog, catalog_bytes = load_catalog(CATALOG)
        artifact = {"library_commit": "52a2be180340e5da3166401a52f652dfd368ad37", "artifact_sha256": "a" * 64}
        results = _run(
            connection,
            connection_bytes,
            catalog,
            catalog_bytes,
            artifact,
            None,
            enable_mutations=False,
        )
        indexed = {item["case_id"]: item for item in results}
        mutations = [case for case in applicable_cases(catalog, "2.1") if case["execution"]["mutation"] == "mutating"]
        self.assertTrue(mutations)
        self.assertTrue(all(indexed[case["case_id"]]["status"] == "policy_restricted" for case in mutations))
        server_cases = [case for case in applicable_cases(catalog, "2.1") if case["execution"]["mode"] == "server_exchange"]
        self.assertFalse(any(indexed[case["case_id"]]["status"] == "passed" for case in server_cases))

    def test_local_enum_and_bitmask_catalog_rows_run_through_installed_ttlv_api(self):
        connection, connection_bytes = load_connection(CONNECTION)
        catalog, catalog_bytes = load_catalog(CATALOG)
        artifact = {"library_commit": "52a2be180340e5da3166401a52f652dfd368ad37", "artifact_sha256": "a" * 64}
        results = _run(
            connection,
            connection_bytes,
            catalog,
            catalog_bytes,
            artifact,
            None,
            enable_mutations=False,
        )
        local_ids = {
            case["case_id"]
            for case in applicable_cases(catalog, "2.1")
            if case["execution"]["mode"] == "local_model"
        }
        indexed = {item["case_id"]: item for item in results}
        self.assertEqual(67, len(local_ids))
        self.assertTrue(all(indexed[case_id]["status"] in {"passed", "not_run"} for case_id in local_ids))
        pending = [indexed[case_id] for case_id in local_ids if indexed[case_id]["status"] == "not_run"]
        self.assertEqual(4, len(pending))
        self.assertTrue(all("enum value map is unavailable" in case["diagnostic"] for case in pending))

    def test_result_report_matches_shared_schema_and_writes_all_formats(self):
        connection, connection_bytes = load_connection(CONNECTION)
        catalog, catalog_bytes = load_catalog(CATALOG)
        artifact = {"library_commit": "52a2be180340e5da3166401a52f652dfd368ad37", "artifact_sha256": "a" * 64}
        result = {
            "case_id": "test-op-ping",
            "status": "not_run",
            "library_correctness": "not_checked",
            "server_compatibility": "unknown",
            "parity": "not_compared",
            "duration_ms": 0,
            "diagnostic": "fixture recipe pending",
        }
        report = _make_report(connection, connection_bytes, catalog, catalog_bytes, artifact, None, [result])
        self.assertEqual(report["summary"]["not_run"], 1)
        with tempfile.TemporaryDirectory() as temporary:
            write_interop_reports(report, Path(temporary))
            data = json.loads((Path(temporary) / "results.json").read_text(encoding="utf-8"))
            self.assertEqual(data, report)
            self.assertIn("Not Run", (Path(temporary) / "report.md").read_text(encoding="utf-8"))
            self.assertIn("Server", (Path(temporary) / "report.html").read_text(encoding="utf-8"))

    def test_plan_command_requires_a_connection_and_never_opens_a_socket(self):
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(main(["plan", "--connection", str(CONNECTION)]), 0)
        planned = json.loads(output.getvalue())
        self.assertFalse(planned["network_attempted"])
        self.assertGreater(len(planned["cases"]), 0)

    def test_equivalence_verifier_and_cleanup_guard(self):
        same = NormalizedResponse(status="Success", object_type="SymmetricKey")
        different = NormalizedResponse(status="OperationFailed", result_reason="Failure")
        self.assertTrue(EquivalenceVerifier.verify_equivalence({"typed": same, "generic": same}).is_equivalent)
        self.assertFalse(EquivalenceVerifier.verify_equivalence({"typed": same, "other": different}).is_equivalent)
        cleaned = []
        with self.assertRaisesRegex(ValueError, "target"):
            RecipeRunner.execute_with_guaranteed_teardown(
                lambda: True,
                lambda: (_ for _ in ()).throw(ValueError("target")),
                lambda: cleaned.append(True) or True,
            )
        self.assertEqual([True], cleaned)


if __name__ == "__main__":
    unittest.main()
