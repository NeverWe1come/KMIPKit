"""Tests for deterministic, injection-safe coverage reports."""

from __future__ import annotations

import json
import locale
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.report import _clause_section_rows, render_report, write_report
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

    def test_reports_source_clause_review_counts_per_section(self) -> None:
        report = render_report(report_catalog())

        self.assertIn("## Source clause review by section", report)
        self.assertIn("KMIPKIT-SRC-spec", report)
        self.assertIn("8.1", report)
        self.assertIn("requirement: 1", report)

    def test_numeric_section_sort_breaks_equivalent_value_ties_stably(self) -> None:
        clauses = [
            {"source_id": "source", "section": "1.1", "disposition": "requirement"},
            {"source_id": "source", "section": "1.01", "disposition": "reviewed_exclusion"},
        ]

        self.assertEqual(_clause_section_rows(clauses), _clause_section_rows(list(reversed(clauses))))

    def test_output_is_independent_of_record_order_and_contains_no_timestamp(self) -> None:
        first = report_catalog()
        second = report_catalog()
        second["sources"].reverse()
        second["source_clauses"].reverse()
        self.assertEqual(render_report(first), render_report(second))
        self.assertNotIn("Generated at", render_report(first))

    def test_output_is_independent_of_permutations_in_every_catalog_collection(self) -> None:
        first = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        second = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        for collection in second.values():
            if isinstance(collection, list):
                collection.reverse()

        self.assertEqual(render_report(first), render_report(second))

    def test_output_is_independent_of_process_locale(self) -> None:
        catalog = report_catalog()
        expected = render_report(catalog)
        with patch.object(locale, "strxfrm", side_effect=AssertionError("locale sort used")):
            self.assertEqual(render_report(catalog), expected)

    def test_report_surfaces_negative_verification_requirements(self) -> None:
        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["source_keyword"] = "MUST NOT"
        requirement["normative_strength"] = "prohibited"
        requirement["negative_verification_required"] = True
        catalog["source_clauses"][0]["source_keywords"] = ["MUST NOT"]

        report = render_report(catalog)

        self.assertIn("## Requirements needing negative verification", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("prohibited", report)
        self.assertIn("required", report)

    def test_report_lists_requirement_level_official_test_evidence_gaps(self) -> None:
        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["review_note"] = (
            "No requirement-specific official Test Cases ID is linked by the pinned sources."
        )

        report = render_report(catalog)

        self.assertIn("## Requirements without official test-case links", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("No requirement-specific official Test Cases ID", report)

    def test_reports_profile_states_missing_fixtures_and_open_discrepancies(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "applicability": "client_1_0",
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
                "affected_requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "affected_element_ids": [],
                "affected_profile_ids": [],
                "affected_policy_ids": [],
            }
        ]
        report = render_report(catalog)
        self.assertIn("evidence\\_incomplete", report)
        self.assertIn("Profiles by applicability and claim state", report)
        self.assertIn("Test evidence by fixture availability", report)
        self.assertIn("unavailable", report)
        self.assertIn("KMIPKIT-DISC-001", report)
        self.assertIn("open", report)
        self.assertIn("blocked for affected records", report)

    def test_lists_unassigned_protocol_capabilities_with_direction_and_scope(self) -> None:
        catalog = report_catalog()
        catalog["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-TAG-420001",
                "kind": "tag",
                "name": "Activation Date",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.7"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "wire_value": "420001",
                "allocation": "assigned",
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            }
        ]
        report = render_report(catalog)
        self.assertIn("## Unassigned protocol elements", report)
        self.assertIn("| Element | Name | Kind | Direction | Scope | Wire value | Allocation | Source |", report)
        self.assertIn("KMIPKIT-ELEM-TAG-420001", report)
        self.assertIn(r"client\_to\_server", report)
        self.assertIn("| KMIPKIT-ELEM-TAG-420001 | Activation Date | tag |", report)
        self.assertIn(r"| client\_1\_0 | 420001 | assigned | KMIPKIT-SRC-spec §6.1.7 |", report)

    def test_reports_tag_allocation_counts_and_each_range(self) -> None:
        catalog = report_catalog()
        catalog["elements"] = [
            {"element_id": "tag-assigned", "kind": "tag", "allocation": "assigned"},
            {"element_id": "tag-reserved", "kind": "tag", "allocation": "reserved"},
        ]
        catalog["tag_ranges"] = [
            {
                "range_id": "KMIPKIT-RANGE-001",
                "value_range": "420001 - 4200FF",
                "allocation": "unused",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
        ]

        report = render_report(catalog)

        self.assertIn("## Tag allocation and ranges", report)
        self.assertIn("| assigned | 1 |", report)
        self.assertIn("| reserved | 1 |", report)
        self.assertIn("| KMIPKIT-RANGE-001 | 420001 - 4200FF | unused |", report)

    def test_profile_report_includes_traceability_and_transport_contract(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-5.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-PROF-5.1-001"],
                "dependency_profile_ids": ["KMIPKIT-PROFILE-OTHER"],
                "test_case_ids": ["KMIPKIT-TEST-PROF-5-1-1"],
                "transport_requirements": ["TLS 1.3"],
                "encoding_requirements": ["TTLV"],
            }
        ]

        report = render_report(catalog)

        self.assertIn("Source clauses", report)
        self.assertIn("Requirements", report)
        self.assertIn("Dependencies", report)
        self.assertIn("Test cases", report)
        self.assertIn("Transport", report)
        self.assertIn("Encoding", report)
        self.assertIn("KMIPKIT-CLAUSE-PROF-5.1-001", report)
        self.assertIn("KMIPKIT-REQ-PROF-5.1-001", report)
        self.assertIn("KMIPKIT-PROFILE-OTHER", report)
        self.assertIn("KMIPKIT-TEST-PROF-5-1-1", report)
        self.assertIn("TLS 1.3", report)
        self.assertIn("TTLV", report)

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
            for arguments in (
                ["git", "init", "--quiet", "--initial-branch=main"],
                ["git", "config", "user.name", "KMIPKit test"],
                ["git", "config", "user.email", "tests@example.invalid"],
                ["git", "add", "specification/oasis"],
                ["git", "commit", "--quiet", "-m", "pinned test evidence"],
            ):
                subprocess.run(arguments, cwd=root, check=True, capture_output=True)
            script = Path(__file__).resolve().parents[1] / "report.py"
            result = subprocess.run(
                [sys.executable, str(script), "--write", "--repo-root", str(root), "--structural-only"],
                cwd=root,
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            report = (root / "specification" / "catalog" / "coverage-report.md").read_text(encoding="utf-8")
            self.assertIn("KMIPKIT-SRC-profiles", report)
            self.assertIn("| Sources | 4 |", report)

    def test_check_detects_stale_report_and_write_replaces_it(self) -> None:
        catalog = report_catalog()
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "coverage-report.md"
            target.write_text("stale\n", encoding="utf-8")
            self.assertFalse(write_report(catalog, target, check=True))
            self.assertTrue(write_report(catalog, target, check=False))
            self.assertEqual(target.read_text(encoding="utf-8"), render_report(catalog))
            self.assertTrue(write_report(catalog, target, check=True))

    @unittest.skipUnless(sys.platform != "win32", "POSIX directory symlinks are required")
    def test_write_report_accepts_a_resolved_temporary_parent_alias(self) -> None:
        catalog = report_catalog()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "canonical"
            root.mkdir()
            alias = Path(directory) / "alias"
            alias.symlink_to(root, target_is_directory=True)
            target = alias / "coverage-report.md"

            self.assertFalse(write_report(catalog, target, check=True))
            self.assertTrue(write_report(catalog, target, check=False))
            self.assertEqual(target.read_text(encoding="utf-8"), render_report(catalog))


if __name__ == "__main__":
    unittest.main()
