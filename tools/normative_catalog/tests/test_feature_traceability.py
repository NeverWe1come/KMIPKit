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
        self.assertEqual(len(result_element_ids), 82)

        for element_id in sorted(result_element_ids):
            with self.subTest(element_id=element_id):
                element = by_id[element_id]
                self.assertEqual(element["feature_spec"], FEATURE_SPEC)
                implementation_refs = element["implementation_refs"]
                verification_refs = element["verification_refs"]
                required_implementation_refs = {"crates/kmipkit-protocol/src/result.rs"}
                if element.get("kind") == "enumeration_value" and element.get("allocation") == "assigned":
                    required_implementation_refs.add(
                        "crates/kmipkit-protocol/src/result_values_generated.rs"
                    )
                self.assertTrue(required_implementation_refs.issubset(implementation_refs))

                if "RESULT-MESSAGE" in element_id:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::result_message_preserves_presence_and_text",
                        "crates/kmipkit-protocol/tests/result_contract.rs::operation_result_display_and_debug_redact_message",
                    }
                elif "RESULT-REASON" in element_id:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::known_reason_values_match_catalog",
                        "crates/kmipkit-protocol/tests/result_contract.rs::unknown_reason_preserves_raw_value",
                        "crates/kmipkit-protocol/tests/result_contract.rs::failure_requires_a_reason",
                        "crates/kmipkit-protocol/tests/result_contract.rs::success_forbids_a_reason",
                    }
                else:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::known_status_values_match_catalog",
                        "crates/kmipkit-protocol/tests/result_contract.rs::unknown_status_preserves_raw_value",
                        "crates/kmipkit-protocol/tests/result_contract.rs::other_statuses_do_not_gain_reason_presence_rules",
                    }
                self.assertTrue(required_verification_refs.issubset(verification_refs))

                for reference in implementation_refs:
                    self.assertTrue((ROOT / reference.split("::", 1)[0]).is_file(), reference)
                for reference in verification_refs:
                    test_path = reference.split("::", 1)[0]
                    self.assertTrue((ROOT / test_path).is_file(), reference)


if __name__ == "__main__":
    unittest.main()
