"""Pin the exact T102 KMIPKIT-0036 protocol error trait exclusions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
PACKAGE = "rust-kmipkit-protocol"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/022-object-lifecycle/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation \u2014 2026-10-11",
}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-003",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-007",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")

# These suffixes are the exact common direct Rust export paths. Borrow,
# BorrowMut, and From carry each error's concrete module-qualified type below.
COMMON_DIRECT = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "alloc::string::ToString>::to_string",
    "core::any::Any>::type_id",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::error::Error>::cause",
    "core::error::Error>::description",
    "core::error::Error>::provide",
    "core::error::Error>::source",
    "core::error::Error>::type_id",
    "core::fmt::Debug>::fmt",
    "core::fmt::Display>::fmt",
}
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
MODULES = {
    "CertifyError": "certify",
    "CheckError": "check",
    "ObtainLeaseError": "obtain_lease",
    "RegisterError": "register",
    "RevokeError": "revoke",
    "RecertifyError": "recertify",
    "RekeyError": "rekey",
    "RekeyKeyPairError": "rekey_key_pair",
    "ReProvisionError": "reprovision",
}
OPERATIONS = {
    "CertifyError": {
        "element": "KMIPKIT-ELEM-OP-C2S-CERTIFY",
        "reference": "OASIS KMIP v2.1 Section 6.1.6, Table 181",
        "caption": "Table 181: Certify Errors",
    },
    "CheckError": {
        "element": "KMIPKIT-ELEM-OP-C2S-CHECK",
        "reference": "OASIS KMIP v2.1 Section 6.1.7, Table 185",
        "caption": "Table 185: Check Errors",
    },
    "ObtainLeaseError": {
        "element": "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
        "reference": "OASIS KMIP v2.1 Section 6.1.35, Table 270",
        "caption": "Table 270: Obtain Lease Errors",
    },
    "RegisterError": {
        "element": "KMIPKIT-ELEM-OP-C2S-REGISTER",
        "reference": "OASIS KMIP v2.1 Section 6.1.43, Table 294",
        "caption": "Table 294: Register Errors",
    },
    "RevokeError": {
        "element": "KMIPKIT-ELEM-OP-C2S-REVOKE",
        "reference": "OASIS KMIP v2.1 Section 6.1.44, Table 297",
        "caption": "Table 297: Revoke Errors",
    },
    "RecertifyError": {
        "element": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        "reference": "OASIS KMIP v2.1 Section 6.1.45, Table 302",
        "caption": "Table 302: Re-certify Errors",
    },
    "RekeyError": {
        "element": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "reference": "OASIS KMIP v2.1 Section 6.1.46, Table 307",
        "caption": "Table 307: Re-key Errors",
    },
    "RekeyKeyPairError": {
        "element": "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
        "reference": "OASIS KMIP v2.1 Section 6.1.47, Table 312",
        "caption": "Table 312: Re-key Key Pair Errors",
    },
    "ReProvisionError": {
        "element": "KMIPKIT-ELEM-OP-C2S-RE-PROVISION",
        "reference": "OASIS KMIP v2.1 Section 6.1.48.1, Table 315",
        "caption": "Table 315: RNG Retrieve Errors",
        "caption_note": (
            "Table 315 has an errant RNG Retrieve caption in the "
            "Re-Provision error section"
        ),
    },
}
ERROR_VARIANTS = {
    "CertifyError": "MalformedSuccessPayload",
    "CheckError": "MalformedResponsePayload",
    "ObtainLeaseError": "MalformedSuccessPayload",
    "RegisterError": "MalformedSuccessPayload",
    "RevokeError": "MalformedSuccessPayload",
    "RecertifyError": "MalformedSuccessPayload",
    "RekeyError": "MalformedSuccessPayload",
    "RekeyKeyPairError": "MalformedSuccessPayload",
    "ReProvisionError": "MalformedSuccessPayload",
}


def exact_paths(type_name: str, suffixes: set[str]) -> set[str]:
    return {f"<kmipkit_protocol::{type_name} as {suffix}" for suffix in suffixes}


def direct_suffixes(type_name: str) -> set[str]:
    module = MODULES[type_name]
    self_type = f"kmipkit_protocol::{module}::{type_name}"
    return COMMON_DIRECT | {
        f"core::borrow::Borrow<{self_type}>>::borrow",
        f"core::borrow::BorrowMut<{self_type}>>::borrow_mut",
        f"core::convert::From<{self_type}>>::from",
    }


class T102LifecycleErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_error_trait_paths_are_owner_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        all_pairs: set[tuple[str, str]] = set()

        for type_name, operation in OPERATIONS.items():
            expected_direct = exact_paths(type_name, direct_suffixes(type_name))
            expected_conditional = exact_paths(type_name, CONDITIONAL)
            expected = expected_direct | expected_conditional
            prefix = f"<kmipkit_protocol::{type_name} as "
            observed_direct = {
                row["exportName"]
                for row in self.report["declarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_conditional = {
                row["exportName"]
                for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            observed_excluded = {
                row["exportName"]
                for row in self.report["excluded"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(observed_direct, expected_direct)
            self.assertEqual(observed_conditional, expected_conditional)
            self.assertEqual(observed_excluded, expected)
            self.assertEqual(len(expected_direct), 21)
            self.assertEqual(len(expected_conditional), 5)

            declarations = {
                row["exportName"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                row["exportName"]: row
                for row in self.manifest["approvedExclusions"]
                if row.get("packageId") == PACKAGE
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(declarations, set())
            self.assertEqual(set(exclusions), expected)
            self.assertEqual(len(exclusions), 26)

            for export_name, row in exclusions.items():
                with self.subTest(type_name=type_name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    for requirement_id in (
                        *OWNER_REQUIREMENTS,
                        operation["element"],
                        *T102_REQUIREMENTS,
                    ):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn(operation["reference"], row["evidence"])
                    self.assertIn(operation["caption"], row["evidence"])
                    if "caption_note" in operation:
                        self.assertIn(operation["caption_note"], row["evidence"])
                        self.assertIn("immediately before Section 6.1.49", row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    self.assertIn(type_name, row["reason"])
            all_pairs.update((PACKAGE, export_name) for export_name in expected)

        self.assertEqual(len(all_pairs), 234)

    def test_error_parent_and_functional_declarations_remain_blocked_without_root_aliases(self) -> None:
        for type_name, malformed_variant in ERROR_VARIANTS.items():
            protocol_name = f"kmipkit_protocol::{type_name}"
            suffixes = {
                "",
                "::InvalidOperationResult",
                "::InvalidOperationResult::0",
                "::MalformedResponseItem",
                f"::{malformed_variant}",
                "::MissingResultStatus",
                "::UnexpectedOperation",
            }
            expected = {f"{protocol_name}{suffix}": "blocked" for suffix in suffixes}
            actual = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == PACKAGE
                and (
                    row.get("exportName") == protocol_name
                    or row.get("exportName", "").startswith(protocol_name + "::")
                )
                and not row.get("exportName", "").startswith("<")
            }
            self.assertEqual(actual, expected)

            root_name = f"kmipkit::{type_name}"
            root_rows = [
                row
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == "rust-kmipkit"
                and (
                    row.get("exportName") == root_name
                    or row.get("exportName", "").startswith(root_name + "::")
                    or row.get("exportName", "").startswith(f"<{root_name} as ")
                )
            ]
            self.assertEqual(root_rows, [])

    def test_manifest_and_t102_candidate_totals_decrease_by_234(self) -> None:
        count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(count, 3026)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 8224)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 6500)


if __name__ == "__main__":
    unittest.main()
