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


if __name__ == "__main__":
    unittest.main()
