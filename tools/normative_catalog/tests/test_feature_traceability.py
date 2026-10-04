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
RESULT_ELEMENT_IDS = {
    "KMIPKIT-ELEM-ENUMERATION-RESULT-REASON",
    "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
    "KMIPKIT-ELEM-RESULT-RESULT-MESSAGE",
    "KMIPKIT-ELEM-RESULT-RESULT-REASON",
    "KMIPKIT-ELEM-RESULT-RESULT-STATUS",
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


class FeatureTraceabilityTests(unittest.TestCase):
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

if __name__ == "__main__":
    unittest.main()
