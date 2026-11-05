"""Public four-language contract for KMIP wire encoding selection."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
LANGUAGES = ("rust", "c", "java", "python")


class WireEncodingPublicApiTests(unittest.TestCase):
    def test_wire_encoding_is_a_mapped_ttlv_default_enum(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        encoding = next(record for record in manifest["types"] if record["id"] == "WireEncoding")

        self.assertEqual(encoding["kind"], "enum")
        self.assertEqual(
            [(value["name"], value["value"]) for value in encoding["enumRepresentation"]["values"]],
            [("Ttlv", 0), ("Json", 1), ("Xml", 2)],
        )
        self.assertIn("KMIPKIT-0057-FR-001", encoding["requirementIds"])

    def test_configuration_and_exchange_override_are_declared_in_every_language(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        functions = {record["id"]: record for record in manifest["functions"]}
        expected = {
            "client_configuration_with_wire_encoding",
            "client_configuration_wire_encoding",
            "client_execute_with_options",
        }

        self.assertTrue(expected <= functions.keys())
        for function_id in expected:
            with self.subTest(function=function_id):
                function = functions[function_id]
                self.assertIn("KMIPKIT-0057-FR-005", function["requirementIds"])
                for language in LANGUAGES:
                    self.assertIn(language, function)

    def test_wire_encoding_exports_belong_to_one_four_language_capability(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            record["declarationId"]
            for record in manifest["publicDeclarations"]
        }
        expected = {
            f"{declaration}:{language}"
            for declaration in (
                "WireEncoding",
                "client_configuration_with_wire_encoding",
                "client_configuration_wire_encoding",
                "client_execute_with_options",
            )
            for language in LANGUAGES
        }
        self.assertTrue(expected <= declarations)

        capability = next(
            record for record in manifest["capabilities"]
            if record["capabilityId"] == "wire_encoding_selection"
        )
        self.assertEqual(capability["parityScope"], "four-language")
        self.assertIn("KMIPKIT-0057-FR-005", capability["requirementIds"])

    def test_public_c_encoding_override_has_a_constructible_client_path(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        functions = {record["c"]["symbol"] for record in manifest["functions"]}
        declarations = {
            (record["packageId"], record["exportName"], record["declarationState"])
            for record in manifest["publicDeclarations"]
        }
        required_symbols = {
            "kmipkit_transport_config_create_v1",
            "kmipkit_client_create",
            "kmipkit_client_batch_create",
            "kmipkit_client_batch_add_item",
            "kmipkit_client_batch_item_discover_versions",
            "kmipkit_client_execute_with_options",
        }
        self.assertTrue(required_symbols <= functions)
        for symbol in required_symbols:
            self.assertIn(("c-kmipkit", symbol, "declared"), declarations)

        type_ids = {record["id"] for record in manifest["types"]}
        self.assertTrue(
            {
                "Client",
                "ClientBatch",
                "ClientBatchItem",
                "ClientBatchResponse",
                "TransportConfigV1",
                "ByteSpanV1",
                "TransportInputV1",
                "TimeoutLimitV1",
            }
            <= type_ids
        )

    def test_transport_configuration_function_maps_to_transport_construction(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        function = next(
            record for record in manifest["functions"]
            if record["id"] == "transport_config_create_v1"
        )

        self.assertEqual(function["java"]["class"], "TransportConfiguration.Builder")
        self.assertEqual(function["java"]["method"], "build")
        self.assertEqual(function["java"]["returnType"], "TransportConfiguration")
        self.assertEqual(function["python"]["function"], "TransportConfig")
        self.assertEqual(function["python"]["returnType"], "TransportConfig")

        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        java_name = "org.kmipkit.extensions.TransportConfiguration.Builder.build"
        python_name = "kmipkit.client.TransportConfig"
        for row in manifest["publicDeclarations"]:
            if row["declarationId"] in {
                "transport_config_create_v1:rust",
                "transport_config_create_v1:c",
                "transport_config_create_v1:java",
            }:
                self.assertEqual(row["languageMappings"]["java"]["name"], java_name)
                self.assertEqual(row["languageMappings"]["python"]["name"], python_name)
        self.assertIn(("java-kmipkit", java_name), declarations)
        self.assertIn(("python-kmipkit", python_name), declarations)

    def test_java_nullable_parameters_are_supported_by_schema_and_generator(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        schema = json.loads((ROOT / "specification" / "api" / "public-api.schema.json").read_text(
            encoding="utf-8"
        ))
        function = next(
            record for record in manifest["functions"]
            if record["id"] == "client_execute_with_options"
        )
        parameter = next(
            row for row in function["java"]["parameters"]
            if row["name"] == "encodingOverride"
        )
        parameter["nullable"] = True

        generate._validate_schema(manifest, schema)
        self.assertIn('"nullable":true', generate.render_java_api(manifest))

    def test_java_nullable_override_matches_python_none_and_c_default_sentinel(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        function = next(
            record for record in manifest["functions"]
            if record["id"] == "client_execute_with_options"
        )
        java_parameter = next(
            row for row in function["java"]["parameters"]
            if row["name"] == "encodingOverride"
        )
        python_parameter = next(
            row for row in function["python"]["parameters"]
            if row["name"] == "wire_encoding"
        )
        c_parameter = next(
            row for row in function["c"]["parameters"]
            if row["name"] == "encoding_override"
        )

        self.assertIs(java_parameter.get("nullable"), True)
        self.assertEqual(python_parameter["type"], "WireEncoding | None")
        self.assertEqual(c_parameter["type"], "int32_t")
        self.assertIn("-1 to use the configured default", " ".join(function["c"]["documentation"]))

    def test_versioned_c_record_layout_is_generated_from_manifest_fields(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        manifest["types"].insert(
            0,
            {
                "id": "TransportConfigProbe",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0057-FR-005"],
                "rust": {"module": "kmipkit_ffi", "name": "Probe", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_transport_config_probe_v1_t",
                    "kind": "record",
                    "fields": [
                        {"name": "struct_size", "type": "uint32_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                        {"name": "endpoint", "type": "kmipkit_byte_span_v1_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "Probe"},
                "python": {"module": "kmipkit.client", "name": "Probe"},
            },
        )

        c_header = generate.render_c_header(manifest, include_blocked=True)
        rust_ffi = generate.render_rust_ffi(manifest)
        self.assertIn("uint32_t struct_size;", c_header)
        self.assertIn("kmipkit_byte_span_v1_t endpoint;", c_header)
        self.assertIn("pub endpoint: kmipkit_byte_span_v1_t,", rust_ffi)
        self.assertLess(
            c_header.index("} kmipkit_transport_input_v1_t;"),
            c_header.index("} kmipkit_transport_config_v1_t;"),
            "dependent by-value C records must be declared before their consumers",
        )


if __name__ == "__main__":
    unittest.main()
