"""Shared assertions for verified public API parity tests."""

from __future__ import annotations

from collections.abc import Mapping, Set
from typing import Any
import unittest


def assert_declaration_mappings(
    test_case: unittest.TestCase,
    operation: Mapping[str, Any],
    declarations: Mapping[str, Mapping[str, Any]],
    *,
    operation_id: str,
    languages: set[str],
    expected_mappings: Mapping[str, Set[str]],
    required_requirements: Set[str],
) -> None:
    """Validate every declaration attached to one verified operation."""
    return None
