"""Tests for feature ownership of implemented protocol elements."""

from __future__ import annotations

import csv
import json
import re
import tempfile
import unittest
from pathlib import Path, PurePosixPath, PureWindowsPath


ROOT = Path(__file__).resolve().parents[3]
MAX_TEST_SOURCE_BYTES = 1_048_576
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FEATURE_SPEC = "KMIPKIT-0003"
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
LIFECYCLE_PLANNED_CODE_PATHS = {
    "crates/kmipkit-protocol/src/activate.rs",
    "crates/kmipkit-protocol/src/archive.rs",
    "crates/kmipkit-protocol/src/destroy.rs",
    "crates/kmipkit-protocol/src/recover.rs",
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


def _read_confined_test_source(test_path: str) -> tuple[Path, str] | None:
    posix_path = PurePosixPath(test_path)
    windows_path = PureWindowsPath(test_path)
    if (
        not test_path
        or "\\" in test_path
        or posix_path.is_absolute()
        or windows_path.is_absolute()
        or windows_path.drive
        or ".." in posix_path.parts
    ):
        return None

    try:
        root = ROOT.resolve(strict=True)
        path = (root / test_path).resolve(strict=True)
        path.relative_to(root)
        if not path.is_file() or path.suffix not in {".py", ".rs"}:
            return None
        with path.open("rb") as source_file:
            source_bytes = source_file.read(MAX_TEST_SOURCE_BYTES + 1)
        if len(source_bytes) > MAX_TEST_SOURCE_BYTES:
            return None
        source = source_bytes.decode("utf-8")
    except (OSError, RuntimeError, ValueError, UnicodeDecodeError):
        return None
    return path, source


def _markdown_catalog_rows(document: str) -> dict[str, list[str]]:
    rows_by_id: dict[str, list[str]] = {}
    for line in document.splitlines():
        if not line.startswith("| `"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        match = re.search(r"`([^`]+)`", cells[0])
        if match:
            rows_by_id[match.group(1)] = cells
    return rows_by_id


class FeatureTraceabilityTests(unittest.TestCase):
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
                self.assertEqual(elements_by_id[element_id]["feature_spec"], "KMIPKIT-0018")
                expected_source = f"KMIP 2.1 §{section} Tables {tables}"
                self.assertEqual(rows_by_id[element_id][1], expected_source)
                self.assertIn(f"§{section} | Tables {tables}", specification)

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
            ("KMIPKIT-REQ-SPEC-9.11-010-001", "deferred"),
        ):
            self.assertEqual(by_id[requirement_id]["status"], status)
        self.assertIn("OD-003", by_id["KMIPKIT-REQ-SPEC-9.11-010-001"]["scope"])
        self.assertIn("caller", by_id["KMIPKIT-REQ-SPEC-9.11-004-002"]["statement"].lower())
        self.assertIn("comparison scope", by_id["KMIPKIT-REQ-SPEC-9.11-004-002"]["statement"].lower())

        for row in rows:
            if row["requirement_id"] in {
                "KMIPKIT-REQ-SPEC-9.4-001-003",
                "KMIPKIT-REQ-SPEC-9.11-010-001",
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

    def test_result_response_clauses_and_elements_use_server_to_client_direction(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        response_clause_ids = {
            "KMIPKIT-CLAUSE-SPEC-9.17-001",
            "KMIPKIT-CLAUSE-SPEC-9.18-001",
            "KMIPKIT-CLAUSE-SPEC-9.19-001",
        }
        clauses_by_id = {clause["clause_id"]: clause for clause in catalog["source_clauses"]}
        self.assertTrue(response_clause_ids.issubset(clauses_by_id))
        for clause_id in sorted(response_clause_ids):
            with self.subTest(clause_id=clause_id):
                self.assertEqual(clauses_by_id[clause_id]["direction"], "server_to_client")

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

    @staticmethod
    def _is_executable_test_ref(test_ref: str) -> bool:
        test_path, separator, test_name = test_ref.partition("::")
        if not separator or not test_name:
            return False

        source_file = _read_confined_test_source(test_path)
        if source_file is None:
            return False
        path, source = source_file

        if path.suffix == ".rs":
            return f"fn {test_name}(" in source
        if path.suffix == ".py":
            class_name, separator, function_name = test_name.rpartition(".")
            if not separator:
                return False
            class_exists = re.search(rf"^class {re.escape(class_name)}(?:\(|:)", source, re.MULTILINE)
            return class_exists is not None and f"def {function_name}(" in source
        return False


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
        self.assertEqual(global_field_order["status"], "unassigned")
        self.assertIsNone(global_field_order["feature_spec"])

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
        self.assertTrue(source_requirements.issubset(by_id))
        self.assertEqual(
            by_id["KMIPKIT-REQ-SPEC-8-003-002"]["status"],
            "verified",
        )
        self.assertIn(
            "one_response_batch_can_mix_completed_and_pending_results",
            by_id["KMIPKIT-REQ-SPEC-8-003-002"]["test_ids"],
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
            else:
                self.assertEqual(record["feature_spec"], row["scope"].split("; owner=")[1])
            for reference in record["verification_refs"]:
                test_path = reference.split("::", 1)[0]
                self.assertTrue((ROOT / test_path).is_file(), reference)
                if "::" in reference:
                    self.assertTrue(self._is_executable_test_ref(reference), reference)

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
                    self.assertEqual(record["implementation_refs"], [])
                    self.assertEqual(record["verification_refs"], [])
                    self.assertIn("OD-004", record["review_note"])
                    self.assertIn("countdown-derived", record["review_note"])
                    self.assertEqual(row["status"], "deferred")
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
