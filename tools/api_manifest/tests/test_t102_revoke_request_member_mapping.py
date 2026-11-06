"""Require four-language mappings for the remaining Revoke request members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-REVOKE"
OWNER = "specs/022-object-lifecycle/spec.md"
MAPPING_EVIDENCE = (
    "tools/api_manifest/tests/test_t102_revoke_request_member_mapping.py"
)
BUILDER = "kmipkit_revoke_request_create"
TARGETS = (
    (
        "rust-observed-12003f4a97a411611f15",
        "kmipkit_protocol::RevokeRequest::compromise_occurrence_date",
    ),
    (
        "rust-observed-3020c4fea7458e2d97c4",
        "kmipkit_protocol::RevokeRequest::revocation_reason",
    ),
    (
        "rust-observed-b20a90b8e34124a1006c",
        "kmipkit_protocol::RevokeRequest::unique_identifier",
    ),
)


class T102RevokeRequestMemberMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.by_name = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.operations = {
            row["operationId"]: row for row in cls.manifest["operations"]
        }

    def _assert_mapped_member(
        self,
        declaration: dict[str, object],
        constructor: dict[str, object],
        mappings: dict[str, object],
        declaration_id: str,
        export_name: str,
        operation: dict[str, object],
    ) -> None:
        self.assertEqual(export_name, declaration["exportName"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual(OPERATION_ID, declaration["capabilityId"])
        self.assertEqual(constructor["approvalSource"], declaration["approvalSource"])
        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(mappings, declaration["languageMappings"])
        self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_revoke_request_members_map_to_the_existing_four_language_builder(self) -> None:
        operation = self.operations[OPERATION_ID]
        builder = self.by_name[("c-kmipkit", BUILDER)]
        constructor = self.by_name[
            ("rust-kmipkit-protocol", "kmipkit_protocol::RevokeRequest::new")
        ]
        expected_mappings = builder["languageMappings"]

        self.assertEqual("Revoke", operation["name"])
        self.assertEqual(OWNER, builder["approvalSource"]["path"])
        self.assertEqual("Approved", builder["approvalSource"]["status"])

        for declaration_id, export_name in TARGETS:
            with self.subTest(declaration=export_name):
                declaration = self.declarations[declaration_id]
                mappings = dict(expected_mappings)
                mappings["rust"] = {
                    "packageId": "rust-kmipkit-protocol",
                    "name": export_name,
                }
                self._assert_mapped_member(
                    declaration,
                    constructor,
                    mappings,
                    declaration_id,
                    export_name,
                    operation,
                )

    def test_mapping_pins_approved_owner_and_oasis_request_source(self) -> None:
        spec = (ROOT / OWNER).read_text(encoding="utf-8")
        self.assertIn(
            "| Revoke | KMIPKIT-ELEM-OP-C2S-REVOKE | §6.1.44 | 295 | 296 | 297 |",
            spec,
        )
        tests = (ROOT / "crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("OASIS KMIP v2.1 §6.1.44, Tables 295–297", tests)


if __name__ == "__main__":
    unittest.main()
