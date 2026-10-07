"""Cross-language parity invariants for the reviewed public API manifest."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_OUTPUTS = {
    "rustFfi": "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "cHeader": "bindings/c/include/kmipkit.h",
    "javaApi": "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "javaParityTests": "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "pythonApi": "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "parityFixtures": "tests/fixtures/extensions/generated/registry_parity.json",
}
LIMIT_IDS = (
    "maxDefinitions",
    "maxSchemaNodes",
    "maxChildRulesPerStructure",
    "maxTextBytesPerField",
    "maxRegistryTextBytes",
    "maxDiscriminatorScalarBytes",
    "maxTotalDiscriminatorScalarBytes",
    "maxConstraintMembersPerRule",
    "maxTotalConstraintMembers",
    "maxPayloadIndexRecords",
    "maxLookupComparisons",
    "maxDepth",
)
REQUIREMENT_IDS = tuple(f"KMIPKIT-0012-FR-{number:03}" for number in range(1, 15))
JAVA_KEYWORDS = {
    "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
    "class", "const", "continue", "default", "do", "double", "else", "enum",
    "extends", "final", "finally", "float", "for", "goto", "if", "implements",
    "import", "instanceof", "int", "interface", "long", "native", "new", "package",
    "private", "protected", "public", "return", "short", "static", "strictfp",
    "super", "switch", "synchronized", "this", "throw", "throws", "transient", "try",
    "void", "volatile", "while", "true", "false", "null",
}


class PublicApiParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_manifest_covers_every_registry_requirement(self) -> None:
        self.assertEqual(self.manifest["requirementIds"], list(REQUIREMENT_IDS))

        for item in [*self.manifest["types"], *self.manifest["functions"]]:
            self.assertTrue(set(item["requirementIds"]).issubset(REQUIREMENT_IDS), item["id"])

    def test_limit_names_defaults_and_hard_maxima_are_ordered_and_equivalent(self) -> None:
        limits = self.manifest["limits"]
        self.assertEqual([limit["id"] for limit in limits], list(LIMIT_IDS))

        for limit in limits:
            with self.subTest(limit=limit["id"]):
                self.assertGreater(limit["default"], 0)
                self.assertLessEqual(limit["default"], limit["hardMaximum"])
                self.assertEqual(limit["errorCategory"], "resource_limit")
                for field in ("rustField", "cField", "javaField", "pythonField"):
                    self.assertTrue(limit[field], f"{limit['id']} has no {field}")

        self.assertEqual(len({limit["rustField"] for limit in limits}), len(LIMIT_IDS))
        self.assertEqual(len({limit["cField"] for limit in limits}), len(LIMIT_IDS))
        self.assertEqual(len({limit["javaField"] for limit in limits}), len(LIMIT_IDS))
        self.assertEqual(len({limit["pythonField"] for limit in limits}), len(LIMIT_IDS))

    def test_adapter_symbols_have_stable_owners_and_no_collisions(self) -> None:
        functions = self.manifest["functions"]
        c_symbols: list[str] = []
        jni_symbols: list[str] = []
        cffi_symbols: list[str] = []

        for function in functions:
            with self.subTest(function=function["id"]):
                rust = function["rust"]
                self.assertIn(rust["owner"], {"protocol", "client", "ffi"})
                self.assertTrue(rust["module"].startswith("kmipkit_"))

                c_symbol = function["c"].get("symbol")
                self.assertIsNotNone(c_symbol)
                self.assertTrue(c_symbol.startswith("kmipkit_"))
                c_symbols.append(c_symbol)

                java = function["java"]
                self.assertTrue(
                    java["package"] == "org.kmipkit"
                    or java["package"].startswith("org.kmipkit.")
                )
                self.assertTrue(java["jniSymbol"].startswith("Java_org_kmipkit_"))
                jni_symbols.append(java["jniSymbol"])

                python = function["python"]
                self.assertTrue(
                    python["module"] == "kmipkit" or python["module"].startswith("kmipkit.")
                )
                self.assertTrue(python["cffiSymbol"].startswith("kmipkit_"))
                cffi_symbols.append(python["cffiSymbol"])

        self.assertEqual(len(c_symbols), len(set(c_symbols)))
        self.assertEqual(len(jni_symbols), len(set(jni_symbols)))
        self.assertEqual(len(cffi_symbols), len(set(cffi_symbols)))

    def test_java_method_names_are_legal_java_identifiers(self) -> None:
        for function in self.manifest["functions"]:
            method = function["java"]["method"]
            with self.subTest(function=function["id"]):
                self.assertTrue(method.isidentifier(), method)
                self.assertNotIn(method, JAVA_KEYWORDS, method)

    def test_generated_destinations_and_parity_fixture_follow_manifest_order(self) -> None:
        self.assertEqual(list(self.manifest["generatedOutputs"]), list(GENERATED_OUTPUTS))
        self.assertEqual(self.manifest["generatedOutputs"], GENERATED_OUTPUTS)

        ids = [item["id"] for item in self.manifest["types"]]
        self.assertEqual(len(ids), len(set(ids)))
        function_ids = [item["id"] for item in self.manifest["functions"]]
        self.assertEqual(len(function_ids), len(set(function_ids)))

        for relative_path in GENERATED_OUTPUTS.values():
            self.assertTrue((ROOT / relative_path).is_file(), relative_path)

        parity_path = ROOT / GENERATED_OUTPUTS["parityFixtures"]
        parity: Any = json.loads(parity_path.read_text(encoding="utf-8"))
        self.assertEqual(parity, self.manifest)


if __name__ == "__main__":
    unittest.main()
