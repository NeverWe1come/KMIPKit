"""The approved KMIP 3.0 fluent operation set must exist in the API matrix."""

from __future__ import annotations

import json
from pathlib import Path
import unittest

from tools.api_manifest.kmip30_draft_api import _validate_fluent_routes

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification/api/public-api.json"
KMIP30_DRAFT_API = ROOT / "specification/api/kmip-3.0-draft-api.json"
OPERATIONS_BY_CODE = {
    60: ("Create Group", "CreateGroup"),
    61: ("Obliterate", "Obliterate"),
    62: ("Create User", "CreateUser"),
    63: ("Create Credential", "CreateCredential"),
    64: ("Deactivate", "Deactivate"),
    65: ("Encapsulate", "Encapsulate"),
    66: ("Decapsulate", "Decapsulate"),
}


class Kmip30FluentManifestTests(unittest.TestCase):
    def test_capability_can_include_method_mappings_beyond_canonical_routes(self) -> None:
        languages = {"rust", "c", "java", "python"}
        names = (
            "CreateGroup", "Obliterate", "CreateUser", "CreateCredential",
            "Deactivate", "Encapsulate", "Decapsulate",
        )
        operations = [
            {
                "operation": name,
                "fluentApi": {language: f"{language}.{name}" for language in languages},
            }
            for name in names
        ]
        mappings = {
            language: [row["fluentApi"][language] for row in operations]
            for language in languages
        }
        capability = {
            "capabilityId": "kmip30_fluent_operations",
            "capabilityState": "verified",
            "ownerSpecId": "KMIPKIT-0116",
            "requirementIds": [f"KMIPKIT-0116-FR-{number:03}" for number in range(1, 7)],
            "approvalEvidence": {
                "kind": "feature-specification",
                "path": "specs/116-kmip30-fluent-operations/spec.md",
                "status": "Approved",
            },
            "parityEvidenceIds": ["bindings/parity-test"],
            "mappings": {
                language: [*mappings[language], f"{language}.Builder.execute"]
                for language in languages
            },
        }

        _validate_fluent_routes(operations, {"capabilities": [capability]})

    def test_kmip30_fluent_mappings_name_real_builder_and_result_exports(self) -> None:
        document = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = document["publicDeclarations"]
        by_package_and_name = {
            (row["packageId"], row["exportName"]): row for row in declarations
        }

        builder_routes = {
            "kmipkit_create_credential_builder_create": (
                "createCredential",
                "create_credential",
            ),
            "kmipkit_create_group_builder_create": ("createGroup", "create_group"),
            "kmipkit_create_user_builder_create": ("createUser", "create_user"),
            "kmipkit_deactivate_builder_create": ("deactivate", "deactivate"),
            "kmipkit_decapsulate_builder_create": ("decapsulate", "decapsulate"),
            "kmipkit_encapsulate_builder_create": ("encapsulate", "encapsulate"),
            "kmipkit_obliterate_builder_create": ("obliterate", "obliterate"),
        }
        for c_export, (java_method, python_method) in builder_routes.items():
            declaration = by_package_and_name[("c-kmipkit", c_export)]
            with self.subTest(c_export=c_export):
                self.assertEqual(
                    f"org.kmipkit.fluent.FluentRequest.{java_method}",
                    declaration["languageMappings"]["java"]["name"],
                )
                self.assertEqual(
                    f"kmipkit.fluent.Request.{python_method}",
                    declaration["languageMappings"]["python"]["name"],
                )

        execute_routes = {
            "CreateCredential": "kmipkit_client_create_credential_v1",
            "CreateGroup": "kmipkit_client_create_group_v1",
            "CreateUser": "kmipkit_client_create_user_v1",
            "Deactivate": "kmipkit_client_deactivate_v1",
            "Decapsulate": "kmipkit_client_decapsulate_v1",
            "Encapsulate": "kmipkit_client_encapsulate_v1",
            "Obliterate": "kmipkit_client_obliterate_v1",
        }
        for operation, c_export in execute_routes.items():
            declaration = by_package_and_name[("c-kmipkit", c_export)]
            with self.subTest(operation=operation):
                self.assertEqual(
                    "org.kmipkit.fluent.Kmip30BuilderBase.execute",
                    declaration["languageMappings"]["java"]["name"],
                )

        for c_export, java_name, python_name in (
            (
                "kmipkit_create_credential_object_hashed_password",
                "org.kmipkit.fluent.CreateCredentialObject.hashedPassword",
                "kmipkit.fluent.CreateCredentialObject.hashed_password",
            ),
            (
                "kmipkit_create_credential_object_one_time_password",
                "org.kmipkit.fluent.CreateCredentialObject.oneTimePassword",
                "kmipkit.fluent.CreateCredentialObject.one_time_password",
            ),
            (
                "kmipkit_client_batch_item_response_encapsulate_data",
                "org.kmipkit.extensions.ClientBatchItemResult.encapsulateData",
                "kmipkit.client.BatchItemResult.encapsulate_data",
            ),
            (
                "kmipkit_client_batch_item_response_kmip30_identifier",
                "org.kmipkit.extensions.ClientBatchItemResult.kmip30Identifier",
                "kmipkit.client.BatchItemResult.kmip30_identifier",
            ),
        ):
            declaration = by_package_and_name[("c-kmipkit", c_export)]
            with self.subTest(c_export=c_export):
                self.assertEqual(java_name, declaration["languageMappings"]["java"]["name"])
                self.assertEqual(python_name, declaration["languageMappings"]["python"]["name"])

        for package_id, export_name in (
            ("java-kmipkit", "org.kmipkit.fluent.CreateCredentialObject"),
            ("python-kmipkit", "kmipkit.fluent.CreateCredentialObject"),
        ):
            declaration = by_package_and_name[(package_id, export_name)]
            with self.subTest(package=package_id):
                self.assertEqual("type", declaration["kind"])

    def test_kmip30_fluent_capability_maps_all_routes_without_changing_21_inventory(self) -> None:
        document = json.loads(MANIFEST.read_text(encoding="utf-8"))
        operations_21 = [
            row for row in document.get("operations", [])
            if row.get("scope") == "client_1_0"
        ]
        self.assertEqual(len(operations_21), 56)

        capabilities = {
            row.get("capabilityId"): row
            for row in document.get("capabilities", [])
        }
        capability = capabilities.get("kmip30_fluent_operations")
        self.assertIsNotNone(capability)
        assert capability is not None
        self.assertEqual(capability.get("ownerSpecId"), "KMIPKIT-0116")
        self.assertEqual(capability.get("capabilityState"), "verified")
        self.assertEqual(capability.get("parityScope"), "four-language")
        self.assertTrue(capability.get("parityEvidenceIds"))
        language_mappings = capability.get("mappings", {})
        self.assertEqual(set(language_mappings), {"rust", "c", "java", "python"})
        self.assertEqual(
            capability.get("parityEvidenceIds", []).count(
                "bindings/java/src/test/java/org/kmipkit/Kmip30FluentExecutionTest.java"
            ),
            1,
        )
        types = {row.get("id"): row for row in document.get("types", [])}
        self.assertEqual(types["CreateCredentialObject"].get("kind"), "enum")
        self.assertEqual(
            types["CreateCredentialObject"]["rust"]["name"],
            "CreateCredentialObject",
        )
        opaque_builder = types["Kmip30OperationBuilder"]
        self.assertEqual(opaque_builder["c"]["kind"], "opaque-handle")
        self.assertNotIn("rust", opaque_builder)
        self.assertNotIn("java", opaque_builder)
        self.assertNotIn("python", opaque_builder)
        c_handle_declaration = next(
            row for row in document["publicDeclarations"]
            if row.get("exportName") == "kmipkit_kmip30_operation_builder_t"
        )
        self.assertEqual(c_handle_declaration.get("parityScope"), "c-only")
        self.assertEqual(set(c_handle_declaration.get("languageMappings", {})), {"c"})
        self.assertNotIn("kmipkit_client::fluent::Kmip30OperationBuilder", language_mappings["rust"])
        self.assertNotIn("kmipkit.fluent.Kmip30OperationBuilder", language_mappings["python"])

        draft = json.loads(KMIP30_DRAFT_API.read_text(encoding="utf-8"))
        routes = {row["operationCode"]: row for row in draft.get("operations", [])}
        self.assertEqual(set(routes), set(OPERATIONS_BY_CODE))
        for code, (name, api_operation_id) in OPERATIONS_BY_CODE.items():
            route = routes[code]
            with self.subTest(operation=name):
                self.assertEqual(route["operation"], api_operation_id)
                mappings = route.get("fluentApi", {})
                self.assertEqual(set(mappings), {"rust", "c", "java", "python"})
                for language, mapping in mappings.items():
                    self.assertIsInstance(mapping, str)
                    self.assertIn(mapping, language_mappings[language])
        direct_c_routes = {
            "kmipkit_client_create_credential_v1",
            "kmipkit_client_create_group_v1",
            "kmipkit_client_create_user_v1",
            "kmipkit_client_deactivate_v1",
            "kmipkit_client_decapsulate_v1",
            "kmipkit_client_encapsulate_v1",
            "kmipkit_client_obliterate_v1",
        }
        self.assertTrue(direct_c_routes.issubset(set(language_mappings["c"])))


if __name__ == "__main__":
    unittest.main()
