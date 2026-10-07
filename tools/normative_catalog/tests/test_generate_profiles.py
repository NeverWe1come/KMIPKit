"""Tests for catalog-derived, fail-closed client profile metadata."""

from __future__ import annotations

import unittest

from tools.normative_catalog.generate_profiles import (
    ProfileGenerationError,
    compute_profile_closure,
    rust_string_literal,
)


def closure_catalog() -> dict[str, object]:
    return {
        "source_clauses": [
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001", "source_id": "profiles", "section": "6.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1.1-001", "source_id": "profiles", "section": "6.1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1-001", "source_id": "spec", "section": "1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1.1-001", "source_id": "spec", "section": "1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-2-001", "source_id": "spec", "section": "2"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001", "source_id": "profiles", "section": "3.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1.1-001", "source_id": "profiles", "section": "3.1.1"},
        ],
        "normative_inclusions": [
            {
                "inclusion_id": "KMIPKIT-INCLUSION-001",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001",
                "included_source_id": "spec",
                "included_section": None,
            },
            {
                "inclusion_id": "KMIPKIT-INCLUSION-002",
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            },
        ],
        "requirements": [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-2-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-2-001"],
                "role": "server",
                "scope_state": "server_only",
            },
            {
                "requirement_id": "KMIPKIT-REQ-PROF-3.1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-3.1.1-001"],
                "role": "client",
                "scope_state": "profile_conditional",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            },
        ],
    }


class ProfileMetadataGeneratorTests(unittest.TestCase):
    def test_closure_expands_whole_sources_sections_and_applicable_requirements(self) -> None:
        catalog = closure_catalog()

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertEqual(
            closure.source_clause_ids,
            (
                "KMIPKIT-CLAUSE-PROF-3.1-001",
                "KMIPKIT-CLAUSE-PROF-3.1.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1.1-001",
                "KMIPKIT-CLAUSE-SPEC-1-001",
                "KMIPKIT-CLAUSE-SPEC-1.1-001",
                "KMIPKIT-CLAUSE-SPEC-2-001",
            ),
        )
        self.assertEqual(
            closure.requirement_ids,
            (
                "KMIPKIT-REQ-PROF-3.1.1-001",
                "KMIPKIT-REQ-SPEC-1-001",
                "KMIPKIT-REQ-SPEC-1.1-001",
            ),
        )

    def test_conditional_requirement_applies_only_to_profiles_linked_by_the_catalog(self) -> None:
        catalog = closure_catalog()

        baseline = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )
        unrelated = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-UNRELATED-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-PROF-3.1.1-001", baseline.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-PROF-3.1.1-001", unrelated.requirement_ids)

    def test_closure_rejects_recursive_normative_inclusion_cycles(self) -> None:
        catalog = closure_catalog()
        inclusions = catalog["normative_inclusions"]
        assert isinstance(inclusions, list)
        inclusions.append(
            {
                "inclusion_id": "KMIPKIT-INCLUSION-CYCLE",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001",
                "included_source_id": "profiles",
                "included_section": "6.1",
            }
        )

        with self.assertRaisesRegex(ProfileGenerationError, "cyclic normative inclusion"):
            compute_profile_closure(
                catalog,
                ["KMIPKIT-CLAUSE-PROF-6.1-001"],
                profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
            )

    def test_rust_string_literal_escapes_syntax_and_control_characters(self) -> None:
        value = 'quote" slash\\ line\n tab\t control\x01 snowman ☃'

        self.assertEqual(
            rust_string_literal(value),
            '"quote\\" slash\\\\ line\\n tab\\t control\\u{1} snowman ☃"',
        )


if __name__ == "__main__":
    unittest.main()
