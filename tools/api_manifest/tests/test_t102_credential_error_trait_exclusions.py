"""Pin T102 exclusions for credential-session errors and ticket error values."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/024-credential-sessions/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
CONDITIONAL_TRAITS = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
COMMON_DIRECT_TRAITS = {
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
ERROR_TRAITS = {
    "core::error::Error>::cause",
    "core::error::Error>::description",
    "core::error::Error>::provide",
    "core::error::Error>::source",
    "core::error::Error>::type_id",
}
BORROW_AND_FROM_TRAITS = {
    "core::borrow::Borrow<{implementation_type}>>::borrow",
    "core::borrow::BorrowMut<{implementation_type}>>::borrow_mut",
    "core::convert::From<{implementation_type}>>::from",
}
OPERATION_ERRORS = {
    "DelegatedLoginError": {
        "module": "delegated_login",
        "direct_count": 21,
        "requirements": "KMIPKIT-0038-FR-001/002/005/008",
        "oasis": "OASIS KMIP v2.1 §6.1.12 Tables 199-201",
        "evidence_suffix": "payload-free local conversion error",
    },
    "LoginError": {
        "module": "login",
        "direct_count": 21,
        "requirements": "KMIPKIT-0038-FR-001/002/005/008",
        "oasis": "OASIS KMIP v2.1 §6.1.30 Tables 253-255",
        "evidence_suffix": "payload-free local conversion error",
    },
    "LogoutError": {
        "module": "logout",
        "direct_count": 21,
        "requirements": "KMIPKIT-0038-FR-001/002/005/008",
        "oasis": "OASIS KMIP v2.1 §6.1.31 Tables 256-258",
        "evidence_suffix": "payload-free local conversion error",
    },
    "TicketValidationError": {
        "module": "ticket",
        "direct_count": 21,
        "requirements": "KMIPKIT-0038-FR-001/002/005/008",
        "oasis": "OASIS KMIP v2.1 §7.39 Table 391; §11.57 Table 488",
        "evidence_suffix": "payload-free local validation error",
    },
    "TicketValidationErrorKind": {
        "module": "ticket",
        "direct_count": 16,
        "requirements": "KMIPKIT-0038-FR-002/005/008",
        "oasis": "OASIS KMIP v2.1 §7.39 Table 391; §11.57 Table 488",
        "evidence_suffix": "payload-free local validation category",
    },
}
PACKAGES = {
    "rust-kmipkit-protocol": "kmipkit_protocol",
    "rust-kmipkit": "kmipkit",
}
EXCLUSION_REASON = (
    "Compiler-generated Rust trait implementation for this payload-free local "
    "error value is not a separate KMIPKit product capability."
)


def direct_traits(type_name: str, implementation_type: str) -> set[str]:
    traits = COMMON_DIRECT_TRAITS
    if type_name == "TicketValidationErrorKind":
        traits = traits - ERROR_TRAITS
    return traits | {
        trait.format(implementation_type=implementation_type)
        for trait in BORROW_AND_FROM_TRAITS
    }


def expected_evidence(type_name: str) -> str:
    details = OPERATION_ERRORS[type_name]
    return (
        f"T102 {T102_REQUIREMENTS[0]}/{T102_REQUIREMENTS[1]}; "
        f"owner {details['requirements']}; {details['oasis']}; "
        f"{details['evidence_suffix']}; no OASIS trait mandate."
    )


class T102CredentialErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_error_trait_paths_are_approved_and_functional_rows_stay_blocked(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(self.rust_report["issues"], [])
        self.assertEqual(self.rust_report["missingFromManifest"], [])
        self.assertEqual(self.rust_report["notObserved"], [])
        self.assertEqual(self.rust_report["kindMismatches"], [])
        self.assertEqual(self.rust_report["conditionalConditionMismatches"], [])
        self.assertEqual(self.rust_report["conditionalStateMismatches"], [])

        for type_name, details in OPERATION_ERRORS.items():
            for package_id, public_crate in PACKAGES.items():
                public_type = f"{public_crate}::{type_name}"
                implementation_type = (
                    f"kmipkit_protocol::{details['module']}::{type_name}"
                )
                prefix = f"<{public_type} as "
                direct = {
                    f"<{public_type} as {trait}"
                    for trait in direct_traits(type_name, implementation_type)
                }
                conditional = {
                    f"<{public_type} as {trait}" for trait in CONDITIONAL_TRAITS
                }
                expected = direct | conditional
                with self.subTest(type=type_name, package=package_id):
                    observed_direct = {
                        row["exportName"]
                        for row in self.rust_report["declarations"]
                        if row.get("packageId") == package_id
                        and row.get("exportName", "").startswith(prefix)
                    }
                    observed_conditional = {
                        row["exportName"]
                        for row in self.rust_report["conditionalDeclarations"]
                        if row.get("packageId") == package_id
                        and row.get("exportName", "").startswith(prefix)
                    }
                    observed_excluded = {
                        row["exportName"]
                        for row in self.rust_report["excluded"]
                        if row.get("packageId") == package_id
                        and row.get("exportName", "").startswith(prefix)
                    }
                    self.assertEqual(observed_direct, direct)
                    self.assertEqual(observed_conditional, conditional)
                    self.assertEqual(observed_excluded, expected)
                    self.assertEqual(len(direct), details["direct_count"])
                    self.assertEqual(len(conditional), 5)

                    declarations = {
                        row["exportName"]: row
                        for row in self.manifest["publicDeclarations"]
                        if row.get("packageId") == package_id
                        and row.get("exportName", "").startswith(prefix)
                    }
                    exclusions = {
                        row["exportName"]: row
                        for row in self.manifest["approvedExclusions"]
                        if row.get("packageId") == package_id
                        and row.get("exportName", "").startswith(prefix)
                    }
                    self.assertEqual(declarations, {})
                    self.assertEqual(set(exclusions), expected)
                    self.assertEqual(len(exclusions), details["direct_count"] + 5)

                    for row in exclusions.values():
                        self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                        self.assertEqual(row["reason"], EXCLUSION_REASON)
                        self.assertEqual(row["evidence"], expected_evidence(type_name))
                        self.assertIn(T102_REQUIREMENTS[0], row["evidence"])
                        self.assertIn(T102_REQUIREMENTS[1], row["evidence"])
                        self.assertLessEqual(len(row["evidence"].encode("utf-8")), 250)

                    functional_rows = [
                        row
                        for row in self.manifest["publicDeclarations"]
                        if row.get("packageId") == package_id
                        and (
                            row.get("exportName") == public_type
                            or row.get("exportName", "").startswith(public_type + "::")
                        )
                    ]
                    if type_name in {"TicketValidationError", "TicketValidationErrorKind"}:
                        self.assertEqual(functional_rows, [])
                    else:
                        self.assertTrue(functional_rows)
                        self.assertTrue(
                            all(row["declarationState"] == "blocked" for row in functional_rows)
                        )


if __name__ == "__main__":
    unittest.main()
