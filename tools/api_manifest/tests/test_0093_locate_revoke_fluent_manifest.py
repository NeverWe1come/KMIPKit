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


class FluentLocateRevokeManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
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
            "org.kmipkit.fluent.FluentRequest.revoke": {
                "public org.kmipkit.fluent.RevokeBuilder revoke(long)",
                "public org.kmipkit.fluent.RevokeBuilder revoke(long, java.lang.String)",
            },
            "org.kmipkit.fluent.LocateBuilder.execute": {
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits)",
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.RequestOptions)",
            },
            "org.kmipkit.fluent.RevokeBuilder.execute": {
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits)",
                "public org.kmipkit.extensions.ClientBatchResponse execute(org.kmipkit.ttlv.CodecLimits, org.kmipkit.extensions.RequestOptions)",
            },
        }
        for export_name, signatures in expected_signatures.items():
            with self.subTest(export_name=export_name):
                row = declarations.get(("java-kmipkit", export_name))
                self.assertIsNotNone(row, export_name)
                self.assertEqual(signatures, set(row.get("javaSignatures", [])))


if __name__ == "__main__":
    unittest.main()
