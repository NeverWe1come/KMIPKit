"""T093 contract for KMIPKIT-0009 public declaration parity."""

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
OWNER_SPEC = "KMIPKIT-0009"
OWNER_PATH = "specs/009-asynchronous-operations/spec.md"
OWNER_APPROVAL = "Approved for implementation by the repository maintainer on 2026-10-29"
LANGUAGES = {"rust", "c", "java", "python"}
PROJECT_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}

EXPECTED_FUNCTIONAL_EXPORTS = {
    "rust-kmipkit-protocol": {
        "kmipkit_protocol::AsynchronousOperationError",
        "kmipkit_protocol::AsynchronousOperationError::UnexpectedOperation",
        "kmipkit_protocol::AsynchronousOperationError::MissingResultStatus",
        "kmipkit_protocol::AsynchronousOperationError::InvalidOperationResult",
        "kmipkit_protocol::AsynchronousOperationError::MissingAsynchronousCorrelationValue",
        "kmipkit_protocol::AsynchronousOperationError::MissingResponsePayload",
        "kmipkit_protocol::AsynchronousOperationError::UnexpectedResponsePayload",
        "kmipkit_protocol::AsynchronousOperationError::MalformedResponsePayload",
        "kmipkit_protocol::AsynchronousOperationError::ForbiddenResultStatus",
        "kmipkit_protocol::CancelRequest",
        "kmipkit_protocol::CancelRequest::new",
        "kmipkit_protocol::CancelRequest::asynchronous_correlation_value",
        "kmipkit_protocol::CancelRequest::to_ttlv_payload",
        "kmipkit_protocol::CancelResponse",
        "kmipkit_protocol::CancelResponse::try_from_response_item",
        "kmipkit_protocol::CancelResponse::result",
        "kmipkit_protocol::CancelResponse::with_asynchronous_correlation_value",
        "kmipkit_protocol::CancelResponse::cancellation_result",
        "kmipkit_protocol::CancellationResult",
        "kmipkit_protocol::CancellationResult::Canceled",
        "kmipkit_protocol::CancellationResult::UnableToCancel",
        "kmipkit_protocol::CancellationResult::Completed",
        "kmipkit_protocol::CancellationResult::Failed",
        "kmipkit_protocol::CancellationResult::Unavailable",
        "kmipkit_protocol::CancellationResult::Unknown",
        "kmipkit_protocol::CancellationResult::from_raw",
        "kmipkit_protocol::CancellationResult::raw",
        "kmipkit_protocol::PollRequest",
        "kmipkit_protocol::PollRequest::new",
        "kmipkit_protocol::PollRequest::asynchronous_correlation_value",
        "kmipkit_protocol::PollRequest::to_ttlv_payload",
        "kmipkit_protocol::PollResponse",
        "kmipkit_protocol::PollResponse::try_from_response_item",
        "kmipkit_protocol::PollResponse::result",
        "kmipkit_protocol::PollResponse::is_pending",
        "kmipkit_protocol::PollResponse::with_asynchronous_correlation_value",
        "kmipkit_protocol::PollResponse::with_response_payload",
        "kmipkit_protocol::ProcessRequest",
        "kmipkit_protocol::ProcessRequest::new",
        "kmipkit_protocol::ProcessRequest::to_ttlv_payload",
        "kmipkit_protocol::ProcessResponse",
        "kmipkit_protocol::ProcessResponse::try_from_response_item",
        "kmipkit_protocol::ProcessResponse::result",
        "kmipkit_protocol::ProcessResponse::is_pending",
        "kmipkit_protocol::ProcessResponse::payload_member_count",
        "kmipkit_protocol::ProcessResponse::with_asynchronous_correlation_value",
        "kmipkit_protocol::ProcessResponse::with_response_payload",
        "kmipkit_protocol::QueryAsyncRequestsRequest",
        "kmipkit_protocol::QueryAsyncRequestsRequest::new",
        "kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values",
        "kmipkit_protocol::QueryAsyncRequestsRequest::with_operations",
        "kmipkit_protocol::QueryAsyncRequestsRequest::to_ttlv_payload",
        "kmipkit_protocol::QueryAsyncRequestsResponse",
        "kmipkit_protocol::QueryAsyncRequestsResponse::try_from_response_item",
        "kmipkit_protocol::QueryAsyncRequestsResponse::result",
        "kmipkit_protocol::QueryAsyncRequestsResponse::with_response_payload",
        "kmipkit_protocol::AsynchronousOperationError::InvalidOperationResult::0",
        "kmipkit_protocol::CancellationResult::Unknown::0",
    },
    "rust-kmipkit-client": {
        "kmipkit_client::ClientOperation::Poll",
        "kmipkit_client::ClientOperation::Cancel",
        "kmipkit_client::ClientOperation::Process",
    },
    "rust-kmipkit": {
        "kmipkit::CancelRequest",
        "kmipkit::PollRequest",
        "kmipkit::ProcessRequest",
        "kmipkit::QueryAsyncRequestsRequest",
        "kmipkit::ClientOperation::Poll",
        "kmipkit::ClientOperation::Cancel",
        "kmipkit::ClientOperation::Process",
        "kmipkit::CancelRequest::new",
        "kmipkit::CancelRequest::asynchronous_correlation_value",
        "kmipkit::CancelRequest::to_ttlv_payload",
        "kmipkit::PollRequest::new",
        "kmipkit::PollRequest::asynchronous_correlation_value",
        "kmipkit::PollRequest::to_ttlv_payload",
        "kmipkit::ProcessRequest::new",
        "kmipkit::ProcessRequest::to_ttlv_payload",
        "kmipkit::QueryAsyncRequestsRequest::new",
        "kmipkit::QueryAsyncRequestsRequest::with_correlation_values",
        "kmipkit::QueryAsyncRequestsRequest::with_operations",
        "kmipkit::QueryAsyncRequestsRequest::to_ttlv_payload",
    },
}

ASYNC_TRAIT_ROOTS = {
    "kmipkit::CancelRequest",
    "kmipkit::PollRequest",
    "kmipkit::ProcessRequest",
    "kmipkit::QueryAsyncRequestsRequest",
    "kmipkit_protocol::AsynchronousOperationError",
    "kmipkit_protocol::CancelRequest",
    "kmipkit_protocol::CancellationResult",
    "kmipkit_protocol::PollRequest",
    "kmipkit_protocol::ProcessRequest",
    "kmipkit_protocol::QueryAsyncRequestsRequest",
}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def expected_trait_exports() -> set[tuple[str, str]]:
    report = load_json(RUST_REPORT)
    rows = report["declarations"] + report["conditionalDeclarations"]
    return {
        (row["packageId"], row["exportName"])
        for row in rows
        if any(row.get("exportName", "").startswith(f"<{root} as ") for root in ASYNC_TRAIT_ROOTS)
    }


class T093AsyncDeclarationParityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = load_json(MANIFEST)

    def test_all_functional_async_exports_have_four_language_owner_mappings(self) -> None:
        declarations = self.manifest["publicDeclarations"]
        expected = {
            (package, export)
            for package, exports in EXPECTED_FUNCTIONAL_EXPORTS.items()
            for export in exports
        }
        actual = {
            (row.get("packageId"), row.get("exportName"))
            for row in declarations
            if row.get("capabilityId") == "client_asynchronous_operations"
            and row.get("approvalSource", {}).get("path") == OWNER_PATH
            and (row.get("packageId"), row.get("exportName")) in expected
        }
        self.assertEqual(expected, actual, "the exact KMIPKIT-0009 functional export set changed")

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row.get("capabilityId") == "client_asynchronous_operations"
        )
        self.assertEqual(
            sorted(set(capability["requirementIds"])),
            capability["requirementIds"],
        )
        for mappings in capability["mappings"].values():
            self.assertEqual(sorted(set(mappings)), mappings)
        for package, export in sorted(expected):
            row = next(
                item
                for item in declarations
                if item.get("packageId") == package and item.get("exportName") == export
            )
            with self.subTest(export=export):
                self.assertEqual("declared", row.get("declarationState"))
                self.assertNotIn("candidateSpecId", row)
                self.assertEqual("four-language", row.get("parityScope"))
                self.assertEqual("client_asynchronous_operations", row.get("capabilityId"))
                self.assertEqual(
                    {"kind": "feature-specification", "path": OWNER_PATH,
                     "status": "Approved", "evidence": OWNER_APPROVAL},
                    row.get("approvalSource"),
                )
                self.assertEqual(LANGUAGES, set(row.get("languageMappings", {})))
                self.assertEqual(export, row["languageMappings"]["rust"]["name"])
                self.assertEqual(package, row["languageMappings"]["rust"]["packageId"])
                self.assertTrue(
                    PROJECT_REQUIREMENTS <= set(row.get("requirementIds", [])),
                    "the owner mapping must cite all T093 API parity requirements",
                )
                self.assertTrue(
                    any(value.startswith("KMIPKIT-0009-FR-") for value in row["requirementIds"]),
                    "the mapping must cite its accepted asynchronous-operation owner",
                )
                for language in ("c", "java", "python"):
                    mapping = row["languageMappings"][language]
                    self.assertTrue(mapping.get("name"))
                    self.assertIn(
                        mapping["name"], capability["mappings"][language],
                        f"{language} mapping is outside the approved async capability inventory",
                    )
                self.assertTrue(row.get("parityEvidenceIds"))
                for evidence_path in row["parityEvidenceIds"]:
                    self.assertTrue((ROOT / evidence_path).is_file(), evidence_path)

        unresolved = [
            row
            for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual([], unresolved, "KMIPKIT-0009 declarations remain unresolved")

    def test_async_trait_implementation_artifacts_are_exact_owner_exclusions(self) -> None:
        expected = expected_trait_exports()
        self.assertEqual(177, len(expected), "the reviewed asynchronous Rust trait inventory changed")
        exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("candidateSpecId") == OWNER_SPEC
            # The 0053 protocol namespace aliases are audited separately. Keep
            # this set scoped to the original 0009 declarations.
            and not row.get("exportName", "").startswith("<kmipkit::protocol::")
        ]
        actual = {(row.get("packageId"), row.get("exportName")) for row in exclusions}
        self.assertEqual(expected, actual, "each exact async Rust trait artifact needs one owner exclusion")

        for row in exclusions:
            with self.subTest(export=row.get("exportName")):
                self.assertTrue(row["exportName"].startswith("<"))
                self.assertIn("trait implementation artifact", row.get("reason", ""))
                self.assertTrue(row.get("evidence"))
                self.assertIn(row["exportName"], row["evidence"])
                self.assertTrue(row["evidence"].startswith("Exact export "))
                self.assertEqual(
                    {"kind": "feature-specification", "path": OWNER_PATH,
                     "status": "Approved", "evidence": OWNER_APPROVAL},
                    row.get("approvalSource"),
                )

    def test_async_request_models_do_not_map_to_network_execution(self) -> None:
        request_types = (
            "CancelRequest",
            "PollRequest",
            "ProcessRequest",
            "QueryAsyncRequestsRequest",
        )
        request_rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if any(
                row.get("exportName", "") == f"{namespace}::{request_type}"
                or row.get("exportName", "").startswith(
                    f"{namespace}::{request_type}::"
                )
                for namespace in ("kmipkit_protocol", "kmipkit")
                for request_type in request_types
            )
            and row.get("capabilityId") == "client_asynchronous_operations"
        ]
        self.assertTrue(request_rows, "typed asynchronous request rows are missing")

        for row in request_rows:
            for language in ("c", "java", "python"):
                name = row["languageMappings"][language]["name"]
                with self.subTest(export=row["exportName"], language=language):
                    self.assertNotIn(
                        "execute",
                        name.lower(),
                        "a pure request model must not map to a one-exchange network call",
                    )


if __name__ == "__main__":
    unittest.main()
