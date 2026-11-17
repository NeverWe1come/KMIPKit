"""Reconcile the approved four-language generic Create builder slice."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REQUIREMENTS = ["KMIPKIT-0093-FR-004", "KMIPKIT-0093-FR-005", "KMIPKIT-0093-FR-011"]
APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/093-fluent-complete-api/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved in scope by the maintainer's direct 2026-11-11 instruction; implementation and source mapping remain subject to PR review.",
}
EVIDENCE = [
    "crates/kmipkit-ffi/src/extension_registry/client_abi/fluent.rs",
    "crates/kmipkit-ffi/tests/support/get_attributes_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java",
    "bindings/python/tests/test_fluent.py",
]

METHODS = {
    "create": {
        "rust_module": "kmipkit_client::fluent::RequestBuilder<'a>",
        "rust_method": "create",
        "c_method": "create",
        "java_class": "FluentRequest",
        "java_method": "create",
        "python_function": "Request.create",
        "python_method": "create",
        "c_parameters": [
            {"name": "out_builder", "kind": "output", "type": "kmipkit_create_builder_t **", "ownership": "library-owned", "nullable": False}
        ],
        "java_parameters": [],
        "python_parameters": [],
        "return_type": "CreateBuilder",
    },
    "set_object_type": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "object_type",
        "c_method": "setObjectType",
        "java_class": "CreateBuilder",
        "java_method": "objectType",
        "python_function": "CreateBuilder.object_type",
        "python_method": "object_type",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "object_type", "kind": "scalar", "type": "uint32_t"},
        ],
        "java_parameters": [{"name": "objectType", "type": "long"}],
        "python_parameters": [{"name": "object_type", "type": "int"}],
        "return_type": "void",
    },
    "set_attributes": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "attributes",
        "c_method": "setAttributes",
        "java_class": "CreateBuilder",
        "java_method": "attributes",
        "python_function": "CreateBuilder.attributes",
        "python_method": "attributes",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "attributes", "kind": "handle", "type": "kmipkit_ttlv_structure_t *", "ownership": "borrowed"},
        ],
        "java_parameters": [{"name": "attributes", "type": "AttributeSet"}],
        "python_parameters": [{"name": "attributes", "type": "AttributeSet"}],
        "return_type": "void",
    },
    "set_protection_storage_masks": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "protection_storage_masks",
        "c_method": "setProtectionStorageMasks",
        "java_class": "CreateBuilder",
        "java_method": "protectionStorageMasks",
        "python_function": "CreateBuilder.protection_storage_masks",
        "python_method": "protection_storage_masks",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "protection_storage_masks", "kind": "handle", "type": "kmipkit_ttlv_structure_t *", "ownership": "borrowed"},
        ],
        "java_parameters": [{"name": "masks", "type": "TtlvStructure"}],
        "python_parameters": [{"name": "masks", "type": "TtlvStructure | None"}],
        "return_type": "void",
    },
    "set_asynchronous_indicator": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "with_asynchronous_indicator",
        "c_method": "setAsynchronousIndicator",
        "java_class": "CreateBuilder",
        "java_method": "withAsynchronousIndicator",
        "python_function": "CreateBuilder.with_asynchronous_indicator",
        "python_method": "with_asynchronous_indicator",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "value", "kind": "scalar", "type": "uint32_t"},
        ],
        "java_parameters": [{"name": "value", "type": "long"}],
        "python_parameters": [{"name": "value", "type": "int"}],
        "return_type": "void",
    },
    "validate": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "validate",
        "c_method": "validate",
        "java_class": "CreateBuilder",
        "java_method": "validate",
        "python_function": "CreateBuilder.validate",
        "python_method": "validate",
        "c_parameters": [{"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"}],
        "java_parameters": [],
        "python_parameters": [],
        "return_type": "void",
    },
    "build_batch_item": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "to_batch_item",
        "c_method": "buildBatchItem",
        "java_class": "CreateBuilder",
        "java_method": "build",
        "python_function": "CreateBuilder.build",
        "python_method": "build",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "out_item", "kind": "output", "type": "kmipkit_client_batch_item_t **", "ownership": "library-owned", "nullable": False},
        ],
        "java_parameters": [],
        "python_parameters": [],
        "return_type": "ClientBatchItem",
    },
    "execute_v1": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "execute",
        "c_method": "executeV1",
        "java_class": "CreateBuilder",
        "java_method": "execute",
        "python_function": "CreateBuilder.execute",
        "python_method": "execute",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "client", "kind": "handle", "type": "kmipkit_client_t *", "ownership": "borrowed"},
            {"name": "limits", "kind": "handle", "type": "kmipkit_codec_limits_t *", "ownership": "borrowed"},
            {"name": "out_response", "kind": "output", "type": "kmipkit_client_batch_response_t **", "ownership": "library-owned", "nullable": False},
        ],
        "java_parameters": [{"name": "client", "type": "Client"}, {"name": "limits", "type": "CodecLimits"}],
        "python_parameters": [{"name": "client", "type": "Client"}, {"name": "limits", "type": "CodecLimits"}],
        "return_type": "ClientBatchResponse",
    },
    "execute_with_options_v1": {
        "rust_module": "kmipkit_client::fluent::CreateBuilder<'a>",
        "rust_method": "with_options",
        "c_method": "executeWithOptionsV1",
        "java_class": "CreateBuilder",
        "java_method": "withOptions",
        "python_function": "CreateBuilder.with_options",
        "python_method": "with_options",
        "c_parameters": [
            {"name": "builder", "kind": "handle", "type": "kmipkit_create_builder_t *", "ownership": "borrowed"},
            {"name": "client", "kind": "handle", "type": "kmipkit_client_t *", "ownership": "borrowed"},
            {"name": "limits", "kind": "handle", "type": "kmipkit_codec_limits_t *", "ownership": "borrowed"},
            {"name": "options", "kind": "input-value-pointer", "type": "const kmipkit_request_options_v1_t *", "valueType": "kmipkit_request_options_v1_t", "ownership": "borrowed"},
            {"name": "out_response", "kind": "output", "type": "kmipkit_client_batch_response_t **", "ownership": "library-owned", "nullable": False},
        ],
        "java_parameters": [{"name": "client", "type": "Client"}, {"name": "limits", "type": "CodecLimits"}, {"name": "options", "type": "RequestOptions"}],
        "python_parameters": [{"name": "client", "type": "Client"}, {"name": "limits", "type": "CodecLimits"}, {"name": "options", "type": "RequestOptions"}],
        "return_type": "ClientBatchResponse",
    },
}


def reconcile(manifest: dict) -> dict:
    result = deepcopy(manifest)
    functions = {row["id"]: row for row in result["functions"]}
    public = result["publicDeclarations"]
    template = functions["create_key_pair_builder_validate"]

    type_row = next(row for row in result["types"] if row["id"] == "CreateKeyPairBuilder")
    create_type = deepcopy(type_row)
    create_type.update(id="CreateBuilder", requirementIds=REQUIREMENTS)
    create_type["rust"].update(module="kmipkit_client::fluent", name="CreateBuilder")
    create_type["c"].update(
        name="kmipkit_create_builder_t",
        handleKind="create_builder",
        releaseFunction="kmipkit_create_builder_release",
    )
    create_type["java"].update(name="CreateBuilder")
    create_type["python"].update(name="CreateBuilder")
    result["types"] = [row for row in result["types"] if row["id"] != "CreateBuilder"] + [create_type]

    for suffix, spec in METHODS.items():
        function_id = "create_builder_" + suffix
        symbol = "kmipkit_create_builder_" + suffix
        record = deepcopy(template)
        record.update(id=function_id, requirementIds=REQUIREMENTS, legacyRegistry=False)
        record["rust"].update(
            module=spec["rust_module"].replace("<'a>", ""),
            name=spec["rust_method"],
            returnType=spec["return_type"],
            parameters=[],
        )
        record["c"].update(
            symbol=symbol,
            parameters=deepcopy(spec["c_parameters"]),
            returnType="int32_t",
            errorCategories=["invalid_input", "resource_limit", "internal"],
        )
        record["java"].update({
            "package": "org.kmipkit.fluent",
            "class": spec["java_class"],
            "method": spec["java_method"],
            "parameters": deepcopy(spec["java_parameters"]),
            "returnType": spec["return_type"],
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
        })
        record["python"].update(
            module="kmipkit.fluent",
            function=spec["python_function"],
            parameters=deepcopy(spec["python_parameters"]),
            cffiSymbol=symbol,
            returnType=spec["return_type"],
            errorCategories=["invalid_input", "resource_limit", "internal"],
        )
        result["functions"] = [row for row in result["functions"] if row["id"] != function_id] + [record]

        mappings = {
            "rust": {"packageId": "rust-kmipkit-client", "name": spec["rust_module"] + "::" + spec["rust_method"]},
            "c": {"packageId": "c-kmipkit", "name": symbol},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.fluent." + spec["java_class"] + "." + spec["java_method"]},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.fluent." + spec["python_function"]},
        }
        declaration_names = [
            ("c-kmipkit", "c-symbol", symbol),
            ("java-kmipkit", "java-member", mappings["java"]["name"]),
            ("python-kmipkit", "python-export", mappings["python"]["name"]),
        ]
        if suffix == "create":
            mappings["rust"] = {"packageId": "rust-kmipkit-client", "name": spec["rust_module"] + "::create"}
            declaration_names.append(("rust-kmipkit-client", "associated-item", mappings["rust"]["name"]))
        for package, kind, name in declaration_names:
            row = next((row for row in public if row["packageId"] == package and row["exportName"] == name), None)
            declaration = {
                "declarationId": row["declarationId"] if row else "kmipkit-0093-create-builder-" + suffix + "-" + package.split("-")[0],
                "packageId": package,
                "kind": kind,
                "exportName": name,
                "capabilityId": "fluent_create_builder_parity",
                "requirementIds": REQUIREMENTS,
                "declarationState": "declared",
                "parityScope": "four-language",
                "approvalSource": APPROVAL,
                "candidateSpecId": "KMIPKIT-0093",
                "languageMappings": deepcopy(mappings),
                "parityEvidenceIds": EVIDENCE,
            }
            if row:
                row.clear()
                row.update(declaration)
            else:
                public.append(declaration)

    method_by_rust_name = {spec["rust_method"]: spec for spec in METHODS.values()}
    method_by_rust_name["build"] = METHODS["build_batch_item"]
    for row in public:
        if row.get("packageId") != "rust-kmipkit-client":
            continue
        name = row.get("exportName", "")
        if name.endswith("::CreateBuilder"):
            mappings = {
                "rust": {"packageId": "rust-kmipkit-client", "name": name},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_create_builder_t"},
                "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.fluent.CreateBuilder"},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.fluent.CreateBuilder"},
            }
        else:
            method = name.rsplit("::", 1)[-1]
            spec = method_by_rust_name.get(method)
            is_create = name.endswith("RequestBuilder<'a>::create")
            if spec is None or ("CreateBuilder" not in name and not is_create):
                continue
            if is_create:
                spec = METHODS["create"]
            suffix = next(suffix for suffix, candidate in METHODS.items() if candidate is spec)
            mappings = {
                "rust": {"packageId": "rust-kmipkit-client", "name": name},
                "c": {"packageId": "c-kmipkit", "name": "kmipkit_create_builder_" + suffix},
                "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.fluent." + spec["java_class"] + "." + spec["java_method"]},
                "python": {"packageId": "python-kmipkit", "name": "kmipkit.fluent." + spec["python_function"]},
            }
        row.update(
            capabilityId="fluent_create_builder_parity",
            requirementIds=REQUIREMENTS,
            declarationState="declared",
            parityScope="four-language",
            approvalSource=APPROVAL,
            candidateSpecId="KMIPKIT-0093",
            languageMappings=mappings,
            parityEvidenceIds=EVIDENCE,
        )
        row.pop("blockedReason", None)
        row.pop("featureSpecId", None)

    # The language surfaces expose one type each; the opaque C release symbol is
    # emitted from the type record by the canonical header generator.
    type_mappings = {
        "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::fluent::CreateBuilder"},
        "c": {"packageId": "c-kmipkit", "name": "kmipkit_create_builder_t"},
        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.fluent.CreateBuilder"},
        "python": {"packageId": "python-kmipkit", "name": "kmipkit.fluent.CreateBuilder"},
    }
    for package, kind, name in [
        ("c-kmipkit", "type", "kmipkit_create_builder_t"),
        ("java-kmipkit", "type", "org.kmipkit.fluent.CreateBuilder"),
        ("python-kmipkit", "type", "kmipkit.fluent.CreateBuilder"),
    ]:
        row = next((row for row in public if row["packageId"] == package and row["exportName"] == name), None)
        declaration = {
            "declarationId": row["declarationId"] if row else "kmipkit-0093-create-builder-type-" + package.split("-")[0],
            "packageId": package,
            "kind": kind,
            "exportName": name,
            "capabilityId": "fluent_create_builder_parity",
            "requirementIds": REQUIREMENTS,
            "declarationState": "declared",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "candidateSpecId": "KMIPKIT-0093",
            "languageMappings": deepcopy(type_mappings),
            "parityEvidenceIds": EVIDENCE,
        }
        if row:
            row.clear()
            row.update(declaration)
        else:
            public.append(declaration)

    release_name = "kmipkit_create_builder_release"
    release_mappings = {
        "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::fluent::CreateBuilder"},
        "c": {"packageId": "c-kmipkit", "name": release_name},
        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.fluent.CreateBuilder"},
        "python": {"packageId": "python-kmipkit", "name": "kmipkit.fluent.CreateBuilder"},
    }
    row = next((row for row in public if row["packageId"] == "c-kmipkit" and row["exportName"] == release_name), None)
    release_declaration = {
        "declarationId": row["declarationId"] if row else "kmipkit-0093-create-builder-release",
        "packageId": "c-kmipkit",
        "kind": "c-symbol",
        "exportName": release_name,
        "capabilityId": "fluent_create_builder_parity",
        "requirementIds": REQUIREMENTS,
        "declarationState": "declared",
        "parityScope": "four-language",
        "approvalSource": APPROVAL,
        "candidateSpecId": "KMIPKIT-0093",
        "languageMappings": release_mappings,
        "parityEvidenceIds": EVIDENCE,
    }
    if row:
        row.clear()
        row.update(release_declaration)
    else:
        public.append(release_declaration)

    lifecycle = {
        "kmipkit.fluent.CreateBuilder.__init__": {
            "rust": "kmipkit_client::fluent::CreateBuilder",
            "c": "kmipkit_create_builder_create",
            "java": "org.kmipkit.fluent.CreateBuilder",
        },
        "kmipkit.fluent.CreateBuilder.__enter__": {
            "rust": "kmipkit_client::fluent::CreateBuilder",
            "c": "kmipkit_create_builder_t",
            "java": "org.kmipkit.fluent.CreateBuilder",
        },
        "kmipkit.fluent.CreateBuilder.__exit__": {
            "rust": "kmipkit_client::fluent::CreateBuilder",
            "c": "kmipkit_create_builder_release",
            "java": "org.kmipkit.fluent.CreateBuilder",
        },
    }
    for export_name, names in lifecycle.items():
        mappings = {
            language: {"packageId": package, "name": names[language]}
            for language, package in [
                ("rust", "rust-kmipkit-client"),
                ("c", "c-kmipkit"),
                ("java", "java-kmipkit"),
            ]
        }
        mappings["python"] = {"packageId": "python-kmipkit", "name": export_name}
        row = next((row for row in public if row["packageId"] == "python-kmipkit" and row["exportName"] == export_name), None)
        declaration = {
            "declarationId": row["declarationId"] if row else "kmipkit-0093-create-builder-python-" + export_name.rsplit(".", 1)[-1].strip("_"),
            "packageId": "python-kmipkit",
            "kind": "python-export",
            "exportName": export_name,
            "capabilityId": "fluent_create_builder_parity",
            "requirementIds": REQUIREMENTS,
            "declarationState": "declared",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "candidateSpecId": "KMIPKIT-0093",
            "languageMappings": mappings,
            "parityEvidenceIds": EVIDENCE,
        }
        if row:
            row.clear()
            row.update(declaration)
        else:
            public.append(declaration)

    aliases = {
        ("java-kmipkit", "org.kmipkit.fluent.CreateBuilder.close", "java-member"): {
            "rust": "kmipkit_client::fluent::CreateBuilder",
            "c": "kmipkit_create_builder_release",
            "java": "org.kmipkit.fluent.CreateBuilder.close",
            "python": "kmipkit.fluent.CreateBuilder.close",
        },
        ("java-kmipkit", "org.kmipkit.fluent.CreateBuilder.toBatchItem", "java-member"): {
            "rust": "kmipkit_client::fluent::CreateBuilder<'a>::to_batch_item",
            "c": "kmipkit_create_builder_build_batch_item",
            "java": "org.kmipkit.fluent.CreateBuilder.toBatchItem",
            "python": "kmipkit.fluent.CreateBuilder.build",
        },
        ("python-kmipkit", "kmipkit.fluent.CreateBuilder.close", "python-export"): {
            "rust": "kmipkit_client::fluent::CreateBuilder",
            "c": "kmipkit_create_builder_release",
            "java": "org.kmipkit.fluent.CreateBuilder.close",
            "python": "kmipkit.fluent.CreateBuilder.close",
        },
        ("python-kmipkit", "kmipkit.fluent.CreateBuilder.to_batch_item", "python-export"): {
            "rust": "kmipkit_client::fluent::CreateBuilder<'a>::to_batch_item",
            "c": "kmipkit_create_builder_build_batch_item",
            "java": "org.kmipkit.fluent.CreateBuilder.build",
            "python": "kmipkit.fluent.CreateBuilder.to_batch_item",
        },
    }
    package_ids = {
        "rust": "rust-kmipkit-client",
        "c": "c-kmipkit",
        "java": "java-kmipkit",
        "python": "python-kmipkit",
    }
    for (package, export_name, kind), names in aliases.items():
        row = next((row for row in public
                    if row["packageId"] == package and row["exportName"] == export_name), None)
        declaration = {
            "declarationId": f"KMIPKIT-0093:{package.split('-')[0]}:{export_name}",
            "packageId": package,
            "kind": kind,
            "exportName": export_name,
            "capabilityId": "fluent_create_builder_parity",
            "requirementIds": REQUIREMENTS,
            "declarationState": "declared",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "candidateSpecId": "KMIPKIT-0093",
            "languageMappings": {
                language: {"packageId": package_ids[language], "name": name}
                for language, name in names.items()
            },
            "parityEvidenceIds": EVIDENCE,
        }
        if export_name.endswith(".close"):
            declaration["mappingSemantics"] = "native-handle-release"
        if row:
            row.clear()
            row.update(declaration)
        else:
            public.append(declaration)

    capability = {
        "capabilityId": "fluent_create_builder_parity",
        "requirementIds": REQUIREMENTS,
        "capabilityState": "declared",
        "parityScope": "four-language",
        "ownerSpecId": "KMIPKIT-0093",
        "approvalEvidence": APPROVAL,
        "mappings": {
            language: sorted({row["languageMappings"][language]["name"] for row in public
                              if row.get("capabilityId") == "fluent_create_builder_parity"
                              and row.get("declarationState") in {"declared", "verified"}})
            for language in ("rust", "c", "java", "python")
        },
    }
    result["capabilities"] = [row for row in result["capabilities"] if row["capabilityId"] != capability["capabilityId"]] + [capability]
    return result


if __name__ == "__main__":
    path = ROOT / "specification/api/public-api.json"
    path.write_text(json.dumps(reconcile(json.loads(path.read_text(encoding="utf-8"))), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
