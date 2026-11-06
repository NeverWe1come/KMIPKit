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
            "publicPackages": [
                {"id": "rust-kmipkit", "language": "rust"},
                {"id": "c-kmipkit", "language": "c"},
                {"id": "java-kmipkit", "language": "java"},
                {"id": "python-kmipkit", "language": "python"},
            ],
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
                    "mappings": {language: [] for language in ("rust", "c", "java", "python")},
                },
                {
                    "capabilityId": "request_message_authentication",
                    "capabilityState": "blocked",
                    "parityScope": "four-language",
                    "ownerSpecId": "KMIPKIT-0050",
                    "mappings": {language: [] for language in ("rust", "c", "java", "python")},
                },
                {
                    "capabilityId": "request_validation",
                    "capabilityState": "blocked",
                    "parityScope": "c-only",
                    "ownerSpecId": "KMIPKIT-0051",
                    "mappings": {"c": []},
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
        replay = reconcile_0053_api_parity.reconcile_manifest(result, self.routes)

        self.assertEqual("blocked", self.manifest["publicDeclarations"][0]["declarationState"])
        for declaration in result["publicDeclarations"]:
            self.assertEqual("verified", declaration["declarationState"])
            self.assertNotIn("candidateSpecId", declaration)
            self.assertNotIn("blockedReason", declaration)
            self.assertEqual({"rust", "c", "java", "python"}, set(declaration["languageMappings"]))
        capability_states = {
            item["capabilityId"]: item["capabilityState"]
            for item in result["capabilities"]
        }
        self.assertEqual("verified", capability_states["typed_object_attributes"])
        self.assertEqual("verified", capability_states["request_message_authentication"])
        self.assertEqual("blocked", capability_states["request_validation"])
        self.assertEqual(result, replay)
        for original, capability in zip(self.manifest["capabilities"], result["capabilities"], strict=True):
            self.assertEqual(original["ownerSpecId"], capability["ownerSpecId"])
            if capability["capabilityState"] == "verified":
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

    def test_ignores_unrouted_verified_declarations_with_declared_capability(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        declaration = copy.deepcopy(manifest["publicDeclarations"][0])
        declaration.update(
            {
                "declarationId": "previously-verified-unrouted",
                "declarationState": "verified",
                "candidateSpecId": "KMIPKIT-0053",
                "capabilityId": "generic_ttlv_model",
            }
        )
        manifest["publicDeclarations"].append(declaration)
        manifest["capabilities"].append(
            {
                "capabilityId": "generic_ttlv_model",
                "capabilityState": "declared",
                "ownerSpecId": "KMIPKIT-0047",
            }
        )

        result = reconcile_0053_api_parity.reconcile_manifest(manifest, self.routes)

        states = {row["capabilityId"]: row["capabilityState"]
                  for row in result["capabilities"]}
        self.assertEqual("declared", states["generic_ttlv_model"])

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
        self.assertEqual([], candidates)
        resolved = [
            declaration for declaration in manifest["publicDeclarations"]
            if declaration.get("declarationId") == "kmipkit-0048-kmipkit-7ff36f3a6017bfcf719b"
        ]
        self.assertEqual(1, len(resolved))
        self.assertEqual("verified", resolved[0]["declarationState"])
        capabilities = {row["capabilityId"]: row["capabilityState"]
                        for row in manifest["capabilities"]}
        self.assertEqual("verified", capabilities["request_message_authentication"])
        for capability_id in (
            "typed_object_attributes",
            "typed_managed_objects_and_key_material",
            "typed_cryptographic_parameters",
        ):
            self.assertEqual("verified", capabilities[capability_id])

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

        crypto_resolutions = [
            resolution for resolution in route_data["candidateResolutions"]
            if resolution["capabilityId"] == "typed_cryptographic_parameters"
        ]
        self.assertEqual(23, len(crypto_resolutions))
        self.assertTrue(all(row["resolution"] == "mapped" for row in crypto_resolutions))

        reconciled = reconcile_0053_api_parity.reconcile_manifest(
            manifest, route_data["candidateRoutes"]
        )
        reconciled = reconcile_0053_api_parity.reconcile_candidate_resolutions(
            reconciled, route_data["candidateResolutions"]
        )
        reconciled = reconcile_0053_api_parity.resolve_public_declarations(
            reconciled, route_data["declarationResolutions"]
        )
        reconciled = reconcile_0053_api_parity.add_additional_types(
            reconciled, route_data["types"]
        )
        reconciled = reconcile_0053_api_parity.add_additional_functions(
            reconciled, route_data["functions"]
        )
        reconciled = reconcile_0053_api_parity.add_public_declarations(
            reconciled,
            route_data["publicDeclarations"],
            route_data.get("pythonStaleMappings", []),
            route_data.get("javaStaleMappings", []),
            route_data.get("pythonKindCorrections", []),
            route_data.get("rustKindCorrections", []),
            route_data.get("languageMappingCorrections", []),
        )
        reconciled = reconcile_0053_api_parity.add_approved_exclusions(
            reconciled, route_data["approvedExclusions"]
        )
        reconciled = reconcile_0053_api_parity._synchronize_capability_mappings(reconciled)
        remaining = [
            declaration
            for declaration in reconciled["publicDeclarations"]
            if declaration.get("candidateSpecId") == "KMIPKIT-0053"
            and declaration.get("capabilityId") == "typed_cryptographic_parameters"
        ]
        self.assertEqual([], remaining)

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
        declarations = {
            row["declarationId"]: row
            for row in route_data["publicDeclarations"]
            if row["declarationId"].split(":", 1)[0] in expected
        }
        for route_id, method in expected.items():
            exact_name = f"org.kmipkit.ttlv.ManagedObject.{method}"
            mapped_rows = [
                row for declaration_id, row in declarations.items()
                if declaration_id.startswith(f"{route_id}:")
            ]
            self.assertEqual(3, len(mapped_rows))
            self.assertTrue(
                all(row["languageMappings"]["java"]["name"] == exact_name
                    for row in mapped_rows)
            )
            self.assertEqual(
                exact_name,
                declarations[f"{route_id}:java"]["exportName"],
            )
        methods_by_c_symbol = {
            route["c"]["symbol"]: expected[route_id]
            for route_id, route in routes.items()
        }
        for route in route_data["candidateRoutes"].values():
            c_name = route.get("c", {}).get("name")
            if c_name in methods_by_c_symbol:
                self.assertEqual(
                    f"org.kmipkit.ttlv.ManagedObject.{methods_by_c_symbol[c_name]}",
                    route["java"]["name"],
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
            manifest,
            route_data["publicDeclarations"],
            route_data.get("pythonStaleMappings", []),
            route_data.get("javaStaleMappings", []),
            route_data.get("pythonKindCorrections", []),
            route_data.get("rustKindCorrections", []),
            route_data.get("languageMappingCorrections", []),
        )
        mapped = next(row for row in manifest["publicDeclarations"]
                      if row["declarationId"] == candidate_id)
        self.assertEqual("verified", mapped["declarationState"])
        self.assertEqual(len(manifest["publicDeclarations"]),
                         len({row["declarationId"] for row in manifest["publicDeclarations"]}))
        self.assertTrue(any(row["exportName"] == "kmipkit_typed_attribute_from_item"
                            and row["declarationState"] == "verified"
                            for row in manifest["publicDeclarations"]))
        standard_attribute = next(
            row for row in manifest["publicDeclarations"]
            if row.get("exportName") == "KMIPKIT_STANDARD_ATTRIBUTE_ACTIVATION_DATE"
        )
        self.assertEqual("uint32_t", standard_attribute["cValueType"])
        invalid_candidate = copy.deepcopy(
            next(
                row
                for row in route_data["publicDeclarations"]
                if row.get("candidateSpecId") == "KMIPKIT-0053"
            )
        )
        invalid_candidate["candidateSpecId"] = "KMIPKIT-0009"
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_public_declarations(
                self.manifest_for_repository(), [invalid_candidate]
            )

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

    def test_approved_verified_addition_is_not_left_as_an_unresolved_candidate(self) -> None:
        declaration = {
            "declarationId": "approved-candidate-export",
            "packageId": "c-kmipkit",
            "kind": "c-symbol",
            "exportName": "kmipkit_approved_candidate",
            "capabilityId": "typed_object_attributes",
            "requirementIds": ["KMIPKIT-0053-FR-002"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_approved_candidate"},
                "java": {"packageId": "java-kmipkit", "name": "Test.approvedCandidate"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.test.approved_candidate"},
            },
            "parityEvidenceIds": ["tests/consumer.c"],
            "candidateSpecId": "KMIPKIT-0053",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Maintainer-approved four-language parity scope.",
            },
        }

        result = reconcile_0053_api_parity.add_public_declarations(
            self.manifest, [declaration]
        )
        stored = next(
            row for row in result["publicDeclarations"]
            if row["declarationId"] == "approved-candidate-export"
        )

        self.assertNotIn("candidateSpecId", stored)
        self.assertEqual(declaration["approvalSource"], stored["approvalSource"])

    def test_declaration_replay_refreshes_stale_approval_evidence_from_same_record(self) -> None:
        approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
        }
        resolution = {
            "declarationId": "validation-issue-code",
            "fromSpecId": "KMIPKIT-0043",
            "capabilityId": "request_validation",
            "requirementIds": ["KMIPKIT-0053-FR-012"],
            "rust": {"packageId": "rust-kmipkit", "name": "kmipkit::protocol::ValidationIssue::code"},
            "c": {"packageId": "c-kmipkit", "name": "kmipkit_validation_report_issue_at_v1"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ValidationIssue.code"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.validation.ValidationIssue.code"},
            "parityEvidenceIds": ["crates/kmipkit-protocol/src/validation.rs"],
            "approvalSource": approval,
        }
        manifest = copy.deepcopy(self.manifest)
        declaration = {
            "declarationId": "validation-issue-code",
            "packageId": "rust-kmipkit",
            "kind": "associated-item",
            "exportName": "kmipkit::protocol::ValidationIssue::code",
            "capabilityId": "request_validation",
            "requirementIds": resolution["requirementIds"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {key: resolution[key] for key in ("rust", "c", "java", "python")},
            "parityEvidenceIds": resolution["parityEvidenceIds"],
            "approvalSource": {**approval, "evidence": "Direct maintainer instruction approving the complete Phase 5 implementation plan and autonomous implementation."},
        }
        manifest["publicDeclarations"].append(declaration)

        result = reconcile_0053_api_parity.resolve_public_declarations(
            manifest, [resolution]
        )

        stored = next(row for row in result["publicDeclarations"]
                      if row["declarationId"] == "validation-issue-code")
        self.assertEqual(approval, stored["approvalSource"])

    def test_additional_export_replay_refreshes_approval_evidence_from_same_record(self) -> None:
        declaration = {
            "declarationId": "approved-candidate-export",
            "packageId": "c-kmipkit",
            "kind": "c-symbol",
            "exportName": "kmipkit_approved_candidate",
            "capabilityId": "typed_object_attributes",
            "requirementIds": ["KMIPKIT-0053-FR-002"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit", "name": "test::rust"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_approved_candidate"},
                "java": {"packageId": "java-kmipkit", "name": "Test.approvedCandidate"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.test.approved_candidate"},
            },
            "parityEvidenceIds": ["tests/consumer.c"],
            "candidateSpecId": "KMIPKIT-0053",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
            },
        }
        manifest = copy.deepcopy(self.manifest)
        stale = copy.deepcopy(declaration)
        stale["candidateSpecId"] = None
        stale["approvalSource"]["evidence"] = "Direct maintainer instruction approving the complete Phase 5 implementation plan and autonomous implementation."
        manifest["publicDeclarations"].append(stale)

        result = reconcile_0053_api_parity.add_public_declarations(
            manifest, [declaration]
        )

        stored = next(row for row in result["publicDeclarations"]
                      if row["declarationId"] == "approved-candidate-export")
        self.assertEqual(declaration["approvalSource"], stored["approvalSource"])

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

    def test_capability_mappings_are_rebuilt_from_all_ready_declarations(self) -> None:
        packages = [
            {"id": "rust-kmipkit", "language": "rust"},
            {"id": "c-kmipkit", "language": "c"},
            {"id": "java-kmipkit", "language": "java"},
            {"id": "python-kmipkit", "language": "python"},
        ]
        declaration = {
            "declarationId": "verified-attribute-route",
            "capabilityId": "typed_object_attributes",
            "declarationState": "verified",
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit", "name": "Attribute.new"},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_attribute_new"},
                "java": {"packageId": "java-kmipkit", "name": "Attribute.new"},
                "python": {"packageId": "python-kmipkit", "name": "Attribute.new"},
            },
        }
        manifest = {
            "publicPackages": packages,
            "publicDeclarations": [declaration],
            "capabilities": [
                {
                    "capabilityId": "typed_object_attributes",
                    "capabilityState": "verified",
                    "parityScope": "four-language",
                    "mappings": {language: [] for language in ("rust", "c", "java", "python")},
                }
            ],
        }

        result = reconcile_0053_api_parity._synchronize_capability_mappings(manifest)

        self.assertEqual(manifest["publicDeclarations"], result["publicDeclarations"])
        self.assertEqual(
            {language: [declaration["languageMappings"][language]["name"]]
             for language in ("rust", "c", "java", "python")},
            result["capabilities"][0]["mappings"],
        )
        self.assertEqual(
            {language: [] for language in ("rust", "c", "java", "python")},
            manifest["capabilities"][0]["mappings"],
        )

    def test_adds_only_the_exact_internal_CFFI_bridge_exclusion_idempotently(self) -> None:
        c_name, c_reason, c_approval, c_approval_evidence = (
            reconcile_0053_api_parity.CFFI_BRIDGE_EXCLUSION
        )
        exclusion = {
            "packageId": "c-kmipkit",
            "exportName": c_name,
            "reason": c_reason,
            "approvalSource": c_approval,
            "evidence": (
                f"{c_approval_evidence} The exact symbol is called from "
                "bindings/java/native/kmipkit_jni.cpp and bindings/python/src/kmipkit/ttlv.py."
            ),
            "candidateSpecId": "KMIPKIT-0053",
        }
        exclusions = [exclusion]

        first = reconcile_0053_api_parity.add_approved_exclusions(
            {"approvedExclusions": []}, exclusions
        )
        second = reconcile_0053_api_parity.add_approved_exclusions(first, exclusions)

        self.assertEqual(exclusions, first["approvedExclusions"])
        self.assertEqual(first, second)
        broad = copy.deepcopy(exclusions[0])
        broad["exportName"] = "kmipkit_ttlv_value_view_*"
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_approved_exclusions(
                {"approvedExclusions": []}, [broad]
            )

    def test_adds_exact_rust_export_audit_exclusions_with_identity_and_evidence(self) -> None:
        exclusion = {
            "declarationId": "KMIPKIT-0053:exclude:exact-test",
            "packageId": "rust-kmipkit",
            "exportName": "<kmipkit::protocol::Example as core::fmt::Debug>::fmt",
            "reason": "This exact compiler-generated trait implementation is not an independent capability.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer authorization for KMIPKIT-0053.",
            },
            "evidence": "Exact public export audit with its Rustdoc identity.",
            "evidencePaths": ["crates/kmipkit-protocol/tests/managed_objects.rs"],
            "candidateSpecId": "KMIPKIT-0053",
            "rustArtifactKind": "rust-trait-implementation-member",
            "identity": "serialized-rustdoc-identity",
        }

        first = reconcile_0053_api_parity.add_approved_exclusions(
            {"approvedExclusions": []}, [exclusion]
        )
        second = reconcile_0053_api_parity.add_approved_exclusions(first, [exclusion])

        self.assertEqual([exclusion], first["approvedExclusions"])
        self.assertEqual(first, second)
        broad = copy.deepcopy(exclusion)
        broad["exportName"] = "<kmipkit::protocol::*>::fmt"
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_approved_exclusions(
                {"approvedExclusions": []}, [broad]
            )

    def test_corrects_only_the_exact_legacy_model_error_approval_source(self) -> None:
        previous = {
            "declarationId": "KMIPKIT-0053:exclude:legacy-model-error",
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::ttlv::ModelError::UnsupportedValue",
            "reason": "The approved KMIPKIT-0004 policy excludes this local error variant.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/004-generic-ttlv-model/spec.md",
                "status": "Approved",
                "requirementIds": ["KMIPKIT-0004-FR-013"],
            },
            "evidence": "Exact prior approved route evidence.",
            "candidateSpecId": "KMIPKIT-0004",
        }
        corrected = {
            **previous,
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/004-generic-ttlv-model/spec.md",
                "status": "Approved",
                "evidence": "KMIPKIT-0004-FR-013 approves excluding these exact local ModelError variants from the cross-language public route inventory.",
            },
        }

        result = reconcile_0053_api_parity.add_approved_exclusions(
            {"approvedExclusions": [previous]}, [corrected]
        )

        self.assertEqual([corrected], result["approvedExclusions"])

    def test_validates_capability_and_counterpart_for_rust_shape_exclusion(self) -> None:
        exclusion = {
            "declarationId": "KMIPKIT-0053:exclude:enum-variant",
            "capabilityId": "typed_object_attributes",
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::protocol::Attribute::Name",
            "declarationKind": "variant",
            "artifactKind": "rust-enum-variant",
            "reason": "The Rust enum variant is represented by its parent capability.",
            "counterpartDeclarationIds": ["rust-attribute-type"],
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer authorization for KMIPKIT-0053.",
            },
            "evidence": "Exact Rust public export audit.",
            "evidencePaths": ["crates/kmipkit-protocol/tests/unit/typed_attribute_tests.rs"],
            "candidateSpecId": "KMIPKIT-0053",
            "rustArtifactKind": "exact-route",
        }
        manifest = copy.deepcopy(self.manifest)
        manifest["capabilities"][0]["capabilityState"] = "verified"
        manifest["publicDeclarations"][0]["declarationState"] = "verified"

        result = reconcile_0053_api_parity.add_approved_exclusions(
            manifest, [exclusion]
        )

        self.assertEqual([exclusion], result["approvedExclusions"])
        invalid_capability = copy.deepcopy(exclusion)
        invalid_capability["capabilityId"] = "unknown_capability"
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_approved_exclusions(
                manifest, [invalid_capability]
            )
        mismatched_counterpart = copy.deepcopy(exclusion)
        mismatched_counterpart["capabilityId"] = "request_message_authentication"
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            reconcile_0053_api_parity.add_approved_exclusions(
                manifest, [mismatched_counterpart]
            )

    def test_removes_only_an_exact_approved_declaration_idempotently(self) -> None:
        declaration = {
            "declarationId": "0053-counterpart-duplicate-route",
            "packageId": "python-kmipkit",
            "exportName": "kmipkit.managed_objects.KeyWrappingData.from_fields",
        }
        removal = {
            "declarationId": declaration["declarationId"],
            "packageId": declaration["packageId"],
            "exportName": declaration["exportName"],
            "reason": "The duplicate Python constructor belongs to its managed-object capability.",
            "candidateSpecId": "KMIPKIT-0053",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer authorization for the KMIPKIT-0053 plan.",
            },
            "evidencePaths": ["bindings/python/tests/test_managed_object_parity.py"],
        }

        first = reconcile_0053_api_parity.remove_public_declarations(
            {"publicDeclarations": [declaration]}, [removal]
        )
        second = reconcile_0053_api_parity.remove_public_declarations(first, [removal])

        self.assertEqual([], first["publicDeclarations"])
        self.assertEqual(first, second)

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

    def test_corrects_only_an_exact_approved_mapping_to_a_listed_export(self) -> None:
        manifest = {
            "publicPackages": [
                {"id": "rust-kmipkit", "language": "rust"},
                {"id": "rust-kmipkit-protocol", "language": "rust"},
            ],
            "publicDeclarations": [
                {
                    "declarationId": "reader-facade",
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "kmipkit::protocol::ManagedValueReader::read_field",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0049-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "four-language",
                    "languageMappings": {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": "kmipkit_protocol::ManagedValueReader::read_field",
                        }
                    },
                },
                {
                    "declarationId": "protocol-reader",
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "associated-item",
                    "exportName": "kmipkit_protocol::ManagedValueReader::read_field",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0049-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "rust-only-adr",
                },
            ],
        }
        correction = {
            "declarationId": "reader-facade",
            "language": "rust",
            "from": {
                "packageId": "rust-kmipkit",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "to": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-012"],
            "evidencePaths": ["crates/kmipkit-protocol/src/managed_value_reader.rs"],
            "reason": "The exact audited Rust item is exported by the protocol crate package.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer approval of the complete Phase 5 plan.",
            },
        }

        result = reconcile_0053_api_parity.apply_language_mapping_corrections(
            manifest, [correction]
        )
        replay = reconcile_0053_api_parity.apply_language_mapping_corrections(
            result, [correction]
        )

        source = next(
            row for row in result["publicDeclarations"]
            if row["declarationId"] == "reader-facade"
        )
        self.assertEqual(correction["to"], source["languageMappings"]["rust"])
        self.assertIn("KMIPKIT-0053-FR-012", source["requirementIds"])
        self.assertNotIn("candidateSpecId", source)
        self.assertEqual(result, replay)

    def test_mapping_correction_rejects_an_unlisted_target(self) -> None:
        manifest = {
            "publicPackages": [
                {"id": "rust-kmipkit", "language": "rust"},
                {"id": "rust-kmipkit-protocol", "language": "rust"},
            ],
            "publicDeclarations": [
                {
                    "declarationId": "reader-facade",
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "kmipkit::protocol::ManagedValueReader::read_field",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0049-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "four-language",
                    "languageMappings": {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": "kmipkit_protocol::ManagedValueReader::read_field",
                        }
                    },
                }
            ],
        }
        correction = {
            "declarationId": "reader-facade",
            "language": "rust",
            "from": {
                "packageId": "rust-kmipkit",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "to": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-012"],
            "evidencePaths": ["crates/kmipkit-protocol/src/managed_value_reader.rs"],
            "reason": "The exact audited Rust item is exported by the protocol crate package.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer approval of the complete Phase 5 plan.",
            },
        }

        with self.assertRaisesRegex(
            reconcile_0053_api_parity.ReconciliationError,
            "exact listed export",
        ):
            reconcile_0053_api_parity.apply_language_mapping_corrections(
                manifest, [correction]
            )

    def test_mapping_correction_can_record_exact_native_release_semantics(self) -> None:
        manifest = {
            "publicPackages": [
                {"id": "rust-kmipkit", "language": "rust"},
                {"id": "rust-kmipkit-protocol", "language": "rust"},
            ],
            "publicDeclarations": [
                {
                    "declarationId": "managed-object-close",
                    "packageId": "rust-kmipkit",
                    "kind": "java-member",
                    "exportName": "org.kmipkit.ttlv.ManagedObject.close",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0049-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "four-language",
                    "languageMappings": {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": "core::mem::drop",
                        }
                    },
                },
                {
                    "declarationId": "managed-object-type",
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "type",
                    "exportName": "kmipkit_protocol::ManagedObject",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "declarationState": "verified",
                },
            ],
        }
        correction = {
            "declarationId": "managed-object-close",
            "language": "rust",
            "from": {"packageId": "rust-kmipkit", "name": "core::mem::drop"},
            "to": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedObject",
            },
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-012"],
            "evidencePaths": ["crates/kmipkit-protocol/src/managed_object.rs"],
            "reason": "Rust releases this owned managed-object value when it is dropped.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer approval of the complete Phase 5 plan.",
            },
            "mappingSemantics": "native-handle-release",
        }

        result = reconcile_0053_api_parity.apply_language_mapping_corrections(
            manifest, [correction]
        )

        declaration = result["publicDeclarations"][0]
        self.assertEqual(
            correction["to"], declaration["languageMappings"]["rust"]
        )
        self.assertEqual("native-handle-release", declaration["mappingSemantics"])

    def test_public_route_replay_accepts_only_the_matching_approved_mapping_correction(self) -> None:
        manifest = {
            "publicPackages": [
                {"id": "rust-kmipkit", "language": "rust"},
                {"id": "rust-kmipkit-protocol", "language": "rust"},
                {"id": "c-kmipkit", "language": "c"},
                {"id": "java-kmipkit", "language": "java"},
                {"id": "python-kmipkit", "language": "python"},
            ],
            "capabilities": [{
                "capabilityId": "typed_managed_objects_and_key_material",
                "parityScope": "four-language",
                "mappings": {"rust": [], "c": [], "java": [], "python": []},
                "requirementIds": [],
                "parityEvidenceIds": [],
            }],
            "publicDeclarations": [
                {
                    "declarationId": "reader-facade",
                    "packageId": "rust-kmipkit",
                    "kind": "associated-item",
                    "exportName": "kmipkit::protocol::ManagedValueReader::read_field",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "requirementIds": ["KMIPKIT-0049-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "four-language",
                    "languageMappings": {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": "kmipkit_protocol::ManagedValueReader::read_field",
                        },
                        "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_read"},
                        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedValueReader.readField"},
                        "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedValueReader.read_field"},
                    },
                    "parityEvidenceIds": ["crates/kmipkit-protocol/src/managed_value_reader.rs"],
                    "approvalSource": {
                        "kind": "feature-specification",
                        "path": "specs/053-api-parity-facade/approval-record.md",
                        "status": "Approved",
                        "evidence": "Direct maintainer approval of the complete Phase 5 plan.",
                    },
                },
                {
                    "declarationId": "protocol-reader",
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "associated-item",
                    "exportName": "kmipkit_protocol::ManagedValueReader::read_field",
                    "capabilityId": "typed_managed_objects_and_key_material",
                    "declarationState": "verified",
                },
            ],
        }
        declaration = {
            **manifest["publicDeclarations"][0],
            "requirementIds": ["KMIPKIT-0049-FR-001", "KMIPKIT-0053-FR-012"],
            "languageMappings": {
                "rust": {
                    "packageId": "rust-kmipkit-protocol",
                    "name": "kmipkit_protocol::ManagedValueReader::read_field",
                },
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_read"},
                "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedValueReader.readField"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.managed_objects.ManagedValueReader.read_field"},
            },
        }
        correction = {
            "declarationId": "reader-facade",
            "language": "rust",
            "from": {
                "packageId": "rust-kmipkit",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "to": {
                "packageId": "rust-kmipkit-protocol",
                "name": "kmipkit_protocol::ManagedValueReader::read_field",
            },
            "capabilityId": "typed_managed_objects_and_key_material",
            "requirementIds": ["KMIPKIT-0053-FR-012"],
            "evidencePaths": ["crates/kmipkit-protocol/src/managed_value_reader.rs"],
            "reason": "The facade mapping resolves to the exact protocol crate export.",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": "Direct maintainer approval of the complete Phase 5 plan.",
            },
        }

        result = reconcile_0053_api_parity.add_public_declarations(
            manifest, [declaration], language_mapping_corrections=[correction]
        )

        self.assertEqual(
            correction["to"],
            result["publicDeclarations"][0]["languageMappings"]["rust"],
        )

    def test_removes_only_exact_query_exclusions_superseded_by_approved_0053_scope(self) -> None:
        old_exclusion = {
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::protocol::QueryResponse",
            "reason": (
                "This exact Rust Query response type/member provides typed field access. "
                "KMIPKIT-0020 excludes adding its Rust-specific typed Query response model "
                "to C, Java, or Python."
            ),
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/020-query-ping/spec.md",
                "status": "Approved",
                "evidence": "Earlier Rust-only Query response boundary.",
            },
            "evidence": "Exact QueryResponse route exclusion.",
            "candidateSpecId": "KMIPKIT-0020",
        }
        unrelated_exclusion = {
            "packageId": "rust-kmipkit",
            "exportName": "<T as core::fmt::Debug>::fmt",
            "candidateSpecId": "KMIPKIT-0053",
        }
        manifest = {
            "approvedExclusions": [old_exclusion, unrelated_exclusion],
            "publicDeclarations": [{
                "declarationId": "KMIPKIT-0053:rust-query-response",
                "packageId": "rust-kmipkit",
                "exportName": "kmipkit::protocol::QueryResponse",
                "candidateSpecId": "KMIPKIT-0053",
            "declarationState": "verified",
            }],
        }
        removal = {
            "declarationId": "KMIPKIT-0053:rust-query-response",
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::protocol::QueryResponse",
            "supersedesSpecId": "KMIPKIT-0020",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": (
                    "KMIPKIT-0053 approves typed results and final Rust/C/Java/Python parity, "
                    "superseding only the prior KMIPKIT-0020 Rust-only exclusions for the exact "
                    "QueryResponse and QueryResponseField declarations listed in "
                    "additional-api-routes.json."
                ),
            },
            "evidencePaths": [
                "specs/053-api-parity-facade/spec.md",
                "specs/053-api-parity-facade/approval-record.md",
            ],
        }

        result = reconcile_0053_api_parity.remove_approved_exclusions(
            manifest, [removal]
        )

        self.assertEqual([unrelated_exclusion], result["approvedExclusions"])
        replay = reconcile_0053_api_parity.remove_approved_exclusions(
            result, [removal]
        )
        self.assertEqual(result, replay)

    def test_query_exclusion_supersession_requires_exact_owner_and_route(self) -> None:
        manifest = {
            "approvedExclusions": [{
                "packageId": "rust-kmipkit",
                "exportName": "kmipkit::protocol::QueryResponse",
                "reason": "KMIPKIT-0020 excludes its Rust-specific typed Query response model.",
                "approvalSource": {
                    "kind": "feature-specification",
                    "path": "specs/020-query-ping/spec.md",
                    "status": "Approved",
                    "evidence": "Earlier scope.",
                },
                "evidence": "Exact route.",
                "candidateSpecId": "KMIPKIT-0020",
            }],
            "publicDeclarations": [{
                "declarationId": "KMIPKIT-0053:rust-query-response",
                "packageId": "rust-kmipkit",
                "exportName": "kmipkit::protocol::QueryResponse",
            }],
        }
        removal = {
            "declarationId": "KMIPKIT-0053:rust-query-response",
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::protocol::QueryResponse",
            "supersedesSpecId": "KMIPKIT-0007",
            "approvalSource": {
                "kind": "feature-specification",
                "path": "specs/053-api-parity-facade/approval-record.md",
                "status": "Approved",
                "evidence": (
                    "KMIPKIT-0053 approves typed results and final Rust/C/Java/Python parity, "
                    "superseding only the prior KMIPKIT-0020 Rust-only exclusions for the exact "
                    "QueryResponse and QueryResponseField declarations listed in "
                    "additional-api-routes.json."
                ),
            },
            "evidencePaths": [
                "specs/053-api-parity-facade/spec.md",
                "specs/053-api-parity-facade/approval-record.md",
            ],
        }

        with self.assertRaisesRegex(
            reconcile_0053_api_parity.ReconciliationError,
            "exact KMIPKIT-0020 exclusion",
        ):
            reconcile_0053_api_parity.remove_approved_exclusions(
                manifest, [removal]
            )

    def test_verified_0053_export_is_not_an_unresolved_candidate_on_replay(self) -> None:
        manifest = {
            "publicDeclarations": [{
                "declarationId": "already-imported-export",
                "capabilityId": "typed_managed_objects_and_key_material",
                "candidateSpecId": "KMIPKIT-0053",
                "declarationState": "verified",
            }],
            "capabilities": [{
                "capabilityId": "typed_managed_objects_and_key_material",
                "capabilityState": "verified",
            }],
        }

        reconcile_0053_api_parity._finalize_candidate_capabilities(
            manifest,
            [{"capabilityId": "typed_managed_objects_and_key_material"}],
        )

        self.assertEqual(
            "verified", manifest["capabilities"][0]["capabilityState"]
        )

    def test_reconciliation_replay_accepts_verified_candidates_on_verified_capabilities(self) -> None:
        manifest = {
            "publicDeclarations": [{
                "declarationId": "already-imported-export",
                "capabilityId": "typed_managed_objects_and_key_material",
                "candidateSpecId": "KMIPKIT-0053",
                "declarationState": "verified",
            }],
            "capabilities": [{
                "capabilityId": "typed_managed_objects_and_key_material",
                "capabilityState": "verified",
            }],
            "approvedExclusions": [],
        }

        result = reconcile_0053_api_parity.reconcile_candidate_resolutions(
            manifest, []
        )

        self.assertEqual(manifest, result)


if __name__ == "__main__":
    unittest.main()
