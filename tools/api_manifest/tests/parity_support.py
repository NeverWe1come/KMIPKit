"""Shared assertions for verified public API parity tests."""

from __future__ import annotations

import unittest
from collections.abc import Mapping, Set
from typing import Any


def assert_declaration_mappings(
    test_case: unittest.TestCase,
    operation: Mapping[str, Any],
    declarations: Mapping[str, Mapping[str, Any]],
    *,
    operation_id: str,
    languages: set[str],
    expected_mappings: Mapping[str, Set[str]],
    required_requirements: Set[str],
    expected_approval_source: str | None = None,
    required_parity_evidence: Set[str] = frozenset(),
) -> tuple[dict[str, set[str]], set[str]]:
    """Validate every declaration attached to one verified operation."""
    operation_declarations = operation["declarations"]
    test_case.assertEqual(languages, set(operation_declarations))
    actual_mappings = {language: set() for language in languages}
    linked_requirements: set[str] = set()

    for language, declaration_ids in operation_declarations.items():
        test_case.assertTrue(declaration_ids, f"{operation_id} has no {language} declarations")
        for declaration_id in declaration_ids:
            test_case.assertIn(declaration_id, declarations)
            declaration = declarations[declaration_id]
            test_case.assertEqual(operation_id, declaration["capabilityId"])
            test_case.assertEqual("verified", declaration["declarationState"])
            test_case.assertEqual("four-language", declaration["parityScope"])
            test_case.assertEqual(languages, set(declaration["languageMappings"]))
            if expected_approval_source is not None:
                test_case.assertEqual(expected_approval_source, declaration["approvalSource"]["path"])
                test_case.assertEqual("Approved", declaration["approvalSource"]["status"])
            if required_parity_evidence:
                test_case.assertTrue(
                    required_parity_evidence <= set(declaration["parityEvidenceIds"]),
                    declaration_id,
                )
            actual_mappings[language].add(declaration["languageMappings"][language]["name"])
            linked_requirements.update(declaration["requirementIds"])

    for language, expected in expected_mappings.items():
        test_case.assertTrue(expected <= actual_mappings[language], language)
    test_case.assertTrue(required_requirements <= linked_requirements)
    return actual_mappings, linked_requirements
