"""Red contracts for catalog-generated typed Rust attribute definitions."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from types import SimpleNamespace
from typing import Any
from unittest.mock import patch

from tools.normative_catalog import generate_attribute_types


ROOT = Path(__file__).resolve().parents[3]


class TypedAttributeGeneratorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

    def test_catalog_defines_all_and_only_direct_typed_attributes(self) -> None:
        definitions = generate_attribute_types.collect_typed_attributes(self.catalog)

        self.assertEqual(len(definitions), 62)
        by_id = {definition.element_id: definition for definition in definitions}
        self.assertNotIn("KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES", by_id)
        self.assertEqual(
            by_id["KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE"].tag,
            0x0042_0001,
        )
        self.assertEqual(
            by_id["KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE"].value_kind,
            "DateTime",
        )
        self.assertEqual(
            by_id[
                "KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION"
            ].source_multiplicity,
            "Yes",
        )
        self.assertEqual(
            by_id["KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER"].value_kind,
            "UniqueIdentifier",
        )
        self.assertEqual(
            by_id["KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE"].tag,
            0x0042_0008,
        )
        self.assertEqual(
            by_id[
                "KMIPKIT-ELEM-ATTRIBUTE-DIGITAL-SIGNATURE-ALGORITHM"
            ].source_multiplicity,
            "Yes for PGP keys. No for X.509 certificates.",
        )

    def test_generated_markers_keep_open_and_structure_value_categories(self) -> None:
        rendered = generate_attribute_types.render_typed_rust(self.catalog)

        self.assertIn("pub struct ActivationDate;", rendered)
        self.assertIn("pub struct VendorAttribute;", rendered)
        self.assertIn("UniqueIdentifierValue", rendered)
        self.assertIn("StructureView", rendered)
        self.assertIn("0x42_00_01", rendered)
        self.assertIn("pub(crate) const ATTRIBUTE_TAGS: &[u32] = &[", rendered)
        self.assertIn("typed::ActivationDate::TAG", rendered)
        self.assertNotIn("CertificateAttributes", rendered)
        self.assertEqual(rendered.count("impl super::AttributeDefinition for "), 62)
        self.assertEqual(rendered.count("::TAG,"), 62)

    def test_generated_registry_retains_source_multiplicity_and_trace_ids(self) -> None:
        rendered = generate_attribute_types.render_typed_rust(self.catalog)

        self.assertIn("KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION", rendered)
        self.assertIn('source_multiplicity: Some("Yes")', rendered)
        self.assertIn(
            'source_multiplicity: Some("Yes for PGP keys. No for X.509 certificates.")',
            rendered,
        )
        self.assertIn("KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", rendered)
        self.assertIn('source_multiplicity: Some("No")', rendered)

    def test_ffi_catalog_generation_exposes_tags_and_source_ttlv_types(self) -> None:
        rendered = generate_attribute_types.render_typed_ffi_rust(self.catalog)

        self.assertIn("ATTRIBUTE_DEFINITIONS: &[(u32, &str)]", rendered)
        self.assertIn('(0x42_00_01, "DateTime")', rendered)
        self.assertIn('(0x42_00_08, "Structure")', rendered)
        self.assertEqual(62, rendered.count('(0x'))

    def test_foreign_attribute_enums_are_generated_from_the_same_normative_catalog(self) -> None:
        java = generate_attribute_types.render_typed_java(self.catalog)
        python = generate_attribute_types.render_typed_python(self.catalog)

        self.assertEqual(62, java.count("(0x"))
        self.assertEqual(62, python.count(" = (0x"))
        self.assertIn("ACTIVATION_DATE(0x420001, \"DateTime\", \"No\")", java)
        self.assertIn(
            "APPLICATION_SPECIFIC_INFORMATION(0x420004, \"Structure\", \"Yes\")",
            java,
        )
        self.assertIn(
            "DIGITAL_SIGNATURE_ALGORITHM(0x4200AE, \"Enumeration\", "
            "\"Yes for PGP keys. No for X.509 certificates.\")",
            java,
        )
        self.assertIn("ACTIVATION_DATE = (0x420001, \"DateTime\", \"No\")", python)
        self.assertIn("def source_multiplicity(self) -> str | None:", python)

    def test_marker_and_multiplicity_validation_reject_invalid_catalog_values(self) -> None:
        with self.assertRaisesRegex(ValueError, "invalid element identity"):
            generate_attribute_types._marker_identifier("not-an-attribute")
        with self.assertRaisesRegex(ValueError, "invalid element identity"):
            generate_attribute_types._marker_identifier("KMIPKIT-ELEM-ATTRIBUTE-")
        self.assertEqual(
            "Attribute123Abc",
            generate_attribute_types._marker_identifier("KMIPKIT-ELEM-ATTRIBUTE-123-ABC"),
        )

        self.assertIsNone(generate_attribute_types._source_multiplicity({}))
        self.assertEqual(
            "Yes",
            generate_attribute_types._source_multiplicity(
                {"source_multiple_instances_permitted": "Yes"}
            ),
        )
        for value in ("", 1, None):
            if value is None:
                continue
            with self.subTest(value=value), self.assertRaisesRegex(
                ValueError, "invalid multiplicity metadata"
            ):
                generate_attribute_types._source_multiplicity(
                    {"source_multiple_instances_permitted": value}
                )

    def test_typed_attribute_collection_fails_closed_on_invalid_catalog_records(self) -> None:
        with self.assertRaisesRegex(ValueError, "no element list"):
            generate_attribute_types.collect_typed_attributes({})

        valid_encodings = generate_attribute_types._attribute_encodings(self.catalog)

        def assert_invalid(
            mutate: Any,
            expected: str,
            extra_patch: Any = None,
            encodings: list[tuple[int, list[str]]] | None = None,
        ) -> None:
            catalog = copy.deepcopy(self.catalog)
            mutate(catalog)
            if encodings is None:
                encodings = valid_encodings
            patches = [
                patch.object(
                    generate_attribute_types,
                    "_attribute_encodings",
                    return_value=encodings,
                )
            ]
            if extra_patch is not None:
                patches.append(extra_patch)
            with self.subTest(expected=expected):
                with patches[0]:
                    if len(patches) == 2:
                        with patches[1]:
                            with self.assertRaisesRegex(ValueError, expected):
                                generate_attribute_types.collect_typed_attributes(catalog)
                    else:
                        with self.assertRaisesRegex(ValueError, expected):
                            generate_attribute_types.collect_typed_attributes(catalog)

        def tag_record(catalog: dict[str, Any]) -> dict[str, Any]:
            return next(row for row in catalog["elements"] if row.get("kind") == "tag")

        def attribute_record(catalog: dict[str, Any], element_id: str) -> dict[str, Any]:
            return next(
                row for row in catalog["elements"]
                if row.get("element_id") == element_id
            )

        activation = "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE"
        certificate = "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES"
        vendor = "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE"
        assert_invalid(
            lambda catalog: tag_record(catalog).update({"element_id": None}),
            "invalid identity or wire value",
        )
        assert_invalid(
            lambda catalog: tag_record(catalog).update({"allocation": "unknown"}),
            "unsupported allocation",
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, activation).update({"element_id": None}),
            "invalid element identity",
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, certificate).update(
                {"source_encoding": "Structure"}
            ),
            "unexpectedly has direct Item metadata",
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, activation).update({"source_name": " "}),
            "invalid source name",
        )
        assert_invalid(
            lambda catalog: catalog["elements"].append(
                copy.deepcopy(attribute_record(catalog, activation))
            ),
            "duplicate typed marker names",
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, activation).update({"source_encoding": None}),
            "invalid source encoding",
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, activation).update(
                {"source_encoding": "Text String or Integer"}
            ),
            "unsupported typed value union",
            patch.object(
                generate_attribute_types,
                "_item_types_for_encoding",
                return_value=["TextString", "Integer"],
            ),
        )
        assert_invalid(
            lambda catalog: attribute_record(catalog, vendor).update(
                {"parent_element_ids": ["unexpected"]}
            ),
            "separate Table 150 structure",
        )

        def remove_vendor_tag(catalog: dict[str, Any]) -> None:
            catalog["elements"] = [
                row for row in catalog["elements"]
                if not (
                    row.get("kind") == "tag"
                    and int(
                        str(row.get("wire_value", "0")).removeprefix("0x").removeprefix("0X"),
                        16,
                    ) == 0x0042_0008
                )
            ]

        assert_invalid(remove_vendor_tag, "tag is not catalogued as assigned")
        assert_invalid(
            lambda catalog: attribute_record(catalog, activation).update({"parent_element_ids": []}),
            "exactly one tag parent",
        )

        def reserve_activation_tag(catalog: dict[str, Any]) -> None:
            parent_id = attribute_record(catalog, activation)["parent_element_ids"][0]
            next(row for row in catalog["elements"] if row.get("element_id") == parent_id).update(
                {"allocation": "reserved"}
            )

        assert_invalid(reserve_activation_tag, "parent tag is not assigned")
        assert_invalid(
            lambda catalog: None,
            "no validated TTLV encoding",
            encodings=[],
        )
        assert_invalid(
            lambda catalog: [
                row.update({"scope_state": "server_1_1"})
                for row in catalog["elements"]
                if row.get("kind") == "attribute"
            ],
            "no direct typed attributes",
        )

        def duplicate_activation_tag(catalog: dict[str, Any]) -> None:
            duplicate = copy.deepcopy(attribute_record(catalog, activation))
            duplicate.update({
                "element_id": "KMIPKIT-ELEM-ATTRIBUTE-TEST-ALIAS",
                "source_name": "Test Alias",
            })
            catalog["elements"].append(duplicate)

        assert_invalid(duplicate_activation_tag, "duplicate typed tags")

    def test_typed_renderer_reports_rustfmt_failure(self) -> None:
        with patch.object(
            generate_attribute_types.subprocess,
            "run",
            return_value=SimpleNamespace(returncode=1),
        ):
            with self.assertRaisesRegex(ValueError, "could not be formatted"):
                generate_attribute_types.render_typed_rust(self.catalog)


if __name__ == "__main__":
    unittest.main()
