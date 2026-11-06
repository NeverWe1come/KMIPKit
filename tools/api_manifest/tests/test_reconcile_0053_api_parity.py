import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import reconcile_0053_api_parity


class Reconcile0053ApiParityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "requirementIds": [],
            "functions": [],
            "publicDeclarations": [
                {
                    "declarationId": "rust-attribute-type",
                    "packageId": "rust-kmipkit",
                    "kind": "type",
                    "exportName": "kmipkit::protocol::Attribute",
                    "capabilityId": "typed_object_attributes",
                    "requirementIds": ["KMIPKIT-0048-FR-001"],
                    "declarationState": "blocked",
                    "parityScope": "four-language",
                    "candidateSpecId": "KMIPKIT-0053",
                    "blockedReason": "foreign parity is pending",
                },
                {
                    "declarationId": "rust-authentication",
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "kmipkit::ClientBatch::with_authentication",
                    "capabilityId": "request_message_authentication",
                    "requirementIds": ["KMIPKIT-0050-FR-004"],
                    "declarationState": "blocked",
                    "parityScope": "four-language",
                    "candidateSpecId": "KMIPKIT-0053",
                    "blockedReason": "foreign parity is pending",
                },
            ],
            "approvedExclusions": [],
            "capabilities": [
                {
                    "capabilityId": "typed_object_attributes",
                    "capabilityState": "blocked",
                    "parityScope": "four-language",
                    "ownerSpecId": "KMIPKIT-0048",
                },
                {
                    "capabilityId": "request_message_authentication",
                    "capabilityState": "blocked",
                    "parityScope": "four-language",
                    "ownerSpecId": "KMIPKIT-0050",
                },
            ],
        }
        self.routes = {
            "rust-attribute-type": {
                "rust": {"packageId": "rust-kmipkit", "name": "Attribute"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_typed_attribute"},
                "java": {"packageId": "java-kmipkit", "name": "TypedAttribute"},
                "python": {"packageId": "python-kmipkit", "name": "TypedAttribute"},
                "parityEvidenceIds": ["bindings/c/tests/typed_attribute_consumer.c"],
                "approvalSource": {
                    "kind": "feature-specification",
                    "path": "specs/048-typed-attributes/approval-record.md",
                    "status": "Approved",
                    "evidence": "direct approval of the Phase 5 plan",
                },
            },
            "rust-authentication": {
                "rust": {"packageId": "rust-kmipkit", "name": "ClientBatch::with_authentication"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_client_batch_with_authentication"},
                "java": {"packageId": "java-kmipkit", "name": "ClientBatch.withAuthentication"},
                "python": {"packageId": "python-kmipkit", "name": "ClientBatch.with_authentication"},
                "parityEvidenceIds": ["bindings/c/tests/authentication_consumer.c"],
                "approvalSource": {
                    "kind": "feature-specification",
                    "path": "specs/050-cryptographic-parameters/approval-record.md",
                    "status": "Approved",
                    "evidence": "direct approval of the Phase 5 plan",
                },
            },
        }

    def test_reconciles_exact_four_language_routes_and_capabilities(self) -> None:
        result = reconcile_0053_api_parity.reconcile_manifest(self.manifest, self.routes)

        self.assertEqual("blocked", self.manifest["publicDeclarations"][0]["declarationState"])
        for declaration in result["publicDeclarations"]:
            self.assertEqual("verified", declaration["declarationState"])
            self.assertNotIn("candidateSpecId", declaration)
            self.assertNotIn("blockedReason", declaration)
            self.assertEqual({"rust", "c", "java", "python"}, set(declaration["languageMappings"]))
        self.assertTrue(all(item["capabilityState"] == "verified" for item in result["capabilities"]))
        for original, capability in zip(self.manifest["capabilities"], result["capabilities"], strict=True):
            self.assertEqual(original["ownerSpecId"], capability["ownerSpecId"])
            self.assertEqual("Approved", capability["approvalEvidence"]["status"])
            self.assertTrue(capability["parityEvidenceIds"])

    def test_rejects_missing_language_or_evidence_mapping(self) -> None:
        routes = copy.deepcopy(self.routes)
        del routes["rust-attribute-type"]["python"]

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

    def test_reconciles_one_audited_candidate_without_unblocking_unrelated_routes(self) -> None:
        routes = {"rust-attribute-type": self.routes["rust-attribute-type"]}

        result = reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

        states = {row["declarationId"]: row["declarationState"]
                  for row in result["publicDeclarations"]}
        self.assertEqual("verified", states["rust-attribute-type"])
        self.assertEqual("blocked", states["rust-authentication"])
        capabilities = {row["capabilityId"]: row["capabilityState"]
                        for row in result["capabilities"]}
        self.assertEqual("verified", capabilities["typed_object_attributes"])
        self.assertEqual("blocked", capabilities["request_message_authentication"])

    def test_rejects_a_general_exclusion_for_typed_capabilities(self) -> None:
        routes = copy.deepcopy(self.routes)
        routes["rust-attribute-type"] = {
            "exclusion": {
                "reason": "foreign APIs are not supported",
                "evidence": ["specs/053-api-parity-facade/spec.md"],
            }
        }

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.reconcile_manifest(self.manifest, routes)

    def test_repository_inventory_contains_only_the_reviewed_0053_candidate_groups(self) -> None:
        manifest_path = Path("specification/api/public-api.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        candidates = [
            declaration
            for declaration in manifest["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0053"
        ]
        self.assertEqual(645, len(candidates))
        self.assertEqual(
            {
                "typed_object_attributes": 279,
                "typed_managed_objects_and_key_material": 342,
            "typed_cryptographic_parameters": 24,
            },
            {
                capability_id: sum(
                    declaration["capabilityId"] == capability_id
                    for declaration in candidates
                )
                for capability_id in {
                    declaration["capabilityId"] for declaration in candidates
                }
            },
        )
        resolved = [
            declaration for declaration in manifest["publicDeclarations"]
            if declaration.get("declarationId") == "kmipkit-0048-kmipkit-7ff36f3a6017bfcf719b"
        ]
        self.assertEqual(1, len(resolved))
        self.assertEqual("verified", resolved[0]["declarationState"])
        capabilities = {row["capabilityId"]: row["capabilityState"]
                        for row in manifest["capabilities"]}
        self.assertEqual("verified", capabilities["request_message_authentication"])

    def test_reconciles_only_executable_table59_field_getters_and_builder_setters(self) -> None:
        manifest = json.loads(Path("specification/api/public-api.json").read_text(encoding="utf-8"))
        route_data = json.loads(
            Path("specs/053-api-parity-facade/additional-api-routes.json").read_text(
                encoding="utf-8"
            )
        )
        field_names = {
            "block_cipher_mode",
            "padding_method",
            "hashing_algorithm",
            "key_role_type",
            "digital_signature_algorithm",
            "cryptographic_algorithm",
            "random_iv",
            "iv_length",
            "tag_length",
            "fixed_field_length",
            "invocation_field_length",
            "counter_length",
            "initial_counter_value",
            "salt_length",
            "mask_generator",
            "mask_generator_hashing_algorithm",
            "p_source",
            "trailer_field",
        }
        field_routes = set()
        operation_setters = set()
        for declaration in manifest["publicDeclarations"]:
            if declaration.get("capabilityId") != "typed_cryptographic_parameters":
                continue
            export_name = declaration["exportName"]
            if export_name.endswith("::with_typed_cryptographic_parameters"):
                operation_setters.add(declaration["declarationId"])
                continue
            getter_prefix = (
                "kmipkit::protocol::CryptographicParameters::",
                "kmipkit_protocol::CryptographicParameters::",
            )
            builder_prefix = (
                "kmipkit::protocol::CryptographicParametersBuilder::",
                "kmipkit_protocol::CryptographicParametersBuilder::",
            )
            if any(export_name.startswith(prefix) for prefix in getter_prefix):
                member = export_name.rsplit("::", maxsplit=1)[-1]
                if member in field_names or member == "with_p_source":
                    field_routes.add(declaration["declarationId"])
            elif any(export_name.startswith(prefix) for prefix in builder_prefix):
                member = export_name.rsplit("::", maxsplit=1)[-1]
                if member.startswith("with_") and member.removeprefix("with_") in field_names:
                    field_routes.add(declaration["declarationId"])

        self.assertEqual(72, len(field_routes))
        mapped_ids = set(route_data["candidateRoutes"])
        self.assertEqual(field_routes, mapped_ids & field_routes)
        self.assertFalse(operation_setters & mapped_ids)
        listed_targets = {
            (row.get("packageId"), row.get("exportName"))
            for row in manifest["publicDeclarations"] + route_data["publicDeclarations"]
        }
        for declaration_id in field_routes:
            route = route_data["candidateRoutes"][declaration_id]
            for language, mapping in route.items():
                if language in {"rust", "c", "java", "python"}:
                    self.assertIn((mapping["packageId"], mapping["name"]), listed_targets)
            evidence = set(route["parityEvidenceIds"])
            for required_path in (
                "bindings/c/tests/authentication_consumer.c",
                "bindings/java/src/test/java/org/kmipkit/typed/ParametersAuthenticationParityTest.java",
                "bindings/python/tests/test_parameters_authentication_parity.py",
            ):
                self.assertIn(required_path, evidence)
            self.assertTrue(
                any(path.startswith("crates/") for path in evidence),
                "Table 59 mapping lacks Rust behavior evidence",
            )

        reconciled = reconcile_0053_api_parity.reconcile_manifest(
            manifest, route_data["candidateRoutes"]
        )
        reconciled = reconcile_0053_api_parity.add_public_declarations(
            reconciled, route_data["publicDeclarations"]
        )
        remaining = [
            declaration
            for declaration in reconciled["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0053"
            and declaration.get("capabilityId") == "typed_cryptographic_parameters"
        ]
        self.assertEqual(24, len(remaining))
        remaining_names = [row["exportName"] for row in remaining]
        self.assertEqual(18, sum(name.endswith("::with_typed_cryptographic_parameters")
                                 for name in remaining_names))
        self.assertEqual(2, sum(name.endswith("CryptographicParameters::with_ttlv")
                                 for name in remaining_names))
        self.assertEqual(2, sum(name.endswith("CryptographicParametersBuilder::new")
                                 for name in remaining_names))
        self.assertEqual(2, sum(name.endswith("CryptographicParametersBuilder::with_extension")
                                 for name in remaining_names))

    def test_managed_object_java_routes_match_exported_camel_case_methods(self) -> None:
        route_data = json.loads(
            Path("specs/053-api-parity-facade/additional-api-routes.json").read_text(
                encoding="utf-8"
            )
        )
        expected = {
            "certificate_from_fields": "certificate",
            "symmetric_key_from_fields": "symmetricKey",
            "public_key_from_fields": "publicKey",
            "private_key_from_fields": "privateKey",
            "split_key_from_fields": "splitKey",
            "secret_data_from_fields": "secretData",
            "opaque_object_from_fields": "opaqueObject",
            "pgp_key_from_fields": "pgpKey",
            "certificate_request_from_fields": "certificateRequest",
        }
        routes = {
            route["id"]: route
            for route in route_data["functions"]
            if route["id"] in expected
        }

        self.assertEqual(set(expected), set(routes))
        self.assertEqual(
            expected,
            {route_id: route["java"]["method"] for route_id, route in routes.items()},
        )

    def test_additional_foreign_route_is_added_once_with_its_requirement_trace(self) -> None:
        route_path = Path("specs/053-api-parity-facade/additional-api-routes.json")
        additions = json.loads(route_path.read_text(encoding="utf-8"))["functions"]

        first = reconcile_0053_api_parity.add_additional_functions(self.manifest, additions)
        second = reconcile_0053_api_parity.add_additional_functions(first, additions)

        self.assertEqual(1, sum(item["id"] == "typed_attribute_from_item" for item in first["functions"]))
        self.assertEqual(first, second)
        self.assertTrue(
            set(additions[0]["requirementIds"]).issubset(first["requirementIds"])
        )

    def test_resolves_only_exact_blocked_0043_declarations_with_four_language_evidence(self) -> None:
        manifest = {
            "requirementIds": [],
            "publicDeclarations": [{
                "declarationId": "rust-validation-report",
                "packageId": "rust-kmipkit",
                "kind": "type",
                "exportName": "kmipkit::protocol::ValidationReport",
                "capabilityId": "public_binding_surface",
                "requirementIds": ["KMIPKIT-0043-FR-021"],
                "declarationState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": "KMIPKIT-0043",
                "blockedReason": "foreign API mapping is pending",
            }],
        }
        resolution = {
            "declarationId": "rust-validation-report",
            "fromSpecId": "KMIPKIT-0043",
            "capabilityId": "request_validation",
            "requirementIds": ["KMIPKIT-0053-FR-006"],
            "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::ValidationReport"},
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_validation_report_t"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ValidationReport"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.validation.ValidationReport"},
            "parityEvidenceIds": ["bindings/c/tests/request_validation_consumer.c"],
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct approval of the Phase 5 implementation plan.",
            },
        }

        result = reconcile_0053_api_parity.resolve_public_declarations(manifest, [resolution])
        replay = reconcile_0053_api_parity.resolve_public_declarations(result, [resolution])

        resolved = result["publicDeclarations"][0]
        self.assertEqual("blocked", manifest["publicDeclarations"][0]["declarationState"])
        self.assertEqual("verified", resolved["declarationState"])
        self.assertEqual("request_validation", resolved["capabilityId"])
        self.assertNotIn("candidateSpecId", resolved)
        self.assertNotIn("blockedReason", resolved)
        self.assertEqual({"rust", "c", "java", "python"}, set(resolved["languageMappings"]))
        self.assertEqual(result, replay)

    def test_refuses_to_transfer_a_declaration_owned_by_any_other_specification(self) -> None:
        manifest = {
            "publicDeclarations": [{
                "declarationId": "unrelated-blocked-export",
                "declarationState": "blocked",
                "candidateSpecId": "KMIPKIT-0010",
            }],
        }
        resolution = {
            "declarationId": "unrelated-blocked-export",
            "fromSpecId": "KMIPKIT-0043",
            "capabilityId": "request_validation",
            "requirementIds": ["KMIPKIT-0053-FR-006"],
            "rust": {"packageId": "rust-kmipkit", "name": "Rust"},
            "c": {"packageId": "c-kmipkit", "name": "C"},
            "java": {"packageId": "java-kmipkit", "name": "Java"},
            "python": {"packageId": "python-kmipkit", "name": "Python"},
            "parityEvidenceIds": ["bindings/c/tests/request_validation_consumer.c"],
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct approval of the Phase 5 implementation plan.",
            },
        }

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.resolve_public_declarations(manifest, [resolution])

    def test_additional_types_are_added_once_with_exact_language_routes(self) -> None:
        manifest = {"requirementIds": [], "types": []}
        addition = {
            "id": "CryptographicParametersFieldsV1",
            "kind": "struct",
            "requirementIds": ["KMIPKIT-0053-FR-004", "KMIPKIT-0053-FR-011"],
            "rust": {"module": "kmipkit_ffi", "name": "CryptographicParametersFieldsV1"},
            "c": {
                "name": "kmipkit_cryptographic_parameters_fields_v1_t",
                "kind": "fixed-width-value",
                "visibility": "public",
                "versioned": True,
                "fields": [{"name": "struct_size", "type": "uint32_t"}],
            },
            "java": {"package": "org.kmipkit.protocol", "name": "CryptographicParameters.Builder"},
            "python": {"module": "kmipkit.cryptographic_parameters", "name": "CryptographicParametersBuilder"},
        }

        first = reconcile_0053_api_parity.add_additional_types(manifest, [addition])
        second = reconcile_0053_api_parity.add_additional_types(first, [addition])

        self.assertEqual(first, second)
        self.assertEqual([addition], first["types"])
        self.assertEqual(addition["requirementIds"], first["requirementIds"])

    def test_additional_type_cannot_redefine_a_public_type(self) -> None:
        manifest = {"requirementIds": [], "types": [{"id": "Existing", "kind": "opaque-handle"}]}
        addition = {"id": "Existing", "kind": "struct"}

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_additional_types(manifest, [addition])

    def test_real_typed_attribute_from_item_route_unblocks_generated_c_surface(self) -> None:
        route_path = Path("specs/053-api-parity-facade/additional-api-routes.json")
        route_data = json.loads(route_path.read_text(encoding="utf-8"))
        candidate_id = "kmipkit-0048-kmipkit-7ff36f3a6017bfcf719b"
        self.assertIn(candidate_id, route_data["candidateRoutes"])

        manifest = reconcile_0053_api_parity.reconcile_manifest(
            self.manifest_for_repository(), route_data["candidateRoutes"]
        )
        manifest = reconcile_0053_api_parity.add_additional_functions(
            manifest, route_data["functions"]
        )
        manifest = reconcile_0053_api_parity.add_public_declarations(
            manifest, route_data["publicDeclarations"]
        )
        mapped = next(row for row in manifest["publicDeclarations"]
                      if row["declarationId"] == candidate_id)
        self.assertEqual("verified", mapped["declarationState"])
        self.assertEqual(len(manifest["publicDeclarations"]),
                         len({row["declarationId"] for row in manifest["publicDeclarations"]}))
        self.assertTrue(any(row["exportName"] == "kmipkit_typed_attribute_from_item"
                            and row["declarationState"] == "verified"
                            for row in manifest["publicDeclarations"]))

    def test_typed_attribute_error_statuses_and_rust_routes_are_specific(self) -> None:
        manifest = self.manifest_for_repository()
        declarations = {row["declarationId"]: row for row in manifest["publicDeclarations"]}
        expected_rust_routes = {
            "typed_attribute_from_value:c": "kmipkit_protocol::Attribute<D>::new",
            "typed_attribute_value:c": "kmipkit_protocol::Attribute<D>::with_value",
            "attribute_set_try_push_typed:c": "kmipkit_protocol::AttributeSet::try_push_typed",
            "attribute_set_typed_count:c": "kmipkit_protocol::AttributeSet::typed",
            "attribute_set_typed_at:c": "kmipkit_protocol::AttributeSet::typed",
        }
        for declaration_id, expected in expected_rust_routes.items():
            with self.subTest(declaration_id=declaration_id):
                route = declarations[declaration_id]["languageMappings"]["rust"]
                self.assertEqual("rust-kmipkit-protocol", route["packageId"])
                self.assertEqual(expected, route["name"])

        errors = {row["id"]: row for row in manifest["errorCategories"]}
        self.assertEqual(
            (12, "KMIPKIT_ERROR_TYPED_ATTRIBUTE_TAG_MISMATCH"),
            (errors["typed_attribute_tag_mismatch"]["cValue"],
             errors["typed_attribute_tag_mismatch"]["cCode"]),
        )
        self.assertEqual(
            (13, "KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH"),
            (errors["typed_attribute_value_type_mismatch"]["cValue"],
             errors["typed_attribute_value_type_mismatch"]["cCode"]),
        )

    @staticmethod
    def manifest_for_repository() -> dict:
        return json.loads(Path("specification/api/public-api.json").read_text(encoding="utf-8"))

    def test_public_four_language_export_is_added_idempotently(self) -> None:
        declarations = [
            {
                "declarationId": "four-language-test-export",
                "packageId": "c-kmipkit",
                "kind": "c-symbol",
                "exportName": "kmipkit_test_export",
                "capabilityId": "typed_object_attributes",
                "requirementIds": ["KMIPKIT-0053-FR-002"],
                "declarationState": "verified",
                "parityScope": "four-language",
                "languageMappings": {
                    "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                    "c": {"packageId": "c-kmipkit", "name": "kmipkit_test_export"},
                    "java": {"packageId": "java-kmipkit", "name": "Test.export"},
                    "python": {"packageId": "python-kmipkit", "name": "kmipkit.test.export"},
                },
                "parityEvidenceIds": ["tests/consumer.c"],
            }
        ]

        first = reconcile_0053_api_parity.add_public_declarations(self.manifest, declarations)
        second = reconcile_0053_api_parity.add_public_declarations(first, declarations)

        self.assertEqual(first, second)
        self.assertEqual(
            "verified",
            next(
                row["declarationState"]
                for row in first["publicDeclarations"]
                if row["declarationId"] == "four-language-test-export"
            ),
        )

    def test_public_export_cannot_claim_parity_with_a_missing_language(self) -> None:
        declarations = [
            {
                "declarationId": "four-language-test-export",
                "packageId": "c-kmipkit",
                "exportName": "kmipkit_test_export",
                "declarationState": "verified",
                "parityScope": "four-language",
                "languageMappings": {
                    "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                    "c": {"packageId": "c-kmipkit", "name": "kmipkit_test_export"},
                    "java": {"packageId": "java-kmipkit", "name": "Test.export"},
                },
                "parityEvidenceIds": ["tests/consumer.c"],
                "requirementIds": ["KMIPKIT-0053-FR-002"],
            }
        ]

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_public_declarations(self.manifest, declarations)

    def test_public_c_layout_field_can_be_recorded_as_an_exact_c_only_surface(self) -> None:
        declaration = {
            "declarationId": "c-only-layout-field",
            "packageId": "c-kmipkit",
            "kind": "field",
            "exportName": "kmipkit_validation_issue_view_v1_t.struct_size",
            "capabilityId": "request_validation",
            "requirementIds": ["KMIPKIT-0053-FR-006"],
            "declarationState": "verified",
            "parityScope": "c-only",
            "languageMappings": {
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_validation_issue_view_v1_t.struct_size",
                }
            },
            "parityEvidenceIds": ["bindings/c/tests/request_validation_consumer.c"],
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "The versioned C carrier field is a C ABI layout detail.",
            },
        }

        result = reconcile_0053_api_parity.add_public_declarations(
            self.manifest, [declaration]
        )

        self.assertIn(declaration, result["publicDeclarations"])


if __name__ == "__main__":
    unittest.main()
