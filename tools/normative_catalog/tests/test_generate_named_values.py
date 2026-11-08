"""Tests for the deterministic four-language named-value generator."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog import generate_named_values, reconcile_named_value_manifest


ROOT = Path(__file__).resolve().parents[3]


class NamedValueGeneratorTests(unittest.TestCase):
    def test_version_delta_catalog_maps_exact_kmip_2_1_and_3_0_assignments(self) -> None:
        versioned = generate_named_values.load_versioned_values(ROOT)
        rendered = generate_named_values.render_versioned_outputs(versioned)

        actual = {
            (group.version, group.name): [(value.name, value.value) for value in group.values]
            for group in versioned.groups
        }
        self.assertEqual(
            {
                ("v2_1", "Operation"): [
                    ("Set Defaults", 0x36),
                    ("Set Constraints", 0x37),
                    ("Get Constraints", 0x38),
                    ("Query Asynchronous Requests", 0x39),
                    ("Process", 0x3A),
                    ("Ping", 0x3B),
                ],
                ("v3_0", "Operation"): [
                    ("Create Group", 0x3C),
                    ("Obliterate", 0x3D),
                    ("Create User", 0x3E),
                    ("Create Credential", 0x3F),
                    ("Deactivate", 0x40),
                    ("Encapsulate", 0x41),
                    ("Decapsulate", 0x42),
                ],
                ("v3_0", "Object Class"): [("User", 1), ("System", 2)],
                ("v3_0", "Object Type"): [
                    ("User", 0x0B),
                    ("Group", 0x0C),
                    ("Password Credential", 0x0D),
                    ("Device Credential", 0x0E),
                    ("One Time Password Credential", 0x0F),
                    ("Hashed Password Credential", 0x10),
                ],
            },
            actual,
        )

        self.assertIn("CREATE_GROUP", rendered["rust"])
        self.assertIn("OBLITERATE", rendered["rust"])
        self.assertIn("CREATE_CREDENTIAL", rendered["rust"])
        self.assertIn("PASSWORD_CREDENTIAL", rendered["rust"])
        self.assertIn("KMIPKIT_V3_0_OPERATION_CREATE_GROUP", rendered["c"])
        self.assertIn("KMIPKIT_V3_0_OPERATION_CREATE_CREDENTIAL", rendered["c"])
        self.assertIn("V3_0_OPERATION_CREATE_GROUP", rendered["java"])
        self.assertIn("V3_0_OBJECT_TYPE_PASSWORD_CREDENTIAL", rendered["python"])
        self.assertNotIn("RE_ENCRYPT", rendered["rust"])
        self.assertNotIn("CREDENTIAL = 0x0000000D", rendered["rust"])

    def test_versioned_namespace_is_emitted_in_all_four_generated_surfaces(self) -> None:
        catalog = generate_named_values.load_validated_catalog(
            generate_named_values.safe_read_bytes(
                ROOT,
                generate_named_values.CATALOG_PATH,
                max_bytes=generate_named_values.MAX_BYTES,
            ),
            ROOT,
            require_complete=True,
        )
        rendered = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog),
            generate_named_values.load_versioned_values(ROOT),
        )

        self.assertIn("pub mod versions", rendered["rust"])
        self.assertIn("pub const OBLITERATE: u32 = 0x0000_003D", rendered["rust"])
        self.assertIn("#define KMIPKIT_V3_0_OPERATION_OBLITERATE UINT32_C(0x0000003D)", rendered["c"])
        self.assertIn("V3_0_OPERATION_OBLITERATE = 0x0000003DL", rendered["java"])
        self.assertIn("V3_0_OPERATION_OBLITERATE = 0x0000003D", rendered["python"])

    def test_versioned_public_values_are_reconciled_with_cross_language_mappings(self) -> None:
        catalog = generate_named_values.load_validated_catalog(
            generate_named_values.safe_read_bytes(
                ROOT,
                generate_named_values.CATALOG_PATH,
                max_bytes=generate_named_values.MAX_BYTES,
            ),
            ROOT,
            require_complete=True,
        )
        declarations, _ = reconcile_named_value_manifest.build_named_value_records(
            catalog,
            generate_named_values.load_versioned_values(ROOT),
        )
        rust_name = (
            "kmipkit::protocol::named_values_generated::values::versions::"
            "v3_0::enumeration::operation::CREATE_CREDENTIAL"
        )
        declaration = next(row for row in declarations if row["exportName"] == rust_name)

        self.assertEqual(
            {
                "rust": rust_name,
                "c": "KMIPKIT_V3_0_OPERATION_CREATE_CREDENTIAL",
                "java": "org.kmipkit.protocol.KmipValues.V3_0_OPERATION_CREATE_CREDENTIAL",
                "python": "kmipkit.protocol_values.V3_0_OPERATION_CREATE_CREDENTIAL",
            },
            {
                language: mapping["name"]
                for language, mapping in declaration["languageMappings"].items()
            },
        )

    def test_public_manifest_bound_covers_current_input_with_bounded_headroom(self) -> None:
        manifest_size = (ROOT / reconcile_named_value_manifest.MANIFEST_PATH).stat().st_size

        self.assertEqual(64 * 1024 * 1024, reconcile_named_value_manifest.MAX_MANIFEST_BYTES)
        self.assertLessEqual(manifest_size, reconcile_named_value_manifest.MAX_MANIFEST_BYTES)

    def test_bitmask_java_helper_names_use_camel_case(self) -> None:
        group = generate_named_values.NamedGroup(
            "mask-1",
            "Cryptographic Usage Mask",
            "CRYPTOGRAPHIC_USAGE_MASK",
            (),
        )

        known = reconcile_named_value_manifest._language_names(
            "bitmask", group, "known_bits"
        )
        unknown = reconcile_named_value_manifest._language_names(
            "bitmask", group, "unknown_bits"
        )

        self.assertEqual(
            "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.knownBits",
            known["java"],
        )
        self.assertEqual(
            "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.unknownBits",
            unknown["java"],
        )
        self.assertTrue(known["rust"].endswith("::known_bits"))
        self.assertTrue(known["python"].endswith(".known_bits"))

    def test_bitmask_name_helper_is_reconciled_as_c_only(self) -> None:
        root = ROOT
        catalog = reconcile_named_value_manifest.load_validated_catalog(
            reconcile_named_value_manifest.safe_read_bytes(
                root,
                generate_named_values.CATALOG_PATH,
                max_bytes=reconcile_named_value_manifest.MAX_BYTES,
            ),
            root,
            require_complete=True,
        )
        declarations, mappings = reconcile_named_value_manifest.build_named_value_records(catalog)
        reconciled = reconcile_named_value_manifest.reconcile(
            {"requirementIds": [], "capabilities": [], "publicDeclarations": []},
            declarations,
            mappings,
        )

        bitmask_name_rows = [
            row
            for row in reconciled["publicDeclarations"]
            if "KMIPKIT-0047-FR-004" in row.get("requirementIds", [])
            and row.get("exportName", "").endswith("name")
        ]
        self.assertEqual(3, len(bitmask_name_rows))
        self.assertEqual({"c-kmipkit"}, {row["packageId"] for row in bitmask_name_rows})
        self.assertEqual(
            {"named_protocol_mask_single_bit_lookup"},
            {row["capabilityId"] for row in bitmask_name_rows},
        )
        capability = next(
            row
            for row in reconciled["capabilities"]
            if row["capabilityId"] == "named_protocol_mask_single_bit_lookup"
        )
        self.assertEqual("c-only", capability["parityScope"])

    def test_rust_named_values_are_mapped_through_facade_and_protocol_crate(self) -> None:
        root = ROOT
        catalog = reconcile_named_value_manifest.load_validated_catalog(
            reconcile_named_value_manifest.safe_read_bytes(
                root,
                generate_named_values.CATALOG_PATH,
                max_bytes=reconcile_named_value_manifest.MAX_BYTES,
            ),
            root,
            require_complete=True,
        )
        declarations, _ = reconcile_named_value_manifest.build_named_value_records(catalog)
        rows = [
            row
            for row in declarations
            if row["exportName"].endswith(
                "named_values_generated::values::bitmask::cryptographic_usage_mask::AUTHENTICATE"
            )
        ]

        self.assertEqual(
            {
                ("rust-kmipkit", "kmipkit::protocol::named_values_generated::values::bitmask::cryptographic_usage_mask::AUTHENTICATE"),
                ("rust-kmipkit-protocol", "kmipkit_protocol::named_values_generated::values::bitmask::cryptographic_usage_mask::AUTHENTICATE"),
            },
            {(row["packageId"], row["exportName"]) for row in rows},
        )

    def test_manifest_reconciliation_preserves_existing_declaration_order(self) -> None:
        existing = [
            {"declarationId": "z-existing", "exportName": "Existing Z"},
            {"declarationId": "a-existing", "exportName": "Existing A"},
        ]
        new_declarations = [
            {"declarationId": "named-z", "approvalSource": {"path": "specs/047-named-protocol-values/spec.md"}},
            {"declarationId": "named-a", "approvalSource": {"path": "specs/047-named-protocol-values/spec.md"}},
        ]
        existing_capabilities = [
            {"capabilityId": "z-capability"},
            {"capabilityId": "a-capability"},
        ]
        manifest = {
            "requirementIds": [],
            "capabilities": existing_capabilities,
            "publicDeclarations": existing,
        }

        reconciled = reconcile_named_value_manifest.reconcile(
            manifest,
            new_declarations,
            {"rust": [], "c": [], "java": [], "python": []},
        )

        self.assertEqual(
            ["z-existing", "a-existing", "named-a", "named-z"],
            [row["declarationId"] for row in reconciled["publicDeclarations"]],
        )
        self.assertEqual(
            ["z-capability", "a-capability", "named_protocol_values"],
            [row["capabilityId"] for row in reconciled["capabilities"]],
        )

    def test_manifest_reconciliation_removes_stale_generated_rows_owned_by_spec(self) -> None:
        stale = {
            "declarationId": "kmipkit-0047-java-stale-name",
            "packageId": "java-kmipkit",
            "exportName": "org.kmipkit.protocol.KmipValues.Mask.known_bits",
            "capabilityId": "named_protocol_values",
            "approvalSource": reconcile_named_value_manifest.APPROVAL_SOURCE,
        }
        current = {
            "declarationId": "kmipkit-0047-java-current-name",
            "packageId": "java-kmipkit",
            "exportName": "org.kmipkit.protocol.KmipValues.Mask.knownBits",
            "capabilityId": "named_protocol_values",
            "approvalSource": reconcile_named_value_manifest.APPROVAL_SOURCE,
        }
        reconciled = reconcile_named_value_manifest.reconcile(
            {
                "requirementIds": [],
                "capabilities": [],
                "publicDeclarations": [stale],
            },
            [current],
            {"rust": [], "c": [], "java": [current["exportName"]], "python": []},
        )

        self.assertEqual([current], reconciled["publicDeclarations"])

    def test_catalog_contains_the_approved_named_value_surface(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

        values = generate_named_values.collect_values(catalog)

        self.assertEqual(len(values.enumerations), 64)
        self.assertEqual(sum(len(value.values) for value in values.enumerations), 644)
        self.assertEqual(len(values.bitmasks), 3)
        self.assertEqual(sum(len(value.values) for value in values.bitmasks), 33)

        catalog_values = [
            value
            for value in catalog["elements"]
            if value.get("kind") in {"enumeration_value", "bitmask_value"}
        ]
        self.assertEqual(sum(value["kind"] == "enumeration_value" for value in catalog_values), 723)
        self.assertEqual(sum(value["kind"] == "bitmask_value" for value in catalog_values), 44)
        self.assertEqual(
            sum(value.get("allocation") == "assigned" for value in catalog_values),
            677,
        )

    def test_outputs_include_named_values_in_every_language(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )

        outputs = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog)
        )

        self.assertEqual(
            set(outputs),
            {"rust", "c", "java", "python", "traceability"},
        )
        for language in ("rust", "c", "java", "python"):
            source = outputs[language]
            self.assertIn("AES", source)
            self.assertIn("ENCRYPT", source)
        self.assertIn("KMIPKIT-ELEM-ENUM", outputs["traceability"])

    def test_rust_output_is_clippy_clean_for_numeric_literals_and_pure_lookups(self) -> None:
        catalog: dict[str, Any] = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        rust = generate_named_values.render_outputs(
            generate_named_values.collect_values(catalog)
        )["rust"]

        self.assertIn("pub const INCREMENT: u32 = 0x0000_0001;", rust)
        self.assertIn("#[must_use]\n            pub const fn name", rust)
        self.assertIn("#[must_use]\n            pub const fn known_bits", rust)
        self.assertIn("/// `AESKeyWrapPadding`;", rust)
        self.assertNotIn("match raw {\n                    _ => None", rust)
        self.assertNotIn("match raw {\n                None", rust)
        self.assertIn("pub const fn name(_raw: u32)", rust)

    def test_duplicate_names_and_values_are_rejected(self) -> None:
        catalog: dict[str, Any] = {
            "elements": [
                {
                    "element_id": "enum-1",
                    "kind": "enumeration",
                    "name": "Example Kind",
                },
                {
                    "element_id": "value-1",
                    "kind": "enumeration_value",
                    "name": "Same",
                    "source_refs": [{"source_id": "source", "section": "11.1"}],
                    "parent_element_ids": ["enum-1"],
                    "wire_value": "00000001",
                    "allocation": "assigned",
                },
                {
                    "element_id": "value-2",
                    "kind": "enumeration_value",
                    "name": "Same",
                    "source_refs": [{"source_id": "source", "section": "11.1"}],
                    "parent_element_ids": ["enum-1"],
                    "wire_value": "00000002",
                    "allocation": "assigned",
                },
            ]
        }

        with self.assertRaisesRegex(ValueError, "duplicate.*name"):
            generate_named_values.collect_values(catalog)

    def test_group_name_normalization_collisions_are_rejected(self) -> None:
        catalog: dict[str, Any] = {
            "elements": [
                {"element_id": "enum-1", "kind": "enumeration", "name": "Same Group"},
                {"element_id": "enum-2", "kind": "enumeration", "name": "Same-Group"},
            ]
        }

        with self.assertRaisesRegex(ValueError, "duplicate generated group identifier"):
            generate_named_values.collect_values(catalog)

    def test_superscript_bits_remain_distinct_in_generated_symbols(self) -> None:
        self.assertNotEqual(
            generate_named_values._identifier("Polynomial Sharing GF (2¹⁶)"),
            generate_named_values._identifier("Polynomial Sharing GF (2⁸)"),
        )

    def test_c_string_literals_escape_syntax_control_and_utf8_bytes(self) -> None:
        self.assertEqual(
            r'"Name\042\134\012\011\303\251"',
            generate_named_values._c_string('Name"\\\n\té'),
        )

    def test_c_renderer_uses_safe_literals_for_every_lookup_helper(self) -> None:
        value = generate_named_values.NamedValue(
            element_id="value-1",
            name='Name"\\\n',
            identifier="NAME",
            value=1,
            source_refs=(("KMIPKIT-SRC-spec", "11.1"),),
        )
        enumeration = generate_named_values.NamedGroup(
            "enum-1", "Example Enum", "EXAMPLE_ENUM", (value,)
        )
        bitmask = generate_named_values.NamedGroup(
            "mask-1", "Example Mask", "EXAMPLE_MASK", (value,)
        )
        outputs = generate_named_values._render_c(
            generate_named_values.CatalogValues((enumeration,), (bitmask,))
        )

        self.assertEqual(3, outputs.count(r'"Name\042\134\012"'))


if __name__ == "__main__":
    unittest.main()
