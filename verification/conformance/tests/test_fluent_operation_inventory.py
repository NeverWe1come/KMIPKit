"""The fluent roadmap inventory must use exact assigned KMIP operations."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from verification.conformance.generate_fluent_operation_inventory import build_inventory

ROOT = Path(__file__).resolve().parents[3]
BASE_CATALOG = ROOT / "verification/conformance/catalog/kmip-2.1.json"
VERSIONED_VALUES = ROOT / "verification/conformance/catalog/kmip-versioned-values.json"
MANIFEST = ROOT / "bindings/shared/public-api.json"
MODEL_OWNERS = ROOT / "verification/api/catalog/fluent-operation-model-owners.json"
OUTPUT = ROOT / "verification/api/catalog/fluent-operation-inventory.json"


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
            and "KMIP-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", [])
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

        version_2_1_additions = {
            (value["name"], value["wire_value"].removeprefix("0x").upper().zfill(8))
            for group in versioned["versions"]["v2_1"]
            if group["name"] == "Operation"
            for value in group["values"]
        }
        for row in base_rows:
            assigned_key = (row["name"], row["wireValue"])
            expected_earliest = "2.1" if assigned_key in version_2_1_additions else "2.0"
            self.assertEqual(row["earliestVersion"], expected_earliest)

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
        self.assertTrue(
            all(row["earliestVersion"] == "3.0" for row in operations if row["version"] == "3.0")
        )

    def test_server_initiated_operations_are_not_client_send_routes(self) -> None:
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        server_rows = [row for row in inventory["operations"] if row["direction"] == "server_to_client"]
        self.assertEqual(len(server_rows), 5)
        self.assertTrue(all(row["clientSend"] is False for row in server_rows))
        self.assertTrue(all(row["listenerRoute"] is True for row in server_rows))

    def test_every_operation_row_names_its_typed_request_and_response_owner(self) -> None:
        inventory = json.loads(OUTPUT.read_text(encoding="utf-8"))
        for row in inventory["operations"]:
            with self.subTest(version=row["version"], name=row["name"], direction=row["direction"]):
                for key in ("typedRequestOwner", "typedResponseOwner"):
                    owner = row[key]
                    self.assertEqual(set(owner), {"path", "type"})
                    self.assertRegex(
                        owner["path"],
                        r"^crates/kmipkit-protocol/src/(?:[a-z0-9_]+/)*[a-z0-9_]+\.rs$",
                    )
                    self.assertRegex(owner["type"], r"^[A-Za-z][A-Za-z0-9]*$")

                if row["version"] == "2.1":
                    for key in ("typedRequestOwner", "typedResponseOwner"):
                        owner = row[key]
                        source = (ROOT / owner["path"]).read_text(encoding="utf-8")
                        self.assertRegex(
                            source,
                            rf"pub (?:struct|enum|type) {owner['type']}\b",
                        )
                self.assertTrue(row["testEvidence"])
                self.assertTrue(all(isinstance(path, str) for path in row["testEvidence"]))
                if row["version"] == "2.1":
                    for path in row["testEvidence"]:
                        if (ROOT / "specs").is_dir() or not path.startswith("specs/"):
                            self.assertTrue((ROOT / path).exists(), path)

    def test_owner_evidence_matches_current_public_api_parity_evidence(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        parity_evidence = {
            operation["name"]: operation["parityEvidenceIds"]
            for operation in manifest["operations"]
        }

        for owner in owners["owners"]:
            if owner["version"] != "2.1" or owner["direction"] != "client_to_server":
                continue
            if owner["name"] not in parity_evidence:
                continue
            with self.subTest(operation=owner["name"]):
                self.assertEqual(owner["testEvidence"], parity_evidence[owner["name"]])

    def test_generator_rejects_missing_typed_owner_route(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        owners["owners"].pop()
        with self.assertRaisesRegex(ValueError, "typed operation owner coverage mismatch"):
            build_inventory(base, versioned, manifest, owners)

    def test_generator_rejects_duplicate_typed_owner_route(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        owners["owners"].append(owners["owners"][0])
        with self.assertRaisesRegex(ValueError, "duplicate typed operation owner route"):
            build_inventory(base, versioned, manifest, owners)

    def test_generator_rejects_owner_rows_without_test_evidence(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        owners["owners"][0]["testEvidence"] = []
        with self.assertRaisesRegex(ValueError, "test evidence is missing or invalid"):
            build_inventory(base, versioned, manifest, owners)

    def test_generator_rejects_stale_public_operation_test_evidence(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        owners["owners"][0]["testEvidence"].pop()
        with self.assertRaisesRegex(ValueError, "typed operation test evidence is stale"):
            build_inventory(base, versioned, manifest, owners)


    def test_generator_rejects_unmatched_kmip_2_1_version_delta(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        next(group for group in versioned["versions"]["v2_1"]
             if group["name"] == "Operation")["values"][0]["name"] = "Unassigned Operation"
        with self.assertRaisesRegex(ValueError, "KMIP 2.1 operation delta has no matching assigned value"):
            build_inventory(base, versioned, manifest, owners)

    def test_generator_rejects_one_wire_value_assigned_to_different_operations(self) -> None:
        base = json.loads(BASE_CATALOG.read_text(encoding="utf-8"))
        versioned = json.loads(VERSIONED_VALUES.read_text(encoding="utf-8"))
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        owners = json.loads(MODEL_OWNERS.read_text(encoding="utf-8"))
        create_value = next(
            element for element in base["elements"]
            if element.get("kind") == "enumeration_value"
            and element.get("name") == "Create"
            and "KMIP-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", [])
        )
        create_value["wire_value"] = "00000002"
        with self.assertRaisesRegex(ValueError, "duplicate KMIP 2.1 operation wire value"):
            build_inventory(base, versioned, manifest, owners)


if __name__ == "__main__":
    unittest.main()
