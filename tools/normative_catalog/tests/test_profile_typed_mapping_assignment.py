"""Regression checks for Baseline Client typed-rule catalog assignments."""

from __future__ import annotations

import json
import unittest

from tools.normative_catalog.tests.test_validate import ROOT


REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.20-001-001"
PROFILE_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"
TEST_ID = (
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_repeated_identical_get_attributes_references"
)


class ProfileTypedMappingAssignmentTests(unittest.TestCase):
    def test_get_attributes_duplicate_reference_prohibition_is_assigned(self) -> None:
        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        requirement = next(
            item for item in catalog["requirements"] if item["requirement_id"] == REQUIREMENT_ID
        )
        baseline = next(
            item for item in catalog["profiles"] if item["profile_id"] == PROFILE_ID
        )

        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-6.1.20-001"])
        self.assertEqual(requirement["source_keyword"], "SHALL NOT")
        self.assertEqual(requirement["status"], "scoped_verified")
        self.assertIn(REQUIREMENT_ID, baseline["requirement_ids"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn(TEST_ID, requirement["verification_refs"])
        self.assertTrue(requirement["negative_verification_required"])


if __name__ == "__main__":
    unittest.main()
