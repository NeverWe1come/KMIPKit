"""Focused structure checks for reviewed Python export-audit resolutions."""

from __future__ import annotations

import ast
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
PYTHON_PACKAGE = ROOT / "bindings" / "python" / "src" / "kmipkit"
RESOLUTIONS = (
    ROOT
    / "specs"
    / "053-api-parity-facade"
    / "candidate-resolutions"
    / "python-export-audit.json"
)


class PythonExportAuditResolutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.document = json.loads(RESOLUTIONS.read_text(encoding="utf-8"))

    def test_resolutions_cover_each_audited_python_export_once(self) -> None:
        self.assertEqual(
            {"formatVersion", "specId", "packageId", "auditSummary", "mappings", "exclusions",
             "unresolved", "staleMappings", "kindCorrections"},
            set(self.document),
        )
        audit = self.document["auditSummary"]
        self.assertEqual(282, audit["missingFromManifest"])
        self.assertEqual(
            {"python-export": 246, "field": 27, "type": 5, "module": 4},
            audit["missingByKind"],
        )

        resolved = [
            *self.document["mappings"],
            *self.document["exclusions"],
            *self.document["unresolved"],
        ]
        identities = [(row["packageId"], row["name"]) for row in resolved]
        # The corrected KeyWrappingData path is an additional route for a stale manifest row.
        self.assertEqual(283, len(identities))
        self.assertEqual(283, len(set(identities)))
        self.assertIn(
            ("python-kmipkit", "kmipkit.managed_objects.KeyWrappingData.from_fields"),
            identities,
        )
        self.assertTrue(all(package == "python-kmipkit" for package, _ in identities))

        required = {"packageId", "name", "capabilityId", "requirementIds",
                    "evidencePaths", "approval", "observedKind"}
        for row in resolved:
            self.assertTrue(required <= set(row), row["name"])
            self.assertTrue(row["capabilityId"], row["name"])
            self.assertTrue(row["requirementIds"], row["name"])
            self.assertTrue(row["evidencePaths"], row["name"])
            self.assertEqual("Approved", row["approval"]["status"], row["name"])
            for evidence_path in row["evidencePaths"]:
                self.assertTrue((ROOT / evidence_path).exists(), evidence_path)
        self.assertEqual([], self.document["unresolved"])

        expected_root_alias_members = {
            f"kmipkit.ProfileValidationDetails.{member}"
            for member in (
                "__dataclass_fields__", "__dataclass_params__", "__delattr__",
                "__eq__", "__hash__", "__init__", "__match_args__", "__repr__",
                "__setattr__", "batch_item_index", "error_code", "profile_id",
                "requirement_id", "source_clause_id", "source_gap_id",
                "source_gap_locator",
            )
        }
        self.assertEqual(
            expected_root_alias_members,
            {row["name"] for row in self.document["exclusions"]
             if row["name"].startswith("kmipkit.ProfileValidationDetails.")},
        )

    def test_mapping_rows_name_exact_four_language_routes(self) -> None:
        package_ids = {
            "rust": {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client", "rust-kmipkit-ttlv"},
            "c": {"c-kmipkit"},
            "java": {"java-kmipkit"},
            "python": {"python-kmipkit"},
        }
        for row in self.document["mappings"]:
            self.assertEqual({"rust", "c", "java", "python"}, set(row["languageMappings"]))
            for language, mapping in row["languageMappings"].items():
                self.assertIn(mapping["packageId"], package_ids[language], row["name"])
                self.assertTrue(mapping["name"], row["name"])
            self.assertEqual(row["name"], row["languageMappings"]["python"]["name"])

    def test_existing_public_python_exports_match_the_manifest_routes(self) -> None:
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json")
            .read_text(encoding="utf-8")
        )
        route_data = json.loads(
            (ROOT / "specs" / "053-api-parity-facade" / "additional-api-routes.json")
            .read_text(encoding="utf-8")
        )
        declarations = {
            (row["exportName"], row["capabilityId"]): row
            for row in manifest["publicDeclarations"]
            if row.get("packageId") == "python-kmipkit"
            if row.get("declarationState") == "verified"
            and isinstance(row.get("languageMappings"), dict)
            and isinstance(row["languageMappings"].get("python"), dict)
        }
        corrections = {
            (row["declarationId"], row["language"]): row
            for row in route_data["languageMappingCorrections"]
        }
        for row in self.document["mappings"]:
            declaration = declarations.get((row["name"], row["capabilityId"]))
            if declaration is None:
                continue
            routes = row.get("manifestMapping", row["languageMappings"])
            for language in ("rust", "c", "java", "python"):
                expected = declaration["languageMappings"][language]
                correction = corrections.get((declaration["declarationId"], language))
                if correction is not None:
                    expected = correction["to"]
                self.assertEqual(expected, routes[language], f"{row['name']} {language}")

    def test_client_batch_validation_report_uses_the_public_rust_facade(self) -> None:
        row = next(
            row
            for row in self.document["mappings"]
            if row["name"] == "kmipkit.extensions.ClientBatchItem.validation_report"
        )
        self.assertEqual(
            {"packageId": "rust-kmipkit", "name": "kmipkit::ClientRequest::validation_report"},
            row["languageMappings"]["rust"],
        )

    def test_builder_close_maps_the_secret_cleanup_lifecycle_across_languages(self) -> None:
        row = next(
            row for row in self.document["mappings"]
            if row["name"]
            == "kmipkit.cryptographic_parameters.CryptographicParametersBuilder.close"
        )
        self.assertEqual("typed_cryptographic_parameters", row["capabilityId"])
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-protocol",
                    "name": "kmipkit_protocol::CryptographicParametersBuilder",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_cryptographic_parameters_release",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.protocol.CryptographicParameters.Builder.close",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.cryptographic_parameters.CryptographicParametersBuilder.close",
                },
            },
            row["languageMappings"],
        )

    def test_standard_attribute_constants_match_the_generated_c_catalog(self) -> None:
        macro_names = set(
            re.findall(
                r"^#define (KMIPKIT_STANDARD_ATTRIBUTE_[A-Z0-9_]+) ",
                (ROOT / "bindings" / "c" / "include" / "kmipkit_standard_attributes_generated.h")
                .read_text(encoding="utf-8"),
                re.MULTILINE,
            )
        )
        for row in self.document["mappings"]:
            match = re.fullmatch(
                r"kmipkit(?:\.typed_attributes)?\.StandardAttribute\.([A-Z][A-Z0-9_]*)",
                row["name"],
            )
            if match is None:
                continue
            macro = f"KMIPKIT_STANDARD_ATTRIBUTE_{match.group(1)}"
            self.assertIn(macro, macro_names, row["name"])
            self.assertEqual(macro, row["languageMappings"]["c"]["name"], row["name"])
        self.assertEqual(62, sum(
            re.fullmatch(r"kmipkit(?:\.typed_attributes)?\.StandardAttribute\.[A-Z][A-Z0-9_]*", row["name"])
            is not None
            for row in self.document["mappings"]
        ))

    def test_exclusions_are_exact_namespaced_runtime_or_model_artifacts(self) -> None:
        allowed_kinds = {
            "module-namespace",
            "dataclass-metadata",
            "dataclass-generated-method",
            "private-native-constructor",
            "local-context-manager",
            "local-repr",
            "exception-runtime-member",
            "python-runtime-member",
            "catalog-metadata",
            "local-builder-lifecycle",
            "catalog-container",
            "python-reexport-alias",
        }
        for row in self.document["exclusions"]:
            self.assertEqual(row["name"], row["exclusion"]["exportName"])
            self.assertIn(row["exclusion"]["artifactKind"], allowed_kinds, row["name"])
            self.assertTrue(row["exclusion"]["reason"], row["name"])
            self.assertNotIn("*", row["name"])

    def test_stale_from_fields_routes_and_kind_correction_are_exact(self) -> None:
        expected_stale = {
            "KeyBlock.from_fields": "kmipkit.managed_objects.KeyBlock.from_fields",
            "KeyMaterial.from_fields": "kmipkit.managed_objects.KeyMaterial.from_fields",
            "KeyValue.from_fields": "kmipkit.managed_objects.KeyValue.from_fields",
            "KeyWrappingData.from_fields": "kmipkit.managed_objects.KeyWrappingData.from_fields",
            "ManagedObject.certificate": "kmipkit.managed_objects.ManagedObject.certificate",
            "ManagedObject.certificate_request": "kmipkit.managed_objects.ManagedObject.certificate_request",
            "ManagedObject.from_fields": "kmipkit.managed_objects.ManagedObject.from_fields",
            "ManagedObject.opaque_object": "kmipkit.managed_objects.ManagedObject.opaque_object",
            "ManagedObject.pgp_key": "kmipkit.managed_objects.ManagedObject.pgp_key",
            "ManagedObject.private_key": "kmipkit.managed_objects.ManagedObject.private_key",
            "ManagedObject.public_key": "kmipkit.managed_objects.ManagedObject.public_key",
            "ManagedObject.secret_data": "kmipkit.managed_objects.ManagedObject.secret_data",
            "ManagedObject.split_key": "kmipkit.managed_objects.ManagedObject.split_key",
            "ManagedObject.symmetric_key": "kmipkit.managed_objects.ManagedObject.symmetric_key",
        }
        self.assertEqual(14, len(self.document["staleMappings"]))
        self.assertEqual(expected_stale, {row["name"]: row["correctedName"] for row in self.document["staleMappings"]})
        mappings_by_name = {row["name"]: row for row in self.document["mappings"]}
        for row in self.document["staleMappings"]:
            self.assertEqual("python-kmipkit", row["packageId"])
            self.assertEqual("Approved", row["approval"]["status"])
            self.assertIn(row["correctedName"], mappings_by_name)
            module_name, class_name, member_name = row["correctedName"].removeprefix("kmipkit.").split(".")
            module_path = PYTHON_PACKAGE / f"{module_name}.py"
            module = ast.parse(module_path.read_text(encoding="utf-8"))
            class_node = next(
                node for node in module.body
                if isinstance(node, ast.ClassDef) and node.name == class_name
            )
            self.assertTrue(
                any(
                    isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
                    and node.name == member_name
                    for node in class_node.body
                ),
                row["correctedName"],
            )

        self.assertEqual(1, len(self.document["kindCorrections"]))
        correction = self.document["kindCorrections"][0]
        self.assertEqual("kmipkit.client.QueryResponse.field_count", correction["name"])
        self.assertEqual("field", correction["fromKind"])
        self.assertEqual("python-export", correction["toKind"])

    def test_composed_routes_have_exact_manifest_mappings_and_preserved_evidence(self) -> None:
        expected_manifest = {
            "kmipkit.managed_objects.KeyWrappingData.from_fields": {
                "rust": ("rust-kmipkit", "kmipkit::protocol::KeyWrappingData::try_from_item"),
                "c": ("c-kmipkit", "kmipkit_key_wrapping_data_from_fields"),
                "java": ("java-kmipkit", "org.kmipkit.ttlv.KeyWrappingData.fromFields"),
                "python": ("python-kmipkit", "kmipkit.managed_objects.KeyWrappingData.from_fields"),
            },
            "kmipkit.managed_objects.ManagedTtlvItem": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::ManagedValueReader::read_field"),
                "c": ("c-kmipkit", "kmipkit_managed_value_field_read"),
                "java": ("java-kmipkit", "org.kmipkit.ttlv.ManagedObject.fieldItem"),
                "python": ("python-kmipkit", "kmipkit.managed_objects.ManagedTtlvItem"),
            },
            "kmipkit.ttlv.ttlv_value_view_bytes": {
                "rust": ("rust-kmipkit", "kmipkit::ttlv::ValueView"),
                "c": ("c-kmipkit", "kmipkit_ttlv_value_view_byte_at"),
                "java": ("java-kmipkit", "org.kmipkit.ttlv.TtlvValueView.byteAt"),
                "python": ("python-kmipkit", "kmipkit.ttlv.ttlv_value_view_bytes"),
            },
            "kmipkit.typed_attributes.TypedAttribute.attribute": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::Attribute<D>::as_item"),
                "c": ("c-kmipkit", "kmipkit_ttlv_item_view_tag"),
                "java": ("java-kmipkit", "org.kmipkit.ttlv.TypedAttribute.asItem"),
                "python": ("python-kmipkit", "kmipkit.typed_attributes.TypedAttribute.attribute"),
            },
        }
        mappings_by_name = {row["name"]: row for row in self.document["mappings"]}
        for name, routes in expected_manifest.items():
            row = mappings_by_name[name]
            actual = {
                language: (route["packageId"], route["name"])
                for language, route in row["manifestMapping"].items()
            }
            self.assertEqual(routes, actual, name)
            self.assertEqual({"rust", "c", "java", "python"}, set(row["manifestMapping"]))
            self.assertTrue(row["compositionEvidence"], name)

        self.assertEqual(
            "kmipkit_protocol::{Item::new + Structure::try_push, KeyWrappingData::try_from_item}",
            mappings_by_name["kmipkit.managed_objects.KeyWrappingData.from_fields"]
            ["compositionEvidence"]["rust"]["name"],
        )
        self.assertEqual(
            "kmipkit_ttlv_item_t with kmipkit_managed_value_field_read context",
            mappings_by_name["kmipkit.managed_objects.ManagedTtlvItem"]
            ["compositionEvidence"]["c"]["name"],
        )
        self.assertEqual(
            "kmipkit_ttlv_value_view_byte_length + kmipkit_ttlv_value_view_byte_at",
            mappings_by_name["kmipkit.ttlv.ttlv_value_view_bytes"]
            ["compositionEvidence"]["c"]["name"],
        )
        self.assertEqual(
            "kmipkit_protocol::Attribute::as_item().tag",
            mappings_by_name["kmipkit.typed_attributes.TypedAttribute.attribute"]
            ["compositionEvidence"]["rust"]["name"],
        )
        self.assertEqual(
            "kmipkit_ttlv_item_view_tag + kmipkit_ttlv_tag_value and KMIPKIT_STANDARD_ATTRIBUTE_*",
            mappings_by_name["kmipkit.typed_attributes.TypedAttribute.attribute"]
            ["compositionEvidence"]["c"]["name"],
        )


if __name__ == "__main__":
    unittest.main()
