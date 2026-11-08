"""Temporary threshold policy for the active Phase 5 implementation stack."""

from __future__ import annotations

import unittest

from scripts import coverage_gate


class CoverageThresholdPolicyTests(unittest.TestCase):
    def test_changed_production_scope_uses_the_approved_ninety_percent_gate(self) -> None:
        self.assertEqual(coverage_gate._CHANGED_PRODUCTION_COVERAGE_MINIMUM, 90)

    def test_protocol_crate_uses_the_approved_ninety_percent_gate(self) -> None:
        self.assertEqual(coverage_gate._RUST_CRATE_COVERAGE_MINIMUMS["kmipkit-protocol"], 90)


if __name__ == "__main__":
    unittest.main()
