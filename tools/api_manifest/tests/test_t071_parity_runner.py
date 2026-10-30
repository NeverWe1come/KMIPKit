"""Contract tests for planning T071 parity runs across language consumers."""

from __future__ import annotations

import json
import io
import re
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

from tools.api_manifest import t071_parity


ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "tools" / "api_manifest" / "t071_parity.py"
CI_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
DISCOVER_VERSIONS_CASE = "discover-versions.supported-versions.v1"
BATCH_MIXED_OUTCOMES_CASE = "batch.ordered-completed-pending-failed.v1"
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


def _script_contract_python_step(workflow: str) -> str:
    job = re.search(r"(?ms)^  script-contracts:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", workflow)
    if job is None:
        return ""
    step = re.search(
        r"(?ms)^      - name: Run Python contracts\n(.*?)(?=^      - name: |\Z)",
        job.group(1),
    )
    return step.group(1) if step is not None else ""


class T071ParityRunnerTests(unittest.TestCase):
    def test_t071_runner_contract_is_not_satisfied_by_another_job(self) -> None:
        workflow = """
  unrelated-job:
    run: |
      & $python -B -m unittest tools.api_manifest.tests.test_t071_parity_runner -v
  script-contracts:
    steps:
      - name: Run Python contracts
        run: |
          & $python -B -m unittest tools.api_manifest.tests.test_parity -v
"""

        self.assertNotIn("test_t071_parity_runner", _script_contract_python_step(workflow))

    def test_t071_runner_contracts_are_part_of_script_contract_ci(self) -> None:
        workflow = _script_contract_python_step(CI_WORKFLOW.read_text(encoding="utf-8"))

        self.assertRegex(
            workflow,
            r"& \$python -B -m unittest tools\.api_manifest\.tests\.test_parity "
            r"tools\.api_manifest\.tests\.test_t076_client_mapping "
            r"tools\.api_manifest\.tests\.test_t071_parity_runner -v",
        )

    def test_discover_versions_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                DISCOVER_VERSIONS_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], DISCOVER_VERSIONS_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["test"],
            "ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
        )
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--lib",
                "ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertIn("--features", consumers["c"]["command"])
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest.executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["java"]["expectedTestName"],
            "executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(
            consumers["python"]["test"],
            "test_discover_versions_response_mapping_matches_shared_fixture",
        )

    def test_batch_mixed_outcomes_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                BATCH_MIXED_OUTCOMES_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], BATCH_MIXED_OUTCOMES_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["test"],
            "ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange",
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest.executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(
            consumers["python"]["test"],
            "test_batch_execute_preserves_ordered_mixed_outcomes_and_invokes_native_execute_once",
        )

    def test_registry_isolation_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                REGISTRY_ISOLATION_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], REGISTRY_ISOLATION_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        rust = consumers["rust"]
        self.assertEqual(
            rust["testFile"],
            "crates/kmipkit-client/tests/extension_registry.rs",
        )
        self.assertEqual(
            rust["test"],
            "client_configurations_own_isolated_immutable_registries",
        )
        self.assertEqual(
            rust["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--test",
                "extension_registry",
                "client_configurations_own_isolated_immutable_registries",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "c_abi_consumer_matches_shared_registry_cleanup_fixture",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ExtensionRegistryTest.clientConfigurationsOwnImmutableIsolatedRegistrySnapshots",
        )
        self.assertEqual(
            consumers["python"]["test"],
            "test_registry_registration_is_immutable_owned_by_configuration_and_isolated",
        )
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_extension_registry.py::ExtensionRegistryTests::"
            "test_registry_registration_is_immutable_owned_by_configuration_and_isolated",
        )

    def test_schema_redaction_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                SCHEMA_REDACTION_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], SCHEMA_REDACTION_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        rust = consumers["rust"]
        self.assertEqual(
            rust["testFile"],
            "crates/kmipkit-protocol/tests/extension_redaction.rs",
        )
        self.assertEqual(
            rust["test"],
            "schema_error_debug_display_and_source_chain_redact_payload_secrets",
        )
        self.assertEqual(
            rust["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-protocol",
                "--test",
                "extension_redaction",
                "schema_error_debug_display_and_source_chain_redact_payload_secrets",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "c_abi_consumer_redacts_shared_fixture_diagnostic_marker",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ExtensionRegistryTest.invalidSchemaErrorsAndValidatedValueFormattingRedactPayloadText",
        )
        self.assertEqual(
            consumers["python"]["test"],
            "test_failure_and_default_diagnostics_redact_secret_fixture_values",
        )
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_extension_registry.py::ExtensionRegistryTests::"
            "test_failure_and_default_diagnostics_redact_secret_fixture_values",
        )

    def test_api_levels_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                API_LEVELS_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], API_LEVELS_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit",
                "--test",
                "api_levels",
                "api_levels_match_the_shared_fixture_without_network_calls",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_compiles_and_runs",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.SharedCapabilityParityFixtureTest.apiLevelQueryMatchesSharedFixtureLocally",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_shared_parity_fixtures.py::"
            "test_api_level_query_matches_shared_fixture_locally",
        )

    def test_open_ttlv_values_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                OPEN_TTLV_VALUES_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], OPEN_TTLV_VALUES_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-ttlv",
                "--test",
                "shared_api_parity",
                "generic_ttlv_shared_fixture_preserves_open_values_and_child_order",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "c_abi_consumer_preserves_shared_ttlv_fixture_values_and_order",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.SharedCapabilityParityFixtureTest."
            "genericTtlvFixturePreservesOpenEnumerationBytesOrderAndViewLifecycle",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_shared_parity_fixtures.py::"
            "test_generic_ttlv_shared_fixture_preserves_open_values_order_and_owner_lifecycle",
        )

    def test_extension_recognition_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                EXTENSION_RECOGNITION_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], EXTENSION_RECOGNITION_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--lib",
                "every_shared_inbound_fixture_matches_internal_outcome_ids_and_preserved_ttlv_in_both_orders",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ExtensionRegistryTest."
            "everySharedInboundFixtureRunsThroughThePublicJavaAdapterInBothOrders",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_extension_registry.py::ExtensionRegistryTests::"
            "test_shared_inbound_fixtures_are_inspected_and_generic_ttlv_is_preserved",
        )

    def test_create_request_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                CREATE_REQUEST_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], CREATE_REQUEST_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--lib",
                "create_convenience_method_maps_shared_fixture_over_one_exchange",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest."
            "executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_client.py::"
            "test_creation_response_mapping_preserves_wire_identifiers_and_failure_message",
        )

    def test_create_key_pair_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                CREATE_KEY_PAIR_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], CREATE_KEY_PAIR_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--lib",
                "create_key_pair_maps_shared_fixture_over_one_exchange",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest.executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_client.py::"
            "test_creation_response_mapping_preserves_wire_identifiers_and_failure_message",
        )

    def test_create_split_key_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                CREATE_SPLIT_KEY_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], CREATE_SPLIT_KEY_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--test",
                "creation_parity_tests",
                "create_split_key_request_and_result_match_shared_creation_fixture",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_compiles_and_runs",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.SharedCapabilityParityFixtureTest.creationSplitKeyMatchesSharedFixture",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_shared_parity_fixtures.py::"
            "test_creation_split_key_matches_shared_fixture",
        )

    def test_create_failure_message_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                CREATE_FAILURE_MESSAGE_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], CREATE_FAILURE_MESSAGE_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                "--test",
                "creation_parity_tests",
                "creation_result_message_fixture_is_limited_to_operation_failure",
                "--",
                "--exact",
            ],
        )
        self.assertEqual(
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.extensions.CreationResultValueTest."
            "failureResultMessageMatchesTheSharedFixture",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_shared_parity_fixtures.py::"
            "test_creation_result_message_matches_failure_fixture",
        )

    def test_codec_limits_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                CODEC_LIMITS_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], CODEC_LIMITS_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["test"],
            "codec_limits_shared_fixture_covers_defaults_ranges_and_rejection",
        )
        self.assertEqual(
            consumers["c"]["test"],
            "c_abi_consumer_matches_the_shared_codec_limits_fixture",
        )
        self.assertIn("coverage-c-consumer", consumers["c"]["command"])
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.SharedCapabilityParityFixtureTest."
            "codecLimitsFixtureMatchesDefaultsRangesAndErrors",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_shared_parity_fixtures.py::"
            "test_codec_limits_shared_fixture_matches_defaults_ranges_and_errors",
        )

    def test_usage_allocation_plan_selects_all_four_runtime_consumers(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--case",
                USAGE_ALLOCATION_CASE,
                "--plan",
            ],
            check=False,
            capture_output=True,
            cwd=ROOT,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads(result.stdout)
        self.assertEqual(plan["caseId"], USAGE_ALLOCATION_CASE)
        consumers = {consumer["language"]: consumer for consumer in plan["consumers"]}
        self.assertEqual(set(consumers), {"rust", "c", "java", "python"})
        self.assertEqual(
            consumers["rust"]["test"],
            "fixture_usage_allocation_guard_matches_shared_accounting_contract",
        )
        self.assertEqual(
            consumers["c"]["test"],
            "ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost",
        )
        self.assertEqual(
            consumers["c"]["command"],
            [
                "cargo",
                "test",
                "-p",
                "kmipkit-ffi",
                "--features",
                "coverage-c-consumer",
                "--lib",
                "ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost",
            ],
        )
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest."
            "usageAllocationContextGuardsJavaProtectionExecutionLocally",
        )
        self.assertEqual(consumers["java"]["workingDirectory"], "bindings/java")
        self.assertEqual(
            consumers["python"]["command"][-1],
            "bindings/python/tests/test_usage_allocation_guard_native_integration.py::"
            "test_native_python_guard_executes_encrypt_and_rejects_over_budget_before_transport",
        )

    def test_java_runner_forces_the_selected_test_to_execute(self) -> None:
        for case_id in (
            DISCOVER_VERSIONS_CASE,
            BATCH_MIXED_OUTCOMES_CASE,
            REGISTRY_ISOLATION_CASE,
            SCHEMA_REDACTION_CASE,
            API_LEVELS_CASE,
            OPEN_TTLV_VALUES_CASE,
            EXTENSION_RECOGNITION_CASE,
            CREATE_REQUEST_CASE,
            CREATE_KEY_PAIR_CASE,
            CREATE_SPLIT_KEY_CASE,
            CREATE_FAILURE_MESSAGE_CASE,
            CODEC_LIMITS_CASE,
            USAGE_ALLOCATION_CASE,
        ):
            with self.subTest(case_id=case_id):
                java_consumer = next(
                    consumer
                    for consumer in t071_parity.build_plan(case_id)["consumers"]
                    if consumer["language"] == "java"
                )
                self.assertIn("--rerun-tasks", java_consumer["command"])
                self.assertIn("--tests", java_consumer["command"])
                self.assertEqual(java_consumer["command"][-1], java_consumer["test"])

    def test_python_runner_uses_the_current_pytest_interpreter(self) -> None:
        python_consumer = next(
            consumer
            for consumer in t071_parity.build_plan(DISCOVER_VERSIONS_CASE)["consumers"]
            if consumer["language"] == "python"
        )
        self.assertEqual(
            python_consumer["command"][:3],
            [sys.executable, "-m", "pytest"],
        )

    def test_runner_rejects_a_successful_command_that_runs_no_test(self) -> None:
        completed = subprocess.CompletedProcess(args=[], returncode=0, stdout="", stderr="")
        error_output = io.StringIO()
        with (
            patch("tools.api_manifest.t071_parity.subprocess.run", return_value=completed),
            redirect_stdout(io.StringIO()),
            redirect_stderr(error_output),
        ):
            status = t071_parity.main(["--case", DISCOVER_VERSIONS_CASE])

        self.assertNotEqual(status, 0)
        self.assertIn("did not run the expected test", error_output.getvalue())

    def test_runner_rejects_gradle_success_when_expected_test_report_is_missing(self) -> None:
        java_consumer = next(
            consumer
            for consumer in t071_parity.build_plan(DISCOVER_VERSIONS_CASE)["consumers"]
            if consumer["language"] == "java"
        )
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            error_output = io.StringIO()
            completed = subprocess.CompletedProcess(
                args=java_consumer["command"], returncode=0, stdout="BUILD SUCCESSFUL", stderr=""
            )
            with (
                patch.object(t071_parity, "ROOT", root),
                patch("tools.api_manifest.t071_parity.subprocess.run", return_value=completed),
                redirect_stdout(io.StringIO()),
                redirect_stderr(error_output),
            ):
                status = t071_parity.run_plan({"consumers": [java_consumer]})

        self.assertNotEqual(status, 0)
        self.assertIn("did not run the expected test", error_output.getvalue())

    def test_gradle_report_with_junit_method_suffix_counts_as_the_expected_test(self) -> None:
        java_consumer = next(
            consumer
            for consumer in t071_parity.build_plan(DISCOVER_VERSIONS_CASE)["consumers"]
            if consumer["language"] == "java"
        )
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            report = (
                root
                / "bindings"
                / "java"
                / "build"
                / "test-results"
                / "test"
                / "TEST-org.kmipkit.ClientFacadeTlsIntegrationTest.xml"
            )
            report.parent.mkdir(parents=True)
            report.write_text(
                '<testsuite tests="1" failures="0" errors="0" skipped="0">'
                '<testcase name="executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange()"/>'
                "</testsuite>",
                encoding="utf-8",
            )
            with patch.object(t071_parity, "ROOT", root):
                self.assertTrue(t071_parity._expected_test_ran(java_consumer, "BUILD SUCCESSFUL"))


if __name__ == "__main__":
    unittest.main()
