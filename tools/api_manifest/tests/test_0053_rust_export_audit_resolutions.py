"""Validate the standalone KMIPKIT-0053 Rust export audit resolutions."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
AUDIT_RESOLUTIONS = (
    ROOT
    / "specs"
    / "053-api-parity-facade"
    / "candidate-resolutions"
    / "rust-export-audit.json"
)


def declaration_id(package_id: str, export_name: str) -> str:
    digest = hashlib.sha256(f"{package_id}\0{export_name}".encode("utf-8")).hexdigest()[:20]
    return f"rust-observed-{digest}"


class RustExportAuditResolutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.audit = json.loads(AUDIT_RESOLUTIONS.read_text(encoding="utf-8"))

    def test_audit_counts_and_resolution_groups_are_exact(self) -> None:
        self.assertEqual(self.audit["formatVersion"], 1)
        self.assertEqual(self.audit["specId"], "KMIPKIT-0053")
        self.assertEqual(self.audit["sourceAudit"]["missingFromManifest"], 226)
        self.assertEqual(len(self.audit["functionalRoutes"]), 38)
        self.assertEqual(len(self.audit["enumShapeExclusions"]), 66)
        self.assertEqual(len(self.audit["traitImplementationExclusions"]), 116)
        self.assertEqual(len(self.audit["exactRouteExclusions"]), 4)
        self.assertEqual(len(self.audit["priorApprovedExclusions"]), 2)
        self.assertEqual(len(self.audit["kindCorrections"]), 2)
        self.assertEqual(
            sum(row["resolution"] == "mapped" for row in self.audit["functionalRoutes"]),
            38,
        )
        self.assertEqual(
            sum(row["resolution"] == "unresolved" for row in self.audit["functionalRoutes"]),
            0,
        )
        self.assertEqual(
            sum(
                len(self.audit[group])
                for group in (
                    "functionalRoutes",
                    "enumShapeExclusions",
                    "traitImplementationExclusions",
                    "exactRouteExclusions",
                    "priorApprovedExclusions",
                )
            ),
            self.audit["sourceAudit"]["missingFromManifest"],
        )

    def test_prior_exclusion_approval_evidence_is_verbatim_in_its_source(self) -> None:
        for row in self.audit["priorApprovedExclusions"]:
            approval = row["approvalSource"]
            source = (ROOT / approval["path"]).read_text(encoding="utf-8")
            with self.subTest(exportName=row["exportName"]):
                self.assertIn(approval["evidence"], source)

    def test_functional_routes_have_unique_exact_keys_and_evidence(self) -> None:
        rows = self.audit["functionalRoutes"]
        keys = [(row["packageId"], row["exportName"]) for row in rows]
        self.assertEqual(len(keys), len(set(keys)))

        for row in rows:
            with self.subTest(exportName=row["exportName"]):
                self.assertIn(row["resolution"], {"mapped", "unresolved"})
                self.assertTrue(row["evidencePaths"])
                for evidence in row["evidencePaths"]:
                    self.assertTrue((ROOT / evidence).is_file(), evidence)
                if row["resolution"] == "mapped":
                    self.assertEqual(
                        set(row["languageMappings"]), {"c", "java", "python"}
                    )
                    for mapping in row["languageMappings"].values():
                        self.assertTrue(mapping["packageId"])
                        if "name" in mapping:
                            self.assertTrue(mapping["name"])
                        else:
                            self.assertTrue(mapping["routes"])
                            self.assertEqual(
                                len(mapping["routes"]),
                                len({route["name"] for route in mapping["routes"]}),
                            )
                            self.assertTrue(all(route["name"] for route in mapping["routes"]))
                else:
                    self.fail(f"functional route remains unresolved: {row['exportName']}")

    def test_functional_routes_are_manifest_ready_and_keep_composition_evidence(self) -> None:
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8")
        )
        declarations = {
            row["declarationId"]: row
            for row in manifest["publicDeclarations"]
            if row.get("declarationId")
        }

        for route in self.audit["functionalRoutes"]:
            with self.subTest(exportName=route["exportName"]):
                self.assertEqual(route["name"], route["exportName"])
                self.assertIn(route["kind"], {"type", "associated-item"})
                self.assertEqual(route["declarationKind"], route["kind"])
                self.assertTrue(route["capabilityId"])
                self.assertTrue(route["requirementIds"])
                self.assertTrue(route["approval"]["path"])
                self.assertEqual(route["approval"]["status"], "Approved")
                self.assertTrue(route["evidencePaths"])
                for evidence in route["evidencePaths"]:
                    self.assertTrue((ROOT / evidence).is_file(), evidence)

                reference = route["manifestMappingRef"]
                self.assertEqual(reference["path"], "specification/api/public-api.json")
                declaration = declarations[reference["declarationId"]]
                self.assertEqual(route["capabilityId"], declaration["capabilityId"])
                self.assertEqual(route["requirementIds"], declaration["requirementIds"])
                self.assertEqual(route["approval"], declaration["approvalSource"])
                self.assertEqual(
                    route["manifestMappingDeclarationKind"], declaration["kind"]
                )

                mapping = route["manifestMapping"]
                self.assertEqual(set(mapping), {"rust", "c", "java", "python"})
                self.assertTrue(all(set(item) == {"packageId", "name"} for item in mapping.values()))
                self.assertEqual(mapping, declaration["languageMappings"])

                composition_ids = route["manifestCompositionDeclarationIds"]
                self.assertIn(reference["declarationId"], composition_ids)
                self.assertTrue(composition_ids)
                self.assertTrue(all(item in declarations for item in composition_ids))
                # The complete route arrays remain as the source composition record.
                self.assertEqual(set(route["languageMappings"]), {"c", "java", "python"})
                for language, details in route["languageMappings"].items():
                    self.assertTrue(details.get("name") or details.get("routes"))

    def test_managed_reader_mappings_match_exact_binding_capabilities(self) -> None:
        rows = {
            (row["packageId"], row["exportName"]): row
            for row in self.audit["functionalRoutes"]
        }
        for package_id, prefix in (
            ("rust-kmipkit", "kmipkit::protocol::"),
            ("rust-kmipkit-protocol", "kmipkit_protocol::"),
        ):
            presence = rows[(package_id, prefix + "ManagedValueReader::has_field")]
            self.assertEqual(presence["resolution"], "mapped")
            self.assertEqual(
                presence["languageMappings"]["c"]["name"],
                "kmipkit_managed_value_field_read (read_kind=1)",
            )
            self.assertEqual(
                presence["languageMappings"]["java"]["name"],
                "org.kmipkit.ttlv.ManagedTtlvValue.hasField",
            )
            self.assertEqual(
                presence["languageMappings"]["python"]["name"],
                "kmipkit.managed_objects.ManagedTtlvItem.has_field",
            )

            typed_conversion = rows[(package_id, prefix + "ManagedValueReader::try_from_item")]
            self.assertEqual(typed_conversion["resolution"], "mapped")
            self.assertEqual(typed_conversion["mappingMode"], "kind-selected-route-set")
            expected_route_suffixes = {
                "c": {
                    "kmipkit_managed_object_from_item",
                    "kmipkit_key_block_from_item",
                    "kmipkit_key_value_from_item",
                    "kmipkit_key_material_from_item",
                    "kmipkit_key_wrapping_data_from_item",
                },
                "java": {
                    "org.kmipkit.ttlv.ManagedObject.fromItem",
                    "org.kmipkit.ttlv.KeyBlock.fromItem",
                    "org.kmipkit.ttlv.KeyValue.fromItem",
                    "org.kmipkit.ttlv.KeyMaterial.fromItem",
                    "org.kmipkit.ttlv.KeyWrappingData.fromItem",
                },
                "python": {
                    "kmipkit.managed_objects.ManagedObject.from_item",
                    "kmipkit.managed_objects.KeyBlock.from_item",
                    "kmipkit.managed_objects.KeyValue.from_item",
                    "kmipkit.managed_objects.KeyMaterial.from_item",
                    "kmipkit.managed_objects.KeyWrappingData.from_item",
                },
            }
            for language, mapping in typed_conversion["languageMappings"].items():
                self.assertEqual(
                    {route["name"] for route in mapping["routes"]},
                    expected_route_suffixes[language],
                )

            if package_id == "rust-kmipkit":
                item_read = rows[(package_id, prefix + "ManagedValueReader::read_field_item")]
                self.assertEqual(item_read["resolution"], "mapped")
                self.assertEqual(
                    item_read["languageMappings"]["c"]["name"],
                    "kmipkit_managed_value_field_item_read",
                )
                self.assertEqual(
                    item_read["languageMappings"]["java"]["name"],
                    "org.kmipkit.ttlv.ManagedObject.fieldItem",
                )
                self.assertEqual(
                    item_read["languageMappings"]["python"]["name"],
                    "kmipkit.managed_objects.ManagedTtlvItem.field_item",
                )
                dynamic_read = rows[(package_id, prefix + "ManagedValueReader::read_field")]
                self.assertEqual(dynamic_read["resolution"], "mapped")
                self.assertEqual(
                    dynamic_read["mappingMode"],
                    "field-item-plus-raw-ttlv-type-and-getter",
                )
                self.assertIn("full Rust ManagedFieldValue sum", dynamic_read["mappingNote"])

            for method in ("field_type", "field_enumeration", "field_integer"):
                route = rows[(package_id, prefix + f"ManagedValueReader::{method}")]
                self.assertEqual(route["resolution"], "mapped")
                self.assertTrue(
                    "Option" in route["mappingNote"] or "None" in route["mappingNote"]
                )

            for method in ("kind", "with_field_bytes"):
                route = rows[(package_id, prefix + f"ManagedValueReader::{method}")]
                self.assertEqual(route["resolution"], "mapped")
                self.assertTrue(route["mappingNote"])

            managed_value = rows[(package_id, prefix + "ManagedFieldValue")]
            self.assertEqual(managed_value["resolution"], "mapped")
            self.assertEqual(
                managed_value["mappingMode"],
                "field-item-plus-raw-ttlv-type-and-getter",
            )
            for method in ("item_type", "integer", "enumeration", "bytes"):
                route = rows[(package_id, prefix + f"ManagedFieldValue::{method}")]
                self.assertEqual(route["resolution"], "mapped")

            selector = rows[(package_id, prefix + "ManagedValueKind")]
            reader_type = rows[(package_id, prefix + "ManagedValueReader")]
            self.assertEqual(selector["mappingMode"], "wrapper-kind-and-context")
            self.assertEqual(reader_type["mappingMode"], "kind-selected-wrapper-family")

    def test_query_raw_enumeration_maps_to_existing_typed_query_routes(self) -> None:
        rows = {
            (row["packageId"], row["exportName"]): row
            for row in self.audit["functionalRoutes"]
        }
        expected = {
            ("rust-kmipkit", "kmipkit::protocol::QueryResponseField::raw_enumeration"),
            ("rust-kmipkit-protocol", "kmipkit_protocol::QueryResponseField::raw_enumeration"),
        }
        self.assertEqual(
            {key for key, row in rows.items() if key in expected and row["resolution"] == "mapped"},
            expected,
        )
        for key in expected:
            row = rows[key]
            self.assertEqual(
                row["languageMappings"]["c"]["name"],
                "kmipkit_client_batch_item_response_query_field_at (out_kind plus out_enumeration)",
            )
            self.assertEqual(
                row["languageMappings"]["java"]["name"],
                "org.kmipkit.extensions.QueryResponseField.enumeration",
            )
            self.assertEqual(
                row["languageMappings"]["python"]["name"],
                "kmipkit.client.QueryField.enumeration",
            )

    def test_raw_ttlv_composition_covers_all_managed_field_value_kinds(self) -> None:
        rows = {
            (row["packageId"], row["exportName"]): row
            for row in self.audit["functionalRoutes"]
        }
        keys = {
            ("rust-kmipkit", "kmipkit::protocol::ManagedFieldValue"),
            ("rust-kmipkit-protocol", "kmipkit_protocol::ManagedFieldValue"),
            ("rust-kmipkit", "kmipkit::protocol::ManagedValueReader::read_field"),
        }
        expected_routes = {
            "c": {
                "kmipkit_managed_value_field_item_read",
                "kmipkit_ttlv_item_view_from_item",
                "kmipkit_ttlv_item_view_value",
                "kmipkit_ttlv_value_view_type",
                "kmipkit_ttlv_value_view_structure",
                "kmipkit_ttlv_value_view_integer",
                "kmipkit_ttlv_value_view_long_integer",
                "kmipkit_ttlv_value_view_enumeration",
                "kmipkit_ttlv_value_view_boolean",
                "kmipkit_ttlv_value_view_date_time",
                "kmipkit_ttlv_value_view_interval",
                "kmipkit_ttlv_value_view_date_time_extended",
                "kmipkit_ttlv_value_view_byte_length",
                "kmipkit_ttlv_value_view_byte_at",
            },
            "java": {
                "org.kmipkit.ttlv.ManagedObject.fieldItem",
                "org.kmipkit.ttlv.TtlvItem.view",
                "org.kmipkit.ttlv.TtlvItemView.value",
                "org.kmipkit.ttlv.TtlvValueView.itemType",
                "org.kmipkit.ttlv.TtlvValueView.structure",
                "org.kmipkit.ttlv.TtlvValueView.integer",
                "org.kmipkit.ttlv.TtlvValueView.longInteger",
                "org.kmipkit.ttlv.TtlvValueView.enumeration",
                "org.kmipkit.ttlv.TtlvValueView.booleanValue",
                "org.kmipkit.ttlv.TtlvValueView.dateTime",
                "org.kmipkit.ttlv.TtlvValueView.interval",
                "org.kmipkit.ttlv.TtlvValueView.dateTimeExtended",
                "org.kmipkit.ttlv.TtlvValueView.bytes",
            },
            "python": {
                "kmipkit.managed_objects.ManagedTtlvItem.field_item",
                "kmipkit.ttlv.ttlv_item_view_from_item",
                "kmipkit.ttlv.ttlv_item_view_value",
                "kmipkit.ttlv.ttlv_value_view_type",
                "kmipkit.ttlv.ttlv_value_view_structure",
                "kmipkit.ttlv.ttlv_value_view_integer",
                "kmipkit.ttlv.ttlv_value_view_long_integer",
                "kmipkit.ttlv.ttlv_value_view_enumeration",
                "kmipkit.ttlv.ttlv_value_view_boolean",
                "kmipkit.ttlv.ttlv_value_view_date_time",
                "kmipkit.ttlv.ttlv_value_view_interval",
                "kmipkit.ttlv.ttlv_value_view_date_time_extended",
                "kmipkit.ttlv.ttlv_value_view_bytes",
            },
        }
        for key in keys:
            row = rows[key]
            self.assertEqual(row["resolution"], "mapped")
            self.assertEqual(
                row["mappingMode"],
                "field-item-plus-raw-ttlv-type-and-getter",
            )
            for language, mapping in row["languageMappings"].items():
                names = {route["name"] for route in mapping["routes"]}
                self.assertTrue(expected_routes[language] <= names)
        for row in self.audit["functionalRoutes"]:
            self.assertNotIn("unresolvedReason", row)

    def test_model_error_routes_reuse_the_approved_model_error_policy(self) -> None:
        rows = self.audit["priorApprovedExclusions"]
        keys = {(row["packageId"], row["exportName"]) for row in rows}
        self.assertEqual(
            keys,
            {
                ("rust-kmipkit", "kmipkit::ttlv::ModelError::UnsupportedValue"),
                ("rust-kmipkit-ttlv", "kmipkit_ttlv::ModelError::UnsupportedValue"),
            },
        )
        for row in rows:
            self.assertEqual(row["resolution"], "excluded")
            self.assertEqual(row["approvalSource"]["status"], "Approved")
            self.assertEqual(row["approvalSource"]["path"], "specs/004-generic-ttlv-model/spec.md")
            source = (ROOT / row["approvalSource"]["path"]).read_text(encoding="utf-8")
            self.assertIn("KMIPKIT-0004-FR-013", source)
            self.assertIn(row["approvalSource"]["evidence"], source)
            self.assertTrue(row["precedent"]["exportName"].endswith("ModelError::RawTagOutOfRange"))
            for evidence in row["evidencePaths"]:
                self.assertTrue((ROOT / evidence).is_file(), evidence)

        manifest = json.loads((ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8"))
        approved_keys = {
            (row["packageId"], row["exportName"])
            for row in manifest["approvedExclusions"]
        }
        self.assertTrue(
            all(
                (row["precedent"]["packageId"], row["precedent"]["exportName"])
                in approved_keys
                for row in rows
            )
        )

    def test_enum_shape_exclusions_name_the_exact_parent_declaration(self) -> None:
        rows = self.audit["enumShapeExclusions"]
        keys = [(row["packageId"], row["exportName"]) for row in rows]
        self.assertEqual(len(keys), len(set(keys)))

        for row in rows:
            with self.subTest(exportName=row["exportName"]):
                self.assertIn(row["artifactKind"], {"rust-enum-variant", "rust-enum-field"})
                self.assertEqual(
                    row["counterpartDeclarationId"],
                    declaration_id(row["packageId"], row["counterpartExportName"]),
                )
                self.assertTrue(row["counterpartExportName"])
                self.assertTrue(row["evidencePaths"])

    def test_trait_exclusions_preserve_exact_identity_and_conditional_bounds(self) -> None:
        rows = self.audit["traitImplementationExclusions"]
        keys = [(row["packageId"], row["exportName"]) for row in rows]
        self.assertEqual(len(keys), len(set(keys)))

        for row in rows:
            with self.subTest(exportName=row["exportName"]):
                self.assertTrue(row["identity"])
                self.assertTrue(row["ownerExportName"])
                self.assertEqual(
                    row["ownerDeclarationId"],
                    declaration_id(row["packageId"], row["ownerExportName"]),
                )
                self.assertIn(row["artifactKind"], {
                    "rust-trait-implementation-member",
                    "rust-conditional-trait-implementation-member",
                })
                if row["artifactKind"] == "rust-conditional-trait-implementation-member":
                    self.assertTrue(row.get("condition"))

    def test_feature_gated_test_support_routes_have_exact_exclusion_evidence(self) -> None:
        rows = [
            row for row in self.audit["exactRouteExclusions"]
            if row["exclusionCategory"] == "test-support"
        ]
        self.assertEqual(len(rows), 2)
        self.assertEqual(
            {row["exportName"] for row in rows},
            {
                "kmipkit_client::test_support",
                "kmipkit_client::test_support::encoded_batch_item_operation",
            },
        )
        for row in rows:
            self.assertEqual(row["resolution"], "excluded")
            self.assertEqual(row["exclusionScope"], "exact-route")
            self.assertTrue(row["reason"])
            self.assertTrue(row["evidence"])
            self.assertTrue(row["evidencePaths"])
            for evidence in row["evidencePaths"]:
                self.assertTrue((ROOT / evidence).is_file(), evidence)

    def test_ownership_transfer_helpers_are_exact_non_capability_exclusions(self) -> None:
        rows = [
            row for row in self.audit["exactRouteExclusions"]
            if row["exclusionCategory"] == "ownership-transfer-helper"
        ]
        self.assertEqual(
            {(row["packageId"], row["exportName"]) for row in rows},
            {
                ("rust-kmipkit", "kmipkit::protocol::ManagedValueReader::into_item"),
                ("rust-kmipkit-protocol", "kmipkit_protocol::ManagedValueReader::into_item"),
            },
        )
        for row in rows:
            self.assertEqual(row["resolution"], "excluded")
            self.assertEqual(row["exclusionScope"], "exact-route")
            self.assertTrue(row["reason"])
            self.assertTrue(row["evidencePaths"])
            for evidence in row["evidencePaths"]:
                self.assertTrue((ROOT / evidence).is_file(), evidence)
        manifest = json.loads((ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8"))
        approved_keys = {
            (row["packageId"], row["exportName"])
            for row in manifest["approvedExclusions"]
        }
        self.assertTrue(
            all(
                (row["precedent"]["packageId"], row["precedent"]["exportName"])
                in approved_keys
                for row in rows
            )
        )

    def test_kind_corrections_are_exact_for_the_two_reported_mismatches(self) -> None:
        corrections = {
            row["exportName"]: (row["manifestKind"], row["observedKind"])
            for row in self.audit["kindCorrections"]
        }
        self.assertEqual(
            corrections,
            {
                "kmipkit_protocol::ManagedValueReader::read_field": (
                    "function",
                    "associated-item",
                ),
                "kmipkit_protocol::ManagedValueReader::read_field_item": (
                    "function",
                    "associated-item",
                ),
            },
        )


if __name__ == "__main__":
    unittest.main()

