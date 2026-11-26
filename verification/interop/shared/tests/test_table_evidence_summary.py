import unittest

from verification.interop.shared.tools.table_evidence_summary import summarize


class TableEvidenceSummaryTests(unittest.TestCase):
    def test_scope_and_route_counts_need_actual_evidence(self):
        controlled = {"case_id": "archive", "scope": "controlled", "status": "server_unsupported",
                      "library_correctness": "passed", "result_status_code": 1, "result_reason_code": 5}
        live = {**controlled, "scope": "live", "status": "passed", "result_status_code": 0,
                "result_reason_code": None}
        generic = {"language": "rust", "api_level": "generic-ttlv",
                   "expectation": "verify_unsupported_response", "case_results": [
            controlled, live, {**live, "scope": "live", "status": "not_run"},
        ], "oasis_fixture_results": [{"checks": {"request": "passed", "response": "failed"}}]}
        fluent = {"language": "rust", "api_level": "fluent", "expectation": "verify_unsupported_response",
                  "case_results": [
            {**controlled, "status": "not_run", "library_correctness": "not_checked"},
        ]}
        inventory = {"protocol_version": "2.1", "rows": [{"server_cases": ["archive"], "obligations": [generic, fluent]}]}
        summary = summarize(inventory, {"test_cases": [{"case_id": "archive", "requires": {}}]})
        self.assertEqual({"local": {"passed": 1, "failed": 1},
                          "controlled": {"unsupported": 1}, "live": {"success": 1}}, generic["evidence_summary"])
        self.assertEqual({"local": {}, "controlled": {}, "live": {}}, fluent["evidence_summary"])
        first, second = summary["routes"]
        self.assertEqual((0, 1), (first["live_cells"], second["live_cells"]))
        self.assertEqual((1, 0), (first["no_evidence"], second["no_evidence"]))
        self.assertEqual((0, 1), (first["failed_cells"], second["failed_cells"]))
        self.assertEqual({"controlled": {"matched": ["archive"], "missing": []},
                          "live": {"matched": [], "missing": ["archive"]}}, generic["expected_case_outcomes"])
        self.assertEqual((0, 1), (first["controlled_expected_cases_met_cells"],
                                 second["controlled_expected_cases_met_cells"]))
        self.assertEqual((0, 0), (first["live_expected_cases_met_cells"],
                                 second["live_expected_cases_met_cells"]))

    def test_controlled_success_does_not_confirm_provider_support_expectation(self):
        case_id = "returned-key-value"
        success = {"case_id": case_id, "status": "passed", "library_correctness": "passed",
                   "result_status_code": 0}
        cell = {"language": "rust", "api_level": "typed-protocol",
                "expectation": "verify_unsupported_response",
                "case_results": [{**success, "scope": "controlled"}, {**success, "scope": "live"}]}
        row = {"server_cases": [case_id], "obligations": [cell]}
        catalog = {"test_cases": [{"case_id": case_id, "requires": {"controlled_expected_status": "Success"}}]}
        summarize({"protocol_version": "2.1", "rows": [row]}, catalog)
        self.assertEqual({"matched": [case_id], "missing": []}, cell["expected_case_outcomes"]["controlled"])
        self.assertEqual({"matched": [], "missing": [case_id]}, cell["expected_case_outcomes"]["live"])

        self.assertEqual("missing_matching_provider_evidence", cell["completion_status"])

    def test_completion_requires_evidence_for_the_selected_route(self):
        archive = {"case_id": "archive", "status": "server_unsupported", "library_correctness": "passed",
                   "result_status_code": 1, "result_reason_code": 5}
        cell = {"language": "rust", "api_level": "typed-protocol", "expectation": "verify_unsupported_response",
                "case_results": [{**archive, "scope": "controlled"}, {**archive, "scope": "live"}]}
        missing = {"language": "c", "api_level": "fluent", "expectation": "verify_unsupported_response"}
        inventory = {"protocol_version": "2.0", "rows": [{"server_cases": ["archive"],
                                                         "obligations": [cell, missing]}]}
        catalog = {"test_cases": [{"case_id": "archive", "requires": {}}]}
        self.assertEqual({"complete": False, "cells_by_status": {
            "complete": 1, "missing_controlled_evidence": 1,
        }}, summarize(inventory, catalog)["completion"])
        self.assertTrue(summarize(inventory, catalog, {"rust"})["completion"]["complete"])

    def test_boundary_needs_a_rejection_and_vendor_fixture_needs_a_binding_route(self):
        boundary = {"language": "c", "api_level": "generic-ttlv", "expectation": "verify_version_boundary",
                    "local_fixture_results": [{"expected": "rejected", "checks": {"construction": "rejected"}}]}
        vendor = {"language": "c", "api_level": "generic-ttlv", "expectation": "verify_provider_fixture"}
        inventory = {"protocol_version": "2.1", "rows": [
            {"obligations": [boundary]},
            {"obligations": [vendor], "provider_fixture_results": [{"scope": "live"}]},
        ]}
        summary = summarize(inventory, {"test_cases": []})
        self.assertEqual({"complete": 1, "missing_provider_fixture": 1}, summary["completion"]["cells_by_status"])
        boundary["local_fixture_results"][0]["checks"]["construction"] = "accepted"
        summarize(inventory, {"test_cases": []})
        self.assertEqual("failed_local_assertion", boundary["completion_status"])
