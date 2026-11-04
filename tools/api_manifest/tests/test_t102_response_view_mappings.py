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
from tools.api_manifest.tests.test_t102_locate_request_builder_mapping import (
    LOCATE_DECLARATIONS_WITH_MISATTRIBUTED_SERVER_REQUIREMENT,
    LOCATE_FUNCTION_IDS,
)
from tools.api_manifest.tests.test_t102_recertify_rekey_key_pair_builder_mapping import (
    RECERTIFY,
    RECERTIFY_TYPE_ID,
    REKEY_KEY_PAIR,
    TARGETS as RECERTIFY_REKEY_KEY_PAIR_TARGETS,
)
from tools.api_manifest.tests.test_recertify_parity_manifest import (
    RETIRED_REQUIREMENT_DECLARATIONS,
    RETIRED_REQUIREMENTS,
    REQUEST_REQUIREMENT_DECLARATIONS,
    OWNER_REQUIREMENTS,
    ACTIVE_REQUEST_REQUIREMENTS,
)

from tools.api_manifest.tests.test_t102_certify_request_builder_mapping import (
    OPERATION as CERTIFY_OPERATION_ID,
    OPERATION_ADDED_EVIDENCE as CERTIFY_OPERATION_ADDED_EVIDENCE,
    TARGETS as CERTIFY_TARGETS,
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

# Baseline fingerprints exclude response views and exact request-builder batches.
UNCHANGED_DECLARATIONS_SHA256 = "de724b98916761845378dc8e4052676b0c6a6a3aba035bc3c9fe1ff72e62e071"
UNCHANGED_OPERATIONS_SHA256 = "eb69336ea313c72f6461f5e67655d985c820969697237b4b2b8efc3d79712735"
OTHER_MANIFEST_SHA256 = "89ce2f39dccd3e4548dc0145da0e8b4463f4f4ae769ed472d80cf44b26679ba6"
EXPECTED_BLOCKED_CANDIDATES_AFTER_BATCH = 654
LOCATE_REQUEST_BUILDER_IDS = {
    "rust-observed-e07fce825be9d33278e1",
    "rust-observed-8fff50bdd49a09ae00bf",
    "rust-observed-aa9a474fd92d1503b86d",
    "rust-observed-8d60104e0b52b30f3496",
    "rust-observed-feb2e57f6cbe856a8fa1",
}
LOCATE_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-LOCATE"
LOCATE_REQUEST_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.28-007"
LOCATE_SERVER_ONLY_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.28-011"
JOIN_SPLIT_SECRET_DATA_TYPE_ID = "rust-observed-7c003b4ec061b8499a9e"
REKEY_KEY_PAIR_UID_ID = "rust-observed-3fc6c1f77db9dabbee97"
REKEY_KEY_PAIR_STRUCTURE_IDS = {
    "rust-observed-9deea2dde23e9a92294d",
    "rust-observed-f67674816d29b1f2879f",
    "rust-observed-c084e31187a809b1d6bb",
    "rust-observed-f428c5bc2011a1634a10",
    "rust-observed-d51793dbfa80b42a8e94",
    "rust-observed-1bdba1f1266aff84ed52",
}
REKEY_STRUCTURE_IDS = {
    "rust-observed-55695fec9108789be098",
    "rust-observed-3c3be169b5ff8899e1c7",
}
JOIN_SPLIT_STRUCTURE_IDS = {
    "rust-observed-f894598b6998d1b416b1",
    "rust-observed-236134f1fc8e195442a0",
}
RECERTIFY_STRUCTURE_IDS = {
    "rust-observed-0cad07b4f1010a7d9a55",
    "rust-observed-6e9fb656f103c43fa2de",
}
JOIN_SPLIT_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY"
REKEY_KEY_PAIR_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR"
REQUEST_BUILDER_BATCH_EVIDENCE = {
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "tools/api_manifest/tests/test_t102_join_split_key_secret_data_type_mapping.py",
    "tools/api_manifest/tests/test_t102_rekey_key_pair_structure_mapping.py",
}
REKEY_OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-KEY"
RETIRED_REKEY_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.46-004-002"
REKEY_FUNCTION_IDS = {
    "client_batch_item_rekey",
    "client_batch_item_response_rekey",
    "client_batch_item_response_rekey_ttlv",
}
REKEY_KEY_PAIR_FUNCTION_IDS = {
    "client_batch_item_rekey_key_pair",
    "client_batch_item_response_rekey_key_pair",
    "client_batch_item_response_rekey_key_pair_ttlv",
}
RETIRED_REKEY_KEY_PAIR_REQUIREMENT = "KMIPKIT-REQ-SPEC-6.1.47-004-002"
SCALAR_REQUEST_BUILDER_IDS = {
    "rust-observed-9cd4c6b6fd190aff211f",
    "rust-observed-506281d0cf8e3c41a90e",
    "rust-observed-61aecad439c9a84c7241",
    "rust-observed-ba1772cec998d92e5148",
    "rust-observed-3beea67907cb67493c50",
    "rust-observed-7ef0a3fa4f7211edc2d4",
    "rust-observed-2117b9df5e767118ae4c",
    "rust-observed-ad3438f583c6a1a34512",
    "rust-observed-4d485551dac41dbd1ef3",
    "rust-observed-66a8b011f8175b8c04f5",
    "rust-observed-684a647c13f5225359fa",
    "rust-observed-c2da75d2d210d98b7ea5",
    "rust-observed-6611a6ba852bddfeb702",
    "rust-observed-e0c241319e5da88830dd",
    "rust-observed-e4656d1822c4f833d1ba",
    "rust-observed-27c208d3d7f52f071e4c",
    "rust-observed-bc4b3a9f02b6b38524fa",
    "rust-observed-ea651cfc24b62abdda90",
    "rust-observed-5cf48b4b17981987d2f7",
    "rust-observed-4781288042e981f6d3bd",
    "rust-observed-43be97b668d6f5efb547",
    "rust-observed-e04ecae89493ca7e5fbf",
    "rust-observed-51a60237c466a9503394",
    "rust-observed-543f908844c7bb7b67bb",
}
SCALAR_REQUEST_BUILDER_OPERATIONS = {
    "KMIPKIT-ELEM-OP-C2S-CHECK",
    "KMIPKIT-ELEM-OP-C2S-GET",
    "KMIPKIT-ELEM-OP-C2S-EXPORT",
    "KMIPKIT-ELEM-OP-C2S-RE-KEY",
    "KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION",
    "KMIPKIT-ELEM-OP-C2S-IMPORT",
}


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
        target_ids.update(LOCATE_REQUEST_BUILDER_IDS)
        target_ids.update(RECERTIFY_REKEY_KEY_PAIR_TARGETS)
        target_ids.update(REKEY_STRUCTURE_IDS)
        target_ids.update(JOIN_SPLIT_STRUCTURE_IDS)
        target_ids.update(RECERTIFY_STRUCTURE_IDS)
        affected_operation_ids = target_ids | SCALAR_REQUEST_BUILDER_IDS | {
            JOIN_SPLIT_SECRET_DATA_TYPE_ID,
            REKEY_KEY_PAIR_UID_ID,
            *REKEY_KEY_PAIR_STRUCTURE_IDS,
            *REKEY_STRUCTURE_IDS,
            *JOIN_SPLIT_STRUCTURE_IDS,
            *RECERTIFY_STRUCTURE_IDS,
            RECERTIFY_TYPE_ID,
            *(target["id"] for target in CERTIFY_TARGETS),
        }
        unchanged_declarations = copy.deepcopy([
            row
            for row in self.manifest["publicDeclarations"]
            if row["declarationId"] not in target_ids
        ])
        for declaration in unchanged_declarations:
            if declaration["declarationId"] in {
                "rust-observed-e0a06a71290bee987cc7",
                "rust-observed-5722377217051635233d",
                "rust-observed-bd90b6e53d6c05c1a6b0",
                "rust-observed-4c37b205dc5bee05df9a",
            }:
                declaration["parityEvidenceIds"].remove(
                    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
                )
            if declaration["declarationId"] == RECERTIFY_TYPE_ID:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration[
                    "blockedReason"
                ] = "The public Rust declaration has no resolved owner in the approved dependency inventory; assign an owner before advertising it."
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in {
                target["id"] for target in CERTIFY_TARGETS
            }:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration[
                    "blockedReason"
                ] = "The public Rust declaration has no resolved owner in the approved dependency inventory; assign an owner before advertising it."
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in {
                JOIN_SPLIT_SECRET_DATA_TYPE_ID,
                REKEY_KEY_PAIR_UID_ID,
                *REKEY_KEY_PAIR_STRUCTURE_IDS,
                *REKEY_STRUCTURE_IDS,
                *JOIN_SPLIT_STRUCTURE_IDS,
                *RECERTIFY_STRUCTURE_IDS,
            }:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration[
                    "blockedReason"
                ] = "The public Rust declaration has no resolved owner in the approved dependency inventory; assign an owner before advertising it."
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in RETIRED_REQUIREMENT_DECLARATIONS:
                requirements = set(declaration["requirementIds"])
                requirements.difference_update(ACTIVE_REQUEST_REQUIREMENTS)
                requirements.update((OWNER_REQUIREMENTS - {"KMIPKIT-REQ-SPEC-6.1.45-002-003"}) | RETIRED_REQUIREMENTS)
                declaration["requirementIds"] = sorted(requirements)
            if declaration["declarationId"] in SCALAR_REQUEST_BUILDER_IDS:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration[
                    "blockedReason"
                ] = "The public Rust declaration has no resolved owner in the approved dependency inventory; assign an owner before advertising it."
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in LOCATE_DECLARATIONS_WITH_MISATTRIBUTED_SERVER_REQUIREMENT:
                requirements = set(declaration["requirementIds"])
                requirements.discard(LOCATE_REQUEST_REQUIREMENT)
                requirements.add(LOCATE_SERVER_ONLY_REQUIREMENT)
                declaration["requirementIds"] = sorted(requirements)
            if declaration.get("capabilityId") == REKEY_OPERATION_ID:
                declaration["requirementIds"] = sorted(
                    set(declaration["requirementIds"]) | {RETIRED_REKEY_REQUIREMENT}
                )
            if declaration.get("capabilityId") == REKEY_KEY_PAIR_OPERATION_ID:
                declaration["requirementIds"] = sorted(
                    set(declaration["requirementIds"])
                    | {RETIRED_REKEY_KEY_PAIR_REQUIREMENT}
                )
        self.assertEqual(
            UNCHANGED_DECLARATIONS_SHA256,
            canonical_sha256(unchanged_declarations),
        )

        target_operations = {
            f"KMIPKIT-ELEM-OP-C2S-{item.operation_suffix}" for item in EXPECTED
        }
        target_operations.update(item.operation_id for item in REMAINING_RESPONSE_VIEWS)
        target_operations.add(LOCATE_OPERATION_ID)
        target_operations.update({RECERTIFY, REKEY_KEY_PAIR})
        target_operations.update({
            "KMIPKIT-ELEM-OP-C2S-RE-KEY",
            "KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY",
            "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY",
        })
        target_operations.update({JOIN_SPLIT_OPERATION_ID, REKEY_KEY_PAIR_OPERATION_ID})
        target_operations.update(SCALAR_REQUEST_BUILDER_OPERATIONS)
        normalized_operations = copy.deepcopy(self.manifest["operations"])
        for operation in normalized_operations:
            if operation["operationId"] in target_operations:
                operation["declarations"]["rust"] = [
                    value
                    for value in operation["declarations"]["rust"]
                    if value not in affected_operation_ids
                ]
                if operation["operationId"] in {
                    JOIN_SPLIT_OPERATION_ID,
                    REKEY_KEY_PAIR_OPERATION_ID,
                }:
                    operation["parityEvidenceIds"] = [
                        value
                        for value in operation["parityEvidenceIds"]
                        if value not in REQUEST_BUILDER_BATCH_EVIDENCE
                    ]
                if operation["operationId"] == CERTIFY_OPERATION_ID:
                    operation["parityEvidenceIds"] = [
                        value
                        for value in operation["parityEvidenceIds"]
                        if value not in CERTIFY_OPERATION_ADDED_EVIDENCE
                    ]
                if operation["operationId"] == RECERTIFY:
                    operation["parityEvidenceIds"] = [
                        value
                        for value in operation["parityEvidenceIds"]
                        if value
                        != "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
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
        other_manifest["requirementIds"] = [
            requirement_id
            for requirement_id in other_manifest["requirementIds"]
            if requirement_id not in {
                LOCATE_REQUEST_REQUIREMENT,
                "KMIPKIT-REQ-SPEC-6.1.45-002-003",
                "KMIPKIT-REQ-SPEC-6.1.18-001-001",
                "KMIPKIT-REQ-SPEC-6.1.18-008",
                "KMIPKIT-REQ-SPEC-7.18-001-001",
                "KMIPKIT-REQ-SPEC-7.18-004",
            }
        ]
        for function in other_manifest["functions"]:
            if function["id"] == "client_batch_item_recertify":
                function["c"]["documentation"][0] = (
                    "Creates a typed Re-certify item. Certificate Request Type is present only with "
                    "Certificate Request Value; request bytes are staged as sensitive data and wiped "
                    "after the synchronous ABI call."
                )
            if function["id"] in {
                "client_batch_item_recertify",
                "client_batch_item_response_recertify",
                "client_batch_item_response_recertify_ttlv",
            }:
                requirements = set(function["requirementIds"])
                requirements.difference_update(ACTIVE_REQUEST_REQUIREMENTS)
                requirements.update((OWNER_REQUIREMENTS - {"KMIPKIT-REQ-SPEC-6.1.45-002-003"}) | RETIRED_REQUIREMENTS)
                function["requirementIds"] = sorted(requirements)
            if function["id"] in REKEY_FUNCTION_IDS:
                function["requirementIds"] = sorted(
                    set(function["requirementIds"]) | {RETIRED_REKEY_REQUIREMENT}
                )
            if function["id"] in REKEY_KEY_PAIR_FUNCTION_IDS:
                function["requirementIds"] = sorted(
                    set(function["requirementIds"])
                    | {RETIRED_REKEY_KEY_PAIR_REQUIREMENT}
                )
            if function["id"] in LOCATE_FUNCTION_IDS:
                requirements = set(function["requirementIds"])
                requirements.discard(LOCATE_REQUEST_REQUIREMENT)
                requirements.add(LOCATE_SERVER_ONLY_REQUIREMENT)
                function["requirementIds"] = sorted(requirements)
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
