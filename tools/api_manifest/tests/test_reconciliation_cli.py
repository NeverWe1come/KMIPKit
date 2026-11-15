"""Tests for the shared manifest reconciliation command-line runner."""

from __future__ import annotations

import io
import json
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from typing import Any

from tools.api_manifest.reconciliation_cli import run_reconciliation_cli


class ReconciliationCliTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.root = Path(self.temporary_directory.name)
        manifest_path = self.root / "specification" / "api" / "public-api.json"
        manifest_path.parent.mkdir(parents=True)
        self.original_manifest = {
            "publicDeclarations": [],
            "approvedExclusions": [],
        }
        manifest_path.write_text(json.dumps(self.original_manifest), encoding="utf-8")
        self.manifest_path = manifest_path
        self.report_path = self.root / "report.json"
        self.report = {
            "publicDeclarations": [{"declarationId": "new-route"}],
            "approvedExclusions": [],
        }
        self.report_path.write_text(json.dumps(self.report), encoding="utf-8")

    @staticmethod
    def _reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
        result = dict(manifest)
        result["publicDeclarations"] = [
            *manifest["publicDeclarations"],
            *report["publicDeclarations"],
        ]
        result["approvedExclusions"] = [
            *manifest["approvedExclusions"],
            *report["approvedExclusions"],
        ]
        return result

    def _run(self, *options: str) -> tuple[int, str, str]:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            result = run_reconciliation_cli(
                self._reconcile,
                spec_id="KMIPKIT-TEST",
                description="test reconciliation",
                summarize=lambda added, excluded, write: (
                    f"added={added}, excluded={excluded}, write={write}"
                ),
                argv=[
                    "--repo-root",
                    str(self.root),
                    "--report",
                    str(self.report_path),
                    *options,
                ],
            )
        return result, stdout.getvalue(), stderr.getvalue()

    def test_dry_run_reports_changes_without_writing_manifest(self) -> None:
        result, stdout, stderr = self._run()

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        self.assertIn("added=1", stdout)
        self.assertEqual(
            json.loads(self.manifest_path.read_text(encoding="utf-8")),
            self.original_manifest,
        )

    def test_write_option_persists_reconciled_manifest(self) -> None:
        result, stdout, stderr = self._run("--write")

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        self.assertIn("write=True", stdout)
        saved = json.loads(self.manifest_path.read_text(encoding="utf-8"))
        self.assertEqual(saved["publicDeclarations"], self.report["publicDeclarations"])

    def test_invalid_report_returns_scoped_error(self) -> None:
        self.report_path.write_text("{invalid", encoding="utf-8")

        result, _stdout, stderr = self._run()

        self.assertEqual(result, 2)
        self.assertIn("KMIPKIT-TEST manifest reconciliation failed", stderr)


if __name__ == "__main__":
    unittest.main()
