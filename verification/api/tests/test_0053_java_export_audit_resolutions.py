"""Guard the reviewable KMIPKIT-0053 Java export audit resolutions."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
RESOLUTIONS = ROOT / "specs" / "053-api-parity-facade" / "candidate-resolutions" / "java-export-audit.json"
EXPECTED_EXPORT_COUNT = 170
EXPECTED_EXPORT_NAMES_SHA256 = "52b606fa78f969ab38381a5e9cffd17f6c57b6cccd2af1aca155072638c0bea5"
EXPECTED_STALE_NAMES = {
    "org.kmipkit.ttlv.ManagedTtlvValue.fieldItem",
    "org.kmipkit.ttlv.ManagedTtlvValue.readField",
}
EXPECTED_MANAGED_WRAPPERS = {
    "org.kmipkit.ttlv.ManagedObject",
    "org.kmipkit.ttlv.KeyBlock",
    "org.kmipkit.ttlv.KeyValue",
    "org.kmipkit.ttlv.KeyMaterial",
    "org.kmipkit.ttlv.KeyWrappingData",
}


class JavaExportAuditResolutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = json.loads(RESOLUTIONS.read_text(encoding="utf-8"))

    def test_resolution_inventory_covers_the_exact_audit_delta(self) -> None:
        self.assertEqual(1, self.document["formatVersion"])
        self.assertEqual("java-kmipkit", self.document["packageId"])
        self.assertEqual(EXPECTED_EXPORT_COUNT, self.document["auditSummary"]["missingFromManifest"])
        self.assertEqual(
            {"mapped": 119, "excluded": 51, "unresolved": 0},
            self.document["auditSummary"]["resolutionCounts"],
        )

        rows = [
            *self.document["mappings"],
            *self.document["exclusions"],
            *self.document["unresolved"],
        ]
        names = [row["name"] for row in rows]
        self.assertEqual(EXPECTED_EXPORT_COUNT, len(names))
        self.assertEqual(EXPECTED_EXPORT_COUNT, len(set(names)))
        digest = hashlib.sha256(("\n".join(sorted(names)) + "\n").encode()).hexdigest()
        self.assertEqual(EXPECTED_EXPORT_NAMES_SHA256, digest)
        self.assertEqual(EXPECTED_EXPORT_NAMES_SHA256, self.document["auditSummary"]["exportNamesSha256"])
        self.assertTrue(all(row["packageId"] == "java-kmipkit" for row in rows))
        self.assertTrue(all(row["capabilityId"] and row["requirementIds"] for row in rows))
        self.assertTrue(all(row["evidencePaths"] and row["approval"]["status"] == "Approved" for row in rows))

    def test_each_mapping_has_four_language_routes_and_existing_evidence(self) -> None:
        c_header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(encoding="utf-8")
        default_packages = {
            "rust": "rust-kmipkit-protocol",
            "c": "c-kmipkit",
            "python": "python-kmipkit",
        }
        package_ids = {
            "rust": {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client", "rust-kmipkit-ttlv"},
            "c": {"c-kmipkit"},
            "python": {"python-kmipkit"},
        }
        for row in self.document["mappings"]:
            with self.subTest(name=row["name"]):
                self.assertEqual({"c", "rust", "python"}, set(row["equivalents"]))
                for language, route in row["equivalents"].items():
                    if isinstance(route, str):
                        route = {"packageId": default_packages[language], "name": route}
                    self.assertEqual({"packageId", "name"}, set(route))
                    self.assertIn(route["packageId"], package_ids[language])
                    if language == "c":
                        self.assertRegex(route["name"], r"^[A-Za-z_][A-Za-z0-9_]*$")
                        self.assertIn(route["name"], c_header)
                    elif language == "rust":
                        self.assertRegex(
                            route["name"],
                            r"^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_<>]*)*$",
                        )
                    else:
                        self.assertRegex(route["name"], r"^[A-Za-z_][A-Za-z0-9_.]*$")
                self.assertTrue(row["capabilityId"])
                self.assertTrue(row["requirementIds"])
                self.assertEqual("Approved", row["approval"]["status"])
                self.assertEqual("specs/053-api-parity-facade/approval-record.md", row["approval"]["path"])
                self._assert_evidence_exists(row["evidencePaths"])

    def test_composed_routes_have_exact_representatives_and_separate_notes(self) -> None:
        mappings = {row["name"]: row for row in self.document["mappings"]}
        query = mappings["org.kmipkit.extensions.QueryResponse.operations"]
        self.assertEqual(
            "kmipkit_client_batch_item_response_query_field_at",
            query["equivalents"]["c"],
        )
        self.assertIn("query_field_count", query["compositionNotes"]["c"])

        bytes_view = mappings["org.kmipkit.ttlv.TtlvValueView.bytes"]
        self.assertEqual("kmipkit_ttlv_value_view_byte_at", bytes_view["equivalents"]["c"])
        self.assertEqual(
            {"packageId": "rust-kmipkit-ttlv", "name": "kmipkit_ttlv::Item::with_value"},
            bytes_view["equivalents"]["rust"],
        )
        self.assertIn("byte_length", bytes_view["compositionNotes"]["c"])

        attribute = mappings["org.kmipkit.ttlv.TypedAttribute.attribute"]
        self.assertEqual("kmipkit_ttlv_item_view_tag", attribute["equivalents"]["c"])
        self.assertEqual(
            {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::attribute::Attribute<D>::as_item"},
            attribute["equivalents"]["rust"],
        )
        self.assertIn("KMIPKIT_STANDARD_ATTRIBUTE_*", attribute["compositionNotes"]["c"])

    def test_java_lifecycle_and_typed_attribute_identity_have_supported_equivalents(self) -> None:
        mappings = {row["name"]: row for row in self.document["mappings"]}
        builder_close = mappings["org.kmipkit.protocol.CryptographicParameters.Builder.close"]
        self.assertEqual("kmipkit_cryptographic_parameters_release", builder_close["equivalents"]["c"])
        self.assertEqual(
            {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::CryptographicParametersBuilder",
            },
            builder_close["equivalents"]["rust"],
        )
        self.assertEqual(
            "kmipkit.cryptographic_parameters.CryptographicParametersBuilder.close",
            builder_close["equivalents"]["python"],
        )
        self.assertIn("KMIPKIT-0053-FR-010", builder_close["requirementIds"])

        attribute = mappings["org.kmipkit.ttlv.TypedAttribute.attribute"]
        self.assertIn("kmipkit_ttlv_item_view_tag", attribute["equivalents"]["c"])
        self.assertIn("KMIPKIT_STANDARD_ATTRIBUTE_*", attribute["compositionNotes"]["c"])
        self.assertIn("Item::tag()", attribute["compositionNotes"]["rust"])
        self.assertEqual(
            "kmipkit.typed_attributes.TypedAttribute.attribute",
            attribute["equivalents"]["python"],
        )
        self._assert_evidence_exists(builder_close["evidencePaths"])
        self._assert_evidence_exists(attribute["evidencePaths"])

    def test_batch_validation_accessor_maps_to_public_cross_language_routes(self) -> None:
        row = next(
            row
            for row in self.document["mappings"]
            if row["name"] == "org.kmipkit.extensions.ClientBatchItem.validationReport"
        )
        self.assertEqual(
            {"packageId": "rust-kmipkit", "name": "kmipkit::ClientRequest::validation_report"},
            row["equivalents"]["rust"],
        )
        self.assertEqual(
            "kmipkit.extensions.ClientBatchItem.validation_report",
            row["equivalents"]["python"],
        )

    def test_batch_validation_python_route_change_has_an_exact_manifest_correction(self) -> None:
        routes = json.loads(
            (ROOT / "specs" / "053-api-parity-facade" / "additional-api-routes.json")
            .read_text(encoding="utf-8")
        )
        matches = [
            correction
            for correction in routes["languageMappingCorrections"]
            if correction["declarationId"] == "KMIPKIT-0053:export:ab7075af25af9a9277bb4b26"
            and correction["language"] == "python"
        ]
        self.assertEqual(1, len(matches))
        correction = matches[0]
        self.assertEqual(
            {"packageId": "python-kmipkit", "name": "kmipkit.validation.validate_request"},
            correction["from"],
        )
        self.assertEqual(
            {"packageId": "python-kmipkit", "name": "kmipkit.extensions.ClientBatchItem.validation_report"},
            correction["to"],
        )
        self.assertEqual("request_validation", correction["capabilityId"])
        self.assertIn("KMIPKIT-0053-FR-009", correction["requirementIds"])

    def test_typed_query_response_uses_the_public_rust_client_route(self) -> None:
        row = next(
            row
            for row in self.document["mappings"]
            if row["name"] == "org.kmipkit.extensions.ClientBatchItemResult.typedQueryResponse"
        )
        self.assertEqual(
            {"packageId": "rust-kmipkit", "name": "kmipkit::ClientResponseView::query"},
            row["equivalents"]["rust"],
        )

    def test_audited_routes_match_existing_exact_manifest_counterparts(self) -> None:
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json")
            .read_text(encoding="utf-8")
        )
        declarations = {
            (row["packageId"], row["exportName"], row["capabilityId"]): row
            for row in manifest["publicDeclarations"]
            if row.get("declarationState") == "verified"
        }
        default_packages = {
            "rust": "rust-kmipkit-protocol",
            "c": "c-kmipkit",
            "python": "python-kmipkit",
        }
        route_data = json.loads(
            (ROOT / "specs" / "053-api-parity-facade" / "additional-api-routes.json")
            .read_text(encoding="utf-8")
        )
        corrections = {
            (row["declarationId"], row["language"]): row
            for row in route_data["languageMappingCorrections"]
        }
        for row in self.document["mappings"]:
            declaration = declarations.get(
                (row["packageId"], row["name"], row["capabilityId"])
            )
            if declaration is None:
                continue
            for language in ("rust", "c", "python"):
                expected = declaration["languageMappings"][language]
                correction = corrections.get((declaration["declarationId"], language))
                if correction is not None:
                    expected = correction["to"]
                audited = row["equivalents"][language]
                if isinstance(audited, str):
                    audited = {"packageId": default_packages[language], "name": audited}
                self.assertEqual(
                    expected,
                    audited,
                    f"{row['name']} {language}",
                )

    def test_exclusions_are_exact_internal_bridges_or_nonfunctional_language_helpers(self) -> None:
        names = {row["name"] for row in self.document["exclusions"]}
        bridge_names = {name for name in names if name.startswith("org.kmipkit.NativeExtensionRegistry.")}
        self.assertEqual(34, len(bridge_names))
        self.assertEqual(51, len(names))
        expected_non_bridge = {
            "org.kmipkit.credentials.Authentication.nativeHandleOwner",
            "org.kmipkit.extensions.QueryResponseFieldKind.valueOf",
            "org.kmipkit.extensions.QueryResponseFieldKind.values",
            "org.kmipkit.ttlv.StandardAttribute.valueOf",
            "org.kmipkit.ttlv.StandardAttribute.values",
            "org.kmipkit.extensions.ValidationIssue.toString",
            "org.kmipkit.protocol.CryptographicParameters.toString",
            "org.kmipkit.ttlv.KeyBlock.toString",
            "org.kmipkit.ttlv.KeyMaterial.toString",
            "org.kmipkit.ttlv.KeyValue.toString",
            "org.kmipkit.ttlv.KeyWrappingData.toString",
            "org.kmipkit.ttlv.ManagedObject.toString",
            "org.kmipkit.ttlv.TypedAttributeTagMismatchException.<init>",
            "org.kmipkit.ttlv.TypedAttributeValueTypeMismatchException.<init>",
            "org.kmipkit.ttlv.StandardAttribute",
            "org.kmipkit.ttlv.StandardAttribute.sourceMultiplicity",
            "org.kmipkit.ttlv.StandardAttribute.valueKind",
        }
        self.assertEqual(expected_non_bridge, names - bridge_names)
        for row in self.document["exclusions"]:
            with self.subTest(name=row["name"]):
                self.assertNotIn("*", row["name"])
                self.assertTrue(row["reason"])
                self.assertTrue(row["evidencePaths"])
                self.assertEqual("Approved", row["approval"]["status"])
                self.assertNotIn("requiredFollowUp", row)
                self._assert_evidence_exists(row["evidencePaths"])
        standard_attribute = next(
            row for row in self.document["exclusions"]
            if row["name"] == "org.kmipkit.ttlv.StandardAttribute"
        )
        self.assertIn("62", standard_attribute["reason"])
        self.assertIn("KMIPKIT_STANDARD_ATTRIBUTE_*", standard_attribute["reason"])
        self.assertIn("crates/kmipkit-ffi/tests/support/typed_attribute_0053_tests.rs", standard_attribute["evidencePaths"])

    def test_all_exports_are_resolved_without_open_followups(self) -> None:
        self.assertEqual([], self.document["unresolved"])
        for row in [*self.document["mappings"], *self.document["exclusions"]]:
            with self.subTest(name=row["name"]):
                self.assertNotIn("requiredFollowUp", row)

    def test_stale_package_private_base_mappings_point_to_public_wrappers(self) -> None:
        stale = {row["oldName"]: row for row in self.document["staleMappings"]}
        self.assertEqual(EXPECTED_STALE_NAMES, set(stale))
        for old_name, row in stale.items():
            with self.subTest(name=old_name):
                self.assertEqual(old_name, row["name"])
                self.assertTrue(row["capabilityId"])
                self.assertTrue(row["requirementIds"])
                targets = row["observedPublicTargets"]
                self.assertEqual(EXPECTED_MANAGED_WRAPPERS, {target["owner"] for target in targets})
                member = old_name.rsplit(".", 1)[1]
                self.assertTrue(all(target["name"].endswith(f".{member}") for target in targets))
                self.assertTrue(row["reason"])
                self.assertEqual("Approved", row["approval"]["status"])
                self._assert_evidence_exists(row["evidencePaths"])

    @staticmethod
    def _assert_evidence_exists(paths: list[str]) -> None:
        for path in paths:
            with unittest.TestCase().subTest(evidence=path):
                if not (ROOT / path).is_file():
                    raise AssertionError(f"evidence path does not exist: {path}")


if __name__ == "__main__":
    unittest.main()
