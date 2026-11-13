"""The fluent roadmap inventory must use exact assigned KMIP operations."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE_CATALOG = ROOT / "specification/catalog/kmip-2.1.json"
VERSIONED_VALUES = ROOT / "specification/catalog/kmip-versioned-values.json"
from tools.normative_catalog.generate_fluent_operation_inventory import build_inventory, render, _validate_evidence_files

OUTPUT = ROOT / "specification/catalog/fluent-operation-inventory.json"


class FluentOperationInventoryTests(unittest.TestCase):
    def test_inventory_covers_exact_assigned_operation_values_and_directions(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        self.assertEqual(inventory["formatVersion"], 2)
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

    def test_every_directional_row_has_source_version_owners_route_and_evidence(self) -> None:
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        for row in inventory["operations"]:
            with self.subTest(name=row["name"], direction=row["direction"]):
                self.assertIn("earliestSupportedVersion", row)
                self.assertIn(row["earliestSupportedVersion"], {"2.0", "2.1", "3.0"})
                self.assertTrue(row["versionSourceRefs"])
                self.assertEqual(row["typedOwners"]["state"], "available")
                self.assertTrue(row["typedOwners"]["request"]["path"])
                self.assertTrue(row["typedOwners"]["response"]["symbol"])
                self.assertEqual(row["route"]["kind"],
                                 "client" if row["clientSend"] else "listener")
                self.assertIn(row["route"]["state"], {"implemented", "unverified"})
                self.assertEqual(row["route"]["versionVerification"], "not_established")
                self.assertTrue(row["testEvidence"]["focused"])
                self.assertIs(row["serverCapabilityClaim"], False)

    def test_direction_specific_version_sources_do_not_use_code_introduction(self) -> None:
        rows = json.loads(OUTPUT.read_text(encoding="utf-8"))["operations"]
        for direction, section in [("client_to_server", "6.1.48"),
                                   ("server_to_client", "6.2.5")]:
            row = next(row for row in rows if row["name"] == "Set Endpoint Role"
                       and row["direction"] == direction)
            self.assertEqual(row.get("earliestSupportedVersion"), "2.0")
            self.assertEqual(row["versionSourceRefs"][0]["section"], section)
        ping = next(row for row in rows if row["name"] == "Ping")
        self.assertEqual(ping["earliestSupportedVersion"], "2.1")

    def test_reserved_assignment_is_rejected(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        for element in base["elements"]:
            if element.get("kind") == "enumeration_value" and element.get("name") == "Create" \
                    and "KMIPKIT-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", []):
                element["wire_value"] = "00000043"
        with self.assertRaisesRegex(ValueError, "assignment"):
            build_inventory(base, json.loads(VERSIONED_VALUES.read_text()),
                            json.loads((ROOT / "specification/api/public-api.json").read_text()))

    def test_invalid_direction_is_rejected(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        next(element for element in base["elements"]
             if element.get("kind") == "operation")["direction"] = "both"
        with self.assertRaisesRegex(ValueError, "direction"):
            build_inventory(base, json.loads(VERSIONED_VALUES.read_text()),
                            json.loads((ROOT / "specification/api/public-api.json").read_text()))

    def test_generated_inventory_is_deterministic_and_current(self) -> None:
        self.assertEqual(render(ROOT), OUTPUT.read_bytes())
        self.assertEqual(render(ROOT), render(ROOT))

    def _build_with_evidence(self, evidence: dict) -> dict:
        return build_inventory(json.loads(BASE_CATALOG.read_text()),
                               json.loads(VERSIONED_VALUES.read_text()),
                               json.loads((ROOT / "specification/api/public-api.json").read_text()),
                               evidence)

    def test_missing_owner_cannot_claim_implemented_route(self) -> None:
        evidence = json.loads((ROOT / "specification/catalog/operation-matrix-evidence.json").read_text())
        evidence["operations"][0]["typedOwners"]["request"] = None
        with self.assertRaisesRegex(ValueError, "missing typed owners"):
            self._build_with_evidence(evidence)

    def test_unverified_route_is_explicit_and_not_executable(self) -> None:
        evidence = json.loads((ROOT / "specification/catalog/operation-matrix-evidence.json").read_text())
        detail = evidence["operations"][0]
        detail["route"]["state"] = "unverified"
        detail["route"]["symbol"] = None
        row = next(row for row in self._build_with_evidence(evidence)["operations"]
                   if row["name"] == detail["name"] and row["direction"] == detail["direction"])
        self.assertFalse(row["clientSend"])
        self.assertFalse(row["listenerRoute"])
        self.assertEqual(row["route"]["state"], "unverified")

    def test_duplicate_or_missing_directional_evidence_is_rejected(self) -> None:
        evidence = json.loads((ROOT / "specification/catalog/operation-matrix-evidence.json").read_text())
        for mutation in (lambda rows: rows.append(copy.deepcopy(rows[0])),
                         lambda rows: rows.pop()):
            invalid = copy.deepcopy(evidence)
            mutation(invalid["operations"])
            with self.assertRaisesRegex(ValueError, "evidence"):
                self._build_with_evidence(invalid)

    def test_fabricated_owner_route_test_and_directional_source_are_rejected(self) -> None:
        inventory = json.loads(OUTPUT.read_text())
        for field in ("owner", "route", "test", "source", "earliest"):
            with self.subTest(field=field):
                invalid = copy.deepcopy(inventory)
                row = invalid["operations"][0]
                if field == "owner":
                    row["typedOwners"]["request"]["symbol"] = "kmipkit_protocol::InventedRequest"
                elif field == "route":
                    row["route"]["symbol"] = "kmipkit_client::Client::invented_send"
                elif field == "test":
                    row["testEvidence"]["focused"][0] += "_invented"
                elif field == "source":
                    row["versionSourceRefs"][0]["section"] = "6.2.1"
                else:
                    row["earliestSupportedVersion"] = "2.1"
                with self.assertRaises(ValueError):
                    _validate_evidence_files(ROOT, invalid)

    def test_unassigned_kmip30_operation_is_rejected(self) -> None:
        versioned = json.loads(VERSIONED_VALUES.read_text())
        next(group for group in versioned["versions"]["v3_0"]
             if group["name"] == "Operation")["values"][0]["wire_value"] = "0x00000043"
        with self.assertRaisesRegex(ValueError, "assignment"):
            build_inventory(json.loads(BASE_CATALOG.read_text()), versioned,
                            json.loads((ROOT / "specification/api/public-api.json").read_text()))


if __name__ == "__main__":
    unittest.main()
