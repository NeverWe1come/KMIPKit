"""The fluent roadmap inventory must use exact assigned KMIP operations."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE_CATALOG = ROOT / "specification/catalog/kmip-2.1.json"
VERSIONED_VALUES = ROOT / "specification/catalog/kmip-versioned-values.json"
OUTPUT = ROOT / "specification/catalog/fluent-operation-inventory.json"


class FluentOperationInventoryTests(unittest.TestCase):
    def test_inventory_covers_exact_assigned_operation_values_and_directions(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        self.assertEqual(inventory["formatVersion"], 1)
        operations = inventory["operations"]
        self.assertEqual(len(operations), 69)
        keys = [(row["version"], row["wireValue"], row["direction"]) for row in operations]
        self.assertEqual(len(keys), len(set(keys)))

        base_rows = [
            row for row in operations if row["version"] == "2.1"
        ]
        operation_values = {
            element["name"]: element["wire_value"]
            for element in base["elements"]
            if element.get("kind") == "enumeration_value"
            and "KMIPKIT-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", [])
        }
        expected = {
            (element["name"], operation_values[element["name"]], element["direction"], element["scope_state"])
            for element in base["elements"]
            if element.get("kind") == "operation"
        }
        actual = {
            (row["name"], row["wireValue"], row["direction"], row["scopeState"])
            for row in base_rows
        }
        self.assertEqual(actual, expected)
        self.assertEqual(len(base_rows), 62)

        expected_3 = {
            (value["name"], value["wire_value"])
            for group in versioned["versions"]["v3_0"]
            if group["name"] == "Operation"
            for value in group["values"]
        }
        expected_3 = {
            (name, raw.removeprefix("0x").upper().zfill(8))
            for name, raw in expected_3
        }
        actual_3 = {
            (row["name"], row["wireValue"])
            for row in operations
            if row["version"] == "3.0"
        }
        self.assertEqual(actual_3, expected_3)
        self.assertEqual(
            {row["name"]: row["wireValue"] for row in operations if row["version"] == "3.0"},
            {
                "Create Group": "0000003C",
                "Obliterate": "0000003D",
                "Create User": "0000003E",
                "Create Credential": "0000003F",
                "Deactivate": "00000040",
                "Encapsulate": "00000041",
                "Decapsulate": "00000042",
            },
        )

    def test_server_initiated_operations_are_not_client_send_routes(self) -> None:
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        server_rows = [row for row in inventory["operations"] if row["direction"] == "server_to_client"]
        self.assertEqual(len(server_rows), 5)
        self.assertTrue(all(row["clientSend"] is False for row in server_rows))
        self.assertTrue(all(row["listenerRoute"] is True for row in server_rows))


if __name__ == "__main__":
    unittest.main()
