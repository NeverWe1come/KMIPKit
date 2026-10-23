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
        elements = {
            record["element_id"]: record for record in catalog["elements"]
        }
        requirement = requirements[REQUIREMENT_ID]
        certify_element = elements["KMIPKIT-ELEM-OP-C2S-CERTIFY"]

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
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", certify_element["implementation_refs"])
        self.assertIn(
            f"{PROFILE_TEST_PATH}::profile_accepts_at_most_one_certify_certificate_request_value_per_batch_item",
            certify_element["verification_refs"],
        )
        self.assertIn(
            f"{PROFILE_TEST_PATH}::profile_rejects_duplicate_certify_certificate_request_values_per_batch_item",
            certify_element["verification_refs"],
        )

        for reference in (
            "crates/kmipkit-protocol/src/profile.rs",
            *requirement["verification_refs"],
        ):
            path = ROOT / reference.partition("::")[0]
            self.assertTrue(path.is_file(), reference)

    def test_certificate_request_omission_may_has_typed_profile_traceability(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.6-002-001"
        clause_id = "KMIPKIT-CLAUSE-SPEC-6.1.6-002"
        profile_test = (
            f"{PROFILE_TEST_PATH}::"
            "profile_maps_omitted_certify_certificate_request_object_as_optional_capability"
        )
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {
            record["requirement_id"]: record for record in catalog["requirements"]
        }
        elements = {
            record["element_id"]: record for record in catalog["elements"]
        }
        requirement = requirements[requirement_id]
        certify_element = elements["KMIPKIT-ELEM-OP-C2S-CERTIFY"]

        self.assertEqual(requirement["source_clause_ids"], [clause_id])
        self.assertEqual(requirement["source_refs"][0]["section"], "6.1.6")
        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertEqual(requirement["normative_strength"], "permission_or_optional")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertIsNone(requirement["condition"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn(profile_test, requirement["verification_refs"])
        self.assertIn(
            "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs::"
            "request_without_request_value_requires_public_key_identifier_and_certificate_type_attribute",
            requirement["verification_refs"],
        )
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", certify_element["implementation_refs"])
        self.assertIn(profile_test, certify_element["verification_refs"])


if __name__ == "__main__":
    unittest.main()
