"""Traceability contract for the direct Key Block pairing slice."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
REQUIREMENT_IDS = (
    "KMIPKIT-REQ-SPEC-3.1-004-002",
    "KMIPKIT-REQ-SPEC-3.1-005-002",
)
ELEMENT_IDS = {
    "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-ALGORITHM",
    "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-LENGTH",
}
POSITIVE_TEST = (
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_accepts_absent_or_present_direct_key_block_pairs"
)
NEGATIVE_TEST = (
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_unpaired_direct_key_block_fields_in_register_and_import"
)
SUPPLEMENTAL_TESTS = {
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_does_not_merge_register_or_key_value_attributes_into_table12_pairing",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_non_structure_table12_key_block_objects",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_non_structure_present_table12_key_blocks",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_duplicate_present_table12_key_blocks",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_rejects_duplicate_direct_table12_pair_fields",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_allows_table12_register_key_value_location_without_key_block",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_exempts_secret_data_and_ignores_valid_opaque_objects_without_key_blocks",
    "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
    "profile_checks_each_mixed_register_import_batch_item",
}


class Table12ProfileTraceabilityTests(unittest.TestCase):
    def test_pairing_requirements_have_exact_request_scope_and_evidence(self) -> None:
        catalog = json.loads(
            (ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(
                encoding="utf-8"
            )
        )
        requirements = {
            row["requirement_id"]: row
            for row in catalog["requirements"]
            if row["requirement_id"] in REQUIREMENT_IDS
        }
        elements = {
            row["element_id"]: row
            for row in catalog["elements"]
            if row["element_id"] in ELEMENT_IDS
        }
        self.assertEqual(set(requirements), set(REQUIREMENT_IDS))
        self.assertEqual(set(elements), ELEMENT_IDS)

        expected_source_refs = {
            ("KMIPKIT-SRC-spec", "3.1"),
            ("KMIPKIT-SRC-spec", "6.1.25"),
            ("KMIPKIT-SRC-spec", "6.1.43"),
        }
        expected_verification_refs = {
            POSITIVE_TEST,
            NEGATIVE_TEST,
            *SUPPLEMENTAL_TESTS,
        }
        shared_implementation_ref = (
            "crates/kmipkit-protocol/src/profile.rs::"
            "client_key_block_direct_field_requires_companion"
        )
        expected_implementation_refs = {
            REQUIREMENT_IDS[0]: {
                shared_implementation_ref,
                "crates/kmipkit-protocol/src/profile.rs::"
                "client_key_block_algorithm_requires_length",
            },
            REQUIREMENT_IDS[1]: {
                shared_implementation_ref,
                "crates/kmipkit-protocol/src/profile.rs::"
                "client_key_block_length_requires_algorithm",
            },
        }

        for requirement_id in REQUIREMENT_IDS:
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                self.assertEqual(
                    {(row["source_id"], row["section"]) for row in requirement["source_refs"]},
                    expected_source_refs,
                )
                self.assertEqual(requirement["source_clause_ids"], [
                    "KMIPKIT-CLAUSE-SPEC-3.1-004"
                    if requirement_id.endswith("004-002")
                    else "KMIPKIT-CLAUSE-SPEC-3.1-005"
                ])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["source_keyword"], "SHALL")
                self.assertEqual(requirement["normative_strength"], "mandatory")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["status"], "verified")
                self.assertEqual(set(requirement["element_ids"]), ELEMENT_IDS)
                self.assertTrue(
                    expected_implementation_refs[requirement_id].issubset(
                        requirement["implementation_refs"]
                    )
                )
                self.assertTrue(
                    expected_verification_refs.issubset(
                        requirement["verification_refs"]
                    )
                )
                self.assertTrue(requirement["negative_verification_required"])
                self.assertEqual(requirement["test_case_ids"], [])
                review_note = requirement["review_note"].casefold()
                for fragment in (
                    "table 12",
                    "table 238",
                    "table 291",
                    "direct key block",
                    "key value attributes",
                    "outer attributes",
                    "secret data",
                    "opaque",
                    "response",
                    "server acceptance",
                    "official oasis test case",
                    "profile-conformance",
                ):
                    self.assertIn(fragment, review_note)

        for element_id, element in elements.items():
            with self.subTest(element_id=element_id):
                self.assertEqual(set(element["requirement_ids"]), set(REQUIREMENT_IDS))
                self.assertTrue(
                    {
                        ref
                        for refs in expected_implementation_refs.values()
                        for ref in refs
                    }.issubset(element["implementation_refs"])
                )
                self.assertTrue(expected_verification_refs.issubset(element["verification_refs"]))
                self.assertEqual(element["test_case_ids"], [])


if __name__ == "__main__":
    unittest.main()
