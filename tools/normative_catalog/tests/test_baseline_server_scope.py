"""Protect the client profile boundary for server-produced KMIP requirements."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from typing import Any

from tools.normative_catalog.generate_profiles import compute_profile_closure


ROOT = Path(__file__).resolve().parents[3]
BASELINE_PROFILE_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"

SERVER_ONLY_REQUIREMENT_DIRECTIONS = {
    "KMIPKIT-REQ-SPEC-4.57-003-001": "not_applicable",
    "KMIPKIT-REQ-SPEC-6.1.25-001": "not_applicable",
    "KMIPKIT-REQ-SPEC-7.12-005-002": "server_to_client",
    "KMIPKIT-REQ-SPEC-7.18-002": "server_to_client",
    "KMIPKIT-REQ-SPEC-7.36-001": "server_to_client",
    "KMIPKIT-REQ-SPEC-7.36-002-001": "server_to_client",
    "KMIPKIT-REQ-SPEC-7.36-002-002": "server_to_client",
    "KMIPKIT-REQ-SPEC-7.36-003": "server_to_client",
    "KMIPKIT-REQ-SPEC-8-003-002": "server_to_client",
    "KMIPKIT-REQ-SPEC-9.21-001-002": "server_to_client",
}

PKCS11_CONDITIONAL_REQUIREMENT_IDS = {
    f"KMIPKIT-REQ-SPEC-7.{section}-001" for section in range(26, 31)
}


def load_catalog() -> dict[str, Any]:
    """Load the checked-in normative catalog used by profile generation."""
    path = ROOT / "specification/catalog/kmip-2.1.json"
    return json.loads(path.read_text(encoding="utf-8"))


class BaselineServerScopeTests(unittest.TestCase):
    """Keep server production obligations out of the Baseline client closure."""

    def test_server_produced_requirements_are_not_client_scope_blockers(self) -> None:
        catalog = load_catalog()
        requirements = {
            item["requirement_id"]: item for item in catalog["requirements"]
        }
        profile = next(
            item
            for item in catalog["profiles"]
            if item["profile_id"] == BASELINE_PROFILE_ID
        )
        closure = compute_profile_closure(
            catalog,
            profile["direct_source_clause_ids"],
            profile_id=BASELINE_PROFILE_ID,
        )

        for requirement_id, expected_direction in SERVER_ONLY_REQUIREMENT_DIRECTIONS.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                self.assertEqual(requirement["role"], "server")
                self.assertEqual(requirement["direction"], expected_direction)
                self.assertEqual(requirement["scope_state"], "server_only")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertIn("project interpretation", requirement["review_note"].lower())
                if requirement_id == "KMIPKIT-REQ-SPEC-4.57-003-001":
                    self.assertIn("active state transition", requirement["review_note"].lower())
                    self.assertNotIn("§4.57 says", requirement["review_note"])
                    self.assertIn(
                        "discrepancy belongs to later state rules",
                        requirement["review_note"].lower(),
                    )
                self.assertNotIn(requirement_id, closure.requirement_ids)
                self.assertNotIn(
                    requirement_id,
                    {item.requirement_id for item in closure.unresolved_scope_obligations},
                )

    def test_pkcs11_specific_fields_do_not_block_baseline_client(self) -> None:
        catalog = load_catalog()
        requirements = {
            item["requirement_id"]: item for item in catalog["requirements"]
        }
        profile = next(
            item
            for item in catalog["profiles"]
            if item["profile_id"] == BASELINE_PROFILE_ID
        )
        closure = compute_profile_closure(
            catalog,
            profile["direct_source_clause_ids"],
            profile_id=BASELINE_PROFILE_ID,
        )
        unresolved_ids = {
            item.requirement_id for item in closure.unresolved_scope_obligations
        }

        for requirement_id in sorted(PKCS11_CONDITIONAL_REQUIREMENT_IDS):
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                self.assertEqual(requirement["role"], "unspecified")
                self.assertEqual(requirement["direction"], "unclear")
                self.assertEqual(requirement["scope_state"], "profile_conditional")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertIn("PKCS#11", requirement["condition"])
                self.assertNotIn(requirement_id, closure.requirement_ids)
                self.assertNotIn(requirement_id, unresolved_ids)

    def test_vendor_extension_structure_is_client_1_0_profile_scope(self) -> None:
        catalog = load_catalog()
        requirement_ids = {
            "KMIPKIT-REQ-SPEC-9.13-001-001",
            "KMIPKIT-REQ-SPEC-9.13-001-002",
            "KMIPKIT-REQ-SPEC-9.13-001-003",
        }
        requirements = {
            item["requirement_id"]: item
            for item in catalog["requirements"]
            if item["requirement_id"] in requirement_ids
        }
        self.assertEqual(set(requirements), requirement_ids)

        profile = next(
            item
            for item in catalog["profiles"]
            if item["profile_id"] == BASELINE_PROFILE_ID
        )
        closure = compute_profile_closure(
            catalog,
            profile["direct_source_clause_ids"],
            profile_id=BASELINE_PROFILE_ID,
        )
        unresolved_ids = {
            item.requirement_id for item in closure.unresolved_scope_obligations
        }

        for requirement_id, requirement in requirements.items():
            with self.subTest(requirement_id=requirement_id):
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["profile_ids"], [])
                self.assertIn("project interpretation", requirement["review_note"].lower())
                self.assertIn("source does not identify the sender", requirement["review_note"].lower())
                self.assertIn(requirement_id, closure.requirement_ids)
                self.assertNotIn(requirement_id, unresolved_ids)

        fixture_path = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
        fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
        review = next(
            item
            for item in fixture["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.13-001"
        )
        self.assertEqual(
            review["expected_client_requirement_ids"],
            [
                "KMIPKIT-REQ-SPEC-9.13-001-001",
                "KMIPKIT-REQ-SPEC-9.13-001-002",
                "KMIPKIT-REQ-SPEC-9.13-001-003",
                "KMIPKIT-REQ-SPEC-9.13-001-004",
                "KMIPKIT-REQ-SPEC-9.13-001-005",
            ],
        )
        source_obligations = {
            item["source_order"]: item for item in review["source_obligations"]
        }
        for source_order in (1, 2, 3, 4, 7):
            with self.subTest(source_order=source_order):
                obligation = source_obligations[source_order]
                self.assertEqual(obligation["actor"], "unspecified")
                self.assertEqual(obligation["direction"], "unspecified")
                self.assertEqual(obligation["scope_disposition"], "client_1_0")


if __name__ == "__main__":
    unittest.main()
