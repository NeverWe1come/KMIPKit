"""Regression checks for KMIPKIT-0037 catalog and executable traceability."""

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
TRACEABILITY = ROOT / "specs" / "023-key-material-transfer" / "traceability.md"
FEATURE = "KMIPKIT-0037"

REQUIREMENT_IDS = (
    "KMIPKIT-REQ-SPEC-6.1.14-001-001",
    "KMIPKIT-REQ-SPEC-6.1.14-001-002",
    "KMIPKIT-REQ-SPEC-6.1.14-001-003",
    "KMIPKIT-REQ-SPEC-6.1.14-001-004",
    "KMIPKIT-REQ-SPEC-6.1.14-002",
    "KMIPKIT-REQ-SPEC-6.1.14-006-001",
    "KMIPKIT-REQ-SPEC-6.1.14-006-002",
    "KMIPKIT-REQ-SPEC-6.1.14-007-001",
    "KMIPKIT-REQ-SPEC-6.1.14-007-002",
    "KMIPKIT-REQ-SPEC-7.12-003",
    "KMIPKIT-REQ-SPEC-7.12-005-001",
    "KMIPKIT-REQ-SPEC-6.1.25-006",
    "KMIPKIT-REQ-SPEC-6.1.27-001",
    "KMIPKIT-REQ-SPEC-6.1.27-002-002",
    "KMIPKIT-REQ-SPEC-6.1.27-005",
)


class KeyDerivationTransferTraceabilityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.requirements = {
            item["requirement_id"]: item for item in cls.catalog["requirements"]
        }
        cls.elements = {item["element_id"]: item for item in cls.catalog["elements"]}

    def test_all_stable_requirements_link_to_code_and_executable_verification(self) -> None:
        for requirement_id in REQUIREMENT_IDS:
            with self.subTest(requirement_id=requirement_id):
                requirement = self.requirements[requirement_id]
                self.assertEqual(requirement["feature_spec"], FEATURE)
                self.assertNotEqual(requirement["status"], "unassigned")
                self.assertTrue(requirement["implementation_refs"])
                self.assertTrue(requirement["verification_refs"])
                for reference in requirement["verification_refs"]:
                    path, separator, test_name = reference.partition("::")
                    self.assertTrue(separator, reference)
                    source = (ROOT / path).read_text(encoding="utf-8")
                    self.assertRegex(source, rf"\bfn\s+{re.escape(test_name)}\s*\(")

    def test_operation_and_import_recover_elements_capture_the_implementation(self) -> None:
        operation_ids = (
            "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY",
            "KMIPKIT-ELEM-OP-C2S-EXPORT",
            "KMIPKIT-ELEM-OP-C2S-IMPORT",
            "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        )
        for element_id in operation_ids:
            with self.subTest(element_id=element_id):
                element = self.elements[element_id]
                self.assertEqual(element["feature_spec"], FEATURE)
                if element_id != "KMIPKIT-ELEM-OP-C2S-EXPORT":
                    self.assertTrue(element["requirement_ids"])
                self.assertTrue(element["implementation_refs"])
                self.assertTrue(element["verification_refs"])

        import_value = self.elements["KMIPKIT-ELEM-ENUM-VALUE-OPERATION-IMPORT-0000002A"]
        recover_value = self.elements["KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RECOVER-00000016"]
        self.assertEqual(import_value["wire_value"], "0000002A")
        self.assertEqual(recover_value["wire_value"], "00000016")
        self.assertTrue(recover_value["verification_refs"])

    def test_traceability_rows_use_executable_references(self) -> None:
        lines = TRACEABILITY.read_text(encoding="utf-8").splitlines()
        for requirement_id in REQUIREMENT_IDS:
            with self.subTest(requirement_id=requirement_id):
                row = next(line for line in lines if requirement_id in line)
                self.assertIn("crates/", row)
                self.assertIn("::", row)


if __name__ == "__main__":
    unittest.main()
