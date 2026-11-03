"""Pin exact T102 ClientResponseView mappings to existing outcome accessors.

The cited KMIP clauses identify operation response payloads only. This
manifest-only mapping adds no protocol behavior or conformance claim.
"""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"

# declaration ID, method, operation suffix, C, Java, Python, section, response table
EXPECTED = (
    ("rust-observed-0fff45b22624c9863341", "certify", "CERTIFY", "kmipkit_client_batch_item_response_certify", "org.kmipkit.extensions.ClientBatchItemResult.certifyResponse", "kmipkit.client.BatchItemResult.certify_response", "6.1.6", 180),
    ("rust-observed-54eed5a0560f0f327be1", "check", "CHECK", "kmipkit_client_batch_item_response_check", "org.kmipkit.extensions.ClientBatchItemResult.checkResponse", "kmipkit.client.BatchItemResult.check_response", "6.1.7", 184),
    ("rust-observed-bab5f3119a04dc27703d", "obtain_lease", "OBTAIN-LEASE", "kmipkit_client_batch_item_response_obtain_lease", "org.kmipkit.extensions.ClientBatchItemResult.obtainLeaseResponse", "kmipkit.client.BatchItemResult.obtain_lease_response", "6.1.35", 269),
    ("rust-observed-d771ff20731cba0fe337", "recertify", "RE-CERTIFY", "kmipkit_client_batch_item_response_recertify", "org.kmipkit.extensions.ClientBatchItemResult.recertifyResponse", "kmipkit.client.BatchItemResult.recertify_response", "6.1.45", 301),
    ("rust-observed-0c3da3aa589125b3f6f2", "rekey", "RE-KEY", "kmipkit_client_batch_item_response_rekey", "org.kmipkit.extensions.ClientBatchItemResult.rekeyResponse", "kmipkit.client.BatchItemResult.rekey_response", "6.1.46", 306),
    ("rust-observed-b92daec87f8a60fd2ba9", "rekey_key_pair", "RE-KEY-KEY-PAIR", "kmipkit_client_batch_item_response_rekey_key_pair", "org.kmipkit.extensions.ClientBatchItemResult.rekeyKeyPairResponse", "kmipkit.client.BatchItemResult.rekey_key_pair_response", "6.1.47", 311),
    ("rust-observed-b878eca12ed7a7e2ea13", "reprovision", "RE-PROVISION", "kmipkit_client_batch_item_response_reprovision", "org.kmipkit.extensions.ClientBatchItemResult.reprovisionResponse", "kmipkit.client.BatchItemResult.reprovision_response", "6.1.48", 314),
    ("rust-observed-0e423c1d2c8da37b44e3", "register", "REGISTER", "kmipkit_client_batch_item_response_register", "org.kmipkit.extensions.ClientBatchItemResult.registerResponse", "kmipkit.client.BatchItemResult.register_response", "6.1.43", 292),
    ("rust-observed-8ca19e8a2669101cd02e", "revoke", "REVOKE", "kmipkit_client_batch_item_response_revoke", "org.kmipkit.extensions.ClientBatchItemResult.revokeResponse", "kmipkit.client.BatchItemResult.revoke_response", "6.1.44", 296),
)

# Baseline fingerprints exclude only the nine target rows and their nine Rust list entries.
UNCHANGED_DECLARATIONS_SHA256 = "c2064f29bd0406ac59af2bd307d35ebe436beecc1b57b9fd063111bbcea4d52c"
UNCHANGED_OPERATIONS_SHA256 = "9e68400351e8ad20d5b41bd28893f79622babfbbd6dbefc559d7283948cb09f5"
OTHER_MANIFEST_SHA256 = "7dc81a2126ba64cbf79a839ba94a8e972c838707e1aecd54d793938e2078c3ea"
EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH = 759


def canonical_sha256(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


class T102ResponseViewMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.declarations_by_id = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.operations = {row["operationId"]: row for row in cls.manifest["operations"]}

    def test_exact_response_views_reuse_the_existing_four_language_outcomes(self) -> None:
        expected_ids = {item[0] for item in EXPECTED}
        expected_exports = {
            f"kmipkit_client::ClientResponseView<'a>::{item[1]}" for item in EXPECTED
        }
        actual = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client" and row["exportName"] in expected_exports
        }
        self.assertEqual(expected_ids, actual)

        for declaration_id, method, suffix, c_name, java_name, python_name, section, table in EXPECTED:
            operation_id = f"KMIPKIT-ELEM-OP-C2S-{suffix}"
            rust_name = f"kmipkit_client::ClientResponseView<'a>::{method}"
            foreign = {
                "c": self.declarations[("c-kmipkit", c_name)],
                "java": self.declarations[("java-kmipkit", java_name)],
                "python": self.declarations[("python-kmipkit", python_name)],
            }
            target = self.declarations_by_id[declaration_id]
            operation = self.operations[operation_id]
            with self.subTest(operation=operation_id, response_table=table):
                self.assertEqual(rust_name, target["exportName"])
                self.assertEqual("associated-item", target["kind"])
                self.assertEqual("verified", target["declarationState"])
                self.assertEqual(operation_id, target["capabilityId"])
                self.assertEqual("four-language", target["parityScope"])
                self.assertNotIn("candidateSpecId", target)
                self.assertNotIn("blockedReason", target)

                source = foreign["c"]
                self.assertEqual("verified", source["declarationState"])
                self.assertEqual(operation_id, source["capabilityId"])
                for source_row in foreign.values():
                    self.assertEqual("verified", source_row["declarationState"])
                    self.assertEqual(source["requirementIds"], source_row["requirementIds"])
                    self.assertEqual(source["approvalSource"], source_row["approvalSource"])
                    self.assertEqual(source["parityEvidenceIds"], source_row["parityEvidenceIds"])
                self.assertEqual(source["requirementIds"], target["requirementIds"])
                self.assertEqual(source["approvalSource"], target["approvalSource"])
                self.assertEqual(source["parityEvidenceIds"], target["parityEvidenceIds"])
                self.assertTrue(
                    {"KMIPKIT-0036-FR-001", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021"}
                    <= set(target["requirementIds"])
                )

                expected_mappings = {
                    "rust": {"packageId": "rust-kmipkit-client", "name": rust_name},
                    "c": {"packageId": "c-kmipkit", "name": c_name},
                    "java": {"packageId": "java-kmipkit", "name": java_name},
                    "python": {"packageId": "python-kmipkit", "name": python_name},
                }
                for language, source_row in foreign.items():
                    self.assertEqual(expected_mappings[language], source_row["languageMappings"][language])
                self.assertEqual(expected_mappings, target["languageMappings"])

                self.assertEqual("verified", operation["apiState"])
                self.assertEqual(section, operation["section"])
                self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
                self.assertEqual(
                    {"specId": "KMIPKIT-0036", "state": "approved"},
                    operation["ownerDependency"],
                )
                self.assertEqual(1, operation["declarations"]["rust"].count(declaration_id))

    def test_no_unlisted_declaration_or_operation_row_changes(self) -> None:
        target_ids = {item[0] for item in EXPECTED}
        unchanged_declarations = [
            row for row in self.manifest["publicDeclarations"]
            if row["declarationId"] not in target_ids
        ]
        self.assertEqual(UNCHANGED_DECLARATIONS_SHA256, canonical_sha256(unchanged_declarations))

        target_operations = {f"KMIPKIT-ELEM-OP-C2S-{item[2]}" for item in EXPECTED}
        normalized_operations = json.loads(json.dumps(self.manifest["operations"]))
        for operation in normalized_operations:
            if operation["operationId"] in target_operations:
                operation["declarations"]["rust"] = [
                    value for value in operation["declarations"]["rust"] if value not in target_ids
                ]
        self.assertEqual(UNCHANGED_OPERATIONS_SHA256, canonical_sha256(normalized_operations))
        other_manifest = {
            key: value for key, value in self.manifest.items()
            if key not in {"publicDeclarations", "operations"}
        }
        self.assertEqual(OTHER_MANIFEST_SHA256, canonical_sha256(other_manifest))

    def test_post_batch_manifest_totals_are_pinned(self) -> None:
        blocked = sum(
            row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH, T102_BLOCKED_CANDIDATES)
        self.assertEqual(EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH, blocked)
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))


if __name__ == "__main__":
    unittest.main()