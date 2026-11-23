"""Regression checks for the KMIP 2.0 and 2.1 OASIS profile corpora."""

from collections import Counter
import json
from pathlib import Path
import sys
import unittest

import jsonschema

SHARED = Path(__file__).resolve().parents[1]
ROOT = SHARED.parents[2]
sys.path.insert(0, str(SHARED / "tools"))

from coverage_matrix import selected_cells, summarize_cells
from oasis_xml import normalize


class LegacyXmlMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows = list(selected_cells(ROOT, ("2.0", "2.1"), "xml"))

    def test_corpora_and_version_filters(self):
        self.assertEqual(1156, summarize_cells(iter(self.rows))["xml_exchange_count"])
        for version in ("2.0", "2.1"):
            with self.subTest(version=version):
                rows = [row for row in self.rows if row["protocol_version"] == version]
                self.assertEqual(613, len(rows))
                paths = {row["fixture"]["path"]: row["fixture"]["category"] for row in rows}
                self.assertEqual(Counter(mandatory=86, optional=7), Counter(paths.values()))
                suffix = version.replace(".", "")
                by_case = {row["case_id"]: row for row in rows}
                self.assertIn(f"CS-BC-M-13-{suffix}", by_case)
                self.assertIn(f"CS-BC-M-14-{suffix}", by_case)
                for label, key, value in (("HTTPS", "transport", "https"),
                                          ("JSON", "encoding", "JSON"),
                                          ("XML", "encoding", "XML")):
                    self.assertEqual(value, by_case[f"MSGENC-{label}-M-1-{suffix}"][key])
                self.assertTrue(any(row["result_reason"] for row in rows))
                self.assertEqual(rows, list(selected_cells(ROOT, (version,), "xml")))

    def test_scenario_rows_validate_and_link_to_versioned_payload_fields(self):
        schema = json.loads((SHARED / "schemas/coverage-matrix-row.schema.json").read_text())
        validator = jsonschema.Draft202012Validator(schema)
        catalog = json.loads((SHARED / "catalog/kmipkit-2x-matrix-catalog.json").read_text())
        targets = {target["target_id"]: target for target in catalog["normative_targets"]}
        observed = set()
        for row in self.rows:
            validator.validate(row)
            for role, ids in row["observed_fields"].items():
                for target_id in ids:
                    target = targets[target_id]
                    self.assertEqual("operation_payload_field", target["kind"])
                    self.assertEqual(role, target["role"])
                    self.assertEqual([row["protocol_version"]], target["protocol_versions"])
                    self.assertEqual(normalize(row["operation"]), normalize(target["operation"]))
                    observed.add(target_id)
        self.assertEqual(204, len(observed))
