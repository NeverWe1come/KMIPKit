"""Render and verify the deterministic Markdown coverage report."""

from __future__ import annotations

import argparse
from collections import defaultdict
from functools import lru_cache
import json
import os
import re
import subprocess
import sys
import tomllib
import unicodedata
from collections.abc import Mapping
from pathlib import Path
from typing import Any

if __package__:
    from .generate_profiles import _is_unresolved_client_scope
    from .safe_io import PathSecurityError, atomic_write_bytes, confined_path, safe_read_bytes
else:
    from generate_profiles import _is_unresolved_client_scope
    from safe_io import PathSecurityError, atomic_write_bytes, confined_path, safe_read_bytes


REPORT_PATH = Path("specification/catalog/coverage-report.md")
PROFILE_RULE_SOURCE = "crates/kmipkit-protocol/src/profile.rs"
_EMPTY_TYPED_RULE_REGISTRY = re.compile(
    r"(?m)^\s*const TYPED_RULE_MAPPINGS:\s*&\[TypedRuleMapping\]\s*=\s*&\[\];\s*$"
)
_TYPED_RULE_REGISTRY_DECLARATION = re.compile(
    r"(?m)^\s*const TYPED_RULE_MAPPINGS:\s*&\[TypedRuleMapping\]\s*=\s*&\["
)
_TYPED_RULE_MAPPING_ENTRY = re.compile(
    r'''\s*TypedRuleMapping\s*\{\s*
    requirement_id:\s*"(?P<requirement_id>KMIPKIT-REQ-[A-Z0-9.-]+)",\s*
    source_clause_id:\s*"(?P<source_clause_id>KMIPKIT-CLAUSE-[A-Z0-9.-]+)",\s*
    verification_category:\s*GeneratedVerificationCategory::(?P<category>[A-Za-z][A-Za-z0-9_]*),\s*
    evaluator:\s*(?P<evaluator>None|Some\([A-Za-z_][A-Za-z0-9_]*\)),\s*
    test_evidence:\s*&\[(?P<test_evidence>.*?)\],\s*
    constraints:\s*&\[(?P<constraints>.*?)\],?\s*
    \}\s*''',
    re.DOTALL | re.VERBOSE,
)
_RULE_TEST_EVIDENCE_ENTRY = re.compile(
    r'''\s*RuleTestEvidence\s*\{\s*
    test_id:\s*"(?P<test_id>[A-Za-z0-9_./:-]+)",\s*
    kind:\s*RuleTestKind::(?P<kind>[A-Za-z][A-Za-z0-9_]*),?\s*
    \}\s*''',
    re.DOTALL | re.VERBOSE,
)
_RULE_CONSTRAINT_ENTRY = re.compile(
    r'''\s*RuleConstraint\s*\{\s*
    key:\s*"(?P<key>[A-Za-z][A-Za-z0-9_-]*)",\s*
    value:\s*"(?P<value>[A-Za-z][A-Za-z0-9_-]*)",?\s*
    \}\s*''',
    re.DOTALL | re.VERBOSE,
)
_VERIFICATION_CATEGORIES = {
    "RequiredBehavior",
    "ProhibitedBehavior",
    "RecommendedBehaviorOrDeviation",
    "DiscouragedAbsenceOrDeviation",
    "OptionalCapability",
}
_RULE_TEST_KINDS = {
    "PositiveBehavior",
    "NegativeBehavior",
    "RecommendedBehavior",
    "DiscouragedBehaviorAbsent",
    "OptionalCapabilityUsable",
}
_RULE_CONSTRAINTS = {
    ("transport", "raw_tls"),
    ("transport", "https"),
    ("transport_security", "tls"),
    ("transport_security", "tls13"),
    ("tls13_cipher_suites", "kmip_profiles_3_1_2"),
    ("encoding", "ttlv"),
}
_DOUBLE_QUOTED_SECRET_VALUE = r'"(?:\\.|[^"\\\r\n])*"'
_SINGLE_QUOTED_SECRET_VALUE = r"'(?:\\.|[^'\\\r\n])*'"
_SENSITIVE_VALUE = rf"(?:{_DOUBLE_QUOTED_SECRET_VALUE}|{_SINGLE_QUOTED_SECRET_VALUE}|[^\s,;]+)"
_SENSITIVE_ASSIGNMENT = re.compile(
    r"(?i)(?P<label>credential|password|secret|token|private[-_ ]?key|raw[-_ ]?kmip[-_ ]?body)"
    r"(?P<separator>\s*[:=]\s*)"
    rf"(?P<value>{_SENSITIVE_VALUE})"
)
_PEM_PRIVATE_KEY = re.compile(
    r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z0-9 ]*PRIVATE KEY-----",
    re.IGNORECASE,
)
SECTION_ORDER = (
    "Source documents",
    "Count reconciliation",
    "Tag allocation and ranges",
    "Source clause dispositions",
    "Source clause review by section",
    "Unassigned requirements",
    "Requirements needing negative verification",
    "Unassigned protocol elements",
    "Profile states",
    "Profiles by applicability and claim state",
    "Test fixture availability",
    "Test evidence by fixture availability",
    "Open discrepancies",
    "Project policies",
)
BASELINE_PROFILE_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"
_RULE_TEST_KIND_BY_CATEGORY = {
    "RequiredBehavior": "PositiveBehavior",
    "ProhibitedBehavior": "NegativeBehavior",
    "RecommendedBehaviorOrDeviation": "RecommendedBehavior",
    "DiscouragedAbsenceOrDeviation": "DiscouragedBehaviorAbsent",
    "OptionalCapability": "OptionalCapabilityUsable",
}


def _visible_controls(value: Any) -> str:
    text = str(value if value is not None else "")
    output: list[str] = []
    for character in text:
        category = unicodedata.category(character)
        if character == "\n":
            output.append("\\n")
        elif character == "\r":
            output.append("\\r")
        elif character == "\t":
            output.append("\\t")
        elif category in {"Cc", "Cf"}:
            output.append(f"\\u{ord(character):04X}")
        else:
            output.append(character)
    return "".join(output)


def _escape_table_text(value: Any) -> str:
    text = _visible_controls(value)
    replacements = (
        ("\\", "\\\\"),
        ("&", "&amp;"),
        ("<", "&lt;"),
        (">", "&gt;"),
        ("|", "\\|"),
        ("`", "\\`"),
        ("[", "\\["),
        ("]", "\\]"),
        ("(", "\\("),
        (")", "\\)"),
        ("*", "\\*"),
        ("_", "\\_"),
        ("~", "\\~"),
    )
    for old, new in replacements:
        text = text.replace(old, new)
    return text


def _escape_link_label(value: Any) -> str:
    text = _visible_controls(value)
    text = text.replace("\\", "\\\\")
    text = text.replace("[", "\\[").replace("]", "\\]")
    text = text.replace("<", "&lt;").replace(">", "&gt;")
    return text


def _redact_sensitive_values(value: str) -> str:
    """Replace inline secret assignments and PEM keys before rendering diagnostics."""
    redacted = _PEM_PRIVATE_KEY.sub("[redacted]", value)
    return _SENSITIVE_ASSIGNMENT.sub(
        lambda match: f"{match.group('label')}{match.group('separator')}[redacted]",
        redacted,
    )


def _safe_diagnostic_text(value: Any, fallback: str) -> str:
    """Keep arbitrary evidence notes out of reports when they resemble secrets."""
    if not isinstance(value, str):
        return fallback
    return _redact_sensitive_values(value)


def _table(headers: tuple[str, ...], rows: list[tuple[Any, ...]]) -> list[str]:
    lines = ["| " + " | ".join(_escape_table_text(item) for item in headers) + " |"]
    lines.append("| " + " | ".join("---" for _ in headers) + " |")
    for row in rows:
        lines.append("| " + " | ".join(_escape_table_text(item) for item in row) + " |")
    if not rows:
        lines.append("| " + " | ".join("—" for _ in headers) + " |")
    return lines


def _count_by(records: list[dict[str, Any]], field: str) -> list[tuple[str, int]]:
    counts: dict[str, int] = {}
    for record in records:
        key = str(record.get(field, "unspecified"))
        counts[key] = counts.get(key, 0) + 1
    return sorted(counts.items())


def _profile_target_state(profile_id: Any, selected_profile_ids: set[str] | None) -> str:
    if selected_profile_ids is None:
        return "not evaluated"
    return "selected" if profile_id in selected_profile_ids else "not selected"


def _split_rust_registry_entries(source: str) -> list[str]:
    entries: list[str] = []
    delimiters: list[str] = []
    matching_delimiter = {"[": "]", "{": "}", "(": ")"}
    start = 0
    in_string = False
    escaped = False
    index = 0
    while index < len(source):
        character = source[index]
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            index += 1
            continue
        if character == '"':
            in_string = True
        elif source.startswith(("//", "/*"), index):
            raise ValueError("unsupported typed profile rule mapping")
        elif character in matching_delimiter:
            delimiters.append(matching_delimiter[character])
        elif character in "]})":
            if not delimiters or delimiters.pop() != character:
                raise ValueError("unsupported typed profile rule mapping")
        elif character == "," and not delimiters:
            entry = source[start:index].strip()
            if entry:
                entries.append(entry)
            start = index + 1
        index += 1
    if in_string or escaped or delimiters:
        raise ValueError("unsupported typed profile rule mapping")
    final_entry = source[start:].strip()
    if final_entry:
        entries.append(final_entry)
    return entries


def _typed_rule_registry_body(text: str) -> str:
    declaration_count = len(
        re.findall(r"(?m)^\s*const\s+TYPED_RULE_MAPPINGS\b", text)
    )
    declaration = _TYPED_RULE_REGISTRY_DECLARATION.search(text)
    if declaration_count != 1 or declaration is None:
        raise ValueError("unsupported typed profile rule mapping")

    body_start = declaration.end()
    delimiters = ["]"]
    matching_delimiter = {"[": "]", "{": "}", "(": ")"}
    in_string = False
    escaped = False
    index = body_start
    while index < len(text):
        character = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            index += 1
            continue
        if character == '"':
            in_string = True
        elif text.startswith(("//", "/*"), index):
            raise ValueError("unsupported typed profile rule mapping")
        elif character in matching_delimiter:
            delimiters.append(matching_delimiter[character])
        elif character in "]})":
            if not delimiters or delimiters.pop() != character:
                raise ValueError("unsupported typed profile rule mapping")
            if not delimiters:
                suffix = text[index + 1 :].lstrip()
                if not suffix.startswith(";"):
                    raise ValueError("unsupported typed profile rule mapping")
                return text[body_start:index]
        index += 1
    raise ValueError("unsupported typed profile rule mapping")


def _parse_typed_rule_registry(text: str) -> list[dict[str, Any]]:
    body = _typed_rule_registry_body(text)
    if not body.strip():
        return []

    mappings: list[dict[str, Any]] = []
    seen_requirement_ids: set[str] = set()
    for entry in _split_rust_registry_entries(body):
        match = _TYPED_RULE_MAPPING_ENTRY.fullmatch(entry)
        if match is None:
            raise ValueError("unsupported typed profile rule mapping")
        fields = match.groupdict()
        requirement_id = fields["requirement_id"]
        category = fields["category"]
        if requirement_id in seen_requirement_ids or category not in _VERIFICATION_CATEGORIES:
            raise ValueError("unsupported typed profile rule mapping")
        seen_requirement_ids.add(requirement_id)

        evaluator = fields["evaluator"]
        if evaluator != "None" and not re.fullmatch(
            r"Some\([A-Za-z_][A-Za-z0-9_]*\)", evaluator
        ):
            raise ValueError("unsupported typed profile rule mapping")

        evidence: list[dict[str, str]] = []
        for evidence_entry in _split_rust_registry_entries(fields["test_evidence"]):
            evidence_match = _RULE_TEST_EVIDENCE_ENTRY.fullmatch(evidence_entry)
            if evidence_match is None:
                raise ValueError("unsupported typed profile rule mapping")
            evidence_fields = evidence_match.groupdict()
            if evidence_fields["kind"] not in _RULE_TEST_KINDS:
                raise ValueError("unsupported typed profile rule mapping")
            evidence.append(evidence_fields)

        constraints: list[dict[str, str]] = []
        for constraint_entry in _split_rust_registry_entries(fields["constraints"]):
            constraint_match = _RULE_CONSTRAINT_ENTRY.fullmatch(constraint_entry)
            if constraint_match is None:
                raise ValueError("unsupported typed profile rule mapping")
            constraint = constraint_match.groupdict()
            if (constraint["key"], constraint["value"]) not in _RULE_CONSTRAINTS:
                raise ValueError("unsupported typed profile rule mapping")
            constraints.append(constraint)

        transport_only = (
            evaluator == "None"
            and bool(constraints)
            and all(
                (
                    constraint["key"] == "transport_security"
                    and constraint["value"] in {"tls", "tls13"}
                )
                or (
                    constraint["key"] == "tls13_cipher_suites"
                    and constraint["value"] == "kmip_profiles_3_1_2"
                )
                for constraint in constraints
            )
        )

        if not evidence:
            evidence.append({"test_id": "", "kind": ""})
        for item in evidence:
            mappings.append({
                "requirement_id": requirement_id,
                "source_clause_id": fields["source_clause_id"],
                "verification_category": category,
                "evaluator_present": evaluator != "None",
                "transport_only": transport_only,
                "test_kind": item["kind"] or None,
                "test_id": item["test_id"] or None,
            })
    return mappings


def _blank_rust_region(source: str, start: int, end: int) -> str:
    """Blank a Rust comment or literal without changing line positions."""
    return "".join("\n" if char == "\n" else "\r" if char == "\r" else " " for char in source[start:end])


def _strip_rust_comments_and_literals(source: str) -> str | None:
    """Remove Rust comments and literals so their text cannot imitate test items."""
    characters = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            if end < 0:
                end = len(source)
            characters[index:end] = _blank_rust_region(source, index, end)
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            if depth:
                return None
            characters[index:end] = _blank_rust_region(source, index, end)
            index = end
            continue

        raw_prefix = index + 1 if source.startswith(("br", "cr"), index) else index
        if raw_prefix < len(source) and source[raw_prefix] == "r":
            quote_index = raw_prefix + 1
            while quote_index < len(source) and source[quote_index] == "#":
                quote_index += 1
            if quote_index < len(source) and source[quote_index] == '"':
                terminator = '"' + ("#" * (quote_index - raw_prefix - 1))
                end_quote = source.find(terminator, quote_index + 1)
                if end_quote < 0:
                    return None
                end = end_quote + len(terminator)
                characters[index:end] = _blank_rust_region(source, index, end)
                index = end
                continue

        quote_index = index + 1 if source.startswith(("b\"", "c\""), index) else index
        if quote_index < len(source) and source[quote_index] == '"':
            end = quote_index + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            else:
                return None
            characters[index:end] = _blank_rust_region(source, index, end)
            index = end
            continue

        index += 1
    return "".join(characters)


def _rust_test_cfgs_are_active(source: str) -> bool:
    """Reject unknown crate cfgs; the referenced test item's cfg is checked separately."""
    marker = re.compile(r"#\s*!\s*\[\s*cfg(?P<attribute>_attr)?\s*\(")
    cursor = 0
    while match := marker.search(source, cursor):
        if not _rust_item_is_top_level(source, match.start()):
            cursor = match.end()
            continue
        depth = 1
        end = match.end()
        while end < len(source) and depth:
            if source[end] == "(":
                depth += 1
            elif source[end] == ")":
                depth -= 1
            end += 1
        if (
            depth
            or match.group("attribute") is not None
            or source[match.end():end - 1].strip() != "test"
        ):
            return False
        cursor = end
    return True


def _rust_module_item_is_active(source: str, item_start: int) -> bool:
    """Fail closed when a module declaration is guarded by an unknown cfg."""
    line_start = source.rfind("\n", 0, item_start) + 1
    lines = source[:line_start].splitlines()
    for line in reversed(lines):
        attribute = line.strip()
        if not attribute:
            continue
        if not attribute.startswith("#"):
            break
        if re.match(r"#\s*!?\s*\[\s*cfg_attr\s*\(", attribute):
            return False
        cfg = re.fullmatch(r"#\s*!?\s*\[\s*cfg\(\s*([^()]*)\s*\)\s*\]", attribute)
        if re.match(r"#\s*!?\s*\[\s*cfg\s*\(", attribute) and cfg is None:
            return False
        if cfg and cfg.group(1).strip() != "test":
            return False
    return True


def _rust_item_is_top_level(source: str, item_start: int) -> bool:
    """Reject module declarations nested in inline modules or function bodies."""
    depth = 0
    for character in source[:item_start]:
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def _rust_inner_module_cfgs_are_active(source: str, opening_brace: int, item_start: int) -> bool:
    """Reject inactive or unknown inner cfg attributes at an inline module's start."""
    cursor = opening_brace + 1
    while cursor < item_start:
        while cursor < item_start and source[cursor].isspace():
            cursor += 1
        if not source.startswith("#!", cursor):
            return True
        bracket_start = source.find("[", cursor + 2, item_start)
        if bracket_start < 0:
            return False
        depth = 1
        end = bracket_start + 1
        while end < item_start and depth:
            if source[end] == "[":
                depth += 1
            elif source[end] == "]":
                depth -= 1
            end += 1
        if depth:
            return False
        attribute = source[bracket_start + 1:end - 1].strip()
        if re.match(r"cfg_attr\s*\(", attribute):
            return False
        if re.match(r"cfg\s*\(", attribute) and not re.fullmatch(
            r"cfg\s*\(\s*test\s*\)", attribute
        ):
            return False
        cursor = end
    return True


def _rust_test_item_has_active_module_context(source: str, item_start: int) -> bool:
    """Accept root or active inline-module tests, but reject tests nested in bodies."""
    module_stack: list[tuple[bool, int | None]] = []
    statement_start = 0
    for index, character in enumerate(source[:item_start]):
        if character == "{":
            statement = source[statement_start:index].strip()
            module_match = re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*$", statement)
            if module_match is None:
                module_stack.append((False, None))
            else:
                module_attributes = statement[:module_match.start()]
                if re.search(r"#\s*\[\s*cfg_attr\s*\(", module_attributes):
                    return False
                active_attributes = re.sub(
                    r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", "", module_attributes
                )
                active = not re.search(r"#\s*\[\s*cfg\s*\(", active_attributes)
                module_stack.append((active, index))
            statement_start = index + 1
        elif character == "}":
            if not module_stack:
                return False
            module_stack.pop()
            statement_start = index + 1
        elif character == ";":
            statement_start = index + 1
    return all(
        active
        and opening_brace is not None
        and _rust_inner_module_cfgs_are_active(source, opening_brace, item_start)
        for active, opening_brace in module_stack
    )


def _confined_rust_path(repository: Path, candidate: Path) -> Path | None:
    """Normalize a module path lexically, then reject links and escapes."""
    try:
        lexical_root = Path(os.path.abspath(repository))
        lexical_candidate = Path(os.path.abspath(candidate))
        relative = lexical_candidate.relative_to(lexical_root).as_posix()
        return confined_path(repository, relative)
    except (OSError, PathSecurityError, ValueError):
        return None


def _rust_child_modules(repository: Path, package_root: Path, source_path: Path) -> set[Path]:
    """Resolve simple Rust module declarations from a package source file."""
    try:
        source = safe_read_bytes(repository, source_path.relative_to(repository).as_posix(), max_bytes=1_048_576).decode("utf-8")
    except (PathSecurityError, UnicodeDecodeError, ValueError):
        return set()
    code = _strip_rust_comments_and_literals(source)
    if code is None:
        return set()

    children: set[Path] = set()
    path_module = re.compile(
        r'#\s*\[\s*path\s*=\s*"(?P<path>[^"\\]+)"\s*\]\s*'
        r'(?:pub(?:\s*\([^]]+\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;'
    )
    path_spans: list[tuple[int, int]] = []
    for match in path_module.finditer(source):
        if code[match.start()] != "#":
            continue
        path_spans.append(match.span())
        if not _rust_item_is_top_level(code, match.start()) or not _rust_module_item_is_active(code, match.start()):
            continue
        candidate = _confined_rust_path(repository, source_path.parent / match.group("path"))
        if candidate is None:
            continue
        try:
            candidate.relative_to(package_root.resolve())
        except ValueError:
            continue
        if candidate.is_file():
            children.add(candidate)

    ordinary_module = re.compile(
        r"(?m)^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+"
        r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*;"
    )
    for match in ordinary_module.finditer(code):
        if any(start <= match.start() < end for start, end in path_spans):
            continue
        if (
            not _rust_item_is_top_level(code, match.start())
            or not _rust_module_item_is_active(code, match.start())
        ):
            continue
        base = source_path.parent if source_path.stem in {"lib", "main", "mod"} else source_path.parent / source_path.stem
        candidates = [base / f"{match.group('name')}.rs", base / match.group("name") / "mod.rs"]
        existing = [
            confined
            for candidate in candidates
            if (confined := _confined_rust_path(repository, candidate)) is not None and confined.is_file()
        ]
        if len(existing) != 1:
            continue
        candidate = existing[0]
        try:
            candidate.relative_to(package_root.resolve())
        except ValueError:
            continue
        children.add(candidate)
    return children


@lru_cache(maxsize=32)
def _rust_package_test_module_paths(repository_text: str, package_text: str) -> frozenset[Path]:
    """Cache Cargo test-root reachability for all evidence rows in one package."""
    repository = Path(repository_text)
    package_root = Path(package_text)
    try:
        manifest = tomllib.loads(safe_read_bytes(repository, (package_root / "Cargo.toml").relative_to(repository).as_posix(), max_bytes=1_048_576).decode("utf-8"))
    except (PathSecurityError, UnicodeDecodeError, tomllib.TOMLDecodeError, ValueError):
        return frozenset()
    package_options = manifest.get("package")
    if not isinstance(package_options, dict):
        return frozenset()
    autolib = package_options.get("autolib", True)
    autotests = package_options.get("autotests", True)
    if not isinstance(autolib, bool) or not isinstance(autotests, bool):
        return frozenset()

    library = manifest.get("lib")
    roots: list[Path] = []
    if library is not None and not isinstance(library, dict):
        return frozenset()
    library_options = library if isinstance(library, dict) else {}
    library_test = library_options.get("test", True)
    library_harness = library_options.get("harness", True)
    if not isinstance(library_test, bool) or not isinstance(library_harness, bool):
        return frozenset()
    if (library is not None or autolib) and library_test and library_harness:
        root_relative = library_options.get("path", "src/lib.rs")
        if not isinstance(root_relative, str):
            return frozenset()
        root = _confined_rust_path(repository, package_root / root_relative)
        if root is None:
            return frozenset()
        try:
            root.relative_to(package_root.resolve())
        except ValueError:
            return frozenset()
        if root.is_file():
            roots.append(root)

    if autotests:
        tests_directory = package_root / "tests"
        if tests_directory.is_dir():
            roots.extend(tests_directory.glob("*.rs"))
    explicit_tests = manifest.get("test", [])
    if isinstance(explicit_tests, dict):
        explicit_tests = [explicit_tests]
    if isinstance(explicit_tests, list):
        for target in explicit_tests:
            if not isinstance(target, dict):
                continue
            target_test = target.get("test", True)
            target_harness = target.get("harness", True)
            if not isinstance(target_test, bool) or not isinstance(target_harness, bool):
                continue
            if not target_test or not target_harness:
                continue
            target_path = target.get("path")
            target_name = target.get("name")
            if isinstance(target_path, str):
                roots.append(package_root / target_path)
            elif isinstance(target_name, str):
                roots.append(package_root / "tests" / f"{target_name}.rs")

    package_root_resolved = package_root.resolve()
    pending: list[Path] = []
    for candidate in roots:
        candidate = _confined_rust_path(repository, candidate)
        if candidate is None:
            continue
        try:
            candidate.relative_to(package_root_resolved)
        except ValueError:
            continue
        if candidate.is_file():
            pending.append(candidate)
    visited: set[Path] = set()
    while pending and len(visited) < 2_000:
        current = pending.pop()
        if current in visited:
            continue
        visited.add(current)
        pending.extend(_rust_child_modules(repository, package_root, current) - visited)
    return frozenset(visited)


def _rust_test_file_is_in_package_module_tree(repository: Path, test_path: Path) -> bool:
    """Require test evidence to be reachable from a Cargo test target root."""
    package_root = next((parent for parent in test_path.parents if (parent / "Cargo.toml").is_file()), None)
    if package_root is None:
        return False
    try:
        test_path_resolved = _confined_rust_path(repository, test_path)
        if test_path_resolved is None:
            return False
        test_path_resolved.relative_to(package_root.resolve())
        reachable = _rust_package_test_module_paths(str(repository.resolve()), str(package_root.resolve()))
    except (OSError, ValueError):
        return False
    return test_path_resolved in reachable


def _typed_rule_test_reference_resolves(repository: Path, reference: str) -> bool:
    """Require typed rule evidence to name an in-repository, active Rust test."""
    test_path, separator, test_name = reference.partition("::")
    if (
        not separator
        or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", test_name)
        or not test_path.endswith(".rs")
    ):
        return False
    try:
        source = safe_read_bytes(repository, test_path, max_bytes=1_048_576).decode("utf-8")
    except (PathSecurityError, UnicodeDecodeError):
        return False
    source = _strip_rust_comments_and_literals(source)
    if source is None or not _rust_test_cfgs_are_active(source):
        return False
    if not _rust_test_file_is_in_package_module_tree(repository, repository / test_path):
        return False

    declaration = re.compile(rf"^\s*(?:async\s+)?fn\s+{re.escape(test_name)}\s*\(", re.MULTILINE)
    test_attribute = re.compile(r"^\s*#\[(?:test|tokio::test)(?:\s*\([^]]*\))?\]\s*$")
    cfg_test_attribute = re.compile(r"^\s*#\[cfg\(\s*test\s*\)\]\s*$")
    ignorable_attribute = re.compile(r"^\s*#\[(?:allow|expect|doc|must_use|inline)(?:\s*\([^]]*\))?\]\s*$")
    lines = source.splitlines()
    line_offsets: list[int] = []
    offset = 0
    for raw_line in source.splitlines(keepends=True):
        line_offsets.append(offset)
        offset += len(raw_line)
    for index, line in enumerate(lines):
        if (
            not declaration.match(line)
            or not _rust_test_item_has_active_module_context(source, line_offsets[index])
        ):
            continue
        cursor = index - 1
        is_test = False
        while cursor >= 0:
            preceding_line = lines[cursor].strip()
            if not preceding_line or preceding_line.startswith("//"):
                cursor -= 1
                continue
            if test_attribute.fullmatch(preceding_line):
                is_test = True
                cursor -= 1
                continue
            if cfg_test_attribute.fullmatch(preceding_line) or ignorable_attribute.fullmatch(preceding_line):
                cursor -= 1
                continue
            if preceding_line.startswith("#["):
                return False
            break
        if is_test:
            return True
    return False


def load_typed_rule_mappings(repository: Path) -> list[dict[str, Any]]:
    """Read only the explicit, static Rust profile-rule registry grammar."""
    source = safe_read_bytes(repository, PROFILE_RULE_SOURCE, max_bytes=4 * 1024 * 1024)
    try:
        text = source.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("typed profile rule registry is not valid UTF-8") from error
    if _EMPTY_TYPED_RULE_REGISTRY.search(text):
        return []
    mappings = _parse_typed_rule_registry(text)
    if any(
        mapping["test_id"]
        and not _typed_rule_test_reference_resolves(repository, mapping["test_id"])
        for mapping in mappings
    ):
        raise ValueError("unresolvable typed profile rule test evidence")
    return mappings


def _clause_section_rows(clauses: list[dict[str, Any]]) -> list[tuple[str, str, int, str]]:
    grouped: dict[tuple[str, str], dict[str, int]] = defaultdict(dict)
    for clause in clauses:
        key = (str(clause.get("source_id", "unspecified")), str(clause.get("section", "unspecified")))
        disposition = str(clause.get("disposition", "unspecified"))
        counts = grouped[key]
        counts[disposition] = counts.get(disposition, 0) + 1

    def section_key(value: str) -> tuple[int, tuple[int, ...], str]:
        parts = value.split(".")
        if all(part.isdigit() for part in parts):
            return (0, tuple(int(part) for part in parts), value)
        return (1, (), value)

    rows = []
    for (source_id, section), counts in sorted(
        grouped.items(), key=lambda item: (item[0][0], section_key(item[0][1]))
    ):
        disposition_text = ", ".join(f"{name}: {count}" for name, count in sorted(counts.items()))
        rows.append((source_id, section, sum(counts.values()), disposition_text))
    return rows


def _source_reference(record: dict[str, Any]) -> str:
    references = record.get("source_refs") or []
    return ", ".join(f"{item.get('source_id', '?')} §{item.get('section', '?')}" for item in references)


def _typed_rule_coverage(
    requirement: dict[str, Any],
    source_clause_ids: list[str],
    candidates: list[dict[str, Any]],
    typed_rule_mappings: list[dict[str, Any]] | None,
    accepted_deviation_id: str | None = None,
) -> tuple[str, str, str, list[str], list[dict[str, Any]], bool | None]:
    if __package__:
        from .generate_profiles import _verification_category
    else:
        from generate_profiles import _verification_category

    source_keyword = requirement.get("source_keyword")
    source_force = requirement.get("source_force")
    category_source = source_keyword if isinstance(source_keyword, str) else source_force
    try:
        category = _verification_category(category_source) if isinstance(category_source, str) else "unknown"
    except ValueError:
        category = "unknown"

    valid_mappings = [
        mapping
        for mapping in candidates
        if mapping.get("source_clause_id") in source_clause_ids
        and mapping.get("verification_category") == category
        and (
            mapping.get("evaluator_present") is True
            or mapping.get("transport_only") is True
        )
    ]
    if typed_rule_mappings is None:
        mapping_status = "not evaluated"
        test_status = "not evaluated"
        complete = None
    else:
        mapping_status = (
            "Typed rule mapping: present"
            if valid_mappings
            else "Invalid typed rule mapping"
            if candidates
            else "No typed rule mapping"
        )
        expected_test_kind = _RULE_TEST_KIND_BY_CATEGORY.get(category)
        test_ids = sorted({
            test_id
            for mapping in valid_mappings
            if mapping.get("test_kind") == expected_test_kind
            and isinstance((test_id := mapping.get("test_id")), str)
            and test_id.strip()
        })
        has_accepted_deviation = accepted_deviation_id is not None and category in {
            "RecommendedBehaviorOrDeviation",
            "DiscouragedAbsenceOrDeviation",
        }
        if has_accepted_deviation:
            test_status = f"Accepted deviation: {accepted_deviation_id}"
            complete = bool(valid_mappings)
        else:
            test_status = (
                "Category test reference: present"
                if test_ids
                else "No category-appropriate rule test"
            )
            complete = bool(valid_mappings and test_ids)

    mapping_test_ids = sorted({
        value
        for mapping in candidates
        if isinstance((value := mapping.get("test_id")), str) and value.strip()
    })
    return (
        f"{category_source or 'unknown'} / {category}",
        mapping_status,
        test_status,
        mapping_test_ids,
        valid_mappings,
        complete,
    )


def _profile_requirement_coverage(
    catalog: dict[str, Any],
    typed_rule_mappings: list[dict[str, Any]] | None,
) -> tuple[list[tuple[Any, ...]], dict[str, bool | None]]:
    profiles = [row for row in catalog.get("profiles", []) if row.get("role") == "client"]
    requirements_by_id = {
        row.get("requirement_id"): row
        for row in catalog.get("requirements", [])
        if row.get("requirement_id")
    }
    profiles_by_requirement: dict[str, set[str]] = defaultdict(set)
    profiles_by_id = {row.get("profile_id"): row for row in profiles if row.get("profile_id")}
    unresolved_scope_by_profile = {
        profile_id: _unresolved_profile_scope_blockers(catalog, profile)
        for profile_id, profile in profiles_by_id.items()
        if isinstance(profile_id, str)
    }
    for profile in profiles:
        profile_id = profile.get("profile_id")
        if not isinstance(profile_id, str):
            continue
        for requirement_id in profile.get("requirement_ids") or []:
            if isinstance(requirement_id, str):
                profiles_by_requirement[requirement_id].add(profile_id)

    mappings_by_requirement: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for mapping in typed_rule_mappings or []:
        requirement_id = mapping.get("requirement_id")
        if isinstance(requirement_id, str):
            mappings_by_requirement[requirement_id].append(mapping)
    decisions_by_id = {
        row.get("decision_id"): row
        for row in catalog.get("decisions", [])
        if isinstance(row.get("decision_id"), str)
    }

    rows: list[tuple[Any, ...]] = []
    requirement_complete: dict[str, bool | None] = {}
    for requirement_id in sorted(profiles_by_requirement):
        requirement = requirements_by_id.get(requirement_id)
        if requirement is None:
            rows.append((
                requirement_id,
                "unresolved catalog requirement",
                "unknown",
                "not evaluated" if typed_rule_mappings is None else "No typed rule mapping",
                "not evaluated" if typed_rule_mappings is None else "No category-appropriate rule test",
                "—", "—", "—", "—", "—", "—",
            ))
            requirement_complete[requirement_id] = False if typed_rule_mappings is not None else None
            continue

        source_clause_ids = sorted(
            value for value in requirement.get("source_clause_ids") or [] if isinstance(value, str)
        )
        candidates = mappings_by_requirement.get(requirement_id, [])
        decision_id = requirement.get("decision_id")
        decision = decisions_by_id.get(decision_id)
        accepted_deviation_id = (
            decision_id
            if isinstance(decision_id, str)
            and decision is not None
            and decision.get("status") == "accepted"
            and requirement_id in decision.get("requirement_ids", [])
            else None
        )
        (
            category_label,
            mapping_status,
            test_status,
            mapping_test_ids,
            valid_mappings,
            requirement_complete[requirement_id],
        ) = _typed_rule_coverage(
            requirement,
            source_clause_ids,
            candidates,
            typed_rule_mappings,
            accepted_deviation_id,
        )

        implementation_refs = sorted(
            value for value in requirement.get("implementation_refs") or [] if isinstance(value, str)
        )
        verification_refs = sorted(
            value for value in requirement.get("verification_refs") or [] if isinstance(value, str)
        )
        official_test_ids = sorted(
            value for value in requirement.get("test_case_ids") or [] if isinstance(value, str)
        )
        rule_implementation_refs = [PROFILE_RULE_SOURCE] if valid_mappings else []
        rows.append((
            requirement_id,
            ", ".join(source_clause_ids),
            category_label,
            mapping_status,
            test_status,
            ", ".join(mapping_test_ids) or "—",
            ", ".join(rule_implementation_refs) or "—",
            ", ".join(implementation_refs) or "—",
            ", ".join(verification_refs) or "—",
            ", ".join(official_test_ids) or "—",
            decision_id or "none catalogued",
            _source_reference(requirement) or "—",
        ))

    runtime_availability: dict[str, bool | None] = {}
    resolving: set[str] = set()

    def is_runtime_available(profile_id: str) -> bool | None:
        if profile_id in runtime_availability:
            return runtime_availability[profile_id]
        profile = profiles_by_id.get(profile_id)
        if profile is None or profile.get("applicability") != "client_1_0":
            runtime_availability[profile_id] = False
            return False
        if unresolved_scope_by_profile.get(profile_id):
            runtime_availability[profile_id] = False
            return False
        if typed_rule_mappings is None:
            runtime_availability[profile_id] = None
            return None
        if profile_id in resolving:
            runtime_availability[profile_id] = False
            return False
        resolving.add(profile_id)
        requirement_ids = profile.get("requirement_ids") or []
        ready = bool(requirement_ids) and all(
            requirement_complete.get(requirement_id) is True
            for requirement_id in requirement_ids
        )
        ready = ready and all(
            is_runtime_available(dependency_id)
            for dependency_id in profile.get("dependency_profile_ids") or []
        )
        resolving.remove(profile_id)
        runtime_availability[profile_id] = bool(ready)
        return runtime_availability[profile_id]

    for profile_id in profiles_by_id:
        is_runtime_available(profile_id)
    return rows, runtime_availability


def _profile_applicability_blockers(profile: dict[str, Any]) -> list[str]:
    """Return only role and applicability blockers for a profile."""
    blockers: list[str] = []
    if profile.get("role") != "client":
        blockers.append("profile is not a client profile")
    applicability = profile.get("applicability")
    if applicability == "conditional":
        blockers.append("conditional profile requires explicit satisfied configuration")
    elif applicability != "client_1_0":
        blockers.append("profile is outside the KMIPKit 1.0 client scope")
    return blockers


def _unresolved_profile_scope_blockers(
    catalog: dict[str, Any],
    profile: dict[str, Any],
) -> list[str]:
    """Identify unresolved client duties linked to a profile's clause closure."""
    clause_ids = {
        value
        for value in profile.get("source_clause_ids") or []
        if isinstance(value, str)
    }
    blockers: list[str] = []
    for requirement in catalog.get("requirements", []):
        if not isinstance(requirement, dict) or not _is_unresolved_client_scope(requirement):
            continue
        scope_state = requirement.get("scope_state")
        requirement_id = requirement.get("requirement_id")
        if not isinstance(requirement_id, str):
            continue
        linked_clauses = sorted(
            clause_id
            for clause_id in requirement.get("source_clause_ids") or []
            if isinstance(clause_id, str) and clause_id in clause_ids
        )
        blockers.extend(
            f"unresolved applicability scope: {requirement_id} at {clause_id} ({scope_state})"
            for clause_id in linked_clauses
        )
    return sorted(set(blockers))


def _effective_catalog_evidence_state(
    catalog_state: str,
    blockers: list[str],
) -> tuple[str, list[str]]:
    """Reconcile only the catalog evidence lifecycle against current blockers."""
    unique_blockers = sorted(set(blockers))
    if unique_blockers and catalog_state == "evidence_complete":
        effective_state = "evidence_incomplete"
        unique_blockers.append("catalog claim_state evidence_complete is stale")
    elif not unique_blockers and catalog_state != "evidence_complete":
        effective_state = catalog_state
        unique_blockers.append("catalog claim_state requires a human-reviewed update")
    else:
        effective_state = catalog_state
    return effective_state, sorted(set(unique_blockers))


def _evaluate_profile_readiness(
    catalog: dict[str, Any],
    profile_id: str,
    *,
    selected_profile_ids: set[str] | None,
    typed_rule_mappings: list[dict[str, Any]] | None,
    verification_manifest: dict[str, Any] | None,
    current_context: dict[str, Any] | None,
    current_head: str | None = None,
    repository_root: Path | None = None,
    dependency_path: tuple[str, ...],
) -> dict[str, Any]:
    """Evaluate stable catalog obligations against one current CI run, without publishing a claim."""
    profiles_by_id = {
        row.get("profile_id"): row
        for row in catalog.get("profiles", [])
        if isinstance(row.get("profile_id"), str)
    }
    profile = profiles_by_id.get(profile_id)
    selected = None if selected_profile_ids is None else profile_id in selected_profile_ids
    if profile is None:
        return {
            "profile_id": profile_id,
            "target_selected": selected,
            "runtime_validation_available": False,
            "effective_state": "evidence_incomplete",
            "blockers": ["profile is missing from the catalog"],
            "published_claim": False,
        }

    blockers: list[str] = []
    if verification_manifest is None:
        blockers.append("current protected-release-push verification manifest is unavailable")
    else:
        try:
            import sys

            root = Path(repository_root or Path(__file__).resolve().parents[2]).resolve(strict=True)
            root_text = str(root)
            if root_text not in sys.path:
                sys.path.insert(0, root_text)
            from scripts.ci_summary import validate_evidence_manifest

            validate_evidence_manifest(
                verification_manifest,
                event_name=str((current_context or {}).get("event_name", "")),
                current_context=current_context or {},
                current_head=current_head or "",
            )
        except (ImportError, OSError, TypeError, ValueError):
            blockers.append("current verification manifest is invalid or does not match this run")

    blockers.extend(_profile_applicability_blockers(profile))
    blockers.extend(_unresolved_profile_scope_blockers(catalog, profile))

    requirement_rows, runtime_by_profile = _profile_requirement_coverage(
        catalog,
        typed_rule_mappings,
    )
    requirement_coverage_by_id = {row[0]: row for row in requirement_rows}
    requirements_by_id = {
        row.get("requirement_id"): row
        for row in catalog.get("requirements", [])
        if isinstance(row.get("requirement_id"), str)
    }
    if typed_rule_mappings is None:
        blockers.append("typed rule registry has not been evaluated")

    requirement_ids = profile.get("requirement_ids")
    if not isinstance(requirement_ids, list) or not requirement_ids:
        blockers.append("profile has no normative requirements")
        requirement_ids = []
    for requirement_id in requirement_ids:
        if not isinstance(requirement_id, str):
            blockers.append("profile contains a malformed requirement link")
            continue
        requirement = requirements_by_id.get(requirement_id)
        if requirement is None:
            blockers.append(f"unresolved normative requirement: {requirement_id}")
            continue
        if not requirement.get("implementation_refs"):
            blockers.append(f"missing implementation assignment: {requirement_id}")
        if not requirement.get("verification_refs"):
            blockers.append(f"missing verification assignment: {requirement_id}")
        coverage = requirement_coverage_by_id.get(requirement_id)
        if coverage is None:
            blockers.append(f"requirement is outside the checked profile closure: {requirement_id}")
            continue
        if coverage[3] != "Typed rule mapping: present":
            blockers.append(f"typed rule mapping incomplete: {requirement_id}")
        if coverage[4] not in {"Category test reference: present"} and not coverage[4].startswith(
            "Accepted deviation: "
        ):
            blockers.append(f"category-appropriate test incomplete: {requirement_id}")

    test_case_ids = set(
        value for value in profile.get("test_case_ids", []) if isinstance(value, str)
    )
    for requirement_id in requirement_ids:
        requirement = requirements_by_id.get(requirement_id) if isinstance(requirement_id, str) else None
        if requirement is not None:
            test_case_ids.update(
                value for value in requirement.get("test_case_ids", []) if isinstance(value, str)
            )
    test_cases_by_id = {
        row.get("test_id"): row
        for row in catalog.get("test_cases", [])
        if isinstance(row.get("test_id"), str)
    }
    if not test_case_ids:
        blockers.append("profile has no linked official or project test cases")
    official_sources: set[str] = set()
    for test_id in sorted(test_case_ids):
        test_case = test_cases_by_id.get(test_id)
        if test_case is None:
            blockers.append(f"unresolved linked test case: {test_id}")
            continue
        if test_case.get("source_id") == "KMIPKIT-SRC-testcases":
            official_sources.add("KMIPKIT-SRC-testcases")
        if test_case.get("source_id") == "KMIPKIT-SRC-profiles":
            official_sources.add("KMIPKIT-SRC-profiles")
        if (
            test_case.get("mandatory_status") == "mandatory"
            and test_case.get("fixture_availability") != "available"
        ):
            blockers.append(f"mandatory fixture unavailable: {test_id}")
    if not official_sources:
        blockers.append("profile has no linked official test case")

    current_dependency_path = (*dependency_path, profile_id)
    for dependency_id in profile.get("dependency_profile_ids", []):
        if not isinstance(dependency_id, str):
            blockers.append("profile contains a malformed dependency link")
            continue
        if dependency_id in current_dependency_path:
            blockers.append(f"dependency cycle detected: {dependency_id}")
            continue
        dependency_readiness = _evaluate_profile_readiness(
            catalog,
            dependency_id,
            selected_profile_ids=selected_profile_ids,
            typed_rule_mappings=typed_rule_mappings,
            verification_manifest=verification_manifest,
            current_context=current_context,
            current_head=current_head,
            repository_root=repository_root,
            dependency_path=current_dependency_path,
        )
        if (
            dependency_readiness["effective_state"] != "evidence_complete"
            or dependency_readiness["blockers"]
        ):
            blockers.append(f"dependency profile is not evidence complete: {dependency_id}")
            blockers.extend(
                f"dependency {dependency_id}: {dependency_blocker}"
                for dependency_blocker in dependency_readiness["blockers"]
            )
    runtime_available = runtime_by_profile.get(profile_id) is True
    if not runtime_available:
        blockers.append("runtime profile validation is unavailable")

    catalog_state = str(profile.get("claim_state", "not_claimed"))
    effective_state, unique_blockers = _effective_catalog_evidence_state(catalog_state, blockers)
    return {
        "profile_id": profile_id,
        "target_selected": selected,
        "runtime_validation_available": runtime_available,
        "effective_state": effective_state,
        "blockers": sorted(set(unique_blockers)),
        "published_claim": False,
    }


def evaluate_profile_readiness(
    catalog: dict[str, Any],
    profile_id: str,
    *,
    selected_profile_ids: set[str] | None,
    typed_rule_mappings: list[dict[str, Any]] | None,
    verification_manifest: dict[str, Any] | None,
    current_context: dict[str, Any] | None,
    current_head: str | None = None,
    repository_root: Path | None = None,
) -> dict[str, Any]:
    """Evaluate one public profile request and its dependency closure."""
    return _evaluate_profile_readiness(
        catalog,
        profile_id,
        selected_profile_ids=selected_profile_ids,
        typed_rule_mappings=typed_rule_mappings,
        verification_manifest=verification_manifest,
        current_context=current_context,
        current_head=current_head,
        repository_root=repository_root,
        dependency_path=(),
    )


def render_report(
    catalog: dict[str, Any],
    *,
    selected_profile_ids: set[str] | None = None,
    typed_rule_mappings: list[dict[str, Any]] | None = None,
) -> str:
    """Render a stable report; all catalog strings are inert escaped text."""
    lines = [
        "# KMIP 2.1 inventory coverage",
        "",
        "This report records inventory coverage and evidence state. It does not claim profile conformance or certification.",
        "",
    ]

    sources = sorted(catalog.get("sources", []), key=lambda row: row.get("source_id", ""))
    lines.extend(["## Source documents", ""])
    lines.extend(_table(
        ("Source ID", "Title", "Authority", "SHA-256"),
        [(row.get("source_id"), row.get("title"), row.get("authority_class"), row.get("sha256")) for row in sources],
    ))
    lines.append("")

    elements = catalog.get("elements", [])
    requirements = catalog.get("requirements", [])
    open_discrepancies = sorted(
        (row for row in catalog.get("discrepancies", []) if row.get("state") == "open"),
        key=lambda row: row.get("discrepancy_id", ""),
    )
    operations = [row for row in elements if row.get("kind") == "operation"]
    count_rows: list[tuple[Any, ...]] = [
        ("Sources", len(sources)),
        ("Source clauses", len(catalog.get("source_clauses", []))),
        ("Protocol elements", len(elements)),
        ("Client-to-server operations", sum(row.get("direction") == "client_to_server" for row in operations)),
        ("Server-to-client operations", sum(row.get("direction") == "server_to_client" for row in operations)),
        ("Tag ranges", len(catalog.get("tag_ranges", []))),
        ("Normative requirements", len(requirements)),
        ("Profiles", len(catalog.get("profiles", []))),
        ("Test cases", len(catalog.get("test_cases", []))),
        ("Open discrepancies", len(open_discrepancies)),
        ("Project policies", len(catalog.get("policies", []))),
    ]
    lines.extend(["## Count reconciliation", ""])
    lines.extend(_table(("Record class", "Count"), count_rows))
    lines.extend(["", "### Elements by kind", ""])
    lines.extend(_table(("Kind", "Count"), _count_by(elements, "kind")))
    lines.extend(["", "### Requirements by strength and scope", ""])
    lines.extend(_table(("Dimension", "Value", "Count"), [
        *(("Strength", key, value) for key, value in _count_by(requirements, "normative_strength")),
        *(("Scope", key, value) for key, value in _count_by(requirements, "scope_state")),
        *(("Direction", key, value) for key, value in _count_by(requirements, "direction")),
    ]))
    lines.append("")

    tags = [row for row in elements if row.get("kind") == "tag"]
    tag_ranges = sorted(catalog.get("tag_ranges", []), key=lambda row: row.get("source_order", 0))
    lines.extend(["## Tag allocation and ranges", ""])
    lines.extend(["### Named tags by OASIS allocation", ""])
    lines.extend(_table(("Allocation", "Count"), _count_by(tags, "allocation")))
    lines.extend(["", "### Tag ranges", ""])
    lines.extend(_table(
        ("Range", "Value range", "Allocation", "Source"),
        [
            (row.get("range_id"), row.get("value_range"), row.get("allocation"), _source_reference(row))
            for row in tag_ranges
        ],
    ))
    lines.append("")

    lines.extend(["## Source clause dispositions", ""])
    lines.extend(_table(("Disposition", "Count"), _count_by(catalog.get("source_clauses", []), "disposition")))
    lines.append("")
    lines.extend([
        "## Source clause review by section",
        "",
        "Every row summarizes audited candidate locators by their pinned source section. The immutable-source audit separately requires exact candidate and clause-ledger locator equality.",
        "",
    ])
    lines.extend(_table(
        ("Source", "Section", "Candidates", "Dispositions"),
        _clause_section_rows(catalog.get("source_clauses", [])),
    ))
    lines.append("")

    unassigned = [
        row for row in requirements
        if not row.get("feature_spec") or not row.get("implementation_refs") or not row.get("verification_refs")
    ]
    unassigned.sort(key=lambda row: row.get("requirement_id", ""))
    lines.extend(["## Unassigned requirements", ""])
    lines.extend(_table(
        ("Requirement", "Strength", "Scope", "Source"),
        [(row.get("requirement_id"), row.get("normative_strength"), row.get("scope_state"), _source_reference(row)) for row in unassigned],
    ))
    lines.append("")

    requirements_without_tests = sorted(
        (row for row in requirements if not row.get("test_case_ids")),
        key=lambda row: row.get("requirement_id", ""),
    )
    lines.extend(["## Requirements without official test-case links", ""])
    lines.extend(_table(
        ("Requirement", "Evidence gap", "Source"),
        [(
            row.get("requirement_id"),
            _safe_diagnostic_text(
                row.get("review_note"),
                "No requirement-specific official Test Cases ID is linked.",
            ),
            _source_reference(row),
        ) for row in requirements_without_tests],
    ))
    lines.append("")

    negative_requirements = sorted(
        (row for row in requirements if row.get("negative_verification_required") is True),
        key=lambda row: row.get("requirement_id", ""),
    )
    lines.extend(["## Requirements needing negative verification", ""])
    lines.extend(_table(
        ("Requirement", "Strength", "Negative verification", "Status", "Source"),
        [(
            row.get("requirement_id"), row.get("normative_strength"), "required",
            row.get("status"), _source_reference(row),
        ) for row in negative_requirements],
    ))
    lines.append("")

    unassigned_elements = [
        row for row in elements
        if not row.get("feature_spec") or not row.get("implementation_refs") or not row.get("verification_refs")
    ]
    unassigned_elements.sort(key=lambda row: row.get("element_id", ""))
    lines.extend(["## Unassigned protocol elements", ""])
    lines.extend(_table(
        ("Element", "Name", "Kind", "Direction", "Scope", "Wire value", "Allocation", "Source"),
        [(
            row.get("element_id"), row.get("name"), row.get("kind"), row.get("direction"),
            row.get("scope_state"), row.get("wire_value"), row.get("allocation"), _source_reference(row),
        ) for row in unassigned_elements],
    ))
    lines.append("")

    profiles = sorted(catalog.get("profiles", []), key=lambda row: row.get("profile_id", ""))
    tests_by_id = {
        row.get("test_id"): row
        for row in catalog.get("test_cases", [])
        if row.get("test_id")
    }

    def profile_tests(profile: dict[str, Any]) -> str:
        labels = []
        for test_id in sorted(profile.get("test_case_ids") or []):
            test_case = tests_by_id.get(test_id, {})
            details = [test_case.get("official_case_id"), test_case.get("mandatory_status")]
            details = [str(value) for value in details if value]
            labels.append(f"{test_id} ({'; '.join(details)})" if details else str(test_id))
        return ", ".join(labels)

    def profile_values(profile: dict[str, Any], field: str) -> str:
        return ", ".join(sorted(str(value) for value in profile.get(field) or []))

    rule_coverage_rows, runtime_validation_availability = _profile_requirement_coverage(
        catalog, typed_rule_mappings
    )
    requirement_rule_status = {
        row[0]: (row[3], row[4]) for row in rule_coverage_rows
    }
    test_cases_by_id = {
        row.get("test_id"): row
        for row in catalog.get("test_cases", [])
        if row.get("test_id")
    }
    profile_rule_summary = []
    for profile in profiles:
        if profile.get("role") != "client":
            continue
        profile_id = profile.get("profile_id")
        requirement_ids = [
            requirement_id
            for requirement_id in profile.get("requirement_ids") or []
            if isinstance(requirement_id, str)
        ]
        if typed_rule_mappings is None:
            mapped_count: Any = "not evaluated"
            tested_count: Any = "not evaluated"
            blockers = ["typed rule registry not evaluated"]
        else:
            mapped_count = sum(
                requirement_rule_status.get(requirement_id, ("", ""))[0]
                == "Typed rule mapping: present"
                for requirement_id in requirement_ids
            )
            tested_count = sum(
                requirement_rule_status.get(requirement_id, ("", ""))[1]
                == "Category test reference: present"
                for requirement_id in requirement_ids
            )
            blockers = []
            if mapped_count < len(requirement_ids):
                blockers.append(f"typed rule mappings missing: {len(requirement_ids) - mapped_count}")
            if tested_count < len(requirement_ids):
                blockers.append(f"category test references missing: {len(requirement_ids) - tested_count}")
        unavailable_fixtures = sorted(
            test_id
            for test_id in profile.get("test_case_ids") or []
            if (test_case := test_cases_by_id.get(test_id)) is not None
            and test_case.get("mandatory_status") == "mandatory"
            and test_case.get("fixture_availability") != "available"
        )
        if unavailable_fixtures:
            blockers.append(f"mandatory fixtures unavailable: {len(unavailable_fixtures)}")
        blockers.extend(_unresolved_profile_scope_blockers(catalog, profile))
        if profile.get("applicability") == "conditional":
            blockers.append("conditional profile conditions are not evaluated")
        unavailable_dependencies = sorted(
            dependency_id
            for dependency_id in profile.get("dependency_profile_ids") or []
            if runtime_validation_availability.get(dependency_id) is not True
        )
        if unavailable_dependencies:
            blockers.append("unavailable dependencies: " + ", ".join(unavailable_dependencies))
        runtime_available = runtime_validation_availability.get(profile_id)
        profile_rule_summary.append((
            profile_id,
            len(requirement_ids),
            mapped_count,
            tested_count,
            "not evaluated" if runtime_available is None else str(runtime_available).lower(),
            ", ".join(unavailable_fixtures) or "none",
            "; ".join(blockers) or "no stable-input blockers",
        ))

    lines.extend([
        "## Profile requirement rule coverage",
        "",
        "Rows list each distinct requirement found in a client profile closure. Profile membership is shown in Profile states. Rule-test references describe checked-in evidence links; current pass/fail is not evaluated by this report. No public claim is published.",
        "",
    ])
    lines.extend(_table(
        (
            "Requirement", "Source clauses", "Source keyword / category",
            "Typed rule mapping", "Rule-test coverage", "Typed rule-test IDs",
            "Typed rule implementation refs", "Catalog implementation refs",
            "Catalog verification refs", "Official test IDs",
            "Decision/deviation ref", "OASIS source",
        ),
        rule_coverage_rows,
    ))
    lines.append("")
    lines.extend(["### Profile rule and fixture blockers", ""])
    lines.extend(_table(
        (
            "Profile", "Closure requirements", "Typed rules present", "Category test refs present",
            "Runtime validation available", "Unavailable mandatory fixture IDs", "Stable-input blockers",
        ),
        profile_rule_summary,
    ))
    lines.append("")

    lines.extend(["## Profile states", ""])
    lines.extend(_table(
        (
            "Profile", "Name", "Role", "Applicability", "Selected target",
            "Runtime validation available", "Claim state", "Source",
            "Source clauses", "Requirements", "Elements", "Dependencies",
            "Mandatory / optional tests", "Transport", "Encoding", "Server prerequisites",
        ),
        [(
            row.get("profile_id"), row.get("name"), row.get("role"), row.get("applicability"),
            _profile_target_state(row.get("profile_id"), selected_profile_ids),
            (
                "not applicable"
                if row.get("role") != "client"
                else "not evaluated"
                if runtime_validation_availability.get(row.get("profile_id")) is None
                else str(runtime_validation_availability.get(row.get("profile_id"))).lower()
            ),
            row.get("claim_state"), _source_reference(row),
            profile_values(row, "source_clause_ids"), profile_values(row, "requirement_ids"),
            profile_values(row, "element_ids"), profile_values(row, "dependency_profile_ids"),
            profile_tests(row), profile_values(row, "transport_requirements"),
            profile_values(row, "encoding_requirements"),
            profile_values(row, "server_prerequisites") or "none catalogued",
        ) for row in profiles],
    ))
    lines.append("")
    lines.extend(["### Profiles by applicability and claim state", ""])
    lines.extend(_table(("Dimension", "Value", "Count"), [
        *(("Applicability", key, value) for key, value in _count_by(profiles, "applicability")),
        *(("Claim state", key, value) for key, value in _count_by(profiles, "claim_state")),
    ]))
    lines.append("")

    test_cases = sorted(catalog.get("test_cases", []), key=lambda row: (row.get("source_id", ""), row.get("official_case_id", ""), row.get("test_id", "")))
    lines.extend(["## Test fixture availability", ""])
    lines.extend(_table(
        ("Test", "Official ID", "Fixture status", "Local fixture"),
        [(row.get("test_id"), row.get("official_case_id"), row.get("fixture_availability"), row.get("fixture_path") or "—") for row in test_cases],
    ))
    lines.append("")
    lines.extend(["### Test evidence by fixture availability", ""])
    lines.extend(_table(("Fixture state", "Count"), _count_by(test_cases, "fixture_availability")))
    lines.append("")

    lines.extend(["## Open discrepancies", ""])
    lines.extend(_table(
        ("Discrepancy", "State", "Implementation gate", "Affected records", "Summary", "Source"),
        [(
            row.get("discrepancy_id"), row.get("state"),
            "blocked for affected records"
            if row.get("state") == "open" and any(
                row.get(field) for field in (
                    "affected_requirement_ids", "affected_element_ids", "affected_profile_ids", "affected_policy_ids",
                )
            )
            else "review before dependent implementation",
            ", ".join(
                f"{len(row.get(field) or [])} {label}"
                for field, label in (
                    ("affected_requirement_ids", "requirements"),
                    ("affected_element_ids", "elements"),
                    ("affected_profile_ids", "profiles"),
                    ("affected_policy_ids", "policies"),
                )
                if row.get(field)
            ) or "none linked",
            _safe_diagnostic_text(
                row.get("summary"), "Open source discrepancy requires review."
            ),
            _source_reference(row),
        ) for row in open_discrepancies],
    ))
    lines.append("")

    policies = sorted(catalog.get("policies", []), key=lambda row: row.get("policy_id", ""))
    lines.extend(["## Project policies", ""])
    lines.extend(_table(
        ("Policy", "Provenance", "Summary"),
        [(row.get("policy_id"), row.get("provenance"), row.get("summary")) for row in policies],
    ))
    lines.append("")

    return "\n".join(lines)


def write_report(
    catalog: dict[str, Any],
    path: Path,
    *,
    check: bool,
    repo_root: Path | None = None,
    selected_profile_ids: set[str] | None = None,
    typed_rule_mappings: list[dict[str, Any]] | None = None,
) -> bool:
    """Check byte equality or write the deterministic UTF-8/LF report."""
    expected = render_report(
        catalog,
        selected_profile_ids=selected_profile_ids,
        typed_rule_mappings=typed_rule_mappings,
    ).encode("utf-8")
    root = (repo_root or path.parent).resolve(strict=True)
    try:
        if repo_root is None:
            relative = path.name
        else:
            relative = path.relative_to(root).as_posix()
    except ValueError as error:
        raise PathSecurityError("coverage report path is outside the repository") from error
    target = confined_path(root, relative, allow_missing_leaf=True)
    try:
        current = safe_read_bytes(root, relative, max_bytes=16 * 1024 * 1024) if target.exists() else None
    except PathSecurityError:
        raise
    except OSError as error:
        raise PathSecurityError("coverage report could not be read safely") from error
    if check:
        return current == expected
    atomic_write_bytes(root, relative, expected)
    return True


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate verification-manifest JSON key")
        result[key] = value
    return result


def render_readiness_summary(readiness: Mapping[str, Any]) -> str:
    """Render only stable readiness blockers; never echo the ephemeral manifest."""
    target_selected = readiness.get("target_selected")
    selected_text = "yes" if target_selected is True else "no" if target_selected is False else "unknown"
    runtime_available = "yes" if readiness.get("runtime_validation_available") is True else "no"
    lines = [
        "### Current profile readiness (ephemeral)",
        "",
        f"- Profile: `{_escape_table_text(readiness.get('profile_id', 'unknown'))}`",
        f"- Target selected: {selected_text}",
        f"- Runtime validation available: {runtime_available}",
        f"- Effective evidence state: `{_escape_table_text(readiness.get('effective_state', 'unknown'))}`",
        "- Public claim: no",
        "",
    ]
    blockers = readiness.get("blockers")
    if isinstance(blockers, list) and blockers:
        lines.extend(("Blockers:", ""))
        lines.extend(f"- {_escape_table_text(blocker)}" for blocker in blockers)
        lines.append("")
    else:
        lines.extend(("No readiness blockers were reported.", ""))
    return "\n".join(lines)


def evaluate_current_readiness(
    repository: Path,
    environment: Mapping[str, str],
) -> dict[str, Any]:
    """Evaluate Baseline against current workflow context and an optional same-run manifest."""
    code_root = Path(__file__).resolve().parents[2]
    if str(code_root) not in sys.path:
        sys.path.insert(0, str(code_root))
    from tools.normative_catalog.validate import load_validated_catalog
    from tools.normative_catalog.generate_profiles import load_profile_targets

    raw = safe_read_bytes(repository, "specification/catalog/kmip-2.1.json", max_bytes=16 * 1024 * 1024)
    catalog = load_validated_catalog(raw, repository, require_complete=True)
    selected_profile_ids = set(load_profile_targets(repository, catalog).profile_ids)
    typed_rule_mappings = load_typed_rule_mappings(repository)
    manifest_text = environment.get("PROFILE_EVIDENCE_MANIFEST", "")
    manifest: dict[str, Any] | None = None
    if manifest_text:
        try:
            if len(manifest_text.encode("utf-8")) > 64 * 1024:
                raise ValueError("verification manifest exceeds the input limit")
            parsed = json.loads(manifest_text, object_pairs_hook=_unique_json_object)
            if not isinstance(parsed, dict):
                raise ValueError("verification manifest must be an object")
            manifest = parsed
        except (UnicodeError, ValueError):
            # An invalid marker is passed to the strict validator below so the
            # readiness result is explicitly invalid, never silently absent.
            manifest = {"invalid_manifest_input": True}

    context = {
        "event_name": environment.get("GITHUB_EVENT_NAME", ""),
        "repository": environment.get("GITHUB_REPOSITORY", ""),
        "ref": environment.get("GITHUB_REF", ""),
        "ref_protected": environment.get("GITHUB_REF_PROTECTED", "") == "true",
        "sha": environment.get("GITHUB_SHA", ""),
        "workflow_ref": environment.get("GITHUB_WORKFLOW_REF", ""),
        "run_id": environment.get("GITHUB_RUN_ID", ""),
        "run_attempt": environment.get("GITHUB_RUN_ATTEMPT", ""),
    }
    head = subprocess.run(
        ["git", "rev-parse", "--verify", "HEAD"],
        cwd=repository,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    return evaluate_profile_readiness(
        catalog,
        BASELINE_PROFILE_ID,
        selected_profile_ids=selected_profile_ids,
        typed_rule_mappings=typed_rule_mappings,
        verification_manifest=manifest,
        current_context=context,
        current_head=head,
        repository_root=repository,
    )


def _append_readiness_summary(summary_path: str | None, readiness: Mapping[str, Any]) -> None:
    markdown = render_readiness_summary(readiness)
    if summary_path:
        with Path(summary_path).open("a", encoding="utf-8", newline="\n") as summary:
            summary.write(markdown)
    else:
        sys.stdout.write(markdown)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the generated report")
    mode.add_argument("--check", action="store_true", help="fail if the checked-in report is stale")
    mode.add_argument("--readiness", action="store_true", help="evaluate current profile evidence ephemerally")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument(
        "--structural-only",
        action="store_true",
        help="render an incomplete catalog for authoring and fixture checks",
    )
    arguments = parser.parse_args(argv)
    try:
        repository = arguments.repo_root.resolve(strict=True)
        code_root = Path(__file__).resolve().parents[2]
        if str(code_root) not in sys.path:
            sys.path.insert(0, str(code_root))
        from tools.normative_catalog.validate import load_validated_catalog

        report_path = repository / REPORT_PATH
        if arguments.readiness:
            readiness = evaluate_current_readiness(repository, os.environ)
            _append_readiness_summary(os.environ.get("GITHUB_STEP_SUMMARY"), readiness)
            return 0

        raw = safe_read_bytes(repository, "specification/catalog/kmip-2.1.json", max_bytes=16 * 1024 * 1024)
        catalog = load_validated_catalog(raw, repository, require_complete=not arguments.structural_only)
        selected_profile_ids = None
        typed_rule_mappings = None
        if not arguments.structural_only:
            from tools.normative_catalog.generate_profiles import load_profile_targets

            selected_profile_ids = set(load_profile_targets(repository, catalog).profile_ids)
            typed_rule_mappings = load_typed_rule_mappings(repository)
        valid = write_report(
            catalog,
            report_path,
            check=arguments.check,
            repo_root=repository,
            selected_profile_ids=selected_profile_ids,
            typed_rule_mappings=typed_rule_mappings,
        )
    except (OSError, ValueError, PathSecurityError, subprocess.CalledProcessError):
        failure = "profile readiness evaluation failed: current evidence or repository input is invalid" if arguments.readiness else "coverage report failed: catalog or report input is invalid or unsafe"
        print(failure, file=sys.stderr)
        return 1
    if not valid:
        print("coverage report is stale; run report.py --write", file=sys.stderr)
        return 1
    print(f"Coverage report {'verified' if arguments.check else 'written'}: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
