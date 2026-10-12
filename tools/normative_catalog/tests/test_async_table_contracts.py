"""Asynchronous operation table fields use source contracts, not invented REQs."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"


class AsyncTableContractTests(unittest.TestCase):
    def test_process_and_query_async_request_rows_are_feature_contracts(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        expected = {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.39-ASYNC-CORRELATION-VALUE-REQUIRED": (
                "6.1.39", "Asynchronous Correlation Value", [31618, 31636], 278,
                "KMIPKIT-ELEM-OP-C2S-PROCESS", ["FR-005"],
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-ASYNC-CORRELATION-VALUES-OPTIONAL": (
                "6.1.41", "Asynchronous Correlation Values", [32456, 32474], 285,
                "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", ["FR-006"],
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-OPERATIONS-OPTIONAL": (
                "6.1.41", "Operations", [32475, 32492], 285,
                "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", ["FR-006"],
            ),
        }
        contracts = {
            row["gap_id"]: row
            for row in oracle["source_table_contracts"]
            if row.get("gap_id") in expected
        }
        gaps = {
            row["gap_id"]: row
            for row in oracle["source_only_requirement_gaps"]
            if row.get("gap_id") in expected
        }

        self.assertEqual(set(contracts), set(expected))
        self.assertEqual(gaps, {})
        for gap_id, (section, label, line_range, table_number, operation_id, fr_ids) in expected.items():
            row = contracts[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(row["source_section"], section)
                self.assertEqual(row["source_locator"]["row_label"], label)
                self.assertEqual(row["html_line_range"], line_range)
                self.assertEqual(row["table_number"], table_number)
                self.assertEqual(row["operation_element_id"], operation_id)
                self.assertEqual(row["functional_requirement_ids"], fr_ids)
                self.assertEqual(row["status"], "mapped_to_approved_feature_contract")
                self.assertNotIn("candidate_requirement_id", row)
                self.assertNotIn("proposed_requirement_id", row)
                self.assertNotIn("requirement_id", row)


if __name__ == "__main__":
    unittest.main()
