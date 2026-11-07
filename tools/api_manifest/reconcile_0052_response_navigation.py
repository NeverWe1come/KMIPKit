"""Add KMIPKIT-0052 response TTLV accessors to the reviewed API manifest."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

SPEC_ID = "KMIPKIT-0052"
PAYLOAD_REQUIREMENTS = [f"{SPEC_ID}-FR-002", f"{SPEC_ID}-FR-003", f"{SPEC_ID}-FR-007"]
HEADER_REQUIREMENTS = [f"{SPEC_ID}-FR-006", f"{SPEC_ID}-FR-007"]
TREE_REQUIREMENTS = [f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005", f"{SPEC_ID}-FR-007"]
ASYNC_TYPED_REQUIREMENTS = [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"]
SUPERSEDED_ASYNC_TYPED_ROUTES = {
    "c": {"kmipkit_client_async_operation_outcome_metadata_v1"},
    "java": {"org.kmipkit.extensions.AsyncOperationOutcome.metadataV1"},
    "python": {"kmipkit.client.AsyncOperationOutcome.metadata_v1"},
}
CAPABILITY_ID = "response_ttlv_navigation"
SPEC_PATH = "specs/052-response-navigation/spec.md"
APPROVAL = {
    "kind": "feature-specification",
    "path": SPEC_PATH,
    "status": "Approved",
    "evidence": "**Status**: Approved for implementation by the maintainer's KMIPKIT-0047–0053 plan.",
}
EVIDENCE = [
    "crates/kmipkit-client/tests/unit/hash_mac_signature_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/query_execution_tests.rs",
    "crates/kmipkit-protocol/tests/message_accessors.rs",
    "crates/kmipkit-ttlv/tests/public_api.rs",
    "crates/kmipkit-ffi/tests/support/query_ping_abi_tests.rs",
    "crates/kmipkit-ffi/tests/support/async_operation_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_response_raw_navigation.py",
    "bindings/python/tests/test_async_operation_mapping.py",
]
JAVA_BRIDGE_EXCLUSIONS = [
    {
        "packageId": "java-kmipkit",
        "exportName": "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseRawTtlv",
        "reason": "This public static native member is raw-handle JNI plumbing for the owned response-tree wrapper and has no independent Java product capability.",
        "approvalSource": APPROVAL,
        "evidence": "Reviewed against KMIPKIT-0052-FR-007. Applications use the typed ClientBatchItemResult.raw() facade; this exact bridge member transfers an owned native TTLV handle.",
    },
    {
        "packageId": "java-kmipkit",
        "exportName": "org.kmipkit.NativeExtensionRegistry.clientBatchItemResponseHeaderTtlv",
        "reason": "This public static native member is raw-handle JNI plumbing for the owned response-tree wrapper and has no independent Java product capability.",
        "approvalSource": APPROVAL,
        "evidence": "Reviewed against KMIPKIT-0052-FR-006 and KMIPKIT-0052-FR-007. Applications use the typed ClientBatchItemResult.responseHeaderTtlv() facade; this exact bridge member transfers an owned native TTLV handle.",
    },
]


def _accessor(suffix: str, java_name: str, python_name: str,
              c_documentation: str, java_return: str,
              python_return: str, requirements: list[str]) -> dict[str, Any]:
    c_symbol = f"kmipkit_client_batch_item_response_{suffix}_ttlv"
    java_native_suffix = "RawTtlv" if suffix == "raw" else "HeaderTtlv"
    java_native = "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponse" + java_native_suffix
    return {
        "id": f"client_batch_item_response_{suffix}_ttlv",
        "requirementIds": requirements,
        "rust": {
            "module": "kmipkit_ffi::extension_registry::client_abi",
            "name": c_symbol,
            "owner": "ffi",
            "requestBoundary": "not-applicable",
            "parameters": [
                {"name": "item", "type": "*mut kmipkit_client_batch_item_response_t"},
                {"name": "out_present", "type": "*mut u8"},
                {"name": "out_structure", "type": "*mut *mut kmipkit_ttlv_structure_t"},
            ],
            "returnType": "i32",
        },
        "c": {
            "symbol": c_symbol,
            "documentation": [c_documentation],
            "parameters": [
                {"name": "item", "kind": "handle", "type": "kmipkit_client_batch_item_response_t *", "ownership": "borrowed"},
                {"name": "out_present", "kind": "output-scalar-pointer", "type": "uint8_t *", "valueType": "uint8_t", "ownership": "none"},
                {"name": "out_structure", "kind": "output", "type": "kmipkit_ttlv_structure_t **", "ownership": "library-owned", "nullable": True},
            ],
            "returnType": "int32_t",
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
        },
        "java": {
            "package": "org.kmipkit.extensions",
            "class": "ClientBatchItemResult",
            "method": java_name,
            "parameters": [],
            "jniSymbol": java_native,
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
            "returnType": java_return,
        },
        "python": {
            "module": "kmipkit.client",
            "function": f"kmipkit.client.BatchItemResult.{python_name}",
            "parameters": [],
            "cffiSymbol": c_symbol,
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
            "returnType": python_return,
        },
        "ordering": "not-applicable",
    }


def _async_response_reader(
    operation: str,
    java_name: str,
    python_name: str,
    java_return: str,
    python_return: str,
    documentation: str,
) -> dict[str, Any]:
    c_symbol = f"kmipkit_client_async_operation_outcome_{operation}_response_v1"
    java_native = (
        "Java_org_kmipkit_extensions_AsyncOperationOutcome_read"
        + "".join(part.title() for part in operation.split("_"))
        + "ResponseV1"
    )
    return {
        "id": c_symbol,
        "requirementIds": ASYNC_TYPED_REQUIREMENTS,
        "rust": {
            "module": "kmipkit_ffi::extension_registry::client_abi",
            "name": c_symbol,
            "owner": "ffi",
            "requestBoundary": "not-applicable",
            "parameters": [
                {"name": "outcome", "type": "*mut kmipkit_client_async_operation_outcome_t"},
                {"name": "out_present", "type": "*mut u8"},
                {"name": "out_response", "type": "*mut kmipkit_client_async_operation_response_v1_t"},
            ],
            "returnType": "i32",
        },
        "c": {
            "symbol": c_symbol,
            "documentation": [documentation],
            "parameters": [
                {"name": "outcome", "kind": "handle", "type": "kmipkit_client_async_operation_outcome_t *", "ownership": "borrowed"},
                {"name": "out_present", "kind": "output-scalar-pointer", "type": "uint8_t *", "valueType": "uint8_t", "ownership": "none"},
                {"name": "out_response", "kind": "output-value-pointer", "type": "kmipkit_client_async_operation_response_v1_t *", "valueType": "kmipkit_client_async_operation_response_v1_t", "ownership": "none"},
            ],
            "returnType": "int32_t",
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
        },
        "java": {
            "package": "org.kmipkit.extensions",
            "class": "AsyncOperationOutcome",
            "method": java_name,
            "parameters": [],
            "jniSymbol": java_native,
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
            "returnType": java_return,
        },
        "python": {
            "module": "kmipkit.client",
            "function": f"AsyncOperationOutcome.{python_name}",
            "parameters": [],
            "cffiSymbol": c_symbol,
            "errorCategories": ["invalid_input", "resource_limit", "internal"],
            "returnType": python_return,
        },
        "ordering": "not-applicable",
    }


def _async_response_view_type() -> dict[str, Any]:
    fields = [
        ("struct_size", "uint32_t"),
        ("abi_version", "uint32_t"),
        ("operation", "uint32_t"),
        ("result_status", "uint32_t"),
        ("is_pending", "uint32_t"),
        ("has_cancellation_result", "uint32_t"),
        ("cancellation_result", "uint32_t"),
        ("has_asynchronous_correlation_value", "uint32_t"),
        ("asynchronous_correlation_value", "kmipkit_byte_span_v1_t"),
        ("has_response_payload", "uint32_t"),
        ("reserved", "uint32_t"),
        ("response_payload_member_count", "uint64_t"),
    ]
    return {
        "id": "AsyncOperationResponseV1",
        "kind": "struct",
        "requirementIds": ASYNC_TYPED_REQUIREMENTS,
        "rust": {
            "module": "kmipkit_client",
            "name": "ClientOperationOutcome",
            "owner": "client",
        },
        "c": {
            "name": "kmipkit_client_async_operation_response_v1_t",
            "kind": "fixed-width-value",
            "visibility": "public",
            "versioned": True,
            "fields": [{"name": name, "type": value_type} for name, value_type in fields],
        },
        "java": {"package": "org.kmipkit.extensions", "name": "AsyncOperationResponseV1"},
        "python": {"module": "kmipkit.client", "name": "AsyncOperationResponseV1"},
    }


def _public_declarations() -> list[dict[str, Any]]:
    return [
        {
            "declarationId": f"kmipkit-0052-response-{surface}-{language}",
            "packageId": package,
            "kind": kind,
            "exportName": name,
            "capabilityId": CAPABILITY_ID,
            "requirementIds": PAYLOAD_REQUIREMENTS if surface == "payload" else HEADER_REQUIREMENTS,
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit-client", "name": rust_name},
                "c": {"packageId": "c-kmipkit", "name": c_name},
                "java": {"packageId": "java-kmipkit", "name": java_export},
                "python": {"packageId": "python-kmipkit", "name": python_export},
            },
            "parityEvidenceIds": EVIDENCE,
        }
        for surface, package, kind, name, rust_name, c_name, java_export, python_export in (
            ("payload", "rust-kmipkit-client", "associated-item", "kmipkit_client::ClientBatchItemResponse::raw", "kmipkit_client::ClientBatchItemResponse::raw", "kmipkit_client_batch_item_response_raw_ttlv", "org.kmipkit.extensions.ClientBatchItemResult.raw", "kmipkit.client.BatchItemResult.raw"),
            ("header", "rust-kmipkit-client", "associated-item", "kmipkit_client::ClientBatchItemResponse::response_header_raw", "kmipkit_client::ClientBatchItemResponse::response_header_raw", "kmipkit_client_batch_item_response_header_ttlv", "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv", "kmipkit.client.BatchItemResult.response_header_ttlv"),
        )
        for language, package, kind, name in (
            ("rust", "rust-kmipkit-client", "associated-item", rust_name),
            ("c", "c-kmipkit", "c-symbol", c_name),
            ("java", "java-kmipkit", "java-member", java_export),
            ("python", "python-kmipkit", "python-export", python_export),
        )
    ]


def _async_typed_public_declarations() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    operations = (
        ("cancel", "cancelResponse", "cancel_response"),
        ("poll", "pollResponse", "poll_response"),
        ("process", "processResponse", "process_response"),
        (
            "query_async_requests",
            "queryAsyncRequestsResponse",
            "query_async_requests_response",
        ),
    )
    for slug, java_method, python_method in operations:
        c_name = f"kmipkit_client_async_operation_outcome_{slug}_response_v1"
        rust_name = f"kmipkit_client::ClientOperationOutcome::{python_method}"
        routes = {
            "rust": ("rust-kmipkit-client", rust_name, "associated-item"),
            "c": ("c-kmipkit", c_name, "c-symbol"),
            "java": (
                "java-kmipkit",
                f"org.kmipkit.extensions.AsyncOperationOutcome.{java_method}",
                "java-member",
            ),
            "python": (
                "python-kmipkit",
                f"kmipkit.client.AsyncOperationOutcome.{python_method}",
                "python-export",
            ),
        }
        for language, (package, name, kind) in routes.items():
            if language == "rust":
                continue
            rows.append({
                "declarationId": f"kmipkit-0052-async-{slug}-{language}",
                "packageId": package,
                "kind": kind,
                "exportName": name,
                "capabilityId": CAPABILITY_ID,
                "requirementIds": [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
                "declarationState": "verified",
                "parityScope": "four-language",
                "approvalSource": APPROVAL,
                "languageMappings": {
                    value_language: {"packageId": value[0], "name": value[1]}
                    for value_language, value in routes.items()
                },
                "parityEvidenceIds": EVIDENCE,
            })

    type_routes = {
        "rust": (
            "rust-kmipkit-client",
            "kmipkit_client::ClientOperationOutcome",
            "type",
        ),
        "c": (
            "c-kmipkit",
            "kmipkit_client_async_operation_response_v1_t",
            "type",
        ),
        "java": (
            "java-kmipkit",
            "org.kmipkit.extensions.AsyncOperationResponseV1",
            "type",
        ),
        "python": (
            "python-kmipkit",
            "kmipkit.client.AsyncOperationResponseV1",
            "type",
        ),
    }
    for language, (package, name, kind) in type_routes.items():
        if language == "rust":
            continue
        rows.append({
            "declarationId": f"kmipkit-0052-async-response-view-{language}",
            "packageId": package,
            "kind": kind,
            "exportName": name,
            "capabilityId": CAPABILITY_ID,
            "requirementIds": [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "languageMappings": {
                value_language: {"packageId": value[0], "name": value[1]}
                for value_language, value in type_routes.items()
            },
            "parityEvidenceIds": EVIDENCE,
        })

    for slug in ("cancel", "poll", "process", "query_async_requests"):
        java_type = {
            "cancel": "AsyncCancelResponse",
            "poll": "AsyncPollResponse",
            "process": "AsyncProcessResponse",
            "query_async_requests": "AsyncQueryAsyncRequestsResponse",
        }[slug]
        python_type = java_type
        routes = {
            "rust": ("rust-kmipkit-client", "kmipkit_client::ClientOperationOutcome", "type"),
            "c": ("c-kmipkit", "kmipkit_client_async_operation_response_v1_t", "type"),
            "java": ("java-kmipkit", f"org.kmipkit.extensions.{java_type}", "type"),
            "python": ("python-kmipkit", f"kmipkit.client.{python_type}", "type"),
        }
        for language, (package, name, kind) in routes.items():
            if language in {"rust", "c"}:
                continue
            rows.append({
                "declarationId": f"kmipkit-0052-async-{slug}-view-{language}",
                "packageId": package,
                "kind": kind,
                "exportName": name,
                "capabilityId": CAPABILITY_ID,
                "requirementIds": [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
                "declarationState": "verified",
                "parityScope": "four-language",
                "approvalSource": APPROVAL,
                "languageMappings": {
                    value_language: {"packageId": value[0], "name": value[1]}
                    for value_language, value in routes.items()
                },
                "parityEvidenceIds": EVIDENCE,
            })

    wrapper_members = (
        (
            "result",
            "kmipkit_client::ClientOperationOutcome::result",
            "kmipkit_client_async_operation_response_v1_t.result_status",
            "org.kmipkit.extensions.AsyncOperationResponseV1.result",
            "kmipkit.client.AsyncOperationResponseV1.result",
        ),
        (
            "raw",
            "kmipkit_client::ClientOperationOutcome::raw",
            "kmipkit_client_async_operation_response_v1_t.has_response_payload",
            "org.kmipkit.extensions.AsyncOperationResponseV1.raw",
            "kmipkit.client.AsyncOperationResponseV1.raw",
        ),
        (
            "cancel_result",
            "kmipkit_protocol::CancelResponse::cancellation_result",
            "kmipkit_client_async_operation_response_v1_t.cancellation_result",
            "org.kmipkit.extensions.AsyncCancelResponse.cancellationResult",
            "kmipkit.client.AsyncCancelResponse.cancellation_result",
        ),
        (
            "poll_pending",
            "kmipkit_protocol::PollResponse::is_pending",
            "kmipkit_client_async_operation_response_v1_t.is_pending",
            "org.kmipkit.extensions.AsyncPollResponse.isPending",
            "kmipkit.client.AsyncPollResponse.is_pending",
        ),
        (
            "process_pending",
            "kmipkit_protocol::ProcessResponse::is_pending",
            "kmipkit_client_async_operation_response_v1_t.is_pending",
            "org.kmipkit.extensions.AsyncProcessResponse.isPending",
            "kmipkit.client.AsyncProcessResponse.is_pending",
        ),
        (
            "query_pending",
            "kmipkit_client::ClientOperationOutcome::is_pending",
            "kmipkit_client_async_operation_response_v1_t.is_pending",
            "org.kmipkit.extensions.AsyncQueryAsyncRequestsResponse.isPending",
            "kmipkit.client.AsyncQueryAsyncRequestsResponse.is_pending",
        ),
        (
            "query_correlation",
            "kmipkit_client::ClientOperationOutcome::with_asynchronous_correlation_value",
            "kmipkit_client_async_operation_response_v1_t.asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncQueryAsyncRequestsResponse.withAsynchronousCorrelationValue",
            "kmipkit.client.AsyncQueryAsyncRequestsResponse.with_asynchronous_correlation_value",
        ),
        (
            "process_payload_count",
            "kmipkit_protocol::ProcessResponse::payload_member_count",
            "kmipkit_client_async_operation_response_v1_t.response_payload_member_count",
            "org.kmipkit.extensions.AsyncProcessResponse.payloadMemberCount",
            "kmipkit.client.AsyncProcessResponse.payload_member_count",
        ),
        (
            "correlation",
            "kmipkit_client::ClientOperationOutcome::with_asynchronous_correlation_value",
            "kmipkit_client_async_operation_response_v1_t.asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncPollResponse.withAsynchronousCorrelationValue",
            "kmipkit.client.AsyncPollResponse.with_asynchronous_correlation_value",
        ),
        (
            "cancel_correlation",
            "kmipkit_protocol::CancelResponse::with_asynchronous_correlation_value",
            "kmipkit_client_async_operation_response_v1_t.asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncCancelResponse.withEchoedCorrelationValue",
            "kmipkit.client.AsyncCancelResponse.with_echoed_correlation_value",
        ),
        (
            "process_correlation",
            "kmipkit_protocol::ProcessResponse::with_asynchronous_correlation_value",
            "kmipkit_client_async_operation_response_v1_t.asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncProcessResponse.withAsynchronousCorrelationValue",
            "kmipkit.client.AsyncProcessResponse.with_asynchronous_correlation_value",
        ),
    )
    for slug, rust_name, c_name, java_name, python_name in wrapper_members:
        routes = {
            "rust": {
                "packageId": "rust-kmipkit-client"
                if rust_name.startswith("kmipkit_client::")
                else "rust-kmipkit-protocol",
                "name": rust_name,
            },
            "c": {"packageId": "c-kmipkit", "name": c_name},
            "java": {"packageId": "java-kmipkit", "name": java_name},
            "python": {"packageId": "python-kmipkit", "name": python_name},
        }
        for language, package, kind, name in (
            ("java", "java-kmipkit", "java-member", java_name),
            ("python", "python-kmipkit", "python-export", python_name),
        ):
            rows.append({
                "declarationId": f"kmipkit-0052-async-wrapper-{slug}-{language}",
                "packageId": package,
                "kind": kind,
                "exportName": name,
                "capabilityId": CAPABILITY_ID,
                "requirementIds": [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
                "declarationState": "verified",
                "parityScope": "four-language",
                "approvalSource": APPROVAL,
                "languageMappings": routes,
                "parityEvidenceIds": EVIDENCE,
            })

    field_members = (
        (
            "operation",
            "kmipkit_client::ClientOperationOutcome::operation",
            "org.kmipkit.extensions.AsyncOperationMetadataV1.operation",
            "kmipkit.client.AsyncOperationMetadataV1.operation",
        ),
        (
            "result_status",
            "kmipkit_client::ClientOperationOutcome::result",
            "org.kmipkit.extensions.AsyncOperationMetadataV1.resultStatus",
            "kmipkit.client.AsyncOperationMetadataV1.result_status",
        ),
        (
            "is_pending",
            "kmipkit_client::ClientOperationOutcome::is_pending",
            "org.kmipkit.extensions.AsyncPollResponse.isPending",
            "kmipkit.client.AsyncPollResponse.is_pending",
        ),
        (
            "has_cancellation_result",
            "kmipkit_client::ClientOperationOutcome::cancellation_result",
            "org.kmipkit.extensions.AsyncCancelResponse.cancellationResult",
            "kmipkit.client.AsyncCancelResponse.cancellation_result",
        ),
        (
            "cancellation_result",
            "kmipkit_client::ClientOperationOutcome::cancellation_result",
            "org.kmipkit.extensions.AsyncCancelResponse.cancellationResult",
            "kmipkit.client.AsyncCancelResponse.cancellation_result",
        ),
        (
            "has_asynchronous_correlation_value",
            "kmipkit_client::ClientOperationOutcome::with_asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncPollResponse.withAsynchronousCorrelationValue",
            "kmipkit.client.AsyncPollResponse.with_asynchronous_correlation_value",
        ),
        (
            "asynchronous_correlation_value",
            "kmipkit_client::ClientOperationOutcome::with_asynchronous_correlation_value",
            "org.kmipkit.extensions.AsyncPollResponse.withAsynchronousCorrelationValue",
            "kmipkit.client.AsyncPollResponse.with_asynchronous_correlation_value",
        ),
        (
            "has_response_payload",
            "kmipkit_client::ClientOperationOutcome::raw",
            "org.kmipkit.extensions.AsyncOperationResponseV1.raw",
            "kmipkit.client.AsyncOperationResponseV1.raw",
        ),
        (
            "response_payload_member_count",
            "kmipkit_protocol::ProcessResponse::payload_member_count",
            "org.kmipkit.extensions.AsyncProcessResponse.payloadMemberCount",
            "kmipkit.client.AsyncProcessResponse.payload_member_count",
        ),
    )
    for field_name, rust_name, java_name, python_name in field_members:
        rows.append({
            "declarationId": f"kmipkit-0052-async-response-field-{field_name}",
            "packageId": "c-kmipkit",
            "kind": "field",
            "exportName": f"kmipkit_client_async_operation_response_v1_t.{field_name}",
            "capabilityId": CAPABILITY_ID,
            "requirementIds": [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
            "declarationState": "verified",
            "parityScope": "four-language",
            "approvalSource": APPROVAL,
            "languageMappings": {
                "rust": {
                    "packageId": "rust-kmipkit-client"
                    if rust_name.startswith("kmipkit_client::")
                    else "rust-kmipkit-protocol",
                    "name": rust_name,
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": f"kmipkit_client_async_operation_response_v1_t.{field_name}",
                },
                "java": {"packageId": "java-kmipkit", "name": java_name},
                "python": {"packageId": "python-kmipkit", "name": python_name},
            },
            "parityEvidenceIds": EVIDENCE,
        })
    return rows


def _async_typed_approved_exclusions() -> list[dict[str, Any]]:
    exclusions = [
        {
            "packageId": "c-kmipkit",
            "exportName": f"kmipkit_client_async_operation_response_v1_t.{field_name}",
            "reason": reason,
            "approvalSource": APPROVAL,
            "evidence": (
                "KMIPKIT-0052-FR-001 requires a versioned C response view; this field carries ABI framing and "
                "has no semantic accessor in the owner-backed Rust, Java, or Python response views."
            ),
        }
        for field_name, reason in (
            ("struct_size", "C ABI record size framing."),
            ("abi_version", "C ABI record version framing."),
            ("reserved", "Reserved C ABI layout field."),
        )
    ]
    python_routes = {
        "kmipkit.client.AsyncOperationResponseV1.__init__": (
            "The base response-view constructor is an implementation detail; typed wrappers are created only by an outcome reader.",
        ),
        "kmipkit.client.AsyncOperationResponseV1.__slots__": (
            "This exact Python storage layout member has no independent cross-language product capability.",
        ),
        "kmipkit.client.AsyncCancelResponse.__init__": (
            "This exact constructor assembles an owner-backed view from an outcome and FFI record; callers obtain it from cancel_response().",
        ),
        "kmipkit.client.AsyncCancelResponse.__slots__": (
            "This exact Python storage layout member has no independent cross-language product capability.",
        ),
        "kmipkit.client.AsyncCancelResponse.raw": (
            "This inherited alias is the generic raw() accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncCancelResponse.result": (
            "This inherited alias is the result accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncPollResponse.__init__": (
            "This exact constructor assembles an owner-backed view from an outcome and FFI record; callers obtain it from poll_response().",
        ),
        "kmipkit.client.AsyncPollResponse.__slots__": (
            "This exact Python storage layout member has no independent cross-language product capability.",
        ),
        "kmipkit.client.AsyncPollResponse.raw": (
            "This inherited alias is the generic raw() accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncPollResponse.result": (
            "This inherited alias is the result accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncProcessResponse.__init__": (
            "This exact constructor assembles an owner-backed view from an outcome and FFI record; callers obtain it from process_response().",
        ),
        "kmipkit.client.AsyncProcessResponse.__slots__": (
            "This exact Python storage layout member has no independent cross-language product capability.",
        ),
        "kmipkit.client.AsyncProcessResponse.raw": (
            "This inherited alias is the generic raw() accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncProcessResponse.result": (
            "This inherited alias is the result accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncQueryAsyncRequestsResponse.__init__": (
            "This exact constructor assembles an owner-backed view from an outcome and FFI record; callers obtain it from query_async_requests_response().",
        ),
        "kmipkit.client.AsyncQueryAsyncRequestsResponse.raw": (
            "This inherited alias is the generic raw() accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
        "kmipkit.client.AsyncQueryAsyncRequestsResponse.result": (
            "This inherited alias is the result accessor already mapped on AsyncOperationResponseV1; it adds no separate capability.",
        ),
    }
    exclusions.extend(
        {
            "packageId": "python-kmipkit",
            "exportName": export_name,
            "reason": reason,
            "approvalSource": APPROVAL,
            "evidence": (
                "KMIPKIT-0052-FR-001 and FR-007 require operation-specific, owner-backed response views. "
                "The typed outcome readers and base view members are mapped directly; these exact Python "
                "constructor, storage, and inherited alias names do not add independent behavior."
            ),
        }
        for export_name, (reason,) in python_routes.items()
    )
    return exclusions


def _additional_rust_declarations() -> list[dict[str, Any]]:
    mappings = [
        (
            "operation_outcome_raw",
            "kmipkit_client::ClientOperationOutcome::raw",
            "kmipkit_client::ClientOperationOutcome::raw",
        "c-kmipkit",
        "kmipkit_client_async_operation_outcome_response_payload_ttlv",
            "java-kmipkit",
        "org.kmipkit.extensions.AsyncOperationOutcome.responsePayloadTtlv",
            "python-kmipkit",
        "kmipkit.client.AsyncOperationOutcome.response_payload_ttlv",
            [f"{SPEC_ID}-FR-002", f"{SPEC_ID}-FR-003", f"{SPEC_ID}-FR-007"],
        ),
        (
            "batch_response_header_raw",
            "kmipkit_client::ClientBatchResponse::response_header_raw",
            "kmipkit_client::ClientBatchResponse::response_header_raw",
            "c-kmipkit",
            "kmipkit_client_batch_item_response_header_ttlv",
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv",
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.response_header_ttlv",
            [f"{SPEC_ID}-FR-006", f"{SPEC_ID}-FR-007"],
        ),
        (
            "cancel_typed_response",
            "kmipkit_client::ClientOperationOutcome::cancel_response",
            "kmipkit_client::ClientOperationOutcome::cancel_response",
            "c-kmipkit",
            "kmipkit_client_async_operation_outcome_cancel_response_v1",
            "java-kmipkit",
            "org.kmipkit.extensions.AsyncOperationOutcome.cancelResponse",
            "python-kmipkit",
            "kmipkit.client.AsyncOperationOutcome.cancel_response",
            [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
        ),
        (
            "poll_typed_response",
            "kmipkit_client::ClientOperationOutcome::poll_response",
            "kmipkit_client::ClientOperationOutcome::poll_response",
            "c-kmipkit",
            "kmipkit_client_async_operation_outcome_poll_response_v1",
            "java-kmipkit",
            "org.kmipkit.extensions.AsyncOperationOutcome.pollResponse",
            "python-kmipkit",
            "kmipkit.client.AsyncOperationOutcome.poll_response",
            [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
        ),
        (
            "process_typed_response",
            "kmipkit_client::ClientOperationOutcome::process_response",
            "kmipkit_client::ClientOperationOutcome::process_response",
            "c-kmipkit",
            "kmipkit_client_async_operation_outcome_process_response_v1",
            "java-kmipkit",
            "org.kmipkit.extensions.AsyncOperationOutcome.processResponse",
            "python-kmipkit",
            "kmipkit.client.AsyncOperationOutcome.process_response",
            [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
        ),
        (
            "query_async_typed_response",
            "kmipkit_client::ClientOperationOutcome::query_async_requests_response",
            "kmipkit_client::ClientOperationOutcome::query_async_requests_response",
            "c-kmipkit",
            "kmipkit_client_async_operation_outcome_query_async_requests_response_v1",
            "java-kmipkit",
            "org.kmipkit.extensions.AsyncOperationOutcome.queryAsyncRequestsResponse",
            "python-kmipkit",
            "kmipkit.client.AsyncOperationOutcome.query_async_requests_response",
            [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-007"],
        ),
        (
            "response_batch_item_raw",
            "kmipkit_protocol::ResponseBatchItemView::raw",
            "kmipkit_protocol::ResponseBatchItemView::raw",
            "c-kmipkit",
            "kmipkit_client_batch_item_response_raw_ttlv",
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.raw",
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.raw",
            [f"{SPEC_ID}-FR-002", f"{SPEC_ID}-FR-003", f"{SPEC_ID}-FR-007"],
        ),
        (
            "response_header_view_raw",
            "kmipkit_protocol::ResponseHeaderView::raw",
            "kmipkit_protocol::ResponseHeaderView::raw",
            "c-kmipkit",
            "kmipkit_client_batch_item_response_header_ttlv",
            "java-kmipkit",
            "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv",
            "python-kmipkit",
            "kmipkit.client.BatchItemResult.response_header_ttlv",
            [f"{SPEC_ID}-FR-006", f"{SPEC_ID}-FR-007"],
        ),
        (
            "item_as_structure",
            "kmipkit_ttlv::Item::as_structure",
            "kmipkit_ttlv::Item::as_structure",
            "c-kmipkit",
            "kmipkit_ttlv_value_view_structure",
            "java-kmipkit",
            "org.kmipkit.ttlv.TtlvValueView.structure",
            "python-kmipkit",
            "kmipkit.ttlv.ttlv_value_view_structure",
            [f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005", f"{SPEC_ID}-FR-007"],
        ),
    ]
    facade_aliases = [
        (
            "client_response_raw_facade",
            "kmipkit::ClientBatchItemResponse::raw",
            "kmipkit_client_batch_item_response_raw_ttlv",
            "org.kmipkit.extensions.ClientBatchItemResult.raw",
            "kmipkit.client.BatchItemResult.raw",
            PAYLOAD_REQUIREMENTS,
        ),
        (
            "client_response_header_facade",
            "kmipkit::ClientBatchItemResponse::response_header_raw",
            "kmipkit_client_batch_item_response_header_ttlv",
            "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv",
            "kmipkit.client.BatchItemResult.response_header_ttlv",
            HEADER_REQUIREMENTS,
        ),
    ]
    aliases = [
        ("operation_outcome_raw_facade", "kmipkit::ClientOperationOutcome::raw", 0),
        (
            "batch_response_header_raw_facade",
            "kmipkit::ClientBatchResponse::response_header_raw",
            1,
        ),
        ("cancel_typed_response_facade", "kmipkit::ClientOperationOutcome::cancel_response", 2),
        ("poll_typed_response_facade", "kmipkit::ClientOperationOutcome::poll_response", 3),
        (
            "process_typed_response_facade",
            "kmipkit::ClientOperationOutcome::process_response",
            4,
        ),
        (
            "query_async_typed_response_facade",
            "kmipkit::ClientOperationOutcome::query_async_requests_response",
            5,
        ),
        (
            "response_batch_item_raw_facade",
            "kmipkit::protocol::ResponseBatchItemView<'a>::raw",
            6,
        ),
        (
            "response_header_view_raw_facade",
            "kmipkit::protocol::ResponseHeaderView<'a>::raw",
            7,
        ),
        ("item_as_structure_facade", "kmipkit::ttlv::Item::as_structure", 8),
    ]
    output: list[dict[str, Any]] = []
    for (
        slug,
        package_route,
        module_route,
        c_package,
        c_route,
        java_package,
        java_route,
        python_package,
        python_route,
        requirement_ids,
    ) in mappings:
        output.append(_declaration(slug, package_route, module_route, c_package, c_route,
                                   java_package, java_route, python_package, python_route,
                                   requirement_ids))
    for slug, route, mapping_index in aliases:
        mapping = mappings[mapping_index] if mapping_index >= 0 else mappings[-mapping_index - 1]
        output.append(_declaration(
            slug, route, route, mapping[3], mapping[4], mapping[5], mapping[6],
            mapping[7], mapping[8], mapping[9],
        ))
    for slug, route, c_route, java_route, python_route, requirement_ids in facade_aliases:
        output.append(_declaration(
            slug,
            route,
            route,
            "c-kmipkit",
            c_route,
            "java-kmipkit",
            java_route,
            "python-kmipkit",
            python_route,
            requirement_ids,
        ))
    return output


def _declaration(
    slug: str, rust_package_route: str, rust_route: str, c_package: str, c_route: str,
    java_package: str, java_route: str, python_package: str, python_route: str,
    requirement_ids: list[str],
) -> dict[str, Any]:
    if rust_package_route.startswith("kmipkit_protocol::"):
        rust_package = "rust-kmipkit-protocol"
    elif rust_package_route.startswith("kmipkit_ttlv::"):
        rust_package = "rust-kmipkit-ttlv"
    elif rust_package_route.startswith("kmipkit::"):
        rust_package = "rust-kmipkit"
    else:
        rust_package = "rust-kmipkit-client"
    return {
        "declarationId": f"{SPEC_ID.lower()}-{slug}",
        "packageId": rust_package,
        "kind": "associated-item",
        "exportName": rust_route,
        "capabilityId": CAPABILITY_ID,
        "requirementIds": requirement_ids,
        "declarationState": "verified",
        "parityScope": "four-language",
        "approvalSource": APPROVAL,
        "languageMappings": {
            "rust": {"packageId": rust_package, "name": rust_route},
            "c": {"packageId": c_package, "name": c_route},
            "java": {"packageId": java_package, "name": java_route},
            "python": {"packageId": python_package, "name": python_route},
        },
        "parityEvidenceIds": [
            "crates/kmipkit-client/tests/unit/object_read_execution_tests.rs",
            "crates/kmipkit-client/tests/unit/query_execution_tests.rs",
            "crates/kmipkit-protocol/tests/message_accessors.rs",
            "crates/kmipkit-ttlv/tests/public_api.rs",
            "crates/kmipkit-ffi/tests/support/query_ping_abi_tests.rs",
            "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
            "bindings/python/tests/test_response_raw_navigation.py",
        ],
    }


def reconcile(manifest: dict[str, Any]) -> dict[str, Any]:
    result = json.loads(json.dumps(manifest))
    result["requirementIds"] = sorted(
        set(result["requirementIds"])
        | set(PAYLOAD_REQUIREMENTS)
        | set(HEADER_REQUIREMENTS)
        | set(TREE_REQUIREMENTS)
        | set(ASYNC_TYPED_REQUIREMENTS)
    )
    functions = result["functions"]
    for function in functions:
        if function.get("id") == "client_async_operation_outcome_response_payload_ttlv":
            function["c"]["documentation"] = [
                "Copies a present generic Response Payload as a caller-owned TTLV structure."
            ]
    expected_functions = [
        _accessor("raw", "raw", "raw", "Copies the optional raw Response Payload as an independently owned TTLV structure.", "Optional<TtlvStructure>", "ttlv.Structure | None", PAYLOAD_REQUIREMENTS),
        _accessor("header", "responseHeaderTtlv", "response_header_ttlv", "Copies the common raw Response Header as an independently owned TTLV structure.", "TtlvStructure", "ttlv.Structure", HEADER_REQUIREMENTS),
        _async_response_reader("cancel", "cancelResponse", "cancel_response", "Optional<AsyncCancelResponse>", "AsyncCancelResponse | None", "Returns the typed Cancel result and borrowed echo span; inspect its Response Payload as generic TTLV."),
        _async_response_reader("poll", "pollResponse", "poll_response", "Optional<AsyncPollResponse>", "AsyncPollResponse | None", "Returns Pending state and borrowed correlation metadata; any terminal Response Payload remains generic TTLV."),
        _async_response_reader("process", "processResponse", "process_response", "Optional<AsyncProcessResponse>", "AsyncProcessResponse | None", "Returns Pending state, payload presence/count, and borrowed correlation metadata."),
        _async_response_reader("query_async_requests", "queryAsyncRequestsResponse", "query_async_requests_response", "Optional<AsyncQueryAsyncRequestsResponse>", "AsyncQueryAsyncRequestsResponse | None", "Returns Query Asynchronous Requests status and generic TTLV. KMIP v2.1 §6.1.41 Table 286 caption/body discrepancy remains unresolved; no payload meaning is inferred."),
    ]
    by_id = {record["id"]: record for record in functions}
    for record in expected_functions:
        existing = by_id.get(record["id"])
        if existing is not None and existing != record:
            comparable_existing = dict(existing)
            comparable_record = dict(record)
            comparable_existing.pop("rust", None)
            comparable_record.pop("rust", None)
            if comparable_existing != comparable_record:
                raise ValueError(f"unexpected existing function contract: {record['id']}")
            existing["rust"] = record["rust"]
        if existing is None:
            functions.append(record)

    types = result.setdefault("types", [])
    async_view_type = _async_response_view_type()
    existing_view_type = next(
        (record for record in types if record.get("id") == async_view_type["id"]), None
    )
    if existing_view_type is not None and existing_view_type != async_view_type:
        raise ValueError("unexpected asynchronous response view record contract")
    if existing_view_type is None:
        types.append(async_view_type)

    declarations = result.setdefault("publicDeclarations", [])
    stale_async_declaration_ids = {
        "kmipkit-0052-async-response-view-rust",
        *(
            f"kmipkit-0052-async-{slug}-{language}"
            for slug in ("cancel", "poll", "process", "query_async_requests")
            for language in ("rust",)
        ),
        *(
            f"kmipkit-0052-async-{slug}-view-{language}"
            for slug in ("cancel", "poll", "process", "query_async_requests")
            for language in ("rust", "c")
        ),
    }
    declarations[:] = [
        row for row in declarations
        if row.get("declarationId") not in stale_async_declaration_ids
    ]
    declaration_by_id = {record["declarationId"]: record for record in declarations}
    for record in [
        *_public_declarations(),
        *_additional_rust_declarations(),
        *_async_typed_public_declarations(),
    ]:
        existing = declaration_by_id.get(record["declarationId"])
        if existing is not None and existing != record:
            comparable_existing = dict(existing)
            comparable_record = dict(record)
            for key in (
                "languageMappings",
                "parityEvidenceIds",
                "packageId",
                "requirementIds",
                "kind",
            ):
                comparable_existing.pop(key, None)
                comparable_record.pop(key, None)
            if comparable_existing != comparable_record:
                raise ValueError(f"unexpected existing declaration contract: {record['declarationId']}")
            existing["languageMappings"] = record["languageMappings"]
            existing["parityEvidenceIds"] = record["parityEvidenceIds"]
            existing["packageId"] = record["packageId"]
            existing["requirementIds"] = record["requirementIds"]
            existing["kind"] = record["kind"]
        if existing is None:
            declarations.append(record)

    exclusions = result.setdefault("approvedExclusions", [])
    exclusion_by_identity = {
        (record["packageId"], record["exportName"]): record for record in exclusions
    }
    for record in [*JAVA_BRIDGE_EXCLUSIONS, *_async_typed_approved_exclusions()]:
        identity = (record["packageId"], record["exportName"])
        existing = exclusion_by_identity.get(identity)
        if existing is not None and existing != record:
            raise ValueError(f"unexpected existing bridge exclusion: {identity[1]}")
        if existing is None:
            exclusions.append(record)

    capabilities = result["capabilities"]
    capability = next((row for row in capabilities if row["capabilityId"] == CAPABILITY_ID), None)
    capability_declarations = [
        *_public_declarations(),
        *_additional_rust_declarations(),
        *_async_typed_public_declarations(),
    ]
    expected_capability = {
        "capabilityId": CAPABILITY_ID,
        "requirementIds": sorted(
            set(PAYLOAD_REQUIREMENTS) | set(HEADER_REQUIREMENTS) | set(TREE_REQUIREMENTS)
            | set(ASYNC_TYPED_REQUIREMENTS)
        ),
        "capabilityState": "verified",
        "parityScope": "four-language",
        "ownerSpecId": SPEC_ID,
        "approvalEvidence": APPROVAL,
        "parityEvidenceIds": EVIDENCE,
        "mappings": {
            language: sorted({
                declaration["languageMappings"][language]["name"]
                for declaration in capability_declarations
            })
            for language in ("rust", "c", "java", "python")
        },
    }
    if capability is None:
        capabilities.append(expected_capability)
    else:
        expected_stable_fields = (
            "capabilityState",
            "parityScope",
            "ownerSpecId",
            "approvalEvidence",
        )
        if any(capability.get(key) != expected_capability[key] for key in expected_stable_fields):
            raise ValueError(f"unexpected existing capability contract: {CAPABILITY_ID}")
        for language, old_routes in capability.get("mappings", {}).items():
            allowed_removed = SUPERSEDED_ASYNC_TYPED_ROUTES.get(language, set())
            if not set(old_routes) <= (
                set(expected_capability["mappings"].get(language, [])) | allowed_removed
            ):
                raise ValueError(f"unexpected existing capability mapping: {CAPABILITY_ID}/{language}")
        capability["requirementIds"] = expected_capability["requirementIds"]
        capability["mappings"] = expected_capability["mappings"]
        capability["parityEvidenceIds"] = list(dict.fromkeys(
            [*capability.get("parityEvidenceIds", []), *expected_capability["parityEvidenceIds"]]
        ))
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    path = args.repo_root / "specification/api/public-api.json"
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
        reconciled = reconcile(manifest)
        text = json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n"
    except (OSError, json.JSONDecodeError, ValueError, StopIteration) as error:
        print(f"KMIPKIT-0052 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2
    print(
        "Reconciled four operation-specific async response readers across C, Java, "
        "Python, and Rust; "
        + ("wrote" if args.write else "would write")
        + " the reviewed manifest."
    )
    if args.write:
        path.write_text(text, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
