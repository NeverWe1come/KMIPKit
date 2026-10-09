"""Tests for the deterministic source-backed attribute-policy lookup."""

from __future__ import annotations

import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog.validate import load_validated_catalog


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


def _normalized_rust(rendered: str) -> str:
    return rendered.lower().replace("_", "")


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
        normalized = _normalized_rust(rendered)
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
                self.assertIn(tag, normalized)

    def test_preserves_conditional_rule_text_and_qualified_source_values(self) -> None:
        rendered = self._render()
        conditional_attributes = [
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("source_policy_table")
        ]
        conditional_rules = [
            rule
            for attribute in conditional_attributes
            for rule in attribute["source_conditional_rules"]
        ]
        qualified_values = [
            attribute[field]
            for attribute in conditional_attributes
            for field in ("source_modifiable_by_client", "source_deletable_by_client")
            if attribute[field] not in {"Yes", "No"}
        ]

        self.assertTrue(conditional_rules)
        self.assertTrue(qualified_values)
        for rule in conditional_rules:
            with self.subTest(source_text=rule["source_text"]):
                self.assertIn(rule["source_text"], rendered)
                for source_ref in rule["source_refs"]:
                    self.assertIn(source_ref["section"], rendered)
        for source_value in qualified_values:
            with self.subTest(source_value=source_value):
                self.assertIn(source_value, rendered)

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

        self.assertNotIn("54abcd", _normalized_rust(rendered))
        self.assertNotIn("Unmapped Vendor Tag", rendered)

    def test_does_not_infer_policy_for_an_unmapped_name_form_reference(self) -> None:
        catalog = {
            "elements": [
                *self.catalog["elements"],
                {
                    "element_id": "KMIPKIT-ELEM-ATTRIBUTE-UNMAPPED-CUSTOM-NAME",
                    "kind": "attribute",
                    "name": "Unmapped Customer Attribute",
                    "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "4.60"}],
                    "parent_element_ids": [],
                    "source_name": "Unmapped Customer Attribute",
                },
            ]
        }

        rendered = self._render(catalog)

        self.assertNotIn("Unmapped Customer Attribute", rendered)

    def test_generates_the_separate_vendor_attribute_y_policy(self) -> None:
        rendered = self._render()
        vendor_attribute = next(
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "attribute" and item.get("name") == "Vendor Attribute"
        )
        policy = vendor_attribute["source_value_policies"][0]

        self.assertIn(policy["source_text"], rendered)
        self.assertIn(policy["value_predicate"]["member_element_id"], rendered)
        self.assertIn('"y"', rendered)
        self.assertIn("server_created", rendered)
        for operation in policy["prohibited_client_operations"]:
            with self.subTest(operation=operation):
                self.assertIn(operation, rendered)

    def test_vendor_attribute_prohibition_matches_y_and_not_other_identifiers(self) -> None:
        rendered = self._render()
        vendor_policy_start = rendered.find("VENDOR_ATTRIBUTE")
        self.assertNotEqual(vendor_policy_start, -1, "separate Vendor Attribute policy is missing")
        vendor_policy = rendered[vendor_policy_start:]

        self.assertIn('equals: "y"', vendor_policy)
        self.assertNotIn('equals: "x"', vendor_policy)


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
