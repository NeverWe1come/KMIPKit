"""The complete manifest operation list must match the normative catalog."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import generate
from tools.api_manifest.generate import ManifestError, _validate_operation_inventory
from tools.api_manifest.tests.test_manifest_v2 import _valid_v2_manifest


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
SCHEMA_PATH = ROOT / "specification" / "api" / "public-api.schema.json"


class OperationCatalogParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        cls.manifest = _valid_v2_manifest()

    def test_manifest_has_exact_numeric_catalog_operation_inventory(self) -> None:
        _validate_operation_inventory(self.manifest, self.catalog)

        operations = self.manifest["operations"]
        sections = [tuple(int(part) for part in row["section"].split(".")) for row in operations]
        self.assertEqual(len(operations), 56)
        self.assertEqual(sections, sorted(sections))
        self.assertEqual({row["scope"] for row in operations}, {"client_1_0"})

    def test_client_1_1_operations_are_not_in_the_1_0_api_inventory(self) -> None:
        endpoint_role = next(
            item
            for item in self.catalog["elements"]
            if item.get("kind") == "operation" and item.get("name") == "Set Endpoint Role"
        )

        self.assertEqual(endpoint_role["scope_state"], "client_1_1")
        self.assertNotIn(
            endpoint_role["element_id"],
            {row["operationId"] for row in self.manifest["operations"]},
        )

    def test_duplicate_missing_extra_and_out_of_scope_rows_are_rejected(self) -> None:
        cases = {}

        duplicate = copy.deepcopy(self.manifest)
        duplicate["operations"][-1] = copy.deepcopy(duplicate["operations"][0])
        cases["duplicate"] = duplicate

        missing = copy.deepcopy(self.manifest)
        missing["operations"].pop()
        cases["missing"] = missing

        extra = copy.deepcopy(self.manifest)
        extra["operations"][-1]["operationId"] = "KMIPKIT-ELEM-OP-OUTSIDE-CATALOG"
        cases["extra"] = extra

        out_of_scope = copy.deepcopy(self.manifest)
        out_of_scope["operations"][0]["scope"] = "server_only"
        cases["out of scope"] = out_of_scope

        for label, manifest in cases.items():
            with self.subTest(case=label), self.assertRaises(ManifestError):
                _validate_operation_inventory(manifest, self.catalog)

    def test_wrong_name_section_and_non_numeric_order_are_rejected(self) -> None:
        mutations = (
            ("name", lambda rows: rows[0].__setitem__("name", "Wrong Name")),
            ("section", lambda rows: rows[0].__setitem__("section", "6.1.999")),
            ("order", lambda rows: rows.reverse()),
        )
        for label, mutate in mutations:
            manifest = copy.deepcopy(self.manifest)
            mutate(manifest["operations"])
            with self.subTest(case=label), self.assertRaises(ManifestError):
                _validate_operation_inventory(manifest, self.catalog)

    def test_v2_validator_checks_catalog_cross_references_and_parity_scope(self) -> None:
        validator = getattr(generate, "_validate_v2_manifest", None)
        self.assertTrue(callable(validator), "v2 manifest validation is not wired")
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
        validator(manifest, schema, self.catalog, ROOT)

        invalid_catalog_copy = copy.deepcopy(manifest)
        invalid_catalog_copy["operations"][0]["name"] = "Wrong Name"
        with self.assertRaises(ManifestError):
            validator(invalid_catalog_copy, schema, self.catalog, ROOT)

        invalid_capability_copy = copy.deepcopy(manifest)
        declared = next(
            record
            for record in invalid_capability_copy["publicDeclarations"]
            if record["declarationState"] == "declared"
        )
        declared["capabilityId"] = "unknown_capability"
        with self.assertRaises(ManifestError):
            validator(invalid_capability_copy, schema, self.catalog, ROOT)

        invalid_owner_copy = copy.deepcopy(manifest)
        owned_declaration = next(
            record
            for record in invalid_owner_copy["publicDeclarations"]
            if record["declarationState"] == "declared"
        )
        owned_declaration["approvalSource"]["path"] = (
            "specs/007-client-execution/approval-record.md"
        )
        with self.assertRaises(ManifestError):
            validator(invalid_owner_copy, schema, self.catalog, ROOT)

        invalid_scope_copy = copy.deepcopy(manifest)
        java_declaration = next(
            record
            for record in invalid_scope_copy["publicDeclarations"]
            if record["packageId"] == "java-kmipkit"
        )
        java_declaration["parityScope"] = "rust-only-adr"
        with self.assertRaises(ManifestError):
            validator(invalid_scope_copy, schema, self.catalog, ROOT)


if __name__ == "__main__":
    unittest.main()
