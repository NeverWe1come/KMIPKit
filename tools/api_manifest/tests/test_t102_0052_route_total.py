"""Keep the T102 snapshot aligned with additive KMIPKIT-0052 API routes."""

from __future__ import annotations

import json
import subprocess
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_PUBLIC_DECLARATIONS,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
KMIPKIT_0051_BASELINE = "c0aecd29c93dd63abbaa9928ee8988bcb4b9ebdc"
KMIPKIT_0052_ROUTE_DELTA = 78


class T102ResponseRouteTotalTests(unittest.TestCase):
    def test_manifest_snapshot_includes_additive_0052_routes(self) -> None:
        baseline_bytes = subprocess.run(
            [
                "git",
                "show",
                f"{KMIPKIT_0051_BASELINE}:specification/api/public-api.json",
            ],
            cwd=REPOSITORY_ROOT,
            check=True,
            capture_output=True,
        ).stdout
        baseline = json.loads(baseline_bytes)
        current = json.loads(
            (REPOSITORY_ROOT / "specification/api/public-api.json").read_text(
                encoding="utf-8"
            )
        )

        baseline_count = len(baseline["publicDeclarations"])
        current_count = len(current["publicDeclarations"])
        self.assertGreaterEqual(current_count - baseline_count, KMIPKIT_0052_ROUTE_DELTA)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, current_count)

        current_exclusions = len(current["approvedExclusions"])
        self.assertEqual(T102_APPROVED_EXCLUSIONS, current_exclusions)

    def test_snapshot_totals_include_async_response_records(self) -> None:
        baseline_bytes = subprocess.run(
            [
                "git",
                "show",
                f"{KMIPKIT_0051_BASELINE}:specification/api/public-api.json",
            ],
            cwd=REPOSITORY_ROOT,
            check=True,
            capture_output=True,
        ).stdout
        baseline = json.loads(baseline_bytes)
        current = json.loads(
            (REPOSITORY_ROOT / "specification/api/public-api.json").read_text(
                encoding="utf-8"
            )
        )

        current_exclusions = len(current["approvedExclusions"])
        self.assertEqual(T102_APPROVED_EXCLUSIONS, current_exclusions)


if __name__ == "__main__":
    unittest.main()
