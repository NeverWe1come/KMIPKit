"""Pin the exact T102 Ticket and Login protocol trait exclusions."""

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
PACKAGE = "rust-kmipkit-protocol"
OWNER = "specs/024-credential-sessions/spec.md"
APPROVAL = {
    "kind": "feature-specification",
    "path": OWNER,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation",
}
CONDITIONAL = {
    "core::convert::Into<U>>::into",
    "core::convert::TryFrom<U>>::Error",
    "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error",
    "core::convert::TryInto<U>>::try_into",
}

# These suffixes form the exact (packageId, exportName) inventory. Ticket's
# Debug and Display are custom secret-safe implementations, retained unchanged.
DIRECT = {
    "TicketType": """
alloc::borrow::ToOwned>::Owned
alloc::borrow::ToOwned>::clone_into
alloc::borrow::ToOwned>::to_owned
core::any::Any>::type_id
core::borrow::Borrow<kmipkit_protocol::ticket::TicketType>>::borrow
core::borrow::BorrowMut<kmipkit_protocol::ticket::TicketType>>::borrow_mut
core::clone::Clone>::clone
core::clone::Clone>::clone_from
core::clone::CloneToUninit>::clone_to_uninit
core::cmp::Eq>::assert_receiver_is_total_eq
core::cmp::PartialEq>::eq
core::cmp::PartialEq>::ne
core::convert::From<kmipkit_protocol::ticket::TicketType>>::from
core::fmt::Debug>::fmt
core::hash::Hash>::hash
core::hash::Hash>::hash_slice
""",
    "Ticket": """
alloc::string::ToString>::to_string
core::any::Any>::type_id
core::borrow::Borrow<kmipkit_protocol::ticket::Ticket>>::borrow
core::borrow::BorrowMut<kmipkit_protocol::ticket::Ticket>>::borrow_mut
core::convert::From<kmipkit_protocol::ticket::Ticket>>::from
core::fmt::Debug>::fmt
core::fmt::Display>::fmt
""",
    "LoginRequest": """
core::any::Any>::type_id
core::borrow::Borrow<kmipkit_protocol::login::LoginRequest>>::borrow
core::borrow::BorrowMut<kmipkit_protocol::login::LoginRequest>>::borrow_mut
core::convert::From<kmipkit_protocol::login::LoginRequest>>::from
core::default::Default>::default
core::fmt::Debug>::fmt
""",
}

OWNER_REQUIREMENTS = (
    "KMIPKIT-0038-FR-001",
    "KMIPKIT-0038-FR-002",
    "KMIPKIT-0038-FR-004",
    "KMIPKIT-0038-FR-005",
    "KMIPKIT-0038-FR-008",
)
NORMATIVE = {
    "TicketType": (
        "OASIS Key Management Interoperability Protocol Specification v2.1 "
        "Section 11.57 Table 488",
        "Table 488 assigns Login = 00000001 and Extensions = 8XXXXXXX",
    ),
    "Ticket": (
        "OASIS Key Management Interoperability Protocol Specification v2.1 "
        "Section 7.39 Table 391 and Section 11.57 Table 488",
        "Table 391 defines the Ticket Structure with required Ticket Type and Ticket Value",
    ),
    "LoginRequest": (
        "OASIS Key Management Interoperability Protocol Specification v2.1 "
        "Section 6.1.30 Tables 253-255",
        "Table 253 is the Login request payload",
    ),
}
PROTOCOL_STATES = {
    "TicketType": {
        "": "verified", "::LOGIN": "verified", "::from_raw": "verified",
        "::known_name": "verified", "::raw": "verified",
    },
    "Ticket": {
        "": "verified", "::into_ttlv": "verified", "::new": "verified",
        "::ticket_type": "verified", "::try_from_ttlv": "verified",
        "::with_ticket_value": "verified", "::with_ttlv": "verified",
    },
    "LoginRequest": {
        "": "verified", "::into_ttlv_payload": "verified", "::new": "verified",
        "::with_lease_time_date_time": "verified", "::with_lease_time_interval": "verified",
        "::with_request_count": "verified", "::with_usage_limits": "verified",
    },
}
ROOT_STATES = {
    "TicketType": {
        "": "verified", "::LOGIN": "verified", "::from_raw": "verified",
        "::known_name": "verified", "::raw": "verified",
    },
    "Ticket": {
        "": "verified", "::into_ttlv": "verified", "::new": "verified",
        "::ticket_type": "verified", "::try_from_ttlv": "verified",
        "::with_ticket_value": "verified", "::with_ttlv": "verified",
    },
    "LoginRequest": {
        "": "verified", "::into_ttlv_payload": "verified", "::new": "verified",
        "::with_lease_time_date_time": "verified", "::with_lease_time_interval": "verified",
        "::with_request_count": "verified", "::with_usage_limits": "verified",
    },
}


def suffixes(lines: str) -> set[str]:
    return {line for line in lines.splitlines() if line}


def paths(public_type: str, trait_suffixes: set[str]) -> set[str]:
    return {f"<{public_type} as {suffix}" for suffix in trait_suffixes}


class T102TicketLoginRequestTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_compiler_paths_are_excluded_under_approved_owner(self) -> None:
        self.assertTrue(self.report["complete"])
        self.assertEqual(self.report["toolchain"], "1.94.0")
        self.assertTrue(self.report["matchesManifest"])
        self.assertEqual(self.report["issues"], [])
        all_pairs: set[tuple[str, str]] = set()

        for type_name, direct_text in DIRECT.items():
            public = f"kmipkit_protocol::{type_name}"
            direct = paths(public, suffixes(direct_text))
            conditional = paths(public, CONDITIONAL)
            expected = direct | conditional
            pairs = {(PACKAGE, path) for path in expected}
            all_pairs.update(pairs)
            prefix = f"<{public} as "

            observed_direct = {
                x["exportName"] for x in self.report["declarations"]
                if x.get("packageId") == PACKAGE and x.get("exportName", "").startswith(prefix)
            }
            observed_conditional = {
                x["exportName"] for x in self.report["conditionalDeclarations"]
                if x.get("packageId") == PACKAGE and x.get("exportName", "").startswith(prefix)
            }
            observed_excluded = {
                (x["packageId"], x["exportName"]) for x in self.report["excluded"]
                if x.get("packageId") == PACKAGE and x.get("exportName", "").startswith(prefix)
            }
            with self.subTest(type_name=type_name, source="Rust audit"):
                self.assertEqual(observed_direct, direct)
                self.assertEqual(observed_conditional, conditional)
                self.assertEqual(observed_excluded, pairs)

            declarations = {
                (x["packageId"], x["exportName"]) for x in self.manifest["publicDeclarations"]
                if x.get("packageId") == PACKAGE and x.get("exportName", "").startswith(prefix)
            }
            exclusions = {
                (x["packageId"], x["exportName"]): x for x in self.manifest["approvedExclusions"]
                if x.get("packageId") == PACKAGE and x.get("exportName", "").startswith(prefix)
                and x.get("candidateSpecId") != "KMIPKIT-0053"
            }
            with self.subTest(type_name=type_name, source="manifest"):
                self.assertEqual(declarations, set())
                self.assertEqual(set(exclusions), pairs)
                self.assertEqual(len(exclusions), len(expected))

            reference, table_evidence = NORMATIVE[type_name]
            for row in exclusions.values():
                with self.subTest(type_name=type_name, export_name=row["exportName"]):
                    self.assertEqual(row["approvalSource"], APPROVAL)
                    for req in (*OWNER_REQUIREMENTS, "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"):
                        self.assertIn(req, row["evidence"])
                    self.assertIn(reference, row["evidence"])
                    self.assertIn(table_evidence, row["evidence"])
                    self.assertIn("inventory-only", row["evidence"])
                    self.assertIn("no OASIS conformance", row["evidence"])
                    self.assertIn("exact Rust standard-trait implementation", row["reason"])
                    self.assertIn("not a separate KMIP client capability", row["reason"])
                    if type_name == "Ticket":
                        self.assertIn("custom Debug and Display redaction remain unchanged", row["evidence"])

        self.assertEqual(len(all_pairs), 44)
        all_target_exclusions = {
            (x["packageId"], x["exportName"]) for x in self.manifest["approvedExclusions"]
            if x.get("packageId") == PACKAGE
            and x.get("candidateSpecId") != "KMIPKIT-0053"
            and any(x.get("exportName", "").startswith(f"<kmipkit_protocol::{name} as ") for name in DIRECT)
        }
        self.assertEqual(all_target_exclusions, all_pairs)

    def test_protocol_functional_states_and_root_trait_exclusions_are_preserved(self) -> None:
        for name in DIRECT:
            protocol = f"kmipkit_protocol::{name}"
            expected_protocol = {f"{protocol}{suffix}": state for suffix, state in PROTOCOL_STATES[name].items()}
            protocol_states = {
                x["exportName"]: x["declarationState"] for x in self.manifest["publicDeclarations"]
                if x.get("packageId") == PACKAGE
                and (x.get("exportName") == protocol or x.get("exportName", "").startswith(protocol + "::"))
                and not x["exportName"].startswith("<")
            }
            root = f"kmipkit::{name}"
            expected_root = {f"{root}{suffix}": state for suffix, state in ROOT_STATES[name].items()}
            root_states = {
                x["exportName"]: x["declarationState"] for x in self.manifest["publicDeclarations"]
                if x.get("packageId") == "rust-kmipkit"
                and (x.get("exportName") == root
                     or x.get("exportName", "").startswith(root + "::"))
            }
            root_trait_paths = paths(root, suffixes(DIRECT[name]) | CONDITIONAL)
            root_trait_exclusions = {
                x["exportName"] for x in self.manifest["approvedExclusions"]
                if x.get("packageId") == "rust-kmipkit"
                and x.get("exportName", "").startswith(f"<{root} as ")
                and x.get("candidateSpecId") != "KMIPKIT-0053"
            }
            with self.subTest(type_name=name):
                self.assertEqual(protocol_states, expected_protocol)
                self.assertEqual(root_states, expected_root)
                self.assertEqual(root_trait_exclusions, root_trait_paths)

    def test_candidate_count_moves_by_only_the_exact_forty_four_paths(self) -> None:
        self.assertEqual(
            sum(x.get("candidateSpecId") == "KMIPKIT-0043" and x["declarationState"] == "blocked"
                for x in self.manifest["publicDeclarations"]),
            T102_BLOCKED_CANDIDATES,
        )


if __name__ == "__main__":
    unittest.main()
