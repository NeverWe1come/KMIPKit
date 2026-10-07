"""Behavioral tests for the public API manifest generator CLI."""

from __future__ import annotations

import errno
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[3]
GENERATOR = ROOT / "tools" / "api_manifest" / "generate.py"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
SCHEMA = ROOT / "specification" / "api" / "public-api.schema.json"
MINIMAL_FIXTURE = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "registry-manifest.json"
GENERATED_OUTPUTS = (
    "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "bindings/c/include/kmipkit.h",
    "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "tests/fixtures/extensions/generated/registry_parity.json",
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


class ManifestGeneratorCliTests(unittest.TestCase):
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
                for relative_path in GENERATED_OUTPUTS
            }
            self.assertTrue(all(first_outputs.values()))

            second = self._run_generator(root, "--write")

            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(
                {relative_path: (root / relative_path).read_bytes() for relative_path in GENERATED_OUTPUTS},
                first_outputs,
            )

    def test_object_key_order_does_not_change_generated_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            first = self._run_generator(root, "--write")
            self.assertEqual(first.returncode, 0, first.stderr)
            first_outputs = {
                relative_path: (root / relative_path).read_bytes()
                for relative_path in GENERATED_OUTPUTS
            }
            manifest = self._load_manifest(root)
            reversed_manifest = _reverse_object_keys(manifest)
            self.assertIsInstance(reversed_manifest, dict)
            self._write_manifest(root, reversed_manifest)

            second = self._run_generator(root, "--write")

            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(
                {relative_path: (root / relative_path).read_bytes() for relative_path in GENERATED_OUTPUTS},
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
                self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

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
                self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

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
                self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

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
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

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
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_parity_fixture_preserves_complete_manifest_mappings(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            result = self._run_generator(root, "--write")
            self.assertEqual(result.returncode, 0, result.stderr)

            parity = json.loads((root / GENERATED_OUTPUTS[5]).read_text(encoding="utf-8"))

            self.assertEqual(parity, manifest)

    def test_check_mode_compares_outputs_without_rewriting_them(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            write_result = self._run_generator(root, "--write")
            self.assertEqual(write_result.returncode, 0, write_result.stderr)
            output_paths = [root / path for path in GENERATED_OUTPUTS]
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

    def test_rejects_malformed_manifest_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            (root / "specification" / "api" / "public-api.json").write_text("{broken\n", encoding="utf-8")

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_rejects_unknown_required_manifest_field_without_writing_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = _create_repo(Path(directory))
            manifest = self._load_manifest(root)
            manifest["generatorRequiredShortcut"] = True
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

    def test_rejects_traversal_output_destination_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            manifest = self._load_manifest(root)
            outputs = manifest["generatedOutputs"]
            self.assertIsInstance(outputs, dict)
            outputs["cHeader"] = "../escaped-kmipkit.h"
            self._write_manifest(root, manifest)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((parent / "escaped-kmipkit.h").exists())
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))

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
                any((root / path).exists() for path in GENERATED_OUTPUTS if path != GENERATED_OUTPUTS[1])
            )

    def test_rejects_symlinked_output_parent_without_writing_outside_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = _create_repo(parent)
            external_directory = parent / "outside-bindings"
            external_directory.mkdir()
            bindings = root / "bindings"
            self._assert_symlink_creation_or_skip(bindings, external_directory, directory=True)

            result = self._run_generator(root, "--write")

            self.assertNotEqual(result.returncode, 0)
            external_outputs = (
                external_directory / Path(path).relative_to("bindings")
                for path in GENERATED_OUTPUTS
                if path.startswith("bindings/")
            )
            self.assertFalse(any(path.exists() for path in external_outputs))
            self.assertFalse(any((root / path).exists() for path in GENERATED_OUTPUTS))


class ManifestSchemaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        cls.validator = Draft202012Validator(cls.schema)

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
