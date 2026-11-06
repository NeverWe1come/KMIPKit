"""Tests for the shared operationId-keyed parity fixture."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.operation_catalog_fixture import (
    find_language_value_mismatches,
    render_fixture,
)


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "operation_catalog_parity.tsv"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PYTHON_VALUES = ROOT / "bindings" / "python" / "src" / "kmipkit" / "protocol_values.py"
NORMATIVE_CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
C_OPERATION_MATRIX = ROOT / "bindings" / "c" / "tests" / "operation_parity_consumer.c"
C_ABI_CONSUMER = ROOT / "bindings" / "c" / "tests" / "client_abi_consumer.c"


def _python_operation_values() -> dict[str, int]:
    python_values = PYTHON_VALUES.read_text(encoding="utf-8")
    operation_enum = python_values[python_values.index("class Operation"):]
    operation_enum = operation_enum[:operation_enum.find("\nclass ", 1)]
    return {
        name: int(raw, 16)
        for name, raw in re.findall(
            r"^    ([A-Z][A-Z0-9_]*) = 0x([0-9a-fA-F]+)$",
            operation_enum,
            re.MULTILINE,
        )
    }


def _function_body(source: str, function_name: str) -> str:
    declaration = re.search(
        rf"static int {re.escape(function_name)}\s*\([^)]*\)\s*\{{", source
    )
    if declaration is None:
        raise AssertionError(f"missing C route helper: {function_name}")
    opening_brace = source.index("{", declaration.start())
    depth = 0
    for index in range(opening_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening_brace:index + 1]
    raise AssertionError(f"unterminated C route helper: {function_name}")


class OperationCatalogFixtureTests(unittest.TestCase):
    def test_language_enum_mismatch_is_reported_against_normative_catalog(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        catalog = json.loads(NORMATIVE_CATALOG.read_text(encoding="utf-8"))
        operation_values = _python_operation_values()
        operation_values["ACTIVATE"] += 1

        mismatches = find_language_value_mismatches(manifest, catalog, operation_values)

        self.assertEqual(len(mismatches), 1)
        self.assertIn("KMIPKIT-ELEM-OP-C2S-ACTIVATE", mismatches[0])
        self.assertIn("0x00000012", mismatches[0])
        self.assertIn("0x00000013", mismatches[0])

    def test_fixture_keys_every_client_operation_and_uses_its_raw_enum_value(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        normative_catalog = json.loads(NORMATIVE_CATALOG.read_text(encoding="utf-8"))
        operations = [
            operation
            for operation in manifest["operations"]
            if operation.get("scope") == "client_1_0"
        ]
        self.assertEqual(
            FIXTURE.read_text(encoding="utf-8"),
            render_fixture(manifest, normative_catalog),
        )
        expected_ids = {operation["operationId"] for operation in operations}
        rows: list[tuple[str, int]] = []
        lines = FIXTURE.read_text(encoding="utf-8").splitlines()
        self.assertEqual(lines[0], "operationId\toperationRaw")
        for line in lines[1:]:
            operation_id, raw = line.split("\t")
            rows.append((operation_id, int(raw)))

        self.assertEqual(len(rows), 56)
        self.assertEqual(len({operation_id for operation_id, _ in rows}), 56)
        self.assertEqual(len({raw for _, raw in rows}), 56)
        self.assertEqual({operation_id for operation_id, _ in rows}, expected_ids)

        self.assertEqual(
            find_language_value_mismatches(
                manifest, normative_catalog, _python_operation_values()
            ),
            [],
        )

    def test_c_route_matrix_joins_to_normative_values_including_four_extra_routes(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operations = [
            operation
            for operation in manifest["operations"]
            if operation.get("scope") == "client_1_0"
        ]
        operation_id_by_name = {
            operation["name"]: operation["operationId"] for operation in operations
        }
        operation_id_by_constructor: dict[str, str] = {}
        declarations = {
            declaration["declarationId"]: declaration
            for declaration in manifest["publicDeclarations"]
        }
        for operation in operations:
            for declaration_id in operation["declarations"]["c"]:
                declaration = declarations[declaration_id]
                constructor = declaration["languageMappings"]["c"]["name"]
                if (
                    declaration.get("kind") == "c-symbol"
                    and constructor.startswith("kmipkit_client_batch_item_")
                    and "_response_" not in constructor
                    and not constructor.endswith("_with_extensions_v1")
                ):
                    self.assertNotIn(constructor, operation_id_by_constructor)
                    operation_id_by_constructor[constructor] = operation["operationId"]

        special_builders = {
            "kmipkit_client_abi_c_consumer_build_add_attribute_request": "Add Attribute",
            "kmipkit_client_abi_c_consumer_build_adjust_attribute": "Adjust Attribute",
            "kmipkit_client_abi_c_consumer_build_delete_attribute": "Delete Attribute",
            "kmipkit_client_abi_c_consumer_build_modify_attribute": "Modify Attribute",
            "kmipkit_client_abi_c_consumer_build_set_attribute_request": "Set Attribute",
            "kmipkit_client_batch_item_discover_versions": "Discover Versions",
        }
        matrix_source = C_OPERATION_MATRIX.read_text(encoding="utf-8")
        route_region = matrix_source.split(
            "/* 52 operation-specific batch item constructors, each called from C. */", 1
        )[1].split("char executable[]", 1)[0]
        route_constructors = re.findall(
            r"KMIPKIT_TRY_(?:OPERATION|EXISTING_C_BUILDER)\s*\(\s*"
            r"(kmipkit_[A-Za-z0-9_]+)\s*\(",
            route_region,
        )
        route_ids = []
        for constructor in route_constructors:
            operation_name = special_builders.get(constructor)
            if operation_name is not None:
                route_ids.append(operation_id_by_name[operation_name])
            else:
                route_ids.append(operation_id_by_constructor[constructor])
        self.assertEqual(len(route_ids), 48)
        self.assertEqual(len(route_ids), len(set(route_ids)))

        fixture_values = {
            operation_id: int(raw)
            for operation_id, raw in (
                line.split("\t")
                for line in FIXTURE.read_text(encoding="utf-8").splitlines()[1:]
            )
        }
        array_source = matrix_source.split(
            "static const uint32_t EXPECTED_BATCH_OPERATIONS[] = {", 1
        )[1].split("};", 1)[0]
        array_values = [
            int(raw, 16)
            for raw in re.findall(r"UINT32_C\(0x([0-9A-Fa-f]+)\)", array_source)
        ]
        self.assertEqual(array_values, [fixture_values[operation_id] for operation_id in route_ids])

        c_consumer_source = C_ABI_CONSUMER.read_text(encoding="utf-8")
        extra_routes = (
            ("Interop", "interop_constructor_is_callable_from_c", "kmipkit_client_batch_item_interop"),
            (
                "Join Split Key",
                "join_split_key_constructor_pairs_each_identifier_with_its_threshold",
                "kmipkit_client_batch_item_join_split_key",
            ),
            ("Logout", "logout_constructor_is_callable_from_c", "kmipkit_client_batch_item_logout"),
            ("PKCS#11", "pkcs11_constructor_is_callable_from_c", "kmipkit_client_batch_item_pkcs11"),
        )
        extra_ids: list[str] = []
        for operation_name, helper_name, constructor in extra_routes:
            operation_id = operation_id_by_name[operation_name]
            body = _function_body(c_consumer_source, helper_name)
            self.assertRegex(body, rf"\b{re.escape(constructor)}\s*\(")
            expected_values = re.findall(
                r"test_encoded_operation_and_consume\s*\(\s*&item\s*,\s*"
                r"UINT32_C\(0x([0-9A-Fa-f]+)\)\s*\)",
                body,
            )
            self.assertEqual(len(expected_values), 1, operation_id)
            self.assertEqual(int(expected_values[0], 16), fixture_values[operation_id])
            extra_ids.append(operation_id)

        self.assertEqual(len(extra_ids), 4)
        self.assertEqual(len(set(route_ids + extra_ids)), 52)
        asynchronous_ids = {
            operation["operationId"]
            for operation in operations
            if operation["name"] in {
                "Cancel",
                "Poll",
                "Process",
                "Query Asynchronous Requests",
            }
        }
        self.assertEqual(set(route_ids + extra_ids), set(fixture_values) - asynchronous_ids)


if __name__ == "__main__":
    unittest.main()
