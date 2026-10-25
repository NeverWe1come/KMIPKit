"""Regression checks for the KMIPKIT-0036 catalog and traceability links."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
TRACEABILITY_PATH = ROOT / "specs/022-object-lifecycle/traceability.md"
SPEC_PATH = ROOT / "specs/022-object-lifecycle/spec.md"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FEATURE_SPEC = "KMIPKIT-0036"
OPERATIONS = {
    "Certify",
    "Check",
    "Obtain Lease",
    "Re-certify",
    "Re-key",
    "Re-key Key Pair",
    "Re-Provision",
    "Register",
    "Revoke",
}
CHECK_ATTRIBUTE_REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.7-002-003"
UNASSIGNED_RECERTIFY_REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.45-002-003"
SCOPED_WRAPPED_ATTRIBUTES_REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.43-005"
ABSENT_LINK_REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-4.31-002"
CHECK_ATTRIBUTE_ELEMENT_IDS = [
    "KMIPKIT-ELEM-OP-C2S-CHECK",
    "KMIPKIT-ELEM-TAG-42002C",
    "KMIPKIT-ELEM-TAG-420049",
    "KMIPKIT-ELEM-TAG-420096",
]


def _cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def _reference_file(reference: str) -> Path:
    return ROOT / reference.partition("::")[0]


class ObjectLifecycleTraceabilityTests(unittest.TestCase):
    def test_all_requirements_link_to_their_operation_and_sources(self) -> None:
        trace = TRACEABILITY_PATH.read_text(encoding="utf-8")
        spec = SPEC_PATH.read_text(encoding="utf-8")
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        elements = {record["element_id"]: record for record in catalog["elements"]}
        rows = [
            _cells(line)
            for line in trace.splitlines()
            if line.startswith("| KMIPKIT-REQ-SPEC-")
        ]

        spec_requirement_ids = {
            _cells(line)[0]
            for line in spec.splitlines()
            if line.startswith("| KMIPKIT-REQ-SPEC-")
        }
        trace_requirement_ids = {row[0] for row in rows}
        catalog_requirement_ids = {
            requirement_id
            for requirement_id, record in requirements.items()
            if record.get("feature_spec") == FEATURE_SPEC
        }

        self.assertEqual(len(spec_requirement_ids), 42)
        self.assertIn("KMIPKIT-REQ-SPEC-3.2-001", spec_requirement_ids)
        self.assertIn(ABSENT_LINK_REQUIREMENT_ID, spec_requirement_ids)
        self.assertEqual(len(rows), len(trace_requirement_ids))
        self.assertEqual(trace_requirement_ids, spec_requirement_ids)
        self.assertEqual(catalog_requirement_ids, spec_requirement_ids)

        operation_elements = [
            element
            for element in elements.values()
            if element.get("feature_spec") == FEATURE_SPEC
            and element.get("kind") == "operation"
        ]
        operation_element_ids = {element["element_id"] for element in operation_elements}
        self.assertEqual({element["name"] for element in operation_elements}, OPERATIONS)
        for row in rows:
            requirement = requirements[row[0]]
            self.assertEqual(requirement["feature_spec"], FEATURE_SPEC, row[0])
            if row[0] == UNASSIGNED_RECERTIFY_REQUIREMENT_ID:
                self.assertEqual(requirement["status"], "unassigned", row[0])
                self.assertEqual(requirement["implementation_refs"], [], row[0])
                self.assertEqual(requirement["verification_refs"], [], row[0])
                self.assertEqual(row[4], "unassigned", row[0])
                self.assertEqual(row[5:7], ["—", "—"], row[0])
            elif row[0] == SCOPED_WRAPPED_ATTRIBUTES_REQUIREMENT_ID:
                self.assertEqual(requirement["status"], "scoped_verified", row[0])
                self.assertEqual(row[4], "scoped_verified", row[0])
                self.assertTrue(requirement["implementation_refs"], row[0])
                self.assertTrue(requirement["verification_refs"], row[0])
                self.assertIn("wire preservation only", requirement["review_note"], row[0])
                self.assertIn("server acceptance", requirement["review_note"], row[0])
            elif row[0] == ABSENT_LINK_REQUIREMENT_ID:
                self.assertEqual(row[3], "Register (US8)", row[0])
                self.assertEqual(requirement["status"], "verified", row[0])
                self.assertEqual(requirement["element_ids"], ["KMIPKIT-ELEM-ATTRIBUTE-LINK"], row[0])
                self.assertTrue(requirement["implementation_refs"], row[0])
                self.assertTrue(requirement["verification_refs"], row[0])
                self.assertIn("Register", requirement["review_note"], row[0])
                self.assertIn("server acceptance", requirement["review_note"], row[0])
            else:
                self.assertIn(requirement["status"], {"verified", "scoped_verified"}, row[0])
            expected_operation_element_id = (
                f"KMIPKIT-ELEM-OP-C2S-{row[3].split('(')[0].strip().upper().replace(' ', '-')}"
            )
            if row[0] != ABSENT_LINK_REQUIREMENT_ID:
                self.assertIn(expected_operation_element_id, operation_element_ids, row[0])
            if row[0] == CHECK_ATTRIBUTE_REQUIREMENT_ID:
                self.assertEqual(row[3].split("(")[0].strip(), "Check", row[0])
                self.assertEqual(requirement["element_ids"], CHECK_ATTRIBUTE_ELEMENT_IDS, row[0])
            elif row[0] in {
                "KMIPKIT-REQ-SPEC-6.1.7-002-001",
                "KMIPKIT-REQ-SPEC-6.1.7-002-002",
            }:
                batch_option_element_id = (
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION"
                    if row[0] == "KMIPKIT-REQ-SPEC-6.1.7-002-001"
                    else "KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION"
                )
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        batch_option_element_id,
                        "KMIPKIT-ELEM-OP-C2S-CHECK",
                    ],
                    row[0],
                )
            elif row[0] == UNASSIGNED_RECERTIFY_REQUIREMENT_ID:
                self.assertEqual(
                    requirement["element_ids"],
                    ["KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER", "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"],
                    row[0],
                )
            elif row[0] == SCOPED_WRAPPED_ATTRIBUTES_REQUIREMENT_ID:
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA",
                        "KMIPKIT-ELEM-OP-C2S-REGISTER",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-VALUE",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-WRAPPING-DATA",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-ENCODING-OPTION",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-3.2-001":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES",
                        "KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE",
                        "KMIPKIT-ELEM-OP-C2S-REGISTER",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-1-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-6.1.44-003-002":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-ATTRIBUTE-COMPROMISE-OCCURRENCE-DATE",
                        "KMIPKIT-ELEM-ATTRIBUTE-REVOCATION-REASON",
                        "KMIPKIT-ELEM-ENUMERATION-REVOCATION-REASON-CODE",
                        "KMIPKIT-ELEM-OP-C2S-REVOKE",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-REASON-CODE",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-6.1.6-002-003":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-TYPE",
                        "KMIPKIT-ELEM-OP-C2S-CERTIFY",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-6.1.6-003":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-ID-PLACEHOLDER-00000001",
                        "KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM",
                        "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION",
                        "KMIPKIT-ELEM-OP-C2S-CERTIFY",
                        "KMIPKIT-ELEM-OP-C2S-GET",
                        "KMIPKIT-ELEM-TAG-420094",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-6.1.6-007":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-OP-C2S-CERTIFY",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE",
                    ],
                    row[0],
                )
            elif row[0] == "KMIPKIT-REQ-SPEC-6.1.45-008":
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE",
                        "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-VALUE",
                    ],
                    row[0],
                )
            elif row[0] != ABSENT_LINK_REQUIREMENT_ID:
                self.assertEqual(len(requirement["element_ids"]), 1, row[0])
                self.assertEqual(
                    requirement["element_ids"][0],
                    expected_operation_element_id,
                    row[0],
                )
            for path in (*requirement["implementation_refs"], *requirement["verification_refs"]):
                self.assertTrue(_reference_file(path).is_file(), f"{row[0]}: {path}")

        for element in operation_elements:
            self.assertTrue(element["requirement_ids"])
            for path in (*element["implementation_refs"], *element["verification_refs"]):
                self.assertTrue(
                    _reference_file(path).is_file(),
                    f"{element['element_id']}: {path}",
                )

    def test_source_rejected_candidates_are_not_assigned_to_client_operations(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        retired_ids = {
            "KMIPKIT-REQ-SPEC-6.1.44-001",
            "KMIPKIT-REQ-SPEC-6.1.45-002-002",
            "KMIPKIT-REQ-SPEC-6.1.45-006-002",
            "KMIPKIT-REQ-SPEC-6.1.48-004",
        }

        for requirement_id in retired_ids:
            self.assertEqual(requirements[requirement_id]["status"], "retired")
            self.assertIsNone(requirements[requirement_id]["feature_spec"])

        for element in catalog["elements"]:
            if element.get("feature_spec") == FEATURE_SPEC:
                self.assertFalse(retired_ids.intersection(element["requirement_ids"]))

    def test_every_payload_row_has_a_catalog_element_and_resolving_test_symbols(self) -> None:
        trace = TRACEABILITY_PATH.read_text(encoding="utf-8")
        spec = SPEC_PATH.read_text(encoding="utf-8")
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        element_ids = {record["element_id"] for record in catalog["elements"]}

        expected_rows = []
        current_operation = None
        operation_names = sorted(OPERATIONS, key=len, reverse=True)
        in_payload_schema = False
        payload_count = 0
        for line in spec.splitlines():
            if line.startswith("### User Story "):
                current_operation = next(
                    (name for name in operation_names if f"— {name} (Priority:" in line),
                    None,
                )
            if line.startswith("**Payload schema**"):
                in_payload_schema = True
                payload_count = 0
                continue
            if in_payload_schema and line.startswith(("| Request |", "| Response |")):
                cells = _cells(line)
                expected_rows.append((current_operation, cells[0], cells[1], cells[2]))
                payload_count += 1
            elif in_payload_schema and line.startswith("| ---") and payload_count:
                in_payload_schema = False

        mapped_rows = [
            _cells(line)
            for line in trace.splitlines()
            if re.match(r"^\| (?:" + "|".join(re.escape(name) for name in operation_names) + r") \| (?:Request|Response) Table ", line)
        ]
        self.assertEqual(len(expected_rows), 53)
        self.assertEqual(len(mapped_rows), len(expected_rows))
        self.assertEqual(
            [(row[0], row[1], row[2], row[3]) for row in mapped_rows],
            [(operation, f"{side} Table {self._table_number(trace, operation, side, field)}", field, required) for operation, side, field, required in expected_rows],
        )

        for row in mapped_rows:
            ids = re.findall(r"KMIPKIT-ELEM-[A-Z0-9-]+", row[4])
            self.assertTrue(ids, row[2])
            for element_id in ids:
                self.assertIn(element_id, element_ids, row[2])

        for path, symbol in re.findall(r"(crates/[^`\s;]+\.rs)::([A-Za-z0-9_]+)", trace):
            source = ROOT / path
            self.assertTrue(source.is_file(), path)
            contents = source.read_text(encoding="utf-8")
            self.assertRegex(contents, rf"\bfn\s+{re.escape(symbol)}\b", f"{path}::{symbol}")

    @staticmethod
    def _table_number(trace: str, operation: str, side: str, field: str) -> str:
        for line in trace.splitlines():
            if line.startswith(f"| {operation} | {side} Table "):
                cells = _cells(line)
                if cells[2] == field:
                    return cells[1].split()[2]
        raise AssertionError(f"no mapped table for {operation} {side} {field}")


if __name__ == "__main__":
    unittest.main()
