import json
import unittest
import tempfile
from pathlib import Path

from jsonschema import Draft202012Validator

from verification.interop.shared.tools.catalog_reports import recipe_digest
from verification.interop.shared.tools.cosmian_minimum import (
    ROOT, attach_oasis_operation_evidence, build_selection, with_execution_evidence, write_json,
)
from verification.interop.shared.tools.operation_catalog import build_normative_catalog, direction
from verification.interop.shared.tools.table_asymmetric_key_exchanges import FORMATS
from verification.interop.shared.tools.table_authenticated_encryption import AAD_CASES
from verification.interop.shared.tools.table_key_block_exchanges import TRANSPARENT_KEY, WRAPPED_KEY
from verification.interop.shared.tools.run_all import (
    Listener, build_command_for_language, incomplete_case_ids, table_operation_expectations,
)


class CosmianMinimumTests(unittest.TestCase):
    def test_every_published_scope_row_is_retained_with_versioned_cases(self):
        snapshot = json.loads((ROOT / "verification/interop/shared/catalog/cosmian-kmip2x-support.json").read_text())
        self.assertEqual(len(snapshot["rows"]), 159)
        schema_dir = ROOT / "verification/interop/shared/schemas"
        validator = Draft202012Validator(json.loads((schema_dir / "test-catalog.schema.json").read_text()))
        reports = {}
        case_maps = {}
        for version in ("2.0", "2.1"):
            catalog, report = build_selection(ROOT, version)
            validator.validate(catalog)
            self.assertEqual(snapshot["source"], report["source"])
            self.assertEqual(version, report["protocol_version"])
            cases = {case["case_id"]: case for case in catalog["test_cases"]}
            case_maps[version] = cases
            self.assertEqual(len(cases), len(catalog["test_cases"]))
            self.assertTrue(all(case["execution"]["protocol_versions"] == [version] for case in cases.values()))
            self.assertEqual(
                [(row["kind"], row["name"], row[version]) for row in snapshot["rows"]],
                [(row["kind"], row["name"], row["declared_support"]) for row in report["rows"]],
            )
            for row in report["rows"]:
                routes = {(item["language"], item["api_level"]) for item in row["obligations"]
                          if item["api_level"] != "high-level"}
                self.assertEqual(routes, {(language, api) for language in ("rust", "c", "java", "python")
                                          for api in ("typed-protocol", "fluent", "generic-ttlv")})
                self.assertLessEqual(set(row["server_cases"]) | set(row["local_cases"]), cases.keys())
                if row["declared_support"] == "not_applicable":
                    self.assertEqual(row["disposition"], "version_boundary_required")
                    self.assertEqual({o["expectation"] for o in row["obligations"]}, {"verify_version_boundary"})
                    self.assertFalse(row["server_cases"])
                if row["kind"] == "operation":
                    expected_direction = "server_to_client" if row["name"] in {"Notify", "Put"} else "client_to_server"
                    for case_id in row["server_cases"]:
                        self.assertEqual(direction(cases[case_id]), expected_direction)
            reports[version] = {(r["kind"], r["name"]): r for r in report["rows"]}
        rotate_date = reports["2.0"]["attribute", "Rotate Date"]
        self.assertEqual(rotate_date["disposition"], "version_boundary_required")
        high_level, = [route for route in rotate_date["obligations"] if route["api_level"] == "high-level"]
        self.assertEqual(("rust", ["test-v20-locate-rotate-date"]),
                         (high_level["language"], high_level["case_ids"]))
        self.assertIn("test-attr-rotate-date", reports["2.1"]["attribute", "Rotate Date"]["server_cases"])
        self.assertIn("test-attr-rotate-date-normative", reports["2.1"]["attribute", "Rotate Date"]["server_cases"])
        for version in ("2.0", "2.1"):
            for name in ("Data", "Correlation Value", "Init Indicator", "Final Indicator"):
                self.assertIn("test-multipart-hash-sha256", reports[version]["base_object", name]["server_cases"])
            for field in ("Authenticated Encryption Additional Data", "Authenticated Encryption Tag"):
                row = reports[version]["base_object", field]
                self.assertLessEqual(set(AAD_CASES), set(row["server_cases"]))
                high_level = next(route for route in row["obligations"] if route["api_level"] == "high-level")
                self.assertEqual("rust", high_level["language"])
                self.assertLessEqual(set(AAD_CASES), set(high_level["case_ids"]))
            for field, case_ids in (
                ("Key Block", {TRANSPARENT_KEY, WRAPPED_KEY}),
                ("Key Value", {TRANSPARENT_KEY, WRAPPED_KEY}),
                ("Key Wrapping Specification", {WRAPPED_KEY}),
                ("Key Wrapping Data", {WRAPPED_KEY}),
            ):
                row = reports[version]["base_object", field]
                self.assertLessEqual(case_ids, set(row["server_cases"]))
                self.assertIn("rust", {item["language"] for item in row["obligations"]
                                       if item["api_level"] == "high-level"})
            self.assertIn(TRANSPARENT_KEY, reports[version]["transparent_key", "Symmetric Key"]["server_cases"])
            for name, format_code in FORMATS.items():
                row = reports[version]["transparent_key", name]
                case_id = f"test-op-get-transparent-{'-'.join(name.lower().split())}"
                self.assertIn(case_id, row["server_cases"])
                self.assertTrue(any(case_id in item.get("case_ids", ()) for item in row["obligations"]
                                    if (item["language"], item["api_level"]) == ("rust", "high-level")))
                self.assertEqual(str(format_code), case_maps[version][case_id]["requires"]["key_material_format"])
            for name in ("Server Information", "Extension Information", "RNG Parameters",
                         "Validation Information", "Profile Information", "Capability Information"):
                case_id = f"test-query-{'-'.join(name.lower().split())}"
                row = reports[version]["base_object", name]
                self.assertIn(case_id, row["server_cases"])
                self.assertNotIn(case_id, reports[version]["operation", "Query"]["server_cases"])
                self.assertIn(case_id, next(
                    route["case_ids"] for route in row["obligations"]
                    if route["language"] == "rust" and route["api_level"] == "high-level"
                ))
            credential_id = "test-query-username-password-credential"
            credential = reports[version]["base_object", "Credential"]
            self.assertIn(credential_id, credential["server_cases"])
            self.assertNotIn(credential_id, reports[version]["operation", "Query"]["server_cases"])
            self.assertEqual({"verify_unsupported_response"}, {route["expectation"] for route in credential["obligations"]})
            self.assertIn("request_contains_the_configured_username_and_password_credential",
                          case_maps[version][credential_id]["execution"]["assertions"])
            for name in ("Lease Time", "Last Change Date"):
                self.assertIn("test-op-obtain-lease", reports[version]["attribute", name]["server_cases"])
        self.assertTrue(reports["2.1"]["operation", "Notify"]["server_cases"])
        self.assertTrue(reports["2.1"]["transparent_key", "RSA Private Key"]["normative_targets"])
        self.assertEqual(reports["2.1"]["attribute", "Attribute Index"]["disposition"], "vendor_fixture_required")

    def test_report_writer_only_creates_new_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            report = path / "report.json"
            write_json(report, {"result": "observed"})
            with self.assertRaises(FileExistsError):
                write_json(report, {"result": "changed"})
            self.assertEqual({"result": "observed"}, json.loads(report.read_text()))
            with self.assertRaises(ValueError):
                write_json(path / "..", {})
            alias = path / "alias.json"
            try:
                alias.symlink_to(report)
            except OSError:
                self.skipTest("Symlink creation is unavailable")
            with self.assertRaises(FileExistsError):
                write_json(alias, {"result": "changed"})
            self.assertEqual({"result": "observed"}, json.loads(report.read_text()))

    def test_table_completion_requires_the_declared_operation_result(self):
        catalog, inventory = build_selection(ROOT, "2.1")
        expectations = table_operation_expectations(inventory)
        query, archive = "test-op-query", "test-op-archive"
        cases = [case for case in catalog["test_cases"] if case["case_id"] in {query, archive}]
        selected = {"test_cases": cases}
        success = {"status": "passed", "library_correctness": "passed", "result_status_code": 0}
        unsupported = {"status": "server_unsupported", "library_correctness": "passed",
                       "result_status_code": 1, "result_reason_code": 5}
        results = {
            "language": "c", "execution": {"api_level": "generic-ttlv"},
            "cases": [{"case_id": query, **unsupported}, {"case_id": archive, **success}],
        }
        self.assertEqual([archive, query], incomplete_case_ids(selected, results, expectations))
        results["cases"] = [{"case_id": query, **success}, {"case_id": archive, **unsupported}]
        self.assertEqual([], incomplete_case_ids(selected, results, expectations))

    def test_row_evidence_matches_the_report_version_language_and_api(self):
        catalog, inventory = build_selection(ROOT, "2.1")
        recipe = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-archive")
        run = {
            "run_id": "controlled-archive", "protocol_version": "2.1", "language": "c",
            "scope": "controlled", "catalog_sha256": "a" * 64,
            "execution": {"api_level": "generic-ttlv", "library_commit": "b" * 40,
                          "artifact_sha256": "c" * 64},
            "cases": [{"case_id": "test-op-archive", "status": "server_unsupported",
                       "result_status_code": 1, "result_reason_code": 5,
                       "assertions": recipe["execution"]["assertions"], "recipe_sha256": recipe_digest(recipe)}],
        }
        mismatched = {**run, "run_id": "different-assertions",
                      "cases": [{**run["cases"][0], "assertions": ["model_only"]}]}
        different_fixture = {**run, "run_id": "different-fixture",
                             "cases": [{**run["cases"][0], "recipe_sha256": "0" * 64}]}
        report = with_execution_evidence(
            inventory, catalog, [run, {**run, "protocol_version": "2.0"}, mismatched, different_fixture],
        )
        archive = next(row for row in report["rows"] if row["kind"] == "operation" and row["name"] == "Archive")
        matched = [item for item in archive["obligations"] if item["case_results"]]
        evidence, = matched
        self.assertEqual((evidence["language"], evidence["api_level"]), ("c", "generic-ttlv"))
        self.assertEqual(evidence["expectation"], "verify_unsupported_response")
        result, = evidence["case_results"]
        self.assertEqual(result["result_reason_code"], 5)
        self.assertEqual(result["scope"], "controlled")

    def test_high_level_results_require_a_distinct_observed_route(self):
        catalog, inventory = build_selection(ROOT, "2.1")
        case = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-query")
        evidence = {"case_id": case["case_id"], "assertions": case["execution"]["assertions"],
                    "recipe_sha256": recipe_digest(case), "status": "passed", "library_correctness": "passed"}
        run = {
            "run_id": "high-query", "scope": "controlled", "protocol_version": "2.1", "language": "rust",
            "catalog_sha256": "a" * 64,
            "execution": {"api_level": "high-level", "library_commit": "b" * 40, "artifact_sha256": "c" * 64},
        }
        reports = [
            {**run, "cases": [{**evidence, "actual_api_level": "high-level"}]},
            {**run, "run_id": "typed-fallback", "cases": [{**evidence, "actual_api_level": "typed-protocol"}]},
            {**run, "run_id": "missing-api", "cases": [evidence]},
        ]
        result = with_execution_evidence(inventory, catalog, reports)
        query = next(row for row in result["rows"] if row["kind"] == "operation" and row["name"] == "Query")
        by_language = {item["language"]: item for item in query["obligations"] if item["api_level"] == "high-level"}
        self.assertEqual(set(by_language), {"c", "java", "python", "rust"})
        obligation = by_language["rust"]
        self.assertEqual([item["run_id"] for item in obligation["case_results"]], ["high-query"])
        self.assertEqual([item["run_id"] for item in obligation["unverified_route_results"]],
                         ["typed-fallback", "missing-api"])

    def test_oasis_fixture_evidence_preserves_operation_variants_and_scope(self):
        _, report = build_selection(ROOT, "2.1")
        artifact = {"public_api_symbol": "kmipkit::ttlv::Item::new"}
        scenario = {
            "kind": "scenario_item", "protocol_version": "2.1", "operation": "ModifyAttribute",
            "fixture": {"path": "fixture.xml", "sha256": "a" * 64}, "step_index": 2, "item_index": 1,
            "checks": {"rust": {"fixture_construction": {"request": "passed", "response": "passed"}}},
        }
        campaign = {
            "rows": [{}, scenario, {**scenario, "protocol_version": "2.0"}, {**scenario, "operation": "Notify"}],
            "oasis_ttlv_construction": {"artifacts": {"rust": artifact}},
        }
        attach_oasis_operation_evidence(report, campaign)
        matches = [(row, obligation) for row in report["rows"] for obligation in row["obligations"]
                   if obligation.get("oasis_fixture_results")]
        (row, obligation), = matches
        self.assertEqual(row["name"], "Set Attribute (Modify)")
        self.assertEqual((obligation["language"], obligation["api_level"]), ("rust", "generic-ttlv"))
        result, = obligation["oasis_fixture_results"]
        self.assertEqual((result["scope"], result["operation"], result["batch_item"]),
                         ("local", "ModifyAttribute", 1))

    def test_full_catalog_retains_distinct_attribute_policy_obligations(self):
        for version, attributes in (("2.0", 56), ("2.1", 63)):
            with self.subTest(version=version):
                catalog, obligations = build_normative_catalog(ROOT, version)
                by_id = {case["case_id"]: case for case in catalog["test_cases"]}
                policy_cases = [case for case in obligations if "attribute_policy" in case["requires"]]
                self.assertEqual(len(policy_cases), attributes)
                for case in policy_cases:
                    self.assertEqual(by_id[case["case_id"]]["execution"]["assertions"], case["execution"]["assertions"])
                    baseline = case["requires"].get("baseline_case_id")
                    if baseline:
                        self.assertIn(baseline, by_id)
                        self.assertNotEqual(case["execution"]["assertions"], by_id[baseline]["execution"]["assertions"])

    def test_orchestrator_routes_listeners_to_each_result_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            listener = Listener("127.0.0.1:0", workspace / "certificate.pem", workspace / "key.pem")
            for language in ("rust", "c", "java", "python"):
                output = workspace / language / "results.json"
                command = build_command_for_language(language, ROOT, workspace / "profile.json",
                    workspace / "connection.json", workspace / "catalog.json", workspace / "vectors.json",
                    output, c_catalog_override=True, listener=listener)
                self.assertEqual(command[command.index("--listen") + 1], listener.address)
                self.assertEqual(command[command.index("--listener-cert") + 1], str(listener.certificate))
                self.assertEqual(command[command.index("--listener-key") + 1], str(listener.key))
                self.assertEqual(command[command.index("--listener-ready-file") + 1],
                                 str(output.parent / "listener-address.txt"))
