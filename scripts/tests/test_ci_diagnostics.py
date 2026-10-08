"""Contracts for clear per-job diagnostics after GitHub Actions failures."""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from unittest import mock


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
DIAGNOSTICS_PATH = REPOSITORY_ROOT / "scripts" / "ci_diagnostics.py"
DIAGNOSTICS = None
DIAGNOSTICS_LOAD_ERROR = None
if DIAGNOSTICS_PATH.is_file():
    try:
        SPEC = importlib.util.spec_from_file_location("ci_diagnostics", DIAGNOSTICS_PATH)
        if SPEC is None or SPEC.loader is None:
            raise ImportError("ci_diagnostics.py has no importable module loader")
        DIAGNOSTICS = importlib.util.module_from_spec(SPEC)
        SPEC.loader.exec_module(DIAGNOSTICS)
    except Exception as error:  # Keep the RED test failure useful when the module is absent.
        DIAGNOSTICS_LOAD_ERROR = error


class CiDiagnosticsTests(unittest.TestCase):
    def require_diagnostics(self):
        self.assertIsNotNone(
            DIAGNOSTICS,
            f"ci_diagnostics.py must provide the tested contract; load error: {DIAGNOSTICS_LOAD_ERROR}",
        )
        return DIAGNOSTICS

    def test_failing_step_emits_specific_explanation_and_identifies_job_context(self) -> None:
        diagnostics = self.require_diagnostics()
        steps = {
            "install-nightly": {"outcome": "success", "conclusion": "success"},
            "run-fuzz-target": {"outcome": "failure", "conclusion": "failure"},
        }

        markdown, records = diagnostics.build_diagnostics(
            job_id="fuzz-smoke",
            matrix={"os": "ubuntu-latest"},
            steps=steps,
        )

        self.assertIn("Fuzz smoke", markdown)
        self.assertIn("ubuntu-latest", markdown)
        self.assertIn("fuzz target", markdown.lower())
        self.assertIn("crash", markdown.lower())
        self.assertEqual("run-fuzz-target", records[0]["step_id"])

    def test_all_failed_steps_are_reported_without_copying_raw_tool_output(self) -> None:
        diagnostics = self.require_diagnostics()
        steps = {
            "run-clippy": {"outcome": "failure", "conclusion": "failure"},
            "run-workspace-tests": {"outcome": "failure", "conclusion": "failure"},
            "ci-diagnostics": {"outcome": "success", "conclusion": "success"},
        }

        markdown, records = diagnostics.build_diagnostics(
            job_id="core",
            matrix={"os": "windows-latest", "rust": "1.94"},
            steps=steps,
        )

        self.assertEqual(2, len(records))
        self.assertIn("Clippy", markdown)
        self.assertIn("workspace tests", markdown.lower())
        self.assertIn("open the failed step's log", markdown.lower())
        self.assertNotIn("raw output", markdown.lower())

    def test_continue_on_error_step_is_still_described_without_gating_the_job(self) -> None:
        diagnostics = self.require_diagnostics()

        markdown, records = diagnostics.build_diagnostics(
            job_id="branch-coverage",
            matrix={},
            steps={
                "branch-report": {"outcome": "failure", "conclusion": "success"},
            },
        )

        self.assertEqual(1, len(records))
        self.assertIn("informational", markdown.lower())
        self.assertIn("branch coverage", markdown.lower())

    def test_diagnostic_values_are_markdown_escaped(self) -> None:
        diagnostics = self.require_diagnostics()

        markdown, _ = diagnostics.build_diagnostics(
            job_id="<img src=x>",
            matrix={"os": "ubuntu|injected"},
            steps={"run-<bad>": {"outcome": "failure"}},
        )

        self.assertNotIn("<img", markdown)
        self.assertNotIn("|injected", markdown)
        self.assertNotIn("<bad>", markdown)

    def test_main_writes_job_summary_and_json_output_for_downstream_summary(self) -> None:
        diagnostics = self.require_diagnostics()
        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "summary.md"
            output_path = Path(directory) / "output.txt"
            environment = {
                "CI_JOB_ID": "ffi-sanitizer",
                "CI_MATRIX_JSON": "{}",
                "CI_STEPS_JSON": json.dumps(
                    {"run-native-sanitizers": {"outcome": "failure", "conclusion": "failure"}}
                ),
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_OUTPUT": str(output_path),
            }

            console = StringIO()
            with mock.patch.dict(os.environ, environment, clear=False), redirect_stdout(console):
                exit_code = diagnostics.main()

            self.assertEqual(0, exit_code)
            self.assertIn("Run Native Sanitizers", summary_path.read_text(encoding="utf-8"))
            outputs = output_path.read_text(encoding="utf-8")
            self.assertIn("diagnostic_details=", outputs)
            self.assertIn("AddressSanitizer", outputs)
            self.assertIn("Failure diagnosis", console.getvalue())
            self.assertIn("sanitizer regression failed", console.getvalue())


if __name__ == "__main__":
    unittest.main(verbosity=2)
