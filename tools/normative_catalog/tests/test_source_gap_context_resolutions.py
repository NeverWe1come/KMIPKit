"""Source excerpts without standalone REQs keep typed, reviewable dispositions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"


class SourceGapContextResolutionTests(unittest.TestCase):
    def test_procedure_steps_and_generic_parameters_are_context_not_modal_reqs(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        procedure_ids = [f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}" for number in range(11, 21)]
        generic_ids = [f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}" for number in range(6, 9)]

        for gap_id in procedure_ids:
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                resolution = gap["source_resolution"]
                self.assertEqual(gap["source_force"], "imperative_procedure_without_formal_modal")
                self.assertEqual(resolution["kind"], "procedure_context")
                self.assertTrue(resolution["context_id"].startswith("KMIPKIT-PROC-SPEC-6.1-"))
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        for gap_id in generic_ids:
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                resolution = gap["source_resolution"]
                self.assertEqual(resolution["kind"], "generic_parameter_context")
                self.assertTrue(resolution["context_id"].startswith("KMIPKIT-CONTEXT-SPEC-6.1-"))
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        for number in (12, 14, 16):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"]
            self.assertEqual(gap["source_resolution"]["related_requirement_ids"], ["KMIPKIT-REQ-SPEC-7.8-001"])

    def test_paragraph_data_rule_remains_unresolved_and_lease_policy_maps_to_attribute(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        data_rule = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        self.assertEqual(data_rule["source_resolution"]["kind"], "unresolved_source_interpretation")
        self.assertEqual(data_rule["scope_disposition"], "unresolved")
        self.assertIsNone(data_rule["candidate_requirement_id"])
        self.assertIsNone(data_rule["proposed_requirement_id"])

        lease_policy = gaps["KMIPKIT-SOURCE-GAP-SPEC-4.30-CLIENT-LEASE-READ-ONLY"]
        resolution = lease_policy["source_resolution"]
        self.assertEqual(resolution["kind"], "attribute_policy_mapping")
        self.assertEqual(resolution["element_id"], "KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME")
        self.assertIsNone(lease_policy["candidate_requirement_id"])
        self.assertIsNone(lease_policy["proposed_requirement_id"])
        lease_time = next(row for row in catalog["elements"] if row["element_id"] == resolution["element_id"])
        self.assertEqual(lease_time["source_modifiable_by_client"], "No")
        self.assertTrue(any("read-only for clients" in row["source_text"] for row in lease_time["source_operation_restrictions"]))


if __name__ == "__main__":
    unittest.main()
