"""Executable contract tests for the strict three-platform coverage gate."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
GATE_PATH = REPOSITORY_ROOT / "scripts" / "coverage_gate.py"
GATE = None
GATE_LOAD_ERROR = None
if GATE_PATH.is_file():
    try:
        SPEC = importlib.util.spec_from_file_location("coverage_gate", GATE_PATH)
        if SPEC is None or SPEC.loader is None:
            raise ImportError("coverage gate has no importable module loader")
        GATE = importlib.util.module_from_spec(SPEC)
        SPEC.loader.exec_module(GATE)
    except Exception as error:  # The tests must run and explain the missing implementation in RED.
        GATE_LOAD_ERROR = error


def llvm_document(regions: list[list[int]], filename: str = "crates/kmipkit-ttlv/src/lib.rs") -> str:
    return json.dumps(
        {
            "data": [
                {
                    "functions": [
                        {
                            "name": "_ZN7kmipkit4test",
                            "filenames": [filename],
                            "regions": regions,
                        }
                    ]
                }
            ]
        }
    )


class CoverageGateTests(unittest.TestCase):
    def require_gate(self) -> None:
        self.assertIsNotNone(
            GATE,
            f"coverage_gate.py must provide the tested coverage contract; load error: {GATE_LOAD_ERROR}",
        )

    def test_code_regions_produce_executable_lines_but_gap_regions_do_not(self) -> None:
        self.require_gate()
        report = GATE.parse_llvm_export(
            llvm_document(
                [
                    [10, 1, 12, 2, 4, 0, 0, 0],
                    [12, 2, 12, 8, 0, 0, 0, 3],
                ]
            ),
            REPOSITORY_ROOT,
        )
        self.assertEqual({10: 4, 11: 4, 12: 4}, report["crates/kmipkit-ttlv/src/lib.rs"])

    def test_llvm_report_rejects_missing_schema_and_invalid_counts(self) -> None:
        self.require_gate()
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export('{"data": []}', REPOSITORY_ROOT)
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export(llvm_document([[10, 1, 10, 4, -1, 0, 0, 0]]), REPOSITORY_ROOT)

    def test_report_paths_must_resolve_inside_the_checkout(self) -> None:
        self.require_gate()
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export(llvm_document([[1, 1, 1, 2, 1, 0, 0, 0]], "../../outside.rs"), REPOSITORY_ROOT)

    def test_rust_source_scanner_ignores_nested_comments_and_raw_strings(self) -> None:
        self.require_gate()
        source = r'''/* outer /* nested fn fake() {} */ still comment */
const TEXT: &str = r###"fn also_fake() {}"###;
trait Example { fn declaration(&self); }
'''
        self.assertFalse(GATE.rust_source_has_function_body(source))

    def test_rust_source_scanner_detects_body_and_treats_ambiguous_syntax_as_eligible(self) -> None:
        self.require_gate()
        self.assertTrue(GATE.rust_source_has_function_body("fn implemented() { let value = 1; }"))
        self.assertTrue(GATE.rust_source_has_function_body("fn incomplete("))

    def test_test_only_paths_are_excluded_from_production_preflight(self) -> None:
        self.require_gate()
        for path in (
            "crates/kmipkit/src/tests.rs",
            "crates/kmipkit/src/parser_tests.rs",
            "crates/kmipkit/src/parser-tests.rs",
            "crates/kmipkit/tests/integration.rs",
        ):
            with self.subTest(path=path):
                self.assertFalse(GATE.is_coverage_source_path(path))
        self.assertTrue(GATE.is_coverage_source_path("crates/kmipkit/src/lib.rs"))

    def test_unreadable_or_invalid_utf8_source_is_eligible(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"\xff\xfe\xfa")
            scan = GATE.scan_production_sources(root)
            self.assertTrue(scan.eligible)
            self.assertIn("read", scan.reason.lower())

    def test_inline_cfg_test_module_fails_production_preflight(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("#[cfg(test)] mod tests { fn check() {} }", encoding="utf-8")
            with self.assertRaises(GATE.InlineTestModuleError):
                GATE.scan_production_sources(root)

    def test_changed_rust_line_parser_uses_zero_context_hunks_and_ignores_removed_lines(self) -> None:
        self.require_gate()
        diff = """diff --git a/crates/kmipkit/src/lib.rs b/crates/kmipkit/src/lib.rs
--- a/crates/kmipkit/src/lib.rs
+++ b/crates/kmipkit/src/lib.rs
@@ -4,2 +4,3 @@
-    old();
+    new();
+    extra();
"""
        self.assertEqual(
            {"crates/kmipkit/src/lib.rs": {4, 5}},
            GATE.parse_added_rust_lines(diff),
        )

    def test_platform_reports_sum_execution_counts_by_source_line(self) -> None:
        self.require_gate()
        merged = GATE.merge_platform_reports(
            {
                "ubuntu": {"crates/kmipkit/src/lib.rs": {7: 2, 8: 0}},
                "windows": {"crates/kmipkit/src/lib.rs": {7: 3}},
                "macos": {"crates/kmipkit/src/lib.rs": {8: 4}},
            }
        )
        self.assertEqual({7: 5, 8: 4}, merged["crates/kmipkit/src/lib.rs"])

    def test_threshold_boundaries_and_not_applicable_changed_metric(self) -> None:
        self.require_gate()
        self.assertTrue(GATE.meets_threshold(95, 100, 95))
        self.assertFalse(GATE.meets_threshold(94, 100, 95))
        self.assertEqual("not applicable", GATE.changed_code_result(set(), {}))

    def test_no_body_scan_is_unavailable_but_body_scan_requires_reports(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("#![no_std]\ntrait Example { fn method(&self); }", encoding="utf-8")
            self.assertFalse(GATE.scan_production_sources(root).eligible)
            source.write_text("pub fn body() {}", encoding="utf-8")
            self.assertTrue(GATE.scan_production_sources(root).eligible)


if __name__ == "__main__":
    unittest.main(verbosity=2)
