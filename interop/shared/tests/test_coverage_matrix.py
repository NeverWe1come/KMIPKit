"""Executable expansion and invariants for all installed public route cells."""
from __future__ import annotations

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


SHARED = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SHARED / "tools"))
from coverage_matrix import _resolve_output_path, build_matrix, iter_cells  # noqa: E402


class CoverageMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((SHARED / "catalog/kmipkit-1.0-test-catalog.json").read_text())
        cls.manifest = json.loads((SHARED / "coverage/route-capabilities.json").read_text())

    def test_every_case_language_has_a_deterministic_nonempty_route_matrix(self):
        matrix = build_matrix(self.catalog, self.manifest)
        self.assertEqual("planned_coverage_obligations", matrix["matrix_kind"])
        self.assertFalse(matrix["execution_evidence_collected"])
        self.assertEqual(len(self.catalog["test_cases"]), matrix["case_count"])
        self.assertEqual(len(self.manifest["routes"]), matrix["public_api_route_count"])
        self.assertEqual(len(self.manifest["execution_routes"]), matrix["execution_route_count"])
        self.assertEqual(4 * len(self.catalog["test_cases"]), matrix["case_language_count"])
        self.assertGreater(matrix["cell_count"], matrix["case_language_count"])
        self.assertRegex(matrix["matrix_sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual([], matrix["public_api_routes_without_cells"])
        self.assertEqual([], matrix["execution_routes_without_cells"])
        self.assertEqual({language: len(self.catalog["test_cases"]) for language in ("rust", "c", "java", "python")},
                         matrix["coverage_by_language"])

    def test_output_path_uses_the_active_runner_temp_root_after_symlink_resolution(self):
        with tempfile.TemporaryDirectory() as directory:
            temporary_root = Path(directory) / "runner-temp"
            temporary_root.mkdir()
            temporary_alias = Path(directory) / "runner-temp-alias"
            temporary_alias.symlink_to(temporary_root, target_is_directory=True)
            requested_output = temporary_alias / "matrix.jsonl"
            with patch.dict(os.environ, {"RUNNER_TEMP": str(temporary_alias)}):
                self.assertEqual(temporary_root / "matrix.jsonl",
                                 _resolve_output_path(requested_output, SHARED.parents[1]))

    def test_every_case_version_language_has_at_least_one_route_cell(self):
        cells = list(iter_cells(self.catalog, self.manifest))
        observed = {
            (cell["case_id"], cell["protocol_version"], cell["language"])
            for cell in cells
        }
        expected = {
            (case["case_id"], version, language)
            for case in self.catalog["test_cases"]
            for version in case["execution"]["protocol_versions"]
            for language in ("rust", "c", "java", "python")
        }
        self.assertEqual(expected, observed)

    def test_batch_workflow_cells_include_each_ordered_operation_target(self):
        case = next(case for case in self.catalog["test_cases"]
                    if case["case_id"] == "test-batch-create-activate-destroy")
        target_ids = case["requires"]["protocol_scenario"]["target_ids"]
        self.assertEqual(4, len(target_ids))
        cells = [cell for cell in iter_cells(self.catalog, self.manifest)
                 if cell["case_id"] == case["case_id"] and cell["action"] == "construct"]
        self.assertTrue(all("target_id" in cell for cell in cells))
        for language in ("rust", "c", "java", "python"):
            steps = {
                (cell.get("workflow_step_index"), cell.get("workflow_operation"), cell["target_id"])
                for cell in cells if cell["language"] == language and cell["target_id"] in target_ids
            }
            self.assertEqual({(index, operation, target) for index, (operation, target)
                              in enumerate(zip(case["requires"]["protocol_scenario"]["operations"], target_ids))},
                             steps)

    def test_repeated_batch_items_remain_separate_ordered_obligations(self):
        case = next(case for case in self.catalog["test_cases"]
                    if case["case_id"] == "test-batch-stop-on-first-failure")
        target_id = "KMIPKIT-ELEM-OP-C2S-GET"
        cells = [cell for cell in iter_cells(self.catalog, self.manifest)
                 if cell["case_id"] == case["case_id"] and cell["language"] == "python"
                 and cell["target_id"] == target_id and cell["action"] == "construct"]
        item_indices = {cell.get("workflow_item_index") for cell in cells}
        self.assertEqual({0, 1}, item_indices)

    def test_missing_version_route_fails_even_if_the_case_has_another_version(self):
        case = {
            "case_id": "two-version-case",
            "category": "normative_inventory",
            "operation": "Ping",
            "requires": {},
            "execution": {"protocol_versions": ["2.0", "2.1"]},
        }
        catalog = {
            "test_cases": [case],
            "normative_targets": [{"kind": "operation", "name": "Ping", "target_id": "PING"}],
        }
        manifest = {
            "routes": [{
                "route_id": "rust.ping",
                "language": "rust",
                "case_categories": ["normative_inventory"],
                "protocol_versions": ["2.0"],
                "normative_target_ids": ["PING"],
                "mode": "local",
                "actions": ["construct"],
                "combinations": [{"encoding": "TTLV", "transport": None}],
                "public_api_symbol": "PingRequest::new",
                "artifact_commit": "a" * 40,
                "execution_route_id": None,
            }],
            "execution_routes": [],
            "unavailable": [],
        }
        with self.assertRaisesRegex(ValueError, r"two-version-case target PING \(rust, KMIP 2.1\)"):
            list(iter_cells(catalog, manifest, languages=("rust",)))

    def test_multistep_scenario_requires_every_operation_route_for_each_version(self):
        case = {
            "case_id": "two-operation-case",
            "category": "normative_inventory",
            "operation": "Ping",
            "requires": {"protocol_scenario": {
                "target_ids": ["PING", "GET"],
                "operations": ["Ping", "Get"],
                "protocol_versions": ["2.0", "2.1"],
            }},
            "execution": {"protocol_versions": ["2.0", "2.1"]},
        }
        catalog = {
            "test_cases": [case],
            "normative_targets": [
                {"kind": "operation", "name": "Ping", "target_id": "PING"},
                {"kind": "operation", "name": "Get", "target_id": "GET"},
            ],
        }

        def route(route_id, target_id, versions):
            return {
                "route_id": route_id, "language": "rust", "case_categories": ["normative_inventory"],
                "protocol_versions": versions, "normative_target_ids": [target_id], "mode": "local",
                "actions": ["construct"], "combinations": [{"encoding": "TTLV", "transport": None}],
                "public_api_symbol": route_id, "artifact_commit": "a" * 40, "execution_route_id": None,
            }

        manifest = {"routes": [route("rust.ping", "PING", ["2.0", "2.1"]),
                                route("rust.get", "GET", ["2.0"])],
                    "execution_routes": [], "unavailable": []}
        with self.assertRaisesRegex(ValueError, r"two-operation-case target GET \(rust, KMIP 2.1\)"):
            list(iter_cells(catalog, manifest, languages=("rust",)))

    def test_scenario_protocol_versions_must_match_case_versions(self):
        case = {
            "case_id": "version-mismatch", "category": "normative_inventory", "operation": "Ping",
            "requires": {"protocol_scenario": {"target_ids": ["PING"], "operations": ["Ping"],
                                                   "protocol_versions": ["2.0"]}},
            "execution": {"protocol_versions": ["2.1"]},
        }
        catalog = {"test_cases": [case], "normative_targets": [{"kind": "operation", "name": "Ping", "target_id": "PING"}]}
        manifest = {"routes": [], "execution_routes": [], "unavailable": []}
        with self.assertRaisesRegex(ValueError, "scenario protocol versions disagree"):
            list(iter_cells(catalog, manifest, languages=("rust",)))

    def test_scenario_operation_order_must_match_normative_target_order(self):
        case = {
            "case_id": "operation-order-mismatch", "category": "normative_inventory", "operation": "Ping",
            "requires": {"protocol_scenario": {"target_ids": ["GET", "PING"],
                                                "operations": ["Ping", "Get"],
                                                "protocol_versions": ["2.0"]}},
            "execution": {"protocol_versions": ["2.0"]},
        }
        catalog = {"test_cases": [case], "normative_targets": [
            {"kind": "operation", "name": "Ping", "target_id": "PING"},
            {"kind": "operation", "name": "Get", "target_id": "GET"},
        ]}
        with self.assertRaisesRegex(ValueError, "operation order disagrees"):
            list(iter_cells(catalog, {"routes": [], "execution_routes": [], "unavailable": []},
                            languages=("rust",)))

    def test_unknown_scenario_operation_or_batch_item_cannot_be_dropped(self):
        case = {
            "case_id": "unknown-operation", "category": "normative_inventory", "operation": "Ping",
            "requires": {"protocol_scenario": {"target_ids": ["PING"], "operations": ["Unknown"],
                                                "protocol_versions": ["2.0"]}},
            "execution": {"protocol_versions": ["2.0"]},
        }
        catalog = {"test_cases": [case], "normative_targets": [
            {"kind": "operation", "name": "Ping", "target_id": "PING"},
        ]}
        with self.assertRaisesRegex(ValueError, "lacks a normative target"):
            list(iter_cells(catalog, {"routes": [], "execution_routes": [], "unavailable": []},
                            languages=("rust",)))

    def test_generated_rows_conform_to_the_versioned_json_schema(self):
        try:
            import jsonschema
        except ImportError:
            self.skipTest("jsonschema dependency is not installed")
        schema = json.loads((SHARED / "schemas/coverage-matrix-row.schema.json").read_text())
        cells = list(iter_cells(self.catalog, self.manifest))
        selected = [next(cell for cell in cells if cell["mode"] == "local"),
                    next(cell for cell in cells if cell["mode"] == "network"),
                    next(cell for cell in cells if "workflow_step_index" in cell)]
        for cell in selected:
            with self.subTest(cell=cell):
                jsonschema.validate(cell, schema)
        invalid = dict(selected[2])
        invalid.pop("workflow_operation")
        with self.assertRaises(jsonschema.ValidationError):
            jsonschema.validate(invalid, schema)

    def test_cells_preserve_each_supported_version_encoding_and_transport(self):
        cells = list(iter_cells(self.catalog, self.manifest))
        self.assertTrue(all(cell["execution_status"] == "required" for cell in cells))
        self.assertTrue(all(cell["evidence_collected"] is False for cell in cells))
        identities = [(cell["case_id"], cell["language"], cell["route_id"], cell.get("target_id"),
                       cell.get("workflow_step_index"), cell.get("workflow_item_index"),
                       cell["execution_route_id"], cell["action"], cell["protocol_version"],
                       cell["encoding"], cell["transport"]) for cell in cells]
        self.assertEqual(len(identities), len(set(identities)))
        cases = {case["case_id"]: case for case in self.catalog["test_cases"]}
        routes = {route["route_id"]: route for route in self.manifest["routes"]}
        execution_routes = {route["execution_route_id"]: route for route in self.manifest["execution_routes"]}
        for cell in cells:
            with self.subTest(cell=cell):
                case = cases[cell["case_id"]]
                route = routes[cell["route_id"]]
                self.assertIn(cell["protocol_version"], case["execution"]["protocol_versions"])
                self.assertIn(cell["protocol_version"], route["protocol_versions"])
                if route["mode"] == "local":
                    self.assertIn(cell["action"], route["actions"])
                    self.assertIsNone(cell["execution_route_id"])
                    self.assertIsNone(cell["transport"])
                    self.assertEqual("TTLV", cell["encoding"])
                elif cell["action"] == "construct":
                    self.assertIn("construct", route["actions"])
                    self.assertIsNotNone(cell["execution_route_id"])
                    self.assertIn({"encoding": cell["encoding"], "transport": cell["transport"]}, route["combinations"])
                else:
                    execution = execution_routes[cell["execution_route_id"]]
                    self.assertIn(cell["action"], {"send", "decode"})
                    self.assertEqual(route["execution_route_id"], execution["execution_route_id"])
                    self.assertEqual(cell["language"], execution["language"])
                    self.assertIsNotNone(cell["execution_api_symbol"])

    def test_network_route_combinations_are_not_satisfied_by_local_ttlv_cells(self):
        cells = list(iter_cells(self.catalog, self.manifest))
        network = [cell for cell in cells if cell["mode"] == "network" and cell["action"] in {"send", "decode"}]
        construction = [cell for cell in cells if cell["mode"] == "network" and cell["action"] == "construct"]
        self.assertTrue(network)
        self.assertTrue(construction)
        self.assertTrue(all(cell["transport"] in {"raw_tls", "https"} for cell in network))
        self.assertTrue(all(cell["encoding"] in {"TTLV", "JSON", "XML"} for cell in network))
        self.assertEqual({"send", "decode"}, {cell["action"] for cell in network})
        self.assertTrue(all(cell["execution_route_id"] for cell in network))

    def test_every_declared_execution_route_gets_send_and_decode_cells(self):
        cells = list(iter_cells(self.catalog, self.manifest))
        observed = {cell["execution_route_id"] for cell in cells if cell["execution_route_id"]}
        expected = {route["execution_route_id"] for route in self.manifest["execution_routes"]}
        self.assertEqual(expected, observed)
        self.assertTrue(all(any(cell["execution_route_id"] == route_id and cell["action"] == action
                                for cell in cells)
                            for route_id in expected for action in ("send", "decode")))


if __name__ == "__main__":
    unittest.main()
