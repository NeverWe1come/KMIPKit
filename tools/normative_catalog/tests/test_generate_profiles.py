"""Tests for catalog-derived, fail-closed client profile metadata."""

from __future__ import annotations

import io
import json
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import call, patch

from tools.normative_catalog.generate_profiles import (
    CATALOG_PATH,
    MAX_CATALOG_BYTES,
    ProfileGenerationError,
    load_profile_targets,
    compute_profile_closure,
    main as generate_profiles_main,
    parse_profile_targets,
    ProfileTargets,
    render_profiles_source,
    rust_string_literal,
)
from tools.normative_catalog.validate import load_validated_catalog


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
    def test_https_client_inherits_baseline_basic_authentication_root(self) -> None:
        """Profiles v2.1 §§6.4 and 6.1 make §3.1 inherited, not direct."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-HTTPS-CLIENT"
        profile = next(item for item in catalog["profiles"] if item["profile_id"] == profile_id)
        expected = next(item for item in oracle["profiles"] if item["profile_id"] == profile_id)
        inherited_clause_id = "KMIPKIT-CLAUSE-PROF-3.1-001"
        reviewed_exclusion = next(
            item
            for item in expected["direct_root_exclusions"]
            if item["source_clause_id"] == inherited_clause_id
        )

        self.assertEqual(reviewed_exclusion["expected_disposition"], "inherited_not_direct")
        self.assertNotIn(inherited_clause_id, profile["direct_source_clause_ids"])
        self.assertIn(inherited_clause_id, profile["source_clause_ids"])

    def test_tape_library_includes_normatively_referenced_mandatory_test_cases(self) -> None:
        """Profiles v2.1 §6.28 item 7 incorporates the §5.12.6 test cases."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT"
        profile = next(item for item in catalog["profiles"] if item["profile_id"] == profile_id)
        expected = next(item for item in oracle["profiles"] if item["profile_id"] == profile_id)
        test_case_clauses = {
            "KMIPKIT-CLAUSE-PROF-5.12.6.1-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-002",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-003",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-004",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-002",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-003",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-004",
        }
        inclusion_id = "KMIPKIT-INCLUSION-PROF-TAPE-LIBRARY-TEST-CASES"
        inclusion = next(
            (item for item in catalog["normative_inclusions"] if item["inclusion_id"] == inclusion_id),
            None,
        )
        oracle_edge = next(
            (item for item in oracle["provenance"]["inclusion_edges"] if item["inclusion_id"] == inclusion_id),
            None,
        )

        self.assertEqual(
            inclusion,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.28-007",
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.12.6",
            },
        )
        self.assertIn("5.12.6", expected["direct_profiles_sections"])
        self.assertIn(inclusion_id, expected["normative_inclusion_ids"])
        self.assertEqual(
            oracle_edge,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.28-007",
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.12.6",
                "source_locator": "Profiles v2.1 §6.28 item 7",
            },
        )
        self.assertTrue(test_case_clauses.issubset(expected["source_clause_ids"]))
        self.assertFalse(test_case_clauses.intersection(profile["direct_source_clause_ids"]))
        self.assertTrue(test_case_clauses.issubset(profile["source_clause_ids"]))
        self.assertTrue(
            test_case_clauses.issubset(
                {
                    item["source_clause_id"]
                    for item in expected["direct_root_exclusions"]
                    if item["expected_disposition"] == "included_not_direct"
                }
            )
        )
        reviewed_client_requirements = {
            "KMIPKIT-REQ-PROF-5.12.6.1-001",
            "KMIPKIT-REQ-PROF-5.12.6.2-002",
            "KMIPKIT-REQ-PROF-5.12.6.2-003",
            "KMIPKIT-REQ-PROF-5.12.6.2-004",
            "KMIPKIT-REQ-PROF-5.12.6.3-001",
            "KMIPKIT-REQ-PROF-5.12.6.3-002",
            "KMIPKIT-REQ-PROF-5.12.6.3-003",
        }
        self.assertTrue(reviewed_client_requirements.issubset(expected["requirement_ids_audited_subset"]))
        self.assertEqual(
            reviewed_client_requirements,
            {item for item in profile["requirement_ids"] if item in reviewed_client_requirements},
        )
        cleanup_requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-PROF-5.12.6.3-004"
        )
        self.assertEqual(cleanup_requirement["role"], "unclear")
        self.assertEqual(cleanup_requirement["scope_state"], "unclear")
        self.assertNotIn(cleanup_requirement["requirement_id"], profile["requirement_ids"])

    def test_unresolved_actor_requirement_is_not_a_client_profile_obligation(self) -> None:
        """Specification v2.1 §4.57 does not assign the Active predicates to a client."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        clause_id = "KMIPKIT-CLAUSE-SPEC-4.57-003"
        requirement_id = "KMIPKIT-REQ-SPEC-4.57-003-004"
        requirement = next(item for item in catalog["requirements"] if item["requirement_id"] == requirement_id)
        review = next(item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == clause_id)
        crosswalk = [item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id]
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(requirement["role"], "unspecified")
        self.assertEqual(requirement["scope_state"], "unclear")
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertEqual(crosswalk, [])
        self.assertEqual(len(oracle["profiles"]), 18)
        for expected in oracle["profiles"]:
            with self.subTest(profile_id=expected["profile_id"]):
                actual = catalog_profiles[expected["profile_id"]]
                self.assertIn(clause_id, expected["source_clause_ids"])
                self.assertIn(clause_id, actual["source_clause_ids"])
                self.assertNotIn(requirement_id, expected["requirement_ids_audited_subset"])
                self.assertNotIn(requirement_id, actual["requirement_ids"])

    def test_oracle_audits_all_mapped_client_1_0_requirements_in_each_profile_closure(self) -> None:
        """Every source-reviewed client_1_0 requirement is represented in the profile audit subset."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        mappings = oracle["requirement_crosswalk"]

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            closure_clause_ids = set(profile["source_clause_ids"])
            expected_requirement_ids = {
                mapping["requirement_id"]
                for mapping in mappings
                if mapping.get("source_clause_id") in closure_clause_ids
                and mapping.get("applies_to") in {"all_client_profiles", profile_id}
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") == "client_1_0"
            }
            missing_requirement_ids = expected_requirement_ids - set(profile["requirement_ids_audited_subset"])

            with self.subTest(profile_id=profile_id):
                self.assertFalse(
                    missing_requirement_ids,
                    f"missing {len(missing_requirement_ids)} source-mapped client_1_0 IDs: "
                    f"{sorted(missing_requirement_ids)[:8]}",
                )

    def test_oracle_audits_all_mapped_conditional_requirements_in_each_profile_closure(self) -> None:
        """Every source-reviewed conditional client requirement is scoped to its applicable profile."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        mappings = oracle["requirement_crosswalk"]
        applicable_scopes = {
            "profile_conditional",
            "client_1_0 + server_only (split/shared)",
        }

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            closure_clause_ids = set(profile["source_clause_ids"])
            expected_requirement_ids = {
                mapping["requirement_id"]
                for mapping in mappings
                if mapping.get("source_clause_id") in closure_clause_ids
                and mapping.get("applies_to") in {"all_client_profiles", profile_id}
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") in applicable_scopes
            }
            missing_requirement_ids = expected_requirement_ids - set(profile["requirement_ids_audited_subset"])

            with self.subTest(profile_id=profile_id):
                self.assertFalse(
                    missing_requirement_ids,
                    f"missing {len(missing_requirement_ids)} mapped conditional IDs: "
                    f"{sorted(missing_requirement_ids)[:8]}",
                )

    def test_profile_oracle_requirement_arrays_match_independent_source_reviews(self) -> None:
        """Full profile requirement sets come from reviewed source records, not the catalog."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        source_clauses = {row["clause_id"]: row for row in oracle["source_clause_crosswalk"]}

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            source_clause_ids = set(profile["source_clause_ids"])
            included_sections = {
                (source_clauses[clause_id]["source_id"], source_clauses[clause_id]["section"])
                for clause_id in source_clause_ids
            }
            expected_requirement_ids: set[str] = set()

            for review in oracle["source_requirement_reviews"]:
                if (
                    review["source_clause_id"] in source_clause_ids
                    and review.get("applies_to") in {"all_client_profiles", profile_id}
                ):
                    expected_requirement_ids.update(review["expected_client_requirement_ids"])

            for mapping in oracle["requirement_crosswalk"]:
                if (
                    mapping.get("applies_to") in {"all_client_profiles", profile_id}
                    and mapping.get("role") in {"client", "both"}
                    and mapping.get("scope_state") in {"client_1_0", "profile_conditional"}
                    and (
                        mapping.get("source_clause_id") in source_clause_ids
                        or (mapping.get("source_id"), mapping.get("source_section")) in included_sections
                    )
                ):
                    expected_requirement_ids.add(mapping["requirement_id"])

            for gap in oracle["source_only_requirement_gaps"]:
                if (
                    gap.get("candidate_requirement_id") is not None
                    and gap.get("scope_disposition") in {"client_1_0", "profile_conditional"}
                    and (
                        gap.get("source_clause_id") in source_clause_ids
                        or (gap.get("source_id"), gap.get("source_section")) in included_sections
                    )
                ):
                    expected_requirement_ids.add(gap["candidate_requirement_id"])

            expected = sorted(expected_requirement_ids)
            with self.subTest(profile_id=profile_id):
                self.assertEqual(
                    profile.get("requirement_ids"),
                    expected,
                    "full requirement closure must be independently enumerated from the source review fixture",
                )
                self.assertEqual(profile["requirement_ids_audited_subset"], expected)

    def test_catalog_scopes_source_reviewed_conditional_requirements_to_profiles(self) -> None:
        """Catalog profile IDs and closures match source-reviewed conditional applicability."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}
        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        mappings = oracle["requirement_crosswalk"]

        for expected_profile in oracle["profiles"]:
            profile_id = expected_profile["profile_id"]
            source_clauses = set(expected_profile["source_clause_ids"])
            applicable_mappings = [
                mapping
                for mapping in mappings
                if mapping.get("source_clause_id") in source_clauses
                and mapping.get("applies_to") in {"all_client_profiles", profile_id}
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") == "profile_conditional"
            ]
            actual_profile = catalog_profiles[profile_id]

            for mapping in applicable_mappings:
                requirement_id = mapping["requirement_id"]
                requirement = requirements[requirement_id]
                with self.subTest(profile_id=profile_id, requirement_id=requirement_id):
                    self.assertIn(profile_id, requirement["profile_ids"])
                    self.assertIn(requirement_id, actual_profile["requirement_ids"])

    def test_client_profiles_audit_the_inherited_baseline_authentication_requirement(self) -> None:
        """Client profile dependencies inherit Baseline Client's §3.1 requirement mapping."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1-001"
        requirement_id = "KMIPKIT-REQ-PROF-3.1-001"
        basic_auth_inclusion = "KMIPKIT-INCLUSION-PROF-BASELINE-BASIC-AUTH"
        baseline_profile = "KMIPKIT-PROFILE-BASELINE-CLIENT"

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            with self.subTest(profile_id=profile_id):
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertIn(basic_auth_inclusion, profile["normative_inclusion_ids"])
                if profile_id != baseline_profile:
                    self.assertIn(baseline_profile, profile["dependency_profile_ids"])
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing inherited requirement {requirement_id}",
                )

    def test_oracle_audits_the_required_derive_key_table_body_row(self) -> None:
        """Specification v2.1 Table 205's required Attributes row is in every client closure."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.14-007-001"
        gap_id = "KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED"
        mapping = next(item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id)
        clauses = {item["clause_id"]: item for item in oracle["source_clause_crosswalk"]}
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(mapping["source_gap_id"], gap_id)
        self.assertEqual(mapping["source_section"], "6.1.14")
        self.assertEqual(mapping["applies_to"], "all_client_profiles")
        self.assertEqual(mapping["scope_state"], "client_1_0")
        self.assertIsNone(mapping.get("source_clause_id"))
        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            included_spec_sections = {
                clauses[clause_id]["section"]
                for clause_id in profile["source_clause_ids"]
                if clauses[clause_id]["source_id"] == "KMIPKIT-SRC-spec"
            }
            with self.subTest(profile_id=profile_id):
                self.assertIn("6.1.14", included_spec_sections)
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing source-reviewed table row {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in catalog_profiles[profile_id]["requirement_ids"],
                    f"catalog omits source-reviewed table row {requirement_id}",
                )

    def test_oracle_crosswalks_the_attestation_indicator_client_requirement(self) -> None:
        """Specification v2.1 §9.3's client SHALL has one exact source mapping."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        requirement_id = "KMIPKIT-REQ-SPEC-9.3-001-001"
        clause_id = "KMIPKIT-CLAUSE-SPEC-9.3-001"
        mapping = next(
            (item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id),
            None,
        )
        review = next(item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == clause_id)
        obligation = next(item for item in review["source_obligations"] if item["candidate_requirement_id"] == requirement_id)
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
        self.assertEqual(obligation["actor"], "client")
        self.assertEqual(obligation["scope_disposition"], "client_1_0")
        self.assertEqual(
            mapping,
            {
                "requirement_id": requirement_id,
                "source_clause_id": clause_id,
                "source_id": "KMIPKIT-SRC-spec",
                "source_section": "9.3",
                "source_locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keyword": "SHALL",
                "role": "client",
                "scope_state": "client_1_0",
                "applies_to": "all_client_profiles",
                "review_note": "Specification v2.1 §9.3 lines 43612–43619; this crosswalk row covers the client's Attestation Capable Indicator obligation only. The separate server response obligation remains server-only.",
                "source_obligation_order": 1,
                "source_actor": "client",
            },
        )
        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            with self.subTest(profile_id=profile_id):
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing reviewed client requirement {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in catalog_profiles[profile_id]["requirement_ids"],
                    f"catalog omits reviewed client requirement {requirement_id}",
                )

    def test_source_oracle_covers_all_profiles_and_expected_scope_counts(self) -> None:
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles = oracle["profiles"]

        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(profiles), 18)
        self.assertEqual(
            sum(profile["applicability"] in {"client_1_0", "conditional"} for profile in profiles),
            15,
        )
        self.assertEqual(
            sum(profile["applicability"] == "out_of_scope" for profile in profiles),
            3,
        )
        baseline = next(
            profile for profile in profiles
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        mandatory_unavailable = [
            case for case in baseline["test_cases"]
            if case["fixture_status"] == "unavailable"
        ]
        self.assertEqual(len(baseline["test_cases"]), 16)
        self.assertEqual(len(mandatory_unavailable), 16)
        self.assertTrue(all("-M-" in case["official_case_id"] for case in baseline["test_cases"]))

    def test_checked_in_client_profile_closures_match_current_scope(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))

        violations = []
        for profile in catalog["profiles"]:
            if profile["role"] not in {"client", "both"}:
                continue
            closure = compute_profile_closure(
                catalog,
                profile["direct_source_clause_ids"],
                profile_id=profile["profile_id"],
            )
            computed_ids = set(closure.requirement_ids)
            stored_ids = set(profile["requirement_ids"])
            stale_ids = sorted(stored_ids - computed_ids)
            missing_ids = sorted(computed_ids - stored_ids)
            if (
                profile["source_clause_ids"] != list(closure.source_clause_ids)
                or stale_ids
                or missing_ids
            ):
                violations.append(
                    f"{profile['profile_id']}: stale={stale_ids[:3]}, missing={missing_ids[:3]}"
                )

        self.assertEqual(violations, [], "client profile closures differ from current catalog scope")

    def test_client_profile_closure_keeps_split_client_obligations_without_server_obligations(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        baseline = next(
            profile for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        closure = compute_profile_closure(
            catalog,
            baseline["direct_source_clause_ids"],
            profile_id=baseline["profile_id"],
        )

        requirements = {
            row["requirement_id"]: row for row in catalog["requirements"]
        }
        source_reviews = {
            row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]
        }
        # The pinned source review assigns client requests and server responses
        # separate requirement IDs, even when they share a source clause.
        for clause_id, client_id, server_id in (
            (
                "KMIPKIT-CLAUSE-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-002",
            ),
            (
                "KMIPKIT-CLAUSE-SPEC-6.1.7-005",
                "KMIPKIT-REQ-SPEC-6.1.7-005",
                "KMIPKIT-REQ-SPEC-6.1.7-006",
            ),
        ):
            with self.subTest(requirement_id=client_id):
                source_review = source_reviews[clause_id]
                client_requirement = requirements[client_id]
                server_requirement = requirements[server_id]
                source_atoms = {
                    atom.get("candidate_requirement_id"): atom
                    for atom in source_review["source_obligations"]
                }

                self.assertEqual(client_requirement["role"], "client")
                self.assertEqual(client_requirement["scope_state"], "client_1_0")
                self.assertIn(client_id, source_review["expected_client_requirement_ids"])
                self.assertIn(client_id, closure.requirement_ids)
                self.assertEqual(source_atoms[client_id]["actor"], "client")
                self.assertEqual(source_atoms[client_id]["scope_disposition"], "client_1_0")

                self.assertEqual(server_requirement["role"], "server")
                self.assertEqual(server_requirement["scope_state"], "server_only")
                self.assertNotIn(server_id, source_review["expected_client_requirement_ids"])
                self.assertNotIn(server_id, closure.requirement_ids)
                self.assertEqual(source_atoms[server_id]["actor"], "server")
                self.assertEqual(source_atoms[server_id]["scope_disposition"], "server_only")

    def test_generated_client_requirement_metadata_covers_profile_closures(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)
        requirement_block = generated.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1]
        requirement_block = requirement_block.split("\n];", 1)[0]
        metadata_ids = set(re.findall(r'requirement_id: "([^"]+)"', requirement_block))
        closed_requirement_ids = {
            requirement_id
            for profile in catalog["profiles"]
            if profile["role"] in {"client", "both"}
            for requirement_id in profile["requirement_ids"]
        }

        missing_metadata = sorted(closed_requirement_ids - metadata_ids)
        self.assertEqual(
            missing_metadata,
            [],
            "every requirement in a client profile closure needs generated runtime metadata",
        )

    def test_profile_closure_generator_matches_the_pinned_source_oracle(self) -> None:
        """Compare closure expansion with static source-derived arrays for all 18 clients."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        oracle_profiles = {profile["profile_id"]: profile for profile in oracle["profiles"]}
        catalog_profiles = {
            profile["profile_id"]: profile
            for profile in catalog["profiles"]
            if profile["role"] == "client"
        }

        self.assertEqual(
            oracle["oracle_status"],
            "complete",
            "profile closure comparison requires the completed pinned-source oracle",
        )
        self.assertIs(
            oracle["counts"]["requirement_crosswalk_complete"],
            False,
            "source-accounted exclusions remain distinct from accepted direct mappings",
        )
        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(oracle_profiles), 18)
        profiles_without_full_requirements = sorted(
            profile_id
            for profile_id, expected in oracle_profiles.items()
            if "requirement_ids" not in expected
        )
        self.assertEqual(
            profiles_without_full_requirements,
            [],
            "every profile must carry a complete source-derived requirement set",
        )

        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        for profile_id, expected in oracle_profiles.items():
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                self.assertEqual(
                    profile["direct_source_clause_ids"],
                    expected["direct_source_clause_ids"],
                    f"{profile_id} direct source roots differ from pinned Profiles section headings",
                )
                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile_id,
                )
                self.assertEqual(
                    closure.source_clause_ids,
                    tuple(expected["source_clause_ids"]),
                    f"{profile_id} clause closure differs from the pinned-source oracle",
                )
                self.assertEqual(
                    set(closure.requirement_ids),
                    set(expected["requirement_ids"]),
                    f"{profile_id} generated requirement closure differs from the complete pinned-source oracle",
                )

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

    def test_table_body_requirements_follow_included_section_boundaries(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1.1-TABLE-OPTIONAL",
                "source_clause_ids": [],
                "source_refs": [{"source_id": "spec", "section": "1.1"}],
                "source_force": "optional_table_body_value",
                "role": "client",
                "scope_state": "client_1_0",
            }
        )
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1-TABLE-OPTIONAL",
                "source_clause_ids": [],
                "source_refs": [{"source_id": "spec", "section": "1"}],
                "source_force": "optional_table_body_value",
                "role": "client",
                "scope_state": "client_1_0",
            }
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-SPEC-1.1-TABLE-OPTIONAL", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-1-TABLE-OPTIONAL", closure.requirement_ids)

    def test_closure_does_not_expand_informative_references(self) -> None:
        catalog = closure_catalog()
        catalog["normative_inclusions"] = catalog["normative_inclusions"][:1]
        catalog["informative_references"] = [
            {
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            }
        ]

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-CLAUSE-SPEC-1-001", closure.source_clause_ids)
        self.assertNotIn("KMIPKIT-CLAUSE-PROF-3.1-001", closure.source_clause_ids)

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

    def test_closure_reports_unresolved_client_scope_without_applying_it(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].extend(
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1.1-UNCLEAR",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
                    "role": "client",
                    "scope_state": "unclear",
                },
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-2-CONDITIONAL",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-2-001"],
                    "role": "both",
                    "scope_state": "profile_conditional",
                    "profile_ids": [],
                },
            ]
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-SPEC-1.1-UNCLEAR", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-2-CONDITIONAL", closure.requirement_ids)
        blockers = getattr(closure, "unresolved_scope_obligations", ())
        self.assertEqual(
            [
                (item.requirement_id, item.source_clause_id, item.scope_state)
                for item in blockers
            ],
            [
                ("KMIPKIT-REQ-SPEC-1.1-UNCLEAR", "KMIPKIT-CLAUSE-SPEC-1.1-001", "unclear"),
                (
                    "KMIPKIT-REQ-SPEC-2-CONDITIONAL",
                    "KMIPKIT-CLAUSE-SPEC-2-001",
                    "profile_conditional",
                ),
            ],
        )

    def test_closure_fails_closed_for_unresolved_endpoint_attribution(self) -> None:
        catalog = closure_catalog()
        catalog["source_clauses"].append(
            {
                "clause_id": "KMIPKIT-CLAUSE-PROF-4.1.2-001",
                "source_id": "profiles",
                "section": "4.1.2",
            }
        )
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-PROF-4.1.2-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-4.1.2-001"],
                "role": "unclear",
                "direction": "unclear",
                "scope_state": "unclear",
            }
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-4.1.2-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-PROF-4.1.2-001", closure.requirement_ids)
        self.assertEqual(
            [
                (item.requirement_id, item.source_clause_id, item.scope_state)
                for item in closure.unresolved_scope_obligations
            ],
            [
                (
                    "KMIPKIT-REQ-PROF-4.1.2-001",
                    "KMIPKIT-CLAUSE-PROF-4.1.2-001",
                    "unclear",
                )
            ],
        )

    def test_closure_excludes_server_only_obligations_from_client_scope(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].extend(
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1-SERVER-ROLE",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                    "role": "server",
                    "scope_state": "server_only",
                },
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1-SERVER-SCOPE",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                    "role": "client",
                    "scope_state": "server_only",
                },
            ]
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-SPEC-1-SERVER-ROLE", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-1-SERVER-SCOPE", closure.requirement_ids)
        self.assertEqual(closure.unresolved_scope_obligations, ())

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

    def test_profile_generator_rejects_invalid_and_over_limit_catalog_inputs(self) -> None:
        for label, raw_catalog in (
            ("invalid JSON", b"{"),
            ("over-limit input", b"x" * (MAX_CATALOG_BYTES + 1)),
        ):
            with self.subTest(label=label):
                reads: list[tuple[Path, str, int]] = []

                def read_catalog(repository: Path, path: str, *, max_bytes: int) -> bytes:
                    reads.append((repository, path, max_bytes))
                    if path != CATALOG_PATH:
                        raise AssertionError(f"generator read {path} after catalog validation should have failed")
                    return raw_catalog

                stderr = io.StringIO()
                with (
                    patch("tools.normative_catalog.generate_profiles.safe_read_bytes", side_effect=read_catalog),
                    patch("sys.stderr", new=stderr),
                ):
                    result = generate_profiles_main(["--check", "--repo-root", str(ROOT)])

                self.assertEqual(result, 1)
                self.assertIn("profile metadata generation failed:", stderr.getvalue())
                self.assertEqual(reads, [(ROOT.resolve(), CATALOG_PATH, MAX_CATALOG_BYTES)])

    def test_rust_string_literal_escapes_syntax_and_control_characters(self) -> None:
        value = 'quote" slash\\ line\n tab\t control\x01 snowman ☃'

        self.assertEqual(
            rust_string_literal(value),
            '"quote\\" slash\\\\ line\\n tab\\t control\\u{1} snowman ☃"',
        )


    @unittest.skipUnless(shutil.which("rustc") and shutil.which("rustfmt"), "Rust toolchain is required")
    def test_generated_source_compiles_and_adversarial_metadata_round_trips(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        value = 'quote" slash\\ line\n carriage\r tab\t nul\x00 control\x01 DEL\x7f C1\x85 snowman ☃ emoji 🦄'
        baseline["name"] = value
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        generated = render_profiles_source(catalog, targets)
        expected_bytes = ", ".join(str(byte) for byte in value.encode("utf-8"))
        generated += "\n".join([
            "#[cfg(test)]",
            "mod generated_metadata_literal_test {",
            "    use super::CLIENT_PROFILES;",
            "",
            "    #[test]",
            "    fn source_metadata_is_data_after_compilation() {",
            "        let baseline = CLIENT_PROFILES",
            "            .iter()",
            "            .find(|profile| profile.profile_id == \"KMIPKIT-PROFILE-BASELINE-CLIENT\")",
            "            .expect(\"the fixture catalog includes Baseline Client\");",
            f"        assert_eq!(baseline.name.as_bytes(), &[{expected_bytes}]);",
            "    }",
            "}",
            "",
        ])
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "profiles_generated.rs"
            binary = Path(directory) / "profiles_generated_tests.exe"
            source.write_text(generated, encoding="utf-8")
            compile_result = subprocess.run(
                ["rustc", "--edition=2024", "--test", str(source), "-o", str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(compile_result.returncode, 0, compile_result.stderr)
            run_result = subprocess.run(
                [str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(run_result.returncode, 0, run_result.stdout + run_result.stderr)


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

    def test_rejects_missing_fields_wrong_types_empty_ids_and_invalid_json(self) -> None:
        invalid_documents = (
            {},
            {"release_line": "1.0", "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"]},
            self.manifest | {"release_line": 1},
            self.manifest | {"profile_ids": "KMIPKIT-PROFILE-BASELINE-CLIENT"},
            self.manifest | {"profile_ids": [None]},
            self.manifest | {"profile_ids": [""]},
            self.manifest | {"decision_ref": []},
            self.manifest | {"decision_ref": ""},
        )
        for document in invalid_documents:
            with self.subTest(document=document), self.assertRaises(ProfileGenerationError):
                self.parse(document)

        with self.assertRaises(ProfileGenerationError):
            self.parse(b'{"release_line":')
        oversized = json.dumps(self.manifest).encode("utf-8")
        oversized += b" " * (65_537 - len(oversized))
        with self.assertRaisesRegex(ProfileGenerationError, "size limit"):
            self.parse(oversized)

    def test_fixed_loader_rejects_a_symlinked_target_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            target = repository / "specification" / "catalog" / "profile-targets.json"
            target.parent.mkdir(parents=True)
            external = repository / "external-targets.json"
            external.write_text(json.dumps(self.manifest), encoding="utf-8")
            try:
                target.symlink_to(external)
            except OSError as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_rejects_a_junctioned_target_manifest_directory(self) -> None:
        if not shutil.which("cmd.exe"):
            self.skipTest("Windows cmd.exe is required to create a directory junction")

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory) / "repository"
            specification = repository / "specification"
            specification.mkdir(parents=True)
            external = Path(directory) / "external-catalog"
            external.mkdir()
            (external / "profile-targets.json").write_text(
                json.dumps(self.manifest), encoding="utf-8"
            )
            junction = specification / "catalog"
            mklink_command = (
                "mklink /J "
                + subprocess.list2cmdline([str(junction), str(external)])
            )
            result = subprocess.run(
                ["cmd.exe", "/d", "/c", mklink_command],
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(
                result.returncode,
                0,
                msg=f"could not create test junction: {result.stdout}{result.stderr}",
            )

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_uses_a_64_kib_read_bound(self) -> None:
        with patch(
            "tools.normative_catalog.generate_profiles.safe_read_bytes",
            side_effect=lambda _root, path, *, max_bytes: (
                json.dumps(self.manifest).encode("utf-8")
                if path == "specification/catalog/profile-targets.json"
                else b"**FR-003**"
            ),
        ) as safe_read:
            load_profile_targets(ROOT, self.catalog)

        self.assertIn(
            call(ROOT, "specification/catalog/profile-targets.json", max_bytes=65_536),
            safe_read.call_args_list,
        )

    def test_generated_metadata_preserves_closure_evidence_and_separate_target_state(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertEqual(generated, render_profiles_source(catalog, targets))
        self.assertEqual(generated.count("\n    GeneratedClientProfile {"), 18)
        self.assertIn('claim_state: "not_claimed"', generated)
        self.assertIn("selected_target: true", generated)
        self.assertIn('fixture_availability: "unavailable"', generated)
        self.assertNotIn('"KMIPKIT-REQ-SPEC-2.1-001"', generated)
        self.assertIn('"KMIPKIT-REQ-PROF-3.1.1-004"', generated)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_target_and_profile_prose_do_not_change_requirement_evidence_metadata(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        alternate_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-AES-XTS-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        baseline_output = render_profiles_source(catalog, baseline_target)
        alternate_target_output = render_profiles_source(catalog, alternate_target)
        changed_prose = json.loads(json.dumps(catalog))
        baseline = next(
            profile
            for profile in changed_prose["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        baseline["name"] = "This prose does not define executable profile rules"
        changed_prose_output = render_profiles_source(changed_prose, baseline_target)

        def requirement_metadata(source: str) -> str:
            return source.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1].split(
                "pub const CLIENT_PROFILES:", 1
            )[0]

        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(alternate_target_output))
        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(changed_prose_output))
        self.assertIn("selected_target: true", baseline_output)
        self.assertIn("selected_target: true", alternate_target_output)
        self.assertNotIn("ProfileRuleMapping", baseline_output)
        self.assertNotIn("ProfileRuleMapping", alternate_target_output)

    def test_generated_requirement_metadata_preserves_exact_normative_keywords(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertTrue("pub struct GeneratedClientRequirement" in generated)
        self.assertTrue("pub const CLIENT_REQUIREMENTS" in generated)
        expected_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        for keyword in expected_keywords:
            with self.subTest(keyword=keyword):
                self.assertTrue(f'source_keyword: Some("{keyword}")' in generated)

        table_requirements = [
            requirement
            for requirement in catalog["requirements"]
            if requirement.get("source_force") in {"required_table_body_value", "optional_table_body_value"}
            and requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        ]
        for requirement in table_requirements:
            marker = f'requirement_id: "{requirement["requirement_id"]}"'
            row_start = generated.index(marker)
            row_end = generated.index("    },", row_start)
            row = generated[row_start:row_end]
            with self.subTest(requirement_id=requirement["requirement_id"]):
                self.assertIn("source_keyword: None", row)
                self.assertIn(f'source_force: Some("{requirement["source_force"]}")', row)
                self.assertIn(f'source_gap_id: Some("{requirement["source_gap_id"]}")', row)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_generated_requirements_classify_keyword_specific_verification_categories(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        catalog = json.loads(json.dumps(catalog))
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        expected_categories = {
            "MUST": "RequiredBehavior",
            "SHALL": "RequiredBehavior",
            "REQUIRED": "RequiredBehavior",
            "MUST NOT": "ProhibitedBehavior",
            "SHALL NOT": "ProhibitedBehavior",
            "SHOULD": "RecommendedBehaviorOrDeviation",
            "RECOMMENDED": "RecommendedBehaviorOrDeviation",
            "SHOULD NOT": "DiscouragedAbsenceOrDeviation",
            "MAY": "OptionalCapability",
            "OPTIONAL": "OptionalCapability",
        }
        observed_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        template = next(
            requirement
            for requirement in catalog["requirements"]
            if requirement["role"] == "client" and requirement["scope_state"] == "client_1_0"
        )
        for index, keyword in enumerate(sorted(set(expected_categories) - observed_keywords), start=1):
            synthetic_requirement = template.copy()
            synthetic_requirement["requirement_id"] = f"KMIPKIT-REQ-SPEC-TEST-KEYWORD-{index:02}"
            synthetic_requirement["source_keyword"] = keyword
            catalog["requirements"].append(synthetic_requirement)

        generated = render_profiles_source(catalog, targets)
        blocks = re.findall(
            r"    GeneratedClientRequirement \{\n(.*?)\n    \},",
            generated,
            flags=re.DOTALL,
        )
        actual_categories: dict[str, str] = {}
        for block in blocks:
            requirement_id = re.search(r'requirement_id: "([^"]+)"', block)
            category = re.search(
                r"verification_category: GeneratedVerificationCategory::([A-Za-z]+)",
                block,
            )
            if requirement_id is not None and category is not None:
                actual_categories[requirement_id.group(1)] = category.group(1)

        scoped_requirements = {
            requirement["requirement_id"]: expected_categories[requirement["source_keyword"]]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        observed_scoped_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        actual_categories = {
            requirement_id: category
            for requirement_id, category in actual_categories.items()
            if requirement_id in scoped_requirements
        }

        self.assertEqual(observed_scoped_keywords, set(expected_categories))
        self.assertEqual(actual_categories, scoped_requirements)
        self.assertEqual(
            set(actual_categories.values()),
            set(expected_categories.values()),
        )


if __name__ == "__main__":
    unittest.main()
