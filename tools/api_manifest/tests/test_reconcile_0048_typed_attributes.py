"""Tests for deterministic KMIPKIT-0048 Rust export reconciliation."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest.reconcile_0048_typed_attributes import (
    ReconciliationError,
    reconcile,
)


class Reconcile0048TypedAttributesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "requirementIds": ["KMIPKIT-0047-FR-001"],
            "capabilities": [
                {
                    "capabilityId": "named_protocol_values",
                    "requirementIds": ["KMIPKIT-0047-FR-001"],
                    "capabilityState": "declared",
                    "parityScope": "four-language",
                    "mappings": {"rust": [
                        "kmipkit::protocol::named_values_generated::values::enumeration::state::ACTIVE"
                    ]},
                }
            ],
            "publicDeclarations": [
                {
                    "declarationId": "facade-active",
                    "packageId": "rust-kmipkit",
                    "kind": "constant",
                    "exportName": "kmipkit::protocol::named_values_generated::values::enumeration::state::ACTIVE",
                    "capabilityId": "named_protocol_values",
                    "requirementIds": ["KMIPKIT-0047-FR-001"],
                    "declarationState": "declared",
                    "parityScope": "four-language",
                    "approvalSource": {"kind": "feature-specification", "path": "specs/047-named-protocol-values/spec.md", "status": "Approved", "evidence": "Approved"},
                    "languageMappings": {
                        "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::named_values_generated::values::enumeration::state::ACTIVE"},
                        "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ENUM_STATE_ACTIVE"},
                        "java": {"packageId": "java-kmipkit", "name": "KmipValues.State.ACTIVE"},
                        "python": {"packageId": "python-kmipkit", "name": "State.ACTIVE"},
                    },
                }
                ,{
                    "declarationId": "c-mask-name",
                    "packageId": "c-kmipkit",
                    "kind": "function",
                    "exportName": "KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name",
                    "capabilityId": "named_protocol_values",
                    "requirementIds": ["KMIPKIT-0047-FR-004"],
                    "declarationState": "declared",
                    "parityScope": "four-language",
                    "approvalSource": {"kind": "feature-specification", "path": "specs/047-named-protocol-values/spec.md", "status": "Approved", "evidence": "Approved"},
                    "languageMappings": {
                        "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::named_values_generated::values::bitmask::cryptographic_usage_mask::name"},
                        "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name"},
                        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.name"},
                        "python": {"packageId": "python-kmipkit", "name": "kmipkit.protocol_values.CryptographicUsageMask.name"},
                    },
                }
            ],
            "approvedExclusions": [],
        }
        self.report = {
            "missingFromManifest": [
                {
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "constant",
                    "exportName": "kmipkit_protocol::named_values_generated::values::enumeration::state::ACTIVE",
                },
                {
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "type",
                    "exportName": "kmipkit_protocol::attribute::typed::Comment",
                },
                {
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "<kmipkit::protocol::Attribute<D> as core::clone::Clone>::clone",
                },
            ]
        }

    def test_reuses_foreign_equivalents_for_raw_protocol_aliases(self) -> None:
        result = reconcile(self.manifest, self.report)
        row = next(
            item for item in result["publicDeclarations"]
            if item["packageId"] == "rust-kmipkit-protocol"
            and item["exportName"].endswith("::enumeration::state::ACTIVE")
        )

        self.assertEqual(row["capabilityId"], "named_protocol_values")
        self.assertEqual(row["languageMappings"]["rust"], {
            "packageId": "rust-kmipkit-protocol",
            "name": "kmipkit_protocol::named_values_generated::values::enumeration::state::ACTIVE",
        })
        self.assertEqual(row["languageMappings"]["c"], self.manifest["publicDeclarations"][0]["languageMappings"]["c"])
        named_values = next(item for item in result["capabilities"] if item["capabilityId"] == "named_protocol_values")
        self.assertIn(row["languageMappings"]["rust"]["name"], named_values["mappings"]["rust"])

    def test_blocks_only_feature_routes_until_four_language_parity(self) -> None:
        result = reconcile(self.manifest, self.report)
        row = next(item for item in result["publicDeclarations"] if item["capabilityId"] == "typed_object_attributes")

        self.assertEqual(row["declarationState"], "blocked")
        self.assertEqual(row["candidateSpecId"], "KMIPKIT-0053")
        self.assertNotIn("languageMappings", row)
        self.assertEqual(row["exportName"], "kmipkit_protocol::attribute::typed::Comment")

    def test_excludes_exact_trait_projection_with_feature_evidence(self) -> None:
        result = reconcile(self.manifest, self.report)
        exclusion = result["approvedExclusions"][0]

        self.assertEqual(exclusion["exportName"], "<kmipkit::protocol::Attribute<D> as core::clone::Clone>::clone")
        self.assertEqual(exclusion["candidateSpecId"], "KMIPKIT-0048")
        self.assertEqual(exclusion["approvalSource"]["path"], "specs/048-typed-attributes/spec.md")

    def test_reconciliation_is_deterministic_and_does_not_mutate_inputs(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        report = copy.deepcopy(self.report)

        first_result = reconcile(copy.deepcopy(manifest), copy.deepcopy(report))
        second_result = reconcile(copy.deepcopy(manifest), copy.deepcopy(report))

        self.assertEqual(first_result, second_result)
        self.assertEqual(manifest, self.manifest)
        self.assertEqual(report, self.report)

    def test_preserves_reviewed_manifest_order_for_existing_rows(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        manifest["publicDeclarations"] = [manifest["publicDeclarations"][0]]
        manifest["capabilities"][0]["mappings"] = {
            "c": [], "java": [], "python": [], "rust": [
                "kmipkit::protocol::named_values_generated::values::enumeration::state::ACTIVE"
            ],
        }
        second_declaration = copy.deepcopy(manifest["publicDeclarations"][0])
        second_declaration["declarationId"] = "reviewed-second"
        second_declaration["exportName"] = "kmipkit::protocol::ZExisting"
        second_capability = {
            "capabilityId": "reviewed-second",
            "requirementIds": ["KMIPKIT-0047-FR-001"],
            "capabilityState": "declared",
            "parityScope": "four-language",
            "mappings": {"rust": ["kmipkit::protocol::ZExisting"]},
        }
        first_exclusion = {
            "packageId": "rust-kmipkit",
            "exportName": "first-reviewed-exclusion",
            "reason": "Reviewed first.",
            "approvalSource": {"kind": "feature-specification", "path": "spec.md", "status": "Approved", "evidence": "Approved"},
            "evidence": "first",
        }
        second_exclusion = {**first_exclusion, "exportName": "second-reviewed-exclusion", "evidence": "second"}
        manifest["publicDeclarations"].append(second_declaration)
        manifest["capabilities"].append(second_capability)
        manifest["approvedExclusions"] = [first_exclusion, second_exclusion]

        result = reconcile(manifest, {"missingFromManifest": []})

        self.assertEqual(
            [row["declarationId"] for row in result["publicDeclarations"][:2]],
            [row["declarationId"] for row in manifest["publicDeclarations"]],
        )
        self.assertEqual(
            [row["capabilityId"] for row in result["capabilities"][:2]],
            [row["capabilityId"] for row in manifest["capabilities"]],
        )
        self.assertEqual(result["approvedExclusions"], manifest["approvedExclusions"])

    def test_unknown_export_fails_closed(self) -> None:
        report = copy.deepcopy(self.report)
        report["missingFromManifest"].append({
            "packageId": "rust-kmipkit",
            "kind": "function",
            "exportName": "kmipkit::unexpected::new_capability",
        })

        with self.assertRaises(ReconciliationError):
            reconcile(self.manifest, report)

    def test_unowned_trait_projection_fails_closed(self) -> None:
        report = {"missingFromManifest": [{
            "packageId": "rust-kmipkit-transport",
            "kind": "associated-item",
            "exportName": "<kmipkit_transport::TlsConfig as core::fmt::Debug>::fmt",
        }]}

        with self.assertRaises(ReconciliationError):
            reconcile(self.manifest, report)

    def test_unowned_attribute_trait_projection_fails_closed(self) -> None:
        report = {"missingFromManifest": [{
            "packageId": "rust-kmipkit-protocol",
            "kind": "associated-item",
            "exportName": "<kmipkit_protocol::attribute::ClientAttributeMutation as core::fmt::Debug>::fmt",
        }]}

        with self.assertRaises(ReconciliationError):
            reconcile(self.manifest, report)

    def test_unowned_attribute_function_fails_closed(self) -> None:
        report = {"missingFromManifest": [{
            "packageId": "rust-kmipkit-protocol",
            "kind": "function",
            "exportName": "kmipkit_protocol::attribute::client_vendor_attribute_mutation_is_prohibited",
        }]}

        with self.assertRaises(ReconciliationError):
            reconcile(self.manifest, report)

    def test_trait_projection_checks_the_exact_package_and_owner(self) -> None:
        cases = (
            ("rust-kmipkit", "<kmipkit::TlsConfig as core::fmt::Debug>::fmt"),
            ("rust-kmipkit", "<kmipkit_protocol::Attribute<D> as core::clone::Clone>::clone"),
            ("rust-kmipkit-protocol", "<kmipkit_protocol::AttributeReference as core::fmt::Debug>::fmt"),
            ("rust-kmipkit", "<kmipkit::TlsConfig as core::convert::Into<kmipkit::protocol::Attribute<D>>>::into"),
        )
        for package_id, export_name in cases:
            with self.subTest(package_id=package_id, export_name=export_name):
                report = {"missingFromManifest": [{
                    "packageId": package_id,
                    "kind": "associated-item",
                    "exportName": export_name,
                }]}
                with self.assertRaises(ReconciliationError):
                    reconcile(self.manifest, report)

    def test_typed_trait_projections_keep_their_feature_exclusion(self) -> None:
        cases = (
            ("rust-kmipkit-protocol", "<kmipkit_protocol::Attribute<D> as core::clone::Clone>::clone"),
            ("rust-kmipkit", "<kmipkit::protocol::attribute::typed::Comment as core::fmt::Debug>::fmt"),
            ("rust-kmipkit-protocol", "<kmipkit_protocol::attribute::AttributeValueError as core::fmt::Debug>::fmt"),
        )
        for package_id, export_name in cases:
            with self.subTest(package_id=package_id, export_name=export_name):
                report = {"missingFromManifest": [{
                    "packageId": package_id,
                    "kind": "associated-item",
                    "exportName": export_name,
                }]}
                result = reconcile(self.manifest, report)
                self.assertEqual(len(result["approvedExclusions"]), 1)
                exclusion = result["approvedExclusions"][0]
                self.assertEqual(exclusion["packageId"], package_id)
                self.assertEqual(exclusion["exportName"], export_name)
                self.assertEqual(exclusion["candidateSpecId"], "KMIPKIT-0048")

    def test_registers_only_the_requirement_ids_used_by_new_rows(self) -> None:
        result = reconcile(self.manifest, self.report)

        self.assertEqual(
            result["requirementIds"],
            ["KMIPKIT-0047-FR-001", *[f"KMIPKIT-0048-FR-{number:03}" for number in range(1, 11)]],
        )

    def test_removes_unimplemented_singular_mask_name_facade_routes(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        rust_paths = [
            "kmipkit::protocol::named_values_generated::values::bitmask::cryptographic_usage_mask::name",
            "kmipkit::protocol::named_values_generated::values::bitmask::protection_storage_mask::name",
            "kmipkit::protocol::named_values_generated::values::bitmask::storage_status_mask::name",
        ]
        manifest["publicDeclarations"].extend({
            "declarationId": f"invalid-{index}",
            "packageId": "rust-kmipkit",
            "kind": "function",
            "exportName": path,
            "capabilityId": "named_protocol_values",
            "requirementIds": ["KMIPKIT-0047-FR-004"],
            "declarationState": "declared",
            "parityScope": "four-language",
            "languageMappings": {"rust": {"packageId": "rust-kmipkit", "name": path}},
        } for index, path in enumerate(rust_paths))
        classes = ("CryptographicUsageMask", "ProtectionStorageMask", "StorageStatusMask")
        java_paths = [f"org.kmipkit.protocol.KmipValues.{name}.name" for name in classes]
        python_paths = [f"kmipkit.protocol_values.{name}.name" for name in classes]
        manifest["publicDeclarations"].extend({
            "declarationId": f"invalid-java-{index}", "packageId": "java-kmipkit",
            "kind": "java-member", "exportName": path, "capabilityId": "named_protocol_values",
            "requirementIds": ["KMIPKIT-0047-FR-004"], "declarationState": "declared",
            "parityScope": "four-language", "languageMappings": {"java": {"packageId": "java-kmipkit", "name": path}},
        } for index, path in enumerate(java_paths))
        manifest["publicDeclarations"].extend({
            "declarationId": f"invalid-python-{index}", "packageId": "python-kmipkit",
            "kind": "python-export", "exportName": path, "capabilityId": "named_protocol_values",
            "requirementIds": ["KMIPKIT-0047-FR-004"], "declarationState": "declared",
            "parityScope": "four-language", "languageMappings": {"python": {"packageId": "python-kmipkit", "name": path}},
        } for index, path in enumerate(python_paths))
        c_paths = [
            "KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name",
            "KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_name",
            "KMIPKIT_BITMASK_STORAGE_STATUS_MASK_name",
        ]
        manifest["capabilities"][0]["mappings"].update({
            "c": c_paths, "java": java_paths, "python": python_paths,
        })
        manifest["capabilities"][0]["mappings"]["rust"].extend(rust_paths)

        result = reconcile(manifest, self.report)

        declarations = {row["exportName"] for row in result["publicDeclarations"]}
        self.assertTrue(set(rust_paths + java_paths + python_paths).isdisjoint(declarations))
        named_values = next(item for item in result["capabilities"] if item["capabilityId"] == "named_protocol_values")
        self.assertTrue(set(rust_paths).isdisjoint(named_values["mappings"]["rust"]))
        self.assertTrue(set(java_paths).isdisjoint(named_values["mappings"]["java"]))
        self.assertTrue(set(python_paths).isdisjoint(named_values["mappings"]["python"]))
        self.assertTrue(set(c_paths).isdisjoint(named_values["mappings"]["c"]))
        c_helper = next(row for row in result["publicDeclarations"] if row["declarationId"] == "c-mask-name")
        self.assertEqual(c_helper["parityScope"], "c-only")
        self.assertEqual(c_helper["capabilityId"], "named_protocol_mask_single_bit_lookup")
        self.assertEqual(set(c_helper["languageMappings"]), {"c"})

    def test_corrects_java_mask_method_names_to_the_published_camel_case(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        old_name = "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.known_bits"
        new_name = "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.knownBits"
        manifest["capabilities"][0]["mappings"]["java"] = [old_name]
        manifest["publicDeclarations"][0]["languageMappings"]["java"] = {
            "packageId": "java-kmipkit", "name": old_name,
        }
        manifest["publicDeclarations"].append({
            "declarationId": "old-java-known-bits",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": old_name,
            "capabilityId": "named_protocol_values",
            "requirementIds": ["KMIPKIT-0047-FR-004"],
            "declarationState": "declared",
            "parityScope": "four-language",
            "languageMappings": {"java": {"packageId": "java-kmipkit", "name": old_name}},
        })

        result = reconcile(manifest, {"missingFromManifest": []})

        declaration = next(row for row in result["publicDeclarations"] if row["packageId"] == "java-kmipkit")
        self.assertEqual(declaration["exportName"], new_name)
        self.assertEqual(declaration["declarationId"], f"kmipkit-0047-java-{new_name}")
        facade_row = next(row for row in result["publicDeclarations"] if row["declarationId"] == "facade-active")
        self.assertEqual(facade_row["languageMappings"]["java"]["name"], new_name)
        named_values = next(item for item in result["capabilities"] if item["capabilityId"] == "named_protocol_values")
        self.assertEqual(named_values["mappings"]["java"], [new_name])


if __name__ == "__main__":
    unittest.main()
