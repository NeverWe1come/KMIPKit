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
            [old_evidence, new_evidence], declaration["parityEvidenceIds"]
        )
        self.assertEqual(
            ["KMIPKIT-0049-FR-002", "KMIPKIT-0053-FR-003"],
            declaration["requirementIds"],
        )
        self.assertEqual([old_evidence, new_evidence], declaration["parityEvidenceIds"])
        capability = reconciled["capabilities"][0]
        self.assertEqual(declaration["requirementIds"], capability["requirementIds"])
        self.assertEqual(declaration["parityEvidenceIds"], capability["parityEvidenceIds"])
        self.assertEqual(declaration["requirementIds"], reconciled["requirementIds"])


if __name__ == "__main__":
    unittest.main()
