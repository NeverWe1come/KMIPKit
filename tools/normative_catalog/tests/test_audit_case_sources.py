"""Regression tests for the offline official case source audit."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

from tools.normative_catalog import audit_case_sources
from tools.normative_catalog.safe_io import PathSecurityError


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

    def test_rejects_data_href_as_the_case_href_attribute(self) -> None:
        result = self._run_audit(
            profile_html=(
                b'<h4>5.1.3.1 BL-M-1-21</h4><p><a data-href="case.xml">fixture</a></p>'
            ),
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("section, official ID, and exact href do not match", result.stderr)

    def test_rejects_href_text_inside_another_quoted_attribute(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.1.3.1 BL-M-1-21</h4>"
                b"<p><a title='label href=\"case.xml\"'>fixture</a></p>"
            ),
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("section, official ID, and exact href do not match", result.stderr)

    def test_uses_real_href_after_decoy_text_in_a_quoted_attribute(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.1.3.1 BL-M-1-21</h4>"
                b"<p><a title='label href=\"decoy.xml\"' href=\"case.xml\">fixture</a></p>"
            ),
            testcases_html=b"",
            cases=[
                self._case("P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1", "profile_test_case", "case.xml"),
            ],
        )

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_duplicate_href_links_are_deduplicated_and_remain_matchable(self) -> None:
        profile_html = (
            b"<h4>5.1.3.1 BL-M-1-21</h4>"
            b'<a href="case.xml">first</a><a href="case.xml">duplicate</a>'
        )
        result = self._run_audit(
            profile_html=profile_html,
            testcases_html=b"",
            cases=[
                self._case(
                    "P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1",
                    "profile_test_case", "case.xml",
                ),
            ],
        )
        evidence = audit_case_sources._parse_case_evidence(profile_html, {"BL-M-1-21"})

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(evidence), 1)
        self.assertEqual(evidence[0].hrefs, {"case.xml"})
        self.assertIsInstance(evidence[0].hrefs, set)

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

    def test_translates_safe_io_path_errors_to_redacted_catalog_errors(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            (repository / "catalog.json").write_text("{}", encoding="utf-8")
            path_marker = "private/path/marker"

            with patch.object(
                audit_case_sources,
                "safe_read_bytes",
                side_effect=PathSecurityError(path_marker),
                create=True,
            ) as safe_read:
                with self.assertRaises(audit_case_sources.CaseAuditError) as error:
                    audit_case_sources._load_catalog(repository, "catalog.json")

            self.assertEqual(str(error.exception), "catalog is unavailable or invalid JSON")
            self.assertNotIn(path_marker, str(error.exception))
            safe_read.assert_called_once_with(
                repository,
                "catalog.json",
                max_bytes=audit_case_sources.MAX_CATALOG_BYTES,
            )

    def test_rejects_catalog_source_redirect_with_self_consistent_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            profile_html = b'<h4>5.1.3.1 BL-M-1-21</h4><a href="case.xml">fixture</a>'
            testcases_html = b""
            profile_digest = hashlib.sha256(profile_html).hexdigest()
            testcases_digest = hashlib.sha256(testcases_html).hexdigest()
            catalog = {
                "sources": [
                    {
                        "source_id": "KMIPKIT-SRC-profiles",
                        "local_path": "attacker/synthetic.html",
                        "sha256": profile_digest,
                    },
                    {
                        "source_id": "KMIPKIT-SRC-testcases",
                        "local_path": "canonical/testcases.html",
                        "sha256": testcases_digest,
                    },
                ],
                "test_cases": [
                    self._case(
                        "P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1",
                        "profile_test_case", "case.xml",
                    ),
                ],
            }
            manifest = {
                "KMIPKIT-SRC-profiles": {
                    "local_path": "canonical/profiles.html",
                    "sha256": "a" * 64,
                },
                "KMIPKIT-SRC-testcases": {
                    "local_path": "canonical/testcases.html",
                    "sha256": testcases_digest,
                },
            }

            def read_synthetic_source(root: Path, path: str, *, max_bytes: int) -> bytes:
                del root, max_bytes
                return profile_html if path == "attacker/synthetic.html" else testcases_html

            with (
                patch.object(
                    audit_case_sources,
                    "_source_manifest",
                    return_value=manifest,
                    create=True,
                ) as source_manifest,
                patch.object(audit_case_sources, "safe_read_bytes", side_effect=read_synthetic_source) as safe_read,
            ):
                with self.assertRaises(audit_case_sources.CaseAuditError) as error:
                    audit_case_sources.audit_catalog_cases(repository, catalog)

            self.assertEqual(
                str(error.exception),
                "catalog source metadata does not match the pinned OASIS source manifest",
            )
            source_manifest.assert_called_once_with(repository)
            safe_read.assert_not_called()

    def test_translates_recursion_error_during_catalog_json_load(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            nested_json = "[" * 5_000 + "0" + "]" * 5_000
            (repository / "catalog.json").write_text(nested_json, encoding="utf-8")

            with self.assertRaises(audit_case_sources.CaseAuditError) as error:
                audit_case_sources._load_catalog(repository, "catalog.json")

            self.assertEqual(str(error.exception), "catalog is unavailable or invalid JSON")

    def test_rejects_symlinked_pinned_source_without_disclosing_target(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            sources = repository / "sources"
            sources.mkdir()
            profile_html = b'<h4>5.1.3.1 BL-M-1-21</h4><a href="case.xml">fixture</a>'
            profile_target = repository / "external-profiles.html"
            profile_target.write_bytes(profile_html)
            try:
                (sources / "profiles.html").symlink_to(profile_target)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            testcases_html = b""
            (sources / "testcases.html").write_bytes(testcases_html)
            catalog = {
                "sources": [
                    {
                        "source_id": "KMIPKIT-SRC-profiles",
                        "local_path": "sources/profiles.html",
                        "sha256": hashlib.sha256(profile_html).hexdigest(),
                    },
                    {
                        "source_id": "KMIPKIT-SRC-testcases",
                        "local_path": "sources/testcases.html",
                        "sha256": hashlib.sha256(testcases_html).hexdigest(),
                    },
                ],
                "test_cases": [
                    self._case(
                        "P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1",
                        "profile_test_case", "case.xml",
                    ),
                ],
            }
            (repository / "catalog.json").write_text(json.dumps(catalog), encoding="utf-8")
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--repo-root",
                    str(repository),
                    "--catalog",
                    "catalog.json",
                ],
                cwd=ROOT,
                capture_output=True,
                check=False,
                text=True,
            )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("pinned source cannot be read safely: KMIPKIT-SRC-profiles", result.stderr)
        self.assertNotIn(str(profile_target), result.stderr)

    def test_many_ids_and_rows_keep_candidate_and_evidence_visits_linear(self) -> None:
        case_count = 120
        rows: list[dict[str, str]] = []
        html_parts: list[str] = []
        for index in range(case_count):
            official_id = f"CASE-{index:03d}-21"
            section = f"5.{index + 1}.1"
            href = f"cases/{official_id}.xml"
            html_parts.append(
                f'<h4>{section} {official_id}</h4><a href="{href}">fixture</a>'
            )
            rows.append(
                self._case(
                    f"P-{index}", official_id, "KMIPKIT-SRC-profiles", section,
                    "profile_test_case", href,
                )
            )
        profile_html = "".join(html_parts).encode("utf-8")
        testcases_html = b""

        class CountingIds(set[str]):
            item_visits = 0

            def __iter__(self):
                for item in super().__iter__():
                    type(self).item_visits += 1
                    yield item

        class CountingEvidence(list):
            item_visits = 0

            def __iter__(self):
                for item in super().__iter__():
                    type(self).item_visits += 1
                    yield item

        with tempfile.TemporaryDirectory() as temporary_directory:
            repository = Path(temporary_directory)
            sources = repository / "sources"
            sources.mkdir()
            (sources / "profiles.html").write_bytes(profile_html)
            (sources / "testcases.html").write_bytes(testcases_html)
            catalog = {
                "sources": [
                    {
                        "source_id": "KMIPKIT-SRC-profiles",
                        "local_path": "sources/profiles.html",
                        "sha256": hashlib.sha256(profile_html).hexdigest(),
                    },
                    {
                        "source_id": "KMIPKIT-SRC-testcases",
                        "local_path": "sources/testcases.html",
                        "sha256": hashlib.sha256(testcases_html).hexdigest(),
                    },
                ],
                "test_cases": rows,
            }
            parse_case_evidence = audit_case_sources._parse_case_evidence

            def counted_parse(raw: bytes, official_ids: set[str]) -> CountingEvidence:
                evidence = parse_case_evidence(raw, CountingIds(official_ids))
                return CountingEvidence(evidence)

            with patch.object(
                audit_case_sources,
                "_parse_case_evidence",
                side_effect=counted_parse,
            ):
                counts, divergences = audit_case_sources.audit_catalog_cases(repository, catalog)

        self.assertEqual(counts["profile_test_case"], case_count)
        self.assertEqual(divergences, [])
        self.assertLessEqual(CountingIds.item_visits, case_count * 2)
        self.assertLessEqual(CountingEvidence.item_visits, case_count * 2)

    def test_rejects_duplicate_headings_for_the_same_section_and_case(self) -> None:
        result = self._run_audit(
            profile_html=(
                b"<h4>5.1.3.1 BL-M-1-21</h4><a href=\"case.xml\">first</a>"
                b"<h4>5.1.3.1 BL-M-1-21</h4><a href=\"case.xml\">duplicate</a>"
            ),
            testcases_html=b"",
            cases=[
                self._case(
                    "P-1", "BL-M-1-21", "KMIPKIT-SRC-profiles", "5.1.3.1",
                    "profile_test_case", "case.xml",
                ),
            ],
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("section, official ID, and exact href do not match", result.stderr)

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
