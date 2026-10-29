"""Tests for deterministic, injection-safe coverage reports."""

from __future__ import annotations

import csv
import json
import locale
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.report import _clause_section_rows, render_report, write_report
from tools.normative_catalog.tests.test_validate import minimal_catalog

ROOT = Path(__file__).resolve().parents[3]


def report_catalog() -> dict[str, object]:
    catalog = minimal_catalog()
    catalog["source_clauses"] = [
        {
            "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "source_id": "KMIPKIT-SRC-spec",
            "section": "8.1",
            "locator": {"ordinal": 1, "block_kind": "paragraph"},
            "source_keywords": ["MUST"],
            "disposition": "requirement",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
            "exclusion_rationale": None,
        }
    ]
    catalog["requirements"] = [
        {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
            "source_keyword": "MUST",
            "normative_strength": "mandatory",
            "subject": "client",
            "summary": "Client obligation",
            "role": "client",
            "direction": "client_to_server",
            "condition": None,
            "scope_state": "client_1_0",
            "element_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
            "negative_verification_required": False,
            "decision_id": None,
            "status": "unassigned",
            "review_note": None,
        }
    ]
    return catalog


class CoverageReportTests(unittest.TestCase):
    def test_profile_review_notes_do_not_duplicate_volatile_baseline_counts(self) -> None:
        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        notes = [
            requirement["review_note"]
            for requirement in catalog["requirements"]
            if isinstance(requirement.get("review_note"), str)
        ]

        stale_count_notes = [
            note
            for note in notes
            if re.search(
                r"Baseline (?:now )?has \d+ of \d+ typed mappings with category-appropriate evidence",
                note,
            )
        ]

        self.assertEqual([], stale_count_notes)

    def test_message_envelope_summary_preserves_request_payload_requirement(self) -> None:
        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        requirement = next(
            record
            for record in catalog["requirements"]
            if record["requirement_id"] == "KMIPKIT-REQ-SPEC-8-001"
        )

        self.assertEqual(
            "A protocol message has a header and one or more Batch Items. Request Payload is required; Response Payload is required unless the Result Status is Failure; Message Extensions are optional.",
            requirement["summary"],
        )
        self.assertIn("§8.3 Table 396", requirement["review_note"])
        self.assertIn("§8.6 Table 399", requirement["review_note"])
        self.assertNotIn("optional request/response payload", requirement["review_note"])

    @staticmethod
    def _complete_profile_readiness_fixture() -> tuple[
        dict[str, object],
        dict[str, object],
        dict[str, object],
        list[dict[str, object]],
        str,
        str,
    ]:
        from scripts import ci_summary

        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["implementation_refs"] = ["crates/kmipkit-protocol/src/profile.rs"]
        requirement["verification_refs"] = ["tools/normative_catalog/tests/test_report.py"]
        requirement_id = requirement["requirement_id"]
        clause_id = "KMIPKIT-CLAUSE-SPEC-8.1-001"
        test_id = "KMIPKIT-TEST-SPEC-READINESS-001"
        profile_a_id = "KMIPKIT-PROFILE-READINESS-A"
        profile_b_id = "KMIPKIT-PROFILE-READINESS-B"
        catalog["profiles"] = [
            {
                "profile_id": profile_a_id,
                "name": "Readiness A",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "evidence_complete",
                "source_clause_ids": [clause_id],
                "requirement_ids": [requirement_id],
                "test_case_ids": [test_id],
                "dependency_profile_ids": [profile_b_id],
            },
            {
                "profile_id": profile_b_id,
                "name": "Readiness B",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "evidence_complete",
                "source_clause_ids": [clause_id],
                "requirement_ids": [requirement_id],
                "test_case_ids": [test_id],
                "dependency_profile_ids": [],
            },
        ]
        catalog["test_cases"] = [
            {
                "test_id": test_id,
                "source_id": "KMIPKIT-SRC-testcases",
                "mandatory_status": "optional",
                "fixture_availability": "available",
            }
        ]
        mappings = [
            {
                "requirement_id": requirement_id,
                "source_clause_id": clause_id,
                "verification_category": "RequiredBehavior",
                "evaluator_present": True,
                "test_kind": "PositiveBehavior",
                "test_id": "profile_validation::readiness_evidence_is_complete",
            }
        ]
        context = {
            "event_name": "push",
            "repository": ci_summary.CANONICAL_REPOSITORY,
            "ref": ci_summary.RELEASE_REF,
            "ref_protected": True,
            "sha": "a" * 40,
            "workflow_ref": ci_summary.EXPECTED_WORKFLOW_REF,
            "run_id": "123456789",
            "run_attempt": "1",
        }
        needs = {job_id: {"result": "success"} for job_id in ci_summary.EVIDENCE_REQUIRED_JOB_IDS}
        branch_protection = {
            "protected": True,
            "required_pull_request_reviews": {
                "required_approving_review_count": 0,
                "dismiss_stale_reviews": True,
                "require_code_owner_reviews": False,
                "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
            },
            "enforce_admins": {"enabled": True},
            "allow_force_pushes": {"enabled": False},
            "allow_deletions": {"enabled": False},
        }
        manifest = ci_summary.build_evidence_manifest(
            event_name="push",
            context=context,
            needs=needs,
            branch_protection=branch_protection,
        )
        return catalog, manifest, context, mappings, profile_a_id, profile_b_id

    def test_confined_rust_path_accepts_a_resolved_repository_alias(self) -> None:
        from tools.normative_catalog.report import _confined_rust_path

        with tempfile.TemporaryDirectory() as directory:
            temporary_root = Path(directory)
            repository = temporary_root / "repository"
            repository.mkdir()
            relative = Path("crates/kmipkit-protocol/src/lib.rs")
            target = repository / relative
            target.parent.mkdir(parents=True)
            target.write_text("", encoding="utf-8")

            alias = temporary_root / "repository-alias"
            if os.name == "nt":
                subprocess.run(
                    ["cmd.exe", "/c", "mklink", "/J", str(alias), str(repository)],
                    check=True,
                    capture_output=True,
                    text=True,
                )
            else:
                alias.symlink_to(repository, target_is_directory=True)

            try:
                self.assertEqual(
                    repository.resolve() / relative,
                    _confined_rust_path(alias, alias / relative),
                )
            finally:
                if os.name == "nt":
                    alias.rmdir()
                else:
                    alias.unlink()

    @staticmethod
    def _write_profile_module_tree(repository: Path, *, include_test_file: bool) -> None:
        crate = repository / "crates/kmipkit-protocol"
        (crate / "src").mkdir(parents=True, exist_ok=True)
        (crate / "Cargo.toml").write_text(
            '[package]\nname = "fixture-protocol"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (crate / "src/lib.rs").write_text("#[cfg(test)]\nmod profile;\n", encoding="utf-8")
        if include_test_file:
            with (crate / "src/profile.rs").open("a", encoding="utf-8") as profile_source:
                profile_source.write(
                    '\n#[cfg(test)]\n#[path = "../tests/support/profile_unit.rs"]\nmod tests;\n'
                )

    def test_typed_rule_registry_reader_parses_const_mapping_constructors(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = '''
const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    typed_rule_mapping(
        "KMIPKIT-REQ-PROF-3.1-001",
        "KMIPKIT-CLAUSE-PROF-3.1-001",
        None,
        GeneratedVerificationCategory::RequiredBehavior,
        None,
        &[rule_test_evidence(
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::requires_tls",
            RuleTestKind::PositiveBehavior,
        )],
        &[rule_constraint("transport_security", "tls")],
    ),
];
'''

        self.assertEqual(
            _parse_typed_rule_registry(source),
            [
                {
                    "requirement_id": "KMIPKIT-REQ-PROF-3.1-001",
                    "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001",
                    "source_gap_id": None,
                    "verification_category": "RequiredBehavior",
                    "evaluator_present": False,
                    "transport_only": True,
                    "configuration_only": True,
                    "test_kind": "PositiveBehavior",
                    "test_id": (
                        "crates/kmipkit-protocol/tests/support/profile_unit.rs::requires_tls"
                    ),
                }
            ],
        )

    def test_typed_rule_registry_reader_parses_tls_and_tls13_mappings(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1-001",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_tls_for_basic_authentication_suite_transport",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-3.1.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_tls_1_3_for_basic_authentication_suite_transport",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "transport_security",
            value: "tls13",
        }],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                "mod tests {\n"
                "#[test]\nfn profile_requires_tls_for_basic_authentication_suite_transport() {}\n"
                "#[test]\nfn profile_recommends_tls_1_3_for_basic_authentication_suite_transport() {}\n"
                "#[cfg(target_pointer_width = \"32\")]\n"
                "#[test]\nfn unrelated_32_bit_only_test() {}\n"
                "}\n",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            self.assertEqual(
                [
                    {
                        "requirement_id": "KMIPKIT-REQ-PROF-3.1-001",
                        "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001",
                        "source_gap_id": None,
                        "verification_category": "RequiredBehavior",
                        "evaluator_present": False,
                        "transport_only": True,
                        "configuration_only": True,
                        "test_kind": "PositiveBehavior",
                        "test_id": (
                            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                            "profile_requires_tls_for_basic_authentication_suite_transport"
                        ),
                    },
                    {
                        "requirement_id": "KMIPKIT-REQ-PROF-3.1.1-002",
                        "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1.1-002",
                        "source_gap_id": None,
                        "verification_category": "RecommendedBehaviorOrDeviation",
                        "evaluator_present": False,
                        "transport_only": True,
                        "configuration_only": True,
                        "test_kind": "RecommendedBehavior",
                        "test_id": (
                            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                            "profile_recommends_tls_1_3_for_basic_authentication_suite_transport"
                        ),
                    }
                ],
                load_typed_rule_mappings(repository),
            )

    def test_typed_rule_registry_reader_parses_client_profile_gate_mappings(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-14.1-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-14.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::selection_resolves_kmip_client_conformance_only_for_a_complete_profile_closure",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_closure",
            value: "all_normative_statements",
        }],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                "#[test]\n"
                "fn selection_resolves_kmip_client_conformance_only_for_a_complete_profile_closure() {}\n",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            self.assertEqual(
                [
                    {
                        "requirement_id": "KMIPKIT-REQ-SPEC-14.1-002",
                        "source_clause_id": "KMIPKIT-CLAUSE-SPEC-14.1-002",
                        "source_gap_id": None,
                        "verification_category": "RequiredBehavior",
                        "evaluator_present": False,
                        "transport_only": False,
                        "configuration_only": True,
                        "test_kind": "PositiveBehavior",
                        "test_id": (
                            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                            "selection_resolves_kmip_client_conformance_only_for_a_complete_profile_closure"
                        ),
                    }
                ],
                load_typed_rule_mappings(repository),
            )

    def test_typed_rule_registry_accepts_caller_owned_distinct_attribute_values(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_distinct_multi_instance_attribute_values_as_recommended_behavior",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "caller_supplies_distinct_values_for_multi_instance_attributes",
        }],
    },
];
"""

        self.assertEqual(
            _parse_typed_rule_registry(source),
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-4-001-002",
                    "source_clause_id": "KMIPKIT-CLAUSE-SPEC-4-001",
                    "source_gap_id": None,
                    "verification_category": "RecommendedBehaviorOrDeviation",
                    "evaluator_present": False,
                    "transport_only": False,
                    "configuration_only": True,
                    "test_kind": "RecommendedBehavior",
                    "test_id": "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_distinct_multi_instance_attribute_values_as_recommended_behavior",
                }
            ],
        )

    def test_scoped_recommendation_decision_keeps_category_test_reference(self) -> None:
        from tools.normative_catalog.report import _profile_requirement_coverage

        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement.update(
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-4-001-002",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-4-001"],
                "source_keyword": "SHOULD",
                "normative_strength": "recommended",
                "role": "client",
                "scope_state": "mixed",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
                "decision_id": "KMIPKIT-DEC-023",
                "status": "scoped_verified",
            }
        )
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "role": "client",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-4-001-002"],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-4-001"],
            }
        ]
        catalog["decisions"] = [
            {
                "decision_id": "KMIPKIT-DEC-023",
                "status": "accepted",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-4-001-002"],
            }
        ]
        mapping = {
            "requirement_id": "KMIPKIT-REQ-SPEC-4-001-002",
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-4-001",
            "verification_category": "RecommendedBehaviorOrDeviation",
            "evaluator_present": False,
            "transport_only": False,
            "configuration_only": True,
            "test_kind": "RecommendedBehavior",
            "test_id": "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_distinct_multi_instance_attribute_values_as_recommended_behavior",
        }

        rows, _ = _profile_requirement_coverage(catalog, [mapping])

        self.assertEqual("Category test reference: present", rows[0][5])

    def test_typed_rule_registry_accepts_baseline_conformance_and_capability_constraints(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-PROF-6.1-004",
        source_clause_id: "KMIPKIT-CLAUSE-PROF-6.1-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[],
        constraints: &[
            RuleConstraint {
                key: "client_profile_conformance_evidence",
                value: "one_or_more_baseline_mandatory_tests_5_1_3_or_5_6_3",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "recover_before_use_caller_controlled",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "name_unique_within_target_server_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "derive_key_source_usage_mask_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "target_usage_mask_allows_requested_function_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "target_usage_mask_excludes_requested_function_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "lease_not_expired_or_renewed_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "usage_allocation_obtained_before_protection_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "usage_allocation_requested_within_object_date_window_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "usage_allocation_remains_available_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "lease_not_expired_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "recertify_once_per_existing_certificate_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "rekey_once_per_existing_key_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "rekey_key_pair_once_per_existing_pair_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "reprovision_when_credentials_are_required_caller_directed",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "iv_counter_nonce_when_required_by_wrapping_method_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "wrapped_key_value_ttlv_when_encoding_option_absent_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "optional_failure_response_payload_and_message_extensions",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "request_response_fields_follow_kmip_specified_order",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "request_response_include_one_or_more_batch_items",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "countdown_timer_relative_dates_from_server_timestamp_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "activation_date_reached_before_use_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "no_cryptographic_use_before_activation_date_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "activation_date_immutable_after_pre_active_until_destruction_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "deactivation_date_exceptional_use_requires_special_permission_caller_asserted",
            },
            RuleConstraint {
                key: "client_profile_capability",
                value: "deactivation_date_change_only_in_pre_active_or_active_caller_asserted",
            },
        ],
    },
];
"""

        try:
            parsed = _parse_typed_rule_registry(source)
        except ValueError as error:
            self.fail(f"the report parser rejected a known caller-owned capability: {error}")

        self.assertEqual(
            parsed,
            [
                {
                    "requirement_id": "KMIPKIT-REQ-PROF-6.1-004",
                    "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.1-004",
                    "source_gap_id": None,
                    "verification_category": "RequiredBehavior",
                    "evaluator_present": False,
                    "transport_only": False,
                    "configuration_only": True,
                    "test_kind": None,
                    "test_id": None,
                }
            ],
        )

    def test_typed_rule_registry_rejects_server_only_pre_active_use_as_client_capability(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.57-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.57-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::ProhibitedBehavior,
        evaluator: None,
        test_evidence: &[],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "pre_active_object_not_used_for_cryptographic_purposes_caller_asserted",
        }],
    },
];
"""

        with self.assertRaisesRegex(ValueError, "unsupported typed profile rule mapping"):
            _parse_typed_rule_registry(source)

    def test_typed_rule_registry_accepts_archived_retrieval_workflow_capability(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.28-004-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RequiredBehavior,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges",
            kind: RuleTestKind::PositiveBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "archived_locate_recover_get_workflow_caller_directed",
        }],
    },
];
"""

        self.assertEqual(
            _parse_typed_rule_registry(source),
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-6.1.28-004-001",
                    "source_clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
                    "source_gap_id": None,
                    "verification_category": "RequiredBehavior",
                    "evaluator_present": False,
                    "transport_only": False,
                    "configuration_only": True,
                    "test_kind": "PositiveBehavior",
                    "test_id": "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges",
                }
            ],
        )

    def test_typed_rule_registry_accepts_single_key_usage_recommendation(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-4.34-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4.34-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_single_key_usage_recommendation_as_caller_owned_behavior",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "single_cryptographic_usage_per_key_object_caller_asserted",
        }],
    },
];
"""

        mappings = _parse_typed_rule_registry(source)

        self.assertEqual(len(mappings), 1)
        self.assertEqual(
            mappings[0]["requirement_id"], "KMIPKIT-REQ-SPEC-4.34-001-002"
        )
        self.assertEqual(
            mappings[0]["verification_category"], "RecommendedBehaviorOrDeviation"
        )
        self.assertTrue(mappings[0]["configuration_only"])
        self.assertEqual(mappings[0]["test_kind"], "RecommendedBehavior")

    def test_typed_rule_registry_accepts_final_baseline_caller_owned_mappings(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        expected = [
            (
                "KMIPKIT-REQ-SPEC-4.38-003",
                "ProhibitedBehavior",
                "NegativeBehavior",
                "original_creation_date_immutable_after_set_caller_asserted",
            ),
            (
                "KMIPKIT-REQ-SPEC-4.57-005-002",
                "RecommendedBehaviorOrDeviation",
                "RecommendedBehavior",
                "compromised_object_protected_information_only_trusted_client_caller_asserted",
            ),
            (
                "KMIPKIT-REQ-SPEC-4.59-002",
                "ProhibitedBehavior",
                "NegativeBehavior",
                "usage_limits_total_immutable_after_protection_begins_caller_asserted",
            ),
            (
                "KMIPKIT-REQ-SPEC-4.59-003-001",
                "ProhibitedBehavior",
                "NegativeBehavior",
                "usage_limits_total_immutable_after_count_allocation_caller_asserted",
            ),
            (
                "KMIPKIT-REQ-SPEC-6.1.14-002",
                "OptionalCapability",
                "OptionalCapabilityUsable",
                "derive_key_multiple_inputs_caller_selected",
            ),
            (
                "KMIPKIT-REQ-SPEC-6.1.34-001-003",
                "OptionalCapability",
                "OptionalCapabilityUsable",
                "modify_attribute_existing_instance_named",
            ),
        ]

        entries = [
            f'''    TypedRuleMapping {{
        requirement_id: "{requirement_id}",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-4-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::{category},
        evaluator: None,
        test_evidence: &[RuleTestEvidence {{
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::example",
            kind: RuleTestKind::{test_kind},
        }}],
        constraints: &[RuleConstraint {{
            key: "client_profile_capability",
            value: "{capability}",
        }}],
    }}'''
            for requirement_id, category, test_kind, capability in expected
        ]
        source = (
            "const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[\n"
            + ",\n".join(entries)
            + "\n];\n"
        )
        mappings = _parse_typed_rule_registry(source)

        for requirement_id, category, test_kind, _ in expected:
            with self.subTest(requirement_id=requirement_id):
                matches = [
                    mapping
                    for mapping in mappings
                    if mapping["requirement_id"] == requirement_id
                ]
                self.assertTrue(matches)
                self.assertTrue(
                    all(mapping["configuration_only"] for mapping in matches)
                )
                self.assertTrue(
                    all(
                        mapping["verification_category"] == category
                        for mapping in matches
                    )
                )
                self.assertTrue(
                    all(mapping["test_kind"] == test_kind for mapping in matches)
                )

    def test_typed_rule_registry_accepts_caller_directed_recover_capabilities(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.42-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.42-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/recover_execution_tests.rs::caller_issues_poll_after_recover_pending_with_the_returned_correlation",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recover_poll_caller_directed",
        }],
    },
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.42-001-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.42-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: None,
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs::caller_directs_archived_locate_recover_then_get_as_three_exchanges",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[RuleConstraint {
            key: "client_profile_capability",
            value: "recover_get_caller_directed",
        }],
    },
];
"""

        mappings = _parse_typed_rule_registry(source)
        self.assertEqual(
            [mapping["requirement_id"] for mapping in mappings],
            [
                "KMIPKIT-REQ-SPEC-6.1.42-001-001",
                "KMIPKIT-REQ-SPEC-6.1.42-001-002",
            ],
        )
        self.assertTrue(all(mapping["configuration_only"] for mapping in mappings))
        self.assertEqual(
            [mapping["test_kind"] for mapping in mappings],
            ["OptionalCapabilityUsable", "OptionalCapabilityUsable"],
        )

    def test_typed_rule_registry_reader_accepts_rustfmt_wrapped_evaluator(self) -> None:
        from tools.normative_catalog.report import _parse_typed_rule_registry

        source = """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.7-002-002",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.7-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: Some(
            client_check_batch_error_continuation_is_stop_or_undo,
        ),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_stop_or_undo_when_check_is_present",
            kind: RuleTestKind::RecommendedBehavior,
        }],
        constraints: &[],
    },
];
"""

        self.assertEqual(
            _parse_typed_rule_registry(source),
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-6.1.7-002-002",
                    "source_clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.7-002",
                    "source_gap_id": None,
                    "verification_category": "RecommendedBehaviorOrDeviation",
                    "evaluator_present": True,
                    "transport_only": False,
                    "configuration_only": False,
                    "test_kind": "RecommendedBehavior",
                    "test_id": (
                        "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
                        "profile_recommends_stop_or_undo_when_check_is_present"
                    ),
                }
            ],
        )

    def test_typed_rule_registry_rejects_unresolvable_test_evidence(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn existing_test() {}\n", encoding="utf-8")
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::missing_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_a_conditionally_compiled_target_test(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::conditional_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                '#[cfg(target_pointer_width = "32")]\n'
                "#[test]\nfn conditional_test() {}\n",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_an_inactive_inline_test_module(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::conditional_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                "#[cfg(any())]\nmod tests {\n#[test]\nfn conditional_test() {}\n}\n",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_an_inactive_inner_cfg_test_module(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::conditional_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                "mod tests {\n#![cfg(any())]\n#[test]\nfn conditional_test() {}\n}\n",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_test_evidence_found_only_in_rust_text(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                'const EXAMPLE: &str = r#"\n#[test]\nfn ghost_test() {}\n"#;\n',
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_test_in_unincluded_rust_file(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=False)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_test_at_wrong_inline_module_path(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/src/profile/tests.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];

mod unrelated {
    mod tests;
}
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/src/profile/tests.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=False)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_test_declaration_inside_macro_tokens(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text(
                """macro_rules! generated_test {
    () => {
        #[test]
        fn ghost_test() {}
    };
}
""",
                encoding="utf-8",
            )
            self._write_profile_module_tree(repository, include_test_file=True)

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_library_tests_disabled_by_cargo(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=True)
            (repository / "crates/kmipkit-protocol/Cargo.toml").write_text(
                '[package]\nname = "fixture-protocol"\nversion = "0.1.0"\n'
                "\n[lib]\ntest = false\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_implicit_library_when_autolib_is_disabled(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=True)
            (repository / "crates/kmipkit-protocol/Cargo.toml").write_text(
                '[package]\nname = "fixture-protocol"\nversion = "0.1.0"\nautolib = false\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_rejects_tests_from_a_virtual_workspace_root(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=True)
            (repository / "crates/kmipkit-protocol/Cargo.toml").write_text(
                "[workspace]\nmembers = []\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_fails_closed_on_invalid_cargo_manifest(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(validate_discover_versions_rule),
        test_evidence: &[RuleTestEvidence {
            test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::ghost_test",
            kind: RuleTestKind::OptionalCapabilityUsable,
        }],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )
            test_path = repository / "crates/kmipkit-protocol/tests/support/profile_unit.rs"
            test_path.parent.mkdir(parents=True)
            test_path.write_text("#[test]\nfn ghost_test() {}\n", encoding="utf-8")
            self._write_profile_module_tree(repository, include_test_file=True)
            (repository / "crates/kmipkit-protocol/Cargo.toml").write_text(
                "this is not valid TOML = [\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unresolvable typed profile rule test evidence"):
                load_typed_rule_mappings(repository)

    def test_typed_rule_registry_reader_rejects_executable_field_expressions(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            source_path = repository / "crates/kmipkit-protocol/src/profile.rs"
            source_path.parent.mkdir(parents=True)
            source_path.write_text(
                """const TYPED_RULE_MAPPINGS: &[TypedRuleMapping] = &[
    TypedRuleMapping {
        requirement_id: "KMIPKIT-REQ-SPEC-6.1.16-001-001",
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-6.1.16-001",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::OptionalCapability,
        evaluator: Some(run_unreviewed_macro!()),
        test_evidence: &[],
        constraints: &[],
    },
];
""",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "unsupported typed profile rule mapping"):
                load_typed_rule_mappings(repository)

    def test_profile_readiness_blocks_unresolved_scope_in_its_clause_closure(self) -> None:
        from tools.normative_catalog.report import evaluate_profile_readiness

        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "dependency_profile_ids": [],
                "test_case_ids": ["KMIPKIT-TEST-PROFILE-BASELINE"],
            }
        ]
        typed_rule_mapping = {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "verification_category": "RequiredBehavior",
            "evaluator_present": True,
            "test_kind": "PositiveBehavior",
            "test_id": "profile_validation::required_rule_enforces_clause",
        }
        resolved_readiness = evaluate_profile_readiness(
            catalog,
            "KMIPKIT-PROFILE-BASELINE",
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE"},
            typed_rule_mappings=[typed_rule_mapping],
            verification_manifest=None,
            current_context=None,
        )
        self.assertTrue(resolved_readiness["runtime_validation_available"])
        catalog["requirements"].extend(
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-8.1-UNCLEAR",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                    "role": "client",
                    "scope_state": "unclear",
                    "implementation_refs": [],
                    "verification_refs": [],
                },
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-8.1-CONDITIONAL",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                    "role": "both",
                    "scope_state": "profile_conditional",
                    "profile_ids": [],
                    "implementation_refs": [],
                    "verification_refs": [],
                },
            ]
        )

        readiness = evaluate_profile_readiness(
            catalog,
            "KMIPKIT-PROFILE-BASELINE",
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE"},
            typed_rule_mappings=[typed_rule_mapping],
            verification_manifest=None,
            current_context=None,
        )

        report = render_report(catalog, typed_rule_mappings=[typed_rule_mapping])
        self.assertIn(
            "unresolved applicability scope: KMIPKIT-REQ-SPEC-8.1-UNCLEAR",
            report,
        )
        self.assertTrue(any(
            blocker.startswith("unresolved applicability scope: KMIPKIT-REQ-SPEC-8.1-UNCLEAR at ")
            for blocker in readiness["blockers"]
        ))
        self.assertTrue(any(
            blocker.startswith("unresolved applicability scope: KMIPKIT-REQ-SPEC-8.1-CONDITIONAL at ")
            for blocker in readiness["blockers"]
        ))
        self.assertFalse(readiness["runtime_validation_available"])

    def test_profile_readiness_names_open_discrepancy_for_linked_requirement(self) -> None:
        from tools.normative_catalog.report import evaluate_profile_readiness

        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        profile_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        readiness = evaluate_profile_readiness(
            catalog,
            profile_id,
            selected_profile_ids={profile_id},
            typed_rule_mappings=[],
            verification_manifest=None,
            current_context=None,
        )

        self.assertIn(
            "unresolved source discrepancy: KMIPKIT-DISC-042 affects "
            "requirement KMIPKIT-REQ-SPEC-9.11-004-001",
            readiness["blockers"],
        )

    def test_profile_readiness_names_open_discrepancy_linked_to_profile(self) -> None:
        from tools.normative_catalog.report import evaluate_profile_readiness

        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        profile_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        discrepancy = next(
            row
            for row in catalog["discrepancies"]
            if row["discrepancy_id"] == "KMIPKIT-DISC-042"
        )
        discrepancy["affected_requirement_ids"] = []
        discrepancy["affected_profile_ids"] = [profile_id]

        readiness = evaluate_profile_readiness(
            catalog,
            profile_id,
            selected_profile_ids={profile_id},
            typed_rule_mappings=[],
            verification_manifest=None,
            current_context=None,
        )

        self.assertIn(
            f"unresolved source discrepancy: KMIPKIT-DISC-042 affects profile {profile_id}",
            readiness["blockers"],
        )

    def test_readiness_cli_reports_missing_current_run_manifest_without_publishing(self) -> None:
        from tools.normative_catalog import report as report_module

        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "summary.md"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_EVENT_NAME": "pull_request",
                "GITHUB_REPOSITORY": "NeverWe1come/KMIPKit",
                "GITHUB_REF": "refs/pull/42/merge",
                "GITHUB_REF_PROTECTED": "false",
                "GITHUB_SHA": "a" * 40,
                "GITHUB_WORKFLOW_REF": "NeverWe1come/KMIPKit/.github/workflows/ci.yml@refs/pull/42/merge",
                "GITHUB_RUN_ID": "123456789",
                "GITHUB_RUN_ATTEMPT": "1",
                "PROFILE_EVIDENCE_MANIFEST": "",
            }
            with patch.dict(os.environ, environment, clear=False):
                exit_code = report_module.main(["--readiness", "--repo-root", str(ROOT)])

            summary = summary_path.read_text(encoding="utf-8")

        self.assertEqual(0, exit_code)
        self.assertIn("Current profile readiness", summary)
        self.assertIn("current protected-release-push verification manifest is unavailable", summary)
        self.assertIn("Public claim: no", summary)
        self.assertNotIn("123456789", summary)

    def test_readiness_cli_consumes_the_same_run_manifest_ephemerally(self) -> None:
        from scripts.ci_summary import build_evidence_manifest
        from tools.normative_catalog import report as report_module

        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            check=True,
            cwd=ROOT,
            capture_output=True,
            text=True,
        ).stdout.strip()
        context = {
            "repository": "NeverWe1come/KMIPKit",
            "ref": "refs/heads/release/1.0.0",
            "ref_protected": True,
            "sha": head,
            "workflow_ref": "NeverWe1come/KMIPKit/.github/workflows/ci.yml@refs/heads/release/1.0.0",
            "run_id": "123456789",
            "run_attempt": "1",
        }
        needs = {job_id: {"result": "success"} for job_id in (
            "core",
            "script-contracts",
            "normative-inventory",
            "coverage",
            "coverage-gate",
            "dependency-policy",
        )}
        manifest = build_evidence_manifest(
            event_name="push",
            context=context,
            needs=needs,
            branch_protection={
                "protected": True,
                "required_pull_request_reviews": {
                    "required_approving_review_count": 0,
                    "dismiss_stale_reviews": True,
                    "require_code_owner_reviews": False,
                    "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
                },
                "enforce_admins": {"enabled": True},
                "allow_force_pushes": {"enabled": False},
                "allow_deletions": {"enabled": False},
            },
        )
        raw_branch_policy = {
            "required_pull_request_reviews": {
                "required_approving_review_count": 0,
                "dismiss_stale_reviews": True,
                "require_code_owner_reviews": False,
                "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
            },
            "enforce_admins": {"enabled": True},
            "allow_force_pushes": {"enabled": False},
            "allow_deletions": {"enabled": False},
        }
        api_responses = {
            f"https://api.github.com/repos/NeverWe1come/KMIPKit/actions/runs/{context['run_id']}": {
                "id": int(context["run_id"]),
                "repository": {"full_name": context["repository"]},
                "event": "push",
                "head_branch": "release/1.0.0",
                "head_sha": head,
                "run_attempt": 1,
                "path": ".github/workflows/ci.yml@refs/heads/release/1.0.0",
                "status": "in_progress",
            },
            f"https://api.github.com/repos/NeverWe1come/KMIPKit/actions/runs/{context['run_id']}/jobs?per_page=100": {
                "total_count": 7,
                "jobs": [
                    {"name": "Core / ubuntu-latest / Rust 1.94", "status": "completed", "conclusion": "success"},
                    {"name": "Script contracts / ubuntu-latest", "status": "completed", "conclusion": "success"},
                    {"name": "Normative inventory / immutable sources and coverage", "status": "completed", "conclusion": "success"},
                    {"name": "Coverage / ubuntu", "status": "completed", "conclusion": "success"},
                    {"name": "Coverage gate / selected scopes", "status": "completed", "conclusion": "success"},
                    {"name": "Dependency policy / pull request", "status": "completed", "conclusion": "success"},
                    {"name": "Profile evidence / protected release aggregate", "status": "in_progress", "conclusion": None},
                ],
            },
            "https://api.github.com/repos/NeverWe1come/KMIPKit/branches/release%2F1.0.0/protection": raw_branch_policy,
        }

        class FakeResponse:
            def __init__(self, body: bytes) -> None:
                self.body = body

            def __enter__(self) -> FakeResponse:
                return self

            def __exit__(self, *_args: object) -> None:
                return None

            def read(self, _limit: int = -1) -> bytes:
                return self.body

        def open_github_api(request: object, **_kwargs: object) -> FakeResponse:
            url = request.full_url
            return FakeResponse(json.dumps(api_responses[url]).encode("utf-8"))

        class FakeOpener:
            def open(self, request: object, **kwargs: object) -> FakeResponse:
                return open_github_api(request, **kwargs)

        with tempfile.TemporaryDirectory() as directory:
            summary_path = Path(directory) / "github-summary.md"
            environment = {
                "GITHUB_STEP_SUMMARY": str(summary_path),
                "GITHUB_ACTIONS": "true",
                "GITHUB_EVENT_NAME": "push",
                "GITHUB_REPOSITORY": context["repository"],
                "GITHUB_REF": context["ref"],
                "GITHUB_REF_PROTECTED": "true",
                "GITHUB_SHA": head,
                "GITHUB_WORKFLOW_REF": context["workflow_ref"],
                "GITHUB_RUN_ID": context["run_id"],
                "GITHUB_RUN_ATTEMPT": context["run_attempt"],
                "PROFILE_EVIDENCE_MANIFEST": json.dumps(manifest),
                "PROFILE_EVIDENCE_GITHUB_TOKEN": "actions-read-token",
                "PROFILE_EVIDENCE_BRANCH_PROTECTION_TOKEN": "administration-read-token",
            }
            with patch("urllib.request.build_opener", return_value=FakeOpener()) as api_client:
                with patch.dict(os.environ, environment, clear=False):
                    exit_code = report_module.main(["--readiness", "--repo-root", str(ROOT)])

                github_summary = summary_path.read_text(encoding="utf-8")
                self.assertEqual(3, api_client.call_count)

                plain_path_summary_path = Path(directory) / "plain-path-summary.md"
                environment["GITHUB_STEP_SUMMARY"] = str(plain_path_summary_path)
                api_responses[
                    f"https://api.github.com/repos/NeverWe1come/KMIPKit/actions/runs/{context['run_id']}"
                ]["path"] = ".github/workflows/ci.yml"
                with patch.dict(os.environ, environment, clear=False):
                    plain_path_exit_code = report_module.main(
                        ["--readiness", "--repo-root", str(ROOT)]
                    )
                plain_path_summary = plain_path_summary_path.read_text(encoding="utf-8")
                self.assertEqual(6, api_client.call_count)

                mismatched_summary_path = Path(directory) / "mismatched-summary.md"
                environment["GITHUB_STEP_SUMMARY"] = str(mismatched_summary_path)
                api_responses[
                    f"https://api.github.com/repos/NeverWe1come/KMIPKit/actions/runs/{context['run_id']}"
                ]["head_sha"] = "b" * 40
                with patch.dict(os.environ, environment, clear=False):
                    mismatched_exit_code = report_module.main(
                        ["--readiness", "--repo-root", str(ROOT)]
                    )
                mismatched_summary = mismatched_summary_path.read_text(encoding="utf-8")
                self.assertEqual(7, api_client.call_count)

                local_summary_path = Path(directory) / "local-summary.md"
                environment["GITHUB_STEP_SUMMARY"] = str(local_summary_path)
                environment["GITHUB_ACTIONS"] = "false"
                environment["PROFILE_EVIDENCE_GITHUB_TOKEN"] = ""
                environment["PROFILE_EVIDENCE_BRANCH_PROTECTION_TOKEN"] = ""
                with patch.dict(os.environ, environment, clear=False):
                    local_exit_code = report_module.main(["--readiness", "--repo-root", str(ROOT)])

                self.assertEqual(7, api_client.call_count)

            local_summary = local_summary_path.read_text(encoding="utf-8")

        self.assertEqual(0, exit_code)
        self.assertEqual(0, plain_path_exit_code)
        self.assertEqual(0, mismatched_exit_code)
        self.assertEqual(0, local_exit_code)
        self.assertIn("Current profile readiness", github_summary)
        self.assertNotIn("current verification manifest is invalid or does not match this run", github_summary)
        self.assertIn("Public claim: no", github_summary)
        self.assertNotIn("123456789", github_summary)
        self.assertNotIn(
            "current verification manifest is invalid or does not match this run",
            plain_path_summary,
        )
        self.assertIn(
            "current protected-release-push verification manifest is unavailable",
            mismatched_summary,
        )
        self.assertIn("current protected-release-push verification manifest is unavailable", local_summary)
        self.assertIn("Public claim: no", local_summary)
        self.assertNotIn("123456789", local_summary)

    def test_emits_fixed_sections_totals_and_unassigned_requirements(self) -> None:
        report = render_report(report_catalog())
        self.assertIn("## Source documents", report)
        self.assertIn("## Count reconciliation", report)
        self.assertIn("## Unassigned requirements", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("mandatory", report)
        self.assertTrue(report.endswith("\n"))

    def test_reports_source_clause_review_counts_per_section(self) -> None:
        report = render_report(report_catalog())

        self.assertIn("## Source clause review by section", report)
        self.assertIn("KMIPKIT-SRC-spec", report)
        self.assertIn("8.1", report)
        self.assertIn("requirement: 1", report)

    def test_numeric_section_sort_breaks_equivalent_value_ties_stably(self) -> None:
        clauses = [
            {"source_id": "source", "section": "1.1", "disposition": "requirement"},
            {"source_id": "source", "section": "1.01", "disposition": "reviewed_exclusion"},
        ]

        self.assertEqual(_clause_section_rows(clauses), _clause_section_rows(list(reversed(clauses))))

    def test_output_is_independent_of_record_order_and_contains_no_timestamp(self) -> None:
        first = report_catalog()
        second = report_catalog()
        second["sources"].reverse()
        second["source_clauses"].reverse()
        self.assertEqual(render_report(first), render_report(second))
        self.assertNotIn("Generated at", render_report(first))

    def test_output_is_independent_of_permutations_in_every_catalog_collection(self) -> None:
        first = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        second = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        for collection in second.values():
            if isinstance(collection, list):
                collection.reverse()

        self.assertEqual(render_report(first), render_report(second))

    def test_output_is_independent_of_process_locale(self) -> None:
        catalog = report_catalog()
        expected = render_report(catalog)
        with patch.object(locale, "strxfrm", side_effect=AssertionError("locale sort used")):
            self.assertEqual(render_report(catalog), expected)

    def test_report_surfaces_negative_verification_requirements(self) -> None:
        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["source_keyword"] = "MUST NOT"
        requirement["normative_strength"] = "prohibited"
        requirement["negative_verification_required"] = True
        catalog["source_clauses"][0]["source_keywords"] = ["MUST NOT"]

        report = render_report(catalog)

        self.assertIn("## Requirements needing negative verification", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("prohibited", report)
        self.assertIn("required", report)

    def test_report_lists_requirement_level_official_test_evidence_gaps(self) -> None:
        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["review_note"] = (
            "No requirement-specific official Test Cases ID is linked by the pinned sources."
        )

        report = render_report(catalog)

        self.assertIn("## Requirements without official test-case links", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("No requirement-specific official Test Cases ID", report)

    def test_reports_profile_states_missing_fixtures_and_open_discrepancies(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "applicability": "client_1_0",
                "claim_state": "evidence_incomplete",
                "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
                "test_case_ids": [],
            }
        ]
        catalog["test_cases"] = [
            {
                "test_id": "KMIPKIT-TEST-PROFILE-BASELINE-001",
                "official_case_id": "KMIP-TC-001",
                "fixture_availability": "unavailable",
                "fixture_path": None,
                "source_id": "KMIPKIT-SRC-testcases",
                "source_section": "1",
                "href": "../missing.xml",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE"],
                "requirement_ids": [],
                "element_ids": [],
            }
        ]
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-001",
                "summary": "Batch continuation wording conflict",
                "state": "open",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.5"}],
                "affected_requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "affected_element_ids": [],
                "affected_profile_ids": [],
                "affected_policy_ids": [],
            }
        ]
        report = render_report(catalog)
        self.assertIn("evidence\\_incomplete", report)
        self.assertIn("Profiles by applicability and claim state", report)
        self.assertIn("Test evidence by fixture availability", report)
        self.assertIn("unavailable", report)
        self.assertIn("KMIPKIT-DISC-001", report)
        self.assertIn("open", report)
        self.assertIn("blocked for affected records", report)

    def test_open_discrepancy_section_omits_resolved_rows_and_matches_count(self) -> None:
        catalog = report_catalog()
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-OPEN",
                "summary": "Unresolved wording conflict",
                "state": "open",
                "source_refs": [],
            },
            {
                "discrepancy_id": "KMIPKIT-DISC-RESOLVED",
                "summary": "Resolved wording conflict",
                "state": "resolved_by_approved_decision",
                "source_refs": [],
            },
        ]

        report = render_report(catalog)
        open_discrepancy_section = report.split("## Open discrepancies\n", 1)[1].split(
            "\n## Project policies", 1
        )[0]

        self.assertIn("| Open discrepancies | 1 |", report)
        self.assertIn("KMIPKIT-DISC-OPEN", open_discrepancy_section)
        self.assertNotIn("KMIPKIT-DISC-RESOLVED", open_discrepancy_section)

    def test_lists_unassigned_protocol_capabilities_with_direction_and_scope(self) -> None:
        catalog = report_catalog()
        catalog["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-TAG-420001",
                "kind": "tag",
                "name": "Activation Date",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.7"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "wire_value": "420001",
                "allocation": "assigned",
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            }
        ]
        report = render_report(catalog)
        self.assertIn("## Unassigned protocol elements", report)
        self.assertIn("| Element | Name | Kind | Direction | Scope | Wire value | Allocation | Source |", report)
        self.assertIn("KMIPKIT-ELEM-TAG-420001", report)
        self.assertIn(r"client\_to\_server", report)
        self.assertIn("| KMIPKIT-ELEM-TAG-420001 | Activation Date | tag |", report)
        self.assertIn(r"| client\_1\_0 | 420001 | assigned | KMIPKIT-SRC-spec §6.1.7 |", report)

    def test_reports_tag_allocation_counts_and_each_range(self) -> None:
        catalog = report_catalog()
        catalog["elements"] = [
            {"element_id": "tag-assigned", "kind": "tag", "allocation": "assigned"},
            {"element_id": "tag-reserved", "kind": "tag", "allocation": "reserved"},
        ]
        catalog["tag_ranges"] = [
            {
                "range_id": "KMIPKIT-RANGE-001",
                "value_range": "420001 - 4200FF",
                "allocation": "unused",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
        ]

        report = render_report(catalog)

        self.assertIn("## Tag allocation and ranges", report)
        self.assertIn("| assigned | 1 |", report)
        self.assertIn("| reserved | 1 |", report)
        self.assertIn("| KMIPKIT-RANGE-001 | 420001 - 4200FF | unused |", report)

    def test_profile_report_includes_traceability_and_transport_contract(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-5.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-PROF-5.1-001"],
                "dependency_profile_ids": ["KMIPKIT-PROFILE-OTHER"],
                "test_case_ids": ["KMIPKIT-TEST-PROF-5-1-1", "KMIPKIT-TEST-PROF-5-1-2"],
                "transport_requirements": ["TLS 1.3"],
                "encoding_requirements": ["TTLV"],
            }
        ]
        catalog["test_cases"] = [
            {
                "test_id": "KMIPKIT-TEST-PROF-5-1-1",
                "official_case_id": "BL-M-1-21",
                "mandatory_status": "mandatory",
            },
            {
                "test_id": "KMIPKIT-TEST-PROF-5-1-2",
                "official_case_id": "BL-O-1-21",
                "mandatory_status": "optional",
            },
        ]

        report = render_report(catalog)

        self.assertIn("Source clauses", report)
        self.assertIn("Requirements", report)
        self.assertIn("Dependencies", report)
        self.assertIn("Test cases", report)
        self.assertIn("Transport", report)
        self.assertIn("Encoding", report)
        self.assertIn("KMIPKIT-CLAUSE-PROF-5.1-001", report)
        self.assertIn("KMIPKIT-REQ-PROF-5.1-001", report)
        self.assertIn("KMIPKIT-PROFILE-OTHER", report)
        self.assertIn("KMIPKIT-TEST-PROF-5-1-1", report)
        self.assertIn("BL-M-1-21", report)
        self.assertIn("mandatory", report)
        self.assertIn("BL-O-1-21", report)
        self.assertIn("optional", report)
        self.assertIn("TLS 1.3", report)
        self.assertIn("TTLV", report)

    def test_profile_report_separates_selected_target_from_claim_state(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE",
                "name": "Baseline",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [],
                "source_clause_ids": [],
                "requirement_ids": [],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]

        report = render_report(catalog, selected_profile_ids={"KMIPKIT-PROFILE-BASELINE"})

        self.assertIn("Selected target", report)
        self.assertIn(r"not\_claimed", report)
        self.assertIn("selected", report)

    def test_profile_requirement_coverage_reports_missing_typed_rule_and_test(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "name": "Baseline Client",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "evidence_incomplete",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]

        report = render_report(
            catalog,
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE-CLIENT"},
            typed_rule_mappings=[],
        )

        self.assertIn("## Profile requirement rule coverage", report)
        self.assertIn("KMIPKIT-PROFILE-BASELINE-CLIENT", report)
        self.assertIn("KMIPKIT-REQ-SPEC-8.1-001", report)
        self.assertIn("KMIPKIT-CLAUSE-SPEC-8.1-001", report)
        self.assertIn("RequiredBehavior", report)
        self.assertIn("No typed rule mapping", report)
        self.assertIn("No category-appropriate rule test", report)
        self.assertIn("| selected | false | evidence\\_incomplete |", report)
        self.assertIn(r"evidence\_incomplete", report)
        self.assertIn("No public claim", report)

    def test_profile_requirement_coverage_links_typed_rule_to_category_test(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "name": "Baseline Client",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]
        mapping = {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "verification_category": "RequiredBehavior",
            "evaluator_present": True,
            "test_kind": "PositiveBehavior",
            "test_id": "profile_validation::required_rule_enforces_clause",
        }

        report = render_report(catalog, typed_rule_mappings=[mapping])

        self.assertIn("Typed rule mapping: present", report)
        self.assertIn("Category test reference: present", report)
        self.assertIn(r"profile\_validation::required\_rule\_enforces\_clause", report)
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", report)
        self.assertIn("current pass/fail is not evaluated", report)

    def test_profile_requirement_coverage_accepts_transport_only_mapping(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "name": "Baseline Client",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]
        mapping = {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "verification_category": "RequiredBehavior",
            "evaluator_present": False,
            "transport_only": True,
            "test_kind": "PositiveBehavior",
            "test_id": "profile_validation::configured_tls_transport_satisfies_rule",
        }

        report = render_report(catalog, typed_rule_mappings=[mapping])

        self.assertIn("Typed rule mapping: present", report)
        self.assertIn("Category test reference: present", report)
        self.assertIn(r"profile\_validation::configured\_tls\_transport\_satisfies\_rule", report)

    def test_profile_requirement_coverage_accepts_configuration_only_mapping(self) -> None:
        catalog = report_catalog()
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "name": "Baseline Client",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]
        mapping = {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "verification_category": "RequiredBehavior",
            "evaluator_present": False,
            "transport_only": False,
            "configuration_only": True,
            "test_kind": "PositiveBehavior",
            "test_id": "profile_validation::complete_profile_closure_is_required",
        }

        report = render_report(catalog, typed_rule_mappings=[mapping])

        self.assertIn("Typed rule mapping: present", report)
        self.assertIn("Category test reference: present", report)
        self.assertIn(r"profile\_validation::complete\_profile\_closure\_is\_required", report)

    def test_profile_requirement_coverage_accepts_approved_recommended_deviation(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = report_catalog()
        requirement = catalog["requirements"][0]
        requirement["source_keyword"] = "SHOULD"
        requirement["normative_strength"] = "recommended"
        requirement["decision_id"] = "KMIPKIT-DEC-PROFILE-001"
        requirement["status"] = "deviated"
        catalog["source_clauses"][0]["source_keywords"] = ["SHOULD"]
        catalog["decisions"] = [
            {
                "decision_id": "KMIPKIT-DEC-PROFILE-001",
                "status": "accepted",
                "requirement_ids": [requirement["requirement_id"]],
            }
        ]
        catalog["profiles"] = [
            {
                "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                "name": "Baseline Client",
                "role": "client",
                "applicability": "client_1_0",
                "claim_state": "not_claimed",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "requirement_ids": [requirement["requirement_id"]],
                "element_ids": [],
                "dependency_profile_ids": [],
                "test_case_ids": [],
                "transport_requirements": [],
                "encoding_requirements": [],
            }
        ]
        mapping = {
            "requirement_id": requirement["requirement_id"],
            "source_clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "verification_category": "RecommendedBehaviorOrDeviation",
            "evaluator_present": True,
            "test_kind": None,
            "test_id": None,
        }

        rows, runtime_availability = report_module._profile_requirement_coverage(
            catalog,
            [mapping],
        )

        self.assertEqual("Accepted deviation: KMIPKIT-DEC-PROFILE-001", rows[0][5])
        self.assertTrue(runtime_availability["KMIPKIT-PROFILE-BASELINE-CLIENT"])

        report = render_report(catalog, typed_rule_mappings=[mapping])
        baseline_blocker_row = next(
            row
            for row in report.splitlines()
            if row.startswith("| KMIPKIT-PROFILE-BASELINE-CLIENT |")
            and len(row.split("|")) == 9
        )
        self.assertNotIn("category test references missing: 1", baseline_blocker_row)
        self.assertIn("no stable-input blockers", baseline_blocker_row)

    def test_requirement_rule_coverage_shows_linked_certify_request_type_element(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))

        report = render_report(catalog, typed_rule_mappings=[])
        coverage_report = report.split("## Profile requirement rule coverage", maxsplit=1)[1]
        requirement_row = next(
            row
            for row in coverage_report.splitlines()
            if row.startswith("| KMIPKIT-REQ-SPEC-6.1.6-007 |")
        )

        self.assertIn("KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE".replace("_", r"\_"), requirement_row)
        self.assertIn("Certificate Request Type", requirement_row)

    def test_requirement_rule_coverage_shows_certify_get_placeholder_path_elements(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))

        report = render_report(catalog, typed_rule_mappings=[])
        coverage_report = report.split("## Profile requirement rule coverage", maxsplit=1)[1]
        requirement_row = next(
            row
            for row in coverage_report.splitlines()
            if row.startswith("| KMIPKIT-REQ-SPEC-6.1.6-003 |")
        )

        for name in ("Certify", "Get", "ID Placeholder", "Unique Identifier", "Batch Item", "Batch Order Option"):
            self.assertIn(name, requirement_row)

    def test_typed_rule_mapping_preserves_table_body_source_gap(self) -> None:
        from tools.normative_catalog import report as report_module

        mapping = next(
            row
            for row in report_module.load_typed_rule_mappings(ROOT)
            if row["requirement_id"] == "KMIPKIT-REQ-SPEC-2.3-003"
        )

        self.assertIsNone(mapping.get("source_clause_id"))
        self.assertEqual(
            mapping.get("source_gap_id"),
            "KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED",
        )

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        report = render_report(
            catalog,
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE-CLIENT"},
            typed_rule_mappings=report_module.load_typed_rule_mappings(ROOT),
        )
        requirement_row = next(
            row
            for row in report.splitlines()
            if row.startswith("| KMIPKIT-REQ-SPEC-2.3-003 |")
        )
        self.assertIn("KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED", requirement_row)
        self.assertIn("§2.3", requirement_row)
        self.assertIn("Opaque Data Type", requirement_row)
        self.assertIn("6927-6945", requirement_row)

    def test_full_profile_report_keeps_evidence_incomplete_with_runtime_ready_baseline_and_pinned_fixtures(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        target_ids = {"KMIPKIT-PROFILE-BASELINE-CLIENT"}
        typed_rule_mappings = report_module.load_typed_rule_mappings(ROOT)

        report = render_report(
            catalog,
            selected_profile_ids=target_ids,
            typed_rule_mappings=typed_rule_mappings,
        )

        self.assertFalse(
            any(
                row["requirement_id"] == "KMIPKIT-REQ-SPEC-4.57-002"
                for row in typed_rule_mappings
            )
        )
        self.assertFalse(
            any(
                row.startswith("| KMIPKIT-REQ-SPEC-4.57-002 |")
                and "Typed rule mapping:" in row
                for row in report.splitlines()
            )
        )
        baseline_blocker_row = next(
            row
            for row in report.splitlines()
            if row.startswith("| KMIPKIT-PROFILE-BASELINE-CLIENT |")
            and len(row.split("|")) == 9
        )
        self.assertNotIn("category test references missing: 1", baseline_blocker_row)

        self.assertCountEqual(
            [
                "KMIPKIT-REQ-PROF-3.1-001",
                "KMIPKIT-REQ-PROF-3.1.1-002",
                "KMIPKIT-REQ-PROF-3.1.1-005",
                "KMIPKIT-REQ-PROF-3.1.2-002",
                "KMIPKIT-REQ-PROF-3.1.2-002",
                "KMIPKIT-REQ-PROF-3.1.2-005",
                "KMIPKIT-REQ-PROF-3.1.2-005",
                "KMIPKIT-REQ-SPEC-2.3-001",
                "KMIPKIT-REQ-SPEC-2.3-001",
                "KMIPKIT-REQ-SPEC-2.3-002",
                "KMIPKIT-REQ-SPEC-2.3-002",
                "KMIPKIT-REQ-SPEC-2.3-002",
                "KMIPKIT-REQ-SPEC-2.3-003",
                "KMIPKIT-REQ-SPEC-2.4-001-001",
                "KMIPKIT-REQ-SPEC-2.4-001-001",
                "KMIPKIT-REQ-SPEC-2.4-001-002",
                "KMIPKIT-REQ-SPEC-2.4-001-002",
                "KMIPKIT-REQ-SPEC-2.4-002-001",
                "KMIPKIT-REQ-SPEC-2.4-002-001",
                "KMIPKIT-REQ-SPEC-2.7-001",
                "KMIPKIT-REQ-SPEC-2.7-001",
                "KMIPKIT-REQ-SPEC-2.8-003",
                "KMIPKIT-REQ-SPEC-2.8-003",
                "KMIPKIT-REQ-SPEC-2.8-003",
                "KMIPKIT-REQ-SPEC-2.8-003",
                "KMIPKIT-REQ-SPEC-2.8-003",
                "KMIPKIT-REQ-SPEC-3.3-001-001",
                "KMIPKIT-REQ-SPEC-3.3-001-002",
                "KMIPKIT-REQ-SPEC-3.3-002",
                "KMIPKIT-REQ-SPEC-3.3-006",
                "KMIPKIT-REQ-SPEC-3.3-004",
                "KMIPKIT-REQ-SPEC-3.1-002",
                "KMIPKIT-REQ-SPEC-3.1-002",
                "KMIPKIT-REQ-SPEC-3.1-006",
                "KMIPKIT-REQ-SPEC-3.1-006",
                "KMIPKIT-REQ-SPEC-3.2-001",
                "KMIPKIT-REQ-SPEC-3.2-001",
                "KMIPKIT-REQ-SPEC-3.1-001",
                "KMIPKIT-REQ-SPEC-3.1-001",
                "KMIPKIT-REQ-SPEC-3.1-001",
                "KMIPKIT-REQ-SPEC-4-001-004",
                "KMIPKIT-REQ-SPEC-4-001-005",
                "KMIPKIT-REQ-SPEC-4.14-001-001",
                "KMIPKIT-REQ-SPEC-4.14-001-001",
                "KMIPKIT-REQ-SPEC-4.14-001-002",
                "KMIPKIT-REQ-SPEC-4.14-001-002",
                "KMIPKIT-REQ-SPEC-5.1-002",
                "KMIPKIT-REQ-SPEC-5.1-002",
                "KMIPKIT-REQ-SPEC-5.2-002",
                "KMIPKIT-REQ-SPEC-5.2-002",
                "KMIPKIT-REQ-SPEC-5.3-002",
                "KMIPKIT-REQ-SPEC-5.3-002",
                "KMIPKIT-REQ-SPEC-5.4-002",
                "KMIPKIT-REQ-SPEC-5.4-002",
                "KMIPKIT-REQ-SPEC-6.1.7-003",
                "KMIPKIT-REQ-SPEC-6.1.7-003",
                "KMIPKIT-REQ-SPEC-6.1.7-004-002",
                "KMIPKIT-REQ-SPEC-6.1.7-004-002",
                "KMIPKIT-REQ-SPEC-6.1.9-001-001",
                "KMIPKIT-REQ-SPEC-6.1.9-001-001",
                "KMIPKIT-REQ-SPEC-6.1.9-001-001",
                "KMIPKIT-REQ-SPEC-6.1.9-001-002",
                "KMIPKIT-REQ-SPEC-6.1.9-001-002",
                "KMIPKIT-REQ-SPEC-6.1.9-001-002",
                "KMIPKIT-REQ-SPEC-6.1.9-006",
                "KMIPKIT-REQ-SPEC-6.1.9-006",
                "KMIPKIT-REQ-SPEC-6.1.9-006",
                "KMIPKIT-REQ-SPEC-6.1.9-006",
                "KMIPKIT-REQ-SPEC-6.1.9-006",
                "KMIPKIT-REQ-SPEC-6.1.9-007",
                "KMIPKIT-REQ-SPEC-6.1.9-007",
                "KMIPKIT-REQ-SPEC-6.1.9-007",
                "KMIPKIT-REQ-SPEC-6.1.9-007",
                "KMIPKIT-REQ-SPEC-6.1.9-007",
                "KMIPKIT-REQ-SPEC-3.1-004-002",
                "KMIPKIT-REQ-SPEC-3.1-005-002",
                "KMIPKIT-REQ-SPEC-6.1.43-001",
                "KMIPKIT-REQ-SPEC-6.1.43-001",
                "KMIPKIT-REQ-SPEC-6.1.43-007",
                "KMIPKIT-REQ-SPEC-6.1.43-007",
                "KMIPKIT-REQ-SPEC-6.1.43-009-001",
                "KMIPKIT-REQ-SPEC-6.1.43-009-001",
                "KMIPKIT-REQ-SPEC-6.1.43-009-001",
                "KMIPKIT-REQ-SPEC-6.1.43-009-001",
                "KMIPKIT-REQ-SPEC-6.1.43-010-001",
                "KMIPKIT-REQ-SPEC-6.1.43-010-001",
                "KMIPKIT-REQ-SPEC-6.1.43-010-001",
                "KMIPKIT-REQ-SPEC-6.1.43-010-001",
                "KMIPKIT-REQ-SPEC-6.1.43-011",
                "KMIPKIT-REQ-SPEC-6.1.43-011",
                "KMIPKIT-REQ-SPEC-6.1.43-011",
                "KMIPKIT-REQ-SPEC-6.1.43-011",
                "KMIPKIT-REQ-SPEC-6.1.43-009-002",
                "KMIPKIT-REQ-SPEC-6.1.43-009-002",
                "KMIPKIT-REQ-SPEC-6.1.43-009-002",
                "KMIPKIT-REQ-SPEC-6.1.43-010-002",
                "KMIPKIT-REQ-SPEC-4.27-001",
                "KMIPKIT-REQ-SPEC-4.28-001-001",
                "KMIPKIT-REQ-SPEC-4.28-001-001",
                "KMIPKIT-REQ-SPEC-4.28-001-001",
                "KMIPKIT-REQ-SPEC-4.28-001-001",
                "KMIPKIT-REQ-SPEC-4.28-001-002",
                "KMIPKIT-REQ-SPEC-4.28-001-003",
                "KMIPKIT-REQ-SPEC-4.28-001-003",
                "KMIPKIT-REQ-SPEC-6.1.6-001",
                "KMIPKIT-REQ-SPEC-6.1.6-002-001",
                "KMIPKIT-REQ-SPEC-6.1.6-002-002",
                "KMIPKIT-REQ-SPEC-6.1.6-002-002",
                "KMIPKIT-REQ-SPEC-6.1.6-002-003",
                "KMIPKIT-REQ-SPEC-6.1.6-003",
                "KMIPKIT-REQ-SPEC-6.1.6-003",
                "KMIPKIT-REQ-SPEC-6.1.6-007",
                "KMIPKIT-REQ-SPEC-6.1.43-005",
                "KMIPKIT-REQ-SPEC-6.1.43-005",
                "KMIPKIT-REQ-SPEC-4.35-001-001",
                "KMIPKIT-REQ-SPEC-4.35-001-001",
                "KMIPKIT-REQ-SPEC-4.35-001-002",
                "KMIPKIT-REQ-SPEC-4.35-001-002",
                "KMIPKIT-REQ-SPEC-4.38-001-001",
                "KMIPKIT-REQ-SPEC-4.38-001-001",
                "KMIPKIT-REQ-SPEC-4.38-001-002",
                "KMIPKIT-REQ-SPEC-4.38-001-002",
                "KMIPKIT-REQ-SPEC-4.38-001-003",
                "KMIPKIT-REQ-SPEC-4.38-001-003",
                "KMIPKIT-REQ-SPEC-4.39-002-001",
                "KMIPKIT-REQ-SPEC-4.46-001-001",
                "KMIPKIT-REQ-SPEC-4.46-001-001",
                "KMIPKIT-REQ-SPEC-4.46-001-002",
                "KMIPKIT-REQ-SPEC-4.46-001-002",
                "KMIPKIT-REQ-SPEC-4.47-001-001",
                "KMIPKIT-REQ-SPEC-4.47-001-001",
                "KMIPKIT-REQ-SPEC-4.47-001-001",
                "KMIPKIT-REQ-SPEC-4.47-001-002",
                "KMIPKIT-REQ-SPEC-4.47-001-002",
                "KMIPKIT-REQ-SPEC-4.47-001-002",
                "KMIPKIT-REQ-SPEC-6.1.44-003-002",
                "KMIPKIT-REQ-SPEC-6.1.44-003-002",
                "KMIPKIT-REQ-SPEC-6.1.45-001",
                "KMIPKIT-REQ-SPEC-6.1.45-008",
                "KMIPKIT-REQ-SPEC-6.1.45-008",
                "KMIPKIT-REQ-SPEC-6.1.45-008",
                "KMIPKIT-REQ-SPEC-6.1.45-008",
                "KMIPKIT-REQ-SPEC-6.1.57-001",
                "KMIPKIT-REQ-SPEC-6.1.57-002-002",
                "KMIPKIT-REQ-SPEC-6.1.57-002-001",
                "KMIPKIT-REQ-SPEC-6.1.57-004",
                "KMIPKIT-REQ-SPEC-6.1.57-005",
                "KMIPKIT-REQ-SPEC-4.62-003",
                "KMIPKIT-REQ-SPEC-4.63-004",
                "KMIPKIT-REQ-SPEC-4.63-005",
                "KMIPKIT-REQ-SPEC-8.1-002",
                "KMIPKIT-REQ-SPEC-8.2-002",
                "KMIPKIT-REQ-SPEC-9.3-001-001",
                "KMIPKIT-REQ-SPEC-9.3-001-001",
                "KMIPKIT-REQ-SPEC-9.3-001-001",
                "KMIPKIT-REQ-SPEC-9.3-001-001",
                "KMIPKIT-REQ-SPEC-9.11-004-003",
                "KMIPKIT-REQ-SPEC-9.11-004-003",
                # KMIPKIT-0010 T056: caller-responsible Device uniqueness.
                "KMIPKIT-REQ-SPEC-9.11-004-002",
                "KMIPKIT-REQ-SPEC-9.11-004-002",
                # KMIPKIT-0010 T056: model-only Credential identification.
                "KMIPKIT-REQ-SPEC-9.11-001",
                "KMIPKIT-REQ-SPEC-9.11-001",
                "KMIPKIT-REQ-SPEC-9.11-006",
                "KMIPKIT-REQ-SPEC-9.11-006",
                "KMIPKIT-REQ-SPEC-9.11-006",
                "KMIPKIT-REQ-SPEC-9.11-006",
                # KMIPKIT-0010 T056: model-level Hashing Algorithm default.
                "KMIPKIT-REQ-SPEC-9.11-010-002",
                "KMIPKIT-REQ-SPEC-9.11-010-002",
                "KMIPKIT-REQ-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.38-001-001",
                "KMIPKIT-REQ-SPEC-6.1.38-001-001",
                "KMIPKIT-REQ-SPEC-6.1.38-001-001",
                "KMIPKIT-REQ-SPEC-9.1-001",
                "KMIPKIT-REQ-SPEC-9.1-001",
                "KMIPKIT-REQ-SPEC-9.1-001",
                "KMIPKIT-REQ-SPEC-9.19-002",
                "KMIPKIT-REQ-SPEC-9.19-002",
                "KMIPKIT-REQ-SPEC-9.19-002",
                "KMIPKIT-REQ-SPEC-9.9-001",
                "KMIPKIT-REQ-SPEC-9.9-001",
                "KMIPKIT-REQ-SPEC-9.9-001",
                "KMIPKIT-REQ-SPEC-9.12-001-001",
                # KMIPKIT-0010 T056: client response size declaration is enforced.
                "KMIPKIT-REQ-SPEC-9.12-001-002",
                "KMIPKIT-REQ-SPEC-9.12-001-002",
                "KMIPKIT-REQ-SPEC-9.12-001-002",
                "KMIPKIT-REQ-SPEC-9.12-001-003",
                "KMIPKIT-REQ-SPEC-9.12-001-003",
                "KMIPKIT-REQ-SPEC-9.20-001-001",
                "KMIPKIT-REQ-SPEC-9.20-001-001",
                # KMIPKIT-0010 T056: caller-owned relative Date interpretation.
                "KMIPKIT-REQ-SPEC-9.20-001-002",
                "KMIPKIT-REQ-SPEC-9.20-001-002",
                "KMIPKIT-REQ-SPEC-9.21-001-001",
                "KMIPKIT-REQ-SPEC-9.21-001-001",
                "KMIPKIT-REQ-SPEC-8.3-003",
                "KMIPKIT-REQ-SPEC-8.3-004",
                "KMIPKIT-REQ-SPEC-8.3-004",
                "KMIPKIT-REQ-SPEC-9.5-001-001",
                "KMIPKIT-REQ-SPEC-9.5-001-002",
                "KMIPKIT-REQ-SPEC-9.6-001-001",
                "KMIPKIT-REQ-SPEC-9.6-001-002",
                "KMIPKIT-REQ-SPEC-9.6-001-002",
                "KMIPKIT-REQ-SPEC-9.6-001-003",
                "KMIPKIT-REQ-PROF-4.1.2-007",
                "KMIPKIT-REQ-PROF-4.1.2-007",
                "KMIPKIT-REQ-SPEC-9.17-001-002",
                "KMIPKIT-REQ-SPEC-9.17-001-002",
                "KMIPKIT-REQ-SPEC-9.17-001-002",
                "KMIPKIT-REQ-SPEC-4.16-001-002",
                "KMIPKIT-REQ-SPEC-4.16-001-001",
                "KMIPKIT-REQ-SPEC-4.16-001-001",
                "KMIPKIT-REQ-SPEC-4.16-001-001",
                "KMIPKIT-REQ-SPEC-4.16-001-001",
                "KMIPKIT-REQ-SPEC-4.16-002",
                "KMIPKIT-REQ-SPEC-4.16-003",
                "KMIPKIT-REQ-SPEC-6.1.16-001-001",
                "KMIPKIT-REQ-SPEC-6.1.16-001-002",
                "KMIPKIT-REQ-SPEC-6.1.10-001",
                "KMIPKIT-REQ-SPEC-6.1.13-006",
                "KMIPKIT-REQ-SPEC-6.1.34-001-001",
                "KMIPKIT-REQ-SPEC-6.1.28-001",
                "KMIPKIT-REQ-SPEC-6.1.28-002",
                "KMIPKIT-REQ-SPEC-6.1.28-007",
                "KMIPKIT-REQ-SPEC-6.1.28-013-001",
                "KMIPKIT-REQ-SPEC-6.1.28-013-002",
                "KMIPKIT-REQ-SPEC-6.1.20-001-001",
                "KMIPKIT-REQ-SPEC-6.1.20-004",
                "KMIPKIT-REQ-SPEC-6.1.18-005",
                "KMIPKIT-REQ-SPEC-6.1.18-005",
                "KMIPKIT-REQ-SPEC-6.1.18-006",
                "KMIPKIT-REQ-SPEC-6.1.18-006",
                "KMIPKIT-REQ-SPEC-6.1.19-002",
                "KMIPKIT-REQ-SPEC-6.1.19-003",
                "KMIPKIT-REQ-SPEC-6.1.19-004",
                "KMIPKIT-REQ-SPEC-6.1.19-005",
                "KMIPKIT-REQ-SPEC-6.1.19-006",
                "KMIPKIT-REQ-SPEC-7.18-006",
                "KMIPKIT-REQ-SPEC-7.18-001-001",
                "KMIPKIT-REQ-SPEC-7.18-004",
                "KMIPKIT-REQ-SPEC-7.18-005",
                "KMIPKIT-REQ-SPEC-6.1.40-014",
                "KMIPKIT-REQ-SPEC-6.1.40-016",
                "KMIPKIT-REQ-SPEC-6.1.14-001-002",
                "KMIPKIT-REQ-SPEC-6.1.14-001-003",
                "KMIPKIT-REQ-SPEC-6.1.14-007-001",
                "KMIPKIT-REQ-SPEC-6.1.14-007-002",
                "KMIPKIT-REQ-SPEC-6.1.14-009",
                "KMIPKIT-REQ-SPEC-6.1.14-010",
                "KMIPKIT-REQ-SPEC-6.1.14-011",
                "KMIPKIT-REQ-SPEC-6.1.14-012",
                "KMIPKIT-REQ-SPEC-6.1.14-001-004",
                "KMIPKIT-REQ-SPEC-6.1.14-006-001",
                "KMIPKIT-REQ-SPEC-6.1.14-006-002",
                "KMIPKIT-REQ-SPEC-10.1.2-001",
                "KMIPKIT-REQ-SPEC-10.1.2-001",
                "KMIPKIT-REQ-SPEC-10.1.2-002-001",
                "KMIPKIT-REQ-SPEC-10.1.2-002-001",
                "KMIPKIT-REQ-SPEC-10.1.2-002-002",
                "KMIPKIT-REQ-SPEC-10.1.5-001-001",
                "KMIPKIT-REQ-SPEC-10.1.5-001-001",
                "KMIPKIT-REQ-SPEC-10.1.5-001-001",
                "KMIPKIT-REQ-SPEC-10.1.5-001-002",
                "KMIPKIT-REQ-SPEC-10.4-001-001",
                "KMIPKIT-REQ-SPEC-10.4-001-001",
                "KMIPKIT-REQ-SPEC-10.4-001-002",
                "KMIPKIT-REQ-SPEC-10.4-001-002",
                "KMIPKIT-REQ-SPEC-10.4-001-003",
                "KMIPKIT-REQ-SPEC-10.4-001-003",
                "KMIPKIT-REQ-SPEC-10.4-001-003",
                "KMIPKIT-REQ-SPEC-10.4-001-003",
                "KMIPKIT-REQ-SPEC-11.56-001",
                "KMIPKIT-REQ-SPEC-11-001",
                "KMIPKIT-REQ-SPEC-11-001",
                "KMIPKIT-REQ-SPEC-12-001-001",
                "KMIPKIT-REQ-SPEC-12-001-001",
                "KMIPKIT-REQ-SPEC-2.8-001-002",
                "KMIPKIT-REQ-SPEC-2.8-001-002",
                "KMIPKIT-REQ-SPEC-2.8-001-002",
                "KMIPKIT-REQ-SPEC-3.3-003",
                "KMIPKIT-REQ-SPEC-3.3-003",
                "KMIPKIT-REQ-SPEC-3.3-009",
                "KMIPKIT-REQ-SPEC-3.3-009",
                "KMIPKIT-REQ-SPEC-4.31-002",
                "KMIPKIT-REQ-SPEC-4.31-004",
                "KMIPKIT-REQ-SPEC-4.31-004",
                "KMIPKIT-REQ-SPEC-4.31-004",
                "KMIPKIT-REQ-SPEC-4.31-004",
                "KMIPKIT-REQ-SPEC-4.31-004",
                "KMIPKIT-REQ-SPEC-4.31-005",
                "KMIPKIT-REQ-SPEC-4.31-005",
                "KMIPKIT-REQ-SPEC-4.31-005",
                "KMIPKIT-REQ-SPEC-4.31-005",
                "KMIPKIT-REQ-SPEC-4.31-005",
                "KMIPKIT-REQ-SPEC-6.1.18-001-001",
                "KMIPKIT-REQ-SPEC-6.1.18-007",
                "KMIPKIT-REQ-SPEC-6.1.18-008",
                "KMIPKIT-REQ-SPEC-14.1-001",
                "KMIPKIT-REQ-SPEC-14.1-002",
                # Keep the hand-reviewed registry baseline synchronized with
                # profile evidence added after the original partial-registry
                # fixture; duplicate IDs represent distinct rule test cases.
                *[
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-001",
                    "KMIPKIT-REQ-SPEC-4.34-001-001",
                    "KMIPKIT-REQ-SPEC-4.34-001-002",
                    "KMIPKIT-REQ-SPEC-4.40-001-001",
                    "KMIPKIT-REQ-SPEC-4.40-001-002",
                    "KMIPKIT-REQ-SPEC-4.40-001-003",
                    "KMIPKIT-REQ-SPEC-4.40-001-004",
                    "KMIPKIT-REQ-SPEC-4.40-001-005",
                    "KMIPKIT-REQ-SPEC-4.41-001-001",
                    "KMIPKIT-REQ-SPEC-4.41-001-002",
                    "KMIPKIT-REQ-SPEC-4.41-001-003",
                    "KMIPKIT-REQ-SPEC-4.41-001-004",
                    "KMIPKIT-REQ-SPEC-4.35-001-003",
                    "KMIPKIT-REQ-SPEC-4.35-001-003",
                    "KMIPKIT-REQ-SPEC-4.35-001-003",
                    "KMIPKIT-REQ-SPEC-6.1.27-001",
                    "KMIPKIT-REQ-SPEC-6.1.27-001",
                    "KMIPKIT-REQ-SPEC-6.1.27-001",
                    "KMIPKIT-REQ-SPEC-6.1.27-001",
                    "KMIPKIT-REQ-SPEC-6.1.27-001",
                    "KMIPKIT-REQ-SPEC-6.1.27-002-002",
                    "KMIPKIT-REQ-SPEC-6.1.27-005",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-001",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-001",
                    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
                    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
                    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
                    "KMIPKIT-REQ-SPEC-6.1.46-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.46-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.46-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.47-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.47-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.47-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.55-001",
                    "KMIPKIT-REQ-SPEC-6.1.55-001",
                    "KMIPKIT-REQ-SPEC-6.1.55-001",
                    "KMIPKIT-REQ-SPEC-6.1.50-001",
                    "KMIPKIT-REQ-SPEC-6.1.50-001",
                    "KMIPKIT-REQ-SPEC-6.1.50-001",
                    "KMIPKIT-REQ-SPEC-6.1.51-001",
                    "KMIPKIT-REQ-SPEC-6.1.56-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.56-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.56-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.56-002",
                    "KMIPKIT-REQ-SPEC-6.1.56-002",
                    "KMIPKIT-REQ-SPEC-6.1.56-002",
                    "KMIPKIT-REQ-SPEC-6.1.7-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.7-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.7-005",
                    "KMIPKIT-REQ-SPEC-6.1.7-005",
                    "KMIPKIT-REQ-SPEC-6.1.7-002-001",
                    "KMIPKIT-REQ-SPEC-6.1.7-002-001",
                    "KMIPKIT-REQ-SPEC-6.1.7-002-002",
                    "KMIPKIT-REQ-SPEC-6.1.7-002-003",
                    "KMIPKIT-REQ-SPEC-4.59-003-002",
                    "KMIPKIT-REQ-SPEC-4.59-003-002",
                    "KMIPKIT-REQ-SPEC-6.1.17-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.17-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.17-006",
                    "KMIPKIT-REQ-SPEC-6.1.17-006",
                    "KMIPKIT-REQ-SPEC-6.1.23-002-001",
                    "KMIPKIT-REQ-SPEC-6.1.11-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.11-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.11-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.11-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.11-005",
                    "KMIPKIT-REQ-SPEC-6.1.11-005",
                    "KMIPKIT-REQ-SPEC-6.1.11-006",
                    "KMIPKIT-REQ-SPEC-6.1.11-006",
                    "KMIPKIT-REQ-SPEC-6.1.12-003",
                    "KMIPKIT-REQ-SPEC-6.1.12-003",
                    "KMIPKIT-REQ-SPEC-6.1.12-005",
                    "KMIPKIT-REQ-SPEC-6.1.12-005",
                    "KMIPKIT-REQ-SPEC-6.1.12-006",
                    "KMIPKIT-REQ-SPEC-6.1.12-006",
                    "KMIPKIT-REQ-SPEC-6.1.12-007",
                    "KMIPKIT-REQ-SPEC-6.1.12-007",
                    "KMIPKIT-REQ-SPEC-6.1.30-001",
                    "KMIPKIT-REQ-SPEC-6.1.30-001",
                    "KMIPKIT-REQ-SPEC-6.1.30-002",
                    "KMIPKIT-REQ-SPEC-6.1.30-002",
                    "KMIPKIT-REQ-SPEC-6.1.30-003",
                    "KMIPKIT-REQ-SPEC-6.1.30-003",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                    "KMIPKIT-REQ-SPEC-6.1.31-001",
                    "KMIPKIT-REQ-SPEC-6.1.31-001",
                    "KMIPKIT-REQ-SPEC-6.1.32-001",
                    "KMIPKIT-REQ-SPEC-6.1.32-001",
                    "KMIPKIT-REQ-SPEC-6.1.33-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.33-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.33-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.33-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.13-005",
                    "KMIPKIT-REQ-SPEC-6.1-001-003",
                    "KMIPKIT-REQ-SPEC-6.1-001-003",
                    "KMIPKIT-REQ-SPEC-7.8-001",
                    "KMIPKIT-REQ-SPEC-7.8-001",
                    "KMIPKIT-REQ-SPEC-7.8-001",
                    "KMIPKIT-REQ-SPEC-7.8-001",
                    # KMIPKIT-0010 T040: message envelope mappings; repeats
                    # reflect distinct category-matched test references.
                    "KMIPKIT-REQ-SPEC-8-001",
                    "KMIPKIT-REQ-SPEC-8-001",
                    "KMIPKIT-REQ-SPEC-8-001",
                    "KMIPKIT-REQ-SPEC-8-001",
                    "KMIPKIT-REQ-SPEC-8-002",
                    "KMIPKIT-REQ-SPEC-8-002",
                    "KMIPKIT-REQ-SPEC-8-002",
                    "KMIPKIT-REQ-SPEC-8-002",
                    "KMIPKIT-REQ-SPEC-9.7-001",
                    "KMIPKIT-REQ-SPEC-9.7-001",
                    "KMIPKIT-REQ-SPEC-9.7-001",
                    # KMIPKIT-0010 T049 and T050: caller-supplied wrapped
                    # Import and optional Archive request evidence.
                    "KMIPKIT-REQ-SPEC-6.1.25-006",
                    "KMIPKIT-REQ-SPEC-6.1.25-006",
                    "KMIPKIT-REQ-SPEC-6.1.4-001",
                    "KMIPKIT-REQ-SPEC-6.1.4-001",
                    "KMIPKIT-REQ-SPEC-6.1.4-001",
                    # KMIPKIT-0010 T051: Baseline profile gates and shared
                    # operation evidence; repeated IDs represent distinct
                    # response/capability test references.
                    "KMIPKIT-REQ-PROF-4.1.2-001",
                    "KMIPKIT-REQ-PROF-6-001",
                    "KMIPKIT-REQ-PROF-6.1-001",
                    "KMIPKIT-REQ-PROF-6.1-002",
                    "KMIPKIT-REQ-PROF-6.1-003",
                    "KMIPKIT-REQ-PROF-6.1-004",
                    "KMIPKIT-REQ-SPEC-6.1-001-001",
                    "KMIPKIT-REQ-SPEC-6.1-001-002",
                    "KMIPKIT-REQ-SPEC-6.1-001-002",
                    "KMIPKIT-REQ-SPEC-6.1-001-002",
                    "KMIPKIT-REQ-SPEC-6.1-003-002",
                    "KMIPKIT-REQ-SPEC-6.1-003-002",
                    "KMIPKIT-REQ-SPEC-6.1-004",
                    "KMIPKIT-REQ-SPEC-6.1-004",
                    "KMIPKIT-REQ-SPEC-6.1-005",
                    "KMIPKIT-REQ-SPEC-6.1-005",
                    # KMIPKIT-0010 profile evidence added after the original
                    # registry snapshot; repeats represent distinct mapped
                    # requirements, not unresolved report rows.
                    "KMIPKIT-REQ-SPEC-3.7-002",
                    "KMIPKIT-REQ-SPEC-3.7-002",
                    "KMIPKIT-REQ-SPEC-3.7-002",
                    "KMIPKIT-REQ-SPEC-3.7-002",
                    "KMIPKIT-REQ-SPEC-4.4-001",
                    "KMIPKIT-REQ-SPEC-4.26-001",
                    "KMIPKIT-REQ-SPEC-4.26-001",
                    "KMIPKIT-REQ-SPEC-6.1.7-001",
                    "KMIPKIT-REQ-SPEC-3.1-004-001",
                    "KMIPKIT-REQ-SPEC-3.1-004-001",
                    "KMIPKIT-REQ-SPEC-3.1-005-001",
                    "KMIPKIT-REQ-SPEC-3.1-005-001",
                    # KMIPKIT-0010 T052: caller-owned Name uniqueness.
                    "KMIPKIT-REQ-SPEC-4.32-001",
                    "KMIPKIT-REQ-SPEC-6.1.44-003-001",
                    # KMIPKIT-0010 T054: caller-directed archived retrieval
                    # and optional Recover follow-up capabilities.
                    "KMIPKIT-REQ-SPEC-6.1.28-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.42-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.42-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.14-001-001",
                    # KMIPKIT-0010 T054: caller-owned lifecycle
                    # recommendations and credential reprovisioning.
                    "KMIPKIT-REQ-SPEC-6.1.45-004",
                    "KMIPKIT-REQ-SPEC-6.1.45-004",
                    "KMIPKIT-REQ-SPEC-6.1.46-001",
                    "KMIPKIT-REQ-SPEC-6.1.46-001",
                    "KMIPKIT-REQ-SPEC-6.1.47-001",
                    "KMIPKIT-REQ-SPEC-6.1.47-001",
                    "KMIPKIT-REQ-SPEC-6.1.48-006",
                    "KMIPKIT-REQ-SPEC-6.1.48-006",
                    # KMIPKIT-0010 T053: caller-owned lifecycle-date,
                    # Usage Mask, lease, and allocation mappings. These tests
                    # verify static responsibility metadata, not local gating.
                    "KMIPKIT-REQ-SPEC-4.1-001-001",
                    "KMIPKIT-REQ-SPEC-4.1-001-002",
                    "KMIPKIT-REQ-SPEC-4.1-001-003",
                    "KMIPKIT-REQ-SPEC-4.17-001-001",
                    "KMIPKIT-REQ-SPEC-4.17-001-002",
                    "KMIPKIT-REQ-SPEC-4.18-001-001",
                    "KMIPKIT-REQ-SPEC-4.18-001-002",
                    "KMIPKIT-REQ-SPEC-4.30-001",
                    # KMIPKIT-0010 T040: State cannot be changed through the
                    # client-to-server Modify Attribute operation.
                    "KMIPKIT-REQ-SPEC-4.57-001",
                    "KMIPKIT-REQ-SPEC-6.1.23-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.23-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.23-002-002",
                    "KMIPKIT-REQ-SPEC-6.1.35-001-002",
                    "KMIPKIT-REQ-SPEC-6.1.35-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.35-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.35-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.35-002",
                    "KMIPKIT-REQ-SPEC-6.1.35-002",
                    "KMIPKIT-REQ-SPEC-6.1.35-002",
                    "KMIPKIT-REQ-SPEC-6.1.35-005",
                    "KMIPKIT-REQ-SPEC-6.1.35-005",
                    "KMIPKIT-REQ-SPEC-6.1.35-005",
                    # KMIPKIT-0010 T055: key-derivation and asynchronous
                    # response mappings added after the partial-registry baseline.
                    "KMIPKIT-REQ-SPEC-7.12-003",
                    "KMIPKIT-REQ-SPEC-7.12-005-001",
                    "KMIPKIT-REQ-SPEC-7.12-005-001",
                    "KMIPKIT-REQ-SPEC-7.12-005-001",
                    "KMIPKIT-REQ-SPEC-8-003-001",
                    "KMIPKIT-REQ-SPEC-8-003-001",
                    # KMIPKIT-0010 T056: authentication, Device Credential,
                    # Hashed Password timestamp, and response Message Extension
                    # profile mappings.
                    "KMIPKIT-REQ-SPEC-9.11-010-001",
                    "KMIPKIT-REQ-SPEC-9.11-010-001",
                    "KMIPKIT-REQ-SPEC-9.11-010-001",
                    "KMIPKIT-REQ-SPEC-9.11-010-001",
                    "KMIPKIT-REQ-SPEC-9.4-001-002",
                    "KMIPKIT-REQ-SPEC-9.4-001-002",
                    "KMIPKIT-REQ-SPEC-9.4-001-001",
                    "KMIPKIT-REQ-SPEC-9.4-001-001",
                    "KMIPKIT-REQ-SPEC-9.4-002",
                    "KMIPKIT-REQ-SPEC-9.4-002",
                    "KMIPKIT-REQ-SPEC-9.4-002",
                    "KMIPKIT-REQ-SPEC-9.11-004-001",
                    "KMIPKIT-REQ-SPEC-9.11-004-001",
                    # KMIPKIT-0010 T040: client-sender Message Extension rules.
                    "KMIPKIT-REQ-SPEC-9.13-001-001",
                    "KMIPKIT-REQ-SPEC-9.13-001-001",
                    "KMIPKIT-REQ-SPEC-9.13-001-002",
                    "KMIPKIT-REQ-SPEC-9.13-001-002",
                    "KMIPKIT-REQ-SPEC-9.13-001-002",
                    "KMIPKIT-REQ-SPEC-9.13-001-003",
                    "KMIPKIT-REQ-SPEC-9.13-001-003",
                    "KMIPKIT-REQ-SPEC-9.13-001-003",
                    "KMIPKIT-REQ-SPEC-9.13-001-004",
                    "KMIPKIT-REQ-SPEC-9.13-001-004",
                    "KMIPKIT-REQ-SPEC-9.13-001-004",
                    "KMIPKIT-REQ-SPEC-9.13-001-004",
                    "KMIPKIT-REQ-SPEC-9.13-001-005",
                    "KMIPKIT-REQ-SPEC-9.13-001-005",
                    "KMIPKIT-REQ-SPEC-9.13-001-005",
                    # KMIPKIT-0010 T040: typed evidence for repeated Name,
                    # caller-owned distinct values, and profile query replies.
                    "KMIPKIT-REQ-SPEC-4-001-001",
                    "KMIPKIT-REQ-SPEC-4-001-002",
                    "KMIPKIT-REQ-PROF-4.1.2-005",
                    "KMIPKIT-REQ-SPEC-7.31-001",
                    "KMIPKIT-REQ-SPEC-7.32-001",
                    "KMIPKIT-REQ-SPEC-7.32-001",
                    # KMIPKIT-0010 T040: final Baseline mappings, including
                    # both state-aware negative tests for Original Creation Date.
                    *["KMIPKIT-REQ-SPEC-4-001-003"] * 11,
                    # Shared RNG Parameters evidence covers client request and
                    # response representation, optional fields, and negatives.
                    *["KMIPKIT-REQ-SPEC-7.36-001"] * 4,
                    *["KMIPKIT-REQ-SPEC-7.36-002-001"] * 6,
                    *["KMIPKIT-REQ-SPEC-7.36-002-002"] * 4,
                    *["KMIPKIT-REQ-SPEC-7.36-003"] * 7,
                    "KMIPKIT-REQ-SPEC-4.38-003",
                    "KMIPKIT-REQ-SPEC-4.38-003",
                    "KMIPKIT-REQ-SPEC-4.57-005-002",
                    "KMIPKIT-REQ-SPEC-4.59-002",
                    "KMIPKIT-REQ-SPEC-4.59-002",
                    "KMIPKIT-REQ-SPEC-4.59-003-001",
                    "KMIPKIT-REQ-SPEC-6.1.14-002",
                    "KMIPKIT-REQ-SPEC-6.1.34-001-003",
                ],
            ],
            [mapping["requirement_id"] for mapping in typed_rule_mappings],
        )
        profile_state_section = report.split("## Profile states\n", 1)[1]
        baseline_row = next(
            line for line in profile_state_section.splitlines()
            if line.startswith("| KMIPKIT-PROFILE-BASELINE-CLIENT |")
        )
        self.assertIn("| selected | true | evidence\\_incomplete |", baseline_row)
        self.assertIn("No typed rule mapping", report)
        self.assertIn("No category-appropriate rule test", report)
        self.assertIn("KMIPKIT-REQ-SPEC-6.1.16-001-001", report)
        self.assertIn("Category test reference: present", report)
        usage_count_source_row = next(
            line
            for line in report.splitlines()
            if line.startswith("| KMIPKIT-REQ-SPEC-6.1.23-002-001 |")
            and "KMIPKIT-SOURCE-GAP-SPEC-6.1.23-USAGE-LIMITS-COUNT-REQUIRED" in line
        )
        self.assertIn("KMIPKIT-SRC-spec §6.1.23, KMIPKIT-SRC-spec §7.40", usage_count_source_row)
        profile_state_section = report.split("## Profile states\n", 1)[1]
        baseline_row = next(
            line for line in profile_state_section.splitlines()
            if line.startswith("| KMIPKIT-PROFILE-BASELINE-CLIENT |")
        )
        self.assertNotIn("mandatory fixtures unavailable:", baseline_row)
        self.assertIn("| available | 23 |", report)
        self.assertIn("KMIPKIT-TEST-PROF-5-1-3-1", report)
        self.assertIn("No public claim", report)

    def test_baseline_report_distinguishes_global_unavailable_profile_fixtures(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        report = render_report(
            catalog,
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE-CLIENT"},
            typed_rule_mappings=report_module.load_typed_rule_mappings(ROOT),
        )

        note_rows = [
            line
            for line in report.splitlines()
            if line.startswith("| KMIPKIT-REQ-PROF-6-001 |")
            or line.startswith("| KMIPKIT-REQ-PROF-6.1-003 |")
        ]
        notes = "\n".join(note_rows)
        if "77 linked profile cases without local fixtures" in notes:
            self.fail("the report incorrectly attributes the broader fixture inventory to Baseline")
        expected_fragments = (
            "77 linked case records without local fixtures",
            "none belongs to Baseline's 16-case list",
        )
        for fragment in expected_fragments:
            if fragment not in notes:
                self.fail(f"generated profile report is missing the fixture-scope clarification: {fragment}")

    def test_baseline_test_case_evidence_matches_oasis_minimum(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        report = render_report(
            catalog,
            selected_profile_ids={"KMIPKIT-PROFILE-BASELINE-CLIENT"},
            typed_rule_mappings=report_module.load_typed_rule_mappings(ROOT),
        )

        baseline_test_requirement = next(
            line
            for line in report.splitlines()
            if line.startswith("| KMIPKIT-REQ-PROF-6.1-004 |")
        )
        requirement = next(
            row
            for row in catalog["requirements"]
            if row["requirement_id"] == "KMIPKIT-REQ-PROF-6.1-004"
        )
        note = requirement["review_note"]
        self.assertEqual(
            requirement["summary"],
            "Support one or more Baseline Mandatory Test Cases KMIP v2 (5.1.3 and 5.6.3).",
        )
        self.assertIn("one or more Baseline Mandatory Test Cases", note)
        self.assertIn("not all 16", note)
        self.assertIn(
            "fully completes 10 of 13 BL-M cases and all 3 SKLC-M cases",
            note,
        )
        self.assertIn(
            "BL-M-4-21, BL-M-7-21, and BL-M-10-21 remain pinned but blocked before full replay",
            note,
        )
        self.assertIn(
            "not formal OASIS test execution, external server interoperability, or a Profile claim",
            note,
        )
        self.assertIn("KMIPKIT-TEST-PROF-5-1-3-4", baseline_test_requirement)

    def test_stale_complete_claim_is_downgraded_without_publishing(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        profile = next(row for row in catalog["profiles"] if row["profile_id"] == profile_id)
        profile["claim_state"] = "evidence_complete"
        evaluator = getattr(report_module, "evaluate_profile_readiness", None)
        self.assertTrue(callable(evaluator), "report.py must expose profile readiness evaluation")

        readiness = evaluator(
            catalog,
            profile_id,
            selected_profile_ids={profile_id},
            typed_rule_mappings=report_module.load_typed_rule_mappings(ROOT),
            verification_manifest=None,
            current_context=None,
        )

        self.assertIs(readiness["target_selected"], True)
        self.assertIs(readiness["runtime_validation_available"], True)
        self.assertEqual("evidence_incomplete", readiness["effective_state"])
        self.assertIn("catalog claim_state evidence_complete is stale", readiness["blockers"])
        self.assertIs(readiness["published_claim"], False)

    def test_conditional_profile_readiness_requires_explicit_configuration(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-HTTPS-CLIENT"
        readiness = report_module.evaluate_profile_readiness(
            catalog,
            profile_id,
            selected_profile_ids={profile_id},
            typed_rule_mappings=[],
            verification_manifest=None,
            current_context=None,
        )

        self.assertIs(readiness["runtime_validation_available"], False)
        self.assertTrue(any("conditional" in blocker for blocker in readiness["blockers"]))
        self.assertIs(readiness["published_claim"], False)

    def test_dependency_readiness_failure_blocks_dependent_profile(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-AES-XTS-CLIENT"
        readiness = report_module.evaluate_profile_readiness(
            catalog,
            profile_id,
            selected_profile_ids={profile_id},
            typed_rule_mappings=[],
            verification_manifest=None,
            current_context=None,
        )

        self.assertIs(readiness["runtime_validation_available"], False)
        self.assertTrue(any("KMIPKIT-PROFILE-BASELINE-CLIENT" in blocker for blocker in readiness["blockers"]))
        self.assertIs(readiness["published_claim"], False)

    def test_dependency_readiness_recomputes_stale_dependency_evidence(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        baseline_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        dependent_id = "KMIPKIT-PROFILE-AES-XTS-CLIENT"
        baseline = next(row for row in catalog["profiles"] if row["profile_id"] == baseline_id)
        dependent = next(row for row in catalog["profiles"] if row["profile_id"] == dependent_id)
        baseline["claim_state"] = "evidence_complete"
        dependent["claim_state"] = "evidence_complete"
        mandatory_test = next(
            row
            for row in catalog["test_cases"]
            if row["test_id"] in baseline["test_case_ids"]
            and row["mandatory_status"] == "mandatory"
        )
        mandatory_test["fixture_path"] = None
        mandatory_test["fixture_availability"] = "unavailable"

        readiness = report_module.evaluate_profile_readiness(
            catalog,
            dependent_id,
            selected_profile_ids={dependent_id},
            typed_rule_mappings=report_module.load_typed_rule_mappings(ROOT),
            verification_manifest=None,
            current_context=None,
        )

        self.assertIn(
            f"dependency profile is not evidence complete: {baseline_id}",
            readiness["blockers"],
        )
        self.assertIs(readiness["published_claim"], False)

    def test_dependency_cycle_downgrades_otherwise_complete_readiness(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog, manifest, context, mappings, profile_a_id, profile_b_id = (
            self._complete_profile_readiness_fixture()
        )

        acyclic_readiness = report_module._evaluate_profile_readiness(
            catalog,
            profile_a_id,
            selected_profile_ids={profile_a_id, profile_b_id},
            typed_rule_mappings=mappings,
            verification_manifest=manifest,
            current_context=context,
            current_head=context["sha"],
            dependency_path=(),
            evidence_proof=report_module._VERIFIED_EVIDENCE_PROOF,
        )
        self.assertEqual("evidence_complete", acyclic_readiness["effective_state"])
        self.assertEqual([], acyclic_readiness["blockers"])

        profile_b = next(
            row for row in catalog["profiles"] if row["profile_id"] == profile_b_id
        )
        profile_b["dependency_profile_ids"] = [profile_a_id]
        cyclic_readiness = report_module._evaluate_profile_readiness(
            catalog,
            profile_a_id,
            selected_profile_ids={profile_a_id, profile_b_id},
            typed_rule_mappings=mappings,
            verification_manifest=manifest,
            current_context=context,
            current_head=context["sha"],
            dependency_path=(),
            evidence_proof=report_module._VERIFIED_EVIDENCE_PROOF,
        )

        self.assertTrue(
            any(
                f"dependency cycle detected: {profile_a_id}" in blocker
                for blocker in cyclic_readiness["blockers"]
            )
        )
        self.assertEqual("evidence_incomplete", cyclic_readiness["effective_state"])

    def test_complete_profile_readiness_rejects_unverified_caller_context(self) -> None:
        from tools.normative_catalog import report as report_module

        catalog, manifest, context, mappings, profile_a_id, profile_b_id = (
            self._complete_profile_readiness_fixture()
        )
        readiness = report_module.evaluate_profile_readiness(
            catalog,
            profile_a_id,
            selected_profile_ids={profile_a_id, profile_b_id},
            typed_rule_mappings=mappings,
            verification_manifest=manifest,
            current_context=context,
            current_head=context["sha"],
        )

        self.assertEqual("evidence_incomplete", readiness["effective_state"])
        self.assertIn(
            "current protected-release-push verification manifest is unavailable",
            readiness["blockers"],
        )
        self.assertIs(readiness["published_claim"], False)

    def test_report_omits_secret_markers_from_untrusted_diagnostic_text(self) -> None:
        catalog = report_catalog()
        credential_marker = "credential=report-secret-sentinel-731"
        private_key_marker = "private-key-sentinel-932"
        raw_body_marker = "raw-kmip-body-sentinel-184"
        catalog["requirements"][0]["review_note"] = (
            f"diagnostic {credential_marker} private-key={private_key_marker} "
            f"raw-kmip-body={raw_body_marker}"
        )
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-REPORT-001",
                "state": "open",
                "summary": f"malformed evidence raw-kmip-body={raw_body_marker}",
                "source_refs": [],
                "affected_requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "affected_element_ids": [],
                "affected_profile_ids": [],
                "affected_policy_ids": [],
            }
        ]

        report = render_report(catalog)

        for marker in (credential_marker, private_key_marker, raw_body_marker):
            with self.subTest(marker=marker):
                self.assertNotIn(marker, report)

    def test_report_redacts_the_complete_quoted_multword_secret_value(self) -> None:
        catalog = report_catalog()
        secret_suffix_marker = "report-secret-suffix-sentinel-184"
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-REPORT-QUOTED-SECRET",
                "state": "open",
                "summary": f'diagnostic credential="alpha {secret_suffix_marker}"',
                "source_refs": [],
                "affected_requirement_ids": [],
                "affected_element_ids": [],
                "affected_profile_ids": [],
                "affected_policy_ids": [],
            }
        ]

        report = render_report(catalog)

        self.assertNotIn(secret_suffix_marker, report)

    def test_device_credential_order_evidence_stays_scoped_and_global_rule_deferred(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-10.1.2-001"
        test_id = (
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
            "profile_requires_a_device_identifier_field_in_device_credentials"
        )
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        requirement = next(row for row in catalog["requirements"] if row["requirement_id"] == requirement_id)

        self.assertEqual(requirement["status"], "scoped_verified")
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn(test_id, requirement["verification_refs"])
        self.assertIn("Device Credential only", requirement["review_note"])

        with (ROOT / "specification/compliance/requirements/KMIPKIT-0013.csv").open(
            newline="", encoding="utf-8"
        ) as stream:
            trace_row = next(row for row in csv.DictReader(stream) if row["requirement_id"] == requirement_id)

        self.assertEqual(trace_row["status"], "deferred")
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", trace_row["implementation_location"].split("; "))
        self.assertIn(test_id, trace_row["test_ids"].split("; "))
        self.assertIn("Device Credential only", trace_row["statement"])

    def test_full_catalog_report_reconciles_tags_ranges_and_profile_cases(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        report = render_report(catalog)

        self.assertIn("| assigned | 354 |", report)
        self.assertIn("| reserved | 20 |", report)
        self.assertEqual(report.count("KMIPKIT-RANGE-00"), 5)
        tag_elements = [row for row in catalog["elements"] if row["kind"] == "tag"]
        self.assertEqual(len(tag_elements), 374)
        self.assertTrue(all(
            row["feature_spec"] == "KMIPKIT-0004"
            and row["implementation_refs"]
            and row["verification_refs"]
            for row in tag_elements
        ))
        unassigned_elements = report.split("## Unassigned protocol elements\n", 1)[1].split("\n## ", 1)[0]
        self.assertTrue(all(row["element_id"] not in unassigned_elements for row in tag_elements))
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        reserved_tag_requirement = requirements["KMIPKIT-REQ-SPEC-11-001"]
        self.assertEqual(reserved_tag_requirement["feature_spec"], "KMIPKIT-0004")
        self.assertTrue(reserved_tag_requirement["implementation_refs"])
        self.assertTrue(reserved_tag_requirement["verification_refs"])
        structure_order_requirement = requirements["KMIPKIT-REQ-SPEC-10.1.2-001"]
        self.assertEqual(structure_order_requirement["feature_spec"], "KMIPKIT-0005")
        self.assertTrue(structure_order_requirement["implementation_refs"])
        self.assertTrue(structure_order_requirement["verification_refs"])
        self.assertIn("BL-M-1-21; mandatory", report)

    def test_escapes_markdown_html_newlines_and_control_characters(self) -> None:
        catalog = report_catalog()
        catalog["discrepancies"] = [
            {
                "discrepancy_id": "KMIPKIT-DISC-001",
                "summary": "| `x` [evil](https://bad) ![img](https://bad)\\\r\n\x1b",
                "state": "open",
                "source_refs": [],
            }
        ]
        report = render_report(catalog)
        self.assertNotIn("][https://bad]", report)
        self.assertNotIn("![img]", report)
        self.assertIn("\\|", report)
        self.assertIn("\\`", report)
        self.assertIn("\\[", report)
        self.assertIn("\\\\", report)
        self.assertIn("\\\\u001B", report)

    def test_source_url_is_never_taken_from_untrusted_catalog_text(self) -> None:
        catalog = report_catalog()
        catalog["sources"][0]["canonical_url"] = "https://attacker.invalid/"
        report = render_report(catalog)
        self.assertNotIn("attacker.invalid", report)
        self.assertNotIn("https://", report)

    def test_script_entrypoint_works_from_a_clean_repository_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = ROOT / "specification" / "oasis" / "kmip-2.1"
            shutil.copytree(source, root / "specification" / "oasis" / "kmip-2.1")
            catalog_path = root / "specification" / "catalog" / "kmip-2.1.json"
            catalog_path.parent.mkdir(parents=True)
            catalog_path.write_text(json.dumps(minimal_catalog(), indent=2) + "\n", encoding="utf-8")
            for arguments in (
                ["git", "init", "--quiet", "--initial-branch=main"],
                ["git", "config", "user.name", "KMIPKit test"],
                ["git", "config", "user.email", "tests@example.invalid"],
                ["git", "add", "specification/oasis"],
                ["git", "commit", "--quiet", "-m", "pinned test evidence"],
            ):
                subprocess.run(arguments, cwd=root, check=True, capture_output=True)
            script = Path(__file__).resolve().parents[1] / "report.py"
            result = subprocess.run(
                [sys.executable, str(script), "--write", "--repo-root", str(root), "--structural-only"],
                cwd=root,
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            report = (root / "specification" / "catalog" / "coverage-report.md").read_text(encoding="utf-8")
            self.assertIn("KMIPKIT-SRC-profiles", report)
            self.assertIn("| Sources | 4 |", report)

    def test_script_entrypoint_does_not_echo_sensitive_values_from_path_errors(self) -> None:
        marker = "raw-kmip-body-path-sentinel-591"
        with tempfile.TemporaryDirectory() as directory:
            missing_root = Path(directory) / marker
            script = Path(__file__).resolve().parents[1] / "report.py"
            result = subprocess.run(
                [sys.executable, str(script), "--write", "--repo-root", str(missing_root)],
                cwd=ROOT,
                capture_output=True,
                check=False,
                text=True,
            )

        self.assertEqual(result.returncode, 1)
        self.assertNotIn(marker, result.stderr)
        self.assertNotIn(marker, result.stdout)

    def test_check_detects_stale_report_and_write_replaces_it(self) -> None:
        catalog = report_catalog()
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "coverage-report.md"
            target.write_text("stale\n", encoding="utf-8")
            self.assertFalse(write_report(catalog, target, check=True))
            self.assertTrue(write_report(catalog, target, check=False))
            self.assertEqual(target.read_text(encoding="utf-8"), render_report(catalog))
            self.assertTrue(write_report(catalog, target, check=True))

    @unittest.skipUnless(sys.platform != "win32", "POSIX directory symlinks are required")
    def test_write_report_accepts_a_resolved_temporary_parent_alias(self) -> None:
        catalog = report_catalog()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "canonical"
            root.mkdir()
            alias = Path(directory) / "alias"
            alias.symlink_to(root, target_is_directory=True)
            target = alias / "coverage-report.md"

            self.assertFalse(write_report(catalog, target, check=True))
            self.assertTrue(write_report(catalog, target, check=False))
            self.assertEqual(target.read_text(encoding="utf-8"), render_report(catalog))


if __name__ == "__main__":
    unittest.main()
