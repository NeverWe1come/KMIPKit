"""Require every ready 0053 mapping target to be listed as an exact export."""

from __future__ import annotations

import unittest
import re
import copy
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]


class MappingTargetInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = generate._load_json(
            ROOT, generate.MANIFEST_PATH, generate.MAX_MANIFEST_BYTES
        )

    def test_every_ready_mapping_resolves_to_an_exact_listed_export(self) -> None:
        packages = {
            package["id"]: package["language"]
            for package in self.manifest["publicPackages"]
        }
        c_release_functions = {
            ("c-kmipkit", type_record["c"]["releaseFunction"])
            for type_record in self.manifest.get("types", [])
            if isinstance(type_record, dict)
            and isinstance(type_record.get("c"), dict)
            and isinstance(type_record["c"].get("releaseFunction"), str)
        }

        generate._validate_ready_mapping_targets(
            self.manifest["publicDeclarations"], packages, c_release_functions
        )

    def test_named_c_attribute_tags_map_to_the_same_four_language_attributes(self) -> None:
        declarations = self.manifest["publicDeclarations"]
        c_tags = [
            row for row in declarations
            if row.get("declarationId", "").startswith("KMIPKIT-0053:c-standard-attribute:")
            and row.get("exportName") != "KMIPKIT_STANDARD_ATTRIBUTE_TAG"
        ]
        self.assertEqual(62, len(c_tags))
        for declaration in c_tags:
            suffix = declaration["exportName"].removeprefix(
                "KMIPKIT_STANDARD_ATTRIBUTE_"
            )
            marker = "".join(
                part.title() for part in re.split(r"[_-]+", suffix.lower())
            )
            mappings = declaration["languageMappings"]
            self.assertEqual("four-language", declaration["parityScope"])
            self.assertEqual(
                "standard-attribute-tag-value", declaration["mappingSemantics"]
            )
            self.assertEqual(
                {"packageId": "c-kmipkit", "name": declaration["exportName"]},
                mappings["c"],
            )
            self.assertEqual(
                {
                    "packageId": "rust-kmipkit",
                    "name": f"kmipkit::protocol::attribute::typed::{marker}",
                },
                mappings["rust"],
            )
            self.assertEqual(
                {
                    "packageId": "java-kmipkit",
                    "name": f"org.kmipkit.ttlv.StandardAttribute.{suffix}",
                },
                mappings["java"],
            )
            self.assertEqual(
                {
                    "packageId": "python-kmipkit",
                    "name": f"kmipkit.typed_attributes.StandardAttribute.{suffix}",
                },
                mappings["python"],
            )

    def test_standard_attribute_tag_mapping_semantics_reject_unrelated_routes(self) -> None:
        packages = {
            package["id"]: package["language"]
            for package in self.manifest["publicPackages"]
        }
        c_release_functions = set()
        declaration = next(
            row for row in self.manifest["publicDeclarations"]
            if row.get("declarationId")
            == "KMIPKIT-0053:c-standard-attribute:activation-date"
        )
        for key, value in (
            ("capabilityId", "client_execution"),
            ("parityScope", "c-only"),
            ("kind", "function"),
        ):
            declarations = copy.deepcopy(self.manifest["publicDeclarations"])
            target = next(
                row for row in declarations
                if row.get("declarationId") == declaration["declarationId"]
            )
            target[key] = value
            with self.subTest(field=key):
                with self.assertRaisesRegex(
                    generate.ManifestError, "standard attribute tag mapping"
                ):
                    generate._validate_ready_mapping_targets(
                        declarations, packages, c_release_functions
                    )

    def test_generic_c_tag_macro_maps_to_the_standard_attribute_accessors(self) -> None:
        declaration = next(
            row for row in self.manifest["publicDeclarations"]
            if row.get("declarationId")
            == "KMIPKIT-0053:c-standard-attribute:tag"
        )
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(
            "standard-attribute-tag-value", declaration["mappingSemantics"]
        )
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit",
                    "name": "kmipkit::protocol::AttributeDefinition::TAG",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "KMIPKIT_STANDARD_ATTRIBUTE_TAG",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.ttlv.StandardAttribute.tag",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.StandardAttribute.tag",
                },
            },
            declaration["languageMappings"],
        )


if __name__ == "__main__":
    unittest.main()
