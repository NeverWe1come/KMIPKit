"""Red contracts for catalog-generated typed Rust attribute definitions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

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
        self.assertNotIn("CertificateAttributes", rendered)
        self.assertEqual(rendered.count("impl super::AttributeDefinition for "), 62)

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


if __name__ == "__main__":
    unittest.main()
