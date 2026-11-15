"""Require root and protocol Ticket members to map to credential APIs."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import T102_BLOCKED_CANDIDATES

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/024-credential-sessions/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN"
MAPPING_EVIDENCE = "tools/api_manifest/tests/test_t102_ticket_model_members_mapping.py"
DECLARATIONS = (
    (
        "rust-observed-3aea4d793e81b69de4c1",
        "rust-kmipkit",
        "kmipkit::Ticket::into_ttlv",
    ),
    (
        "rust-observed-90e17cd1f37c1b13ee4e",
        "rust-kmipkit",
        "kmipkit::Ticket::new",
    ),
    (
        "rust-observed-ecb07fb359ca3c15dc52",
        "rust-kmipkit",
        "kmipkit::Ticket::try_from_ttlv",
    ),
    (
        "rust-observed-217734a91c391cce3c90",
        "rust-kmipkit",
        "kmipkit::TicketType::LOGIN",
    ),
    (
        "rust-observed-4381db95e713c012fc3b",
        "rust-kmipkit",
        "kmipkit::TicketType::from_raw",
    ),
    (
        "rust-observed-fa0a474eca458bb09446",
        "rust-kmipkit",
        "kmipkit::TicketType::known_name",
    ),
    (
        "rust-observed-1905a67e3c9d0194df4c",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Ticket::into_ttlv",
    ),
    (
        "rust-observed-c90435b97b41feb3b186",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Ticket::new",
    ),
    (
        "rust-observed-6ba9278cc880d5ad6d62",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::Ticket::try_from_ttlv",
    ),
    (
        "rust-observed-fe30c871f85e00feff91",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::TicketType::LOGIN",
    ),
    (
        "rust-observed-0b9e5262cf1e637d14d5",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::TicketType::from_raw",
    ),
    (
        "rust-observed-38e4871cc0f7451554da",
        "rust-kmipkit-protocol",
        "kmipkit_protocol::TicketType::known_name",
    ),
)


class T102TicketModelMemberMappingTests(unittest.TestCase):
    def read_source(self, *parts: str) -> str:
        return ROOT.joinpath(*parts).read_text(encoding="utf-8")

    def assert_source_contains(self, parts: tuple[str, ...], *fragments: str) -> None:
        source = self.read_source(*parts)
        for fragment in fragments:
            with self.subTest(path="/".join(parts), fragment=fragment):
                self.assertIn(fragment, source)

    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.anchors = {
            row["exportName"]: row
            for row in cls.manifest["publicDeclarations"]
            if row["exportName"]
            in {
                "kmipkit::Ticket",
                "kmipkit::TicketType",
                "kmipkit_protocol::Ticket",
                "kmipkit_protocol::TicketType",
            }
        }
        cls.operation = next(
            row
            for row in cls.manifest["operations"]
            if row["operationId"] == OPERATION_ID
        )

    def test_ticket_members_map_to_the_existing_delegated_login_api(self) -> None:
        for declaration_id, package_id, name in DECLARATIONS:
            with self.subTest(declaration=name):
                declaration = self.declarations[declaration_id]
                type_name = name.rsplit("::", 1)[0]
                anchor = self.anchors[type_name]
                expected_mappings = copy.deepcopy(anchor["languageMappings"])
                expected_mappings["rust"] = {"packageId": package_id, "name": name}

                self.assertEqual(name, declaration["exportName"])
                self.assertEqual("associated-item", declaration["kind"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual(OPERATION_ID, declaration["capabilityId"])
                self.assertEqual(anchor["requirementIds"], declaration["requirementIds"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
                self.assertIn(declaration_id, self.operation["declarations"]["rust"])
                self.assertIn(MAPPING_EVIDENCE, self.operation["parityEvidenceIds"])

    def test_mapping_has_normative_ttlv_and_secret_safe_source_evidence(self) -> None:
        self.assert_source_contains(
            ("crates", "kmipkit-protocol", "src", "ticket.rs"),
            "§7.39, Table 391",
            "§11.57, Table 488",
        )
        self.assert_source_contains(
            ("crates", "kmipkit-protocol", "tests", "unit", "ticket_model_tests.rs"),
            "ticket_constructor_owns_value_and_redacts_debug_and_display",
            "ticket_type_preserves_unassigned_and_vendor_enumeration_values",
            "ttlv_parse_preserves_unknown_children_and_original_order",
        )
        self.assert_source_contains(
            ("bindings", "c", "include", "kmipkit.h"),
            "kmipkit_client_batch_item_response_delegated_login",
        )
        self.assert_source_contains(
            (
                "bindings",
                "java",
                "src",
                "main",
                "java",
                "org",
                "kmipkit",
                "extensions",
                "ClientBatchItemResult.java",
            ),
            "ticketTypeRaw()",
            "ticketTtlv()",
        )
        self.assert_source_contains(
            ("bindings", "python", "src", "kmipkit", "client.py"),
            "ticket_type_raw",
            "ticket_ttlv",
        )

        self.assertEqual(
            T102_BLOCKED_CANDIDATES,
            sum(
                row.get("declarationState") == "blocked"
                and row.get("candidateSpecId") == "KMIPKIT-0043"
                for row in self.manifest["publicDeclarations"]
            ),
        )


if __name__ == "__main__":
    unittest.main()
