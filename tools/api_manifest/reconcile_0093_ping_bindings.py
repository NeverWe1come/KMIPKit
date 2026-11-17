"""Reconcile the approved four-language fluent Ping surface."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path

REQUIREMENTS = [
    "KMIPKIT-0093-FR-004",
    "KMIPKIT-0093-FR-005",
    "KMIPKIT-0093-FR-011",
]
APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/093-fluent-complete-api/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved in scope by the maintainer's direct 2026-11-11 instruction; implementation and source mapping remain subject to PR review.",
}
EVIDENCE = [
    "crates/kmipkit-ffi/tests/client_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java",
    "bindings/python/tests/test_fluent.py",
    "tools/api_manifest/tests/test_0093_ping_builder_manifest.py",
]
LANGUAGES = ("rust", "c", "java", "python")


def _declaration(
    *, package: str, kind: str, name: str, mappings: dict[str, str]
) -> dict:
    return {
        "declarationId": f"KMIPKIT-0093-ping-{package}-{name}",
        "packageId": package,
        "kind": kind,
        "exportName": name,
        "capabilityId": "fluent_ping_builder",
        "requirementIds": REQUIREMENTS,
        "declarationState": "declared",
        "parityScope": "four-language",
        "approvalSource": APPROVAL,
        "candidateSpecId": "KMIPKIT-0093",
        "languageMappings": {
            language: {
                "packageId": {
                    "rust": "rust-kmipkit-client",
                    "c": "c-kmipkit",
                    "java": "java-kmipkit",
                    "python": "python-kmipkit",
                }[language],
                "name": mappings[language],
            }
            for language in LANGUAGES
        },
        "parityEvidenceIds": EVIDENCE,
    }


def reconcile(manifest: dict) -> dict:
    result = deepcopy(manifest)
    result["capabilities"] = [
        row for row in result["capabilities"]
        if row["capabilityId"] not in {"fluent_ping_builder", "fluent_ping_builder_c_cleanup"}
    ]
    result["capabilities"].append({
        "capabilityId": "fluent_ping_builder",
        "requirementIds": REQUIREMENTS,
        "capabilityState": "declared",
        "parityScope": "four-language",
        "ownerSpecId": "KMIPKIT-0093",
        "approvalEvidence": APPROVAL,
        "mappings": {
            "rust": [
                "kmipkit_client::fluent::PingBuilder",
                "kmipkit_client::fluent::RequestBuilder::ping",
            ],
            "c": [
                "kmipkit_ping_builder_create",
                "kmipkit_ping_builder_set_asynchronous_indicator",
                "kmipkit_ping_builder_validate",
                "kmipkit_ping_builder_build_batch_item",
                "kmipkit_ping_builder_execute_v1",
                "kmipkit_ping_builder_execute_with_options_v1",
                "kmipkit_ping_builder_release",
                "kmipkit_client_ping_v1",
            ],
            "java": [
                "org.kmipkit.fluent.FluentRequest.ping",
                "org.kmipkit.fluent.PingBuilder",
            ],
            "python": [
                "kmipkit.fluent.Request.ping",
                "kmipkit.fluent.PingBuilder",
            ],
        },
    })

    type_template = next(row for row in result["types"] if row["id"] == "RevokeBuilder")
    ping_type = deepcopy(type_template)
    ping_type.update(id="PingBuilder", requirementIds=REQUIREMENTS)
    ping_type["rust"].update(name="PingBuilder")
    ping_type["c"].update(
        name="kmipkit_ping_builder_t",
        handleKind="ping_builder",
        releaseFunction="kmipkit_ping_builder_release",
    )
    ping_type["java"].update(name="PingBuilder")
    ping_type["python"].update(name="PingBuilder")
    result["types"] = [row for row in result["types"] if row["id"] != "PingBuilder"]
    result["types"].append(ping_type)

    function_templates = {
        "create": "revoke_builder_create",
        "set_async": "revoke_builder_set_asynchronous_indicator",
        "validate": "locate_builder_validate",
        "build": "locate_builder_build_batch_item",
        "execute": "locate_builder_execute_v1",
        "execute_options": "locate_builder_execute_with_options_v1",
    }
    template_rows = {
        key: next(row for row in result["functions"] if row["id"] == value)
        for key, value in function_templates.items()
    }
    function_specs = [
        ("ping_builder_create", "create", "kmipkit_ping_builder_create", "ping", [], "PingBuilder"),
        ("ping_builder_set_asynchronous_indicator", "set_async", "kmipkit_ping_builder_set_asynchronous_indicator", "asynchronousIndicator", [
            {"name": "value", "type": "uint32_t", "kind": "scalar"}
        ], "void"),
        ("ping_builder_validate", "validate", "kmipkit_ping_builder_validate", "validate", [], "void"),
        ("ping_builder_build_batch_item", "build", "kmipkit_ping_builder_build_batch_item", "toBatchItem", [], "ClientBatchItem"),
        ("ping_builder_execute_v1", "execute", "kmipkit_ping_builder_execute_v1", "execute", [], "ClientBatchResponse"),
        ("ping_builder_execute_with_options_v1", "execute_options", "kmipkit_ping_builder_execute_with_options_v1", "executeWithOptions", [], "ClientBatchResponse"),
    ]
    for function_id, template_key, symbol, method, parameters, return_type in function_specs:
        row = deepcopy(template_rows[template_key])
        row.update(id=function_id, requirementIds=REQUIREMENTS)
        row["rust"].update(
            module="kmipkit_client::fluent::RequestBuilder" if template_key == "create" else "kmipkit_client::fluent::PingBuilder",
            name={
                "create": "ping",
                "set_async": "with_asynchronous_indicator",
                "validate": "validate",
                "build": "to_batch_item",
                "execute": "execute",
                "execute_options": "with_options",
            }[template_key],
            parameters=[{"name": "value", "type": "u32"}] if template_key == "set_async" else
                [{"name": "limits", "type": "&CodecLimits"}] if template_key == "execute" else
                [{"name": "options", "type": "RequestOptions"}] if template_key == "execute_options" else [],
            returnType="PingBuilder" if template_key in ("create", "set_async", "execute_options") else "ClientBatchItem" if template_key == "build" else "ClientBatchItemResponse" if template_key == "execute" else return_type,
        )
        row["c"].update(symbol=symbol, parameters=parameters)
        row["c"]["documentation"] = [{
            "create": "Creates an opaque builder for the empty KMIP 2.1 Ping request.",
            "set_async": "Sets the raw-preserving Request Header Asynchronous Indicator.",
            "validate": "Validates that the opaque empty Ping builder is live.",
            "build": "Builds an independently owned typed Ping batch item.",
            "execute": "Executes one typed Ping exchange using default exchange options.",
            "execute_options": "Executes one typed Ping exchange using explicit exchange-local options.",
        }[template_key]]
        builder_param = {
            "name": "builder", "type": "kmipkit_ping_builder_t *",
            "kind": "handle", "ownership": "borrowed",
        }
        if template_key != "create":
            row["c"]["parameters"].insert(0, builder_param)
        else:
            row["c"]["parameters"].append({
                "name": "out_builder", "type": "kmipkit_ping_builder_t **",
                "kind": "output", "ownership": "library-owned", "nullable": False,
            })
        if template_key in ("build",):
            row["c"]["parameters"].append({
                "name": "out_item", "type": "kmipkit_client_batch_item_t **",
                "kind": "output", "ownership": "library-owned", "nullable": False,
            })
        if template_key in ("execute", "execute_options"):
            row["c"]["parameters"].extend([
                {"name": "client", "type": "kmipkit_client_t *", "kind": "handle", "ownership": "borrowed"},
                {"name": "limits", "type": "kmipkit_codec_limits_t *", "kind": "handle", "ownership": "borrowed"},
            ])
            if template_key == "execute_options":
                row["c"]["parameters"].append({
                    "name": "options", "type": "const kmipkit_request_options_v1_t *",
                    "kind": "input-value-pointer", "valueType": "kmipkit_request_options_v1_t", "ownership": "borrowed",
                })
            row["c"]["parameters"].append({
                "name": "out_response", "type": "kmipkit_client_batch_response_t **",
                "kind": "output", "ownership": "library-owned", "nullable": False,
            })
        row["java"].update(
            package="org.kmipkit.fluent",
            **{"class": "FluentRequest" if template_key == "create" else "PingBuilder"},
            method=method,
            parameters=[{"name": "value", "type": "long"}] if template_key == "set_async" else
                [{"name": "limits", "type": "CodecLimits"}, {"name": "options", "type": "RequestOptions"}] if template_key == "execute_options" else
                [{"name": "limits", "type": "CodecLimits"}] if template_key == "execute" else [],
            returnType="PingBuilder" if template_key in ("create", "set_async") else "ClientBatchItem" if template_key == "build" else return_type,
        )
        python_method = {"set_async": "asynchronous_indicator", "build": "to_batch_item", "execute_options": "execute"}.get(
            template_key, method[0].lower() + method[1:]
        )
        row["python"].update(
            module="kmipkit.fluent",
            function="Request.ping" if template_key == "create" else f"PingBuilder.{python_method}",
            parameters=[{"name": "value", "type": "int"}] if template_key == "set_async" else
                [{"name": "limits", "type": "ttlv.CodecLimits"}, {"name": "options", "type": "RequestOptions"}] if template_key == "execute_options" else
                [{"name": "limits", "type": "ttlv.CodecLimits"}] if template_key == "execute" else [],
            cffiSymbol=symbol,
            returnType="PingBuilder" if template_key in ("create", "set_async") else "ClientBatchItem" if template_key == "build" else "BatchResult" if template_key in ("execute", "execute_options") else return_type,
        )
        result["functions"] = [r for r in result["functions"] if r["id"] != function_id] + [row]

    direct = deepcopy(next(row for row in result["functions"] if row["id"] == "client_create_symmetric_key_v1"))
    direct.update(id="client_ping_v1", requirementIds=REQUIREMENTS)
    direct["rust"].update(module="kmipkit_client::fluent::PingBuilder", name="execute", parameters=[], returnType="ClientBatchItemResponse")
    direct["c"].update(symbol="kmipkit_client_ping_v1", parameters=[
        {"name": "client", "type": "kmipkit_client_t *", "kind": "handle", "ownership": "borrowed"},
        {"name": "limits", "type": "kmipkit_codec_limits_t *", "kind": "handle", "ownership": "borrowed"},
        {"name": "out_response", "type": "kmipkit_client_batch_response_t **", "kind": "output", "ownership": "library-owned", "nullable": False},
    ], documentation=["Executes one empty typed KMIP 2.1 Ping request using the existing client and default exchange options."])
    direct["java"].update(package="org.kmipkit.fluent", **{"class": "PingBuilder"}, method="execute", parameters=[{"name": "limits", "type": "CodecLimits"}], returnType="ClientBatchResponse")
    direct["python"].update(module="kmipkit.fluent", function="PingBuilder.execute", parameters=[{"name": "limits", "type": "ttlv.CodecLimits"}], cffiSymbol="kmipkit_client_ping_v1", returnType="BatchResult")
    result["functions"] = [row for row in result["functions"] if row["id"] != "client_ping_v1"] + [direct]

    mappings = {
        "kmipkit_client::fluent::PingBuilder": ("type", "kmipkit_client::fluent::PingBuilder", "kmipkit_ping_builder_t", "org.kmipkit.fluent.PingBuilder", "kmipkit.fluent.PingBuilder"),
        "kmipkit_client::fluent::RequestBuilder<'a>::ping": ("associated-item", "kmipkit_client::fluent::RequestBuilder<'a>::ping", "kmipkit_ping_builder_create", "org.kmipkit.fluent.FluentRequest.ping", "kmipkit.fluent.Request.ping"),
        "kmipkit_client::fluent::PingBuilder<'a>::build": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::build", "kmipkit_ping_builder_build_batch_item", "org.kmipkit.fluent.PingBuilder.build", "kmipkit.fluent.PingBuilder.build"),
        "kmipkit_client::fluent::PingBuilder<'a>::to_batch_item": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::to_batch_item", "kmipkit_ping_builder_build_batch_item", "org.kmipkit.fluent.PingBuilder.toBatchItem", "kmipkit.fluent.PingBuilder.to_batch_item"),
        "kmipkit_client::fluent::PingBuilder<'a>::execute": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::execute", "kmipkit_ping_builder_execute_v1", "org.kmipkit.fluent.PingBuilder.execute", "kmipkit.fluent.PingBuilder.execute"),
        "kmipkit_client::fluent::PingBuilder<'a>::validate": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::validate", "kmipkit_ping_builder_validate", "org.kmipkit.fluent.PingBuilder.validate", "kmipkit.fluent.PingBuilder.validate"),
        "kmipkit_client::fluent::PingBuilder<'a>::with_asynchronous_indicator": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::with_asynchronous_indicator", "kmipkit_ping_builder_set_asynchronous_indicator", "org.kmipkit.fluent.PingBuilder.asynchronousIndicator", "kmipkit.fluent.PingBuilder.asynchronous_indicator"),
        "kmipkit_client::fluent::PingBuilder<'a>::with_options": ("associated-item", "kmipkit_client::fluent::PingBuilder<'a>::with_options", "kmipkit_ping_builder_execute_with_options_v1", "org.kmipkit.fluent.PingBuilder.withOptions", "kmipkit.fluent.PingBuilder.with_options"),
    }
    packages = {"rust": "rust-kmipkit-client", "c": "c-kmipkit", "java": "java-kmipkit", "python": "python-kmipkit"}
    declarations = result["publicDeclarations"]
    for rust_name, (kind, rust, c, java, python) in mappings.items():
        names = {"rust": rust_name, "c": c, "java": java, "python": python}
        row = _declaration(
            package=packages["rust"], kind=kind, name=rust_name,
            mappings={"rust": rust, "c": c, "java": java, "python": python},
        )
        for language, name in names.items():
            package = packages[language]
            current = next((item for item in declarations if item["packageId"] == package and item["exportName"] == name), None)
            if language != "rust":
                row = _declaration(package=package, kind={"c": "c-symbol", "java": "java-member", "python": "python-export"}.get(language, kind), name=name, mappings={"rust": rust, "c": c, "java": java, "python": python})
            else:
                row["packageId"] = package
            if current:
                current.clear()
                current.update(row)
            else:
                declarations.append(row)

    direct_names = {
        "rust": "kmipkit_client::fluent::PingBuilder<'a>::execute",
        "c": "kmipkit_client_ping_v1",
        "java": "org.kmipkit.fluent.PingBuilder.execute",
        "python": "kmipkit.fluent.PingBuilder.execute",
    }
    direct_mappings = dict(direct_names)
    for language, name in direct_names.items():
        package = packages[language]
        row = _declaration(
            package=package,
            kind={"rust": "associated-item", "c": "c-symbol", "java": "java-member", "python": "python-export"}[language],
            name=name, mappings=direct_mappings,
        )
        current = next((item for item in declarations if item["packageId"] == package and item["exportName"] == name), None)
        if current:
            current.clear(); current.update(row)
        else:
            declarations.append(row)

    release = _declaration(
        package="c-kmipkit", kind="c-symbol", name="kmipkit_ping_builder_release",
        mappings={
            "rust": "kmipkit_client::fluent::PingBuilder",
            "c": "kmipkit_ping_builder_release",
            "java": "org.kmipkit.fluent.PingBuilder",
            "python": "kmipkit.fluent.PingBuilder",
        },
    )
    release["parityScope"] = "c-only"
    release["languageMappings"] = {"c": {"packageId": "c-kmipkit", "name": "kmipkit_ping_builder_release"}}
    release["capabilityId"] = "fluent_ping_builder_c_cleanup"
    result["capabilities"].append({
        "capabilityId": "fluent_ping_builder_c_cleanup",
        "requirementIds": REQUIREMENTS,
        "capabilityState": "declared",
        "parityScope": "c-only",
        "ownerSpecId": "KMIPKIT-0093",
        "approvalEvidence": APPROVAL,
        "mappings": {"c": ["kmipkit_ping_builder_release"]},
    })
    existing = next((item for item in declarations if item["packageId"] == "c-kmipkit" and item["exportName"] == "kmipkit_ping_builder_release"), None)
    if existing:
        existing.clear(); existing.update(release)
    else:
        declarations.append(release)
    ping_capability = next(
        row for row in result["capabilities"]
        if row["capabilityId"] == "fluent_ping_builder"
    )
    for language, package in packages.items():
        ping_capability["mappings"][language] = sorted({
            row["exportName"] for row in declarations
            if row["packageId"] == package
            and row.get("capabilityId") == "fluent_ping_builder"
            and row.get("declarationState") in {"declared", "verified"}
        })
    return result


if __name__ == "__main__":
    path = Path("specification/api/public-api.json")
    path.write_text(json.dumps(reconcile(json.loads(path.read_text())), indent=2, ensure_ascii=False) + "\n")
