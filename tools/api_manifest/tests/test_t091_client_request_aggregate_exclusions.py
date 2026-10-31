"""T091 exclusions for Rust aggregate request shapes without foreign equivalents."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
AGGREGATE_EXPORTS = {
    (package_id, f"{prefix}{member}")
    for package_id, prefix in (
        ("rust-kmipkit-client", "kmipkit_client::"),
        ("rust-kmipkit", "kmipkit::"),
    )
    for member in (
        "ClientRequest",
        "ClientBatchItem::new",
        "ClientBatchItem::request",
    )
}
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}


class T091ClientRequestAggregateExclusionTests(unittest.TestCase):
    def test_generic_request_union_and_payload_accessor_are_exactly_excluded(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }

        for key in sorted(AGGREGATE_EXPORTS):
            with self.subTest(export=key):
                exclusion = exclusions.get(key)
                self.assertIsNotNone(exclusion, f"missing exact aggregate exclusion: {key}")
                self.assertNotIn(key, declarations)
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertEqual(
                    {field: APPROVAL_SOURCE[field] for field in ("kind", "path", "status")},
                    {
                        field: exclusion["approvalSource"][field]
                        for field in ("kind", "path", "status")
                    },
                )

        remaining = [
            row["exportName"]
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
        ]
        self.assertEqual([], remaining, f"unresolved KMIPKIT-0007 candidates: {remaining}")


if __name__ == "__main__":
    unittest.main()
