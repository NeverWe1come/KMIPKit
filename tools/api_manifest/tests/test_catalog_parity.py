"""The complete manifest operation list must match the normative catalog."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.generate import ManifestError, _validate_operation_inventory
from tools.api_manifest.tests.test_manifest_v2 import _valid_v2_manifest


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"


class OperationCatalogParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.manifest = _valid_v2_manifest()

    def test_manifest_has_exact_numeric_catalog_operation_inventory(self) -> None:
        _validate_operation_inventory(self.manifest, self.catalog)

        operations = self.manifest["operations"]
        sections = [tuple(int(part) for part in row["section"].split(".")) for row in operations]
        self.assertEqual(len(operations), 57)
        self.assertEqual(sections, sorted(sections))
        self.assertEqual({row["scope"] for row in operations}, {"client_1_0"})

    def test_duplicate_missing_extra_and_out_of_scope_rows_are_rejected(self) -> None:
        cases = {}

        duplicate = copy.deepcopy(self.manifest)
        duplicate["operations"][-1] = copy.deepcopy(duplicate["operations"][0])
        cases["duplicate"] = duplicate

        missing = copy.deepcopy(self.manifest)
        missing["operations"].pop()
        cases["missing"] = missing

        extra = copy.deepcopy(self.manifest)
        extra["operations"][-1]["operationId"] = "KMIPKIT-ELEM-OP-OUTSIDE-CATALOG"
        cases["extra"] = extra

        out_of_scope = copy.deepcopy(self.manifest)
        out_of_scope["operations"][0]["scope"] = "server_only"
        cases["out of scope"] = out_of_scope

        for label, manifest in cases.items():
            with self.subTest(case=label), self.assertRaises(ManifestError):
                _validate_operation_inventory(manifest, self.catalog)

    def test_wrong_name_section_and_non_numeric_order_are_rejected(self) -> None:
        mutations = (
            ("name", lambda rows: rows[0].__setitem__("name", "Wrong Name")),
            ("section", lambda rows: rows[0].__setitem__("section", "6.1.999")),
            ("order", lambda rows: rows.reverse()),
        )
        for label, mutate in mutations:
            manifest = copy.deepcopy(self.manifest)
            mutate(manifest["operations"])
            with self.subTest(case=label), self.assertRaises(ManifestError):
                _validate_operation_inventory(manifest, self.catalog)


if __name__ == "__main__":
    unittest.main()
