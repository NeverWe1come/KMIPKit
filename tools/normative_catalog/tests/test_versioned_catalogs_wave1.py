from __future__ import annotations

import json
from pathlib import Path
import unittest

from tools.normative_catalog import (
    generate_attribute_policy,
    generate_named_values,
    generate_result_values,
    generate_wire_names,
)

REPO_ROOT = Path(__file__).resolve().parents[3]


class VersionedCatalogsWave1Tests(unittest.TestCase):
    def test_wire_names_include_kmip30_tags_and_enumerations(self) -> None:
        catalog = json.loads((REPO_ROOT / "specification/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_tags = json.loads(
            (REPO_ROOT / "specification/catalog/kmip-versioned-tags.json").read_text("utf-8")
        )
        versioned_values = json.loads(
            (REPO_ROOT / "specification/catalog/kmip-versioned-values.json").read_text("utf-8")
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

    def test_attribute_policy_includes_version_bounds_and_kmip30_attributes(self) -> None:
        catalog = json.loads((REPO_ROOT / "specification/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_tags = json.loads(
            (REPO_ROOT / "specification/catalog/kmip-versioned-tags.json").read_text("utf-8")
        )
        rust = generate_attribute_policy.render_rust(catalog, versioned_tags=versioned_tags)
        self.assertIn("0x0042_0190", rust)  # Certificate Link
        self.assertIn("0x0042_01BD", rust)  # Certify Counter
        self.assertIn("0x0042_019E", rust)  # Object Class
        self.assertIn("0x0042_01B8", rust)  # Deactivation Reason
        self.assertIn("0x0042_01C2", rust)  # NIST Security Category
        self.assertIn("0x0042_01B6", rust)  # Split Key Polynomial
        self.assertIn("min_version", rust)
        self.assertIn("max_version", rust)

    def test_result_values_include_kmip30_circular_link_error(self) -> None:
        catalog = json.loads((REPO_ROOT / "specification/catalog/kmip-2.1.json").read_text("utf-8"))
        versioned_values = json.loads(
            (REPO_ROOT / "specification/catalog/kmip-versioned-values.json").read_text("utf-8")
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


if __name__ == "__main__":
    unittest.main()
