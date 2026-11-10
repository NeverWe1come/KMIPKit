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
        cases = {
            case["operation"]: case
            for case in catalog["test_cases"]
            if case["execution"]["mode"] == "server_exchange"
        }
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        self.assertTrue(PythonKmipDriver.supports(cases["Query"], "typed-protocol"))
        self.assertTrue(PythonKmipDriver.supports(cases["Ping"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-s2c-query"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(by_id["test-enum-cryptographic-algorithm"], "typed-protocol"))
        self.assertFalse(PythonKmipDriver.supports(cases["Query"], "high-level"))
        self.assertFalse(PythonKmipDriver.supports(cases["Create"], "typed-protocol"))

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
        self.assertFalse(any(item["status"] == "passed" for item in results))

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
