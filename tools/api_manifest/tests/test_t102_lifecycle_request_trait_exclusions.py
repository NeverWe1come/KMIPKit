"""Pin exact T102 trait exclusions for six approved lifecycle requests."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = ROOT / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-11-01.json"
APPROVAL = {"kind": "feature-specification", "path": "specs/022-object-lifecycle/spec.md", "status": "Approved", "evidence": "**Status**: Approved for implementation"}
OWNER_REQS = ("KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004")

TYPES = {
    "CheckRequest": {
        "module": "check", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.7, Tables 183-185", "request": "Table 183 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.7-001", "KMIPKIT-REQ-SPEC-6.1.7-002-001", "KMIPKIT-REQ-SPEC-6.1.7-002-002", "KMIPKIT-REQ-SPEC-6.1.7-002-003", "KMIPKIT-REQ-SPEC-6.1.7-003", "KMIPKIT-REQ-SPEC-6.1.7-004-001", "KMIPKIT-REQ-SPEC-6.1.7-004-002", "KMIPKIT-REQ-SPEC-6.1.7-005"),
        "states": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "blocked", "::unique_identifier": "blocked", "::with_cryptographic_usage_mask": "blocked", "::with_lease_time": "blocked", "::with_unique_identifier": "blocked", "::with_usage_limits_count": "blocked"},
    },
    "ObtainLeaseRequest": {
        "module": "obtain_lease", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.35, Tables 268-270", "request": "Table 268 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.35-001-001", "KMIPKIT-REQ-SPEC-6.1.35-001-002", "KMIPKIT-REQ-SPEC-6.1.35-002", "KMIPKIT-REQ-SPEC-6.1.35-005"),
        "states": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "verified", "::unique_identifier": "blocked", "::with_unique_identifier": "verified"},
    },
    "ReProvisionRequest": {
        "module": "reprovision", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.48 Tables 313-314; Section 6.1.48.1 Table 315", "request": "Table 313 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.48-001", "KMIPKIT-REQ-SPEC-6.1.48-006"),
        "caption_note": "Table 315 appears in the Re-Provision Error Handling body immediately before Section 6.1.49; its printed caption incorrectly reads 'RNG Retrieve Errors'.",
        "states": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "verified", "::with_certificate": "verified", "::with_certificate_request": "verified"},
    },
    "RecertifyRequest": {
        "module": "recertify", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.45, Tables 300-302", "request": "Table 300 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.45-001", "KMIPKIT-REQ-SPEC-6.1.45-002-001", "KMIPKIT-REQ-SPEC-6.1.45-002-003", "KMIPKIT-REQ-SPEC-6.1.45-004", "KMIPKIT-REQ-SPEC-6.1.45-006-001", "KMIPKIT-REQ-SPEC-6.1.45-008"),
        "states": {"": "verified", "::certificate_request_unique_identifier": "blocked", "::into_ttlv_payload": "blocked", "::new": "blocked", "::unique_identifier": "blocked", "::with_attributes": "blocked", "::with_certificate_request_type": "blocked", "::with_certificate_request_unique_identifier": "blocked", "::with_certificate_request_value": "blocked", "::with_offset": "blocked", "::with_protection_storage_masks": "blocked", "::with_unique_identifier": "blocked"},
    },
    "RekeyKeyPairRequest": {
        "module": "rekey_key_pair", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.47, Tables 310-312", "request": "Table 310 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.47-001", "KMIPKIT-REQ-SPEC-6.1.47-004-001"),
        "states": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "blocked", "::private_key_unique_identifier": "blocked", "::with_common_attributes": "blocked", "::with_common_protection_storage_masks": "blocked", "::with_offset": "blocked", "::with_private_key_attributes": "blocked", "::with_private_key_unique_identifier": "blocked", "::with_private_protection_storage_masks": "blocked", "::with_public_key_attributes": "blocked", "::with_public_protection_storage_masks": "blocked"},
    },
    "RekeyRequest": {
        "module": "rekey", "section": "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.46, Tables 305-307", "request": "Table 305 as request",
        "normative": ("KMIPKIT-REQ-SPEC-6.1.46-001", "KMIPKIT-REQ-SPEC-6.1.46-004-001"),
        "states": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "blocked", "::unique_identifier": "blocked", "::with_attributes": "blocked", "::with_offset": "blocked", "::with_protection_storage_masks": "blocked", "::with_unique_identifier": "blocked"},
    },
}


def direct_traits(public: str, implementation: str) -> set[str]:
    return {
        f"<{public} as core::any::Any>::type_id",
        f"<{public} as core::borrow::Borrow<{implementation}>>::borrow",
        f"<{public} as core::borrow::BorrowMut<{implementation}>>::borrow_mut",
        f"<{public} as core::convert::From<{implementation}>>::from",
        f"<{public} as core::default::Default>::default",
        f"<{public} as core::fmt::Debug>::fmt",
    }


def conditional_traits(public: str) -> set[str]:
    return {
        f"<{public} as core::convert::Into<U>>::into",
        f"<{public} as core::convert::TryFrom<U>>::Error",
        f"<{public} as core::convert::TryFrom<U>>::try_from",
        f"<{public} as core::convert::TryInto<U>>::Error",
        f"<{public} as core::convert::TryInto<U>>::try_into",
    }


class T102LifecycleRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_trait_paths_are_compiler_observed_and_owner_approved(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        expected_all: set[tuple[str, str]] = set()
        for name, details in TYPES.items():
            public = f"kmipkit_protocol::{name}"
            implementation = f"kmipkit_protocol::{details['module']}::{name}"
            direct = direct_traits(public, implementation)
            conditional = conditional_traits(public)
            expected = direct | conditional
            pairs = {("rust-kmipkit-protocol", path) for path in expected}
            expected_all.update(pairs)
            prefix = f"<{public} as "
            observed_direct = {row["exportName"] for row in self.report["declarations"] if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(prefix)}
            observed_conditional = {row["exportName"] for row in self.report["conditionalDeclarations"] if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(prefix)}
            observed_excluded = {row["exportName"] for row in self.report["excluded"] if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(prefix)}
            with self.subTest(type=name, report=True):
                self.assertEqual(observed_direct, direct)
                self.assertEqual(observed_conditional, conditional)
                self.assertEqual(observed_excluded, expected)
                self.assertEqual((len(observed_direct), len(observed_conditional)), (6, 5))
            declarations = {(row["packageId"], row["exportName"]) for row in self.manifest["publicDeclarations"] if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(prefix)}
            exclusions = {(row["packageId"], row["exportName"]): row for row in self.manifest.get("approvedExclusions", []) if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(prefix)}
            with self.subTest(type=name, manifest=True):
                self.assertEqual(declarations, set())
                self.assertEqual(set(exclusions), pairs)
                for row in exclusions.values():
                    evidence = row["evidence"]
                    self.assertEqual(row["approvalSource"], APPROVAL)
                    for trace in (*OWNER_REQS, "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021", *details["normative"]):
                        self.assertIn(trace, evidence)
                    self.assertIn(details["section"], evidence)
                    self.assertIn(details["request"], evidence)
                    self.assertIn("inventory-only", evidence)
                    self.assertIn("does not claim OASIS conformance", evidence)
                    self.assertIn("compiler-generated standard-trait", evidence)
                    self.assertIn("standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP", row["reason"])
                    if "caption_note" in details:
                        self.assertIn(details["caption_note"], evidence)
        self.assertEqual(len(expected_all), 66)
        actual = {(row["packageId"], row["exportName"]) for row in self.manifest.get("approvedExclusions", []) if row.get("packageId") == "rust-kmipkit-protocol" and any(row.get("exportName", "").startswith(f"<kmipkit_protocol::{name} as ") for name in TYPES)}
        self.assertEqual(actual, expected_all)

    def test_functional_states_and_root_facades_are_unchanged(self) -> None:
        for name, details in TYPES.items():
            protocol = f"kmipkit_protocol::{name}"
            root = f"kmipkit::{name}"
            states = {row["exportName"]: row["declarationState"] for row in self.manifest["publicDeclarations"] if row.get("packageId") == "rust-kmipkit-protocol" and row.get("exportName", "").startswith(protocol) and not row["exportName"].startswith("<")}
            expected = {f"{protocol}{suffix}": state for suffix, state in details["states"].items()}
            roots = {row["exportName"]: row["declarationState"] for row in self.manifest["publicDeclarations"] if row.get("packageId") == "rust-kmipkit" and (row.get("exportName", "").startswith(root) or row.get("exportName", "").startswith(f"<{root} as "))}
            with self.subTest(type=name):
                self.assertEqual(states, expected)
                self.assertEqual(roots, {})

    def test_candidate_count_drops_by_exactly_66(self) -> None:
        count = sum(row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked" for row in self.manifest["publicDeclarations"])
        self.assertEqual(count, 2225)


if __name__ == "__main__":
    unittest.main()
