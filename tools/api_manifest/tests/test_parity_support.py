"""Tests for shared API parity assertion helpers."""

from __future__ import annotations

import unittest

from tools.api_manifest.tests.parity_support import assert_declaration_mappings


class DeclarationMappingAssertionTests(unittest.TestCase):
    def test_rejects_declaration_mapping_that_does_not_match_expectation(self) -> None:
        operation = {
            "operationId": "KMIPKIT-ELEM-OP-C2S-TEST",
            "declarations": {"rust": ["declaration-1"]},
        }
        declarations = {
            "declaration-1": {
                "capabilityId": "KMIPKIT-ELEM-OP-C2S-TEST",
                "declarationState": "verified",
                "parityScope": "four-language",
                "languageMappings": {"rust": {"name": "actual::Mapping"}},
                "requirementIds": ["KMIPKIT-0043-FR-001"],
            }
        }

        with self.assertRaises(AssertionError):
            assert_declaration_mappings(
                self,
                operation,
                declarations,
                operation_id="KMIPKIT-ELEM-OP-C2S-TEST",
                languages={"rust"},
                expected_mappings={"rust": {"expected::Mapping"}},
                required_requirements={"KMIPKIT-0043-FR-001"},
            )


if __name__ == "__main__":
    unittest.main()
