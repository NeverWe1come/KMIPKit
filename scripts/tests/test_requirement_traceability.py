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
    "FR-004": "scripts/tests/test_dependency_policy.py::DependencyPolicyRunnerContractTests.test_runner_refreshes_rustsec_per_workspace_and_reports_sha_and_timestamp",
    "FR-011": "scripts/tests/test_workflow.py::WorkflowContractTests.test_dependency_policy_local_command_and_review_process_are_documented",
    "SC-004": "scripts/tests/test_workflow.py::WorkflowContractTests.test_scheduled_policy_reports_scanned_commit_and_each_rustsec_revision",
    "SC-006": "scripts/tests/test_workflow.py::WorkflowContractTests.test_dependency_policy_local_command_and_review_process_are_documented",
}


class RequirementTraceabilityTests(unittest.TestCase):
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
