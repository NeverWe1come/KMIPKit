"""Tests for deterministic catalog-derived TTLV tag allocation metadata."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog.validate import load_validated_catalog


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools" / "normative_catalog" / "generate_ttlv_tags.py"
OUTPUT_PATH = "crates/kmipkit-ttlv/src/generated/tag_allocations.rs"
GENERATOR_MODULE = "tools.normative_catalog.generate_ttlv_tags"


def small_catalog() -> dict[str, Any]:
    """Return a hand-checked catalog slice with exact and aggregate allocations."""
    return {
        "elements": [
            {
                "element_id": "KMIPKIT-ELEM-TAG-420001",
                "kind": "tag",
                "name": "Activation Date",
                "wire_value": "420001",
                "allocation": "assigned",
            },
            {
                "element_id": "KMIPKIT-ELEM-TAG-420009",
                "kind": "tag",
                "name": "(Reserved)",
                "wire_value": "420009",
                "allocation": "reserved",
            },
            {
                "element_id": "KMIPKIT-ELEM-TAG-420174",
                "kind": "tag",
                "name": "Submission Date",
                "wire_value": "0x420174",
                "allocation": "assigned",
            },
            {
                "element_id": "KMIPKIT-ELEM-TAG-420175",
                "kind": "tag",
                "name": "Off-Line Extension",
                "wire_value": "0x420175",
                "allocation": "assigned",
            },
            {
                "element_id": "KMIPKIT-ELEM-TAG-420176",
                "kind": "tag",
                "name": "Asynchronous Correlation Values",
                "wire_value": "0x420176",
                "allocation": "assigned",
            },
        ],
        "tag_ranges": [
            {
                "range_id": "KMIPKIT-RANGE-002",
                "value_range": "420XXX – 42FFFF",
                "allocation": "reserved",
            },
            {
                "range_id": "KMIPKIT-RANGE-004",
                "value_range": "540000 - 54FFFF",
                "allocation": "extension",
            },
        ],
    }


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


def _load_renderer(testcase: unittest.TestCase):
    module_spec = importlib.util.find_spec(GENERATOR_MODULE)
    testcase.assertIsNotNone(
        module_spec,
        "catalog tag renderer must be implemented before its behavior can be exercised",
    )
    from tools.normative_catalog.generate_ttlv_tags import render_rust

    return render_rust


class TagAllocationRenderingTests(unittest.TestCase):
    def _render(self, catalog: dict[str, Any]) -> str:
        return _load_renderer(self)(catalog)

    def _assert_allocation_near(self, rendered: str, value: str, allocation: str) -> None:
        normalized = rendered.lower().replace("_", "")
        index = normalized.find(value.lower().replace("_", "").removeprefix("0x"))
        self.assertNotEqual(index, -1, f"missing numeric allocation value {value}")
        context = normalized[max(0, index - 100) : index + 120]
        self.assertIn(allocation.lower(), context)

    def test_renders_assigned_and_individually_reserved_exact_entries(self) -> None:
        rendered = self._render(small_catalog())

        self._assert_allocation_near(rendered, "420001", "assigned")
        self._assert_allocation_near(rendered, "420009", "reserved")

    def test_renders_every_exact_catalog_tag_and_range(self) -> None:
        catalog = load_validated_catalog(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_bytes(),
            ROOT,
        )
        rendered = self._render(catalog)

        tags = [row for row in catalog["elements"] if row.get("kind") == "tag"]
        self.assertEqual(len(tags), 374)
        for tag in tags:
            value = tag["wire_value"]
            normalized = value[2:] if value.lower().startswith("0x") else value
            self._assert_allocation_near(rendered, normalized, tag["allocation"])

        expected_ranges = (
            ("000000", "420000", "unused"),
            ("420000", "42FFFF", "reserved"),
            ("430000", "53FFFF", "unused"),
            ("540000", "54FFFF", "extension"),
            ("550000", "FFFFFF", "unused"),
        )
        for lower, upper, allocation in expected_ranges:
            self._assert_allocation_near(rendered, lower, allocation)
            self._assert_allocation_near(rendered, upper, allocation)

    def test_keeps_exact_assignments_visible_alongside_overlapping_reserved_range(self) -> None:
        rendered = self._render(small_catalog())

        for value in ("420174", "420175", "420176"):
            self._assert_allocation_near(rendered, value, "assigned")
        self._assert_allocation_near(rendered, "420000", "reserved")
        self._assert_allocation_near(rendered, "42FFFF", "reserved")

    def test_renders_extension_range_as_numeric_endpoints_and_classification(self) -> None:
        rendered = self._render(small_catalog())

        self._assert_allocation_near(rendered, "540000", "extension")
        self._assert_allocation_near(rendered, "54FFFF", "extension")

    def test_rejects_malformed_exact_tag_value(self) -> None:
        catalog = small_catalog()
        catalog["elements"][0]["wire_value"] = "42Z001"

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_rejects_duplicate_numeric_exact_tag_values(self) -> None:
        catalog = small_catalog()
        duplicate = dict(catalog["elements"][0])
        duplicate.update(element_id="KMIPKIT-ELEM-TAG-4200AA", wire_value="0x420001")
        catalog["elements"].append(duplicate)

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_rejects_exact_tag_values_outside_the_24_bit_range(self) -> None:
        catalog = small_catalog()
        catalog["elements"][0]["wire_value"] = "1000000"

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_rejects_malformed_aggregate_range(self) -> None:
        catalog = small_catalog()
        catalog["tag_ranges"][0]["value_range"] = "420000 through 42FFFF"

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_rejects_duplicate_aggregate_ranges(self) -> None:
        catalog = small_catalog()
        catalog["tag_ranges"].append(dict(catalog["tag_ranges"][0]))

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_rejects_aggregate_range_endpoints_outside_the_24_bit_range(self) -> None:
        catalog = small_catalog()
        catalog["tag_ranges"][0]["value_range"] = "420000 - 1000000"

        with self.assertRaises(ValueError):
            self._render(catalog)

    def test_numeric_order_and_rendering_are_stable_when_catalog_rows_are_reordered(self) -> None:
        catalog = small_catalog()
        reordered = {
            "elements": list(reversed(catalog["elements"])),
            "tag_ranges": list(reversed(catalog["tag_ranges"])),
        }

        rendered = self._render(catalog)
        self.assertEqual(self._render(reordered), rendered)
        normalized = rendered.lower().replace("_", "")
        positions = [
            normalized.index(value)
            for value in ("420001", "420009", "420174", "420175", "420176")
        ]
        self.assertEqual(positions, sorted(positions))


class TagAllocationGeneratorCliTests(unittest.TestCase):
    def _repo_root(self, directory: str) -> Path:
        root = Path(directory)
        _copy_validated_catalog_repo(root)
        return root

    def test_write_is_idempotent_and_clean_check_is_read_only(self) -> None:
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

            clean_check = _run_generator(root, "--check")
            self.assertEqual(clean_check.returncode, 0, clean_check.stderr)
            self.assertEqual(output.read_bytes(), first_bytes)

    def test_check_reports_stale_output_without_rewriting_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = self._repo_root(directory)
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)
            stale_bytes = b"stale generated tag allocation metadata\n"
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
