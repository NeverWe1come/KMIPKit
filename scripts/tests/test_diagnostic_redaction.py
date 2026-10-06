"""Security tests for cargo-deny diagnostic output redaction."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
POLICY_PATH = REPOSITORY_ROOT / "scripts" / "dependency_policy.py"
SPEC = importlib.util.spec_from_file_location("dependency_policy_diagnostic_redaction", POLICY_PATH)
if SPEC is None or SPEC.loader is None:
    raise ImportError("dependency policy module has no importable module loader")
POLICY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(POLICY)


class DiagnosticRedactionTests(unittest.TestCase):
    def test_license_evidence_cannot_inject_report_lines(self) -> None:
        raw_output = (
            '{"type":"diagnostic","fields":{"severity":"error","code":"rejected",'
            '"labels":[{"span":"GPL-3.0-only\\nOR MIT"}],'
            '"graphs":[{"Krate":{"name":"safe-package","version":"1.0.0"}}]}}'
        )

        report = POLICY.format_cargo_deny_diagnostics(
            raw_output,
            {"root": {"packages": []}, "fuzz": {"packages": []}},
        )

        self.assertEqual(len(report.splitlines()), 1)
        self.assertIn("safe-package@1.0.0", report)
        self.assertNotIn("leaked-package", report)
        self.assertNotIn("secret-token", report)


if __name__ == "__main__":
    unittest.main(verbosity=2)
