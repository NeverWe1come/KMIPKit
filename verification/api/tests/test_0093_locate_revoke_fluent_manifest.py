"""Require the fluent Locate and Revoke binding declarations."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXPECTED_FUNCTIONS = {
    "locate_builder_create",
    "locate_builder_set_maximum_items",
    "locate_builder_set_offset_items",
    "locate_builder_set_storage_status_mask",
    "locate_builder_set_object_group_member",
    "locate_builder_set_asynchronous_indicator",
    "locate_builder_validate",
    "locate_builder_build_batch_item",
    "locate_builder_execute_v1",
    "locate_builder_execute_with_options_v1",
    "revoke_builder_create",
    "revoke_builder_set_unique_identifier",
    "revoke_builder_set_revocation_message",
    "revoke_builder_set_compromise_occurrence_date",
    "revoke_builder_set_asynchronous_indicator",
    "revoke_builder_validate",
    "revoke_builder_build_batch_item",
    "revoke_builder_execute_v1",
    "revoke_builder_execute_with_options_v1",
}
EXPECTED_RUST_ROOT_EXPORTS = {
    "kmipkit_client::LocateBuilder",
    "kmipkit_client::LocateBuilder<'a>::attributes",
    "kmipkit_client::LocateBuilder<'a>::build_request",
    "kmipkit_client::LocateBuilder<'a>::execute",
    "kmipkit_client::LocateBuilder<'a>::execute_with_options",
    "kmipkit_client::LocateBuilder<'a>::maximum_items",
    "kmipkit_client::LocateBuilder<'a>::object_group_member",
    "kmipkit_client::LocateBuilder<'a>::offset_items",
    "kmipkit_client::LocateBuilder<'a>::storage_status_mask",
    "kmipkit_client::LocateBuilder<'a>::to_batch_item",
    "kmipkit_client::LocateBuilder<'a>::validate",
    "kmipkit_client::LocateBuilder<'a>::with_asynchronous_indicator",
    "kmipkit_client::LocateBuilder<'a>::with_options",
    "kmipkit_client::RequestBuilder<'a>::locate",
    "kmipkit_client::RequestBuilder<'a>::revoke",
    "kmipkit_client::RevokeBuilder",
    "kmipkit_client::RevokeBuilder<'a>::build_request",
    "kmipkit_client::RevokeBuilder<'a>::compromise_occurrence_date",
    "kmipkit_client::RevokeBuilder<'a>::execute",
    "kmipkit_client::RevokeBuilder<'a>::execute_with_options",
    "kmipkit_client::RevokeBuilder<'a>::revocation_message",
    "kmipkit_client::RevokeBuilder<'a>::to_batch_item",
    "kmipkit_client::RevokeBuilder<'a>::unique_identifier",
    "kmipkit_client::RevokeBuilder<'a>::validate",
    "kmipkit_client::RevokeBuilder<'a>::with_asynchronous_indicator",
    "kmipkit_client::RevokeBuilder<'a>::with_options",
}


class FluentLocateRevokeManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "bindings/shared/public-api.json").read_text(encoding="utf-8")
        )

    def test_builder_functions_are_declared_for_all_bindings(self) -> None:
        rows = [
            row for row in self.manifest["functions"]
            if row["id"] in EXPECTED_FUNCTIONS
        ]
        self.assertEqual(EXPECTED_FUNCTIONS, {row["id"] for row in rows})
        for row in rows:
            with self.subTest(function=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )
                self.assertTrue(row["requirementIds"])

    def test_fluent_types_are_declared_for_all_bindings(self) -> None:
        expected = {"LocateBuilder", "RevokeBuilder"}
        rows = [row for row in self.manifest["types"] if row["id"] in expected]
        self.assertEqual(expected, {row["id"] for row in rows})
        for row in rows:
            with self.subTest(builder=row["id"]):
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    {language for language in ("rust", "c", "java", "python")
                     if row.get(language)},
                )
        locate_declaration = next(
            row for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] == "kmipkit_client::fluent::LocateBuilder"
        )
        self.assertEqual(
            "kmipkit_locate_builder_t",
            locate_declaration["languageMappings"]["c"]["name"],
        )

    def test_all_root_reexports_are_declared_with_four_language_mappings(self) -> None:
        rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] in EXPECTED_RUST_ROOT_EXPORTS
        }
        self.assertEqual(EXPECTED_RUST_ROOT_EXPORTS, set(rows))
        for export_name, row in rows.items():
            with self.subTest(export_name=export_name):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("fluent_locate_revoke_parity", row["capabilityId"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )
                self.assertEqual(export_name, row["languageMappings"]["rust"]["name"])

    def test_binding_aliases_and_lifecycle_methods_are_declared(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        expected_java = {
            "org.kmipkit.fluent.LocateBuilder.toBatchItem",
            "org.kmipkit.fluent.LocateBuilder.withOptions",
            "org.kmipkit.fluent.RevokeBuilder.toBatchItem",
            "org.kmipkit.fluent.RevokeBuilder.withOptions",
        }
        expected_python = {
            "kmipkit.fluent.LocateBuilder.__init__",
            "kmipkit.fluent.LocateBuilder.__enter__",
            "kmipkit.fluent.LocateBuilder.__exit__",
            "kmipkit.fluent.LocateBuilder.build",
            "kmipkit.fluent.LocateBuilder.with_options",
            "kmipkit.fluent.RevokeBuilder.__init__",
            "kmipkit.fluent.RevokeBuilder.build",
            "kmipkit.fluent.RevokeBuilder.with_options",
        }
        for export_name in expected_java:
            with self.subTest(export_name=export_name):
                self.assertTrue(("java-kmipkit", export_name) in declarations, export_name)
        for export_name in expected_python:
            with self.subTest(export_name=export_name):
                self.assertTrue(("python-kmipkit", export_name) in declarations, export_name)

        self.assertNotIn(
            ("java-kmipkit", "org.kmipkit.fluent.RevokeBuilder.close"),
            declarations,
        )
        self.assertNotIn(
            ("python-kmipkit", "kmipkit.fluent.RevokeBuilder.close"),
            declarations,
        )

    def test_java_overload_signatures_are_complete(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        expected_signatures = {
            "org.kmipkit.fluent.FluentRequest.locate": {
                "public org.kmipkit.fluent.LocateBuilder locate(org.kmipkit.extensions.AttributeSet)",
            },
            "org.kmipkit.fluent.FluentRequest.revoke": {
                "public org.kmipkit.fluent.RevokeBuilder revoke(long)",
                "public org.kmipkit.fluent.RevokeBuilder revoke(long, java.lang.String)",
            },
            "org.kmipkit.fluent.LocateBuilder.execute": {
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits) throws org.kmipkit.ClientOperationException",
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.RequestOptions) throws org.kmipkit.ClientOperationException",
            },
            "org.kmipkit.fluent.RevokeBuilder.execute": {
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits) throws org.kmipkit.ClientOperationException",
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.RequestOptions) throws org.kmipkit.ClientOperationException",
            },
        }
        for export_name, signatures in expected_signatures.items():
            with self.subTest(export_name=export_name):
                row = declarations.get(("java-kmipkit", export_name))
                self.assertIsNotNone(row, export_name)
                self.assertEqual(signatures, set(row.get("javaSignatures", [])))


if __name__ == "__main__":
    unittest.main()
