"""Pin T102's root RNG error and request trait artifact exclusions."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = ROOT / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-11-01.json"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/027-rng-operations/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
OWNER_REQUIREMENTS = (
    "KMIPKIT-0041-FR-001",
    "KMIPKIT-0041-FR-004",
    "KMIPKIT-0041-FR-005",
)
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
CONDITIONAL_TRAITS = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
ERROR_DIRECT_TRAITS = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "alloc::string::ToString>::to_string",
    "core::any::Any>::type_id",
    "core::borrow::Borrow<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::borrow",
    "core::borrow::BorrowMut<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::borrow_mut",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::convert::From<kmipkit_protocol::cryptographic_operation::CryptographicOperationError>>::from",
    "core::error::Error>::cause",
    "core::error::Error>::description",
    "core::error::Error>::provide",
    "core::error::Error>::source",
    "core::error::Error>::type_id",
    "core::fmt::Debug>::fmt",
    "core::fmt::Display>::fmt",
}
REQUEST_DIRECT_TRAITS = {
    "alloc::borrow::ToOwned>::Owned",
    "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned",
    "core::any::Any>::type_id",
    "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from",
    "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq",
    "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne",
    "core::fmt::Debug>::fmt",
}
REQUESTS = (
    (
        "kmipkit::RngRetrieveRequest",
        "kmipkit_protocol::rng_retrieve::RngRetrieveRequest",
        REQUEST_DIRECT_TRAITS
        | {
            "core::borrow::Borrow<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::rng_retrieve::RngRetrieveRequest>>::from",
        },
    ),
    (
        "kmipkit::RngSeedRequest",
        "kmipkit_protocol::rng_seed::RngSeedRequest",
        {
            "core::any::Any>::type_id",
            "core::borrow::Borrow<kmipkit_protocol::rng_seed::RngSeedRequest>>::borrow",
            "core::borrow::BorrowMut<kmipkit_protocol::rng_seed::RngSeedRequest>>::borrow_mut",
            "core::convert::From<kmipkit_protocol::rng_seed::RngSeedRequest>>::from",
            "core::fmt::Debug>::fmt",
        },
    ),
)
ERROR_PREFIXES = (
    ("rust-kmipkit", "kmipkit::RngRetrieveError"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngRetrieveError"),
    ("rust-kmipkit", "kmipkit::RngSeedError"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::RngSeedError"),
)
NORMATIVE_CONTEXT = {
    "RngRetrieveError": "OASIS KMIP v2.1 Section 6.1.49 Tables 316-318",
    "RngSeedError": "OASIS KMIP v2.1 Section 6.1.50 Tables 319-321",
}


def expected_paths() -> dict[tuple[str, str], str]:
    expected: dict[tuple[str, str], str] = {}
    for package, type_name in ERROR_PREFIXES:
        for trait in ERROR_DIRECT_TRAITS | CONDITIONAL_TRAITS:
            name = f"<{type_name} as {trait}"
            expected[(package, name)] = "error"
    for type_name, implementation_type, direct_traits in REQUESTS:
        for trait in direct_traits | CONDITIONAL_TRAITS:
            name = f"<{type_name} as {trait}"
            expected[("rust-kmipkit", name)] = "request"
    return expected


class T102RngRootTraitBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_root_error_and_request_traits_are_approved_exclusions(self) -> None:
        expected = expected_paths()
        self.assertEqual(len(expected), 133)
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        observed_direct = {
            (row["packageId"], row["exportName"])
            for row in self.report["declarations"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        observed_conditional = {
            (row["packageId"], row["exportName"])
            for row in self.report["conditionalDeclarations"]
            if (row.get("packageId"), row.get("exportName")) in expected
        }
        expected_conditional = {
            key for key in expected if any(key[1].endswith(trait) for trait in CONDITIONAL_TRAITS)
        }
        expected_direct = set(expected) - expected_conditional
        self.assertEqual(observed_direct, expected_direct)
        self.assertEqual(observed_conditional, expected_conditional)
        self.assertEqual(len(expected_direct), 103)
        self.assertEqual(len(expected_conditional), 30)
        self.assertEqual(observed_direct | observed_conditional, set(expected))

        declarations = {
            (row.get("packageId"), row.get("exportName")): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row.get("packageId"), row.get("exportName")): row
            for row in self.manifest["approvedExclusions"]
        }
        self.assertEqual(set(exclusions).intersection(expected), set(expected))
        for key, category in expected.items():
            with self.subTest(package=key[0], export_name=key[1]):
                self.assertNotIn(key, declarations)
                row = exclusions[key]
                self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                for requirement_id in (*OWNER_REQUIREMENTS, *T102_REQUIREMENTS):
                    self.assertIn(requirement_id, row["evidence"])
                owner_type = "RngSeedError" if "RngSeed" in key[1] else "RngRetrieveError"
                if category == "request":
                    owner_type = "RngSeedError" if "RngSeedRequest" in key[1] else "RngRetrieveError"
                self.assertIn(NORMATIVE_CONTEXT[owner_type], row["evidence"])
                self.assertIn("no OASIS standard-trait requirement", row["evidence"])
                self.assertIn("inventory-only", row["evidence"])
                if category == "error":
                    for requirement_id in ("KMIPKIT-0041-FR-003", "KMIPKIT-0041-FR-006"):
                        self.assertIn(requirement_id, row["evidence"])
                    self.assertIn("payload-free ResultValidationError source", row["evidence"])
                elif "RngSeedRequest" in key[1]:
                    self.assertIn("SecretBytes Debug output is redacted", row["evidence"])
                    self.assertIn("trait paths do not expose seed bytes", row["evidence"])
                else:
                    self.assertIn("only the requested data length", row["evidence"])

        candidate_count = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidate_count, 1_432)

    def test_root_parents_and_non_trait_rng_paths_remain_blocked(self) -> None:
        declarations = {
            (row.get("packageId"), row.get("exportName")): row["declarationState"]
            for row in self.manifest["publicDeclarations"]
        }
        for package, type_name in ERROR_PREFIXES:
            self.assertEqual(declarations[(package, type_name)], "blocked")
            self.assertEqual(declarations[(package, f"{type_name}::kind")], "blocked")
        for type_name, _, _ in REQUESTS:
            self.assertEqual(declarations[("rust-kmipkit", type_name)], "blocked")
            self.assertEqual(declarations[("rust-kmipkit", f"{type_name}::new")], "blocked")


if __name__ == "__main__":
    unittest.main()
