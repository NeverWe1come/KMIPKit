"""Tests for feature ownership of implemented protocol elements."""

from __future__ import annotations

import ast
import csv
import json
import re
import tempfile
import unittest
from collections.abc import Mapping
from pathlib import Path, PurePosixPath, PureWindowsPath

from tools.normative_catalog.rust_source_analysis import (
    _rust_item_is_top_level,
    _rust_test_item_has_active_module_context,
)


ROOT = Path(__file__).resolve().parents[3]
MAX_TEST_SOURCE_BYTES = 1_048_576
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FEATURE_SPEC = "KMIPKIT-0003"
PROFILE_CONFORMANCE_FEATURE_SPEC = "KMIPKIT-0010"
PROFILE_CONFORMANCE_GATE_REQUIREMENT_IDS = {
    "KMIPKIT-REQ-SPEC-14.1-001",
    "KMIPKIT-REQ-SPEC-14.1-002",
}
MESSAGE_MODEL_FEATURE_SPEC = "KMIPKIT-0006"
MESSAGE_MODEL_REQUIREMENTS_PATH = ROOT / "specification/compliance/requirements/KMIPKIT-0006.csv"
CLIENT_EXECUTION_FEATURE_SPEC = "KMIPKIT-0007"
CLIENT_EXECUTION_SPEC_PATH = ROOT / "specs/007-client-execution/spec.md"
CLIENT_EXECUTION_REQUIREMENTS_PATH = (
    ROOT / "specification/compliance/requirements/KMIPKIT-0007.csv"
)
CLIENT_CREDENTIALS_REQUIREMENTS_PATH = (
    ROOT / "specification/compliance/requirements/KMIPKIT-0008.csv"
)
CLIENT_CREDENTIALS_SPEC_PATH = ROOT / "specs/008-credentials-attestation/spec.md"
CLIENT_EXECUTION_OWNED_REQUIREMENT_IDS = {
    "KMIPKIT-REQ-SPEC-8-003-001",
    "KMIPKIT-REQ-SPEC-9.12-001-002",
    "KMIPKIT-REQ-SPEC-9.12-001-003",
    "KMIPKIT-REQ-SPEC-9.13-001-004",
    "KMIPKIT-REQ-SPEC-9.13-001-005",
    "KMIPKIT-REQ-SPEC-9.20-001-002",
    "KMIPKIT-REQ-SPEC-9.21-001-002",
    "KMIPKIT-REQ-SPEC-9.6-001-002",
}
LIFECYCLE_TRACEABILITY_PATH = ROOT / "specs/018-managed-object-lifecycle/traceability.md"
LIFECYCLE_SPEC_PATH = ROOT / "specs/018-managed-object-lifecycle/spec.md"
LIFECYCLE_TASKS_PATH = ROOT / "specs/018-managed-object-lifecycle/tasks.md"
CLIENT_EXECUTION_DEFERRED_REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-9.20-001-002"
CLIENT_EXECUTION_DEFERRED_PROFILE_IMPLEMENTATION_REFS = {
    "crates/kmipkit-protocol/src/profile.rs",
    "crates/kmipkit-protocol/src/message/header.rs",
    "crates/kmipkit-ttlv/src/value.rs",
}
CLIENT_EXECUTION_DEFERRED_PROFILE_VERIFICATION_REFS = {
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_countdown_relative_date_use_as_caller_owned_optional_capability",
    "crates/kmipkit-protocol/tests/message_accessors.rs::response_header_accessors_expose_every_represented_field",
}
RESULT_ELEMENT_IDS = {
    "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON",
    "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
    "KMIPKIT-ELEM-RESULT-RESULT-MESSAGE",
    "KMIPKIT-ELEM-RESULT-RESULT-REASON",
    "KMIPKIT-ELEM-RESULT-RESULT-STATUS",
}
LIFECYCLE_OPERATION_TABLES = {
    "KMIPKIT-ELEM-OP-C2S-ACTIVATE": ("6.1.1", "164–166"),
    "KMIPKIT-ELEM-OP-C2S-ARCHIVE": ("6.1.4", "173–175"),
    "KMIPKIT-ELEM-OP-C2S-DESTROY": ("6.1.15", "208–210"),
    "KMIPKIT-ELEM-OP-C2S-RECOVER": ("6.1.42", "288–290"),
}
LIFECYCLE_UNIQUE_IDENTIFIER_ENCODING_CLAIM = (
    "§4.58 Tables 145–146 define permitted Unique Identifier encodings"
)
LIFECYCLE_UNIQUE_IDENTIFIER_TAG_CLAIM = (
    "§11.56 Table 487 assigns tag 0x420094 to Unique Identifier"
)
LIFECYCLE_UNIQUE_IDENTIFIER_REFERENCES = {
    "crates/kmipkit-protocol/src/activate.rs",
    "crates/kmipkit-protocol/src/destroy.rs",
    "crates/kmipkit-protocol/tests/unit/activate_operation_tests.rs",
    "crates/kmipkit-protocol/tests/unit/destroy_operation_tests.rs",
    "crates/kmipkit-client/tests/unit/activate_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/destroy_execution_tests.rs",
}
LIFECYCLE_CLIENT_REQUIREMENT_IDS = {
    "KMIPKIT-REQ-SPEC-6.1.4-001",
    "KMIPKIT-REQ-SPEC-6.1.42-001-001",
    "KMIPKIT-REQ-SPEC-6.1.42-001-002",
}
LIFECYCLE_SERVER_ONLY_CLAUSE_IDS = {
    "KMIPKIT-CLAUSE-SPEC-6.1.1-001",
    "KMIPKIT-CLAUSE-SPEC-6.1.15-001",
}
LIFECYCLE_TEST_MODULES_BY_ROW = {
    "KMIPKIT-ELEM-OP-C2S-ACTIVATE": (
        "crates/kmipkit-protocol/tests/unit/activate_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/activate_execution_tests.rs",
    ),
    "KMIPKIT-ELEM-OP-C2S-ARCHIVE": (
        "crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/archive_execution_tests.rs",
    ),
    "KMIPKIT-ELEM-OP-C2S-DESTROY": (
        "crates/kmipkit-protocol/tests/unit/destroy_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/destroy_execution_tests.rs",
    ),
    "KMIPKIT-ELEM-OP-C2S-RECOVER": (
        "crates/kmipkit-protocol/tests/unit/recover_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/recover_execution_tests.rs",
    ),
    "KMIPKIT-CLAUSE-SPEC-6.1.1-001": (
        "crates/kmipkit-client/tests/unit/activate_execution_tests.rs",
    ),
    "KMIPKIT-REQ-SPEC-6.1.4-001": (
        "crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/archive_execution_tests.rs",
    ),
    "KMIPKIT-CLAUSE-SPEC-6.1.15-001": (
        "crates/kmipkit-client/tests/unit/destroy_execution_tests.rs",
    ),
    "KMIPKIT-REQ-SPEC-6.1.42-001-001": (
        "crates/kmipkit-protocol/tests/unit/recover_operation_tests.rs",
        "crates/kmipkit-client/tests/unit/recover_execution_tests.rs",
    ),
    "KMIPKIT-REQ-SPEC-6.1.42-001-002": (
        "crates/kmipkit-client/tests/unit/recover_execution_tests.rs",
    ),
}
LIFECYCLE_SUPPORT_TEST_MODULES = {
    "crates/kmipkit-client/tests/unit/lifecycle_redaction_tests.rs",
    "crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs",
}
LIFECYCLE_OPERATION_EXECUTION_MODULES = {
    "crates/kmipkit-client/tests/unit/activate_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/archive_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/destroy_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/recover_execution_tests.rs",
}
LIFECYCLE_PLANNED_CODE_PATHS = {
    "crates/kmipkit-protocol/src/activate.rs",
    "crates/kmipkit-protocol/src/archive.rs",
    "crates/kmipkit-protocol/src/destroy.rs",
    "crates/kmipkit-protocol/src/recover.rs",
    "crates/kmipkit-protocol/src/lib.rs",
    "crates/kmipkit-client/src/execute.rs",
    "crates/kmipkit-client/src/lib.rs",
}


def _result_element_ids(elements: list[dict[str, object]]) -> set[str]:
    result_enumerations = {
        "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON",
        "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
    }
    return RESULT_ELEMENT_IDS | {
        element["element_id"]
        for element in elements
        if element.get("kind") == "enumeration_value"
        and result_enumerations.intersection(element.get("parent_element_ids", []))
        and element.get("allocation") in {"assigned", "extension"}
    }


def _requirements_by_id() -> dict[str, dict[str, str]]:
    path = ROOT / "specification/compliance/requirements/KMIPKIT-0003.csv"
    with path.open(encoding="utf-8", newline="") as stream:
        return {row["requirement_id"]: row for row in csv.DictReader(stream)}


def _read_confined_source(
    source_path: str,
    allowed_suffixes: frozenset[str],
) -> tuple[Path, str] | None:
    posix_path = PurePosixPath(source_path)
    windows_path = PureWindowsPath(source_path)
    if (
        not source_path
        or "\\" in source_path
        or posix_path.is_absolute()
        or windows_path.is_absolute()
        or windows_path.drive
        or ".." in posix_path.parts
    ):
        return None

    try:
        root = ROOT.resolve(strict=True)
        path = (root / source_path).resolve(strict=True)
        path.relative_to(root)
        if not path.is_file() or path.suffix not in allowed_suffixes:
            return None
        with path.open("rb") as source_file:
            source_bytes = source_file.read(MAX_TEST_SOURCE_BYTES + 1)
        if len(source_bytes) > MAX_TEST_SOURCE_BYTES:
            return None
        source = source_bytes.decode("utf-8")
    except (OSError, RuntimeError, ValueError, UnicodeDecodeError):
        return None
    return path, source


def _read_confined_test_source(test_path: str) -> tuple[Path, str] | None:
    return _read_confined_source(test_path, frozenset({".py", ".rs"}))


def _read_confined_implementation_source(source_path: str) -> tuple[Path, str] | None:
    return _read_confined_source(source_path, frozenset({".md", ".py", ".rs"}))


def _blank_rust_source_region(source: str, start: int, end: int) -> str:
    """Blank a Rust comment or literal while retaining its original line layout."""
    return "".join(
        "\n" if char == "\n" else "\r" if char == "\r" else " "
        for char in source[start:end]
    )


def _rust_raw_string_start(source: str, index: int) -> tuple[int, int] | None:
    """Return the quote index and hash count for a raw string starting here."""
    raw_prefix = index
    if source.startswith(("br", "cr"), index):
        raw_prefix += 1
    elif index >= len(source) or source[index] != "r":
        return None

    if raw_prefix >= len(source) or source[raw_prefix] != "r":
        return None
    quote_index = raw_prefix + 1
    while quote_index < len(source) and source[quote_index] == "#":
        quote_index += 1
    if quote_index < len(source) and source[quote_index] == '"':
        return quote_index, quote_index - raw_prefix - 1
    return None


def _strip_rust_comments_and_literals(source: str) -> str | None:
    """Remove Rust comments/literals lexically so their text cannot mimic test items."""
    characters = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            if end < 0:
                end = len(source)
            characters[index:end] = _blank_rust_source_region(source, index, end)
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
            characters[index:end] = _blank_rust_source_region(source, index, end)
            index = end
            continue

        raw_start = _rust_raw_string_start(source, index)
        if raw_start is not None:
            quote_index, hash_count = raw_start
            terminator = '"' + ("#" * hash_count)
            end_quote = source.find(terminator, quote_index + 1)
            if end_quote < 0:
                return None
            end = end_quote + len(terminator)
            characters[index:end] = _blank_rust_source_region(source, index, end)
            index = end
            continue

        quote_index = index
        if source.startswith(("b\"", "c\""), index):
            quote_index += 1
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
            characters[index:end] = _blank_rust_source_region(source, index, end)
            index = end
            continue

        index += 1
    return "".join(characters)


def _rust_cfg_condition_is_enabled(expression: str) -> bool | None:
    """Evaluate cfg logic known in a Rust test build; None means unknown."""
    tokens = re.findall(
        r'"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z0-9_]*|[(),=]|\S', expression
    )
    cursor = 0

    def parse_predicate() -> bool | None:
        nonlocal cursor
        if cursor >= len(tokens) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", tokens[cursor]):
            return None
        name = tokens[cursor]
        cursor += 1
        if name not in {"any", "all", "not"}:
            if cursor < len(tokens) and tokens[cursor] == "=":
                cursor += 1
                if cursor < len(tokens) and tokens[cursor] not in {",", ")"}:
                    cursor += 1
                return None
            return True if name == "test" else None
        if cursor >= len(tokens) or tokens[cursor] != "(":
            return None
        cursor += 1
        values: list[bool | None] = []
        if cursor < len(tokens) and tokens[cursor] != ")":
            while True:
                values.append(parse_predicate())
                if cursor >= len(tokens):
                    return None
                if tokens[cursor] == ")":
                    break
                if tokens[cursor] != ",":
                    return None
                cursor += 1
                if cursor < len(tokens) and tokens[cursor] == ")":
                    break
        if cursor >= len(tokens) or tokens[cursor] != ")":
            return None
        cursor += 1
        if name == "not":
            if len(values) != 1 or values[0] is None:
                return None
            return not values[0]
        if name == "all":
            if any(value is False for value in values):
                return False
            if all(value is True for value in values):
                return True
            return None
        if any(value is True for value in values):
            return True
        if all(value is False for value in values):
            return False
        return None

    result = parse_predicate()
    return result if cursor == len(tokens) else None


def _rust_source_has_only_unconditionally_enabled_cfg(source: str) -> bool:
    """Fail closed on unresolved crate cfg; target-item cfg is checked separately."""
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
        if depth or match.group("attribute") is not None:
            return False
        expression = source[match.end():end - 1]
        if _rust_cfg_condition_is_enabled(expression) is not True:
            return False
        cursor = end
    return True


def _python_ast_dotted_name(node: ast.AST) -> str | None:
    if isinstance(node, ast.Call):
        return _python_ast_dotted_name(node.func)
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        prefix = _python_ast_dotted_name(node.value)
        return f"{prefix}.{node.attr}" if prefix else node.attr
    return None


def _python_skip_decorator_aliases(module: ast.Module) -> set[str]:
    skip_decorators = {"skip", "skipif", "skipunless"}
    aliases: set[str] = set()
    for node in module.body:
        if not isinstance(node, ast.ImportFrom) or node.module not in {"unittest", "unittest.case"}:
            continue
        for imported in node.names:
            if imported.name.casefold() in skip_decorators:
                aliases.add(imported.asname or imported.name)
    return aliases


def _python_callable_is_skip_marker(node: ast.AST, aliases: set[str]) -> bool:
    name = _python_ast_dotted_name(node)
    if name is None:
        return False
    return name in aliases or name.rsplit(".", 1)[-1].casefold() in {
        "skip",
        "skipif",
        "skipunless",
    }


def _python_expression_has_skip_marker(
    node: ast.AST, aliases: set[str], *, allow_alias_values: bool = False
) -> bool:
    if isinstance(node, ast.Name) and _python_callable_is_skip_marker(node, aliases):
        return True
    for child in ast.walk(node):
        if isinstance(child, ast.Attribute) and child.attr.casefold() in {
            "skip",
            "skipif",
            "skipunless",
        }:
            return True
        if isinstance(child, ast.Call) and _python_callable_is_skip_marker(
            child.func, aliases
        ):
            return True
        if (
            allow_alias_values
            and isinstance(child, ast.Name)
            and child.id in aliases
        ):
            return True
    return False


def _python_node_has_skip_decorator(node: ast.AST, aliases: set[str]) -> bool:
    return any(
        _python_decorator_is_skip_marker(decorator, aliases)
        for decorator in getattr(node, "decorator_list", [])
    )


def _python_decorator_is_skip_marker(node: ast.AST, aliases: set[str]) -> bool:
    callable_node = node.func if isinstance(node, ast.Call) else node
    return _python_callable_is_skip_marker(callable_node, aliases)


def _python_module_has_skip_marker(module: ast.Module, aliases: set[str]) -> bool:
    for node in module.body:
        if isinstance(node, ast.Assign):
            pytestmark_targets = [
                target
                for target in node.targets
                if isinstance(target, ast.Name) and target.id == "pytestmark"
            ]
            if pytestmark_targets and isinstance(node.value, ast.Name):
                return True
            if pytestmark_targets and _python_expression_has_skip_marker(
                node.value, aliases, allow_alias_values=True
            ):
                return True
            alias_targets = [
                target
                for target in node.targets
                if isinstance(target, ast.Name) and target.id != "pytestmark"
            ]
            if _python_expression_has_skip_marker(
                node.value, aliases, allow_alias_values=True
            ):
                aliases.update(target.id for target in alias_targets)
            elif pytestmark_targets and _python_module_scope_uses_pytestmark(node.value):
                return True
            elif not pytestmark_targets and _python_module_scope_uses_pytestmark(node):
                return True
            continue
        if isinstance(node, ast.AnnAssign):
            if isinstance(node.target, ast.Name) and node.target.id == "pytestmark":
                if node.value is not None and _python_expression_has_skip_marker(
                    node.value, aliases, allow_alias_values=True
                ):
                    return True
                if node.value is not None and _python_module_scope_uses_pytestmark(node.value):
                    return True
                continue
            if node.value is not None and _python_expression_has_skip_marker(
                node.value, aliases, allow_alias_values=True
            ) and isinstance(node.target, ast.Name):
                aliases.add(node.target.id)
        if _python_module_scope_uses_pytestmark(node):
            return True
    return False


def _python_module_scope_uses_pytestmark(node: ast.AST) -> bool:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
        return False
    if isinstance(node, ast.Name) and node.id == "pytestmark":
        return True
    return any(
        _python_module_scope_uses_pytestmark(child)
        for child in ast.iter_child_nodes(node)
    )


def _is_discoverable_positive_test(path: Path, test_name: str, source: str) -> bool:
    """Recognize runner-shaped Rust/Python tests, not their assertion semantics."""
    if path.suffix == ".rs":
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", test_name):
            return False
        source = _strip_rust_comments_and_literals(source)
        if source is None or not _rust_source_has_only_unconditionally_enabled_cfg(source):
            return False
        declaration = re.compile(
            rf"^\s*(?:async\s+)?fn\s+{re.escape(test_name)}\s*\(", re.MULTILINE
        )
        test_attribute = re.compile(
            r"^\s*#\[(?:test|tokio::test)(?:\s*\([^]]*\))?\]\s*$"
        )
        ignored_test_attribute = re.compile(r"^\s*#\[\s*ignore(?:\s*=.*)?\s*\]\s*$")
        cfg_attribute = re.compile(r"^\s*#\[\s*cfg\((?P<condition>.*)\)\s*\]\s*$")
        cfg_attr_marker = re.compile(r"^\s*#\[\s*cfg_attr\s*\(")
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
                if preceding_line.startswith("#["):
                    if ignored_test_attribute.fullmatch(preceding_line):
                        return False
                    if test_attribute.fullmatch(preceding_line):
                        is_test = True
                    elif cfg_attr_marker.match(preceding_line):
                        return False
                    elif cfg_match := cfg_attribute.fullmatch(preceding_line):
                        if _rust_cfg_condition_is_enabled(cfg_match.group("condition")) is not True:
                            return False
                    cursor -= 1
                    continue
                break
            if is_test:
                return True
        return False

    if path.suffix == ".py":
        try:
            module = ast.parse(source)
        except SyntaxError:
            return False
        skip_aliases = _python_skip_decorator_aliases(module)
        if _python_module_has_skip_marker(module, skip_aliases):
            return False
        class_name, separator, function_name = test_name.rpartition(".")
        if separator:
            if not function_name.startswith("test_"):
                return False
            for node in module.body:
                if not isinstance(node, ast.ClassDef) or node.name != class_name:
                    continue
                is_pytest_class = class_name.startswith("Test")
                is_unittest_class = any(
                    isinstance(base, ast.Name) and base.id == "TestCase"
                    or isinstance(base, ast.Attribute) and base.attr == "TestCase"
                    for base in node.bases
                )
                if not (is_pytest_class or is_unittest_class) or _python_node_has_skip_decorator(
                    node, skip_aliases
                ):
                    return False
                return any(
                    isinstance(method, (ast.FunctionDef, ast.AsyncFunctionDef))
                    and method.name == function_name
                    and not _python_node_has_skip_decorator(method, skip_aliases)
                    for method in node.body
                )
            return False
        return test_name.startswith("test_") and any(
            isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == test_name
            and not _python_node_has_skip_decorator(node, skip_aliases)
            for node in module.body
        )

    return False


def _markdown_table_rows(document: str) -> list[list[str]]:
    return [
        [cell.strip() for cell in line.strip().strip("|").split("|")]
        for line in document.splitlines()
        if line.startswith("| ")
    ]


def _markdown_catalog_rows(document: str) -> dict[str, list[str]]:
    rows_by_id: dict[str, list[str]] = {}
    for cells in _markdown_table_rows(document):
        if not cells[0].startswith("`"):
            continue
        match = re.search(r"`([^`]+)`", cells[0])
        if match:
            rows_by_id[match.group(1)] = cells
    return rows_by_id


_ACCEPTED_DEVIATION_CATEGORIES = {
    "RecommendedBehaviorOrDeviation",
    "DiscouragedAbsenceOrDeviation",
}


def _is_accepted_recommendation_deviation(
    requirement_id: str,
    requirement: Mapping[str, object],
    decisions_by_id: Mapping[str, Mapping[str, object]],
    category: str,
) -> bool:
    decision_id = requirement.get("decision_id")
    decision = (
        decisions_by_id.get(decision_id)
        if isinstance(decision_id, str)
        else None
    )
    decision_requirement_ids = (
        decision.get("requirement_ids") if decision is not None else None
    )
    return (
        requirement.get("status") == "deviated"
        and category in _ACCEPTED_DEVIATION_CATEGORIES
        and decision is not None
        and decision.get("status") == "accepted"
        and isinstance(decision_requirement_ids, list)
        and all(isinstance(value, str) for value in decision_requirement_ids)
        and requirement_id in decision_requirement_ids
    )


def _missing_category_tests_without_accepted_deviation(
    missing_category_test_ids: set[str],
    requirements_by_id: Mapping[str, Mapping[str, object]],
    decisions_by_id: Mapping[str, Mapping[str, object]],
    verification_categories: Mapping[str, str],
) -> set[str]:
    return {
        requirement_id
        for requirement_id in missing_category_test_ids
        if not _is_accepted_recommendation_deviation(
            requirement_id,
            requirements_by_id.get(requirement_id, {}),
            decisions_by_id,
            verification_categories.get(requirement_id, ""),
        )
    }


class FeatureTraceabilityTests(unittest.TestCase):
    def test_accepted_recommendation_deviation_is_exempt_from_category_test(self) -> None:
        requirement_id = "KMIPKIT-REQ-EXAMPLE-001"
        result = _missing_category_tests_without_accepted_deviation(
            {requirement_id},
            {
                requirement_id: {
                    "status": "deviated",
                    "decision_id": "KMIPKIT-DEC-EXAMPLE-001",
                }
            },
            {
                "KMIPKIT-DEC-EXAMPLE-001": {
                    "status": "accepted",
                    "requirement_ids": [requirement_id],
                }
            },
            {requirement_id: "RecommendedBehaviorOrDeviation"},
        )
        self.assertEqual(result, set())

    def test_missing_category_test_requires_accepted_recommendation_deviation(self) -> None:
        requirement_id = "KMIPKIT-REQ-EXAMPLE-001"
        self.assertEqual(
            _missing_category_tests_without_accepted_deviation(
                {requirement_id},
                {requirement_id: {"status": "active"}},
                {},
                {requirement_id: "RequiredBehavior"},
            ),
            {requirement_id},
        )

        deviations_without_valid_exception = (
            (
                {"status": "deviated", "decision_id": "KMIPKIT-DEC-EXAMPLE-001"},
                {"status": "proposed", "requirement_ids": [requirement_id]},
                "RecommendedBehaviorOrDeviation",
            ),
            (
                {"status": "deviated", "decision_id": "KMIPKIT-DEC-EXAMPLE-001"},
                {"status": "accepted", "requirement_ids": ["KMIPKIT-REQ-OTHER-001"]},
                "RecommendedBehaviorOrDeviation",
            ),
            (
                {"status": "deviated", "decision_id": "KMIPKIT-DEC-EXAMPLE-001"},
                {"status": "accepted", "requirement_ids": [requirement_id]},
                "RequiredBehavior",
            ),
        )
        for requirement, decision, category in deviations_without_valid_exception:
            with self.subTest(requirement=requirement, decision=decision, category=category):
                self.assertEqual(
                    _missing_category_tests_without_accepted_deviation(
                        {requirement_id},
                        {requirement_id: requirement},
                        {"KMIPKIT-DEC-EXAMPLE-001": decision},
                        {requirement_id: category},
                    ),
                    {requirement_id},
                )

    def test_join_split_key_threshold_uses_exact_unique_identifier_sources(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirement = next(
            row
            for row in catalog["requirements"]
            if row["requirement_id"] == "KMIPKIT-REQ-SPEC-6.1.27-001"
        )

        source_sections = {
            reference["section"] for reference in requirement["source_refs"]
        }
        self.assertTrue({"4.58", "6.1.27", "11.58"}.issubset(source_sections))
        self.assertIn("§4.58 Table 145", requirement["review_note"])
        self.assertIn("§11.58 Table 489", requirement["review_note"])
        self.assertNotIn("§11.58 Table 145", requirement["review_note"])

    def test_confined_implementation_reader_accepts_docs_but_test_reader_does_not(self) -> None:
        documentation_path = "docs/user-guide/en/object-retrieval.md"
        implementation = _read_confined_implementation_source(documentation_path)
        self.assertIsNotNone(implementation)
        self.assertIn("Key Format Type", implementation[1] if implementation else "")
        self.assertIsNone(_read_confined_test_source(documentation_path))
        self.assertIsNone(_read_confined_implementation_source("../AGENTS.md"))

    def test_lifecycle_traceability_covers_catalog_sources_and_planned_modules(self) -> None:
        traceability = LIFECYCLE_TRACEABILITY_PATH.read_text(encoding="utf-8")
        specification = LIFECYCLE_SPEC_PATH.read_text(encoding="utf-8")
        tasks = LIFECYCLE_TASKS_PATH.read_text(encoding="utf-8")
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))

        rows_by_id = _markdown_catalog_rows(traceability)
        self.assertEqual(
            set(rows_by_id),
            set(LIFECYCLE_OPERATION_TABLES)
            | LIFECYCLE_CLIENT_REQUIREMENT_IDS
            | LIFECYCLE_SERVER_ONLY_CLAUSE_IDS,
        )

        elements_by_id = {item["element_id"]: item for item in catalog["elements"]}
        requirements_by_id = {
            item["requirement_id"]: item for item in catalog["requirements"]
        }
        clauses_by_id = {item["clause_id"]: item for item in catalog["source_clauses"]}
        for element_id, (section, tables) in LIFECYCLE_OPERATION_TABLES.items():
            with self.subTest(element_id=element_id):
                element = elements_by_id[element_id]
                self.assertEqual(element["feature_spec"], "KMIPKIT-0018")
                expected_source = f"KMIP 2.1 §{section} Tables {tables}"
                source = rows_by_id[element_id][1]
                self.assertTrue(source.startswith(expected_source))
                self.assertIn(f"§{section} | Tables {tables}", specification)

                payload_tables = element["payload_tables"]
                self.assertEqual(
                    [table["role"] for table in payload_tables], ["request", "response"]
                )
                payload_numbers = tuple(
                    table["table_number"] for table in payload_tables
                )
                payload_range = re.search(
                    r"request/response payloads: Tables (\d+)–(\d+)", source
                )
                self.assertIsNotNone(payload_range)
                self.assertEqual(
                    tuple(map(int, payload_range.groups())), payload_numbers
                )

        normalized_specification = " ".join(specification.split())
        self.assertIn(LIFECYCLE_UNIQUE_IDENTIFIER_ENCODING_CLAIM, normalized_specification)
        self.assertIn(LIFECYCLE_UNIQUE_IDENTIFIER_TAG_CLAIM, normalized_specification)
        for element_id in LIFECYCLE_OPERATION_TABLES:
            with self.subTest(identifier_source_row=element_id):
                source = rows_by_id[element_id][1]
                normalized_source = " ".join(source.split())
                self.assertIn(LIFECYCLE_UNIQUE_IDENTIFIER_ENCODING_CLAIM, normalized_source)
                self.assertIn(LIFECYCLE_UNIQUE_IDENTIFIER_TAG_CLAIM, normalized_source)
        for source_path in LIFECYCLE_UNIQUE_IDENTIFIER_REFERENCES:
            with self.subTest(identifier_source_reference=source_path):
                _, source = _read_confined_test_source(source_path) or (None, "")
                comment_text = re.sub(r"(?m)^\s*//[/!] ?", "", source)
                normalized_comment = " ".join(comment_text.split())
                self.assertIn(
                    LIFECYCLE_UNIQUE_IDENTIFIER_ENCODING_CLAIM,
                    normalized_comment,
                )
                self.assertIn(
                    LIFECYCLE_UNIQUE_IDENTIFIER_TAG_CLAIM,
                    normalized_comment,
                )
        tag_source_row = _markdown_table_rows(traceability)
        generic_row = next(
            cells for cells in tag_source_row if cells[0] == "Generic TTLV and secret handling"
        )
        self.assertIn(LIFECYCLE_UNIQUE_IDENTIFIER_TAG_CLAIM, generic_row[1])

        for requirement_id in LIFECYCLE_CLIENT_REQUIREMENT_IDS:
            with self.subTest(requirement_id=requirement_id):
                self.assertEqual(
                    requirements_by_id[requirement_id]["feature_spec"], "KMIPKIT-0018"
                )
        for clause_id in LIFECYCLE_SERVER_ONLY_CLAUSE_IDS:
            with self.subTest(clause_id=clause_id):
                self.assertEqual(clauses_by_id[clause_id]["scope_state"], "server_only")

        all_planned_test_modules = set().union(
            *LIFECYCLE_TEST_MODULES_BY_ROW.values(), LIFECYCLE_SUPPORT_TEST_MODULES
        )
        for row_id, test_modules in LIFECYCLE_TEST_MODULES_BY_ROW.items():
            with self.subTest(row_id=row_id):
                verification = rows_by_id[row_id][4]
                for module in test_modules:
                    self.assertIn(module, verification)
        for module in all_planned_test_modules:
            with self.subTest(planned_test_module=module):
                self.assertIn(f"`{module}`", tasks)
                self.assertIn(f"`{module}`", traceability)

        table_rows = {cells[0]: cells for cells in _markdown_table_rows(traceability)}
        common_verification = table_rows[
            "Common message, batch, and result model"
        ][4]
        for module in LIFECYCLE_OPERATION_EXECUTION_MODULES:
            with self.subTest(shared_execution_module=module):
                self.assertIn(module, common_verification)
        self.assertNotIn(
            "crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs",
            common_verification,
        )
        self.assertIn(
            "crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs",
            table_rows["Generic TTLV and secret handling"][4],
        )

        for code_path in LIFECYCLE_PLANNED_CODE_PATHS:
            with self.subTest(planned_code_path=code_path):
                self.assertIn(f"`{code_path}`", tasks)
                self.assertIn(f"`{code_path}`", traceability)


    def test_credentials_and_attestation_traceability_rows_are_complete(self) -> None:
        with CLIENT_CREDENTIALS_REQUIREMENTS_PATH.open(
            encoding="utf-8", newline=""
        ) as stream:
            rows = list(csv.DictReader(stream))

        self.assertTrue(rows)
        by_id = {row["requirement_id"]: row for row in rows}
        self.assertEqual(len(rows), len(by_id))

        specification = CLIENT_CREDENTIALS_SPEC_PATH.read_text(encoding="utf-8")
        expected_feature_ids = {
            f"KMIPKIT-0008-FR-{number}"
            for number in re.findall(r"^\s*- \*\*FR-(\d{3})\*\*", specification, re.MULTILINE)
        }
        actual_feature_ids = {
            row["requirement_id"]
            for row in rows
            if row["requirement_id"].startswith("KMIPKIT-0008-FR-")
        }
        self.assertEqual(actual_feature_ids, expected_feature_ids)

        expected_oasis_ids = {
            "KMIPKIT-REQ-SPEC-9.3-001-001",
            "KMIPKIT-REQ-SPEC-9.3-001-002",
            "KMIPKIT-REQ-SPEC-9.4-001-001",
            "KMIPKIT-REQ-SPEC-9.4-001-002",
            "KMIPKIT-REQ-SPEC-9.4-001-003",
            "KMIPKIT-REQ-SPEC-9.4-002",
            "KMIPKIT-REQ-SPEC-9.11-001",
            "KMIPKIT-REQ-SPEC-9.11-004-001",
            "KMIPKIT-REQ-SPEC-9.11-004-002",
            "KMIPKIT-REQ-SPEC-9.11-004-003",
            "KMIPKIT-REQ-SPEC-9.11-006",
            "KMIPKIT-REQ-SPEC-9.11-010-001",
            "KMIPKIT-REQ-SPEC-9.11-010-002",
        }
        actual_oasis_ids = {
            row["requirement_id"]
            for row in rows
            if row["requirement_kind"] == "OASIS normative"
        }
        self.assertEqual(actual_oasis_ids, expected_oasis_ids)

        attestation_requirement = by_id["KMIPKIT-REQ-SPEC-9.3-001-001"]
        self.assertIn(
            "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::execute_advertises_attestation_without_authentication_or_credential_payload",
            attestation_requirement["test_ids"],
        )
        self.assertIn(
            "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::async_follow_up_request_advertises_attestation_capability",
            attestation_requirement["test_ids"],
        )
        attestation_feature_requirement = by_id["KMIPKIT-0008-FR-008"]
        self.assertIn(
            "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::execute_advertises_attestation_without_authentication_or_credential_payload",
            attestation_feature_requirement["test_ids"],
        )
        self.assertIn(
            "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::async_follow_up_request_advertises_attestation_capability",
            attestation_feature_requirement["test_ids"],
        )

        for row in rows:
            requirement_id = row["requirement_id"]
            with self.subTest(requirement_id=requirement_id):
                self.assertTrue(row["source_document"])
                self.assertTrue(row["source_section"])
                implementation_refs = row["implementation_location"].split("; ")
                test_refs = row["test_ids"].split("; ")
                for reference in filter(None, implementation_refs):
                    self.assertTrue((ROOT / reference).is_file(), reference)
                if row["status"] in {"verified", "scoped_verified"}:
                    self.assertTrue(row["implementation_location"], requirement_id)
                    self.assertTrue(row["test_ids"], requirement_id)
                for reference in filter(None, test_refs):
                    self.assertTrue(
                        self._is_executable_test_ref(reference),
                        f"{requirement_id}: {reference}",
                    )

        for requirement_id, status in (
            ("KMIPKIT-REQ-SPEC-9.4-001-003", "server_only"),
            ("KMIPKIT-REQ-SPEC-9.11-004-002", "scoped_verified"),
            ("KMIPKIT-REQ-SPEC-9.11-010-001", "verified"),
        ):
            self.assertEqual(by_id[requirement_id]["status"], status)
        self.assertIn("OD-003", by_id["KMIPKIT-REQ-SPEC-9.11-010-001"]["scope"])
        self.assertIn("caller", by_id["KMIPKIT-REQ-SPEC-9.11-004-002"]["statement"].lower())
        self.assertIn("comparison scope", by_id["KMIPKIT-REQ-SPEC-9.11-004-002"]["statement"].lower())

        for row in rows:
            if row["requirement_id"] in {
                "KMIPKIT-REQ-SPEC-9.4-001-003",
            }:
                continue
            if row["requirement_id"] == "KMIPKIT-REQ-SPEC-9.11-004-002":
                continue
            self.assertEqual(row["status"], "verified", row["requirement_id"])

    def test_result_contract_elements_link_to_their_spec_code_and_tests(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        elements = catalog["elements"]
        result_element_ids = _result_element_ids(elements)

        by_id = {element["element_id"]: element for element in elements}
        self.assertTrue(result_element_ids.issubset(by_id))
        self.assertEqual(len(result_element_ids), 82)

        for element_id in sorted(result_element_ids):
            with self.subTest(element_id=element_id):
                element = by_id[element_id]
                self.assertEqual(element["feature_spec"], FEATURE_SPEC)
                implementation_refs = element["implementation_refs"]
                verification_refs = element["verification_refs"]
                required_implementation_refs = {"crates/kmipkit-protocol/src/result.rs"}
                if element.get("kind") == "enumeration_value" and element.get("allocation") == "assigned":
                    required_implementation_refs.add(
                        "crates/kmipkit-protocol/src/result_values_generated.rs"
                    )
                self.assertTrue(required_implementation_refs.issubset(implementation_refs))

                if "RESULT-MESSAGE" in element_id:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::result_message_preserves_presence_and_text",
                        "crates/kmipkit-protocol/tests/result_contract.rs::operation_result_display_and_debug_redact_message",
                    }
                elif "RESULT-REASON" in element_id:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::known_reason_values_match_catalog",
                        "crates/kmipkit-protocol/tests/result_contract.rs::unknown_reason_preserves_raw_value",
                        "crates/kmipkit-protocol/tests/result_contract.rs::failure_requires_a_reason",
                        "crates/kmipkit-protocol/tests/result_contract.rs::success_forbids_a_reason",
                    }
                else:
                    required_verification_refs = {
                        "crates/kmipkit-protocol/tests/result_contract.rs::known_status_values_match_catalog",
                        "crates/kmipkit-protocol/tests/result_contract.rs::unknown_status_preserves_raw_value",
                        "crates/kmipkit-protocol/tests/result_contract.rs::other_statuses_do_not_gain_reason_presence_rules",
                    }
                self.assertTrue(required_verification_refs.issubset(verification_refs))

                for reference in implementation_refs:
                    self.assertTrue((ROOT / reference.split("::", 1)[0]).is_file(), reference)
                for reference in verification_refs:
                    test_path = reference.split("::", 1)[0]
                    self.assertTrue((ROOT / test_path).is_file(), reference)

    def test_result_response_clause_directions_preserve_source_scope(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        expected_clause_directions = {
            "KMIPKIT-CLAUSE-SPEC-9.17-001": "server_to_client",
            "KMIPKIT-CLAUSE-SPEC-9.18-001": "server_to_client",
            "KMIPKIT-CLAUSE-SPEC-9.19-001": "server_to_client",
        }
        clauses_by_id = {clause["clause_id"]: clause for clause in catalog["source_clauses"]}
        self.assertTrue(set(expected_clause_directions).issubset(clauses_by_id))
        for clause_id, expected_direction in sorted(expected_clause_directions.items()):
            with self.subTest(clause_id=clause_id):
                self.assertEqual(clauses_by_id[clause_id]["direction"], expected_direction)

        result_element_ids = _result_element_ids(catalog["elements"])
        elements_by_id = {element["element_id"]: element for element in catalog["elements"]}
        for element_id in sorted(result_element_ids):
            with self.subTest(element_id=element_id):
                self.assertEqual(elements_by_id[element_id]["direction"], "server_to_client")

    def test_sc005_references_executable_traceability_tests(self) -> None:
        rows = _requirements_by_id()

        self.assertEqual(
            rows["KMIPKIT-0003-SC-005"]["test_ids"].split("; "),
            [
                "tools/normative_catalog/tests/test_feature_traceability.py::FeatureTraceabilityTests.test_result_contract_elements_link_to_their_spec_code_and_tests",
                "tools/normative_catalog/tests/test_feature_traceability.py::FeatureTraceabilityTests.test_normative_csv_rows_have_executable_test_refs",
            ],
        )

    def test_normative_csv_rows_have_executable_test_refs(self) -> None:
        rows = _requirements_by_id()

        for requirement_id in ("KMIPKIT-0003-NR-001", "KMIPKIT-0003-NR-002", "KMIPKIT-0003-NR-003"):
            with self.subTest(requirement_id=requirement_id):
                test_refs = rows[requirement_id]["test_ids"].split("; ")
                self.assertTrue(test_refs)
                for test_ref in test_refs:
                    self.assertTrue(self._is_executable_test_ref(test_ref), test_ref)

        for test_ref in rows["KMIPKIT-0003-SC-005"]["test_ids"].split("; "):
            with self.subTest(sc005_test_ref=test_ref):
                self.assertTrue(self._is_executable_test_ref(test_ref), test_ref)

    def test_executable_test_refs_reject_absolute_and_traversal_paths(self) -> None:
        test_name = "FeatureTraceabilityTests.test_executable_test_refs_reject_absolute_and_traversal_paths"
        test_file = Path(__file__).resolve()
        references = (
            f"{test_file}::{test_name}",
            f"tools/normative_catalog/tests/../tests/test_feature_traceability.py::{test_name}",
        )
        for reference in references:
            with self.subTest(reference=reference):
                self.assertFalse(self._is_executable_test_ref(reference))

    def test_rust_executable_test_refs_require_an_active_test_declaration(self) -> None:
        test_name = "request_capability_is_available"
        source_by_case = {
            "block comment": (
                "/*\n#[test]\nfn request_capability_is_available() { assert!(true); }\n*/\n",
                False,
            ),
            "nested block comment": (
                "/* outer /* nested */\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n*/\n",
                False,
            ),
            "line comments": (
                "// #[test]\n// fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "raw string": (
                'const DECOY: &str = r###"\n#[test]\n'
                "fn request_capability_is_available() { assert!(true); }\n"
                '"###;\n',
                False,
            ),
            "disabled cfg": (
                "#[cfg(any())]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "disabled crate cfg": (
                "#![cfg(any())]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "unknown crate cfg is not accepted as active": (
                "#![cfg(feature = \"optional-tests\")]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "unknown crate cfg_attr is not accepted as active": (
                "#![cfg_attr(feature = \"optional-tests\", cfg(any()))]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "disabled cfg on parent module": (
                "#[cfg(any())]\nmod inactive {\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n}\n",
                False,
            ),
            "unknown cfg is not accepted as active": (
                "#[cfg(feature = \"optional-tests\")]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "unrelated conditional test does not disable active test reference": (
                "#[cfg(target_pointer_width = \"32\")]\n#[test]\n"
                "fn unrelated_32_bit_test() { assert!(true); }\n"
                "#[test]\nfn request_capability_is_available() { assert!(true); }\n",
                True,
            ),
            "missing test attribute": (
                "fn request_capability_is_available() { assert!(true); }\n",
                False,
            ),
            "active test": (
                "#[test]\nfn request_capability_is_available() { assert!(true); }\n",
                True,
            ),
            "active test inside an inline test module": (
                "mod tests {\n    #[test]\n    fn request_capability_is_available() { assert!(true); }\n}\n",
                True,
            ),
            "disabled cfg on inline test module": (
                "#[cfg(any())]\nmod tests {\n    #[test]\n"
                "    fn request_capability_is_available() { assert!(true); }\n}\n",
                False,
            ),
            "disabled inner cfg on inline test module": (
                "mod tests {\n    #![cfg(any())]\n    #[test]\n"
                "    fn request_capability_is_available() { assert!(true); }\n}\n",
                False,
            ),
            "active inner test cfg on inline module": (
                "mod tests {\n    #![cfg(test)]\n    #[test]\n"
                "    fn request_capability_is_available() { assert!(true); }\n}\n",
                True,
            ),
            "test cfg is enabled in a test build": (
                "#[cfg(test)]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                True,
            ),
            "crate test cfg is enabled in a test build": (
                "#![cfg(test)]\n#[test]\n"
                "fn request_capability_is_available() { assert!(true); }\n",
                True,
            ),
        }

        with tempfile.TemporaryDirectory(dir=ROOT) as repo_temporary:
            test_file = Path(repo_temporary) / "evidence_fixture.rs"
            reference_path = test_file.relative_to(ROOT).as_posix()
            for label, (source, expected) in source_by_case.items():
                with self.subTest(label):
                    test_file.write_text(source, encoding="utf-8")
                    reference = f"{reference_path}::{test_name}"
                    self.assertEqual(expected, self._is_executable_test_ref(reference))

    def test_executable_test_refs_reject_symlinks_outside_repository(self) -> None:
        test_name = "Outside.test_referenced_test"
        with tempfile.TemporaryDirectory(dir=ROOT) as repo_temporary:
            with tempfile.TemporaryDirectory() as external_temporary:
                external_test = Path(external_temporary) / "outside.py"
                external_test.write_text(
                    "class Outside:\\n    def test_referenced_test(self):\\n        pass\\n",
                    encoding="utf-8",
                )
                link = Path(repo_temporary) / "outside.py"
                try:
                    link.symlink_to(external_test)
                except OSError as error:
                    self.skipTest(f"symlink creation is unavailable: {error}")

                reference = f"{link.relative_to(ROOT).as_posix()}::{test_name}"
                self.assertFalse(self._is_executable_test_ref(reference))

    def test_executable_test_refs_reject_oversized_source_files(self) -> None:
        test_name = "Oversized.test_referenced_test"
        with tempfile.TemporaryDirectory(dir=ROOT) as repo_temporary:
            source = (
                "class Oversized:\\n    def test_referenced_test(self):\\n        pass\\n"
                + (" " * (1024 * 1024))
            )
            test_file = Path(repo_temporary) / "oversized.py"
            test_file.write_text(source, encoding="utf-8")
            reference = f"{test_file.relative_to(ROOT).as_posix()}::{test_name}"

            self.assertFalse(self._is_executable_test_ref(reference))

    def test_profile_requirements_are_owned_by_profile_conformance(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        in_scope_client_profiles = {
            row["profile_id"]
            for row in catalog["profiles"]
            if row.get("role") == "client"
            and row.get("applicability") in {"client_1_0", "conditional"}
        }
        in_scope_requirement_ids = {
            requirement_id
            for profile in catalog["profiles"]
            if profile.get("profile_id") in in_scope_client_profiles
            for requirement_id in profile.get("requirement_ids", [])
        }
        requirements = {
            row["requirement_id"]: row
            for row in catalog["requirements"]
        }
        profile_requirement_ids = {
            requirement_id
            for requirement_id in in_scope_requirement_ids
            if requirement_id.startswith("KMIPKIT-REQ-PROF-")
        }

        self.assertEqual(163, len(profile_requirement_ids))
        incorrectly_owned_profile_requirements = [
            (requirement_id, requirements[requirement_id].get("feature_spec"))
            for requirement_id in profile_requirement_ids
            if requirements[requirement_id].get("feature_spec") != "KMIPKIT-0010"
        ]
        self.assertEqual([], incorrectly_owned_profile_requirements)

        incorrectly_owned_base_requirements = sorted(
            requirement_id
            for requirement_id in in_scope_requirement_ids
            if requirement_id.startswith("KMIPKIT-REQ-SPEC-")
            and requirements[requirement_id].get("feature_spec")
            == PROFILE_CONFORMANCE_FEATURE_SPEC
            and requirement_id not in PROFILE_CONFORMANCE_GATE_REQUIREMENT_IDS
        )
        self.assertEqual([], incorrectly_owned_base_requirements)
        for requirement_id in PROFILE_CONFORMANCE_GATE_REQUIREMENT_IDS:
            self.assertIn(requirement_id, requirements)
            self.assertEqual(
                requirements[requirement_id].get("feature_spec"),
                PROFILE_CONFORMANCE_FEATURE_SPEC,
            )

    def test_typed_profile_mappings_resolve_to_closure_and_traceable_code_and_tests(self) -> None:
        from tools.normative_catalog.generate_profiles import _verification_category
        from tools.normative_catalog.report import (
            _RULE_TEST_KIND_BY_CATEGORY,
            load_typed_rule_mappings,
        )

        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            row["requirement_id"]: row for row in catalog["requirements"]
        }
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        oracle_path = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        oracle_profiles = {row["profile_id"]: row for row in oracle["profiles"]}
        in_scope_client_requirement_ids = {
            requirement_id
            for profile in catalog["profiles"]
            if profile.get("role") == "client"
            and profile.get("applicability") in {"client_1_0", "conditional"}
            for requirement_id in profile.get("requirement_ids", [])
        }
        source_clause_ids = {row["clause_id"] for row in catalog["source_clauses"]}
        for profile in catalog["profiles"]:
            if profile.get("role") != "client" or profile.get("applicability") not in {
                "client_1_0",
                "conditional",
            }:
                continue
            with self.subTest(profile_id=profile["profile_id"]):
                self.assertTrue(profile.get("source_clause_ids"))
                self.assertTrue(profile.get("requirement_ids"))
                self.assertIn(profile["profile_id"], oracle_profiles)
                expected = oracle_profiles[profile["profile_id"]]
                self.assertEqual(
                    set(profile["source_clause_ids"]), set(expected["source_clause_ids"])
                )
                self.assertEqual(
                    set(profile["requirement_ids"]), set(expected["requirement_ids"])
                )
                self.assertTrue(set(profile["source_clause_ids"]).issubset(source_clause_ids))
                self.assertTrue(
                    set(profile["requirement_ids"]).issubset(requirements),
                    profile["profile_id"],
                )

        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile.get("profile_id") == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        for inherited_requirement_id in (
            "KMIPKIT-REQ-PROF-3.1.1-002",
            "KMIPKIT-REQ-PROF-3.1.1-005",
            "KMIPKIT-REQ-PROF-3.1.2-002",
            "KMIPKIT-REQ-PROF-3.1.2-005",
        ):
            self.assertIn(inherited_requirement_id, baseline["requirement_ids"])

        mappings = load_typed_rule_mappings(ROOT)
        self.assertTrue(mappings)
        required_test_kinds: dict[str, str] = {}
        observed_test_kinds: dict[str, set[str]] = {}
        missing_category_test_ids: set[str] = set()

        for mapping in mappings:
            requirement_id = mapping["requirement_id"]
            with self.subTest(requirement_id=requirement_id, test_id=mapping["test_id"]):
                self.assertIn(requirement_id, requirements)
                self.assertIn(requirement_id, in_scope_client_requirement_ids)
                requirement = requirements[requirement_id]
                if requirement_id.startswith("KMIPKIT-REQ-PROF-"):
                    self.assertEqual(
                        requirement.get("feature_spec"),
                        PROFILE_CONFORMANCE_FEATURE_SPEC,
                    )
                elif requirement_id in PROFILE_CONFORMANCE_GATE_REQUIREMENT_IDS:
                    self.assertEqual(
                        requirement.get("feature_spec"),
                        PROFILE_CONFORMANCE_FEATURE_SPEC,
                    )
                else:
                    self.assertNotEqual(
                        requirement.get("feature_spec"),
                        PROFILE_CONFORMANCE_FEATURE_SPEC,
                    )

                source_clause_id = mapping["source_clause_id"]
                if source_clause_id is None:
                    source_gap_id = mapping["source_gap_id"]
                    self.assertIsNotNone(source_gap_id)
                    self.assertEqual(source_gap_id, requirement.get("source_gap_id"))
                    source_locator = requirement.get("source_locator")
                    self.assertIsInstance(source_locator, dict)
                    self.assertEqual(source_locator.get("kind"), "table_body_row")
                    self.assertTrue(source_locator.get("row_label"))
                    self.assertTrue(source_locator.get("html_line_range"))
                    self.assertIn(
                        source_locator.get("section"),
                        [
                            source_ref.get("section")
                            for source_ref in requirement.get("source_refs", [])
                            if source_ref.get("source_id") == "KMIPKIT-SRC-spec"
                        ],
                    )
                else:
                    self.assertIn(source_clause_id, clauses)
                    self.assertIn(source_clause_id, requirement.get("source_clause_ids", []))
                keyword = requirement.get("source_keyword") or requirement.get("source_force")
                category = _verification_category(keyword)
                self.assertEqual(mapping["verification_category"], category)
                expected_test_kind = _RULE_TEST_KIND_BY_CATEGORY[category]
                if mapping["test_id"] is None:
                    missing_category_test_ids.add(requirement_id)
                    self.assertIsNone(mapping["test_kind"])
                else:
                    required_test_kinds[requirement_id] = expected_test_kind
                    observed_test_kinds.setdefault(requirement_id, set()).add(mapping["test_kind"])
                    if category == "ProhibitedBehavior" and mapping["test_kind"] == "PositiveBehavior":
                        self.assertTrue(
                            mapping["test_id"].startswith(
                                "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                            ),
                            "positive evidence for a prohibition is limited to a profile configuration check",
                        )
                    elif category == "RequiredBehavior" and mapping["test_kind"] == "NegativeBehavior":
                        # Negative checks may supplement a required rule; the positive kind is asserted below.
                        self.assertIn(mapping["test_id"], requirement.get("verification_refs", []))
                    else:
                        self.assertEqual(mapping["test_kind"], expected_test_kind)
                    if mapping["test_id"] not in requirement.get("verification_refs", []):
                        self.assertIn(
                            requirement_id,
                            {
                                "KMIPKIT-REQ-SPEC-9.6-001-002",
                                "KMIPKIT-REQ-SPEC-9.12-001-003",
                                "KMIPKIT-REQ-SPEC-6.1.43-001",
                                "KMIPKIT-REQ-SPEC-6.1.43-007",
                                "KMIPKIT-REQ-SPEC-6.1.43-009-001",
                                "KMIPKIT-REQ-SPEC-6.1.43-010-001",
                                "KMIPKIT-REQ-SPEC-6.1.43-011",
                                "KMIPKIT-REQ-SPEC-6.1.5-001",
                                "KMIPKIT-REQ-SPEC-6.1.38-001-001",
                                "KMIPKIT-REQ-SPEC-9.1-001",
                                "KMIPKIT-REQ-SPEC-9.19-002",
                                "KMIPKIT-REQ-SPEC-6.1.19-002",
                                "KMIPKIT-REQ-SPEC-6.1.19-003",
                                "KMIPKIT-REQ-SPEC-6.1.19-004",
                                "KMIPKIT-REQ-SPEC-6.1.19-005",
                                "KMIPKIT-REQ-SPEC-6.1.19-006",
                                "KMIPKIT-REQ-SPEC-6.1.13-006",
                                "KMIPKIT-REQ-SPEC-3.3-001-001",
                                "KMIPKIT-REQ-SPEC-3.3-001-002",
                                "KMIPKIT-REQ-SPEC-7.18-006",
                                # The KMIPKIT-0007 owner record remains deferred under OD-004;
                                # the separate KMIPKIT-0010 OptionalCapability carries its own
                                # typed profile evidence.
                                "KMIPKIT-REQ-SPEC-9.20-001-002",
                            },
                            "profile evaluator evidence is maintained in the typed rule registry, not the owning feature inventory",
                        )
                    self.assertTrue(
                        self._is_executable_test_ref(mapping["test_id"]),
                        mapping["test_id"],
                    )

                for reference in requirement.get("implementation_refs", []):
                    path, separator, symbol = reference.partition("::")
                    source_file = _read_confined_implementation_source(path)
                    self.assertIsNotNone(source_file, reference)
                    if separator:
                        _, source = source_file or (None, "")
                        source_without_comments = _strip_rust_comments_and_literals(source)
                        self.assertIsNotNone(source_without_comments, reference)
                        self.assertIsNotNone(
                            re.search(
                                rf"\b(?:fn|struct|enum|trait|type|const|static)\s+{re.escape(symbol.split('::')[-1])}\b",
                                source_without_comments or "",
                            ),
                            reference,
                        )

                for reference in requirement.get("verification_refs", []):
                    path, separator, _ = reference.partition("::")
                    self.assertIsNotNone(_read_confined_test_source(path), reference)
                    if separator:
                        self.assertTrue(self._is_executable_test_ref(reference), reference)

        self.assertEqual(missing_category_test_ids, set())
        for requirement_id, required_test_kind in required_test_kinds.items():
            with self.subTest(requirement_id=requirement_id):
                self.assertIn(required_test_kind, observed_test_kinds[requirement_id])

    @staticmethod
    def _is_executable_test_ref(test_ref: str) -> bool:
        test_path, separator, test_name = test_ref.partition("::")
        if not separator or not test_name:
            return False

        source_file = _read_confined_test_source(test_path)
        if source_file is None:
            return False
        path, source = source_file

        return _is_discoverable_positive_test(path, test_name, source)

    def test_ignored_rust_tests_are_not_positive_evidence(self) -> None:
        """An ignored test does not run in the normal suite and cannot prove a capability."""
        for ignore_attribute in ('#[ignore]', '#[ignore = "requires unavailable service"]'):
            with self.subTest(ignore_attribute=ignore_attribute):
                source = (
                    "#[test]\n"
                    f"{ignore_attribute}\n"
                    "fn ignored_capability_check() {\n"
                    "    assert!(true);\n"
                    "}\n"
                )
                self.assertFalse(
                    _is_discoverable_positive_test(
                        Path("tests/ignored_capability.rs"),
                        "ignored_capability_check",
                        source,
                    )
                )

    def test_skipped_python_tests_are_not_positive_evidence(self) -> None:
        """Python tests excluded by standard skip markers cannot prove a capability."""
        cases = (
            (
                "unittest skip decorator",
                "test_capability",
                "import unittest\n"
                "@unittest.skip('requires unavailable service')\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "pytest skipif decorator",
                "test_capability",
                "import pytest\n"
                "@pytest.mark.skipif(True, reason='not available')\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "aliased unittest skip decorator",
                "test_capability",
                "from unittest import skip as omit_test\n"
                "@omit_test('requires unavailable service')\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "aliased pytest function skip marker",
                "test_capability",
                "import pytest\n"
                "skip_marker = pytest.mark.skip(reason='service unavailable')\n"
                "@skip_marker\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "unittest class skip decorator",
                "TestSkipped.test_capability",
                "import unittest\n"
                "@unittest.skip('class unavailable')\n"
                "class TestSkipped(unittest.TestCase):\n"
                "    def test_capability(self):\n"
                "        self.assertTrue(True)\n",
            ),
            (
                "pytest module skip marker",
                "test_capability",
                "import pytest\n"
                "pytestmark = pytest.mark.skip(reason='module unavailable')\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "aliased pytest module skip marker",
                "test_capability",
                "import pytest\n"
                "skip_marker = pytest.mark.skip(reason='module unavailable')\n"
                "pytestmark = [skip_marker]\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "mutated pytest module skip marker",
                "test_capability",
                "import pytest\n"
                "pytestmark = []\n"
                "pytestmark.append(pytest.mark.skip(reason='module unavailable'))\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
            (
                "mutated aliased pytest module skip marker",
                "test_capability",
                "import pytest\n"
                "skip_marks = []\n"
                "pytestmark = skip_marks\n"
                "skip_marks.append(pytest.mark.skip(reason='module unavailable'))\n"
                "def test_capability():\n"
                "    assert True\n",
            ),
        )
        for label, test_name, source in cases:
            with self.subTest(label=label):
                self.assertFalse(
                    _is_discoverable_positive_test(Path("tests/evidence.py"), test_name, source)
                )

    def test_non_skip_decorator_keyword_named_skip_remains_discoverable(self) -> None:
        source = (
            "skip = False\n"
            "def custom_marker(**kwargs):\n"
            "    return lambda function: function\n"
            "@custom_marker(skip=skip)\n"
            "def test_capability():\n"
            "    assert True\n"
        )
        self.assertTrue(
            _is_discoverable_positive_test(
                Path("tests/evidence.py"), "test_capability", source
            )
        )
        source_with_attribute = (
            "def custom_marker(**kwargs):\n"
            "    return lambda function: function\n"
            "class Config:\n"
            "    skip = False\n"
            "config = Config()\n"
            "@custom_marker(option=config.skip)\n"
            "def test_capability():\n"
            "    assert True\n"
        )
        self.assertTrue(
            _is_discoverable_positive_test(
                Path("tests/evidence.py"), "test_capability", source_with_attribute
            )
        )


class MessageModelTraceabilityTests(unittest.TestCase):
    @staticmethod
    def _is_executable_test_ref(test_ref: str) -> bool:
        return FeatureTraceabilityTests._is_executable_test_ref(test_ref)

    def test_message_model_csv_maps_requirements_and_executable_evidence(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        with MESSAGE_MODEL_REQUIREMENTS_PATH.open(encoding="utf-8", newline="") as stream:
            rows = list(csv.DictReader(stream))

        expected_header = [
            "requirement_id",
            "requirement_kind",
            "source_document",
            "source_section",
            "normative_level",
            "scope",
            "statement",
            "implementation_location",
            "test_ids",
            "status",
        ]
        self.assertEqual(list(rows[0]), expected_header)
        by_id = {row["requirement_id"]: row for row in rows}
        self.assertEqual(len(by_id), len(rows))

        field_order_id = "KMIPKIT-REQ-SPEC-10.1.2-001"
        field_order_row = by_id[field_order_id]
        self.assertEqual(field_order_row["status"], "scoped_verified")
        self.assertIn("only the 0006 scope", field_order_row["statement"])
        field_order_refs = field_order_row["test_ids"].split("; ")
        self.assertTrue(field_order_refs)
        for reference in field_order_refs:
            self.assertTrue(self._is_executable_test_ref(reference), reference)

        changed_ttlv_sources = {
            "crates/kmipkit-ttlv/src/lib.rs",
            "crates/kmipkit-ttlv/src/structure.rs",
            "crates/kmipkit-ttlv/src/value.rs",
        }
        for requirement_id in (
            "KMIPKIT-0006-FR-017",
            "KMIPKIT-0006-SC-006",
            "KMIPKIT-0006-SC-007",
        ):
            implementation_paths = set(
                by_id[requirement_id]["implementation_location"].split("; ")
            )
            self.assertTrue(
                changed_ttlv_sources.issubset(implementation_paths),
                requirement_id,
            )

        global_field_order = next(
            record
            for record in catalog["requirements"]
            if record["requirement_id"] == field_order_id
        )
        self.assertEqual(global_field_order["status"], "scoped_verified")
        self.assertEqual(global_field_order["feature_spec"], "KMIPKIT-0005")
        self.assertEqual(
            global_field_order["implementation_refs"],
            [
                "crates/kmipkit-client/src/wire_encoder.rs",
                "crates/kmipkit-protocol/src/profile.rs",
            ],
        )
        self.assertIn(
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_ttlv_structure_order_and_numeric_padding",
            global_field_order["verification_refs"],
        )
        self.assertIn(
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_a_device_identifier_field_in_device_credentials",
            global_field_order["verification_refs"],
        )

        coverage_row = by_id["KMIPKIT-0006-SC-007"]
        self.assertEqual(coverage_row["status"], "verified")
        self.assertEqual(
            coverage_row["test_ids"].split("; "),
            [
                "cargo llvm-cov --workspace --all-features --summary-only",
                "cargo llvm-cov -p kmipkit-ttlv --all-features --summary-only",
                "cargo llvm-cov -p kmipkit-protocol --all-features --summary-only",
            ],
        )

        source_requirements = {
            record["requirement_id"]
            for record in catalog["requirements"]
            if any(
                reference["source_id"] == "KMIPKIT-SRC-spec"
                and reference["section"].split(".")[0] in {"8", "9"}
                for reference in record["source_refs"]
            )
        }
        retired_false_client_claim = by_id["KMIPKIT-REQ-SPEC-9.8-001-001"]
        self.assertEqual(retired_false_client_claim["status"], "retired")
        self.assertEqual(retired_false_client_claim["scope"], "out_of_scope")
        self.assertIn("client candidate", retired_false_client_claim["statement"].lower())
        self.assertTrue(source_requirements.issubset(by_id))
        self.assertEqual(
            by_id["KMIPKIT-REQ-SPEC-8-003-002"]["status"],
            "verified",
        )
        self.assertIn(
            "one_response_batch_can_mix_completed_and_pending_results",
            by_id["KMIPKIT-REQ-SPEC-8-003-002"]["test_ids"],
        )
        result_message_requirement = by_id["KMIPKIT-REQ-SPEC-9.17-001-002"]
        self.assertEqual(result_message_requirement["status"], "scoped_verified")
        self.assertEqual(result_message_requirement["scope"], "client_1_0; owner=KMIPKIT-0006")
        self.assertIn("caller-controlled", result_message_requirement["statement"])
        self.assertIn("does not automatically log", result_message_requirement["statement"])
        self.assertIn(
            "crates/kmipkit-protocol/tests/result_contract.rs::result_message_preserves_presence_and_text",
            result_message_requirement["test_ids"],
        )
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-9.8-001-002",
            "KMIPKIT-REQ-SPEC-9.8-001-003",
        ):
            self.assertEqual(by_id[requirement_id]["status"], "server_only")
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-9.12-001-002",
            "KMIPKIT-REQ-SPEC-9.12-001-003",
            "KMIPKIT-REQ-SPEC-9.13-001-004",
            "KMIPKIT-REQ-SPEC-9.13-001-005",
        ):
            self.assertIn("owner=KMIPKIT-0007", by_id[requirement_id]["scope"])
            self.assertEqual(by_id[requirement_id]["status"], "deferred")
        self.assertIn("owner=KMIPKIT-0009", by_id["KMIPKIT-REQ-SPEC-9.19-002"]["scope"])
        for discrepancy_id, owner in (
            ("KMIPKIT-DISC-001", "KMIPKIT-0007"),
            ("KMIPKIT-DISC-022", "KMIPKIT-0007"),
            ("KMIPKIT-DISC-039", "KMIPKIT-0009"),
            ("KMIPKIT-DISC-041", "KMIPKIT-0008"),
            ("KMIPKIT-DISC-042", "KMIPKIT-0008"),
        ):
            self.assertIn(discrepancy_id, by_id)
            self.assertIn(f"owner={owner}", by_id[discrepancy_id]["scope"])
            self.assertEqual(by_id[discrepancy_id]["status"], "deferred")

        for row in rows:
            with self.subTest(requirement_id=row["requirement_id"]):
                self.assertTrue(row["source_section"])
                self.assertTrue(row["implementation_location"])
                if row["status"] != "verified":
                    continue
                if row["requirement_id"] == "KMIPKIT-0006-SC-007":
                    continue
                references = row["test_ids"].split("; ")
                self.assertTrue(references)
                for reference in references:
                    self.assertTrue(self._is_executable_test_ref(reference), reference)

        for requirement_id in source_requirements:
            record = next(
                item for item in catalog["requirements"] if item["requirement_id"] == requirement_id
            )
            row = by_id[requirement_id]
            if row["status"] == "verified":
                self.assertEqual(record["feature_spec"], MESSAGE_MODEL_FEATURE_SPEC)
            elif row["status"] == "server_only":
                self.assertEqual(record["scope_state"], "server_only")
                self.assertIn(record["feature_spec"], {None, MESSAGE_MODEL_FEATURE_SPEC})
            elif row["status"] == "retired":
                self.assertEqual(record["role"], "not_applicable")
                self.assertEqual(record["scope_state"], "out_of_scope")
                self.assertEqual(record["status"], "retired")
            else:
                self.assertEqual(record["feature_spec"], row["scope"].split("; owner=")[1])
            for reference in record["verification_refs"]:
                test_path = reference.split("::", 1)[0]
                self.assertTrue((ROOT / test_path).is_file(), reference)
                if "::" in reference:
                    self.assertTrue(self._is_executable_test_ref(reference), reference)

    def test_multi_item_identifier_requirement_cites_exact_oasis_table(self) -> None:
        with MESSAGE_MODEL_REQUIREMENTS_PATH.open(encoding="utf-8", newline="") as stream:
            rows = list(csv.DictReader(stream))
        requirement = next(
            row
            for row in rows
            if row["requirement_id"] == "KMIPKIT-REQ-SPEC-8.3-003"
        )

        self.assertEqual(requirement["source_section"], "§8.3, Table 396")

    def test_message_extension_private_test_uses_excluded_test_path(self) -> None:
        with MESSAGE_MODEL_REQUIREMENTS_PATH.open(encoding="utf-8", newline="") as stream:
            rows = list(csv.DictReader(stream))
        requirement = next(
            row
            for row in rows
            if row["requirement_id"] == "KMIPKIT-0006-FR-021"
        )
        expected_reference = (
            "crates/kmipkit-protocol/tests/support/message_validation_unit.rs::"
            "message_extension_validation_checks_required_fields_order_and_values"
        )
        self.assertIn(expected_reference, requirement["test_ids"].split("; "))

    def test_message_model_field_catalog_links_code_and_tests(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        message_sections = {
            *(f"8.{index}" for index in range(1, 7)),
            "9.1",
            "9.2",
            "9.3",
            "9.4",
            "9.5",
            "9.6",
            "9.7",
            "9.8",
            "9.9",
            "9.10",
            "9.12",
            "9.13",
            "9.14",
            "9.15",
            "9.16",
            "9.17",
            "9.18",
            "9.19",
            "9.20",
            "9.21",
        }
        elements = [
            element
            for element in catalog["elements"]
            if element.get("kind") == "message_field"
            and any(
                reference["source_id"] == "KMIPKIT-SRC-spec"
                and reference["section"] in message_sections
                for reference in element.get("source_refs", [])
            )
            and element["element_id"] != "KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED"
        ]
        self.assertGreaterEqual(len(elements), 70)
        for element in elements:
            with self.subTest(element_id=element["element_id"]):
                self.assertEqual(element["feature_spec"], MESSAGE_MODEL_FEATURE_SPEC)
                self.assertTrue(element["implementation_refs"])
                self.assertTrue(element["verification_refs"])
                for reference in (*element["implementation_refs"], *element["verification_refs"]):
                    path = reference.split("::", 1)[0]
                    self.assertTrue((ROOT / path).is_file(), reference)

    def test_message_model_boundary_has_no_io_or_scheduling_dependencies(self) -> None:
        production_files = (
            ROOT / "crates/kmipkit-protocol/src/message/mod.rs",
            ROOT / "crates/kmipkit-protocol/src/message/header.rs",
            ROOT / "crates/kmipkit-protocol/src/message/batch.rs",
            ROOT / "crates/kmipkit-protocol/src/message/validation.rs",
            ROOT / "crates/kmipkit-protocol/src/message/version.rs",
        )
        forbidden = (
            "std::net",
            "std::thread",
            "tokio::",
            "reqwest::",
            "TcpStream",
            "TcpListener",
        )
        for path in production_files:
            source = path.read_text(encoding="utf-8")
            for token in forbidden:
                with self.subTest(path=path.name, token=token):
                    self.assertNotIn(token, source)


class ClientExecutionTraceabilityTests(unittest.TestCase):
    def test_client_execution_csv_covers_oasis_project_and_policy_records(self) -> None:
        self.assertTrue(CLIENT_EXECUTION_REQUIREMENTS_PATH.is_file())
        with CLIENT_EXECUTION_REQUIREMENTS_PATH.open(
            encoding="utf-8", newline=""
        ) as stream:
            rows = list(csv.DictReader(stream))

        expected_header = [
            "requirement_id",
            "requirement_kind",
            "source_document",
            "source_section",
            "normative_level",
            "scope",
            "statement",
            "implementation_location",
            "test_ids",
            "status",
        ]
        self.assertTrue(rows)
        self.assertEqual(list(rows[0]), expected_header)
        by_id = {row["requirement_id"]: row for row in rows}
        self.assertEqual(len(rows), len(by_id))

        oasis_requirement_ids = CLIENT_EXECUTION_OWNED_REQUIREMENT_IDS
        actual_oasis_ids = {
            row["requirement_id"]
            for row in rows
            if row["requirement_kind"] == "OASIS normative"
        }
        self.assertEqual(actual_oasis_ids, oasis_requirement_ids)

        specification = CLIENT_EXECUTION_SPEC_PATH.read_text(encoding="utf-8")
        expected_feature_ids = set(
            re.findall(r"KMIPKIT-0007-(?:FR|SC)-\d{3}", specification)
        )
        actual_feature_ids = {
            row["requirement_id"]
            for row in rows
            if row["requirement_id"].startswith("KMIPKIT-0007-FR-")
            or row["requirement_id"].startswith("KMIPKIT-0007-SC-")
        }
        self.assertEqual(actual_feature_ids, expected_feature_ids)

        policy_record_ids = {
            "KMIPKIT-POLICY-BATCH-ERROR-CONTINUATION-ASSIGNED-OUTBOUND",
            "KMIPKIT-POLICY-EXTENSION-PRESERVATION",
            "KMIPKIT-POLICY-UNKNOWN-FUTURE-VALUE-PRESERVATION",
        }
        discrepancy_and_decision_ids = {
            "KMIPKIT-DISC-001",
            "KMIPKIT-DISC-022",
            "KMIPKIT-DISC-043",
            "KMIPKIT-DEC-002",
        }
        for requirement_id in policy_record_ids:
            with self.subTest(policy_id=requirement_id):
                row = by_id[requirement_id]
                self.assertEqual(row["requirement_kind"], "KMIPKit project policy")
                self.assertEqual(row["normative_level"], "project policy")
        for record_id in discrepancy_and_decision_ids:
            with self.subTest(record_id=record_id):
                self.assertIn(record_id, by_id)
                self.assertNotEqual(by_id[record_id]["requirement_kind"], "OASIS normative")

        for row in rows:
            requirement_id = row["requirement_id"]
            with self.subTest(requirement_id=requirement_id):
                self.assertTrue(row["source_document"])
                self.assertTrue(row["source_section"])
                if row["requirement_kind"] == "OASIS normative":
                    self.assertIn(
                        "not an official oasis test case", row["statement"].lower()
                    )
                implementation_refs = row["implementation_location"].split("; ")
                test_refs = row["test_ids"].split("; ")
                for reference in filter(None, implementation_refs):
                    self.assertTrue((ROOT / reference).is_file(), reference)
                for reference in filter(None, test_refs):
                    self.assertTrue(
                        FeatureTraceabilityTests._is_executable_test_ref(reference),
                        reference,
                    )
                if row["status"] == "verified":
                    self.assertTrue(row["implementation_location"], requirement_id)
                    self.assertTrue(row["test_ids"], requirement_id)

        countdown = by_id[CLIENT_EXECUTION_DEFERRED_REQUIREMENT_ID]
        self.assertEqual(countdown["status"], "deferred")
        self.assertIn("OD-004", countdown["statement"])
        self.assertIn("countdown-derived", countdown["statement"])
        self.assertEqual(countdown["test_ids"], "")

    def test_client_execution_catalog_references_match_only_0007_owned_records(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        owned_records = {
            record["requirement_id"]: record
            for record in catalog["requirements"]
            if record.get("feature_spec") == CLIENT_EXECUTION_FEATURE_SPEC
        }
        self.assertEqual(set(owned_records), CLIENT_EXECUTION_OWNED_REQUIREMENT_IDS)
        with CLIENT_EXECUTION_REQUIREMENTS_PATH.open(
            encoding="utf-8", newline=""
        ) as stream:
            rows = {
                row["requirement_id"]: row for row in csv.DictReader(stream)
            }

        for requirement_id, record in owned_records.items():
            with self.subTest(requirement_id=requirement_id):
                row = rows[requirement_id]
                source_ref = record["source_refs"][0]
                self.assertIn(source_ref["source_id"], row["source_document"])
                self.assertTrue(
                    row["source_section"].startswith(f"§{source_ref['section']}")
                )
                self.assertEqual(record["test_case_ids"], [])
                if requirement_id == CLIENT_EXECUTION_DEFERRED_REQUIREMENT_ID:
                    self.assertEqual(record["status"], "deferred")
                    self.assertEqual(
                        set(record["implementation_refs"]),
                        CLIENT_EXECUTION_DEFERRED_PROFILE_IMPLEMENTATION_REFS,
                    )
                    self.assertEqual(
                        set(record["verification_refs"]),
                        CLIENT_EXECUTION_DEFERRED_PROFILE_VERIFICATION_REFS,
                    )
                    self.assertIn("OD-004", record["review_note"])
                    self.assertIn("countdown-derived", record["review_note"])
                    self.assertIn(
                        "KMIPKIT-0010 separately maps the caller-facing relative-date OptionalCapability",
                        record["review_note"],
                    )
                    self.assertEqual(row["status"], "deferred")
                    self.assertEqual(
                        row["implementation_location"],
                        "specs/007-client-execution/spec.md; specs/007-client-execution/research.md",
                    )
                    continue

                self.assertEqual(record["status"], "verified")
                self.assertTrue(record["implementation_refs"])
                self.assertTrue(record["verification_refs"])
                self.assertEqual(row["status"], "verified")
                row_implementation = set(row["implementation_location"].split("; "))
                row_verification = set(row["test_ids"].split("; "))
                self.assertEqual(set(record["implementation_refs"]), row_implementation)
                self.assertEqual(set(record["verification_refs"]), row_verification)
                for reference in record["implementation_refs"]:
                    self.assertTrue((ROOT / reference).is_file(), reference)
                for reference in record["verification_refs"]:
                    self.assertTrue(
                        FeatureTraceabilityTests._is_executable_test_ref(reference),
                        reference,
                    )

if __name__ == "__main__":
    unittest.main()
