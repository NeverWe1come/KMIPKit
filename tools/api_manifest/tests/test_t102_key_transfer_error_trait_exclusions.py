"""Pin exact T102 KMIPKIT-0037 protocol error-trait exclusions."""

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
PROTOCOL = "rust-kmipkit-protocol"
ROOT_PACKAGE = "rust-kmipkit"
CLIENT = "rust-kmipkit-client"
OWNER_APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/023-key-material-transfer/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
OWNER_REQUIREMENTS = tuple(f"KMIPKIT-0037-FR-{number:03d}" for number in (1, 2, 4, 5, 8))
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
COMMON_DIRECT = {
    "alloc::borrow::ToOwned>::Owned", "alloc::borrow::ToOwned>::clone_into",
    "alloc::borrow::ToOwned>::to_owned", "alloc::string::ToString>::to_string",
    "core::any::Any>::type_id", "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from", "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq", "core::cmp::PartialEq>::eq",
    "core::cmp::PartialEq>::ne", "core::error::Error>::cause",
    "core::error::Error>::description", "core::error::Error>::provide",
    "core::error::Error>::source", "core::error::Error>::type_id",
    "core::fmt::Debug>::fmt", "core::fmt::Display>::fmt",
}
CONDITIONAL = {
    "core::convert::Into<U>>::into", "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from", "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}
OPERATIONS = {
    "DeriveKeyError": {
        "module": "derive_key", "element": "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY",
        "reference": "OASIS KMIP v2.1 Section 6.1.14, Tables 205-207",
        "captions": ("Table 205: Derive Key Request Payload", "Table 206: Derive Key Response Payload", "Table 207: Derive Key Errors"),
        "request": ("DeriveKeyRequest", "blocked"), "response": ("DeriveKeyResponse", "verified"), "token": "derive_key",
    },
    "ExportError": {
        "module": "export", "element": "KMIPKIT-ELEM-OP-C2S-EXPORT",
        "reference": "OASIS KMIP v2.1 Section 6.1.18, Tables 217-219",
        "captions": ("Table 217: Export Request Payload", "Table 218: Export Response Payload", "Table 219: Export Errors"),
        "request": ("ExportRequest", "verified"), "response": ("ExportResponse", "verified"), "token": "export",
    },
    "ImportError": {
        "module": "import", "element": "KMIPKIT-ELEM-OP-C2S-IMPORT",
        "reference": "OASIS KMIP v2.1 Section 6.1.25, Tables 238-240",
        "captions": ("Table 238: Import Request Payload", "Table 239: Import Response Payload", "Table 240: Import Errors"),
        "request": ("ImportRequest", "verified"), "response": ("ImportResponse", "verified"), "token": "import",
    },
    "JoinSplitKeyError": {
        "module": "join_split_key", "element": "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
        "reference": "OASIS KMIP v2.1 Section 6.1.27, Tables 244-246",
        "captions": ("Table 244: Join Split Key Request Payload", "Table 245: Join Split Key Response Payload", "Table 246: Join Split Key Errors"),
        "request": ("JoinSplitKeyRequest", "verified"), "response": ("JoinSplitKeyResponse", "verified"), "token": "join_split_key",
    },
}
ERROR_SUFFIXES = {
    "": "blocked", "::InvalidOperationResult": "blocked", "::InvalidOperationResult::0": "blocked",
    "::MalformedResponseItem": "blocked", "::MalformedSuccessPayload": "blocked",
    "::MissingResultStatus": "blocked", "::MissingSuccessPayload": "blocked", "::UnexpectedOperation": "blocked",
}
RELATED_STATES = {
    "DeriveKeyRequest": {"": "blocked", "::into_ttlv_payload": "blocked", "::new": "blocked"},
    "DeriveKeyResponse": {"": "verified", "::result": "blocked", "::try_from_response_item": "blocked", "::unique_identifier": "verified", "::with_ttlv": "verified"},
    "ExportRequest": {"": "verified", "::into_ttlv_payload": "blocked", "::new": "blocked", "::with_key_compression_type": "blocked", "::with_key_format_type": "blocked", "::with_key_wrap_type": "blocked", "::with_key_wrapping_specification": "blocked", "::with_unique_identifier": "blocked", "::unique_identifier": "blocked"},
    "ExportResponse": {"": "verified", "::attributes": "verified", "::object": "verified", "::object_type": "verified", "::result": "blocked", "::try_from_response_item": "blocked", "::unique_identifier": "verified", "::with_ttlv": "verified"},
    "ImportRequest": {"": "verified", "::attributes": "blocked", "::into_ttlv_payload": "blocked", "::key_wrap_type": "blocked", "::new": "verified", "::object": "blocked", "::object_type": "blocked", "::replace_existing": "blocked", "::unique_identifier": "blocked", "::with_key_wrap_type": "blocked", "::with_replace_existing": "blocked"},
    "ImportResponse": {"": "verified", "::result": "blocked", "::try_from_response_item": "blocked", "::unique_identifier": "verified", "::with_ttlv": "verified"},
    "JoinSplitKeyRequest": {"": "verified", "::attributes": "blocked", "::into_ttlv_payload": "blocked", "::new": "verified", "::object_type": "blocked", "::protection_storage_masks": "blocked", "::references": "blocked", "::secret_data_type": "blocked", "::with_attributes": "blocked", "::with_protection_storage_masks": "blocked", "::with_secret_data_type": "blocked"},
    "JoinSplitKeyResponse": {"": "verified", "::result": "blocked", "::try_from_response_item": "blocked", "::unique_identifier": "verified", "::with_ttlv": "verified"},
}


def trait_paths(name: str, suffixes: set[str]) -> set[str]:
    return {f"<kmipkit_protocol::{name} as {suffix}" for suffix in suffixes}


def direct_suffixes(name: str) -> set[str]:
    model = f"kmipkit_protocol::{OPERATIONS[name]['module']}::{name}"
    return COMMON_DIRECT | {
        f"core::borrow::Borrow<{model}>>::borrow",
        f"core::borrow::BorrowMut<{model}>>::borrow_mut",
        f"core::convert::From<{model}>>::from",
    }


def operation_rows() -> dict[tuple[str, str], str]:
    result: dict[tuple[str, str], str] = {}

    def add(package: str, *names: str, state: str = "verified") -> None:
        result.update({(package, name): state for name in names})

    add(ROOT_PACKAGE, "kmipkit::Client::derive_key", "kmipkit::Client::derive_key_with_options", "kmipkit::ClientBatchOutcome::derive_key_response", "kmipkit::ClientRequest::derive_key")
    add(CLIENT, "kmipkit_client::Client::derive_key", "kmipkit_client::Client::derive_key_with_options", "kmipkit_client::ClientBatchOutcome::derive_key_response", "kmipkit_client::ClientRequest::derive_key")
    add(CLIENT, "kmipkit_client::ClientResponseView<'a>::derive_key", state="blocked")
    for method, title in (("export", "Export"), ("import", "Import")):
        for package, prefix in ((ROOT_PACKAGE, "kmipkit"), (CLIENT, "kmipkit_client")):
            add(package, f"{prefix}::Client::{method}", f"{prefix}::Client::{method}_with_options", f"{prefix}::ClientBatchOutcome::{title}", f"{prefix}::ClientBatchOutcome::{title}::0", f"{prefix}::ClientBatchOutcome::{method}_response", f"{prefix}::ClientOperation::{title}", f"{prefix}::ClientRequest::{title}", f"{prefix}::ClientRequest::{title}::0", f"{prefix}::ClientRequest::{method}")
        add(CLIENT, f"kmipkit_client::ClientResponseView<'a>::{method}", state="blocked")
    add(ROOT_PACKAGE, "kmipkit::Client::join_split_key", "kmipkit::Client::join_split_key_with_options", "kmipkit::ClientBatchOutcome::join_split_key_response", "kmipkit::ClientRequest::join_split_key")
    add(CLIENT, "kmipkit_client::Client::join_split_key", "kmipkit_client::Client::join_split_key_with_options", "kmipkit_client::ClientBatchOutcome::join_split_key_response", "kmipkit_client::ClientRequest::join_split_key")
    add(CLIENT, "kmipkit_client::ClientResponseView<'a>::join_split_key", state="blocked")
    return result


class T102KeyTransferErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_protocol_error_traits_are_approved_exclusions(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        pairs: set[tuple[str, str]] = set()
        for name, operation in OPERATIONS.items():
            direct = trait_paths(name, direct_suffixes(name))
            conditional = trait_paths(name, CONDITIONAL)
            expected = direct | conditional
            prefix = f"<kmipkit_protocol::{name} as "
            for key, expected_set in (("declarations", direct), ("conditionalDeclarations", conditional), ("excluded", expected)):
                actual = {r["exportName"] for r in self.report[key] if r.get("packageId") == PROTOCOL and r.get("exportName", "").startswith(prefix)}
                self.assertEqual(actual, expected_set)
            self.assertEqual((len(direct), len(conditional)), (21, 5))
            declarations = {r["exportName"] for r in self.manifest["publicDeclarations"] if r.get("packageId") == PROTOCOL and r.get("exportName", "").startswith(prefix)}
            exclusions = {r["exportName"]: r for r in self.manifest["approvedExclusions"] if r.get("packageId") == PROTOCOL and r.get("exportName", "").startswith(prefix)}
            self.assertEqual(declarations, set())
            self.assertEqual(set(exclusions), expected)
            for export_name, row in exclusions.items():
                with self.subTest(name=name, export_name=export_name):
                    self.assertEqual(row["approvalSource"], OWNER_APPROVAL)
                    for req in (*OWNER_REQUIREMENTS, operation["element"], *T102_REQUIREMENTS):
                        self.assertIn(req, row["evidence"])
                    self.assertIn(operation["reference"], row["evidence"])
                    for caption in operation["captions"]:
                        self.assertIn(caption, row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("makes no conformance claim", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
            pairs.update((PROTOCOL, path) for path in expected)
        self.assertEqual(len(pairs), 104)

    def test_error_parent_function_rows_and_absent_root_aliases_are_pinned(self) -> None:
        for name in OPERATIONS:
            protocol_name = f"kmipkit_protocol::{name}"
            expected = {f"{protocol_name}{suffix}": state for suffix, state in ERROR_SUFFIXES.items()}
            actual = {r["exportName"]: r["declarationState"] for r in self.manifest["publicDeclarations"] if r.get("packageId") == PROTOCOL and (r.get("exportName") == protocol_name or r.get("exportName", "").startswith(protocol_name + "::")) and not r.get("exportName", "").startswith("<")}
            self.assertEqual(actual, expected)
            root_name = f"kmipkit::{name}"
            self.assertFalse(any(r.get("packageId") == ROOT_PACKAGE and (r.get("exportName") == root_name or r.get("exportName", "").startswith(root_name + "::") or r.get("exportName", "").startswith(f"<kmipkit::{name} as ")) for r in self.manifest["publicDeclarations"]))

    def test_request_response_and_operation_rows_keep_exact_states(self) -> None:
        for operation in OPERATIONS.values():
            for key in ("request", "response"):
                name, _state = operation[key]
                prefix = f"kmipkit_protocol::{name}"
                expected = {f"{prefix}{suffix}": state for suffix, state in RELATED_STATES[name].items()}
                actual = {r["exportName"]: r["declarationState"] for r in self.manifest["publicDeclarations"] if r.get("packageId") == PROTOCOL and (r.get("exportName") == prefix or r.get("exportName", "").startswith(prefix + "::"))}
                self.assertEqual(actual, expected)
                root_prefix = f"kmipkit::{name}"
                self.assertFalse(any(r.get("packageId") == ROOT_PACKAGE and (r.get("exportName") == root_prefix or r.get("exportName", "").startswith(root_prefix + "::") or r.get("exportName", "").startswith(f"<kmipkit::{name} as ")) for r in self.manifest["publicDeclarations"]))
        expected_operations = operation_rows()
        actual_operations = {(r.get("packageId"), r.get("exportName")): r["declarationState"] for r in self.manifest["publicDeclarations"] if (r.get("packageId"), r.get("exportName")) in expected_operations}
        self.assertEqual(actual_operations, expected_operations)

    def test_manifest_and_t102_candidate_totals_decrease_by_104(self) -> None:
        candidates = sum(r.get("candidateSpecId") == "KMIPKIT-0043" and r["declarationState"] == "blocked" for r in self.manifest["publicDeclarations"])
        self.assertEqual(candidates, T102_BLOCKED_CANDIDATES)
        self.assertEqual(len(self.manifest["publicDeclarations"]), T102_PUBLIC_DECLARATIONS)
        self.assertEqual(len(self.manifest["approvedExclusions"]), T102_APPROVED_EXCLUSIONS)


if __name__ == "__main__":
    unittest.main()
