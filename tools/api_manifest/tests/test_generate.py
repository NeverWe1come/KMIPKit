"""Behavioral tests for the public API manifest generator CLI."""

from __future__ import annotations

import copy
import errno
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from jsonschema import Draft202012Validator

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
GENERATOR = ROOT / "tools" / "api_manifest" / "generate.py"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
SCHEMA = ROOT / "specification" / "api" / "public-api.schema.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
MINIMAL_FIXTURE = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "registry-manifest.json"
PUBLIC_API_V1_FIXTURE = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "public-api-v1.json"
GENERATED_OUTPUTS = (
    "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "bindings/c/include/kmipkit.h",
    "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "tests/fixtures/extensions/generated/registry_parity.json",
)
V2_GENERATED_OUTPUTS = (
    *GENERATED_OUTPUTS,
    "crates/kmipkit-ffi/src/public_api_generated.rs",
    "bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java",
    "bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java",
    "bindings/python/src/kmipkit/_generated/public_api.py",
    "tests/fixtures/api/generated/public_api_parity.json",
    "tests/fixtures/api/generated/kmipkit_internal.h",
)
GENERATED_OUTPUT_MAP = {
    "rustFfi": GENERATED_OUTPUTS[0],
    "cHeader": GENERATED_OUTPUTS[1],
    "javaApi": GENERATED_OUTPUTS[2],
    "javaParityTests": GENERATED_OUTPUTS[3],
    "pythonApi": GENERATED_OUTPUTS[4],
    "parityFixtures": GENERATED_OUTPUTS[5],
}
HANDLE_ARRAY_CONTRACTS = (
    (
        "extension_schema_structure",
        "children",
        "kmipkit_extension_child_rule_t",
        "child_count",
    ),
    (
        "extension_schema_structure",
        "order_constraints",
        "kmipkit_extension_order_constraint_t",
        "order_constraint_count",
    ),
    (
        "client_extension_registry_create",
        "definitions",
        "kmipkit_extension_definition_t",
        "definition_count",
    ),
)
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
SYMLINK_DENIAL_ERRNOS = {
    errno.EACCES,
    errno.EPERM,
    getattr(errno, "ENOTSUP", errno.EPERM),
    getattr(errno, "EOPNOTSUPP", errno.EPERM),
}
SYMLINK_DENIAL_WINERRORS = {5, 50, 1314}


def _create_repo(parent: Path) -> Path:
    """Copy the checked-in generation inputs into an isolated temporary root."""
    root = parent / "repo"
    api_dir = root / "specification" / "api"
    api_dir.mkdir(parents=True)
    shutil.copy2(MANIFEST, api_dir / MANIFEST.name)
    shutil.copy2(SCHEMA, api_dir / SCHEMA.name)
    catalog_path = root / "specification" / "catalog" / CATALOG.name
    catalog_path.parent.mkdir(parents=True)
    shutil.copy2(CATALOG, catalog_path)

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    dependency_matrix = ROOT / "specs" / "043-public-api-bindings" / "contracts" / "operation-dependencies.md"
    dependency_destination = root / "specs" / "043-public-api-bindings" / "contracts" / dependency_matrix.name
    dependency_destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(dependency_matrix, dependency_destination)
    for operation in manifest.get("operations", []):
        dependency = operation.get("ownerDependency", {})
        owner_id = dependency.get("specId")
        if not isinstance(owner_id, str):
            continue
        owner_directory = generate._feature_directory(ROOT, owner_id)
        source_path = owner_directory / "spec.md"
        destination = root / "specs" / owner_directory.name / "spec.md"
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source_path, destination)
    referenced_paths: set[str] = set()
    for collection_name in (
        "operations",
        "capabilities",
        "publicDeclarations",
        "approvedExclusions",
    ):
        for record in manifest.get(collection_name, []):
            for source_name in ("approvalSource", "approvalEvidence", "acceptedEvidence"):
                source = record.get(source_name)
                if not isinstance(source, dict):
                    continue
                relative_path = source.get("path")
                if isinstance(relative_path, str):
                    referenced_paths.add(relative_path)
                    parts = Path(relative_path).parts
                    if parts and parts[0] == "specs":
                        referenced_paths.add(str(Path(*parts[:2]) / "spec.md"))
            referenced_paths.update(record.get("parityEvidenceIds", []))

    for relative_path in referenced_paths:
        source_path = ROOT / relative_path
        destination = root / relative_path
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source_path, destination)
    return root


def _create_fixture_repo(parent: Path) -> Path:
    """Create an isolated repo root with the minimal valid T002 manifest."""
    root = parent / "repo"
    api_dir = root / "specification" / "api"
    api_dir.mkdir(parents=True)
    shutil.copy2(MINIMAL_FIXTURE, api_dir / "public-api.json")
    shutil.copy2(SCHEMA, api_dir / SCHEMA.name)
    return root


def _reverse_object_keys(value: object) -> object:
    if isinstance(value, dict):
        return {key: _reverse_object_keys(child) for key, child in reversed(list(value.items()))}
    if isinstance(value, list):
        return [_reverse_object_keys(child) for child in value]
    return value


def _find_function(manifest: dict[str, object], function_id: str) -> dict[str, object]:
    return next(function for function in manifest["functions"] if function["id"] == function_id)


def _find_c_parameter(
    manifest: dict[str, object], function_id: str, parameter_name: str
) -> dict[str, object]:
    function = _find_function(manifest, function_id)
    return next(
        parameter
        for parameter in function["c"]["parameters"]
        if parameter["name"] == parameter_name
    )


def _apply_handle_array_contract(manifest: dict[str, object]) -> None:
    for function_id, parameter_name, handle_type, count_parameter in HANDLE_ARRAY_CONTRACTS:
        parameter = _find_c_parameter(manifest, function_id, parameter_name)
        parameter.update(
            {
                "kind": "handle-array",
                "type": f"{handle_type} **",
                "handleType": handle_type,
                "ownership": "borrowed",
                "countParameter": count_parameter,
                "nullable": True,
            }
        )


def _apply_input_array_contract(manifest: dict[str, object]) -> None:
    parameter = _find_c_parameter(manifest, "extension_schema_structure", "children")
    parameter.clear()
    parameter.update(
        {
            "name": "children",
            "kind": "input-array",
            "type": "const uint32_t *",
            "elementType": "uint32_t",
            "ownership": "borrowed",
            "countParameter": "child_count",
            "nullable": True,
        }
    )


class ManifestGeneratorCliTests(unittest.TestCase):
    def test_v1_reference_validation_does_not_require_v2_declarations(self) -> None:
        manifest = json.loads(PUBLIC_API_V1_FIXTURE.read_text(encoding="utf-8"))

        generate._validate_references(manifest)

    def test_v2_full_api_header_shares_new_symbols_without_polluting_registry_slice(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        version_query = copy.deepcopy(manifest["functions"][0])
        version_query["id"] = "version_query_fixture"
        version_query["requirementIds"] = ["KMIPKIT-0043-FR-007"]
        version_query["c"]["symbol"] = "kmipkit_version_get"
        version_query["java"]["method"] = "versionGet"
        version_query["python"]["function"] = "version_get"
        manifest["functions"].append(version_query)

        outputs = generate.render_outputs(manifest)
        c_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")
        registry_python = outputs[
            "bindings/python/src/kmipkit/_generated/extension_registry.py"
        ].decode("utf-8")

        self.assertIn("kmipkit_version_get", c_header)
        self.assertNotIn("version_get", registry_python)

    def test_v2_rendering_does_not_mutate_the_canonical_manifest(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        original = copy.deepcopy(manifest)

        generate.render_outputs(manifest)

        self.assertEqual(manifest, original)

    def test_c_header_renders_manifest_documentation_for_error_snapshot_outputs(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        c_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")

        self.assertIn("out_category receives 0 none, 1 validation, 2 protocol", c_header)
        self.assertIn(
            "out_cause receives 0 none, 1 invalid input, 2 protocol invalid value, "
            "3 protocol invalid encoding, 4 other protocol, 5 transport I/O, 6 TLS, "
            "7 HTTP, 8 timeout, 9 other transport, or 10 other.",
            c_header,
        )
        self.assertIn(
            "out_delivery receives 0 none, 1 not sent, 2 possibly sent, or 3 response started; "
            "server results use 0, while protocol and transport failures report the strongest observed delivery state.",
            c_header,
        )

    def test_public_c_header_exposes_generic_ttlv_builders_for_import(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        private_header = outputs[
            "tests/fixtures/api/generated/kmipkit_internal.h"
        ].decode("utf-8")
        public_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")

        self.assertIn("KMIPKIT_INTERNAL_H", private_header)
        for symbol in (
            "kmipkit_ttlv_value_big_integer",
            "kmipkit_ttlv_value_byte_string",
            "kmipkit_ttlv_value_text_string",
            "kmipkit_ttlv_value_structure",
            "kmipkit_ttlv_item_create",
            "kmipkit_ttlv_structure_with_item",
            "kmipkit_raw_tag_release",
            "kmipkit_ttlv_value_release",
            "kmipkit_ttlv_item_release",
        ):
            with self.subTest(symbol=symbol):
                self.assertIn(f"{symbol}(", public_header)

    def test_public_big_integer_ttlv_declaration_has_four_language_parity(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = {
            row["declarationId"]: row
            for row in manifest["publicDeclarations"]
            if row["declarationId"].startswith("ttlv_value_big_integer:")
        }
        expected = {
            "ttlv_value_big_integer:c": "kmipkit_ttlv_value_big_integer",
            "ttlv_value_big_integer:java": "org.kmipkit.ttlv.TtlvValue.BigInteger",
            "ttlv_value_big_integer:python": "kmipkit.ttlv.ttlv_value_big_integer",
        }
        self.assertEqual(set(expected), set(declarations))
        rust_declaration = next(
            row
            for row in manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit_ttlv::Value::big_integer"
        )
        self.assertEqual("declared", rust_declaration["declarationState"])
        self.assertEqual("generic_ttlv_model", rust_declaration["capabilityId"])
        self.assertIn("KMIPKIT-0004-FR-005", rust_declaration["requirementIds"])
        self.assertIn("KMIPKIT-0043-FR-021", rust_declaration["requirementIds"])
        expected_mappings = {
            "rust": "kmipkit_ttlv::Value::big_integer",
            "c": "kmipkit_ttlv_value_big_integer",
            "java": "org.kmipkit.ttlv.TtlvValue.BigInteger",
            "python": "kmipkit.ttlv.ttlv_value_big_integer",
        }
        for declaration_id, mapping_name in expected.items():
            with self.subTest(declaration_id=declaration_id):
                declaration = declarations[declaration_id]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("generic_ttlv_model", declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(mapping_name, declaration["exportName"])
                self.assertIn("KMIPKIT-0004-FR-005", declaration["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                mappings = declaration["languageMappings"]
                self.assertEqual(set(expected_mappings), set(mappings))
                for language, expected_name in expected_mappings.items():
                    self.assertEqual(expected_name, mappings[language]["name"])

    def test_public_c_header_documents_consumed_ttlv_handle_transfer(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        public_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")

        required_contracts = {
            "ttlv_value_structure": (
                "Transfers ownership of structure after the output slot and limits handle are valid. "
                "If validation fails before transfer, the caller retains structure; after transfer, "
                "the caller must not use or release it, even if this function returns an error."
            ),
            "ttlv_item_create": (
                "Transfers ownership of value only after the output slot, tag, and limits are valid. "
                "If validation fails before transfer, the caller retains value; after transfer, "
                "the caller must not use or release it, even if this function returns an error."
            ),
            "ttlv_structure_with_item": (
                "Transfers ownership of each input structure and item as it is accepted. Validation "
                "errors before transfer leave that handle caller-owned; after transfer, the caller "
                "must not use or release it, even if later processing returns an error."
            ),
        }
        functions = {function["id"]: function for function in manifest["functions"]}
        for function_id, contract in required_contracts.items():
            with self.subTest(function_id=function_id):
                documentation = " ".join(functions[function_id]["c"].get("documentation", []))
                self.assertIn(contract, documentation)
                self.assertIn(contract, public_header)

    def test_public_header_exposes_error_ttlv_after_t081_mapping(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        public_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")

        t081_declaration = next(
            row
            for row in manifest["publicDeclarations"]
            if row["declarationId"] == "T081:client-execute-error-ttlv:c"
        )
        self.assertEqual("declared", t081_declaration["declarationState"])
        self.assertEqual("crypto_response_ttlv", t081_declaration["capabilityId"])
        self.assertIn("KMIPKIT-0021-FR-015", t081_declaration["requirementIds"])
        self.assertEqual(
            {"rust", "c", "java", "python"},
            set(t081_declaration["languageMappings"]),
        )
        self.assertIn("kmipkit_client_execute_with_error_ttlv_v1(", public_header)
        self.assertIn("kmipkit_client_execute(", public_header)
        self.assertIn("typedef struct kmipkit_client_t kmipkit_client_t;", public_header)
        self.assertIn("kmipkit_transport_config_create_v1(", public_header)

    def test_t081_public_c_header_exposes_retained_generic_response_access(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        public_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")

        required_symbols = {
            "kmipkit_client_execute_with_error_ttlv_v1",
            "kmipkit_ttlv_structure_view",
            "kmipkit_ttlv_structure_view_item_count",
            "kmipkit_ttlv_structure_view_item_at",
            "kmipkit_ttlv_item_view_tag",
            "kmipkit_ttlv_item_view_type",
            "kmipkit_ttlv_item_view_value",
            "kmipkit_ttlv_value_view_type",
            "kmipkit_ttlv_value_view_structure",
            "kmipkit_ttlv_value_view_boolean",
            "kmipkit_ttlv_value_view_integer",
            "kmipkit_ttlv_value_view_long_integer",
            "kmipkit_ttlv_value_view_enumeration",
            "kmipkit_ttlv_value_view_date_time",
            "kmipkit_ttlv_value_view_interval",
            "kmipkit_ttlv_value_view_date_time_extended",
            "kmipkit_ttlv_value_view_byte_length",
            "kmipkit_ttlv_value_view_byte_at",
            "kmipkit_ttlv_structure_view_release",
            "kmipkit_ttlv_item_view_release",
            "kmipkit_ttlv_value_view_release",
        }
        for symbol in required_symbols:
            with self.subTest(symbol=symbol):
                self.assertTrue(
                    f"{symbol}(" in public_header,
                    f"generated public C header is missing {symbol}",
                )

        for handle_type in (
            "kmipkit_ttlv_structure_t",
            "kmipkit_ttlv_structure_view_t",
            "kmipkit_ttlv_item_view_t",
            "kmipkit_ttlv_value_view_t",
        ):
            with self.subTest(handle_type=handle_type):
                self.assertTrue(
                    f"typedef struct {handle_type} {handle_type};" in public_header,
                    f"generated public C header is missing {handle_type}",
                )

    def test_t081_error_ttlv_helper_maps_to_its_distinct_python_abi_symbol(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        helper = next(
            row for row in manifest["functions"]
            if row["id"] == "client_execute_with_error_ttlv"
        )

        self.assertEqual(
            "kmipkit_client_execute_with_error_ttlv_v1",
            helper["c"]["symbol"],
        )
        self.assertEqual(
            "kmipkit_client_execute_with_error_ttlv_v1",
            helper["python"]["cffiSymbol"],
            "Python must call the additive ABI that returns the retained error TTLV",
        )
        self.assertEqual("Client.execute", helper["python"]["function"])
        documentation = " ".join(helper["c"]["documentation"])
        self.assertIn("only for an approved missing, repeated, or malformed Unique Identifier", documentation)
        self.assertNotIn("when typed response validation fails", documentation)

        declaration = next(
            row for row in manifest["publicDeclarations"]
            if row["declarationId"] == "T081:client-execute-error-ttlv:c"
        )
        self.assertEqual("declared", declaration["declarationState"])
        self.assertEqual("crypto_response_ttlv", declaration["capabilityId"])
        self.assertEqual(
            {"rust", "c", "java", "python"},
            set(declaration["languageMappings"]),
        )

        python_abi_symbols = [row["python"]["cffiSymbol"] for row in manifest["functions"]]
        self.assertEqual(
            len(python_abi_symbols),
            len(set(python_abi_symbols)),
            "each Python facade function must resolve to its actual CFFI ABI symbol",
        )

    def test_t076_batch_metadata_carrier_is_not_approved_or_public(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        public_header = outputs["bindings/c/include/kmipkit.h"].decode("utf-8")
        carrier_name = "kmipkit_client_batch_item_metadata_v1_t"
        t076_declarations = [
            row for row in manifest["publicDeclarations"]
            if row["declarationId"].startswith("T076:")
        ]

        carrier = next(
            row for row in manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit" and row["exportName"] == carrier_name
        )
        self.assertEqual(carrier_name, carrier["cType"]["name"])
        self.assertEqual(
            ["struct_size", "abi_version", "outcome_kind", "result_status", "has_result_reason", "result_reason"],
            [field["name"] for field in carrier["cType"]["fields"]],
        )
        self.assertIn(f"typedef struct {carrier_name} {carrier_name};", public_header)
        self.assertIn(f"struct {carrier_name} {{", public_header)
        self.assertIn("kmipkit_client_batch_item_response_metadata_v1(", public_header)
        self.assertTrue(t076_declarations)

    def test_versioned_fixed_width_values_render_declared_c_fields(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        type_name = "kmipkit_test_config_v1_t"
        manifest["types"].append(
            {
                "id": "TestConfigV1",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "TestConfigV1", "owner": "ffi"},
                "c": {
                    "name": type_name,
                    "kind": "fixed-width-value",
                    "versioned": True,
                    "fields": [
                        {"name": "struct_size", "type": "uint32_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                        {"name": "port", "type": "uint16_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "TestConfigV1"},
                "python": {"module": "kmipkit", "name": "TestConfigV1"},
            }
        )
        manifest["publicDeclarations"].append(
            {
                "declarationId": "test-config-v1:c",
                "packageId": "c-kmipkit",
                "kind": "type",
                "exportName": type_name,
                "capabilityId": "public_binding_surface",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "declarationState": "declared",
            }
        )
        manifest["publicDeclarations"].extend(
            {
                "declarationId": f"test-config-v1:{field['name']}",
                "packageId": "c-kmipkit",
                "kind": "field",
                "exportName": f"{type_name}.{field['name']}",
                "capabilityId": "public_binding_surface",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "declarationState": "declared",
            }
            for field in manifest["types"][-1]["c"]["fields"][2:]
        )
        prefix_exclusions = [
            copy.deepcopy(exclusion)
            for exclusion in manifest["approvedExclusions"]
            if exclusion["exportName"]
            in {
                "kmipkit_extension_registry_limits_v1_t.struct_size",
                "kmipkit_extension_registry_limits_v1_t.abi_version",
            }
        ]
        for exclusion in prefix_exclusions:
            exclusion["exportName"] = f"{type_name}.{exclusion['exportName'].rsplit('.', 1)[1]}"
            manifest["approvedExclusions"].append(exclusion)

        header = generate.render_c_header(manifest)

        self.assertIn("typedef struct kmipkit_test_config_v1_t kmipkit_test_config_v1_t;", header)
        self.assertIn(
            "struct kmipkit_test_config_v1_t {\n"
            "    uint32_t struct_size;\n"
            "    uint32_t abi_version;\n"
            "    uint16_t port;\n"
            "};",
            header,
        )

    def test_rust_ffi_renders_fixed_width_value_fields_from_the_type_record(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        type_name = "kmipkit_test_config_v1_t"
        manifest["types"].append(
            {
                "id": "TestConfigV1",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "TestConfigV1", "owner": "ffi"},
                "c": {
                    "name": type_name,
                    "kind": "fixed-width-value",
                    "versioned": True,
                    "fields": [
                        {"name": "struct_size", "type": "uint32_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                        {"name": "port", "type": "uint16_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "TestConfigV1"},
                "python": {"module": "kmipkit", "name": "TestConfigV1"},
            }
        )

        generated = generate.render_rust_ffi(manifest)
        record = generated.split(f"pub struct {type_name} {{", maxsplit=1)[1].split("}", maxsplit=1)[0]

        self.assertIn("pub struct_size: u32,", record)
        self.assertIn("pub abi_version: u32,", record)
        self.assertIn("pub port: u16,", record)
        self.assertNotIn("pub max_definitions: u64,", record)

    def test_rust_ffi_preserves_const_only_at_the_innermost_c_pointer_level(self) -> None:
        self.assertEqual(generate._rust_c_type("const uint8_t **"), "*mut *const u8")

    def test_rust_ffi_escapes_keyword_names_in_fixed_width_fields(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "KeywordFieldConfig",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "KeywordFieldConfig", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_keyword_field_config_t",
                    "kind": "fixed-width-value",
                    "fields": [{"name": "type", "type": "uint32_t"}],
                },
                "java": {"package": "org.kmipkit", "name": "KeywordFieldConfig"},
                "python": {"module": "kmipkit", "name": "KeywordFieldConfig"},
            }
        )

        generated = generate.render_rust_ffi(manifest)

        self.assertIn("pub r#type: u32,", generated)

    def test_c_header_orders_nested_value_definitions_after_forward_declarations(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

        def add_ready_value(name: str, fields: list[dict[str, str]]) -> None:
            manifest["types"].append(
                {
                    "id": name.removeprefix("kmipkit_").removesuffix("_t").title(),
                    "kind": "struct",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "rust": {"module": "kmipkit_ffi", "name": "TestValue", "owner": "ffi"},
                    "c": {"name": name, "kind": "fixed-width-value", "fields": fields},
                    "java": {"package": "org.kmipkit", "name": "TestValue"},
                    "python": {"module": "kmipkit", "name": "TestValue"},
                }
            )
            manifest["publicDeclarations"].append(
                {
                    "declarationId": f"{name}:c",
                    "packageId": "c-kmipkit",
                    "kind": "type",
                    "exportName": name,
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": "declared",
                }
            )
            for field in fields:
                manifest["publicDeclarations"].append(
                    {
                        "declarationId": f"{name}.{field['name']}:c",
                        "packageId": "c-kmipkit",
                        "kind": "field",
                        "exportName": f"{name}.{field['name']}",
                        "capabilityId": "public_binding_surface",
                        "requirementIds": ["KMIPKIT-0043-FR-007"],
                        "declarationState": "declared",
                    }
                )

        add_ready_value("kmipkit_outer_t", [{"name": "inner", "type": "kmipkit_inner_t"}])
        add_ready_value("kmipkit_inner_t", [{"name": "value", "type": "uint32_t"}])

        header = generate.render_c_header(manifest)

        self.assertLess(header.index("uint32_t value;"), header.index("kmipkit_inner_t inner;"))
        self.assertIn("typedef struct kmipkit_outer_t kmipkit_outer_t;", header)
        self.assertIn("typedef struct kmipkit_inner_t kmipkit_inner_t;", header)

    def test_c_header_rejects_ready_values_that_reference_blocked_types(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outer_name = "kmipkit_outer_blocked_ref_t"
        blocked_name = "kmipkit_blocked_inner_t"
        for name, field, state in (
            (outer_name, {"name": "inner", "type": blocked_name}, "declared"),
            (blocked_name, {"name": "value", "type": "uint32_t"}, "blocked"),
        ):
            manifest["types"].append(
                {
                    "id": name,
                    "kind": "struct",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "rust": {"module": "kmipkit_ffi", "name": "TestValue", "owner": "ffi"},
                    "c": {"name": name, "kind": "fixed-width-value", "fields": [field]},
                    "java": {"package": "org.kmipkit", "name": "TestValue"},
                    "python": {"module": "kmipkit", "name": "TestValue"},
                }
            )
            manifest["publicDeclarations"].append(
                {
                    "declarationId": f"{name}:c",
                    "packageId": "c-kmipkit",
                    "kind": "type",
                    "exportName": name,
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": state,
                }
            )
            manifest["publicDeclarations"].append(
                {
                    "declarationId": f"{name}.{field['name']}:c",
                    "packageId": "c-kmipkit",
                    "kind": "field",
                    "exportName": f"{name}.{field['name']}",
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": state,
                }
            )

        with self.assertRaisesRegex(generate.ManifestError, "references a blocked C type"):
            generate.render_c_header(manifest)

    def test_c_header_rejects_ready_function_using_partial_value_by_value(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        type_name = "kmipkit_partial_value_t"
        manifest["types"].append(
            {
                "id": "PartialValue",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "PartialValue", "owner": "ffi"},
                "c": {
                    "name": type_name,
                    "kind": "fixed-width-value",
                    "fields": [
                        {"name": "first", "type": "uint32_t"},
                        {"name": "second", "type": "uint32_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "PartialValue"},
                "python": {"module": "kmipkit", "name": "PartialValue"},
            }
        )
        manifest["publicDeclarations"].extend(
            [
                {
                    "declarationId": f"{type_name}:c",
                    "packageId": "c-kmipkit",
                    "kind": "type",
                    "exportName": type_name,
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": "declared",
                },
                {
                    "declarationId": f"{type_name}.first:c",
                    "packageId": "c-kmipkit",
                    "kind": "field",
                    "exportName": f"{type_name}.first",
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": "declared",
                },
                {
                    "declarationId": f"{type_name}.second:c",
                    "packageId": "c-kmipkit",
                    "kind": "field",
                    "exportName": f"{type_name}.second",
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": "blocked",
                },
                {
                    "declarationId": "partial_value_roundtrip:c",
                    "packageId": "c-kmipkit",
                    "kind": "function",
                    "exportName": "kmipkit_partial_value_roundtrip",
                    "capabilityId": "public_binding_surface",
                    "requirementIds": ["KMIPKIT-0043-FR-007"],
                    "declarationState": "declared",
                },
            ]
        )
        manifest["functions"].append(
            {
                "id": "partial_value_roundtrip",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "c": {
                    "symbol": "kmipkit_partial_value_roundtrip",
                    "returnType": type_name,
                    "parameters": [{"kind": "scalar", "type": type_name, "name": "value"}],
                },
            }
        )

        with self.assertRaisesRegex(generate.ManifestError, "incomplete C value by value"):
            generate.render_c_header(manifest)

    def test_references_reject_duplicate_fixed_width_value_field_names(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "DuplicateFieldConfig",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "DuplicateFieldConfig", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_duplicate_field_config_t",
                    "kind": "fixed-width-value",
                    "fields": [
                        {"name": "port", "type": "uint16_t"},
                        {"name": "port", "type": "uint32_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "DuplicateFieldConfig"},
                "python": {"module": "kmipkit", "name": "DuplicateFieldConfig"},
            }
        )

        with self.assertRaisesRegex(generate.ManifestError, "duplicate field names"):
            generate._validate_references(manifest)

    def test_references_reject_c_keywords_as_fixed_width_field_names(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "ReservedCField",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "ReservedCField", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_reserved_c_field_t",
                    "kind": "fixed-width-value",
                    "fields": [{"name": "struct", "type": "uint32_t"}],
                },
                "java": {"package": "org.kmipkit", "name": "ReservedCField"},
                "python": {"module": "kmipkit", "name": "ReservedCField"},
            }
        )

        with self.assertRaisesRegex(generate.ManifestError, "C.*keyword"):
            generate._validate_references(manifest)

    def test_references_reject_cpp_keywords_as_fixed_width_field_names(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "ReservedCppField",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "ReservedCppField", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_reserved_cpp_field_t",
                    "kind": "fixed-width-value",
                    "fields": [{"name": "class", "type": "uint32_t"}],
                },
                "java": {"package": "org.kmipkit", "name": "ReservedCppField"},
                "python": {"module": "kmipkit", "name": "ReservedCppField"},
            }
        )

        with self.assertRaisesRegex(generate.ManifestError, "keyword"):
            generate._validate_references(manifest)

    def test_references_allow_nonstandard_cpp_extension_identifiers(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "ExtensionIdentifierFields",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {
                    "module": "kmipkit_ffi",
                    "name": "ExtensionIdentifierFields",
                    "owner": "ffi",
                },
                "c": {
                    "name": "kmipkit_extension_identifier_fields_t",
                    "kind": "fixed-width-value",
                    "fields": [
                        {"name": "atomic_cancel", "type": "uint32_t"},
                        {"name": "atomic_commit", "type": "uint32_t"},
                        {"name": "atomic_noexcept", "type": "uint32_t"},
                        {"name": "synchronized", "type": "uint32_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "ExtensionIdentifierFields"},
                "python": {"module": "kmipkit", "name": "ExtensionIdentifierFields"},
            }
        )

        generate._validate_references(manifest)

    def test_references_require_versioned_c_value_header_fields(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        manifest["types"].append(
            {
                "id": "MalformedVersionedConfig",
                "kind": "struct",
                "requirementIds": ["KMIPKIT-0043-FR-007"],
                "rust": {"module": "kmipkit_ffi", "name": "MalformedVersionedConfig", "owner": "ffi"},
                "c": {
                    "name": "kmipkit_malformed_versioned_config_t",
                    "kind": "fixed-width-value",
                    "versioned": True,
                    "fields": [
                        {"name": "port", "type": "uint16_t"},
                        {"name": "abi_version", "type": "uint32_t"},
                    ],
                },
                "java": {"package": "org.kmipkit", "name": "MalformedVersionedConfig"},
                "python": {"module": "kmipkit", "name": "MalformedVersionedConfig"},
            }
        )

        with self.assertRaisesRegex(generate.ManifestError, "begin with its size and ABI version"):
            generate._validate_references(manifest)

    def test_generated_rust_metadata_lists_only_unblocked_c_mapped_capabilities(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        outputs = generate.render_outputs(manifest)
        generated = generate.render_public_rust_api(
            generate._ready_api_projection(manifest)
        )

        self.assertEqual(set(outputs), set(generate.V2_OUTPUT_PATHS.values()))
        self.assertEqual(len(outputs), 12)
        self.assertIn('"ffi_runtime_metadata"', generated)
        self.assertIn('"generic_ttlv_model"', generated)
        self.assertIn('"ttlv_wire_codec"', generated)
        self.assertIn('"vendor_extension_registry"', generated)
        self.assertNotIn('"public_binding_surface"', generated)
        self.assertNotIn('"rust_transport_exchange"', generated)

    def test_java_public_api_splits_large_declaration_json_without_losing_content(self) -> None:
        long_name = "x" * 70_000
        projection = {
            "publicPackages": [{"id": "java-kmipkit", "language": "java"}],
            "publicDeclarations": [
                {
                    "declarationId": "large-java-declaration",
                    "packageId": "java-kmipkit",
                    "kind": "java-member",
                    "languageMappings": {"java": {"name": long_name}},
                }
            ],
        }

        generated = generate.render_public_java_api(projection)
        self.assertIn("DECLARATIONS_JSON_PARTS", generated)
        self.assertIn('String.join("", DECLARATIONS_JSON_PARTS)', generated)
        parts = [
            json.loads(line.strip()[:-1])
            for line in generated.splitlines()
            if line.strip().startswith('"') and line.strip().endswith('",')
        ]
        self.assertGreater(len(parts), 1)
        self.assertTrue(all(len(part.encode("utf-8")) < 60_000 for part in parts))
        self.assertEqual(
            '[{"declarationId":"large-java-declaration","kind":"java-member",'
            f'"name":"{long_name}","packageId":"java-kmipkit"}}]',
            "".join(parts),
        )

    def _run_generator(self, root: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        self.assertTrue(
            GENERATOR.is_file(),
            "manifest generator CLI must be implemented before its behavior can be exercised",
        )
        return subprocess.run(
            [sys.executable, "-B", str(GENERATOR), *arguments],
            cwd=root,
            capture_output=True,
            check=False,
            text=True,
        )

    def _load_manifest(self, root: Path) -> dict[str, object]:
        path = root / "specification" / "api" / "public-api.json"
        return json.loads(path.read_text(encoding="utf-8"))

    def _write_manifest(self, root: Path, manifest: dict[str, object]) -> None:
        path = root / "specification" / "api" / "public-api.json"
        path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def test_v1_registry_slice_generation_does_not_require_the_v2_catalog(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_fixture_repo(Path(directory))
            shutil.copy2(
                PUBLIC_API_V1_FIXTURE,
                root / "specification" / "api" / "public-api.json",
            )

            result = self._run_generator(root, "--write")

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(all((root / path).is_file() for path in GENERATED_OUTPUTS))

    def _assert_symlink_creation_or_skip(self, link: Path, target: Path, *, directory: bool) -> None:
        try:
            link.symlink_to(target, target_is_directory=directory)
        except OSError as error:
            denied = error.errno in SYMLINK_DENIAL_ERRNOS or getattr(error, "winerror", None) in SYMLINK_DENIAL_WINERRORS
            if denied:
                self.skipTest(f"the operating system denied symlink creation: {error}")
            raise

    def test_write_is_deterministic_across_repeated_runs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))

            first = self._run_generator(root, "--write")
            self.assertEqual(first.returncode, 0, first.stderr)
            first_outputs = {
                relative_path: (root / relative_path).read_bytes()
                for relative_path in V2_GENERATED_OUTPUTS
            }
            self.assertTrue(all(first_outputs.values()))

            second = self._run_generator(root, "--write")

            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(
                {relative_path: (root / relative_path).read_bytes() for relative_path in V2_GENERATED_OUTPUTS},
                first_outputs,
            )

    def test_v2_outputs_do_not_depend_on_repository_path_or_hash_seed(self) -> None:
        results: list[dict[str, bytes]] = []
        for seed in ("1", "987654"):
            with tempfile.TemporaryDirectory(prefix=f"kmipkit-{seed}-") as directory:
                root = _create_repo(Path(directory))
                environment = os.environ.copy()
                environment["PYTHONHASHSEED"] = seed
                result = subprocess.run(
                    [sys.executable, "-B", str(GENERATOR), "--write"],
                    cwd=root,
                    env=environment,
                    capture_output=True,
                    check=False,
                    text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                results.append({
                    relative_path: (root / relative_path).read_bytes()
                    for relative_path in V2_GENERATED_OUTPUTS
                })

        self.assertEqual(results[0], results[1])

    def test_object_key_order_does_not_change_generated_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            first = self._run_generator(root, "--write")
            self.assertEqual(first.returncode, 0, first.stderr)
            first_outputs = {
                relative_path: (root / relative_path).read_bytes()
                for relative_path in V2_GENERATED_OUTPUTS
            }
            manifest = self._load_manifest(root)
            reversed_manifest = _reverse_object_keys(manifest)
            self.assertIsInstance(reversed_manifest, dict)
            self._write_manifest(root, reversed_manifest)

            second = self._run_generator(root, "--write")

            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(
                {relative_path: (root / relative_path).read_bytes() for relative_path in V2_GENERATED_OUTPUTS},
                first_outputs,
            )

    def test_unsorted_requirement_ids_are_rejected_before_writing(self) -> None:
        mutations = (
            ("root requirement IDs", lambda value: value["requirementIds"].reverse()),
            ("type requirement IDs", lambda value: value["types"][0]["requirementIds"].reverse()),
            ("function requirement IDs", lambda value: value["functions"][0]["requirementIds"].reverse()),
        )
        for label, mutate in mutations:
            with self.subTest(location=label), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                mutate(manifest)
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_required_invalid_input_error_mappings_are_enforced(self) -> None:
        mutations = (
            ("C handle mapping", "client_extension_registry_inspect", "c"),
            ("Java handle mapping", "client_extension_registry_inspect", "java"),
            ("Python handle mapping", "client_extension_registry_inspect", "python"),
            ("Java Enumeration range", "ttlv_value_enumeration", "java"),
            ("Python Enumeration range", "ttlv_value_enumeration", "python"),
            ("Java Interval range", "ttlv_value_interval", "java"),
            ("Python Interval range", "ttlv_value_interval", "python"),
            ("C Boolean input", "ttlv_value_boolean", "c"),
        )
        for label, function_id, language in mutations:
            with self.subTest(mapping=label), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                function = next(item for item in manifest["functions"] if item["id"] == function_id)
                function[language]["errorCategories"].remove("invalid_input")
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_handle_arrays_preserve_typed_pointer_to_pointer_abi(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            _apply_handle_array_contract(manifest)
            for function_id, _, _, _ in HANDLE_ARRAY_CONTRACTS:
                function = _find_function(manifest, function_id)
                for language in ("c", "java", "python"):
                    self.assertIn("invalid_input", function[language]["errorCategories"])
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertEqual(result.returncode, 0, result.stderr)
            c_header = (root / GENERATED_OUTPUTS[1]).read_text(encoding="utf-8")
            rust_ffi = (root / GENERATED_OUTPUTS[0]).read_text(encoding="utf-8")
            self.assertIn("NULL is valid only when the linked count is zero", c_header)
            self.assertIn("check the count before reading elements", c_header)
            for _, parameter_name, handle_type, _ in HANDLE_ARRAY_CONTRACTS:
                self.assertIn(f"{handle_type} ** {parameter_name}", c_header)
                self.assertIn(f"{parameter_name}: *mut *mut {handle_type}", rust_ffi)

    def test_consumed_handle_slot_and_output_value_pointer_keep_exact_abi(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            function = _find_function(manifest, "client_configuration_new")
            function["c"]["parameters"] = [
                {
                    "name": "extension_registry",
                    "kind": "consumed-handle-slot",
                    "type": "kmipkit_client_extension_registry_t **",
                    "handleType": "kmipkit_client_extension_registry_t",
                    "ownership": "consumed",
                },
                {
                    "name": "out_limits",
                    "kind": "output-value-pointer",
                    "type": "kmipkit_byte_span_v1_t *",
                    "valueType": "kmipkit_byte_span_v1_t",
                    "ownership": "none",
                },
            ]
            manifest["publicDeclarations"] = [
                declaration
                for declaration in manifest["publicDeclarations"]
                if not (
                    declaration.get("packageId") == "c-kmipkit"
                    and declaration.get("exportName") == "kmipkit_byte_span_v1_t"
                )
            ]
            manifest["approvedExclusions"] = [
                exclusion
                for exclusion in manifest["approvedExclusions"]
                if not (
                    exclusion.get("packageId") == "c-kmipkit"
                    and exclusion.get("exportName") == "kmipkit_byte_span_v1_t"
                )
            ]
            manifest["approvedExclusions"].append(
                {
                    "packageId": "c-kmipkit",
                    "exportName": "kmipkit_byte_span_v1_t",
                    "reason": (
                        "The C byte-span carrier is an ABI layout; Rust exposes a byte slice, "
                        "Java exposes copied byte arrays, and Python exposes copied bytes."
                    ),
                    "approvalSource": {
                        "kind": "feature-specification",
                        "path": "specs/043-public-api-bindings/spec.md",
                        "status": "Approved",
                        "evidence": "**Status**: Approved for implementation",
                    },
                    "evidence": (
                        "The exact C layout is retained in the generated header; the "
                        "cross-language value is the copied or borrowed byte sequence."
                    ),
                    "cType": {
                        "name": "kmipkit_byte_span_v1_t",
                        "kind": "fixed-width-value",
                        "fields": [
                            {"name": "data", "type": "const uint8_t *"},
                            {"name": "length", "type": "uint64_t"},
                        ],
                    },
                }
            )
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertEqual(0, result.returncode, result.stderr)
            public_header = (root / GENERATED_OUTPUTS[1]).read_text(encoding="utf-8")
            rust_ffi = (root / GENERATED_OUTPUTS[0]).read_text(encoding="utf-8")
            self.assertIn(
                "kmipkit_client_extension_registry_t ** extension_registry", public_header
            )
            self.assertIn(
                "extension_registry: *mut *mut kmipkit_client_extension_registry_t",
                rust_ffi,
            )
            self.assertIn(
                "kmipkit_byte_span_v1_t * out_limits", public_header
            )
            self.assertIn("struct kmipkit_byte_span_v1_t", public_header)
            self.assertIn("const uint8_t * data", public_header)

    def test_python_function_mapping_accepts_qualified_facade_methods(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            function = _find_function(manifest, "client_configuration_new")
            function["python"]["function"] = "Client.create"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertEqual(0, result.returncode, result.stderr)

    def test_consumed_handle_slot_and_output_value_pointer_reject_type_mismatch(self) -> None:
        mutations = (
            (
                "consumed handle slot type mismatch",
                {
                    "name": "extension_registry",
                    "kind": "consumed-handle-slot",
                    "type": "kmipkit_client_extension_registry_t **",
                    "handleType": "kmipkit_client_configuration_t",
                    "ownership": "consumed",
                },
            ),
            (
                "output value pointer type mismatch",
                {
                    "name": "out_limits",
                    "kind": "output-value-pointer",
                    "type": "kmipkit_extension_registry_limits_t *",
                    "valueType": "kmipkit_extension_registry_limits_v1_t",
                    "ownership": "none",
                },
            ),
        )
        for label, parameter in mutations:
            with self.subTest(case=label), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                function = _find_function(manifest, "client_configuration_new")
                function["c"]["parameters"] = [parameter]
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertNotEqual(result.returncode, 0, result.stderr)
                self.assertIn("generation failed", result.stderr)

    def test_approved_c_only_function_exclusion_keeps_exact_header_prototype(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            approval_source = root / "specs" / "043-public-api-bindings" / "spec.md"
            approval_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / "specs" / "043-public-api-bindings" / "spec.md", approval_source)
            manifest = self._load_manifest(root)
            symbol = "kmipkit_c_only_excluded_fixture"
            manifest["publicDeclarations"] = [
                declaration
                for declaration in manifest["publicDeclarations"]
                if not (
                    declaration.get("packageId") == "c-kmipkit"
                    and declaration.get("exportName") == symbol
                )
            ]
            manifest["approvedExclusions"].append(
                {
                    "packageId": "c-kmipkit",
                    "exportName": symbol,
                "reason": "The signature has no exact callable counterpart in the other facades.",
                    "approvalSource": {
                        "kind": "feature-specification",
                        "path": "specs/043-public-api-bindings/spec.md",
                        "status": "Approved",
                        "evidence": "**Status**: Approved for implementation",
                    },
                    "evidence": "The C-only function is retained in the public C ABI header with its exact prototype.",
                    "cFunction": {
                        "symbol": symbol,
                        "parameters": [
                            {
                                "name": "schema",
                                "kind": "handle",
                                "type": "kmipkit_extension_schema_t *",
                                "ownership": "borrowed",
                            },
                            {
                                "name": "out_count",
                                "kind": "output",
                                "type": "uint64_t *",
                                "ownership": "none",
                            },
                        ],
                        "returnType": "int32_t",
                        "errorCategories": ["invalid_input"],
                    },
                }
            )
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertEqual(0, result.returncode, result.stderr)
            public_header = (root / GENERATED_OUTPUTS[1]).read_text(encoding="utf-8")
            self.assertIn(
                "int32_t kmipkit_c_only_excluded_fixture(kmipkit_extension_schema_t * schema, uint64_t * out_count);",
                public_header,
            )

    def test_handle_arrays_require_known_types_counts_and_invalid_input_mappings(self) -> None:
        def missing_invalid_input(language: str):
            def mutate(manifest: dict[str, object]) -> None:
                _find_function(manifest, "extension_schema_structure")[language][
                    "errorCategories"
                ].remove("invalid_input")

            return mutate

        mutations = (
            (
                "unknown handle type",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update(
                    {"type": "kmipkit_unknown_handle_t **", "handleType": "kmipkit_unknown_handle_t"}
                ),
            ),
            (
                "array pointer does not match handle type",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update({"type": "kmipkit_extension_order_constraint_t **"}),
            ),
            (
                "unresolved count parameter",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update({"countParameter": "missing_count"}),
            ),
            (
                "count parameter is not uint64",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "child_count"
                ).update({"type": "uint32_t"}),
            ),
            *(
                (
                    f"{language} invalid_input mapping",
                    missing_invalid_input(language),
                )
                for _ in HANDLE_ARRAY_CONTRACTS[:1]
                for language in ("c", "java", "python")
            ),
        )
        for label, mutate in mutations:
            with self.subTest(case=label), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                _apply_handle_array_contract(manifest)
                mutate(manifest)
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_input_arrays_require_supported_types_counts_and_invalid_input_mappings(self) -> None:
        def missing_invalid_input(language: str):
            def mutate(manifest: dict[str, object]) -> None:
                _find_function(manifest, "extension_schema_structure")[language][
                    "errorCategories"
                ].remove("invalid_input")

            return mutate

        mutations = (
            (
                "unknown input element type",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update(
                    {
                        "elementType": "kmipkit_unknown_value_t",
                        "type": "const kmipkit_unknown_value_t *",
                    }
                ),
            ),
            (
                "array pointer does not match element type",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update({"type": "const uint64_t *"}),
            ),
            (
                "unresolved count parameter",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "children"
                ).update({"countParameter": "missing_count"}),
            ),
            (
                "count parameter is not uint64",
                lambda manifest: _find_c_parameter(
                    manifest, "extension_schema_structure", "child_count"
                ).update({"type": "uint32_t"}),
            ),
            *(
                (f"{language} invalid_input mapping", missing_invalid_input(language))
                for language in ("c", "java", "python")
            ),
        )
        for label, mutate in mutations:
            with self.subTest(case=label), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                _apply_input_array_contract(manifest)
                mutate(manifest)
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_duplicate_c_parameter_names_are_rejected_before_span_resolution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            function = next(item for item in manifest["functions"] if item["id"] == "extension_identity_create")
            parameters = function["c"]["parameters"]
            length = next(item for item in parameters if item["name"] == "name_length")
            length["name"] = "vendor_identifier_length"
            name_span = next(item for item in parameters if item["name"] == "name_data")
            name_span["byteLengthParameter"] = "vendor_identifier_length"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_rust_2024_keywords_are_raw_escaped_and_unescapable_names_rejected(self) -> None:
        for keyword in ("const", "false", "abstract", "gen"):
            with self.subTest(keyword=keyword), tempfile.TemporaryDirectory() as directory:
                root = _create_repo(Path(directory))
                manifest = self._load_manifest(root)
                function = next(item for item in manifest["functions"] if item["id"] == "ttlv_value_integer")
                function["c"]["parameters"][0]["name"] = keyword
                self._write_manifest(root, manifest)

                result = self._run_generator(root, "--write")

                self.assertEqual(result.returncode, 0, result.stderr)
                rust_output = (root / GENERATED_OUTPUTS[0]).read_text(encoding="utf-8")
                self.assertIn(f"r#{keyword}: ", rust_output)

        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            function = next(item for item in manifest["functions"] if item["id"] == "ttlv_value_integer")
            function["c"]["parameters"][0]["name"] = "self"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_registry_parity_fixture_preserves_the_v1_registry_slice(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            result = self._run_generator(root, "--write")
            self.assertEqual(result.returncode, 0, result.stderr)

            parity = json.loads((root / GENERATED_OUTPUTS[5]).read_text(encoding="utf-8"))

            expected = json.loads(PUBLIC_API_V1_FIXTURE.read_text(encoding="utf-8"))
            self.assertEqual(parity, expected)

    def test_v2_c_ownership_docs_do_not_leak_into_v1_registry_projection(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        projection = generate._registry_manifest_v2(manifest)
        projected_functions = {
            function["id"]: function for function in projection["functions"]
        }
        for function_id in (
            "ttlv_value_structure",
            "ttlv_item_create",
            "ttlv_structure_with_item",
        ):
            with self.subTest(function_id=function_id):
                self.assertNotIn("documentation", projected_functions[function_id]["c"])

    def test_registry_parity_excludes_v2_only_error_categories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            result = self._run_generator(root, "--write")
            self.assertEqual(result.returncode, 0, result.stderr)

            parity = json.loads((root / GENERATED_OUTPUTS[5]).read_text(encoding="utf-8"))
            expected = json.loads(PUBLIC_API_V1_FIXTURE.read_text(encoding="utf-8"))

            self.assertEqual(parity["errorCategories"], expected["errorCategories"])

    def test_check_mode_compares_outputs_without_rewriting_them(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            write_result = self._run_generator(root, "--write")
            self.assertEqual(write_result.returncode, 0, write_result.stderr)
            output_paths = [root / path for path in V2_GENERATED_OUTPUTS]
            before = {path: path.read_bytes() for path in output_paths}
            timestamp_ns = 946_684_800_000_000_000
            for path in output_paths:
                os.utime(path, ns=(timestamp_ns, timestamp_ns))
            timestamps = {path: path.stat().st_mtime_ns for path in output_paths}

            result = self._run_generator(root, "--check")

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual({path: path.read_bytes() for path in output_paths}, before)
            self.assertEqual({path: path.stat().st_mtime_ns for path in output_paths}, timestamps)

    def test_check_mode_reports_stale_output_without_rewriting_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            write_result = self._run_generator(root, "--write")
            self.assertEqual(write_result.returncode, 0, write_result.stderr)
            stale_output = root / GENERATED_OUTPUTS[0]
            sentinel = b"stale generated output\n"
            stale_output.write_bytes(sentinel)
            timestamp_ns = 946_684_800_000_000_000
            os.utime(stale_output, ns=(timestamp_ns, timestamp_ns))
            expected_mtime_ns = stale_output.stat().st_mtime_ns

            result = self._run_generator(root, "--check")

            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(stale_output.read_bytes(), sentinel)
            self.assertEqual(stale_output.stat().st_mtime_ns, expected_mtime_ns)

    def test_require_complete_is_only_valid_with_check_mode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))

            result = self._run_generator(root, "--write", "--require-complete")

            self.assertEqual(result.returncode, 2)
            self.assertIn("--require-complete can only be used with --check", result.stderr)
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_check_require_complete_runs_release_gate_on_checked_in_manifest(self) -> None:
        output_before = (ROOT / V2_GENERATED_OUTPUTS[0]).read_bytes()

        result = self._run_generator(
            ROOT, "--repo-root", str(ROOT), "--check", "--require-complete"
        )

        self.assertEqual(result.returncode, 2)
        self.assertEqual(
            result.stderr,
            "registry API generation failed: manifest or path validation failed\n",
        )
        self.assertEqual((ROOT / V2_GENERATED_OUTPUTS[0]).read_bytes(), output_before)

    def test_check_require_complete_validates_complete_fixture_and_runs_release_audits(
        self,
    ) -> None:
        from tools.api_manifest.tests.test_operation_readiness import (
            _complete_readiness_manifest,
            _write_dependency_matrix_fixture_if_missing,
        )

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
            api_directory = root / "specification" / "api"
            api_directory.mkdir(parents=True)
            shutil.copy2(SCHEMA, api_directory / SCHEMA.name)
            catalog_path = root / "specification" / "catalog" / CATALOG.name
            catalog_path.parent.mkdir(parents=True)
            shutil.copy2(CATALOG, catalog_path)

            for relative_path in version_paths:
                destination = root / relative_path
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative_path, destination)

            manifest = _complete_readiness_manifest(root)
            _write_dependency_matrix_fixture_if_missing(manifest, root)
            for package in manifest["publicPackages"]:
                package["versionSource"] = "releaseLine"

            error_declarations: list[dict[str, object]] = []
            error_exports: dict[str, dict[str, str]] = {}
            package_ids = {
                "rust": "rust-kmipkit-protocol",
                "c": "c-kmipkit",
                "java": "java-kmipkit",
                "python": "python-kmipkit",
            }
            for error in manifest["errorCategories"]:
                error_id = error["id"]
                name = "".join(part.title() for part in error_id.split("_"))
                exports = {
                    "rust": f"kmipkit_protocol::fixture::Error::{name}",
                    "c": f"KMIPKIT_ERROR_{error_id.upper()}",
                    "java": f"org.kmipkit.fixture.{name}Exception",
                    "python": f"kmipkit.errors.fixture_{error_id}_error",
                }
                error["cCode"] = exports["c"]
                error["javaException"] = exports["java"]
                error["pythonException"] = exports["python"]
                error_exports[error_id] = exports
                for language, export_name in exports.items():
                    error_declarations.append(
                        {
                            "declarationId": f"strict_fixture_error_{error_id}_{language}",
                            "packageId": package_ids[language],
                            "kind": {
                                "rust": "associated-item",
                                "c": "constant",
                                "java": "type",
                                "python": "type",
                            }[language],
                            "exportName": export_name,
                            "capabilityId": "public_binding_surface",
                            "requirementIds": ["KMIPKIT-0043-FR-001"],
                            "declarationState": "verified",
                            "parityScope": "four-language",
                            "approvalSource": {
                                "kind": "feature-specification",
                                "path": "specs/007-client-execution/approval-record.md",
                                "status": "Approved",
                                "evidence": "Maintainer acceptance for release 1.0.0",
                            },
                            "languageMappings": {
                                mapped_language: {
                                    "packageId": package_ids[mapped_language],
                                    "name": mapped_export,
                                }
                                for mapped_language, mapped_export in exports.items()
                            },
                            "parityEvidenceIds": ["tests/fixtures/api/parity.json"],
                        }
                    )
            manifest["publicDeclarations"].extend(error_declarations)
            public_binding_surface = next(
                capability
                for capability in manifest["capabilities"]
                if capability["capabilityId"] == "public_binding_surface"
            )
            for language in package_ids:
                public_binding_surface["mappings"][language] = sorted(
                    [
                        *public_binding_surface["mappings"][language],
                        *(exports[language] for exports in error_exports.values()),
                    ]
                )
            (api_directory / "public-api.json").write_text(
                json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
            )

            write_result = generate.main(["--repo-root", str(root), "--write"])
            self.assertEqual(write_result, 0)
            output_paths = [root / path for path in V2_GENERATED_OUTPUTS]
            before_check = {
                path: (path.read_bytes(), path.stat().st_mtime_ns)
                for path in output_paths
            }

            with mock.patch(
                "tools.api_manifest.generate.subprocess.run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as run_audits:
                result = generate.main(
                    [
                        "--repo-root",
                        str(root),
                        "--check",
                        "--require-complete",
                    ]
                )

            self.assertEqual(result, 0)
            audit_commands = [call.args[0] for call in run_audits.call_args_list]
            self.assertEqual(len(audit_commands), 2)
            self.assertEqual(
                {Path(command[2]).name for command in audit_commands},
                {"audit_rust_exports.py", "audit_binding_exports.py"},
            )
            self.assertTrue(all(command[-1] == "--check" for command in audit_commands))
            self.assertEqual(
                {
                    path: (path.read_bytes(), path.stat().st_mtime_ns)
                    for path in output_paths
                },
                before_check,
            )

    def test_rejects_malformed_manifest_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            (root / "specification" / "api" / "public-api.json").write_text("{broken\n", encoding="utf-8")

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_rejects_unknown_required_manifest_field_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            manifest["generatorRequiredShortcut"] = True
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_rejects_traversal_output_destination_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            manifest = self._load_manifest(root)
            outputs = manifest["generatedOutputs"]
            self.assertIsInstance(outputs, list)
            c_header = next(output for output in outputs if output["name"] == "cHeader")
            c_header["path"] = "../escaped-kmipkit.h"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((parent / "escaped-kmipkit.h").exists())
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))

    def test_rejects_symlinked_output_file_without_modifying_its_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            external_file = parent / "outside.h"
            sentinel = b"external sentinel\n"
            external_file.write_bytes(sentinel)
            link = root / GENERATED_OUTPUTS[1]
            link.parent.mkdir(parents=True)
            self._assert_symlink_creation_or_skip(link, external_file, directory=False)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(external_file.read_bytes(), sentinel)
            self.assertFalse(
                any((root / path).exists() for path in V2_GENERATED_OUTPUTS if path != GENERATED_OUTPUTS[1])
            )

    def test_rejects_symlinked_output_parent_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            external_directory = parent / "outside-bindings"
            external_directory.mkdir()
            bindings = root / "bindings"
            if bindings.is_dir():
                shutil.rmtree(bindings)
            self._assert_symlink_creation_or_skip(bindings, external_directory, directory=True)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("symbolic link or reparse point", result.stderr)
            external_outputs = (
                external_directory / Path(path).relative_to("bindings")
                for path in V2_GENERATED_OUTPUTS
                if path.startswith("bindings/")
            )
            self.assertFalse(any(path.exists() for path in external_outputs))
            self.assertFalse(any((root / path).exists() for path in V2_GENERATED_OUTPUTS))


class ManifestSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        legacy_schema = {
            **cls.schema["$defs"]["registrySliceV1"],
            "$defs": cls.schema["$defs"],
        }
        cls.validator = Draft202012Validator(legacy_schema)

    def _load_fixture(self) -> dict[str, object]:
        return json.loads(MINIMAL_FIXTURE.read_text(encoding="utf-8"))

    def _assert_schema_rejects(
        self,
        manifest: dict[str, object],
        validator_name: str | None,
        expected_path: tuple[str | int, ...] | None = None,
    ) -> None:
        errors = list(self.validator.iter_errors(manifest))
        self.assertTrue(errors, "expected the mutated manifest to fail schema validation")
        if validator_name is not None:
            self.assertTrue(
                any(error.validator == validator_name for error in errors),
                f"expected a {validator_name} schema error, got {[error.message for error in errors]}",
            )
        if expected_path is not None:
            self.assertTrue(
                any(tuple(error.absolute_path) == expected_path for error in errors),
                f"expected a schema error at {expected_path}, got {[list(error.absolute_path) for error in errors]}",
            )

    def _run_generator(self, root: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
        self.assertTrue(
            GENERATOR.is_file(),
            "manifest generator CLI must be implemented before format validation can be exercised",
        )
        return subprocess.run(
            [sys.executable, "-B", str(GENERATOR), *arguments],
            cwd=root,
            capture_output=True,
            check=False,
            text=True,
        )

    def _assert_cli_rejects_before_output(self, manifest: dict[str, object]) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_fixture_repo(Path(directory))
            (root / "specification" / "api" / "public-api.json").write_text(
                json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_schema_is_valid_draft_2020_12(self) -> None:
        Draft202012Validator.check_schema(self.schema)

    def test_minimal_registry_fixture_validates(self) -> None:
        manifest = self._load_fixture()
        errors = list(self.validator.iter_errors(manifest))
        self.assertEqual(errors, [])
        self.assertEqual(len(manifest["types"]), 1)
        self.assertEqual(len(manifest["functions"]), 1)
        self.assertEqual(len(manifest["errorCategories"]), 1)
        self.assertEqual(tuple(limit["id"] for limit in manifest["limits"]), LIMIT_IDS)
        self.assertEqual(manifest["generatedOutputs"], GENERATED_OUTPUT_MAP)

    def test_schema_rejects_missing_required_root_property(self) -> None:
        manifest = self._load_fixture()
        del manifest["security"]

        self._assert_schema_rejects(manifest, "required")

    def test_schema_rejects_unknown_nested_property(self) -> None:
        manifest = self._load_fixture()
        manifest["types"][0]["rust"]["unexpectedModuleAlias"] = "wrong"

        self._assert_schema_rejects(manifest, "additionalProperties")

    def test_schema_rejects_mismatched_enum_shape_and_unknown_policy(self) -> None:
        manifest = self._load_fixture()
        declaration = manifest["types"][0]
        declaration["kind"] = "enum"
        declaration["c"] = {
            "name": "kmipkit_minimal_type_t",
            "kind": "enum",
            "underlyingType": "uint8_t",
            "unknownValuePolicy": "preserve-raw",
        }
        declaration["enumRepresentation"] = {
            "shape": "closed-numeric",
            "unknownValuePolicy": "preserve-raw",
            "values": [{"name": "Known", "value": 1}],
        }

        self._assert_schema_rejects(manifest, "const")

    def test_schema_rejects_invalid_rust_java_and_python_signatures(self) -> None:
        mutations = (
            ("Rust", "rust", "returnType", "Result<>"),
            ("Java", "java", "returnType", "List<int>"),
            ("Python", "python", "returnType", "list[]"),
        )
        for language_name, language, field, invalid_type in mutations:
            with self.subTest(language=language_name):
                manifest = self._load_fixture()
                manifest["functions"][0][language][field] = invalid_type
                self._assert_schema_rejects(
                    manifest,
                    None,
                    ("functions", 0, language, field),
                )

    def test_schema_accepts_borrowed_value_inside_optional_rust_result(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["rust"]["returnType"] = (
            "Option<&ValidatedExtensionValue>"
        )

        self.assertEqual(list(self.validator.iter_errors(manifest)), [])

    def test_schema_accepts_optional_java_byte_array_return(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["java"]["returnType"] = "Optional<byte[]>"

        self.assertEqual(list(self.validator.iter_errors(manifest)), [])

    def test_schema_accepts_optional_java_protocol_version_list(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["java"]["returnType"] = (
            "Optional<List<ProtocolVersion>>"
        )

        self.assertEqual(list(self.validator.iter_errors(manifest)), [])

    def test_schema_rejects_limit_values_outside_the_approved_profile(self) -> None:
        manifest = self._load_fixture()
        manifest["limits"][0]["hardMaximum"] = 1025

        self._assert_schema_rejects(manifest, "const")

    def test_semantic_reference_cases_pass_schema_validation(self) -> None:
        cases: list[tuple[str, dict[str, object]]] = []

        byte_span = self._load_fixture()
        byte_span["functions"][0]["c"]["parameters"] = [
            {
                "name": "bytes",
                "kind": "byte-span",
                "type": "const uint8_t *",
                "encoding": "octets",
                "byteLengthParameter": "missing_length",
                "byteLengthType": "uint64_t",
                "lengthUnit": "bytes",
                "limitCheck": "before-dereference",
                "nulTerminatedScan": False,
            },
            {"name": "actual_length", "kind": "scalar", "type": "uint64_t"},
            {"name": "out_value", "kind": "output", "type": "uint32_t *", "ownership": "none"},
        ]
        cases.append(("unresolved byte-span length", byte_span))

        requirement = self._load_fixture()
        requirement["functions"][0]["requirementIds"] = ["KMIPKIT-0012-FR-999"]
        cases.append(("unresolved requirement ID", requirement))

        category = self._load_fixture()
        category["functions"][0]["c"]["errorCategories"] = ["unresolved_error"]
        cases.append(("unresolved error category", category))

        for label, manifest in cases:
            with self.subTest(reference=label):
                self.assertEqual(list(self.validator.iter_errors(manifest)), [])

    def test_cli_rejects_format_invalid_manifests_before_writing_outputs(self) -> None:
        mutations = (
            ("missing required property", lambda value: value.pop("security")),
            (
                "nested unknown property",
                lambda value: value["types"][0]["rust"].update({"unexpectedModuleAlias": "wrong"}),
            ),
            (
                "enum shape policy mismatch",
                lambda value: value["types"][0].update(
                    {
                        "kind": "enum",
                        "c": {
                            "name": "kmipkit_minimal_type_t",
                            "kind": "enum",
                            "underlyingType": "uint8_t",
                            "unknownValuePolicy": "preserve-raw",
                        },
                        "enumRepresentation": {
                            "shape": "closed-numeric",
                            "unknownValuePolicy": "preserve-raw",
                            "values": [{"name": "Known", "value": 1}],
                        },
                    }
                ),
            ),
            (
                "invalid Rust signature",
                lambda value: value["functions"][0]["rust"].update({"returnType": "Result<>"}),
            ),
            (
                "invalid Java signature",
                lambda value: value["functions"][0]["java"].update({"returnType": "List<int>"}),
            ),
            (
                "invalid Python signature",
                lambda value: value["functions"][0]["python"].update({"returnType": "list[]"}),
            ),
            ("invalid limit", lambda value: value["limits"][0].update({"hardMaximum": 1025})),
        )
        for label, mutate in mutations:
            with self.subTest(case=label):
                manifest = self._load_fixture()
                mutate(manifest)
                self._assert_cli_rejects_before_output(manifest)

    def test_cli_rejects_unresolved_byte_span_length_before_writing_outputs(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["c"]["parameters"] = [
            {
                "name": "bytes",
                "kind": "byte-span",
                "type": "const uint8_t *",
                "encoding": "octets",
                "byteLengthParameter": "missing_length",
                "byteLengthType": "uint64_t",
                "lengthUnit": "bytes",
                "limitCheck": "before-dereference",
                "nulTerminatedScan": False,
            },
            {"name": "actual_length", "kind": "scalar", "type": "uint64_t"},
            {"name": "out_value", "kind": "output", "type": "uint32_t *", "ownership": "none"},
        ]

        self._assert_cli_rejects_before_output(manifest)

    def test_cli_rejects_unresolved_requirement_reference_before_writing_outputs(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["requirementIds"] = ["KMIPKIT-0012-FR-999"]

        self._assert_cli_rejects_before_output(manifest)

    def test_cli_rejects_unresolved_error_reference_before_writing_outputs(self) -> None:
        manifest = self._load_fixture()
        manifest["functions"][0]["c"]["errorCategories"] = ["unresolved_error"]

        self._assert_cli_rejects_before_output(manifest)


if __name__ == "__main__":
    unittest.main()
