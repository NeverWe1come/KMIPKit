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
                    actual = [row for row in manifest[field] if row["id"] in registry_ids]
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

    def test_checked_in_internal_error_status_is_additive_and_uses_generic_facades(self) -> None:
        categories = {
            item["id"]: item for item in self.checked_in_manifest["errorCategories"]
        }
        self.assertIn("internal", categories)
        internal = categories["internal"]
        self.assertEqual(internal["cCode"], "KMIPKIT_ERROR_INTERNAL")
        self.assertEqual(internal["cValue"], 7)
        self.assertEqual(internal["javaException"], "org.kmipkit.KmipException")
        self.assertEqual(internal["pythonException"], "kmipkit.errors.KmipKitError")

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
        self.assertEqual(generic_python_error["declarationState"], "declared")
        self.assertEqual(generic_python_error["capabilityId"], "ffi_runtime_metadata")
        self.assertIn("languageMappings", generic_python_error)
        generic_java_error = declarations["org.kmipkit.KmipException"]
        self.assertEqual(generic_java_error["declarationState"], "declared")
        self.assertEqual(generic_java_error["capabilityId"], "ffi_runtime_metadata")
        self.assertIn("languageMappings", generic_java_error)
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
        self.assertEqual(len(declarations), 9673)
        self.assertEqual(len(exclusions), 784)
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
        self.assertEqual(len(rust_exclusions), 126 + len(t087_exclusions))
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
        self.assertEqual(len(declarations), 939)
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
        self.assertEqual(len(declarations), 907)
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


if __name__ == "__main__":
    unittest.main()
