from contextlib import redirect_stdout
import io
import importlib
import json
from pathlib import Path
import tempfile
import unittest

from kmipkit_validator.cli import _make_report, _plan, _run, main
from kmipkit_validator.contract import applicable_cases, load_catalog, load_connection
from kmipkit_validator.driver import (
    EquivalenceVerifier,
    LocalRecipeUnavailable,
    NormalizedResponse,
    PythonKmipDriver,
    RecipeRunner,
)
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
        self.assertTrue(PythonKmipDriver.supports(by_id["test-op-create"], "typed-protocol"))

    def test_python_driver_exercises_the_public_high_level_attribute_requests(self):
        from kmipkit import extensions

        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        driver = object.__new__(PythonKmipDriver)
        driver._extensions = extensions
        driver._api_level = "high-level"
        identifier = extensions.UniqueIdentifier.text_string("dedicated-fixture")
        for case_id in (
            "test-op-add-attribute",
            "test-op-adjust-attribute",
            "test-op-delete-attribute",
            "test-op-get-attributes",
            "test-op-get-attribute-list",
            "test-op-modify-attribute",
            "test-op-set-attribute",
            "test-attr-comment",
        ):
            with self.subTest(case_id=case_id):
                self.assertTrue(PythonKmipDriver.supports(by_id[case_id], "high-level"))
                item = driver._build_high_level_item(by_id[case_id], identifier, name="dedicated-fixture")
                self.assertIsInstance(item, extensions.ClientBatchItem)
                with item.validation_report() as report:
                    self.assertTrue(report.is_valid)
                item.close()

    def test_python_driver_advertises_catalogued_async_client_routes(self):
        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        for case_id in (
            "test-op-poll",
            "test-op-process",
            "test-op-cancel",
            "test-op-query-asynchronous-requests",
            "test-async-lifecycle-poll",
        ):
            with self.subTest(case_id=case_id):
                self.assertTrue(PythonKmipDriver.supports(by_id[case_id], "typed-protocol"))

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
            "test-op-derive-key",
            "test-op-join-split-key",
        ):
            with self.subTest(case_id=case_id):
                self.assertTrue(PythonKmipDriver.supports(by_id[case_id], "typed-protocol"))

    def test_every_advertised_case_has_a_public_request_recipe_or_special_runner(self):
        from kmipkit import extensions

        catalog, _ = load_catalog(CATALOG)
        driver = object.__new__(PythonKmipDriver)
        driver._extensions = extensions
        identifier = extensions.UniqueIdentifier.text_string("dedicated-fixture")
        advertised = [
            case
            for case in catalog["test_cases"]
            if PythonKmipDriver.supports(case, "typed-protocol")
        ]
        self.assertGreater(len(advertised), 100)
        for case in advertised:
            if case["operation"] in {"Logout", "Poll", "Process", "Cancel", "Query Asynchronous Requests"} or case["requires"].get("workflow_type") == "async_poll":
                continue  # These routes require a server-issued Ticket or async correlation.
            with self.subTest(case_id=case["case_id"]):
                item = driver._build_item(case, identifier, name="dedicated-fixture")
                self.assertIsInstance(item, extensions.ClientBatchItem)
                with item.validation_report() as report:
                    self.assertTrue(report.is_valid)
                item.close()

    def test_every_catalogued_python_network_request_constructor_is_invoked(self):
        from kmipkit import extensions

        catalog, _ = load_catalog(CATALOG)
        manifest = json.loads((SHARED / "coverage" / "route-capabilities.json").read_text())
        expected = {
            route["public_api_symbol"]
            for route in manifest["routes"]
            if route["language"] == "python"
            and route["mode"] == "network"
            and "construct" in route["actions"]
            and route["public_api_symbol"].startswith("kmipkit.extensions.client_batch_item_")
        }
        invoked = set()
        originals = {}
        for symbol in expected:
            owner_name, attribute = symbol.rsplit(".", 1)
            owner = importlib.import_module(owner_name)
            original = getattr(owner, attribute)

            def track_call(*args, _original=original, _symbol=symbol, **kwargs):
                invoked.add(_symbol)
                return _original(*args, **kwargs)

            originals[(owner, attribute)] = original
            setattr(owner, attribute, track_call)

        driver = object.__new__(PythonKmipDriver)
        driver._extensions = extensions
        identifier = extensions.UniqueIdentifier.text_string("constructor-route-fixture")
        try:
            for case in catalog["test_cases"]:
                if not PythonKmipDriver.supports(case, "typed-protocol"):
                    continue
                if case["operation"] in {
                    "Logout", "Poll", "Process", "Cancel", "Query Asynchronous Requests"
                } or case["requires"].get("workflow_type") == "async_poll":
                    continue
                item = driver._build_item(case, identifier, name="constructor-route-fixture")
                item.close()

            # Logout needs a Ticket supplied by a Login response in a live run.
            # A structurally valid fixture still verifies its public constructor offline.
            ticket_type = driver._item(0x0042_014A, "enumeration", 1)
            ticket_value = driver._item(0x0042_014B, "bytes", b"fixture-ticket")
            ticket = driver._structure([ticket_type, ticket_value])
            try:
                logout = extensions.client_batch_item_logout(ticket)
                with logout.validation_report() as report:
                    self.assertTrue(report.is_valid)
                logout.close()
            finally:
                ticket.close()
                ticket_type.close()
                ticket_value.close()
        finally:
            for (owner, attribute), original in originals.items():
                setattr(owner, attribute, original)

        self.assertEqual(expected, invoked, f"uninvoked Python request routes: {sorted(expected - invoked)}")

    def test_python_mutation_gate_includes_required_setup_and_cleanup(self):
        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-op-get"]))
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-attr-comment"]))
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-op-poll"]))
        self.assertTrue(PythonKmipDriver.requires_mutation(by_id["test-op-ping"]) is False)

    def test_public_typed_recipes_construct_real_batch_items_without_network(self):
        from kmipkit import extensions

        catalog, _ = load_catalog(CATALOG)
        by_id = {case["case_id"]: case for case in catalog["test_cases"]}
        driver = object.__new__(PythonKmipDriver)
        driver._extensions = extensions
        identifier = extensions.UniqueIdentifier.text_string("dedicated-fixture")
        for case_id in (
            "test-op-create",
            "test-op-destroy",
            "test-op-locate",
            "test-attr-comment",
            "test-crypto-encrypt-aes-256-cbc",
            "test-op-re-provision",
            "test-obj-certificate",
            "test-op-derive-key",
            "test-op-join-split-key",
        ):
            with self.subTest(case_id=case_id):
                item = driver._build_item(by_id[case_id], identifier, name="dedicated-fixture")
                self.assertIsInstance(item, extensions.ClientBatchItem)
                item.close()

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

    def test_local_catalog_rows_execute_or_report_a_specific_public_api_gap(self):
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
        expanded_contract = bool(catalog.get("normative_targets"))
        self.assertGreater(len(local_ids), 2000 if expanded_contract else 60)
        gaps = (
            {
                "test-batch-count-zero-rejected",
                "test-batch-single-item-options-omitted",
                "test-batch-placeholder-forward-reference-rejected",
                "test-batch-placeholder-without-producer-rejected",
            }
            if expanded_contract
            else set()
        )
        self.assertEqual(
            {case_id for case_id in local_ids if indexed[case_id]["status"] == "not_run"},
            gaps,
        )
        self.assertTrue(
            all(
                indexed[case_id]["status"] == "passed"
                or (
                    indexed[case_id]["status"] == "not_run"
                    and indexed[case_id]["diagnostic"].startswith("API gap:")
                    and indexed[case_id]["library_correctness"] == "not_checked"
                )
                for case_id in local_ids
            ),
            [indexed[case_id] for case_id in sorted(local_ids) if indexed[case_id]["status"] not in {"passed", "not_run"}][:20],
        )

    def test_local_kmip_20_version_boundary_cases_have_documented_api_limits_when_catalogued(self):
        catalog, _ = load_catalog(CATALOG)
        cases = [
            case
            for case in applicable_cases(catalog, "2.0")
            if case["category"] == "version_boundaries"
            and case["execution"]["mode"] == "local_model"
        ]
        if not cases and not catalog.get("normative_targets"):
            return
        self.assertEqual(len(cases), 6)
        for case in cases:
            with self.subTest(case_id=case["case_id"]):
                with self.assertRaisesRegex(LocalRecipeUnavailable, "^API gap:"):
                    PythonKmipDriver.execute_local_case(case)

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
        with tempfile.TemporaryDirectory(dir=Path.cwd() / "build") as temporary:
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

    def test_preflight_includes_the_same_offline_plan_and_local_api_gaps(self):
        output = io.StringIO()
        with redirect_stdout(output):
            self.assertEqual(main(["preflight", "--connection", str(CONNECTION)]), 1)
        report = json.loads(output.getvalue())
        self.assertFalse(report["network_attempted"])
        cases = {case["case_id"]: case for case in report["plan"]["cases"]}
        expected = {
            "test-batch-count-zero-rejected",
            "test-batch-single-item-options-omitted",
            "test-batch-placeholder-forward-reference-rejected",
            "test-batch-placeholder-without-producer-rejected",
        }
        if not load_catalog(CATALOG)[0].get("normative_targets"):
            return
        for case_id in expected:
            with self.subTest(case_id=case_id):
                self.assertEqual(cases[case_id]["status"], "not_run_documented_api_gap")
                self.assertTrue(cases[case_id]["limitation"].startswith("API gap:"))

    def test_plan_documents_each_route_that_the_pinned_python_api_cannot_emit(self):
        connection, _ = load_connection(CONNECTION)
        catalog, catalog_bytes = load_catalog(CATALOG)
        plan = _plan(connection, catalog, catalog_bytes)
        missing = [case for case in plan["cases"] if not case["route_available"]]
        self.assertTrue(missing)
        self.assertTrue(all(case["status"] == "not_run_documented_api_gap" for case in missing))
        self.assertTrue(all(case["limitation"] for case in missing))

    def test_typed_plan_names_the_known_server_role_batch_and_multipart_api_gaps(self):
        connection, _ = load_connection(CONNECTION)
        connection["api_level"] = "typed-protocol"
        catalog, catalog_bytes = load_catalog(CATALOG)
        plan = _plan(connection, catalog, catalog_bytes)
        by_id = {case["case_id"]: case for case in plan["cases"]}
        for case_id, phrase in (
            ("test-s2c-query", "server-initiated"),
            ("test-batch-create-activate-destroy", "ID Placeholder"),
            ("test-multipart-encrypt-aes-cbc", "multipart Init"),
        ):
            with self.subTest(case_id=case_id):
                self.assertFalse(by_id[case_id]["route_available"])
                self.assertIn(phrase, by_id[case_id]["limitation"])

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
