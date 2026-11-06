"""Tests for importing exact audited exports into the reviewed 0053 routes."""

from __future__ import annotations

import unittest
import json
from pathlib import Path

from tools.api_manifest import prepare_0053_export_audit_routes


APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/053-api-parity-facade/approval-record.md",
    "status": "Approved",
    "evidence": "The maintainer approved the complete KMIPKIT-0053 implementation plan.",
}


class Prepare0053ExportAuditRoutesTests(unittest.TestCase):
    def test_real_export_audits_merge_with_exact_previously_reviewed_routes(self) -> None:
        root = prepare_0053_export_audit_routes.ROOT
        route_path = root / prepare_0053_export_audit_routes.ROUTE_PATH
        manifest_path = root / prepare_0053_export_audit_routes.MANIFEST_PATH
        route_data = json.loads(route_path.read_text(encoding="utf-8"))
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        documents = prepare_0053_export_audit_routes._read_documents(root)
        declarations, exclusions = prepare_0053_export_audit_routes.compile_documents(
            documents, manifest["publicDeclarations"]
        )
        route_data = prepare_0053_export_audit_routes.remove_reviewed_declarations(
            route_data, route_data.get("removePublicDeclarations", [])
        )
        route_data = prepare_0053_export_audit_routes.expand_java_stale_mappings(
            route_data, documents[0].get("staleMappings", [])
        )
        python_document = next(
            document
            for document in documents
            if document.get("packageId") == "python-kmipkit"
        )
        route_data = prepare_0053_export_audit_routes.normalize_python_stale_mappings(
            route_data, python_document.get("staleMappings", [])
        )
        rust_document = next(
            document for document in documents if "functionalRoutes" in document
        )
        route_data = prepare_0053_export_audit_routes.apply_kind_corrections(
            route_data,
            python_document.get("kindCorrections", []),
            rust_document.get("kindCorrections", []),
        )

        merged = prepare_0053_export_audit_routes.merge_declarations(
            route_data, declarations, exclusions
        )

        keys = [
            (row["packageId"], row["exportName"])
            for row in merged["publicDeclarations"]
        ]
        self.assertEqual(len(keys), len(set(keys)))
        self.assertEqual(len(route_data["publicDeclarations"]), len(merged["publicDeclarations"]))
        rerun = prepare_0053_export_audit_routes.merge_declarations(
            merged, declarations, exclusions
        )
        self.assertEqual(merged, rerun)

    def test_reviewed_routes_persist_exact_python_stale_mappings_for_reconciliation(self) -> None:
        route_path = (
            prepare_0053_export_audit_routes.ROOT
            / prepare_0053_export_audit_routes.ROUTE_PATH
        )
        route_data = json.loads(route_path.read_text(encoding="utf-8"))

        stale = next(
            row
            for row in route_data.get("pythonStaleMappings", [])
            if row.get("name") == "KeyBlock.from_fields"
        )

        self.assertEqual(
            "kmipkit.managed_objects.KeyBlock.from_fields", stale["correctedName"]
        )
        self.assertEqual("python-kmipkit", stale["packageId"])
        self.assertEqual("Approved", stale["approval"]["status"])

    def test_compiles_rust_composed_route_using_one_exact_manifest_route_per_language(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "functionalRoutes": [
                {
                    "packageId": "rust-kmipkit-protocol",
                    "name": "kmipkit_protocol::ManagedValueReader::read_field_item",
                    "kind": "associated-item",
                    "resolution": "mapped",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0053-FR-003"],
                    "manifestMapping": {
                        "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::ManagedValueReader::read_field_item"},
                        "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
                        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.managed.ManagedObject.fieldItem"},
                        "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item"},
                    },
                    "manifestMappingRef": {
                        "path": "specification/api/public-api.json",
                        "declarationId": "managed-value-reader-route",
                    },
                    "manifestCompositionDeclarationIds": ["managed-value-reader-route"],
                    "languageMappings": {
                        "rust": ["composed Rust route evidence"],
                        "c": ["C route evidence"],
                        "java": ["Java route evidence"],
                        "python": ["Python route evidence"],
                    },
                    "evidencePaths": [
                        "crates/kmipkit-protocol/tests/managed_objects.rs",
                        "crates/kmipkit-ffi/tests/support/managed_object_0053_tests.rs",
                        "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java",
                        "bindings/python/tests/test_managed_object_parity.py",
                    ],
                    "approval": APPROVAL,
                }
            ],
            "enumShapeExclusions": [],
            "traitImplementationExclusions": [],
            "exactRouteExclusions": [],
            "kindCorrections": [],
            "priorApprovedExclusions": [],
        }

        route = {
            "declarationId": "managed-value-reader-route",
            "packageId": "rust-kmipkit-protocol",
            "exportName": "kmipkit_protocol::ManagedValueReader::read_field_item",
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "declarationState": "verified",
            "languageMappings": document["functionalRoutes"][0]["manifestMapping"],
        }
        declarations, exclusions = prepare_0053_export_audit_routes.compile_documents(
            [document], [route]
        )

        self.assertEqual([], exclusions)
        self.assertEqual([], declarations)
        self.assertEqual(
            route["languageMappings"],
            document["functionalRoutes"][0]["manifestMapping"],
        )

    def test_compiles_audited_java_route_into_verified_four_language_declaration(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "packageId": "java-kmipkit",
            "mappings": [
                {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.ttlv.QueryResponseField.enumeration",
                    "kind": "java-member",
                    "resolution": "mapped",
                    "capabilityId": "response_ttlv_navigation",
                    "requirementIds": ["KMIPKIT-0053-FR-008"],
                    "equivalents": {
                        "c": "kmipkit_client_batch_item_response_query_field_at",
                        "rust": "kmipkit_protocol::QueryResponseField::raw_enumeration",
                        "python": "kmipkit.client.QueryField.enumeration",
                    },
                    "evidencePaths": [
                        "bindings/java/src/test/java/org/kmipkit/extensions/QueryResponseTypedFieldsTest.java",
                        "bindings/c/tests/client_abi_consumer.c",
                    ],
                    "approval": APPROVAL,
                }
            ],
            "exclusions": [],
            "unresolved": [],
        }

        declarations, exclusions = prepare_0053_export_audit_routes.compile_documents(
            [document]
        )

        self.assertEqual([], exclusions)
        self.assertEqual(1, len(declarations))
        declaration = declarations[0]
        self.assertEqual("java-kmipkit", declaration["packageId"])
        self.assertEqual(
            "org.kmipkit.ttlv.QueryResponseField.enumeration",
            declaration["exportName"],
        )
        self.assertEqual("java-member", declaration["kind"])
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("four-language", declaration["parityScope"])
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-protocol",
                    "name": "kmipkit_protocol::QueryResponseField::raw_enumeration",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_response_query_field_at",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.ttlv.QueryResponseField.enumeration",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.QueryField.enumeration",
                },
            },
            declaration["languageMappings"],
        )
        self.assertEqual(
            [
                "bindings/c/tests/client_abi_consumer.c",
                "bindings/java/src/test/java/org/kmipkit/extensions/QueryResponseTypedFieldsTest.java",
            ],
            declaration["parityEvidenceIds"],
        )
        self.assertEqual(APPROVAL, declaration["approvalSource"])

    def test_same_capability_has_distinct_declarations_for_each_language_export(self) -> None:
        mappings = {
            "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::QueryResponseField::raw_enumeration"},
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_response_query_field_at"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.QueryResponseField.enumeration"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.QueryField.enumeration"},
        }
        evidence = [
            "bindings/java/src/test/java/org/kmipkit/extensions/QueryResponseTypedFieldsTest.java",
            "bindings/python/tests/test_query_response_typed_fields.py",
        ]
        common = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "mappings": [],
            "exclusions": [],
            "unresolved": [],
        }
        java = {
            **common,
            "packageId": "java-kmipkit",
            "mappings": [
                {
                    "packageId": "java-kmipkit",
                    "name": mappings["java"]["name"],
                    "kind": "java-member",
                    "resolution": "mapped",
                    "capabilityId": "response_ttlv_navigation",
                    "requirementIds": ["KMIPKIT-0053-FR-008"],
                    "languageMappings": mappings,
                    "evidencePaths": evidence,
                    "approval": APPROVAL,
                }
            ],
        }
        python = {
            **common,
            "packageId": "python-kmipkit",
            "mappings": [
                {
                    "packageId": "python-kmipkit",
                    "name": mappings["python"]["name"],
                    "kind": "python-export",
                    "resolution": "mapped",
                    "capabilityId": "response_ttlv_navigation",
                    "requirementIds": ["KMIPKIT-0053-FR-008"],
                    "languageMappings": mappings,
                    "evidencePaths": evidence,
                    "approval": APPROVAL,
                }
            ],
        }

        declarations, _ = prepare_0053_export_audit_routes.compile_documents([java, python])

        self.assertEqual(2, len(declarations))
        self.assertEqual(
            {mappings["java"]["name"], mappings["python"]["name"]},
            {row["exportName"] for row in declarations},
        )
        self.assertEqual(2, len({row["declarationId"] for row in declarations}))

    def test_compiles_only_exact_approved_java_exclusions(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "packageId": "java-kmipkit",
            "mappings": [],
            "exclusions": [
                {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.NativeExtensionRegistry.internalBridge",
                    "resolution": "excluded",
                    "reason": "This exact JNI bridge has no standalone application capability.",
                    "evidencePaths": [
                        "bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java"
                    ],
                    "approval": APPROVAL,
                }
            ],
            "unresolved": [],
        }

        declarations, exclusions = prepare_0053_export_audit_routes.compile_documents(
            [document]
        )

        self.assertEqual([], declarations)
        self.assertEqual(1, len(exclusions))
        self.assertEqual("java-kmipkit", exclusions[0]["packageId"])
        self.assertEqual(
            "org.kmipkit.NativeExtensionRegistry.internalBridge",
            exclusions[0]["exportName"],
        )
        self.assertEqual("KMIPKIT-0053", exclusions[0]["candidateSpecId"])
        self.assertIn("no standalone application capability", exclusions[0]["reason"])
        self.assertEqual(APPROVAL, exclusions[0]["approvalSource"])
        self.assertIn("NativeExtensionRegistry.java", exclusions[0]["evidence"])

    def test_normalizes_only_exact_python_stale_and_kind_routes(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "mappings": [
                {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.QueryResponse.field_count",
                    "kind": "field",
                    "resolution": "mapped",
                    "capabilityId": "response_ttlv_navigation",
                    "requirementIds": ["KMIPKIT-0053-FR-008"],
                    "equivalents": {
                        "rust": "kmipkit_client::QueryResponse::field_count",
                        "c": "kmipkit_client_batch_item_response_query_field_count",
                        "java": "org.kmipkit.client.QueryResponse.fieldCount",
                    },
                    "evidencePaths": [
                        "bindings/python/tests/test_query_response_typed_fields.py",
                        "bindings/java/src/test/java/org/kmipkit/extensions/QueryResponseTypedFieldsTest.java",
                    ],
                    "approval": APPROVAL,
                }
            ],
            "staleMappings": [
                {
                    "packageId": "python-kmipkit",
                    "name": "QueryResponse.field_count",
                    "correctedName": "kmipkit.client.QueryResponse.field_count",
                }
            ],
            "kindCorrections": [
                {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.client.QueryResponse.field_count",
                    "fromKind": "field",
                    "toKind": "python-export",
                    "evidencePaths": [
                        "bindings/python/tests/test_query_response_typed_fields.py"
                    ],
                    "approval": APPROVAL,
                }
            ],
            "exclusions": [],
            "unresolved": [],
        }

        declarations, _ = prepare_0053_export_audit_routes.compile_documents([document])

        self.assertEqual(1, len(declarations))
        self.assertEqual("kmipkit.client.QueryResponse.field_count", declarations[0]["exportName"])
        self.assertEqual("python-export", declarations[0]["kind"])

    def test_compiles_rust_type_and_trait_exclusions_with_exact_identity(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "functionalRoutes": [
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::protocol::ManagedFieldValue",
                    "kind": "type",
                    "resolution": "mapped",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0053-FR-003"],
                    "manifestMapping": {
                        "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::ManagedFieldValue"},
                        "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
                        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.managed.ManagedFieldValue"},
                        "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedFieldValue"},
                    },
                    "manifestMappingRef": {
                        "path": "specification/api/public-api.json",
                        "declarationId": "managed-field-value-route",
                    },
                    "manifestCompositionDeclarationIds": ["managed-field-value-route"],
                    "evidencePaths": [
                        "crates/kmipkit-protocol/tests/managed_objects.rs",
                        "crates/kmipkit-ffi/tests/support/managed_object_0053_tests.rs",
                        "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java",
                        "bindings/python/tests/test_managed_object_parity.py",
                    ],
                    "approval": APPROVAL,
                }
            ],
            "enumShapeExclusions": [
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::protocol::ManagedFieldValue::Boolean",
                    "artifactKind": "rust-enum-variant",
                    "counterpartExportName": "kmipkit::protocol::ManagedFieldValue",
                    "reason": "The exact Rust enum variant is represented by its mapped parent type.",
                    "evidencePaths": ["crates/kmipkit-protocol/tests/managed_objects.rs"],
                }
            ],
            "traitImplementationExclusions": [
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "<kmipkit::protocol::ManagedFieldValue as core::fmt::Debug>::fmt",
                    "artifactKind": "rust-trait-implementation-member",
                    "identity": "exact-rustdoc-identity-1",
                    "condition": "where T: core::fmt::Debug",
                    "ownerExportName": "kmipkit::protocol::ManagedFieldValue",
                    "reason": "The exact trait implementation member is compiler-visible surface, not an independent route.",
                    "evidencePaths": ["crates/kmipkit-protocol/tests/managed_objects.rs"],
                }
            ],
            "exactRouteExclusions": [],
            "kindCorrections": [],
            "priorApprovedExclusions": [],
        }

        route = {
            "declarationId": "managed-field-value-route",
            "packageId": "rust-kmipkit-protocol",
            "exportName": "kmipkit_protocol::ManagedValueReader::read_field_item",
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "declarationState": "verified",
            "languageMappings": document["functionalRoutes"][0]["manifestMapping"],
        }
        declarations, exclusions = prepare_0053_export_audit_routes.compile_documents(
            [document], [route]
        )

        self.assertEqual([], declarations)
        self.assertEqual(2, len(exclusions))
        by_name = {row["exportName"]: row for row in exclusions}
        variant = by_name["kmipkit::protocol::ManagedFieldValue::Boolean"]
        self.assertEqual("rust-enum-variant", variant["artifactKind"])
        self.assertEqual([route["declarationId"]], variant["counterpartDeclarationIds"])
        trait_member = by_name["<kmipkit::protocol::ManagedFieldValue as core::fmt::Debug>::fmt"]
        self.assertEqual("exact-rustdoc-identity-1", trait_member["identity"])
        self.assertEqual("where T: core::fmt::Debug", trait_member["condition"])

    def test_refuses_unresolved_audit_rows_before_generating_routes(self) -> None:
        document = {
            "formatVersion": 1,
            "specId": "KMIPKIT-0053",
            "packageId": "java-kmipkit",
            "mappings": [],
            "exclusions": [],
            "unresolved": [
                {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.ttlv.Missing.member",
                }
            ],
        }

        with self.assertRaises(prepare_0053_export_audit_routes.ResolutionError):
            prepare_0053_export_audit_routes.compile_documents([document])

    def test_reviewed_route_merge_is_idempotent_and_replaces_only_imported_rows(self) -> None:
        declaration = {
            "declarationId": "KMIPKIT-0053:export:exact-test-route",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": "org.kmipkit.Example.value",
        }
        reviewed = {
            "functions": [{"id": "existing"}],
            "publicDeclarations": [
                {"declarationId": "human-reviewed", "packageId": "java-kmipkit", "exportName": "org.kmipkit.Existing.value"},
                {"declarationId": "KMIPKIT-0053:export:stale", "packageId": "java-kmipkit", "exportName": "org.kmipkit.Stale.value"},
            ],
        }

        first = prepare_0053_export_audit_routes.merge_declarations(reviewed, [declaration])
        second = prepare_0053_export_audit_routes.merge_declarations(first, [declaration])

        self.assertEqual(first, second)
        self.assertEqual(reviewed["functions"], first["functions"])
        self.assertEqual(
            ["KMIPKIT-0053:export:exact-test-route", "human-reviewed"],
            [row["declarationId"] for row in first["publicDeclarations"]],
        )

    def test_audited_route_reuses_existing_exact_declaration_without_duplicate(self) -> None:
        existing = {
            "declarationId": "reviewed-stable-id",
            "packageId": "python-kmipkit",
            "kind": "python-export",
            "exportName": "kmipkit.client.QueryResponse.field_count",
            "capabilityId": "response_ttlv_navigation",
            "requirementIds": ["KMIPKIT-0053-FR-008"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::QueryResponse::field_count"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_item_response_query_field_count"},
                "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.client.QueryResponse.fieldCount"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.QueryResponse.field_count"},
            },
            "parityEvidenceIds": ["bindings/python/tests/test_query_response_typed_fields.py"],
            "approvalSource": APPROVAL,
        }
        candidate = {**existing, "declarationId": "KMIPKIT-0053:export:audit-route"}
        route_data = {"publicDeclarations": [existing]}

        merged = prepare_0053_export_audit_routes.merge_declarations(route_data, [candidate])

        self.assertEqual([existing], merged["publicDeclarations"])

    def test_audited_exact_route_unions_existing_requirement_and_evidence_traceability(self) -> None:
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
                "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::ManagedValueReader::read_field_item"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
                "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.KeyBlock.fieldItem"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.KeyBlock.field_item"},
            },
            "parityEvidenceIds": ["bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java"],
            "approvalSource": APPROVAL,
        }
        candidate = {
            **existing,
            "declarationId": "KMIPKIT-0053:export:key-block-item",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "parityEvidenceIds": ["bindings/python/tests/test_managed_object_parity.py"],
        }

        merged = prepare_0053_export_audit_routes.merge_declarations(
            {"publicDeclarations": [existing]}, [candidate]
        )

        self.assertEqual(["KMIPKIT-0049-FR-002", "KMIPKIT-0053-FR-003"], merged["publicDeclarations"][0]["requirementIds"])
        self.assertEqual(
            [
                "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java",
                "bindings/python/tests/test_managed_object_parity.py",
            ],
            merged["publicDeclarations"][0]["parityEvidenceIds"],
        )
        self.assertEqual("reviewed-stable-id", merged["publicDeclarations"][0]["declarationId"])

    def test_expands_package_private_java_base_to_all_public_wrapper_exports(self) -> None:
        old = "org.kmipkit.ttlv.ManagedTtlvValue.fieldItem"
        targets = [
            "org.kmipkit.ttlv.ManagedObject.fieldItem",
            "org.kmipkit.ttlv.KeyBlock.fieldItem",
            "org.kmipkit.ttlv.KeyValue.fieldItem",
        ]
        route = {
            "declarationId": "managed-field-item:java",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": old,
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::ManagedValueReader::read_field_item"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
                "java": {"packageId": "java-kmipkit", "name": old},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item"},
            },
            "parityEvidenceIds": ["bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java"],
            "approvalSource": APPROVAL,
        }
        source = {
            "publicDeclarations": [route],
            "javaStaleMappings": [
                {
                    "declarationId": route["declarationId"],
                    "oldName": old,
                    "observedPublicTargets": [
                        {"name": name, "owner": name.rsplit(".", 1)[0]}
                        for name in targets
                    ],
                }
            ],
        }

        first = prepare_0053_export_audit_routes.expand_java_stale_mappings(source)
        second = prepare_0053_export_audit_routes.expand_java_stale_mappings(first)

        self.assertEqual(first, second)
        self.assertEqual(set(targets), {
            row["exportName"] for row in first["publicDeclarations"]
            if row["packageId"] == "java-kmipkit"
        })
        self.assertTrue(all(
            row["languageMappings"]["java"]["name"] in targets
            for row in first["publicDeclarations"]
        ))

    def test_java_wrapper_expansion_merges_exact_existing_evidence_idempotently(self) -> None:
        old = "org.kmipkit.ttlv.ManagedTtlvValue.fieldItem"
        target = "org.kmipkit.ttlv.KeyBlock.fieldItem"
        mappings = {
            "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::ManagedValueReader::read_field_item"},
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
            "java": {"packageId": "java-kmipkit", "name": old},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item"},
        }
        canonical = {
            "declarationId": "managed-field-item:java",
            "packageId": "java-kmipkit",
            "kind": "java-member",
            "exportName": old,
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-003"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": mappings,
            "parityEvidenceIds": ["bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java"],
            "approvalSource": APPROVAL,
        }
        wrapper = {
            **canonical,
            "declarationId": "managed-field-item:java:wrapper:keyblock",
            "exportName": target,
            "languageMappings": {
                **mappings,
                "java": {"packageId": "java-kmipkit", "name": target},
            },
            "parityEvidenceIds": ["bindings/java/src/main/java/org/kmipkit/ttlv/KeyBlock.java"],
        }
        source = {
            "publicDeclarations": [canonical, wrapper],
            "javaStaleMappings": [
                {
                    "declarationId": canonical["declarationId"],
                    "oldName": old,
                    "observedPublicTargets": [
                        {"name": old, "owner": "ManagedTtlvValue"},
                        {"name": target, "owner": "KeyBlock"},
                    ],
                }
            ],
        }

        first = prepare_0053_export_audit_routes.expand_java_stale_mappings(source)
        second = prepare_0053_export_audit_routes.expand_java_stale_mappings(first)

        expanded = next(row for row in first["publicDeclarations"] if row["exportName"] == target)
        self.assertEqual(wrapper["declarationId"], expanded["declarationId"])
        self.assertEqual(
            [
                "bindings/java/src/main/java/org/kmipkit/ttlv/KeyBlock.java",
                "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java",
            ],
            expanded["parityEvidenceIds"],
        )
        self.assertEqual(first, second)

    def test_normalizes_python_stale_paths_in_all_manifest_rows_idempotently(self) -> None:
        old = "KeyBlock.from_fields"
        corrected = "kmipkit.managed_objects.KeyBlock.from_fields"
        reviewed = {
            "publicDeclarations": [
                {
                    "packageId": "c-kmipkit",
                    "exportName": "kmipkit_key_block_from_fields",
                    "languageMappings": {"python": {"packageId": "python-kmipkit", "name": old}},
                },
                {
                    "packageId": "python-kmipkit",
                    "exportName": old,
                    "languageMappings": {"python": {"packageId": "python-kmipkit", "name": old}},
                },
            ]
        }
        stale = [{"packageId": "python-kmipkit", "name": old, "correctedName": corrected}]

        first = prepare_0053_export_audit_routes.normalize_python_stale_mappings(reviewed, stale)
        second = prepare_0053_export_audit_routes.normalize_python_stale_mappings(first, stale)

        self.assertEqual(first, second)
        self.assertEqual(corrected, first["publicDeclarations"][0]["languageMappings"]["python"]["name"])
        self.assertEqual(corrected, first["publicDeclarations"][1]["exportName"])

    def test_applies_exact_python_and_rust_kind_corrections(self) -> None:
        reviewed = {
            "publicDeclarations": [
                {"packageId": "python-kmipkit", "exportName": "kmipkit.client.QueryResponse.field_count", "kind": "field"},
                {"packageId": "rust-kmipkit-protocol", "exportName": "kmipkit_protocol::ManagedValueReader::read_field", "kind": "function"},
            ]
        }
        python_corrections = [
            {
                "packageId": "python-kmipkit",
                "name": "kmipkit.client.QueryResponse.field_count",
                "fromKind": "field",
                "toKind": "python-export",
            }
        ]
        rust_corrections = [
            {
                "packageId": "rust-kmipkit-protocol",
                "exportName": "kmipkit_protocol::ManagedValueReader::read_field",
                "manifestKind": "function",
                "observedKind": "associated-item",
            }
        ]

        first = prepare_0053_export_audit_routes.apply_kind_corrections(
            reviewed, python_corrections, rust_corrections
        )
        second = prepare_0053_export_audit_routes.apply_kind_corrections(
            first, python_corrections, rust_corrections
        )

        self.assertEqual(first, second)
        self.assertEqual("python-export", first["publicDeclarations"][0]["kind"])
        self.assertEqual("associated-item", first["publicDeclarations"][1]["kind"])

    def test_removes_only_an_exact_approved_duplicate_declaration_idempotently(self) -> None:
        declaration = {
            "declarationId": "0053-counterpart-duplicate-route",
            "packageId": "python-kmipkit",
            "exportName": "kmipkit.managed_objects.KeyWrappingData.from_fields",
        }
        removal = {
            "declarationId": declaration["declarationId"],
            "packageId": declaration["packageId"],
            "exportName": declaration["exportName"],
            "reason": "The same Python constructor is recorded under its owning managed-object capability.",
            "candidateSpecId": "KMIPKIT-0053",
            "approvalSource": APPROVAL,
            "evidencePaths": ["bindings/python/tests/test_managed_object_parity.py"],
        }
        route_data = {"publicDeclarations": [declaration]}

        first = prepare_0053_export_audit_routes.remove_reviewed_declarations(
            route_data, [removal]
        )
        second = prepare_0053_export_audit_routes.remove_reviewed_declarations(
            first, [removal]
        )

        self.assertEqual([], first["publicDeclarations"])
        self.assertEqual(first, second)


if __name__ == "__main__":
    unittest.main()
