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
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
DISCOVER_VERSIONS_CASE = "discover-versions.supported-versions.v1"
MIXED_BATCH_OUTCOMES_CASE = "batch.ordered-completed-pending-failed.v1"
REGISTRY_ISOLATION_CASE = "client-configuration.registry-isolation.v1"
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
}


class ParityRunnerError(ValueError):
    """Raised when a fixture does not have a supported executable consumer set."""


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
        command = [
            "cargo",
            "test",
            "-p",
            "kmipkit-ffi",
            "--features",
            "coverage-c-consumer",
            "--test",
            "c_api_coverage",
            test_name,
            "--",
            "--exact",
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
            test_path = Path(expected_path)
            if test_path.parent == Path("crates/kmipkit-client/tests"):
                target = ["--test", test_path.stem]
                exact_filter = ["--", "--exact"]
            else:
                target = ["--lib"]
                exact_filter = []
            command = [
                "cargo",
                "test",
                "-p",
                "kmipkit-client",
                *target,
                test_name,
                *exact_filter,
            ]
        else:
            command = [
                sys.executable,
                "-m",
                "pytest",
                "-q",
                f"{test_file}::{test_name}",
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
