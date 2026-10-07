"""Tests for catalog-derived, fail-closed client profile metadata."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.generate_profiles import (
    ProfileGenerationError,
    load_profile_targets,
    compute_profile_closure,
    parse_profile_targets,
    rust_string_literal,
)


ROOT = Path(__file__).resolve().parents[3]


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


class ProfileTargetManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = {
            "profiles": [
                {
                    "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                    "role": "client",
                    "applicability": "client_1_0",
                },
                {
                    "profile_id": "KMIPKIT-PROFILE-SERVER-ONLY",
                    "role": "server",
                    "applicability": "server_only",
                },
            ]
        }
        self.manifest = {
            "release_line": "1.0",
            "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            "decision_ref": "KMIPKIT-0010/FR-003",
        }

    def parse(self, manifest: dict[str, object] | bytes | str):
        if isinstance(manifest, dict):
            raw = json.dumps(manifest).encode("utf-8")
        elif isinstance(manifest, str):
            raw = manifest.encode("utf-8")
        else:
            raw = manifest
        return parse_profile_targets(raw, self.catalog, ROOT)

    def test_loads_fixed_baseline_target_and_keeps_selection_separate(self) -> None:
        targets = self.parse(self.manifest)

        self.assertEqual(targets.release_line, "1.0")
        self.assertEqual(targets.profile_ids, ("KMIPKIT-PROFILE-BASELINE-CLIENT",))
        self.assertEqual(targets.decision_ref, "KMIPKIT-0010/FR-003")

    def test_rejects_duplicate_keys_and_invalid_utf8(self) -> None:
        duplicate_keys = (
            b'{"release_line":"1.0","release_line":"1.0",'
            b'"profile_ids":["KMIPKIT-PROFILE-BASELINE-CLIENT"],'
            b'"decision_ref":"KMIPKIT-0010/FR-003"}'
        )
        with self.assertRaises(ProfileGenerationError):
            self.parse(duplicate_keys)
        with self.assertRaises(ProfileGenerationError):
            self.parse(b"\xff")

    def test_rejects_wrong_release_unknown_profile_and_unapproved_role(self) -> None:
        for overrides in (
            {"release_line": "2.0"},
            {"profile_ids": ["KMIPKIT-PROFILE-UNKNOWN"]},
            {"profile_ids": ["KMIPKIT-PROFILE-SERVER-ONLY"]},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_rejects_empty_duplicate_and_unresolved_decision_targets(self) -> None:
        for overrides in (
            {"profile_ids": []},
            {"profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"] * 2},
            {"decision_ref": "KMIPKIT-0010/FR-999"},
            {"extra": "unknown"},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_fixed_loader_uses_a_64_kib_read_bound(self) -> None:
        with patch(
            "tools.normative_catalog.generate_profiles.safe_read_bytes",
            return_value=json.dumps(self.manifest).encode("utf-8"),
        ) as safe_read:
            load_profile_targets(ROOT, self.catalog)

        self.assertEqual(safe_read.call_args.kwargs["max_bytes"], 65_536)


if __name__ == "__main__":
    unittest.main()
