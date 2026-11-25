import json
import unittest
import tempfile
from pathlib import Path

from jsonschema import Draft202012Validator

from verification.interop.shared.tools.cosmian_minimum import ROOT, build_selection
from verification.interop.shared.tools.operation_catalog import build_normative_catalog, direction
from verification.interop.shared.tools.run_all import Listener, build_command_for_language


class CosmianMinimumTests(unittest.TestCase):
    def test_every_published_scope_row_is_retained_with_versioned_cases(self):
        snapshot = json.loads((ROOT / "verification/interop/shared/catalog/cosmian-kmip2x-support.json").read_text())
        self.assertEqual(len(snapshot["rows"]), 159)
        schema_dir = ROOT / "verification/interop/shared/schemas"
        validator = Draft202012Validator(json.loads((schema_dir / "test-catalog.schema.json").read_text()))
        reports = {}
        for version in ("2.0", "2.1"):
            catalog, report = build_selection(ROOT, version)
            validator.validate(catalog)
            self.assertEqual(snapshot["source"], report["source"])
            self.assertEqual(version, report["protocol_version"])
            cases = {case["case_id"]: case for case in catalog["test_cases"]}
            self.assertEqual(len(cases), len(catalog["test_cases"]))
            self.assertTrue(all(case["execution"]["protocol_versions"] == [version] for case in cases.values()))
            self.assertEqual(
                [(row["kind"], row["name"], row[version]) for row in snapshot["rows"]],
                [(row["kind"], row["name"], row["declared_support"]) for row in report["rows"]],
            )
            for row in report["rows"]:
                self.assertLessEqual(set(row["server_cases"]) | set(row["local_cases"]), cases.keys())
                if row["declared_support"] == "not_applicable":
                    self.assertEqual(row["disposition"], "out_of_scope")
                    self.assertFalse(row["server_cases"])
                if row["kind"] == "operation":
                    expected_direction = "server_to_client" if row["name"] in {"Notify", "Put"} else "client_to_server"
                    for case_id in row["server_cases"]:
                        self.assertEqual(direction(cases[case_id]), expected_direction)
            reports[version] = {(r["kind"], r["name"]): r for r in report["rows"]}
        self.assertEqual(reports["2.0"]["attribute", "Rotate Date"]["disposition"], "out_of_scope")
        self.assertIn("test-attr-rotate-date", reports["2.1"]["attribute", "Rotate Date"]["server_cases"])
        self.assertIn("test-attr-rotate-date-normative", reports["2.1"]["attribute", "Rotate Date"]["server_cases"])
        for version in ("2.0", "2.1"):
            for name in ("Data", "Correlation Value", "Init Indicator", "Final Indicator"):
                self.assertIn("test-multipart-hash-sha256", reports[version]["base_object", name]["server_cases"])
        self.assertTrue(reports["2.1"]["operation", "Notify"]["server_cases"])
        self.assertTrue(reports["2.1"]["transparent_key", "RSA Private Key"]["normative_targets"])
        self.assertEqual(reports["2.1"]["attribute", "Attribute Index"]["disposition"], "vendor_fixture_required")

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
