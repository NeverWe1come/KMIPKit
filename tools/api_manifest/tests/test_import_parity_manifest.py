"""Manifest regressions for the T050 Import mapping."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.api_manifest.tests.parity_support import assert_declaration_mappings

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
GENERATED_PARITY = ROOT / "tests" / "fixtures" / "api" / "generated" / "public_api_parity.json"
OPERATION_ID = "KMIPKIT-ELEM-OP-C2S-IMPORT"
OWNER_SPEC = "specs/023-key-material-transfer/spec.md"
LANGUAGES = {"rust", "c", "java", "python"}
EXPECTED_MAPPINGS = {
    "rust": {
        "kmipkit_client::ClientRequest::Import",
        "kmipkit_client::ClientBatchOutcome::Import",
        "kmipkit_protocol::ImportRequest",
        "kmipkit_protocol::ImportRequest::new",
        "kmipkit_protocol::ImportResponse",
        "kmipkit_protocol::ImportResponse::with_ttlv",
        "kmipkit_protocol::ImportResponse::unique_identifier",
    },
    "c": {
        "kmipkit_client_batch_item_import",
        "kmipkit_client_batch_item_response_import",
        "kmipkit_client_batch_item_response_import_ttlv",
    },
    "java": {
        "org.kmipkit.extensions.ClientBatchItem.importObject",
        "org.kmipkit.extensions.ClientBatchItemResult.importResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ImportResponse",
        "org.kmipkit.extensions.ClientBatchItemResult.ImportResponse.uniqueIdentifier",
        "org.kmipkit.extensions.ClientBatchItemResult.ImportResponse.responseTtlv",
        "org.kmipkit.extensions.ClientBatchItemResult.ImportResponse.close",
    },
    "python": {
        "kmipkit.extensions.client_batch_item_import",
        "kmipkit.client.BatchItemResult.import_response",
        "kmipkit.client.ImportResponse",
        "kmipkit.client.ImportResponse.unique_identifier",
        "kmipkit.client.ImportResponse.response_ttlv",
        "kmipkit.client.ImportResponse.close",
    },
}
REQUIREMENTS = {
    "KMIPKIT-REQ-SPEC-6.1.25-001",
    "KMIPKIT-REQ-SPEC-6.1.25-004",
    "KMIPKIT-REQ-SPEC-6.1.25-006",
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-007",
    "KMIPKIT-0043-FR-008",
    "KMIPKIT-0043-FR-009",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-011",
    "KMIPKIT-0043-FR-012",
    "KMIPKIT-0043-FR-013",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-018",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/import_execution_tests.rs",
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ImportResponseMappingTest.java",
    "bindings/python/tests/test_import_operation.py",
}


class ImportParityManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.operations = [
            row for row in self.manifest["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.declarations = {
            row["declarationId"]: row for row in self.manifest["publicDeclarations"]
        }

    def test_import_has_approved_four_language_mapping_and_evidence(self) -> None:
        self.assertEqual(1, len(self.operations))
        operation = self.operations[0]
        self.assertEqual("Import", operation["name"])
        self.assertEqual("6.1.25", operation["section"])
        self.assertEqual("verified", operation["apiState"])
        self.assertEqual("KMIPKIT-0037", operation["featureSpecId"])
        self.assertEqual("approved", operation["ownerDependency"]["state"])
        self.assertEqual("KMIPKIT-0037", operation["ownerDependency"]["specId"])
        self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
        self.assertEqual("Approved", operation["approvalEvidence"]["status"])
        self.assertNotIn("blockedReason", operation)
        self.assertEqual(LANGUAGES, set(operation["declarations"]))
        self.assertTrue(PARITY_EVIDENCE <= set(operation["parityEvidenceIds"]))
        for evidence in PARITY_EVIDENCE:
            self.assertTrue((ROOT / evidence).is_file(), evidence)

        linked_requirements: set[str] = set()
        actual_mappings = {language: set() for language in LANGUAGES}
        actual_mappings, linked_requirements = assert_declaration_mappings(
            self,
            operation,
            self.declarations,
            operation_id=OPERATION_ID,
            languages=LANGUAGES,
            expected_mappings=EXPECTED_MAPPINGS,
            required_requirements=REQUIREMENTS,
            expected_approval_source=OWNER_SPEC,
        )
        for language, expected in EXPECTED_MAPPINGS.items():
            self.assertTrue(expected <= actual_mappings[language], language)
        self.assertTrue(REQUIREMENTS <= linked_requirements)

    def test_generated_parity_inventory_contains_verified_import_mapping(self) -> None:
        generated = json.loads(GENERATED_PARITY.read_text(encoding="utf-8"))
        operations = [
            row for row in generated["operations"] if row["operationId"] == OPERATION_ID
        ]
        self.assertEqual(1, len(operations))
        self.assertEqual("verified", operations[0]["apiState"])
        self.assertEqual(LANGUAGES, set(operations[0]["declarations"]))

    def test_import_generic_ttlv_builders_map_to_approved_shared_model(self) -> None:
        expected_mappings = {
            "ttlv_value_byte_string:c": {
                "rust": "kmipkit_ttlv::Value::byte_string",
                "c": "kmipkit_ttlv_value_byte_string",
                "java": "org.kmipkit.ttlv.TtlvValue.ByteString",
                "python": "kmipkit.ttlv.ttlv_value_byte_string",
            },
            "ttlv_value_text_string:c": {
                "rust": "kmipkit_ttlv::Value::text_string",
                "c": "kmipkit_ttlv_value_text_string",
                "java": "org.kmipkit.ttlv.TtlvValue.TextString",
                "python": "kmipkit.ttlv.ttlv_value_text_string",
            },
            "ttlv_value_structure:c": {
                "rust": "kmipkit_ttlv::Value::structure",
                "c": "kmipkit_ttlv_value_structure",
                "java": "org.kmipkit.ttlv.TtlvValue.structure",
                "python": "kmipkit.ttlv.ttlv_value_structure",
            },
            "ttlv_item_create:c": {
                "rust": "kmipkit_ttlv::Item::new",
                "c": "kmipkit_ttlv_item_create",
                "java": "org.kmipkit.ttlv.TtlvItem.create",
                "python": "kmipkit.ttlv.ttlv_item_create",
            },
            "ttlv_structure_with_item:c": {
                "rust": "kmipkit_ttlv::Structure::try_push",
                "c": "kmipkit_ttlv_structure_with_item",
                "java": "org.kmipkit.ttlv.TtlvStructure.withItem",
                "python": "kmipkit.ttlv.ttlv_structure_with_item",
            },
        }
        required_requirements = {
            "ttlv_value_byte_string:c": {
                "KMIPKIT-0004-FR-001", "KMIPKIT-0004-FR-005",
                "KMIPKIT-0004-FR-009", "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            },
            "ttlv_value_text_string:c": {
                "KMIPKIT-0004-FR-001", "KMIPKIT-0004-FR-005",
                "KMIPKIT-0004-FR-009", "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            },
            "ttlv_value_structure:c": {
                "KMIPKIT-0004-FR-001", "KMIPKIT-0004-FR-003",
                "KMIPKIT-0004-FR-006", "KMIPKIT-0004-FR-008",
                "KMIPKIT-0004-FR-009", "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            },
            "ttlv_item_create:c": {
                "KMIPKIT-0004-FR-003", "KMIPKIT-0004-FR-006",
                "KMIPKIT-0004-FR-008", "KMIPKIT-0004-FR-009",
                "KMIPKIT-0004-FR-010", "KMIPKIT-0004-FR-012",
            },
            "ttlv_structure_with_item:c": {
                "KMIPKIT-0004-FR-003", "KMIPKIT-0004-FR-006",
                "KMIPKIT-0004-FR-009", "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            },
        }

        for declaration_id, expected in expected_mappings.items():
            siblings = [
                self.declarations[
                    f"{declaration_id.rsplit(':', 1)[0]}:{language}"
                ]
                for language in {"c", "java", "python"}
            ]
            rust_declaration = next(
                declaration
                for declaration in self.manifest["publicDeclarations"]
                if declaration.get("exportName") == expected["rust"]
                and declaration.get("packageId") == "rust-kmipkit-ttlv"
            )
            siblings.append(rust_declaration)
            for declaration in siblings:
                sibling_id = declaration["declarationId"]
                with self.subTest(declaration=sibling_id):
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertEqual("generic_ttlv_model", declaration["capabilityId"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertEqual("specs/004-generic-ttlv-model/spec.md",
                                     declaration["approvalSource"]["path"])
                    self.assertEqual("Approved", declaration["approvalSource"]["status"])
                    self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                    self.assertEqual(
                        expected,
                        {
                            language: mapping["name"]
                            for language, mapping in declaration["languageMappings"].items()
                        },
                    )
                    required = required_requirements[declaration_id] | {
                        "KMIPKIT-0043-FR-004",
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    }
                    self.assertTrue(required <= set(declaration["requirementIds"]))
                    self.assertFalse(
                        any(requirement.startswith("KMIPKIT-0012-")
                            for requirement in declaration["requirementIds"])
                    )

        release_mappings = {
            "RawTag:c-release": "kmipkit_ttlv::RawTag",
            "TtlvValue:c-release": "kmipkit_ttlv::Value",
            "TtlvItem:c-release": "kmipkit_ttlv::Item",
        }
        for declaration_id, rust_name in release_mappings.items():
            with self.subTest(declaration=declaration_id):
                declaration = self.declarations[declaration_id]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual("generic_ttlv_model", declaration["capabilityId"])
                self.assertEqual("specs/004-generic-ttlv-model/spec.md",
                                 declaration["approvalSource"]["path"])
                self.assertEqual(LANGUAGES, set(declaration["languageMappings"]))
                self.assertEqual(rust_name,
                                 declaration["languageMappings"]["rust"]["name"])
                self.assertFalse(
                    any(requirement.startswith("KMIPKIT-0012-")
                        for requirement in declaration["requirementIds"])
                )


if __name__ == "__main__":
    unittest.main()
