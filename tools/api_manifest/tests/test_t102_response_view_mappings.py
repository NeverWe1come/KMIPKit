"""Pin exact T102 ClientResponseView mappings to existing outcome accessors.

Foreign facades expose common result metadata on the batch result and expose
operation-specific payload fields through their typed response accessors. This
manifest-only mapping adds no protocol behavior or conformance claim.
"""

from __future__ import annotations

import copy
import hashlib
import json
import unittest
from dataclasses import dataclass
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
)
from tools.api_manifest.tests.test_t102_remaining_response_view_mappings import (
    EXPECTED as REMAINING_RESPONSE_VIEWS,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


@dataclass(frozen=True)
class ExpectedResponseView:
    declaration_id: str
    method: str
    operation_name: str
    operation_suffix: str
    c_name: str
    java_name: str
    python_name: str
    section: str
    response_table: int


EXPECTED = (
    ExpectedResponseView(
        declaration_id="rust-observed-0fff45b22624c9863341",
        method="certify",
        operation_name="Certify",
        operation_suffix="CERTIFY",
        c_name="kmipkit_client_batch_item_response_certify",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.certifyResponse",
        python_name="kmipkit.client.BatchItemResult.certify_response",
        section="6.1.6",
        response_table=180,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-54eed5a0560f0f327be1",
        method="check",
        operation_name="Check",
        operation_suffix="CHECK",
        c_name="kmipkit_client_batch_item_response_check",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.checkResponse",
        python_name="kmipkit.client.BatchItemResult.check_response",
        section="6.1.7",
        response_table=184,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-bab5f3119a04dc27703d",
        method="obtain_lease",
        operation_name="Obtain Lease",
        operation_suffix="OBTAIN-LEASE",
        c_name="kmipkit_client_batch_item_response_obtain_lease",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.obtainLeaseResponse",
        python_name="kmipkit.client.BatchItemResult.obtain_lease_response",
        section="6.1.35",
        response_table=269,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-d771ff20731cba0fe337",
        method="recertify",
        operation_name="Re-certify",
        operation_suffix="RE-CERTIFY",
        c_name="kmipkit_client_batch_item_response_recertify",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.recertifyResponse",
        python_name="kmipkit.client.BatchItemResult.recertify_response",
        section="6.1.45",
        response_table=301,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-0c3da3aa589125b3f6f2",
        method="rekey",
        operation_name="Re-key",
        operation_suffix="RE-KEY",
        c_name="kmipkit_client_batch_item_response_rekey",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.rekeyResponse",
        python_name="kmipkit.client.BatchItemResult.rekey_response",
        section="6.1.46",
        response_table=306,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-b92daec87f8a60fd2ba9",
        method="rekey_key_pair",
        operation_name="Re-key Key Pair",
        operation_suffix="RE-KEY-KEY-PAIR",
        c_name="kmipkit_client_batch_item_response_rekey_key_pair",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.rekeyKeyPairResponse",
        python_name="kmipkit.client.BatchItemResult.rekey_key_pair_response",
        section="6.1.47",
        response_table=311,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-b878eca12ed7a7e2ea13",
        method="reprovision",
        operation_name="Re-Provision",
        operation_suffix="RE-PROVISION",
        c_name="kmipkit_client_batch_item_response_reprovision",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.reprovisionResponse",
        python_name="kmipkit.client.BatchItemResult.reprovision_response",
        section="6.1.48",
        response_table=314,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-0e423c1d2c8da37b44e3",
        method="register",
        operation_name="Register",
        operation_suffix="REGISTER",
        c_name="kmipkit_client_batch_item_response_register",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.registerResponse",
        python_name="kmipkit.client.BatchItemResult.register_response",
        section="6.1.43",
        response_table=292,
    ),
    ExpectedResponseView(
        declaration_id="rust-observed-8ca19e8a2669101cd02e",
        method="revoke",
        operation_name="Revoke",
        operation_suffix="REVOKE",
        c_name="kmipkit_client_batch_item_response_revoke",
        java_name="org.kmipkit.extensions.ClientBatchItemResult.revokeResponse",
        python_name="kmipkit.client.BatchItemResult.revoke_response",
        section="6.1.44",
        response_table=296,
    ),
)

# Baseline fingerprints exclude only the nine target rows and their Rust list entries.
UNCHANGED_DECLARATIONS_SHA256 = "d0fb0b046df5cb15d574af1748a3779888c307b7b5f69800d6b9df3ed70a4793"
UNCHANGED_OPERATIONS_SHA256 = "eb69336ea313c72f6461f5e67655d985c820969697237b4b2b8efc3d79712735"
OTHER_MANIFEST_SHA256 = "89ce2f39dccd3e4548dc0145da0e8b4463f4f4ae769ed472d80cf44b26679ba6"
EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH = 708


def canonical_sha256(value: object) -> str:
    encoded = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
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
        cls.lifecycle_spec = (ROOT / "specs" / "022-object-lifecycle" / "spec.md").read_text(
            encoding="utf-8"
        )

    def lifecycle_story(self, operation_name: str) -> str:
        for story in self.lifecycle_spec.split("### User Story ")[1:]:
            heading = story.splitlines()[0]
            name = heading.split("— ", maxsplit=1)[1].split(" (Priority:", maxsplit=1)[0]
            if name == operation_name:
                return story
        self.fail(f"approved lifecycle spec has no user story for {operation_name}")

    def test_common_result_remains_available_on_all_four_facades(self) -> None:
        result = self.declarations[
            ("rust-kmipkit-client", "kmipkit_client::ClientResponseView::result")
        ]
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::ClientResponseView::result",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_response_t",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItemResult",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.BatchItemResult",
                },
            },
            result["languageMappings"],
        )
        self.assertEqual("client_execution", result["capabilityId"])

        common_result_exports = (
            ("c-kmipkit", "kmipkit_client_batch_item_response_metadata_v2"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultStatus"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultReason"),
            ("java-kmipkit", "org.kmipkit.extensions.ClientBatchItemResult.resultMessage"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_status"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_reason"),
            ("python-kmipkit", "kmipkit.client.BatchItemResult.result_message"),
        )
        for declaration_key in common_result_exports:
            with self.subTest(export=declaration_key):
                self.assertIn(declaration_key, self.declarations)

    def test_exact_response_views_reuse_the_existing_four_language_outcomes(self) -> None:
        expected_ids = {item.declaration_id for item in EXPECTED}
        expected_exports = {
            f"kmipkit_client::ClientResponseView<'a>::{item.method}" for item in EXPECTED
        }
        actual = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit-client"
            and row["exportName"] in expected_exports
        }
        self.assertEqual(expected_ids, actual)

        for item in EXPECTED:
            operation_id = f"KMIPKIT-ELEM-OP-C2S-{item.operation_suffix}"
            rust_name = f"kmipkit_client::ClientResponseView<'a>::{item.method}"
            foreign = {
                "c": self.declarations[("c-kmipkit", item.c_name)],
                "java": self.declarations[("java-kmipkit", item.java_name)],
                "python": self.declarations[("python-kmipkit", item.python_name)],
            }
            target = self.declarations_by_id[item.declaration_id]
            operation = self.operations[operation_id]
            with self.subTest(operation=operation_id, response_table=item.response_table):
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
                    {
                        "KMIPKIT-0036-FR-001",
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    }
                    <= set(target["requirementIds"])
                )

                expected_mappings = {
                    "rust": {"packageId": "rust-kmipkit-client", "name": rust_name},
                    "c": {"packageId": "c-kmipkit", "name": item.c_name},
                    "java": {"packageId": "java-kmipkit", "name": item.java_name},
                    "python": {"packageId": "python-kmipkit", "name": item.python_name},
                }
                for language, source_row in foreign.items():
                    self.assertEqual(
                        expected_mappings[language],
                        source_row["languageMappings"][language],
                    )
                self.assertEqual(expected_mappings, target["languageMappings"])

                self.assertEqual("verified", operation["apiState"])
                self.assertEqual(item.section, operation["section"])
                self.assertIn(
                    f"Given a valid Table {item.response_table} success",
                    self.lifecycle_story(item.operation_name),
                )
                self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
                self.assertEqual(
                    {"specId": "KMIPKIT-0036", "state": "approved"},
                    operation["ownerDependency"],
                )
                self.assertEqual(
                    1,
                    operation["declarations"]["rust"].count(item.declaration_id),
                )

    def test_no_unlisted_declaration_or_operation_row_changes(self) -> None:
        target_ids = {item.declaration_id for item in EXPECTED}
        target_ids.update(item.declaration_id for item in REMAINING_RESPONSE_VIEWS)
        unchanged_declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] not in target_ids
        ]
        self.assertEqual(
            UNCHANGED_DECLARATIONS_SHA256,
            canonical_sha256(unchanged_declarations),
        )

        target_operations = {
            f"KMIPKIT-ELEM-OP-C2S-{item.operation_suffix}" for item in EXPECTED
        }
        target_operations.update(item.operation_id for item in REMAINING_RESPONSE_VIEWS)
        normalized_operations = copy.deepcopy(self.manifest["operations"])
        for operation in normalized_operations:
            if operation["operationId"] in target_operations:
                operation["declarations"]["rust"] = [
                    value
                    for value in operation["declarations"]["rust"]
                    if value not in target_ids
                ]
        self.assertEqual(
            UNCHANGED_OPERATIONS_SHA256,
            canonical_sha256(normalized_operations),
        )
        other_manifest = {
            key: value
            for key, value in self.manifest.items()
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
