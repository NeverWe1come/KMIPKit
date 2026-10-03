"""Executable contract tests for the strict three-platform coverage gate."""

from __future__ import annotations

import importlib.util
import json
import subprocess
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


def llvm_document(
    regions: list[list[int]],
    filename: str = "crates/kmipkit-ttlv/src/lib.rs",
    *,
    segments: list[list[object]] | None = None,
    summary_lines: int | None = None,
) -> str:
    if segments is None:
        segments = []
        executable_lines: set[int] = set()
        for region in regions:
            if len(region) != 8 or region[7] != 0:
                continue
            start_line, start_column, end_line, end_column, count = region[:5]
            segments.extend(
                [
                    [start_line, start_column, count, True, True, False],
                    [end_line, end_column, 0, False, False, False],
                ]
            )
            last_line = end_line - 1 if end_line > start_line and end_column == 1 else end_line
            executable_lines.update(range(start_line, last_line + 1))
        segments.sort(key=lambda segment: (segment[0], segment[1]))
        segments.append([max((segment[0] for segment in segments), default=1) + 1, 1, 0, False, False, False])
        if summary_lines is None:
            summary_lines = len(executable_lines)
    if summary_lines is None:
        summary_lines = 0
    segment_lines: dict[int, int] = {}
    for index, segment in enumerate(segments[:-1]):
        line, _, count, has_count, _, is_gap = segment
        if not has_count or is_gap:
            continue
        end_line, end_column = segments[index + 1][:2]
        last_line = end_line - 1 if end_line > line and end_column == 1 else end_line
        for source_line in range(line, last_line + 1):
            segment_lines[source_line] = max(segment_lines.get(source_line, 0), count)
    summary_covered = sum(count > 0 for count in segment_lines.values())
    return json.dumps(
        {
            "type": "llvm.coverage.json.export",
            "version": "3.0.1",
            "data": [
                {
                    "files": [
                        {
                            "filename": filename,
                            "segments": segments,
                            "summary": {"lines": {"count": summary_lines, "covered": summary_covered}},
                        }
                    ],
                    "functions": [
                        {
                            "name": "_ZN7kmipkit4test",
                            "count": 1,
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
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("\n" * 11 + "           x\n", encoding="utf-8")
            report = GATE.parse_llvm_export(
                llvm_document(
                    [
                        [10, 1, 12, 2, 4, 0, 0, 0],
                        [12, 2, 12, 8, 0, 0, 0, 3],
                    ]
                ),
                root,
            )
        self.assertEqual({10: 4, 11: 4, 12: 4}, report["crates/kmipkit-ttlv/src/lib.rs"])

    def test_file_segments_keep_unreported_function_lines_and_nested_zero_counts(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn first() {}\npub fn second() {}\n", encoding="utf-8")
            document = llvm_document(
                [[1, 1, 1, 17, 1, 0, 0, 0]],
                segments=[
                    [1, 1, 1, True, True, False],
                    [1, 17, 0, False, False, False],
                    [2, 1, 0, True, True, False],
                    [2, 18, 0, False, False, False],
                    [3, 1, 0, False, False, False],
                ],
                summary_lines=2,
            )
            report = GATE.parse_llvm_export(document, root)
        self.assertEqual({1: 1, 2: 0}, report["crates/kmipkit-ttlv/src/lib.rs"])

    def test_missing_function_record_cannot_hide_an_uncovered_line_from_the_gate(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn first() {}\npub fn second() {}\n", encoding="utf-8")
            document = llvm_document(
                [[1, 1, 1, 17, 1, 0, 0, 0]],
                segments=[
                    [1, 1, 1, True, True, False],
                    [1, 17, 0, False, False, False],
                    [2, 1, 0, True, True, False],
                    [2, 18, 0, False, False, False],
                    [3, 1, 0, False, False, False],
                ],
                summary_lines=2,
            )
            reports = {
                platform: GATE.parse_llvm_export(document, root)
                for platform in ("ubuntu", "windows", "macos")
            }
            with self.assertRaisesRegex(GATE.CoverageDataError, "TTLV/protocol coverage"):
                GATE._evaluate_coverage(root, reports, "")

    def test_file_segments_override_outer_positive_region_for_nested_uncovered_code(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn choose(flag: bool) -> i32 {\n    if flag {\n        1\n    } else {\n        2\n    }\n}\n",
                encoding="utf-8",
            )
            document = llvm_document(
                [
                    [1, 1, 7, 2, 1, 0, 0, 0],
                    [5, 9, 5, 10, 0, 0, 0, 0],
                ],
                segments=[
                    [1, 1, 1, True, True, False],
                    [1, 33, 0, False, False, False],
                    [2, 8, 1, True, True, False],
                    [2, 12, 0, False, False, False],
                    [3, 9, 1, True, True, False],
                    [3, 10, 0, False, False, False],
                    [5, 9, 0, True, True, False],
                    [5, 10, 0, False, False, False],
                    [7, 1, 1, True, True, False],
                    [7, 2, 0, False, False, False],
                    [8, 1, 0, False, False, False],
                ],
                summary_lines=5,
            )
            report = GATE.parse_llvm_export(document, root)
        self.assertEqual(0, report["crates/kmipkit-ttlv/src/lib.rs"][5])

    def test_pinned_cargo_llvm_cov_fixture_matches_its_real_line_summary(self) -> None:
        self.require_gate()
        fixtures = Path(__file__).parent / "fixtures"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_bytes((fixtures / "cargo-llvm-cov-0.9.1-branch.rs").read_bytes())
            report = GATE.parse_llvm_export(
                (fixtures / "cargo-llvm-cov-0.9.1-branch.json").read_text(encoding="utf-8"),
                root,
            )
        self.assertEqual({1: 1, 2: 1, 3: 1, 5: 0, 7: 1}, report["crates/sample/src/lib.rs"])

    def test_file_segments_outside_source_line_bounds_fail_closed(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn run() {}\n", encoding="utf-8")
            document = llvm_document(
                [[2, 1, 2, 2, 1, 0, 0, 0]],
                segments=[[2, 1, 1, True, True, False], [2, 2, 0, False, False, False], [3, 1, 0, False, False, False]],
                summary_lines=1,
            )
            with self.assertRaisesRegex(GATE.CoverageDataError, "outside the source file"):
                GATE.parse_llvm_export(document, root)

    def test_llvm_report_rejects_missing_schema_and_invalid_counts(self) -> None:
        self.require_gate()
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export('{"data": []}', REPOSITORY_ROOT)
        valid_report = json.loads(llvm_document([[1, 1, 1, 2, 1, 0, 0, 0]]))
        invalid_schemas = []
        for key in ("type", "version"):
            missing = json.loads(json.dumps(valid_report))
            del missing[key]
            invalid_schemas.append(missing)
        wrong_type = json.loads(json.dumps(valid_report))
        wrong_type["type"] = "other.export"
        invalid_schemas.append(wrong_type)
        for unsupported_version in ("2.0.0", "3.1.0", "3.x.1"):
            unsupported = json.loads(json.dumps(valid_report))
            unsupported["version"] = unsupported_version
            invalid_schemas.append(unsupported)
        for document in invalid_schemas:
            with self.subTest(document=document):
                with self.assertRaises(GATE.CoverageDataError):
                    GATE.parse_llvm_export(document, REPOSITORY_ROOT)
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export(llvm_document([[10, 1, 10, 4, -1, 0, 0, 0]]), REPOSITORY_ROOT)
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export(llvm_document([[10, 1, 10, 4, 0, 0, 0, 99]]), REPOSITORY_ROOT)

    def test_report_paths_must_resolve_inside_the_checkout(self) -> None:
        self.require_gate()
        with self.assertRaises(GATE.CoverageDataError):
            GATE.parse_llvm_export(llvm_document([[1, 1, 1, 2, 1, 0, 0, 0]], "../../outside.rs"), REPOSITORY_ROOT)

    def test_normalization_rewrites_absolute_platform_paths_to_repository_relative_paths(self) -> None:
        self.require_gate()
        absolute_source = (REPOSITORY_ROOT / "crates" / "kmipkit-ttlv" / "src" / "lib.rs").as_posix()
        normalized = GATE.normalize_llvm_export(
            llvm_document([[1, 1, 1, 2, 1, 0, 0, 0]], absolute_source),
            REPOSITORY_ROOT,
        )
        report = GATE.parse_llvm_export(normalized, REPOSITORY_ROOT)
        self.assertEqual({1: 1}, report["crates/kmipkit-ttlv/src/lib.rs"])

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

    def test_only_external_test_directories_are_excluded_from_production_preflight(self) -> None:
        self.require_gate()
        for path in (
            "crates/kmipkit/tests/integration.rs",
            "crates/kmipkit/test/integration.rs",
        ):
            with self.subTest(path=path):
                self.assertFalse(GATE.is_coverage_source_path(path))
        for path in (
            "crates/kmipkit/src/lib.rs",
            "crates/kmipkit/src/tests.rs",
            "crates/kmipkit/src/parser_tests.rs",
            "crates/kmipkit/src/parser-tests.rs",
            "crates/kmipkit/src/test/helpers.rs",
            "crates/kmipkit/src/tests/integration.rs",
        ):
            with self.subTest(path=path):
                self.assertTrue(GATE.is_coverage_source_path(path))
        self.assertTrue(GATE.is_coverage_source_path("crates/kmipkit/src/test/helpers.rs"))

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

    def test_untraversed_symlink_source_directory_cannot_emit_unavailable_status(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_dir = root / "crates" / "sample" / "src"
            source_dir.mkdir(parents=True)
            (source_dir / "lib.rs").write_text("trait Declaration { fn method(); }\n", encoding="utf-8")
            external = root / "external-source"
            external.mkdir()
            (external / "hidden.rs").write_text("pub fn hidden() {}\n", encoding="utf-8")
            try:
                (source_dir / "linked").symlink_to(external, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            scan = GATE.scan_production_sources(root)
        self.assertTrue(scan.eligible)
        self.assertFalse(scan.complete)
        self.assertIn("symlink", scan.reason.lower())

    def test_symlinked_crate_src_root_cannot_emit_unavailable_status(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            crate_dir = root / "crates" / "sample"
            crate_dir.mkdir(parents=True)
            external = root / "shared-source"
            external.mkdir()
            (external / "lib.rs").write_text("pub fn hidden() {}\n", encoding="utf-8")
            try:
                (crate_dir / "src").symlink_to(external, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            scan = GATE.scan_production_sources(root)
        self.assertTrue(scan.eligible)
        self.assertFalse(scan.complete)
        self.assertIn("symlink", scan.reason.lower())

    def test_symlinked_crate_root_cannot_emit_unavailable_status(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            crates = root / "crates"
            crates.mkdir()
            external_crate = root / "external-crate"
            source_dir = external_crate / "src"
            source_dir.mkdir(parents=True)
            (source_dir / "lib.rs").write_text("pub fn hidden() {}\n", encoding="utf-8")
            try:
                (crates / "sample").symlink_to(external_crate, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            scan = GATE.scan_production_sources(root)
        self.assertTrue(scan.eligible)
        self.assertFalse(scan.complete)
        self.assertIn("symlink", scan.reason.lower())

    def test_inline_cfg_test_module_fails_production_preflight(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("#[cfg(test)] mod tests { fn check() {} }", encoding="utf-8")
            with self.assertRaises(GATE.InlineTestModuleError):
                GATE.scan_production_sources(root)

    def test_inline_cfg_test_module_with_intervening_attribute_fails_preflight(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "#[cfg(test)] #[allow(dead_code)] mod tests { fn check() {} }", encoding="utf-8"
            )
            with self.assertRaises(GATE.InlineTestModuleError):
                GATE.scan_production_sources(root)

    def test_path_attribute_keeps_suffix_named_production_module_in_coverage(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_dir = root / "crates" / "sample" / "src"
            source_dir.mkdir(parents=True)
            (source_dir / "lib.rs").write_text(
                '#[path = "parser_tests.rs"] mod parser;\n', encoding="utf-8"
            )
            (source_dir / "parser_tests.rs").write_text("pub fn parse() {}\n", encoding="utf-8")
            scan = GATE.scan_production_sources(root)
            self.assertTrue(scan.eligible)
            self.assertIn("crates/sample/src/parser_tests.rs", scan.eligible_files)
            document = llvm_document(
                [[1, 1, 1, 18, 1, 0, 0, 0]],
                "crates/sample/src/parser_tests.rs",
            )
            report = GATE.parse_llvm_export(document, root)
        self.assertEqual({1: 1}, report["crates/sample/src/parser_tests.rs"])

    def test_preflight_scans_later_source_files_after_finding_a_function_body(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_dir = root / "crates" / "sample" / "src"
            source_dir.mkdir(parents=True)
            (source_dir / "a.rs").write_text("pub fn production() {}", encoding="utf-8")
            (source_dir / "z.rs").write_text("#[cfg(test)] mod tests { fn check() {} }", encoding="utf-8")
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

    def test_changed_rust_line_parser_decodes_git_quoted_unicode_paths(self) -> None:
        self.require_gate()
        diff = r'''diff --git "a/crates/kmipkit-ttlv/src/parser-\303\261 name.rs" "b/crates/kmipkit-ttlv/src/parser-\303\261 name.rs"
--- "a/crates/kmipkit-ttlv/src/parser-\303\261 name.rs"
+++ "b/crates/kmipkit-ttlv/src/parser-\303\261 name.rs"
@@ -0,0 +1 @@
+pub fn added() {}
'''
        self.assertEqual(
            {"crates/kmipkit-ttlv/src/parser-ñ name.rs": {1}},
            GATE.parse_added_rust_lines(diff),
        )

    def test_exact_git_diff_retains_changed_lines_for_quoted_unicode_paths(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates" / "kmipkit-ttlv" / "src" / "parser-ñ name.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn first() {}\n", encoding="utf-8")
            commands = [
                ["git", "init", "--quiet", str(root)],
                ["git", "-C", str(root), "config", "user.name", "KMIPKit Test"],
                ["git", "-C", str(root), "config", "user.email", "kmipkit-test@example.invalid"],
                ["git", "-C", str(root), "config", "core.quotePath", "true"],
                ["git", "-C", str(root), "add", "--all"],
                ["git", "-C", str(root), "commit", "--quiet", "-m", "base"],
            ]
            for command in commands:
                subprocess.run(command, check=True, capture_output=True, text=True)
            base = subprocess.run(
                ["git", "-C", str(root), "rev-parse", "HEAD"], check=True, capture_output=True, text=True
            ).stdout.strip()
            source.write_text("pub fn first() {}\npub fn second() {}\n", encoding="utf-8")
            subprocess.run(["git", "-C", str(root), "add", "--all"], check=True, capture_output=True, text=True)
            subprocess.run(
                ["git", "-C", str(root), "commit", "--quiet", "-m", "change"],
                check=True,
                capture_output=True,
                text=True,
            )
            merge = subprocess.run(
                ["git", "-C", str(root), "rev-parse", "HEAD"], check=True, capture_output=True, text=True
            ).stdout.strip()
            diff = GATE._run_git_diff(root, base, merge)
        self.assertEqual(
            {"crates/kmipkit-ttlv/src/parser-ñ name.rs": {2}},
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

    def test_aggregation_fails_when_an_executable_production_file_is_absent_from_reports(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_dir = root / "crates" / "sample" / "src"
            source_dir.mkdir(parents=True)
            (source_dir / "covered.rs").write_text("pub fn covered() {}", encoding="utf-8")
            (source_dir / "missing.rs").write_text("pub fn missing() {}", encoding="utf-8")
            reports = {
                platform: {"crates/sample/src/covered.rs": {1: 1}}
                for platform in ("ubuntu", "windows", "macos")
            }
            with self.assertRaisesRegex(GATE.CoverageDataError, "missing from LLVM coverage reports"):
                GATE._evaluate_coverage(root, reports, "")

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

    def test_three_unavailable_platform_sentinels_are_accepted_only_as_a_complete_set(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            reports = Path(directory)
            for platform in ("ubuntu", "windows", "macos"):
                artifact = reports / f"coverage-{platform}"
                artifact.mkdir()
                (artifact / "coverage-status.json").write_text(
                    '{"reason":"no production function bodies","status":"unavailable"}', encoding="utf-8"
                )
            self.assertEqual({"status": "unavailable"}, GATE._load_platform_artifacts(reports, REPOSITORY_ROOT))
            (reports / "coverage-windows" / "coverage-status.json").unlink()
            with self.assertRaises(GATE.CoverageDataError):
                GATE._load_platform_artifacts(reports, REPOSITORY_ROOT)

    def test_missing_or_mixed_platform_coverage_artifacts_fail_closed(self) -> None:
        self.require_gate()
        with tempfile.TemporaryDirectory() as directory:
            reports = Path(directory)
            for platform in ("ubuntu", "windows"):
                artifact = reports / f"coverage-{platform}"
                artifact.mkdir()
                (artifact / "coverage-status.json").write_text(
                    '{"reason":"no production function bodies","status":"unavailable"}', encoding="utf-8"
                )
            with self.assertRaises(GATE.CoverageDataError):
                GATE._load_platform_artifacts(reports, REPOSITORY_ROOT)

            artifact = reports / "coverage-macos"
            artifact.mkdir()
            source = "crates/kmipkit/src/lib.rs"
            self.assertTrue((REPOSITORY_ROOT / source).is_file())
            (artifact / "coverage.json").write_text(
                llvm_document([[1, 1, 1, 2, 1, 0, 0, 0]], source), encoding="utf-8"
            )
            with self.assertRaises(GATE.CoverageDataError):
                GATE._load_platform_artifacts(reports, REPOSITORY_ROOT)


if __name__ == "__main__":
    unittest.main(verbosity=2)
