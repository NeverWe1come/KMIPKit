"""Exact T102 exclusions for root SecretBytes standard-trait artifacts."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
OWNER_SPEC = "specs/008-credentials-attestation/spec.md"
TRAIT_PREFIX = "<kmipkit::SecretBytes as "
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": OWNER_SPEC,
    "status": "Approved",
    "evidence": (
        "**Status**: Approved for implementation under the delegated "
        "authorization recorded in [approval-record.md](approval-record.md)."
    ),
}
EXPECTED_DIRECT = {
    "<kmipkit::SecretBytes as alloc::string::ToString>::to_string",
    "<kmipkit::SecretBytes as core::any::Any>::type_id",
    "<kmipkit::SecretBytes as core::borrow::Borrow<kmipkit_protocol::credential::secret::SecretBytes>>::borrow",
    "<kmipkit::SecretBytes as core::borrow::BorrowMut<kmipkit_protocol::credential::secret::SecretBytes>>::borrow_mut",
    "<kmipkit::SecretBytes as core::convert::From<kmipkit_protocol::credential::secret::SecretBytes>>::from",
    "<kmipkit::SecretBytes as core::fmt::Debug>::fmt",
    "<kmipkit::SecretBytes as core::fmt::Display>::fmt",
}
EXPECTED_CONDITIONAL = {
    "<kmipkit::SecretBytes as core::convert::Into<U>>::into",
    "<kmipkit::SecretBytes as core::convert::TryFrom<U>>::Error",
    "<kmipkit::SecretBytes as core::convert::TryFrom<U>>::try_from",
    "<kmipkit::SecretBytes as core::convert::TryInto<U>>::Error",
    "<kmipkit::SecretBytes as core::convert::TryInto<U>>::try_into",
}
EXPECTED_FUNCTIONAL_EXPORTS = {
    "kmipkit::SecretBytes",
    "kmipkit::SecretBytes::new",
    "kmipkit::SecretBytes::with_bytes",
    "kmipkit::SecretBytes::into_ttlv_value",
}
EXPECTED_LANGUAGE_MAPPINGS = {
    "rust": ("rust-kmipkit", "kmipkit::SecretBytes"),
    "c": ("c-kmipkit", "kmipkit_secret_bytes_t"),
    "java": ("java-kmipkit", "org.kmipkit.credentials.SecretBytes"),
    "python": ("python-kmipkit", "kmipkit.credentials.SecretBytes"),
}


class T102SecretBytesTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_only_exact_compiler_trait_paths_are_excluded_under_secret_owner(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertEqual(self.rust_report["rustdocFormat"], 57)
        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(self.rust_report["issues"], [])

        expected = EXPECTED_DIRECT | EXPECTED_CONDITIONAL
        self.assertEqual(len(EXPECTED_DIRECT), 7)
        self.assertEqual(len(EXPECTED_CONDITIONAL), 5)

        reported_direct = {
            row["exportName"]
            for row in self.rust_report["declarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        reported_conditional = {
            row["exportName"]
            for row in self.rust_report["conditionalDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        self.assertEqual(reported_direct, EXPECTED_DIRECT)
        self.assertEqual(reported_conditional, EXPECTED_CONDITIONAL)

        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName", "").startswith(TRAIT_PREFIX)
        }
        self.assertEqual(set(declarations), set())
        self.assertEqual(set(exclusions), expected)

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0008")
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE)
                self.assertIn("standard-trait implementation artifact", row["reason"])
                self.assertIn("KMIPKIT-0008-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                self.assertNotIn("KMIPKIT-0043-FR-007", row["evidence"])
                self.assertIn("No OASIS clause applies", row["evidence"])
                self.assertIn("inventory-only", row["evidence"])

    def test_secret_bytes_parent_and_lifecycle_surface_remain_mapped(self) -> None:
        protocol_surface = {
            "kmipkit_protocol::SecretBytes",
            "kmipkit_protocol::SecretBytes::new",
            "kmipkit_protocol::SecretBytes::with_bytes",
            "kmipkit_protocol::SecretBytes::into_ttlv_value",
        }
        expected_surfaces = {
            "rust-kmipkit": EXPECTED_FUNCTIONAL_EXPORTS,
            "rust-kmipkit-protocol": protocol_surface,
        }

        for package_id, expected_exports in expected_surfaces.items():
            rows = {
                row["exportName"]: row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package_id
                and row.get("exportName") in expected_exports
            }
            with self.subTest(package_id=package_id):
                self.assertEqual(set(rows), expected_exports)
                for export_name, row in rows.items():
                    self.assertEqual(row["declarationState"], "declared")
                    self.assertNotIn("candidateSpecId", row)
                    self.assertEqual(row["ownerSpecId"], "KMIPKIT-0008")
                    self.assertIn("KMIPKIT-0008-FR-010", row["requirementIds"])
                    self.assertEqual(row["approvalSource"]["path"], OWNER_SPEC)
                    self.assertEqual(row["approvalSource"]["status"], "Approved")

        root_parent = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName") == "kmipkit::SecretBytes"
        )
        self.assertEqual(
            {
                language: (mapping["packageId"], mapping["name"])
                for language, mapping in root_parent["languageMappings"].items()
            },
            EXPECTED_LANGUAGE_MAPPINGS,
        )

        protocol_traits = {
            row["exportName"]
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(
                "<kmipkit_protocol::SecretBytes as "
            )
        }
        self.assertEqual(len(protocol_traits), 12)


if __name__ == "__main__":
    unittest.main()
