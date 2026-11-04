"""Require four-language mappings for lifecycle request identifier members."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER = "specs/022-object-lifecycle/spec.md"
MAPPING_EVIDENCE = (
    "tools/api_manifest/tests/test_t102_lifecycle_request_identifier_member_mapping.py"
)
OPERATIONS = (
    {
        "name": "Check",
        "capability": "KMIPKIT-ELEM-OP-C2S-CHECK",
        "builder": "kmipkit_client_batch_item_check",
        "constructor": "kmipkit_protocol::CheckRequest::new",
        "table": "| Check | KMIPKIT-ELEM-OP-C2S-CHECK | §6.1.7 | 183 | 184 | 185 |",
        "section": "§6.1.7",
        "test": "crates/kmipkit-protocol/tests/unit/check_operation_tests.rs",
        "targets": (
            ("rust-observed-0cac70728b14d2cc2029", "kmipkit_protocol::CheckRequest::unique_identifier"),
        ),
    },
    {
        "name": "Obtain Lease",
        "capability": "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE",
        "builder": "kmipkit_client_batch_item_obtain_lease",
        "constructor": "kmipkit_protocol::ObtainLeaseRequest::new",
        "table": "| Obtain Lease | KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE | §6.1.35 | 268 | 269 | 270 |",
        "section": "§6.1.35",
        "test": "crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs",
        "targets": (
            ("rust-observed-c2c1ef4a9b7cde16f0b8", "kmipkit_protocol::ObtainLeaseRequest::unique_identifier"),
        ),
    },
    {
        "name": "Re-certify",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        "builder": "kmipkit_client_batch_item_recertify",
        "constructor": "kmipkit_protocol::RecertifyRequest::new",
        "table": "| Re-certify | KMIPKIT-ELEM-OP-C2S-RE-CERTIFY | §6.1.45 | 300 | 301 | 302 |",
        "section": "§6.1.45",
        "test": "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
        "targets": (
            ("rust-observed-1f4d5ae1f197c06d274f", "kmipkit_protocol::RecertifyRequest::certificate_request_unique_identifier"),
            ("rust-observed-e1413e6ad346eed590c5", "kmipkit_protocol::RecertifyRequest::unique_identifier"),
        ),
    },
    {
        "name": "Re-key",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-KEY",
        "builder": "kmipkit_client_batch_item_rekey",
        "constructor": "kmipkit_protocol::RekeyRequest::new",
        "table": "| Re-key | KMIPKIT-ELEM-OP-C2S-RE-KEY | §6.1.46 | 305 | 306 | 307 |",
        "section": "§6.1.46",
        "test": "crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs",
        "targets": (
            ("rust-observed-c0e54cfc91f60a80e3b9", "kmipkit_protocol::RekeyRequest::unique_identifier"),
        ),
    },
    {
        "name": "Re-key Key Pair",
        "capability": "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR",
        "builder": "kmipkit_client_batch_item_rekey_key_pair",
        "constructor": "kmipkit_protocol::RekeyKeyPairRequest::new",
        "table": "| Re-key Key Pair | KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR | §6.1.47 | 310 | 311 | 312 |",
        "section": "§6.1.47",
        "test": "crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs",
        "targets": (
            ("rust-observed-0f930bd1ba867141cf20", "kmipkit_protocol::RekeyKeyPairRequest::private_key_unique_identifier"),
        ),
    },
)


class T102LifecycleRequestIdentifierMemberMappingTests(unittest.TestCase):
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

    def test_lifecycle_request_identifier_members_map_to_existing_builders(self) -> None:
        for spec in OPERATIONS:
            operation = self.operations[spec["capability"]]
            builder = self.by_name[("c-kmipkit", spec["builder"])]
            constructor = self.by_name[("rust-kmipkit-protocol", spec["constructor"])]
            with self.subTest(operation=spec["name"]):
                self.assertEqual(spec["name"], operation["name"])
                self.assertEqual(spec["capability"], builder["capabilityId"])
                self.assertEqual(OWNER, builder["approvalSource"]["path"])
                self.assertEqual("Approved", builder["approvalSource"]["status"])
                for declaration_id, export_name in spec["targets"]:
                    with self.subTest(declaration=export_name):
                        declaration = self.declarations[declaration_id]
                        mappings = dict(builder["languageMappings"])
                        mappings["rust"] = {
                            "packageId": "rust-kmipkit-protocol",
                            "name": export_name,
                        }
                        self.assertEqual(export_name, declaration["exportName"])
                        self.assertEqual("verified", declaration["declarationState"])
                        self.assertEqual(spec["capability"], declaration["capabilityId"])
                        self.assertEqual(constructor["approvalSource"], declaration["approvalSource"])
                        self.assertEqual(constructor["requirementIds"], declaration["requirementIds"])
                        self.assertEqual("four-language", declaration["parityScope"])
                        self.assertEqual(mappings, declaration["languageMappings"])
                        self.assertIn(MAPPING_EVIDENCE, declaration["parityEvidenceIds"])
                        self.assertIn(declaration_id, operation["declarations"]["rust"])

    def test_each_mapping_cites_approved_owner_and_exact_oasis_request_source(self) -> None:
        specification = (ROOT / OWNER).read_text(encoding="utf-8")
        for spec in OPERATIONS:
            with self.subTest(operation=spec["name"]):
                self.assertIn(spec["table"], specification)
                source_test = (ROOT / spec["test"]).read_text(encoding="utf-8")
                self.assertIn("OASIS KMIP Specification v2.1", source_test)
                self.assertIn(spec["section"].removeprefix("§"), source_test)


if __name__ == "__main__":
    unittest.main()
