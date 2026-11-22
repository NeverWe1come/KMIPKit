"""Regression checks for the pinned draft and its OASIS XML fixtures."""

from collections import Counter
import json
from pathlib import Path
import sys
import unittest

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[2]
sys.path.insert(0, str(SHARED / "tools"))

from coverage_matrix import selected_cells, summarize_cells


class DraftMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cells = list(selected_cells(ROOT, ("3.0",), None))

    def test_fixture_inventory_and_batches_preserve_order_and_expected_outcomes(self):
        rows = [row for row in self.cells if row["origin"] == "xml"]
        paths = {row["fixture"]["path"]: row["fixture"]["category"] for row in rows}
        self.assertEqual(Counter(mandatory=95, optional=7), Counter(paths.values()))
        self.assertEqual(617, len({(row["case_id"], row["step_index"]) for row in rows}))
        self.assertEqual(654, len(rows))
        self.assertEqual({"Success": 615, "OperationFailed": 37, "OperationUndone": 2},
                         Counter(row["result_status"] for row in rows))
        self.assertTrue(any(row["result_reason"] for row in rows))
        for row in rows:
            if row["case_id"] == "MSGENC-HTTPS-M-1-30":
                self.assertEqual("https", row["transport"])
            if row["case_id"] == "MSGENC-JSON-M-1-30":
                self.assertEqual("JSON", row["encoding"])
            if row["case_id"] == "MSGENC-XML-M-1-30":
                self.assertEqual("XML", row["encoding"])

    def test_draft_inventory_has_source_references_and_xml_links_are_field_specific(self):
        targets = {row["case_id"]: row["target"] for row in self.cells if row["kind"] == "inventory"}
        counts = Counter(target["kind"] for target in targets.values())
        self.assertEqual(69, counts["operation"])
        self.assertEqual(275, counts["payload_field"])
        self.assertEqual(3911, counts["table_row"])
        self.assertEqual(396, counts["normative_prose"])
        self.assertEqual(189, counts["referenced_enum_value"])
        self.assertEqual(28, sum(row["kind"] == "public_route" for row in self.cells))
        observed = set()
        for row in self.cells:
            for role, ids in row.get("observed_fields", {}).items():
                for target_id in ids:
                    target = targets[target_id]
                    self.assertEqual("payload_field", target["kind"])
                    self.assertEqual(role, target["role"])
                    self.assertTrue(target["source"]["captions"])
                    observed.add(target_id)
        self.assertEqual(94, len(observed))
        self.assertEqual(observed, {row["case_id"] for row in self.cells if row.get("xml_presence_observed")})

    def test_origin_filters_are_exact_and_output_is_deterministic(self):
        for origin in ("xml", "normative"):
            selected = [row for row in self.cells if row["origin"] == origin]
            self.assertEqual(selected, list(selected_cells(ROOT, ("3.0",), origin)))
        self.assertEqual(summarize_cells(iter(self.cells)),
                         summarize_cells(selected_cells(ROOT, ("3.0",), None)))

    def test_draft_rows_match_schema(self):
        import jsonschema

        schema = json.loads((SHARED / "schemas/coverage-matrix-row.schema.json").read_text())
        validator = jsonschema.Draft202012Validator(schema)
        for row in self.cells:
            validator.validate(row)
        duplicate = self.cells[0]
        with self.assertRaisesRegex(ValueError, "duplicate coverage matrix cell"):
            summarize_cells(iter([duplicate, duplicate]))
