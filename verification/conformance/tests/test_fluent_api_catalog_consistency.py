"""Cross-check fluent API operation and tag inventories against pinned KMIP sources.

KMIP Specification v2.1 §11.36/Table 470 and §11.56/Table 487 define the
baseline operation and tag assignments. The pinned KMIP 3.0 draft §11.37/
Table 584 and §11.58/Table 605 define the seven operation and 60 tag deltas.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
BASE_CATALOG = ROOT / "verification/conformance/catalog/kmip-2.1.json"
VERSIONED_VALUES = ROOT / "verification/conformance/catalog/kmip-versioned-values.json"
VERSIONED_TAGS = ROOT / "verification/conformance/catalog/kmip-versioned-tags.json"
OPERATION_INVENTORY = ROOT / "verification/api/catalog/fluent-operation-inventory.json"


def _numeric(raw: str) -> int:
    return int(raw.removeprefix("0x").removeprefix("0X"), 16)


class FluentApiCatalogConsistencyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        cls.values = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        cls.tags = json.loads(VERSIONED_TAGS.read_text(encoding="utf-8"))
        cls.inventory = json.loads(OPERATION_INVENTORY.read_text(encoding="utf-8"))

    def test_operation_inventory_has_exact_assignments_direction_and_versions(self) -> None:
        base_operations = [
            row for row in self.base["elements"] if row.get("kind") == "operation"
        ]
        operation_values = {
            row["name"]: row["wire_value"]
            for row in self.base["elements"]
            if row.get("kind") == "enumeration_value"
            and "KMIP-ELEM-ENUMERATION-OPERATION"
            in row.get("parent_element_ids", [])
        }
        expected_v21 = {
            (
                row["name"],
                operation_values[row["name"]].upper(),
                row["direction"],
                row["scope_state"],
            )
            for row in base_operations
        }
        actual_v21 = {
            (row["name"], row["wireValue"], row["direction"], row["scopeState"])
            for row in self.inventory["operations"]
            if row["version"] == "2.1"
        }
        self.assertEqual(expected_v21, actual_v21)
        self.assertEqual(59, len({wire for _, wire, _, _ in expected_v21}))

        v30_groups = [
            group
            for group in self.values["versions"]["v3_0"]
            if group.get("kind") == "enumeration" and group.get("name") == "Operation"
        ]
        self.assertEqual(1, len(v30_groups))
        expected_v30 = {
            (row["name"], _numeric(row["wire_value"]))
            for row in v30_groups[0]["values"]
        }
        actual_v30 = {
            (row["name"], int(row["wireValue"], 16))
            for row in self.inventory["operations"]
            if row["version"] == "3.0"
        }
        self.assertEqual(
            {
                ("Create Group", 0x3C),
                ("Obliterate", 0x3D),
                ("Create User", 0x3E),
                ("Create Credential", 0x3F),
                ("Deactivate", 0x40),
                ("Encapsulate", 0x41),
                ("Decapsulate", 0x42),
            },
            expected_v30,
        )
        self.assertEqual(expected_v30, actual_v30)
        self.assertTrue(
            all(
                row["direction"] == "client_to_server"
                and row["clientSend"] is True
                and row["listenerRoute"] is False
                for row in self.inventory["operations"]
                if row["version"] == "3.0"
            )
        )

    def test_kmip_21_tag_assignments_match_the_normative_name_and_value_pairs(self) -> None:
        normative_tags = {
            _numeric(row["wire_value"]): row["name"]
            for row in self.base["elements"]
            if row.get("kind") == "tag"
        }
        assignments = self.tags["v2_1_tags"]
        self.assertEqual(36, len(assignments))
        for row in assignments:
            with self.subTest(tag=row["wire_value"]):
                value = _numeric(row["wire_value"])
                self.assertEqual(row["name"], normative_tags.get(value))
                self.assertEqual("2.1", row["introduced_in"])

    def test_kmip_30_tag_delta_contains_only_the_60_assigned_values(self) -> None:
        assignments = self.tags["tagEnumeration"]["assignments"]
        values = [_numeric(row["wire_value"]) for row in assignments]
        self.assertEqual(60, len(assignments))
        self.assertEqual(len(values), len(set(values)))
        self.assertTrue(all(row["introduced_in"] == "3.0" for row in assignments))
        base_values = {
            _numeric(row["wire_value"])
            for row in self.base["elements"]
            if row.get("kind") == "tag"
        }
        self.assertTrue(set(values).isdisjoint(base_values))


if __name__ == "__main__":
    unittest.main()
