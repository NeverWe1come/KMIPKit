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
            "languageMappings": mappings,
            "parityEvidenceIds": [old_evidence],
            "approvalSource": approval,
        }
        candidate = {
            **existing,
            "declarationId": "audit-stable-id",
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

        reconciled = reconcile_0053_api_parity.add_public_declarations(
            manifest, [candidate]
        )

        declaration = reconciled["publicDeclarations"][0]
        self.assertEqual("reviewed-stable-id", declaration["declarationId"])
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
