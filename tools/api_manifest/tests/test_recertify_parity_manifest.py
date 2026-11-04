"""Manifest regressions for the KMIP 2.1 §6.1.45 Re-certify mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
TRACEABILITY = ROOT / "specs" / "043-public-api-bindings" / "traceability.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.45-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
    "KMIPKIT-REQ-SPEC-6.1.45-004",
    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
    "KMIPKIT-REQ-SPEC-6.1.45-008",
}
RETIRED_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.45-002-002",
    "KMIPKIT-REQ-SPEC-6.1.45-006-002",
}
RETIRED_REQUIREMENT_DECLARATIONS = {
    'T058:recertify:constructor:c',
    'T058:recertify:constructor:java',
    'T058:recertify:constructor:python',
    'T058:recertify:response-close:java',
    'T058:recertify:response-close:python',
    'T058:recertify:response-ttlv:c',
    'T058:recertify:response-ttlv:java',
    'T058:recertify:response-ttlv:python',
    'T058:recertify:response:c',
    'T058:recertify:response:java',
    'T058:recertify:response:python',
    'T058:recertify:result:type:java',
    'T058:recertify:result:type:python',
    'T058:recertify:result:uniqueIdentifier:java',
    'T058:recertify:result:uniqueIdentifier:python',
    'rust-observed-03b7476ae491e5a6e8d3',
    'rust-observed-06ead91548184c12fd56',
    'rust-observed-0d0bac2451886e74583a',
    'rust-observed-12bf8df7cd387813c576',
    'rust-observed-1f59cf7cfe8fbb9bfe63',
    'rust-observed-2e85d64836128fdd1bb6',
    'rust-observed-2e966e0f7d4d9d9debcd',
    'rust-observed-362f5fb1bf041c2d4e03',
    'rust-observed-3e27eacc084fbb531e78',
    'rust-observed-3f429f5ae2d84d55badf',
    'rust-observed-406cce712597bf6d0f2f',
    'rust-observed-4c37b205dc5bee05df9a',
    'rust-observed-5dd12d4ecfe06212aa42',
    'rust-observed-65ae22056bd47722dca2',
    'rust-observed-6d846e7c70fe25fc18e5',
    'rust-observed-7701b4c5b2bb60b96c7c',
    'rust-observed-96cbaa2e43262aca95ac',
    'rust-observed-bd90b6e53d6c05c1a6b0',
    'rust-observed-c12b6edc157fdc7037b1',
    'rust-observed-d229a05b8f66624a27fb',
    'rust-observed-d5527a5f01b746b4daa2',
    'rust-observed-d771ff20731cba0fe337',
    'rust-observed-ef05c2a46dbe1a74a5c5',
    'rust-observed-f12ee8b89c80c88461c7',
}
REQUEST_DECLARATIONS = {
    "rust-observed-d229a05b8f66624a27fb",
    "rust-observed-2e966e0f7d4d9d9debcd",
    "rust-observed-2e85d64836128fdd1bb6",
    "rust-observed-d5527a5f01b746b4daa2",
    "rust-observed-f12ee8b89c80c88461c7",
    "rust-observed-06ead91548184c12fd56",
    "rust-observed-96cbaa2e43262aca95ac",
    "rust-observed-0d0bac2451886e74583a",
    "rust-observed-5dd12d4ecfe06212aa42",
    "rust-observed-ef05c2a46dbe1a74a5c5",
    "rust-observed-e98ebdedfd3bd6dc5178",
    "rust-observed-3b7ec1d8b422ce2f856a",
    "rust-observed-a305050686931a657f67",
    "rust-observed-18147c6b3bc743f37b88",
    "T058:recertify:constructor:c",
    "T058:recertify:constructor:java",
    "T058:recertify:constructor:python",
}
NON_REQUEST_DECLARATIONS = {
    "rust-observed-03b7476ae491e5a6e8d3",
    "rust-observed-7701b4c5b2bb60b96c7c",
    "rust-observed-6d846e7c70fe25fc18e5",
    "rust-observed-bd90b6e53d6c05c1a6b0",
    "rust-observed-362f5fb1bf041c2d4e03",
    "rust-observed-65ae22056bd47722dca2",
    "rust-observed-406cce712597bf6d0f2f",
    "rust-observed-4c37b205dc5bee05df9a",
    "rust-observed-d771ff20731cba0fe337",
    "rust-observed-12bf8df7cd387813c576",  # foreign mappings resolve to response classes
    "rust-observed-c12b6edc157fdc7037b1",
    "rust-observed-1f59cf7cfe8fbb9bfe63",
    "rust-observed-3f429f5ae2d84d55badf",
    "rust-observed-3e27eacc084fbb531e78",
    "T058:recertify:response:c",
    "T058:recertify:response-ttlv:c",
    "T058:recertify:response:java",
    "T058:recertify:result:type:java",
    "T058:recertify:result:uniqueIdentifier:java",
    "T058:recertify:response-ttlv:java",
    "T058:recertify:response-close:java",
    "T058:recertify:response:python",
    "T058:recertify:result:type:python",
    "T058:recertify:result:uniqueIdentifier:python",
    "T058:recertify:response-ttlv:python",
    "T058:recertify:response-close:python",
}
ACTIVE_REQUEST_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.45-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-001",
    "KMIPKIT-REQ-SPEC-6.1.45-002-003",
    "KMIPKIT-REQ-SPEC-6.1.45-004",
    "KMIPKIT-REQ-SPEC-6.1.45-006-001",
    "KMIPKIT-REQ-SPEC-6.1.45-008",
}
RESPONSE_FUNCTIONS = {
    "client_batch_item_response_recertify",
    "client_batch_item_response_recertify_ttlv",
}
REQUEST_REQUIREMENT_DECLARATIONS = {
    "rust-observed-d229a05b8f66624a27fb",  # kmipkit::Client::recertify
    "rust-observed-2e966e0f7d4d9d9debcd",  # kmipkit::Client::recertify_with_options
    "rust-observed-2e85d64836128fdd1bb6",  # kmipkit::ClientRequest::Recertify
    "rust-observed-d5527a5f01b746b4daa2",  # kmipkit::ClientRequest::Recertify payload
    "rust-observed-f12ee8b89c80c88461c7",  # kmipkit::ClientRequest::recertify
    "rust-observed-06ead91548184c12fd56",  # kmipkit_client::Client::recertify
    "rust-observed-96cbaa2e43262aca95ac",  # kmipkit_client::Client::recertify_with_options
    "rust-observed-0d0bac2451886e74583a",  # kmipkit_client::ClientRequest::Recertify
    "rust-observed-5dd12d4ecfe06212aa42",  # kmipkit_client::ClientRequest::Recertify payload
    "rust-observed-ef05c2a46dbe1a74a5c5",  # kmipkit_client::ClientRequest::recertify
    "rust-observed-3b7ec1d8b422ce2f856a",  # RecertifyRequest::with_unique_identifier
    "T058:recertify:constructor:c",
    "T058:recertify:constructor:java",
    "T058:recertify:constructor:python",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/RecertifyMappingTest.java",
    "bindings/python/tests/test_recertify_operation.py",
    "crates/kmipkit-client/tests/unit/recertify_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs",
    "tools/api_manifest/tests/test_recertify_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::recertify",
        "kmipkit_client::ClientRequest::Recertify",
        "kmipkit_client::ClientBatchOutcome::recertify_response",
        "kmipkit_protocol::RecertifyRequest",
        "kmipkit_protocol::RecertifyResponse",
        "kmipkit_protocol::RecertifyResponse::unique_identifier",
        "kmipkit_protocol::RecertifyResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_recertify",
        "kmipkit_client_batch_item_response_recertify",
        "kmipkit_client_batch_item_response_recertify_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.recertify",
        "org.kmipkit.extensions.ClientBatchItemResult.recertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.RecertifyResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_recertify",
        "kmipkit.client.BatchItemResult.recertify_response",
        "kmipkit.client.RecertifyResponse",
        "kmipkit.client.RecertifyResponse.unique_identifier",
        "kmipkit.client.RecertifyResponse.response_ttlv",
        "kmipkit.client.RecertifyResponse.close",
    },
}


class RecertifyParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_recertify_has_approved_four_language_mapping(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Re-certify", operation["name"])
        self.assertEqual("6.1.45", operation["section"])
        self.assertEqual("client_1_0", operation["scope"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0036", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0036", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))

        actual_mappings = {language: set() for language in LANGUAGES}
        linked_requirements: set[str] = set()
        for language, declaration_ids in operation["declarations"].items():
            self.assertTrue(declaration_ids, f"Re-certify has no {language} declarations")
            for declaration_id in declaration_ids:
                declaration = self.declarations[declaration_id]
                self.assertIn(declaration["capabilityId"], {OPERATION_ID, "client_execution"})
                self.assertEqual("verified", declaration["declarationState"])
                self.assertTrue(PARITY_EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(OWNER_SPEC, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                actual_mappings[language].add(declaration["languageMappings"][language]["name"])
                linked_requirements.update(declaration["requirementIds"])

        missing = {
            language: sorted(expected - actual_mappings[language])
            for language, expected in EXPECTED_MAPPINGS.items()
            if expected - actual_mappings[language]
        }
        self.assertEqual({}, missing)
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_requirements)
        self.assertFalse(RETIRED_REQUIREMENTS & linked_requirements)
        self.assertTrue(ACTIVE_REQUEST_REQUIREMENTS <= linked_requirements)

    def test_conditional_identifier_requirement_is_request_only_and_retired_ids_are_unassigned(self) -> None:
        active = "KMIPKIT-REQ-SPEC-6.1.45-002-003"
        self.assertIn(active, self.manifest["requirementIds"])
        self.assertTrue(RETIRED_REQUIREMENTS <= set(self.manifest["requirementIds"]))

        operation_declarations = [
            row for row in self.manifest["publicDeclarations"]
            if row.get("capabilityId") == OPERATION_ID
        ]
        declarations_by_id = {row["declarationId"]: row for row in operation_declarations}
        self.assertEqual(43, len(operation_declarations))
        self.assertEqual(set(declarations_by_id), REQUEST_DECLARATIONS | NON_REQUEST_DECLARATIONS)
        active_declarations = {
            row["declarationId"] for row in operation_declarations
            if active in row.get("requirementIds", [])
        }
        self.assertEqual(REQUEST_REQUIREMENT_DECLARATIONS, active_declarations)
        self.assertFalse(any(ACTIVE_REQUEST_REQUIREMENTS & set(declarations_by_id[i].get("requirementIds", []))
                             for i in NON_REQUEST_DECLARATIONS))
        self.assertFalse(any(RETIRED_REQUIREMENTS & set(row.get("requirementIds", []))
                             for row in operation_declarations))
        self.assertTrue(RETIRED_REQUIREMENT_DECLARATIONS <= {
            row["declarationId"] for row in operation_declarations
        })

        functions = {
            row["id"]: row for row in self.manifest["functions"]
            if row["id"].startswith("client_batch_item_")
            and "recertify" in row["id"]
        }
        self.assertEqual(
            {"client_batch_item_recertify"},
            {function_id for function_id, row in functions.items()
             if active in row.get("requirementIds", [])},
        )
        for function_id in RESPONSE_FUNCTIONS:
            self.assertFalse(ACTIVE_REQUEST_REQUIREMENTS & set(functions[function_id]["requirementIds"]))
        self.assertFalse(any(RETIRED_REQUIREMENTS & set(row.get("requirementIds", []))
                             for row in functions.values()))

    def test_recertify_jni_bridge_is_an_exact_approved_exclusion(self) -> None:
        bridge_names = {
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemRecertify",
            "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseRecertify",
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit" and row["exportName"] in bridge_names
        }
        self.assertEqual(bridge_names, set(exclusions))
        for export_name, exclusion in exclusions.items():
            with self.subTest(export=export_name):
                self.assertIn("raw-handle JNI bridge", exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-008", exclusion["evidence"])
                self.assertIn("T058", exclusion["evidence"])

    def test_recertify_traceability_links_normative_and_api_requirements(self) -> None:
        rows = [line for line in TRACEABILITY.read_text(encoding="utf-8").splitlines()
                if line.startswith("| T058 |")]
        self.assertEqual(1, len(rows))
        linked_ids = set(re.findall(
            r"KMIPKIT-(?:REQ-SPEC-6\.1\.45-[0-9-]+|0036-FR-\d{3}|0043-FR-\d{3})",
            rows[0],
        ))
        self.assertTrue(OWNER_REQUIREMENTS | API_REQUIREMENTS <= linked_ids)
        self.assertFalse(RETIRED_REQUIREMENTS & linked_ids)

    def test_generated_parity_inventory_contains_recertify_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [row for row in generated["operations"] if row["operationId"] == OPERATION_ID]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))


if __name__ == "__main__":
    unittest.main()
