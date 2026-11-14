"""Wave 3 Red tests for KMIP 3.0 object attributes in generate_attribute_types.py."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog import generate_attribute_types

ROOT = Path(__file__).resolve().parents[3]


class Wave3AttributeTypesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        cls.versioned_tags = json.loads(
            (ROOT / "specification/catalog/kmip-versioned-tags.json").read_text(encoding="utf-8")
        )

    def test_attribute_encodings_include_kmip30_object_attributes(self) -> None:
        rendered = generate_attribute_types.render_rust(
            self.catalog,
            versioned_tags=self.versioned_tags,
        )
        for tag_hex in (
            "0x42_01_90",  # Certificate Link
            "0x42_01_9E",  # Object Class
            "0x42_01_AE",  # OTP Counter
            "0x42_01_B6",  # Split Key Polynomial
            "0x42_01_B8",  # Deactivation Reason
            "0x42_01_BD",  # Certify Counter
            "0x42_01_C2",  # NIST Security Category
        ):
            self.assertIn(tag_hex, rendered)

    def test_typed_attributes_include_kmip30_links_counters_and_promoted(self) -> None:
        definitions = generate_attribute_types.collect_typed_attributes(
            self.catalog,
            versioned_tags=self.versioned_tags,
        )
        by_tag = {d.tag: d for d in definitions}
        expected_tags = {
            0x0042_0190: ("CertificateLink", "UniqueIdentifier"),
            0x0042_0199: ("PrivateKeyLink", "UniqueIdentifier"),
            0x0042_019E: ("ObjectClass", "Enumeration"),
            0x0042_01AE: ("OtpCounter", "BigInteger"),
            0x0042_01B6: ("SplitKeyPolynomial", "ByteString"),
            0x0042_01B8: ("DeactivationReason", "Structure"),
            0x0042_01BA: ("CertificateSubjectDn", "TextString"),
            0x0042_01BB: ("CertificateIssuerDn", "TextString"),
            0x0042_01BC: ("CertificateRequestLink", "UniqueIdentifier"),
            0x0042_01BD: ("CertifyCounter", "BigInteger"),
            0x0042_01C2: ("NistSecurityCategory", "Integer"),
            0x0042_0025: ("CredentialType", "Enumeration"),
            0x0042_0045: ("KeyPartIdentifier", "Integer"),
            0x0042_008A: ("SplitKeyMethod", "Enumeration"),
            0x0042_008B: ("SplitKeyParts", "Integer"),
            0x0042_008C: ("SplitKeyThreshold", "Integer"),
        }
        for tag, (marker, kind) in expected_tags.items():
            self.assertIn(tag, by_tag, f"missing tag 0x{tag:06X}")
            self.assertEqual(by_tag[tag].marker_name, marker)
            self.assertEqual(by_tag[tag].value_kind, kind)


if __name__ == "__main__":
    unittest.main()
