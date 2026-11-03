"""Exact T102 inventory and preservation contract for cryptographic values."""

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
API_SPEC_PATH = "specs/043-public-api-bindings/spec.md"
OWNER_SPEC_PATH = "specs/021-hash-mac-signature-operations/spec.md"


CRYPTographic_TYPES = {
    "CryptographicAlgorithm": {
        "implementation": "kmipkit_protocol::cryptographic_operation::CryptographicAlgorithm",
        "section": "§11.12",
        "owner_requirements": (
            "KMIPKIT-0021-FR-004",
            "KMIPKIT-0021-FR-006",
            "KMIPKIT-0021-FR-007",
            "KMIPKIT-0021-FR-009",
            "KMIPKIT-0021-FR-013",
        ),
        "owner_trace": "KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM",
    },
    "DigitalSignatureAlgorithm": {
        "implementation": "kmipkit_protocol::cryptographic_operation::DigitalSignatureAlgorithm",
        "section": "§11.16",
        "owner_requirements": (
            "KMIPKIT-0021-FR-004",
            "KMIPKIT-0021-FR-006",
            "KMIPKIT-0021-FR-007",
            "KMIPKIT-0021-FR-009",
            "KMIPKIT-0021-FR-013",
        ),
        "owner_trace": "KMIPKIT-ELEM-ENUMERATION-DIGITAL-SIGNATURE-ALGORITHM",
    },
    "HashingAlgorithm": {
        "implementation": "kmipkit_protocol::cryptographic_operation::HashingAlgorithm",
        "section": "§11.21",
        "owner_requirements": (
            "KMIPKIT-0021-FR-002",
            "KMIPKIT-0021-FR-009",
            "KMIPKIT-0021-FR-013",
        ),
        "owner_trace": "KMIPKIT-ELEM-ENUMERATION-HASHING-ALGORITHM",
    },
    "ValidityIndicator": {
        "implementation": "kmipkit_protocol::cryptographic_operation::ValidityIndicator",
        "section": "§11.61",
        "owner_requirements": (
            "KMIPKIT-0021-FR-008",
            "KMIPKIT-0021-FR-009",
        ),
        "owner_trace": "KMIPKIT-ELEM-ENUMERATION-VALIDITY-INDICATOR",
    },
    "CryptographicOperationResponseContext": {
        "implementation": "kmipkit_protocol::cryptographic_operation::CryptographicOperationResponseContext",
        "section": None,
        "owner_requirements": (
            "KMIPKIT-0021-FR-008",
            "KMIPKIT-0021-FR-015",
        ),
        "owner_trace": None,
    },
    "VerificationResponseContext": {
        "implementation": "kmipkit_protocol::cryptographic_operation::CryptographicOperationResponseContext",
        "section": None,
        "owner_requirements": (
            "KMIPKIT-0021-FR-008",
            "KMIPKIT-0021-FR-015",
        ),
        "owner_trace": None,
    },
}


def expected_direct_traits(public_name: str, implementation_name: str) -> set[str]:
    prefix = f"<{public_name} as "
    return {
        f"{prefix}alloc::borrow::ToOwned>::Owned",
        f"{prefix}alloc::borrow::ToOwned>::clone_into",
        f"{prefix}alloc::borrow::ToOwned>::to_owned",
        f"{prefix}core::any::Any>::type_id",
        f"{prefix}core::borrow::Borrow<{implementation_name}>>::borrow",
        f"{prefix}core::borrow::BorrowMut<{implementation_name}>>::borrow_mut",
        f"{prefix}core::clone::Clone>::clone",
        f"{prefix}core::clone::Clone>::clone_from",
        f"{prefix}core::clone::CloneToUninit>::clone_to_uninit",
        f"{prefix}core::cmp::Eq>::assert_receiver_is_total_eq",
        f"{prefix}core::cmp::PartialEq>::eq",
        f"{prefix}core::cmp::PartialEq>::ne",
        f"{prefix}core::convert::From<{implementation_name}>>::from",
        f"{prefix}core::fmt::Debug>::fmt",
        f"{prefix}core::hash::Hash>::hash",
        f"{prefix}core::hash::Hash>::hash_slice",
    }


def expected_conditional_traits(public_name: str) -> set[str]:
    prefix = f"<{public_name} as "
    return {
        f"{prefix}core::convert::Into<U>>::into",
        f"{prefix}core::convert::TryFrom<U>>::Error",
        f"{prefix}core::convert::TryFrom<U>>::try_from",
        f"{prefix}core::convert::TryInto<U>>::Error",
        f"{prefix}core::convert::TryInto<U>>::try_into",
    }


class T102CryptographicValueTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))

    def test_exact_compiler_observed_paths_have_owner_backed_exclusions(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertEqual(self.rust_report["issues"], [])
        self.assertEqual(self.rust_report["missingFromManifest"], [])
        self.assertEqual(self.rust_report["notObserved"], [])
        self.assertEqual(self.rust_report["kindMismatches"], [])
        self.assertEqual(self.rust_report["conditionalConditionMismatches"], [])
        self.assertEqual(self.rust_report["conditionalStateMismatches"], [])

        all_expected: set[str] = set()
        for public_name, details in CRYPTographic_TYPES.items():
            public_type = f"kmipkit_protocol::{public_name}"
            expected_direct = expected_direct_traits(public_type, details["implementation"])
            expected_conditional = expected_conditional_traits(public_type)
            all_expected.update(expected_direct | expected_conditional)

            with self.subTest(public_name=public_name):
                observed_direct = {
                    row["exportName"]
                    for row in self.rust_report["declarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                }
                observed_conditional = {
                    row["exportName"]
                    for row in self.rust_report["conditionalDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName", "").startswith(f"<{public_type} as ")
                }
                self.assertEqual(observed_direct, expected_direct)
                self.assertEqual(observed_conditional, expected_conditional)
                self.assertEqual(len(observed_direct), 16)
                self.assertEqual(len(observed_conditional), 5)

                declarations = {
                    row["exportName"]: row
                    for row in self.manifest["publicDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName") in expected_direct | expected_conditional
                }
                exclusions = [
                    row
                    for row in self.manifest.get("approvedExclusions", [])
                    if row.get("packageId") == "rust-kmipkit-protocol"
                    and row.get("exportName") in expected_direct | expected_conditional
                ]
                self.assertEqual(set(declarations), set())
                self.assertEqual(
                    {row["exportName"] for row in exclusions},
                    expected_direct | expected_conditional,
                )
                self.assertEqual(len(exclusions), 21)

                for row in exclusions:
                    with self.subTest(public_name=public_name, export_name=row["exportName"]):
                        self.assertIn("standard-trait implementation", row["reason"])
                        self.assertIn("not a separate KMIPKit capability", row["reason"])
                        self.assertEqual(
                            row["approvalSource"],
                            {
                                "kind": "feature-specification",
                                "path": API_SPEC_PATH,
                                "status": "Approved",
                                "evidence": "**Status**: Approved for implementation",
                            },
                        )
                        self.assertIn("KMIPKIT-0043-FR-010", row["evidence"])
                        self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])
                        self.assertIn(OWNER_SPEC_PATH, row["evidence"])
                        for requirement_id in details["owner_requirements"]:
                            self.assertIn(requirement_id, row["evidence"])
                        if details["section"] is None:
                            self.assertIn("local response-framing model", row["evidence"])
                            self.assertIn("no OASIS requirement applies to compiler-generated traits", row["evidence"])
                        else:
                            self.assertIn("OASIS KMIP Specification v2.1", row["evidence"])
                            self.assertIn(details["section"], row["evidence"])

        self.assertEqual(len(all_expected), 126)

    def test_functional_value_and_response_framing_exports_remain_blocked(self) -> None:
        expected_functional_exports = {
            "kmipkit_protocol::CryptographicAlgorithm",
            "kmipkit_protocol::CryptographicAlgorithm::from_raw",
            "kmipkit_protocol::CryptographicAlgorithm::raw",
            "kmipkit_protocol::DigitalSignatureAlgorithm",
            "kmipkit_protocol::DigitalSignatureAlgorithm::from_raw",
            "kmipkit_protocol::DigitalSignatureAlgorithm::raw",
            "kmipkit_protocol::HashingAlgorithm",
            "kmipkit_protocol::HashingAlgorithm::from_raw",
            "kmipkit_protocol::HashingAlgorithm::raw",
            "kmipkit_protocol::ValidityIndicator",
            "kmipkit_protocol::ValidityIndicator::from_raw",
            "kmipkit_protocol::ValidityIndicator::raw",
            "kmipkit_protocol::CryptographicOperationResponseContext",
            "kmipkit_protocol::CryptographicOperationResponseContext::SinglePart",
            "kmipkit_protocol::CryptographicOperationResponseContext::MultipartNonFinal",
            "kmipkit_protocol::CryptographicOperationResponseContext::MultipartFinal",
            "kmipkit_protocol::VerificationResponseContext",
            "kmipkit_protocol::VerificationResponseContext::SinglePart",
            "kmipkit_protocol::VerificationResponseContext::MultipartNonFinal",
            "kmipkit_protocol::VerificationResponseContext::MultipartFinal",
        }
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName") in expected_functional_exports
        }
        self.assertEqual(set(declarations), expected_functional_exports)
        self.assertTrue(
            all(row["declarationState"] == "blocked" for row in declarations.values())
        )
        self.assertTrue(
            all(row["candidateSpecId"] == "KMIPKIT-0043" for row in declarations.values())
        )

    def test_response_payload_error_and_ttlv_policies_are_unchanged(self) -> None:
        expected_response_states = {
            "kmipkit_protocol::HashResponse": "declared",
            "kmipkit_protocol::HashResponse::result": "verified",
            "kmipkit_protocol::HashResponse::with_ttlv": "verified",
            "kmipkit_protocol::HashResponse::data": "declared",
            "kmipkit_protocol::HashResponse::correlation_value": "declared",
            "kmipkit_protocol::MacResponse": "declared",
            "kmipkit_protocol::MacResponse::result": "verified",
            "kmipkit_protocol::MacResponse::with_ttlv": "verified",
            "kmipkit_protocol::MacResponse::unique_identifier": "declared",
            "kmipkit_protocol::MacResponse::mac_data": "declared",
            "kmipkit_protocol::MacResponse::correlation_value": "declared",
            "kmipkit_protocol::MacVerifyResponse": "declared",
            "kmipkit_protocol::MacVerifyResponse::result": "verified",
            "kmipkit_protocol::MacVerifyResponse::with_ttlv": "verified",
            "kmipkit_protocol::MacVerifyResponse::unique_identifier": "declared",
            "kmipkit_protocol::MacVerifyResponse::validity_indicator": "verified",
            "kmipkit_protocol::MacVerifyResponse::correlation_value": "declared",
            "kmipkit_protocol::SignResponse": "declared",
            "kmipkit_protocol::SignResponse::result": "verified",
            "kmipkit_protocol::SignResponse::with_ttlv": "verified",
            "kmipkit_protocol::SignResponse::unique_identifier": "declared",
            "kmipkit_protocol::SignResponse::signature_data": "declared",
            "kmipkit_protocol::SignResponse::correlation_value": "declared",
            "kmipkit_protocol::SignatureVerifyResponse": "declared",
            "kmipkit_protocol::SignatureVerifyResponse::result": "verified",
            "kmipkit_protocol::SignatureVerifyResponse::with_ttlv": "verified",
            "kmipkit_protocol::SignatureVerifyResponse::unique_identifier": "declared",
            "kmipkit_protocol::SignatureVerifyResponse::validity_indicator": "verified",
            "kmipkit_protocol::SignatureVerifyResponse::recovered_data": "declared",
            "kmipkit_protocol::SignatureVerifyResponse::correlation_value": "declared",
        }
        response_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName") in expected_response_states
        }
        self.assertEqual(
            {name: row["declarationState"] for name, row in response_rows.items()},
            expected_response_states,
        )

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row.get("capabilityId") == "crypto_response_ttlv"
        )
        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["ownerSpecId"], "KMIPKIT-0021")
        self.assertEqual(
            capability["requirementIds"],
            [
                "KMIPKIT-0021-FR-015",
                "KMIPKIT-0043-FR-005",
                "KMIPKIT-0043-FR-010",
                "KMIPKIT-0043-FR-021",
            ],
        )
        self.assertEqual(
            capability["mappings"],
            {
                "rust": [
                    "kmipkit::ClientError::ProtocolResponse::response_ttlv",
                    "kmipkit::ClientError::response_ttlv",
                    "kmipkit_client::ClientError::ProtocolResponse::response_ttlv",
                    "kmipkit_client::ClientError::response_ttlv",
                ],
                "c": ["kmipkit_client_execute_with_error_ttlv_v1"],
                "java": ["org.kmipkit.ProtocolException.responseTtlv"],
                "python": ["kmipkit.errors.ClientExecutionError.response_ttlv"],
            },
        )

        error_prefixes = (
            "kmipkit_protocol::CryptographicOperationError",
            "kmipkit_protocol::CryptographicOperationErrorKind",
        )
        error_rows = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("exportName", "").startswith(error_prefixes)
        }
        self.assertTrue(error_rows)
        self.assertTrue(
            all(
                row["declarationState"] == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in error_rows.values()
            )
        )
        error_names = set(error_rows)
        self.assertTrue(
            all(
                row.get("exportName") not in error_names
                for row in self.manifest.get("approvedExclusions", [])
            )
        )

        owner_spec = (ROOT / OWNER_SPEC_PATH).read_text(encoding="utf-8")
        owner_traceability = (
            ROOT / "specs" / "021-hash-mac-signature-operations" / "traceability.md"
        ).read_text(encoding="utf-8")
        self.assertIn("PR #70 merged into `release/1.0.0`", owner_spec)
        self.assertIn("KMIPKIT-DISC-048", owner_spec)
        self.assertIn("remains open", owner_spec)
        self.assertIn("KMIPKIT-DISC-048", owner_traceability)
        self.assertIn("either final-part form", owner_traceability)


if __name__ == "__main__":
    unittest.main()
