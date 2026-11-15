"""Traceability guard for the Re-certify omitted-request identifier ruling."""

from __future__ import annotations

import json
import unittest

from tools.normative_catalog.tests.test_validate import ROOT


REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.45-002-003"
DISCREPANCY_ID = "KMIPKIT-DISC-054"
DECISION_ID = "KMIPKIT-DEC-019"
BASELINE_CLIENT_ID = "KMIPKIT-PROFILE-BASELINE-CLIENT"


class RecertifyIdentifierDecisionTests(unittest.TestCase):
    def test_omitted_request_identifier_uses_a_separate_adjudication_and_evidence(self) -> None:
        catalog = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )
        requirement = next(
            (item for item in catalog["requirements"] if item["requirement_id"] == REQUIREMENT_ID),
            None,
        )
        baseline_profile = next(
            item
            for item in catalog["profiles"]
            if item["profile_id"] == BASELINE_CLIENT_ID
        )
        discrepancy = next(
            (item for item in catalog["discrepancies"] if item["discrepancy_id"] == DISCREPANCY_ID),
            None,
        )
        decision = next(
            (item for item in catalog["decisions"] if item["decision_id"] == DECISION_ID), None
        )
        certify_decision = next(
            item for item in catalog["decisions"] if item["decision_id"] == "KMIPKIT-DEC-013"
        )

        self.assertIsNotNone(requirement, "the normative requirement remains in the catalog")
        self.assertIsNotNone(discrepancy, "the Re-certify source tension has its own discrepancy")
        self.assertIsNotNone(decision, "the Re-certify interpretation has its own decision")

        self.assertIn(REQUIREMENT_ID, baseline_profile["requirement_ids"])
        self.assertEqual(requirement["decision_id"], DECISION_ID)
        self.assertEqual(requirement["status"], "scoped_verified")
        self.assertTrue(requirement["implementation_refs"])
        self.assertTrue(requirement["verification_refs"])
        self.assertIn(
            "profile_rejects_recertify_without_identifier_when_request_value_is_omitted",
            " ".join(requirement["verification_refs"]),
        )
        self.assertIn(
            "profile_rejects_recertify_id_placeholder_when_request_value_is_omitted",
            " ".join(requirement["verification_refs"]),
        )

        self.assertEqual(discrepancy["source_authority"], "primary_normative")
        self.assertEqual(discrepancy["normative_status"], "normative_conflict")
        self.assertEqual(discrepancy["affected_requirement_ids"], [REQUIREMENT_ID])
        self.assertEqual(discrepancy["decision_id"], DECISION_ID)
        self.assertIn("Table 300", " ".join(discrepancy["alternatives"]))
        self.assertEqual(decision["discrepancy_ids"], [DISCREPANCY_ID])
        self.assertEqual(decision["requirement_ids"], [REQUIREMENT_ID])
        self.assertEqual(decision["status"], "accepted")

        self.assertEqual(
            certify_decision["requirement_ids"],
            ["KMIPKIT-REQ-SPEC-6.1.6-002-002"],
            "DEC-013 remains Certify-only",
        )


if __name__ == "__main__":
    unittest.main()
