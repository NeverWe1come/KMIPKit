"""Pin exact Rust-only request inspection helper exclusions.

Operation behavior remains available through the opaque four-language builders;
these methods only inspect local builder state or serialize a request payload.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
TRACEABILITY = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"
OWNER = "docs/adr/0006-public-api-and-bindings.md"
EXPECTED = {
    "rust-observed-5c3085bfc5d5023a37fa": "kmipkit_client::ClientBatchItem::request",
    "rust-observed-ba435cff9cb4abd90525": "kmipkit::ClientBatchItem::request",
    "rust-observed-92db6c972143c59934bc": "kmipkit_protocol::RegisterRequest::attributes",
    "rust-observed-7be9f190a261dc18fcc6": "kmipkit_protocol::RegisterRequest::into_ttlv_payload",
    "rust-observed-aa4bae39527938a24e12": "kmipkit_protocol::RegisterRequest::object",
    "rust-observed-f92cf0fe6010d7112b8c": "kmipkit_protocol::RegisterRequest::object_type",
    "rust-observed-7d893b054d3545a78661": "kmipkit_protocol::SplitKeyReference::split_key_threshold",
    "rust-observed-debf26c2f328bc05efb6": "kmipkit_protocol::SplitKeyReference::unique_identifier",
}


class T102OpaqueRequestHelperExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_exact_rust_builder_helpers_follow_opaque_handle_boundary(self) -> None:
        ids = set(EXPECTED)
        declarations = {
            row["declarationId"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("declarationId") in ids
        }
        self.assertEqual({}, declarations)
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("candidateSpecId") == "KMIPKIT-0043"
            and row.get("exportName") in EXPECTED.values()
        }
        self.assertEqual(set(EXPECTED.values()), set(exclusions))
        for identifier, export_name in EXPECTED.items():
            with self.subTest(export_name=export_name):
                record = exclusions[export_name]
                self.assertIn("opaque-handle boundary", record["reason"])
                self.assertEqual("accepted-adr", record["approvalSource"]["kind"])
                self.assertEqual(OWNER, record["approvalSource"]["path"])
                self.assertEqual("Accepted", record["approvalSource"]["status"])
                self.assertIn("Status: Accepted", record["approvalSource"]["evidence"])
                self.assertIn(identifier, record["evidence"])
                self.assertIn("ADR-0006", record["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", record["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", record["evidence"])

        # Excluding only inspection helpers must not remove Register or Join Split Key.
        required_operations = {
            "KMIPKIT-ELEM-OP-C2S-REGISTER",
            "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        }
        operations = {row["operationId"] for row in self.manifest["operations"]}
        self.assertTrue(required_operations.issubset(operations))
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)

    def test_traceability_records_request_readback_as_an_adr_exclusion(self) -> None:
        rows = TRACEABILITY.read_text(encoding="utf-8").splitlines()
        requirements = {
            "KMIPKIT-0043-FR-021": next(
                line for line in rows if line.startswith("| KMIPKIT-0043-FR-021 |")
            ),
            "KMIPKIT-0043-FR-022": next(
                line for line in rows if line.startswith("| KMIPKIT-0043-FR-022 |")
            ),
        }
        for requirement_id, row in requirements.items():
            with self.subTest(requirement_id=requirement_id):
                self.assertIn(
                    "ClientBatchItem::request readback exports are exact reviewed exclusions under ADR-0006",
                    row,
                )
                self.assertNotIn(
                    "ClientBatchItem::request readback exports remain blocked under KMIPKIT-0043",
                    row,
                )


if __name__ == "__main__":
    unittest.main()
