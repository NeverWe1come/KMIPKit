"""Require Log request members to map to the existing operation API."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import T102_BLOCKED_CANDIDATES

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/028-diagnostics-interop/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-LOG"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_log_request_members_mapping.py"
DECLARATIONS = (
    (
        "rust-observed-d8b817e7fba66b7889b4",
        "kmipkit_protocol::LogRequest::with_extension",
    ),
    (
        "rust-observed-e52c7650d6ca4e6ae9d3",
        "kmipkit_protocol::LogRequest::with_log_message",
    ),
)


class T102LogRequestMemberMappingTests(unittest.TestCase):
    def read_source(self, *parts: str) -> str:
        return ROOT.joinpath(*parts).read_text(encoding="utf-8")

    def assert_source_contains(self, parts: tuple[str, ...], *fragments: str) -> None:
        source = self.read_source(*parts)
        for fragment in fragments:
            with self.subTest(path="/".join(parts), fragment=fragment):
                self.assertIn(fragment, source)

    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.anchor = next(
            row
            for row in cls.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-protocol"
            and row["exportName"] == "kmipkit_protocol::LogRequest::new"
        )
        cls.builder = next(
            row
            for row in cls.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
            and row["exportName"] == "kmipkit_client_batch_item_log"
        )
        cls.operation = next(
            row
            for row in cls.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )

    def test_log_request_members_map_to_the_existing_four_language_builder(self) -> None:
        for declaration_id, name in DECLARATIONS:
            with self.subTest(declaration=name):
                declaration = self.declarations[declaration_id]
                expected_mappings = copy.deepcopy(self.builder["languageMappings"])
                expected_mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": name,
                }

                self.assertEqual(name, declaration["exportName"])
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual(self.anchor["requirementIds"], declaration["requirementIds"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
                self.assertIn(declaration_id, self.operation["declarations"]["rust"])
                self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_mapping_is_grounded_in_oasis_tables_redaction_and_binding_builders(self) -> None:
        self.assert_source_contains(
            tuple(OWNER.split("/")),
            "| Log | KMIPKIT-ELEM-OP-C2S-LOG | §6.1.29 | 250 | 251 | 252 |",
        )
        self.assert_source_contains(
            ("crates", "kmipkit-protocol", "tests", "unit", "log_operation_tests.rs"),
            "table_250_required_message_is_preserved_and_redacted",
            "unknown_request_extensions_preserve_order_without_overriding_table_fields",
        )
        self.assert_source_contains(
            ("bindings", "c", "include", "kmipkit.h"),
            "kmipkit_client_batch_item_log",
        )
        self.assert_source_contains(
            (
                "bindings",
                "java",
                "src",
                "main",
                "java",
                "org",
                "kmipkit",
                "extensions",
                "ClientBatchItem.java",
            ),
            "log(byte[] logMessage)",
        )
        self.assert_source_contains(
            ("bindings", "python", "src", "kmipkit", "extensions.py"),
            "def client_batch_item_log(",
        )

        self.assertEqual(
            T102_BLOCKED_CANDIDATES,
            sum(
                row.get("declarationState") == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in self.manifest["publicDeclarations"]
            ),
        )


if __name__ == "__main__":
    unittest.main()
