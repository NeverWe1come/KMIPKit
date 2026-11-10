from __future__ import annotations

import json
import hashlib
import unittest
from pathlib import Path

from tools.normative_catalog.attribute_policy_source import _SourceParser


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
            and (
                element.get("scope_state") in {"client_1_0", "client_1_1"}
                or (element.get("kind") == "tag" and element.get("allocation") == "reserved")
            )
        }
        expected_ids.update(item["range_id"] for item in self.source["tag_ranges"])
        target_ids = [target["target_id"] for target in self.targets]
        self.assertEqual(len(target_ids), len(set(target_ids)), "normative target IDs must be unique")
        self.assertEqual(expected_ids, set(target_ids))
        for target in self.targets:
            with self.subTest(target=target["target_id"]):
                self.assertIn(target["kind"], expected_kinds | {"tag_range"})
                self.assertIn(target["protocol_versions"], (["2.0"], ["2.1"], ["2.0", "2.1"]))
                self.assertTrue(target["source_refs"])

    def test_assigned_values_tags_and_fields_are_individual_targets(self) -> None:
        targets_by_kind = {}
        for target in self.targets:
            targets_by_kind.setdefault(target["kind"], []).append(target)

        self.assertEqual(644, sum(target["kind"] == "enumeration_value" and target.get("allocation") == "assigned" for target in self.targets))
        self.assertEqual(19, sum(target["kind"] == "enumeration_value" and target.get("allocation") == "reserved" for target in self.targets))
        self.assertEqual(60, sum(target["kind"] == "enumeration_value" and target.get("allocation") == "extension" for target in self.targets))
        self.assertEqual(33, sum(target["kind"] == "bitmask_value" and target.get("allocation") == "assigned" for target in self.targets))
        self.assertEqual(8, sum(target["kind"] == "bitmask_value" and target.get("allocation") == "reserved" for target in self.targets))
        self.assertEqual(3, sum(target["kind"] == "bitmask_value" and target.get("allocation") == "extension" for target in self.targets))
        self.assertEqual(354, sum(target["kind"] == "tag" and target.get("allocation") == "assigned" for target in self.targets))
        self.assertEqual(20, sum(target["kind"] == "tag" and target.get("allocation") == "reserved" for target in self.targets))
        self.assertEqual(5, sum(target["kind"] == "tag_range" for target in self.targets))
        self.assertEqual(11, sum(target["kind"] == "data_type" for target in self.targets))
        for target in targets_by_kind["enumeration_value"] + targets_by_kind["bitmask_value"] + targets_by_kind["tag"]:
            with self.subTest(target=target["target_id"]):
                self.assertIsInstance(target["wire_value"], str)
                self.assertTrue(target["wire_value"])
        self.assertGreaterEqual(len(targets_by_kind["message_field"]), 71)
        self.assertGreaterEqual(len(targets_by_kind["structure_member"]), 241)

    def test_kmip_21_only_enum_assignments_are_not_run_against_kmip_20(self) -> None:
        parent_names = {
            element["element_id"]: element["name"]
            for element in self.source["elements"] if element["kind"] == "enumeration"
        }
        actual = {
            (parent_names[target["parent_element_ids"][0]], target["name"], target["wire_value"])
            for target in self.targets
            if target["kind"] == "enumeration_value" and target.get("allocation") == "assigned" and target["protocol_versions"] == ["2.1"]
        }
        expected = {
            ("Operation", "Get Constraints", "00000038"),
            ("Operation", "Ping", "0000003B"),
            ("Operation", "Process", "0000003A"),
            ("Operation", "Query Asynchronous Requests", "00000039"),
            ("Operation", "Set Constraints", "00000037"),
            ("Operation", "Set Defaults", "00000036"),
            ("Processing Stage", "Completed", "00000003"),
            ("Processing Stage", "In Process", "00000002"),
            ("Processing Stage", "Submitted", "00000001"),
            ("Result Reason", "Constraint Violation", "0000004B"),
            ("Result Reason", "Duplicate Process Request", "0000004C"),
            ("Result Reason", "Unknown Object Group", "0000004A"),
            ("Rotate Name Type", "Uninterpreted Text String", "00000001"),
            ("Rotate Name Type", "URI", "00000002"),
        }
        self.assertEqual(expected, actual)

        delta_path = ROOT / "interop/shared/catalog/kmip-2.1-enum-deltas.json"
        deltas = json.loads(delta_path.read_text(encoding="utf-8"))
        source_path = ROOT / deltas["source"]["path"]
        source_bytes = source_path.read_bytes()
        self.assertEqual(deltas["source"]["sha256"], hashlib.sha256(source_bytes).hexdigest())
        parser = _SourceParser()
        parser.feed(source_bytes.decode("cp1252"))
        for group in deltas["deltas"]:
            table_caption = f"Table {group['table']}:"
            matching_tables = [table for table in parser.tables if any(caption.startswith(table_caption) for caption in table["captions"])]
            self.assertEqual(1, len(matching_tables), group["enumeration"])
            source_values = {(row[0], row[1].upper()) for row in matching_tables[0]["rows"] if len(row) >= 2}
            for value in group["values"]:
                with self.subTest(enumeration=group["enumeration"], value=value["name"]):
                    self.assertIn((value["name"], value["wire_value"]), source_values)

    def test_every_target_has_one_case_per_applicable_version(self) -> None:
        cases_by_target: dict[str, list[dict]] = {}
        for case in self.catalog["test_cases"]:
            target_id = case.get("requires", {}).get("normative_target_id")
            if target_id:
                cases_by_target.setdefault(target_id, []).append(case)
        for target in self.targets:
            with self.subTest(target=target["target_id"]):
                matches = cases_by_target.get(target["target_id"], [])
                self.assertEqual(1, len(matches))
                self.assertEqual(target["protocol_versions"], matches[0]["execution"]["protocol_versions"])
                self.assertEqual(target["protocol_versions"], matches[0]["requires"]["target_protocol_versions"])
        self.assertEqual({target["target_id"] for target in self.targets}, set(cases_by_target))

    def test_set_endpoint_role_is_an_executable_client_operation(self) -> None:
        matches = [
            case for case in self.catalog["test_cases"]
            if case.get("operation") == "Set Endpoint Role"
            and case["category"] != "server_initiated_operations"
        ]
        self.assertEqual(1, len(matches))
        self.assertEqual(["2.0", "2.1"], matches[0]["execution"]["protocol_versions"])

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
