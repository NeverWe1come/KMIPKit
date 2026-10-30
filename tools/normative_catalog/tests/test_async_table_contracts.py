"""Asynchronous operation table fields use source contracts, not invented REQs."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.validate import validate_source_table_contracts


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"

REVIEWED_ASYNC_CONTRACT_SEMANTICS = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.39-ASYNC-CORRELATION-VALUE-REQUIRED": {
        "requiredness": "Yes",
        "actor": "client",
        "direction": "include the Asynchronous Correlation Value in the Process request payload",
        "scope_disposition": "client_1_0",
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-ASYNC-CORRELATION-VALUES-OPTIONAL": {
        "requiredness": "No",
        "actor": "client",
        "direction": "optional Asynchronous Correlation Values request payload value",
        "scope_disposition": "client_1_0",
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-OPERATIONS-OPTIONAL": {
        "requiredness": "No",
        "actor": "client",
        "direction": "optional Operations request payload value",
        "scope_disposition": "client_1_0",
    },
}


def validate_with_catalog_fixture(oracle: object, catalog: dict[str, object]) -> list[str]:
    """Run the source-table validator against an isolated catalog file copy."""
    original_read_text = Path.read_text
    with tempfile.TemporaryDirectory() as temporary_directory:
        fixture_path = Path(temporary_directory) / "kmip-2.1.json"
        fixture_path.write_text(json.dumps(catalog), encoding="utf-8")
        catalog_path = CATALOG_PATH.resolve()

        def read_text(path: Path, *args: object, **kwargs: object) -> str:
            if path.resolve() == catalog_path:
                return original_read_text(fixture_path, *args, **kwargs)
            return original_read_text(path, *args, **kwargs)

        with patch.object(Path, "read_text", new=read_text):
            return validate_source_table_contracts(oracle, ROOT)


def source_request_rows_for_gap(oracle: dict[str, object], gap_id: str) -> list[dict[str, object]]:
    return [
        row
        for review in oracle["source_requirement_reviews"]
        for row in review.get("source_request_rows", [])
        if row.get("source_gap_id") == gap_id
    ]


class AsyncTableContractTests(unittest.TestCase):
    def test_process_and_query_async_request_rows_are_feature_contracts(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        expected = {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.39-ASYNC-CORRELATION-VALUE-REQUIRED": (
                "6.1.39", "Asynchronous Correlation Value", [31618, 31636], 278,
                "KMIPKIT-ELEM-OP-C2S-PROCESS", ["FR-005"],
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-ASYNC-CORRELATION-VALUES-OPTIONAL": (
                "6.1.41", "Asynchronous Correlation Values", [32456, 32474], 285,
                "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", ["FR-006"],
            ),
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.41-OPERATIONS-OPTIONAL": (
                "6.1.41", "Operations", [32475, 32492], 285,
                "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", ["FR-006"],
            ),
        }
        contracts = {
            row["gap_id"]: row
            for row in oracle["source_table_contracts"]
            if row.get("gap_id") in expected
        }
        gaps = {
            row["gap_id"]: row
            for row in oracle["source_only_requirement_gaps"]
            if row.get("gap_id") in expected
        }

        self.assertEqual(set(contracts), set(expected))
        self.assertEqual(gaps, {})
        operations = {row["element_id"]: row for row in catalog["elements"] if row.get("kind") == "operation"}
        for operation_id in {
            "KMIPKIT-ELEM-OP-C2S-PROCESS",
            "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS",
        }:
            with self.subTest(operation_id=operation_id):
                self.assertEqual(operations[operation_id]["feature_spec"], "KMIPKIT-0009")
        for gap_id, (section, label, line_range, table_number, operation_id, fr_ids) in expected.items():
            row = contracts[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(row["source_section"], section)
                self.assertEqual(row["source_locator"]["row_label"], label)
                self.assertEqual(row["html_line_range"], line_range)
                self.assertEqual(row["table_number"], table_number)
                self.assertEqual(row["operation_element_id"], operation_id)
                self.assertEqual(row["functional_requirement_ids"], fr_ids)
                self.assertEqual(
                    {field: row[field] for field in REVIEWED_ASYNC_CONTRACT_SEMANTICS[gap_id]},
                    REVIEWED_ASYNC_CONTRACT_SEMANTICS[gap_id],
                )
                source_rows = source_request_rows_for_gap(oracle, gap_id)
                self.assertEqual(len(source_rows), 1)
                self.assertEqual(row["requiredness"], source_rows[0]["required"])
                self.assertEqual(row["actor"], source_rows[0]["actor"])
                self.assertEqual(
                    row["scope_disposition"],
                    operations[operation_id]["scope_state"],
                )
                self.assertEqual(row["status"], "mapped_to_approved_feature_contract")
                self.assertNotIn("candidate_requirement_id", row)
                self.assertNotIn("proposed_requirement_id", row)
                self.assertNotIn("requirement_id", row)

    def test_source_table_validator_rejects_wrong_async_contract_semantics(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        self.assertEqual(validate_source_table_contracts(oracle, ROOT), [])
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))

        for index, contract in enumerate(oracle["source_table_contracts"]):
            gap_id = contract.get("gap_id")
            if gap_id not in REVIEWED_ASYNC_CONTRACT_SEMANTICS:
                continue
            source_rows = source_request_rows_for_gap(oracle, gap_id)
            self.assertEqual(len(source_rows), 1)
            source_row = source_rows[0]
            operation = next(
                row for row in catalog["elements"]
                if row.get("element_id") == contract["operation_element_id"]
            )
            wrong_values = (
                (
                    "requiredness",
                    "No" if source_row["required"] == "Yes" else "Yes",
                ),
                ("actor", "server" if source_row["actor"] == "client" else "client"),
                (
                    "scope_disposition",
                    "client_1_1" if operation["scope_state"] == "client_1_0" else "client_1_0",
                ),
            )
            for field, wrong_value in wrong_values:
                with self.subTest(gap_id=gap_id, field=field):
                    mutated_oracle = copy.deepcopy(oracle)
                    mutated_contract = next(
                        row for row in mutated_oracle["source_table_contracts"]
                        if row.get("gap_id") == gap_id
                    )
                    mutated_contract[field] = wrong_value

                    diagnostics = validate_source_table_contracts(mutated_oracle, ROOT)
                    contract_label = f"source_table_contracts[{index}]"
                    self.assertTrue(
                        any(
                            contract_label in diagnostic and f".{field}" in diagnostic
                            for diagnostic in diagnostics
                        ),
                        f"validator accepted wrong {field} for {gap_id}: {diagnostics!r}",
                    )

    def test_source_table_validator_checks_linked_operation_and_scope_metadata(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        operation_ids = (
            "KMIPKIT-ELEM-OP-C2S-PROCESS",
            "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS",
        )

        for operation_id in operation_ids:
            affected_contracts = [
                (index, contract)
                for index, contract in enumerate(oracle["source_table_contracts"])
                if contract.get("operation_element_id") == operation_id
            ]
            self.assertTrue(affected_contracts)
            for mutation in ("direction", "request_table_role", "scope_state"):
                with self.subTest(operation_id=operation_id, mutation=mutation):
                    mutated_catalog = copy.deepcopy(catalog)
                    operation = next(
                        row for row in mutated_catalog["elements"]
                        if row.get("element_id") == operation_id
                    )
                    if mutation == "direction":
                        operation["direction"] = "server_to_client"
                    elif mutation == "request_table_role":
                        request_table = next(
                            table for table in operation["payload_tables"]
                            if table.get("role") == "request"
                        )
                        request_table["role"] = "response"
                    else:
                        operation["scope_state"] = (
                            "client_1_1"
                            if operation["scope_state"] == "client_1_0"
                            else "client_1_0"
                        )

                    diagnostics = validate_with_catalog_fixture(oracle, mutated_catalog)
                    for contract_index, contract in affected_contracts:
                        contract_label = f"source_table_contracts[{contract_index}]"
                        diagnostic_fragment = (
                            "scope_disposition"
                            if mutation == "scope_state"
                            else "operation_element_id must reference a client-to-server operation with the request table"
                        )
                        with self.subTest(contract_gap_id=contract["gap_id"]):
                            self.assertTrue(
                                any(
                                    contract_label in diagnostic and diagnostic_fragment in diagnostic
                                    for diagnostic in diagnostics
                                ),
                                f"validator accepted linked operation {mutation} for {contract['gap_id']}: {diagnostics!r}",
                            )


if __name__ == "__main__":
    unittest.main()
