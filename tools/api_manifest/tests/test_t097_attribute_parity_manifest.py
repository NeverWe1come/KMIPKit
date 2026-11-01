"""T097 contract for KMIPKIT-0016 attribute-operation declarations."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from tools.api_manifest import generate as api_manifest_generator


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0016"
BASE_CANDIDATE_COUNT = 880
BASE_CANDIDATE_SHA256 = "4aca129ebaa89b347c0fedfc7453408b0f4222d9737cc6f355ec568178fd00b9"
BASE_TRAIT_COUNT = 524
BASE_TRAIT_SHA256 = "86e73fe96baf9ee1249010f7cfa8d927e18d22cbf48fb6a86df4a87b8cfe2554"
MAPPED_COUNT = 219
EXCLUDED_COUNT = 661
OWNER_SPEC_PATH = "specs/016-attribute-operations/spec.md"
SHARED_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
LANGUAGES = {"rust", "c", "java", "python"}
OPERATION_REQUIREMENTS = {
    "add_attribute": "KMIPKIT-0016-FR-003",
    "adjust_attribute": "KMIPKIT-0016-FR-004",
    "delete_attribute": "KMIPKIT-0016-FR-005",
    "get_attributes": "KMIPKIT-0016-FR-006",
    "get_attribute_list": "KMIPKIT-0016-FR-007",
    "modify_attribute": "KMIPKIT-0016-FR-008",
    "set_attribute": "KMIPKIT-0016-FR-009",
}
MUTATION_OPERATIONS = {
    "add_attribute",
    "adjust_attribute",
    "delete_attribute",
    "modify_attribute",
    "set_attribute",
}
REQUEST_MODEL_METHODS = {
    "add_attribute": ("unique_identifier", "new_attribute", "to_ttlv_payload"),
    "adjust_attribute": (
        "unique_identifier",
        "attribute_reference",
        "adjustment_type",
        "adjustment_value",
        "to_ttlv_payload",
    ),
    "delete_attribute": (
        "unique_identifier",
        "current_attribute",
        "attribute_reference",
        "to_ttlv_payload",
    ),
    "modify_attribute": (
        "unique_identifier",
        "current_attribute",
        "new_attribute",
        "to_ttlv_payload",
    ),
    "set_attribute": ("unique_identifier", "new_attribute", "to_ttlv_payload"),
}
READ_REQUEST_MODEL_METHODS = {
    "get_attributes": (
        "unique_identifier",
        "attribute_reference_count",
        "to_ttlv_payload",
        "to_batch_item",
        "attribute_reference_at",
    ),
    "get_attribute_list": (
        "unique_identifier",
        "to_ttlv_payload",
        "to_batch_item",
    ),
}


def _identity_hash(rows: list[dict[str, object]]) -> str:
    identities = sorted(f"{row['packageId']}|{row['exportName']}" for row in rows)
    return hashlib.sha256("\n".join(identities).encode("utf-8")).hexdigest()


def _expected_mapped_exports() -> set[str]:
    operations = tuple(OPERATION_REQUIREMENTS)
    mapped = {
        f"kmipkit::ClientRequest::{operation}"
        for operation in operations
    } | {
        f"kmipkit_client::ClientResponseView::{operation}"
        for operation in operations
    }
    for module in ("kmipkit_protocol::attribute::", "kmipkit_protocol::"):
        mapped.add(f"{module}AttributeSet")
        mapped.update(
            f"{module}AttributeSet::{method}"
            for method in (
                "try_new",
                "try_push",
                "as_items",
                "len",
                "is_empty",
                "into_items",
            )
        )
        for attribute_type in ("CurrentAttribute", "NewAttribute"):
            mapped.add(f"{module}{attribute_type}")
            mapped.update(
                f"{module}{attribute_type}::{method}"
                for method in ("new", "item")
            )
    mapped.add("kmipkit_protocol::AdjustmentType")
    mapped.update(
        f"kmipkit_protocol::AdjustmentType::{member}"
        for member in ("INCREMENT", "DECREMENT", "NEGATE", "from_raw", "raw")
    )
    request_types = {
        "add_attribute": "AddAttributeRequest",
        "adjust_attribute": "AdjustAttributeRequest",
        "delete_attribute": "DeleteAttributeRequest",
        "modify_attribute": "ModifyAttributeRequest",
        "set_attribute": "SetAttributeRequest",
    }
    for operation, request_type in request_types.items():
        mapped.add(f"kmipkit_protocol::{request_type}")
        mapped.add(f"kmipkit_protocol::{request_type}::new")
        response_type = request_type.replace("Request", "Response")
        mapped.add(f"kmipkit_protocol::{response_type}::result")
        class_name = request_type
        c_model = f"kmipkit_{operation}_request_t"
        java_model = f"org.kmipkit.extensions.{class_name}"
        python_model = f"kmipkit.attribute_operations.{class_name}"
        mapped.update({c_model, java_model, python_model})
        for method in ("new", *REQUEST_MODEL_METHODS[operation]):
            if method == "new":
                mapped.update(
                    {
                        f"kmipkit_{operation}_request_create",
                        f"{java_model}.create",
                        f"{python_model}.create",
                    }
                )
                continue
            java_method = "".join(
                part.title() if index else part
                for index, part in enumerate(method.split("_"))
            )
            mapped.update(
                {
                    f"kmipkit_protocol::{request_type}::{method}",
                    f"kmipkit_{operation}_request_{method}",
                    f"{java_model}.{java_method}",
                    f"{python_model}.{method}",
                }
            )
        mapped.update(
            {
                f"kmipkit_{operation}_request_release",
                f"{java_model}.close",
                f"{python_model}.close",
                f"kmipkit_client_batch_item_{operation}_from_request",
                f"{java_model}.toBatchItem",
                f"{python_model}.to_batch_item",
            }
        )
    for operation in READ_REQUEST_MODEL_METHODS:
        class_name = "".join(part.title() for part in operation.split("_")) + "Request"
        java_model = f"org.kmipkit.extensions.{class_name}"
        python_model = f"kmipkit.attribute_operations.{class_name}"
        mapped.update(
            {
                java_model,
                f"{java_model}.create",
                f"{java_model}.close",
                f"{java_model}.uniqueIdentifier",
                f"{java_model}.toTtlvPayload",
                f"{java_model}.toBatchItem",
                python_model,
                f"{python_model}.create",
                f"{python_model}.close",
                f"{python_model}.unique_identifier",
                f"{python_model}.to_ttlv_payload",
                f"{python_model}.to_batch_item",
            }
        )
        if operation == "get_attributes":
            mapped.update(
                {
                    f"{java_model}.attributeReferences",
                    f"{python_model}.attribute_references",
                }
            )
    return mapped


def _expected_owner_requirements(export_name: str) -> set[str]:
    for operation, requirement in OPERATION_REQUIREMENTS.items():
        class_name = "".join(part.title() for part in operation.split("_")) + "Request"
        if (
            f"::{operation}" in export_name
            or class_name in export_name
            or f"_{operation}_request" in export_name
            or f"_{operation}_from_request" in export_name
        ):
            return {
                "KMIPKIT-0016-FR-001",
                "KMIPKIT-0016-FR-002",
                requirement,
                "KMIPKIT-0016-FR-010",
                "KMIPKIT-0016-FR-011",
                "KMIPKIT-0016-FR-012",
                "KMIPKIT-0016-FR-013",
                *({"KMIPKIT-0016-FR-015"} if operation in MUTATION_OPERATIONS else set()),
            }
    if "AttributeSet" in export_name:
        return {
            "KMIPKIT-0016-FR-001",
            "KMIPKIT-0016-FR-002",
            "KMIPKIT-0016-FR-006",
            "KMIPKIT-0016-FR-011",
            "KMIPKIT-0016-FR-013",
        }
    if "CurrentAttribute" in export_name or "NewAttribute" in export_name:
        return {
            "KMIPKIT-0016-FR-001",
            "KMIPKIT-0016-FR-002",
            "KMIPKIT-0016-FR-011",
            "KMIPKIT-0016-FR-005" if "CurrentAttribute" in export_name else "KMIPKIT-0016-FR-003",
        }
    if "AdjustmentType" in export_name:
        return {
            "KMIPKIT-0016-FR-001",
            "KMIPKIT-0016-FR-002",
            "KMIPKIT-0016-FR-004",
            "KMIPKIT-0016-FR-010",
            "KMIPKIT-0016-FR-011",
            "KMIPKIT-0016-FR-012",
            "KMIPKIT-0016-FR-013",
            "KMIPKIT-0016-FR-015",
        }
    for operation, request_type in {
        "add_attribute": "AddAttribute",
        "adjust_attribute": "AdjustAttribute",
        "delete_attribute": "DeleteAttribute",
        "modify_attribute": "ModifyAttribute",
        "set_attribute": "SetAttribute",
    }.items():
        if request_type in export_name:
            return {
                "KMIPKIT-0016-FR-001",
                "KMIPKIT-0016-FR-002",
                OPERATION_REQUIREMENTS[operation],
                "KMIPKIT-0016-FR-010",
                "KMIPKIT-0016-FR-011",
                "KMIPKIT-0016-FR-012",
                "KMIPKIT-0016-FR-013",
                *({"KMIPKIT-0016-FR-015"} if operation in MUTATION_OPERATIONS else set()),
            }
    return set()


class T097AttributeParityManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_base_candidate_inventory_is_pinned(self) -> None:
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC_PATH
        ]
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC_PATH
        ]
        candidates = mapped + excluded
        self.assertEqual(BASE_CANDIDATE_COUNT, len(candidates))
        self.assertEqual(BASE_CANDIDATE_SHA256, _identity_hash(candidates))
        traits = [row for row in excluded if row["exportName"].startswith("<")]
        self.assertEqual(BASE_TRAIT_COUNT, len(traits))
        self.assertEqual(BASE_TRAIT_SHA256, _identity_hash(traits))

    def test_no_attribute_owner_candidates_remain_blocked(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual(
            0,
            len(unresolved),
            "all exact KMIPKIT-0016 exports must be mapped or specifically excluded; "
            f"unresolved={[(row['packageId'], row['exportName']) for row in unresolved[:8]]}",
        )

    def test_attribute_candidates_are_exactly_mapped_or_excluded(self) -> None:
        mapped = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC_PATH
        ]
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC_PATH
        ]
        resolved = mapped + excluded
        self.assertEqual(MAPPED_COUNT, len(mapped))
        self.assertEqual(EXCLUDED_COUNT, len(excluded))
        self.assertEqual(BASE_CANDIDATE_COUNT, len(resolved))
        self.assertEqual(BASE_CANDIDATE_SHA256, _identity_hash(resolved))
        self.assertEqual(
            _expected_mapped_exports(),
            {row["exportName"] for row in mapped},
        )

        available_targets = {
            (row["packageId"], row["exportName"])
            for row in self.manifest["publicDeclarations"]
        }
        for row in mapped:
            with self.subTest(export=row["exportName"]):
                self.assertEqual("declared", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                self.assertTrue(SHARED_REQUIREMENTS <= set(row["requirementIds"]))
                self.assertEqual(
                    _expected_owner_requirements(row["exportName"]),
                    {
                        requirement
                        for requirement in row["requirementIds"]
                        if requirement.startswith(f"{OWNER_SPEC}-FR-")
                    },
                )
                for language, target in row["languageMappings"].items():
                    self.assertIn((target["packageId"], target["name"]), available_targets)

        trait_exclusions = [
            row for row in excluded if row["exportName"].startswith("<")
        ]
        self.assertEqual(BASE_TRAIT_COUNT, len(trait_exclusions))
        self.assertEqual(BASE_TRAIT_SHA256, _identity_hash(trait_exclusions))
        for row in excluded:
            with self.subTest(exclusion=row["exportName"]):
                self.assertTrue(row["reason"].strip())
                self.assertTrue(row["evidence"].strip())
                self.assertEqual("Approved", row["approvalSource"]["status"])
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])

    def test_request_model_declarations_map_to_standalone_typed_models(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
        }
        for operation, methods in REQUEST_MODEL_METHODS.items():
            class_name = "".join(part.title() for part in operation.split("_")) + "Request"
            rust_type = f"kmipkit_protocol::{class_name}"
            targets = {
                "c": f"kmipkit_{operation}_request_t",
                "java": f"org.kmipkit.extensions.{class_name}",
                "python": f"kmipkit.attribute_operations.{class_name}",
            }
            with self.subTest(declaration=rust_type):
                self.assertTrue(rust_type in declarations, f"missing declaration {rust_type}")
                if rust_type in declarations:
                    self.assertEqual(
                        targets,
                        {
                            language: declarations[rust_type]["languageMappings"][language]["name"]
                            for language in targets
                        },
                        "the Rust request type must map to a standalone request model, "
                        "not a high-level batch constructor",
                    )

            method_targets = {
                "new": (
                    f"kmipkit_{operation}_request_create",
                    f"org.kmipkit.extensions.{class_name}.create",
                    f"kmipkit.attribute_operations.{class_name}.create",
                ),
                **{
                    method: (
                        f"kmipkit_{operation}_request_{method}",
                        f"org.kmipkit.extensions.{class_name}.{''.join(part.title() if index else part for index, part in enumerate(method.split('_')))}",
                        f"kmipkit.attribute_operations.{class_name}.{method}",
                    )
                    for method in methods
                },
            }
            for method, expected_targets in method_targets.items():
                rust_export = f"{rust_type}::{method}"
                with self.subTest(declaration=rust_export):
                    self.assertTrue(
                        rust_export in declarations,
                        f"missing standalone mapping for {rust_export}",
                    )
                    if rust_export in declarations:
                        self.assertEqual(
                            expected_targets,
                            tuple(
                                declarations[rust_export]["languageMappings"][language]["name"]
                                for language in ("c", "java", "python")
                            ),
                            "typed request methods must map to standalone C, Java, and Python APIs",
                        )

    def test_read_request_models_map_to_standalone_typed_models(self) -> None:
        declarations = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        models = {
            "GetAttributesRequest": {
                "constructor": "try_new",
                "c_prefix": "kmipkit_get_attributes_request",
                "methods": {
                    "unique_identifier": "unique_identifier",
                    "attribute_references": "attribute_reference_count",
                    "to_ttlv_payload": "to_ttlv_payload",
                },
            },
            "GetAttributeListRequest": {
                "constructor": "new",
                "c_prefix": "kmipkit_get_attribute_list_request",
                "methods": {
                    "unique_identifier": "unique_identifier",
                    "to_ttlv_payload": "to_ttlv_payload",
                },
            },
        }
        for class_name, contract in models.items():
            rust_type = f"kmipkit_protocol::{class_name}"
            java_model = f"org.kmipkit.extensions.{class_name}"
            python_model = f"kmipkit.attribute_operations.{class_name}"
            with self.subTest(model=class_name):
                self.assertEqual(
                    {
                        "c": f"{contract['c_prefix']}_t",
                        "java": java_model,
                        "python": python_model,
                    },
                    {
                        language: declarations[rust_type]["languageMappings"][language]["name"]
                        for language in ("c", "java", "python")
                    },
                    "read requests must map to standalone typed models",
                )
                constructor = f"{rust_type}::{contract['constructor']}"
                self.assertEqual(
                    (
                        f"{contract['c_prefix']}_create",
                        f"{java_model}.create",
                        f"{python_model}.create",
                    ),
                    tuple(
                        declarations.get(constructor, {})
                        .get("languageMappings", {})
                        .get(language, {})
                        .get("name")
                        for language in ("c", "java", "python")
                    ),
                )
                for method, c_suffix in contract["methods"].items():
                    export = f"{rust_type}::{method}"
                    java_method = "".join(
                        part.title() if index else part
                        for index, part in enumerate(method.split("_"))
                    )
                    with self.subTest(export=export):
                        self.assertEqual(
                            (
                                f"{contract['c_prefix']}_{c_suffix}",
                                f"{java_model}.{java_method}",
                                f"{python_model}.{method}",
                            ),
                            tuple(
                                declarations.get(export, {})
                                .get("languageMappings", {})
                                .get(language, {})
                                .get("name")
                                for language in ("c", "java", "python")
                            ),
                        )

    def test_attribute_set_maps_to_validated_collection_facades(self) -> None:
        declarations = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        expected = {
            "type": (
                "kmipkit_attribute_set_t",
                "org.kmipkit.extensions.AttributeSet",
                "kmipkit.attribute_operations.AttributeSet",
            ),
            "kmipkit_attribute_set_create": (
                "try_new",
                "org.kmipkit.extensions.AttributeSet.tryNew",
                "kmipkit.attribute_operations.AttributeSet.try_new",
            ),
            "kmipkit_attribute_set_try_push": (
                "try_push",
                "org.kmipkit.extensions.AttributeSet.tryPush",
                "kmipkit.attribute_operations.AttributeSet.try_push",
            ),
            "kmipkit_attribute_set_as_items": (
                "as_items",
                "org.kmipkit.extensions.AttributeSet.asItems",
                "kmipkit.attribute_operations.AttributeSet.as_items",
            ),
            "kmipkit_attribute_set_len": (
                "len",
                "org.kmipkit.extensions.AttributeSet.len",
                "kmipkit.attribute_operations.AttributeSet.len",
            ),
            "kmipkit_attribute_set_is_empty": (
                "is_empty",
                "org.kmipkit.extensions.AttributeSet.isEmpty",
                "kmipkit.attribute_operations.AttributeSet.is_empty",
            ),
            "kmipkit_attribute_set_into_items": (
                "into_items",
                "org.kmipkit.extensions.AttributeSet.intoItems",
                "kmipkit.attribute_operations.AttributeSet.into_items",
            ),
        }
        for module in ("kmipkit_protocol::", "kmipkit_protocol::attribute::"):
            type_export = f"{module}AttributeSet"
            type_row = declarations.get(type_export)
            with self.subTest(export=type_export):
                self.assertIsNotNone(type_row, f"missing {type_export}")
                if type_row is not None:
                    self.assertEqual(
                        {
                            "c": expected["type"][0],
                            "java": expected["type"][1],
                            "python": expected["type"][2],
                        },
                        {
                            language: type_row["languageMappings"][language]["name"]
                            for language in ("c", "java", "python")
                        },
                    )
            for c_name, (method, java_name, python_name) in expected.items():
                if c_name == "type":
                    continue
                export = f"{type_export}::{method}"
                with self.subTest(export=export):
                    row = declarations.get(export)
                    self.assertIsNotNone(
                        row,
                        f"{export} must map to a validated collection facade",
                    )
                    if row is None:
                        continue
                    self.assertEqual(
                        {
                            "c": c_name,
                            "java": java_name,
                            "python": python_name,
                        },
                        {
                            language: row["languageMappings"][language]["name"]
                            for language in ("c", "java", "python")
                        },
                    )

    def test_get_attributes_c_abi_uses_a_bounded_versioned_reference_array(self) -> None:
        create = next(
            row for row in self.manifest["functions"]
            if row["id"] == "attribute_request_get_attributes_create"
        )
        parameters = create["c"]["parameters"]
        self.assertEqual(
            {
                "name": "attribute_references",
                "type": "const kmipkit_attribute_reference_v1_t *",
                "kind": "input-array",
                "elementType": "kmipkit_attribute_reference_v1_t",
                "ownership": "borrowed",
                "countParameter": "attribute_reference_count",
                "nullable": True,
            },
            parameters[1],
        )
        self.assertEqual("uint64_t", parameters[2]["type"])
        self.assertEqual("attribute_reference_count", parameters[1]["countParameter"])
        api_manifest_generator._validate_references(self.manifest)

    def test_read_request_functions_use_the_expected_java_jni_bridges(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        expected = {
            "attribute_request_get_attributes_create": (
                "Java_org_kmipkit_extensions_NativeAttributeRequests_create"
            ),
            "attribute_request_get_attribute_list_create": (
                "Java_org_kmipkit_extensions_NativeAttributeRequests_create"
            ),
            "attribute_request_get_attributes_to_ttlv_payload": (
                "Java_org_kmipkit_NativeExtensionRegistry_attributeRequestToTtlvPayload"
            ),
            "attribute_request_get_attributes_to_batch_item": (
                "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemFromAttributeRequest"
            ),
            "attribute_request_get_attribute_list_to_ttlv_payload": (
                "Java_org_kmipkit_NativeExtensionRegistry_attributeRequestToTtlvPayload"
            ),
            "attribute_request_get_attribute_list_to_batch_item": (
                "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemFromAttributeRequest"
            ),
        }
        for function_id, jni_symbol in expected.items():
            with self.subTest(function=function_id):
                self.assertEqual(
                    jni_symbol,
                    functions[function_id]["java"].get("jniSymbol"),
                )

    def test_java_attribute_request_handle_kind_constants_are_exactly_excluded(self) -> None:
        expected = {
            f"org.kmipkit.NativeExtensionRegistry.{name}"
            for name in (
                "ADD_ATTRIBUTE_REQUEST",
                "ADJUST_ATTRIBUTE_REQUEST",
                "DELETE_ATTRIBUTE_REQUEST",
                "MODIFY_ATTRIBUTE_REQUEST",
                "SET_ATTRIBUTE_REQUEST",
                "GET_ATTRIBUTES_REQUEST",
                "GET_ATTRIBUTE_LIST_REQUEST",
            )
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "java-kmipkit"
        }
        self.assertTrue(expected <= exclusions.keys())
        for export_name in expected:
            with self.subTest(export=export_name):
                row = exclusions.get(export_name)
                self.assertIsNotNone(row)
                if row is not None:
                    self.assertEqual(
                        "specs/043-public-api-bindings/spec.md",
                        row["approvalSource"]["path"],
                    )
                    self.assertEqual("Approved", row["approvalSource"]["status"])
                    self.assertTrue(row["reason"].strip())
                    self.assertIn("T097", row["evidence"])
                    self.assertRegex(row["evidence"], r"KMIPKIT-0043-FR-[^ ]*021")

    def test_typed_request_lifecycle_batch_conversion_and_abi_are_mapped(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        types = {row["id"]: row for row in self.manifest["types"]}
        function_ids = {row["id"] for row in self.manifest["functions"]}
        for operation, methods in REQUEST_MODEL_METHODS.items():
            class_name = "".join(part.title() for part in operation.split("_")) + "Request"
            java_model = f"org.kmipkit.extensions.{class_name}"
            python_model = f"kmipkit.attribute_operations.{class_name}"
            release_symbol = f"kmipkit_{operation}_request_release"
            batch_symbol = f"kmipkit_client_batch_item_{operation}_from_request"
            with self.subTest(model=class_name):
                self.assertIn(class_name, types)
                self.assertEqual(release_symbol, types[class_name]["c"]["releaseFunction"])
                release = declarations[("c-kmipkit", release_symbol)]
                self.assertEqual("native-handle-release", release["mappingSemantics"])
                self.assertEqual(java_model + ".close", release["languageMappings"]["java"]["name"])
                self.assertEqual(python_model + ".close", release["languageMappings"]["python"]["name"])
                self.assertIn(("c-kmipkit", batch_symbol), declarations)
                self.assertEqual(java_model + ".toBatchItem", declarations[("c-kmipkit", batch_symbol)]["languageMappings"]["java"]["name"])
                self.assertEqual(python_model + ".to_batch_item", declarations[("c-kmipkit", batch_symbol)]["languageMappings"]["python"]["name"])
                for method in (*methods, "create", "to_batch_item"):
                    self.assertIn(f"attribute_request_{operation}_{method}", function_ids)

    def test_python_model_plumbing_exports_are_exactly_excluded(self) -> None:
        expected = {"kmipkit.attribute_operations"}
        for operation in (*REQUEST_MODEL_METHODS, *READ_REQUEST_MODEL_METHODS):
            class_name = "".join(part.title() for part in operation.split("_")) + "Request"
            prefix = f"kmipkit.attribute_operations.{class_name}"
            expected.update(
                {
                    f"{prefix}.__init__",
                    f"{prefix}.closed",
                    f"{prefix}.__enter__",
                    f"{prefix}.__exit__",
                }
            )
        expected.add("kmipkit.attribute_operations.GetAttributesRequest.__slots__")
        excluded = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC_PATH
            and row["packageId"] == "python-kmipkit"
            and row["exportName"].startswith("kmipkit.attribute_operations")
        }
        self.assertEqual(expected, set(excluded))
        for export_name, row in excluded.items():
            with self.subTest(export=export_name):
                self.assertTrue(row["reason"].strip())
                self.assertIn("KMIPKIT-0043-FR-021", row["evidence"])

    def test_get_attributes_facade_mappings_use_the_get_attributes_requirement(self) -> None:
        expected_ids = {
            "client_batch_item_get_attributes",
            "client_batch_item_response_get_attributes",
        }
        functions = {
            row["id"]: set(row["requirementIds"])
            for row in self.manifest["functions"]
            if row["id"] in expected_ids
        }
        self.assertEqual(expected_ids, set(functions))
        for function_id, requirement_ids in functions.items():
            with self.subTest(function=function_id):
                self.assertIn("KMIPKIT-0016-FR-006", requirement_ids)
                self.assertNotIn("KMIPKIT-0016-FR-007", requirement_ids)

        operation_rows = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("capabilityId") == "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES"
        ]
        self.assertTrue(operation_rows)
        for row in operation_rows:
            with self.subTest(declaration=row["exportName"]):
                self.assertIn("KMIPKIT-0016-FR-006", row["requirementIds"])
                self.assertNotIn("KMIPKIT-0016-FR-007", row["requirementIds"])


if __name__ == "__main__":
    unittest.main()
