from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
SOURCE_PATH = ROOT / "specification/catalog/kmip-2.1.json"


class NormativeInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.source = json.loads(SOURCE_PATH.read_text(encoding="utf-8"))
        cls.targets = cls.catalog.get("normative_targets", [])

    def test_all_supported_normative_elements_have_versioned_targets(self) -> None:
        expected_kinds = {
            "operation", "object_type", "attribute", "tag", "enumeration_value",
            "bitmask_value", "data_type", "message_field", "structure_member",
            "operation_structure", "object_structure", "attribute_structure", "credential",
        }
        expected_ids = {
            element["element_id"] for element in self.source["elements"]
            if element.get("kind") in expected_kinds
            and element.get("scope_state") in {"client_1_0", "client_1_1"}
            and element.get("allocation", "assigned") == "assigned"
        }
        target_ids = [target["target_id"] for target in self.targets]
        self.assertEqual(len(target_ids), len(set(target_ids)), "normative target IDs must be unique")
        self.assertEqual(expected_ids, set(target_ids))
        for target in self.targets:
            with self.subTest(target=target["target_id"]):
                self.assertIn(target["kind"], expected_kinds)
                self.assertIn(target["protocol_versions"], (["2.0"], ["2.1"], ["2.0", "2.1"]))
                self.assertTrue(target["source_refs"])

    def test_assigned_values_tags_and_fields_are_individual_targets(self) -> None:
        targets_by_kind = {}
        for target in self.targets:
            targets_by_kind.setdefault(target["kind"], []).append(target)

        self.assertEqual(644, sum(target["kind"] == "enumeration_value" for target in self.targets))
        self.assertEqual(33, sum(target["kind"] == "bitmask_value" for target in self.targets))
        self.assertEqual(354, sum(target["kind"] == "tag" for target in self.targets))
        self.assertEqual(11, sum(target["kind"] == "data_type" for target in self.targets))
        for target in targets_by_kind["enumeration_value"] + targets_by_kind["bitmask_value"] + targets_by_kind["tag"]:
            with self.subTest(target=target["target_id"]):
                self.assertIsInstance(target["wire_value"], str)
                self.assertTrue(target["wire_value"])
        self.assertGreaterEqual(len(targets_by_kind["message_field"]), 71)
        self.assertGreaterEqual(len(targets_by_kind["structure_member"]), 241)

    def test_set_endpoint_role_is_an_executable_client_operation(self) -> None:
        matches = [
            case for case in self.catalog["test_cases"]
            if case.get("operation") == "Set Endpoint Role"
            and case["category"] != "server_initiated_operations"
        ]
        self.assertEqual(1, len(matches))
        self.assertEqual(["2.1"], matches[0]["execution"]["protocol_versions"])

    def test_create_recipe_does_not_use_create_for_noncreatable_object_types(self) -> None:
        cases = {
            case["requires"]["object_type"]: case
            for case in self.catalog["test_cases"] if case["category"] == "managed_objects"
        }
        self.assertEqual("Create Key Pair", cases["Private Key"]["operation"])
        self.assertEqual("Create Key Pair", cases["Public Key"]["operation"])
        self.assertEqual("Create Split Key", cases["Split Key"]["operation"])

    def test_version_rejection_cases_assert_the_request_was_never_sent(self) -> None:
        for case in self.catalog["test_cases"]:
            if case["category"] != "version_boundaries":
                continue
            with self.subTest(case=case["case_id"]):
                self.assertEqual("local_model", case["execution"]["mode"])
                self.assertIn("transport_call_count_is_zero", case["execution"]["assertions"])


if __name__ == "__main__":
    unittest.main()
