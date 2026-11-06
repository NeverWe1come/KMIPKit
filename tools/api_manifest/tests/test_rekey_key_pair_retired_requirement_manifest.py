"""Pin the retired §6.1.47 extraction away from client API mappings."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
TRACEABILITY = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"
REQUIREMENT_ID = "KMIPKIT-REQ-SPEC-6.1.47-004-002"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
EXPECTED_DECLARATION_IDS = {
    "rust-observed-b467bfa3e40d71f19254",
    "rust-observed-cc9dd6c8044970a53510",
    "rust-observed-61b9b81e384284c75a40",
    "rust-observed-bd921e7a4b0ed843db9f",
    "rust-observed-58f6c50db233a1d8e84e",
    "rust-observed-3100cae15017b4c5a58d",
    "rust-observed-85efc189fa4defe3324f",
    "rust-observed-d2e1e440cc7838c81692",
    "rust-observed-f396916f1529c64fd411",
    "rust-observed-c70bc13dfc6a8142ae3c",
    "rust-observed-f01e496c4f3e0f0b076e",
    "rust-observed-f598b03dfd81641a2dfb",
    "rust-observed-3d631236bfeee23c861a",
    "rust-observed-28e4f9dec1c7c47da0c3",
    "rust-observed-d66bd37a113f03c407a4",
    "rust-observed-ea621624ffc08446e4eb",
    "rust-observed-6866150b70bd57c825d2",
    "rust-observed-4ab6ba3b1c05356bdc16",
    "rust-observed-b92daec87f8a60fd2ba9",
    "rust-observed-f89901b2baab877ba993",
    "rust-observed-62176a97906083757b55",
    "rust-observed-2ef5a2a7d883e3b658c3",
    "rust-observed-fe952cc60febe33b881f",
    "rust-observed-b67075ad68cf5a5efea5",
    "rust-observed-4e59ec498a3147eee9b0",
    "T060:rekey-key-pair:constructor:c",
    "T060:rekey-key-pair:response:c",
    "T060:rekey-key-pair:response-ttlv:c",
    "T060:rekey-key-pair:constructor:java",
    "T060:rekey-key-pair:response:java",
    "T060:rekey-key-pair:result:type:java",
    "T060:rekey-key-pair:result:uniqueIdentifier:java",
    "T060:rekey-key-pair:response-ttlv:java",
    "T060:rekey-key-pair:response-close:java",
    "T060:rekey-key-pair:constructor:python",
    "T060:rekey-key-pair:response:python",
    "T060:rekey-key-pair:result:type:python",
    "T060:rekey-key-pair:result:uniqueIdentifier:python",
    "T060:rekey-key-pair:response-ttlv:python",
    "T060:rekey-key-pair:response-close:python",
    "T060:rekey-key-pair:result:publicKeyUniqueIdentifier:java",
    "T060:rekey-key-pair:result:publicKeyUniqueIdentifier:python",
}
EXPECTED_FUNCTION_IDS = {
    "client_batch_item_rekey_key_pair",
    "client_batch_item_response_rekey_key_pair",
    "client_batch_item_response_rekey_key_pair_ttlv",
}


class RekeyKeyPairRetiredRequirementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))

    def test_catalog_retains_retired_not_applicable_disposition_and_global_inventory(self) -> None:
        record = next(
            item for item in self.catalog["requirements"]
            if item["requirement_id"] == REQUIREMENT_ID
        )
        self.assertEqual("not_applicable", record["role"])
        self.assertEqual("not_applicable", record["direction"])
        self.assertEqual("out_of_scope", record["scope_state"])
        self.assertEqual("retired", record["status"])
        self.assertIn(REQUIREMENT_ID, self.manifest["requirementIds"])

    def test_retired_requirement_is_absent_from_exact_rekey_key_pair_declarations(self) -> None:
        declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }
        operation_declarations = {
            declaration_id
            for declaration_id, row in declarations.items()
            if row.get("capabilityId") == OPERATION_ID
        }
        self.assertTrue(EXPECTED_DECLARATION_IDS <= operation_declarations)
        linked = {
            declaration_id
            for declaration_id, row in declarations.items()
            if row.get("capabilityId") == OPERATION_ID
            and REQUIREMENT_ID in row.get("requirementIds", [])
        }
        self.assertFalse(linked, f"retired requirement remains on declarations: {sorted(linked)}")

    def test_retired_requirement_is_absent_from_exact_rekey_key_pair_function_records(self) -> None:
        function_ids = {row["id"] for row in self.manifest["functions"]}
        self.assertTrue(EXPECTED_FUNCTION_IDS <= function_ids)
        linked = {
            row["id"] for row in self.manifest["functions"]
            if REQUIREMENT_ID in row.get("requirementIds", [])
        }
        self.assertFalse(linked, f"retired requirement remains on functions: {sorted(linked)}")

    def test_t060_traceability_does_not_claim_the_retired_requirement(self) -> None:
        traceability = TRACEABILITY.read_text(encoding="utf-8")
        t060 = traceability.split("## T060", maxsplit=1)[1].split("\n## ", maxsplit=1)[0]
        self.assertNotIn(REQUIREMENT_ID, t060)


if __name__ == "__main__":
    unittest.main()
