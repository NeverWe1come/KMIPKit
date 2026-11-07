"""Pin the next exact, mixed-owner T102 Rust-trait inventory batch."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = ROOT / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-11-01.json"

COMMON_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
COMMON_EVIDENCE = (
    "No OASIS normative clause directly applies to Rust standard-trait paths",
    "inventory-only",
    "makes no OASIS conformance, interoperability, profile-support, or certification claim",
)

SESSION_OWNER = "specs/024-credential-sessions/spec.md"
SESSION_APPROVAL = {
    "kind": "feature-specification",
    "path": SESSION_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
SESSION_REQUIREMENTS = (
    "KMIPKIT-0038-FR-001",
    "KMIPKIT-0038-FR-002",
    "KMIPKIT-0038-FR-004",
    "KMIPKIT-0038-FR-005",
    "KMIPKIT-0038-FR-008",
)
SESSION_TYPES = {
    "TicketType": (21, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 11.57 Table 488"),
    "Ticket": (12, "OASIS Key Management Interoperability Protocol Specification v2.1 Sections 7.39 Table 391 and 11.57 Table 488"),
    "LoginRequest": (11, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.30 Tables 253-255"),
    "DelegatedLoginRequest": (10, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.12"),
    "DelegatedLoginResponse": (10, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.12"),
    "LoginResponse": (10, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.30"),
    "LogoutRequest": (10, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.31"),
    "LogoutResponse": (10, "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.31"),
}

CREDENTIAL_OWNER = "specs/008-credentials-attestation/spec.md"
CREDENTIAL_APPROVAL = {
    "kind": "feature-specification",
    "path": CREDENTIAL_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation under the delegated authorization recorded in [approval-record.md](approval-record.md).",
}
CREDENTIAL_REQUIREMENTS = (
    "KMIPKIT-0008-FR-001",
    "KMIPKIT-0008-FR-003",
    "KMIPKIT-0008-FR-010",
    "KMIPKIT-0008-FR-011",
)

PKCS11_OWNER = "specs/026-pkcs11/spec.md"
PKCS11_APPROVAL = {
    "kind": "feature-specification",
    "path": PKCS11_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
PKCS11_REQUIREMENTS = (
    "KMIPKIT-0040-FR-001",
    "KMIPKIT-0040-FR-002",
    "KMIPKIT-0040-FR-004",
    "KMIPKIT-0040-FR-005",
    "KMIPKIT-0040-FR-007",
)

CRYPTO_OWNER = "specs/019-cryptographic-operations/spec.md"
CRYPTO_APPROVAL = {
    "kind": "feature-specification",
    "path": CRYPTO_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation under the maintainer's standing direct instruction to execute the complete KMIPKit plan autonomously (reaffirmed 2026-10-19).",
}
CRYPTO_REQUIREMENTS = (
    "KMIPKIT-0019-FR-001",
    "KMIPKIT-0019-FR-007",
    "KMIPKIT-0019-FR-009",
)


class T102MixedTraitBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def assert_exact_inventory_exclusions(
        self,
        *,
        package: str,
        prefix: str,
        expected_count: int,
        owner: str,
        approval: dict[str, str],
        requirements: tuple[str, ...],
        normative_source: str,
        reason_type: str,
        extra_evidence: tuple[str, ...] = (),
    ) -> None:
        observed = {
            (row["packageId"], row["exportName"])
            for field in ("declarations", "conditionalDeclarations")
            for row in self.report[field]
            if row.get("packageId") == package
            and row.get("exportName", "").startswith(prefix)
        }
        declarations = {
            (row["packageId"], row["exportName"])
            for row in self.manifest["publicDeclarations"]
            if row.get("packageId") == package
            and row.get("exportName", "").startswith(prefix)
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest.get("approvedExclusions", [])
            if row.get("packageId") == package
            and row.get("exportName", "").startswith(prefix)
            and row.get("candidateSpecId") != "KMIPKIT-0053"
        }

        with self.subTest(package=package, prefix=prefix, check="compiler inventory"):
            self.assertEqual(len(observed), expected_count)
        with self.subTest(package=package, prefix=prefix, check="manifest declarations"):
            self.assertEqual(declarations, set())
        with self.subTest(package=package, prefix=prefix, check="exact exclusions"):
            self.assertEqual(set(exclusions), observed)

        for row in exclusions.values():
            with self.subTest(package=package, export_name=row["exportName"]):
                self.assertEqual(row["approvalSource"], approval)
                self.assertEqual(row["approvalSource"]["path"], owner)
                for requirement in (*requirements, *COMMON_REQUIREMENTS):
                    self.assertIn(requirement, row["evidence"])
                for evidence in (*COMMON_EVIDENCE, normative_source, *extra_evidence):
                    self.assertIn(evidence.casefold(), row["evidence"].casefold())
                self.assertIn("exact Rust standard-trait implementation", row["reason"])
                self.assertIn(reason_type, row["reason"])

    def test_mixed_owner_trait_paths_are_exact_approved_inventory_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        for type_name, (count, normative_source) in SESSION_TYPES.items():
            with self.subTest(owner=SESSION_OWNER, type_name=type_name):
                self.assert_exact_inventory_exclusions(
                    package="rust-kmipkit",
                    prefix=f"<kmipkit::{type_name} as ",
                    expected_count=count,
                    owner=SESSION_OWNER,
                    approval=SESSION_APPROVAL,
                    requirements=SESSION_REQUIREMENTS,
                    normative_source=normative_source,
                    reason_type=type_name,
                    extra_evidence=("the root re-export remains separately classified",),
                )

        self.assert_exact_inventory_exclusions(
            package="rust-kmipkit-protocol",
            prefix="<kmipkit_protocol::CredentialView<'a> as ",
            expected_count=10,
            owner=CREDENTIAL_OWNER,
            approval=CREDENTIAL_APPROVAL,
            requirements=CREDENTIAL_REQUIREMENTS,
            normative_source="OASIS Key Management Interoperability Protocol Specification v2.1 Sections 9.4 and 9.11",
            reason_type="CredentialView",
            extra_evidence=("CredentialView Debug exposes only the credential type",),
        )

        for package, prefix in (
            ("rust-kmipkit-client", "<kmipkit_client::Pkcs11Error as "),
            ("rust-kmipkit-protocol", "<kmipkit_protocol::Pkcs11Error as "),
        ):
            self.assert_exact_inventory_exclusions(
                package=package,
                prefix=prefix,
                expected_count=26,
                owner=PKCS11_OWNER,
                approval=PKCS11_APPROVAL,
                requirements=PKCS11_REQUIREMENTS,
                normative_source="OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.37 Tables 273-275",
                reason_type="Pkcs11Error",
                extra_evidence=(
                    "formalAcceptance remains false",
                    "The Pkcs11Error type and variants map through the shared normalized four-language client outcome",
                    "Pkcs11Request and Pkcs11Response declarations retain their existing states",
                ),
            )

        for package, module in (
            ("rust-kmipkit-client", "kmipkit_client"),
            ("rust-kmipkit-protocol", "kmipkit_protocol"),
        ):
            declarations = {
                row["exportName"]: row["declarationState"]
                for row in self.manifest["publicDeclarations"]
                if row.get("packageId") == package
                and row.get("exportName") in {
                    f"{module}::Pkcs11Request",
                    f"{module}::Pkcs11Response",
                    f"{module}::Pkcs11Error",
                }
            }
            self.assertEqual(
                declarations,
                {
                    f"{module}::Pkcs11Request": "verified",
                    f"{module}::Pkcs11Response": "verified",
                    f"{module}::Pkcs11Error": "declared",
                },
            )

        self.assert_exact_inventory_exclusions(
            package="rust-kmipkit-protocol",
            prefix="<kmipkit_protocol::OperationData as ",
            expected_count=12,
            owner=CRYPTO_OWNER,
            approval=CRYPTO_APPROVAL,
            requirements=CRYPTO_REQUIREMENTS,
            normative_source="OASIS Key Management Interoperability Protocol Specification v2.1 Section 7.9 and Sections 6.1.11/6.1.17, Tables 197/198 and 215/216",
            reason_type="OperationData",
            extra_evidence=(
                "Debug and Display use fixed redaction labels",
                "Byte String storage uses SecretBytes",
            ),
        )


if __name__ == "__main__":
    unittest.main()
