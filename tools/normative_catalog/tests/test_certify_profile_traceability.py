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

    def test_omitted_certificate_request_requires_explicit_unique_identifier(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.6-002-002"
        clause_id = "KMIPKIT-CLAUSE-SPEC-6.1.6-002"
        positive_test = (
            f"{PROFILE_TEST_PATH}::"
            "profile_accepts_omitted_certify_request_value_with_explicit_unique_identifier"
        )
        negative_test = (
            f"{PROFILE_TEST_PATH}::"
            "profile_rejects_omitted_certify_request_value_without_unique_identifier"
        )
        boundary_test = (
            f"{PROFILE_TEST_PATH}::"
            "profile_accepts_certify_request_value_without_unique_identifier"
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
        self.assertEqual(requirement["source_keyword"], "SHALL")
        self.assertEqual(requirement["normative_strength"], "mandatory")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["condition"], "When Certificate Request is omitted.")
        self.assertEqual(requirement["decision_id"], "KMIPKIT-DEC-013")
        decision = next(
            record
            for record in catalog["decisions"]
            if record["decision_id"] == requirement["decision_id"]
        )
        self.assertIn(requirement_id, decision["requirement_ids"])
        self.assertIn(
            "Do not apply the conflicting Table 179 sentence as an ID Placeholder substitution",
            decision["interpretation"],
        )
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        for reference in (positive_test, negative_test, boundary_test):
            self.assertIn(reference, requirement["verification_refs"])
            self.assertIn(reference, certify_element["verification_refs"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", certify_element["implementation_refs"])

        for reference in (
            "crates/kmipkit-protocol/src/profile.rs",
            *requirement["verification_refs"],
        ):
            path = ROOT / reference.partition("::")[0]
            self.assertTrue(path.is_file(), reference)

    def test_omitted_certify_request_fields_require_certificate_type_attribute(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.6-002-003"
        clause_id = "KMIPKIT-CLAUSE-SPEC-6.1.6-002"
        profile_tests = (
            f"{PROFILE_TEST_PATH}::profile_accepts_omitted_certify_certificate_type_in_attributes",
            f"{PROFILE_TEST_PATH}::profile_rejects_omitted_certify_certificate_type_when_attribute_missing",
            f"{PROFILE_TEST_PATH}::profile_rejects_malformed_certificate_type_attribute_when_request_fields_are_omitted",
            f"{PROFILE_TEST_PATH}::profile_accepts_certificate_request_fields_without_certificate_type_attribute",
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
        certificate_type_element = elements["KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-TYPE"]

        self.assertEqual(requirement["source_clause_ids"], [clause_id])
        self.assertEqual(requirement["source_refs"][0]["section"], "6.1.6")
        self.assertEqual(requirement["source_keyword"], "SHALL")
        self.assertEqual(requirement["normative_strength"], "mandatory")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(
            requirement["condition"],
            "When both request fields are omitted.",
        )
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertIsNone(requirement["decision_id"])
        self.assertIn("KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-TYPE", requirement["element_ids"])
        self.assertIn(requirement_id, certificate_type_element["requirement_ids"])
        self.assertEqual(certificate_type_element["source_encoding"], "Enumeration")
        self.assertIn(
            {"source_id": "KMIPKIT-SRC-spec", "section": "4.7"},
            certificate_type_element["source_refs"],
        )
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", certify_element["implementation_refs"])
        self.assertIn(
            "crates/kmipkit-protocol/src/profile.rs",
            certificate_type_element["implementation_refs"],
        )
        for profile_test in profile_tests:
            self.assertIn(profile_test, requirement["verification_refs"])
            self.assertIn(profile_test, certify_element["verification_refs"])
            self.assertIn(profile_test, certificate_type_element["verification_refs"])

        for reference in (
            "crates/kmipkit-protocol/src/profile.rs",
            *requirement["verification_refs"],
        ):
            path = ROOT / reference.partition("::")[0]
            self.assertTrue(path.is_file(), reference)

    def test_certify_request_type_is_required_when_request_value_is_present(self) -> None:
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.6-007"
        clause_id = "KMIPKIT-CLAUSE-SPEC-6.1.6-007"
        profile_tests = (
            f"{PROFILE_TEST_PATH}::profile_maps_certify_request_type_when_request_value_is_present_as_required_behavior",
            f"{PROFILE_TEST_PATH}::profile_accepts_certificate_request_type_with_request_value",
            f"{PROFILE_TEST_PATH}::profile_rejects_certificate_request_value_without_request_type",
            f"{PROFILE_TEST_PATH}::profile_rejects_malformed_certificate_request_type_when_request_value_is_present",
            f"{PROFILE_TEST_PATH}::profile_accepts_valid_certify_omission_without_request_type",
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
        request_type_element = elements[
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE"
        ]

        self.assertEqual(requirement["source_clause_ids"], [clause_id])
        self.assertEqual(requirement["source_refs"][0]["section"], "6.1.6")
        self.assertEqual(requirement["source_keyword"], "REQUIRED")
        self.assertEqual(requirement["normative_strength"], "mandatory")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(
            requirement["condition"],
            "When Certificate Request is present.",
        )
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertIn("KMIPKIT-ELEM-OP-C2S-CERTIFY", requirement["element_ids"])
        self.assertIn(requirement_id, certify_element["requirement_ids"])
        self.assertIn(
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE",
            requirement["element_ids"],
        )
        self.assertIn(requirement_id, request_type_element["requirement_ids"])
        self.assertEqual(request_type_element["source_encoding"], "Enumeration")
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", requirement["implementation_refs"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", certify_element["implementation_refs"])
        self.assertIn("crates/kmipkit-protocol/src/profile.rs", request_type_element["implementation_refs"])
        for profile_test in profile_tests:
            self.assertIn(profile_test, requirement["verification_refs"])
            self.assertIn(profile_test, certify_element["verification_refs"])
            self.assertIn(profile_test, request_type_element["verification_refs"])

        for reference in (
            "crates/kmipkit-protocol/src/profile.rs",
            *requirement["verification_refs"],
        ):
            path = ROOT / reference.partition("::")[0]
            self.assertTrue(path.is_file(), reference)


if __name__ == "__main__":
    unittest.main()
