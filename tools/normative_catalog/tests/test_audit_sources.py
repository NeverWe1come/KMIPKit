"""Tests for bounded, offline OASIS normative-candidate extraction."""

from __future__ import annotations

import unittest
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.audit_sources import SourceAuditError, audit_document, audit_git_sources

ROOT = Path(__file__).resolve().parents[3]


class SourceAuditTests(unittest.TestCase):
    def test_concatenates_inline_text_and_detects_compound_keywords(self) -> None:
        candidates = audit_document(
            b"<h2>8.1 Example</h2><p>The client MUST <b>NOT</b> repeat.</p>",
            "KMIPKIT-SRC-spec",
        )
        self.assertEqual(candidates[0]["source_keywords"], ["MUST NOT"])
        self.assertEqual(candidates[0]["section"], "8.1")

    def test_uses_case_insensitive_whole_word_matching(self) -> None:
        candidates = audit_document(
            b"<h2>1 Scope</h2><p>MUSTARD is not a keyword; may is one.</p>",
            "KMIPKIT-SRC-spec",
        )
        self.assertEqual(candidates[0]["source_keywords"], ["MAY"])

    def test_candidate_boundaries_do_not_double_count_nested_paragraphs_or_cells(self) -> None:
        html = (
            b"<h2>2 Scope</h2>"
            b"<ul><li><p>The client MUST comply.</p></li></ul>"
            b"<table><tr><td>Servers</td><td>SHALL comply</td></tr></table>"
            b"<dl><dt>MAY be optional</dt><dd>SHOULD be retained</dd></dl>"
        )
        candidates = audit_document(html, "KMIPKIT-SRC-spec")
        self.assertEqual([item["locator"]["block_kind"] for item in candidates], [
            "list_item", "table_row", "definition_item", "definition_item",
        ])
        self.assertEqual(len({item["clause_id"] for item in candidates}), 4)

    def test_uses_declared_windows_1252_charset_independent_of_locale(self) -> None:
        raw = b'<meta charset="windows-1252"><h2>3 Scope</h2><p>Caf\xe9 MUST work.</p>'
        self.assertEqual(audit_document(raw, "KMIPKIT-SRC-spec")[0]["source_keywords"], ["MUST"])

    def test_reads_charset_from_pinned_oasis_content_type_metadata(self) -> None:
        raw = b'<meta http-equiv=Content-Type content="text/html; charset=windows-1252"><p>Caf\xe9 MUST work.</p>'
        self.assertEqual(audit_document(raw, "KMIPKIT-SRC-spec")[0]["source_keywords"], ["MUST"])

    def test_rejects_malformed_or_unsupported_charset(self) -> None:
        for raw in (
            b'<meta charset="unknown-codec"><p>MUST work.</p>',
            b'<meta charset=""><p>MUST work.</p>',
        ):
            with self.subTest(raw=raw), self.assertRaises(SourceAuditError):
                audit_document(raw, "KMIPKIT-SRC-spec")

    def test_emits_stable_section_local_structural_ids(self) -> None:
        html = b"<h2>4.2 Example</h2><p>MUST one.</p><p>MAY two.</p>"
        first = audit_document(html, "KMIPKIT-SRC-spec")
        second = audit_document(html, "KMIPKIT-SRC-spec")
        self.assertEqual(first, second)
        self.assertEqual([item["clause_id"] for item in first], [
            "KMIPKIT-CLAUSE-SPEC-4.2-001", "KMIPKIT-CLAUSE-SPEC-4.2-002",
        ])

    def test_ignores_base_links_and_external_resources_without_fetching(self) -> None:
        html = (
            b'<base href="https://example.invalid/"><img src="secret">'
            b'<h2>5 Scope</h2><p>MUST remain offline.</p>'
        )
        self.assertEqual(len(audit_document(html, "KMIPKIT-SRC-spec")), 1)

    def test_rejects_oversized_documents_and_excessive_html_depth(self) -> None:
        with self.assertRaises(SourceAuditError):
            audit_document(b" " * (8 * 1024 * 1024 + 1), "KMIPKIT-SRC-spec")
        deeply_nested = b"<div>" * 65 + b"MUST" + b"</div>" * 65
        with self.assertRaises(SourceAuditError):
            audit_document(deeply_nested, "KMIPKIT-SRC-spec")

    def test_script_entrypoint_audits_the_local_pinned_sources(self) -> None:
        base_sha = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, check=True, capture_output=True, text=True
        ).stdout.strip()
        script = ROOT / "tools" / "normative_catalog" / "audit_sources.py"
        result = subprocess.run(
            [sys.executable, str(script), "--base-sha", base_sha, "--repo-root", str(ROOT)],
            cwd=ROOT,
            capture_output=True,
            check=False,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("normative source candidates", result.stdout)

    def test_git_audit_does_not_read_normative_documents_from_working_tree(self) -> None:
        base_sha = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, check=True, capture_output=True, text=True
        ).stdout.strip()
        with patch.object(Path, "read_bytes", side_effect=AssertionError("working-tree read")):
            candidates = audit_git_sources(ROOT, base_sha)
        self.assertTrue(candidates)


if __name__ == "__main__":
    unittest.main()
