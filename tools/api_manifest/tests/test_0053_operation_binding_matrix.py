"""Cross-language route matrix for the KMIP 2.1 client operation catalog."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class OperationBindingMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(_read(MANIFEST))
        cls.operations = [
            operation
            for operation in cls.manifest["operations"]
            if operation.get("scope") == "client_1_0"
        ]
        cls.declarations = {
            declaration["declarationId"]: declaration
            for declaration in cls.manifest["publicDeclarations"]
        }
        cls.c_header = _read(ROOT / "bindings" / "c" / "include" / "kmipkit.h")
        cls.java_source = "\n".join(
            _read(path)
            for path in (ROOT / "bindings" / "java" / "src" / "main" / "java").rglob("*.java")
        )
        cls.python_source = "\n".join(
            _read(path)
            for path in (ROOT / "bindings" / "python" / "src" / "kmipkit").rglob("*.py")
        )
        cls.rust_source = "\n".join(
            _read(path)
            for crate in ("kmipkit", "kmipkit-client", "kmipkit-protocol")
            for path in (ROOT / "crates" / crate / "src").rglob("*.rs")
        )

    def test_catalog_has_exactly_56_verified_client_initiated_operations(self) -> None:
        self.assertEqual(len(self.operations), 56)
        self.assertEqual(
            len({operation["operationId"] for operation in self.operations}), 56
        )
        unverified = [
            operation["name"]
            for operation in self.operations
            if operation.get("apiState") != "verified"
        ]
        self.assertEqual(unverified, [], f"operation evidence is incomplete: {unverified}")

    def test_every_operation_maps_routes_in_four_languages(self) -> None:
        languages = {"rust", "c", "java", "python"}
        for operation in self.operations:
            with self.subTest(operation=operation["name"]):
                declarations = operation["declarations"]
                self.assertEqual(set(declarations), languages)
                self.assertTrue(all(declarations[language] for language in languages))
                mapped = []
                for language, declaration_ids in declarations.items():
                    for declaration_id in declaration_ids:
                        declaration = self.declarations.get(declaration_id)
                        self.assertIsNotNone(declaration, declaration_id)
                        assert declaration is not None
                        self.assertIn(language, declaration["languageMappings"])
                        self.assertIn(
                            declaration["declarationState"], {"declared", "verified"}
                        )
                        mapped.append(declaration)
                self.assertTrue(
                    any(
                        operation["operationId"] == declaration["capabilityId"]
                        for declaration in mapped
                    )
                    or (
                        operation["operationId"].endswith(("-CANCEL", "-POLL", "-PROCESS"))
                        and any(
                            declaration["capabilityId"] == "client_asynchronous_operations"
                            for declaration in mapped
                        )
                    ),
                    f"missing operation-specific routes for {operation['name']}",
                )

    def test_every_operation_has_executable_evidence_in_all_four_languages(self) -> None:
        language_roots = {
            "rust": ("crates/",),
            "c": ("bindings/c/tests/",),
            "java": ("bindings/java/",),
            "python": ("bindings/python/",),
        }
        for operation in self.operations:
            evidence = operation.get("parityEvidenceIds", [])
            with self.subTest(operation=operation["name"]):
                self.assertTrue(evidence, "operation has no behavior evidence")
                self.assertTrue(
                    all((ROOT / path).is_file() for path in evidence),
                    "operation evidence references a missing file",
                )
                for language, roots in language_roots.items():
                    self.assertTrue(
                        any(path.startswith(roots) for path in evidence),
                        f"{operation['name']} has no {language} behavior evidence",
                    )

    def test_runtime_operation_identity_and_response_matrices_cover_all_56(self) -> None:
        c_source = _read(ROOT / "bindings" / "c" / "tests" / "operation_parity_consumer.c")
        c_test = _read(ROOT / "crates" / "kmipkit-ffi" / "tests" / "operation_parity_0053.rs")
        java_test = _read(
            ROOT / "bindings" / "java" / "src" / "test" / "java" / "org" / "kmipkit"
            / "extensions" / "OperationFacadeParityMatrixTest.java"
        )
        python_test = _read(
            ROOT / "bindings" / "python" / "tests" / "test_operation_binding_parity_0053.py"
        )

        # The C fixture compares each encoded request operation to the pinned
        # KMIP operation values. The 48 generic batch constructors plus four
        # existing specialized constructors and four explicit async operations
        # account for the complete 56-operation client catalog.
        expected_values = c_source.split("static const uint32_t EXPECTED_BATCH_OPERATIONS[] = {", 1)[1]
        expected_values = expected_values.split("};", 1)[0]
        self.assertEqual(expected_values.count("UINT32_C("), 48)
        self.assertIn("invoked_count != UINT64_C(52)", c_source)
        self.assertIn("async_operation_routes_are_callable()", c_source)
        self.assertIn("compiled_c_consumer_invokes_all_56_client_operation_routes", c_test)

        # Java crosses the JNI ABI for each builder, then checks the raw
        # operation identity returned by Rust against the generated KMIP 2.1
        # Operation constants. Its response list covers the same operation set.
        self.assertIn("assertEquals(56, exercised.size())", java_test)
        self.assertIn(
            "assertEquals(operationRaw(route.operation()), readBack.operation()",
            java_test,
        )
        self.assertEqual(java_test.count('result("'), 52)
        self.assertEqual(java_test.count("new AsyncResponseAccessor("), 4)
        self.assertIn("assertEquals(exercisedOperations, allResponseOperations)", java_test)

        # Python checks the precise C ABI entrypoint used by each public builder
        # and keeps async dispatch separate from synchronous batch operations.
        self.assertIn("test_request_entrypoint_matrix_accounts_for_all_56_operations", python_test)
        self.assertIn("set(_BATCH_OPERATION_CASES) | set(_ASYNC_OPERATION_CASES)", python_test)
        self.assertIn("test_batch_operation_builder_invokes_its_native_request_entrypoint", python_test)
        self.assertIn("test_async_operation_entrypoint_calls_native_method", python_test)

    def test_mapped_foreign_route_names_exist_in_public_source_surfaces(self) -> None:
        for operation in self.operations:
            for language in ("rust", "c", "java", "python"):
                with self.subTest(operation=operation["name"], language=language):
                    for declaration_id in operation["declarations"][language]:
                        declaration = self.declarations[declaration_id]
                        mapping = declaration["languageMappings"][language]
                        name = mapping["name"]
                        member = name.rsplit(".", maxsplit=1)[-1]
                        if language == "rust":
                            member = name.rsplit("::", maxsplit=1)[-1]
                            self.assertIn(member, self.rust_source, declaration_id)
                        elif language == "c":
                            components = name.split(".")
                            self.assertTrue(
                                all(component in self.c_header for component in components),
                                declaration_id,
                            )
                        elif language == "java":
                            self.assertIn(member, self.java_source, declaration_id)
                        else:
                            self.assertIn(member, self.python_source, declaration_id)


if __name__ == "__main__":
    unittest.main()
