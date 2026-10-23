from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.6-001"
SOURCE_CLAUSE_ID = "KMIPKIT-CLAUSE-SPEC-6.1.6-001"
PROFILE_TEST_PATH = "crates/kmipkit-protocol/tests/support/profile_unit.rs"


class CertifyProfileTraceabilityTests(unittest.TestCase):
    def test_single_certificate_requirement_links_typed_profile_rule_and_tests(self) -> None:
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        requirement = requirements[REQUIREMENT_ID]

        self.assertEqual(requirement["source_clause_ids"], [SOURCE_CLAUSE_ID])
        self.assertEqual(requirement["source_keyword"], "SHALL")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertIsNone(requirement["condition"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn(
            f"{PROFILE_TEST_PATH}::profile_accepts_at_most_one_certify_certificate_request_value_per_batch_item",
            requirement["verification_refs"],
        )
        self.assertIn(
            f"{PROFILE_TEST_PATH}::profile_rejects_duplicate_certify_certificate_request_values_per_batch_item",
            requirement["verification_refs"],
        )

        for reference in (
            "crates/kmipkit-protocol/src/profile.rs",
            *requirement["verification_refs"],
        ):
            path = ROOT / reference.partition("::")[0]
            self.assertTrue(path.is_file(), reference)


if __name__ == "__main__":
    unittest.main()
