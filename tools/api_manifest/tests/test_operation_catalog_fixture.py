"""Tests for the shared operationId-keyed parity fixture."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "operation_catalog_parity.tsv"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PYTHON_VALUES = ROOT / "bindings" / "python" / "src" / "kmipkit" / "protocol_values.py"


class OperationCatalogFixtureTests(unittest.TestCase):
    def test_fixture_keys_every_client_operation_and_uses_its_raw_enum_value(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operations = [
            operation
            for operation in manifest["operations"]
            if operation.get("scope") == "client_1_0"
        ]
        expected_ids = {operation["operationId"] for operation in operations}
        rows: list[tuple[str, int]] = []
        for line in FIXTURE.read_text(encoding="utf-8").splitlines()[1:]:
            operation_id, raw = line.split("\t")
            rows.append((operation_id, int(raw)))

        self.assertEqual(len(rows), 56)
        self.assertEqual(len({operation_id for operation_id, _ in rows}), 56)
        self.assertEqual({operation_id for operation_id, _ in rows}, expected_ids)

        python_values = PYTHON_VALUES.read_text(encoding="utf-8")
        operation_enum = python_values[python_values.index("class Operation"):]
        operation_enum = operation_enum[:operation_enum.find("\nclass ", 1)]
        enum_values = {
            name: int(raw, 16)
            for name, raw in re.findall(
                r"^    ([A-Z][A-Z0-9_]*) = 0x([0-9a-fA-F]+)$",
                operation_enum,
                re.MULTILINE,
            )
        }
        by_id = {operation["operationId"]: operation for operation in operations}
        for operation_id, raw in rows:
            operation_name = by_id[operation_id]["name"]
            enum_name = re.sub(r"[^A-Za-z0-9]+", "_", operation_name).upper()
            with self.subTest(operation_id=operation_id):
                self.assertEqual(raw, enum_values[enum_name])


if __name__ == "__main__":
    unittest.main()
