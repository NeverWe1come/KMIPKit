"""Exact T091 exclusions for Rust-only ClientError constructors and payload fields."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"

ERROR_DECLARATIONS = {
    (package_id, f"{prefix}ClientError::{member}")
    for package_id, prefix in (
        ("rust-kmipkit-client", "kmipkit_client::"),
        ("rust-kmipkit", "kmipkit::"),
    )
    for member in (
        "validation",
        "protocol",
        "transport",
        "server_result",
        "server_operation_result",
        "server_status",
        "profile_validation",
        "Protocol::error",
        "ProtocolResponse::error",
        "ServerResult::0",
    )
}


class T091ClientErrorExclusionTests(unittest.TestCase):
    def test_error_constructors_and_payload_fields_have_exact_exclusions(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }

        for key in sorted(ERROR_DECLARATIONS):
            with self.subTest(export=key):
                exclusion = exclusions.get(key)
                self.assertIsNotNone(exclusion, f"missing exact approved exclusion: {key}")
                self.assertNotIn(key, declarations, "an export cannot be both declared and excluded")
                self.assertIn("KMIPKIT-0007-FR-013", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/007-client-execution/spec.md",
                        "status": "Approved",
                    },
                    {
                        field: exclusion["approvalSource"][field]
                        for field in ("kind", "path", "status")
                    },
                )

    def test_safe_error_metadata_and_scoped_response_ttlv_remain_mapped(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }

        expected = {
            ("rust-kmipkit-client", "kmipkit_client::ClientError::category"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::cause_category"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::delivery_state"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::profile_validation_error"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::response_ttlv"),
            ("rust-kmipkit-client", "kmipkit_client::ClientError::ProtocolResponse::response_ttlv"),
            ("rust-kmipkit", "kmipkit::ClientError::category"),
            ("rust-kmipkit", "kmipkit::ClientError::cause_category"),
            ("rust-kmipkit", "kmipkit::ClientError::delivery_state"),
            ("rust-kmipkit", "kmipkit::ClientError::profile_validation_error"),
            ("rust-kmipkit", "kmipkit::ClientError::response_ttlv"),
            ("rust-kmipkit", "kmipkit::ClientError::ProtocolResponse::response_ttlv"),
        }

        for key in sorted(expected):
            with self.subTest(export=key):
                self.assertEqual("declared", declarations[key]["declarationState"])
                self.assertNotIn("candidateSpecId", declarations[key])


if __name__ == "__main__":
    unittest.main()
