"""Execute the T071 shared parity cases through their language consumers."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
DISCOVER_VERSIONS_CASE = "discover-versions.supported-versions.v1"
DISCOVER_VERSIONS_TESTS = {
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
}


class ParityRunnerError(ValueError):
    """Raised when a fixture does not have a supported executable consumer set."""


def _fixture_case(case_id: str) -> dict[str, Any]:
    if case_id != DISCOVER_VERSIONS_CASE:
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


def _consumer_command(language: str, consumer: dict[str, Any]) -> dict[str, Any]:
    expected_path, expected_test = DISCOVER_VERSIONS_TESTS[language]
    test_file = expected_path
    test_name = expected_test

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
            command = ["cmd.exe", "/d", "/c", str(wrapper), "test", "--tests", selector]
        else:
            command = [str(wrapper), "test", "--tests", selector]
        test_name = selector
    else:
        if consumer.get("testFile") != expected_path or consumer.get("test") != expected_test:
            raise ParityRunnerError(f"{language} fixture consumer does not match its reviewed test")
        if language == "rust":
            command = ["cargo", "test", "-p", "kmipkit-client", "--lib", test_name]
        else:
            command = [
                "uv",
                "run",
                "--project",
                "bindings/python",
                "pytest",
                "-q",
                f"{test_file}::{test_name}",
            ]

    return {"language": language, "testFile": test_file, "test": test_name, "command": command}


def build_plan(case_id: str) -> dict[str, Any]:
    case = _fixture_case(case_id)
    consumers = case.get("consumerTests")
    if not isinstance(consumers, dict) or set(consumers) != set(DISCOVER_VERSIONS_TESTS):
        raise ParityRunnerError("fixture must declare one consumer for each supported language")

    planned_consumers = []
    for language in ("rust", "c", "java", "python"):
        consumer = consumers[language]
        if not isinstance(consumer, dict) or consumer.get("status") != "available":
            raise ParityRunnerError(f"{language} fixture consumer is not available")
        planned_consumers.append(_consumer_command(language, consumer))

    return {"caseId": case_id, "consumers": planned_consumers}


def run_plan(plan: dict[str, Any]) -> int:
    for consumer in plan["consumers"]:
        language = consumer["language"]
        command = consumer["command"]
        print(f"T071 {language}: {subprocess.list2cmdline(command)}", flush=True)
        result = subprocess.run(command, cwd=ROOT, check=False)
        if result.returncode != 0:
            print(f"T071 {language} consumer failed with exit code {result.returncode}", file=sys.stderr)
            return result.returncode
    return 0


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
