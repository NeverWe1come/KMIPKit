"""Regression tests for the offline official case source audit."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools" / "normative_catalog" / "audit_case_sources.py"


class CaseSourceAuditTests(unittest.TestCase):
    def _run_audit(
        self,
        *,
        profile_html: bytes,
        testcases_html: bytes,
        cases: list[dict[str, str]],
        profile_sha256: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            profile_path = "sources/profiles.html"
            testcases_path = "sources/testcases.html"
            (repository / "sources").mkdir()
            (repository / profile_path).write_bytes(profile_html)
            (repository / testcases_path).write_bytes(testcases_html)
            catalog = {
                "sources": [
                    {
                        "source_id": "KMIPKIT-SRC-profiles",
                        "local_path": profile_path,
                        "sha256": profile_sha256 or hashlib.sha256(profile_html).hexdigest(),
                    },
                    {
                        "source_id": "KMIPKIT-SRC-testcases",
                        "local_path": testcases_path,
                        "sha256": hashlib.sha256(testcases_html).hexdigest(),
                    },
                ],
                "test_cases": cases,
            }
            (repository / "catalog.json").write_text(json.dumps(catalog), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(SCRIPT), "--repo-root", str(repository), "--catalog", "catalog.json"],
                cwd=ROOT,
                capture_output=True,
                check=False,
                text=True,
            )

    @staticmethod
    def _case(
        test_id: str,
        official_id: str,
        source_id: str,
        section: str,
        category: str,
        href: str,
    ) -> dict[str, str]:
        return {
            "test_id": test_id,
            "official_case_id": official_id,
            "source_id": source_id,
            "source_section": section,
            "evidence_category": category,
            "raw_href": href,
        }

    def test_binds_each_category_to_its_source_and_matches_section_heading_and_href(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.1.3.1 BL-M-1-21</h4>"
                b'<p><a href="test-cases/BL-M-1-21.xml">fixture</a></p>'
            ),
            testcases_html=(
                b"<h2>2.1 TC-ASYNC-1-21</h2>"
                b'<p><a href="test-cases/TC-ASYNC-1-21.xml">fixture</a></p>'
            ),
            cases=[
                self._case(
                    "P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1",
                    "profile_test_case", "test-cases/BL-M-1-21.xml",
                ),
                self._case(
                    "T-1", "TC-ASYNC-1-21", "KMIPKIT-SRC-testcases", "2.1",
                    "official_conformance_test", "test-cases/TC-ASYNC-1-21.xml",
                ),
            ],
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("profile_test_case: 1 rows", result.stdout)
        self.assertIn("official_conformance_test: 1 rows", result.stdout)
        self.assertIn("Catalog-to-source mismatches: 0", result.stdout)

    def test_reports_source_internal_href_name_divergence_without_failing_match(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.9.7.13 CS-BC-M-13-21</h4>"
                b'<p><a href="test-cases/CS-BC-M-14-21.xml">fixture</a></p>'
            ),
            testcases_html=b"",
            cases=[
                self._case(
                    "P-13", "CS-BC-M-13-21", "KMIPKIT-SRC-profiles", "5.9.7.13",
                    "profile_test_case", "test-cases/CS-BC-M-14-21.xml",
                ),
            ],
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Catalog-to-source mismatches: 0", result.stdout)
        self.assertIn("Source-internal official-ID/href-name divergences: 1", result.stdout)
        self.assertIn("CS-BC-M-13-21 -> CS-BC-M-14-21.xml", result.stdout)

    def test_rejects_case_category_bound_to_the_wrong_catalog_source(self) -> None:
        result = self._run_audit(
            profile_html=b"<h4>5.1.3.1 BL-M-1-21</h4><a href=\"case.xml\">fixture</a>",
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-testcases", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("evidence category is bound to a different source ID", result.stderr)

    def test_requires_the_catalog_href_in_the_matching_html_section(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.1.3.1 BL-M-1-21</h4><p>no link here</p>"
                b"<h4>5.1.3.2 BL-M-2-21</h4><a href=\"case.xml\">other section</a>"
            ),
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("section, official ID, and exact href do not match", result.stderr)

    def test_verifies_catalog_source_sha256_before_parsing(self) -> None:
        result = self._run_audit(
            profile_html=b"<h4>5.1.3.1 BL-M-1-21</h4><a href=\"case.xml\">fixture</a>",
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
            profile_sha256="0" * 64,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 does not match the catalog source record", result.stderr)

    def test_audits_all_pinned_catalogued_cases_offline(self) -> None:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--repo-root", str(ROOT)],
            cwd=ROOT,
            capture_output=True,
            check=False,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("profile_test_case: 93 rows", result.stdout)
        self.assertIn("official_conformance_test: 110 rows", result.stdout)
        self.assertIn("Catalog-to-source mismatches: 0", result.stdout)
        self.assertIn("Source-internal official-ID/href-name divergences: 10", result.stdout)


if __name__ == "__main__":
    unittest.main()
