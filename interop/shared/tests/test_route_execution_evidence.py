"""Real invocation evidence must discharge planned route/action cells."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest


SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[1]
sys.path.insert(0, str(SHARED / "tools"))
from validate_route_evidence import (
    _expected_batch_operations,
    _expected_tag_decode_rejection_code,
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

    def test_scoped_report_validates_exact_cells_without_claiming_full_matrix(self):
        scope = self.report["events"][:2]
        report = copy.deepcopy(self.report)
        report["events"] = copy.deepcopy(scope)
        summary = validate_report(
            report,
            self.catalog,
            self.manifest,
            catalog_sha256="c" * 64,
            route_manifest_sha256="d" * 64,
            expected_artifact_sha256="b" * 64,
            scope_cells=scope,
        )
        self.assertTrue(summary["selected_scope_complete"])
        self.assertFalse(summary["complete_matrix"])
        self.assertEqual(2, summary["expected_cells"])
        self.assertEqual(2, summary["invoked_cells"])

    def test_scoped_report_rejects_missing_or_out_of_scope_events(self):
        scope = self.report["events"][:2]
        report = copy.deepcopy(self.report)
        report["events"] = copy.deepcopy(scope)
        report["events"].pop(0)
        with self.assertRaisesRegex(ValueError, "missing route execution cells"):
            validate_report(
                report,
                self.catalog,
                self.manifest,
                catalog_sha256="c" * 64,
                route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
                scope_cells=scope,
            )
        report = copy.deepcopy(self.report)
        report["events"] = copy.deepcopy(scope)
        report["events"].append(copy.deepcopy(self.report["events"][-1]))
        with self.assertRaisesRegex(ValueError, "outside the declared route scope"):
            validate_report(
                report,
                self.catalog,
                self.manifest,
                catalog_sha256="c" * 64,
                route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
                scope_cells=scope,
            )

    def test_report_conforms_to_route_execution_evidence_schema(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        schema = json.loads((SHARED / "schemas/route-execution-evidence.schema.json").read_text())
        jsonschema.validate(self.report, schema)

    def test_single_item_batch_conforms_to_route_execution_evidence_schema(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        report = copy.deepcopy(self.report)
        send_event = next(event for event in report["events"] if event["action"] == "send")
        send_event.update({
            "batch_operation_sequence": ["Get"],
            "peer_observed_operation_sequence": ["Get"],
            "batch_item_count": 1,
            "peer_observed_response_count": 1,
        })
        schema = json.loads((SHARED / "schemas/route-execution-evidence.schema.json").read_text())
        jsonschema.validate(report, schema)

    def test_batch_effects_schema_requires_both_sha256_peer_state_hashes(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        report = copy.deepcopy(self.report)
        send_event = next(event for event in report["events"] if event["action"] == "send")
        peer_state = json.dumps({"objects": ["controlled-fixture-key"]},
                                sort_keys=True, separators=(",", ":")).encode()
        peer_state_sha256 = hashlib.sha256(peer_state).hexdigest()
        send_event.update({
            "batch_operation_sequence": ["Get", "Get"],
            "peer_observed_operation_sequence": ["Get", "Get"],
            "batch_item_count": 2,
            "peer_observed_response_count": 2,
            "peer_observed_effects": ["first_item_failed", "second_item_executed"],
            "peer_observed_state_before_sha256": peer_state_sha256,
            "peer_observed_state_after_sha256": peer_state_sha256,
        })
        schema = json.loads((SHARED / "schemas/route-execution-evidence.schema.json").read_text())
        jsonschema.validate(report, schema)

        send_event.pop("peer_observed_state_before_sha256")
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(report, schema)

        send_event["peer_observed_state_before_sha256"] = peer_state_sha256
        send_event["peer_observed_state_after_sha256"] = "not-a-sha256"
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(report, schema)

    def test_expected_local_batch_rejection_conforms_without_peer_observation(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        report = copy.deepcopy(self.report)
        send_event = next(event for event in report["events"] if event["action"] == "send")
        send_event.update({
            "batch_operation_sequence": ["Activate", "Create"],
            "batch_item_count": 2,
            "request_delivery_state": "not_sent",
            "transport_invocation_count": 0,
            "local_rejection_code": "batch_placeholder_forward_reference",
            "client_error_category": "validation",
            "client_error_cause": "invalid_input",
        })
        send_event.pop("request_tree_sha256")
        schema = json.loads((SHARED / "schemas/route-execution-evidence.schema.json").read_text())
        jsonschema.validate(report, schema)

    def test_missing_action_fails_coverage_even_when_the_route_was_called(self):
        report = copy.deepcopy(self.report)
        report["events"].pop()
        with self.assertRaisesRegex(ValueError, "missing route execution cells"):
            self.validate(report)

    def test_constructed_request_tree_must_match_the_request_sent(self):
        report = copy.deepcopy(self.report)
        send_event = next(event for event in report["events"] if event["action"] == "send")
        send_event["request_tree_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "constructed request tree differs from sent request"):
            self.validate(report)

    def test_expected_not_sent_boundary_requires_zero_transport_calls_and_no_decode(self):
        self.catalog["test_cases"][0]["requires"]["expected_status"] = "NotSent"
        report = copy.deepcopy(self.report)
        report["events"] = [event for event in report["events"] if event["action"] != "decode"]
        send_event = next(event for event in report["events"] if event["action"] == "send")
        send_event["request_delivery_state"] = "not_sent"
        send_event["transport_invocation_count"] = 0
        self.assertTrue(self.validate(report)["selected_scope_complete"])
        send_event["transport_invocation_count"] = 1
        with self.assertRaisesRegex(ValueError, "not-sent route invocation must prove transport count zero"):
            self.validate(report)

    def test_expected_not_sent_boundary_can_prove_no_request_was_constructed(self):
        self.catalog["test_cases"][0]["requires"]["expected_status"] = "NotSent"
        report = copy.deepcopy(self.report)
        report["events"] = [event for event in report["events"] if event["action"] != "decode"]
        for event in report["events"]:
            event.pop("request_tree_sha256", None)
            event["request_delivery_state"] = "not_sent"
            event["transport_invocation_count"] = 0
        self.assertTrue(self.validate(report)["selected_scope_complete"])

    def test_expected_reserved_tag_decode_rejection_needs_input_hash_not_response_tree(self):
        self.catalog['test_cases'][0]['requires'] = {
            'normative_target_id': 'KMIPKIT-ELEM-TAG-420009',
            'target_kind': 'tag',
            'allocation': 'reserved',
            'wire_value': '420009',
        }
        self.manifest['routes'].append({
            'route_id': 'c.generic-ttlv.decode',
            'language': 'c',
            'api_level': 'generic-ttlv',
            'mode': 'local',
            'public_api_symbol': 'kmipkit_ttlv_decode',
            'operation_ids': [],
            'normative_target_ids': ['*'],
            'case_categories': ['normative_inventory'],
            'protocol_versions': ['2.0'],
            'combinations': [{'encoding': 'TTLV', 'transport': None}],
            'actions': ['decode'],
            'execution_route_id': None,
            'artifact_commit': 'a' * 40,
        })
        report = copy.deepcopy(self.report)
        for event in report['events']:
            event['target_id'] = 'KMIPKIT-ELEM-TAG-420009'
        report['events'].append({
            'case_id': 'case-one',
            'route_id': 'c.generic-ttlv.decode',
            'target_id': 'KMIPKIT-ELEM-TAG-420009',
            'public_api_symbol': 'kmipkit_ttlv_decode',
            'execution_route_id': None,
            'execution_api_symbol': None,
            'action': 'decode',
            'protocol_version': '2.0',
            'encoding': 'TTLV',
            'transport': None,
            'status': 'passed',
            'invocation_count': 1,
            'invocation_id': 'c.generic-ttlv.decode:decode',
            'decode_outcome': 'rejected',
            'decode_rejection_code': 'tag_reserved',
            'decode_input_sha256': 'e' * 64,
        })
        summary = self.validate(report)
        self.assertTrue(summary['all_invocations_passed'])

        wrong_classification = copy.deepcopy(report)
        wrong_classification['events'][-1]['decode_rejection_code'] = 'tag_not_allocated'
        with self.assertRaisesRegex(ValueError, 'wrong rejection code'):
            self.validate(wrong_classification)

        unused_tag = copy.deepcopy(report)
        unused_tag_id = 'KMIPKIT-RANGE-001'
        self.catalog['test_cases'][0]['requires'].update({
            'normative_target_id': unused_tag_id,
            'target_kind': 'tag_range',
            'allocation': 'unused',
        })
        for event in unused_tag['events']:
            event['target_id'] = unused_tag_id
        unused_tag['events'][-1]['decode_rejection_code'] = 'tag_not_allocated'
        self.assertTrue(self.validate(unused_tag)['all_invocations_passed'])
        self.catalog['test_cases'][0]['requires'] = {
            'normative_target_id': 'KMIPKIT-ELEM-TAG-420009',
            'target_kind': 'tag',
            'allocation': 'reserved',
            'wire_value': '420009',
        }

        try:
            import jsonschema
        except ImportError:
            self.skipTest('jsonschema dependency is not installed')
        schema = json.loads((SHARED / 'schemas/route-execution-evidence.schema.json').read_text())
        jsonschema.validate(report, schema)

        missing_hash = copy.deepcopy(report)
        del missing_hash['events'][-1]['decode_input_sha256']
        with self.assertRaisesRegex(ValueError, 'input SHA-256'):
            self.validate(missing_hash)

        unexpected_tree = copy.deepcopy(report)
        unexpected_tree['events'][-1]['response_tree_sha256'] = 'f' * 64
        with self.assertRaisesRegex(ValueError, 'cannot have a decoded response tree'):
            self.validate(unexpected_tree)

        self.catalog['test_cases'][0]['requires']['allocation'] = 'assigned'
        with self.assertRaisesRegex(ValueError, 'rejection is not expected'):
            self.validate(report)

    def test_decode_rejection_classification_uses_the_expanded_normative_target(self):
        case = {'requires': {'enum_type': 'Tag'}}
        cell = {
            'action': 'decode',
            'mode': 'local',
            'route_id': 'rust.generic-ttlv.codec.decode',
            'public_api_symbol': 'kmipkit_ttlv::codec::decode',
        }
        self.assertEqual(
            _expected_tag_decode_rejection_code(
                cell, case, {'kind': 'tag', 'allocation': 'reserved'}
            ),
            'tag_reserved',
        )
        self.assertEqual(
            _expected_tag_decode_rejection_code(
                cell, case, {'kind': 'tag_range', 'allocation': 'unused'}
            ),
            'tag_not_allocated',
        )

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

    def test_collection_accepts_scoped_reports_only_when_the_union_is_complete(self):
        local = copy.deepcopy(self.report)
        local["run_id"] = "local-shard"
        local["events"] = local["events"][:2]
        network = copy.deepcopy(self.report)
        network["run_id"] = "network-shard"
        network["events"] = network["events"][2:]
        scopes = {
            local["run_id"]: local["events"],
            network["run_id"]: network["events"],
        }

        summary = validate_collection(
            [local, network], self.catalog, self.manifest,
            catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
            expected_artifact_sha256="b" * 64,
            scope_cells_by_run=scopes,
        )
        self.assertTrue(summary["complete_matrix"])
        self.assertEqual(5, summary["observed_cells"])

        del network["events"][-1]
        del scopes[network["run_id"]][-1]
        with self.assertRaisesRegex(ValueError, "misses full matrix cells"):
            validate_collection(
                [local, network], self.catalog, self.manifest,
                catalog_sha256="c" * 64, route_manifest_sha256="d" * 64,
                expected_artifact_sha256="b" * 64,
                scope_cells_by_run=scopes,
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
            "category": "batch_workflows",
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

    def test_controlled_peer_effects_require_valid_before_and_after_state_hashes(self):
        case = {
            "case_id": "batch-get-continue",
            "category": "batch_workflows",
            "requires": {"protocol_scenario": {
                "parameters": {"items": ["Get(missing)", "Get(existing)"]},
                "expected": {"response_count": 2,
                             "effects": ["first_item_failed", "second_item_executed"]},
            }},
        }
        event = {
            "status": "passed",
            "batch_operation_sequence": ["Get", "Get"],
            "peer_observed_operation_sequence": ["Get", "Get"],
            "batch_item_count": 2,
            "peer_observed_response_count": 2,
            "peer_observed_effects": ["first_item_failed", "second_item_executed"],
        }
        with self.assertRaisesRegex(ValueError, "controlled peer state hashes"):
            _validate_batch_event(event, case, {}, ("batch-get-continue", "send"))

        peer_state = json.dumps({"objects": ["controlled-fixture-key"]},
                                sort_keys=True, separators=(",", ":")).encode()
        peer_state_sha256 = hashlib.sha256(peer_state).hexdigest()
        event["peer_observed_state_before_sha256"] = peer_state_sha256
        event["peer_observed_state_after_sha256"] = peer_state_sha256
        _validate_batch_event(event, case, {}, ("batch-get-continue", "send"))

        event["peer_observed_state_after_sha256"] = "invalid"
        with self.assertRaisesRegex(ValueError, "controlled peer state hashes"):
            _validate_batch_event(event, case, {}, ("batch-get-continue", "send"))

    def test_expected_forward_placeholder_rejection_can_prove_no_send_without_peer(self):
        case = {
            "case_id": "test-batch-placeholder-forward-reference-rejected",
            "category": "batch_workflows",
            "requires": {"protocol_scenario": {
                "parameters": {"items": [
                    "Activate(IDPlaceholder from item 2)",
                    "Create(unique-key)",
                ]},
                "expected": {"construction": "rejected_or_server_reports_protocol_error"},
            }},
        }
        event = {
            "status": "passed",
            "batch_operation_sequence": ["Activate", "Create"],
            "batch_item_count": 2,
            "request_delivery_state": "not_sent",
            "transport_invocation_count": 0,
            "local_rejection_code": "batch_placeholder_forward_reference",
            "client_error_category": "validation",
            "client_error_cause": "invalid_input",
        }

        _validate_batch_event(event, case, {}, ("forward-placeholder", "send"))

        event["transport_invocation_count"] = 1
        with self.assertRaisesRegex(ValueError, "invoked the transport"):
            _validate_batch_event(event, case, {}, ("forward-placeholder", "send"))

    def test_async_lifecycle_steps_are_not_validated_as_a_single_batch(self):
        case = {
            "case_id": "test-async-lifecycle-poll",
            "category": "asynchronous_workflows",
            "requires": {"protocol_scenario": {
                "parameters": {"operations_in_request_order": [
                    "Create", "Query Asynchronous Requests", "Poll",
                ]},
                "expected": {"operation_order": [
                    "Create", "Query Asynchronous Requests", "Poll",
                ]},
            }},
        }
        event = {"batch_operation_sequence": ["Poll"], "batch_item_count": 1}

        _validate_batch_event(event, case, {}, ("async-poll",))

    def test_async_workflow_order_is_not_mistaken_for_a_single_batch(self):
        operations = ["Create", "Query Asynchronous Requests", "Poll"]
        case = {
            "case_id": "test-async-lifecycle-poll",
            "category": "asynchronous_workflows",
            "requires": {
                "protocol_scenario": {
                    "parameters": {"operations_in_request_order": operations},
                    "expected": {"operation_order": operations},
                },
            },
        }

        self.assertIsNone(
            _validate_batch_event(
                {"workflow_operation": "Poll", "workflow_step_index": 2},
                case,
                {"normative_targets": []},
                ("test-async-lifecycle-poll", "python.execute_poll"),
            )
        )

    def test_batch_workflow_request_order_remains_batch_validated(self):
        operations = ["Create", "Activate", "Get Attribute List", "Revoke", "Destroy"]
        case = {
            "case_id": "test-batch-create-activate-destroy",
            "category": "batch_workflows",
            "requires": {
                "protocol_scenario": {
                    "parameters": {"operations_in_request_order": operations},
                    "expected": {"operation_order": operations, "response_count": len(operations)},
                },
            },
        }
        event = {
            "batch_operation_sequence": operations,
            "peer_observed_operation_sequence": operations,
            "batch_item_count": len(operations),
            "peer_observed_response_count": len(operations),
        }

        self.assertIsNone(
            _validate_batch_event(
                event,
                case,
                {"normative_targets": []},
                ("test-batch-create-activate-destroy", "batch"),
            )
        )
    def test_async_lifecycle_is_not_misclassified_as_one_batch(self):
        case = {
            "case_id": "async-create-query-poll",
            "category": "asynchronous_workflows",
            "requires": {"protocol_scenario": {
                "operations": ["Create", "Query Asynchronous Requests", "Poll"],
                "parameters": {"operations_in_request_order": [
                    "Create", "Query Asynchronous Requests", "Poll"
                ]},
            }},
        }
        self.assertIsNone(_expected_batch_operations(case))

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

    def test_cross_language_parity_compares_expected_decoder_rejection_inputs(self):
        reports, manifest = self._four_language_fixture()
        self.catalog['test_cases'][0]['requires'] = {
            'normative_target_id': 'KMIPKIT-ELEM-TAG-420009',
            'target_kind': 'tag',
            'allocation': 'reserved',
            'wire_value': '420009',
        }
        for language, report in zip(('rust', 'c', 'java', 'python'), reports, strict=True):
            for event in report['events']:
                event['target_id'] = 'KMIPKIT-ELEM-TAG-420009'
            route_id = f'{language}.generic-ttlv.decoder'
            manifest['routes'].append({
                'route_id': route_id,
                'language': language,
                'api_level': 'generic-ttlv',
                'mode': 'local',
                'public_api_symbol': f'{language}_ttlv_decode',
                'operation_ids': [],
                'normative_target_ids': ['*'],
                'case_categories': ['normative_inventory'],
                'protocol_versions': ['2.0'],
                'combinations': [{'encoding': 'TTLV', 'transport': None}],
                'actions': ['decode'],
                'execution_route_id': None,
                'artifact_commit': 'a' * 40,
            })
            report['events'].append({
                'case_id': 'case-one',
                'route_id': route_id,
                'target_id': 'KMIPKIT-ELEM-TAG-420009',
                'public_api_symbol': f'{language}_ttlv_decode',
                'execution_route_id': None,
                'execution_api_symbol': None,
                'action': 'decode',
                'protocol_version': '2.0',
                'encoding': 'TTLV',
                'transport': None,
                'status': 'passed',
                'invocation_count': 1,
                'invocation_id': f'{language}:reserved-tag-decode',
                'decode_outcome': 'rejected',
                'decode_rejection_code': 'tag_reserved',
                'decode_input_sha256': 'e' * 64,
            })
        artifacts = {report['language']: report['artifact_sha256'] for report in reports}
        summary = validate_parity_collection(
            reports, self.catalog, manifest, catalog_sha256='c' * 64,
            route_manifest_sha256='d' * 64,
            expected_artifact_sha256_by_language=artifacts,
        )
        self.assertTrue(summary['tree_parity_passed'])

        reports[-1]['events'][-1]['decode_input_sha256'] = 'f' * 64
        with self.assertRaisesRegex(ValueError, 'languages disagree on normalized KMIP trees'):
            validate_parity_collection(
                reports, self.catalog, manifest, catalog_sha256='c' * 64,
                route_manifest_sha256='d' * 64,
                expected_artifact_sha256_by_language=artifacts,
            )

if __name__ == "__main__":
    unittest.main()
