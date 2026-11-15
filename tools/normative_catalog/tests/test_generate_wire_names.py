"""Tests for OASIS Profiles v2.1 §§5.4.1.1–5.4.1.3, 5.4.1.6.7,
5.5.1.1–5.5.1.3, and 5.5.1.6.7 catalog-derived names."""

from __future__ import annotations

import os
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from unittest.mock import patch
from pathlib import Path

from tools.normative_catalog.generate_wire_names import (
    _assigned_bitmask_names,
    _assigned_enumeration_names,
    _assigned_tag_names,
    _tag_bitmask_families,
    _tag_enumeration_families,
    main,
    render_tsv,
    render_rust,
)
from tools.normative_catalog.validate import CatalogValidationError


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools" / "normative_catalog" / "generate_wire_names.py"
OUTPUT_PATH = "crates/kmipkit-client/src/wire_names_generated.rs"


class WireNameRenderingTests(unittest.TestCase):
    def test_generated_metadata_does_not_use_unapproved_macros(self) -> None:
        catalog = {"elements": [{"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Activation Date"}]}
        self.assertNotIn("include_str!", render_rust(catalog))

    def test_renders_only_assigned_tags_in_numeric_order(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420002", "name": "Application Data"},
                {"kind": "tag", "allocation": "reserved", "wire_value": "420001", "name": "Reserved"},
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Activation Date"},
            ]
        }

        rendered = render_tsv(catalog)

        self.assertEqual(
            rendered.splitlines(),
            ["T\t420001\tActivation Date", "T\t420002\tApplication Data"],
        )
        self.assertNotIn("Reserved", rendered)

    def test_renders_assigned_enumeration_names_with_their_family(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420057", "name": "Object Type"},
                {"kind": "enumeration", "element_id": "ENUM-OBJECT", "name": "Object Type"},
                {"kind": "structure_member", "name": "Object Type", "source_encoding": "Enumeration"},
                {"kind": "message_field", "name": "Object Type", "source_encoding": "Enumeration"},
                {
                    "kind": "enumeration_value",
                    "allocation": "assigned",
                    "parent_element_ids": ["ENUM-OBJECT"],
                    "wire_value": "00000002",
                    "name": "Certificate",
                },
                {
                    "kind": "enumeration_value",
                    "allocation": "extension",
                    "parent_element_ids": ["ENUM-OBJECT"],
                    "wire_value": "8XXXXXXX",
                    "name": "Extensions",
                },
            ]
        }

        rendered = render_tsv(catalog)

        self.assertEqual(
            rendered.splitlines(),
            [
                "T\t420057\tObject Type",
                "F\t420057\tObject Type",
                "E\tObject Type\t00000002\tCertificate",
            ],
        )
        self.assertNotIn("Extensions", rendered)

    def test_maps_enum_typed_tag_to_same_named_enumeration_family(self) -> None:
        catalog = {
            "elements": [
                {
                    "kind": "tag",
                    "allocation": "assigned",
                    "wire_value": "420074",
                    "name": "Query Function",
                    "parent_element_ids": ["ENUMERATION-TAG"],
                },
                {
                    "kind": "enumeration",
                    "element_id": "ENUMERATION-TAG",
                    "name": "Tag",
                },
                {
                    "kind": "enumeration",
                    "element_id": "ENUM-QUERY-FUNCTION",
                    "name": "Query Function",
                },
                {
                    "kind": "enumeration_value",
                    "allocation": "assigned",
                    "parent_element_ids": ["ENUM-QUERY-FUNCTION"],
                    "wire_value": "00000001",
                    "name": "Query Operations",
                },
            ]
        }

        self.assertEqual(_tag_enumeration_families(catalog), [(0x420074, "Query Function")])

    def test_same_named_tag_without_enumeration_type_parent_is_not_inferred(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420074", "name": "Query Function"},
                {"kind": "enumeration", "element_id": "ENUMERATION-TAG", "name": "Tag"},
                {"kind": "enumeration", "element_id": "ENUM-QUERY-FUNCTION", "name": "Query Function"},
            ]
        }

        self.assertEqual(_tag_enumeration_families(catalog), [])

    def test_renders_mask_families_and_assigned_flags_in_stable_order(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "42008E", "name": "Storage Status Mask"},
                {"kind": "tag", "allocation": "assigned", "wire_value": "42002C", "name": "Cryptographic Usage Mask"},
                {"kind": "bitmask", "element_id": "MASK-STORAGE", "name": "Storage Status Mask"},
                {"kind": "bitmask", "element_id": "MASK-CRYPTO", "name": "Cryptographic Usage Mask"},
                {
                    "kind": "bitmask_value",
                    "allocation": "assigned",
                    "parent_element_ids": ["MASK-CRYPTO"],
                    "wire_value": "00000008",
                    "name": "Decrypt",
                },
                {
                    "kind": "bitmask_value",
                    "allocation": "assigned",
                    "parent_element_ids": ["MASK-CRYPTO"],
                    "wire_value": "00000004",
                    "name": "Encrypt",
                },
                {
                    "kind": "bitmask_value",
                    "allocation": "extension",
                    "parent_element_ids": ["MASK-CRYPTO"],
                    "wire_value": "XXX00000",
                    "name": "Extensions",
                },
                {
                    "kind": "bitmask_value",
                    "allocation": "reserved",
                    "parent_element_ids": ["MASK-STORAGE"],
                    "wire_value": "00000004",
                    "name": "Reserved Flag",
                },
            ]
        }

        rendered = render_tsv(catalog)

        self.assertEqual(
            rendered.splitlines(),
            [
                "T\t42002C\tCryptographic Usage Mask",
                "T\t42008E\tStorage Status Mask",
                "M\t42002C\tCryptographic Usage Mask",
                "M\t42008E\tStorage Status Mask",
                "B\tCryptographic Usage Mask\t00000004\tEncrypt",
                "B\tCryptographic Usage Mask\t00000008\tDecrypt",
            ],
        )

    def test_resolves_mask_families_from_the_matching_assigned_tag(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "42002C", "name": "Cryptographic Usage Mask"},
                {"kind": "bitmask", "element_id": "MASK-CRYPTO", "name": "Cryptographic Usage Mask"},
            ]
        }

        self.assertEqual(
            _tag_bitmask_families(catalog), [(0x42002C, "Cryptographic Usage Mask")]
        )

    def test_rejects_bitmask_values_without_unique_families_or_valid_values(self) -> None:
        invalid_catalogs = (
            {
                "elements": [
                    {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
                    {
                        "kind": "bitmask_value",
                        "allocation": "assigned",
                        "parent_element_ids": ["MISSING"],
                        "wire_value": "00000001",
                        "name": "Encrypt",
                    },
                ]
            },
            {
                "elements": [
                    {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
                    {
                        "kind": "bitmask_value",
                        "allocation": "assigned",
                        "parent_element_ids": ["MASK"],
                        "wire_value": "XXXXXXXX",
                        "name": "Encrypt",
                    },
                ]
            },
        )

        for catalog in invalid_catalogs:
            with self.subTest(catalog=catalog), self.assertRaises(ValueError):
                _assigned_bitmask_names(catalog)

    def test_rejects_ambiguous_bitmask_families_components_and_tag_mappings(self) -> None:
        duplicate_families = {"elements": [
            {"kind": "bitmask", "element_id": "MASK-1", "name": "Usage Mask"},
            {"kind": "bitmask", "element_id": "MASK-2", "name": "Usage Mask"},
        ]}
        duplicate_values = {"elements": [
            {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
            {"kind": "bitmask_value", "allocation": "assigned", "parent_element_ids": ["MASK"], "wire_value": "00000001", "name": "Encrypt"},
            {"kind": "bitmask_value", "allocation": "assigned", "parent_element_ids": ["MASK"], "wire_value": "00000001", "name": "Decrypt"},
        ]}
        duplicate_names = {"elements": [
            {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
            {"kind": "bitmask_value", "allocation": "assigned", "parent_element_ids": ["MASK"], "wire_value": "00000001", "name": "Encrypt"},
            {"kind": "bitmask_value", "allocation": "assigned", "parent_element_ids": ["MASK"], "wire_value": "00000002", "name": "Encrypt"},
        ]}
        invalid_family = {"elements": [{"kind": "bitmask", "element_id": "MASK", "name": ""}]}
        invalid_component = {"elements": [
            {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
            {"kind": "bitmask_value", "allocation": "assigned", "parent_element_ids": ["MASK"], "wire_value": "00000001", "name": ""},
        ]}
        missing_tag = {"elements": [
            {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Other Tag"},
            {"kind": "bitmask", "element_id": "MASK", "name": "Usage Mask"},
        ]}

        for catalog, helper in (
            (duplicate_families, _assigned_bitmask_names),
            (duplicate_values, _assigned_bitmask_names),
            (duplicate_names, _assigned_bitmask_names),
            (invalid_family, _assigned_bitmask_names),
            (invalid_component, _assigned_bitmask_names),
            (missing_tag, _tag_bitmask_families),
            ({"elements": [{"kind": "bitmask", "element_id": "MASK", "name": None}]}, _tag_bitmask_families),
            ({"elements": [{"kind": "bitmask", "element_id": "MASK-1", "name": "Usage Mask"}, {"kind": "bitmask", "element_id": "MASK-2", "name": "Usage Mask"}]}, _tag_bitmask_families),
        ):
            with self.subTest(catalog=catalog), self.assertRaises(ValueError):
                helper(catalog)

    def test_bitmask_helpers_reject_a_missing_element_list(self) -> None:
        for helper in (_assigned_bitmask_names, _tag_bitmask_families):
            with self.subTest(helper=helper.__name__), self.assertRaises(ValueError):
                helper({})

    def test_catalog_bitmask_inventory_exposes_only_assigned_flags(self) -> None:
        import json

        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8")
        )
        bitmask_families = [
            element for element in catalog["elements"] if element.get("kind") == "bitmask"
        ]
        assigned_flags = _assigned_bitmask_names(catalog)
        rendered = render_tsv(catalog).splitlines()

        self.assertEqual(len(bitmask_families), 3)
        self.assertEqual(len(assigned_flags), 33)
        self.assertEqual(sum(line.startswith("M\t") for line in rendered), 3)
        self.assertEqual(sum(line.startswith("B\t") for line in rendered), 33)
        self.assertFalse(any("\tExtensions" in line for line in rendered))
        self.assertIn((0x420074, "Query Function"), _tag_enumeration_families(catalog))

    def test_renders_catalog_metadata_inside_a_raw_rust_literal(self) -> None:
        catalog = {"elements": [{"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Activation Date"}]}
        rendered = render_rust(catalog)

        self.assertEqual(
            rendered,
            "// @generated by tools/normative_catalog/generate_wire_names.py; do not edit.\n"
            'pub(crate) const WIRE_NAMES: &str = r"T\t420001\tActivation Date\n";\n',
        )

    def test_rust_literal_delimiter_does_not_conflict_with_catalog_names(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": 'Name "# in title'},
            ]
        }

        rendered = render_rust(catalog)

        self.assertIn('r##"T\t420001\tName "# in title\n"##', rendered)
        self.assertNotIn('title\n"#;', rendered)

    def test_rejects_catalog_names_that_exhaust_rust_raw_string_delimiters(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": 'Name "' + ("#" * 256)},
            ]
        }

        with self.assertRaises(ValueError):
            render_rust(catalog)

    def test_rejects_tsv_delimiters_in_catalog_names(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Bad\tName"},
            ]
        }

        for invalid_name in ("Bad\tName", "Bad\rName", "Bad\nName"):
            catalog["elements"][0]["name"] = invalid_name
            with self.subTest(name=invalid_name), self.assertRaises(ValueError):
                render_tsv(catalog)

    def test_rejects_invalid_tag_and_enumeration_catalog_shapes(self) -> None:
        for renderer in (_assigned_tag_names, _assigned_enumeration_names, _tag_enumeration_families):
            with self.subTest(renderer=renderer.__name__), self.assertRaises(ValueError):
                renderer({"elements": "invalid"})

        with self.assertRaises(ValueError):
            _assigned_tag_names({"elements": [{"kind": "tag", "allocation": "reserved"}]})
        with self.assertRaises(ValueError):
            _assigned_tag_names({"elements": [{"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": ""}]})
        with self.assertRaises(ValueError):
            _assigned_tag_names({"elements": [{"kind": "tag", "allocation": "assigned", "wire_value": "not-hex", "name": "Tag"}]})
        with self.assertRaises(ValueError):
            _assigned_enumeration_names({"elements": [{"kind": "enumeration", "element_id": "", "name": "Invalid"}]})
        with self.assertRaises(ValueError):
            _assigned_enumeration_names({"elements": [
                {"kind": "enumeration", "element_id": "E", "name": "Family"},
                {"kind": "enumeration", "element_id": "E", "name": "Family"},
            ]})
        with self.assertRaises(ValueError):
            _assigned_enumeration_names({"elements": [
                {"kind": "enumeration", "element_id": "E", "name": "Family"},
                {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["UNKNOWN"], "wire_value": "00000001", "name": "Value"},
            ]})
        with self.assertRaises(ValueError):
            _assigned_enumeration_names({"elements": [
                {"kind": "enumeration", "element_id": "E", "name": "Family"},
                {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["E"], "wire_value": "not-hex!", "name": "Value"},
            ]})
        with self.assertRaises(ValueError):
            _assigned_enumeration_names({"elements": [
                {"kind": "enumeration", "element_id": "E", "name": "Family"},
                {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["E"], "wire_value": "00000001", "name": ""},
            ]})

    def test_rejects_enumeration_name_or_value_collisions_and_ambiguous_tag_mapping(self) -> None:
        base = [
            {"kind": "enumeration", "element_id": "E", "name": "Object Type"},
            {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["E"], "wire_value": "00000001", "name": "Value"},
        ]
        for duplicate in (
            {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["E"], "wire_value": "00000001", "name": "Other"},
            {"kind": "enumeration_value", "allocation": "assigned", "parent_element_ids": ["E"], "wire_value": "00000002", "name": "Value"},
        ):
            with self.subTest(duplicate=duplicate), self.assertRaises(ValueError):
                _assigned_enumeration_names({"elements": [*base, duplicate]})

        tag = {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Result Status"}
        with self.assertRaises(ValueError):
            _tag_enumeration_families({"elements": [
                tag,
                {"kind": "enumeration", "name": "Reason"},
                {"kind": "structure_member", "name": "Result Status", "source_encoding": "Enumeration"},
            ]})
        with self.assertRaises(ValueError):
            _tag_enumeration_families({"elements": [tag, {"kind": "structure_member", "name": "Result Status", "source_encoding": "Enumeration"}]})
        with self.assertRaises(ValueError):
            _tag_enumeration_families({"elements": [tag, {"kind": "enumeration", "name": "Result Status"}, {"kind": "enumeration", "name": "Result Status"}]})
        with self.assertRaises(ValueError):
            _tag_enumeration_families({"elements": [tag, {"kind": "enumeration", "element_id": "E", "name": "Result Status"}, {"kind": "structure_member", "name": "Missing Tag", "source_encoding": "Enumeration"}]})
        with self.assertRaises(ValueError):
            _tag_enumeration_families({"elements": [{"kind": "enumeration", "element_id": "E", "name": ""}]})

    def test_rejects_ambiguous_names_and_values(self) -> None:
        catalog = {
            "elements": [
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "First"},
                {"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Second"},
            ]
        }

        with self.assertRaises(ValueError):
            render_tsv(catalog)


class WireNameGeneratorCliTests(unittest.TestCase):
    def test_main_reports_all_safe_outcomes_without_payloads(self) -> None:
        catalog = {"elements": [{"kind": "tag", "allocation": "assigned", "wire_value": "420001", "name": "Tag"}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)

            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", return_value=b"catalog"), patch(
                "tools.normative_catalog.generate_wire_names.load_validated_catalog", return_value=catalog
            ), patch("tools.normative_catalog.generate_wire_names.atomic_write_bytes") as write_output:
                with redirect_stdout(StringIO()):
                    self.assertEqual(main(["--write", "--repo-root", str(root)]), 0)
                write_output.assert_called_once()

            generated_source = render_rust(catalog).encode("utf-8")
            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", side_effect=[b"catalog", generated_source]), patch(
                "tools.normative_catalog.generate_wire_names.load_validated_catalog", return_value=catalog
            ), redirect_stdout(StringIO()):
                self.assertEqual(main(["--check", "--repo-root", str(root)]), 0)

            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", side_effect=[b"catalog", b"stale"]), patch(
                "tools.normative_catalog.generate_wire_names.load_validated_catalog", return_value=catalog
            ), redirect_stderr(StringIO()):
                self.assertEqual(main(["--check", "--repo-root", str(root)]), 1)

            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", side_effect=OSError("secret path")), redirect_stderr(StringIO()) as error:
                self.assertEqual(main(["--check", "--repo-root", str(root)]), 2)
                self.assertNotIn("secret path", error.getvalue())

            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", return_value=b"catalog"), patch(
                "tools.normative_catalog.generate_wire_names.load_validated_catalog", side_effect=CatalogValidationError("secret catalog")
            ), redirect_stderr(StringIO()) as error:
                self.assertEqual(main(["--check", "--repo-root", str(root)]), 2)
                self.assertNotIn("secret catalog", error.getvalue())

            with patch("tools.normative_catalog.generate_wire_names.safe_read_bytes", return_value=b"catalog"), patch(
                "tools.normative_catalog.generate_wire_names.load_validated_catalog", return_value=catalog
            ), patch("tools.normative_catalog.generate_wire_names.render_rust", side_effect=ValueError("secret value")), redirect_stderr(StringIO()) as error:
                self.assertEqual(main(["--check", "--repo-root", str(root)]), 2)
                self.assertNotIn("secret value", error.getvalue())

    def test_script_can_load_from_its_standalone_import_path(self) -> None:
        with patch("sys.path", [str(SCRIPT.parent), *sys.path]):
            namespace = runpy.run_path(str(SCRIPT), run_name="wire_names_standalone")
        self.assertIn("render_rust", namespace)

    def test_script_main_entrypoint_checks_the_generated_metadata(self) -> None:
        arguments = [str(SCRIPT), "--check", "--repo-root", str(ROOT)]
        with patch("sys.path", [str(SCRIPT.parent), *sys.path]), patch("sys.argv", arguments):
            with redirect_stdout(StringIO()), self.assertRaises(SystemExit) as exit_code:
                runpy.run_path(str(SCRIPT), run_name="__main__")

        self.assertEqual(exit_code.exception.code, 0)

    def test_write_check_stale_and_missing_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "specification").mkdir()
            shutil.copy2(ROOT / "AGENTS.md", root / "AGENTS.md")
            shutil.copytree(ROOT / "specification" / "oasis", root / "specification" / "oasis")
            shutil.copytree(ROOT / "specification" / "catalog", root / "specification" / "catalog")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(["git", "add", "specification/oasis"], cwd=root, check=True)
            subprocess.run(
                ["git", "-c", "user.name=Wire Name Tests", "-c", "user.email=wire-name-tests@example.invalid", "commit", "-qm", "test fixture sources"],
                cwd=root,
                check=True,
            )
            output = root / OUTPUT_PATH
            output.parent.mkdir(parents=True)

            def run(mode: str) -> subprocess.CompletedProcess[str]:
                return subprocess.run(
                    [sys.executable, str(SCRIPT), mode, "--repo-root", str(root)],
                    cwd=ROOT,
                    capture_output=True,
                    check=False,
                    text=True,
                )

            missing = run("--check")
            self.assertEqual(missing.returncode, 2, missing.stderr)
            self.assertFalse(output.exists())

            written = run("--write")
            self.assertEqual(written.returncode, 0, written.stderr)
            original = output.read_bytes()
            self.assertTrue(original)
            os.utime(output, ns=(946_684_800_000_000_000, 946_684_800_000_000_000))
            timestamp = output.stat().st_mtime_ns

            checked = run("--check")
            self.assertEqual(checked.returncode, 0, checked.stderr)
            self.assertEqual(output.read_bytes(), original)
            self.assertEqual(output.stat().st_mtime_ns, timestamp)
            self.assertEqual(run("--write").returncode, 0)
            self.assertEqual(output.read_bytes(), original)

            output.write_bytes(b"stale metadata\n")
            stale_check = run("--check")
            self.assertEqual(stale_check.returncode, 1, stale_check.stderr)
            self.assertEqual(output.read_bytes(), b"stale metadata\n")


if __name__ == "__main__":
    unittest.main()
