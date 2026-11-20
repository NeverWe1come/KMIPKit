"""Check that generated profile data maps to the public binding declarations."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "bindings/shared/public-api.json"
MODULE_NAME = "kmipkit_protocol::profiles_generated"
CAPABILITY_ID = "client_profile_readiness"


class ProfileCatalogMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest: dict[str, Any] = json.loads(
            MANIFEST_PATH.read_text(encoding="utf-8")
        )
        cls.module_declarations = [
            declaration
            for declaration in cls.manifest["publicDeclarations"]
            if declaration.get("exportName", "").startswith(MODULE_NAME + "::")
        ]

    def test_generated_namespace_is_not_a_separate_public_api_item(self) -> None:
        standalone_declarations = [
            declaration
            for declaration in self.manifest["publicDeclarations"]
            if declaration.get("exportName") == MODULE_NAME
        ]
        self.assertEqual([], standalone_declarations)

        self.assertTrue(self.module_declarations)
        for declaration in self.module_declarations:
            with self.subTest(export=declaration["exportName"]):
                self.assertEqual(CAPABILITY_ID, declaration["capabilityId"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("four-language", declaration["parityScope"])

    def test_every_profile_declaration_maps_to_all_four_binding_packages(self) -> None:
        expected_languages = {"rust", "c", "java", "python"}
        for declaration in self.module_declarations:
            mappings = declaration.get("languageMappings", {})
            with self.subTest(export=declaration["exportName"]):
                self.assertEqual(expected_languages, set(mappings))
                for language, mapping in mappings.items():
                    self.assertTrue(mapping["packageId"])
                    self.assertTrue(mapping["name"])

        capability = next(
            capability
            for capability in self.manifest["capabilities"]
            if capability["capabilityId"] == CAPABILITY_ID
        )
        self.assertNotIn(MODULE_NAME, capability["mappings"]["rust"])

    def test_profile_api_sources_exist_for_each_language(self) -> None:
        sources = (
            ROOT / "crates/kmipkit-protocol/src/profiles_generated.rs",
            ROOT / "bindings/c/include/kmipkit.h",
            ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/ProfileCatalog.java",
            ROOT / "bindings/python/src/kmipkit/profile_catalog.py",
        )
        for source in sources:
            with self.subTest(source=source.relative_to(ROOT)):
                self.assertTrue(source.is_file())


if __name__ == "__main__":
    unittest.main()
