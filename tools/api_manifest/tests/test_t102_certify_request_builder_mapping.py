"""Pin exactly four optional Certify request-builder mappings for T102."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.tests.t102_manifest_totals import (
    T102_APPROVED_EXCLUSIONS,
    T102_BLOCKED_CANDIDATES,
    T102_PUBLIC_DECLARATIONS,
    without_0047_versioned_values,
)

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OASIS_HTML = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
OWNER = "specs/022-object-lifecycle/spec.md"
OPERATION = "KMIPKIT-ELEM-OP-C2S-CERTIFY"
C_BUILDER = "kmipkit_client_batch_item_certify"
JAVA_BUILDER = "org.kmipkit.extensions.ClientBatchItem.certify"
PYTHON_BUILDER = "kmipkit.extensions.client_batch_item_certify"
LIFECYCLE_SERIALIZER_IDS = {
    "KMIPKIT-ELEM-OP-C2S-CERTIFY": "rust-observed-455bf70fd49c2d1c0584",
    "KMIPKIT-ELEM-OP-C2S-CHECK": "rust-observed-1aa8855f67251843003b",
    "KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE": "rust-observed-3f7281ffc6f28316cdb2",
    "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY": "rust-observed-d7bf84b1a5fd4c705597",
    "KMIPKIT-ELEM-OP-C2S-RE-KEY": "rust-observed-1477c49c457fdc10fa5b",
    "KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR": "rust-observed-92442b7d3422ffe465c3",
    "KMIPKIT-ELEM-OP-C2S-RE-PROVISION": "rust-observed-d1446684e4f2641b3c96",
    "KMIPKIT-ELEM-OP-C2S-REVOKE": "rust-observed-3110caf46cc1e36cbcc9",
}
EVIDENCE = {
    "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs",
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_certify_operation.py",
    "tools/api_manifest/tests/test_t102_certify_request_builder_mapping.py",
}
OPERATION_ADDED_EVIDENCE = {
    "crates/kmipkit-ffi/tests/c_api_coverage.rs",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "tools/api_manifest/tests/test_t102_certify_request_builder_mapping.py",
    "tools/api_manifest/tests/test_t102_certify_request_identifier_member_mapping.py",
    "tools/api_manifest/tests/test_t102_lifecycle_request_serializer_mapping.py",
}
CERTIFY_REQUEST_IDENTIFIER_ID = "rust-observed-d0317408cb64ea049eaf"
PKCS11_REQUEST_EXTENSIONS_C_DECLARATION_ID = "T102:pkcs11-request-extensions:c"
RAW_CERTIFY_DECLARATIONS = {
    "rust-observed-e0a06a71290bee987cc7",
    "rust-observed-5722377217051635233d",
}
RAW_RECERTIFY_DECLARATIONS = {
    "rust-observed-bd90b6e53d6c05c1a6b0",
    "rust-observed-4c37b205dc5bee05df9a",
}
WRAPPING_REQUEST_BUILDER_IDS = {
    "rust-observed-9cd4c6b6fd190aff211f",
    "rust-observed-506281d0cf8e3c41a90e",
    "rust-observed-61aecad439c9a84c7241",
    "rust-observed-ba1772cec998d92e5148",
}
PKCS11_REQUEST_SETTER_IDS = {
    "rust-observed-60273eccedf62e75a65e",
    "rust-observed-b1e7666a5a57f054a891",
    "rust-observed-37ff26bed9cff3029512",
    "rust-observed-837fc199aac73b1ffd00",
    "rust-observed-cc71aab290dc65b5928f",
    "rust-observed-573ae1d0de80ffa2884b",
}
PKCS11_REQUEST_EXTENSION_IDS = {
    "rust-observed-ad5c5204337d486b06bb",
    "rust-observed-358ceac77d04d65b6238",
}
RECERTIFY_TYPE_ID = "rust-observed-4e379cb3b97bca658dbf"
RAW_CERTIFY_EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_certify_operation.py",
}
TARGETS = (
    {
        "id": "rust-observed-ff892d7f1e1585911b9c",
        "name": "kmipkit_protocol::CertifyRequest::with_unique_identifier",
        "field": "Unique Identifier",
        "tag": 0x420094,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-002-001", "KMIPKIT-REQ-SPEC-6.1.6-002-002"},
    },
    {
        "id": "rust-observed-25fb2f73d24e03d2ab13",
        "name": "kmipkit_protocol::CertifyRequest::with_certificate_request_type",
        "field": "Certificate Request Type",
        "tag": 0x420019,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-007"},
    },
    {
        "id": "rust-observed-75b1748bcbc69c385a66",
        "name": "kmipkit_protocol::CertifyRequest::with_attributes",
        "field": "Attributes",
        "tag": 0x420125,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-002-003"},
    },
    {
        "id": "rust-observed-ddcac6317bf0ae830eab",
        "name": "kmipkit_protocol::CertifyRequest::with_protection_storage_masks",
        "field": "Protection Storage Masks",
        "tag": 0x42015F,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-008"},
        "requirements": set(),
    },
    {
        "id": "rust-observed-3620d8e9bb89e3bbe251",
        "name": "kmipkit_protocol::CertifyRequest::with_certificate_request_value",
        "field": "Certificate Request Value",
        "tag": 0x420140,
        "fr": {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-002", "KMIPKIT-0036-FR-004", "KMIPKIT-0036-FR-005", "KMIPKIT-0036-FR-006", "KMIPKIT-0036-FR-008"},
        "requirements": {"KMIPKIT-REQ-SPEC-6.1.6-007"},
    },
)
REQUEST_CONSTRUCTION_TARGETS = (
    {
        "id": "rust-observed-f928aa3b375fa686e506",
        "name": "kmipkit_protocol::CertifyRequest",
        "kind": "type",
    },
    {
        "id": "rust-observed-9a4cd073497d09724f7e",
        "name": "kmipkit_protocol::CertifyRequest::new",
        "kind": "associated-item",
    },
)
REQUEST_CONSTRUCTION_REQUIREMENTS = {
    "KMIPKIT-0036-FR-001",
    "KMIPKIT-0036-FR-002",
    "KMIPKIT-0036-FR-004",
    "KMIPKIT-0036-FR-006",
    "KMIPKIT-0036-FR-008",
    "KMIPKIT-REQ-SPEC-6.1.6-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-001",
    "KMIPKIT-REQ-SPEC-6.1.6-002-002",
    "KMIPKIT-REQ-SPEC-6.1.6-002-003",
    "KMIPKIT-REQ-SPEC-6.1.6-003",
    "KMIPKIT-REQ-SPEC-6.1.6-007",
}
BLOCKED_BEFORE = 224
BLOCKED_AFTER = 221
# The non-target snapshot includes subsequent approved T102 manifest slices.
# Includes the reviewed 0047 release stack and complete 0048/0049 inventories.
# Snapshot includes final 0049 and exact 0050 constructor mappings.
# The snapshot includes the approved KMIPKIT-0052 async typed-reader routes.
# The snapshot includes the final KMIPKIT-0053 verified operation surfaces.
# The snapshot includes approved KMIPKIT-0099 Create Split Key and KMIPKIT-0100
# Register declarations and KMIPKIT-0101 lifecycle declarations.
# Includes the approved KMIPKIT-0093 inventory reconciliation.
DECLARATIONS_SHA256 = "7bf25865777a84305bb9bf4ca256649dfae8c5433afc15a921e339b5740f3d20"
OPERATIONS_SHA256 = "2a6eb9fb6823b3456b3fb10b6d4af8477be9f1a1f68c06c252bb4873b272f3bc"


def canonical_sha256(value: object) -> str:
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


class T102CertifyRequestBuilderMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.declarations = {row["declarationId"]: row for row in cls.manifest["publicDeclarations"]}
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.elements = {row["element_id"]: row for row in cls.catalog["elements"]}

    def test_exact_certify_request_setters_map_to_existing_builders(self) -> None:
        self.assertEqual(5, len(TARGETS))
        for target in TARGETS:
            with self.subTest(declaration=target["id"], field=target["field"]):
                declaration = self.declarations[target["id"]]
                self.assertEqual(target["name"], declaration["exportName"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual(OPERATION, declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual(target["fr"] | target["requirements"], set(declaration["requirementIds"]))
                self.assertEqual(
                    {
                        "rust": {"packageId": "rust-kmipkit-protocol", "name": target["name"]},
                        "c": {"packageId": "c-kmipkit", "name": C_BUILDER},
                        "java": {"packageId": "java-kmipkit", "name": JAVA_BUILDER},
                        "python": {"packageId": "python-kmipkit", "name": PYTHON_BUILDER},
                    },
                    declaration["languageMappings"],
                )
                self.assertTrue(EVIDENCE <= set(declaration["parityEvidenceIds"]))
                tag_id = f"KMIPKIT-ELEM-TAG-{target['tag']:06X}"
                self.assertIn(tag_id, self.elements)
                self.assertEqual("assigned", self.elements[tag_id]["allocation"])
                self.assertEqual("client_1_0", self.elements[tag_id]["scope_state"])
                self.assertIn(target["id"], self.operation["declarations"]["rust"])

    def test_candidate_total_reflects_only_audited_certify_mappings(self) -> None:
        target_ids = {target["id"] for target in TARGETS}
        self.assertEqual(5, len(target_ids))
        blocked = sum(
            row.get("candidateSpecId") == "KMIPKIT-0043" and row["declarationState"] == "blocked"
            for row in self.manifest["publicDeclarations"]
        )
        self.assertEqual(BLOCKED_BEFORE - 3, BLOCKED_AFTER)
        self.assertEqual(4, len(WRAPPING_REQUEST_BUILDER_IDS))
        self.assertEqual(T102_BLOCKED_CANDIDATES, blocked)
        self.assertEqual(
            "verified",
            self.declarations[RECERTIFY_TYPE_ID]["declarationState"],
        )
        self.assertEqual(T102_PUBLIC_DECLARATIONS, len(self.manifest["publicDeclarations"]))
        self.assertEqual(T102_APPROVED_EXCLUSIONS, len(self.manifest["approvedExclusions"]))

    def test_certify_request_type_and_constructor_map_to_the_four_language_builder(self) -> None:
        expected_mappings = {
            "rust": {"packageId": "rust-kmipkit-protocol"},
            "c": {"packageId": "c-kmipkit", "name": C_BUILDER},
            "java": {"packageId": "java-kmipkit", "name": JAVA_BUILDER},
            "python": {"packageId": "python-kmipkit", "name": PYTHON_BUILDER},
        }
        operation_declarations = self.operation["declarations"]["rust"]
        self.assertEqual(2, len(REQUEST_CONSTRUCTION_TARGETS))
        for target in REQUEST_CONSTRUCTION_TARGETS:
            expected_mappings["rust"]["name"] = target["name"]
            declaration = self.declarations[target["id"]]
            with self.subTest(export=target["name"]):
                self.assertEqual(target["name"], declaration["exportName"])
                self.assertEqual(target["kind"], declaration["kind"])
                self.assertEqual("verified", declaration["declarationState"])
                self.assertNotIn("candidateSpecId", declaration)
                self.assertNotIn("blockedReason", declaration)
                self.assertEqual(OPERATION, declaration["capabilityId"])
                self.assertEqual("four-language", declaration["parityScope"])
                self.assertEqual(REQUEST_CONSTRUCTION_REQUIREMENTS, set(declaration["requirementIds"]))
                self.assertEqual(OWNER, declaration["approvalSource"]["path"])
                self.assertEqual("Approved", declaration["approvalSource"]["status"])
                self.assertEqual(expected_mappings, declaration["languageMappings"])
                self.assertTrue(EVIDENCE <= set(declaration["parityEvidenceIds"]))
                self.assertIn(target["id"], operation_declarations)

    def test_certificate_request_value_mapping_preserves_secret_input_and_wire_evidence(self) -> None:
        rust = (ROOT / "crates/kmipkit-protocol/src/certify.rs").read_text(encoding="utf-8")
        c_header = (ROOT / "bindings/c/include/kmipkit.h").read_text(encoding="utf-8")
        c_ffi = (ROOT / "crates/kmipkit-ffi/src/extension_registry.rs").read_text(encoding="utf-8")
        java = (ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings/python/src/kmipkit/extensions.py").read_text(encoding="utf-8")
        self.assertIn("with_certificate_request_value(mut self, value: SecretBytes)", rust)
        self.assertIn("kmipkit_byte_span_v1_t * certificate_request_value", c_header)
        self.assertIn("copy_optional_secret_span(certificate_request_value)", c_ffi)
        certify_java = java.split("public static ClientBatchItem certify(", 1)[1].split("/** Creates a Re-certify", 1)[0]
        self.assertIn("byte[] certificateRequestValue", certify_java)
        self.assertIn("copyForNative(certificateRequestValue)", certify_java)
        self.assertIn("clearNativeCopies(identifierText, requestValueCopy)", certify_java)
        certify_python = python.split("def client_batch_item_certify(", 1)[1].split("def client_batch_item_recertify(", 1)[0]
        self.assertIn("certificate_request_value: bytes | bytearray | memoryview | None", certify_python)
        self.assertIn("_native_optional_span(\n            certificate_request_value, staging_buffers", certify_python)

        c_consumer = (ROOT / "bindings/c/tests/client_abi_consumer.c").read_text(encoding="utf-8")
        java_consumer = (ROOT / "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java").read_text(encoding="utf-8")
        python_consumer = (ROOT / "bindings/python/tests/test_certify_operation.py").read_text(encoding="utf-8")
        self.assertIn("0x420140", python_consumer)
        self.assertIn("0x420140", java_consumer)
        self.assertIn("0x0042_0140", (ROOT / "crates/kmipkit-ffi/tests/c_api_coverage.rs").read_text(encoding="utf-8"))
        self.assertIn("&certify_request_value", c_consumer)

    def test_oasis_table_179_fields_and_child_mask_tag_are_pinned(self) -> None:
        raw_html = OASIS_HTML.read_bytes()
        self.assertEqual(
            "8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF",
            hashlib.sha256(raw_html).hexdigest().upper(),
        )
        text = " ".join(re.sub(r"<[^>]+>", " ", raw_html.decode("cp1252")).split())
        caption = text.index("Table 179: Certify Request Payload")
        header = text.rfind("Request Payload Item REQUIRED Description", 0, caption)
        payload = text[header:caption]
        for field in (
            "Unique Identifier", "Certificate Request Type", "Certificate Request Value",
            "Attributes", "Protection Storage Masks",
        ):
            with self.subTest(field=field):
                self.assertRegex(payload, rf"{re.escape(field)}\s+No\b")
        self.assertIn(
            "It is REQUIRED if the Certificate Request is present.",
            payload,
        )
        self.assertNotRegex(
            payload,
            r"Certificate Request Type[^.]*REQUIRED if the Certificate Request Value is absent",
        )
        self.assertIn("Table 117: Protection Storage Mask", text)
        self.assertIn("Protection Storage Mask Integer", text)
        self.assertEqual("Protection Storage Masks", self.elements["KMIPKIT-ELEM-TAG-42015F"]["name"])
        self.assertEqual("Protection Storage Mask", self.elements["KMIPKIT-ELEM-TAG-42015E"]["name"])

    def test_raw_certify_discriminators_have_direct_three_language_metadata_evidence(self) -> None:
        self.assertEqual(2, len(RAW_CERTIFY_DECLARATIONS))
        for declaration_id in RAW_CERTIFY_DECLARATIONS:
            declaration = self.declarations[declaration_id]
            self.assertIn("ClientOperation::Certify", declaration["exportName"])
            self.assertTrue(RAW_CERTIFY_EVIDENCE <= set(declaration["parityEvidenceIds"]))

        proof_sources = {
            "bindings/c/tests/client_abi_consumer.c": "metadata.operation != UINT32_C(6)",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java": "metadataV2().operation()",
            "bindings/python/tests/test_certify_operation.py":
                "[item.metadata_v2.operation for item in result.items] == [6, 6, 6, 6]",
        }
        for evidence_path, proof in proof_sources.items():
            with self.subTest(evidence=evidence_path):
                source = (ROOT / evidence_path).read_text(encoding="utf-8")
                self.assertTrue(proof in source, evidence_path)

        manifest_v2 = (ROOT / "tools/api_manifest/tests/test_manifest_v2.py").read_text(encoding="utf-8")
        raw_operation_block = manifest_v2.split("raw_operation_evidence = {", 1)[1].split("}", 1)[0]
        for raw_metadata_source in (
            "bindings/c/tests/client_abi_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/SharedCapabilityParityFixtureTest.java",
            "bindings/python/tests/test_client.py",
            "crates/kmipkit-client/tests/creation_parity_tests.rs",
        ):
            self.assertIn(raw_metadata_source, raw_operation_block)
        for builder_only_path in (
            "crates/kmipkit-ffi/tests/c_api_coverage.rs",
            "crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs",
            "tools/api_manifest/tests/test_t102_certify_request_builder_mapping.py",
        ):
            self.assertNotIn(builder_only_path, raw_operation_block)

    def test_type_without_value_is_exercised_as_an_exact_wire_case_in_all_bindings(self) -> None:
        sources = {
            "c": ROOT / "bindings/c/tests/client_abi_consumer.c",
            "java": ROOT / "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "python": ROOT / "bindings/python/tests/test_certify_operation.py",
        }
        for language, path in sources.items():
            text = path.read_text(encoding="utf-8")
            with self.subTest(language=language):
                self.assertTrue("certify-type-only" in text, f"{language} type-only wire case")
        c_source = sources["c"].read_text(encoding="utf-8")
        self.assertTrue("&identifiers[14], 1, 1, 0, 0, 0, &item" in c_source)
        java_source = sources["java"].read_text(encoding="utf-8")
        self.assertTrue("enumeration(0x420019, 1)" in java_source)
        python_source = sources["python"].read_text(encoding="utf-8")
        self.assertTrue("ttlv_enumeration(0x420019, 1)" in python_source)
        ffi_wire = (ROOT / "crates/kmipkit-ffi/tests/c_api_coverage.rs").read_text(encoding="utf-8")
        start = ffi_wire.index("const CERTIFY_TYPE_ONLY: &[RequestBuilderWireField]")
        end = ffi_wire.index("];", start)
        expected_fields = ffi_wire[start:end]
        self.assertTrue("0x0042_0019" in expected_fields and "V::Enumeration(1)" in expected_fields)
        for omitted_tag in ("0x0042_0140", "0x0042_0125", "0x0042_015f"):
            self.assertTrue(omitted_tag not in expected_fields, omitted_tag)
        self.assertTrue("fields: CERTIFY_TYPE_ONLY" in ffi_wire)

    def test_non_target_declarations_and_operation_rows_match_snapshot(self) -> None:
        target_ids = {
            target["id"] for target in TARGETS
        } | {
            CERTIFY_REQUEST_IDENTIFIER_ID,
            PKCS11_REQUEST_EXTENSIONS_C_DECLARATION_ID,
        } | PKCS11_REQUEST_EXTENSION_IDS | {
            target["id"] for target in REQUEST_CONSTRUCTION_TARGETS
        }
        other_declarations = copy.deepcopy([
            row for row in self.manifest["publicDeclarations"] if row["declarationId"] not in target_ids
        ])
        for declaration in other_declarations:
            if declaration["declarationId"] in LIFECYCLE_SERIALIZER_IDS.values():
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration["blockedReason"] = (
                    "The public Rust declaration has no resolved owner in the approved dependency inventory; "
                    "assign an owner before advertising it."
                )
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in RAW_CERTIFY_DECLARATIONS | RAW_RECERTIFY_DECLARATIONS:
                declaration["parityEvidenceIds"].remove(
                    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
                )
            if declaration["declarationId"] == RECERTIFY_TYPE_ID:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration["blockedReason"] = (
                    "The public Rust declaration has no resolved owner in the approved dependency inventory; "
                    "assign an owner before advertising it."
                )
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] == "rust-observed-be7b44dc6c8bae24ad53":
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration["blockedReason"] = (
                    "The public Rust declaration has no resolved owner in the approved dependency inventory; "
                    "assign an owner before advertising it."
                )
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in WRAPPING_REQUEST_BUILDER_IDS:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration["blockedReason"] = (
                    "The public Rust declaration has no resolved owner in the approved dependency inventory; "
                    "assign an owner before advertising it."
                )
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
            if declaration["declarationId"] in PKCS11_REQUEST_SETTER_IDS | PKCS11_REQUEST_EXTENSION_IDS:
                declaration["capabilityId"] = "public_binding_surface"
                declaration["requirementIds"] = ["KMIPKIT-0043-FR-021"]
                declaration["declarationState"] = "blocked"
                declaration["candidateSpecId"] = "KMIPKIT-0043"
                declaration["blockedReason"] = (
                    "The public Rust declaration has no resolved owner in the approved dependency inventory; "
                    "assign an owner before advertising it."
                )
                declaration.pop("approvalSource", None)
                declaration.pop("languageMappings", None)
                declaration.pop("parityEvidenceIds", None)
        # Final T102 route resolution and predecessor stack additions remain
        # inside this exact non-target fingerprint.
        for capability, count in (
            ("typed_object_attributes", 544),
            ("typed_managed_objects_and_key_material", 762),
        ):
            with self.subTest(stack_capability=capability):
                typed_rows = [
                    row for row in other_declarations
                    if row["capabilityId"] == capability
                ]
                self.assertEqual(count, len(typed_rows))
        other_declarations = without_0047_versioned_values(other_declarations)
        self.assertEqual(DECLARATIONS_SHA256, canonical_sha256(other_declarations))
        other_operations = copy.deepcopy(self.manifest["operations"])
        for operation in other_operations:
            if operation["operationId"] in LIFECYCLE_SERIALIZER_IDS:
                serializer_id = LIFECYCLE_SERIALIZER_IDS[operation["operationId"]]
                operation["declarations"]["rust"] = [
                    declaration_id
                    for declaration_id in operation["declarations"]["rust"]
                    if declaration_id != serializer_id
                ]
                operation["parityEvidenceIds"] = [
                    evidence_id
                    for evidence_id in operation["parityEvidenceIds"]
                    if evidence_id
                    != "tools/api_manifest/tests/test_t102_lifecycle_request_serializer_mapping.py"
                ]
            if operation["operationId"] == OPERATION:
                operation["declarations"]["rust"] = [
                    declaration_id for declaration_id in operation["declarations"]["rust"]
                    if declaration_id not in target_ids | {"rust-observed-455bf70fd49c2d1c0584"}
                ]
                operation["parityEvidenceIds"] = [
                    evidence_id for evidence_id in operation["parityEvidenceIds"]
                    if evidence_id not in OPERATION_ADDED_EVIDENCE
                ]
            if operation["operationId"] == "KMIPKIT-ELEM-OP-C2S-RE-CERTIFY":
                operation["declarations"]["rust"] = [
                    declaration_id for declaration_id in operation["declarations"]["rust"]
                    if declaration_id not in {RECERTIFY_TYPE_ID, "rust-observed-be7b44dc6c8bae24ad53"}
                ]
                operation["parityEvidenceIds"] = [
                    evidence_id for evidence_id in operation["parityEvidenceIds"]
                    if evidence_id != "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java"
                ]
            if operation["operationId"] in {
                "KMIPKIT-ELEM-OP-C2S-IMPORT",
                "KMIPKIT-ELEM-OP-C2S-EXPORT",
                "KMIPKIT-ELEM-OP-C2S-GET",
            }:
                operation["declarations"]["rust"] = [
                    declaration_id
                    for declaration_id in operation["declarations"]["rust"]
                    if declaration_id not in WRAPPING_REQUEST_BUILDER_IDS
                ]
            if operation["operationId"] == "KMIPKIT-ELEM-OP-C2S-PKCS-11":
                operation["declarations"]["rust"] = [
                    declaration_id
                    for declaration_id in operation["declarations"]["rust"]
                    if declaration_id not in PKCS11_REQUEST_SETTER_IDS | PKCS11_REQUEST_EXTENSION_IDS
                ]
                operation["declarations"]["c"] = [
                    declaration_id
                    for declaration_id in operation["declarations"]["c"]
                    if declaration_id != PKCS11_REQUEST_EXTENSIONS_C_DECLARATION_ID
                ]
        self.assertEqual(OPERATIONS_SHA256, canonical_sha256(other_operations))

    @property
    def operation(self) -> dict[str, object]:
        return next(row for row in self.manifest["operations"] if row["operationId"] == OPERATION)


if __name__ == "__main__":
    unittest.main()
