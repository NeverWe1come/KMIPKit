"""Contract tests for planning T071 parity runs across language consumers."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "tools" / "api_manifest" / "t071_parity.py"
DISCOVER_VERSIONS_CASE = "discover-versions.supported-versions.v1"


class T071ParityRunnerTests(unittest.TestCase):
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
            consumers["c"]["test"],
            "generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls",
        )
        self.assertEqual(
            consumers["java"]["test"],
            "org.kmipkit.ClientFacadeTlsIntegrationTest.executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange",
        )
        self.assertEqual(
            consumers["python"]["test"],
            "test_discover_versions_response_mapping_matches_shared_fixture",
        )


if __name__ == "__main__":
    unittest.main()
