"""Contract tests for the complete public API manifest v2 schema."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"
REGISTRY_FIXTURE_PATH = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "registry-manifest.json"
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"

EXPECTED_PACKAGES = {
    "rust-kmipkit-ttlv",
    "rust-kmipkit-protocol",
    "rust-kmipkit-transport",
    "rust-kmipkit-client",
    "rust-kmipkit",
    "c-kmipkit",
    "java-kmipkit",
    "python-kmipkit",
}
EXPECTED_LANGUAGES = ["rust", "c", "java", "python"]
EXPECTED_API_LEVELS = ["high-level", "typed-protocol", "generic-ttlv"]
EXPECTED_ROOT_FIELDS = {
    "formatVersion",
    "scope",
    "releaseLine",
    "catalog",
    "requirementIds",
    "languages",
    "publicPackages",
    "publicDeclarations",
    "approvedExclusions",
    "apiLevels",
    "operations",
    "capabilities",
    "types",
    "functions",
    "errorCategories",
    "limits",
    "security",
    "generatedOutputs",
}
GENERATED_OUTPUTS = (
    ("registryRustFfi", "crates/kmipkit-ffi/src/extension_registry_generated.rs"),
    ("cHeader", "bindings/c/include/kmipkit.h"),
    ("registryJavaApi", "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java"),
    ("registryJavaParityTests", "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java"),
    ("registryPythonApi", "bindings/python/src/kmipkit/_generated/extension_registry.py"),
    ("registryParityFixtures", "tests/fixtures/extensions/generated/registry_parity.json"),
    ("rustFfi", "crates/kmipkit-ffi/src/public_api_generated.rs"),
    ("javaApi", "bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java"),
    ("javaParityTests", "bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java"),
    ("pythonApi", "bindings/python/src/kmipkit/_generated/public_api.py"),
    ("parityFixtures", "tests/fixtures/api/generated/public_api_parity.json"),
)


def _valid_v2_manifest() -> dict[str, object]:
    manifest = json.loads(REGISTRY_FIXTURE_PATH.read_text(encoding="utf-8"))
    catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
    requirement_ids = sorted(
        {*manifest["requirementIds"], "KMIPKIT-0043-FR-001"}
    )
    package_languages = {
        "rust-kmipkit-ttlv": "rust",
        "rust-kmipkit-protocol": "rust",
        "rust-kmipkit-transport": "rust",
        "rust-kmipkit-client": "rust",
        "rust-kmipkit": "rust",
        "c-kmipkit": "c",
        "java-kmipkit": "java",
        "python-kmipkit": "python",
    }
    manifest.update(
        {
            "formatVersion": 2,
            "scope": "full-1.0",
            "releaseLine": "1.0.0",
            "catalog": {"path": "specification/catalog/kmip-2.1.json", "schemaVersion": "1"},
            "requirementIds": requirement_ids,
            "languages": EXPECTED_LANGUAGES,
            "publicPackages": [
                {
                    "id": package_id,
                    "language": language,
                    "packageName": package_id,
                    "artifactFamily": (
                        "rust-crate" if language == "rust" else
                        "c-distribution" if language == "c" else
                        "java-maven-family" if language == "java" else
                        "python-pypi-family"
                    ),
                    "versionSource": "fixture",
                    "supportedTargets": ["fixture"],
                }
                for package_id, language in package_languages.items()
            ],
            "publicDeclarations": [],
            "approvedExclusions": [],
            "apiLevels": [
                {"id": level, "requirementIds": ["KMIPKIT-0043-FR-001"]}
                for level in EXPECTED_API_LEVELS
            ],
            "operations": sorted(
                [
                {
                    "operationId": item["element_id"],
                    "name": item["name"],
                    "section": item["source_refs"][0]["section"],
                    "scope": "client_1_0",
                    "apiState": "blocked",
                    "blockedReason": "The owner specification or canonical Rust API is not ready.",
                }
                for item in catalog["elements"]
                if item.get("kind") == "operation" and item.get("scope_state") == "client_1_0"
                ],
                key=lambda operation: tuple(
                    int(part) for part in operation["section"].split(".")
                ),
            ),
            "capabilities": [],
            "generatedOutputs": [
                {"name": name, "path": path} for name, path in GENERATED_OUTPUTS
            ],
        }
    )
    return manifest


class PublicApiManifestV2SchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        cls.contract = cls.schema["$defs"]["publicApiV2"]
        cls.validator = Draft202012Validator(cls.schema)

    def test_schema_identifies_the_complete_v2_contract(self) -> None:
        self.assertEqual(
            self.schema["$id"], "urn:kmipkit:schemas:public-api-manifest:2"
        )
        self.assertEqual(self.contract["properties"]["formatVersion"]["const"], 2)
        self.assertEqual(self.contract["properties"]["scope"]["const"], "full-1.0")
        self.assertEqual(self.contract["properties"]["releaseLine"]["const"], "1.0.0")
        self.assertTrue(EXPECTED_ROOT_FIELDS.issubset(self.contract["required"]))
        self.assertFalse(self.contract["additionalProperties"])

    def test_schema_requires_exactly_the_eight_public_package_ids(self) -> None:
        package_id = self.schema["$defs"]["publicPackage"]["properties"]["id"]
        self.assertEqual(set(package_id["enum"]), EXPECTED_PACKAGES)
        self.assertEqual(
            self.contract["properties"]["publicPackages"]["minItems"], 8
        )
        self.assertEqual(
            self.contract["properties"]["publicPackages"]["maxItems"], 8
        )

    def test_schema_fixes_language_and_api_level_sets(self) -> None:
        languages = self.contract["properties"]["languages"]
        api_levels = self.contract["properties"]["apiLevels"]
        self.assertEqual(languages["const"], EXPECTED_LANGUAGES)
        self.assertEqual(
            [entry["properties"]["id"]["const"] for entry in api_levels["prefixItems"]],
            EXPECTED_API_LEVELS,
        )
        self.assertFalse(api_levels["items"])

    def test_declaration_and_exclusion_records_are_closed_and_complete(self) -> None:
        declaration = self.schema["$defs"]["publicDeclaration"]
        exclusion = self.schema["$defs"]["approvedExclusion"]
        self.assertTrue(
            {
                "declarationId",
                "packageId",
                "kind",
                "exportName",
                "capabilityId",
                "requirementIds",
                "declarationState",
                "parityScope",
            }.issubset(declaration["required"])
        )
        self.assertTrue(
            {"packageId", "exportName", "reason", "approvalSource", "evidence"}.issubset(
                exclusion["required"]
            )
        )
        self.assertFalse(declaration["additionalProperties"])
        self.assertFalse(exclusion["additionalProperties"])

    def test_package_and_declaration_schema_reject_unknown_fields(self) -> None:
        for definition in ("publicPackage", "publicDeclaration", "approvedExclusion"):
            with self.subTest(definition=definition):
                self.assertFalse(self.schema["$defs"][definition]["additionalProperties"])

    def test_complete_v2_fixture_is_valid_and_unknown_root_fields_are_rejected(self) -> None:
        manifest = _valid_v2_manifest()
        self.assertEqual(list(self.validator.iter_errors(manifest)), [])

        manifest["unreviewedField"] = True

        self.assertTrue(list(self.validator.iter_errors(manifest)))

    def test_blocked_declaration_requires_a_reason_and_forbids_mappings(self) -> None:
        manifest = _valid_v2_manifest()
        declaration = {
            "declarationId": "blocked_example",
            "packageId": "rust-kmipkit",
            "kind": "function",
            "exportName": "kmipkit::candidate_operation",
            "capabilityId": "candidate_operation",
            "requirementIds": ["KMIPKIT-0043-FR-001"],
            "declarationState": "blocked",
            "parityScope": "four-language",
            "candidateSpecId": "KMIPKIT-0017",
        }
        manifest["publicDeclarations"] = [declaration]
        self.assertTrue(list(self.validator.iter_errors(manifest)))

        declaration["blockedReason"] = "The owning operation specification is not approved."
        declaration["languageMappings"] = {"rust": {"name": "candidate_operation"}}
        self.assertTrue(list(self.validator.iter_errors(manifest)))

    def test_declared_declaration_requires_approval_and_language_mappings(self) -> None:
        manifest = _valid_v2_manifest()
        manifest["publicDeclarations"] = [
            {
                "declarationId": "declared_example",
                "packageId": "rust-kmipkit",
                "kind": "function",
                "exportName": "kmipkit::ready_operation",
                "capabilityId": "ready_operation",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "declarationState": "declared",
                "parityScope": "four-language",
            }
        ]

        self.assertTrue(list(self.validator.iter_errors(manifest)))


if __name__ == "__main__":
    unittest.main()
