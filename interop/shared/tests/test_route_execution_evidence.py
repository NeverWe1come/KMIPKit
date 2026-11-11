"""Real invocation evidence must discharge planned route/action cells."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
import unittest


SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[1]
sys.path.insert(0, str(SHARED / "tools"))
from validate_route_evidence import (
    _validate_batch_event,
    validate_collection,
    validate_parity_collection,
    validate_report,
)  # noqa: E402


class RouteExecutionEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.catalog = {
            "version": "2.0.0",
            "target_versions": ["2.0"],
            "test_cases": [{
                "case_id": "case-one",
                "category": "normative_inventory",
                "operation": "TTLV Roundtrip",
                "requires": {"normative_target_id": "KMIPKIT-ELEM-TAG-420001"},
                "execution": {"protocol_versions": ["2.0"]},
            }],
            "normative_targets": [],
        }
        self.manifest = {
            "library_commit": "a" * 40,
            "routes": [
                {
                    "route_id": "c.local.construct",
                    "language": "c",
                    "api_level": "generic-ttlv",
                    "mode": "local",
                    "public_api_symbol": "kmipkit_ttlv_item_create",
                    "operation_ids": [],
                    "normative_target_ids": ["*"],
                    "case_categories": ["normative_inventory"],
                    "protocol_versions": ["2.0"],
                    "combinations": [{"encoding": "TTLV", "transport": None}],
                    "actions": ["construct"],
                    "execution_route_id": None,
                    "artifact_commit": "a" * 40,
                },
                {
                    "route_id": "c.local.encode",
                    "language": "c",
                    "api_level": "typed-protocol",
                    "mode": "local",
                    "public_api_symbol": "kmipkit_extension_information_to_ttlv",
                    "operation_ids": [],
                    "normative_target_ids": ["*"],
                    "case_categories": ["normative_inventory"],
                    "protocol_versions": ["2.0"],
                    "combinations": [{"encoding": "TTLV", "transport": None}],
                    "actions": ["encode"],
                    "execution_route_id": None,
                    "artifact_commit": "a" * 40,
                },
                {
                    "route_id": "c.network.execute",
                    "language": "c",
                    "api_level": "typed-protocol",
                    "mode": "network",
                    "public_api_symbol": "kmipkit_client_batch_item_create",
                    "operation_ids": [],
                    "normative_target_ids": ["*"],
                    "case_categories": ["normative_inventory"],
                    "protocol_versions": ["2.0"],
                    "combinations": [{"encoding": "TTLV", "transport": "raw_tls"}],
                    "actions": ["construct"],
                    "execution_route_id": "c.execute",
                    "artifact_commit": "a" * 40,
                },
            ],
            "execution_routes": [{
                "execution_route_id": "c.execute",
                "language": "c",
                "public_api_symbol": "kmipkit_client_execute",
                "source_paths": ["bindings/c/include/kmipkit.h"],
            }],
            "unavailable": [],
        }
        self.report = {
            "schema_version": "1.0",
            "run_id": "run-one",
            "language": "c",
            "server": {"id": "test-kms", "product": "controlled-peer", "version": "2.1",
                       "configuration_fingerprint": "f" * 64},
            "library_commit": "a" * 40,
            "artifact_sha256": "b" * 64,
            "catalog_sha256": "c" * 64,
            "route_manifest_sha256": "d" * 64,
            "protocol_versions": ["2.0"],
            "network_combinations": [{"encoding": "TTLV", "transport": "raw_tls"}],
            "events": [
                self.event("c.local.construct", None, None, "construct", None, "request_tree_sha256"),
                self.event("c.local.encode", None, None, "encode", None, "request_tree_sha256"),
                self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "construct", "raw_tls", "request_tree_sha256"),
                self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "send", "raw_tls", "request_tree_sha256"),
                self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "decode", "raw_tls", "response_tree_sha256"),
            ],
        }

    @staticmethod
    def event(route_id, execution_route_id, execution_api_symbol, action, transport, tree_field, encoding="TTLV"):
        return {
            "case_id": "case-one",
            "route_id": route_id,
            "target_id": "KMIPKIT-ELEM-TAG-420001",
            "public_api_symbol": ("kmipkit_ttlv_item_create" if route_id == "c.local.construct" else
                                   "kmipkit_extension_information_to_ttlv" if route_id == "c.local.encode" else
                                   "kmipkit_client_batch_item_create"),
            "execution_route_id": execution_route_id,
            "execution_api_symbol": execution_api_symbol,
            "action": action,
            "protocol_version": "2.0",
            "encoding": encoding,
            "transport": transport,
            "status": "passed",
            "invocation_count": 1,
            "invocation_id": f"{route_id}:{action}",
            tree_field: "e" * 64,
        }

    def validate(self, report=None):
        return validate_report(report or self.report, self.catalog, self.manifest,
                              catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                              expected_artifact_sha256="b" * 64)

    def test_complete_report_proves_each_route_action_not_only_symbol_availability(self):
        summary = self.validate()
        self.assertTrue(summary["selected_scope_complete"])
        self.assertTrue(summary["complete_matrix"])
        self.assertTrue(summary["all_invocations_passed"])
        self.assertEqual(5, summary["expected_cells"])
        self.assertEqual(5, summary["invoked_cells"])

    def test_report_conforms_to_route_execution_evidence_schema(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        schema = json.loads((SHARED / "schemas/route-execution-evidence.schema.json").read_text())
        jsonschema.validate(self.report, schema)

    def test_missing_action_fails_coverage_even_when_the_route_was_called(self):
        report = copy.deepcopy(self.report)
        report["events"].pop()
        with self.assertRaisesRegex(ValueError, "missing route execution cells"):
            self.validate(report)

    def test_network_events_must_name_the_real_separate_execution_entrypoint(self):
        report = copy.deepcopy(self.report)
        report["events"][3]["execution_api_symbol"] = "kmipkit_client_batch_item_create"
        with self.assertRaisesRegex(ValueError, "execution API symbol"):
            self.validate(report)

    def test_artifact_digest_must_match_the_installed_artifact_provenance(self):
        report = copy.deepcopy(self.report)
        report["artifact_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "installed artifact"):
            self.validate(report)

    def test_subset_run_is_not_reported_as_complete_global_matrix(self):
        report = copy.deepcopy(self.report)
        network_route = next(route for route in self.manifest["routes"] if route["route_id"] == "c.network.execute")
        network_route["combinations"].append({"encoding": "JSON", "transport": "https"})
        summary = self.validate(report)
        self.assertTrue(summary["selected_scope_complete"])
        self.assertFalse(summary["complete_matrix"])

    def test_unavailable_encoding_transport_pair_is_rejected(self):
        report = copy.deepcopy(self.report)
        report["network_combinations"] = [{"encoding": "XML", "transport": "https"}]
        with self.assertRaisesRegex(ValueError, "absent from the public API"):
            self.validate(report)

    def test_not_run_cell_does_not_count_as_route_execution(self):
        report = copy.deepcopy(self.report)
        report["events"][0]["status"] = "not_run"
        report["events"][0]["invocation_count"] = 0
        report["events"][0]["not_run_reason"] = "API route not implemented"
        with self.assertRaisesRegex(ValueError, "not actually invoked"):
            self.validate(report)

    def test_failed_library_route_is_covered_but_fails_quality_gate(self):
        report = copy.deepcopy(self.report)
        report["events"][2]["status"] = "failed"
        report["events"][2]["failure_code"] = "KMIP_RESPONSE_SHAPE_MISMATCH"
        summary = self.validate(report)
        self.assertTrue(summary["selected_scope_complete"])
        self.assertFalse(summary["all_invocations_passed"])

    def test_duplicate_and_unplanned_invocations_are_rejected(self):
        report = copy.deepcopy(self.report)
        report["events"].append(copy.deepcopy(report["events"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate route execution cell"):
            self.validate(report)
        report = copy.deepcopy(self.report)
        report["events"][0]["route_id"] = "c.not-a-route"
        with self.assertRaisesRegex(ValueError, "unplanned route execution cell"):
            self.validate(report)

    def test_construct_and_decode_events_require_normalized_tree_hashes(self):
        report = copy.deepcopy(self.report)
        del report["events"][0]["request_tree_sha256"]
        with self.assertRaisesRegex(ValueError, "request tree hash"):
            self.validate(report)
        report = copy.deepcopy(self.report)
        del report["events"][4]["response_tree_sha256"]
        with self.assertRaisesRegex(ValueError, "response tree hash"):
            self.validate(report)

    def test_deterministic_wire_byte_claim_requires_a_valid_digest(self):
        report = copy.deepcopy(self.report)
        report["events"][3]["wire_bytes_deterministic"] = True
        with self.assertRaisesRegex(ValueError, "deterministic wire bytes"):
            self.validate(report)
        report["events"][3]["wire_bytes_sha256"] = "f" * 64
        self.assertTrue(self.validate(report)["complete_matrix"])

    def test_multiple_subset_reports_must_jointly_cover_the_full_matrix(self):
        network_route = next(route for route in self.manifest["routes"] if route["route_id"] == "c.network.execute")
        network_route["combinations"].append({"encoding": "JSON", "transport": "https"})
        second = copy.deepcopy(self.report)
        second["run_id"] = "run-two"
        second["network_combinations"] = [{"encoding": "JSON", "transport": "https"}]
        second["events"] = [copy.deepcopy(self.report["events"][0]), copy.deepcopy(self.report["events"][1])]
        second["events"][0]["invocation_id"] = "run-two:local-construct"
        second["events"][1]["invocation_id"] = "run-two:local-encode"
        second["events"].extend([
            self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "construct", "https", "request_tree_sha256", "JSON"),
            self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "send", "https", "request_tree_sha256", "JSON"),
            self.event("c.network.execute", "c.execute", "kmipkit_client_execute", "decode", "https", "response_tree_sha256", "JSON"),
        ])
        summary = validate_collection(
            [self.report, second], self.catalog, self.manifest,
            catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
            expected_artifact_sha256="b" * 64,
        )
        self.assertTrue(summary["complete_matrix"])
        self.assertEqual("f" * 64, summary["server"]["configuration_fingerprint"])
        self.assertEqual("a" * 40, summary["library_commit"])
        self.assertEqual("b" * 64, summary["artifact_sha256"])
        with self.assertRaisesRegex(ValueError, "misses full matrix cells"):
            validate_collection(
                [self.report], self.catalog, self.manifest,
                catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
            )
        conflict = copy.deepcopy(self.report)
        conflict["run_id"] = "run-conflict"
        conflict["events"][3]["status"] = "failed"
        conflict["events"][3]["failure_code"] = "DIFFERENT_RESPONSE"
        with self.assertRaisesRegex(ValueError, "conflicting repeated route execution cell"):
            validate_collection(
                [self.report, conflict], self.catalog, self.manifest,
                catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
            )

    def test_collection_rejects_different_server_identities(self):
        second = copy.deepcopy(self.report)
        second["run_id"] = "run-two"
        second["server"]["id"] = "different-server"
        with self.assertRaisesRegex(ValueError, "different KMIP servers"):
            validate_collection(
                [self.report, second], self.catalog, self.manifest,
                catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
            )

    def test_collection_rejects_different_server_configuration_fingerprints(self):
        second = copy.deepcopy(self.report)
        second["run_id"] = "run-two"
        second["server"]["configuration_fingerprint"] = "1" * 64
        with self.assertRaisesRegex(ValueError, "different server configurations"):
            validate_collection(
                [self.report, second], self.catalog, self.manifest,
                catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
            )

    def test_controlled_peer_evidence_preserves_repeated_batch_item_order(self):
        case = {
            "case_id": "batch-get-stop",
            "requires": {"protocol_scenario": {
                "parameters": {"items": ["Get(missing)", "Get(existing)"]},
                "expected": {"first": "OperationFailed", "second": "OperationUndone", "response_count": 2},
            }},
        }
        event = {
            "batch_operation_sequence": ["Get", "Get"],
            "peer_observed_operation_sequence": ["Get", "Get"],
            "batch_item_count": 2,
            "peer_observed_response_count": 2,
        }
        catalog = {"normative_targets": [
            {"kind": "enumeration_value", "name": "Operation Failed", "wire_value": "00000001",
             "parent_element_ids": ["KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS"]},
            {"kind": "enumeration_value", "name": "Operation Undone", "wire_value": "00000003",
             "parent_element_ids": ["KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS"]},
        ]}
        event["peer_observed_result_status_codes"] = [1, 3]
        _validate_batch_event(event, case, catalog, ("batch-get-stop",))
        event["peer_observed_operation_sequence"] = ["Get"]
        with self.assertRaisesRegex(ValueError, "controlled peer did not observe"):
            _validate_batch_event(event, case, catalog, ("batch-get-stop",))
        event["peer_observed_operation_sequence"] = ["Get", "Get"]
        event["peer_observed_result_status_codes"] = [0, 3]
        with self.assertRaisesRegex(ValueError, "per-item status outcomes"):
            _validate_batch_event(event, case, catalog, ("batch-get-stop",))

    def _four_language_fixture(self):
        reports = []
        base_routes = copy.deepcopy(self.manifest["routes"])
        base_execution = copy.deepcopy(self.manifest["execution_routes"])
        manifest = {"library_commit": self.manifest["library_commit"], "routes": [],
                    "execution_routes": [], "unavailable": []}
        for language in ("rust", "c", "java", "python"):
            route_ids = {route["route_id"]: route["route_id"].replace("c.", f"{language}.", 1)
                         for route in base_routes}
            execution_ids = {item["execution_route_id"]: f"{language}.execute"
                             for item in base_execution}
            language_routes = []
            for base_route in base_routes:
                route = copy.deepcopy(base_route)
                route["route_id"] = route_ids[base_route["route_id"]]
                route["language"] = language
                if route.get("execution_route_id"):
                    route["execution_route_id"] = execution_ids[route["execution_route_id"]]
                language_routes.append(route)
            manifest["routes"].extend(language_routes)
            manifest["execution_routes"].extend([
                {**item, "execution_route_id": f"{language}.execute", "language": language,
                 "public_api_symbol": f"{language}_execute"}
                for item in base_execution
            ])
            report = copy.deepcopy(self.report)
            report["language"] = language
            report["run_id"] = f"{language}-run"
            report["artifact_sha256"] = ({"rust": "1", "c": "2", "java": "3", "python": "4"}[language]) * 64
            for event in report["events"]:
                event["route_id"] = route_ids[event["route_id"]]
                if event.get("execution_route_id"):
                    event["execution_route_id"] = execution_ids[event["execution_route_id"]]
                    event["execution_api_symbol"] = f"{language}_execute"
            reports.append(report)
        return reports, manifest

    def test_cross_language_parity_requires_same_normalized_request_and_response_trees(self):
        reports, manifest = self._four_language_fixture()
        artifact_digests = {report["language"]: report["artifact_sha256"] for report in reports}
        summary = validate_parity_collection(
            reports, self.catalog, manifest, catalog_sha256="c" * 64,
            route_manifest_sha256="d" * 64,
            expected_artifact_sha256_by_language=artifact_digests,
        )
        self.assertTrue(summary["tree_parity_passed"])
        self.assertGreater(summary["parity_groups_all_languages"], 0)

        reports, manifest = self._four_language_fixture()
        reports[0]["events"][3]["request_tree_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "constructed request tree differs"):
            validate_parity_collection(
                reports, self.catalog, manifest, catalog_sha256="c" * 64,
                route_manifest_sha256="d" * 64,
                expected_artifact_sha256_by_language={report["language"]: report["artifact_sha256"]
                                                      for report in reports},
            )

        reports, manifest = self._four_language_fixture()
        for event in reports[-1]["events"]:
            if event["action"] in {"construct", "send"}:
                event["request_tree_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "languages disagree"):
            validate_parity_collection(
                reports, self.catalog, manifest, catalog_sha256="c" * 64,
                route_manifest_sha256="d" * 64,
                expected_artifact_sha256_by_language={report["language"]: report["artifact_sha256"]
                                                      for report in reports},
            )


if __name__ == "__main__":
    unittest.main()
