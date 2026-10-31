"""Contract tests for the complete public API manifest v2 schema."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

from jsonschema import Draft202012Validator

from tools.api_manifest import generate
from tools.api_manifest.audit_rust_exports import _canonical_export_name
from tools.api_manifest.generate import (
    ManifestError,
    _validate_operation_dependencies,
    _validate_operation_inventory,
    _validate_references,
    _validate_schema,
    _validate_v2_readiness,
)

ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"
CHECKED_IN_MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REGISTRY_FIXTURE_PATH = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "public-api-v1.json"
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"
RUST_EXPORT_REPORT_DIR = ROOT / "specs" / "043-public-api-bindings" / "reports"
RUST_EXPORT_REPORT_PATH = max(RUST_EXPORT_REPORT_DIR.glob("rust-export-report-*.json"))

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
EXPECTED_RELEASE_SHARED_CAPABILITIES = {
    "generic_ttlv_model",
    "public_binding_surface",
    "rust_transport_exchange",
    "ttlv_wire_codec",
    "vendor_extension_registry",
    "ffi_runtime_metadata",
    "discover_versions_request_item",
    "opaque_register_context",
    "client_batch_model",
    "client_execution",
    "production_transport",
    "api_level_discovery",
    "server_generated_creation",
    "crypto_response_ttlv",
    "usage_allocation_guard",
}
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
    ("internalCHeader", "tests/fixtures/api/generated/kmipkit_internal.h"),
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
    manifest["errorCategories"] = [
        {**category, "mappingState": "ready"}
        for category in manifest["errorCategories"]
    ]
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

    def _approved_exclusions_by_key(self) -> dict[tuple[str, str], dict[str, object]]:
        return {
            (exclusion["packageId"], exclusion["exportName"]): exclusion
            for exclusion in self.checked_in_manifest["approvedExclusions"]
        }

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

    def test_java_overload_signatures_are_only_valid_on_java_members(self) -> None:
        declaration = copy.deepcopy(next(
            row for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == "T052:request:constructor:java"
        ))
        declaration["javaSignatures"] = [
            "public static org.kmipkit.extensions.ClientBatchItem joinSplitKey(long, "
            "java.util.List<org.kmipkit.extensions.SplitKeyReference>, java.lang.Long, "
            "org.kmipkit.ttlv.TtlvStructure, org.kmipkit.ttlv.TtlvStructure)",
            "public static org.kmipkit.extensions.ClientBatchItem joinSplitKey(long, "
            "java.util.List<org.kmipkit.extensions.SplitKeyReference>)",
        ]
        manifest = copy.deepcopy(self.checked_in_manifest)
        manifest["publicDeclarations"] = [declaration]
        self.assertTrue(self.validator.is_valid(manifest))

        malformed = copy.deepcopy(declaration)
        malformed["kind"] = "type"
        manifest["publicDeclarations"] = [malformed]
        self.assertFalse(self.validator.is_valid(manifest))

        malformed = copy.deepcopy(declaration)
        malformed["javaSignatures"].append(malformed["javaSignatures"][0])
        manifest["publicDeclarations"] = [malformed]
        self.assertFalse(self.validator.is_valid(manifest))

    def test_rust_type_schema_accepts_mutable_pointer_to_pointer_c_abi_output(self) -> None:
        validator = Draft202012Validator(self.schema["$defs"]["rustType"])

        self.assertTrue(validator.is_valid("*mut *mut kmipkit_ttlv_structure_t"))

    def test_c_parameter_schema_accepts_counted_fixed_width_input_arrays(self) -> None:
        validator = Draft202012Validator(
            {"$defs": self.schema["$defs"], "$ref": "#/$defs/cParameter"}
        )
        parameter = {
            "name": "functions",
            "kind": "input-array",
            "type": "const uint32_t *",
            "elementType": "uint32_t",
            "ownership": "borrowed",
            "countParameter": "function_count",
            "nullable": True,
        }

        self.assertTrue(validator.is_valid(parameter))

    def test_requirement_ids_accept_referenced_kmip_7_12_sections(self) -> None:
        for definition in ("requirementId", "v2RequirementId"):
            validator = Draft202012Validator(self.schema["$defs"][definition])
            self.assertTrue(validator.is_valid("KMIPKIT-REQ-SPEC-7.12-003"))
            self.assertTrue(validator.is_valid("KMIPKIT-REQ-SPEC-7.12-005-001"))
        self.assertFalse(validator.is_valid("*mut *mut *mut kmipkit_ttlv_structure_t"))

    def test_blocked_condition_schema_round_trips_lifetime_and_trait_bounds(self) -> None:
        manifest = _valid_v2_manifest()
        condition = {
            "subjectBindings": {"T": "kmipkit_protocol::View<'a>"},
            "predicates": [
                {"kind": "outlives", "subject": "T", "lifetime": "'static"},
                {
                    "kind": "traitBound",
                    "subject": "T",
                    "traitPath": ["core", "marker", "Sized"],
                    "arguments": [],
                    "modifier": "maybe",
                },
            ],
        }
        declaration = {
            "declarationId": "rust-observed-test-lifetime-bound",
            "packageId": "rust-kmipkit-protocol",
            "kind": "associated-item",
            "exportName": "<kmipkit_protocol::View<'a> as core::any::Any>::type_id",
            "capabilityId": "public_binding_surface",
            "requirementIds": ["KMIPKIT-0043-FR-021"],
            "declarationState": "blocked",
            "parityScope": "four-language",
            "candidateSpecId": "KMIPKIT-0043",
            "blockedReason": "The generic lifetime condition is not a reviewed API mapping.",
            "condition": condition,
        }
        manifest["publicDeclarations"].append(declaration)

        encoded = json.dumps(manifest, sort_keys=True)
        round_tripped = json.loads(encoded)
        self.assertEqual(round_tripped["publicDeclarations"][-1]["condition"], condition)
        try:
            _validate_schema(round_tripped, self.schema)
        except ManifestError as error:
            self.fail(f"generator rejected a valid blocked condition: {error}")
        errors = list(self.validator.iter_errors(round_tripped))
        self.assertTrue(
            len(errors) == 0,
            "schema rejected exact blocked condition at "
            f"{list(errors[0].absolute_path) if errors else []}",
        )

        malformed = json.loads(json.dumps(manifest))
        malformed["publicDeclarations"][-1]["condition"]["predicates"][0].pop("lifetime")
        self.assertTrue(list(self.validator.iter_errors(malformed)))

        malformed = json.loads(json.dumps(manifest))
        malformed["publicDeclarations"][-1]["condition"]["unexpected"] = True
        self.assertTrue(list(self.validator.iter_errors(malformed)))

        malformed = json.loads(json.dumps(manifest))
        malformed["publicDeclarations"][-1]["condition"]["predicates"][1]["modifier"] = "maybe-ish"
        self.assertTrue(list(self.validator.iter_errors(malformed)))
        with self.assertRaises(ManifestError):
            _validate_schema(malformed, self.schema)

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

    def test_release_gate_requires_all_eight_package_families_to_use_release_line(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        generate._validate_release_package_versions(manifest, ROOT)

        mutations = (
            lambda value: value["publicPackages"].pop(),
            lambda value: value["publicPackages"][-1].update({"id": "rust-kmipkit"}),
            lambda value: value["publicPackages"][0].update({"versionSource": "1.0.0"}),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate.__code__.co_firstlineno):
                malformed = copy.deepcopy(self.checked_in_manifest)
                mutate(malformed)
                with self.assertRaises(ManifestError):
                    generate._validate_release_package_versions(malformed, ROOT)

    def test_release_gate_rejects_rust_package_name_drift(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        rust_package = next(
            package
            for package in manifest["publicPackages"]
            if package["id"] == "rust-kmipkit-ttlv"
        )
        rust_package["packageName"] = "kmipkit-ttlv-unexpected"

        with self.assertRaises(ManifestError):
            generate._validate_release_package_versions(manifest, ROOT)

    def test_release_gate_rejects_python_package_name_drift(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        python_package = next(
            package
            for package in manifest["publicPackages"]
            if package["id"] == "python-kmipkit"
        )
        python_package["packageName"] = "kmipkit-unexpected"

        with self.assertRaises(ManifestError):
            generate._validate_release_package_versions(manifest, ROOT)

    def test_release_gate_rejects_java_coordinate_drift(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        java_package = next(
            package
            for package in manifest["publicPackages"]
            if package["id"] == "java-kmipkit"
        )
        java_package["packageName"] = "org.kmipkit:unexpected-artifact"

        with self.assertRaises(ManifestError):
            generate._validate_release_package_versions(manifest, ROOT)

    def test_release_gate_rejects_c_distribution_identity_drift(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        c_package = next(
            package
            for package in manifest["publicPackages"]
            if package["id"] == "c-kmipkit"
        )
        c_package["packageName"] = "kmipkit-c-unverified"

        with self.assertRaises(ManifestError):
            generate._validate_release_package_versions(manifest, ROOT)

    def test_release_gate_rejects_rust_c_python_and_java_metadata_identity_drift(self) -> None:
        metadata_paths = (
            "Cargo.toml",
            "crates/kmipkit-ttlv/Cargo.toml",
            "crates/kmipkit-protocol/Cargo.toml",
            "crates/kmipkit-transport/Cargo.toml",
            "crates/kmipkit-client/Cargo.toml",
            "crates/kmipkit/Cargo.toml",
            "crates/kmipkit-ffi/Cargo.toml",
            "bindings/c/package.toml",
            "bindings/python/pyproject.toml",
            "bindings/java/build.gradle.kts",
            "bindings/java/pom.xml",
        )
        mutations = (
            (
                "crates/kmipkit-ttlv/Cargo.toml",
                'name = "kmipkit-ttlv"',
                'name = "kmipkit-ttlv-unexpected"',
            ),
            (
                "bindings/c/package.toml",
                'name = "kmipkit-c"',
                'name = "kmipkit-c-unexpected"',
            ),
            (
                "bindings/python/pyproject.toml",
                'name = "kmipkit"\n',
                'name = "kmipkit-unexpected"\n',
            ),
            (
                "bindings/java/pom.xml",
                "<artifactId>kmipkit-java</artifactId>",
                "<artifactId>unexpected-artifact</artifactId>",
            ),
        )
        for relative, expected, replacement in mutations:
            with self.subTest(path=relative), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for metadata_path in metadata_paths:
                    destination = root / metadata_path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(ROOT / metadata_path, destination)
                source = root / relative
                original = source.read_text(encoding="utf-8")
                mutated = original.replace(expected, replacement, 1)
                self.assertNotEqual(mutated, original, relative)
                source.write_text(mutated, encoding="utf-8")

                with self.assertRaises(ManifestError):
                    generate._validate_release_package_versions(
                        self.checked_in_manifest, root
                    )

    def test_release_gate_rejects_whitespace_only_java_coordinate_parts(self) -> None:
        metadata_paths = (
            "Cargo.toml",
            "crates/kmipkit-ttlv/Cargo.toml",
            "crates/kmipkit-protocol/Cargo.toml",
            "crates/kmipkit-transport/Cargo.toml",
            "crates/kmipkit-client/Cargo.toml",
            "crates/kmipkit/Cargo.toml",
            "crates/kmipkit-ffi/Cargo.toml",
            "bindings/python/pyproject.toml",
            "bindings/java/build.gradle.kts",
            "bindings/java/pom.xml",
        )
        mutations = (
            (
                "   :kmipkit-java",
                "group = \"org.kmipkit\"",
                'group = "   "',
                "<groupId>org.kmipkit</groupId>",
                "<groupId>   </groupId>",
            ),
            (
                "org.kmipkit:   ",
                "<artifactId>kmipkit-java</artifactId>",
                "<artifactId>   </artifactId>",
            ),
        )
        for mutation in mutations:
            package_name, *replacements = mutation
            with self.subTest(package_name=package_name, replacements=replacements), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for metadata_path in metadata_paths:
                    destination = root / metadata_path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(ROOT / metadata_path, destination)
                for index in range(0, len(replacements), 2):
                    relative = (
                        "bindings/java/build.gradle.kts"
                        if replacements[index].startswith("group =")
                        else "bindings/java/pom.xml"
                    )
                    source = root / relative
                    original = source.read_text(encoding="utf-8")
                    mutated = original.replace(
                        replacements[index], replacements[index + 1], 1
                    )
                    self.assertNotEqual(mutated, original, relative)
                    source.write_text(mutated, encoding="utf-8")
                manifest = copy.deepcopy(self.checked_in_manifest)
                java_package = next(
                    package
                    for package in manifest["publicPackages"]
                    if package["id"] == "java-kmipkit"
                )
                java_package["packageName"] = package_name

                with self.assertRaises(ManifestError):
                    generate._validate_release_package_versions(manifest, root)

    def test_release_gate_requires_the_complete_shared_capability_inventory(self) -> None:
        operations = [
            {"apiState": "verified"}
            for _ in range(generate._EXPECTED_CLIENT_1_0_OPERATION_COUNT)
        ]
        capabilities = [
            {"capabilityId": capability_id, "capabilityState": "verified"}
            for capability_id in EXPECTED_RELEASE_SHARED_CAPABILITIES
        ]
        declarations = []
        generate._require_complete_readiness(operations, capabilities, declarations)

        with self.assertRaisesRegex(ManifestError, "capability inventory"):
            generate._require_complete_readiness(
                operations,
                [row for row in capabilities if row["capabilityId"] != "usage_allocation_guard"],
                declarations,
            )

    def test_requirement_references_resolve_for_capabilities_and_api_levels(self) -> None:
        mutations = (
            lambda manifest: manifest["capabilities"][0]["requirementIds"].append(
                "KMIPKIT-9999-FR-999"
            ),
            lambda manifest: manifest["apiLevels"][0]["requirementIds"].append(
                "KMIPKIT-9999-FR-999"
            ),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate.__code__.co_firstlineno):
                malformed = copy.deepcopy(self.checked_in_manifest)
                mutate(malformed)
                with self.assertRaisesRegex(ManifestError, "unknown requirement ID"):
                    _validate_references(malformed)

    def test_capability_and_api_level_requirements_are_unique_and_sorted(self) -> None:
        mutations = (
            lambda manifest: manifest["capabilities"][0]["requirementIds"].append(
                manifest["capabilities"][0]["requirementIds"][0]
            ),
            lambda manifest: manifest["capabilities"][0]["requirementIds"].reverse(),
            lambda manifest: manifest["apiLevels"][0]["requirementIds"].append(
                manifest["apiLevels"][0]["requirementIds"][0]
            ),
            lambda manifest: manifest["apiLevels"][0]["requirementIds"].reverse(),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate.__code__.co_firstlineno):
                malformed = copy.deepcopy(self.checked_in_manifest)
                mutate(malformed)
                with self.assertRaisesRegex(
                    ManifestError, "requirement IDs must be unique and lexicographically sorted"
                ):
                    _validate_references(malformed)

    def test_release_gate_runs_all_export_audits_and_fails_closed(self) -> None:
        with mock.patch(
            "tools.api_manifest.generate.subprocess.run",
            return_value=subprocess.CompletedProcess([], 0),
        ) as run:
            generate._validate_release_export_inventories(ROOT)

        commands = [call.args[0] for call in run.call_args_list]
        self.assertEqual(len(commands), 2)
        self.assertTrue(all("--check" in command for command in commands))
        self.assertTrue(any("audit_rust_exports.py" in " ".join(command) for command in commands))
        self.assertTrue(any("audit_binding_exports.py" in " ".join(command) for command in commands))

        with mock.patch(
            "tools.api_manifest.generate.subprocess.run",
            return_value=subprocess.CompletedProcess([], 1),
        ):
            with self.assertRaisesRegex(ManifestError, "export inventory"):
                generate._validate_release_export_inventories(ROOT)

    def test_release_gate_checks_rust_python_and_java_package_version_sources(self) -> None:
        version_paths = (
            "Cargo.toml",
            "crates/kmipkit-ttlv/Cargo.toml",
            "crates/kmipkit-protocol/Cargo.toml",
            "crates/kmipkit-transport/Cargo.toml",
            "crates/kmipkit-client/Cargo.toml",
            "crates/kmipkit/Cargo.toml",
            "crates/kmipkit-ffi/Cargo.toml",
            "bindings/python/pyproject.toml",
            "bindings/java/build.gradle.kts",
            "bindings/java/pom.xml",
        )
        mutations = (
            (
                "Cargo.toml",
                'version = "1.0.0"',
                'version = "9.9.9"',
            ),
            (
                "crates/kmipkit-client/Cargo.toml",
                "version.workspace = true",
                'version = "1.0.0"',
            ),
            (
                "crates/kmipkit-ffi/Cargo.toml",
                "version.workspace = true",
                'version = "1.0.0"',
            ),
            (
                "bindings/python/pyproject.toml",
                'version = "1.0.0"',
                'version = "9.9.9"',
            ),
            (
                "bindings/java/build.gradle.kts",
                'version = "1.0.0"',
                'version = "9.9.9"',
            ),
            (
                "bindings/java/pom.xml",
                "<version>1.0.0</version>",
                "<version>9.9.9</version>",
            ),
        )
        for relative, expected, replacement in mutations:
            with self.subTest(path=relative), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for version_path in version_paths:
                    destination = root / version_path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(ROOT / version_path, destination)
                source = root / relative
                original = source.read_text(encoding="utf-8")
                mutated = original.replace(expected, replacement, 1)
                self.assertNotEqual(mutated, original, relative)
                source.write_text(mutated, encoding="utf-8")
                with self.assertRaises(ManifestError):
                    generate._validate_release_package_versions(self.checked_in_manifest, root)

    def test_release_gate_rejects_oversized_package_version_source(self) -> None:
        version_paths = (
            "Cargo.toml",
            "crates/kmipkit-ttlv/Cargo.toml",
            "crates/kmipkit-protocol/Cargo.toml",
            "crates/kmipkit-transport/Cargo.toml",
            "crates/kmipkit-client/Cargo.toml",
            "crates/kmipkit/Cargo.toml",
            "crates/kmipkit-ffi/Cargo.toml",
            "bindings/python/pyproject.toml",
            "bindings/java/build.gradle.kts",
            "bindings/java/pom.xml",
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in version_paths:
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, destination)
            oversized = root / "bindings/java/build.gradle.kts"
            oversized.write_text(
                'version = "1.0.0"\n' + (" " * (1024 * 1024 + 1)),
                encoding="utf-8",
            )

            with self.assertRaises(ManifestError):
                generate._validate_release_package_versions(self.checked_in_manifest, root)

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
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.version"),
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.capabilityIsAvailable"),
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


    def test_runtime_metadata_namespace_exports_remain_blocked_under_0043(self) -> None:
        declarations = {
            (entry["packageId"], entry["exportName"]): entry
            for entry in self.checked_in_manifest["publicDeclarations"]
        }
        blocked_namespace_candidates = {
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata"): "KMIPKIT-0043",
            ("python-kmipkit", "kmipkit.metadata"): "KMIPKIT-0043",
        }
        for key, candidate_spec_id in blocked_namespace_candidates.items():
            with self.subTest(declaration=key):
                declaration = declarations[key]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertEqual(declaration["candidateSpecId"], candidate_spec_id)
                self.assertNotIn("languageMappings", declaration)

    def test_c_documentation_rejects_comment_and_line_splice_injection(self) -> None:
        unsafe_documentation = "*\\\r/#define KMIPKIT_INJECTED 1/*"
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        function = next(
            record for record in manifest["functions"]
            if record["id"] == "ffi_last_error_get"
        )
        function["c"]["documentation"] = [unsafe_documentation]

        with self.assertRaises(ManifestError):
            _validate_schema(manifest, self.schema)

    def test_checked_in_manifest_retains_every_registry_slice_record(self) -> None:
        manifest = self.checked_in_manifest
        registry = json.loads(REGISTRY_FIXTURE_PATH.read_text(encoding="utf-8"))
        for field in ("types", "errorCategories", "limits", "security"):
            with self.subTest(field=field):
                if field == "types":
                    actual = [
                        record for record in manifest[field]
                        if any(
                            requirement.startswith("KMIPKIT-0012-")
                            for requirement in record["requirementIds"]
                        )
                    ]
                    expected = json.loads(json.dumps(registry[field]))
                    batch_item_type = next(row for row in actual if row["id"] == "ClientBatchItem")
                    expected_batch_item_type = next(
                        row for row in expected if row["id"] == "ClientBatchItem"
                    )
                    expected_batch_item_type["java"]["package"] = batch_item_type["java"]["package"]
                    expected_batch_item_type["python"]["module"] = batch_item_type["python"]["module"]
                elif field == "errorCategories":
                    expected = registry[field]
                    registry_ids = {row["id"] for row in expected}
                    actual = [
                        {
                            key: value
                            for key, value in row.items()
                            if key in expected[0]
                        }
                        for row in manifest[field]
                        if row["id"] in registry_ids
                    ]
                else:
                    actual = manifest[field]
                    expected = registry[field]
                self.assertEqual(actual, expected)
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
                elif function["id"] == "client_batch_item_discover_versions":
                    expected_function = json.loads(json.dumps(function))
                    expected_function["requirementIds"] = manifest_functions[function["id"]]["requirementIds"]
                    expected_function["rust"]["module"] = "kmipkit_client::ClientBatchItem"
                    expected_function["java"]["jniSymbol"] = (
                        "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDiscoverVersions"
                    )
                    self.assertEqual(manifest_functions[function["id"]], expected_function)
                elif function["id"] in {
                    "ttlv_value_structure",
                    "ttlv_item_create",
                    "ttlv_structure_with_item",
                }:
                    expected_function = json.loads(json.dumps(function))
                    expected_function["c"]["documentation"] = {
                        "ttlv_value_structure": [
                            "Transfers ownership of structure after the output slot and limits handle are valid. If validation fails before transfer, the caller retains structure; after transfer, the caller must not use or release it, even if this function returns an error."
                        ],
                        "ttlv_item_create": [
                            "Transfers ownership of value only after the output slot, tag, and limits are valid. If validation fails before transfer, the caller retains value; after transfer, the caller must not use or release it, even if this function returns an error."
                        ],
                        "ttlv_structure_with_item": [
                            "Transfers ownership of each input structure and item as it is accepted. Validation errors before transfer leave that handle caller-owned; after transfer, the caller must not use or release it, even if later processing returns an error."
                        ],
                    }[function["id"]]
                    self.assertEqual(manifest_functions[function["id"]], expected_function)
                else:
                    self.assertEqual(manifest_functions[function["id"]], function)
        added_versioned_type = next(
            row for row in manifest["types"] if row["id"] == "ExtensionRegistryLimitsV1"
        )
        self.assertEqual(added_versioned_type["c"]["name"], "kmipkit_extension_registry_limits_v1_t")
        self.assertEqual(added_versioned_type["requirementIds"], ["KMIPKIT-0043-FR-007"])
        self.assertTrue(set(registry["requirementIds"]) <= set(manifest["requirementIds"]))
        outputs = manifest["generatedOutputs"]
        if isinstance(outputs, list):
            paths = {entry["path"] for entry in outputs}
            self.assertTrue(set(LEGACY_GENERATED_OUTPUTS.values()) <= paths)
        else:
            self.assertEqual(outputs, LEGACY_GENERATED_OUTPUTS)

    def test_discover_versions_function_maps_the_public_item_builder(self) -> None:
        function = next(
            record for record in self.checked_in_manifest["functions"]
            if record["id"] == "client_batch_item_discover_versions"
        )

        self.assertEqual(function["rust"]["module"], "kmipkit_client::ClientBatchItem")
        self.assertEqual(
            function["java"]["jniSymbol"],
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDiscoverVersions",
        )
        self.assertTrue(
            {
                "KMIPKIT-0007-FR-001",
                "KMIPKIT-0007-FR-002",
                "KMIPKIT-0043-FR-004",
                "KMIPKIT-0043-FR-021",
            } <= set(function["requirementIds"])
        )

    def test_checked_in_operation_inventory_matches_the_normative_catalog(self) -> None:
        _validate_operation_inventory(self.checked_in_manifest, self.catalog)

    def test_async_operation_owners_match_the_approved_release_spec(self) -> None:
        asynchronous_operations = [
            operation
            for operation in self.checked_in_manifest["operations"]
            if operation.get("ownerDependency", {}).get("specId") == "KMIPKIT-0009"
        ]
        self.assertEqual(
            [operation["name"] for operation in asynchronous_operations],
            ["Cancel", "Poll", "Process", "Query Asynchronous Requests"],
        )
        _validate_operation_dependencies(self.checked_in_manifest["operations"], ROOT)

    def test_checked_in_manifest_resolves_all_public_declaration_mappings(self) -> None:
        _validate_v2_readiness(self.checked_in_manifest, ROOT)

    def test_error_categories_resolve_to_canonical_language_exports(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        manifest["errorCategories"][0]["pythonException"] = "kmipkit.errors.MissingError"

        with self.assertRaisesRegex(ManifestError, "error category references an undeclared language export"):
            _validate_references(manifest)

    def test_error_category_rejects_a_stale_ready_declaration_mapping(self) -> None:
        manifest = json.loads(json.dumps(self.checked_in_manifest))
        error = next(
            item for item in manifest["errorCategories"]
            if item["id"] == "resource_limit"
        )
        declaration = next(
            item for item in manifest["publicDeclarations"]
            if item["packageId"] == "python-kmipkit"
            and item["exportName"] == error["pythonException"]
        )
        declaration["languageMappings"]["python"]["name"] = "kmipkit.errors.StaleError"

        with self.assertRaisesRegex(ManifestError, "canonical language export"):
            _validate_references(manifest)

    def test_checked_in_error_categories_keep_real_python_names_and_status_values(self) -> None:
        categories = {
            item["id"]: item for item in self.checked_in_manifest["errorCategories"]
        }
        self.assertEqual(
            [categories[name]["pythonException"] for name in (
                "compatibility_mismatch", "duplicate_key", "invalid_identity"
            )],
            [
                "kmipkit.errors.CompatibilityMismatchError",
                "kmipkit.errors.DuplicateKeyError",
                "kmipkit.errors.InvalidIdentityError",
            ],
        )
        self.assertEqual(
            [categories[name]["cValue"] for name in (
                "compatibility_mismatch", "duplicate_key", "invalid_identity"
            )],
            [1, 2, 3],
        )

    def test_checked_in_internal_status_is_stable_but_parity_remains_blocked(self) -> None:
        categories = {
            item["id"]: item for item in self.checked_in_manifest["errorCategories"]
        }
        self.assertIn("internal", categories)
        internal = categories["internal"]
        self.assertEqual(internal["cCode"], "KMIPKIT_ERROR_INTERNAL")
        self.assertEqual(internal["cValue"], 7)
        self.assertEqual(internal["javaException"], "org.kmipkit.KmipException")
        self.assertEqual(internal["pythonException"], "kmipkit.errors.KmipKitError")
        self.assertEqual(internal["mappingState"], "blocked")
        self.assertTrue(internal["stableCHeaderCode"])
        self.assertIn("no public rust", internal["blockedReason"].lower())

    def test_ready_api_projection_omits_blocked_error_taxonomy(self) -> None:
        projected = generate._ready_api_projection(self.checked_in_manifest)
        projected_ids = {item["id"] for item in projected["errorCategories"]}
        expected_ids = {
            item["id"]
            for item in self.checked_in_manifest["errorCategories"]
            if item["mappingState"] == "ready"
        }

        self.assertEqual(expected_ids, projected_ids)
        self.assertNotIn("internal", projected_ids)
        self.assertNotIn("compatibility_mismatch", projected_ids)
        self.assertIn("resource_limit", projected_ids)
        self.assertIn("protocol", projected_ids)
        self.assertIn("transport", projected_ids)
        self.assertIn("server_result", projected_ids)

    def test_c_error_status_constant_remains_in_header_when_parity_mapping_is_blocked(self) -> None:
        header = generate.render_c_header(self.checked_in_manifest)

        self.assertIn("#define KMIPKIT_ERROR_INTERNAL ((int32_t)7)", header)

    def test_c_header_does_not_emit_unreleased_blocked_error_statuses(self) -> None:
        header = generate.render_c_header(self.checked_in_manifest)
        blocked_codes = {
            item["cCode"]
            for item in self.checked_in_manifest["errorCategories"]
            if not item.get("stableCHeaderCode", False)
            and not any(
                row["packageId"] == "c-kmipkit"
                and row["exportName"] == item["cCode"]
                and row["declarationState"] in {"declared", "verified"}
                for row in self.checked_in_manifest["publicDeclarations"]
            )
        }

        for code in blocked_codes:
            with self.subTest(code=code):
                self.assertNotIn(f"#define {code} ", header)

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
            "kmipkit.extensions.with_extension",
            "org.kmipkit.NativeExtensionRegistry",
            "org.kmipkit.generated.ExtensionRegistryApi",
            "kmipkit.errors.raise_for_status",
        ):
            with self.subTest(export_name=export_name):
                declaration = declarations[export_name]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertIn("candidateSpecId", declaration)
                self.assertIn("blockedReason", declaration)
                self.assertNotIn("languageMappings", declaration)
        ttlv_namespace = self._approved_exclusions_by_key()[
            ("python-kmipkit", "kmipkit.ttlv")
        ]
        self.assertIn("namespace boundary", ttlv_namespace["reason"])
        self.assertEqual(
            ttlv_namespace["approvalSource"]["path"],
            "specs/004-generic-ttlv-model/spec.md",
        )
        generic_python_error = declarations["kmipkit.errors.KmipKitError"]
        self.assertEqual(generic_python_error["declarationState"], "blocked")
        self.assertEqual(generic_python_error["capabilityId"], "ffi_runtime_metadata")
        self.assertEqual(generic_python_error["candidateSpecId"], "KMIPKIT-0043")
        self.assertNotIn("languageMappings", generic_python_error)
        generic_java_error = declarations["org.kmipkit.KmipException"]
        self.assertEqual(generic_java_error["declarationState"], "blocked")
        self.assertEqual(generic_java_error["capabilityId"], "ffi_runtime_metadata")
        self.assertEqual(generic_java_error["candidateSpecId"], "KMIPKIT-0043")
        self.assertNotIn("languageMappings", generic_java_error)
        batch_item = declarations["kmipkit.extensions.ClientBatchItem"]
        self.assertEqual(batch_item["declarationState"], "declared")
        self.assertIn("languageMappings", batch_item)

    def test_generic_ttlv_standard_trait_exports_have_exact_exclusions(self) -> None:
        mapped_ttlv_types = {
            declaration["exportName"]
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration["packageId"] == "rust-kmipkit-ttlv"
            and declaration["capabilityId"] == "generic_ttlv_model"
            and declaration["declarationState"] in {"declared", "verified"}
            and declaration["kind"] == "type"
        }
        report = json.loads(
            (RUST_EXPORT_REPORT_DIR / "rust-export-report-2026-10-30.json").read_text(
                encoding="utf-8"
            )
        )
        observed_trait_exports = {
            ("rust-kmipkit-ttlv", declaration["exportName"])
            for declaration in report["declarations"] + report["conditionalDeclarations"]
            if declaration["packageId"] == "rust-kmipkit-ttlv"
            and declaration["kind"] == "associated-item"
            and any(
                declaration["exportName"].startswith(f"<{type_name} as ")
                for type_name in mapped_ttlv_types
            )
        }
        self.assertEqual(len(observed_trait_exports), 92)

        exclusions = self._approved_exclusions_by_key()
        missing = sorted(observed_trait_exports - exclusions.keys())
        self.assertFalse(missing, f"missing exact TTLV trait exclusions: {missing[:5]}")
        for key in sorted(observed_trait_exports):
            exclusion = exclusions[key]
            self.assertIn("standard Rust trait implementation", exclusion["reason"])
            self.assertEqual(
                exclusion["approvalSource"]["path"],
                "specs/004-generic-ttlv-model/spec.md",
            )

    def test_profile_test_support_selection_helpers_have_exact_exclusions(self) -> None:
        expected = {
            ("rust-kmipkit", "kmipkit::ProfileValidationSelection::for_test"),
            (
                "rust-kmipkit-client",
                "kmipkit_client::ProfileValidationSelection::for_test",
            ),
            (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::ProfileValidationSelection::for_test",
            ),
        }
        exclusions = self._approved_exclusions_by_key()
        missing = sorted(expected - exclusions.keys())
        self.assertFalse(missing, f"missing test-support-only exclusions: {missing}")
        for key in sorted(expected):
            exclusion = exclusions[key]
            self.assertIn("test-support", exclusion["reason"])
            self.assertEqual(
                exclusion["approvalSource"]["path"],
                "specs/010-profile-conformance/spec.md",
            )

    def test_core_error_owner_exports_are_mapped_or_exactly_excluded(self) -> None:
        unresolved = [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0003"
        ]
        self.assertEqual(
            len(unresolved),
            0,
            "KMIPKIT-0003 exports must be mapped or specifically excluded: "
            f"{[row['exportName'] for row in unresolved[:5]]}",
        )

        expected_mappings = {
            ("rust-kmipkit-protocol", "kmipkit_protocol::KmipOperationResult"): (
                "kmipkit_client_batch_item_response_t",
                "org.kmipkit.extensions.ClientBatchItemResult",
                "kmipkit.client.BatchItemResult",
            ),
            ("rust-kmipkit", "kmipkit::KmipOperationResult"): (
                "kmipkit_client_batch_item_response_t",
                "org.kmipkit.extensions.ClientBatchItemResult",
                "kmipkit.client.BatchItemResult",
            ),
            ("rust-kmipkit", "kmipkit::KmipOperationResult::status"): (
                "kmipkit_client_batch_item_response_t",
                "org.kmipkit.extensions.ClientBatchItemResult",
                "kmipkit.client.BatchItemResult",
            ),
            ("rust-kmipkit", "kmipkit::KmipOperationResult::reason"): (
                "kmipkit_client_batch_item_response_t",
                "org.kmipkit.extensions.ClientBatchItemResult",
                "kmipkit.client.BatchItemResult",
            ),
            ("rust-kmipkit", "kmipkit::KmipOperationResult::message"): (
                "kmipkit_client_batch_item_metadata_v2_t.result_message_data",
                "org.kmipkit.extensions.ClientBatchItemMetadataV2.resultMessage",
                "kmipkit.client.BatchItemMetadataV2.result_message",
            ),
        }
        expected_owners = {
            key: (
                "client_execution",
                "specs/007-client-execution/spec.md",
                "KMIPKIT-0007-FR-001",
            )
            for key in expected_mappings
        }
        expected_owners[("rust-kmipkit", "kmipkit::KmipOperationResult::message")] = (
            "server_generated_creation",
            "specs/014-server-generated-creation/spec.md",
            "KMIPKIT-0014-FR-008",
        )
        declarations = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        for key, targets in expected_mappings.items():
            declaration = declarations[key]
            capability_id, approval_path, owner_requirement_id = expected_owners[key]
            self.assertEqual(declaration["declarationState"], "declared")
            self.assertEqual(declaration["capabilityId"], capability_id)
            self.assertEqual(declaration["approvalSource"]["path"], approval_path)
            self.assertIn("KMIPKIT-0003-FR-", " ".join(declaration["requirementIds"]))
            self.assertIn(owner_requirement_id, declaration["requirementIds"])
            self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
            self.assertEqual(
                [declaration["languageMappings"][language]["name"]
                 for language in ("c", "java", "python")],
                list(targets),
            )

        exclusions = self._approved_exclusions_by_key()
        owner_exclusions = [
            exclusion
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["approvalSource"]["path"] == "specs/003-core-types-errors/spec.md"
        ]
        owner_keys = sorted(
            f"{exclusion['packageId']}\0{exclusion['exportName']}"
            for exclusion in owner_exclusions
        )
        self.assertEqual(len(owner_keys), 434)
        self.assertEqual(
            hashlib.sha256(("\n".join(owner_keys) + "\n").encode()).hexdigest(),
            "0ab680a0592918b88cc098f7bd424ae6db4a6f94728078c51c9007c019c696cf",
        )
        for exclusion in owner_exclusions:
            self.assertRegex(exclusion["evidence"], r"KMIPKIT-0003-FR-00[1-8]")
            self.assertEqual(exclusion["approvalSource"]["status"], "Approved")

        model_error_exclusions = [
            exclusion
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] == "rust-kmipkit-ttlv"
            and exclusion["approvalSource"]["path"] == "specs/004-generic-ttlv-model/spec.md"
            and (
                exclusion["exportName"] == "kmipkit_ttlv::ModelError"
                or exclusion["exportName"].startswith(
                    "<kmipkit_ttlv::ModelError as "
                )
            )
        ]
        model_error_keys = sorted(
            f"{exclusion['packageId']}\0{exclusion['exportName']}"
            for exclusion in model_error_exclusions
        )
        self.assertEqual(len(model_error_keys), 27)
        self.assertEqual(
            hashlib.sha256(("\n".join(model_error_keys) + "\n").encode()).hexdigest(),
            "f604cdc7477795da34c3a100e1543d8d3481665462cbe53b3990c01386228f72",
        )
        for exclusion in model_error_exclusions:
            self.assertIn("KMIPKIT-0004-FR-013", exclusion["evidence"])
            self.assertEqual(exclusion["approvalSource"]["status"], "Approved")

    def test_generic_ttlv_owner_has_no_unresolved_public_declarations(self) -> None:
        expected_mappings = {
            "kmipkit_ttlv::Value::item_type": (
                "kmipkit_ttlv_value_view_type",
                "org.kmipkit.ttlv.TtlvValueView.itemType",
                "kmipkit.ttlv.ttlv_value_view_type",
            ),
            "kmipkit_ttlv::ValueView::Structure": (
                "kmipkit_ttlv_value_view_structure",
                "org.kmipkit.ttlv.TtlvValueView.structure",
                "kmipkit.ttlv.ttlv_value_view_structure",
            ),
            "kmipkit_ttlv::ValueView::Integer": (
                "kmipkit_ttlv_value_view_integer",
                "org.kmipkit.ttlv.TtlvValueView.integer",
                "kmipkit.ttlv.ttlv_value_view_integer",
            ),
            "kmipkit_ttlv::ValueView::LongInteger": (
                "kmipkit_ttlv_value_view_long_integer",
                "org.kmipkit.ttlv.TtlvValueView.longInteger",
                "kmipkit.ttlv.ttlv_value_view_long_integer",
            ),
            "kmipkit_ttlv::ValueView::Enumeration": (
                "kmipkit_ttlv_value_view_enumeration",
                "org.kmipkit.ttlv.TtlvValueView.enumeration",
                "kmipkit.ttlv.ttlv_value_view_enumeration",
            ),
            "kmipkit_ttlv::ValueView::Boolean": (
                "kmipkit_ttlv_value_view_boolean",
                "org.kmipkit.ttlv.TtlvValueView.booleanValue",
                "kmipkit.ttlv.ttlv_value_view_boolean",
            ),
            "kmipkit_ttlv::ValueView::DateTime": (
                "kmipkit_ttlv_value_view_date_time",
                "org.kmipkit.ttlv.TtlvValueView.dateTime",
                "kmipkit.ttlv.ttlv_value_view_date_time",
            ),
            "kmipkit_ttlv::ValueView::Interval": (
                "kmipkit_ttlv_value_view_interval",
                "org.kmipkit.ttlv.TtlvValueView.interval",
                "kmipkit.ttlv.ttlv_value_view_interval",
            ),
            "kmipkit_ttlv::ValueView::DateTimeExtended": (
                "kmipkit_ttlv_value_view_date_time_extended",
                "org.kmipkit.ttlv.TtlvValueView.dateTimeExtended",
                "kmipkit.ttlv.ttlv_value_view_date_time_extended",
            ),
        }
        expected_exclusions = {
            "kmipkit.ttlv",
            "kmipkit_ttlv::codec",
            "kmipkit_ttlv::ModelError::RawTagOutOfRange",
            "kmipkit_ttlv::ModelError::TagNotAllocated",
            "kmipkit_ttlv::ModelError::StructureDepthExceeded",
            "kmipkit_ttlv::StructureView::try_clone",
            "kmipkit_ttlv::ValueView::BigInteger",
            "kmipkit_ttlv::ValueView::ByteString",
            "kmipkit_ttlv::ValueView::TextString",
            "kmipkit_ttlv::try_clone_value",
            "kmipkit_ttlv::Item::try_clone",
            "kmipkit_ttlv::ValueView::BigInteger::0",
            "kmipkit_ttlv::ValueView::ByteString::0",
            "kmipkit_ttlv::ValueView::TextString::0",
        }
        expected_keys = {
            ("rust-kmipkit-ttlv", export_name)
            for export_name in (*expected_mappings, *expected_exclusions - {"kmipkit.ttlv"})
        }
        expected_keys.add(("python-kmipkit", "kmipkit.ttlv"))
        declarations = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        exclusions = self._approved_exclusions_by_key()
        unresolved = [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0004"
        ]
        self.assertFalse(
            unresolved,
            "KMIPKIT-0004 public exports must be mapped or specifically excluded: "
            f"{[row['exportName'] for row in unresolved]}",
        )
        self.assertEqual(
            {
                key for key, row in declarations.items()
                if row["packageId"] == "rust-kmipkit-ttlv"
                and row["exportName"] in expected_mappings
            } | {
                key for key in exclusions if key[1] in expected_exclusions
            },
            expected_keys,
        )
        for export_name, targets in expected_mappings.items():
            declaration = declarations[("rust-kmipkit-ttlv", export_name)]
            self.assertEqual(declaration["declarationState"], "declared")
            self.assertEqual(declaration["capabilityId"], "generic_ttlv_model")
            self.assertEqual(
                declaration["requirementIds"],
                [
                    "KMIPKIT-0004-FR-001",
                    "KMIPKIT-0004-FR-008",
                    "KMIPKIT-0004-FR-012",
                    "KMIPKIT-0043-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                ],
            )
            self.assertEqual(
                [declaration["languageMappings"][language]["name"]
                 for language in ("c", "java", "python")],
                list(targets),
            )
        for key in expected_keys - {
            ("rust-kmipkit-ttlv", name) for name in expected_mappings
        }:
            exclusion = exclusions[key]
            self.assertTrue(exclusion["reason"])
            self.assertTrue(exclusion["evidence"])
            self.assertEqual(
                exclusion["approvalSource"]["path"],
                "specs/005-ttlv-wire-codec/spec.md"
                if key == ("rust-kmipkit-ttlv", "kmipkit_ttlv::codec")
                else "specs/004-generic-ttlv-model/spec.md",
            )

    def test_ttlv_codec_owner_has_no_unresolved_public_declarations(self) -> None:
        unresolved = [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0005"
        ]
        self.assertFalse(
            unresolved,
            "KMIPKIT-0005 public exports must be mapped or specifically excluded: "
            f"{[row['exportName'] for row in unresolved[:5]]}",
        )

        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        expected_decode_mappings = {
            "kmipkit_ttlv::codec::decode": (
                "kmipkit_ttlv_decode",
                "org.kmipkit.ttlv.TtlvCodec.decode",
                "kmipkit.ttlv.ttlv_decode",
            ),
            "kmipkit_ttlv::codec::decode_with_limits": (
                "kmipkit_ttlv_decode_with_limits",
                "org.kmipkit.ttlv.TtlvCodec.decodeWithLimits",
                "kmipkit.ttlv.ttlv_decode_with_limits",
            ),
        }
        for export_name, targets in expected_decode_mappings.items():
            declaration = declarations[("rust-kmipkit-ttlv", export_name)]
            self.assertEqual(declaration["declarationState"], "declared")
            self.assertEqual(declaration["capabilityId"], "ttlv_wire_codec")
            self.assertEqual(
                declaration["requirementIds"],
                [
                    "KMIPKIT-0005-FR-004",
                    "KMIPKIT-0005-FR-005",
                    "KMIPKIT-0005-FR-006",
                    "KMIPKIT-0005-FR-007",
                    "KMIPKIT-0005-FR-008",
                    "KMIPKIT-0005-FR-009",
                    "KMIPKIT-0005-FR-010",
                    "KMIPKIT-0043-FR-004",
                    "KMIPKIT-0043-FR-005",
                    "KMIPKIT-0043-FR-021",
                ],
            )
            self.assertEqual(
                [declaration["languageMappings"][language]["name"]
                 for language in ("c", "java", "python")],
                list(targets),
            )

    def test_t090_protocol_version_accessors_map_to_existing_four_language_surface(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        expected = {
            "kmipkit_protocol::ProtocolVersion::from_raw": (
                "kmipkit_protocol_version_v1_t",
                "org.kmipkit.extensions.ProtocolVersion.<init>",
                "kmipkit.client.ProtocolVersion.__init__",
            ),
            "kmipkit_protocol::ProtocolVersion::major": (
                "kmipkit_protocol_version_v1_t.major",
                "org.kmipkit.extensions.ProtocolVersion.major",
                "kmipkit.client.ProtocolVersion.major",
            ),
            "kmipkit_protocol::ProtocolVersion::minor": (
                "kmipkit_protocol_version_v1_t.minor",
                "org.kmipkit.extensions.ProtocolVersion.minor",
                "kmipkit.client.ProtocolVersion.minor",
            ),
        }
        expected_requirements = [
            "KMIPKIT-0006-FR-012",
            "KMIPKIT-0007-FR-002",
            "KMIPKIT-0043-FR-004",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-021",
        ]
        client_execution = next(
            capability
            for capability in self.checked_in_manifest["capabilities"]
            if capability["capabilityId"] == "client_execution"
        )
        expected_capability_exports = {
            "rust": set(expected),
            "c": {targets[0] for targets in expected.values()},
            "java": {targets[1] for targets in expected.values()},
            "python": {targets[2] for targets in expected.values()},
        }
        for language, names in expected_capability_exports.items():
            with self.subTest(capability_language=language):
                self.assertTrue(
                    names.issubset(set(client_execution["mappings"][language])),
                    "the owning capability inventory must list each mapped export",
                )

        for rust_name, (c_name, java_name, python_name) in expected.items():
            with self.subTest(export=rust_name):
                declaration = declarations[rust_name]
                self.assertEqual(declaration["declarationState"], "declared")
                self.assertEqual(declaration["capabilityId"], "client_execution")
                self.assertEqual(declaration["requirementIds"], expected_requirements)
                self.assertEqual(
                    declaration["languageMappings"],
                    {
                        "rust": {
                            "packageId": "rust-kmipkit-protocol",
                            "name": rust_name,
                        },
                        "c": {"packageId": "c-kmipkit", "name": c_name},
                        "java": {"packageId": "java-kmipkit", "name": java_name},
                        "python": {
                            "packageId": "python-kmipkit",
                            "name": python_name,
                        },
                    },
                )

    def test_t090_message_model_rust_trait_exports_have_exact_owner_exclusions(self) -> None:
        unresolved = {
            row["exportName"]
            for row in self.checked_in_manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0006"
            and row["exportName"].startswith("<kmipkit_protocol::")
        }
        self.assertFalse(
            unresolved,
            "Rust standard-trait implementations must be exact exclusions: "
            f"{sorted(unresolved)[:5]}",
        )

        exclusions = [
            row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["approvalSource"]["path"]
            == "specs/006-message-batch-model/spec.md"
            and row["exportName"].startswith("<kmipkit_protocol::")
        ]
        self.assertEqual(len(exclusions), 92)
        for exclusion in exclusions:
            export_name = exclusion["exportName"]
            with self.subTest(export=export_name):
                self.assertTrue(exclusion["reason"])
                self.assertTrue(exclusion["evidence"])
                self.assertEqual(
                    exclusion["approvalSource"]["path"],
                    "specs/006-message-batch-model/spec.md",
                )
                self.assertIn("KMIPKIT-0006-FR-001", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

    def test_t091_discover_versions_rust_trait_exports_have_exact_owner_exclusions(self) -> None:
        prefix = "<kmipkit_protocol::DiscoverVersions"
        unresolved = {
            row["exportName"]
            for row in self.checked_in_manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
            and row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"].startswith(prefix)
        }
        unresolved_concrete = {
            row["exportName"]
            for row in self.checked_in_manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
            and row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"].startswith("kmipkit_protocol::DiscoverVersions")
        }
        exclusions = [
            row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"].startswith(prefix)
        ]
        exclusion_names = {row["exportName"] for row in exclusions}
        direct_declarations = {
            row["exportName"]
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"].startswith("kmipkit_protocol::DiscoverVersions")
        }
        expected_direct_declarations = {
            "kmipkit_protocol::DiscoverVersionsRequest",
            "kmipkit_protocol::DiscoverVersionsRequest::new",
            "kmipkit_protocol::DiscoverVersionsResponse",
            "kmipkit_protocol::DiscoverVersionsResponse::result",
            "kmipkit_protocol::DiscoverVersionsResponse::supported_versions",
        }

        self.assertEqual(expected_direct_declarations, direct_declarations)
        self.assertFalse(
            direct_declarations & exclusion_names,
            "concrete Discover Versions API declarations must remain individually inventoried",
        )
        self.assertFalse(
            unresolved,
            "Discover Versions types, variants, fields, and methods remain inventoried; "
            f"only standard-trait paths may be excluded: {sorted(unresolved)[:5]}",
        )
        self.assertFalse(
            unresolved_concrete,
            "concrete Discover Versions declarations must be mapped or excluded: "
            f"{sorted(unresolved_concrete)}",
        )
        self.assertEqual(65, len(exclusions))
        self.assertEqual(65, len(exclusion_names))
        for exclusion in exclusions:
            with self.subTest(export=exclusion["exportName"]):
                self.assertIn("standard-trait implementation", exclusion["reason"])
                self.assertIn("separately inventoried", exclusion["reason"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/007-client-execution/spec.md",
                        "status": "Approved",
                    },
                    {
                        key: exclusion["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertIn("KMIPKIT-0007-FR-001", exclusion["evidence"])
                self.assertIn("KMIPKIT-0007-FR-002", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                if "DiscoverVersionsError" in exclusion["exportName"]:
                    self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])

    def test_t091_client_execution_standard_traits_have_exact_owner_exclusions(self) -> None:
        target_packages = {"rust-kmipkit", "rust-kmipkit-client"}
        trait_type_prefixes = (
            "kmipkit::Client",
            "kmipkit::PendingOutcome",
            "kmipkit::RequestDeliveryState",
            "kmipkit::Transport",
            "kmipkit_client::Client",
            "kmipkit_client::PendingOutcome",
        )
        rust_report = json.loads(
            (RUST_EXPORT_REPORT_DIR / "rust-export-report-2026-10-30.json").read_text(
                encoding="utf-8"
            )
        )
        observed_traits = {
            (row["packageId"], row["exportName"])
            for row in (
                rust_report["declarations"]
                + rust_report["conditionalDeclarations"]
                + rust_report["excluded"]
            )
            if row["packageId"] in target_packages
            and row["exportName"].startswith("<")
            and row["exportName"].split(" as ", 1)[0][1:].startswith(
                trait_type_prefixes
            )
        }

        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] in target_packages
            and row["approvalSource"]["path"] == "specs/007-client-execution/spec.md"
            and row["exportName"].startswith("<")
        }
        unresolved = sorted(
            (row["packageId"], row["exportName"])
            for row in self.checked_in_manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
            and row["packageId"] in target_packages
            and row["exportName"].startswith("<")
        )
        self.assertFalse(
            unresolved,
            "KMIPKIT-0007 standard traits require exact exclusions while direct types "
            f"stay inventoried; unresolved={unresolved[:5]}",
        )
        self.assertEqual(466, len(exclusions))
        missing = sorted(exclusions.keys() - observed_traits)
        self.assertFalse(
            missing,
            f"standard-trait exclusions must match observed exports exactly: {missing[:5]}",
        )

        directly_accounted_types = {
            (row["packageId"], row["exportName"])
            for row in (
                self.checked_in_manifest["publicDeclarations"]
                + self.checked_in_manifest["approvedExclusions"]
            )
            if not row["exportName"].startswith("<")
        }
        for package_id, export_name in observed_traits:
            type_name = export_name.split(" as ", 1)[0][1:].split("<", 1)[0]
            with self.subTest(type=type_name, package=package_id):
                self.assertIn((package_id, type_name), directly_accounted_types)

        for key in sorted(exclusions):
            exclusion = exclusions[key]
            type_name = key[1].split(" as ", 1)[0][1:].rsplit("::", 1)[-1]
            with self.subTest(export=key[1]):
                self.assertIn("standard-trait implementation", exclusion["reason"])
                self.assertIn("concrete declaration remains separately inventoried", exclusion["reason"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/007-client-execution/spec.md",
                        "status": "Approved",
                    },
                    {
                        key: exclusion["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                if type_name in {"RequestDeliveryState", "TransportCauseCategory", "TransportError"}:
                    self.assertIn("KMIPKIT-0007-FR-005", exclusion["evidence"])
                elif type_name in {"ClientError", "ClientErrorResponseTtlv"}:
                    self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])
                else:
                    self.assertIn("KMIPKIT-0007-FR-001", exclusion["evidence"])

    def test_t091_error_ttlv_wrapper_is_excluded_under_the_narrow_0021_capability(self) -> None:
        expected = {
            "kmipkit_client::ClientErrorResponseTtlv",
            "kmipkit_client::ClientErrorResponseTtlv::with_ttlv",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        exclusions = {
            row["exportName"]: row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] in expected
        }
        self.assertEqual(expected, set(exclusions))

        for export_name in expected:
            with self.subTest(export=export_name):
                self.assertNotIn(export_name, declarations)
                exclusion = exclusions[export_name]
                self.assertIn("Rust response-TTLV wrapper", exclusion["reason"])
                self.assertIn("not a separate four-language capability", exclusion["reason"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/021-hash-mac-signature-operations/spec.md",
                        "status": "Approved",
                    },
                    {
                        key: exclusion["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertIn("KMIPKIT-0021-FR-015", exclusion["evidence"])
                self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "crypto_response_ttlv"
        )
        self.assertEqual("KMIPKIT-0021", capability["ownerSpecId"])
        self.assertIn("KMIPKIT-0021-FR-015", capability["requirementIds"])
        self.assertIn(
            "kmipkit_client::ClientError::response_ttlv",
            capability["mappings"]["rust"],
        )
        self.assertIn(
            "kmipkit_client_execute_with_error_ttlv_v1",
            capability["mappings"]["c"],
        )
        self.assertIn(
            "org.kmipkit.ProtocolException.responseTtlv",
            capability["mappings"]["java"],
        )
        self.assertIn(
            "kmipkit.errors.ClientExecutionError.response_ttlv",
            capability["mappings"]["python"],
        )

    def test_t091_discover_versions_concrete_api_maps_to_four_language_operation(self) -> None:
        expected_mappings = {
            "kmipkit_protocol::DiscoverVersionsRequest": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::DiscoverVersionsRequest"),
                "c": ("c-kmipkit", "kmipkit_client_batch_item_discover_versions"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItem.discoverVersions"),
                "python": ("python-kmipkit", "kmipkit.extensions.client_batch_item_discover_versions"),
            },
            "kmipkit_protocol::DiscoverVersionsRequest::new": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::DiscoverVersionsRequest::new"),
                "c": ("c-kmipkit", "kmipkit_client_batch_item_discover_versions"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItem.discoverVersions"),
                "python": ("python-kmipkit", "kmipkit.extensions.client_batch_item_discover_versions"),
            },
            "kmipkit_protocol::DiscoverVersionsResponse": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::DiscoverVersionsResponse"),
                "c": ("c-kmipkit", "kmipkit_client_batch_item_response_t"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult"),
                "python": ("python-kmipkit", "kmipkit.client.BatchItemResult"),
            },
            "kmipkit_protocol::DiscoverVersionsResponse::result": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::DiscoverVersionsResponse::result"),
                "c": ("c-kmipkit", "kmipkit_client_batch_item_metadata_v2_t"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemMetadataV2"),
                "python": ("python-kmipkit", "kmipkit.client.BatchItemMetadataV2"),
            },
            "kmipkit_protocol::DiscoverVersionsResponse::supported_versions": {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::DiscoverVersionsResponse::supported_versions"),
                "c": ("c-kmipkit", "kmipkit_client_batch_item_response_discover_versions_supported_versions"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.supportedVersions"),
                "python": ("python-kmipkit", "kmipkit.client.BatchItemResult.supported_versions"),
            },
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS"
        )
        requirement_ids = {
            "KMIPKIT-0007-FR-002",
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-021",
        }

        self.assertEqual("verified", operation["apiState"])
        for export_name, mappings in expected_mappings.items():
            declaration = declarations[export_name]
            with self.subTest(export=export_name):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual(
                    "KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS",
                    declaration["capabilityId"],
                )
                self.assertNotIn("candidateSpecId", declaration)
                self.assertTrue(requirement_ids.issubset(declaration["requirementIds"]))
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/007-client-execution/spec.md",
                        "status": "Approved",
                    },
                    {
                        key: declaration["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertEqual(
                    {
                        language: {"packageId": package_id, "name": name}
                        for language, (package_id, name) in mappings.items()
                    },
                    declaration["languageMappings"],
                )
                self.assertEqual(
                    declaration["packageId"],
                    declaration["languageMappings"]["rust"]["packageId"],
                )
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])
                self.assertTrue(
                    set(operation["parityEvidenceIds"]).issubset(
                        set(declaration["parityEvidenceIds"])
                    )
                )

    def test_t091_discover_versions_rust_helpers_and_error_details_have_exact_exclusions(self) -> None:
        expected_exclusions = {
            "kmipkit_protocol::DiscoverVersionsRequest::protocol_versions",
            "kmipkit_protocol::DiscoverVersionsRequest::to_ttlv_payload",
            "kmipkit_protocol::DiscoverVersionsResponse::try_from_response_item",
            "kmipkit_protocol::DiscoverVersionsError",
            "kmipkit_protocol::DiscoverVersionsError::UnexpectedOperation",
            "kmipkit_protocol::DiscoverVersionsError::MissingResultStatus",
            "kmipkit_protocol::DiscoverVersionsError::InvalidOperationResult",
            "kmipkit_protocol::DiscoverVersionsError::MissingSuccessPayload",
            "kmipkit_protocol::DiscoverVersionsError::MalformedProtocolVersion",
            "kmipkit_protocol::DiscoverVersionsError::UnofferedProtocolVersion",
            "kmipkit_protocol::DiscoverVersionsError::InvalidOperationResult::0",
        }
        exclusions = {
            row["exportName"]: row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"].startswith("kmipkit_protocol::DiscoverVersions")
        }

        self.assertEqual(expected_exclusions, set(exclusions))
        for export_name, exclusion in exclusions.items():
            with self.subTest(export=export_name):
                self.assertTrue(exclusion["reason"])
                self.assertTrue(exclusion["evidence"])
                self.assertIn("KMIPKIT-0007-FR-001", exclusion["evidence"])
                self.assertIn("KMIPKIT-0007-FR-002", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/007-client-execution/spec.md",
                        "status": "Approved",
                    },
                    {
                        key: exclusion["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                if "DiscoverVersionsError" in export_name:
                    self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])

    def test_t091_facade_batch_response_reexports_map_to_existing_parity_surface(self) -> None:
        expected_sources = {
            "kmipkit::ClientBatchResponse": "kmipkit_client::ClientBatchResponse",
            "kmipkit::ClientBatchResponse::len": "kmipkit_client::ClientBatchResponse::len",
            "kmipkit::ClientBatchResponse::is_empty": "kmipkit_client::ClientBatchResponse::len",
            "kmipkit::ClientBatchResponse::get": "kmipkit_client::ClientBatchResponse::get",
            "kmipkit::ClientBatchResponse::iter": "kmipkit_client::ClientBatchResponse::get",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for facade_name, client_name in expected_sources.items():
            facade = declarations[facade_name]
            client = declarations[client_name]
            with self.subTest(export=facade_name):
                self.assertEqual("declared", facade["declarationState"])
                self.assertEqual("client_execution", facade["capabilityId"])
                self.assertNotIn("candidateSpecId", facade)
                self.assertEqual(client["requirementIds"], facade["requirementIds"])
                self.assertEqual(client["approvalSource"], facade["approvalSource"])
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in client["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit",
                    "name": facade_name,
                }
                self.assertEqual(expected_mappings, facade["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_client_batch_response_iteration_maps_to_existing_parity_surface(self) -> None:
        expected_facades = {
            "kmipkit_client::ClientBatchResponse::is_empty": "kmipkit::ClientBatchResponse::is_empty",
            "kmipkit_client::ClientBatchResponse::iter": "kmipkit::ClientBatchResponse::iter",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for client_name, facade_name in expected_facades.items():
            client = declarations[client_name]
            facade = declarations[facade_name]
            with self.subTest(export=client_name):
                self.assertEqual("declared", client["declarationState"])
                self.assertEqual("client_execution", client["capabilityId"])
                self.assertNotIn("candidateSpecId", client)
                self.assertEqual(facade["requirementIds"], client["requirementIds"])
                self.assertEqual(facade["approvalSource"], client["approvalSource"])
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in facade["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-client",
                    "name": client_name,
                }
                self.assertEqual(expected_mappings, client["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_client_execution_candidate_exports_are_reconciled(self) -> None:
        unresolved = [
            row["exportName"]
            for row in self.checked_in_manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
        ]
        self.assertEqual(
            [],
            unresolved,
            "KMIPKIT-0007 candidate exports must be mapped or specifically excluded; "
            f"{len(unresolved)} remain, including {unresolved[:10]}",
        )

    def test_t091_discover_versions_request_constructors_map_to_four_language_operation(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        canonical = declarations["kmipkit_client::ClientBatchItem::discover_versions"]
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == canonical["capabilityId"]
        )
        request_constructors = {
            "kmipkit_client::ClientRequest::discover_versions": "rust-kmipkit-client",
            "kmipkit::ClientRequest::discover_versions": "rust-kmipkit",
        }

        for export_name, rust_package in request_constructors.items():
            with self.subTest(export=export_name):
                declaration = declarations[export_name]
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": rust_package,
                    "name": export_name,
                }

                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_discover_versions_request_enum_variants_map_to_four_language_operation(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        canonical = declarations["kmipkit_client::ClientBatchItem::discover_versions"]
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == canonical["capabilityId"]
        )
        request_variants = {
            "kmipkit_client::ClientRequest::DiscoverVersions": (
                "rust-kmipkit-client",
            ),
            "kmipkit_client::ClientRequest::DiscoverVersions::0": (
                "rust-kmipkit-client",
            ),
            "kmipkit::ClientRequest::DiscoverVersions": ("rust-kmipkit",),
            "kmipkit::ClientRequest::DiscoverVersions::0": ("rust-kmipkit",),
        }

        for export_name, (rust_package,) in request_variants.items():
            with self.subTest(export=export_name):
                declaration = declarations[export_name]
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": rust_package,
                    "name": export_name,
                }

                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_verification_request_variants_map_to_approved_operation_builders(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "MacVerify": (
                "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY",
                "mac_verify",
            ),
            "SignatureVerify": (
                "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY",
                "signature_verify",
            ),
        }

        for variant, (operation_id, method) in cases.items():
            operation = operations[operation_id]
            self.assertEqual("declared", operation["apiState"])
            self.assertEqual("KMIPKIT-0021", operation["featureSpecId"])

            constructor_declarations = {}
            for language in ("c", "java", "python"):
                constructor_id = next(
                    declaration_id
                    for declaration_id in operation["declarations"][language]
                    if declaration_id.endswith(f":constructor:{language}")
                )
                constructor_declarations[language] = declarations_by_id[constructor_id]

            for rust_prefix, package_id in (
                ("kmipkit_client", "rust-kmipkit-client"),
                ("kmipkit", "rust-kmipkit"),
            ):
                export_names = (
                    f"{rust_prefix}::ClientRequest::{method}",
                    f"{rust_prefix}::ClientRequest::{variant}",
                    f"{rust_prefix}::ClientRequest::{variant}::0",
                )
                for export_name in export_names:
                    with self.subTest(export=export_name):
                        declaration = declarations[export_name]
                        expected_mappings = {
                            language: constructor_declarations[language][
                                "languageMappings"
                            ][language]
                            for language in ("c", "java", "python")
                        }
                        expected_mappings["rust"] = {
                            "packageId": package_id,
                            "name": export_name,
                        }

                        self.assertEqual("declared", declaration["declarationState"])
                        self.assertNotIn("candidateSpecId", declaration)
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual(
                            constructor_declarations["c"]["requirementIds"],
                            declaration["requirementIds"],
                        )
                        self.assertEqual(
                            operation["approvalEvidence"], declaration["approvalSource"]
                        )
                        self.assertEqual(
                            expected_mappings, declaration["languageMappings"]
                        )
                        self.assertIn(
                            declaration["declarationId"], operation["declarations"]["rust"]
                        )

    def test_t091_direct_client_helpers_map_to_single_item_batch_execution(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "get_usage_allocation": (
                "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
                "kmipkit_client_batch_item_get_usage_allocation",
                "org.kmipkit.extensions.ClientBatchItem.getUsageAllocation",
                "kmipkit.extensions.client_batch_item_get_usage_allocation",
            ),
            "import": (
                "KMIPKIT-ELEM-OP-C2S-IMPORT",
                "kmipkit_client_batch_item_import",
                "org.kmipkit.extensions.ClientBatchItem.importObject",
                "kmipkit.extensions.client_batch_item_import",
            ),
            "interop": (
                "KMIPKIT-ELEM-OP-C2S-INTEROP",
                "kmipkit_client_batch_item_interop",
                "org.kmipkit.extensions.ClientBatchItem.interop",
                "kmipkit.extensions.client_batch_item_interop",
            ),
            "log": (
                "KMIPKIT-ELEM-OP-C2S-LOG",
                "kmipkit_client_batch_item_log",
                "org.kmipkit.extensions.ClientBatchItem.log",
                "kmipkit.extensions.client_batch_item_log",
            ),
            "validate": (
                "KMIPKIT-ELEM-OP-C2S-VALIDATE",
                "kmipkit_client_batch_item_validate",
                "org.kmipkit.extensions.ClientBatchItem.validate",
                "kmipkit.extensions.client_batch_item_validate",
            ),
        }
        execute_mappings = {
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_client_execute",
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.Client.execute",
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.client.Client.execute",
            },
        }
        required_facade_requirements = {
            "KMIPKIT-0043-FR-005",
            "KMIPKIT-0043-FR-007",
            "KMIPKIT-0043-FR-008",
            "KMIPKIT-0043-FR-009",
        }

        for method, (operation_id, *constructor_names) in cases.items():
            operation = operations[operation_id]
            self.assertEqual("verified", operation["apiState"])
            self.assertTrue(operation["ownerDependency"]["state"] == "approved")

            constructor_declarations = {}
            for language, constructor_name in zip(
                ("c", "java", "python"), constructor_names, strict=True
            ):
                constructor = declarations[constructor_name]
                self.assertEqual(operation_id, constructor["capabilityId"])
                self.assertEqual("verified", constructor["declarationState"])
                self.assertIn(
                    constructor["declarationId"], operation["declarations"][language]
                )
                constructor_declarations[language] = constructor

            for rust_prefix, package_id in (
                ("kmipkit_client", "rust-kmipkit-client"),
                ("kmipkit", "rust-kmipkit"),
            ):
                export_name = f"{rust_prefix}::Client::{method}"
                with self.subTest(export=export_name):
                    declaration = declarations[export_name]
                    expected_mappings = {
                        **execute_mappings,
                        "rust": {
                            "packageId": package_id,
                            "name": export_name,
                        },
                    }

                    self.assertEqual("verified", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertEqual(operation_id, declaration["capabilityId"])
                    expected_requirements = set(
                        constructor_declarations["c"]["requirementIds"]
                    ) | required_facade_requirements
                    self.assertEqual(
                        sorted(expected_requirements), declaration["requirementIds"]
                    )
                    self.assertEqual(
                        operation["approvalEvidence"], declaration["approvalSource"]
                    )
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    self.assertLessEqual(
                        required_facade_requirements,
                        set(declaration["requirementIds"]),
                    )
                    required_parity_evidence = {
                        evidence_id
                        for constructor in constructor_declarations.values()
                        for evidence_id in constructor.get("parityEvidenceIds", [])
                    }
                    required_parity_evidence.update(
                        {
                            f"crates/kmipkit-client/tests/unit/{method}_execution_tests.rs",
                            "bindings/python/tests/test_direct_client_helper_execution.py",
                            "tools/api_manifest/tests/test_manifest_v2.py",
                            "tools/api_manifest/tests/test_t091_root_reexport_aliases.py",
                        }
                    )
                    self.assertLessEqual(
                        required_parity_evidence,
                        set(declaration.get("parityEvidenceIds", [])),
                    )
                    self.assertIn(
                        declaration["declarationId"], operation["declarations"]["rust"]
                    )

    def test_t091_add_attribute_request_variants_map_to_verified_operation(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        canonical = declarations["kmipkit_client::ClientRequest::add_attribute"]
        operation_id = "KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE"
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == operation_id
        )
        request_variants = {
            "kmipkit_client::ClientRequest::AddAttribute": "rust-kmipkit-client",
            "kmipkit_client::ClientRequest::AddAttribute::0": "rust-kmipkit-client",
            "kmipkit::ClientRequest::AddAttribute": "rust-kmipkit",
            "kmipkit::ClientRequest::AddAttribute::0": "rust-kmipkit",
        }

        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0016", operation["featureSpecId"])
        for export_name, rust_package in request_variants.items():
            with self.subTest(export=export_name):
                declaration = declarations[export_name]
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": rust_package,
                    "name": export_name,
                }

                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_attribute_request_variants_map_to_verified_operations(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "AdjustAttribute": (
                "KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE",
                "adjust_attribute",
                ("", "::0"),
            ),
            "DeleteAttribute": (
                "KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE",
                "delete_attribute",
                ("", "::0"),
            ),
            "GetAttributeList": (
                "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST",
                "get_attribute_list",
                ("::0",),
            ),
            "GetAttributes": (
                "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                "get_attributes",
                ("::0",),
            ),
            "ModifyAttribute": (
                "KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE",
                "modify_attribute",
                ("", "::0"),
            ),
            "SetAttribute": (
                "KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE",
                "set_attribute",
                ("", "::0"),
            ),
        }

        for variant, (operation_id, method, suffixes) in cases.items():
            operation = operations[operation_id]
            canonical = declarations[f"kmipkit_client::ClientRequest::{method}"]
            self.assertEqual("verified", operation["apiState"])
            self.assertEqual("KMIPKIT-0016", operation["featureSpecId"])

            for prefix, package_id in (
                ("kmipkit_client", "rust-kmipkit-client"),
                ("kmipkit", "rust-kmipkit"),
            ):
                for suffix in suffixes:
                    export_name = f"{prefix}::ClientRequest::{variant}{suffix}"
                    with self.subTest(export=export_name):
                        declaration = declarations[export_name]
                        expected_mappings = {
                            language: mapping.copy()
                            for language, mapping in canonical["languageMappings"].items()
                        }
                        expected_mappings["rust"] = {
                            "packageId": package_id,
                            "name": export_name,
                        }

                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertNotIn("candidateSpecId", declaration)
                        self.assertEqual(operation_id, declaration["capabilityId"])
                        self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                        self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                        self.assertEqual(expected_mappings, declaration["languageMappings"])
                        self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_object_lifecycle_request_fields_and_helpers_map_to_verified_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        variants = {
            "Certify": (
                "KMIPKIT-ELEM-OP-C2S-CERTIFY",
                "kmipkit::ClientRequest::Certify",
            ),
            "Check": (
                "KMIPKIT-ELEM-OP-C2S-CHECK",
                "kmipkit_client::ClientRequest::check",
            ),
            "ObtainLease": (
                "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
                "kmipkit_client::ClientRequest::obtain_lease",
            ),
            "ReProvision": (
                "KMIPKIT-ELEM-OP-C2S-RE-PROVISION",
                "kmipkit_client::ClientRequest::reprovision",
            ),
            "Recertify": (
                "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
                "kmipkit_client::ClientRequest::Recertify",
            ),
            "Register": (
                "KMIPKIT-ELEM-OP-C2S-REGISTER",
                "kmipkit_client::ClientRequest::Register",
            ),
            "Rekey": (
                "KMIPKIT-ELEM-OP-C2S-RE-KEY",
                "kmipkit_client::ClientRequest::Rekey",
            ),
            "RekeyKeyPair": (
                "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
                "kmipkit_client::ClientRequest::RekeyKeyPair",
            ),
            "Revoke": (
                "KMIPKIT-ELEM-OP-C2S-REVOKE",
                "kmipkit_client::ClientRequest::Revoke",
            ),
        }
        helpers = {
            "certify": "KMIPKIT-ELEM-OP-C2S-CERTIFY",
            "recertify": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
            "register": "KMIPKIT-ELEM-OP-C2S-REGISTER",
            "rekey": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
            "rekey_key_pair": "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
            "revoke": "KMIPKIT-ELEM-OP-C2S-REVOKE",
        }
        expected_exports = []
        for variant, (operation_id, canonical_name) in variants.items():
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.append(
                    (f"{prefix}::ClientRequest::{variant}::0", operation_id, canonical_name)
                )
        for method, operation_id in helpers.items():
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.append(
                    (
                        f"{prefix}::ClientRequest::{method}",
                        operation_id,
                        variants[next(name for name, value in variants.items() if value[0] == operation_id)][1],
                    )
                )

        self.assertEqual(30, len(expected_exports))
        for export_name, operation_id, canonical_name in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[canonical_name]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_constraint_request_fields_and_helpers_map_to_verified_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        variants = {
            "GetConstraints": (
                "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS",
                "kmipkit_client::ClientRequest::GetConstraints",
            ),
            "GetUsageAllocation": (
                "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
                "kmipkit_client::ClientRequest::get_usage_allocation",
            ),
            "SetConstraints": (
                "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS",
                "kmipkit_client::ClientRequest::SetConstraints",
            ),
            "SetDefaults": (
                "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS",
                "kmipkit_client::ClientRequest::SetDefaults",
            ),
        }
        helpers = {
            "get_constraints": "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS",
            "set_constraints": "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS",
            "set_defaults": "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS",
        }
        expected_exports = []
        for variant, (operation_id, canonical_name) in variants.items():
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.append(
                    (f"{prefix}::ClientRequest::{variant}::0", operation_id, canonical_name)
                )
        for method, operation_id in helpers.items():
            canonical_name = next(
                canonical
                for candidate_id, canonical in variants.values()
                if candidate_id == operation_id
            )
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.append(
                    (f"{prefix}::ClientRequest::{method}", operation_id, canonical_name)
                )

        self.assertEqual(14, len(expected_exports))
        for export_name, operation_id, canonical_name in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[canonical_name]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0039", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_session_request_variants_and_payloads_map_to_verified_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "DelegatedLogin": (
                "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN",
                "delegated_login",
            ),
            "Login": ("KMIPKIT-ELEM-OP-C2S-LOGIN", "login"),
            "Logout": ("KMIPKIT-ELEM-OP-C2S-LOGOUT", "logout"),
        }
        expected_exports = [
            (f"{prefix}::ClientRequest::{variant}{suffix}", operation_id, method)
            for variant, (operation_id, method) in cases.items()
            for prefix in ("kmipkit_client", "kmipkit")
            for suffix in ("", "::0")
        ]

        self.assertEqual(12, len(expected_exports))
        for export_name, operation_id, method in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[f"kmipkit_client::ClientRequest::{method}"]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0038", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_key_transfer_request_fields_and_helpers_map_to_verified_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "Export": ("KMIPKIT-ELEM-OP-C2S-EXPORT", "export"),
            "Import": ("KMIPKIT-ELEM-OP-C2S-IMPORT", "import"),
            "JoinSplitKey": ("KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY", "join_split_key"),
        }
        expected_exports = []
        for variant, (operation_id, method) in cases.items():
            canonical_name = f"kmipkit_client::ClientRequest::{variant}"
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.extend(
                    (
                        (f"{prefix}::ClientRequest::{variant}::0", operation_id, canonical_name),
                        (f"{prefix}::ClientRequest::{method}", operation_id, canonical_name),
                    )
                )

        self.assertEqual(12, len(expected_exports))
        for export_name, operation_id, canonical_name in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[canonical_name]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0037", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_object_query_request_fields_and_helpers_map_to_verified_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "Get": ("KMIPKIT-ELEM-OP-C2S-GET", "get"),
            "Locate": ("KMIPKIT-ELEM-OP-C2S-LOCATE", "locate"),
        }
        expected_exports = []
        for variant, (operation_id, method) in cases.items():
            canonical_name = f"kmipkit_client::ClientRequest::{variant}"
            for prefix in ("kmipkit_client", "kmipkit"):
                expected_exports.extend(
                    (
                        (f"{prefix}::ClientRequest::{variant}::0", operation_id, canonical_name),
                        (f"{prefix}::ClientRequest::{method}", operation_id, canonical_name),
                    )
                )

        self.assertEqual(8, len(expected_exports))
        for export_name, operation_id, canonical_name in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[canonical_name]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0017", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_remaining_verified_request_payloads_map_to_their_operations(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = {
            "Pkcs11": (
                "KMIPKIT-ELEM-OP-C2S-PKCS-11",
                "KMIPKIT-0040",
                "kmipkit_client::ClientRequest::pkcs11",
            ),
            "RngRetrieve": (
                "KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE",
                "KMIPKIT-0041",
                "kmipkit_client::ClientRequest::rng_retrieve",
            ),
            "RngSeed": (
                "KMIPKIT-ELEM-OP-C2S-RNG-SEED",
                "KMIPKIT-0041",
                "kmipkit_client::ClientRequest::rng_seed",
            ),
            "Interop": (
                "KMIPKIT-ELEM-OP-C2S-INTEROP",
                "KMIPKIT-0042",
                "kmipkit_client::ClientRequest::Interop",
            ),
            "Log": (
                "KMIPKIT-ELEM-OP-C2S-LOG",
                "KMIPKIT-0042",
                "kmipkit_client::ClientRequest::Log",
            ),
        }
        expected_exports = [
            (f"{prefix}::ClientRequest::{variant}::0", operation_id, owner_id, canonical)
            for variant, (operation_id, owner_id, canonical) in cases.items()
            for prefix in ("kmipkit_client", "kmipkit")
        ]

        self.assertEqual(10, len(expected_exports))
        for export_name, operation_id, owner_id, canonical_name in expected_exports:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            canonical = declarations[canonical_name]
            with self.subTest(export=export_name):
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": declaration["packageId"],
                    "name": export_name,
                }
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual(owner_id, operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_discover_versions_response_view_maps_to_existing_version_accessor(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        canonical = declarations[
            "kmipkit_client::ClientBatchItemResponse::discover_versions_supported_versions"
        ]
        export_name = "kmipkit_client::ClientResponseView::supported_versions"
        declaration = declarations[export_name]
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == canonical["capabilityId"]
        )
        expected_mappings = {
            language: mapping.copy()
            for language, mapping in canonical["languageMappings"].items()
        }
        expected_mappings["rust"] = {
            "packageId": "rust-kmipkit-client",
            "name": export_name,
        }

        self.assertEqual("declared", declaration["declarationState"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
        self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
        self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
        self.assertEqual(expected_mappings, declaration["languageMappings"])
        for language, mapping in expected_mappings.items():
            self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_response_view_and_result_accessor_map_to_existing_result_surfaces(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        mappings = {
            "kmipkit_client::ClientResponseView": (
                "kmipkit_client::ClientBatchItemResponse",
                "rust-kmipkit-client",
                "kmipkit_client::ClientResponseView",
            ),
            "kmipkit_client::ClientResponseView::result": (
                "kmipkit_protocol::KmipOperationResult",
                "rust-kmipkit-client",
                "kmipkit_client::ClientResponseView::result",
            ),
        }

        for export_name, (canonical_name, rust_package, rust_mapping_name) in mappings.items():
            canonical = declarations[canonical_name]
            declaration = declarations[export_name]
            capability = next(
                row
                for row in self.checked_in_manifest["capabilities"]
                if row["capabilityId"] == canonical["capabilityId"]
            )
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": rust_package,
                "name": rust_mapping_name,
            }

            with self.subTest(export=export_name):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_batch_outcome_response_accessors_map_to_four_language_result_handles(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        response_accessors = {
            "kmipkit_client::ClientBatchOutcome::response": (
                "kmipkit_client::ClientBatchItemResponse",
                "rust-kmipkit-client",
            ),
            "kmipkit::ClientBatchOutcome::response": (
                "kmipkit::ClientBatchItemResponse",
                "rust-kmipkit",
            ),
        }

        for export_name, (canonical_name, rust_package) in response_accessors.items():
            canonical = declarations[canonical_name]
            declaration = declarations[export_name]
            capability = next(
                row
                for row in self.checked_in_manifest["capabilities"]
                if row["capabilityId"] == canonical["capabilityId"]
            )
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": rust_package,
                "name": export_name,
            }

            with self.subTest(export=export_name):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_discover_versions_response_accessors_map_to_generic_result_handles(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        canonical_name = "kmipkit_client::ClientBatchItemResponse"
        canonical = declarations[canonical_name]
        expected_requirements = sorted(
            set(canonical["requirementIds"])
            | {"KMIPKIT-0007-FR-002", "KMIPKIT-0043-FR-010"}
        )
        response_accessors = {
            "kmipkit_client::ClientResponseView::discover_versions": (
                "rust-kmipkit-client",
            ),
            "kmipkit_client::ClientBatchOutcome::discover_versions_response": (
                "rust-kmipkit-client",
            ),
            "kmipkit::ClientBatchOutcome::discover_versions_response": (
                "rust-kmipkit",
            ),
        }

        for export_name, (rust_package,) in response_accessors.items():
            declaration = declarations[export_name]
            capability = next(
                row
                for row in self.checked_in_manifest["capabilities"]
                if row["capabilityId"] == canonical["capabilityId"]
            )
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": rust_package,
                "name": export_name,
            }

            with self.subTest(export=export_name):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(expected_requirements, declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_transport_exchange_error_contract_maps_rust_only_under_adr_0014(self) -> None:
        expected_exports = {
            "kmipkit_transport::RequestDeliveryState",
            "kmipkit_transport::RequestDeliveryState::NotSent",
            "kmipkit_transport::RequestDeliveryState::PossiblySent",
            "kmipkit_transport::RequestDeliveryState::ResponseStarted",
            "kmipkit_transport::RequestDeliveryState::not_sent",
            "kmipkit_transport::RequestDeliveryState::write_started",
            "kmipkit_transport::RequestDeliveryState::response_bytes_received",
            "kmipkit_transport::TransportCauseCategory",
            "kmipkit_transport::TransportCauseCategory::Io",
            "kmipkit_transport::TransportCauseCategory::Tls",
            "kmipkit_transport::TransportCauseCategory::Http",
            "kmipkit_transport::TransportCauseCategory::Timeout",
            "kmipkit_transport::TransportCauseCategory::Other",
            "kmipkit_transport::TransportError",
            "kmipkit_transport::TransportError::new",
            "kmipkit_transport::TransportError::delivery_state",
            "kmipkit_transport::TransportError::cause_category",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        evidence = {
            "crates/kmipkit-transport/tests/public_contract.rs",
            "crates/kmipkit-transport/tests/delivery_state.rs",
            "crates/kmipkit-transport/tests/unit/raw_tls_error_mapping_tests.rs",
            "crates/kmipkit-transport/tests/secret_redaction_current.rs",
        }

        for export_name in sorted(expected_exports):
            with self.subTest(export=export_name):
                declaration = declarations[export_name]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("rust-only-adr", declaration["parityScope"])
                self.assertEqual("rust_transport_exchange", declaration["capabilityId"])
                self.assertTrue(
                    {"KMIPKIT-0007-FR-005", "KMIPKIT-0043-FR-020", "KMIPKIT-0043-FR-021"}
                    <= set(declaration["requirementIds"])
                )
                self.assertEqual(
                    {
                        "kind": "accepted-adr",
                        "path": "docs/adr/0014-public-transport-exchange-contract.md",
                        "status": "Accepted",
                    },
                    {
                        key: declaration["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertEqual(
                    {
                        "rust": {
                            "packageId": "rust-kmipkit-transport",
                            "name": export_name,
                        }
                    },
                    declaration["languageMappings"],
                )
                self.assertTrue(evidence <= set(declaration["parityEvidenceIds"]))

    def test_t091_transport_standard_trait_exports_have_exact_adr_exclusions(self) -> None:
        exclusions = [
            row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "rust-kmipkit-transport"
            and row["exportName"].startswith("<kmipkit_transport::")
        ]
        self.assertEqual(71, len(exclusions))
        for exclusion in exclusions:
            with self.subTest(export=exclusion["exportName"]):
                self.assertTrue(exclusion["reason"])
                self.assertEqual(
                    {
                        "kind": "accepted-adr",
                        "path": "docs/adr/0014-public-transport-exchange-contract.md",
                        "status": "Accepted",
                    },
                    {
                        key: exclusion["approvalSource"][key]
                        for key in ("kind", "path", "status")
                    },
                )
                self.assertIn("KMIPKIT-0007-FR-005", exclusion["evidence"])
                self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])

    def test_t091_client_certify_convenience_maps_to_verified_operation_and_shared_execute(self) -> None:
        declaration_id = "rust-observed-05c7d86aafdeccc15e11"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-CERTIFY"
        )
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        self.assertEqual("kmipkit_client::Client::certify", declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-CERTIFY", declaration["capabilityId"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::Client::certify",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_execute",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.Client.execute",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.Client.execute",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])
        for language in ("c", "java", "python"):
            self.assertIn(
                declaration["languageMappings"][language]["name"],
                client_execution["mappings"][language],
            )

    def test_t091_client_check_convenience_maps_to_verified_operation_and_shared_execute(self) -> None:
        declaration_id = "rust-observed-7da0265a86b0aaaa0e43"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-CHECK"
        )
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        self.assertEqual("kmipkit_client::Client::check", declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-CHECK", declaration["capabilityId"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::Client::check",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_execute",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.Client.execute",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.Client.execute",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])
        for language in ("c", "java", "python"):
            self.assertIn(
                declaration["languageMappings"][language]["name"],
                client_execution["mappings"][language],
            )

    def test_t091_async_operation_options_map_to_direct_specialized_methods(self) -> None:
        cases = {
            "poll": (
                "KMIPKIT-ELEM-OP-C2S-POLL",
                "KMIPKIT-0009-FR-003",
                "kmipkit_client_execute_poll_with_options_v1",
                "executePollWithOptions",
                "execute_poll_with_options",
            ),
            "cancel": (
                "KMIPKIT-ELEM-OP-C2S-CANCEL",
                "KMIPKIT-0009-FR-004",
                "kmipkit_client_execute_cancel_with_options_v1",
                "executeCancelWithOptions",
                "execute_cancel_with_options",
            ),
            "process": (
                "KMIPKIT-ELEM-OP-C2S-PROCESS",
                "KMIPKIT-0009-FR-005",
                "kmipkit_client_execute_process_with_options_v1",
                "executeProcessWithOptions",
                "execute_process_with_options",
            ),
            "query_async_requests": (
                "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS",
                "KMIPKIT-0009-FR-006",
                "kmipkit_client_execute_query_async_requests_with_options_v1",
                "executeQueryAsyncRequestsWithOptions",
                "execute_query_async_requests_with_options",
            ),
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        functions = {
            row["id"]: row for row in self.checked_in_manifest["functions"]
        }
        operations_by_id = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        async_capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_asynchronous_operations"
        )

        for method, (operation_id, owner_requirement, c_symbol, java_method, python_method) in cases.items():
            operation = operations_by_id[operation_id]
            function_id = f"client_execute_{method}_with_options"
            with self.subTest(function=function_id):
                function = functions[function_id]
                self.assertEqual(
                    f"execute_{method}_with_options", function["rust"]["name"]
                )
                self.assertEqual(c_symbol, function["c"]["symbol"])
                self.assertEqual(java_method, function["java"]["method"])
                self.assertEqual(
                    f"Client.{python_method}", function["python"]["function"]
                )
                self.assertIn(owner_requirement, function["requirementIds"])
                self.assertIn("KMIPKIT-0013-FR-011", function["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-010", function["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", function["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-022", function["requirementIds"])

            for package_id, rust_prefix in (
                ("rust-kmipkit-client", "kmipkit_client::"),
                ("rust-kmipkit", "kmipkit::"),
            ):
                export_name = f"{rust_prefix}Client::execute_{method}_with_options"
                with self.subTest(export=export_name):
                    declaration = declarations[export_name]
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertEqual("client_asynchronous_operations", declaration["capabilityId"])
                    self.assertEqual(package_id, declaration["packageId"])
                    self.assertEqual(
                        operation["approvalEvidence"]["path"],
                        declaration["approvalSource"]["path"],
                    )
                    self.assertIn(owner_requirement, declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0013-FR-011", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0043-FR-010", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0043-FR-022", declaration["requirementIds"])
                    self.assertEqual(
                        {
                            "rust": {"packageId": package_id, "name": export_name},
                            "c": {"packageId": "c-kmipkit", "name": c_symbol},
                            "java": {
                                "packageId": "java-kmipkit",
                                "name": f"org.kmipkit.extensions.Client.{java_method}",
                            },
                            "python": {
                                "packageId": "python-kmipkit",
                                "name": f"kmipkit.client.Client.{python_method}",
                            },
                        },
                        declaration["languageMappings"],
                    )
                    self.assertNotIn("execute_with_options", str(declaration["languageMappings"]))
                    self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

            for language, name in (
                ("c", c_symbol),
                ("java", f"org.kmipkit.extensions.Client.{java_method}"),
                ("python", f"kmipkit.client.Client.{python_method}"),
            ):
                self.assertIn(name, async_capability["mappings"][language])

    def test_t091_client_operation_options_methods_map_to_shared_execute_options(self) -> None:
        cases = (
            (
                "rust-observed-88587ed0c8f26b4fe362",
                "kmipkit_client::Client::certify_with_options",
                "KMIPKIT-ELEM-OP-C2S-CERTIFY",
            ),
            (
                "rust-observed-08b1b13b97c0d8bcb4b7",
                "kmipkit_client::Client::check_with_options",
                "KMIPKIT-ELEM-OP-C2S-CHECK",
            ),
        )
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for declaration_id, export_name, operation_id in cases:
            with self.subTest(export=export_name):
                declaration = next(
                    row
                    for row in self.checked_in_manifest["publicDeclarations"]
                    if row["declarationId"] == declaration_id
                )
                operation = next(
                    row
                    for row in self.checked_in_manifest["operations"]
                    if row["operationId"] == operation_id
                )

                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(
                    {
                        "rust": {
                            "packageId": "rust-kmipkit-client",
                            "name": export_name,
                        },
                        "c": {
                            "packageId": "c-kmipkit",
                            "name": "kmipkit_client_execute_with_options_v1",
                        },
                        "java": {
                            "packageId": "java-kmipkit",
                            "name": "org.kmipkit.extensions.Client.executeWithOptions",
                        },
                        "python": {
                            "packageId": "python-kmipkit",
                            "name": "kmipkit.client.Client.execute_with_options",
                        },
                    },
                    declaration["languageMappings"],
                )
                self.assertEqual(
                    operation["approvalEvidence"]["path"],
                    declaration["approvalSource"]["path"],
                )
                self.assertIn(declaration_id, operation["declarations"]["rust"])
                for language in ("c", "java", "python"):
                    self.assertIn(
                        declaration["languageMappings"][language]["name"],
                        client_execution["mappings"][language],
                    )

    def test_t091_high_level_operation_options_map_to_four_language_execution(self) -> None:
        operations = {
            "activate": "KMIPKIT-ELEM-OP-C2S-ACTIVATE",
            "archive": "KMIPKIT-ELEM-OP-C2S-ARCHIVE",
            "destroy": "KMIPKIT-ELEM-OP-C2S-DESTROY",
            "recover": "KMIPKIT-ELEM-OP-C2S-RECOVER",
            "ping": "KMIPKIT-ELEM-OP-C2S-PING",
            "query": "KMIPKIT-ELEM-OP-C2S-QUERY",
            "hash": "KMIPKIT-ELEM-OP-C2S-HASH",
            "mac": "KMIPKIT-ELEM-OP-C2S-MAC",
            "mac_verify": "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY",
            "sign": "KMIPKIT-ELEM-OP-C2S-SIGN",
            "signature_verify": "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY",
            "create": "KMIPKIT-ELEM-OP-C2S-CREATE",
            "create_key_pair": "KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR",
            "create_split_key": "KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY",
            "import": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations_by_id = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for method, operation_id in operations.items():
            operation = operations_by_id[operation_id]
            owner_declarations = [
                declarations_by_id[declaration_id]
                for declaration_id in operation["declarations"]["rust"]
            ]
            owner_requirements = {
                requirement_id
                for owner_declaration in owner_declarations
                for requirement_id in owner_declaration["requirementIds"]
            }
            owner_source = next(
                owner_declaration["approvalSource"]
                for owner_declaration in owner_declarations
                if "approvalSource" in owner_declaration
            )
            for package_id, rust_prefix in (
                ("rust-kmipkit-client", "kmipkit_client::"),
                ("rust-kmipkit", "kmipkit::"),
            ):
                export_name = f"{rust_prefix}Client::{method}_with_options"
                with self.subTest(export=export_name):
                    declaration = declarations[export_name]
                    self.assertEqual(operation["apiState"], declaration["declarationState"])
                    self.assertEqual(operation_id, declaration["capabilityId"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertNotIn("blockedReason", declaration)
                    self.assertEqual(owner_source, declaration["approvalSource"])
                    self.assertTrue(owner_requirements.issubset(declaration["requirementIds"]))
                    self.assertEqual(
                        {
                            "rust": {"packageId": package_id, "name": export_name},
                            "c": {
                                "packageId": "c-kmipkit",
                                "name": "kmipkit_client_execute_with_options_v1",
                            },
                            "java": {
                                "packageId": "java-kmipkit",
                                "name": "org.kmipkit.extensions.Client.executeWithOptions",
                            },
                            "python": {
                                "packageId": "python-kmipkit",
                                "name": "kmipkit.client.Client.execute_with_options",
                            },
                        },
                        declaration["languageMappings"],
                    )
                    self.assertIn(
                        declaration["declarationId"], operation["declarations"]["rust"]
                    )
                    if operation["apiState"] == "verified":
                        self.assertTrue(operation.get("parityEvidenceIds"))
                    self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                    for language in ("c", "java", "python"):
                        self.assertIn(
                            declaration["languageMappings"][language]["name"],
                            client_execution["mappings"][language],
                        )

    def test_t091_client_derive_key_with_options_maps_to_shared_execute_options(self) -> None:
        declaration_id = "rust-observed-c1b5fc8d2a0980e520c3"
        export_name = "kmipkit_client::Client::derive_key_with_options"
        operation_id = "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == operation_id
        )
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        self.assertEqual(export_name, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(operation_id, declaration["capabilityId"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {"packageId": "rust-kmipkit-client", "name": export_name},
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_execute_with_options_v1",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.Client.executeWithOptions",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.Client.execute_with_options",
                },
            },
            declaration["languageMappings"],
        )
        self.assertEqual(
            operation["approvalEvidence"]["path"], declaration["approvalSource"]["path"]
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])
        for language in ("c", "java", "python"):
            self.assertIn(
                declaration["languageMappings"][language]["name"],
                client_execution["mappings"][language],
            )

    def test_t091_derive_key_request_and_outcome_payloads_map_to_approved_operation(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operation_id = "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY"
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == operation_id
        )
        request_canonical = declarations["kmipkit.extensions.client_batch_item_derive_key"]
        outcome_canonical = declarations[
            "kmipkit_client::ClientBatchOutcome::DeriveKey"
        ]
        cases = (
            (
                "kmipkit_client::ClientRequest::DeriveKey",
                "rust-kmipkit-client",
                request_canonical,
            ),
            (
                "kmipkit_client::ClientRequest::DeriveKey::0",
                "rust-kmipkit-client",
                request_canonical,
            ),
            (
                "kmipkit_client::ClientRequest::derive_key",
                "rust-kmipkit-client",
                request_canonical,
            ),
            (
                "kmipkit::ClientRequest::DeriveKey",
                "rust-kmipkit",
                request_canonical,
            ),
            (
                "kmipkit::ClientRequest::DeriveKey::0",
                "rust-kmipkit",
                request_canonical,
            ),
            (
                "kmipkit::ClientRequest::derive_key",
                "rust-kmipkit",
                request_canonical,
            ),
            (
                "kmipkit_client::ClientBatchOutcome::DeriveKey::0",
                "rust-kmipkit-client",
                outcome_canonical,
            ),
            (
                "kmipkit::ClientBatchOutcome::DeriveKey::0",
                "rust-kmipkit",
                outcome_canonical,
            ),
        )

        for export_name, package_id, canonical in cases:
            declaration = declarations[export_name]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0037", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_create_outcome_paths_map_to_accepted_operation(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operation_id = "KMIPKIT-ELEM-OP-C2S-CREATE"
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == operation_id
        )
        canonical = declarations_by_id[
            "T032:create:response:client_batch_item_response_create:rust"
        ]

        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            for suffix in (
                "ClientBatchOutcome::CreateCompleted",
                "ClientBatchOutcome::CreateCompleted::0",
            ):
                export_name = f"{prefix}::{suffix}"
                declaration = declarations[export_name]
                expected_mappings = {
                    language: mapping.copy()
                    for language, mapping in canonical["languageMappings"].items()
                }
                expected_mappings["rust"] = {
                    "packageId": package_id,
                    "name": export_name,
                }
                with self.subTest(export=export_name):
                    self.assertEqual("declared", operation["apiState"])
                    self.assertEqual("approved", operation["ownerDependency"]["state"])
                    self.assertEqual("Accepted", operation["approvalEvidence"]["status"])
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertEqual(operation_id, declaration["capabilityId"])
                    self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                    self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    if "parityEvidenceIds" in canonical:
                        self.assertEqual(
                            canonical["parityEvidenceIds"], declaration["parityEvidenceIds"]
                        )
                    else:
                        self.assertNotIn("parityEvidenceIds", declaration)
                    self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_server_generated_creation_payloads_map_to_accepted_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = []
        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            for operation, variant, request_name, response_id in (
                (
                    "KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR",
                    "CreateKeyPairCompleted",
                    "kmipkit.extensions.client_batch_item_create_key_pair",
                    "T032:create-key-pair:response:client_batch_item_response_create_key_pair:rust",
                ),
                (
                    "KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY",
                    "CreateSplitKeyCompleted",
                    "kmipkit.extensions.client_batch_item_create_split_key",
                    "T032:create-split-key:response:kmipkit_client_batch_item_response_create_split_key_count:rust",
                ),
            ):
                cases.extend(
                    (
                        (
                            f"{prefix}::ClientRequest::{variant.removesuffix('Completed')}::0",
                            package_id,
                            operation,
                            declarations[request_name],
                        ),
                        (
                            f"{prefix}::ClientBatchOutcome::{variant}",
                            package_id,
                            operation,
                            declarations_by_id[response_id],
                        ),
                        (
                            f"{prefix}::ClientBatchOutcome::{variant}::0",
                            package_id,
                            operation,
                            declarations_by_id[response_id],
                        ),
                    )
                )

        self.assertEqual(12, len(cases))
        for export_name, package_id, operation_id, canonical in cases:
            declaration = declarations[export_name]
            operation = operations[operation_id]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("declared", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("Accepted", operation["approvalEvidence"]["status"])
                self.assertEqual("declared", canonical["declarationState"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                if "parityEvidenceIds" in canonical:
                    self.assertEqual(
                        canonical["parityEvidenceIds"], declaration["parityEvidenceIds"]
                    )
                else:
                    self.assertNotIn("parityEvidenceIds", declaration)
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_interop_and_log_request_helpers_map_to_approved_operations(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        cases = (
            (
                "kmipkit_client::ClientRequest::interop",
                "rust-kmipkit-client",
                "kmipkit.extensions.client_batch_item_interop",
                "KMIPKIT-ELEM-OP-C2S-INTEROP",
            ),
            (
                "kmipkit::ClientRequest::interop",
                "rust-kmipkit",
                "kmipkit.extensions.client_batch_item_interop",
                "KMIPKIT-ELEM-OP-C2S-INTEROP",
            ),
            (
                "kmipkit_client::ClientRequest::log",
                "rust-kmipkit-client",
                "kmipkit.extensions.client_batch_item_log",
                "KMIPKIT-ELEM-OP-C2S-LOG",
            ),
            (
                "kmipkit::ClientRequest::log",
                "rust-kmipkit",
                "kmipkit.extensions.client_batch_item_log",
                "KMIPKIT-ELEM-OP-C2S-LOG",
            ),
        )

        for export_name, package_id, canonical_name, operation_id in cases:
            declaration = declarations[export_name]
            canonical = declarations[canonical_name]
            operation = operations[operation_id]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(canonical["parityEvidenceIds"], declaration["parityEvidenceIds"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_declared_attribute_outcome_payloads_map_to_response_accessors(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        variants = (
            "AddAttribute",
            "AdjustAttribute",
            "DeleteAttribute",
            "GetAttributes",
            "ModifyAttribute",
            "SetAttribute",
        )

        cases = []
        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            for variant in variants:
                parent_name = f"{prefix}::ClientBatchOutcome::{variant}"
                cases.append(
                    (
                        f"{parent_name}::0",
                        package_id,
                        declarations[parent_name],
                    )
                )

        self.assertEqual(12, len(cases))
        for export_name, package_id, canonical in cases:
            declaration = declarations[export_name]
            operation = operations[canonical["capabilityId"]]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("declared", canonical["declarationState"])
                self.assertIn("response", expected_mappings["c"]["name"].lower())
                self.assertIn("ClientBatchItemResult", expected_mappings["java"]["name"])
                self.assertIn("BatchItemResult", expected_mappings["python"]["name"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(canonical["parityEvidenceIds"], declaration["parityEvidenceIds"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_verified_batch_outcome_payloads_map_to_response_accessors(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        variants = (
            "Export",
            "GetCompleted",
            "GetUsageAllocation",
            "Import",
            "Interop",
            "JoinSplitKeyCompleted",
            "LocateCompleted",
            "Log",
        )

        cases = []
        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            for variant in variants:
                parent_name = f"{prefix}::ClientBatchOutcome::{variant}"
                cases.append(
                    (
                        f"{parent_name}::0",
                        package_id,
                        declarations[parent_name],
                    )
                )

        self.assertEqual(16, len(cases))
        for export_name, package_id, canonical in cases:
            declaration = declarations[export_name]
            operation = operations[canonical["capabilityId"]]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("verified", canonical["declarationState"])
                self.assertTrue(
                    "response" in expected_mappings["c"]["name"].lower()
                )
                self.assertIn(
                    "ClientBatchItemResult", expected_mappings["java"]["name"]
                )
                self.assertIn(
                    "BatchItemResult", expected_mappings["python"]["name"]
                )
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(canonical["capabilityId"], declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(canonical["parityEvidenceIds"], declaration["parityEvidenceIds"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_pending_outcome_metadata_maps_to_existing_four_language_result(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capabilities = {
            row["capabilityId"]: row for row in self.checked_in_manifest["capabilities"]
        }
        metadata_function = next(
            row
            for row in self.checked_in_manifest["functions"]
            if row["id"] == "client_batch_item_response_metadata_v2"
        )
        metadata_mappings = {
            "c": {
                "packageId": "c-kmipkit",
                "name": metadata_function["c"]["symbol"],
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": ".".join(
                    (
                        metadata_function["java"]["package"],
                        metadata_function["java"]["class"],
                        metadata_function["java"]["method"],
                    )
                ),
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": ".".join(
                    (
                        metadata_function["python"]["module"],
                        metadata_function["python"]["function"],
                    )
                ),
            },
        }
        pending_discriminator_mappings = {
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_client_batch_item_metadata_v2_t.outcome_kind",
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.ClientBatchItemResult.outcomeKind",
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.client.BatchItemResult.outcome",
            },
        }

        cases = []
        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            response_basis = declarations[f"{prefix}::ClientBatchOutcome::response"]
            operation_basis = declarations[f"{prefix}::ClientBatchOutcome::operation"]
            correlation_basis = declarations[
                f"{prefix}::ClientBatchOutcome::asynchronous_correlation_value"
            ]

            def mappings_from(basis: dict[str, object], export_name: str) -> dict[str, dict[str, str]]:
                mappings = {
                    language: mapping.copy()
                    for language, mapping in basis["languageMappings"].items()
                }
                mappings["rust"] = {"packageId": package_id, "name": export_name}
                return mappings

            def add_case(
                export_name: str,
                basis: dict[str, object],
                mappings: dict[str, dict[str, str]],
                requirement_ids: list[str] | None = None,
            ) -> None:
                cases.append(
                    (
                        export_name,
                        basis,
                        mappings,
                        requirement_ids or basis["requirementIds"],
                    )
                )

            add_case(
                f"{prefix}::ClientBatchOutcome::Pending",
                response_basis,
                {
                    **pending_discriminator_mappings,
                    "rust": {
                        "packageId": package_id,
                        "name": f"{prefix}::ClientBatchOutcome::Pending",
                    },
                },
            )
            for export_name in (
                f"{prefix}::ClientBatchOutcome::Pending::0",
                f"{prefix}::PendingOutcome",
                f"{prefix}::PendingOutcome::response",
            ):
                add_case(
                    export_name,
                    response_basis,
                    mappings_from(response_basis, export_name),
                )
            add_case(
                f"{prefix}::PendingOutcome::operation",
                operation_basis,
                mappings_from(operation_basis, f"{prefix}::PendingOutcome::operation"),
            )
            add_case(
                f"{prefix}::PendingOutcome::result",
                operation_basis,
                {
                    **metadata_mappings,
                    "rust": {
                        "packageId": package_id,
                        "name": f"{prefix}::PendingOutcome::result",
                    },
                },
                metadata_function["requirementIds"],
            )
            add_case(
                f"{prefix}::ClientBatchOutcome::result",
                operation_basis,
                {
                    **metadata_mappings,
                    "rust": {
                        "packageId": package_id,
                        "name": f"{prefix}::ClientBatchOutcome::result",
                    },
                },
                metadata_function["requirementIds"],
            )
            add_case(
                f"{prefix}::PendingOutcome::asynchronous_correlation_value",
                correlation_basis,
                mappings_from(
                    correlation_basis,
                    f"{prefix}::PendingOutcome::asynchronous_correlation_value",
                ),
            )

        self.assertEqual(16, len(cases))
        for export_name, basis, expected_mappings, requirement_ids in cases:
            declaration = declarations[export_name]
            capability = capabilities[basis["capabilityId"]]
            with self.subTest(export=export_name):
                self.assertEqual("declared", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(basis["capabilityId"], declaration["capabilityId"])
                self.assertEqual(requirement_ids, declaration["requirementIds"])
                self.assertEqual(basis["approvalSource"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertEqual(basis["approvalSource"], capability["approvalEvidence"])
                for language, mapping in expected_mappings.items():
                    self.assertIn(mapping["name"], capability["mappings"][language])

    def test_t091_validate_request_and_outcome_paths_map_to_verified_operation(
        self,
    ) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operation_id = "KMIPKIT-ELEM-OP-C2S-VALIDATE"
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == operation_id
        )
        request_canonical = declarations[
            "kmipkit.extensions.client_batch_item_validate"
        ]
        response_canonical = declarations["org.kmipkit.extensions.ClientBatchItemResult.validateResponse"]
        response_payload_canonical = declarations[
            "org.kmipkit.extensions.ClientBatchItemResult.ValidateResponse.validityIndicatorRaw"
        ]
        cases = []
        for prefix, package_id in (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        ):
            cases.extend(
                (
                    (f"{prefix}::ClientRequest::Validate", package_id, request_canonical),
                    (f"{prefix}::ClientRequest::Validate::0", package_id, request_canonical),
                    (f"{prefix}::ClientRequest::validate", package_id, request_canonical),
                    (
                        f"{prefix}::ClientBatchOutcome::Validate",
                        package_id,
                        response_canonical,
                    ),
                    (
                        f"{prefix}::ClientBatchOutcome::Validate::0",
                        package_id,
                        response_payload_canonical,
                    ),
                )
            )

        self.assertEqual(10, len(cases))
        for export_name, package_id, canonical in cases:
            declaration = declarations[export_name]
            expected_mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            expected_mappings["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            with self.subTest(export=export_name):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("KMIPKIT-0042", operation["featureSpecId"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertEqual(canonical["requirementIds"], declaration["requirementIds"])
                self.assertEqual(operation["approvalEvidence"], declaration["approvalSource"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(declaration["declarationId"], operation["declarations"]["rust"])

    def test_t091_verified_client_operations_map_options_to_shared_execute(self) -> None:
        cases = (
            (
                "rust-observed-140bdc0f9fae454fc042",
                "kmipkit_client::Client::export_with_options",
                "KMIPKIT-ELEM-OP-C2S-EXPORT",
            ),
            (
                "rust-observed-ebb77d7fd426167a9803",
                "kmipkit_client::Client::join_split_key_with_options",
                "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
            ),
            (
                "rust-observed-5195c370b6caa0ab9688",
                "kmipkit_client::Client::obtain_lease_with_options",
                "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
            ),
            (
                "rust-observed-96cbaa2e43262aca95ac",
                "kmipkit_client::Client::recertify_with_options",
                "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
            ),
            (
                "rust-observed-47920171a3daac07bad8",
                "kmipkit_client::Client::register_with_options",
                "KMIPKIT-ELEM-OP-C2S-REGISTER",
            ),
            (
                "rust-observed-f01e496c4f3e0f0b076e",
                "kmipkit_client::Client::rekey_key_pair_with_options",
                "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
            ),
            (
                "rust-observed-be39cf02d9bfc818336d",
                "kmipkit_client::Client::rekey_with_options",
                "KMIPKIT-ELEM-OP-C2S-RE-KEY",
            ),
            (
                "rust-observed-e3bff71088cf1caee803",
                "kmipkit_client::Client::revoke_with_options",
                "KMIPKIT-ELEM-OP-C2S-REVOKE",
            ),
        )
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for declaration_id, export_name, operation_id in cases:
            with self.subTest(export=export_name):
                declaration = next(
                    row
                    for row in self.checked_in_manifest["publicDeclarations"]
                    if row["declarationId"] == declaration_id
                )
                base_declaration = next(
                    row
                    for row in self.checked_in_manifest["publicDeclarations"]
                    if row["exportName"] == export_name.removesuffix("_with_options")
                )
                operation = next(
                    row
                    for row in self.checked_in_manifest["operations"]
                    if row["operationId"] == operation_id
                )

                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(base_declaration["requirementIds"], declaration["requirementIds"])
                self.assertEqual(
                    {
                        "rust": {"packageId": "rust-kmipkit-client", "name": export_name},
                        "c": {
                            "packageId": "c-kmipkit",
                            "name": "kmipkit_client_execute_with_options_v1",
                        },
                        "java": {
                            "packageId": "java-kmipkit",
                            "name": "org.kmipkit.extensions.Client.executeWithOptions",
                        },
                        "python": {
                            "packageId": "python-kmipkit",
                            "name": "kmipkit.client.Client.execute_with_options",
                        },
                    },
                    declaration["languageMappings"],
                )
                self.assertEqual(
                    operation["approvalEvidence"]["path"],
                    declaration["approvalSource"]["path"],
                )
                self.assertIn(declaration_id, operation["declarations"]["rust"])
                for language in ("c", "java", "python"):
                    self.assertIn(
                        declaration["languageMappings"][language]["name"],
                        client_execution["mappings"][language],
                    )

    def test_t091_root_client_reexports_map_to_the_same_verified_operations(self) -> None:
        methods = (
            ("certify", "KMIPKIT-ELEM-OP-C2S-CERTIFY"),
            ("check", "KMIPKIT-ELEM-OP-C2S-CHECK"),
            ("derive_key", "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY"),
            ("export", "KMIPKIT-ELEM-OP-C2S-EXPORT"),
            ("join_split_key", "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY"),
            ("obtain_lease", "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE"),
            ("recertify", "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"),
            ("register", "KMIPKIT-ELEM-OP-C2S-REGISTER"),
            ("rekey_key_pair", "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"),
            ("rekey", "KMIPKIT-ELEM-OP-C2S-RE-KEY"),
            ("revoke", "KMIPKIT-ELEM-OP-C2S-REVOKE"),
        )
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )

        for method, operation_id in methods:
            operation = operations[operation_id]
            with self.subTest(method=method):
                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                for suffix in ("", "_with_options"):
                    client_name = f"kmipkit_client::Client::{method}{suffix}"
                    facade_name = f"kmipkit::Client::{method}{suffix}"
                    client = declarations[("rust-kmipkit-client", client_name)]
                    facade = declarations[("rust-kmipkit", facade_name)]
                    expected_mappings = dict(client["languageMappings"])
                    expected_mappings["rust"] = {
                        "packageId": "rust-kmipkit",
                        "name": facade_name,
                    }

                    self.assertEqual("verified", client["declarationState"])
                    self.assertEqual("verified", facade["declarationState"])
                    self.assertEqual(operation_id, facade["capabilityId"])
                    self.assertNotIn("candidateSpecId", facade)
                    self.assertEqual(client["requirementIds"], facade["requirementIds"])
                    self.assertEqual(client["approvalSource"], facade["approvalSource"])
                    self.assertEqual(client["parityEvidenceIds"], facade["parityEvidenceIds"])
                    self.assertEqual(expected_mappings, facade["languageMappings"])
                    self.assertIn(facade["declarationId"], operation["declarations"]["rust"])
                    for language in ("c", "java", "python"):
                        if suffix:
                            self.assertIn(
                                facade["languageMappings"][language]["name"],
                                client_execution["mappings"][language],
                            )
                        else:
                            self.assertTrue(operation["declarations"][language])

    def test_t091_root_client_execution_and_async_reexports_keep_owner_state(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        cases = (
            ("Client", "client_execution"),
            ("Client::new", "client_execution"),
            ("Client::execute", "client_execution"),
            ("Client::execute_poll", "client_asynchronous_operations"),
            ("Client::execute_cancel", "client_asynchronous_operations"),
            ("Client::execute_process", "client_asynchronous_operations"),
            (
                "Client::execute_query_async_requests",
                "client_asynchronous_operations",
            ),
        )
        capabilities = {
            row["capabilityId"]: row for row in self.checked_in_manifest["capabilities"]
        }

        for suffix, capability_id in cases:
            with self.subTest(export=suffix):
                canonical_name = f"kmipkit_client::{suffix}"
                facade_name = f"kmipkit::{suffix}"
                canonical = declarations[("rust-kmipkit-client", canonical_name)]
                facade = declarations[("rust-kmipkit", facade_name)]
                capability = capabilities[capability_id]
                expected_mappings = dict(canonical["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit",
                    "name": facade_name,
                }

                self.assertEqual("declared", canonical["declarationState"])
                self.assertEqual("declared", facade["declarationState"])
                self.assertEqual(capability_id, canonical["capabilityId"])
                self.assertEqual(capability_id, facade["capabilityId"])
                self.assertNotIn("candidateSpecId", facade)
                self.assertEqual(canonical["requirementIds"], facade["requirementIds"])
                self.assertEqual(canonical["approvalSource"], facade["approvalSource"])
                self.assertEqual(
                    canonical.get("parityEvidenceIds"),
                    facade.get("parityEvidenceIds"),
                )
                self.assertEqual(expected_mappings, facade["languageMappings"])
                self.assertEqual("declared", capability["capabilityState"])
                self.assertIn(facade_name, capability["mappings"]["rust"])

    def test_t091_root_profile_reexports_keep_incomplete_claim_blocked(self) -> None:
        methods = (
            "execute_with_profiles",
            "execute_with_profiles_and_context",
            "execute_with_profiles_and_context_and_options",
            "execute_with_profiles_and_options",
        )
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        baseline = next(
            row
            for row in self.catalog["profiles"]
            if row["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual("evidence_incomplete", baseline["claim_state"])
        for method in methods:
            with self.subTest(method=method):
                client_name = f"kmipkit_client::Client::{method}"
                facade_name = f"kmipkit::Client::{method}"
                canonical = declarations[("rust-kmipkit-client", client_name)]
                facade = declarations[("rust-kmipkit", facade_name)]
                expected_mappings = dict(canonical["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit",
                    "name": facade_name,
                }

                self.assertEqual("declared", canonical["declarationState"])
                self.assertEqual("declared", facade["declarationState"])
                self.assertEqual("client_profile_validation", facade["capabilityId"])
                self.assertNotIn("candidateSpecId", facade)
                self.assertEqual(canonical["requirementIds"], facade["requirementIds"])
                self.assertEqual(canonical["approvalSource"], facade["approvalSource"])
                self.assertEqual(expected_mappings, facade["languageMappings"])
                self.assertIn(facade_name, capability["mappings"]["rust"])

    def test_t091_root_guarded_protection_reexports_keep_owner_state(self) -> None:
        methods = (
            "execute_guarded_protection_item",
            "execute_guarded_protection_item_with_asynchronous_indicator",
        )
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "usage_allocation_guard"
        )

        self.assertEqual("declared", capability["capabilityState"])
        self.assertEqual("specs/043-public-api-bindings/spec.md", capability["approvalEvidence"]["path"])
        self.assertEqual("Approved", capability["approvalEvidence"]["status"])
        for method in methods:
            with self.subTest(method=method):
                client_name = f"kmipkit_client::Client::{method}"
                facade_name = f"kmipkit::Client::{method}"
                canonical = declarations[("rust-kmipkit-client", client_name)]
                facade = declarations[("rust-kmipkit", facade_name)]
                expected_mappings = dict(canonical["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit",
                    "name": facade_name,
                }

                self.assertEqual("declared", canonical["declarationState"])
                self.assertEqual("declared", facade["declarationState"])
                self.assertEqual("usage_allocation_guard", canonical["capabilityId"])
                self.assertEqual("usage_allocation_guard", facade["capabilityId"])
                self.assertNotIn("candidateSpecId", facade)
                self.assertEqual(canonical["requirementIds"], facade["requirementIds"])
                self.assertEqual(canonical["approvalSource"], facade["approvalSource"])
                self.assertEqual(expected_mappings, facade["languageMappings"])
                self.assertIn(facade_name, capability["mappings"]["rust"])

    def test_t091_remaining_root_client_methods_map_to_verified_operations(self) -> None:
        methods = (
            ("get_attribute_list", "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST"),
            ("get_constraints", "KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS"),
            ("reprovision", "KMIPKIT-ELEM-OP-C2S-RE-PROVISION"),
            ("reprovision_with_options", "KMIPKIT-ELEM-OP-C2S-RE-PROVISION"),
            ("rng_retrieve", "KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE"),
            ("rng_seed", "KMIPKIT-ELEM-OP-C2S-RNG-SEED"),
            ("set_constraints", "KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS"),
            ("set_defaults", "KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS"),
        )
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )
        shared_options = {
            "rust": {
                "packageId": "rust-kmipkit-client",
                "name": "kmipkit_client::Client::execute_with_options",
            },
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_client_execute_with_options_v1",
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.Client.executeWithOptions",
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.client.Client.execute_with_options",
            },
        }

        for method, operation_id in methods:
            with self.subTest(method=method):
                client_name = f"kmipkit_client::Client::{method}"
                facade_name = f"kmipkit::Client::{method}"
                client = declarations[("rust-kmipkit-client", client_name)]
                facade = declarations[("rust-kmipkit", facade_name)]
                operation = operations[operation_id]
                expected_mappings = dict(client["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit",
                    "name": facade_name,
                }

                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual("verified", client["declarationState"])
                self.assertEqual(operation_id, client["capabilityId"])
                self.assertEqual("verified", facade["declarationState"])
                self.assertEqual(operation_id, facade["capabilityId"])
                self.assertNotIn("candidateSpecId", facade)
                self.assertEqual(client["requirementIds"], facade["requirementIds"])
                self.assertEqual(client["approvalSource"], facade["approvalSource"])
                self.assertEqual(client["parityEvidenceIds"], facade["parityEvidenceIds"])
                self.assertEqual(expected_mappings, facade["languageMappings"])
                self.assertIn(facade["declarationId"], operation["declarations"]["rust"])

                if method == "reprovision_with_options":
                    self.assertEqual(
                        {
                            "packageId": "rust-kmipkit-client",
                            "name": client_name,
                        },
                        client["languageMappings"]["rust"],
                    )
                    for language in ("c", "java", "python"):
                        self.assertIn(
                            shared_options[language]["name"],
                            client_execution["mappings"][language],
                        )
                else:
                    for language in ("c", "java", "python"):
                        self.assertTrue(operation["declarations"][language])

    def test_t091_set_constraints_client_method_maps_to_matching_operation_constructor(self) -> None:
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] == "kmipkit_client::Client::set_constraints"
        )

        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS", declaration["capabilityId"])
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::Client::set_constraints",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_set_constraints",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.setConstraints",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.client_batch_item_set_constraints",
                },
            },
            declaration["languageMappings"],
        )

    def test_t091_check_request_variant_maps_to_verified_check_operation(self) -> None:
        declaration_id = "rust-observed-8a97c84f665c60d4b3c4"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-CHECK"
        )

        self.assertEqual("kmipkit::ClientRequest::Check", declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("KMIPKIT-ELEM-OP-C2S-CHECK", declaration["capabilityId"])
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit",
                    "name": "kmipkit::ClientRequest::Check",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_check",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.check",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.client_batch_item_check",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_t091_obtain_lease_facade_maps_to_verified_operation(self) -> None:
        declaration_id = "rust-observed-6d4c2acdd725a6525a7b"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE"
        )

        self.assertEqual("kmipkit::ClientRequest::obtain_lease", declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(
            "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE", declaration["capabilityId"]
        )
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit",
                    "name": "kmipkit::ClientRequest::obtain_lease",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_obtain_lease",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.obtainLease",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.client_batch_item_obtain_lease",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_t091_get_usage_allocation_facade_maps_to_verified_operation(self) -> None:
        declaration_id = "rust-observed-15202a6508bea1672eae"
        declaration = next(
            row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"] == declaration_id
        )
        operation = next(
            row
            for row in self.checked_in_manifest["operations"]
            if row["operationId"] == "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION"
        )

        self.assertEqual(
            "kmipkit::ClientRequest::get_usage_allocation", declaration["exportName"]
        )
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(
            "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION", declaration["capabilityId"]
        )
        self.assertNotIn("candidateSpecId", declaration)
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit",
                    "name": "kmipkit::ClientRequest::get_usage_allocation",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_get_usage_allocation",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.getUsageAllocation",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.client_batch_item_get_usage_allocation",
                },
            },
            declaration["languageMappings"],
        )
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_t091_client_operation_variants_map_to_four_language_raw_metadata(self) -> None:
        expected_variants = {
            "AddAttribute",
            "AdjustAttribute",
            "Certify",
            "Check",
            "DelegatedLogin",
            "DeleteAttribute",
            "DeriveKey",
            "DiscoverVersions",
            "Export",
            "Get",
            "GetAttributeList",
            "GetAttributes",
            "GetConstraints",
            "GetUsageAllocation",
            "Import",
            "Interop",
            "JoinSplitKey",
            "Locate",
            "Log",
            "Login",
            "Logout",
            "MacVerify",
            "ModifyAttribute",
            "ObtainLease",
            "QueryAsyncRequests",
            "ReProvision",
            "Recertify",
            "Register",
            "Rekey",
            "RekeyKeyPair",
            "Revoke",
            "RngRetrieve",
            "RngSeed",
            "SetAttribute",
            "SetConstraints",
            "SetDefaults",
            "SignatureVerify",
            "Validate",
        }
        declarations = {
            row["exportName"]: row for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = self.checked_in_manifest["operations"]
        operation_by_name = {
            "".join(character.lower() for character in row["name"] if character.isalnum()): row
            for row in operations
        }
        operation_name_overrides = {
            "QueryAsyncRequests": "queryasynchronousrequests",
        }
        mappings = {
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_client_batch_item_metadata_v2_t.operation",
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.ClientBatchItemMetadataV2.operation",
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.client.BatchItemMetadataV2.operation",
            },
        }
        operation_ids = {row["operationId"]: row for row in operations}
        declarations_by_capability: dict[str, list[dict[str, object]]] = {}
        for row in declarations.values():
            declarations_by_capability.setdefault(row["capabilityId"], []).append(row)

        for variant in sorted(expected_variants):
            operation_key = operation_name_overrides.get(
                variant,
                "".join(character.lower() for character in variant if character.isalnum()),
            )
            operation = operation_by_name.get(operation_key)
            self.assertIsNotNone(operation, f"missing KMIP 2.1 operation for {variant}")
            if operation is None:
                continue
            operation = operation_ids[operation["operationId"]]
            self.assertEqual("client_1_0", operation["scope"])
            self.assertEqual("approved", operation["ownerDependency"]["state"])

            if variant == "QueryAsyncRequests":
                canonical = declarations[
                    "kmipkit_client::Client::execute_query_async_requests"
                ]
            else:
                canonical = next(
                    row
                    for row in declarations_by_capability[operation["operationId"]]
                    if row["packageId"].startswith("rust-")
                    and not row.get("candidateSpecId")
                    and "ClientOperation::" not in row["exportName"]
                )
            expected_requirement_ids = canonical["requirementIds"]
            expected_approval = canonical.get("approvalSource")
            parity_evidence_ids = sorted(
                set(operation.get("parityEvidenceIds", []))
                | {"tools/api_manifest/tests/test_manifest_v2.py"}
            )

            for rust_package, rust_prefix in (
                ("rust-kmipkit-client", "kmipkit_client"),
                ("rust-kmipkit", "kmipkit"),
            ):
                export_name = f"{rust_prefix}::ClientOperation::{variant}"
                with self.subTest(export=export_name):
                    declaration = declarations[export_name]
                    expected_mappings = {
                        "rust": {"packageId": rust_package, "name": export_name},
                        **mappings,
                    }
                    self.assertEqual(operation["apiState"], declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertNotIn("blockedReason", declaration)
                    self.assertEqual(operation["operationId"], declaration["capabilityId"])
                    self.assertEqual(expected_requirement_ids, declaration["requirementIds"])
                    if expected_approval is not None:
                        self.assertEqual(expected_approval, declaration["approvalSource"])
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    self.assertTrue(
                        set(parity_evidence_ids)
                        <= set(declaration.get("parityEvidenceIds", []))
                    )
                    self.assertIn(
                        declaration["declarationId"],
                        operation["declarations"]["rust"],
                    )

    def test_t091_safe_client_error_taxonomy_maps_to_existing_error_metadata(self) -> None:
        declaration_suffixes = {
            "ClientCauseCategory": ("cause-type",),
            "ClientCauseCategory::InvalidInput": ("cause-invalid-input",),
            "ClientCauseCategory::Protocol": ("cause-protocol",),
            "ClientCauseCategory::Protocol::0": ("cause-protocol",),
            "ClientCauseCategory::Transport": ("cause-transport",),
            "ClientCauseCategory::Transport::0": ("cause-transport",),
            "ClientCauseCategory::Other": ("cause-other",),
            "ClientErrorCategory::Validation": ("category-validation",),
            "ClientError::Validation": ("validation-error",),
            "ClientError::Validation::cause": ("validation-cause",),
            "ClientError::Validation::delivery_state": ("delivery-state",),
            "ClientError::ProtocolResponse": ("protocol-response-error",),
            "ClientError::Protocol::delivery_state": ("delivery-state",),
            "ClientError::ProtocolResponse::delivery_state": ("delivery-state",),
            "ClientError::Transport::0": ("transport-error",),
            "ClientError::ProfileValidation::delivery_state": ("delivery-state",),
        }
        foreign_names = {
            "cause-type": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientErrorCause",
            ),
            "cause-invalid-input": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientErrorCause.INVALID_INPUT",
            ),
            "cause-protocol": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientErrorCause",
            ),
            "cause-transport": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientErrorCause",
            ),
            "cause-other": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientErrorCause.OTHER",
            ),
            "category-validation": (
                "org.kmipkit.ClientOperationException.categoryCode",
                "kmipkit.errors.ClientErrorCategory.VALIDATION",
            ),
            "validation-error": (
                "org.kmipkit.ClientOperationException",
                "kmipkit.errors.ClientExecutionError",
            ),
            "validation-cause": (
                "org.kmipkit.ClientOperationException.causeCode",
                "kmipkit.errors.ClientExecutionError.cause",
            ),
            "delivery-state": (
                "org.kmipkit.ClientOperationException.deliveryStateCode",
                "kmipkit.errors.ClientExecutionError.delivery",
            ),
            "protocol-response-error": (
                "org.kmipkit.ProtocolException",
                "kmipkit.errors.ClientExecutionError",
            ),
            "transport-error": (
                "org.kmipkit.TransportException",
                "kmipkit.errors.ClientExecutionError",
            ),
        }
        declarations = {
            row["exportName"]: row for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )
        canonical = declarations["kmipkit_client::ClientError::category"]
        requirements = sorted(
            set(canonical["requirementIds"])
            | {
                "KMIPKIT-0003-FR-001",
                "KMIPKIT-0003-FR-004",
                "KMIPKIT-0003-FR-005",
                "KMIPKIT-0003-FR-006",
                "KMIPKIT-0043-FR-007",
                "KMIPKIT-0043-FR-008",
                "KMIPKIT-0043-FR-009",
                "KMIPKIT-0043-FR-010",
            }
        )
        prefixes = (
            ("kmipkit_client", "rust-kmipkit-client"),
            ("kmipkit", "rust-kmipkit"),
        )

        for prefix, rust_package in prefixes:
            for suffix, (group,) in declaration_suffixes.items():
                export_name = f"{prefix}::{suffix}"
                with self.subTest(export=export_name):
                    declaration = declarations[export_name]
                    java_name, python_name = foreign_names[group]
                    expected_mappings = {
                        "rust": {"packageId": rust_package, "name": export_name},
                        "c": {
                            "packageId": "c-kmipkit",
                            "name": "kmipkit_last_error_get",
                        },
                        "java": {"packageId": "java-kmipkit", "name": java_name},
                        "python": {"packageId": "python-kmipkit", "name": python_name},
                    }
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertNotIn("blockedReason", declaration)
                    self.assertEqual("client_execution", declaration["capabilityId"])
                    self.assertEqual(requirements, declaration["requirementIds"])
                    self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    for language, mapping in expected_mappings.items():
                        self.assertIn(mapping["name"], capability["mappings"][language])

        profile_validation_suffixes = {
            "ClientError::ProfileValidation": "profile-validation-error",
            "ClientError::ProfileValidation::error": "profile-validation-details",
            "ClientError::profile_validation_error": "profile-validation-error",
        }
        profile_validation_foreign_names = {
            "profile-validation-error": (
                "kmipkit_last_error_profile_validation_error_get_v1",
                "org.kmipkit.ClientOperationException.profileValidation",
                "kmipkit.errors.ClientExecutionError.profile_validation",
            ),
            "profile-validation-details": (
                "kmipkit_profile_validation_error_t",
                "org.kmipkit.ProfileValidationDetails",
                "kmipkit.errors.ProfileValidationDetails",
            ),
        }
        profile_validation_requirements = [
            "KMIPKIT-0010-FR-012",
            "KMIPKIT-0010-FR-013",
            "KMIPKIT-0010-FR-019",
            "KMIPKIT-0043-FR-007",
            "KMIPKIT-0043-FR-013",
            "KMIPKIT-0043-FR-021",
        ]
        profile_validation_approval = declarations[
            "kmipkit_client::ClientError::ProfileValidation"
        ]["approvalSource"]
        self.assertEqual("feature-specification", profile_validation_approval["kind"])
        self.assertEqual(
            "specs/010-profile-conformance/spec.md", profile_validation_approval["path"]
        )
        self.assertEqual("Accepted", profile_validation_approval["status"])
        profile_capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_profile_validation"
        )
        for prefix, rust_package in prefixes:
            for suffix, group in profile_validation_suffixes.items():
                export_name = f"{prefix}::{suffix}"
                java_name, python_name = profile_validation_foreign_names[group][1:]
                c_name = profile_validation_foreign_names[group][0]
                expected_mappings = {
                    "rust": {"packageId": rust_package, "name": export_name},
                    "c": {"packageId": "c-kmipkit", "name": c_name},
                    "java": {"packageId": "java-kmipkit", "name": java_name},
                    "python": {"packageId": "python-kmipkit", "name": python_name},
                }
                with self.subTest(profile_validation=export_name):
                    declaration = declarations[export_name]
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertNotIn("candidateSpecId", declaration)
                    self.assertNotIn("blockedReason", declaration)
                    self.assertEqual(
                        "client_profile_validation", declaration["capabilityId"]
                    )
                    self.assertEqual(
                        profile_validation_requirements, declaration["requirementIds"]
                    )
                    self.assertEqual(
                        profile_validation_approval, declaration["approvalSource"]
                    )
                    self.assertEqual(expected_mappings, declaration["languageMappings"])
                    for language, mapping in expected_mappings.items():
                        self.assertIn(
                            mapping["name"], profile_capability["mappings"][language]
                        )
                    serialized = json.dumps(declaration).casefold()
                    self.assertNotIn("response_ttlv", serialized)
                    self.assertNotIn("response-ttlv", serialized)

        remain_unmapped = (
            "ClientError::Protocol::error",
            "ClientError::ProtocolResponse::error",
            "ClientError::ServerResult::0",
            "ClientError::server_operation_result",
            "ClientError::server_status",
            "ClientError::validation",
            "ClientError::protocol",
            "ClientError::transport",
            "ClientError::server_result",
            "ClientError::profile_validation",
        )
        for prefix, _ in prefixes:
            for suffix in remain_unmapped:
                declaration = declarations[f"{prefix}::{suffix}"]
                with self.subTest(unmapped=f"{prefix}::{suffix}"):
                    self.assertEqual("blocked", declaration["declarationState"])
                    self.assertEqual("KMIPKIT-0007", declaration["candidateSpecId"])

    def test_t091_generic_ffi_errors_do_not_map_to_client_cause_other(self) -> None:
        declaration_ids = {
            "KMIPKIT_ERROR_INTERNAL:c",
            "java-observed-d645cd79e0aaa7fea0bb",
            "python-kmipkit-error",
        }
        declarations = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        capability = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "ffi_runtime_metadata"
        )

        for declaration_id in declaration_ids:
            with self.subTest(declaration=declaration_id):
                declaration = declarations[declaration_id]
                self.assertEqual("blocked", declaration["declarationState"])
                self.assertEqual("KMIPKIT-0043", declaration["candidateSpecId"])
                self.assertIn("ClientCauseCategory::Other", declaration["blockedReason"])
                self.assertNotIn("languageMappings", declaration)
                self.assertNotIn("approvalSource", declaration)

        self.assertNotIn(
            "kmipkit::ClientCauseCategory::Other", capability["mappings"]["rust"]
        )

    def test_t091_client_crate_operation_entries_map_to_verified_functions(self) -> None:
        cases = (
            (
                "rust-observed-4d05f9d9fc8377cd7a6f",
                "kmipkit_client::ClientRequest::Check",
                "KMIPKIT-ELEM-OP-C2S-CHECK",
                "kmipkit_client_batch_item_check",
                "org.kmipkit.extensions.ClientBatchItem.check",
                "kmipkit.extensions.client_batch_item_check",
            ),
            (
                "rust-observed-12029d442c5430d1f445",
                "kmipkit_client::ClientRequest::get_usage_allocation",
                "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
                "kmipkit_client_batch_item_get_usage_allocation",
                "org.kmipkit.extensions.ClientBatchItem.getUsageAllocation",
                "kmipkit.extensions.client_batch_item_get_usage_allocation",
            ),
            (
                "rust-observed-1966bfc9acb5c4e64ccb",
                "kmipkit_client::ClientRequest::obtain_lease",
                "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
                "kmipkit_client_batch_item_obtain_lease",
                "org.kmipkit.extensions.ClientBatchItem.obtainLease",
                "kmipkit.extensions.client_batch_item_obtain_lease",
            ),
        )
        declarations = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }

        for declaration_id, export_name, operation_id, c_name, java_name, python_name in cases:
            with self.subTest(export=export_name):
                declaration = declarations[declaration_id]
                operation = operations[operation_id]
                self.assertEqual(export_name, declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(
                    set(operation["parityEvidenceIds"])
                    <= set(declaration.get("parityEvidenceIds", []))
                )
                self.assertEqual(operation_id, declaration["capabilityId"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(
                    {
                        "rust": {"packageId": "rust-kmipkit-client", "name": export_name},
                        "c": {"packageId": "c-kmipkit", "name": c_name},
                        "java": {"packageId": "java-kmipkit", "name": java_name},
                        "python": {"packageId": "python-kmipkit", "name": python_name},
                    },
                    declaration["languageMappings"],
                )
                self.assertIn(declaration_id, operations[operation_id]["declarations"]["rust"])

    def test_obtain_lease_function_contract_uses_obtain_lease_models(self) -> None:
        functions = {
            row["id"]: row for row in self.checked_in_manifest["functions"]
        }

        self.assertEqual(
            {
                "request": "ObtainLeaseRequest",
                "response": "ObtainLeaseResponse",
            },
            {
                "request": functions["client_batch_item_obtain_lease"]["rust"][
                    "parameters"
                ][0]["type"],
                "response": functions["client_batch_item_response_obtain_lease"][
                    "rust"
                ]["returnType"],
            },
        )

    def test_optional_unique_identifier_is_nullable_in_java_function_mappings(self) -> None:
        functions = {
            row["id"]: row for row in self.checked_in_manifest["functions"]
        }
        for function_id in (
            "client_batch_item_get_usage_allocation",
            "client_batch_item_obtain_lease",
        ):
            with self.subTest(function=function_id):
                parameters = functions[function_id]["java"]["parameters"]
                unique_identifier = next(
                    parameter
                    for parameter in parameters
                    if parameter["name"] == "uniqueIdentifier"
                )
                self.assertIs(unique_identifier.get("nullable"), True)

    def test_ttlv_decoder_python_exception_runtime_members_are_exactly_excluded(self) -> None:
        expected = {
            "kmipkit.errors.TtlvDecodeError.__init__",
            "kmipkit.errors.TtlvDecodeError.args",
            "kmipkit.errors.TtlvDecodeError.category",
        }
        exclusions = {
            row["exportName"]: row
            for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"] in expected
        }
        self.assertEqual(set(exclusions), expected)
        for name, exclusion in exclusions.items():
            self.assertTrue(exclusion["reason"], name)
            self.assertTrue(exclusion["evidence"], name)
            self.assertEqual(
                exclusion["approvalSource"]["path"],
                "specs/005-ttlv-wire-codec/spec.md",
            )
            self.assertEqual(
                exclusion["approvalSource"]["evidence"],
                "**Created**: 2026-10-04",
            )

    def test_ttlv_decoder_python_error_properties_are_inventoried_as_fields(self) -> None:
        expected = {
            "kmipkit.errors.TtlvDecodeError.kind",
            "kmipkit.errors.TtlvDecodeError.byte_offset",
        }
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
            and row["exportName"] in expected
        }
        self.assertEqual(set(declarations), expected)
        self.assertTrue(all(row["kind"] == "field" for row in declarations.values()))

    def test_ttlv_codec_java_static_facade_is_exactly_excluded(self) -> None:
        name = "org.kmipkit.ttlv.TtlvCodec"
        exclusion = next(
            (
                row
                for row in self.checked_in_manifest["approvedExclusions"]
                if row["packageId"] == "java-kmipkit" and row["exportName"] == name
            ),
            None,
        )
        self.assertIsNotNone(exclusion)
        self.assertTrue(exclusion["reason"])
        self.assertTrue(exclusion["evidence"])
        self.assertEqual(
            exclusion["approvalSource"]["path"],
            "specs/005-ttlv-wire-codec/spec.md",
        )

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
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.<init>"),
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
            ("python-kmipkit", "kmipkit.metadata.Version.major"),
            ("python-kmipkit", "kmipkit.metadata.Version.minor"),
            ("python-kmipkit", "kmipkit.metadata.Version.patch"),
        })
        for key in blocked_exports:
            with self.subTest(export=key):
                declaration = declarations[key]
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertEqual(declaration["candidateSpecId"], "KMIPKIT-0043")
                self.assertEqual(declaration["capabilityId"], "public_binding_surface")
                self.assertEqual(declaration["requirementIds"], ["KMIPKIT-0043-FR-021"])
                self.assertNotIn("languageMappings", declaration)

        for key in (
            ("java-kmipkit", "org.kmipkit.RuntimeMetadata.Version.abiVersion"),
            ("python-kmipkit", "kmipkit.metadata.Version.abi_version"),
            ("python-kmipkit", "kmipkit.extensions.validate_extension_value"),
        ):
            with self.subTest(ready_export=key):
                declaration = declarations[key]
                self.assertEqual(declaration["declarationState"], "declared")
                self.assertIn("languageMappings", declaration)

        exclusions = {
            (item["packageId"], item["exportName"])
            for item in self.checked_in_manifest["approvedExclusions"]
        }
        java_query_bridge_exports = {
            ("java-kmipkit", f"org.kmipkit.NativeExtensionRegistry.{name}")
            for name in (
                "clientBatchItemPing",
                "clientBatchItemQuery",
                "clientBatchItemQueryWithLimits",
                "clientBatchItemResponsePing",
                "clientBatchItemResponseQueryTtlv",
            )
        }
        self.assertTrue(java_query_bridge_exports <= exclusions)
        for key in java_query_bridge_exports:
            exclusion = next(
                item for item in self.checked_in_manifest["approvedExclusions"]
                if (item["packageId"], item["exportName"]) == key
            )
            self.assertIn("no independent Java product capability", exclusion["reason"])
            self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
            self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
        aws_lc_transitive_exports = {
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
        }
        self.assertFalse(aws_lc_transitive_exports & declarations.keys())
        self.assertFalse(aws_lc_transitive_exports & exclusions)
        self.assertFalse(
            {
                ("c-kmipkit", "KMIPKIT_H"),
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

    def test_checked_in_manifest_maps_ttlv_item_type_macros_and_blocks_unversioned_fields(self) -> None:
        declarations = {
            declaration["exportName"]: declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        declarations_by_package_and_name = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
        }
        macro_exports = [
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
        field_exports = [
            f"kmipkit_extension_registry_limits_v1_t.{name}"
            for name in (
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
            )
        ]
        self.assertEqual(len(macro_exports), len(set(macro_exports)))
        self.assertEqual(len(field_exports), len(set(field_exports)))
        item_type = next(
            item_type
            for item_type in self.checked_in_manifest["types"]
            if item_type["id"] == "TtlvItemType"
        )
        item_type_codes = {
            f"KMIPKIT_TTLV_ITEM_TYPE_{re.sub(r'(?<!^)(?=[A-Z])', '_', value['name']).upper()}": value[
                "value"
            ]
            for value in item_type["enumRepresentation"]["values"]
        }
        for export_name in macro_exports:
            with self.subTest(export_name=export_name):
                declaration = declarations[export_name]
                self.assertEqual(declaration["declarationState"], "declared")
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertIn("approvalSource", declaration)
                self.assertEqual(
                    declaration["capabilityId"],
                    "generic_ttlv_model",
                )
                self.assertEqual(
                    set(declaration["languageMappings"]),
                    {"rust", "c", "java", "python"},
                )
                self.assertEqual(
                    declaration["languageMappings"]["c"]["name"], export_name
                )
                self.assertEqual(
                    declaration["approvalSource"]["path"],
                    "specs/004-generic-ttlv-model/spec.md",
                )
                self.assertEqual(
                    set(declaration["requirementIds"]),
                    {
                        "KMIPKIT-0004-FR-001",
                        "KMIPKIT-0004-FR-008",
                        "KMIPKIT-0043-FR-004",
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    },
                )
                suffix = export_name.removeprefix("KMIPKIT_TTLV_ITEM_TYPE_")
                variant = "".join(part.title() for part in suffix.split("_"))
                self.assertEqual(item_type_codes[export_name], macro_exports.index(export_name) + 1)
                self.assertEqual(
                    declaration["languageMappings"]["rust"]["name"],
                    f"kmipkit_ttlv::ItemType::{variant}",
                )
                self.assertEqual(
                    declaration["languageMappings"]["java"]["name"],
                    f"org.kmipkit.ttlv.TtlvItemType.{variant}",
                )
                self.assertEqual(
                    declaration["languageMappings"]["python"]["name"],
                    f"kmipkit.ttlv.ItemType.{variant}",
                )
                for mapping in declaration["languageMappings"].values():
                    target = declarations_by_package_and_name[
                        (mapping["packageId"], mapping["name"])
                    ]
                    with self.subTest(target=target["declarationId"]):
                        self.assertEqual(target["declarationState"], "declared")
                        self.assertEqual(target["capabilityId"], "generic_ttlv_model")
                        self.assertEqual(
                            target["approvalSource"]["path"],
                            "specs/004-generic-ttlv-model/spec.md",
                        )
        for export_name in field_exports:
            with self.subTest(export_name=export_name):
                declaration = declarations[export_name]
                self.assertEqual(declaration["declarationState"], "declared")
                self.assertEqual(declaration["capabilityId"], "vendor_extension_registry")
                self.assertEqual(
                    declaration["approvalSource"]["path"],
                    "specs/012-vendor-extension-registry/spec.md",
                )
                self.assertEqual(
                    set(declaration["languageMappings"]),
                    {"rust", "c", "java", "python"},
                )

    def test_extension_registry_limits_preserves_legacy_and_adds_versioned_c_representations(self) -> None:
        public_header = generate.render_c_header(self.checked_in_manifest)
        private_header = generate.render_internal_c_header(self.checked_in_manifest)
        legacy_limits = next(
            row for row in self.checked_in_manifest["types"]
            if row["id"] == "ExtensionRegistryLimits"
        )
        versioned_limits = next(
            row for row in self.checked_in_manifest["types"]
            if row["id"] == "ExtensionRegistryLimitsV1"
        )
        declarations = [
            row for row in self.checked_in_manifest["publicDeclarations"]
            if row["declarationId"].startswith("ExtensionRegistryLimitsV1:")
        ]

        self.assertIn("kmipkit_extension_registry_limits_t", public_header)
        self.assertIn("kmipkit_extension_registry_limits_v1_t", public_header)
        self.assertIn("kmipkit_extension_registry_limits_v1_t", private_header)
        self.assertIn("uint32_t struct_size;", public_header)
        self.assertIn("uint32_t abi_version;", public_header)
        self.assertEqual(legacy_limits["c"]["name"], "kmipkit_extension_registry_limits_t")
        self.assertEqual(
            versioned_limits["c"],
            {
                "name": "kmipkit_extension_registry_limits_v1_t",
                "kind": "fixed-width-value",
                "visibility": "public",
                "versioned": True,
            },
        )
        self.assertTrue(declarations)
        self.assertTrue(all(row["declarationState"] == "declared" for row in declarations))
        self.assertTrue(all("languageMappings" in row for row in declarations))
        self.assertRegex(
            public_header,
            r"typedef struct kmipkit_extension_registry_limits_t "
            r"kmipkit_extension_registry_limits_t;",
        )
        legacy_layout = re.search(
            r"typedef struct \{(?P<body>.*?)\} kmipkit_extension_registry_limits_t;",
            public_header,
            re.DOTALL,
        )
        self.assertIsNone(legacy_layout, "the legacy C limits name must remain opaque")
        metadata_fields = {
            row["exportName"] for row in self.checked_in_manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit"
            and row["exportName"].startswith("kmipkit_extension_registry_limits_v1_t.")
        }
        self.assertEqual(
            metadata_fields,
            {
                "kmipkit_extension_registry_limits_v1_t.struct_size",
                "kmipkit_extension_registry_limits_v1_t.abi_version",
            },
        )

    def test_generator_rejects_ready_mapping_to_blocked_export(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        declaration = next(
            row for row in manifest["publicDeclarations"]
            if row["declarationId"] == "ClientBatchItem:c"
        )
        declaration["languageMappings"]["python"] = {
            "packageId": "python-kmipkit",
            "name": "kmipkit.extensions.with_extension",
        }

        with self.assertRaisesRegex(ManifestError, "blocked|mapping target"):
            _validate_v2_readiness(manifest, ROOT)

    def test_generator_rejects_capability_mapping_without_a_ready_export(self) -> None:
        manifest = copy.deepcopy(self.checked_in_manifest)
        capability = next(
            row for row in manifest["capabilities"]
            if row["capabilityId"] == "vendor_extension_registry"
        )
        capability["mappings"]["rust"].append("kmipkit_client::NotARealExport")

        with self.assertRaisesRegex(ManifestError, "capability.*mapping|exact.*export"):
            _validate_v2_readiness(manifest, ROOT)

    def test_capability_inventory_uses_each_language_mapping_name(self) -> None:
        manifest = self.checked_in_manifest
        package_languages = {
            package["id"]: package["language"]
            for package in manifest["publicPackages"]
        }

        generate._validate_capability_mapping_inventories(
            manifest["capabilities"],
            manifest["publicDeclarations"],
            package_languages,
        )

    def test_checked_in_manifest_inventories_all_rust_public_access_paths(self) -> None:
        rust_packages = EXPECTED_PACKAGES - {"c-kmipkit", "java-kmipkit", "python-kmipkit"}
        declarations = [
            declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration["packageId"] in rust_packages
        ]
        exclusions = [
            exclusion
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] in rust_packages
        ]
        self.assertEqual(len(declarations), 9058)
        self.assertEqual(len(exclusions), 1399)
        report = json.loads(RUST_EXPORT_REPORT_PATH.read_text(encoding="utf-8"))
        observed_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]), row["kind"])
            for row in report["declarations"]
        }
        conditional_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]), row["kind"])
            for row in report["conditionalDeclarations"]
        }
        manifest_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]), row["kind"])
            for row in declarations
        }
        approved_exclusion_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]))
            for row in exclusions
        }
        reported_exclusion_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]))
            for row in report["excluded"]
        }
        manifest_export_keys = {(package, name) for package, name, _kind in manifest_keys}
        observed_export_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]))
            for row in [*report["declarations"], *report["conditionalDeclarations"]]
        }
        not_observed_keys = {
            (row["packageId"], _canonical_export_name(row["exportName"]), row["kind"])
            for row in report["notObserved"]
        }
        self.assertEqual(len(observed_keys), len(report["declarations"]))
        self.assertEqual(len(conditional_keys), len(report["conditionalDeclarations"]))
        self.assertEqual(len(not_observed_keys), len(report["notObserved"]))
        self.assertEqual(approved_exclusion_keys, reported_exclusion_keys)
        self.assertFalse(manifest_export_keys & approved_exclusion_keys)
        self.assertEqual(
            manifest_export_keys | approved_exclusion_keys,
            observed_export_keys,
        )
        self.assertEqual(report["counts"]["missingFromManifest"], 0)
        self.assertEqual(report["counts"]["excluded"], len(exclusions))
        self.assertEqual(report["counts"]["notObserved"], 0)
        self.assertEqual(report["counts"]["kindMismatches"], 0)
        self.assertEqual(report["counts"]["issues"], 0)
        self.assertTrue(report["complete"])
        self.assertTrue(report["matchesManifest"])
        for row in report["notObserved"]:
            with self.subTest(not_observed=row["exportName"]):
                self.assertEqual(row["declarationState"], "blocked")
                self.assertNotIn("languageMappings", row)
        by_package_and_export = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in declarations
        }
        for row in report["conditionalDeclarations"]:
            if (
                row["packageId"],
                _canonical_export_name(row["exportName"]),
            ) in approved_exclusion_keys:
                continue
            with self.subTest(conditional=row["exportName"]):
                manifest_row = by_package_and_export[(row["packageId"], row["exportName"])]
                self.assertEqual(manifest_row["declarationState"], "blocked")
                self.assertEqual(manifest_row["condition"], row["condition"])
                self.assertNotIn("languageMappings", manifest_row)
        self.assertEqual(report["issues"], [])
        any_lifetime_rows = [
            row for row in report["conditionalDeclarations"]
            if "core::any::Any" in row["exportName"]
            and "<'" in row["exportName"]
        ]
        self.assertEqual(len(any_lifetime_rows), 13)
        for row in any_lifetime_rows:
            with self.subTest(any_lifetime=row["exportName"]):
                declaration = by_package_and_export[(row["packageId"], row["exportName"])]
                predicates = row["condition"]["predicates"]
                self.assertEqual(predicates[0], {
                    "kind": "outlives", "subject": "T", "lifetime": "'static",
                })
                self.assertEqual(predicates[1], {
                    "kind": "traitBound", "subject": "T",
                    "traitPath": ["core", "marker", "Sized"], "arguments": [],
                    "modifier": "maybe",
                })
                self.assertEqual(declaration["declarationState"], "blocked")
                self.assertNotIn("languageMappings", declaration)
                self.assertNotIn("approvalSource", declaration)
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

        declarations_by_c_symbol: dict[str, list[dict[str, object]]] = {}
        for declaration in self.checked_in_manifest["publicDeclarations"]:
            symbol = declaration.get("languageMappings", {}).get("c", {}).get("name")
            if symbol:
                declarations_by_c_symbol.setdefault(symbol, []).append(declaration)
        c_declarations_by_symbol = {
            declaration["exportName"]: declaration
            for declaration in self.checked_in_manifest["publicDeclarations"]
            if declaration["packageId"] == "c-kmipkit"
        }
        self.assertTrue(expected_symbols <= set(declarations_by_c_symbol))
        for symbol in expected_symbols:
            declaration = c_declarations_by_symbol[symbol]
            self.assertEqual(declaration["capabilityId"], "ffi_runtime_metadata")
            self.assertEqual(declaration["declarationState"], "declared")

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

        c_exclusions = {
            item["exportName"]: item
            for item in self.checked_in_manifest["approvedExclusions"]
            if item["packageId"] == "c-kmipkit"
        }
        runtime_metadata_exclusions = {
            "kmipkit_last_error_clear",
            "kmipkit_extension_registry_limits_v1_t.struct_size",
            "kmipkit_extension_registry_limits_v1_t.abi_version",
        }
        self.assertTrue(runtime_metadata_exclusions <= set(c_exclusions))
        for symbol in c_exclusions:
            self.assertTrue(c_exclusions[symbol]["reason"])
            self.assertTrue(c_exclusions[symbol]["evidence"])
            if symbol == "kmipkit_client_batch_item_response_release":
                lifecycle_mappings = declarations_by_c_symbol[symbol]
                self.assertEqual(
                    {item["declarationId"] for item in lifecycle_mappings},
                    {
                        "T040:encrypt:response-close:java",
                        "T040:encrypt:response-close:python",
                        "T040:decrypt:response-close:java",
                        "T040:decrypt:response-close:python",
                    },
                )
                self.assertEqual(
                    {item["capabilityId"] for item in lifecycle_mappings},
                    {
                        "KMIPKIT-ELEM-OP-C2S-ENCRYPT",
                        "KMIPKIT-ELEM-OP-C2S-DECRYPT",
                    },
                )
                self.assertIn("lifecycle counterpart", c_exclusions[symbol]["reason"])
                self.assertIn("distinct four-language capability", c_exclusions[symbol]["reason"])
            else:
                self.assertNotIn(symbol, declarations_by_c_symbol)

        rust_exclusions = [
            item
            for item in self.checked_in_manifest["approvedExclusions"]
            if item["packageId"] == "rust-kmipkit-protocol"
        ]
        t087_exclusions = [
            item for item in rust_exclusions
            if item["approvalSource"]["path"] == "specs/003-core-types-errors/spec.md"
        ]
        self.assertEqual(len(t087_exclusions), 217)
        self.assertEqual(len(rust_exclusions), 202 + len(t087_exclusions))
        self.assertTrue(
            any(
                item["exportName"] == "kmipkit_protocol::RegisterRequest::validate_local_shape"
                for item in rust_exclusions
            )
        )
        self.assertEqual(
            {
                item["approvalSource"]["path"]
                for item in rust_exclusions
            },
            {
                "docs/adr/0003-layered-cargo-workspace.md",
                "specs/029-opaque-object-context/spec.md",
                "specs/010-profile-conformance/spec.md",
                "specs/003-core-types-errors/spec.md",
                "specs/006-message-batch-model/spec.md",
                "specs/007-client-execution/spec.md",
                "specs/043-public-api-bindings/spec.md",
            },
        )

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
            if declaration["capabilityId"] not in {
                "generic_ttlv_model",
                "ttlv_wire_codec",
            }:
                continue
            rust_mapping = declaration.get("languageMappings", {}).get("rust", {})
            rust_name = rust_mapping.get("name", "")
            if not rust_name.startswith("kmipkit_ttlv::"):
                continue
            expected_capability = (
                "ttlv_wire_codec"
                if rust_name.startswith("kmipkit_ttlv::codec::")
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
                if declaration.get("mappingSemantics") == "native-handle-release":
                    binding_requirement = {
                        "c-kmipkit": "KMIPKIT-0043-FR-007",
                        "java-kmipkit": "KMIPKIT-0043-FR-008",
                        "python-kmipkit": "KMIPKIT-0043-FR-009",
                    }[declaration["packageId"]]
                    self.assertTrue(
                        "KMIPKIT-0043-FR-004" in declaration["requirementIds"]
                        or (
                            binding_requirement in declaration["requirementIds"]
                            and "KMIPKIT-0043-FR-010" in declaration["requirementIds"]
                        )
                    )
                else:
                    self.assertIn("KMIPKIT-0043-FR-004", declaration["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                self.assertEqual(
                    declaration["approvalSource"]["path"],
                    "specs/004-generic-ttlv-model/spec.md"
                    if expected_capability == "generic_ttlv_model"
                    else "specs/005-ttlv-wire-codec/spec.md",
                )
        self.assertTrue(ttlv_declarations)
        self.assertEqual(len(ttlv_declarations), 372)
        self.assertEqual(
            sum(
                declaration["capabilityId"] == "generic_ttlv_model"
                for declaration in ttlv_declarations
            ),
            255,
        )
        self.assertEqual(
            sum(
                declaration["capabilityId"] == "ttlv_wire_codec"
                for declaration in ttlv_declarations
            ),
            117,
        )

    def test_codec_limits_release_maps_managed_java_value_lifecycle(self) -> None:
        declarations = self.checked_in_manifest["publicDeclarations"]
        by_package_and_export = {
            (declaration["packageId"], declaration["exportName"]): declaration
            for declaration in declarations
        }
        release = by_package_and_export[("c-kmipkit", "kmipkit_codec_limits_release")]
        python_close = by_package_and_export[
            ("python-kmipkit", "kmipkit.ttlv.CodecLimits.close")
        ]
        expected_mappings = {
            "rust": ("rust-kmipkit-ttlv", "kmipkit_ttlv::codec::CodecLimits"),
            "c": ("c-kmipkit", "kmipkit_codec_limits_release"),
            "java": ("java-kmipkit", "org.kmipkit.ttlv.CodecLimits"),
            "python": ("python-kmipkit", "kmipkit.ttlv.CodecLimits.close"),
        }

        for declaration in (release, python_close):
            with self.subTest(declaration=declaration["declarationId"]):
                self.assertEqual(declaration["declarationState"], "declared")
                self.assertEqual(
                    declaration["mappingSemantics"],
                    "native-handle-release-managed-java-value",
                )
                self.assertEqual(
                    {
                        language: (mapping["packageId"], mapping["name"])
                        for language, mapping in declaration["languageMappings"].items()
                    },
                    expected_mappings,
                )

        java_close = by_package_and_export.get(
            ("java-kmipkit", "org.kmipkit.ttlv.CodecLimits.close")
        )
        self.assertIsNone(java_close)

    def test_checked_in_manifest_inventories_all_java_public_access_paths(self) -> None:
        declarations = self.declarations_for_package("java-kmipkit")
        self.assertEqual(len(declarations), 953)
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

    def test_crypto_response_java_types_and_accessors_are_inventoried(self) -> None:
        by_name = {
            declaration["exportName"]: declaration
            for declaration in self.declarations_for_package("java-kmipkit")
        }
        expected = {
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse": "type",
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse.uniqueIdentifier": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse.data": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse.ivCounterNonce": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse.correlationValue": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.EncryptResponse.authenticatedEncryptionTag": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.DecryptResponse": "type",
            "org.kmipkit.extensions.ClientBatchItemResult.DecryptResponse.uniqueIdentifier": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.DecryptResponse.data": "java-member",
            "org.kmipkit.extensions.ClientBatchItemResult.DecryptResponse.correlationValue": "java-member",
        }

        for export_name, expected_kind in expected.items():
            with self.subTest(export_name=export_name):
                declaration = by_name[export_name]
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(expected_kind, declaration["kind"])

    def test_only_unmapped_internal_java_close_remains_blocked(self) -> None:
        declarations = self.declarations_for_package("java-kmipkit")
        close_declarations = [
            declaration
            for declaration in declarations
            if declaration["exportName"].endswith(".close")
            and declaration["declarationState"] == "blocked"
        ]

        self.assertEqual(
            {declaration["exportName"] for declaration in close_declarations},
            {"org.kmipkit.internal.NativeHandle.close"},
        )
        generic_ttlv_closes = [
            declaration
            for declaration in declarations
            if declaration["exportName"].endswith(".close")
            and declaration["capabilityId"] == "generic_ttlv_model"
        ]
        self.assertEqual(
            {declaration["exportName"] for declaration in generic_ttlv_closes},
            {
                "org.kmipkit.ttlv.Tag.close",
                "org.kmipkit.ttlv.RawTag.close",
                "org.kmipkit.ttlv.TtlvItem.close",
                "org.kmipkit.ttlv.TtlvItemView.close",
                "org.kmipkit.ttlv.TtlvStructure.close",
                "org.kmipkit.ttlv.TtlvStructureView.close",
                "org.kmipkit.ttlv.TtlvValue.close",
                "org.kmipkit.ttlv.TtlvValueView.close",
            },
        )
        self.assertTrue(
            all(declaration["declarationState"] == "declared" for declaration in generic_ttlv_closes)
        )

    def test_checked_in_manifest_accounts_for_every_ready_c_header_declaration(self) -> None:
        header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(
            encoding="utf-8"
        )
        declarations = self.declarations_for_package("c-kmipkit")
        ready_by_kind = {
            kind: {
                item["exportName"]
                for item in declarations
                if item["kind"] == kind
                and item["declarationState"] in {"declared", "verified"}
            }
            for kind in {item["kind"] for item in declarations}
        }
        approved_c_layouts = {
            item["cType"]["name"]
            for item in self.checked_in_manifest["approvedExclusions"]
            if item["packageId"] == "c-kmipkit"
            and item.get("cType", {}).get("kind") == "fixed-width-value"
        }
        ready_by_kind["type"].update(approved_c_layouts)

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
            and "cType" not in exclusion
            and re.fullmatch(r"kmipkit_[A-Za-z0-9_]+", exclusion["exportName"])
        }
        self.assertIn("kmipkit_client_batch_item_response_release", header_functions)
        self.assertIn(
            "kmipkit_client_batch_item_response_release", excluded_c_functions
        )
        published_header_symbols = {
            name for name in ready_by_kind["c-symbol"] if name.startswith("kmipkit_")
        }
        self.assertEqual(
            header_functions,
            published_header_symbols | excluded_c_functions,
        )

        header_types = set(
            re.findall(r"^typedef struct \w+\s+(kmipkit_\w+);$", header, re.MULTILINE)
        )
        header_types.update(
            re.findall(
                r"^typedef (?:u?int(?:8|16|32|64)_t) (kmipkit_\w+);$",
                header,
                re.MULTILINE,
            )
        )
        header_types.update(
            re.findall(r"^} (kmipkit_\w+);$", header, re.MULTILINE)
        )
        self.assertEqual(header_types, ready_by_kind["type"])

        header_fields = {
            f"{match['named_type'] or match['typedef_type']}.{field_name}"
            for match in re.finditer(
                r"(?:typedef\s+)?struct(?:\s+(?P<named_type>kmipkit_\w+))?\s*"
                r"\{(?P<body>.*?)\}\s*(?P<typedef_type>kmipkit_\w+)?\s*;",
                header,
                re.DOTALL,
            )
            for field_name in re.findall(
                r"^\s*(?:const\s+)?[A-Za-z_]\w*(?:\s*\*)*\s+"
                r"([A-Za-z_]\w*)\s*(?:\[[^\]]*\])?\s*;",
                match["body"],
                re.MULTILINE,
            )
        }
        approved_c_fields = {
            item["exportName"]
            for item in self.checked_in_manifest["approvedExclusions"]
            if item["packageId"] == "c-kmipkit"
            and re.fullmatch(r"kmipkit_\w+\.\w+", item["exportName"])
        }
        self.assertEqual(header_fields, ready_by_kind["field"] | approved_c_fields)

        header_ttlv_macros = set(
            re.findall(
                r"^#define (KMIPKIT_(?:TTLV_ITEM_TYPE|ATTRIBUTE_REFERENCE_KIND|OPAQUE_OBJECT_CONTEXT|UNIQUE_IDENTIFIER_KIND)_\w+)",
                header,
                re.MULTILINE,
            )
        )
        self.assertEqual(header_ttlv_macros, ready_by_kind["macro"])
        header_error_macros = set(
            re.findall(r"^#define (KMIPKIT_ERROR_\w+)", header, re.MULTILINE)
        )
        manifest_error_macros = {
            item["exportName"]
            for item in declarations
            if item["kind"] == "constant"
            and item["declarationState"] in {"declared", "verified"}
            and item["exportName"].startswith("KMIPKIT_ERROR_")
        }
        manifest_error_macros.update(
            item["cCode"]
            for item in self.checked_in_manifest["errorCategories"]
            if item.get("stableCHeaderCode", False)
        )
        self.assertEqual(header_error_macros, manifest_error_macros)

    def test_c_header_only_declares_ready_manifest_entries(self) -> None:
        header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(
            encoding="utf-8"
        )
        declarations = self.declarations_for_package("c-kmipkit")
        approved_exclusions = {
            exclusion["exportName"]
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] == "c-kmipkit"
            and "cType" not in exclusion
            and re.fullmatch(r"kmipkit_[A-Za-z0-9_]+", exclusion["exportName"])
        }
        ready_functions = {
            declaration["exportName"]
            for declaration in declarations
            if declaration["kind"] == "c-symbol"
            and declaration["declarationState"] in {"declared", "verified"}
        } | approved_exclusions
        ready_types = {
            declaration["exportName"]
            for declaration in declarations
            if declaration["kind"] == "type"
            and declaration["declarationState"] in {"declared", "verified"}
        }
        ready_types.update(
            exclusion["cType"]["name"]
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] == "c-kmipkit"
            and exclusion.get("cType", {}).get("kind") == "fixed-width-value"
        )
        header_functions = set(
            re.findall(
                r"^(?:void|int32_t)\s+(kmipkit_[A-Za-z0-9_]+)\s*\(",
                header,
                re.MULTILINE,
            )
        )
        header_types = set(
            re.findall(r"^typedef struct \w+\s+(kmipkit_\w+);$", header, re.MULTILINE)
        )
        header_types.update(
            re.findall(
                r"^typedef (?:u?int(?:8|16|32|64)_t) (kmipkit_\w+);$",
                header,
                re.MULTILINE,
            )
        )
        header_types.update(
            re.findall(r"^typedef struct \{.*?\} (kmipkit_\w+);$", header, re.MULTILINE | re.DOTALL)
        )

        self.assertEqual(header_functions, ready_functions)
        self.assertEqual(header_types, ready_types)

    def test_opaque_register_binding_support_exports_are_inventoried(self) -> None:
        declarations = self.declarations_for_package("c-kmipkit")
        declarations += self.declarations_for_package("java-kmipkit")
        declarations += self.declarations_for_package("python-kmipkit")
        by_name = {row["exportName"]: row for row in declarations}

        blocked_paths = {
            "org.kmipkit.NativeExtensionRegistry.REGISTER_REQUEST": "java-member",
            "org.kmipkit.NativeExtensionRegistry.registerRequestCreate": "java-member",
            "org.kmipkit.NativeExtensionRegistry.registerRequestWithOpaqueObjectContext": "java-member",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemRegister": "java-member",
            "org.kmipkit.extensions.OpaqueObjectContext.valueOf": "java-member",
            "org.kmipkit.extensions.OpaqueObjectContext.values": "java-member",
            "org.kmipkit.ttlv.TtlvItem.nativeHandleOwner": "java-member",
            "org.kmipkit.ttlv.TtlvStructure.nativeHandleOwner": "java-member",
            "kmipkit.extensions.RegisterRequest.closed": "python-export",
            "kmipkit.extensions.RegisterRequest.__enter__": "python-export",
            "kmipkit.extensions.RegisterRequest.__exit__": "python-export",
        }
        for export_name, kind in blocked_paths.items():
            with self.subTest(export_name=export_name):
                declaration = by_name[export_name]
                self.assertEqual(declaration["kind"], kind)
                self.assertEqual(declaration["declarationState"], "blocked")

        lifecycle_exports = (
            ("kmipkit_register_request_release", "c-symbol"),
            ("org.kmipkit.extensions.RegisterRequest.close", "java-member"),
            ("kmipkit.extensions.RegisterRequest.close", "python-export"),
        )
        for export_name, kind in lifecycle_exports:
            with self.subTest(export_name=export_name):
                declaration = by_name[export_name]
                self.assertEqual(declaration["kind"], kind)
                self.assertEqual(declaration["declarationState"], "verified")
                self.assertEqual(
                    declaration["mappingSemantics"], "native-handle-release"
                )
                self.assertEqual(
                    declaration["languageMappings"]["rust"]["name"],
                    "kmipkit_protocol::RegisterRequest",
                )

        for export_name in (
            "KMIPKIT_OPAQUE_OBJECT_CONTEXT_GENERIC",
            "KMIPKIT_OPAQUE_OBJECT_CONTEXT_MANAGED_CRYPTOGRAPHIC_OBJECT",
        ):
            with self.subTest(export_name=export_name):
                self.assertEqual(by_name[export_name]["kind"], "macro")

        create = next(
            row for row in self.checked_in_manifest["functions"]
            if row["id"] == "register_request_create"
        )
        self.assertEqual(
            create["python"]["module"] + "." + create["python"]["function"],
            "kmipkit.extensions.RegisterRequest.create",
        )
        selector = next(
            row for row in self.checked_in_manifest["functions"]
            if row["id"] == "register_request_with_opaque_object_context"
        )
        self.assertEqual(
            selector["python"]["module"] + "." + selector["python"]["function"],
            "kmipkit.extensions.RegisterRequest.with_opaque_object_context",
        )

    def test_declared_c_function_handle_types_are_ready(self) -> None:
        declarations = self.declarations_for_package("c-kmipkit")
        states = {
            (declaration["kind"], declaration["exportName"]): declaration[
                "declarationState"
            ]
            for declaration in declarations
        }
        ready_types = {
            name
            for (kind, name), state in states.items()
            if kind == "type" and state in {"declared", "verified"}
        }
        ready_types.update(
            exclusion["cType"]["name"]
            for exclusion in self.checked_in_manifest["approvedExclusions"]
            if exclusion["packageId"] == "c-kmipkit"
            and exclusion.get("cType", {}).get("kind") == "fixed-width-value"
        )

        for function in self.checked_in_manifest["functions"]:
            if states.get(("c-symbol", function["c"]["symbol"])) not in {
                "declared",
                "verified",
            }:
                continue
            parameter_types = [
                parameter["type"] for parameter in function["c"]["parameters"]
            ]
            signature_types = [*parameter_types, function["c"]["returnType"]]
            for signature_type in signature_types:
                name = signature_type.strip()
                while name.endswith("*"):
                    name = name[:-1].strip()
                if name.startswith("const "):
                    name = name[6:]
                if name.startswith("kmipkit_"):
                    with self.subTest(symbol=function["c"]["symbol"], type=name):
                        self.assertIn(name, ready_types)

    def test_checked_in_manifest_inventories_python_exports_and_lifecycle(self) -> None:
        declarations = self.declarations_for_package("python-kmipkit")
        self.assertEqual(len(declarations), 921)
        by_name = {declaration["exportName"]: declaration for declaration in declarations}
        close_method = by_name["kmipkit.extensions.ClientBatchItem.close"]
        self.assertEqual(close_method["kind"], "python-export")
        self.assertEqual(close_method["declarationState"], "declared")
        item_type = by_name["kmipkit.ttlv.ItemType"]
        self.assertEqual(item_type["declarationState"], "declared")
        enum_value = by_name["kmipkit.ttlv.ItemType.Structure"]
        self.assertEqual(enum_value["declarationState"], "declared")
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

    def test_approved_c_exclusion_can_describe_an_abi_only_carrier_layout(self) -> None:
        manifest = _valid_v2_manifest()
        manifest["approvedExclusions"] = [
            {
                "packageId": "c-kmipkit",
                "exportName": "kmipkit_example_v1_t",
                "reason": "This fixed ABI carrier has no four-language facade value.",
                "approvalSource": copy.deepcopy(
                    self.checked_in_manifest["approvedExclusions"][0]["approvalSource"]
                ),
                "evidence": "The versioned C ABI uses this carrier at a native boundary.",
                "cType": {
                    "name": "kmipkit_example_v1_t",
                    "kind": "fixed-width-value",
                    "fields": [
                        {"name": "struct_size", "type": "uint32_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                        {"name": "data", "type": "const uint8_t *"},
                        {"name": "length", "type": "uint64_t"},
                    ],
                },
            }
        ]

        self.assertEqual(len(list(self.validator.iter_errors(manifest))), 0)

        manifest["approvedExclusions"][0]["cType"]["fields"][0]["name"] = "class"
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)
        manifest["approvedExclusions"][0]["cType"]["fields"][0]["name"] = "field__internal"
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)

        manifest["approvedExclusions"][0]["cType"]["fields"][0]["name"] = "struct_size"
        manifest["approvedExclusions"][0]["cType"]["fields"][0]["type"] = (
            "uint32_t; injected"
        )
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)

        manifest["approvedExclusions"][0]["cType"]["fields"][0]["type"] = "uint32_t"
        manifest["approvedExclusions"][0]["packageId"] = "java-kmipkit"
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)

    def test_declared_c_value_fields_reject_reserved_c_and_cpp_names(self) -> None:
        manifest = _valid_v2_manifest()
        manifest["types"].append(
            {
                "id": "ExampleCValue",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-001"],
                "rust": {
                    "module": "kmipkit_ffi::example",
                    "name": "ExampleCValue",
                    "owner": "ffi",
                },
                "c": {
                    "name": "kmipkit_example_value_t",
                    "kind": "fixed-width-value",
                    "fields": [{"name": "valid_name", "type": "uint32_t"}],
                },
                "java": {"package": "org.kmipkit", "name": "ExampleCValue"},
                "python": {"module": "kmipkit", "name": "ExampleCValue"},
            }
        )

        self.assertEqual(len(list(self.validator.iter_errors(manifest))), 0)
        manifest["types"][-1]["c"]["fields"][0]["name"] = "class"
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)
        manifest["types"][-1]["c"]["fields"][0]["name"] = "field__internal"
        self.assertGreater(len(list(self.validator.iter_errors(manifest))), 0)

    def test_c_header_renders_approved_exclusion_carrier_layout(self) -> None:
        def exclusion(name: str, fields: list[dict[str, str]]) -> dict[str, object]:
            return {
                "packageId": "c-kmipkit",
                "exportName": name,
                "reason": "This fixed ABI carrier has no four-language facade value.",
                "approvalSource": {"kind": "feature-specification", "path": "unused"},
                "evidence": "The versioned C ABI uses this carrier at a native boundary.",
                "cType": {"name": name, "kind": "fixed-width-value", "fields": fields},
            }

        manifest = {
            "formatVersion": 2,
            "approvedExclusions": [
                exclusion(
                    "kmipkit_transport_config_v1_t",
                    [
                        {"name": "struct_size", "type": "uint32_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                        {"name": "client_certificates", "type": "const kmipkit_transport_input_v1_t *"},
                        {"name": "client_private_key", "type": "kmipkit_transport_input_v1_t"},
                        {"name": "connect_timeout", "type": "kmipkit_timeout_limit_v1_t"},
                    ],
                ),
                exclusion(
                    "kmipkit_transport_input_v1_t",
                    [
                        {"name": "source_kind", "type": "uint32_t"},
                        {"name": "encoding", "type": "uint32_t"},
                        {"name": "data", "type": "kmipkit_byte_span_v1_t"},
                    ],
                ),
                exclusion(
                    "kmipkit_timeout_limit_v1_t",
                    [
                        {"name": "kind", "type": "uint32_t"},
                        {"name": "reserved", "type": "uint32_t"},
                        {"name": "seconds", "type": "uint64_t"},
                    ],
                ),
                exclusion(
                    "kmipkit_byte_span_v1_t",
                    [
                        {"name": "data", "type": "const uint8_t *"},
                        {"name": "length", "type": "uint64_t"},
                    ],
                ),
            ],
            "publicDeclarations": [],
            "types": [],
            "limits": [],
            "errorCategories": [],
            "functions": [],
        }

        header = generate.render_c_header(manifest)

        for name in (
            "kmipkit_transport_config_v1_t",
            "kmipkit_transport_input_v1_t",
            "kmipkit_timeout_limit_v1_t",
            "kmipkit_byte_span_v1_t",
        ):
            self.assertIn(f"typedef struct {name} {name};", header)
        self.assertLess(
            header.index("struct kmipkit_byte_span_v1_t {"),
            header.index("struct kmipkit_transport_input_v1_t {"),
        )
        self.assertLess(
            header.index("struct kmipkit_transport_input_v1_t {"),
            header.index("struct kmipkit_transport_config_v1_t {"),
        )
        self.assertLess(
            header.index("struct kmipkit_timeout_limit_v1_t {"),
            header.index("struct kmipkit_transport_config_v1_t {"),
        )
        self.assertIn(
            "    const kmipkit_transport_input_v1_t * client_certificates;", header
        )
        self.assertIn(
            "    kmipkit_transport_input_v1_t client_private_key;", header
        )

    def test_approved_c_carrier_layout_references_must_resolve_without_cycles(self) -> None:
        def exclusion(name: str, field_type: str) -> dict[str, object]:
            return {
                "packageId": "c-kmipkit",
                "exportName": name,
                "reason": "This fixed ABI carrier has no four-language facade value.",
                "approvalSource": {"kind": "feature-specification", "path": "unused"},
                "evidence": "The versioned C ABI uses this carrier at a native boundary.",
                "cType": {
                    "name": name,
                    "kind": "fixed-width-value",
                    "fields": [{"name": "value", "type": field_type}],
                },
            }

        manifest = {
            "approvedExclusions": [
                exclusion("kmipkit_outer_v1_t", "kmipkit_inner_v1_t"),
                exclusion("kmipkit_inner_v1_t", "uint32_t"),
            ],
            "types": [],
        }
        ordered = generate._ordered_approved_c_type_layouts(manifest)
        self.assertEqual(
            [record["name"] for record in ordered],
            ["kmipkit_inner_v1_t", "kmipkit_outer_v1_t"],
        )

        manifest["approvedExclusions"][1]["cType"]["fields"][0]["type"] = (
            "kmipkit_missing_v1_t"
        )
        with self.assertRaises(generate.ManifestError):
            generate._ordered_approved_c_type_layouts(manifest)
        manifest["approvedExclusions"][1]["cType"]["fields"][0]["type"] = (
            "kmipkit_outer_v1_t"
        )
        with self.assertRaises(generate.ManifestError):
            generate._ordered_approved_c_type_layouts(manifest)

    def test_approved_c_carrier_rejects_reserved_c_and_cpp_field_names(self) -> None:
        manifest = {
            "approvedExclusions": [
                {
                    "packageId": "c-kmipkit",
                    "exportName": "kmipkit_reserved_field_v1_t",
                    "reason": "Native ABI carrier without a four-language value.",
                    "approvalSource": {"kind": "feature-specification", "path": "unused"},
                    "evidence": "The native ABI declares this fixed-width record.",
                    "cType": {
                        "name": "kmipkit_reserved_field_v1_t",
                        "kind": "fixed-width-value",
                        "fields": [{"name": "class", "type": "uint32_t"}],
                    },
                }
            ],
            "types": [],
        }

        with self.assertRaisesRegex(generate.ManifestError, "reserved fields"):
            generate._ordered_approved_c_type_layouts(manifest)
        manifest["approvedExclusions"][0]["cType"]["fields"][0]["name"] = (
            "field__internal"
        )
        with self.assertRaisesRegex(generate.ManifestError, "reserved fields"):
            generate._ordered_approved_c_type_layouts(manifest)

    def test_approved_c_carrier_rejects_partially_declared_value_dependency(self) -> None:
        manifest = {
            "approvedExclusions": [
                {
                    "packageId": "c-kmipkit",
                    "exportName": "kmipkit_outer_v1_t",
                    "reason": "Native ABI carrier without a four-language value.",
                    "approvalSource": {"kind": "feature-specification", "path": "unused"},
                    "evidence": "The native ABI declares this fixed-width record.",
                    "cType": {
                        "name": "kmipkit_outer_v1_t",
                        "kind": "fixed-width-value",
                        "fields": [{"name": "inner", "type": "kmipkit_partial_v1_t"}],
                    },
                }
            ],
            "types": [
                {
                    "c": {
                        "name": "kmipkit_partial_v1_t",
                        "kind": "fixed-width-value",
                    }
                }
            ],
            "limits": [{"cField": "first"}, {"cField": "second"}],
            "publicDeclarations": [
                {
                    "packageId": "c-kmipkit",
                    "kind": "field",
                    "exportName": "kmipkit_partial_v1_t.first",
                    "declarationState": "declared",
                }
            ],
        }

        with self.assertRaisesRegex(generate.ManifestError, "incomplete declared value"):
            generate._ordered_approved_c_type_layouts(manifest)

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

    def test_t091_client_batch_outcome_exports_map_to_existing_response_surfaces(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }
        operations = {
            row["operationId"]: row for row in self.checked_in_manifest["operations"]
        }
        expected = {}

        def add_case(suffix: str, canonical_name: str, package_id: str) -> None:
            prefix = {
                "rust-kmipkit-client": "kmipkit_client",
                "rust-kmipkit": "kmipkit",
            }[package_id]
            export_name = f"{prefix}::{suffix}"
            canonical = declarations[canonical_name]
            mappings = {
                language: mapping.copy()
                for language, mapping in canonical["languageMappings"].items()
            }
            mappings["rust"] = {"packageId": package_id, "name": export_name}
            owner_requirements = {
                "KMIPKIT-ELEM-OP-C2S-REGISTER": [
                    f"KMIPKIT-0036-FR-{number:03d}" for number in range(1, 9)
                ],
                "KMIPKIT-ELEM-OP-C2S-REVOKE": [
                    *(f"KMIPKIT-0036-FR-{number:03d}" for number in range(1, 7)),
                    "KMIPKIT-0036-FR-008",
                ],
            }.get(canonical["capabilityId"], [])
            requirements = sorted(set(canonical["requirementIds"]) | set(owner_requirements))
            expected[export_name] = (canonical, mappings, requirements)

        # The Rust enum is represented by the common four-language result handle.
        for package_id in ("rust-kmipkit-client", "rust-kmipkit"):
            prefix = {
                "rust-kmipkit-client": "kmipkit_client",
                "rust-kmipkit": "kmipkit",
            }[package_id]
            add_case(
                "ClientBatchOutcome",
                "org.kmipkit.extensions.ClientBatchItemResult",
                package_id,
            )
            add_case(
                "ClientBatchOutcome::Completed",
                f"{prefix}::ClientBatchOutcome::Pending",
                package_id,
            )
            add_case(
                "ClientBatchOutcome::Completed::0",
                f"{prefix}::ClientBatchOutcome::discover_versions_response",
                package_id,
            )

        # Each operation variant maps through its existing result accessor; its tuple
        # field maps through the existing typed response declaration where one exists.
        operation_responses = (
            ("Certify", "certifyResponse"),
            ("Check", "checkResponse"),
            ("DelegatedLogin", "delegatedLoginResponse"),
            ("Login", "loginResponse"),
            ("Logout", "logoutResponse"),
            ("ObtainLease", "obtainLeaseResponse"),
            ("ReProvision", "reprovisionResponse"),
            ("Recertify", "recertifyResponse"),
            ("Register", "registerResponse"),
            ("Rekey", "rekeyResponse"),
            ("RekeyKeyPair", "rekeyKeyPairResponse"),
            ("Revoke", "revokeResponse"),
            ("RngRetrieve", "rngRetrieveResponse"),
            ("RngSeed", "rngSeedResponse"),
            ("MacVerify", "macVerifyResponse"),
            ("SignatureVerify", "signatureVerifyResponse"),
        )
        for package_id in ("rust-kmipkit-client", "rust-kmipkit"):
            prefix = {
                "rust-kmipkit-client": "kmipkit_client",
                "rust-kmipkit": "kmipkit",
            }[package_id]
            for variant, method in operation_responses:
                method_name = f"org.kmipkit.extensions.ClientBatchItemResult.{method}"
                add_case(f"ClientBatchOutcome::{variant}", method_name, package_id)
                field_name = f"org.kmipkit.extensions.ClientBatchItemResult.{variant}Response"
                if variant in ("Logout",):
                    field_name = method_name
                add_case(f"ClientBatchOutcome::{variant}::0", field_name, package_id)

        # Five additional payload types have no corresponding outcome accessor.
        for package_id in ("rust-kmipkit-client", "rust-kmipkit"):
            for variant in (
                "GetAttributeList",
                "GetConstraints",
                "Pkcs11",
                "SetConstraints",
                "SetDefaults",
            ):
                add_case(
                    f"ClientBatchOutcome::{variant}::0",
                    f"org.kmipkit.extensions.ClientBatchItemResult.{variant}Response",
                    package_id,
                )

        # These four typed accessors return success data; their foreign accessor maps
        # retain the existing typed/TTLV ownership contract and operation status.
        for package_id in ("rust-kmipkit-client", "rust-kmipkit"):
            for method in ("export", "get", "import", "locate"):
                java_method = method + "Response"
                add_case(
                    f"ClientBatchOutcome::{method}_response",
                    f"org.kmipkit.extensions.ClientBatchItemResult.{java_method}",
                    package_id,
                )

        self.assertEqual(88, len(expected))
        candidates = {
            row["exportName"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
            if row["exportName"] in expected
        }
        self.assertEqual(set(expected), set(candidates))
        self.assertFalse(
            [
                row["exportName"]
                for row in self.checked_in_manifest["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0007"
                and "ClientBatchOutcome" in row["exportName"]
            ]
        )
        capabilities = {
            row["capabilityId"]: row for row in self.checked_in_manifest["capabilities"]
        }
        declarations_by_id = {
            row["declarationId"]: row
            for row in self.checked_in_manifest["publicDeclarations"]
        }

        for export_name, (canonical, mappings, requirement_ids) in expected.items():
            declaration = candidates[export_name]
            capability_id = canonical["capabilityId"]
            if "::ClientBatchOutcome::Register" in export_name:
                self.assertLessEqual(
                    {
                        *(f"KMIPKIT-0036-FR-{number:03d}" for number in range(1, 9))
                    },
                    set(canonical["requirementIds"]),
                )
            elif "::ClientBatchOutcome::Revoke" in export_name:
                self.assertLessEqual(
                    {
                        *(f"KMIPKIT-0036-FR-{number:03d}" for number in range(1, 7)),
                        "KMIPKIT-0036-FR-008",
                    },
                    set(canonical["requirementIds"]),
                )
            if capability_id in operations:
                owner = operations[capability_id]
                self.assertEqual("approved", owner["ownerDependency"]["state"])
                self.assertEqual(owner["apiState"], canonical["declarationState"])
                self.assertEqual(
                    owner["approvalEvidence"]["path"], canonical["approvalSource"]["path"]
                )
                self.assertEqual(
                    owner["approvalEvidence"]["status"], canonical["approvalSource"]["status"]
                )

            with self.subTest(export=export_name):
                self.assertEqual(canonical["declarationState"], declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertEqual(capability_id, declaration["capabilityId"])
                self.assertEqual(requirement_ids, declaration["requirementIds"])
                self.assertEqual(canonical["approvalSource"], declaration["approvalSource"])
                self.assertEqual(mappings, declaration["languageMappings"])
                if capability_id in operations:
                    owner_declarations = operations[capability_id]["declarations"]
                    self.assertIn(
                        declaration["declarationId"],
                        owner_declarations["rust"],
                    )
                    for language in ("c", "java", "python"):
                        mapped_exports = {
                            declarations_by_id[item]["languageMappings"][language]["name"]
                            for item in owner_declarations[language]
                        }
                        self.assertIn(mappings[language]["name"], mapped_exports)
                else:
                    for language, mapping in mappings.items():
                        self.assertIn(
                            mapping["name"], capabilities[capability_id]["mappings"][language]
                        )
                if "parityEvidenceIds" in canonical:
                    self.assertEqual(
                        canonical["parityEvidenceIds"], declaration["parityEvidenceIds"]
                    )
                else:
                    self.assertNotIn("parityEvidenceIds", declaration)

        # Completed is the common discriminator plus Discover Versions' typed list;
        # Logout's response is only its presence plus shared result metadata.
        client_execution = next(
            row
            for row in self.checked_in_manifest["capabilities"]
            if row["capabilityId"] == "client_execution"
        )
        for language, target in (
            ("c", "kmipkit_client_batch_item_response_discover_versions_supported_versions"),
            ("java", "org.kmipkit.extensions.ClientBatchItemResult.supportedVersions"),
            ("python", "kmipkit.client.BatchItemResult.supported_versions"),
        ):
            self.assertIn(target, client_execution["mappings"][language])
        self.assertNotIn(
            "org.kmipkit.extensions.ClientBatchItemResult.LogoutResponse",
            declarations,
        )


if __name__ == "__main__":
    unittest.main()
