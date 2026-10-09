"""Tests for catalog-derived Rust attribute TTLV type metadata."""

from __future__ import annotations

import runpy
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch

import tools.normative_catalog.generate_attribute_types as generator
from tools.normative_catalog.generate_attribute_types import render_rust
from tools.normative_catalog.safe_io import PathSecurityError
from tools.normative_catalog.validate import CatalogValidationError


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]


def small_catalog() -> dict[str, object]:
    return {
        "elements": [
            {
                "element_id": "tag-420001",
                "kind": "tag",
                "wire_value": "420001",
                "allocation": "assigned",
            },
            {
                "element_id": "tag-420028",
                "kind": "tag",
                "wire_value": "420028",
                "allocation": "assigned",
            },
            {
                "element_id": "tag-420094",
                "kind": "tag",
                "wire_value": "420094",
                "allocation": "assigned",
            },
            {
                "element_id": "tag-4200DE",
                "kind": "tag",
                "wire_value": "4200DE",
                "allocation": "assigned",
            },
            {
                "element_id": "attribute-activation-date",
                "kind": "attribute",
                "name": "Activation Date",
                "parent_element_ids": ["tag-420001"],
                "source_encoding": "Date-Time",
            },
            {
                "element_id": "attribute-algorithm",
                "kind": "attribute",
                "name": "Cryptographic Algorithm",
                "parent_element_ids": ["tag-420028"],
                "source_encoding": "Enumeration",
            },
            {
                "element_id": "attribute-id",
                "kind": "attribute",
                "name": "Unique Identifier",
                "parent_element_ids": ["tag-420094"],
                "source_encoding": "Text String, Enumeration or Integer",
            },
            {
                "element_id": "attribute-rng",
                "kind": "attribute",
                "name": "Random Number Generator",
                "parent_element_ids": ["tag-4200DE"],
                "source_encoding": "RNG Parameters",
            },
            {
                "element_id": "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE",
                "kind": "attribute",
                "name": "Vendor Attribute",
                "parent_element_ids": [],
                "source_encoding": "Structure",
            },
        ]
    }


class AttributeTypeGeneratorTests(unittest.TestCase):
    def assert_catalog_rejected(self, catalog: dict[str, object]) -> None:
        with self.assertRaises(ValueError):
            render_rust(catalog)

    def test_renders_sorted_catalog_types_and_all_permitted_wire_forms(self) -> None:
        rendered = render_rust(small_catalog())

        self.assertIn("(0x42_00_28, &[ItemType::Enumeration])", rendered)
        self.assertIn("(0x42_00_01, &[ItemType::DateTime])", rendered)
        self.assertIn("(0x42_00_DE, &[ItemType::Structure])", rendered)
        self.assertIn("0x42_00_94,", rendered)
        self.assertIn("ItemType::TextString,", rendered)
        self.assertIn("ItemType::Enumeration,", rendered)
        self.assertIn("ItemType::Integer,", rendered)
        self.assertNotIn("Vendor Attribute", rendered)
        self.assertLess(rendered.index("0x42_00_01"), rendered.index("0x42_00_28"))
        self.assertLess(rendered.index("0x42_00_28"), rendered.index("0x42_00_94"))

    def test_rejects_unrecognized_source_encoding(self) -> None:
        catalog = small_catalog()
        elements = catalog["elements"]
        assert isinstance(elements, list)
        elements[5]["source_encoding"] = "Unknown Wire Encoding"

        with self.assertRaisesRegex(ValueError, "encoding"):
            render_rust(catalog)

    def test_rejects_attribute_without_one_catalog_tag_parent(self) -> None:
        catalog = small_catalog()
        elements = catalog["elements"]
        assert isinstance(elements, list)
        elements[5]["parent_element_ids"] = []

        with self.assertRaisesRegex(ValueError, "parent"):
            render_rust(catalog)

    def test_rejects_invalid_catalog_shapes_and_tag_allocations(self) -> None:
        invalid_catalogs = []
        invalid_catalogs.append({})

        invalid_tag = small_catalog()
        invalid_tag["elements"][0]["element_id"] = 3
        invalid_catalogs.append(invalid_tag)

        duplicate_identity = small_catalog()
        duplicate_identity["elements"].append(dict(duplicate_identity["elements"][0]))
        invalid_catalogs.append(duplicate_identity)

        duplicate_tag_value = small_catalog()
        duplicate_tag_value["elements"][1]["wire_value"] = "420001"
        invalid_catalogs.append(duplicate_tag_value)

        unsupported_allocation = small_catalog()
        unsupported_allocation["elements"][0]["allocation"] = "vendor"
        invalid_catalogs.append(unsupported_allocation)

        unassigned_parent = small_catalog()
        unassigned_parent["elements"][0]["allocation"] = "reserved"
        invalid_catalogs.append(unassigned_parent)

        duplicate_attribute = small_catalog()
        duplicate_attribute["elements"].append(dict(duplicate_attribute["elements"][5]))
        invalid_catalogs.append(duplicate_attribute)

        no_attributes = {"elements": [small_catalog()["elements"][0]]}
        invalid_catalogs.append(no_attributes)

        for catalog in invalid_catalogs:
            with self.subTest(catalog=catalog):
                self.assert_catalog_rejected(catalog)

    def test_rejects_malformed_attribute_encodings_and_vendor_shape(self) -> None:
        for source_encoding in ("", "  ", 12):
            catalog = small_catalog()
            catalog["elements"][8]["source_encoding"] = source_encoding
            with self.subTest(source_encoding=source_encoding):
                with self.assertRaisesRegex(ValueError, "source encoding"):
                    render_rust(catalog)

        catalog = small_catalog()
        catalog["elements"][8]["source_encoding"] = "Text String"
        with self.assertRaisesRegex(ValueError, "Vendor Attribute"):
            render_rust(catalog)

        catalog = small_catalog()
        catalog["elements"][8]["parent_element_ids"] = ["tag-420001"]
        with self.assertRaisesRegex(ValueError, "Vendor Attribute"):
            render_rust(catalog)

        for parents in (None, [], [3], ["missing-tag"], ["tag-420001", "tag-420028"]):
            catalog = small_catalog()
            catalog["elements"][5]["parent_element_ids"] = parents
            with self.subTest(parents=parents):
                with self.assertRaisesRegex(ValueError, "parent"):
                    render_rust(catalog)

    def test_covers_all_supported_encodings_and_empty_type_lists(self) -> None:
        encodings = (
            "Big Integer",
            "Boolean",
            "Byte String",
            "Date Time",
            "Date Time Extended",
            "Enumeration",
            "Integer",
            "Interval",
            "Structure",
            "Text String",
            "Text String or Text String",
        )
        for encoding in encodings:
            catalog = small_catalog()
            catalog["elements"][5]["source_encoding"] = encoding
            with self.subTest(encoding=encoding):
                self.assertIn("ATTRIBUTE_TYPES", render_rust(catalog))

        with self.assertRaisesRegex(ValueError, "encoding"):
            generator._item_types_for_encoding("")

    def test_main_write_check_stale_and_redacted_error_paths(self) -> None:
        catalog_bytes = b"catalog"
        expected = render_rust(small_catalog()).encode("utf-8")

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(generator, "safe_read_bytes", return_value=catalog_bytes),
                patch.object(generator, "load_validated_catalog", return_value=small_catalog()),
                patch.object(generator, "atomic_write_bytes") as write,
                redirect_stdout(StringIO()),
            ):
                self.assertEqual(generator.main(["--repo-root", str(root), "--write"]), 0)
            write.assert_called_once_with(
                root.resolve(strict=True), generator.OUTPUT_PATH, expected
            )

            with (
                patch.object(generator, "safe_read_bytes", side_effect=[catalog_bytes, expected]),
                patch.object(generator, "load_validated_catalog", return_value=small_catalog()),
                redirect_stdout(StringIO()),
            ):
                self.assertEqual(generator.main(["--repo-root", str(root), "--check"]), 0)

            with (
                patch.object(generator, "safe_read_bytes", side_effect=[catalog_bytes, b"stale"]),
                patch.object(generator, "load_validated_catalog", return_value=small_catalog()),
                redirect_stderr(StringIO()),
            ):
                self.assertEqual(generator.main(["--repo-root", str(root), "--check"]), 1)

            failures = (
                (CatalogValidationError("private details"), "catalog validation failed"),
                (PathSecurityError("private path"), "repository path validation failed"),
                (OSError("private path"), "repository I/O failed"),
                (ValueError("unsupported input"), "unsupported input"),
            )
            for error, message in failures:
                with self.subTest(error=type(error).__name__):
                    stderr = StringIO()
                    with (
                        patch.object(generator, "safe_read_bytes", side_effect=error),
                        redirect_stderr(stderr),
                    ):
                        self.assertEqual(generator.main(["--repo-root", str(root), "--check"]), 2)
                    self.assertIn(message, stderr.getvalue())
                    self.assertNotIn("private", stderr.getvalue())

    def test_direct_cli_check_exercises_script_import_path(self) -> None:
        script = REPOSITORY_ROOT / "tools" / "normative_catalog" / "generate_attribute_types.py"
        arguments = [str(script), "--repo-root", str(REPOSITORY_ROOT), "--check"]
        with (
            patch.object(sys, "argv", arguments),
            patch.object(sys, "path", [str(script.parent), *sys.path]),
            redirect_stdout(StringIO()),
            self.assertRaises(SystemExit) as result,
        ):
            runpy.run_path(str(script), run_name="__main__")
        self.assertEqual(result.exception.code, 0)


if __name__ == "__main__":
    unittest.main()
