#!/usr/bin/env python3
"""Validate conservative source preflight and aggregate LLVM JSON coverage."""

from __future__ import annotations

import argparse
from bisect import bisect_left, bisect_right
import html
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping
import xml.etree.ElementTree as ET


class CoverageDataError(RuntimeError):
    """Raised when required coverage input is missing, unsafe, or malformed."""


class CoverageReport(dict[str, dict[int, int]]):
    """A physical-line map plus conservative per-file summary residuals."""

    def __init__(
        self,
        lines: dict[str, dict[int, int]],
        summary_uncovered_excess: dict[str, int],
    ) -> None:
        super().__init__(lines)
        self.summary_uncovered_excess = summary_uncovered_excess


class InlineTestModuleError(CoverageDataError):
    """Raised when tests are embedded in a production source file."""


class SourceScanResult:
    def __init__(
        self,
        eligible: bool,
        reason: str,
        files_scanned: int,
        eligible_files: set[str] | None = None,
        complete: bool = True,
    ) -> None:
        self.eligible = eligible
        self.reason = reason
        self.files_scanned = files_scanned
        self.eligible_files = eligible_files or set()
        self.complete = complete


def _mask_rust_non_code(source: str) -> str:
    """Replace comments and literals with spaces while preserving newlines."""
    output = list(source)
    length = len(source)
    index = 0

    def blank(start: int, end: int) -> None:
        for position in range(start, min(end, length)):
            if source[position] not in "\r\n":
                output[position] = " "

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end < 0:
                end = length
            blank(index, end)
            index = end
            continue
        if source.startswith("/*", index):
            start = index
            depth = 1
            index += 2
            while index < length and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            blank(start, index)
            if depth:
                # Unterminated comments are ambiguous; retain a marker for conservative scanning.
                output[start] = "?"
            continue

        # Rust raw strings may be prefixed with b or c. Hash count is unbounded.
        raw_match = re.match(r"(?:br|cr|r)(?P<hashes>#{0,})\"", source[index:])
        if raw_match:
            start = index
            hashes = raw_match.group("hashes")
            opening_length = len(raw_match.group(0))
            closing = '"' + hashes
            close_at = source.find(closing, index + opening_length)
            end = length if close_at < 0 else close_at + len(closing)
            blank(start, end)
            if close_at < 0:
                output[start] = "?"
            index = end
            continue

        # Normal, byte, and C string literals.
        string_start = index
        quote_at = index
        if source.startswith(('b"', 'c"'), index):
            quote_at += 1
        if source[quote_at] == '"':
            index = quote_at + 1
            escaped = False
            while index < length:
                char = source[index]
                index += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    break
            else:
                blank(string_start, index)
                output[string_start] = "?"
                continue
            blank(string_start, index)
            continue

        # Mask character literals but leave Rust lifetimes such as 'a untouched.
        if source[index] == "'":
            cursor = index + 1
            escaped = False
            while cursor < length and source[cursor] not in "\r\n":
                char = source[cursor]
                cursor += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == "'":
                    blank(index, cursor)
                    index = cursor
                    break
                elif char == "{" or char == "}" or char == ";":
                    break
            else:
                index += 1
                continue
            if index == cursor:
                continue

        index += 1
    return "".join(output)


def rust_source_has_function_body(source: str) -> bool:
    """Return true on any body-bearing or ambiguous `fn` construct."""
    masked = _mask_rust_non_code(source)
    if "?" in masked:
        return True

    for match in re.finditer(r"\bfn\b", masked):
        depth = {"(": 0, "[": 0, "<": 0}
        index = match.end()
        while index < len(masked):
            char = masked[index]
            if char in depth:
                depth[char] += 1
            elif char == ")":
                if depth["("] == 0:
                    return True
                depth["("] -= 1
            elif char == "]":
                if depth["["] == 0:
                    return True
                depth["["] -= 1
            elif char == ">":
                if depth["<"] > 0:
                    depth["<"] -= 1
            elif char == ";" and not any(depth.values()):
                break
            elif char == "{" and not any(depth.values()):
                return True
            elif char == "}" and not any(depth.values()):
                return True
            elif char == "<":
                depth["<"] += 1
            elif char == "f" and masked.startswith("fn", index) and not any(depth.values()):
                return True
            index += 1
        else:
            return True
    return False


def _is_rust_source_tree_path(path: str | Path) -> bool:
    """Return whether a relative Rust path is under a production crate source tree."""
    normalized = str(path).replace("\\", "/").lstrip("./")
    parts = PurePosixPath(normalized).parts
    if len(parts) < 4 or parts[0] != "crates" or parts[2] != "src" or not normalized.endswith(".rs"):
        return False
    return True


def is_coverage_source_path(path: str | Path) -> bool:
    """Return whether a relative path names an audited production source file."""
    normalized = str(path).replace("\\", "/").lstrip("./")
    if _is_rust_source_tree_path(normalized):
        return True
    parts = PurePosixPath(normalized).parts
    if len(parts) >= 7 and parts[:5] == (
        "bindings",
        "java",
        "src",
        "main",
        "java",
    ) and parts[5] == "org" and parts[6] == "kmipkit":
        return normalized.endswith(".java")
    if len(parts) >= 5 and parts[:4] == ("bindings", "python", "src", "kmipkit"):
        return normalized.endswith(".py")
    return normalized == "bindings/java/native/kmipkit_jni.cpp"


def scan_adapter_sources(workspace_root: str | Path) -> dict[str, set[str]]:
    """List handwritten and generated product adapter sources, excluding consumers."""
    root = Path(workspace_root).resolve()
    scopes = {
        "Java adapters": root / "bindings/java/src/main/java/org/kmipkit",
        "Python adapters": root / "bindings/python/src/kmipkit",
    }
    sources: dict[str, set[str]] = {name: set() for name in (*scopes, "JNI bridge")}
    for scope, source_root in scopes.items():
        if not source_root.exists():
            continue
        if source_root.is_symlink():
            raise CoverageDataError(f"Adapter source root is a symlink and cannot be scanned: {source_root}")
        suffix = ".java" if scope == "Java adapters" else ".py"
        walk_errors: list[str] = []

        def on_walk_error(error: OSError) -> None:
            walk_errors.append(str(error))

        for current, directories, filenames in os.walk(source_root, topdown=True, onerror=on_walk_error):
            current_path = Path(current)
            symlinked = _find_symlinked_directories(current_path, directories, walk_errors, "adapter source directory")
            directories[:] = [name for name in directories if name not in symlinked]
            for filename in filenames:
                candidate = current_path / filename
                if candidate.suffix != suffix:
                    continue
                relative = candidate.relative_to(root).as_posix()
                if is_coverage_source_path(relative):
                    sources[scope].add(relative)
        if walk_errors:
            raise CoverageDataError(f"Adapter source scan is incomplete: {walk_errors[0]}")

    jni_source = root / "bindings/java/native/kmipkit_jni.cpp"
    if jni_source.is_file():
        sources["JNI bridge"].add(jni_source.relative_to(root).as_posix())
    return sources


def _excluded_test_module(
    source: str,
    masked: str,
    source_path: Path,
    workspace_root: Path,
    attributes: list[tuple[int, int]],
    module_index: int,
) -> bool:
    """Return whether a test-only module points to an existing excluded test file."""
    path_attributes = []
    for start, end in attributes:
        path_match = re.fullmatch(
            r'\s*#\s*\[\s*path\s*=\s*"([^"\\]*)"\s*\]\s*',
            source[start:end],
        )
        if path_match is not None:
            path_attributes.append(path_match.group(1))
    if len(path_attributes) != 1:
        return False

    module_match = re.match(
        r"(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+[A-Za-z_]\w*\s*;",
        masked[module_index:],
    )
    if module_match is None:
        return False

    source_parts = source_path.relative_to(workspace_root).parts
    if len(source_parts) < 4 or source_parts[:1] != ("crates",) or source_parts[2] != "src":
        return False
    crate_root = workspace_root.joinpath(*source_parts[:2])
    tests_root = crate_root / "tests"
    try:
        resolved_tests_root = tests_root.resolve(strict=True)
        resolved_test_file = (source_path.parent / path_attributes[0]).resolve(strict=True)
        resolved_test_file.relative_to(resolved_tests_root)
    except (OSError, ValueError):
        return False
    return resolved_test_file.is_file()


def _inline_test_attribute(source: str, source_path: Path, workspace_root: Path) -> bool:
    masked = _mask_rust_non_code(source)
    for match in re.finditer(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", masked):
        index = match.end()
        attributes: list[tuple[int, int]] = []
        while True:
            while index < len(masked) and masked[index].isspace():
                index += 1
            if not masked.startswith("#[", index):
                break
            attribute_start = index
            bracket_start = index + 1
            depth = 0
            while bracket_start < len(masked):
                char = masked[bracket_start]
                if char == "[":
                    depth += 1
                elif char == "]":
                    depth -= 1
                    if depth == 0:
                        index = bracket_start + 1
                        break
                bracket_start += 1
            else:
                index = len(masked)
                break
            attributes.append((attribute_start, index))
        while index < len(masked) and masked[index].isspace():
            index += 1
        visibility = re.match(r"pub(?:\s*\([^)]*\))?\s+", masked[index:])
        if visibility:
            index += visibility.end()
        if re.match(r"mod\b", masked[index:]):
            if _excluded_test_module(source, masked, source_path, workspace_root, attributes, index):
                continue
            return True
    return False


def _find_symlinked_directories(
    parent: Path,
    names: Iterable[str],
    walk_errors: list[str],
    directory_kind: str,
) -> set[str]:
    symlinked = {name for name in names if (parent / name).is_symlink()}
    for name in sorted(symlinked):
        walk_errors.append(f"{directory_kind} is a symlink and was not traversed: {parent / name}")
    return symlinked


def scan_production_sources(workspace_root: str | Path) -> SourceScanResult:
    """Scan production sources; uncertainty and adapter files require coverage."""
    root = Path(workspace_root).resolve()
    crates = root / "crates"
    if not crates.is_dir():
        raise CoverageDataError(f"Rust source tree is missing: {crates}")

    source_files: list[Path] = []
    walk_errors: list[str] = []

    def on_walk_error(error: OSError) -> None:
        walk_errors.append(str(error))

    for current, directories, filenames in os.walk(crates, topdown=True, onerror=on_walk_error):
        current_path = Path(current)
        relative_directory = current_path.relative_to(root).as_posix()
        parts = PurePosixPath(relative_directory).parts
        if len(parts) < 3 or parts[0] != "crates" or parts[2] != "src":
            if len(parts) >= 3 and parts[0] == "crates" and parts[2] != "src":
                directories[:] = []
            elif len(parts) == 1 and parts[0] == "crates":
                symlinked_crates = _find_symlinked_directories(
                    current_path, directories, walk_errors, "crate directory"
                )
                directories[:] = [name for name in directories if name not in symlinked_crates]
            elif len(parts) == 2 and parts[0] == "crates":
                symlinked_sources = _find_symlinked_directories(
                    current_path, ("src",), walk_errors, "source directory"
                )
                directories[:] = [
                    name for name in directories if name == "src" and name not in symlinked_sources
                ]
            continue
        symlinked_directories = _find_symlinked_directories(
            current_path, directories, walk_errors, "source directory"
        )
        directories[:] = [name for name in directories if name not in symlinked_directories]
        for filename in filenames:
            candidate = current_path / filename
            relative = candidate.relative_to(root).as_posix()
            if candidate.suffix == ".rs" and _is_rust_source_tree_path(relative):
                source_files.append(candidate)

    eligible_files: set[str] = set()
    scan_errors = list(walk_errors)
    for source_path in sorted(source_files):
        relative = source_path.relative_to(root).as_posix()
        if not is_coverage_source_path(relative):
            continue
        try:
            source = source_path.read_text(encoding="utf-8", errors="strict")
        except (OSError, UnicodeError) as error:
            eligible_files.add(relative)
            scan_errors.append(f"source could not be read completely: {source_path}: {error}")
            continue
        if _inline_test_attribute(source, source_path, root):
            raise InlineTestModuleError(
                f"Inline #[cfg(test)] module found in production source {source_path}; move tests to an excluded test path."
            )
        if rust_source_has_function_body(source):
            eligible_files.add(relative)
    if scan_errors:
        reason = f"source scan was incomplete; coverage is required: {scan_errors[0]}"
        return SourceScanResult(True, reason, len(source_files), eligible_files, complete=False)
    adapter_sources = scan_adapter_sources(root)
    adapter_file_count = sum(len(paths) for paths in adapter_sources.values())
    if eligible_files:
        reason = f"executable or ambiguous function body found in {len(eligible_files)} source file(s)"
        return SourceScanResult(True, reason, len(source_files) + adapter_file_count, eligible_files)
    if adapter_file_count:
        return SourceScanResult(
            True,
            "adapter production source requires coverage",
            len(source_files) + adapter_file_count,
        )
    return SourceScanResult(
        False,
        "no production function bodies found after a complete source scan",
        len(source_files),
    )


def _canonical_source_path(
    filename: str,
    workspace_root: Path,
) -> str | None:
    candidate = Path(filename)
    if not candidate.is_absolute():
        candidate = workspace_root / candidate
    try:
        resolved = candidate.resolve(strict=False)
        relative = resolved.relative_to(workspace_root)
    except (OSError, ValueError) as error:
        raise CoverageDataError(f"LLVM report path is outside the workspace: {filename}") from error
    relative_name = relative.as_posix()
    if not is_coverage_source_path(relative_name):
        return None
    if not (workspace_root / relative).is_file():
        raise CoverageDataError(f"LLVM report references missing source file: {relative_name}")
    return relative_name


def _load_json_document(document: str | bytes | Mapping[str, Any]) -> dict[str, Any]:
    if isinstance(document, Mapping):
        value = dict(document)
    else:
        try:
            value = json.loads(document)
        except (json.JSONDecodeError, UnicodeDecodeError, TypeError) as error:
            raise CoverageDataError(f"LLVM report is not valid JSON: {error}") from error
    if not isinstance(value, dict):
        raise CoverageDataError("LLVM report root must be an object.")
    if value.get("type") != "llvm.coverage.json.export":
        raise CoverageDataError("LLVM report has an unsupported or missing export type.")
    version = value.get("version")
    if not isinstance(version, str) or re.fullmatch(r"(?:2\.0|3\.(?:0|1))\.\d+", version) is None:
        raise CoverageDataError(f"LLVM report has an unsupported or missing schema version: {version!r}")
    if not isinstance(value.get("data"), list) or not value["data"]:
        raise CoverageDataError("LLVM report is missing a non-empty data array.")
    for data_item in value["data"]:
        if (
            not isinstance(data_item, dict)
            or not isinstance(data_item.get("files"), list)
            or not isinstance(data_item.get("functions"), list)
        ):
            raise CoverageDataError("LLVM report data item is missing its files or functions array.")
        for file_record in data_item["files"]:
            if not isinstance(file_record, dict) or not isinstance(file_record.get("filename"), str):
                raise CoverageDataError("LLVM report file record is missing its filename.")
        if not data_item["files"]:
            raise CoverageDataError("LLVM report data item has an empty files array.")
    return value


def _parse_file_segments(file_record: Mapping[str, Any], source_path: Path) -> dict[int, int]:
    """Reconstruct LLVM's line counts from its ordered per-file coverage segments."""
    segments = file_record.get("segments")
    if not isinstance(segments, list) or not segments:
        raise CoverageDataError("LLVM file record has no coverage segments.")
    parsed_segments: list[tuple[int, int, int, bool, bool, bool]] = []
    previous_position = (0, 0)
    for segment in segments:
        if not isinstance(segment, list) or len(segment) != 6:
            raise CoverageDataError("LLVM file segment does not have the supported 6-field schema.")
        line, column, count, has_count, is_region_entry, is_gap_region = segment
        if (
            type(line) is not int
            or type(column) is not int
            or type(count) is not int
            or type(has_count) is not bool
            or type(is_region_entry) is not bool
            or type(is_gap_region) is not bool
        ):
            raise CoverageDataError("LLVM file segment contains an invalid field type.")
        if line < 1 or column < 1 or count < 0:
            raise CoverageDataError("LLVM file segment contains invalid bounds or execution count.")
        position = (line, column)
        if position < previous_position:
            raise CoverageDataError("LLVM file segments are not in source order.")
        previous_position = position
        parsed_segments.append((line, column, count, has_count, is_region_entry, is_gap_region))

    try:
        source_lines = source_path.read_text(encoding="utf-8", errors="strict").splitlines()
    except (OSError, UnicodeError) as error:
        raise CoverageDataError(f"LLVM source file could not be read as UTF-8: {source_path}") from error
    for line, column, _, has_count, is_region_entry, is_gap_region in parsed_segments:
        is_trailing_boundary = (
            line == len(source_lines) + 1
            and column == 1
            and not has_count
            and not is_region_entry
            and not is_gap_region
        )
        if is_trailing_boundary:
            continue
        if line > len(source_lines):
            raise CoverageDataError(f"LLVM file segment is outside the source file: {source_path}:{line}.")
        maximum_column = len(source_lines[line - 1].encode("utf-8")) + 1
        if column > maximum_column:
            raise CoverageDataError(f"LLVM file segment column is outside the source file: {source_path}:{line}:{column}.")

    line_counts: dict[int, int] = {}
    for index, (start_line, _, count, has_count, _, is_gap_region) in enumerate(parsed_segments[:-1]):
        if not has_count or is_gap_region:
            continue
        end_line, end_column = parsed_segments[index + 1][:2]
        last_line = end_line - 1 if end_line > start_line and end_column == 1 else end_line
        for line in range(start_line, last_line + 1):
            line_counts[line] = max(line_counts.get(line, 0), count)
    if parsed_segments[-1][3]:
        raise CoverageDataError("LLVM file segments have no trailing boundary for the final region.")

    summary = file_record.get("summary")
    line_summary = summary.get("lines") if isinstance(summary, dict) else None
    expected_lines = line_summary.get("count") if isinstance(line_summary, dict) else None
    expected_covered = line_summary.get("covered") if isinstance(line_summary, dict) else None
    if (
        type(expected_lines) is not int
        or expected_lines < 0
        or type(expected_covered) is not int
        or expected_covered < 0
        or expected_covered > expected_lines
    ):
        raise CoverageDataError("LLVM file summary has no valid line count.")
    # LLVM sums line summaries per source-level function group, but file
    # segments merge coverage by physical source location. Shared lines can
    # therefore make the summary larger than the unique segment line map.
    if len(line_counts) > expected_lines:
        raise CoverageDataError(
            "LLVM file segments exceed the file summary line count "
            f"({len(line_counts)} parsed, {expected_lines} reported)."
        )
    covered_lines = sum(count > 0 for count in line_counts.values())
    if covered_lines > expected_covered:
        raise CoverageDataError(
            "LLVM file segments exceed the file summary covered-line count "
            f"({covered_lines} parsed, {expected_covered} reported)."
        )
    return line_counts


def parse_llvm_export(
    document: str | bytes | Mapping[str, Any],
    workspace_root: str | Path,
    *,
    source_filter: set[str] | frozenset[str] | None = None,
) -> dict[str, dict[int, int]]:
    """Parse LLVM file segments, optionally projecting onto exact source files."""
    root = Path(workspace_root).resolve()
    parsed = _load_json_document(document)
    allowed_sources: frozenset[str] | None = None
    if source_filter is not None:
        if not isinstance(source_filter, (set, frozenset)) or not source_filter:
            raise CoverageDataError("LLVM source filter must be a non-empty set of production source paths.")
        normalized_sources: set[str] = set()
        for requested_source in source_filter:
            if not isinstance(requested_source, str):
                raise CoverageDataError("LLVM source filter contains a non-string path.")
            canonical = _canonical_source_path(requested_source, root)
            if canonical is None or canonical != requested_source.replace("\\", "/"):
                raise CoverageDataError(f"LLVM source filter path is not a canonical production source: {requested_source}.")
            normalized_sources.add(canonical)
        allowed_sources = frozenset(normalized_sources)
    lines: dict[str, dict[int, int]] = {}
    summary_uncovered_counts: dict[str, int] = {}
    function_region_lines: dict[str, set[int]] = {}
    function_region_ranges: dict[str, list[tuple[str, int, int, int]]] = {}
    source_lines_by_path: dict[str, list[str]] = {}
    source_data_items: dict[str, int] = {}
    file_records_seen: set[tuple[int, str]] = set()
    regions_seen = 0

    def register_source_mapping(source: str, data_item_index: int) -> None:
        previous_data_item = source_data_items.get(source)
        if previous_data_item is not None and previous_data_item != data_item_index:
            raise CoverageDataError(f"LLVM report has multiple coverage mappings for source file: {source}.")
        source_data_items[source] = data_item_index

    for data_item_index, data_item in enumerate(parsed["data"]):
        for file_record in data_item["files"]:
            if not isinstance(file_record, dict):
                raise CoverageDataError("LLVM report file record is not an object.")
            source = _canonical_source_path(file_record["filename"], root)
            if source is None:
                path = Path(file_record["filename"])
                if not path.is_absolute():
                    path = root / path
                try:
                    path.resolve(strict=False).relative_to(root)
                except (OSError, ValueError) as error:
                    raise CoverageDataError(f"LLVM report path is outside the workspace: {file_record['filename']}") from error
                continue
            if allowed_sources is not None and source not in allowed_sources:
                raise CoverageDataError(f"Scoped LLVM report contains an unexpected source file: {source}.")
            register_source_mapping(source, data_item_index)
            file_record_key = (data_item_index, source)
            if file_record_key in file_records_seen:
                raise CoverageDataError(f"LLVM coverage mapping repeats source file: {source}.")
            file_records_seen.add(file_record_key)
            file_lines = _parse_file_segments(file_record, root / source)
            line_summary = file_record["summary"]["lines"]
            summary_uncovered_counts[source] = max(
                summary_uncovered_counts.get(source, 0),
                line_summary["count"] - line_summary["covered"],
            )
            for line, count in file_lines.items():
                current = lines.setdefault(source, {}).get(line, 0)
                lines[source][line] = max(current, count)
        for function in data_item["functions"]:
            if not isinstance(function, dict):
                raise CoverageDataError("LLVM function record is not an object.")
            if not isinstance(function.get("name"), str) or type(function.get("count")) is not int or function["count"] < 0:
                raise CoverageDataError("LLVM function record has an invalid name or execution count.")
            filenames = function.get("filenames")
            regions = function.get("regions")
            if not isinstance(filenames, list) or not filenames or not all(isinstance(name, str) for name in filenames):
                raise CoverageDataError("LLVM function record has no valid filename table.")
            if not isinstance(regions, list):
                raise CoverageDataError("LLVM function record has no regions array.")
            for region in regions:
                if not isinstance(region, list) or len(region) != 8:
                    raise CoverageDataError("LLVM function region does not have the supported 8-field schema.")
                start_line, start_column, end_line, end_column, count, file_id, expanded_file_id, kind = region
                integer_values = (start_line, start_column, end_line, end_column, count, file_id, expanded_file_id, kind)
                if any(type(value) is not int for value in integer_values):
                    raise CoverageDataError("LLVM function region contains a non-integer field.")
                if start_line < 1 or end_line < start_line or start_column < 1 or end_column < 1 or count < 0:
                    raise CoverageDataError("LLVM function region contains invalid bounds or execution count.")
                if file_id < 0 or file_id >= len(filenames):
                    raise CoverageDataError("LLVM function region refers to a missing filename table entry.")
                if kind not in {0, 1, 2, 3}:
                    raise CoverageDataError(f"LLVM function region has an unsupported region kind: {kind}")
                source = _canonical_source_path(filenames[file_id], root)
                if source is None or kind != 0 or (allowed_sources is not None and source not in allowed_sources):
                    continue
                register_source_mapping(source, data_item_index)
                regions_seen += 1
                if source not in source_lines_by_path:
                    try:
                        source_lines_by_path[source] = (root / source).read_text(
                            encoding="utf-8", errors="strict"
                        ).splitlines()
                    except (OSError, UnicodeError) as error:
                        raise CoverageDataError(f"LLVM source file could not be read as UTF-8: {source}") from error
                source_lines = source_lines_by_path[source]
                last_line = end_line - 1 if end_line > start_line and end_column == 1 else end_line
                if start_line > len(source_lines) or last_line > len(source_lines):
                    raise CoverageDataError(f"LLVM function code region is outside the source file: {source}:{start_line}.")
                start_maximum_column = len(source_lines[start_line - 1].encode("utf-8")) + 1
                if start_column > start_maximum_column:
                    raise CoverageDataError(
                        f"LLVM function region column is outside the source file: {source}:{start_line}:{start_column}."
                    )
                if end_line <= len(source_lines):
                    end_maximum_column = len(source_lines[end_line - 1].encode("utf-8")) + 1
                    if end_column > end_maximum_column:
                        raise CoverageDataError(
                            f"LLVM function region column is outside the source file: {source}:{end_line}:{end_column}."
                        )
                # A region can span source lines that have no executable code (for
                # example braces and `else` clauses). Its start line is the
                # executable location that must be represented in file segments.
                function_region_lines.setdefault(source, set()).add(start_line)
                function_region_ranges.setdefault(source, []).append(
                    (function["name"], start_line, last_line, count)
                )

    if regions_seen == 0:
        raise CoverageDataError("LLVM report contains no production code regions.")
    if allowed_sources is not None:
        missing_sources = allowed_sources.difference(lines)
        if missing_sources:
            raise CoverageDataError(
                "Requested source is missing from LLVM file segments: " + ", ".join(sorted(missing_sources))
            )
    for source, region_lines in function_region_lines.items():
        missing_lines = region_lines.difference(lines.get(source, {}))
        if missing_lines:
            first_missing = min(missing_lines)
            raise CoverageDataError(
                "LLVM function code region line is missing from file segments "
                f"and would shrink the denominator: {source}:{first_missing}."
            )
    summary_uncovered_excess: dict[str, int] = {}
    for source, summary_uncovered in summary_uncovered_counts.items():
        line_counts = lines.get(source, {})
        segment_uncovered = sum(count == 0 for count in line_counts.values())
        line_numbers = sorted(line_counts)
        function_line_counts: dict[str, dict[int, int]] = {}
        for function_name, start_line, last_line, count in function_region_ranges.get(source, []):
            first_index = bisect_left(line_numbers, start_line)
            after_last_index = bisect_right(line_numbers, last_line)
            function_lines = function_line_counts.setdefault(function_name, {})
            for line in line_numbers[first_index:after_last_index]:
                function_lines[line] = max(function_lines.get(line, 0), count)
        known_shared_uncovered = 0
        for line in line_numbers:
            group_counts = [
                function_lines[line]
                for function_lines in function_line_counts.values()
                if line in function_lines
            ]
            if len(group_counts) < 2:
                continue
            group_uncovered = sum(count == 0 for count in group_counts)
            known_shared_uncovered += max(
                0,
                group_uncovered - int(line_counts[line] == 0),
            )
        summary_uncovered_excess[source] = max(
            0,
            summary_uncovered - segment_uncovered - known_shared_uncovered,
        )
    return CoverageReport(lines, summary_uncovered_excess)


def _coverage_report_source(candidate: Path, workspace_root: Path, report_kind: str) -> str:
    try:
        resolved = candidate.resolve(strict=True)
        relative = resolved.relative_to(workspace_root).as_posix()
    except (OSError, ValueError) as error:
        raise CoverageDataError(f"{report_kind} source path is outside the workspace or missing: {candidate}") from error
    if not is_coverage_source_path(relative):
        raise CoverageDataError(f"{report_kind} references a non-production source path: {relative}")
    return relative


def _parse_coverage_xml(document: str | bytes, report_kind: str) -> ET.Element:
    try:
        root = ET.fromstring(document)
    except (ET.ParseError, TypeError, ValueError) as error:
        raise CoverageDataError(f"{report_kind} coverage report is not valid XML: {error}") from error
    return root


def parse_jacoco_report(document: str | bytes, workspace_root: str | Path) -> dict[str, dict[int, int]]:
    """Parse JaCoCo source-file line counters for the Java production package."""
    root = Path(workspace_root).resolve()
    xml_root = _parse_coverage_xml(document, "JaCoCo")
    if xml_root.tag != "report":
        raise CoverageDataError("JaCoCo coverage report root must be <report>.")
    source_root = root / "bindings/java/src/main/java"
    report: dict[str, dict[int, int]] = {}
    for package in xml_root.findall("package"):
        package_name = package.get("name")
        if not isinstance(package_name, str) or not package_name:
            raise CoverageDataError("JaCoCo package is missing its source path.")
        package_path = PurePosixPath(package_name)
        if package_path.is_absolute() or ".." in package_path.parts:
            raise CoverageDataError(f"JaCoCo package path is invalid: {package_name}")
        for source_file in package.findall("sourcefile"):
            source_name = source_file.get("name")
            if not isinstance(source_name, str) or Path(source_name).name != source_name:
                raise CoverageDataError("JaCoCo sourcefile has an invalid name.")
            source = _coverage_report_source(source_root.joinpath(*package_path.parts, source_name), root, "JaCoCo")
            if source in report:
                raise CoverageDataError(f"JaCoCo report repeats source file: {source}.")
            lines: dict[int, int] = {}
            for line_element in source_file.findall("line"):
                try:
                    line = int(line_element.attrib["nr"])
                    missed = int(line_element.attrib["mi"])
                    covered = int(line_element.attrib["ci"])
                except (KeyError, ValueError) as error:
                    raise CoverageDataError(f"JaCoCo line counter is malformed for {source}.") from error
                if line < 1 or min(missed, covered) < 0 or line in lines:
                    raise CoverageDataError(f"JaCoCo line counter is invalid or repeated for {source}:{line}.")
                lines[line] = int(covered > 0)
            report[source] = lines
    if not report:
        raise CoverageDataError("JaCoCo coverage report contains no source files.")
    return report


def parse_cobertura_report(document: str | bytes, workspace_root: str | Path) -> dict[str, dict[int, int]]:
    """Parse coverage.py Cobertura lines, keeping only KMIPKit Python package code."""
    root = Path(workspace_root).resolve()
    xml_root = _parse_coverage_xml(document, "Cobertura")
    if xml_root.tag != "coverage":
        raise CoverageDataError("Cobertura coverage report root must be <coverage>.")
    source_elements = xml_root.findall("./sources/source")
    source_roots = [Path(element.text or "") for element in source_elements if (element.text or "").strip()]
    if not source_roots:
        source_roots = [Path(".")]
    report: dict[str, dict[int, int]] = {}
    for class_element in xml_root.findall(".//class"):
        filename = class_element.get("filename")
        if not isinstance(filename, str) or not filename:
            raise CoverageDataError("Cobertura class is missing its source filename.")
        relative_filename = Path(filename)
        candidates = []
        for source_root in source_roots:
            base = source_root if source_root.is_absolute() else root / source_root
            candidates.append(relative_filename if relative_filename.is_absolute() else base / relative_filename)
        if not relative_filename.is_absolute():
            candidates.append(root / relative_filename)
        resolved_source: str | None = None
        last_error: CoverageDataError | None = None
        for candidate in candidates:
            try:
                resolved_source = _coverage_report_source(candidate, root, "Cobertura")
                break
            except CoverageDataError as error:
                last_error = error
        if resolved_source is None:
            raise last_error or CoverageDataError(f"Cobertura source path is invalid: {filename}")
        if resolved_source in report:
            raise CoverageDataError(f"Cobertura report repeats source file: {resolved_source}.")
        lines: dict[int, int] = {}
        for line_element in class_element.findall("./lines/line"):
            try:
                line = int(line_element.attrib["number"])
                hits = int(line_element.attrib["hits"])
            except (KeyError, ValueError) as error:
                raise CoverageDataError(f"Cobertura line counter is malformed for {resolved_source}.") from error
            if line < 1 or hits < 0 or line in lines:
                raise CoverageDataError(f"Cobertura line counter is invalid or repeated for {resolved_source}:{line}.")
            lines[line] = hits
        report[resolved_source] = lines
    if not report:
        raise CoverageDataError("Cobertura coverage report contains no source files.")
    return report


def normalize_llvm_export(document: str | bytes, workspace_root: str | Path) -> str:
    """Normalize report file paths to workspace-relative names before upload."""
    root = Path(workspace_root).resolve()
    parsed = _load_json_document(document)
    for data_item in parsed["data"]:
        for file_record in data_item["files"]:
            filename = file_record["filename"]
            canonical = _canonical_source_path(filename, root)
            if canonical is None:
                path = Path(filename)
                if not path.is_absolute():
                    path = root / path
                try:
                    canonical = path.resolve(strict=False).relative_to(root).as_posix()
                except (OSError, ValueError) as error:
                    raise CoverageDataError(f"LLVM report path is outside the workspace: {filename}") from error
            file_record["filename"] = canonical
        for function in data_item["functions"]:
            if not isinstance(function, dict) or not isinstance(function.get("filenames"), list):
                raise CoverageDataError("LLVM function record has no valid filename table.")
            normalized: list[str] = []
            for filename in function["filenames"]:
                if not isinstance(filename, str):
                    raise CoverageDataError("LLVM filename table contains a non-string path.")
                normalized.append(_normalize_llvm_function_filename(filename, root))
            function["filenames"] = normalized
    return json.dumps(parsed, sort_keys=True, separators=(",", ":"))


def _normalize_llvm_function_filename(filename: str, root: Path) -> str:
    """Normalize one function-table path while keeping file records fail-closed."""
    try:
        canonical = _canonical_source_path(filename, root)
    except CoverageDataError:
        path = Path(filename)
        if not path.is_absolute():
            path = root / path
        try:
            path.resolve(strict=False).relative_to(root)
        except (OSError, ValueError):
            # Macro expansion can add registry or standard-library paths to a
            # function's filename table without adding coverage file records.
            return "__external_source__"
        raise
    if canonical is not None:
        return canonical

    # Keep non-production paths relative to the workspace when they belong to it.
    path = Path(filename)
    if not path.is_absolute():
        path = root / path
    try:
        return path.resolve(strict=False).relative_to(root).as_posix()
    except (OSError, ValueError) as error:
        raise CoverageDataError(f"LLVM report path is outside the workspace: {filename}") from error


def _decode_git_diff_path(value: str) -> str:
    """Decode Git's quoted path representation, including UTF-8 octal escapes."""
    if not value.startswith('"'):
        return value
    value = value.rstrip("\t")
    if len(value) < 2 or not value.endswith('"'):
        raise CoverageDataError("Git diff contains an unterminated quoted path.")
    encoded = bytearray()
    index = 1
    while index < len(value) - 1:
        character = value[index]
        if character != "\\":
            encoded.extend(character.encode("utf-8"))
            index += 1
            continue
        index += 1
        if index >= len(value) - 1:
            raise CoverageDataError("Git diff contains an incomplete path escape.")
        escaped = value[index]
        escape_bytes = {"a": 7, "b": 8, "t": 9, "n": 10, "v": 11, "f": 12, "r": 13, "\\": 92, '"': 34}
        if escaped in escape_bytes:
            encoded.append(escape_bytes[escaped])
            index += 1
            continue
        if escaped not in "01234567":
            raise CoverageDataError(f"Git diff contains an unsupported path escape: \\{escaped}.")
        end = index
        while end < min(index + 3, len(value) - 1) and value[end] in "01234567":
            end += 1
        encoded.append(int(value[index:end], 8))
        index = end
    try:
        return encoded.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise CoverageDataError("Git diff path is not valid UTF-8.") from error


def _parse_added_source_lines(diff: str, source_filter: Any) -> dict[str, set[int]]:
    """Return added destination-tree line numbers matching a source predicate."""
    changed: dict[str, set[int]] = {}
    current_path: str | None = None
    new_line = 0
    in_hunk = False
    for line in diff.splitlines():
        if line.startswith("+++ "):
            destination = _decode_git_diff_path(line[4:])
            current_path = None if destination == "/dev/null" else destination.removeprefix("b/")
            if current_path and not source_filter(current_path):
                current_path = None
        elif line.startswith("@@ "):
            match = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", line)
            if not match:
                raise CoverageDataError(f"Malformed unified diff hunk header: {line}")
            new_line = int(match.group(1))
            in_hunk = True
        elif in_hunk and line.startswith("+") and not line.startswith("+++"):
            if current_path:
                changed.setdefault(current_path, set()).add(new_line)
            new_line += 1
        elif in_hunk and line.startswith("-") and not line.startswith("---"):
            continue
        elif in_hunk and line.startswith(" "):
            new_line += 1
        elif line.startswith("diff --git "):
            current_path = None
            in_hunk = False
    return changed


def parse_added_rust_lines(diff: str) -> dict[str, set[int]]:
    """Return added Rust line numbers from production crate source files."""
    return _parse_added_source_lines(diff, _is_rust_source_tree_path)


def parse_added_production_lines(diff: str) -> dict[str, set[int]]:
    """Return added Rust and adapter line numbers from production source files."""
    return _parse_added_source_lines(diff, is_coverage_source_path)


def merge_coverage_reports(reports: Mapping[str, Mapping[str, Mapping[int, int]]]) -> dict[str, dict[int, int]]:
    """Union source-line hits across reports without counting duplicate lines twice."""
    if not isinstance(reports, Mapping) or not reports:
        raise CoverageDataError("Coverage aggregation requires at least one report.")
    merged: dict[str, dict[int, int]] = {}
    for report_name, report in reports.items():
        if not isinstance(report, Mapping):
            raise CoverageDataError(f"{report_name} coverage report is malformed.")
        for source, line_counts in report.items():
            if not is_coverage_source_path(source) or not isinstance(line_counts, Mapping):
                raise CoverageDataError(f"{report_name} report contains an invalid source path or line map.")
            for line, count in line_counts.items():
                if type(line) is not int or line < 1 or type(count) is not int or count < 0:
                    raise CoverageDataError(f"{report_name} report contains invalid line data for {source}.")
                target = merged.setdefault(source, {})
                target[line] = max(target.get(line, 0), count)
    return merged


def merge_platform_reports(reports: Mapping[str, Mapping[str, Mapping[int, int]]]) -> dict[str, dict[int, int]]:
    if set(reports) != {"ubuntu", "windows", "macos"}:
        raise CoverageDataError("Coverage aggregation requires exactly ubuntu, windows, and macos reports.")
    return merge_coverage_reports(reports)


def meets_threshold(covered: int, total: int, threshold: float) -> bool:
    if total <= 0 or covered < 0 or covered > total:
        return False
    return (covered * 100) >= (total * threshold)


def changed_code_result(changed_lines: set[tuple[str, int]] | set[int], coverage: Mapping[str, Mapping[int, int]] | Mapping[int, int]) -> str:
    if not changed_lines:
        return "not applicable"
    executable = 0
    covered = 0
    for item in changed_lines:
        if isinstance(item, tuple):
            source, line = item
            count = coverage.get(source, {}).get(line)
        else:
            line = item
            count = coverage.get(line) if isinstance(coverage, Mapping) else None
        if count is not None:
            executable += 1
            covered += int(count > 0)
    if executable == 0:
        return "not applicable"
    percent = covered * 100 / executable
    return f"{covered}/{executable} executable changed lines covered ({percent:.2f}%)"


def _run_git_diff(workspace: Path, base: str, merge: str) -> str:
    if not re.fullmatch(r"[0-9a-fA-F]{40,64}", base) or not re.fullmatch(r"[0-9a-fA-F]{40,64}", merge):
        raise CoverageDataError("Base and merge identifiers must be full Git object IDs.")
    command = [
        "git",
        "-C",
        str(workspace),
        "diff",
        "--no-ext-diff",
        "--no-renames",
        "--unified=0",
        base,
        merge,
        "--",
        "*.rs",
        "*.java",
        "*.py",
        "*.cpp",
    ]
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True, encoding="utf-8")
    except (OSError, subprocess.CalledProcessError) as error:
        raise CoverageDataError(f"Unable to compute the exact base-to-merge Rust diff: {error}") from error
    return result.stdout


def _load_platform_artifacts(report_root: Path, workspace: Path) -> dict[str, Any]:
    reports: dict[str, Any] = {}
    for platform in ("ubuntu", "windows", "macos"):
        directory = report_root / f"coverage-{platform}"
        report_path = directory / "coverage.json"
        status_path = directory / "coverage-status.json"
        if report_path.is_file() and status_path.exists():
            raise CoverageDataError(f"{platform} artifact contains both measured data and an unavailable status.")
        if report_path.is_file():
            try:
                reports[platform] = parse_llvm_export(report_path.read_text(encoding="utf-8"), workspace)
            except OSError as error:
                raise CoverageDataError(f"Could not read {platform} coverage report: {error}") from error
        elif status_path.is_file():
            try:
                status = json.loads(status_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as error:
                raise CoverageDataError(f"{platform} unavailable artifact is malformed: {error}") from error
            if status != {"status": "unavailable", "reason": "no production function bodies"}:
                raise CoverageDataError(f"{platform} artifact has an invalid unavailable status.")
            reports[platform] = None
        else:
            raise CoverageDataError(f"Required coverage artifact is missing for {platform}.")
    if all(report is None for report in reports.values()):
        print("Coverage unavailable: all platforms verified that no production function bodies exist.")
        return {"status": "unavailable"}
    if any(report is None for report in reports.values()):
        raise CoverageDataError("Platform reports disagree about whether production code is eligible.")
    return reports


def _load_ffi_c_consumer_artifact(
    report_root: Path,
    workspace: Path,
) -> Mapping[str, Mapping[int, int]] | None:
    """Load Rust FFI line hits produced by the instrumented C ABI consumer."""
    source_scan = scan_production_sources(workspace)
    if not source_scan.complete:
        raise CoverageDataError(f"Production source scan is incomplete: {source_scan.reason}")
    expected_sources = {
        source
        for source in source_scan.eligible_files
        if source.startswith("crates/kmipkit-ffi/src/")
    }
    if not expected_sources:
        return None
    report_path = report_root / "coverage-ffi" / "coverage.json"
    if not report_path.is_file():
        raise CoverageDataError(f"Required Rust C ABI coverage artifact is missing: {report_path}.")
    try:
        report = parse_llvm_export(
            report_path.read_text(encoding="utf-8"),
            workspace,
            source_filter=expected_sources,
        )
    except OSError as error:
        raise CoverageDataError(f"Could not read Rust C ABI coverage report: {error}") from error
    missing_sources = expected_sources.difference(report)
    unexpected_sources = set(report).difference(expected_sources)
    if missing_sources:
        raise CoverageDataError(
            f"Eligible Rust FFI source is missing from C ABI coverage report: {', '.join(sorted(missing_sources))}"
        )
    if unexpected_sources:
        raise CoverageDataError(
            f"Rust C ABI coverage report contains a source outside its package: {', '.join(sorted(unexpected_sources))}"
        )
    return report


def _load_adapter_reports(
    report_root: Path,
    workspace: Path,
    required_scopes: set[str] | None = None,
) -> dict[str, Mapping[str, Mapping[int, int]]] | None:
    sources = scan_adapter_sources(workspace)
    scope_names = {
        "java": "Java adapters",
        "python": "Python adapters",
        "jni": "JNI bridge",
    }
    selected_scopes = set(scope_names) if required_scopes is None else required_scopes.intersection(scope_names)
    if not any(sources[scope_names[scope]] for scope in selected_scopes):
        return None
    inputs = {
        "Java adapters": (report_root / "coverage-java" / "jacoco.xml", parse_jacoco_report),
        "Python adapters": (report_root / "coverage-python" / "coverage.xml", parse_cobertura_report),
        "JNI bridge": (report_root / "coverage-jni" / "coverage.json", parse_llvm_export),
    }
    parsed: dict[str, Mapping[str, Mapping[int, int]]] = {}
    for short_scope, scope in scope_names.items():
        if short_scope not in selected_scopes:
            continue
        report_path, parser = inputs[scope]
        expected_sources = sources[scope]
        if not expected_sources:
            continue
        if not report_path.is_file():
            raise CoverageDataError(f"Required {scope} coverage artifact is missing: {report_path}.")
        try:
            document = report_path.read_bytes()
            adapter_report = parser(document, workspace)
        except OSError as error:
            raise CoverageDataError(f"Could not read {scope} coverage report: {error}") from error
        missing_sources = expected_sources.difference(adapter_report)
        unexpected_sources = set(adapter_report).difference(expected_sources)
        if missing_sources:
            raise CoverageDataError(
                f"Eligible {scope} source is missing from its coverage report: {', '.join(sorted(missing_sources))}"
            )
        if unexpected_sources:
            raise CoverageDataError(
                f"{scope} coverage report contains a source outside its package: {', '.join(sorted(unexpected_sources))}"
            )
        parsed[scope] = adapter_report
    return parsed


def _evaluate_coverage(
    workspace: Path,
    reports: Mapping[str, Mapping[str, Mapping[int, int]]],
    diff: str,
    adapter_reports: Mapping[str, Mapping[str, Mapping[int, int]]] | None = None,
    ffi_c_consumer_report: Mapping[str, Mapping[int, int]] | None = None,
    required_scopes: set[str] | None = None,
) -> list[str]:
    if required_scopes is not None:
        allowed_scopes = {"rust", "ffi-c", "java", "python", "jni"}
        unknown_scopes = required_scopes.difference(allowed_scopes)
        if unknown_scopes:
            raise CoverageDataError(f"Unknown coverage scope: {', '.join(sorted(unknown_scopes))}.")
        if required_scopes != allowed_scopes:
            return _evaluate_scoped_coverage(
                workspace,
                reports,
                diff,
                adapter_reports,
                ffi_c_consumer_report,
                required_scopes,
            )

    merged = merge_platform_reports(reports)
    source_scan = scan_production_sources(workspace)
    if not source_scan.eligible:
        raise CoverageDataError("Coverage reports contain production code but source preflight found no executable production files.")
    if not source_scan.complete:
        raise CoverageDataError(f"Production source scan is incomplete: {source_scan.reason}")
    expected_ffi_sources = {
        source
        for source in source_scan.eligible_files
        if source.startswith("crates/kmipkit-ffi/src/")
    }
    if expected_ffi_sources and ffi_c_consumer_report is None:
        raise CoverageDataError("Rust FFI C-consumer coverage report is required for this workspace.")
    if ffi_c_consumer_report is not None:
        missing_ffi_sources = expected_ffi_sources.difference(ffi_c_consumer_report)
        unexpected_ffi_sources = set(ffi_c_consumer_report).difference(expected_ffi_sources)
        if missing_ffi_sources:
            raise CoverageDataError(
                f"Eligible Rust FFI source is missing from C ABI coverage report: {', '.join(sorted(missing_ffi_sources))}"
            )
        if unexpected_ffi_sources:
            raise CoverageDataError(
                f"Rust C ABI coverage report contains a source outside its package: {', '.join(sorted(unexpected_ffi_sources))}"
            )
        merged = merge_coverage_reports({"platforms": merged, "ffi-c-consumer": ffi_c_consumer_report})
    summary_uncovered_excess: dict[str, int] = {}
    for report in reports.values():
        if isinstance(report, CoverageReport):
            for source, count in report.summary_uncovered_excess.items():
                summary_uncovered_excess[source] = summary_uncovered_excess.get(source, 0) + count
    if isinstance(ffi_c_consumer_report, CoverageReport):
        for source, count in ffi_c_consumer_report.summary_uncovered_excess.items():
            summary_uncovered_excess[source] = summary_uncovered_excess.get(source, 0) + count
    missing_sources = source_scan.eligible_files.difference(merged)
    if missing_sources:
        missing = ", ".join(sorted(missing_sources))
        raise CoverageDataError(f"Eligible production source is missing from LLVM coverage reports: {missing}")
    all_lines = {(source, line): count for source, line_counts in merged.items() for line, count in line_counts.items()}
    if not all_lines:
        raise CoverageDataError("Eligible production source has no executable lines in the LLVM reports.")

    adapter_sources = scan_adapter_sources(workspace)
    if any(adapter_sources.values()) and adapter_reports is None:
        raise CoverageDataError("Java, Python, and JNI coverage reports are required for this workspace.")
    adapter_reports = adapter_reports or {}
    adapter_scopes = {
        "Java adapters": 85,
        "Python adapters": 85,
        "JNI bridge": 85,
    }
    for label, threshold in adapter_scopes.items():
        source_paths = adapter_sources[label]
        if not source_paths:
            continue
        if label not in adapter_reports:
            raise CoverageDataError(f"Required {label} coverage report is missing.")
        report = adapter_reports[label]
        missing_adapter_sources = source_paths.difference(report)
        if missing_adapter_sources:
            raise CoverageDataError(
                f"Eligible {label} source is missing from its coverage report: {', '.join(sorted(missing_adapter_sources))}"
            )

    results: list[str] = []
    changed = parse_added_production_lines(diff)
    production_lines = dict(all_lines)
    adapter_summary_excess: dict[str, int] = {}
    for report in adapter_reports.values():
        for source, line_counts in report.items():
            for line, count in line_counts.items():
                production_lines[(source, line)] = count
        if isinstance(report, CoverageReport):
            for source, count in report.summary_uncovered_excess.items():
                adapter_summary_excess[source] = adapter_summary_excess.get(source, 0) + count

    changed_executable = {
        (source, line)
        for source, line_numbers in changed.items()
        for line in line_numbers
        if (source, line) in production_lines
    }
    if changed_executable:
        covered = sum(production_lines[item] > 0 for item in changed_executable)
        changed_sources = {source for source in changed if source in merged}
        summary_only = sum(summary_uncovered_excess.get(source, 0) for source in changed_sources)
        changed_adapter_sources = {source for source in changed if source in adapter_summary_excess}
        summary_only += sum(adapter_summary_excess.get(source, 0) for source in changed_adapter_sources)
        total = len(changed_executable) + summary_only
        rust_only = all(source.endswith(".rs") for source, _ in changed_executable)
        if not meets_threshold(covered, total, 95):
            prefix = "Changed Rust code" if rust_only else "Changed production code"
            raise CoverageDataError(f"{prefix} coverage {covered}/{total} is below 95%.")
        percentage = covered * 100 / total
        label = "Changed Rust" if rust_only else "Changed production"
        results.append(
            f"{label} coverage: {covered}/{total} ({percentage:.2f}%) including {summary_only} summary-only line(s) as uncovered (95% minimum)."
        )
    else:
        changed_sources = {source for source in changed if source in merged}
        summary_only = sum(summary_uncovered_excess.get(source, 0) for source in changed_sources)
        summary_only += sum(adapter_summary_excess.get(source, 0) for source in changed if source in adapter_summary_excess)
        if summary_only:
            changed_rust_only = all(source.endswith(".rs") for source in changed)
            prefix = "Changed Rust code coverage" if changed_rust_only else "Changed production code coverage"
            raise CoverageDataError(
                f"{prefix} cannot be marked not applicable while LLVM summary lines are absent from segments."
            )
        results.append("Changed executable production lines: not applicable.")

    crate_thresholds = {
        "kmipkit-ttlv": 95,
        "kmipkit-protocol": 95,
        "kmipkit-transport": 85,
        "kmipkit-ffi": 85,
    }
    for crate_name, threshold in crate_thresholds.items():
        crate_lines = {
            (source, line): count
            for (source, line), count in all_lines.items()
            if len(PurePosixPath(source).parts) > 1
            and PurePosixPath(source).parts[0] == "crates"
            and PurePosixPath(source).parts[1] == crate_name
        }
        if not crate_lines:
            continue
        covered = sum(count > 0 for count in crate_lines.values())
        summary_only = sum(
            count
            for source, count in summary_uncovered_excess.items()
            if len(PurePosixPath(source).parts) > 1 and PurePosixPath(source).parts[1] == crate_name
        )
        total = len(crate_lines) + summary_only
        if not meets_threshold(covered, total, threshold):
            raise CoverageDataError(f"{crate_name} coverage {covered}/{total} is below {threshold}%.")
        percentage = covered * 100 / total
        results.append(
            f"{crate_name} coverage: {covered}/{total} ({percentage:.2f}%) including {summary_only} summary-only line(s) as uncovered ({threshold}% minimum)."
        )

    for label, threshold in adapter_scopes.items():
        source_paths = adapter_sources[label]
        if not source_paths:
            continue
        report = adapter_reports[label]
        scoped_lines = {
            (source, line): count
            for source, line_counts in report.items()
            if source in source_paths
            for line, count in line_counts.items()
        }
        summary_only = sum(
            count for source, count in adapter_summary_excess.items() if source in source_paths
        )
        covered = sum(count > 0 for count in scoped_lines.values())
        total = len(scoped_lines) + summary_only
        if not meets_threshold(covered, total, threshold):
            raise CoverageDataError(f"{label} coverage {covered}/{total} is below {threshold}%.")
        percentage = covered * 100 / total
        results.append(
            f"{label} coverage: {covered}/{total} ({percentage:.2f}%) including {summary_only} summary-only line(s) as uncovered ({threshold}% minimum)."
        )

    covered = sum(count > 0 for count in all_lines.values())
    summary_only = sum(summary_uncovered_excess.values())
    total = len(all_lines) + summary_only
    if not meets_threshold(covered, total, 90):
        raise CoverageDataError(f"Workspace coverage {covered}/{total} is below 90%.")
    percentage = covered * 100 / total
    results.append(
        f"Workspace coverage: {covered}/{total} ({percentage:.2f}%) including {summary_only} summary-only line(s) as uncovered (90% minimum)."
    )
    return results


def _evaluate_scoped_coverage(
    workspace: Path,
    reports: Mapping[str, Mapping[str, Mapping[int, int]]],
    diff: str,
    adapter_reports: Mapping[str, Mapping[str, Mapping[int, int]]] | None,
    ffi_c_consumer_report: Mapping[str, Mapping[int, int]] | None,
    required_scopes: set[str],
) -> list[str]:
    """Gate only explicitly selected scopes while keeping their normal thresholds."""
    if not required_scopes:
        return ["Selected coverage scopes: not applicable."]

    results: list[str] = []
    scoped_reports: dict[str, Mapping[str, Mapping[int, int]]] = {}
    summary_excess: dict[str, int] = {}
    rust_source_scan: SourceScanResult | None = None

    if "rust" in required_scopes or "ffi-c" in required_scopes:
        rust_source_scan = scan_production_sources(workspace)
        if not rust_source_scan.complete:
            raise CoverageDataError(f"Production source scan is incomplete: {rust_source_scan.reason}")

    if "rust" in required_scopes:
        if reports.get("status") == "unavailable":
            results.append("Rust coverage: unavailable because all three platforms confirmed no production function bodies.")
        elif rust_source_scan is None or not rust_source_scan.eligible:
            results.append("Rust coverage: unavailable because no executable production Rust function bodies exist.")
        else:
            rust_lines = merge_platform_reports(reports)
            missing = rust_source_scan.eligible_files.difference(rust_lines)
            if missing:
                raise CoverageDataError(
                    "Eligible production source is missing from LLVM coverage reports: "
                    + ", ".join(sorted(missing))
                )
            scoped_reports["rust"] = rust_lines
            for report in reports.values():
                if isinstance(report, CoverageReport):
                    for source, count in report.summary_uncovered_excess.items():
                        summary_excess[source] = summary_excess.get(source, 0) + count

            crate_thresholds = {
                "kmipkit-ttlv": 95,
                "kmipkit-protocol": 95,
                "kmipkit-transport": 85,
                "kmipkit-ffi": 85,
            }
            for crate_name, threshold in crate_thresholds.items():
                crate_lines = {
                    (source, line): count
                    for source, line_counts in rust_lines.items()
                    if len(PurePosixPath(source).parts) > 1
                    and PurePosixPath(source).parts[0] == "crates"
                    and PurePosixPath(source).parts[1] == crate_name
                    for line, count in line_counts.items()
                }
                if not crate_lines:
                    continue
                extra = sum(
                    count
                    for source, count in summary_excess.items()
                    if len(PurePosixPath(source).parts) > 1 and PurePosixPath(source).parts[1] == crate_name
                )
                covered = sum(count > 0 for count in crate_lines.values())
                total = len(crate_lines) + extra
                if not meets_threshold(covered, total, threshold):
                    raise CoverageDataError(f"{crate_name} coverage {covered}/{total} is below {threshold}%.")
                results.append(
                    f"{crate_name} coverage: {covered}/{total} ({covered * 100 / total:.2f}%) including {extra} summary-only line(s) as uncovered ({threshold}% minimum)."
                )

            covered = sum(count > 0 for counts in rust_lines.values() for count in counts.values())
            extra = sum(summary_excess.values())
            total = sum(len(counts) for counts in rust_lines.values()) + extra
            if not meets_threshold(covered, total, 90):
                raise CoverageDataError(f"Workspace coverage {covered}/{total} is below 90%.")
            results.append(
                f"Workspace coverage: {covered}/{total} ({covered * 100 / total:.2f}%) including {extra} summary-only line(s) as uncovered (90% minimum)."
            )

    if "ffi-c" in required_scopes:
        if rust_source_scan is None:
            raise CoverageDataError("Rust source preflight is required for FFI/C coverage.")
        ffi_sources = {
            source for source in rust_source_scan.eligible_files if source.startswith("crates/kmipkit-ffi/src/")
        }
        if ffi_sources:
            if ffi_c_consumer_report is None:
                raise CoverageDataError("Rust FFI C-consumer coverage report is required for the selected ffi-c scope.")
            missing = ffi_sources.difference(ffi_c_consumer_report)
            extra_sources = set(ffi_c_consumer_report).difference(ffi_sources)
            if missing:
                raise CoverageDataError(
                    "Eligible Rust FFI source is missing from C ABI coverage report: " + ", ".join(sorted(missing))
                )
            if extra_sources:
                raise CoverageDataError(
                    "Rust C ABI coverage report contains a source outside its package: "
                    + ", ".join(sorted(extra_sources))
                )
            scoped_reports["ffi-c"] = ffi_c_consumer_report
            if isinstance(ffi_c_consumer_report, CoverageReport):
                for source, count in ffi_c_consumer_report.summary_uncovered_excess.items():
                    summary_excess[source] = summary_excess.get(source, 0) + count
            ffi_lines = {
                (source, line): count
                for source, counts in ffi_c_consumer_report.items()
                for line, count in counts.items()
            }
            extra = sum(count for source, count in summary_excess.items() if source in ffi_sources)
            covered = sum(count > 0 for count in ffi_lines.values())
            total = len(ffi_lines) + extra
            if not meets_threshold(covered, total, 85):
                raise CoverageDataError(f"kmipkit-ffi coverage {covered}/{total} is below 85%.")
            results.append(
                f"kmipkit-ffi coverage: {covered}/{total} ({covered * 100 / total:.2f}%) including {extra} summary-only line(s) as uncovered (85% minimum)."
            )
        else:
            results.append("FFI/C coverage: unavailable because no executable Rust FFI source exists.")

    adapter_source_map = scan_adapter_sources(workspace)
    adapter_labels = {"java": "Java adapters", "python": "Python adapters", "jni": "JNI bridge"}
    for short_scope, label in adapter_labels.items():
        if short_scope not in required_scopes:
            continue
        sources = adapter_source_map[label]
        if not sources:
            results.append(f"{label} coverage: unavailable because no production source exists.")
            continue
        if adapter_reports is None or label not in adapter_reports:
            raise CoverageDataError(f"Required {label} coverage report is missing.")
        report = adapter_reports[label]
        missing = sources.difference(report)
        extra_sources = set(report).difference(sources)
        if missing:
            raise CoverageDataError(f"Eligible {label} source is missing from its coverage report: {', '.join(sorted(missing))}")
        if extra_sources:
            raise CoverageDataError(f"{label} coverage report contains a source outside its package: {', '.join(sorted(extra_sources))}")
        scoped_reports[short_scope] = report
        if isinstance(report, CoverageReport):
            for source, count in report.summary_uncovered_excess.items():
                summary_excess[source] = summary_excess.get(source, 0) + count
        line_map = {
            (source, line): count
            for source, counts in report.items()
            for line, count in counts.items()
        }
        extra = sum(count for source, count in summary_excess.items() if source in sources)
        covered = sum(count > 0 for count in line_map.values())
        total = len(line_map) + extra
        if not meets_threshold(covered, total, 85):
            raise CoverageDataError(f"{label} coverage {covered}/{total} is below 85%.")
        results.append(
            f"{label} coverage: {covered}/{total} ({covered * 100 / total:.2f}%) including {extra} summary-only line(s) as uncovered (85% minimum)."
        )

    changed = parse_added_production_lines(diff)
    source_scopes = {
        source: _coverage_scope_for_source(source, scoped_reports)
        for source in changed
    }
    changed_executable = {
        (source, line)
        for source, line_numbers in changed.items()
        if source_scopes[source] is not None
        for line in line_numbers
        if line in scoped_reports[source_scopes[source]].get(source, {})
    }
    if changed_executable:
        covered = sum(
            scoped_reports[source_scopes[source]][source][line] > 0
            for source, line in changed_executable
        )
        extra = sum(count for source, count in summary_excess.items() if source in changed)
        total = len(changed_executable) + extra
        if not meets_threshold(covered, total, 95):
            raise CoverageDataError(f"Changed production code coverage {covered}/{total} is below 95%.")
        results.append(f"Changed production coverage: {covered}/{total} ({covered * 100 / total:.2f}%) (95% minimum).")
    else:
        results.append("Changed executable production lines: not applicable.")
    return results


def _coverage_scope_for_source(
    source: str, scoped_reports: Mapping[str, Mapping[str, Mapping[int, int]]]
) -> str | None:
    """Resolve a changed source to its selected report, preferring ABI coverage for FFI."""
    if source.startswith("crates/kmipkit-ffi/src/") and "ffi-c" in scoped_reports:
        return "ffi-c"
    if source.endswith(".rs") and "rust" in scoped_reports:
        return "rust"
    if source.startswith("bindings/java/src/") and "java" in scoped_reports:
        return "java"
    if source.startswith("bindings/python/src/") and "python" in scoped_reports:
        return "python"
    if source.startswith("bindings/java/native/") and "jni" in scoped_reports:
        return "jni"
    return None


def _command_preflight(args: argparse.Namespace) -> int:
    result = scan_production_sources(args.workspace)
    print(f"source_scan={result.reason}; files={result.files_scanned}")
    if args.status_out:
        path = Path(args.status_out)
        path.parent.mkdir(parents=True, exist_ok=True)
        if result.eligible:
            path.unlink(missing_ok=True)
        else:
            path.write_text(
                json.dumps({"status": "unavailable", "reason": "no production function bodies"}, sort_keys=True),
                encoding="utf-8",
            )
    if args.github_output:
        with Path(args.github_output).open("a", encoding="utf-8", newline="\n") as output:
            output.write(f"eligible={'true' if result.eligible else 'false'}\n")
    return 0


def _command_normalize(args: argparse.Namespace) -> int:
    source = Path(args.input).read_text(encoding="utf-8")
    normalized = normalize_llvm_export(source, args.workspace)
    Path(args.output).write_text(normalized + "\n", encoding="utf-8")
    return 0


def _command_aggregate(args: argparse.Namespace) -> int:
    workspace = Path(args.workspace).resolve()
    required_scopes: set[str] | None = None
    if args.scopes_json is not None:
        try:
            parsed_scopes = json.loads(args.scopes_json)
        except json.JSONDecodeError as error:
            raise CoverageDataError(f"Coverage scope JSON is malformed: {error}") from error
        if not isinstance(parsed_scopes, list) or any(not isinstance(scope, str) for scope in parsed_scopes):
            raise CoverageDataError("Coverage scopes must be a JSON array of strings.")
        required_scopes = set(parsed_scopes)
        allowed_scopes = {"rust", "ffi-c", "java", "python", "jni"}
        if required_scopes.difference(allowed_scopes) or len(required_scopes) != len(parsed_scopes):
            raise CoverageDataError("Coverage scopes contain unknown or duplicate values.")
        if not required_scopes:
            _append_coverage_summary(args.summary_file, "unavailable", ["No production coverage scope was selected."])
            print("Selected coverage scopes: not applicable.")
            return 0

    report_root = Path(args.report_dir)
    reports: Mapping[str, Any] = {}
    if required_scopes is None or "rust" in required_scopes:
        reports = _load_platform_artifacts(report_root, workspace)
    if required_scopes is None and reports.get("status") == "unavailable":
        _append_coverage_summary(
            args.summary_file,
            "unavailable",
            ["All three platforms verified that no production function bodies exist; no threshold is claimed."],
        )
        return 0
    ffi_c_consumer_report = (
        _load_ffi_c_consumer_artifact(report_root, workspace)
        if required_scopes is None or "ffi-c" in required_scopes
        else None
    )
    adapter_reports = _load_adapter_reports(report_root, workspace, required_scopes)
    diff = _run_git_diff(workspace, args.base, args.merge)
    results = _evaluate_coverage(
        workspace,
        reports,
        diff,
        adapter_reports,
        ffi_c_consumer_report,
        required_scopes,
    )
    for result in results:
        print(result)
    _append_coverage_summary(args.summary_file, "passed", results)
    return 0


def _append_coverage_summary(summary_file: str | None, status: str, details: Iterable[str]) -> None:
    if not summary_file:
        return
    headings = {
        "passed": "✅ PASS — coverage thresholds met",
        "unavailable": "⚪ UNAVAILABLE — no coverage threshold claimed",
        "failed": "❌ FAIL — coverage gate failed",
    }
    heading = headings.get(status, "❓ Coverage gate status unknown")
    lines = ["### Coverage gate", "", f"**Result:** {heading}", ""]
    for detail in details:
        safe_detail = " ".join(str(detail).split())
        safe_detail = html.escape(safe_detail, quote=False).replace("|", "&#124;").replace("`", "&#96;")
        lines.append(f"- {safe_detail}")
    lines.append("")
    with Path(summary_file).open("a", encoding="utf-8", newline="\n") as summary:
        summary.write("\n".join(lines))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    preflight = subparsers.add_parser("preflight", help="conservatively scan production Rust sources")
    preflight.add_argument("--workspace", default=".")
    preflight.add_argument("--status-out")
    preflight.add_argument("--github-output")
    preflight.set_defaults(handler=_command_preflight)

    normalize = subparsers.add_parser("normalize", help="normalize an LLVM export for artifact upload")
    normalize.add_argument("--workspace", default=".")
    normalize.add_argument("--input", required=True)
    normalize.add_argument("--output", required=True)
    normalize.set_defaults(handler=_command_normalize)

    aggregate = subparsers.add_parser("aggregate", help="enforce selected line coverage gates")
    aggregate.add_argument("--workspace", default=".")
    aggregate.add_argument("--report-dir", required=True)
    aggregate.add_argument("--base", required=True)
    aggregate.add_argument("--merge", required=True)
    aggregate.add_argument("--summary-file")
    aggregate.add_argument("--scopes-json", help="JSON array of required scopes; omission means full legacy scope")
    aggregate.set_defaults(handler=_command_aggregate)
    return parser


def main(argv: Iterable[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.handler(args)
    except (CoverageDataError, OSError) as error:
        if getattr(args, "command", None) == "aggregate":
            try:
                _append_coverage_summary(getattr(args, "summary_file", None), "failed", [str(error)])
            except OSError as summary_error:
                print(f"coverage summary: {summary_error}", file=sys.stderr)
        print(f"coverage gate: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
