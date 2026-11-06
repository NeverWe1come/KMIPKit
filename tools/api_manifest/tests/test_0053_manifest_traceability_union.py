"""Regression tests for preserving existing declaration provenance."""

import unittest

from tools.api_manifest import reconcile_0053_api_parity


class ManifestTraceabilityUnionTests(unittest.TestCase):
    def test_exact_existing_export_unions_requirements_and_evidence(self) -> None:
        mappings = {
            "rust": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field_item",
            },
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_managed_value_field_item_read",
            },
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.ttlv.KeyBlock.fieldItem",
            },
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.managed_objects.KeyBlock.field_item",
            },
        }
        approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
        }
        old_evidence = "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java"
        new_evidence = "bindings/python/tests/test_managed_object_parity.py"
        existing = {
            "declarationId": "reviewed-stable-id",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": "org.kmipkit.ttlv.KeyBlock.fieldItem",
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0049-FR-002"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                **mappings,
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "KeyBlock.from_fields",
                },
            },
            "parityEvidenceIds": [old_evidence],
            "approvalSource": approval,
        }
        candidate_mappings = {
            **mappings,
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.managed_objects.KeyBlock.from_fields",
            },
        }
        candidate = {
            **existing,
            "declarationId": "audit-stable-id",
            "languageMappings": candidate_mappings,
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "parityEvidenceIds": [new_evidence],
        }
        manifest = {
            "requirementIds": ["KMIPKIT-0049-FR-002"],
            "publicDeclarations": [existing],
            "capabilities": [
                {
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "mappings": {language: [] for language in mappings},
                    "requirementIds": ["KMIPKIT-0049-FR-002"],
                    "parityEvidenceIds": [old_evidence],
                }
            ],
        }
        stale_mapping = {
            "packageId": "python-kmipkit",
            "name": "KeyBlock.from_fields",
            "correctedName": "kmipkit.managed_objects.KeyBlock.from_fields",
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "evidencePaths": [new_evidence],
            "approval": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
            },
            "reason": "Adds the exact package prefix omitted by the earlier route.",
        }

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate], python_stale_mappings=[stale_mapping]
        )

        declaration = reconciled["publicDeclarations"][0]
        self.assertEqual("reviewed-stable-id", declaration["declarationId"])
        self.assertEqual(
            "kmipkit.managed_objects.KeyBlock.from_fields",
            declaration["languageMappings"]["python"]["name"],
        )
        self.assertEqual(
            ["KMIPKIT-0049-FR-002", "KMIPKIT-0053-FR-003"],
            declaration["requirementIds"],
        )
        self.assertEqual(
            [old_evidence, new_evidence], declaration["parityEvidenceIds"]
        )
        capability = reconciled["capabilities"][0]
        self.assertEqual(declaration["requirementIds"], capability["requirementIds"])
        self.assertEqual(declaration["parityEvidenceIds"], capability["parityEvidenceIds"])
        self.assertEqual(declaration["requirementIds"], reconciled["requirementIds"])

    def test_java_package_private_export_is_renamed_to_first_exact_public_wrapper(self) -> None:
        old_name = "org.kmipkit.ttlv.ManagedTtlvValue.fieldItem"
        corrected_name = "org.kmipkit.ttlv.ManagedObject.fieldItem"
        mappings = {
            "rust": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field_item",
            },
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
            "java": {"packageId": "java-kmipkit", "name": corrected_name},
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item",
            },
        }
        approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
        }
        existing = {
            "declarationId": "managed_value_field_item_read:java",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": old_name,
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                **mappings,
                "java": {"packageId": "java-kmipkit", "name": old_name},
            },
            "parityEvidenceIds": ["bindings/java/src/main/java/org/kmipkit/ttlv/ManagedTtlvValue.java"],
            "approvalSource": approval,
        }
        candidate = {
            **existing,
            "exportName": corrected_name,
            "languageMappings": mappings,
            "parityEvidenceIds": ["bindings/java/src/main/java/org/kmipkit/ttlv/ManagedObject.java"],
        }
        stale_mapping = {
            "packageId": "java-kmipkit",
            "declarationId": existing["declarationId"],
            "capabilityId": existing["capabilityId"],
            "oldName": old_name,
            "observedPublicTargets": [
                {"owner": "org.kmipkit.ttlv.ManagedObject", "name": corrected_name}
            ],
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "reason": "The package-private base is replaced by its first public wrapper.",
            "evidencePaths": [
                "bindings/java/src/main/java/org/kmipkit/ttlv/ManagedTtlvValue.java",
                "bindings/java/src/main/java/org/kmipkit/ttlv/ManagedObject.java",
            ],
            "approval": approval,
            "name": old_name,
        }
        manifest = {
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "publicDeclarations": [existing],
            "capabilities": [
                {
                    "capabilityId": existing["capabilityId"],
                    "mappings": {language: [] for language in mappings},
                    "requirementIds": ["KMIPKIT-0053-FR-003"],
                    "parityEvidenceIds": existing["parityEvidenceIds"],
                }
            ],
        }

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate], java_stale_mappings=[stale_mapping]
        )

        self.assertEqual(1, len(reconciled["publicDeclarations"]))
        declaration = reconciled["publicDeclarations"][0]
        self.assertEqual(existing["declarationId"], declaration["declarationId"])
        self.assertEqual(corrected_name, declaration["exportName"])
        self.assertEqual(corrected_name, declaration["languageMappings"]["java"]["name"])
        self.assertNotIn(old_name, [row["exportName"] for row in reconciled["publicDeclarations"]])

    def test_python_export_kind_changes_only_with_exact_approved_correction(self) -> None:
        export_name = "kmipkit.client.QueryResponse.field_count"
        mappings = {
            "rust": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::QueryResponse::response_field_count",
            },
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_response_query_field_count"},
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.extensions.QueryResponse.fieldCount",
            },
            "python": {"packageId": "python-kmipkit", "name": export_name},
        }
        approval = {
            "kind": "feature-specification",
            "path": "specs/052-response-navigation/spec.md",
            "status": "Approved",
            "evidence": "Approved response navigation route.",
        }
        correction_approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
        }
        existing = {
            "declarationId": "T064:query-field-count:python",
            "packageId": "python-kmipkit",
            "kind": "field",
            "exportName": export_name,
            "capabilityId": "response_ttlv_navigation",
            "requirementIds": ["KMIPKIT-0053-FR-008"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": mappings,
            "parityEvidenceIds": [
                "bindings/python/tests/test_query_response_typed_fields.py"
            ],
            "approvalSource": approval,
        }
        candidate = {**existing, "kind": "python-export"}
        correction = {
            "packageId": "python-kmipkit",
            "name": export_name,
            "fromKind": "field",
            "toKind": "python-export",
            "capabilityId": "response_ttlv_navigation",
            "requirementIds": ["KMIPKIT-0053-FR-008"],
            "evidencePaths": [
                "bindings/python/tests/test_query_response_typed_fields.py"
            ],
            "approval": correction_approval,
            "reason": "The inspected public export is callable, not a data field.",
        }
        manifest = {
            "requirementIds": ["KMIPKIT-0053-FR-008"],
            "publicDeclarations": [existing],
            "capabilities": [
                {
                    "capabilityId": "response_ttlv_navigation",
                    "mappings": {language: [mappings[language]["name"]] for language in mappings},
                    "requirementIds": ["KMIPKIT-0053-FR-008"],
                    "parityEvidenceIds": existing["parityEvidenceIds"],
                }
            ],
        }

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate], python_kind_corrections=[correction]
        )

        self.assertEqual("python-export", reconciled["publicDeclarations"][0]["kind"])
        self.assertEqual(existing["declarationId"], reconciled["publicDeclarations"][0]["declarationId"])

    def test_python_stale_export_path_renames_stable_manifest_declaration(self) -> None:
        old_name = "KeyBlock.from_fields"
        corrected_name = "kmipkit.managed_objects.KeyBlock.from_fields"
        mappings = {
            "rust": {
                "packageId": "rust-kmipkit",
                "name": "kmipkit::protocol::KeyBlock::try_from_item",
            },
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_key_block_from_fields"},
            "java": {
                "packageId": "java-kmipkit",
                "name": "org.kmipkit.ttlv.KeyBlock.fromFields",
            },
            "python": {"packageId": "python-kmipkit", "name": corrected_name},
        }
        approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
        }
        evidence = "bindings/python/tests/test_managed_object_parity.py"
        existing = {
            "declarationId": "key_block_from_fields:python",
            "packageId": "python-kmipkit",
            "kind": "python-export",
            "exportName": old_name,
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0049-FR-001"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                **mappings,
                "python": {"packageId": "python-kmipkit", "name": old_name},
            },
            "parityEvidenceIds": ["bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java"],
            "approvalSource": approval,
        }
        candidate = {
            **existing,
            "exportName": corrected_name,
            "languageMappings": mappings,
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "parityEvidenceIds": [evidence],
        }
        correction = {
            "packageId": "python-kmipkit",
            "name": old_name,
            "correctedName": corrected_name,
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "evidencePaths": [evidence],
            "approval": approval,
            "reason": "The earlier declaration omitted the exact Python package path.",
        }
        manifest = {
            "requirementIds": ["KMIPKIT-0049-FR-001"],
            "publicDeclarations": [existing],
            "capabilities": [
                {
                    "capabilityId": existing["capabilityId"],
                    "mappings": {language: [] for language in mappings},
                    "requirementIds": existing["requirementIds"],
                    "parityEvidenceIds": existing["parityEvidenceIds"],
                }
            ],
        }

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate], python_stale_mappings=[correction]
        )

        self.assertEqual(1, len(reconciled["publicDeclarations"]))
        declaration = reconciled["publicDeclarations"][0]
        self.assertEqual(existing["declarationId"], declaration["declarationId"])
        self.assertEqual(corrected_name, declaration["exportName"])
        self.assertEqual(corrected_name, declaration["languageMappings"]["python"]["name"])
        self.assertNotIn(old_name, [row["exportName"] for row in reconciled["publicDeclarations"]])

    def test_exact_existing_export_corrects_approved_java_wrapper_route(self) -> None:
        old_name = "org.kmipkit.ttlv.ManagedTtlvValue.fieldItem"
        corrected_name = "org.kmipkit.ttlv.KeyBlock.fieldItem"
        mappings = {
            "rust": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field_item",
            },
            "c": {
                "packageId": "c-kmipkit",
                "name": "kmipkit_managed_value_field_item_read",
            },
            "java": {"packageId": "java-kmipkit", "name": corrected_name},
            "python": {
                "packageId": "python-kmipkit",
                "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item",
            },
        }
        approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Maintainer-approved KMIPKIT-0053 scope.",
        }
        old_evidence = "bindings/java/src/main/java/org/kmipkit/ttlv/ManagedTtlvValue.java"
        new_evidence = "bindings/java/src/main/java/org/kmipkit/ttlv/KeyBlock.java"
        existing = {
            "declarationId": "reviewed-c-route",
            "packageId": "c-kmipkit",
            "kind": "c-symbol",
            "exportName": "kmipkit_managed_value_field_item_read",
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0049-FR-002"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                **mappings,
                "java": {"packageId": "java-kmipkit", "name": old_name},
            },
            "parityEvidenceIds": [old_evidence],
            "approvalSource": approval,
        }
        candidate = {
            **existing,
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "languageMappings": mappings,
            "parityEvidenceIds": [new_evidence],
        }
        stale_mapping = {
            "packageId": "java-kmipkit",
            "declarationId": "reviewed-java-base-route",
            "capabilityId": "typed_managed_objects_and_key_material",
            "oldName": old_name,
            "observedPublicTargets": [
                {"owner": "org.kmipkit.ttlv.KeyBlock", "name": corrected_name}
            ],
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "reason": "The package-private base is replaced by its public KeyBlock wrapper.",
            "evidencePaths": [old_evidence, new_evidence],
            "approval": approval,
            "name": old_name,
        }
        manifest = {
            "requirementIds": ["KMIPKIT-0049-FR-002"],
            "publicDeclarations": [existing],
            "capabilities": [
                {
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "mappings": {language: [] for language in mappings},
                    "requirementIds": ["KMIPKIT-0049-FR-002"],
                    "parityEvidenceIds": [old_evidence],
                }
            ],
        }

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate], java_stale_mappings=[stale_mapping]
        )

        declaration = reconciled["publicDeclarations"][0]
        self.assertEqual("reviewed-c-route", declaration["declarationId"])
        self.assertEqual(corrected_name, declaration["languageMappings"]["java"]["name"])
        self.assertEqual(
            [
                "KMIPKIT-0049-FR-002",
                "KMIPKIT-0053-FR-003",
            ],
            declaration["requirementIds"],
        )
        self.assertEqual(
            sorted([old_evidence, new_evidence]), declaration["parityEvidenceIds"]
        )
        capability = reconciled["capabilities"][0]
        self.assertEqual(declaration["requirementIds"], capability["requirementIds"])
        self.assertEqual(declaration["parityEvidenceIds"], capability["parityEvidenceIds"])
        self.assertEqual(declaration["requirementIds"], reconciled["requirementIds"])


if __name__ == "__main__":
    unittest.main()
