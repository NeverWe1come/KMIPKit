"""Cross-language parity invariants for the reviewed public API manifest."""

from __future__ import annotations

import ast
import json
import re
import unittest
from pathlib import Path
from typing import Any

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REGISTRY_MANIFEST = ROOT / "tools" / "api_manifest" / "tests" / "fixtures" / "public-api-v1.json"
GENERATED_OUTPUTS = {
    "rustFfi": "crates/kmipkit-ffi/src/extension_registry_generated.rs",
    "cHeader": "bindings/c/include/kmipkit.h",
    "javaApi": "bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java",
    "javaParityTests": "bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java",
    "pythonApi": "bindings/python/src/kmipkit/_generated/extension_registry.py",
    "parityFixtures": "tests/fixtures/extensions/generated/registry_parity.json",
}
ALL_GENERATED_OUTPUTS = [
    {"name": "registryRustFfi", "path": GENERATED_OUTPUTS["rustFfi"]},
    {"name": "cHeader", "path": GENERATED_OUTPUTS["cHeader"]},
    {"name": "registryJavaApi", "path": GENERATED_OUTPUTS["javaApi"]},
    {"name": "registryJavaParityTests", "path": GENERATED_OUTPUTS["javaParityTests"]},
    {"name": "registryPythonApi", "path": GENERATED_OUTPUTS["pythonApi"]},
    {"name": "registryParityFixtures", "path": GENERATED_OUTPUTS["parityFixtures"]},
    {"name": "rustFfi", "path": "crates/kmipkit-ffi/src/public_api_generated.rs"},
    {"name": "javaApi", "path": "bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java"},
    {"name": "javaParityTests", "path": "bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java"},
    {"name": "pythonApi", "path": "bindings/python/src/kmipkit/_generated/public_api.py"},
    {"name": "parityFixtures", "path": "tests/fixtures/api/generated/public_api_parity.json"},
    {"name": "internalCHeader", "path": "tests/fixtures/api/generated/kmipkit_internal.h"},
]
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
LOGIN_TTLV_AGGREGATE_JNI_SYMBOL = (
    "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLogin"
)
REQUIREMENT_IDS = tuple(f"KMIPKIT-0012-FR-{number:03}" for number in range(1, 15))
SHARED_PARITY_FIXTURE = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
SHARED_PARITY_CASES = {
    "api-levels.shared.v1",
    "client-configuration.registry-isolation.v1",
    "errors.schema-redaction.v1",
    "batch.ordered-completed-pending-failed.v1",
    "discover-versions.supported-versions.v1",
    "ttlv.open-values-and-child-order.v1",
    "extensions.recognition-and-generic-preservation.v1",
    "creation.create.request-response.v1",
    "creation.key-pair.request-response.v1",
    "creation.split-key.request-response.v1",
    "creation.result-message-on-failure.v1",
    "codec-limits.defaults-and-ranges.v1",
    "usage-allocation.guard-accounting.v1",
}
SHARED_PARITY_LANGUAGES = ["rust", "c", "java", "python"]
T029_CORE_FIXTURE_ASSERTIONS = {
    "api-levels.shared.v1": ("apiLevels", "networkCalls", "operationMappingCompleteness"),
    "errors.schema-redaction.v1": (
        "diagnosticMarker",
        "diagnosticOmitsInputMarker",
        "diagnosticContainsPayload",
    ),
    "batch.ordered-completed-pending-failed.v1": (
        "resultStatuses",
        "uniqueBatchItemIds",
        "pendingCount",
        "correlationValueHex",
        "transportExchangeCount",
    ),
    "ttlv.open-values-and-child-order.v1": (
        "childTagsInOrder",
        "enumerationValue",
        "byteStringHex",
        "unknownValuesPreserved",
    ),
    "codec-limits.defaults-and-ranges.v1": (
        "maxMessageBytes",
        "maxStructureDepth",
        "maxElements",
        "errorCategory",
    ),
    "usage-allocation.guard-accounting.v1": (
        "allocationCount",
        "successfulCost",
        "overBudgetCost",
        "afterSuccessfulProtection",
        "transportExchangeCount",
    ),
    "client-configuration.registry-isolation.v1": (
        "configurationDefinitionCounts",
        "configurationIdentities",
        "registryStateIsShared",
        "close-configuration-owned-native-handles-once",
        "release-returned-registry-views-once",
    ),
}
T029_C_HARNESS_TESTS = {
    "client-configuration.registry-isolation.v1": (
        "c_abi_consumer_matches_shared_registry_cleanup_fixture",
        "shared_registry_fixture_matches_cleanup_and_isolation",
    ),
    "api-levels.shared.v1": (
        "generated_public_client_abi_consumer_compiles_and_runs",
        "kmipkit_client_abi_c_consumer_main",
    ),
    "errors.schema-redaction.v1": (
        "c_abi_consumer_redacts_shared_fixture_diagnostic_marker",
        "shared_schema_redaction_fixture_matches_fixture",
    ),
    "batch.ordered-completed-pending-failed.v1": (
        "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        "kmipkit_client_abi_c_consumer_execute_tls",
    ),
    "ttlv.open-values-and-child-order.v1": (
        "c_abi_consumer_preserves_shared_ttlv_fixture_values_and_order",
        "shared_ttlv_fixture_preserves_open_values_and_order",
    ),
    "codec-limits.defaults-and-ranges.v1": (
        "c_abi_consumer_matches_the_shared_codec_limits_fixture",
        "codec_limits_shared_fixture_matches_defaults_ranges_and_errors",
    ),
    "usage-allocation.guard-accounting.v1": (
        "ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost",
        "shared_usage_allocation_guard_matches_fixture",
    ),
}
JAVA_TEST_ROOT = ROOT / "bindings" / "java" / "src" / "test" / "java"
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
        self.assertTrue(set(REQUIREMENT_IDS).issubset(self.manifest["requirementIds"]))

        registry_records = [
            item
            for item in [*self.manifest["types"], *self.manifest["functions"]]
            if any(
                requirement.startswith("KMIPKIT-0012-")
                for requirement in item["requirementIds"]
            )
        ]
        for item in registry_records:
            registry_requirements = {
                requirement for requirement in item["requirementIds"]
                if requirement.startswith("KMIPKIT-0012-")
            }
            self.assertTrue(registry_requirements.issubset(REQUIREMENT_IDS), item["id"])

    def test_shared_capability_fixture_maps_executable_consumer_contracts(self) -> None:
        self.assertTrue(
            SHARED_PARITY_FIXTURE.is_file(),
            "shared API capability fixture corpus is missing",
        )
        fixtures = json.loads(SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))

        self.assertEqual(fixtures["format"], "kmipkit-shared-api-parity-fixtures")
        self.assertEqual(fixtures["formatVersion"], 2)
        self.assertEqual(fixtures["releaseLine"], "1.0.0")
        self.assertEqual(fixtures["parityLanguages"], SHARED_PARITY_LANGUAGES)
        self.assertEqual(fixtures["plannedConsumerLanguages"], [])
        self.assertEqual(
            fixtures["executableConsumerLanguages"], SHARED_PARITY_LANGUAGES
        )

        cases = fixtures["cases"]
        case_ids = [case["id"] for case in cases]
        self.assertEqual(len(case_ids), len(set(case_ids)))
        self.assertEqual(
            {case["id"] for case in cases},
            SHARED_PARITY_CASES,
        )

        available_languages: set[str] = set()
        for case in cases:
            with self.subTest(fixture=case["id"]):
                self.assertRegex(case["id"], r"^[a-z0-9]+(?:[.-][a-z0-9]+)*\.v1$")
                self.assertTrue(case["requirementIds"])
                self.assertEqual(len(case["requirementIds"]), len(set(case["requirementIds"])))
                self.assertTrue(
                    all(
                        requirement.startswith((
                            "KMIPKIT-0043-FR-",
                            "KMIPKIT-0003-FR-",
                            "KMIPKIT-0004-FR-",
                            "KMIPKIT-0005-FR-",
                            "KMIPKIT-0006-FR-",
                            "KMIPKIT-0007-FR-",
                            "KMIPKIT-0012-FR-",
                            "KMIPKIT-0014-FR-",
                            "KMIPKIT-0039-FR-",
                            "KMIPKIT-REQ-SPEC-6.1.23-",
                        ))
                        for requirement in case["requirementIds"]
                    ),
                    case["id"],
                )
                self.assertTrue(case["input"])
                self.assertTrue(case["expectedOutput"])
                self.assertTrue(case["javaObservedOutput"])
                self.assertTrue(case["cleanup"])
                self.assertEqual(len(case["cleanup"]), len(set(case["cleanup"])))
                self.assertEqual(
                    set(case["consumerTests"]),
                    set(SHARED_PARITY_LANGUAGES),
                    f"{case['id']} must declare an available or blocked mapping for every parity language",
                )
                self.assertTrue(set(case["consumerTests"]).issubset(SHARED_PARITY_LANGUAGES))
                for language, mapping in case["consumerTests"].items():
                    with self.subTest(fixture=case["id"], language=language):
                        self.assertIn(mapping["status"], {"available", "blocked"})
                        if case["id"] == "errors.schema-redaction.v1" and language == "c":
                            self.assertEqual(
                                mapping["status"],
                                "available",
                                "the C numeric-only error surface can satisfy redaction by exposing no diagnostic text",
                            )
                            self.assertEqual(mapping.get("diagnosticSurface"), "none")
                            c_header = (ROOT / "bindings" / "c" / "include" / "kmipkit.h").read_text(
                                encoding="utf-8"
                            )
                            last_error = re.search(
                                r"int32_t\s+kmipkit_last_error_get\s*\(([^;]+)\)\s*;",
                                c_header,
                            )
                            self.assertIsNotNone(last_error)
                            assert last_error is not None
                            self.assertNotRegex(last_error.group(1), r"\b(?:char|uint8_t)\s*\*")
                            c_test_source = (ROOT / mapping["testFile"]).read_text(
                                encoding="utf-8"
                            )
                            self.assertIn("this ABI has no text channel", c_test_source)
                        if mapping["status"] == "available":
                            available_languages.add(language)
                            self.assertTrue(mapping["test"])
                            if language != "java":
                                self.assertTrue(mapping["testFile"])
                            if case["id"] == "api-levels.shared.v1" and language == "rust":
                                self.assertEqual(
                                    mapping["testFile"],
                                    "crates/kmipkit/tests/api_levels.rs",
                                    "the Rust API-level fixture must map to an integration test outside production sources",
                                )
                        else:
                            self.assertTrue(mapping.get("reason"), case["id"])

                        if language == "java":
                            java_mapping = mapping
                            self.assertTrue(java_mapping["test"])
                            class_name, method_name = java_mapping["test"].split("#", 1)
                            source_path = JAVA_TEST_ROOT / Path(*class_name.split(".")).with_suffix(".java")
                            self.assertTrue(source_path.is_file(), case["id"])
                            source = source_path.read_text(encoding="utf-8")
                            self.assertRegex(
                                source,
                                rf"@Test\s+void\s+{method_name}\s*\(",
                                f"{case['id']} must map to an executable Java 17 JUnit test",
                            )
                            continue

                        if mapping["status"] == "available":
                            source_path = ROOT / mapping["testFile"]
                            self.assertTrue(source_path.is_file(), case["id"])
                            source = source_path.read_text(encoding="utf-8")
                            test_name = mapping["test"]
                            if language == "rust":
                                test_name = test_name.rsplit("::", 1)[-1]
                                declaration = rf"\bfn\s+{re.escape(test_name)}\s*\("
                            elif language == "python":
                                declaration = rf"\bdef\s+{re.escape(test_name)}\s*\("
                            else:
                                declaration = (
                                    rf"\b(?:static\s+)?(?:bool|int|void)\s+"
                                    rf"{re.escape(test_name)}\s*\("
                                )
                            self.assertRegex(
                                source,
                                declaration,
                                f"{case['id']} must map to an executable {language} consumer test",
                            )

                        if (
                            case["id"] == "batch.ordered-completed-pending-failed.v1"
                            and language in {"c", "python"}
                            and mapping["status"] == "available"
                        ):
                            adapter_assertions = mapping.get("adapterAssertions", [])
                            self.assertIn("ordered-mixed-outcomes", adapter_assertions)
                            support = next(
                                (
                                    item
                                    for item in mapping.get("supportingTests", [])
                                    if "transportExchangeCount" in item.get("assertions", [])
                                ),
                                None,
                            )
                            self.assertIsNotNone(
                                support,
                                f"{language} must map evidence for the shared one-exchange result",
                            )
                            assert support is not None
                            self.assertEqual(support["language"], "rust")
                            support_source = (ROOT / support["testFile"]).read_text(
                                encoding="utf-8"
                            )
                            self.assertRegex(
                                support_source,
                                rf"\bfn\s+{re.escape(support['test'])}\s*\(",
                            )
                            if language == "c":
                                self.assertIn("fixture_statuses[index]", source)
                                self.assertIn("fixture_correlation", source)
                                self.assertIn("fixture_item_count", source)
                                self.assertIn("fixture_pending_count", source)
                                self.assertIn(
                                    "expected_exchange_count",
                                    support_source,
                                )
                                self.assertIn(
                                    'expected["transportExchangeCount"]',
                                    support_source,
                                )
                            elif language == "python":
                                self.assertIn(
                                    'len(execution_calls) == expected["transportExchangeCount"]',
                                    source,
                                )
                                self.assertIn(
                                    "fake.borrow().exchange_count()",
                                    support_source,
                                )
                                self.assertIn("transportExchangeCount", support_source)
                                self.assertIn(
                                    "singleNativeExecuteInvocation", adapter_assertions
                                )

                java_mapping = case["consumerTests"]["java"]
                declared_capabilities = {
                    entry["capabilityId"]
                    for entry in self.manifest.get("capabilities", [])
                    if entry.get("capabilityState") == "declared"
                }
                if java_mapping["status"] == "available":
                    self.assertIn(case["capability"], declared_capabilities, case["id"])
                    observed = {
                        key: value
                        for key, value in case["javaObservedOutput"].items()
                        if key != "available"
                    }
                    self.assertEqual(observed, case["expectedOutput"], case["id"])
                else:
                    self.assertNotIn(case["capability"], declared_capabilities, case["id"])
                    self.assertTrue(java_mapping.get("reason"), case["id"])
                    self.assertFalse(case["javaObservedOutput"].get("available", True), case["id"])

        self.assertEqual(
            available_languages,
            set(fixtures["executableConsumerLanguages"]),
        )
        self.assertEqual(
            set(fixtures["plannedConsumerLanguages"]),
            set(fixtures["parityLanguages"]) - available_languages,
        )

        redaction = next(case for case in cases if case["id"] == "errors.schema-redaction.v1")
        self.assertEqual(redaction["errorCategory"], "invalid_schema")
        self.assertTrue(redaction["expectedOutput"]["diagnosticOmitsInputMarker"])

        batch = next(case for case in cases if case["id"] == "batch.ordered-completed-pending-failed.v1")
        self.assertEqual(batch["expectedOutput"]["resultStatuses"], [
            "Success", "Operation Pending", "Operation Failed"
        ])
        batch_java_output = {
            key: value
            for key, value in batch["javaObservedOutput"].items()
            if key != "available"
        }
        self.assertEqual(batch_java_output, batch["expectedOutput"])

        ttlv = next(case for case in cases if case["id"] == "ttlv.open-values-and-child-order.v1")
        self.assertEqual(ttlv["javaObservedOutput"]["childTagsInOrder"], ["0x540001", "0x540002"])
        self.assertEqual(ttlv["expectedOutput"]["enumerationValue"], 0xE1020304)
        self.assertTrue(ttlv["expectedOutput"]["unknownValuesPreserved"])

        extension = next(
            case for case in cases
            if case["id"] == "extensions.recognition-and-generic-preservation.v1"
        )
        extension_fixtures = json.loads(
            (ROOT / extension["input"]["sourceFixture"]).read_text(encoding="utf-8")
        )
        source_case_ids = {case["id"] for case in extension_fixtures["cases"]}
        self.assertTrue(set(extension["input"]["caseIds"]).issubset(source_case_ids))

    def test_discover_versions_operation_has_four_language_shared_parity_evidence(self) -> None:
        fixtures = json.loads(SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))
        case = next(
            item
            for item in fixtures["cases"]
            if item["id"] == "discover-versions.supported-versions.v1"
        )
        operation = next(
            item
            for item in self.manifest["operations"]
            if item["operationId"] == "KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS"
        )

        self.assertEqual(operation["apiState"], "verified")
        evidence = set(operation["parityEvidenceIds"])
        self.assertIn(SHARED_PARITY_FIXTURE.relative_to(ROOT).as_posix(), evidence)
        self.assertIn(
            "crates/kmipkit-protocol/tests/unit/discover_versions_tests.rs",
            evidence,
        )

        consumers = case["consumerTests"]
        self.assertEqual(set(consumers), set(SHARED_PARITY_LANGUAGES))
        self.assertEqual(
            (
                consumers["rust"]["testFile"],
                consumers["rust"]["test"],
            ),
            (
                "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs",
                "ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
            ),
        )
        for language, consumer in consumers.items():
            with self.subTest(language=language):
                self.assertEqual(consumer["status"], "available")
                if language == "java":
                    class_name = consumer["test"].split("#", 1)[0]
                    test_path = f"bindings/java/src/test/java/{class_name.replace('.', '/')}.java"
                else:
                    test_path = consumer["testFile"]
                self.assertIn(test_path, evidence)
        c_harness = consumers["c"]["supportingTests"][0]
        self.assertEqual(c_harness["testFile"], "crates/kmipkit-ffi/tests/c_api_coverage.rs")
        self.assertIn(c_harness["testFile"], evidence)

    def test_generated_rust_api_uses_rust_mapping_and_excludes_blocked_records(self) -> None:
        manifest = {
            "formatVersion": 2,
            "releaseLine": "1.0.0",
            "languages": ["Rust", "C", "Java", "Python"],
            "publicPackages": [
                {"id": "rust-kmipkit-client", "language": "rust"},
                {"id": "c-kmipkit", "language": "c"},
            ],
            "apiLevels": [],
            "capabilities": [],
            "operations": [
                {"operationId": "blocked", "apiState": "blocked"},
                {"operationId": "ready", "apiState": "declared"},
            ],
            "publicDeclarations": [
                {
                    "declarationId": "blocked_declaration",
                    "packageId": "rust-kmipkit-client",
                    "kind": "function",
                    "declarationState": "blocked",
                    "languageMappings": {},
                },
                {
                    "declarationId": "ready_declaration",
                    "packageId": "rust-kmipkit-client",
                    "kind": "function",
                    "declarationState": "declared",
                    "languageMappings": {
                        "rust": {"name": "kmipkit_client::Client::execute"},
                        "c": {"name": "kmipkit_client_execute"},
                    },
                },
            ],
            "approvedExclusions": [],
            "types": [],
            "functions": [],
            "errorCategories": [],
            "limits": [],
            "security": {},
        }

        projection = generate._ready_api_projection(manifest)
        output = generate.render_public_rust_api(projection)

        self.assertEqual([item["operationId"] for item in projection["operations"]], ["ready"])
        self.assertIn("kmipkit_client::Client::execute", output)
        self.assertNotIn("kmipkit_client_execute", output)
        self.assertNotIn("blocked_declaration", output)

    def test_manifest_exposes_identity_fields_and_ordered_batch_extension_views(self) -> None:
        function_ids = {function["id"] for function in self.manifest["functions"]}
        required_functions = {
            "extension_identity_vendor_identifier",
            "extension_identity_name",
            "extension_identity_version",
            "client_batch_item_discover_versions",
            "client_batch_item_extension_count",
            "client_batch_item_extension_identity_at",
            "client_batch_item_extension_criticality_indicator_at",
        }

        self.assertTrue(
            required_functions.issubset(function_ids),
            sorted(required_functions - function_ids),
        )

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
        jni_symbols: dict[str, list[dict[str, Any]]] = {}
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

                if function["id"] in {
                    "client_batch_item_response_delegated_login_ttlv",
                    "client_batch_item_response_login_ttlv",
                    "client_batch_item_response_locate_unique_identifier_at",
                    "client_batch_item_response_obtain_lease_ttlv",
                    "client_batch_item_response_pkcs11_interface_ttlv",
                    "client_batch_item_response_pkcs11_function_ttlv",
                    "client_batch_item_response_pkcs11_correlation_value",
                    "client_batch_item_response_pkcs11_output_parameters_ttlv",
                    "client_batch_item_response_pkcs11_return_code_ttlv",
                    "client_batch_item_response_recertify_ttlv",
                    "client_batch_item_response_rekey_ttlv",
                    "client_batch_item_response_rekey_key_pair_ttlv",
                    "client_batch_item_response_register_ttlv",
                    "client_batch_item_response_reprovision_ttlv",
                    "kmipkit_client_batch_item_response_rng_retrieve_ttlv",
                    "kmipkit_client_batch_item_response_rng_seed_ttlv",
                }:
                    self.assertNotIn("jniSymbol", java)
                    if "jniAggregateSymbol" in java:
                        self.assertTrue(
                            java["jniAggregateSymbol"].startswith("Java_org_kmipkit_")
                        )
                    if function["id"] in {
                        "client_batch_item_response_recertify_ttlv",
                        "client_batch_item_response_rekey_ttlv",
                        "client_batch_item_response_rekey_key_pair_ttlv",
                        "client_batch_item_response_register_ttlv",
                        "client_batch_item_response_reprovision_ttlv",
                        "kmipkit_client_batch_item_response_rng_retrieve_ttlv",
                        "kmipkit_client_batch_item_response_rng_seed_ttlv",
                    }:
                        self.assertIn("jniAggregateSymbol", java)
                else:
                    self.assertTrue(java["jniSymbol"].startswith("Java_org_kmipkit_"))
                    jni_symbols.setdefault(java["jniSymbol"], []).append(function)

                python = function["python"]
                self.assertTrue(
                    python["module"] == "kmipkit" or python["module"].startswith("kmipkit.")
                )
                self.assertTrue(python["cffiSymbol"].startswith("kmipkit_"))
                cffi_symbols.append(python["cffiSymbol"])

        self.assertEqual(len(c_symbols), len(set(c_symbols)))
        self.assertEqual(len(cffi_symbols), len(set(cffi_symbols)))
        for symbol, aliases in jni_symbols.items():
            with self.subTest(jni_symbol=symbol):
                # Additive C ABI compatibility symbols can share the one
                # Java entrypoint when they describe the same Java operation.
                java_signatures = {
                    (
                        row["java"]["package"],
                        row["java"]["class"],
                        row["java"]["method"],
                        json.dumps(row["java"]["parameters"], sort_keys=True),
                        row["java"]["returnType"],
                    )
                    for row in aliases
                }
                self.assertEqual(len(java_signatures), 1)

    def test_t029_fixture_consumers_assert_runtime_values_for_only_core_shared_cases(self) -> None:
        """T029 parity is limited to shared capabilities, not operation-specific rows."""
        fixtures = json.loads(SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))
        cases = {case["id"]: case for case in fixtures["cases"]}
        self.assertTrue(set(T029_CORE_FIXTURE_ASSERTIONS).issubset(cases))
        self.assertNotIn("discover-versions.supported-versions.v1", T029_CORE_FIXTURE_ASSERTIONS)
        self.assertFalse(any(case_id.startswith("creation.") for case_id in T029_CORE_FIXTURE_ASSERTIONS))

        for case_id, fields in T029_CORE_FIXTURE_ASSERTIONS.items():
            case = cases[case_id]
            with self.subTest(fixture=case_id):
                self.assertEqual(set(case["consumerTests"]), set(SHARED_PARITY_LANGUAGES))
                for language, mapping in case["consumerTests"].items():
                    with self.subTest(fixture=case_id, language=language):
                        self.assertEqual(mapping["status"], "available")
                        if language != "c":
                            body = self._mapped_test_body(language, mapping)
                            source_path = (
                                JAVA_TEST_ROOT / Path(*mapping["test"].split("#", 1)[0].split(".")).with_suffix(".java")
                                if language == "java"
                                else ROOT / mapping["testFile"]
                            )
                            file_source = source_path.read_text(encoding="utf-8")
                            if language == "python" and case_id == "usage-allocation.guard-accounting.v1":
                                self.assertIn("_usage_allocation_fixture()", body)
                                self.assertIn(case_id, file_source)
                            else:
                                if case_id not in body:
                                    self.assertIn(case_id, file_source)
                                    self.assertIn("caseById(", body)
                            for field in fields:
                                self.assertIn(field, body, f"{case_id}/{language} must consume {field}")
                            assertion = {
                                "rust": r"\bassert(?:_eq|_ne)?!",
                                "java": r"\bassert(?:Equals|True|False|ArrayEquals|Throws)\s*\(",
                                "python": r"(?:\bassert\b|\.assert[A-Z]\w*\s*\()",
                            }[language]
                            self.assertRegex(body, assertion, f"{case_id}/{language} must assert runtime behavior")
                            if case_id == "usage-allocation.guard-accounting.v1" and language == "java":
                                allocated_state_assertions = re.findall(
                                    r"assertEquals\s*\(\s*SharedCapabilityParityFixtureTest\.numberField\(\s*"
                                    r"afterSuccess\s*,\s*\"allocated\"\s*\)\s*,\s*"
                                    r"context\.allocatedUnits\(\)\s*\)",
                                    body,
                                    flags=re.DOTALL,
                                )
                                self.assertGreaterEqual(
                                    len(allocated_state_assertions),
                                    3,
                                    "Java Usage Allocation must compare fixture allocated units after success "
                                    "and each rejected attempt",
                                )
                            continue

                        rust_test, c_entry = T029_C_HARNESS_TESTS[case_id]
                        support = next(
                            (
                                item
                                for item in mapping.get("supportingTests", [])
                                if item.get("language") == "rust" and item.get("test") == rust_test
                            ),
                            None,
                        )
                        self.assertIsNotNone(
                            support,
                            f"{case_id}/c must have a Rust fixture parser that passes data into compiled C",
                        )
                        assert support is not None
                        support_source = (ROOT / support["testFile"]).read_text(encoding="utf-8")
                        support_body = self._rust_function_body(support_source, rust_test)
                        fixture_body = support_body
                        if case_id not in fixture_body:
                            shared_runner = "run_generated_public_client_abi_consumer"
                            self.assertIn(f"{shared_runner}(", support_body)
                            fixture_body = self._rust_function_body(support_source, shared_runner)
                        self.assertIn(case_id, fixture_body)
                        self.assertIn('tests/fixtures/api/shared-capability-parity.json', fixture_body)
                        self.assertIn(c_entry, fixture_body)
                        self.assertRegex(fixture_body, r"\bassert(?:_eq|_ne)?!")
                        for field in fields:
                            if field in {"close-configuration-owned-native-handles-once",
                                         "release-returned-registry-views-once"}:
                                self.assertIn(field, fixture_body, f"{case_id}/c harness must verify {field}")
                            else:
                                self.assertIn(field, fixture_body, f"{case_id}/c harness must pass {field}")

    def _mapped_test_body(self, language: str, mapping: dict[str, Any]) -> str:
        if language == "java":
            class_name, method_name = mapping["test"].split("#", 1)
            source_path = JAVA_TEST_ROOT / Path(*class_name.split(".")).with_suffix(".java")
            source = source_path.read_text(encoding="utf-8")
            match = re.search(rf"@Test\s+void\s+{re.escape(method_name)}\s*\([^)]*\)[^{{]*\{{", source)
            self.assertIsNotNone(match, mapping["test"])
            assert match is not None
            return self._braced_body(source, match.end() - 1)

        source_path = ROOT / mapping["testFile"]
        source = source_path.read_text(encoding="utf-8")
        test_name = mapping["test"].rsplit("::", 1)[-1]
        if language == "python":
            tree = ast.parse(source, filename=str(source_path))
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == test_name:
                    return ast.get_source_segment(source, node) or ""
            self.fail(f"Python consumer test {test_name} is missing from {source_path}")
        return self._rust_function_body(source, test_name)

    def _rust_function_body(self, source: str, test_name: str) -> str:
        match = re.search(rf"\bfn\s+{re.escape(test_name)}\s*\([^)]*\)[^{{]*\{{", source)
        self.assertIsNotNone(match, test_name)
        assert match is not None
        return self._braced_body(source, match.end() - 1)

    def _braced_body(self, source: str, opening_brace: int) -> str:
        depth = 0
        for index in range(opening_brace, len(source)):
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
                if depth == 0:
                    return source[opening_brace:index + 1]
        self.fail("mapped consumer test has an unterminated body")
        return ""

    def test_locate_identifier_index_helper_uses_the_response_jni_entrypoint(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        response = functions["client_batch_item_response_locate"]
        identifier_at = functions["client_batch_item_response_locate_unique_identifier_at"]

        self.assertEqual(
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLocate",
            response["java"]["jniSymbol"],
        )
        self.assertEqual("locateResponse", identifier_at["java"]["method"])
        self.assertNotIn("jniSymbol", identifier_at["java"])

    def test_login_ticket_ttlv_names_its_aggregate_jni_owner(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        login_response = functions["client_batch_item_response_login"]
        ticket_ttlv = functions["client_batch_item_response_login_ttlv"]

        self.assertNotIn("jniSymbol", ticket_ttlv["java"])
        self.assertEqual(
            LOGIN_TTLV_AGGREGATE_JNI_SYMBOL,
            ticket_ttlv["java"].get("jniAggregateSymbol"),
        )
        self.assertEqual(
            login_response["java"]["jniSymbol"],
            ticket_ttlv["java"]["jniAggregateSymbol"],
        )

    def test_obtain_lease_ttlv_names_its_aggregate_jni_owner(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        response = functions["client_batch_item_response_obtain_lease"]
        response_ttlv = functions["client_batch_item_response_obtain_lease_ttlv"]

        self.assertNotIn("jniSymbol", response_ttlv["java"])
        self.assertEqual("responseTtlv", response_ttlv["java"]["method"])
        self.assertEqual(
            response["java"]["jniSymbol"],
            response_ttlv["java"].get("jniAggregateSymbol"),
        )

    def test_rekey_response_ttlv_names_its_aggregate_jni_owner(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        response = functions["client_batch_item_response_rekey"]
        response_ttlv = functions["client_batch_item_response_rekey_ttlv"]

        self.assertNotIn("jniSymbol", response_ttlv["java"])
        self.assertEqual("rekeyResponse", response_ttlv["java"]["method"])
        self.assertEqual(
            response["java"]["jniSymbol"],
            response_ttlv["java"].get("jniAggregateSymbol"),
        )

    def test_register_response_ttlv_names_its_aggregate_jni_owner(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        response = functions["client_batch_item_response_register"]
        response_ttlv = functions["client_batch_item_response_register_ttlv"]

        self.assertNotIn("jniSymbol", response_ttlv["java"])
        self.assertEqual(
            response["java"]["jniSymbol"],
            response_ttlv["java"].get("jniAggregateSymbol"),
        )

    def test_pkcs11_response_accessors_name_the_aggregate_jni_owner(self) -> None:
        functions = {function["id"]: function for function in self.manifest["functions"]}
        response = functions["client_batch_item_response_pkcs11_ttlv"]
        aggregate = response["java"]["jniSymbol"]
        accessor_ids = (
            "client_batch_item_response_pkcs11_interface_ttlv",
            "client_batch_item_response_pkcs11_function_ttlv",
            "client_batch_item_response_pkcs11_correlation_value",
            "client_batch_item_response_pkcs11_output_parameters_ttlv",
            "client_batch_item_response_pkcs11_return_code_ttlv",
        )

        for function_id in accessor_ids:
            with self.subTest(function=function_id):
                java = functions[function_id]["java"]
                self.assertNotIn("jniSymbol", java)
                self.assertEqual(aggregate, java.get("jniAggregateSymbol"))

    def test_java_method_names_are_legal_java_identifiers(self) -> None:
        for function in self.manifest["functions"]:
            method = function["java"]["method"]
            with self.subTest(function=function["id"]):
                self.assertTrue(method.isidentifier(), method)
                self.assertNotIn(method, JAVA_KEYWORDS, method)

    def test_generated_destinations_and_parity_fixture_follow_manifest_order(self) -> None:
        self.assertEqual(self.manifest["generatedOutputs"], ALL_GENERATED_OUTPUTS)

        ids = [item["id"] for item in self.manifest["types"]]
        self.assertEqual(len(ids), len(set(ids)))
        function_ids = [item["id"] for item in self.manifest["functions"]]
        self.assertEqual(len(function_ids), len(set(function_ids)))

        for relative_path in (output["path"] for output in ALL_GENERATED_OUTPUTS):
            self.assertTrue((ROOT / relative_path).is_file(), relative_path)

        registry_parity = json.loads(
            (ROOT / GENERATED_OUTPUTS["parityFixtures"]).read_text(encoding="utf-8")
        )
        registry_baseline = json.loads(REGISTRY_MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(registry_parity, registry_baseline)

        parity_path = ROOT / "tests/fixtures/api/generated/public_api_parity.json"
        parity: Any = json.loads(parity_path.read_text(encoding="utf-8"))
        expected_parity = {
            "formatVersion": self.manifest["formatVersion"],
            "releaseLine": self.manifest["releaseLine"],
            "languages": self.manifest["languages"],
            "publicPackages": self.manifest["publicPackages"],
            "apiLevels": self.manifest["apiLevels"],
            "capabilities": [
                record for record in self.manifest["capabilities"]
                if record["capabilityState"] in {"declared", "verified"}
            ],
            "operations": [
                record for record in self.manifest["operations"]
                if record["apiState"] in {"declared", "verified"}
            ],
            "publicDeclarations": [
                record for record in self.manifest["publicDeclarations"]
                if record["declarationState"] in {"declared", "verified"}
            ],
            "approvedExclusions": self.manifest["approvedExclusions"],
            "types": self.manifest["types"],
            "functions": self.manifest["functions"],
            "errorCategories": self.manifest["errorCategories"],
            "limits": self.manifest["limits"],
            "security": self.manifest["security"],
        }
        self.assertEqual(parity, expected_parity)

    def test_t029_shared_corpus_covers_defaults_ranges_and_usage_guard(self) -> None:
        fixtures = json.loads(SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))
        cases = {case["id"]: case for case in fixtures["cases"]}
        required_cases = {
            "codec-limits.defaults-and-ranges.v1",
            "usage-allocation.guard-accounting.v1",
        }

        self.assertTrue(required_cases.issubset(cases))
        self.assertEqual(
            cases["batch.ordered-completed-pending-failed.v1"]["expectedOutput"][
                "transportExchangeCount"
            ],
            1,
        )
        self.assertNotIn(
            "no-batch-or-transport-resource-created",
            cases["batch.ordered-completed-pending-failed.v1"]["cleanup"],
        )
        codec_limits = cases["codec-limits.defaults-and-ranges.v1"]
        self.assertEqual(
            codec_limits["expectedOutput"]["defaults"],
            {
                "maxMessageBytes": 16 * 1024 * 1024,
                "maxStructureDepth": 64,
                "maxElements": 100_000,
            },
        )
        self.assertEqual(
            codec_limits["input"]["configured"],
            {
                "maxMessageBytes": 0,
                "maxStructureDepth": 64,
                "maxElements": 0,
            },
        )
        self.assertEqual(
            codec_limits["expectedOutput"]["invalidDepth"],
            {
                "value": 65,
                "accepted": False,
                "errorCategory": "resource_limit",
            },
        )
        allocation = cases["usage-allocation.guard-accounting.v1"]
        self.assertEqual(allocation["input"]["usageLimitsUnit"], 0xFEDCBA98)
        self.assertEqual(
            allocation["expectedOutput"]["afterSuccessfulProtection"],
            {"allocated": 5, "reserved": 2, "available": 3},
        )
        self.assertEqual(allocation["expectedOutput"]["transportExchangeCount"], 2)
        for case in cases.values():
            for language in SHARED_PARITY_LANGUAGES:
                with self.subTest(fixture=case["id"], language=language):
                    self.assertEqual(
                        case["consumerTests"][language]["status"],
                        "available",
                        f"T029 requires an executable {language} consumer",
                    )


    def test_discover_versions_consumers_read_the_shared_expected_versions(self) -> None:
        consumers = {
            "rust": ROOT / "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs",
            "c_harness": ROOT / "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "c_abi": ROOT / "bindings/c/tests/client_abi_consumer.c",
            "java": ROOT / "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "python": ROOT / "bindings/python/tests/test_client.py",
        }
        source = {
            language: path.read_text(encoding="utf-8")
            for language, path in consumers.items()
        }

        self.assertIn("discover-versions.supported-versions.v1", source["rust"])
        self.assertIn('discover_expected["supportedVersions"]', source["rust"])
        self.assertIn("discover-versions.supported-versions.v1", source["c_harness"])
        self.assertIn('discover_expected["supportedVersions"]', source["c_harness"])
        self.assertIn("fixture_version_count", source["c_abi"])
        self.assertIn("fixture_versions", source["c_abi"])
        self.assertIn("discover-versions.supported-versions.v1", source["java"])
        self.assertIn('objectArrayFields(discoverVersionsExpected, "supportedVersions")', source["java"])
        self.assertIn("discover-versions.supported-versions.v1", source["python"])

if __name__ == "__main__":
    unittest.main()
