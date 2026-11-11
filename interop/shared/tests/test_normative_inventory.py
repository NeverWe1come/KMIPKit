from __future__ import annotations

import json
import hashlib
import re
import unittest
from pathlib import Path

from tools.normative_catalog.attribute_policy_source import _SourceParser
from interop.shared.catalog.generate_test_catalog import _section_versions, oasis_section_indexes


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
            "operation_payload_field", "operation_payload_value_variant", "referenced_enum_value",
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
        expected_ids.update(
            target["target_id"] for target in self.targets
            if target["kind"] in {"operation_payload_field", "operation_payload_value_variant"}
        )
        header_dir = ROOT / "specification/oasis/pkcs11-3.0/upstream/include"
        functions = re.findall(
            r"^CK_PKCS11_FUNCTION_INFO\((C_[A-Za-z0-9_]+)\)",
            (header_dir / "pkcs11f.h").read_text(), re.MULTILINE,
        )
        expected_ids.update(f"KMIPKIT-PKCS11-FUNCTION-{offset:03d}" for offset in range(1, len(functions) + 1))
        return_codes = re.findall(
            r"^#define\s+(CKR_[A-Za-z0-9_]+)\s+0x[0-9A-Fa-f]+UL\b",
            (header_dir / "pkcs11t.h").read_text(), re.MULTILINE,
        )
        expected_ids.update(
            "KMIPKIT-PKCS11-RETURN-" + re.sub(r"[^A-Za-z0-9]+", "-", name.removeprefix("CKR_")).strip("-")
            for name in return_codes
        )
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
        self.assertEqual(281, sum(target["kind"] == "operation_payload_field" and target["protocol_versions"] == ["2.0"] and target["direction"] == "client_to_server" for target in self.targets))
        self.assertEqual(289, sum(target["kind"] == "operation_payload_field" and target["protocol_versions"] == ["2.1"] and target["direction"] == "client_to_server" for target in self.targets))
        self.assertEqual(24, sum(target["kind"] == "operation_payload_field" and target["protocol_versions"] == ["2.0"] and target["direction"] == "server_to_client" for target in self.targets))
        self.assertEqual(24, sum(target["kind"] == "operation_payload_field" and target["protocol_versions"] == ["2.1"] and target["direction"] == "server_to_client" for target in self.targets))
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

    def test_every_operation_payload_field_matches_its_pinned_oasis_table_row(self) -> None:
        tables_by_version = {}
        for version in ("2.0", "2.1"):
            source_path = ROOT / (
                "specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html"
                if version == "2.0"
                else "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
            )
            parser = _SourceParser()
            parser.feed(source_path.read_bytes().decode("cp1252"))
            tables_by_version[version] = parser.tables

        targets = [target for target in self.targets if target["kind"] == "operation_payload_field"]
        self.assertEqual(618, len(targets))
        source_descriptions = {}
        for target in targets:
            version = target["protocol_versions"][0]
            source_ref = target["source_refs"][0]
            candidates = [
                table for table in tables_by_version[version]
                if table["heading"].startswith(source_ref["section"] + " ")
            ]
            table_number = source_ref.get("table")
            if table_number:
                candidates = [
                    table for table in candidates
                    if any(f"Table {table_number}:" in caption for caption in table["captions"])
                ]
            else:
                candidates = [
                    table for table in candidates
                    if table["rows"] and table["rows"][0] and (
                        table["rows"][0][0] == f"{target['role'].title()} Payload"
                        or (target["role"] == "request" and target["direction"] == "server_to_client" and table["rows"][0][0] == "Message Payload")
                    )
                ]
            self.assertEqual(1, len(candidates), target["target_id"])
            rows = candidates[0]["rows"]
            self.assertIn(["Item", "REQUIRED", "Description"], rows, target["target_id"])
            header = rows.index(["Item", "REQUIRED", "Description"])
            exact_row = rows[header + target["row_index"]]
            self.assertGreaterEqual(len(exact_row), 2, target["target_id"])
            self.assertEqual(target["name"], exact_row[0], target["target_id"])
            self.assertEqual(target["requiredness"], exact_row[1], target["target_id"])
            source_descriptions[target["target_id"]] = exact_row[2] if len(exact_row) > 2 else ""
            self.assertIn("present", target["branch_classes"], target["target_id"])
            requiredness = target["requiredness"].casefold()
            if requiredness.startswith("yes") and not requiredness.startswith(("yes for single-part", "yes, unless")):
                self.assertIn("missing_rejected", target["branch_classes"], target["target_id"])
            else:
                if requiredness.startswith("no"):
                    self.assertIn("omitted", target["branch_classes"], target["target_id"])
                else:
                    self.assertNotIn("missing_rejected", target["branch_classes"], target["target_id"])
            if "repeat" in target["requiredness"].casefold():
                self.assertIn("occurrence_count_boundaries", target["branch_classes"], target["target_id"])
            description = target["description"].casefold()
            required_if = bool(re.search(r"\b(?:required|shall be present|must be present|shall be returned)\s+(?:if|when)\b", description)) or "if and only if" in target["requiredness"].casefold()
            recommended_if = bool(re.search(r"\bshould be specified\s+(?:if|when)\b", description))
            prohibited_if = bool(re.search(r"\bshall not be (?:present|specified)\s+(?:if|when|for)\b", description))
            if required_if or recommended_if or prohibited_if:
                self.assertIn("condition_true_and_false", target["branch_classes"], target["target_id"])
            if required_if:
                self.assertIn("required_when_condition_true", target["branch_classes"], target["target_id"])
            if recommended_if:
                self.assertIn("recommended_when_condition_true", target["branch_classes"], target["target_id"])
            if prohibited_if:
                self.assertIn("prohibited_when_condition_true", target["branch_classes"], target["target_id"])
        type_names = ("Date Time Extended", "Long Integer", "Big Integer", "Text String", "Byte String",
                      "Structure", "Integer", "Enumeration", "Boolean", "Date Time", "Interval")
        variants = [target for target in self.targets if target["kind"] == "operation_payload_value_variant"]
        expected_variants = {}
        for target in targets:
            description = source_descriptions[target["target_id"]]
            expected = []
            for name in type_names:
                type_pattern = re.escape(name).replace(r"\ ", r"[\s-]+")
                if re.search(r"\b(?:a|an)\s+" + type_pattern + r"\b", description, re.IGNORECASE):
                    expected.append(name)
            if expected:
                expected_variants[target["target_id"]] = expected
                self.assertEqual(expected, target["value_type_alternatives"])
            else:
                self.assertNotIn("value_type_alternatives", target)
        actual_variants = {}
        parent_targets = {target["target_id"]: target for target in targets}
        for variant in variants:
            parent_id = variant["parent_target_id"]
            actual_variants.setdefault(parent_id, []).append(variant["value_type"])
            with self.subTest(variant=variant["target_id"]):
                self.assertIn(variant["value_type"], expected_variants[parent_id])
                self.assertEqual(parent_targets[parent_id]["name"], variant["field_name"])
                self.assertEqual(parent_targets[parent_id]["operation_id"], variant["operation_id"])
                self.assertEqual(parent_targets[parent_id]["source_refs"], variant["source_refs"])
                self.assertEqual(parent_targets[parent_id]["protocol_versions"], variant["protocol_versions"])
                case = next(case for case in self.catalog["test_cases"] if case["requires"].get("normative_target_id") == variant["target_id"])
                self.assertEqual(variant["value_type"], case["requires"]["value_type"])
                self.assertIn("field_ttlv_type_matches_normative_variant", case["execution"]["assertions"])
        self.assertEqual(expected_variants, actual_variants)
        self.assertEqual(8, sum(variant["operation"] == "PKCS#11" for variant in variants))

    def test_payload_requiredness_cells_preserve_conditional_presence_semantics(self) -> None:
        targets = [target for target in self.targets if target["kind"] == "operation_payload_field"]
        self.assertEqual(618, len(targets))
        requiredness_values = {target["requiredness"].strip().casefold() for target in targets}
        self.assertTrue(any(value.startswith("yes, unless a failure") for value in requiredness_values))
        self.assertTrue(any(value.startswith("yes for single-part") for value in requiredness_values))
        self.assertIn("if and only if the key object is wrapped.", requiredness_values)
        self.assertTrue(any("may be repeated" in value for value in requiredness_values))

        for target in targets:
            with self.subTest(target=target["target_id"]):
                requiredness = target["requiredness"].strip().casefold()
                branches = target.get("presence_branches", [])
                if requiredness.startswith("yes for single-part"):
                    self.assertNotIn("missing_rejected", target["branch_classes"])
                    self.assertTrue(branches)
                    self.assertEqual({"required", "optional"}, {branch["presence"] for branch in branches})
                    self.assertTrue(any("single-part" in branch["condition"].casefold() for branch in branches))
                    self.assertTrue(any("multi-part" in branch["condition"].casefold() for branch in branches))
                elif requiredness.startswith("yes, unless a failure"):
                    self.assertNotIn("missing_rejected", target["branch_classes"])
                    self.assertEqual(
                        [
                            {"condition": "no failure", "presence": "required"},
                            {"condition": "failure", "presence": "optional"},
                        ],
                        branches,
                    )
                elif requiredness.startswith("if and only if"):
                    self.assertEqual(
                        [
                            {"condition": "key object is wrapped", "presence": "required"},
                            {"condition": "key object is not wrapped", "presence": "prohibited"},
                        ],
                        branches,
                    )
                elif requiredness.startswith("yes"):
                    self.assertIn("missing_rejected", target["branch_classes"])
                    self.assertEqual([], branches)
                else:
                    self.assertIn("omitted", target["branch_classes"])

        sign_data = [
            target for target in targets
            if target.get("operation") == "Sign" and target.get("role") == "request" and target["name"] == "Data"
        ]
        self.assertEqual({"2.0", "2.1"}, {target["protocol_versions"][0] for target in sign_data})
        for target in sign_data:
            self.assertEqual(
                [
                    {"condition": "single-part and Digested Data is absent", "presence": "required"},
                    {"condition": "single-part and Digested Data is supplied", "presence": "optional"},
                    {"condition": "multi-part", "presence": "optional"},
                ],
                target["presence_branches"],
            )

    def test_query_response_conditional_fields_have_both_presence_branches(self) -> None:
        targets = [
            target for target in self.targets
            if target["kind"] == "operation_payload_field"
            and target.get("operation") == "Query"
            and target.get("role") == "response"
            and target["name"] in {"Vendor Identification", "Extension Information"}
        ]
        self.assertEqual(8, len(targets))
        self.assertEqual(
            {("2.0", "client_to_server"), ("2.1", "client_to_server"),
             ("2.0", "server_to_client"), ("2.1", "server_to_client")},
            {(target["protocol_versions"][0], target["direction"]) for target in targets},
        )
        cases = {
            case["requires"]["normative_target_id"]: case
            for case in self.catalog["test_cases"]
            if case.get("requires", {}).get("normative_target_id")
        }
        for target in targets:
            with self.subTest(target=target["target_id"]):
                if target["name"] == "Vendor Identification":
                    expected = [
                        {"condition": "Query Server Information is requested", "presence": "required"},
                        {"condition": "Query Server Information is not requested", "presence": "optional"},
                    ]
                else:
                    peer = "server" if target["direction"] == "client_to_server" else "client"
                    condition = f"Query Extension List or Query Extension Map is requested and supported by the {peer}"
                    expected = [
                        {"condition": condition, "presence": "required"},
                        {"condition": f"{condition} is false", "presence": "optional"},
                    ]
                self.assertEqual(expected, target.get("presence_branches"))
                self.assertIn("SHALL be returned if", target["description"])
                case = cases[target["target_id"]]
                self.assertEqual(expected, case["requires"].get("presence_branches"))
                source_ref = target["source_refs"][0]
                self.assertIn(
                    {"document": f"OASIS KMIP {target['protocol_versions'][0]} Specification", "section": source_ref["section"], "table": source_ref["table"]}
                    if "table" in source_ref
                    else {"document": f"OASIS KMIP {target['protocol_versions'][0]} Specification", "section": source_ref["section"]},
                    case["normative_refs"],
                )

    def test_sign_missing_managed_object_parameters_requires_source_backed_error(self) -> None:
        case_id = "test-sign-missing-managed-object-cryptographic-parameters"
        case = next((case for case in self.catalog["test_cases"] if case["case_id"] == case_id), None)
        self.assertIsNotNone(case, case_id)
        scenario = case["requires"]["protocol_scenario"]
        self.assertEqual(["Sign"], scenario["operations"])
        self.assertEqual(["2.0", "2.1"], scenario["protocol_versions"])
        self.assertEqual(
            {"managed_object_has_cryptographic_parameters": False, "algorithm_requires_parameters": True},
            scenario["parameters"],
        )
        self.assertEqual("Operation Failed", scenario["expected"]["result_status"])
        self.assertEqual(["Bad Cryptographic Parameters"], scenario["expected"]["allowed_result_reasons"])

        target_by_id = {target["target_id"]: target for target in self.targets}
        targets = [target_by_id[target_id] for target_id in scenario["target_ids"]]
        self.assertTrue(any(target.get("operation") == "Sign" and target["name"] == "Cryptographic Parameters" for target in targets))
        self.assertTrue(any(target.get("name") == "Operation Failed" and target.get("parent_element_ids") == ["KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS"] for target in targets))
        self.assertTrue(any(target.get("name") == "Bad Cryptographic Parameters" and target.get("parent_element_ids") == ["KMIPKIT-ELEM-ENUMERATION-RESULT-REASON"] for target in targets))

        error_table_by_version = {"2.0": "304", "2.1": "336"}
        error_section_by_version = {"2.0": "6.1.49.1", "2.1": "6.1.55.1"}
        for version, table_number in error_table_by_version.items():
            source_path = ROOT / (
                "specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html"
                if version == "2.0"
                else "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
            )
            parser = _SourceParser()
            parser.feed(source_path.read_bytes().decode("cp1252"))
            tables = [table for table in parser.tables if table["captions"] == [f"Table {table_number}: Sign Errors"]]
            self.assertEqual(1, len(tables), version)
            source_text = " ".join(" ".join(row) for row in tables[0]["rows"])
            self.assertIn("Operation Failed", source_text)
            self.assertIn("Bad Cryptographic Parameters", source_text)
            self.assertIn(
                {"document": f"OASIS KMIP {version} Specification", "section": error_section_by_version[version]},
                case["normative_refs"],
            )
            self.assertIn(
                {"document": f"OASIS KMIP {version} Specification", "section": error_section_by_version[version], "table": table_number},
                case["normative_refs"],
            )
        self.assertIn("operation_failed_required_when_managed_object_lacks_required_parameters", scenario["assertions"])
        self.assertIn("allowed_sign_error_reason_is_source_backed", scenario["assertions"])

    def test_legacy_batch_and_async_lifecycles_list_every_ordered_operation(self) -> None:
        expected_operations = {
            "test-batch-create-activate-destroy": ["Create", "Activate", "Get Attribute List", "Destroy"],
            "test-batch-register-encrypt-decrypt-destroy": ["Register", "Activate", "Encrypt", "Decrypt", "Destroy"],
            "test-async-lifecycle-poll": ["Create", "Query Asynchronous Requests", "Poll"],
        }
        cases = {case["case_id"]: case for case in self.catalog["test_cases"]}
        operations = [target for target in self.targets if target["kind"] == "operation"]
        indexes = oasis_section_indexes(ROOT)
        for case_id, expected in expected_operations.items():
            case = cases[case_id]
            scenario = case["requires"].get("protocol_scenario")
            self.assertIsInstance(scenario, dict, case_id)
            self.assertEqual(expected, scenario["operations"], case_id)
            expected_versions = ["2.1"] if case_id == "test-async-lifecycle-poll" else ["2.0", "2.1"]
            self.assertEqual(expected_versions, scenario["protocol_versions"], case_id)
            for version in scenario["protocol_versions"]:
                version_refs = {
                    (reference["document"], reference["section"])
                    for reference in scenario["normative_refs"]
                    if reference["document"] == f"OASIS KMIP {version} Specification"
                }
                for operation_name in expected:
                    matching_targets = [
                        target for target in operations
                        if target["name"] == operation_name
                        and target["target_id"].startswith("KMIPKIT-ELEM-OP-C2S-")
                        and version in target["protocol_versions"]
                    ]
                    self.assertEqual(1, len(matching_targets), (case_id, version, operation_name))
                    for source_ref in matching_targets[0]["source_refs"]:
                        expected_section = _section_versions(source_ref["section"], indexes).get(version)
                        self.assertIsNotNone(expected_section, (case_id, version, operation_name))
                        self.assertIn(
                            (f"OASIS KMIP {version} Specification", expected_section),
                            version_refs,
                            (case_id, version, operation_name),
                        )

    def test_ttlv_data_type_boundaries_match_both_oasis_length_tables(self) -> None:
        allowed_lengths = {
            "Structure": "Varies, multiple of 8",
            "Integer": "4",
            "Long Integer": "8",
            "Big Integer": "Varies, multiple of 8",
            "Enumeration": "4",
            "Boolean": "8",
            "Text String": "Varies",
            "Byte String": "Varies",
            "Date Time": "8",
            "Interval": "4",
            "Date Time Extended": "8",
        }
        expected_branches = {"length_multiple_of_8", "exact_item_length_4", "exact_item_length_8",
                             "invalid_item_length_rejected", "minimal_alignment_padding"}
        targets = {target["name"]: target for target in self.targets if target["kind"] == "data_type"}
        self.assertEqual(set(allowed_lengths), set(targets))
        catalog_cases = {
            case["requires"]["normative_target_id"]: case
            for case in self.catalog["test_cases"]
            if case.get("requires", {}).get("target_kind") == "data_type"
        }
        for version, expected_table in (("2.0", "389"), ("2.1", "427")):
            path = ROOT / ("specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html" if version == "2.0"
                           else "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html")
            parser = _SourceParser()
            parser.feed(path.read_bytes().decode("cp1252"))
            table = [item for item in parser.tables if item["captions"] == [f"Table {expected_table}: Allowed Item Length Values"]]
            self.assertEqual(1, len(table))
            self.assertEqual([["Data Type", "Length"], *[[name, length] for name, length in allowed_lengths.items()]], table[0]["rows"])
        for name, target in targets.items():
            with self.subTest(data_type=name):
                self.assertTrue(target["branch_classes"])
                self.assertTrue(set(target["branch_classes"]).intersection(expected_branches))
                case = catalog_cases[target["target_id"]]
                self.assertEqual(target["branch_classes"], case["requires"]["branch_classes"])
                self.assertEqual("interop/shared/vectors/ttlv-boundaries.json", case["requires"]["boundary_vector_file"])
                self.assertIn("every_declared_boundary_branch_exercised", case["execution"]["assertions"])

    def test_ttlv_boundary_vectors_cover_every_public_data_type_and_wire_branch(self) -> None:
        vector_path = ROOT / "interop/shared/vectors/ttlv-boundaries.json"
        fixture = json.loads(vector_path.read_text())
        self.assertEqual(1, fixture["format_version"])
        type_codes = {target["name"]: target["wire_value"][-2:] for target in self.targets if target["kind"] == "data_type"}
        vectors = {vector["data_type"]: vector for vector in fixture["vectors"]}
        self.assertEqual(set(type_codes), set(vectors))
        targets = {target["name"]: target for target in self.targets if target["kind"] == "data_type"}
        expected_samples = {
            "Structure": {"empty_structure", "nested_integer", "ordered_children", "reversed_children", "non_multiple_of_8_length", "truncated_child"},
            "Integer": {"signed_minimum", "zero", "signed_maximum", "wrong_length_3"},
            "Long Integer": {"signed_minimum", "zero", "signed_maximum", "wrong_length_4"},
            "Big Integer": {"positive_one", "negative_one", "positive_multiword", "negative_multiword", "non_multiple_length_4", "excessive_sign_extension"},
            "Enumeration": {"assigned_zero", "unknown_value_preserved", "extension_allocation", "wrong_length_3"},
            "Boolean": {"false", "true", "invalid_boolean_2", "wrong_length_4"},
            "Text String": {"empty", "single_ascii_and_padding", "seven_bytes", "eight_bytes", "nine_bytes", "multibyte_utf8", "invalid_utf8"},
            "Byte String": {"empty", "one_byte", "seven_bytes", "eight_bytes", "nine_bytes"},
            "Date Time": {"one_second_before_epoch", "unix_epoch", "one_second_after_epoch", "wrong_length_7"},
            "Interval": {"zero", "unsigned_maximum", "wrong_length_3"},
            "Date Time Extended": {"zero_microseconds", "one_microsecond", "negative_one_microsecond", "wrong_length_7"},
        }
        padding = {"Integer": 4, "Enumeration": 4, "Interval": 4}
        for name, vector in vectors.items():
            with self.subTest(data_type=name):
                self.assertEqual(type_codes[name], vector["type_code"])
                self.assertEqual(targets[name]["branch_classes"], vector["branch_classes"])
                self.assertEqual(expected_samples[name], {sample["branch"] for sample in vector["samples"]})
                for sample in vector["samples"]:
                    raw = bytes.fromhex(sample["hex"])
                    self.assertEqual("420001", sample["hex"][:6])
                    self.assertEqual(vector["type_code"].casefold(), sample["hex"][6:8].casefold())
                    declared_length = int(sample["hex"][8:16], 16)
                    value = raw[8:]
                    if sample["valid"]:
                        extra_padding = 0 if name in {"Structure", "Long Integer", "Big Integer", "Boolean", "Date Time", "Date Time Extended"} else padding.get(name, (-declared_length) % 8)
                        self.assertEqual(declared_length + extra_padding, len(value))
                        self.assertEqual(0, len(raw) % 8)
                    else:
                        self.assertTrue(sample.get("error_class"))

    def test_date_time_extended_precision_branch_is_pinned_to_oasis_definition(self) -> None:
        for version, table_number in (("2.0", "390"), ("2.1", "428")):
            path = ROOT / ("specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html" if version == "2.0"
                           else "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html")
            parser = _SourceParser()
            parser.feed(path.read_bytes().decode("cp1252"))
            tables = [table for table in parser.tables if table["captions"] == [f"Table {table_number}: Adjustment Type Descriptions"]]
            self.assertEqual(1, len(tables))
            increment = next(row[1] for row in tables[0]["rows"] if row[0] == "Increment")
            self.assertIn("1 microsecond for Date Time Extended", increment)

    def test_attribute_cases_retain_every_source_policy_and_exercise_its_branches(self) -> None:
        source_attributes = {
            element["element_id"]: element for element in self.source["elements"]
            if element.get("kind") == "attribute"
        }
        targets = {target["target_id"]: target for target in self.targets if target["kind"] == "attribute"}
        cases = {
            case["requires"].get("normative_target_id"): case
            for case in self.catalog["test_cases"] if case["category"] == "attributes"
        }
        self.assertEqual(63, len(targets))
        for target_id, target in targets.items():
            with self.subTest(attribute=target["name"]):
                source = source_attributes[target_id]
                policy = target["attribute_policy"]
                case = cases[target_id]
                self.assertEqual(policy, case["requires"]["attribute_policy"])
                self.assertEqual(source.get("source_encoding"), policy["value_type"])
                self.assertEqual(source.get("source_initially_set_by"), policy["initially_set_by"])
                self.assertEqual(source.get("source_modifiable_by_client"), policy["modifiable_by_client"])
                self.assertEqual(source.get("source_deletable_by_client"), policy["deletable_by_client"])
                self.assertEqual(source.get("source_always_required"), policy["always_required"])
                self.assertEqual(source.get("source_multiple_instances_permitted"), policy["multiple_instances_permitted"])
                self.assertEqual(source.get("source_policy_table"), policy["policy_table"])
                self.assertEqual([rule.get("source_text", "") for rule in source.get("source_conditional_rules", [])], policy["conditional_rules"])
                self.assertEqual([rule.get("source_text", "") for rule in source.get("source_operation_restrictions", [])], policy["operation_restrictions"])
                self.assertIn("read_back_attribute", policy["branch_classes"])
                self.assertIn("policy_branches_match_oasis", case["execution"]["assertions"])
                self.assertEqual("mutating", case["execution"]["mutation"])
                if source.get("source_encoding") is None:
                    self.assertIn("value_type_defined_by_source_child_fields", policy["branch_classes"])
                if source.get("source_multiple_instances_permitted") == "Yes":
                    self.assertIn("multiple_instances_permitted", policy["branch_classes"])
                elif source.get("source_multiple_instances_permitted") == "No":
                    self.assertIn("single_instance_only", policy["branch_classes"])

    def test_pkcs11_referenced_enumerations_match_the_pinned_oasis_headers(self) -> None:
        header_dir = ROOT / "specification/oasis/pkcs11-3.0/upstream/include"
        function_path = header_dir / "pkcs11f.h"
        return_path = header_dir / "pkcs11t.h"
        expected_hashes = {
            "pkcs11.h": "260bbef88360ddbc8f7f668622c688ed7e6dab939e1eb2fdd6392abb571637e5",
            "pkcs11f.h": "c52bdac9e30f982cb5ad57db946a8ff6076efea744a90d684835d7269babab9b",
            "pkcs11t.h": "1f6763a72dc33a97d964d3d4b860870e1b4d4bf76b8705bdee26078caf4994cc",
        }
        self.assertEqual(expected_hashes["pkcs11.h"], hashlib.sha256((header_dir / "pkcs11.h").read_bytes()).hexdigest())
        self.assertEqual(expected_hashes["pkcs11f.h"], hashlib.sha256(function_path.read_bytes()).hexdigest())
        self.assertEqual(expected_hashes["pkcs11t.h"], hashlib.sha256(return_path.read_bytes()).hexdigest())

        functions = re.findall(r"^CK_PKCS11_FUNCTION_INFO\((C_[A-Za-z0-9_]+)\)", function_path.read_text(), re.MULTILINE)
        expected = {
            ("PKCS#11 Function", name, f"{offset:08X}", "assigned")
            for offset, name in enumerate(functions, start=1)
        }
        return_values = re.findall(
            r"^#define\s+(CKR_[A-Za-z0-9_]+)\s+(0x[0-9A-Fa-f]+)UL\b",
            return_path.read_text(),
            re.MULTILINE,
        )
        expected.update(
            ("PKCS#11 Return Code", name, f"{int(value, 16):08X}", "extension" if name == "CKR_VENDOR_DEFINED" else "assigned")
            for name, value in return_values
        )
        actual = {
            (target["parent_name"], target["name"], target["wire_value"], target.get("allocation", "assigned"))
            for target in self.targets if target["kind"] == "referenced_enum_value"
        }
        self.assertEqual(92, len(functions))
        self.assertEqual(96, sum(allocation == "assigned" for parent, _, _, allocation in expected if parent == "PKCS#11 Return Code"))
        self.assertEqual(1, sum(allocation == "extension" for parent, _, _, allocation in expected if parent == "PKCS#11 Return Code"))
        self.assertEqual(expected, actual)
        for target in self.targets:
            if target["kind"] != "referenced_enum_value":
                continue
            with self.subTest(target=target["target_id"]):
                self.assertEqual(["2.0", "2.1"], target["protocol_versions"])
                kmip_refs = {ref["document"]: ref["section"] for ref in target["source_refs"] if "section" in ref}
                self.assertEqual({"OASIS KMIP 2.0 Specification": "11.38" if target["parent_name"] == "PKCS#11 Function" else "11.39",
                                  "OASIS KMIP 2.1 Specification": "11.38" if target["parent_name"] == "PKCS#11 Function" else "11.39"}, kmip_refs)
                external = [ref for ref in target["source_refs"] if ref.get("document") == "OASIS PKCS #11 3.0"]
                self.assertEqual(1, len(external))
                self.assertEqual(expected_hashes[external[0]["artifact"].rsplit("/", 1)[-1]], external[0]["sha256"])
    def test_set_endpoint_role_is_an_executable_client_operation(self) -> None:
        matches = [
            case for case in self.catalog["test_cases"]
            if case.get("case_id") == "test-op-set-endpoint-role"
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

    def test_batch_and_async_state_scenarios_are_explicit_and_source_linked(self) -> None:
        cases = {
            case["case_id"]: case for case in self.catalog["test_cases"]
            if case["category"] in {"batch_workflows", "asynchronous_workflows"}
        }
        required = {
            "test-batch-single-item-default-continuation",
            "test-batch-stop-on-first-failure",
            "test-batch-continue-after-failure",
            "test-batch-undo-rolls-back-prior-success",
            "test-batch-order-required-placeholder-resolution",
            "test-batch-order-false-correlates-response-items",
            "test-batch-order-omitted-defaults-true",
            "test-batch-count-matches-items",
            "test-batch-unique-item-id-response-correlation",
            "test-batch-count-zero-rejected",
            "test-batch-count-mismatch-rejected",
            "test-batch-single-item-options-omitted",
            "test-batch-placeholder-forward-reference-rejected",
            "test-batch-placeholder-without-producer-rejected",
            "test-batch-placeholder-chain-resolves",
            "test-async-indicator-omitted-default",
            "test-async-indicator-prohibited-synchronous",
            "test-async-indicator-optional-completes-or-pends",
            "test-async-indicator-mandatory-pending",
            "test-async-poll-repeated-pending-to-success",
            "test-async-poll-final-operation-failure",
            "test-async-unknown-correlation-rejected",
            "test-async-cancel-result-canceled",
            "test-async-cancel-result-unable-to-cancel",
            "test-async-cancel-result-completed",
            "test-async-cancel-result-failed",
            "test-async-cancel-result-unavailable",
            "test-async-cancel-vs-poll-race",
            "test-async-process-then-poll-state",
            "test-async-query-outstanding-filter",
            "test-async-no-recursive-async-poll-cancel",
        }
        self.assertTrue(required <= set(cases), f"missing executable state scenarios: {sorted(required - set(cases))}")
        target_ids = {target["target_id"] for target in self.targets}
        for case_id in required:
            case = cases[case_id]
            scenario = case["requires"].get("protocol_scenario")
            self.assertIsInstance(scenario, dict, case_id)
            self.assertEqual(case_id, scenario["scenario_id"], case_id)
            self.assertTrue(scenario["target_ids"], case_id)
            self.assertTrue(set(scenario["target_ids"]) <= target_ids, case_id)
            self.assertTrue(scenario["operations"], case_id)
            self.assertTrue(scenario["steps"], case_id)
            self.assertTrue(scenario["assertions"], case_id)
            self.assertTrue(case["normative_refs"], case_id)
            self.assertEqual("planned", case["execution"]["status"], case_id)
            operation_target_ids = {
                target["target_id"] for target in self.targets
                if target["kind"] == "operation" and target["name"] in scenario["operations"]
                and set(target["protocol_versions"]).intersection(scenario["protocol_versions"])
            }
            self.assertTrue(operation_target_ids <= set(scenario["target_ids"]), case_id)
            if case["category"] == "batch_workflows":
                self.assertEqual(["2.0", "2.1"], case["execution"]["protocol_versions"], case_id)
                self.assertTrue({"9.5", "9.7"} & {ref["section"] for ref in case["normative_refs"]}, case_id)
                if case_id in {"test-batch-count-zero-rejected", "test-batch-count-mismatch-rejected"}:
                    self.assertEqual("local_model", case["execution"]["mode"], case_id)
            elif case_id == "test-async-query-outstanding-filter":
                self.assertEqual(["2.1"], case["execution"]["protocol_versions"], case_id)
        cancellation_cases = {
            case["requires"]["protocol_scenario"]["expected"]["cancel_response_value"]
            for case_id, case in cases.items() if case_id.startswith("test-async-cancel-result-")
        }
        self.assertEqual({"Canceled", "Unable to Cancel", "Completed", "Failed", "Unavailable"}, cancellation_cases)
        indicator_cases = {
            case["requires"]["protocol_scenario"]["parameters"].get("asynchronous_indicator")
            for case_id, case in cases.items() if case_id.startswith("test-async-indicator-")
        }
        self.assertEqual({"omitted", "Prohibited", "Optional", "Mandatory"}, indicator_cases)
        indicator_values = {
            target["name"] for target in self.targets
            if target["kind"] == "enumeration_value"
            and "KMIPKIT-ELEM-ENUMERATION-ASYNCHRONOUS-INDICATOR" in target.get("parent_element_ids", [])
            and target.get("allocation") == "assigned"
        }
        self.assertEqual({"Prohibited", "Optional", "Mandatory"}, indicator_values)

    def test_every_assigned_bitmask_combination_is_a_required_local_case(self) -> None:
        expected_counts = {
            "Cryptographic Usage Mask": 1 << 16,
            "Protection Storage Mask": 1 << 14,
            "Storage Status Mask": 1 << 3,
        }
        for family, combination_count in expected_counts.items():
            case_id = "test-bitmask-" + re.sub(r"[^a-z0-9]+", "-", family.casefold()).strip("-") + "-all-combinations"
            case = next((item for item in self.catalog["test_cases"] if item["case_id"] == case_id), None)
            self.assertIsNotNone(case, case_id)
            requirements = case["requires"]["exhaustive_bitmask_combinations"]
            self.assertEqual(family, case["requires"]["bitmask_type"])
            self.assertEqual(combination_count, requirements["combination_count"])
            self.assertEqual(["2.0", "2.1"], case["execution"]["protocol_versions"])
            self.assertEqual("local_model", case["execution"]["mode"])
            self.assertIn("enumerate_every_assigned_flag_combination", case["execution"]["steps"])
            self.assertIn("every_assigned_combination_roundtrips", case["execution"]["assertions"])
            values = [
                target for target in self.targets
                if target["kind"] == "bitmask_value" and target.get("allocation") == "assigned"
                and target["target_id"] in requirements["flag_target_ids"]
            ]
            self.assertEqual(16 if family == "Cryptographic Usage Mask" else 14 if family == "Protection Storage Mask" else 3, len(values))
            self.assertEqual({target["target_id"] for target in values}, set(requirements["flag_target_ids"]))
            self.assertTrue(all(target["protocol_versions"] == ["2.0", "2.1"] for target in values))
            flag_values = [int(value, 16) for value in requirements["flag_wire_values"]]
            self.assertTrue(all(value > 0 and value & (value - 1) == 0 for value in flag_values))
            masks = set()
            for subset in range(combination_count):
                mask = 0
                for index, flag in enumerate(flag_values):
                    if subset & (1 << index):
                        mask |= flag
                masks.add(mask)
            self.assertEqual(combination_count, len(masks))
            self.assertIn(0, masks)


if __name__ == "__main__":
    unittest.main()
