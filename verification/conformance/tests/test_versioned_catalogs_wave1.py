from __future__ import annotations

import json
from pathlib import Path
import unittest

from verification.conformance import (
    generate_attribute_policy,
    generate_named_values,
    generate_result_values,
    generate_wire_names,
)

REPO_ROOT = Path(__file__).resolve().parents[3]


class VersionedCatalogsWave1Tests(unittest.TestCase):
    def test_wire_names_include_kmip30_tags_and_enumerations(self) -> None:
        catalog = json.loads((REPO_ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_tags = json.loads(
            (REPO_ROOT / "verification/conformance/catalog/kmip-versioned-tags.json").read_text("utf-8")
        )
        versioned_values = json.loads(
            (REPO_ROOT / "verification/conformance/catalog/kmip-versioned-values.json").read_text("utf-8")
        )
        tsv = generate_wire_names.render_tsv(
            catalog, versioned_tags=versioned_tags, versioned_values=versioned_values
        )
        self.assertIn("T\t420190\tCertificate Link\n", tsv)
        self.assertIn("T\t4201BD\tCertify Counter\n", tsv)
        self.assertIn("T\t4201C3\tKEM Algorithm\n", tsv)
        self.assertIn("F\t4201C3\tKEM Algorithm\n", tsv)
        self.assertIn("E\tKEM Algorithm\t00000003\tMLKEM\n", tsv)
        self.assertIn("E\tCryptographic Algorithm\t0000003A\tML-KEM-768\n", tsv)
        self.assertIn("E\tKey Format Type\t00000018\tSeed Private Key\n", tsv)
        self.assertIn("E\tDeactivation Reason Code\t00000001\tUnspecified\n", tsv)
        self.assertIn("E\tOTP Algorithm\t00000001\tHOTP\n", tsv)
        self.assertIn("M\t42019F\tObject Class Mask\n", tsv)
        self.assertIn("B\tObject Class Mask\t00000001\tUser\n", tsv)
        self.assertIn("T\t4201B6\tSplit Key Polynomial\n", tsv)
        self.assertIn("F\t4201B6\tSplit Key Polynomial\n", tsv)
        self.assertIn("E\tSplit Key Polynomial\t00000001\tPolynomial-283\n", tsv)
        self.assertIn("E\tSplit Key Polynomial\t00000002\tPolynomial-285\n", tsv)

    def test_wire_names_fail_closed_on_unmapped_enumeration_tag(self) -> None:
        catalog = json.loads((REPO_ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text("utf-8"))
        bad_versioned_tags = {
            "tagEnumeration": {
                "assignments": [
                    {
                        "name": "Corrupt Enum Tag",
                        "wire_value": "0x4201FA",
                        "introduced_in": "3.0",
                        "source_encoding": "Enumeration",
                    }
                ]
            }
        }
        with self.assertRaises(ValueError) as ctx:
            generate_wire_names.render_tsv(catalog, versioned_tags=bad_versioned_tags)
        self.assertIn("enumeration", str(ctx.exception).lower())


    def test_attribute_policy_includes_version_bounds_and_kmip30_attributes(self) -> None:
        catalog = json.loads((REPO_ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_tags = json.loads(
            (REPO_ROOT / "verification/conformance/catalog/kmip-versioned-tags.json").read_text("utf-8")
        )
        rust = generate_attribute_policy.render_rust(catalog, versioned_tags=versioned_tags)
        self.assertIn("0x42_01_90", rust)  # Certificate Link
        self.assertIn("0x42_01_BD", rust)  # Certify Counter
        self.assertIn("0x42_01_9E", rust)  # Object Class
        self.assertIn("0x42_01_B8", rust)  # Deactivation Reason
        self.assertIn("0x42_01_C2", rust)  # NIST Security Category
        self.assertIn("0x42_01_B6", rust)  # Split Key Polynomial
        self.assertIn("min_version", rust)
        self.assertIn("max_version", rust)

    def test_result_values_include_kmip30_circular_link_error(self) -> None:
        catalog = json.loads((REPO_ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_values = json.loads(
            (REPO_ROOT / "verification/conformance/catalog/kmip-versioned-values.json").read_text("utf-8")
        )
        rust = generate_result_values.render_rust(catalog, versioned_values=versioned_values)
        self.assertIn("Circular Link Error", rust)

    def test_named_values_include_v20_v21_and_v30_pqc_and_kem_groups(self) -> None:
        versioned_values = generate_named_values.load_versioned_values(REPO_ROOT)
        versions = {g.version for g in versioned_values.groups}
        self.assertIn("v2_0", versions)
        self.assertIn("v2_1", versions)
        self.assertIn("v3_0", versions)
        group_names_v30 = {g.name for g in versioned_values.groups if g.version == "v3_0"}
        self.assertIn("KEM Algorithm", group_names_v30)
        self.assertIn("Cryptographic Algorithm", group_names_v30)
        self.assertIn("Key Format Type", group_names_v30)
        self.assertIn("Deactivation Reason Code", group_names_v30)
        self.assertIn("OTP Algorithm", group_names_v30)
        self.assertIn("Object Class Mask", group_names_v30)
        self.assertIn("Credential Type", group_names_v30)
        self.assertIn("Item Type", group_names_v30)
        self.assertIn("Query Function", group_names_v30)
        self.assertIn("Recommended Curve", group_names_v30)
        self.assertIn("Revocation Reason Code", group_names_v30)
        self.assertIn("Unique Identifier", group_names_v30)


    def test_kmip21_versioned_tags_and_rotate_attribute_policy_bounds(self) -> None:
        versioned_tags = json.loads(
            (REPO_ROOT / "verification/conformance/catalog/kmip-versioned-tags.json").read_text("utf-8")
        )
        v21_tags = versioned_tags.get("v2_1_tags", [])
        self.assertEqual(len(v21_tags), 36)
        tag_by_wire = {t["wire_value"].lower(): t for t in v21_tags}
        self.assertEqual(tag_by_wire["0x420166"]["name"], "Object Groups")
        self.assertEqual(tag_by_wire["0x420167"]["name"], "Object Types")
        self.assertEqual(tag_by_wire["0x420168"]["name"], "Constraints")
        self.assertEqual(tag_by_wire["0x420169"]["name"], "Constraint")
        self.assertEqual(tag_by_wire["0x42016a"]["name"], "Rotate Interval")
        self.assertTrue(tag_by_wire["0x42016a"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x42016b"]["name"], "Rotate Automatic")
        self.assertTrue(tag_by_wire["0x42016b"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x42016c"]["name"], "Rotate Offset")
        self.assertTrue(tag_by_wire["0x42016c"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x42016d"]["name"], "Rotate Date")
        self.assertTrue(tag_by_wire["0x42016d"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x42016e"]["name"], "Rotate Generation")
        self.assertTrue(tag_by_wire["0x42016e"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x42016f"]["name"], "Rotate Name")
        self.assertTrue(tag_by_wire["0x42016f"].get("is_object_attribute"))
        self.assertEqual(tag_by_wire["0x420172"]["name"], "Rotate Latest")
        self.assertTrue(tag_by_wire["0x420172"].get("is_object_attribute"))

        catalog = json.loads((REPO_ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text("utf-8"))
        rust = generate_attribute_policy.render_rust(catalog, versioned_tags=versioned_tags)
        for tag_hex, name in [
            ("0x42_01_6A", "Rotate Interval"),
            ("0x42_01_6B", "Rotate Automatic"),
            ("0x42_01_6C", "Rotate Offset"),
            ("0x42_01_6D", "Rotate Date"),
            ("0x42_01_6E", "Rotate Generation"),
            ("0x42_01_6F", "Rotate Name"),
            ("0x42_01_72", "Rotate Latest"),
        ]:
            self.assertIn(
                f'tag: {tag_hex},\n        element_id: "KMIPKIT-ELEM-ATTRIBUTE-{name.upper().replace(" ", "-")}",\n        name: "{name}",\n        min_version: (2, 1),',
                rust,
            )



if __name__ == "__main__":
    unittest.main()
