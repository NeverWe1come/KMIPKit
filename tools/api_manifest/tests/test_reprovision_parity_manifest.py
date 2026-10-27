"""Manifest acceptance tests for the KMIP 2.1 §6.1.48 Re-Provision mapping."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "specs/022-object-lifecycle/spec.md"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-RE-PROVISION"
LANGUAGES = {"rust", "c", "java", "python"}
OWNER_REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.48-001",
    "KMIPKIT-REQ-SPEC-6.1.48-004",
    "KMIPKIT-REQ-SPEC-6.1.48-006",
}
API_REQUIREMENTS = {
    f"KMIPKIT-0043-FR-{index:03}"
    for index in (4, 5, 7, 8, 9, 10, 11, 12, 13, 17, 18, 21, 22)
}
PARITY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/java/src/test/java/org/kmipkit/extensions/ReProvisionMappingTest.java",
    "bindings/python/tests/test_reprovision_native_integration.py",
    "bindings/python/tests/test_reprovision_operation.py",
    "crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "crates/kmipkit-protocol/tests/unit/reprovision_operation_tests.rs",
    "tools/api_manifest/tests/test_reprovision_parity_manifest.py",
}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::Client::reprovision",
        "kmipkit_client::ClientRequest::ReProvision",
        "kmipkit_client::ClientBatchOutcome::reprovision_response",
        "kmipkit_protocol::ReProvisionRequest",
        "kmipkit_protocol::ReProvisionRequest::new",
        "kmipkit_protocol::ReProvisionRequest::with_certificate_request",
        "kmipkit_protocol::ReProvisionRequest::with_certificate",
        "kmipkit_protocol::ReProvisionResponse",
        "kmipkit_protocol::ReProvisionResponse::unique_identifier",
        "kmipkit_protocol::ReProvisionResponse::with_ttlv",
    },
    "c": {
        "kmipkit_client_batch_item_reprovision",
        "kmipkit_client_batch_item_response_reprovision",
        "kmipkit_client_batch_item_response_reprovision_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.reprovision",
        "org.kmipkit.extensions.ClientBatchItemResult.reprovisionResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ReProvisionResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ReProvisionResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.ReProvisionResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.ReProvisionResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_reprovision",
        "kmipkit.client.BatchItemResult.reprovision_response",
        "kmipkit.client.ReProvisionResponse",
        "kmipkit.client.ReProvisionResponse.unique_identifier",
        "kmipkit.client.ReProvisionResponse.response_ttlv",
        "kmipkit.client.ReProvisionResponse.close",
    },
}


class ReProvisionParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_operation_has_approved_four_language_mapping_and_traceability(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Re-Provision", operation["name"])
        self.assertEqual("6.1.48", operation["section"])
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
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_declarations = {
            item for ids in operation["declarations"].values() for item in ids
        }
        self.assertTrue(linked_declarations)
        assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=OWNER_REQUIREMENTS | API_REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
            required_parity_evidence=PARITY_EVIDENCE,
        )

    def test_manifest_declares_both_mutually_exclusive_objects_and_optional_identifier(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        constructor = functions["client_batch_item_reprovision"]
        expected_request_fields = ["certificate_request", "certificate"]
        self.assertEqual(
            expected_request_fields,
            [item["name"] for item in constructor["python"]["parameters"]],
        )
        self.assertEqual(
            ["certificateRequest", "certificate"],
            [item["name"] for item in constructor["java"]["parameters"]],
        )
        self.assertIn(
            "copied before the constructor returns",
            constructor["c"]["documentation"][1],
        )

        response = functions["client_batch_item_response_reprovision"]
        self.assertIn("out_identifier_present", [item["name"] for item in response["c"]["parameters"]])
        self.assertIn("optional", " ".join(response["c"]["documentation"]).lower())
        self.assertIn("Table 314", " ".join(response["c"]["documentation"]))
        typed = functions["client_batch_item_response_reprovision_ttlv"]
        self.assertIn("caller-owned copy", typed["c"]["documentation"][0])

        owner_text = (ROOT / OWNER_SPEC).read_text(encoding="utf-8")
        self.assertIn("KMIPKIT-DEC-014", owner_text)
        self.assertIn("allow absence for all three forms", owner_text.lower())

    def test_response_jni_mappings_match_the_declared_native_export(self) -> None:
        native_method = "clientBatchItemResponseReprovision"
        native_symbol = f"Java_org_kmipkit_NativeExtensionRegistry_{native_method}"
        java_source = (
            ROOT / "bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java"
        ).read_text(encoding="utf-8")
        cpp_source = (ROOT / "bindings/java/native/kmipkit_jni.cpp").read_text(
            encoding="utf-8"
        )
        self.assertRegex(
            java_source,
            rf"public static native Object\[\] {native_method}\(",
        )
        self.assertIn(f"JNIEXPORT jobjectArray JNICALL {native_symbol}(", cpp_source)

        functions = {row["id"]: row for row in self.manifest["functions"]}
        self.assertEqual(
            native_symbol,
            functions["client_batch_item_response_reprovision"]["java"]["jniSymbol"],
        )
        self.assertEqual(
            native_symbol,
            functions["client_batch_item_response_reprovision_ttlv"]["java"][
                "jniAggregateSymbol"
            ],
        )


if __name__ == "__main__":
    unittest.main()
