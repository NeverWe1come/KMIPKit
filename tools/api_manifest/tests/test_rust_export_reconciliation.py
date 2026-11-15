"""Exact blocked-manifest reconciliation for compiler-observed Rust exports."""

from __future__ import annotations

import hashlib
import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "reconcile_rust_exports.py"
SPEC = importlib.util.spec_from_file_location("reconcile_rust_exports", MODULE_PATH)
RECONCILER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RECONCILER)


def blocked_owner(export_name: str, candidate_spec_id: str) -> dict[str, object]:
    return {
        "declarationId": f"owner-{candidate_spec_id}",
        "packageId": "rust-kmipkit-protocol",
        "kind": "type",
        "exportName": export_name,
        "capabilityId": "public_binding_surface",
        "requirementIds": ["KMIPKIT-0043-FR-021"],
        "declarationState": "blocked",
        "parityScope": "four-language",
        "candidateSpecId": candidate_spec_id,
        "blockedReason": "The observed Rust declaration lacks a reviewed four-language mapping in the public API manifest.",
    }


def observed(package_id: str, export_name: str, kind: str = "associated-item") -> dict[str, str]:
    return {"packageId": package_id, "exportName": export_name, "kind": kind}


class RustExportReconciliationTests(unittest.TestCase):
    def test_export_key_is_the_package_and_export_path_pair(self):
        row = observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::method")

        self.assertEqual(
            RECONCILER._export_key(row),
            (row.get("packageId"), row.get("exportName")),
        )

    def test_trait_member_inherits_only_the_exact_manifest_owner_candidate(self):
        manifest = {
            "capabilities": [],
            "approvedExclusions": [],
            "publicDeclarations": [
                blocked_owner("kmipkit_protocol::Owner", "KMIPKIT-0017"),
            ],
        }
        record = RECONCILER.blocked_record(
            observed(
                "rust-kmipkit-protocol",
                "<kmipkit_protocol::Owner as core::fmt::Debug>::fmt",
            ),
            manifest,
        )

        self.assertEqual(record.get("candidateSpecId"), "KMIPKIT-0017")
        self.assertEqual(record.get("capabilityId"), "public_binding_surface")
        self.assertEqual(record.get("requirementIds"), ["KMIPKIT-0043-FR-021"])
        self.assertEqual(record.get("declarationState"), "blocked")
        self.assertNotIn("languageMappings", record)
        self.assertNotIn("approvalSource", record)
        expected_id = hashlib.sha256(
            b"rust-kmipkit-protocol\0<kmipkit_protocol::Owner as core::fmt::Debug>::fmt"
        ).hexdigest()[:20]
        self.assertEqual(record.get("declarationId"), f"rust-observed-{expected_id}")

    def test_alias_projected_member_uses_the_alias_record_as_its_owner(self):
        manifest = {
            "capabilities": [],
            "approvedExclusions": [],
            "publicDeclarations": [
                blocked_owner("kmipkit_protocol::Owner", "KMIPKIT-0017"),
                blocked_owner("kmipkit_protocol::Alias", "KMIPKIT-0042"),
            ],
        }

        record = RECONCILER.blocked_record(
            observed(
                "rust-kmipkit-protocol",
                "<kmipkit_protocol::Alias as core::fmt::Debug>::fmt",
            ),
            manifest,
        )

        self.assertEqual(record.get("candidateSpecId"), "KMIPKIT-0042")

    def test_inherent_member_uses_the_nearest_exact_manifest_owner(self):
        manifest = {
            "capabilities": [],
            "approvedExclusions": [],
            "publicDeclarations": [
                blocked_owner("kmipkit_protocol::Owner", "KMIPKIT-0017"),
            ],
        }

        record = RECONCILER.blocked_record(
            observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::method"),
            manifest,
        )

        self.assertEqual(record.get("candidateSpecId"), "KMIPKIT-0017")

    def test_unknown_owner_uses_the_existing_public_surface_candidate_without_mappings(self):
        manifest = {"capabilities": [], "approvedExclusions": [], "publicDeclarations": []}

        record = RECONCILER.blocked_record(
            observed("rust-kmipkit-protocol", "<external::Hidden as core::fmt::Debug>::fmt"),
            manifest,
        )

        self.assertEqual(record.get("candidateSpecId"), "KMIPKIT-0043")
        self.assertIn("no resolved owner", record.get("blockedReason", ""))
        self.assertNotIn("languageMappings", record)

    def test_reconciliation_adds_only_exact_sorted_blocked_records(self):
        manifest = {
            "capabilities": [],
            "approvedExclusions": [],
            "publicDeclarations": [blocked_owner("kmipkit_protocol::Owner", "KMIPKIT-0017")],
        }
        report = {
            "formatVersion": 1,
            "rustdocFormat": 57,
            "crates": ["kmipkit_protocol"],
            "missingFromManifest": [
                observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::z", "associated-item"),
                observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::a", "associated-item"),
            ],
            "issues": [{"reason": "unresolved-rustdoc-implementation-predicate"}],
        }
        issues_before = list(report["issues"])

        added = RECONCILER.reconcile(manifest, report)

        self.assertEqual(
            [record["exportName"] for record in added],
            ["kmipkit_protocol::Owner::a", "kmipkit_protocol::Owner::z"],
        )
        self.assertEqual(manifest["publicDeclarations"][-2:], added)
        self.assertTrue(all(record["declarationState"] == "blocked" for record in added))
        self.assertTrue(all("languageMappings" not in record for record in added))
        self.assertEqual(report["issues"], issues_before)

    def test_duplicate_report_path_is_rejected_before_manifest_mutation(self):
        manifest = {"capabilities": [], "approvedExclusions": [], "publicDeclarations": []}
        report = {
            "formatVersion": 1,
            "rustdocFormat": 57,
            "crates": ["kmipkit_protocol"],
            "missingFromManifest": [
                observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::method"),
                observed("rust-kmipkit-protocol", "kmipkit_protocol::Owner::method"),
            ],
            "issues": [],
        }

        with self.assertRaisesRegex(ValueError, "duplicate observed Rust path"):
            RECONCILER.reconcile(manifest, report)
        self.assertEqual(manifest["publicDeclarations"], [])


if __name__ == "__main__":
    unittest.main()
