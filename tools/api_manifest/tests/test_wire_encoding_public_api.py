"""Public four-language contract for KMIP wire encoding selection."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


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


if __name__ == "__main__":
    unittest.main()
