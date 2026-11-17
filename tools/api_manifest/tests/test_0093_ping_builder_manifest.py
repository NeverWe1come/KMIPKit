"""Require four-language declarations for the fluent Ping builder."""

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXPECTED_EXPORTS = {
    "rust-kmipkit-client": {
        "kmipkit_client::fluent::PingBuilder",
        "kmipkit_client::fluent::RequestBuilder<'a>::ping",
        "kmipkit_client::fluent::PingBuilder<'a>::build",
        "kmipkit_client::fluent::PingBuilder<'a>::execute",
        "kmipkit_client::fluent::PingBuilder<'a>::to_batch_item",
        "kmipkit_client::fluent::PingBuilder<'a>::validate",
        "kmipkit_client::fluent::PingBuilder<'a>::with_asynchronous_indicator",
        "kmipkit_client::fluent::PingBuilder<'a>::with_options",
    },
    "c-kmipkit": {
        "kmipkit_client_ping_v1",
        "kmipkit_ping_builder_create",
        "kmipkit_ping_builder_set_asynchronous_indicator",
        "kmipkit_ping_builder_validate",
        "kmipkit_ping_builder_build_batch_item",
        "kmipkit_ping_builder_execute_v1",
        "kmipkit_ping_builder_execute_with_options_v1",
    },
    "java-kmipkit": {
        "org.kmipkit.fluent.FluentRequest.ping",
        "org.kmipkit.fluent.PingBuilder",
        "org.kmipkit.fluent.PingBuilder.asynchronousIndicator",
        "org.kmipkit.fluent.PingBuilder.build",
        "org.kmipkit.fluent.PingBuilder.execute",
        "org.kmipkit.fluent.PingBuilder.toBatchItem",
        "org.kmipkit.fluent.PingBuilder.validate",
        "org.kmipkit.fluent.PingBuilder.withOptions",
    },
    "python-kmipkit": {
        "kmipkit.fluent.Request.ping",
        "kmipkit.fluent.PingBuilder",
        "kmipkit.fluent.PingBuilder.asynchronous_indicator",
        "kmipkit.fluent.PingBuilder.build",
        "kmipkit.fluent.PingBuilder.execute",
        "kmipkit.fluent.PingBuilder.to_batch_item",
        "kmipkit.fluent.PingBuilder.validate",
        "kmipkit.fluent.PingBuilder.with_options",
    },
}
EXPECTED_C_FUNCTIONS = {
    "kmipkit_client_ping_v1",
    "kmipkit_ping_builder_create",
    "kmipkit_ping_builder_set_asynchronous_indicator",
    "kmipkit_ping_builder_validate",
    "kmipkit_ping_builder_build_batch_item",
    "kmipkit_ping_builder_execute_v1",
    "kmipkit_ping_builder_execute_with_options_v1",
}


class PingBuilderManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(
            (ROOT / "specification/api/public-api.json").read_text(encoding="utf-8")
        )

    def test_ping_builder_surface_is_declared_in_all_four_languages(self) -> None:
        rows = {
            package: {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row["packageId"] == package
            }
            for package in EXPECTED_EXPORTS
        }
        for package, exports in EXPECTED_EXPORTS.items():
            with self.subTest(package=package):
                self.assertTrue(exports <= rows[package].keys())
                for export in exports:
                    declaration = rows[package][export]
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertEqual(
                        {"rust", "c", "java", "python"},
                        declaration["languageMappings"].keys(),
                    )

    def test_ping_builder_c_abi_functions_are_in_the_catalog(self) -> None:
        symbols = {row["c"]["symbol"] for row in self.manifest["functions"]}
        self.assertTrue(EXPECTED_C_FUNCTIONS <= symbols)

    def test_ping_builder_opaque_type_has_library_release(self) -> None:
        declarations = {row["id"]: row for row in self.manifest["types"]}
        self.assertIn("PingBuilder", declarations)
        self.assertEqual(
            "kmipkit_ping_builder_release",
            declarations["PingBuilder"]["c"]["releaseFunction"],
        )
        public_declaration = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
            and row["exportName"] == "kmipkit_ping_builder_t"
        )
        self.assertEqual("type", public_declaration["kind"])


if __name__ == "__main__":
    unittest.main()
