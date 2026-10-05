"""Executable contract checks for dependency-policy requirement traceability."""

from __future__ import annotations

import ast
import csv
import re
import unittest
from collections import Counter
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SPECIFICATION = REPOSITORY_ROOT / "specs" / "011-dependency-policy" / "spec.md"
TRACEABILITY = REPOSITORY_ROOT / "specification" / "compliance" / "requirements" / "KMIPKIT-0011.csv"
CSV_COLUMNS = ("requirement_id", "configuration_path", "implementation_path", "test_reference")
CONFIGURATION_SUFFIXES = {".json", ".toml", ".yaml", ".yml"}
EXPECTED_REQUIREMENT_TESTS = {
    "FR-003": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_exception_free_config_must_preserve_policy_and_remove_waivers",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_local_cargo_deny_exception_file_is_rejected",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_uses_affected_crate_not_graph_parents",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_accepts_unpaired_license_help_summary_counts",
        )
    ),
    "FR-004": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyPolicyRunnerContractTests.test_runner_refreshes_rustsec_per_workspace_and_reports_sha_and_timestamp",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_exact_yanked_exception_covers_only_its_package_version",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_uses_affected_crate_not_graph_parents",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_exact_yanked_package_spec_can_be_ignored_by_the_reviewed_config",
        )
    ),
    "FR-005": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_optional_feature_only_external_path_dependency_is_rejected",
            "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_symlink_that_escapes_the_checkout_is_rejected_after_canonicalization",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unknown_registry_reports_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unapproved_local_git_source_reports_package_and_version",
        )
    ),
    "FR-006": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_architecture_ban_exceptions_cannot_waive_the_three_adr_0005_crates",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_rejects_unknown_errors_and_incomplete_json",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_duplicate_baseline_finding_covers_each_top_level_version_exactly",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_each_architecture_ban_reports_banned_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_wildcard_dependency_requirement_reports_exact_rule",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_normal_and_dev_duplicate_versions_report_both_versions",
        )
    ),
    "FR-007": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_exception_free_config_must_preserve_policy_and_remove_waivers",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_local_cargo_deny_exception_file_is_rejected",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_policy_config_rejects_unregistered_duplicate_skip_trees",
        )
    ),
    "FR-011": "scripts/tests/test_workflow.py::WorkflowContractTests.test_dependency_policy_local_command_and_review_process_are_documented",
    "FR-012": "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_failure_report_retains_allowlisted_finding_fields_and_redacts_secrets",
    "FR-013": "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions",
    "SC-004": "scripts/tests/test_workflow.py::WorkflowContractTests.test_scheduled_policy_reports_scanned_commit_and_each_rustsec_revision",
    "SC-006": "scripts/tests/test_workflow.py::WorkflowContractTests.test_dependency_policy_local_command_and_review_process_are_documented",
    "SC-008": "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions",
    "SC-002": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_uses_affected_crate_not_graph_parents",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_accepts_unpaired_license_help_summary_counts",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_baseline_parser_rejects_unknown_errors_and_incomplete_json",
            "scripts/tests/test_dependency_policy.py::CargoDenyDiagnosticTests.test_duplicate_baseline_finding_covers_each_top_level_version_exactly",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_exact_yanked_exception_covers_only_its_package_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_exact_yanked_package_spec_can_be_ignored_by_the_reviewed_config",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_vulnerability_reports_exact_vulnerability_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_unsoundness_reports_exact_unsound_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_unmaintained_reports_exact_unmaintained_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_yanked_registry_version_reports_exact_yanked_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_disallowed_license_reports_rejected_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unknown_registry_reports_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unapproved_local_git_source_reports_package_and_version",
            "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_optional_feature_only_external_path_dependency_is_rejected",
            "scripts/tests/test_dependency_policy.py::DependencyPolicyApiTests.test_symlink_that_escapes_the_checkout_is_rejected_after_canonicalization",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_each_architecture_ban_reports_banned_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_wildcard_dependency_requirement_reports_exact_rule",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_normal_and_dev_duplicate_versions_report_both_versions",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_expired_exception_is_rejected",
        )
    ),
    "SC-003": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyPolicyRunnerContractTests.test_runner_checks_the_python_metadata_validator_and_preserves_lockfiles",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_vulnerability_reports_exact_vulnerability_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_unsoundness_reports_exact_unsound_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_unmaintained_reports_exact_unmaintained_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_yanked_registry_version_reports_exact_yanked_code",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_disallowed_license_reports_rejected_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unknown_registry_reports_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_unapproved_local_git_source_reports_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_each_architecture_ban_reports_banned_package_and_version",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_wildcard_dependency_requirement_reports_exact_rule",
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_normal_and_dev_duplicate_versions_report_both_versions",
        )
    ),
    "SC-005": ";".join(
        (
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_valid_but_orphaned_exception_entry_is_rejected",
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_exact_yanked_exception_covers_only_its_package_version",
            "scripts/tests/test_dependency_policy.py::DependencyPolicyRunnerContractTests.test_runner_matches_waiver_free_findings_before_final_configured_scans",
        )
    ),
}


class RequirementTraceabilityTests(unittest.TestCase):
    def test_one_requirement_reference_can_cover_multiple_executable_tests(self) -> None:
        self.assert_executable_test_reference(
            "scripts/tests/test_cargo_deny_fixtures.py::CargoDenyNegativeFixtureTests.test_rustsec_vulnerability_reports_exact_vulnerability_code;"
            "scripts/tests/test_dependency_policy.py::DependencyExceptionTests.test_expired_exception_is_rejected"
        )

    def repository_file(self, relative_path: str, *, description: str) -> Path:
        path = Path(relative_path)
        self.assertFalse(path.is_absolute(), f"{description} must use a repository-relative path: {relative_path!r}")
        self.assertNotIn("..", path.parts, f"{description} must not traverse outside the repository: {relative_path!r}")
        resolved = (REPOSITORY_ROOT / path).resolve(strict=False)
        try:
            resolved.relative_to(REPOSITORY_ROOT.resolve())
        except ValueError:
            self.fail(f"{description} resolves outside the repository: {relative_path!r}")
        self.assertTrue(resolved.is_file(), f"{description} must point to an existing file: {relative_path!r}")
        return resolved

    def assert_executable_test_reference(self, reference: str) -> None:
        """Validate one or more semicolon-separated unittest references."""
        references = [item.strip() for item in reference.split(";")]
        self.assertTrue(references and all(references), "Test references must not be empty.")
        for single_reference in references:
            self._assert_single_executable_test_reference(single_reference)

    def _assert_single_executable_test_reference(self, reference: str) -> None:
        self.assertEqual(reference.count("::"), 1, f"Test reference must be path::TestClass.test_method: {reference!r}")
        relative_path, target = reference.split("::", maxsplit=1)
        self.assertTrue(
            relative_path.startswith("scripts/tests/test_") and relative_path.endswith(".py"),
            f"Test reference must point to a Python test module: {reference!r}",
        )
        test_path = self.repository_file(relative_path, description="Test reference")
        match = re.fullmatch(r"([A-Za-z_][A-Za-z0-9_]*)\.(test_[A-Za-z0-9_]*)", target)
        self.assertIsNotNone(match, f"Test reference must name a unittest test method: {reference!r}")
        assert match is not None
        class_name, method_name = match.groups()

        module = ast.parse(test_path.read_text(encoding="utf-8"), filename=relative_path)
        test_class = next(
            (
                node
                for node in module.body
                if isinstance(node, ast.ClassDef)
                and node.name == class_name
                and any(
                    (isinstance(base, ast.Name) and base.id == "TestCase")
                    or (
                        isinstance(base, ast.Attribute)
                        and base.attr == "TestCase"
                        and isinstance(base.value, ast.Name)
                        and base.value.id == "unittest"
                    )
                    for base in node.bases
                )
            ),
            None,
        )
        self.assertIsNotNone(test_class, f"Test reference must name a unittest.TestCase class: {reference!r}")
        assert test_class is not None
        self.assertEqual(
            sum(isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == method_name for node in test_class.body),
            1,
            f"Test reference must name one existing executable test_* method: {reference!r}",
        )

    def test_every_fr_and_sc_has_one_valid_traceability_row(self) -> None:
        specification = SPECIFICATION.read_text(encoding="utf-8")
        required_ids = set(re.findall(r"\*\*((?:FR|SC)-\d{3})\*\*", specification))
        self.assertTrue(required_ids, "The approved feature specification must declare FR and SC identifiers.")

        self.assertTrue(
            TRACEABILITY.is_file(),
            "The normative requirement traceability CSV must exist at "
            "specification/compliance/requirements/KMIPKIT-0011.csv.",
        )
        with TRACEABILITY.open(encoding="utf-8", newline="") as csv_file:
            reader = csv.DictReader(csv_file)
            self.assertEqual(
                tuple(reader.fieldnames or ()),
                CSV_COLUMNS,
                "CSV schema must be requirement_id,configuration_path,implementation_path,test_reference.",
            )
            rows = list(reader)

        rows_by_id = {row["requirement_id"]: row for row in rows}
        for requirement_id, expected_reference in EXPECTED_REQUIREMENT_TESTS.items():
            with self.subTest(requirement_test=requirement_id):
                self.assertEqual(
                    rows_by_id[requirement_id]["test_reference"],
                    expected_reference,
                    f"{requirement_id} must link to a test that exercises its acceptance criteria.",
                )

        observed_counts = Counter(row["requirement_id"] for row in rows)
        self.assertEqual(
            set(observed_counts),
            required_ids,
            "The traceability CSV must cover every FR/SC exactly, with no unknown identifiers.",
        )
        self.assertTrue(
            all(observed_counts[requirement_id] == 1 for requirement_id in required_ids),
            "Every FR/SC identifier must appear exactly once in the traceability CSV.",
        )

        for row in rows:
            requirement_id = row["requirement_id"]
            with self.subTest(requirement_id=requirement_id):
                for column in CSV_COLUMNS[1:]:
                    self.assertTrue(row[column].strip(), f"{requirement_id} must provide {column}.")

                configuration = self.repository_file(
                    row["configuration_path"], description=f"{requirement_id} configuration location"
                )
                self.assertIn(
                    configuration.suffix.lower(),
                    CONFIGURATION_SUFFIXES,
                    f"{requirement_id} configuration location must be a TOML, JSON, YAML, or workflow config file.",
                )

                implementation_path = row["implementation_path"]
                self.assertTrue(
                    implementation_path.startswith("scripts/") or implementation_path.startswith(".github/workflows/"),
                    f"{requirement_id} implementation location must identify a policy script or workflow.",
                )
                self.repository_file(implementation_path, description=f"{requirement_id} implementation location")
                self.assert_executable_test_reference(row["test_reference"])


if __name__ == "__main__":
    unittest.main()
