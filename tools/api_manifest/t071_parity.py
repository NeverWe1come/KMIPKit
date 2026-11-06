"""Execute the T071 shared parity cases through their language consumers."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from pathlib import PurePosixPath
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
DISCOVER_VERSIONS_CASE = "discover-versions.supported-versions.v1"
MIXED_BATCH_OUTCOMES_CASE = "batch.ordered-completed-pending-failed.v1"
REGISTRY_ISOLATION_CASE = "client-configuration.registry-isolation.v1"
SCHEMA_REDACTION_CASE = "errors.schema-redaction.v1"
API_LEVELS_CASE = "api-levels.shared.v1"
OPEN_TTLV_VALUES_CASE = "ttlv.open-values-and-child-order.v1"
EXTENSION_RECOGNITION_CASE = "extensions.recognition-and-generic-preservation.v1"
CREATE_REQUEST_CASE = "creation.create.request-response.v1"
CREATE_KEY_PAIR_CASE = "creation.key-pair.request-response.v1"
CREATE_SPLIT_KEY_CASE = "creation.split-key.request-response.v1"
CREATE_FAILURE_MESSAGE_CASE = "creation.result-message-on-failure.v1"
CODEC_LIMITS_CASE = "codec-limits.defaults-and-ranges.v1"
USAGE_ALLOCATION_CASE = "usage-allocation.guard-accounting.v1"
PYTHON_TEST_CLASSES = {
    (
        "bindings/python/tests/test_extension_registry.py",
        "test_registry_registration_is_immutable_owned_by_configuration_and_isolated",
    ): "ExtensionRegistryTests",
    (
        "bindings/python/tests/test_extension_registry.py",
        "test_failure_and_default_diagnostics_redact_secret_fixture_values",
    ): "ExtensionRegistryTests",
    (
        "bindings/python/tests/test_extension_registry.py",
        "test_shared_inbound_fixtures_are_inspected_and_generic_ttlv_is_preserved",
    ): "ExtensionRegistryTests",
}
CASE_TESTS = {
    DISCOVER_VERSIONS_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs",
            "ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_discover_versions_response_mapping_matches_shared_fixture",
        ),
    },
    MIXED_BATCH_OUTCOMES_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs",
            "execute_lifecycle_tests::ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_batch_execute_preserves_ordered_mixed_outcomes_and_invokes_native_execute_once",
        ),
    },
    REGISTRY_ISOLATION_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/extension_registry.rs",
            "client_configurations_own_isolated_immutable_registries",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "c_abi_consumer_matches_shared_registry_cleanup_fixture",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ExtensionRegistryTest.java",
            "org.kmipkit.ExtensionRegistryTest#clientConfigurationsOwnImmutableIsolatedRegistrySnapshots",
        ),
        "python": (
            "bindings/python/tests/test_extension_registry.py",
            "test_registry_registration_is_immutable_owned_by_configuration_and_isolated",
        ),
    },
    SCHEMA_REDACTION_CASE: {
        "rust": (
            "crates/kmipkit-protocol/tests/extension_redaction.rs",
            "schema_error_debug_display_and_source_chain_redact_payload_secrets",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "c_abi_consumer_redacts_shared_fixture_diagnostic_marker",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ExtensionRegistryTest.java",
            "org.kmipkit.ExtensionRegistryTest#invalidSchemaErrorsAndValidatedValueFormattingRedactPayloadText",
        ),
        "python": (
            "bindings/python/tests/test_extension_registry.py",
            "test_failure_and_default_diagnostics_redact_secret_fixture_values",
        ),
    },
    API_LEVELS_CASE: {
        "rust": (
            "crates/kmipkit/tests/api_levels.rs",
            "api_levels_match_the_shared_fixture_without_network_calls",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_compiles_and_runs",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/SharedCapabilityParityFixtureTest.java",
            "org.kmipkit.SharedCapabilityParityFixtureTest#apiLevelQueryMatchesSharedFixtureLocally",
        ),
        "python": (
            "bindings/python/tests/test_shared_parity_fixtures.py",
            "test_api_level_query_matches_shared_fixture_locally",
        ),
    },
    OPEN_TTLV_VALUES_CASE: {
        "rust": (
            "crates/kmipkit-ttlv/tests/shared_api_parity.rs",
            "generic_ttlv_shared_fixture_preserves_open_values_and_child_order",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "c_abi_consumer_preserves_shared_ttlv_fixture_values_and_order",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/SharedCapabilityParityFixtureTest.java",
            "org.kmipkit.SharedCapabilityParityFixtureTest#genericTtlvFixturePreservesOpenEnumerationBytesOrderAndViewLifecycle",
        ),
        "python": (
            "bindings/python/tests/test_shared_parity_fixtures.py",
            "test_generic_ttlv_shared_fixture_preserves_open_values_order_and_owner_lifecycle",
        ),
    },
    EXTENSION_RECOGNITION_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/extension_registry_tests.rs",
            "every_shared_inbound_fixture_matches_internal_outcome_ids_and_preserved_ttlv_in_both_orders",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ExtensionRegistryTest.java",
            "org.kmipkit.ExtensionRegistryTest#everySharedInboundFixtureRunsThroughThePublicJavaAdapterInBothOrders",
        ),
        "python": (
            "bindings/python/tests/test_extension_registry.py",
            "test_shared_inbound_fixtures_are_inspected_and_generic_ttlv_is_preserved",
        ),
    },
    CREATE_REQUEST_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/create_execution_tests.rs",
            "create_convenience_method_maps_shared_fixture_over_one_exchange",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_creation_response_mapping_preserves_wire_identifiers_and_failure_message",
        ),
    },
    CREATE_KEY_PAIR_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/create_key_pair_execution_tests.rs",
            "create_key_pair_maps_shared_fixture_over_one_exchange",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_creation_response_mapping_preserves_wire_identifiers_and_failure_message",
        ),
    },
    CREATE_SPLIT_KEY_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs",
            "create_split_key_maps_shared_fixture_success_and_pending_outcomes",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_maps_split_key_fixture_success_and_pending_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#createSplitKeyFixtureSuccessAndPendingMatchSharedFixtureOverMutualTls",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_create_split_key_fixture_success_and_pending_metadata",
        ),
    },
    CREATE_FAILURE_MESSAGE_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/create_execution_tests.rs",
            "create_failure_preserves_result_and_does_not_retry",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        ),
        "python": (
            "bindings/python/tests/test_client.py",
            "test_creation_response_mapping_preserves_wire_identifiers_and_failure_message",
        ),
    },
    CODEC_LIMITS_CASE: {
        "rust": (
            "crates/kmipkit-ttlv/tests/shared_api_parity.rs",
            "codec_limits_shared_fixture_covers_defaults_ranges_and_rejection",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "c_abi_consumer_matches_the_shared_codec_limits_fixture",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/SharedCapabilityParityFixtureTest.java",
            "org.kmipkit.SharedCapabilityParityFixtureTest#codecLimitsFixtureMatchesDefaultsRangesAndErrors",
        ),
        "python": (
            "bindings/python/tests/test_shared_parity_fixtures.py",
            "test_codec_limits_shared_fixture_matches_defaults_ranges_and_errors",
        ),
    },
    USAGE_ALLOCATION_CASE: {
        "rust": (
            "crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs",
            "fixture_usage_allocation_guard_matches_shared_accounting_contract",
        ),
        "c": (
            "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
            "ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost",
        ),
        "java": (
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "org.kmipkit.ClientFacadeTlsIntegrationTest#usageAllocationContextGuardsJavaProtectionExecutionLocally",
        ),
        "python": (
            "bindings/python/tests/test_usage_allocation_guard_native_integration.py",
            "test_native_python_guard_executes_encrypt_and_rejects_over_budget_before_transport",
        ),
    },
}


class ParityRunnerError(ValueError):
    """Raised when a fixture does not have a supported executable consumer set."""


def _rust_package_and_target(expected_path: str) -> tuple[str, list[str], list[str]]:
    test_path = PurePosixPath(expected_path)
    parts = test_path.parts
    package_name = parts[1] if len(parts) > 1 else ""
    if (
        test_path.is_absolute()
        or len(parts) < 4
        or parts[0] != "crates"
        or parts[2] != "tests"
        or test_path.suffix != ".rs"
        or any(part in {".", ".."} for part in parts)
        or re.fullmatch(r"[a-z][a-z0-9_-]*", package_name) is None
    ):
        raise ParityRunnerError("Rust fixture test path is not a safe crate test path")
    if len(parts) == 4:
        return package_name, ["--test", test_path.stem], ["--", "--exact"]
    return package_name, ["--lib"], []


def _fixture_case(case_id: str) -> dict[str, Any]:
    if case_id not in CASE_TESTS:
        raise ParityRunnerError(f"unsupported T071 parity case: {case_id}")

    try:
        fixtures = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ParityRunnerError("cannot read the shared parity fixture") from error

    cases = fixtures.get("cases")
    if not isinstance(cases, list):
        raise ParityRunnerError("shared parity fixture has no case list")
    for case in cases:
        if isinstance(case, dict) and case.get("id") == case_id:
            return case
    raise ParityRunnerError(f"shared parity fixture does not contain {case_id}")


def _consumer_command(
    language: str,
    consumer: dict[str, Any],
    expected_path: str,
    expected_test: str,
) -> dict[str, Any]:
    test_file = expected_path
    test_name = expected_test
    working_directory = "."

    if language == "c":
        supporting_tests = consumer.get("supportingTests")
        if not isinstance(supporting_tests, list):
            raise ParityRunnerError("C fixture consumer has no compiled harness test")
        harness = next(
            (
                item
                for item in supporting_tests
                if isinstance(item, dict)
                and item.get("testFile") == expected_path
                and item.get("test") == expected_test
            ),
            None,
        )
        if harness is None:
            raise ParityRunnerError("C fixture consumer does not name the compiled harness test")
        test_file = harness["testFile"]
        test_name = harness["test"]
        package_name, target, exact_filter = _rust_package_and_target(test_file)
        command = [
            "cargo",
            "test",
            "-p",
            package_name,
            "--features",
            "coverage-c-consumer",
            *target,
            test_name,
            *exact_filter,
        ]
    elif language == "java":
        if consumer.get("test") != expected_test:
            raise ParityRunnerError("Java fixture consumer does not match its reviewed test")
        class_name, method_name = test_name.split("#", maxsplit=1)
        selector = f"{class_name}.{method_name}"
        wrapper = ROOT / "bindings" / "java" / (
            "gradlew.bat" if os.name == "nt" else "gradlew"
        )
        if os.name == "nt":
            command = [
                "cmd.exe",
                "/d",
                "/c",
                str(wrapper),
                "test",
                "--rerun-tasks",
                "--tests",
                selector,
            ]
        else:
            command = [str(wrapper), "test", "--rerun-tasks", "--tests", selector]
        working_directory = "bindings/java"
        test_name = selector
    else:
        if consumer.get("testFile") != expected_path or consumer.get("test") != expected_test:
            raise ParityRunnerError(f"{language} fixture consumer does not match its reviewed test")
        if language == "rust":
            test_name = expected_test.rsplit("::", maxsplit=1)[-1]
            package_name, target, exact_filter = _rust_package_and_target(expected_path)
            command = [
                "cargo",
                "test",
                "-p",
                package_name,
                *target,
                test_name,
                *exact_filter,
            ]
        else:
            pytest_selector = f"{test_file}::{test_name}"
            test_class = PYTHON_TEST_CLASSES.get((expected_path, expected_test))
            if test_class is not None:
                pytest_selector = f"{test_file}::{test_class}::{test_name}"
            command = [
                sys.executable,
                "-m",
                "pytest",
                "-q",
                pytest_selector,
            ]

    expected_test_name = test_name.rsplit(".", maxsplit=1)[-1] if language == "java" else test_name
    return {
        "language": language,
        "testFile": test_file,
        "test": test_name,
        "expectedTestName": expected_test_name,
        "workingDirectory": working_directory,
        "command": command,
    }


def build_plan(case_id: str) -> dict[str, Any]:
    case = _fixture_case(case_id)
    consumers = case.get("consumerTests")
    case_tests = CASE_TESTS[case_id]
    if not isinstance(consumers, dict) or set(consumers) != set(case_tests):
        raise ParityRunnerError("fixture must declare one consumer for each supported language")

    planned_consumers = []
    for language in ("rust", "c", "java", "python"):
        consumer = consumers[language]
        if not isinstance(consumer, dict) or consumer.get("status") != "available":
            raise ParityRunnerError(f"{language} fixture consumer is not available")
        expected_path, expected_test = case_tests[language]
        planned_consumers.append(
            _consumer_command(language, consumer, expected_path, expected_test)
        )

    return {"caseId": case_id, "consumers": planned_consumers}


def run_plan(plan: dict[str, Any]) -> int:
    for consumer in plan["consumers"]:
        language = consumer["language"]
        command = consumer["command"]
        working_directory = ROOT / consumer["workingDirectory"]
        print(f"T071 {language}: {shlex.join(command)}", flush=True)
        result = subprocess.run(
            command,
            capture_output=True,
            check=False,
            cwd=working_directory,
            encoding="utf-8",
            errors="replace",
            text=True,
        )
        if result.stdout:
            print(result.stdout, end="")
        if result.stderr:
            print(result.stderr, end="", file=sys.stderr)
        if result.returncode != 0:
            print(f"T071 {language} consumer failed with exit code {result.returncode}", file=sys.stderr)
            return result.returncode
        if not _expected_test_ran(consumer, result.stdout):
            print(f"T071 {language} command did not run the expected test", file=sys.stderr)
            return 1
    return 0


def _expected_test_ran(consumer: dict[str, Any], output: str) -> bool:
    language = consumer["language"]
    test_name = consumer["expectedTestName"]
    if language in {"rust", "c"}:
        return any(
            line.startswith("test ") and test_name in line and line.rstrip().endswith("... ok")
            for line in output.splitlines()
        )
    if language == "python":
        return re.search(r"\b1 passed\b", output) is not None
    if language == "java":
        class_name = consumer["test"].rsplit(".", maxsplit=1)[0]
        report = ROOT / consumer["workingDirectory"] / "build" / "test-results" / "test" / f"TEST-{class_name}.xml"
        try:
            suite = ET.parse(report).getroot()
        except (OSError, ET.ParseError):
            return False
        cases = suite.findall(".//testcase")
        return (
            len(cases) == 1
            and cases[0].get("name", "").removesuffix("()") == test_name
            and not cases[0].find("failure")
            and not cases[0].find("error")
            and not cases[0].find("skipped")
        )
    return False


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True, help="shared parity case ID to execute")
    parser.add_argument("--plan", action="store_true", help="print resolved commands without running them")
    args = parser.parse_args(argv)

    try:
        plan = build_plan(args.case)
        if args.plan:
            print(json.dumps(plan, indent=2, sort_keys=True))
            return 0
        return run_plan(plan)
    except ParityRunnerError as error:
        print(f"T071 parity runner: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
