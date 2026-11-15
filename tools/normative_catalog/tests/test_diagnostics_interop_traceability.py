"""Source-to-implementation contracts for the approved diagnostics scope."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CATALOG = ROOT / "specification/catalog/kmip-2.1.json"
TRACE = ROOT / "specs/028-diagnostics-interop/traceability.md"
RESEARCH = ROOT / "specs/028-diagnostics-interop/research.md"
EXPECTED = {
    "KMIPKIT-REQ-SPEC-6.1.57-001": "req_001_models_one_chain_and_rejects_malformed_certificate_wrapper",
    "KMIPKIT-REQ-SPEC-6.1.57-002-001": "req_002_001_certificate_identifier_and_mixed_inputs_are_available",
    "KMIPKIT-REQ-SPEC-6.1.57-002-002": "req_002_002_optional_validity_date_preserves_exact_value",
    "KMIPKIT-REQ-SPEC-6.1.57-004": "req_004_repeated_certificate_values_preserve_order",
    "KMIPKIT-REQ-SPEC-6.1.57-005": "req_005_repeated_identifiers_preserve_order_and_open_values",
    "KMIPKIT-REQ-SPEC-6.1.57-006": "req_006_omitted_date_remains_absent_in_request_serialization",
}
UNSPECIFIED_ROLE_REQUIREMENTS = {"KMIPKIT-REQ-SPEC-6.1.57-006"}
PROFILELESS_FEATURE_REQUIREMENTS = {"KMIPKIT-REQ-SPEC-6.1.57-006"}
OMITTED_DATE_RESEARCH_CONTRACT = (
    "source actor remains unspecified",
    "Client verification covers preserving the omission in request serialization only",
    "date evaluation is unverified",
)
OMITTED_DATE_TRACEABILITY_CONTRACT = (
    "denotes feature/API serialization applicability",
    "evaluation is unverified",
    "req_006_omitted_date_remains_absent_in_request_serialization",
)


class DiagnosticsInteropTraceabilityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        self.trace = TRACE.read_text(encoding="utf-8")

    def test_validate_feature_requirement_roles_scope_and_traceability(self) -> None:
        records = {
            r["requirement_id"]: r
            for r in self.catalog["requirements"]
            if r["feature_spec"] == "KMIPKIT-0042"
        }
        self.assertEqual(set(records), set(EXPECTED))
        for identifier, test in EXPECTED.items():
            record = records[identifier]
            expected_role = "unspecified" if identifier in UNSPECIFIED_ROLE_REQUIREMENTS else "client"
            self.assertEqual(record["role"], expected_role)
            self.assertEqual(record["status"], "scoped_verified")
            self.assertEqual(record["scope_state"], "client_1_0")
            self.assertIn("KMIPKIT-ELEM-OP-C2S-VALIDATE", record["element_ids"])
            if identifier in PROFILELESS_FEATURE_REQUIREMENTS:
                self.assertEqual(record["profile_ids"], [])
            self.assertEqual(record["test_case_ids"], [])
            self.assertIn({"source_id": "KMIPKIT-SRC-spec", "section": "6.1.57"}, record["source_refs"])
            expected = f"crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs::{test}"
            self.assertIn(expected, record["verification_refs"])
            for reference in record["implementation_refs"] + record["verification_refs"]:
                path, _, symbol = reference.partition("::")
                source = ROOT / path
                self.assertTrue(source.is_file(), reference)
                if symbol:
                    self.assertRegex(source.read_text(encoding="utf-8"), rf"\bfn\s+{re.escape(symbol)}\s*\(")
            self.assertIn(identifier, self.trace)
            self.assertIn(test, self.trace)

    def test_operation_tables_codes_and_all_payload_rows_remain_mapped(self) -> None:
        elements = {r["element_id"]: r for r in self.catalog["elements"]}
        for operation, tables, code in [("INTEROP", [241, 242], "00000034"), ("LOG", [250, 251], "0000002C"), ("VALIDATE", [340, 341], "00000017")]:
            record = elements[f"KMIPKIT-ELEM-OP-C2S-{operation}"]
            self.assertEqual(record["feature_spec"], "KMIPKIT-0042")
            self.assertEqual([table["table_number"] for table in record["payload_tables"]], tables)
            self.assertTrue(record["implementation_refs"])
            self.assertTrue(record["verification_refs"])
            self.assertEqual(elements[f"KMIPKIT-ELEM-ENUM-VALUE-OPERATION-{operation}-{code}"]["wire_value"], code)
        for tag in ["420160", "420161", "420141", "420013", "420094", "42009A", "42009B"]:
            identifier = f"KMIPKIT-ELEM-TAG-{tag}"
            self.assertIn(identifier, self.trace)
            self.assertIn(identifier, elements)
        rows = re.findall(r"^\| KMIPKIT-028-T\d+-\w+", self.trace, re.MULTILINE)
        self.assertEqual(len(rows), 12)

    def test_omitted_validity_date_evidence_stays_actor_neutral(self) -> None:
        research = RESEARCH.read_text(encoding="utf-8")
        for phrase in OMITTED_DATE_RESEARCH_CONTRACT:
            self.assertIn(phrase, research)
        for phrase in OMITTED_DATE_TRACEABILITY_CONTRACT:
            self.assertIn(phrase, self.trace)
        self.assertNotIn("for_server_default", self.trace)

    def test_extension_and_unavailable_evidence_are_explicit(self) -> None:
        for operation in ["interop", "log", "validate"]:
            source = (ROOT / f"crates/kmipkit-protocol/src/{operation}.rs").read_text(encoding="utf-8")
            self.assertIn("crate::diagnostics::append_extensions", source)
        for text in ["KMIPKIT-DISC-017", "with_extension(Item)", "two-server", "official", "KMIPKIT-0043"]:
            self.assertIn(text, self.trace)


if __name__ == "__main__":
    unittest.main()
