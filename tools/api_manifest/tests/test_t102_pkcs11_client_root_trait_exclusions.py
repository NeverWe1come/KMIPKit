"""Pin the exact KMIPKIT-0043 exclusions for client-root PKCS#11 traits."""

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
OWNER_SPEC = ROOT / "specs" / "026-pkcs11" / "spec.md"
OWNER_APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/026-pkcs11/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}

REQUEST_PREFIX = "<kmipkit_client::Pkcs11Request as "
RESPONSE_PREFIX = "<kmipkit_client::Pkcs11Response as "

EXPECTED_REQUEST_DIRECT = {
    "<kmipkit_client::Pkcs11Request as core::any::Any>::type_id",
    "<kmipkit_client::Pkcs11Request as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow",
    "<kmipkit_client::Pkcs11Request as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Request>>::borrow_mut",
    "<kmipkit_client::Pkcs11Request as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Request>>::from",
    "<kmipkit_client::Pkcs11Request as core::fmt::Debug>::fmt",
}
EXPECTED_REQUEST_CONDITIONAL = {
    "<kmipkit_client::Pkcs11Request as core::convert::Into<U>>::into",
    "<kmipkit_client::Pkcs11Request as core::convert::TryFrom<U>>::Error",
    "<kmipkit_client::Pkcs11Request as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_client::Pkcs11Request as core::convert::TryInto<U>>::Error",
    "<kmipkit_client::Pkcs11Request as core::convert::TryInto<U>>::try_into",
}
EXPECTED_RESPONSE_DIRECT = {
    "<kmipkit_client::Pkcs11Response as core::any::Any>::type_id",
    "<kmipkit_client::Pkcs11Response as core::borrow::Borrow<kmipkit_protocol::pkcs11::Pkcs11Response>>::borrow",
    "<kmipkit_client::Pkcs11Response as core::borrow::BorrowMut<kmipkit_protocol::pkcs11::Pkcs11Response>>::borrow_mut",
    "<kmipkit_client::Pkcs11Response as core::convert::From<kmipkit_protocol::pkcs11::Pkcs11Response>>::from",
    "<kmipkit_client::Pkcs11Response as core::fmt::Debug>::fmt",
}
EXPECTED_RESPONSE_CONDITIONAL = {
    "<kmipkit_client::Pkcs11Response as core::convert::Into<U>>::into",
    "<kmipkit_client::Pkcs11Response as core::convert::TryFrom<U>>::Error",
    "<kmipkit_client::Pkcs11Response as core::convert::TryFrom<U>>::try_from",
    "<kmipkit_client::Pkcs11Response as core::convert::TryInto<U>>::Error",
    "<kmipkit_client::Pkcs11Response as core::convert::TryInto<U>>::try_into",
}
EXPECTED_TRAITS = (
    EXPECTED_REQUEST_DIRECT
    | EXPECTED_REQUEST_CONDITIONAL
    | EXPECTED_RESPONSE_DIRECT
    | EXPECTED_RESPONSE_CONDITIONAL
)


class T102Pkcs11ClientRootTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.rust_report = json.loads(RUST_REPORT.read_text(encoding="utf-8"))

    def test_rust_1_94_report_observes_exact_direct_and_conditional_paths(self) -> None:
        self.assertTrue(self.rust_report["complete"])
        self.assertTrue(self.rust_report["matchesManifest"])
        self.assertEqual(self.rust_report["toolchain"], "1.94.0")
        self.assertEqual(self.rust_report["rustdocFormat"], 57)

        for prefix, expected_direct, expected_conditional in (
            (REQUEST_PREFIX, EXPECTED_REQUEST_DIRECT, EXPECTED_REQUEST_CONDITIONAL),
            (RESPONSE_PREFIX, EXPECTED_RESPONSE_DIRECT, EXPECTED_RESPONSE_CONDITIONAL),
        ):
            with self.subTest(prefix=prefix):
                direct = {
                    row["exportName"]
                    for row in self.rust_report["declarations"]
                    if row.get("packageId") == "rust-kmipkit-client"
                    and row.get("exportName", "").startswith(prefix)
                }
                conditional = {
                    row["exportName"]
                    for row in self.rust_report["conditionalDeclarations"]
                    if row.get("packageId") == "rust-kmipkit-client"
                    and row.get("exportName", "").startswith(prefix)
                }
                excluded = {
                    row["exportName"]
                    for row in self.rust_report["excluded"]
                    if row.get("packageId") == "rust-kmipkit-client"
                    and row.get("exportName", "").startswith(prefix)
                }
                self.assertEqual(direct, expected_direct)
                self.assertEqual(conditional, expected_conditional)
                self.assertEqual(len(direct), 5)
                self.assertEqual(len(conditional), 5)
                self.assertEqual(excluded, expected_direct | expected_conditional)

    def test_client_root_traits_are_exact_approved_exclusions(self) -> None:
        declarations = {
            row["exportName"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName") in EXPECTED_TRAITS
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName") in EXPECTED_TRAITS
        }

        self.assertEqual(declarations, set())
        self.assertEqual(set(exclusions), EXPECTED_TRAITS)
        self.assertEqual(len(exclusions), 20)
        approval = OWNER_SPEC.read_text(encoding="utf-8")
        self.assertIn("**Status**: Approved for implementation — 2026-10-11", approval)

        for export_name, row in exclusions.items():
            with self.subTest(export_name=export_name):
                self.assertEqual(row["approvalSource"], OWNER_APPROVAL_SOURCE)
                for evidence in (
                    "KMIPKIT-0040-FR-001",
                    "KMIPKIT-0040-FR-004",
                    "KMIPKIT-0043-FR-010",
                    "KMIPKIT-0043-FR-021",
                    "OASIS Key Management Interoperability Protocol Specification v2.1 §6.1.37 Tables 273–275",
                    "These compiler-generated Rust traits add no OASIS requirement",
                    "KMIPKIT-0040-FR-005 applies only to unchanged redaction behavior",
                    "No conformance, interoperability, profile-support, or certification claim",
                ):
                    self.assertIn(evidence, row["evidence"])
                self.assertIn("compiler-visible language behavior", row["reason"])
                self.assertIn("not a separate KMIP client capability", row["reason"])
                self.assertIn("typed PKCS#11 model", row["reason"])

    def test_functional_mappings_operation_state_and_t022_remain_unchanged(self) -> None:
        expected_states = {
            "kmipkit_client::Pkcs11Request": "verified",
            "kmipkit_client::Pkcs11Request::new": "verified",
            "kmipkit_client::Pkcs11Request::into_ttlv_payload": "blocked",
            "kmipkit_client::Pkcs11Request::with_correlation_value": "blocked",
            "kmipkit_client::Pkcs11Request::with_extension": "blocked",
            "kmipkit_client::Pkcs11Request::with_input_parameters": "blocked",
            "kmipkit_client::Pkcs11Request::with_interface": "blocked",
            "kmipkit_client::Pkcs11Response": "verified",
            "kmipkit_client::Pkcs11Response::result": "blocked",
            "kmipkit_client::Pkcs11Response::try_from_response_item": "blocked",
            "kmipkit_client::Pkcs11Response::with_correlation_value": "blocked",
            "kmipkit_client::Pkcs11Response::with_function": "blocked",
            "kmipkit_client::Pkcs11Response::with_interface": "blocked",
            "kmipkit_client::Pkcs11Response::with_output_parameters": "blocked",
            "kmipkit_client::Pkcs11Response::with_return_code": "blocked",
            "kmipkit_client::Pkcs11Response::with_ttlv": "verified",
            "kmipkit_client::Pkcs11Response::with_ttlv_payload": "verified",
        }
        client_rows = {
            row["exportName"]: row["declarationState"]
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName") in expected_states
        }
        self.assertEqual(client_rows, expected_states)

        operation_rows = [
            row
            for row in self.manifest["operations"]
            if row.get("operationId") == "KMIPKIT-ELEM-OP-C2S-PKCS-11"
        ]
        self.assertEqual(len(operation_rows), 1)
        operation = operation_rows[0]
        self.assertEqual(operation["apiState"], "verified")
        self.assertEqual(operation["featureSpecId"], "KMIPKIT-0040")
        self.assertEqual(
            operation["ownerDependency"],
            {"specId": "KMIPKIT-0040", "state": "approved"},
        )
        self.assertEqual(operation["scope"], "client_1_0")
        self.assertEqual(operation["section"], "6.1.37")
        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, 1588)
        self.assertEqual(len(self.manifest["publicDeclarations"]), 6786)
        self.assertEqual(len(self.manifest["approvedExclusions"]), 7938)
        self.assertEqual(self.rust_report["counts"]["manifest"], 3871)
        self.assertEqual(self.rust_report["counts"]["excluded"], 6592)

        pkcs11_tasks = (ROOT / "specs" / "026-pkcs11" / "tasks.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("- [ ] T022 ", pkcs11_tasks)


if __name__ == "__main__":
    unittest.main()
