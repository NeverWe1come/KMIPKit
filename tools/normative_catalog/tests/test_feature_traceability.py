"""Tests for feature ownership of implemented protocol elements."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FEATURE_SPEC = "KMIPKIT-0003"
RESULT_ELEMENT_IDS = {
    "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON",
    "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
    "KMIPKIT-ELEM-RESULT-RESULT-MESSAGE",
    "KMIPKIT-ELEM-RESULT-RESULT-REASON",
    "KMIPKIT-ELEM-RESULT-RESULT-STATUS",
}


class FeatureTraceabilityTests(unittest.TestCase):
    def test_result_contract_elements_link_to_their_spec_code_and_tests(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        elements = catalog["elements"]
        result_enumerations = {
            element["element_id"]
            for element in elements
            if element["element_id"]
            in {
                "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON",
                "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
            }
        }
        result_element_ids = set(RESULT_ELEMENT_IDS)
        result_element_ids.update(
            element["element_id"]
            for element in elements
            if element.get("kind") == "enumeration_value"
            and result_enumerations.intersection(element.get("parent_element_ids", []))
            and element.get("allocation") in {"assigned", "extension"}
        )

        by_id = {element["element_id"]: element for element in elements}
        self.assertTrue(result_element_ids.issubset(by_id))
        self.assertGreaterEqual(len(result_element_ids), 80)

        for element_id in sorted(result_element_ids):
            with self.subTest(element_id=element_id):
                element = by_id[element_id]
                self.assertEqual(element["feature_spec"], FEATURE_SPEC)
                implementation_refs = element["implementation_refs"]
                verification_refs = element["verification_refs"]
                self.assertTrue(implementation_refs)
                self.assertTrue(verification_refs)
                for reference in implementation_refs:
                    self.assertTrue((ROOT / reference.split("::", 1)[0]).is_file(), reference)
                for reference in verification_refs:
                    test_path = reference.split("::", 1)[0]
                    self.assertTrue((ROOT / test_path).is_file(), reference)


if __name__ == "__main__":
    unittest.main()
