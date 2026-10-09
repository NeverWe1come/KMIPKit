"""Tests for the deterministic source-backed attribute-policy lookup."""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog.validate import ELEMENT_FIELDS, ELEMENT_KINDS, load_validated_catalog


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools" / "normative_catalog" / "generate_attribute_policy.py"
OUTPUT_PATH = "crates/kmipkit-protocol/src/generated/attribute_policy.rs"
GENERATOR_MODULE = "tools.normative_catalog.generate_attribute_policy"
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"


def _load_catalog() -> dict[str, Any]:
    return load_validated_catalog(CATALOG_PATH.read_bytes(), ROOT)


def _load_renderer(testcase: unittest.TestCase):
    module_spec = importlib.util.find_spec(GENERATOR_MODULE)
    testcase.assertIsNotNone(
        module_spec,
        "attribute-policy generator must be implemented before its output can be exercised",
    )
    from tools.normative_catalog.generate_attribute_policy import render_rust

    return render_rust


def _tag_value_by_id(catalog: dict[str, Any], element_id: str) -> str:
    for element in catalog["elements"]:
        if element.get("element_id") == element_id and element.get("kind") == "tag":
            return str(element["wire_value"]).lower().removeprefix("0x").replace("_", "")
    raise AssertionError(f"attribute references missing assigned tag {element_id}")


def _rust_struct_records(rendered: str, struct_name: str) -> list[str]:
    """Return Rust struct bodies, including nested source-rule records."""
    records: list[str] = []
    declaration = re.compile(rf"\b{re.escape(struct_name)}\s*\{{")
    for match in declaration.finditer(rendered):
        opening = rendered.find("{", match.start())
        depth = 0
        in_string = False
        escaped = False
        for index in range(opening, len(rendered)):
            character = rendered[index]
            if in_string:
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == '"':
                    in_string = False
                continue
            if character == '"':
                in_string = True
            elif character == "{":
                depth += 1
            elif character == "}":
                depth -= 1
                if depth == 0:
                    records.append(rendered[opening + 1 : index])
                    break
        else:
            raise AssertionError(f"unterminated generated {struct_name} record")
    return records


def _attribute_policy_record(
    testcase: unittest.TestCase,
    rendered: str,
    wire_value: str,
) -> str:
    """Find the unique AttributePolicy record whose numeric tag is wire_value.

    Renderer contract for T014: each standard policy is an `AttributePolicy`
    struct literal with a numeric `tag` field, `source_policy_table`, the four
    source cells, and `source_operation_restrictions` / `source_conditional_rules`
    arrays in that same struct body. Vendor Attribute uses a separate
    `VendorAttributePolicy` record.
    """
    expected_value = int(wire_value, 16)
    records = []
    for record in _rust_struct_records(rendered, "AttributePolicy"):
        actual_value = _tag_value_from_record(record)
        if actual_value is None:
            continue
        if actual_value == expected_value:
            records.append(record)
    testcase.assertEqual(
        len(records),
        1,
        f"expected exactly one AttributePolicy record keyed by assigned tag {wire_value}",
    )
    return records[0]


def _tag_value_from_record(record: str) -> int | None:
    tag_field = re.search(r"\btag\s*:\s*(0x[\da-fA-F_]+|\d+)", record)
    if tag_field is None:
        return None
    tag_literal = tag_field.group(1).replace("_", "")
    return int(tag_literal, 16) if tag_literal.lower().startswith("0x") else int(tag_literal)


def _vendor_attribute_policy_record(testcase: unittest.TestCase, rendered: str) -> str:
    records = [
        record
        for record in _rust_struct_records(rendered, "VendorAttributePolicy")
        if re.search(r"\bequals\s*:\s*\"[^\"]+\"", record)
    ]
    testcase.assertEqual(len(records), 1, "expected one separate VendorAttributePolicy record")
    return records[0]


def _assert_source_value(
    testcase: unittest.TestCase,
    record: str,
    field: str,
    value: str,
) -> None:
    testcase.assertIn(f"{field}: {json.dumps(value, ensure_ascii=False)}", record)


def _assert_source_literal(testcase: unittest.TestCase, record: str, value: str) -> None:
    testcase.assertIn(json.dumps(value, ensure_ascii=False), record)


def _copy_validated_catalog_repo(destination: Path) -> None:
    """Create the minimal committed input tree required by catalog validation."""
    (destination / "specification").mkdir()
    shutil.copy2(ROOT / "AGENTS.md", destination / "AGENTS.md")
    shutil.copytree(ROOT / "specification" / "oasis", destination / "specification" / "oasis")
    shutil.copytree(ROOT / "specification" / "catalog", destination / "specification" / "catalog")
    subprocess.run(["git", "init", "-q"], cwd=destination, check=True)
    subprocess.run(
        ["git", "-c", "core.autocrlf=false", "add", "specification/oasis"],
        cwd=destination,
        check=True,
    )
    subprocess.run(
        [
            "git",
            "-c",
            "user.name=Catalog Generator Tests",
            "-c",
            "user.email=catalog-generator-tests@example.invalid",
            "commit",
            "-qm",
            "test fixture sources",
        ],
        cwd=destination,
        check=True,
    )


def _run_generator(root: Path, mode: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), mode, "--repo-root", str(root)],
        cwd=ROOT,
        capture_output=True,
        check=False,
        text=True,
    )


class AttributePolicyRenderingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = _load_catalog()

    def _render(self, catalog: dict[str, Any] | None = None) -> str:
        return _load_renderer(self)(self.catalog if catalog is None else catalog)

    def test_renders_a_tag_keyed_record_for_every_standard_attribute(self) -> None:
        rendered = self._render()
        standard_attributes = [
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("source_policy_table")
        ]

        self.assertEqual(len(standard_attributes), 62)
        for attribute in standard_attributes:
            with self.subTest(attribute=attribute["name"]):
                self.assertEqual(len(attribute["parent_element_ids"]), 1)
                tag = _tag_value_by_id(self.catalog, attribute["parent_element_ids"][0])
                record = _attribute_policy_record(self, rendered, tag)

                _assert_source_value(self, record, "source_policy_table", attribute["source_policy_table"])
                for field in (
                    "source_initially_set_by",
                    "source_modifiable_by_client",
                    "source_deletable_by_client",
                    "source_always_required",
                ):
                    _assert_source_value(self, record, field, attribute[field])
                for field in ("source_operation_restrictions", "source_conditional_rules"):
                    self.assertRegex(record, rf"\b{field}\s*:\s*&?\[")
                    for rule in attribute[field]:
                        _assert_source_value(self, record, "source_text", rule["source_text"])
                        for source_ref in rule["source_refs"]:
                            _assert_source_literal(self, record, source_ref["section"])

    def test_preserves_conditional_rule_text_and_qualified_source_values(self) -> None:
        rendered = self._render()
        conditional_attributes = [
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("source_policy_table")
        ]
        for attribute in conditional_attributes:
            tag = _tag_value_by_id(self.catalog, attribute["parent_element_ids"][0])
            record = _attribute_policy_record(self, rendered, tag)
            for rule in attribute["source_conditional_rules"]:
                with self.subTest(attribute=attribute["name"], source_text=rule["source_text"]):
                    _assert_source_value(self, record, "source_text", rule["source_text"])
                    for source_ref in rule["source_refs"]:
                        _assert_source_literal(self, record, source_ref["section"])
            for field in ("source_modifiable_by_client", "source_deletable_by_client"):
                if attribute[field] not in {"Yes", "No"}:
                    with self.subTest(attribute=attribute["name"], source_value=attribute[field]):
                        _assert_source_value(self, record, field, attribute[field])

    def test_does_not_generate_entries_for_unknown_tags(self) -> None:
        catalog = {
            "elements": [
                *self.catalog["elements"],
                {
                    "element_id": "KMIPKIT-ELEM-TAG-54ABCD",
                    "kind": "tag",
                    "name": "Unmapped Vendor Tag",
                    "wire_value": "54ABCD",
                    "allocation": "extension",
                },
            ]
        }

        rendered = self._render(catalog)

        unknown_records = [
            record
            for record in _rust_struct_records(rendered, "AttributePolicy")
            if _tag_value_from_record(record) == 0x54ABCD
        ]
        self.assertEqual(unknown_records, [])

    def test_does_not_infer_policy_from_an_unmapped_name_form_reference(self) -> None:
        standard_attribute = next(
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("source_name") == "Activation Date"
        )
        name_form_reference = {
            "vendor_identification": "KMIPKit.UnmappedVendor",
            "attribute_name": standard_attribute["source_name"],
        }
        attribute_reference_structure = next(
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute_structure" and item.get("name") == "Attribute Reference"
        )
        # A message_field's allowed source_comment carries the request probe into renderer input.
        probe_field = {
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-NAME-FORM-PROBE",
            "kind": "message_field",
            "name": "Attribute Reference",
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "5.5"}],
            "direction": "client_to_server",
            "scope_state": "client_1_0",
            "parent_element_ids": [attribute_reference_structure["element_id"]],
            "requirement_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": "KMIPKIT-0016",
            "implementation_refs": [],
            "verification_refs": [],
            "source_encoding": "Structure",
            "source_requiredness": "Optional",
            "source_comment": (
                "Name-form reference probe: "
                f"Vendor Identification={name_form_reference['vendor_identification']}; "
                f"Attribute Name={name_form_reference['attribute_name']}"
            ),
        }
        self.assertIn(probe_field["kind"], ELEMENT_KINDS)
        self.assertFalse(set(probe_field) - ELEMENT_FIELDS)
        catalog = {
            **self.catalog,
            "elements": [*self.catalog["elements"], probe_field],
        }
        rendered = self._render(catalog)
        standard_tag = _tag_value_by_id(self.catalog, standard_attribute["parent_element_ids"][0])
        standard_record = _attribute_policy_record(self, rendered, standard_tag)
        policy_records = [
            record
            for record in _rust_struct_records(rendered, "AttributePolicy")
            if _tag_value_from_record(record) is not None
        ]

        self.assertEqual(len(policy_records), 62)
        for record in policy_records:
            with self.subTest(record=record[:100]):
                self.assertIsNotNone(_tag_value_from_record(record))
                self.assertIsNone(
                    re.search(r"\b(?:vendor_identification|attribute_name)\s*:", record),
                    "standard policy records must use assigned tags, not name-form keys",
                )
        self.assertIn(standard_attribute["source_policy_table"], standard_record)
        self.assertNotIn(name_form_reference["vendor_identification"], rendered)
        self.assertNotIn(
            f"{json.dumps(name_form_reference['vendor_identification'])}, "
            f"{json.dumps(name_form_reference['attribute_name'])}",
            rendered,
        )

    def test_generates_the_separate_vendor_attribute_y_policy(self) -> None:
        rendered = self._render()
        vendor_attribute = next(
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("name") == "Vendor Attribute"
        )
        policy = vendor_attribute["source_value_policies"][0]

        vendor_record = _vendor_attribute_policy_record(self, rendered)

        _assert_source_value(self, vendor_record, "source_text", policy["source_text"])
        _assert_source_value(
            self,
            vendor_record,
            "member_element_id",
            policy["value_predicate"]["member_element_id"],
        )
        _assert_source_value(self, vendor_record, "equals", policy["value_predicate"]["equals"])
        self.assertIn("server_created", vendor_record)
        for operation in policy["prohibited_client_operations"]:
            with self.subTest(operation=operation):
                _assert_source_literal(self, vendor_record, operation)

    def test_vendor_attribute_prohibition_matches_y_and_not_other_identifiers(self) -> None:
        rendered = self._render()
        vendor_policy = _vendor_attribute_policy_record(self, rendered)

        _assert_source_value(self, vendor_policy, "equals", "y")
        self.assertNotIn('equals: "x"', vendor_policy)
        self.assertNotIn('prohibited_client_operations: "x"', vendor_policy)


class AttributePolicyGeneratorCliTests(unittest.TestCase):
    def _repo_root(self, directory: str) -> Path:
        root = Path(directory)
        _copy_validated_catalog_repo(root)
        return root

    def test_write_is_deterministic_and_clean_check_is_read_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = self._repo_root(directory)
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)

            first_write = _run_generator(root, "--write")
            self.assertEqual(first_write.returncode, 0, first_write.stderr)
            self.assertTrue(output.is_file())
            first_bytes = output.read_bytes()
            self.assertTrue(first_bytes)

            second_write = _run_generator(root, "--write")
            self.assertEqual(second_write.returncode, 0, second_write.stderr)
            self.assertEqual(output.read_bytes(), first_bytes)

            old_timestamp_ns = 946_684_800_000_000_000
            os.utime(output, ns=(old_timestamp_ns, old_timestamp_ns))
            expected_mtime_ns = output.stat().st_mtime_ns
            clean_check = _run_generator(root, "--check")
            self.assertEqual(clean_check.returncode, 0, clean_check.stderr)
            self.assertEqual(output.read_bytes(), first_bytes)
            self.assertEqual(output.stat().st_mtime_ns, expected_mtime_ns)

    def test_check_reports_stale_output_without_rewriting_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = self._repo_root(directory)
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)
            stale_bytes = b"stale generated attribute policy metadata\n"
            output.write_bytes(stale_bytes)

            result = _run_generator(root, "--check")

            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertEqual(output.read_bytes(), stale_bytes)

    def test_check_reports_missing_output_without_creating_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = self._repo_root(directory)
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)

            result = _run_generator(root, "--check")

            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("failed", result.stderr.lower())
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
