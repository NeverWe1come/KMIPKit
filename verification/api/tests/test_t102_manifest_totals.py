"""Keep current and historical T102 Rust audit snapshots distinct."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from verification.api.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_HISTORICAL_RUST_AUDIT_EXCLUDED,
    T102_HISTORICAL_RUST_AUDIT_MANIFEST,
    T102_HISTORICAL_RUST_AUDIT_OBSERVED,
    T102_PUBLIC_DECLARATIONS,
)


ROOT = Path(__file__).resolve().parents[3]
HISTORICAL_REPORT = (
    ROOT / "verification" / "api" / "tests" / "fixtures" / "public-api" / "rust-export-inventory.json"
)
MANIFEST = ROOT / "bindings" / "shared" / "public-api.json"


class T102ManifestTotalsTests(unittest.TestCase):
    def test_manifest_totals_match_the_current_generated_catalog(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(manifest["approvedExclusions"]))

    def test_historical_rust_audit_totals_match_the_checked_in_fixture(self) -> None:
        historical = json.loads(HISTORICAL_REPORT.read_text(encoding="utf-8"))["counts"]

        self.assertEqual(
            (
                T102_HISTORICAL_RUST_AUDIT_MANIFEST,
                T102_HISTORICAL_RUST_AUDIT_EXCLUDED,
                T102_HISTORICAL_RUST_AUDIT_OBSERVED,
            ),
            (historical["manifest"], historical["excluded"], historical["observed"]),
        )


if __name__ == "__main__":
    unittest.main()
