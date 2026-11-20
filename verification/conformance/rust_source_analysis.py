"""Shared helpers for recognizing Rust test items in source files."""

from __future__ import annotations

import re
from collections.abc import Iterable


def _rust_item_is_top_level(source: str, item_start: int) -> bool:
    """Return whether a Rust item starts outside an inline module or body."""
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
    return _rust_test_item_contexts(source, (item_start,))[item_start]


def _rust_test_item_contexts(source: str, item_starts: Iterable[int]) -> dict[int, bool]:
    """Index module context for many Rust item offsets in one source pass."""
    ordered_starts = sorted(set(item_starts))
    if not ordered_starts:
        return {}

    module_stack: list[tuple[bool, int | None]] = []
    statement_start = 0
    cursor = 0
    contexts: dict[int, bool] = {}
    malformed = False

    for item_start in ordered_starts:
        target = min(item_start, len(source))
        while cursor < target:
            character = source[cursor]
            if character == "{":
                statement = source[statement_start:cursor].strip()
                module_match = re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*$", statement)
                if module_match is None:
                    module_stack.append((False, None))
                else:
                    module_attributes = statement[:module_match.start()]
                    if re.search(r"#\s*\[\s*cfg_attr\s*\(", module_attributes):
                        malformed = True
                        break
                    active_attributes = re.sub(
                        r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", "", module_attributes
                    )
                    active = not re.search(r"#\s*\[\s*cfg\s*\(", active_attributes)
                    module_stack.append((active, cursor))
                statement_start = cursor + 1
            elif character == "}":
                if not module_stack:
                    malformed = True
                    break
                module_stack.pop()
                statement_start = cursor + 1
            elif character == ";":
                statement_start = cursor + 1
            cursor += 1

        contexts[item_start] = not malformed and all(
            active
            and opening_brace is not None
            and _rust_inner_module_cfgs_are_active(source, opening_brace, target)
            for active, opening_brace in module_stack
        )

    return contexts
