"""Pin the safe message, TTLV, async, Certify, and policy T102 trait paths."""

from __future__ import annotations

import json
import unittest

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
REPORT = ROOT / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-11-01.json"

T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
CONDITIONAL_SUFFIXES = (
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
)
COMMON_EVIDENCE = (
    "No OASIS normative clause directly applies to Rust standard-trait paths",
    "inventory-only",
    "makes no OASIS conformance, interoperability, profile-support, or certification claim",
)

MESSAGE_OWNER = "specs/006-message-batch-model/spec.md"
MESSAGE_APPROVAL = {
    "kind": "feature-specification",
    "path": MESSAGE_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation under delegated maintainer authorization; see [approval-record.md](approval-record.md).",
}
MESSAGE_REQUIREMENTS = (
    "KMIPKIT-0006-FR-001",
    "KMIPKIT-0006-FR-014",
    "KMIPKIT-0006-FR-021",
)

TTLV_OWNER = "specs/004-generic-ttlv-model/spec.md"
TTLV_APPROVAL = {
    "kind": "feature-specification",
    "path": TTLV_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved",
}
TTLV_REQUIREMENTS = (
    "KMIPKIT-0004-FR-006",
    "KMIPKIT-0004-FR-009",
    "KMIPKIT-0004-FR-010",
    "KMIPKIT-0004-FR-012",
)

ASYNC_OWNER = "specs/009-asynchronous-operations/spec.md"
ASYNC_APPROVAL = {
    "kind": "feature-specification",
    "path": ASYNC_OWNER,
    "status": "Approved",
    "evidence": "Approved for implementation by the repository maintainer on 2026-10-29",
}
ASYNC_REQUIREMENTS = (
    "KMIPKIT-0009-FR-003",
    "KMIPKIT-0009-FR-005",
    "KMIPKIT-0009-FR-007",
    "KMIPKIT-0009-FR-009",
    "KMIPKIT-0009-FR-010",
)

LIFECYCLE_OWNER = "specs/022-object-lifecycle/spec.md"
LIFECYCLE_APPROVAL = {
    "kind": "feature-specification",
    "path": LIFECYCLE_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
CERTIFY_REQUIREMENTS = (
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-005",
)

POLICY_OWNER = "specs/025-policy-controls/spec.md"
POLICY_APPROVAL = {
    "kind": "feature-specification",
    "path": POLICY_OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
POLICY_REQUIREMENTS = (
    "KMIPKIT-0039-FR-001",
    "KMIPKIT-0039-FR-002",
    "KMIPKIT-0039-FR-005",
    "KMIPKIT-0039-FR-009",
)

def group(
    *,
    package: str,
    prefix: str,
    count: int,
    direct: int,
    owner: str,
    candidate_id: str,
    approval: dict[str, str],
    requirements: tuple[str, ...],
    normative: str,
    type_name: str,
    extra: tuple[str, ...] = (),
) -> dict[str, object]:
    return {
        "package": package,
        "prefix": prefix,
        "count": count,
        "direct": direct,
        "conditional": count - direct,
        "owner": owner,
        "candidate_id": candidate_id,
        "approval": approval,
        "requirements": requirements,
        "normative": normative,
        "type_name": type_name,
        "extra": extra,
    }


GROUPS = (
    *(
        group(
            package="rust-kmipkit-protocol",
            prefix=f"<{name}<'a> as ",
            count=15,
            direct=9,
            owner=MESSAGE_OWNER,
            candidate_id="KMIPKIT-0006",
            approval=MESSAGE_APPROVAL,
            requirements=MESSAGE_REQUIREMENTS,
            normative="OASIS Key Management Interoperability Protocol Specification v2.1 Sections 8.1-8.6, 9.1-9.21, and 10.1.2; Tables 394-409 and 417-426",
            type_name=name.rsplit("::", 1)[-1],
        )
        for name in (
            "kmipkit_protocol::RequestHeaderView",
            "kmipkit_protocol::ResponseHeaderView",
            "kmipkit_protocol::RequestBatchItemView",
            "kmipkit_protocol::ResponseBatchItemView",
            "kmipkit_protocol::MessageExtensionView",
        )
    ),
    *(
        group(
            package="rust-kmipkit-ttlv",
            prefix=f"<{name}<'a> as ",
            count=10,
            direct=4,
            owner=TTLV_OWNER,
            candidate_id="KMIPKIT-0004",
            approval=TTLV_APPROVAL,
            requirements=TTLV_REQUIREMENTS,
            normative="OASIS Key Management Interoperability Protocol Specification v2.1 Sections 10.1.2 and 11.23",
            type_name=name.rsplit("::", 1)[-1],
            extra=("Debug reports TTLV metadata without revealing values",),
        )
        for name in (
            "kmipkit_ttlv::StructureView",
            "kmipkit_ttlv::ValueView",
        )
    ),
    *(
        group(
            package="rust-kmipkit-protocol",
            prefix=f"<kmipkit_protocol::{name}<'a> as ",
            count=10,
            direct=4,
            owner=ASYNC_OWNER,
            candidate_id="KMIPKIT-0009",
            approval=ASYNC_APPROVAL,
            requirements=ASYNC_REQUIREMENTS,
            normative=normative,
            type_name=name,
            extra=(
                "response correlation remains redacted",
                *(
                    ("Query Asynchronous Requests response remains generic under KMIPKIT-0009-FR-007",)
                    if name == "QueryAsyncRequestsResponse"
                    else ()
                ),
            ),
        )
        for name, normative in (
            (
                "PollResponse",
                "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.38, Tables 276-277, and Section 8.6, Table 399",
            ),
            (
                "ProcessResponse",
                "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.39, Tables 278-280, and Section 8.6, Table 399",
            ),
            (
                "QueryAsyncRequestsResponse",
                "OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.41, Tables 285-287, and Section 8.6, Table 399",
            ),
        )
    ),
    group(
        package="rust-kmipkit-protocol",
        prefix="<kmipkit_protocol::CertifyRequest as ",
        count=11,
        direct=6,
        owner=LIFECYCLE_OWNER,
        candidate_id="KMIPKIT-0036",
        approval=LIFECYCLE_APPROVAL,
        requirements=CERTIFY_REQUIREMENTS,
        normative="OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.6, Tables 179-181",
        type_name="CertifyRequest",
        extra=("Certificate Request Value is represented by presence only in Debug",),
    ),
    group(
        package="rust-kmipkit",
        prefix="<kmipkit::UsageAllocationContext as ",
        count=10,
        direct=5,
        owner=POLICY_OWNER,
        candidate_id="KMIPKIT-0039",
        approval=POLICY_APPROVAL,
        requirements=POLICY_REQUIREMENTS,
        normative="OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.23, Tables 232-234",
        type_name="UsageAllocationContext",
        extra=("Debug omits the identifier and reports only permitted local context state",),
    ),
    group(
        package="rust-kmipkit",
        prefix="<kmipkit::SetDefaultsResponse as ",
        count=10,
        direct=5,
        owner=POLICY_OWNER,
        candidate_id="KMIPKIT-0039",
        approval=POLICY_APPROVAL,
        requirements=POLICY_REQUIREMENTS,
        normative="OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.53, Tables 328-330",
        type_name="SetDefaultsResponse",
        extra=("Debug redacts the response TTLV",),
    ),
    group(
        package="rust-kmipkit",
        prefix="<kmipkit::GetUsageAllocationResponse as ",
        count=10,
        direct=5,
        owner=POLICY_OWNER,
        candidate_id="KMIPKIT-0039",
        approval=POLICY_APPROVAL,
        requirements=POLICY_REQUIREMENTS,
        normative="OASIS Key Management Interoperability Protocol Specification v2.1 Section 6.1.23, Tables 232-234",
        type_name="GetUsageAllocationResponse",
        extra=("Debug redacts the response TTLV",),
    ),
)

HOLDOUTS = (
    ("rust-kmipkit-client", "<kmipkit_client::ClientResponseView<'a> as ", 16),
    ("rust-kmipkit-protocol", "<kmipkit_protocol::DerivationParameters as ", 11),
    ("rust-kmipkit-protocol", "<kmipkit_protocol::DeriveKeyRequest as ", 10),
    ("rust-kmipkit", "<kmipkit::RngRetrieveResponse as ", 10),
    ("rust-kmipkit", "<kmipkit::RngSeedResponse as ", 10),
    (
        "rust-kmipkit",
        "<kmipkit::GetUsageAllocationRequest as core::fmt::Debug>::fmt",
        1,
    ),
)
HOLDOUT_OWNERS = (
    ("rust-kmipkit-client", "<kmipkit_client::ClientResponseView<'a> as ", "specs/007-client-execution/spec.md"),
    ("rust-kmipkit-protocol", "<kmipkit_protocol::DerivationParameters as ", "specs/023-key-material-transfer/spec.md"),
    ("rust-kmipkit-protocol", "<kmipkit_protocol::DeriveKeyRequest as ", "specs/023-key-material-transfer/spec.md"),
    ("rust-kmipkit", "<kmipkit::RngRetrieveResponse as ", "specs/027-rng-operations/spec.md"),
    ("rust-kmipkit", "<kmipkit::RngSeedResponse as ", "specs/027-rng-operations/spec.md"),
    ("rust-kmipkit", "<kmipkit::GetUsageAllocationRequest as ", "specs/025-policy-controls/spec.md"),
)
REDACTION_REVIEW_PENDING: set[tuple[str, str]] = set()


class T102ViewsAsyncPolicyTraitsBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_safe_trait_paths_are_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])

        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest.get("approvedExclusions", [])
        }
        selected: set[tuple[str, str]] = set()

        for group in GROUPS:
            package = group["package"]
            prefix = group["prefix"]
            direct = {
                (row["packageId"], row["exportName"])
                for row in self.report["declarations"]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(prefix)
            }
            conditional = {
                (row["packageId"], row["exportName"])
                for row in self.report["conditionalDeclarations"]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(prefix)
            }
            expected = direct | conditional
            selected.update(expected)
            self.assertEqual(len(direct), group["direct"], prefix)
            self.assertEqual(len(conditional), group["conditional"], prefix)
            self.assertEqual(len(expected), group["count"], prefix)

            # The Green state moves each compiler-observed path out of the
            # declarations list and into exactly one approved exclusion.
            self.assertFalse(expected.intersection(declarations), prefix)
            self.assertEqual(expected.intersection(exclusions), expected, prefix)

            for key in expected:
                row = exclusions[key]
                evidence = row["evidence"]
                self.assertEqual(row["approvalSource"], group["approval"])
                self.assertEqual(row["approvalSource"]["path"], group["owner"])
                for requirement in (*group["requirements"], *T102_REQUIREMENTS):
                    self.assertIn(requirement, evidence)
                for required_text in (*COMMON_EVIDENCE, group["normative"], *group["extra"]):
                    self.assertIn(required_text.casefold(), evidence.casefold())
                self.assertIn("exact Rust standard-trait implementation", row["reason"])
                self.assertIn("not a separate kmip client capability", row["reason"].casefold())
                self.assertIn(group["type_name"], row["reason"])

        self.assertEqual(len(selected), 166)
        conditional_paths = {
            (row["packageId"], row["exportName"])
            for row in self.report["conditionalDeclarations"]
        }
        self.assertEqual(len(selected.intersection(conditional_paths)), 80)
        self.assertEqual(len(selected) - 80, 86)

    def test_all_remaining_traits_have_specific_owner_backed_exclusions(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest.get("approvedExclusions", [])
        }
        holdouts: set[tuple[str, str]] = set()
        for package, prefix, count in HOLDOUTS:
            paths = {
                (row.get("packageId"), row.get("exportName"))
                for field in ("declarations", "conditionalDeclarations")
                for row in self.report[field]
                if row.get("packageId") == package
                and row.get("exportName", "").startswith(prefix)
            }
            self.assertEqual(len(paths), count, prefix)
            holdouts.update(paths)

        self.assertEqual(len(holdouts), 58)
        self.assertTrue(REDACTION_REVIEW_PENDING.issubset(holdouts))
        resolved = holdouts - REDACTION_REVIEW_PENDING
        self.assertEqual(58, len(resolved))
        self.assertTrue(resolved.isdisjoint(declarations))
        self.assertEqual(resolved, resolved.intersection(exclusions))

        for path in resolved:
            package, export_name = path
            owner = next(
                owner_path
                for owner_package, prefix, owner_path in HOLDOUT_OWNERS
                if package == owner_package and export_name.startswith(prefix)
            )
            owner_id = {
                "specs/007-client-execution/spec.md": "KMIPKIT-0007",
                "specs/023-key-material-transfer/spec.md": "KMIPKIT-0037",
                "specs/027-rng-operations/spec.md": "KMIPKIT-0041",
                "specs/025-policy-controls/spec.md": "KMIPKIT-0039",
            }[owner]
            exclusion = exclusions[path]
            self.assertEqual(owner_id, exclusion["candidateSpecId"])
            self.assertEqual(owner, exclusion["approvalSource"]["path"])
            self.assertEqual("Approved", exclusion["approvalSource"]["status"])
            self.assertIn("exact Rust standard-trait implementation", exclusion["reason"])
            self.assertIn("not a separate KMIPKit product capability", exclusion["reason"])
            for requirement in T102_REQUIREMENTS:
                self.assertIn(requirement, exclusion["evidence"])
            if "core::fmt::Debug>::fmt" in export_name:
                self.assertIn("KMIPKIT-0043-FR-013", exclusion["evidence"])
            if "ClientResponseView" in export_name and "core::fmt::Debug" in export_name:
                self.assertIn("this Debug path emits only the operation label", exclusion["evidence"])
            if "RngRetrieveResponse" in export_name and "core::fmt::Debug" in export_name:
                self.assertIn("KMIP_RNG_RETRIEVE_RESPONSE_SECRET_SENTINEL", exclusion["evidence"])

    def test_public_parent_states_and_candidate_totals_are_preserved(self) -> None:
        expected_parents = {
            ("rust-kmipkit-protocol", "kmipkit_protocol::RequestHeaderView"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::ResponseHeaderView"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::RequestBatchItemView"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::ResponseBatchItemView"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::MessageExtensionView"): "declared",
            ("rust-kmipkit-ttlv", "kmipkit_ttlv::StructureView"): "declared",
            ("rust-kmipkit-ttlv", "kmipkit_ttlv::ValueView"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::PollResponse"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::ProcessResponse"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::QueryAsyncRequestsResponse"): "declared",
            ("rust-kmipkit-protocol", "kmipkit_protocol::CertifyRequest"): "verified",
            ("rust-kmipkit", "kmipkit::UsageAllocationContext"): "declared",
            ("rust-kmipkit", "kmipkit::SetDefaultsResponse"): "verified",
            ("rust-kmipkit", "kmipkit::GetUsageAllocationResponse"): "verified",
        }
        declarations = {
            (row["packageId"], row["exportName"]): row["declarationState"]
            for row in self.manifest["publicDeclarations"]
        }
        for key, expected_state in expected_parents.items():
            self.assertEqual(declarations[key], expected_state)

        candidates = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043"
            and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(candidates, T102_BLOCKED_CANDIDATES)
        self.assertEqual(len(self.manifest["publicDeclarations"]), T102_PUBLIC_DECLARATIONS)
        self.assertEqual(len(self.manifest["approvedExclusions"]), T102_APPROVED_EXCLUSIONS)


if __name__ == "__main__":
    unittest.main()
