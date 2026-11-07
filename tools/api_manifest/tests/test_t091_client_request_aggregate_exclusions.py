"""T091 exclusions for Rust aggregate request shapes without foreign equivalents."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
AGGREGATE_EXCLUSIONS = {
    (package_id, f"{prefix}{member}")
    for package_id, prefix in (
        ("rust-kmipkit-client", "kmipkit_client::"),
        ("rust-kmipkit", "kmipkit::"),
    )
    for member in (
        "ClientRequest",
        "ClientBatchItem::new",
    )
}
REQUEST_READBACK_EXPORTS = {
    (package_id, f"{prefix}ClientBatchItem::request")
    for package_id, prefix in (
        ("rust-kmipkit-client", "kmipkit_client::"),
        ("rust-kmipkit", "kmipkit::"),
    )
}
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/043-public-api-bindings/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}


class T091ClientRequestAggregateExclusionTests(unittest.TestCase):
    def test_generic_request_union_and_constructor_are_exactly_excluded(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }

        for key in sorted(AGGREGATE_EXCLUSIONS):
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

    def test_typed_request_readback_has_exact_opaque_handle_exclusions(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }

        for key in sorted(REQUEST_READBACK_EXPORTS):
            with self.subTest(export=key):
                declaration = declarations.get(key)
                self.assertIsNone(declaration)
                exclusion = exclusions.get(key)
                self.assertIsNotNone(exclusion, f"missing exact request accessor exclusion: {key}")
                self.assertEqual("KMIPKIT-0043", exclusion["candidateSpecId"])
                self.assertEqual("accepted-adr", exclusion["approvalSource"]["kind"])
                self.assertEqual("docs/adr/0006-public-api-and-bindings.md", exclusion["approvalSource"]["path"])
                self.assertIn("opaque-handle boundary", exclusion["reason"])

        remaining = [
            row["exportName"]
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0007"
        ]
        self.assertEqual([], remaining, f"unresolved KMIPKIT-0007 candidates: {remaining}")


if __name__ == "__main__":
    unittest.main()
