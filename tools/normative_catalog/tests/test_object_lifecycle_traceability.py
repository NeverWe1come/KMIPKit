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


def _cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


class ObjectLifecycleTraceabilityTests(unittest.TestCase):
    def test_all_39_requirements_link_to_their_operation_and_sources(self) -> None:
        trace = TRACEABILITY_PATH.read_text(encoding="utf-8")
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

        self.assertEqual(len(rows), 39)
        self.assertEqual(len({row[0] for row in rows}), 39)
        self.assertEqual(sum(record.get("feature_spec") == FEATURE_SPEC for record in requirements.values()), 39)

        operation_elements = [
            element
            for element in elements.values()
            if element.get("feature_spec") == FEATURE_SPEC
            and element.get("kind") == "operation"
        ]
        self.assertEqual({element["name"] for element in operation_elements}, OPERATIONS)
        for row in rows:
            requirement = requirements[row[0]]
            self.assertEqual(requirement["feature_spec"], FEATURE_SPEC, row[0])
            self.assertIn(requirement["status"], {"verified", "scoped_verified"}, row[0])
            self.assertEqual(len(requirement["element_ids"]), 1, row[0])
            self.assertEqual(requirement["element_ids"][0], f"KMIPKIT-ELEM-OP-C2S-{row[3].split('(')[0].strip().upper().replace(' ', '-')}", row[0])
            for path in (*requirement["implementation_refs"], *requirement["verification_refs"]):
                self.assertTrue((ROOT / path).is_file(), f"{row[0]}: {path}")

        for element in operation_elements:
            self.assertTrue(element["requirement_ids"])
            for path in (*element["implementation_refs"], *element["verification_refs"]):
                self.assertTrue((ROOT / path).is_file(), f"{element['element_id']}: {path}")

    def test_retired_server_obligations_are_not_assigned_to_client_operations(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        retired_ids = {
            "KMIPKIT-REQ-SPEC-6.1.44-001",
            "KMIPKIT-REQ-SPEC-6.1.45-002-002",
            "KMIPKIT-REQ-SPEC-6.1.45-006-002",
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
