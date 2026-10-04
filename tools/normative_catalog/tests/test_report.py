"""Tests for deterministic, injection-safe coverage reports."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from tools.normative_catalog.report import render_report, write_report
from tools.normative_catalog.tests.test_validate import minimal_catalog

ROOT = Path(__file__).resolve().parents[3]


def report_catalog() -> dict[str, object]:
    catalog = minimal_catalog()
    catalog["source_clauses"] = [
        {
            "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "source_id": "KMIPKIT-SRC-spec",
            "section": "8.1",
            "locator": {"ordinal": 1, "block_kind": "paragraph"},
            "source_keywords": ["MUST"],
            "normative_strength": "mandatory",
            "disposition": "requirement",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
            "exclusion_rationale": None,
        }
    ]
    catalog["requirements"] = [
        {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
            "source_keyword": "MUST",
            "normative_strength": "mandatory",
            "subject": "client",
            "summary": "Client obligation",
            "role": "client",
            "direction": "client_to_server",
            "condition": None,
            "scope_state": "client_1_0",
            "element_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
            "negative_verification_required": False,
            "decision_id": None,
            "status": "unassigned",
            "review_note": None,
        }
    ]
    return catalog


class CoverageReportTests(unittest.TestCase):
    def test_emits_fixed_sections_totals_and_unassigned_requirements(self) -> None:
        report = render_report(report_catalog())
        self.assertIn("## Source documents", report)
        self.assertIn("## Count reconciliation", report)
        self.assertIn("## Unassigned requirements", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("mandatory", report)
        self.assertTrue(report.endswith("\n"))

    def test_output_is_independent_of_record_order_and_contains_no_timestamp(self) -> None:
        first = report_catalog()
        second = report_catalog()
        second["sources"].reverse()
        second["source_clauses"].reverse()
        self.assertEqual(render_report(first), render_report(second))
        self.assertNotIn("Generated at", render_report(first))

    def test_reports_profile_states_missing_fixtures_and_open_discrepancies(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "applicability": "candidate",
                "claim_state": "evidence_incomplete",
                "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
                "test_case_ids": [],
            }
        ]
        catalog["test_cases"] = [
            {
                "test_id": "KMIPKIT-TEST-PROFILE-BASELINE-001",
                "official_case_id": "KMIP-TC-001",
                "fixture_availability": "unavailable",
                "fixture_path": None,
                "source_id": "KMIPKIT-SRC-testcases",
                "source_section": "1",
                "href": "../missing.xml",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE"],
                "requirement_ids": [],
                "element_ids": [],
            }
        ]
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-001",
                "summary": "Batch continuation wording conflict",
                "state": "open",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.5"}],
            }
        ]
        report = render_report(catalog)
        self.assertIn("evidence\\_incomplete", report)
        self.assertIn("unavailable", report)
        self.assertIn("KMIPKIT-DISC-001", report)
        self.assertIn("open", report)

    def test_escapes_markdown_html_newlines_and_control_characters(self) -> None:
        catalog = report_catalog()
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-001",
                "summary": "| `x` [evil](https://bad) ![img](https://bad)\\\r\n\x1b",
                "state": "open",
                "source_refs": [],
            }
        ]
        report = render_report(catalog)
        self.assertNotIn("][https://bad]", report)
        self.assertNotIn("![img]", report)
        self.assertIn("\\|", report)
        self.assertIn("\\`", report)
        self.assertIn("\\[", report)
        self.assertIn("\\\\", report)
        self.assertIn("\\\\u001B", report)

    def test_source_url_is_never_taken_from_untrusted_catalog_text(self) -> None:
        catalog = report_catalog()
        catalog["sources"][0]["canonical_url"] = "https://attacker.invalid/"
        report = render_report(catalog)
        self.assertNotIn("attacker.invalid", report)
        self.assertNotIn("https://", report)

    def test_script_entrypoint_works_from_a_clean_repository_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = ROOT / "specification" / "oasis" / "kmip-2.1"
            shutil.copytree(source, root / "specification" / "oasis" / "kmip-2.1")
            catalog_path = root / "specification" / "catalog" / "kmip-2.1.json"
            catalog_path.parent.mkdir(parents=True)
            catalog_path.write_text(json.dumps(minimal_catalog(), indent=2) + "\n", encoding="utf-8")
            script = Path(__file__).resolve().parents[1] / "report.py"
            result = subprocess.run(
                [sys.executable, str(script), "--write", "--repo-root", str(root)],
                cwd=root,
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / "specification" / "catalog" / "coverage-report.md").is_file())

    def test_check_detects_stale_report_and_write_replaces_it(self) -> None:
        catalog = report_catalog()
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "coverage-report.md"
            target.write_text("stale\n", encoding="utf-8")
            self.assertFalse(write_report(catalog, target, check=True))
            self.assertTrue(write_report(catalog, target, check=False))
            self.assertEqual(target.read_text(encoding="utf-8"), render_report(catalog))
            self.assertTrue(write_report(catalog, target, check=True))


if __name__ == "__main__":
    unittest.main()
