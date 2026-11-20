"""Require four-language ownership for verified client response parsers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
LANGUAGES = {"rust", "c", "java", "python"}

PARSER_TO_OUTCOME = {
    "rust-observed-1b8cb0a8d56246ab66e9": "rust-observed-20e3cdcae0d30c3da56a",
    "rust-observed-4695f9e51055f443c3b9": "rust-observed-97fade1732968c846085",
    "rust-observed-1dcbc84657cd895290f7": "rust-observed-d234aacaf6de7859bf6c",
    "rust-observed-0610c33a16ec5a353d86": "rust-observed-bfdd7e7f68d0ea53fe62",
    "rust-observed-dbad599893bba8c03e36": "rust-observed-d05e6c52e9f6ea26088c",
    "rust-observed-d6f4dbf40589616963fd": "rust-observed-e6d4db1da9e4840c4b5b",
    "rust-observed-613af65a5997ed2edf79": "rust-observed-624d255e28682b6de1a7",
    "rust-observed-76ba35209de969593fdb": "rust-observed-0b144a99493d5cd37742",
    "rust-observed-e2187613a4b8ee42f8b5": "rust-observed-2432ebf2bae2fed9ff9c",
    "rust-observed-95ba49335064e09db4b6": "rust-observed-93dc05d4654712f049c8",
    "rust-observed-ff9e3164ada9db83e6d8": "rust-observed-ed315de4cf8184c82427",
    "rust-observed-65d774b35eb37037489c": "rust-observed-e09c91aebc408386020e",
    "rust-observed-6e234e6ce5c01318e250": "rust-observed-20e3cdcae0d30c3da56a",
    "rust-observed-ef324d291b7451ad5843": "rust-observed-b2e1bdc113799d986c97",
    "rust-observed-90bfa44ea5623d8f6792": "rust-observed-a89215e8aee625343317",
    "rust-observed-f746ae0a1f695ade560c": "rust-observed-fe847c708f4afc2b0fc5",
    "rust-observed-14a1affd80ecee9f95c5": "rust-observed-5c541a270677bdb02c69",
    "rust-observed-e794f58c65bd52f46387": "rust-observed-397ee0115e79bd8b9749",
    "rust-observed-af22e4e946ea3577bea1": "rust-observed-86fbf036a7cfd336d39f",
    "rust-observed-e6f88db020bec8429977": "rust-observed-9e0225a1993c296be35f",
    "rust-observed-3be3955363e3c117651f": "rust-observed-c02406dfc35f329dd0b9",
    "rust-observed-40b23b68dd8b437e73db": "rust-observed-b4671df45a077a12336e",
    "rust-observed-ca43cac14fbf69e278f6": "rust-observed-bfdd7e7f68d0ea53fe62",
    "rust-observed-b1d6981fd09b9ea8ed3c": "rust-observed-d05e6c52e9f6ea26088c",
    "rust-observed-434fb63f67544fafe593": "rust-observed-09e7553e1b75c9f30386",
    "rust-observed-d9a8a9909feb4901b637": "rust-observed-93dc05d4654712f049c8",
    "rust-observed-0f7458266f8799509101": "rust-observed-0a63207cf9e22b447e5a",
    "rust-observed-df01f6f04f8d9cf4ba99": "rust-observed-6d846e7c70fe25fc18e5",
    "rust-observed-479c345f536d32cba703": "rust-observed-2ec9dc60baa7dc32a050",
    "rust-observed-68df328df6ad480b4be4": "rust-observed-58f6c50db233a1d8e84e",
    "rust-observed-4d0d3b95f24eb43db67a": "rust-observed-5e584ed44d346ebc07dd",
    "rust-observed-6098d75144d33ae15635": "rust-observed-c9bc226ded2cf2e16a82",
    "rust-observed-064d9aba97c6ee2db83e": "rust-observed-e6d4db1da9e4840c4b5b",
    "rust-observed-16999bd0ed66996be312": "rust-observed-624d255e28682b6de1a7",
    "rust-observed-216cb6798b6ce784fed4": "rust-observed-dafdbc59fc336cf55b3d",
    "rust-observed-7dd5ec89374050e73f7e": "rust-observed-867763c25321166b9ea2",
}


class T102ResponseParserMappingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        self.operations = {
            row["operationId"]: row for row in self.manifest["operations"]
        }

    def test_owned_response_parsers_have_four_language_mappings(self) -> None:
        self.assertEqual(36, sum(outcome_id is not None for outcome_id in PARSER_TO_OUTCOME.values()))

        for parser_id, outcome_id in PARSER_TO_OUTCOME.items():
            if outcome_id is None:
                continue

            with self.subTest(parser_id=parser_id):
                parser = self.declarations[parser_id]
                outcome = self.declarations[outcome_id]
                operation_id = outcome["capabilityId"]
                operation = self.operations[operation_id]

                self.assertEqual("verified", outcome["declarationState"])
                self.assertEqual(
                    "ClientBatchOutcome",
                    outcome["exportName"].split("::")[-2],
                )
                self.assertEqual(LANGUAGES, set(outcome["languageMappings"]))
                self.assertEqual("verified", parser["declarationState"])
                self.assertEqual("four-language", parser["parityScope"])
                self.assertEqual(operation_id, parser["capabilityId"])
                self.assertEqual(outcome["requirementIds"], parser["requirementIds"])
                self.assertEqual(outcome["approvalSource"], parser["approvalSource"])
                self.assertEqual(outcome["parityEvidenceIds"], parser["parityEvidenceIds"])
                self.assertEqual(LANGUAGES, set(parser["languageMappings"]))
                self.assertEqual(
                    {
                        "packageId": parser["packageId"],
                        "name": parser["exportName"],
                    },
                    parser["languageMappings"]["rust"],
                )
                for language in LANGUAGES - {"rust"}:
                    self.assertEqual(
                        outcome["languageMappings"][language],
                        parser["languageMappings"][language],
                    )

                self.assertEqual("verified", operation["apiState"])
                self.assertEqual("client_1_0", operation["scope"])
                self.assertEqual("approved", operation["ownerDependency"]["state"])
                self.assertEqual(
                    operation["approvalEvidence"]["path"],
                    parser["approvalSource"]["path"],
                )
                self.assertIn(parser_id, operation["declarations"]["rust"])
                self.assertNotIn("blockedReason", parser)
                for evidence_path in parser["parityEvidenceIds"]:
                    self.assertTrue((ROOT / evidence_path).is_file(), evidence_path)

    def test_every_response_parser_candidate_is_accounted_for(self) -> None:
        parser_candidate_ids = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row.get("exportName", "").endswith("::try_from_response_item")
            and (
                row.get("candidateSpecId") == "KMIPKIT-0043"
                or row.get("declarationId") in PARSER_TO_OUTCOME
            )
        }
        self.assertEqual(set(PARSER_TO_OUTCOME), parser_candidate_ids)

    def test_generated_parity_inventory_contains_each_mapped_parser(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        generated_operations = {
            row["operationId"]: row for row in generated["operations"]
        }
        for parser_id, outcome_id in PARSER_TO_OUTCOME.items():
            if outcome_id is None:
                continue
            operation_id = self.declarations[outcome_id]["capabilityId"]
            with self.subTest(parser_id=parser_id):
                self.assertIn(
                    parser_id,
                    generated_operations[operation_id]["declarations"]["rust"],
                )


if __name__ == "__main__":
    unittest.main()
