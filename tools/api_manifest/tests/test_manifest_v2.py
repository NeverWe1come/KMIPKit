"""Contract tests for the complete public API manifest v2 schema."""

from __future__ import annotations

import json
import re
import tomllib
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

from tools.api_manifest.generate import (
    ManifestError,
    _validate_operation_inventory,
    _validate_v2_readiness,
)


ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"
CHECKED_IN_MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REGISTRY_FIXTURE_PATH = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "public-api-v1.json"
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
LEGACY_GENERATED_OUTPUTS = {
    "rustFfi": "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "cHeader": "bindings/c/include/kmipkit.h",
    "javaApi": "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "javaParityTests": "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "pythonApi": "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "parityFixtures": "tests/fixtures/extensions/generated/registry_parity.json",
}


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
                    "ownerDependency": {"state": "unassigned", "specId": None},
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
        cls.checked_in_manifest = json.loads(
            CHECKED_IN_MANIFEST_PATH.read_text(encoding="utf-8")
        )
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))

    def declarations_for_package(self, package_id: str) -> list[dict[str, object]]:
        return [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration["packageId"] == package_id
        ]

    def test_checked_in_manifest_is_the_complete_v2_contract(self) -> None:
        manifest = self.checked_in_manifest
        self.assertEqual(manifest["formatVersion"], 2)
        self.assertEqual(manifest["scope"], "full-1.0")
        self.assertEqual(manifest["releaseLine"], "1.0.0")
        self.assertEqual(
            {package["id"] for package in manifest["publicPackages"]},
            EXPECTED_PACKAGES,
        )
        self.assertEqual(len(manifest["operations"]), 56)
        self.assertTrue(manifest["publicDeclarations"])
        self.assertTrue(manifest["capabilities"])
        self.assertIn("approvedExclusions", manifest)
        self.assertEqual(list(self.validator.iter_errors(manifest)), [])

    def test_official_package_versions_match_the_manifest_release_line(self) -> None:
        expected = self.checked_in_manifest["releaseLine"]
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        python = tomllib.loads(
            (ROOT / "bindings" / "python" / "pyproject.toml").read_text(encoding="utf-8")
        )
        java_gradle = (ROOT / "bindings" / "java" / "build.gradle.kts").read_text(
            encoding="utf-8"
        )
        java_pom = (ROOT / "bindings" / "java" / "pom.xml").read_text(encoding="utf-8")

        self.assertEqual(workspace["workspace"]["package"]["version"], expected)
        self.assertEqual(python["project"]["version"], expected)
        self.assertRegex(java_gradle, rf'(?m)^version = "{re.escape(expected)}"$')
        self.assertRegex(java_pom, rf"(?m)^\s*<version>{re.escape(expected)}</version>$")

    def test_runtime_metadata_exports_are_inventoried_in_every_language(self) -> None:
        declarations = {
            (entry["packageId"], entry["exportName"]): entry
            for entry in self.checked_in_manifest["publicDeclarations"]
        }
        expected = {
            ("rust-kmipkit", "kmipkit::VERSION"),
            ("rust-kmipkit", "kmipkit::ABI_VERSION"),
            ("rust-kmipkit", "kmipkit::RuntimeVersion"),
            ("rust-kmipkit", "kmipkit::RuntimeVersion::product_version"),
            ("rust-kmipkit", "kmipkit::RuntimeVersion::abi_version"),
            ("rust-kmipkit", "kmipkit::runtime_version"),
            ("rust-kmipkit", "kmipkit::capability_is_available"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.version"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.capabilityIsAvailable"),
            ("python-kmipkit", "kmipkit.metadata"),
            ("python-kmipkit", "kmipkit.metadata.Version"),
            ("python-kmipkit", "kmipkit.metadata.get_version"),
            ("python-kmipkit", "kmipkit.metadata.capability_is_available"),
        }

        self.assertTrue(expected <= declarations.keys())
        for key in expected:
            with self.subTest(declaration=key):
                declaration = declarations[key]
                self.assertEqual(declaration["capabilityId"], "ffi_runtime_metadata")
                self.assertEqual(declaration["declarationState"], "declared")

    def test_checked_in_manifest_retains_every_registry_slice_record(self) -> None:
        manifest = self.checked_in_manifest
        registry = json.loads(REGISTRY_FIXTURE_PATH.read_text(encoding="utf-8"))
        for field in ("types", "errorCategories", "limits", "security"):
            with self.subTest(field=field):
                self.assertEqual(manifest[field], registry[field])
        manifest_functions = {item["id"]: item for item in manifest["functions"]}
        for function in registry["functions"]:
            with self.subTest(function=function["id"]):
                if function["id"] in {"ffi_last_error_get", "ffi_last_error_clear"}:
                    expected_function = json.loads(json.dumps(function))
                    if function["id"] == "ffi_last_error_get":
                        expected_function["python"]["function"] = "_last_error_snapshot"
                    else:
                        expected_function["python"]["function"] = "_clear_last_error"
                    self.assertEqual(manifest_functions[function["id"]], expected_function)
                else:
                    self.assertEqual(manifest_functions[function["id"]], function)
        self.assertTrue(set(registry["requirementIds"]) <= set(manifest["requirementIds"]))
        outputs = manifest["generatedOutputs"]
        if isinstance(outputs, list):
            paths = {entry["path"] for entry in outputs}
            self.assertTrue(set(LEGACY_GENERATED_OUTPUTS.values()) <= paths)
        else:
            self.assertEqual(outputs, LEGACY_GENERATED_OUTPUTS)

    def test_checked_in_operation_inventory_matches_the_normative_catalog(self) -> None:
        _validate_operation_inventory(self.checked_in_manifest, self.catalog)

    def test_checked_in_manifest_resolves_all_public_declaration_mappings(self) -> None:
        _validate_v2_readiness(self.checked_in_manifest, ROOT)

    def test_readiness_rejects_duplicate_package_export_declarations(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        declaration = next(
            item for item in manifest["publicDeclarations"]
            if item["declarationState"] == "blocked"
        )
        duplicate = json.loads(json.dumps(declaration))
        duplicate["declarationId"] += ":duplicate"
        manifest["publicDeclarations"].append(duplicate)

        with self.assertRaisesRegex(ManifestError, "duplicate public export path"):
            _validate_v2_readiness(manifest, ROOT)

    def test_readiness_rejects_duplicate_approved_exclusion_paths(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        exclusion = json.loads(json.dumps(manifest["approvedExclusions"][0]))
        exclusion["reason"] += " Duplicate record with a different reason."
        manifest["approvedExclusions"].append(exclusion)

        with self.assertRaisesRegex(ManifestError, "duplicate approved exclusion path"):
            _validate_v2_readiness(manifest, ROOT)

    def test_readiness_rejects_declaration_and_exclusion_overlap(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        exclusion = manifest["approvedExclusions"][0]
        declaration = next(
            item for item in manifest["publicDeclarations"]
            if item["packageId"] == exclusion["packageId"]
            and item["declarationState"] == "blocked"
        )
        overlapping = json.loads(json.dumps(declaration))
        overlapping["declarationId"] += ":overlap"
        overlapping["exportName"] = exclusion["exportName"]
        manifest["publicDeclarations"].append(overlapping)

        with self.assertRaisesRegex(ManifestError, "overlaps an approved exclusion"):
            _validate_v2_readiness(manifest, ROOT)

    def test_checked_in_manifest_records_unmapped_binding_exports_as_blocked(self) -> None:
        declarations = {
            declaration["exportName"]: declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        for export_name in (
            "kmipkit.errors",
            "kmipkit.extensions",
            "kmipkit.ttlv",
            "kmipkit.extensions.ClientBatchItem",
            "kmipkit.extensions.with_extension",
            "org.kmipkit.NativeExtensionRegistry",
            "org.kmipkit.generated.ExtensionRegistryApi",
            "kmipkit.errors.KmipKitError",
            "kmipkit.errors.raise_for_status",
        ):
            with self.subTest(export_name=export_name):
                declaration = declarations[export_name]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertIn("candidateSpecId", declaration)
                self.assertIn("blockedReason", declaration)
                self.assertNotIn("languageMappings", declaration)

    def test_t011_observed_binding_exports_are_reconciled_without_invented_mappings(self) -> None:
        declarations = {
            (item["packageId"], item["exportName"]): item
            for item in self.checked_in_manifest["publicDeclarations"]
        }
        blocked_exports = {
            ("c-kmipkit", f"KMIPKIT_ERROR_{name}")
            for name in (
                "COMPATIBILITY_MISMATCH",
                "DUPLICATE_KEY",
                "INVALID_IDENTITY",
                "INVALID_INPUT",
                "INVALID_SCHEMA",
            )
        }
        blocked_exports.update({
            ("c-kmipkit", f"aws_lc_0_45_0_jent_{name}")
            for name in (
                "entropy_collector_alloc",
                "entropy_collector_free",
                "entropy_init",
                "entropy_init_ex",
                "entropy_switch_notime_impl",
                "read_entropy",
                "read_entropy_safe",
                "set_fips_failure_callback",
                "version",
            )
        })
        blocked_exports.update({
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.<init>"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.abiVersion"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.equals"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.hashCode"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.major"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.minor"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.patch"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.toString"),
            ("java-kmipkit", "org.kmipkit.generated.PublicApi"),
            ("java-kmipkit", "org.kmipkit.generated.PublicApi.DECLARATIONS_JSON"),
            ("java-kmipkit", "org.kmipkit.generated.PublicApi.FORMAT_VERSION"),
            ("python-kmipkit", "kmipkit"),
            ("python-kmipkit", "kmipkit.errors.CompatibilityMismatchError.category"),
            ("python-kmipkit", "kmipkit.errors.DuplicateKeyError.category"),
            ("python-kmipkit", "kmipkit.errors.InvalidExtensionSchemaError.category"),
            ("python-kmipkit", "kmipkit.errors.InvalidIdentityError.category"),
            ("python-kmipkit", "kmipkit.errors.InvalidInputError.category"),
            ("python-kmipkit", "kmipkit.errors.KmipKitError.category"),
            ("python-kmipkit", "kmipkit.errors.ResourceLimitError.category"),
            ("python-kmipkit", "kmipkit.metadata.Version.abi_version"),
            ("python-kmipkit", "kmipkit.metadata.Version.major"),
            ("python-kmipkit", "kmipkit.metadata.Version.minor"),
            ("python-kmipkit", "kmipkit.metadata.Version.patch"),
            ("python-kmipkit", "kmipkit.extensions.validate_extension_value"),
        })
        for key in blocked_exports:
            with self.subTest(export=key):
                declaration = declarations[key]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertEqual(declaration["candidateSpecId"], "KMIPKIT-0043")
                self.assertEqual(declaration["capabilityId"], "public_binding_surface")
                self.assertEqual(declaration["requirementIds"], ["KMIPKIT-0043-FR-021"])
                self.assertNotIn("languageMappings", declaration)

        exclusions = {
            (item["packageId"], item["exportName"])
            for item in self.checked_in_manifest["approvedExclusions"]
        }
        self.assertFalse(
            {
                ("c-kmipkit", "KMIPKIT_H"),
                *(key for key in blocked_exports if key[0] == "c-kmipkit" and key[1].startswith("aws_lc_")),
            }
            & exclusions
        )
        self.assertNotIn(("c-kmipkit", "KMIPKIT_H"), declarations)
        self.assertNotIn(
            ("java-kmipkit", "org.kmipkit.internal.NativeHandle.run"),
            declarations,
        )
        runtime_version = declarations[
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version")
        ]
        self.assertEqual(
            runtime_version["languageMappings"]["java"]["name"],
            "org.kmipkit.RuntimeMetadata.Version",
        )
        self.assertEqual(
            runtime_version["languageMappings"]["python"]["name"],
            "kmipkit.metadata.Version",
        )

    def test_checked_in_manifest_records_transport_as_the_only_rust_only_capability(self) -> None:
        transport = next(
            capability
            for capability in self.checked_in_manifest["capabilities"]
            if capability["capabilityId"] == "rust_transport_exchange"
        )
        self.assertEqual(transport["parityScope"], "rust-only-adr")
        self.assertEqual(transport["capabilityState"], "declared")
        self.assertEqual(transport["ownerAdrId"], "ADR-0014")

    def test_checked_in_manifest_accounts_for_c_struct_fields_and_ttlv_macros(self) -> None:
        declarations = {
            declaration["exportName"]: declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        exports = [
            *(f"kmipkit_extension_registry_limits_t.{name}" for name in (
                "max_definitions",
                "max_schema_nodes",
                "max_child_rules_per_structure",
                "max_text_bytes_per_field",
                "max_registry_text_bytes",
                "max_discriminator_scalar_bytes",
                "max_total_discriminator_scalar_bytes",
                "max_constraint_members_per_rule",
                "max_total_constraint_members",
                "max_payload_index_records",
                "max_lookup_comparisons",
                "max_depth",
            )),
            *(f"KMIPKIT_TTLV_ITEM_TYPE_{name}" for name in (
                "STRUCTURE",
                "INTEGER",
                "LONG_INTEGER",
                "BIG_INTEGER",
                "ENUMERATION",
                "BOOLEAN",
                "TEXT_STRING",
                "BYTE_STRING",
                "DATE_TIME",
                "INTERVAL",
                "DATE_TIME_EXTENDED",
            )),
        ]
        self.assertEqual(len(exports), len(set(exports)))
        for export_name in exports:
            with self.subTest(export_name=export_name):
                declaration = declarations[export_name]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertIn("candidateSpecId", declaration)
                self.assertNotIn("languageMappings", declaration)

    def test_checked_in_manifest_inventories_all_rust_public_access_paths(self) -> None:
        rust_packages = EXPECTED_PACKAGES - {"c-kmipkit", "java-kmipkit", "python-kmipkit"}
        declarations = [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration["packageId"] in rust_packages
        ]
        self.assertEqual(len(declarations), 1767)
        by_package_and_name = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in declarations
        }
        codec_limits = by_package_and_name[
            ("rust-kmipkit-ttlv", "kmipkit_ttlv::codec::CodecLimits")
        ]
        self.assertEqual(codec_limits["kind"], "type")
        self.assertEqual(codec_limits["declarationState"], "declared")
        self.assertEqual(
            codec_limits["languageMappings"]["rust"]["packageId"],
            "rust-kmipkit-ttlv",
        )
        batch_method = by_package_and_name[
            ("rust-kmipkit-client", "kmipkit_client::ClientBatchItem::discover_versions")
        ]
        self.assertEqual(batch_method["kind"], "associated-item")
        facade_item = by_package_and_name[
            ("rust-kmipkit", "kmipkit::ClientBatchItem")
        ]
        self.assertEqual(facade_item["declarationState"], "blocked")
        self.assertEqual(facade_item["candidateSpecId"], "KMIPKIT-0012")
        self.assertNotIn("languageMappings", facade_item)
        unknown_owner = by_package_and_name[
            (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::CryptographicAlgorithm",
            )
        ]
        self.assertEqual(unknown_owner["declarationState"], "blocked")
        transport_exchange = by_package_and_name[
            ("rust-kmipkit-transport", "kmipkit_transport::Transport::exchange")
        ]
        self.assertEqual(transport_exchange["declarationState"], "declared")
        self.assertEqual(transport_exchange["parityScope"], "rust-only-adr")

    def test_ffi_runtime_metadata_is_owned_and_mapped_in_all_languages(self) -> None:
        capability = next(
            (
                record
                for record in self.checked_in_manifest["capabilities"]
                if record["capabilityId"] == "ffi_runtime_metadata"
            ),
            None,
        )
        self.assertIsNotNone(capability)
        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["ownerSpecId"], "KMIPKIT-0043")
        self.assertEqual(
            set(capability["mappings"]),
            {"rust", "c", "java", "python"},
        )

        expected_symbols = {
            "kmipkit_version_get",
            "kmipkit_capability_is_available",
        }
        functions = {
            function["c"]["symbol"]: function
            for function in self.checked_in_manifest["functions"]
        }
        self.assertTrue(expected_symbols <= set(functions))
        for symbol in expected_symbols:
            with self.subTest(symbol=symbol):
                function = functions[symbol]
                self.assertIn("KMIPKIT-0043-FR-007", function["requirementIds"])

        capability_function = functions["kmipkit_capability_is_available"]
        capability_span = next(
            parameter
            for parameter in capability_function["c"]["parameters"]
            if parameter["name"] == "capability_id_data"
        )
        self.assertEqual(capability_span["maxBytes"], 128)

        declarations = {
            declaration.get("languageMappings", {}).get("c", {}).get("name"):
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        self.assertTrue(expected_symbols <= set(declarations))
        for symbol in expected_symbols:
            self.assertEqual(declarations[symbol]["capabilityId"], "ffi_runtime_metadata")
            self.assertEqual(declarations[symbol]["declarationState"], "declared")

        rust_lookup = next(
            item for item in self.checked_in_manifest["publicDeclarations"]
            if item["exportName"] == "kmipkit::capability_is_available"
        )
        self.assertEqual(rust_lookup["packageId"], "rust-kmipkit")
        self.assertEqual(
            rust_lookup["languageMappings"]["rust"]["name"],
            "kmipkit::capability_is_available",
        )
        rust_limit_error = next(
            item for item in self.checked_in_manifest["publicDeclarations"]
            if item["exportName"] == "kmipkit::RuntimeMetadataError::ResourceLimit"
        )
        self.assertEqual(rust_limit_error["declarationState"], "declared")
        self.assertEqual(
            rust_limit_error["languageMappings"]["rust"]["name"],
            "kmipkit::RuntimeMetadataError::ResourceLimit",
        )
        self.assertEqual(
            {
                language: mapping["name"]
                for language, mapping in rust_limit_error["languageMappings"].items()
            },
            {
                "rust": "kmipkit::RuntimeMetadataError::ResourceLimit",
                "c": "KMIPKIT_ERROR_RESOURCE_LIMIT",
                "java": "org.kmipkit.ResourceLimitException",
                "python": "kmipkit.errors.ResourceLimitError",
            },
        )

        runtime_version = next(
            item for item in self.checked_in_manifest["publicDeclarations"]
            if item["exportName"] == "kmipkit::RuntimeVersion"
        )
        self.assertEqual(runtime_version["kind"], "type")
        self.assertEqual(
            runtime_version["languageMappings"]["rust"]["name"],
            "kmipkit::RuntimeVersion",
        )
        self.assertEqual(
            runtime_version["languageMappings"]["java"]["name"],
            "org.kmipkit.RuntimeMetadata.Version",
        )
        self.assertEqual(
            runtime_version["languageMappings"]["python"]["name"],
            "kmipkit.metadata.Version",
        )
        runtime_version_function = next(
            item for item in self.checked_in_manifest["publicDeclarations"]
            if item["exportName"] == "kmipkit::runtime_version"
        )
        self.assertEqual(runtime_version_function["kind"], "function")
        self.assertEqual(
            runtime_version_function["languageMappings"]["c"]["name"],
            "kmipkit_version_get",
        )

        exclusions = {
            item["exportName"]: item
            for item in self.checked_in_manifest["approvedExclusions"]
        }
        self.assertEqual(
            set(exclusions),
            {"kmipkit_last_error_get", "kmipkit_last_error_clear"},
        )
        for symbol in exclusions:
            self.assertEqual(exclusions[symbol]["packageId"], "c-kmipkit")
            self.assertTrue(exclusions[symbol]["reason"])
            self.assertTrue(exclusions[symbol]["evidence"])
            self.assertNotIn(symbol, declarations)

        public_exports = {
            item["exportName"]
            for item in self.checked_in_manifest["publicDeclarations"]
            if item["packageId"] == "python-kmipkit"
        }
        self.assertFalse(
            {
                "kmipkit.metadata.last_error_snapshot",
                "kmipkit.metadata.clear_last_error",
                "kmipkit.metadata.FfiErrorSnapshot",
                "kmipkit.metadata.ClientCauseCategory",
                "kmipkit.metadata.RequestDeliveryState",
            }
            & public_exports
        )
        functions_by_id = {
            item["id"]: item for item in self.checked_in_manifest["functions"]
        }
        self.assertEqual(
            functions_by_id["ffi_last_error_get"]["python"]["function"],
            "_last_error_snapshot",
        )
        self.assertEqual(
            functions_by_id["ffi_last_error_clear"]["python"]["function"],
            "_clear_last_error",
        )

        capability_mappings = capability["mappings"]
        for language in ("rust", "c", "java", "python"):
            self.assertFalse(
                any("last_error" in name or "lastError" in name
                    for name in capability_mappings[language])
            )

    def test_ttlv_declarations_use_their_approved_shared_spec_owners(self) -> None:
        capabilities = {
            record["capabilityId"]: record
            for record in self.checked_in_manifest["capabilities"]
        }
        expected_owners = {
            "generic_ttlv_model": "KMIPKIT-0004",
            "ttlv_wire_codec": "KMIPKIT-0005",
        }
        for capability_id, owner_spec_id in expected_owners.items():
            with self.subTest(capability=capability_id):
                capability = capabilities[capability_id]
                self.assertEqual(capability["ownerSpecId"], owner_spec_id)
                self.assertEqual(capability["capabilityState"], "declared")
                self.assertIn("KMIPKIT-0043-FR-004", capability["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", capability["requirementIds"])

        ttlv_declarations = []
        for declaration in self.checked_in_manifest["publicDeclarations"]:
            rust_mapping = declaration.get("languageMappings", {}).get("rust", {})
            rust_name = rust_mapping.get("name", "")
            if not rust_name.startswith("kmipkit_ttlv::"):
                continue
            expected_capability = (
                "ttlv_wire_codec"
                if rust_name.startswith("kmipkit_ttlv::codec::CodecLimits")
                else "generic_ttlv_model"
            )
            ttlv_declarations.append(declaration)
            with self.subTest(declaration=declaration["declarationId"]):
                self.assertEqual(declaration["capabilityId"], expected_capability)
                self.assertEqual(declaration["declarationState"], "declared")
                owner_prefix = (
                    "KMIPKIT-0005-FR-"
                    if expected_capability == "ttlv_wire_codec"
                    else "KMIPKIT-0004-FR-"
                )
                self.assertTrue(
                    any(
                        requirement.startswith(owner_prefix)
                        for requirement in declaration["requirementIds"]
                    )
                )
                self.assertIn("KMIPKIT-0043-FR-004", declaration["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                self.assertEqual(
                    declaration["approvalSource"]["path"],
                    "specs/004-generic-ttlv-model/spec.md"
                    if expected_capability == "generic_ttlv_model"
                    else "specs/005-ttlv-wire-codec/spec.md",
                )
        self.assertTrue(ttlv_declarations)
        self.assertEqual(len(ttlv_declarations), 101)
        self.assertEqual(
            sum(
                declaration["capabilityId"] == "generic_ttlv_model"
                for declaration in ttlv_declarations
            ),
            76,
        )
        self.assertEqual(
            sum(
                declaration["capabilityId"] == "ttlv_wire_codec"
                for declaration in ttlv_declarations
            ),
            25,
        )

    def test_checked_in_manifest_inventories_all_java_public_access_paths(self) -> None:
        declarations = self.declarations_for_package("java-kmipkit")
        self.assertEqual(len(declarations), 329)
        by_name = {declaration["exportName"]: declaration for declaration in declarations}
        batch_item = by_name["org.kmipkit.extensions.ClientBatchItem"]
        self.assertEqual(batch_item["kind"], "type")
        self.assertEqual(batch_item["declarationState"], "declared")
        batch_extension = by_name[
            "org.kmipkit.extensions.ClientBatchItem.withExtension"
        ]
        self.assertEqual(batch_extension["kind"], "java-member")
        self.assertEqual(batch_extension["declarationState"], "declared")
        native_handle = by_name["org.kmipkit.internal.NativeHandle"]
        self.assertEqual(native_handle["declarationState"], "blocked")

    def test_checked_in_manifest_accounts_for_every_c_header_declaration(self) -> None:
        header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(
            encoding="utf-8"
        )
        declarations = self.declarations_for_package("c-kmipkit")
        by_kind = {
            kind: {item["exportName"] for item in declarations if item["kind"] == kind}
            for kind in {item["kind"] for item in declarations}
        }

        header_functions = set(
            re.findall(
                r"^(?:void|int32_t)\s+(kmipkit_[A-Za-z0-9_]+)\s*\(",
                header,
                re.MULTILINE,
            )
        )
        excluded_c_functions = {
            exclusion["exportName"]
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] == "c-kmipkit"
        }
        self.assertEqual(header_functions, by_kind["c-symbol"] | excluded_c_functions)

        header_types = set(
            re.findall(r"^typedef struct \w+\s+(kmipkit_\w+);$", header, re.MULTILINE)
        )
        header_types.update(
            re.findall(r"^typedef uint8_t (kmipkit_\w+);$", header, re.MULTILINE)
        )
        header_types.update(
            re.findall(r"^} (kmipkit_\w+);$", header, re.MULTILINE)
        )
        self.assertEqual(header_types, by_kind["type"])

        limits_match = re.search(
            r"typedef struct \{(?P<body>.*?)\} kmipkit_extension_registry_limits_t;",
            header,
            re.DOTALL,
        )
        self.assertIsNotNone(limits_match)
        assert limits_match is not None
        header_fields = {
            f"kmipkit_extension_registry_limits_t.{name}"
            for name in re.findall(
                r"^\s*uint64_t (\w+);$", limits_match["body"], re.MULTILINE
            )
        }
        self.assertEqual(header_fields, by_kind["field"])

        header_ttlv_macros = set(
            re.findall(r"^#define (KMIPKIT_TTLV_ITEM_TYPE_\w+)", header, re.MULTILINE)
        )
        self.assertEqual(header_ttlv_macros, by_kind["macro"])
        header_error_macros = set(
            re.findall(r"^#define (KMIPKIT_ERROR_\w+)", header, re.MULTILINE)
        )
        manifest_error_macros = {
            item["cCode"] for item in self.checked_in_manifest["errorCategories"]
        }
        self.assertEqual(header_error_macros, manifest_error_macros)

    def test_checked_in_manifest_inventories_python_exports_and_lifecycle(self) -> None:
        declarations = self.declarations_for_package("python-kmipkit")
        self.assertEqual(len(declarations), 274)
        by_name = {declaration["exportName"]: declaration for declaration in declarations}
        close_method = by_name["kmipkit.extensions.ClientBatchItem.close"]
        self.assertEqual(close_method["kind"], "python-export")
        self.assertEqual(close_method["declarationState"], "blocked")
        item_type = by_name["kmipkit.ttlv.ItemType"]
        self.assertEqual(item_type["declarationState"], "blocked")
        enum_value = by_name["kmipkit.ttlv.ItemType.Structure"]
        self.assertEqual(enum_value["declarationState"], "blocked")
        error_type = by_name["kmipkit.errors.CompatibilityMismatchError"]
        self.assertEqual(error_type["kind"], "type")
        registry_field = by_name[
            "kmipkit.extensions.ExtensionRegistryLimits.max_definitions"
        ]
        self.assertEqual(registry_field["kind"], "field")

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

    def test_schema_rejects_identical_duplicate_declarations_and_exclusions(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        manifest["publicDeclarations"].append(
            json.loads(json.dumps(manifest["publicDeclarations"][0]))
        )
        manifest["approvedExclusions"].append(
            json.loads(json.dumps(manifest["approvedExclusions"][0]))
        )

        errors = list(self.validator.iter_errors(manifest))

        self.assertTrue(errors)

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
