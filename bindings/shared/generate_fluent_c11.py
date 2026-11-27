#!/usr/bin/env python3
"""Generate and validate the C11 64/64 fluent header and cross-language parity."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "bindings" / "shared" / "fluent_ops.json"
OUT_HEADER_PATH = REPO_ROOT / "bindings" / "c" / "include" / "kmipkit_fluent_ops.h"

COMMON_EXEC_FIELDS = [
    ("kmipkit_codec_limits_t *", "limits"),
    ("const kmipkit_request_options_v1_t *", "options"),
    ("uint8_t", "has_asynchronous_indicator"),
    ("uint32_t", "asynchronous_indicator"),
]

# Each c_kind defines (fields, item_builder_body) where `fields` are (c_type, field_name).
OP_SPECS: dict[str, tuple[list[tuple[str, str]], list[str]]] = {
    "no_arg": (
        [],
        ["    return kmipkit_client_batch_item_{c_name}(out_item);"],
    ),
    "uid_only": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = (params && params->has_unique_identifier)",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_{c_name}(uid, out_item);",
        ],
    ),
    "create": (
        [
            ("uint32_t", "object_type"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    return kmipkit_client_batch_item_create(",
            "        params->object_type, params->attributes, params->protection_storage_masks, out_item);",
        ],
    ),
    "create_key_pair": (
        [
            ("kmipkit_ttlv_structure_t *", "common_attributes"),
            ("kmipkit_ttlv_structure_t *", "private_key_attributes"),
            ("kmipkit_ttlv_structure_t *", "public_key_attributes"),
            ("kmipkit_ttlv_structure_t *", "common_protection_storage_masks"),
            ("kmipkit_ttlv_structure_t *", "private_protection_storage_masks"),
            ("kmipkit_ttlv_structure_t *", "public_protection_storage_masks"),
        ],
        [
            "    return kmipkit_client_batch_item_create_key_pair(",
            "        params->common_attributes, params->private_key_attributes,",
            "        params->public_key_attributes, params->common_protection_storage_masks,",
            "        params->private_protection_storage_masks, params->public_protection_storage_masks,",
            "        out_item);",
        ],
    ),
    "register": (
        [
            ("uint32_t", "object_type"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_item_t *", "object"),
            ("uint8_t", "has_opaque_object_context"),
            ("uint32_t", "opaque_object_context"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    kmipkit_ttlv_structure_t *attrs_copy = NULL;",
            "    kmipkit_ttlv_item_t *obj_copy = NULL;",
            "    kmipkit_register_request_t *req = NULL;",
            "    kmipkit_register_request_t *next = NULL;",
            "    int32_t rc = kmipkit_ttlv_structure_copy(params->attributes, &attrs_copy);",
            "    if (rc != 0) return rc;",
            "    rc = kmipkit_ttlv_item_copy(params->object, &obj_copy);",
            "    if (rc != 0) { kmipkit_ttlv_structure_release(attrs_copy); return rc; }",
            "    /* kmipkit_register_request_create consumes attrs_copy and obj_copy on entry. */",
            "    rc = kmipkit_register_request_create(params->object_type, attrs_copy, obj_copy, &req);",
            "    if (rc != 0) return rc;",
            "    if (params->has_opaque_object_context) {",
            "        rc = kmipkit_register_request_with_opaque_object_context(",
            "            req, params->opaque_object_context, &next);",
            "        kmipkit_register_request_release(req);",
            "        if (rc != 0) return rc;",
            "        req = next;",
            "        next = NULL;",
            "    }",
            "    if (params->protection_storage_masks != NULL) {",
            "        rc = kmipkit_register_request_with_protection_storage_masks(",
            "            req, params->protection_storage_masks, &next);",
            "        kmipkit_register_request_release(req);",
            "        if (rc != 0) return rc;",
            "        req = next;",
            "    }",
            "    /* kmipkit_client_batch_item_register consumes req on entry. */",
            "    return kmipkit_client_batch_item_register(req, out_item);",
        ],
    ),
    "rekey": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_offset"),
            ("uint32_t", "offset"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_rekey(",
            "        uid, params->has_offset, params->offset, params->attributes,",
            "        params->protection_storage_masks, out_item);",
        ],
    ),
    "derive_key": (
        [
            ("uint32_t", "object_type"),
            ("const kmipkit_unique_identifier_v1_t *", "unique_identifiers"),
            ("uint64_t", "unique_identifier_count"),
            ("uint32_t", "derivation_method"),
            ("kmipkit_ttlv_structure_t *", "derivation_parameters"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
        ],
        [
            "    return kmipkit_client_batch_item_derive_key(",
            "        params->object_type, params->unique_identifiers, params->unique_identifier_count,",
            "        params->derivation_method, params->derivation_parameters, params->attributes, out_item);",
        ],
    ),
    "certify": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_certificate_request_type"),
            ("uint32_t", "certificate_request_type"),
            ("uint8_t", "has_certificate_request_value"),
            ("kmipkit_byte_span_v1_t", "certificate_request_value"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *crv = params->has_certificate_request_value",
            "        ? &params->certificate_request_value : NULL;",
            "    return kmipkit_client_batch_item_certify(",
            "        uid, params->has_certificate_request_type, params->certificate_request_type,",
            "        crv, params->attributes, params->protection_storage_masks, out_item);",
        ],
    ),
    "recertify": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_certificate_request_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "certificate_request_unique_identifier"),
            ("uint8_t", "has_certificate_request_type"),
            ("uint32_t", "certificate_request_type"),
            ("uint8_t", "has_certificate_request_value"),
            ("kmipkit_byte_span_v1_t", "certificate_request_value"),
            ("uint8_t", "has_offset"),
            ("uint32_t", "offset"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_unique_identifier_v1_t *cruid = params->has_certificate_request_unique_identifier",
            "        ? &params->certificate_request_unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *crv = params->has_certificate_request_value",
            "        ? &params->certificate_request_value : NULL;",
            "    return kmipkit_client_batch_item_recertify(",
            "        uid, cruid, params->has_certificate_request_type, params->certificate_request_type,",
            "        crv, params->has_offset, params->offset, params->attributes,",
            "        params->protection_storage_masks, out_item);",
        ],
    ),
    "locate": (
        [
            ("uint8_t", "has_maximum_items"),
            ("int32_t", "maximum_items"),
            ("uint8_t", "has_offset_items"),
            ("int32_t", "offset_items"),
            ("uint8_t", "has_storage_status_mask"),
            ("uint32_t", "storage_status_mask"),
            ("uint8_t", "has_object_group_member"),
            ("uint32_t", "object_group_member"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
        ],
        [
            "    return kmipkit_client_batch_item_locate(",
            "        params->has_maximum_items, params->maximum_items,",
            "        params->has_offset_items, params->offset_items,",
            "        params->has_storage_status_mask, params->storage_status_mask,",
            "        params->has_object_group_member, params->object_group_member,",
            "        params->attributes, out_item);",
        ],
    ),
    "check": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_usage_limits_count"),
            ("int64_t", "usage_limits_count"),
            ("uint8_t", "has_cryptographic_usage_mask"),
            ("int32_t", "cryptographic_usage_mask"),
            ("uint8_t", "has_lease_time"),
            ("uint32_t", "lease_time"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_check(",
            "        uid, params->has_usage_limits_count, params->usage_limits_count,",
            "        params->has_cryptographic_usage_mask, params->cryptographic_usage_mask,",
            "        params->has_lease_time, params->lease_time, out_item);",
        ],
    ),
    "get": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_key_format_type"),
            ("uint32_t", "key_format_type"),
            ("uint8_t", "has_key_wrap_type"),
            ("uint32_t", "key_wrap_type"),
            ("uint8_t", "has_key_compression_type"),
            ("uint32_t", "key_compression_type"),
            ("kmipkit_ttlv_structure_t *", "key_wrapping_specification"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_get(",
            "        uid, params->has_key_format_type, params->key_format_type,",
            "        params->has_key_wrap_type, params->key_wrap_type,",
            "        params->has_key_compression_type, params->key_compression_type,",
            "        params->key_wrapping_specification, out_item);",
        ],
    ),
    "export": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_key_format_type"),
            ("uint32_t", "key_format_type"),
            ("uint8_t", "has_key_wrap_type"),
            ("uint32_t", "key_wrap_type"),
            ("uint8_t", "has_key_compression_type"),
            ("uint32_t", "key_compression_type"),
            ("kmipkit_ttlv_structure_t *", "key_wrapping_specification"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_export(",
            "        uid, params->has_key_format_type, params->key_format_type,",
            "        params->has_key_wrap_type, params->key_wrap_type,",
            "        params->has_key_compression_type, params->key_compression_type,",
            "        params->key_wrapping_specification, out_item);",
        ],
    ),
    "get_attributes": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("const kmipkit_attribute_reference_v1_t *", "attribute_references"),
            ("uint64_t", "attribute_reference_count"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_get_attributes(",
            "        uid, params->attribute_references, params->attribute_reference_count, out_item);",
        ],
    ),
    "get_attribute_list": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_get_attribute_list(uid, out_item);",
        ],
    ),
    "add_attribute": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_item_t *", "new_attribute"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_add_attribute(uid, params->new_attribute, out_item);",
        ],
    ),
    "set_attribute": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_item_t *", "new_attribute"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_set_attribute(uid, params->new_attribute, out_item);",
        ],
    ),
    "modify_attribute": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_item_t *", "current_attribute"),
            ("kmipkit_ttlv_item_t *", "new_attribute"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_modify_attribute(",
            "        uid, params->current_attribute, params->new_attribute, out_item);",
        ],
    ),
    "delete_attribute": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_item_t *", "current_attribute"),
            ("uint8_t", "has_attribute_reference"),
            ("kmipkit_attribute_reference_v1_t", "attribute_reference"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_attribute_reference_v1_t *ref = params->has_attribute_reference",
            "        ? &params->attribute_reference : NULL;",
            "    return kmipkit_client_batch_item_delete_attribute(",
            "        uid, params->current_attribute, ref, out_item);",
        ],
    ),
    "adjust_attribute": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_attribute_reference_v1_t", "attribute_reference"),
            ("uint32_t", "adjustment_type"),
            ("kmipkit_ttlv_value_t *", "adjustment_value"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_adjust_attribute(",
            "        uid, &params->attribute_reference, params->adjustment_type,",
            "        params->adjustment_value, out_item);",
        ],
    ),
    "get_usage_allocation": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("int64_t", "usage_limits_count"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_get_usage_allocation(",
            "        uid, params->usage_limits_count, out_item);",
        ],
    ),
    "revoke": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint32_t", "revocation_reason_code"),
            ("uint8_t", "has_revocation_message"),
            ("const uint8_t *", "revocation_message_data"),
            ("uint64_t", "revocation_message_length"),
            ("uint8_t", "has_compromise_occurrence_date"),
            ("int64_t", "compromise_occurrence_date"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    kmipkit_revocation_reason_t *reason = NULL;",
            "    kmipkit_revoke_request_t *req = NULL;",
            "    int32_t rc = kmipkit_revocation_reason_create(",
            "        params->revocation_reason_code, params->has_revocation_message,",
            "        params->revocation_message_data, params->revocation_message_length, &reason);",
            "    if (rc != 0) return rc;",
            "    /* kmipkit_revoke_request_create consumes reason on entry. */",
            "    rc = kmipkit_revoke_request_create(",
            "        uid, reason, params->has_compromise_occurrence_date,",
            "        params->compromise_occurrence_date, &req);",
            "    if (rc != 0) return rc;",
            "    /* kmipkit_client_batch_item_revoke consumes req on entry. */",
            "    return kmipkit_client_batch_item_revoke(req, out_item);",
        ],
    ),
    "validate": (
        [
            ("kmipkit_ttlv_structure_t **", "certificates"),
            ("uint64_t", "certificate_count"),
            ("const kmipkit_unique_identifier_v1_t *", "identifiers"),
            ("uint64_t", "identifier_count"),
            ("kmipkit_ttlv_item_t **", "payload_extensions"),
            ("uint64_t", "payload_extension_count"),
            ("uint8_t", "has_validity_date"),
            ("int64_t", "validity_date"),
        ],
        [
            "    return kmipkit_client_batch_item_validate(",
            "        params->certificates, params->certificate_count,",
            "        params->identifiers, params->identifier_count,",
            "        params->payload_extensions, params->payload_extension_count,",
            "        params->has_validity_date, params->validity_date, out_item);",
        ],
    ),
    "query": (
        [
            ("const uint32_t *", "functions"),
            ("uint64_t", "function_count"),
            ("uint8_t", "has_object_groups"),
            ("const kmipkit_byte_span_v1_t *", "object_groups"),
            ("uint64_t", "object_group_count"),
        ],
        [
            "    return kmipkit_client_batch_item_query(",
            "        params->functions, params->function_count,",
            "        params->has_object_groups, params->object_groups, params->object_group_count, out_item);",
        ],
    ),
    "cancel": (
        [
            ("const uint8_t *", "correlation_value_data"),
            ("uint64_t", "correlation_value_length"),
        ],
        [
            "    kmipkit_cancel_request_t *req = NULL;",
            "    int32_t rc = kmipkit_cancel_request_create(",
            "        params->correlation_value_data, params->correlation_value_length, &req);",
            "    if (rc != 0) return rc;",
            "    rc = kmipkit_client_batch_item_cancel_from_request(req, out_item);",
            "    kmipkit_cancel_request_release(req);",
            "    return rc;",
        ],
    ),
    "poll": (
        [
            ("const uint8_t *", "correlation_value_data"),
            ("uint64_t", "correlation_value_length"),
        ],
        [
            "    kmipkit_poll_request_t *req = NULL;",
            "    int32_t rc = kmipkit_poll_request_create(",
            "        params->correlation_value_data, params->correlation_value_length, &req);",
            "    if (rc != 0) return rc;",
            "    rc = kmipkit_client_batch_item_poll_from_request(req, out_item);",
            "    kmipkit_poll_request_release(req);",
            "    return rc;",
        ],
    ),
    "process": (
        [
            ("const uint8_t *", "correlation_value_data"),
            ("uint64_t", "correlation_value_length"),
        ],
        [
            "    kmipkit_process_request_t *req = NULL;",
            "    int32_t rc = kmipkit_process_request_create(",
            "        params->correlation_value_data, params->correlation_value_length, &req);",
            "    if (rc != 0) return rc;",
            "    rc = kmipkit_client_batch_item_process_from_request(req, out_item);",
            "    kmipkit_process_request_release(req);",
            "    return rc;",
        ],
    ),
    "query_async_requests": (
        [
            ("const kmipkit_byte_span_v1_t *", "correlation_values"),
            ("uint64_t", "correlation_value_count"),
            ("const uint32_t *", "operations"),
            ("uint64_t", "operation_count"),
        ],
        [
            "    kmipkit_query_async_requests_request_t *req = NULL;",
            "    kmipkit_query_async_requests_request_t *next = NULL;",
            "    int32_t rc = kmipkit_query_async_requests_request_create(&req);",
            "    if (rc != 0) return rc;",
            "    if (params->correlation_values != NULL || params->correlation_value_count > 0) {",
            "        rc = kmipkit_query_async_requests_request_with_correlation_values(",
            "            req, params->correlation_values, params->correlation_value_count, &next);",
            "        kmipkit_query_async_requests_request_release(req);",
            "        if (rc != 0) return rc;",
            "        req = next;",
            "        next = NULL;",
            "    }",
            "    if (params->operations != NULL || params->operation_count > 0) {",
            "        rc = kmipkit_query_async_requests_request_with_operations(",
            "            req, params->operations, params->operation_count, &next);",
            "        kmipkit_query_async_requests_request_release(req);",
            "        if (rc != 0) return rc;",
            "        req = next;",
            "    }",
            "    rc = kmipkit_client_batch_item_query_async_requests_from_request(req, out_item);",
            "    kmipkit_query_async_requests_request_release(req);",
            "    return rc;",
        ],
    ),
    "rekey_key_pair": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint8_t", "has_offset"),
            ("uint32_t", "offset"),
            ("kmipkit_ttlv_structure_t *", "common_attributes"),
            ("kmipkit_ttlv_structure_t *", "private_key_attributes"),
            ("kmipkit_ttlv_structure_t *", "public_key_attributes"),
            ("kmipkit_ttlv_structure_t *", "common_protection_storage_masks"),
            ("kmipkit_ttlv_structure_t *", "private_protection_storage_masks"),
            ("kmipkit_ttlv_structure_t *", "public_protection_storage_masks"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_rekey_key_pair(",
            "        uid, params->has_offset, params->offset, params->common_attributes,",
            "        params->private_key_attributes, params->public_key_attributes,",
            "        params->common_protection_storage_masks, params->private_protection_storage_masks,",
            "        params->public_protection_storage_masks, out_item);",
        ],
    ),
    "discover_versions": (
        [
            ("const kmipkit_protocol_version_v1_t *", "versions"),
            ("uint64_t", "version_count"),
        ],
        [
            "    if (params != NULL && params->version_count > 0) {",
            "        return kmipkit_client_batch_item_discover_versions_with_versions(",
            "            params->versions, params->version_count, out_item);",
            "    }",
            "    return kmipkit_client_batch_item_discover_versions(out_item);",
        ],
    ),
    "encrypt": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_iv_counter_nonce"),
            ("kmipkit_byte_span_v1_t", "iv_counter_nonce"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
            ("uint8_t", "has_aad"),
            ("kmipkit_byte_span_v1_t", "aad"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *iv = params->has_iv_counter_nonce ? &params->iv_counter_nonce : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    const kmipkit_byte_span_v1_t *aad = params->has_aad ? &params->aad : NULL;",
            "    return kmipkit_client_batch_item_encrypt(",
            "        uid, params->parameters, params->data, iv, corr,",
            "        params->init_indicator, params->final_indicator, aad, out_item);",
        ],
    ),
    "decrypt": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_iv_counter_nonce"),
            ("kmipkit_byte_span_v1_t", "iv_counter_nonce"),
            ("uint8_t", "has_authenticated_encryption_tag"),
            ("kmipkit_byte_span_v1_t", "authenticated_encryption_tag"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
            ("uint8_t", "has_aad"),
            ("kmipkit_byte_span_v1_t", "aad"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *iv = params->has_iv_counter_nonce ? &params->iv_counter_nonce : NULL;",
            "    const kmipkit_byte_span_v1_t *tag = params->has_authenticated_encryption_tag ? &params->authenticated_encryption_tag : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    const kmipkit_byte_span_v1_t *aad = params->has_aad ? &params->aad : NULL;",
            "    return kmipkit_client_batch_item_decrypt(",
            "        uid, params->parameters, params->data, iv, tag, corr,",
            "        params->init_indicator, params->final_indicator, aad, out_item);",
        ],
    ),
    "sign": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_digested_data"),
            ("kmipkit_byte_span_v1_t", "digested_data"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *dig = params->has_digested_data ? &params->digested_data : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    return kmipkit_client_batch_item_sign(",
            "        uid, params->parameters, params->data, dig, corr,",
            "        params->init_indicator, params->final_indicator, out_item);",
        ],
    ),
    "signature_verify": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_digested_data"),
            ("kmipkit_byte_span_v1_t", "digested_data"),
            ("uint8_t", "has_signature_data"),
            ("kmipkit_byte_span_v1_t", "signature_data"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *dig = params->has_digested_data ? &params->digested_data : NULL;",
            "    const kmipkit_byte_span_v1_t *sig = params->has_signature_data ? &params->signature_data : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    return kmipkit_client_batch_item_signature_verify(",
            "        uid, params->parameters, params->data, dig, sig, corr,",
            "        params->init_indicator, params->final_indicator, out_item);",
        ],
    ),
    "mac": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    return kmipkit_client_batch_item_mac(",
            "        uid, params->parameters, params->data, corr,",
            "        params->init_indicator, params->final_indicator, out_item);",
        ],
    ),
    "mac_verify": (
        [
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_mac_data"),
            ("kmipkit_byte_span_v1_t", "mac_data"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    const kmipkit_byte_span_v1_t *mac = params->has_mac_data ? &params->mac_data : NULL;",
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    return kmipkit_client_batch_item_mac_verify(",
            "        uid, params->parameters, params->data, mac, corr,",
            "        params->init_indicator, params->final_indicator, out_item);",
        ],
    ),
    "hash": (
        [
            ("kmipkit_ttlv_structure_t *", "parameters"),
            ("kmipkit_ttlv_value_t *", "data"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("uint8_t", "init_indicator"),
            ("uint8_t", "final_indicator"),
        ],
        [
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    return kmipkit_client_batch_item_hash(",
            "        params->parameters, params->data, corr,",
            "        params->init_indicator, params->final_indicator, out_item);",
        ],
    ),
    "rng_retrieve": (
        [("int32_t", "data_length")],
        ["    return kmipkit_client_batch_item_rng_retrieve(params->data_length, out_item);"],
    ),
    "rng_seed": (
        [("kmipkit_byte_span_v1_t", "data")],
        ["    return kmipkit_client_batch_item_rng_seed(&params->data, out_item);"],
    ),
    "create_split_key": (
        [
            ("uint32_t", "object_type"),
            ("uint8_t", "has_unique_identifier"),
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("int32_t", "split_key_parts"),
            ("int32_t", "split_key_threshold"),
            ("uint32_t", "split_key_method"),
            ("uint8_t", "has_prime_field_size"),
            ("const uint8_t *", "prime_field_size_data"),
            ("uint64_t", "prime_field_size_length"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    const kmipkit_unique_identifier_v1_t *uid = params->has_unique_identifier",
            "        ? &params->unique_identifier : NULL;",
            "    return kmipkit_client_batch_item_create_split_key(",
            "        params->object_type, uid, params->split_key_parts, params->split_key_threshold,",
            "        params->split_key_method, params->has_prime_field_size,",
            "        params->prime_field_size_data, params->prime_field_size_length,",
            "        params->attributes, params->protection_storage_masks, out_item);",
        ],
    ),
    "join_split_key": (
        [
            ("uint32_t", "object_type"),
            ("kmipkit_split_key_reference_t **", "references"),
            ("uint64_t", "reference_count"),
            ("uint8_t", "has_secret_data_type"),
            ("uint32_t", "secret_data_type"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "protection_storage_masks"),
        ],
        [
            "    return kmipkit_client_batch_item_join_split_key(",
            "        params->object_type, params->references, params->reference_count,",
            "        params->has_secret_data_type, params->secret_data_type,",
            "        params->attributes, params->protection_storage_masks, out_item);",
        ],
    ),
    "import_object": (
        [
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("uint32_t", "object_type"),
            ("uint8_t", "has_replace_existing"),
            ("uint8_t", "replace_existing"),
            ("uint8_t", "has_key_wrap_type"),
            ("uint32_t", "key_wrap_type"),
            ("kmipkit_ttlv_structure_t *", "attributes"),
            ("kmipkit_ttlv_structure_t *", "any_object"),
        ],
        [
            "    return kmipkit_client_batch_item_import(",
            "        &params->unique_identifier, params->object_type,",
            "        params->has_replace_existing, params->replace_existing,",
            "        params->has_key_wrap_type, params->key_wrap_type,",
            "        params->attributes, params->any_object, out_item);",
        ],
    ),
    "log": (
        [
            ("const uint8_t *", "log_message_data"),
            ("uint64_t", "log_message_length"),
        ],
        [
            "    return kmipkit_client_batch_item_log(",
            "        params->log_message_data, params->log_message_length, out_item);",
        ],
    ),
    "login": (
        [
            ("uint32_t", "lease_time_kind"),
            ("int64_t", "lease_time_value"),
            ("uint8_t", "has_request_count"),
            ("int32_t", "request_count"),
            ("kmipkit_ttlv_structure_t *", "usage_limits"),
        ],
        [
            "    return kmipkit_client_batch_item_login(",
            "        params->lease_time_kind, params->lease_time_value,",
            "        params->has_request_count, params->request_count, params->usage_limits, out_item);",
        ],
    ),
    "logout": (
        [("kmipkit_ttlv_structure_t *", "ticket")],
        ["    return kmipkit_client_batch_item_logout(params->ticket, out_item);"],
    ),
    "delegated_login": (
        [
            ("kmipkit_ttlv_structure_t *", "rights"),
            ("uint32_t", "lease_time_kind"),
            ("int64_t", "lease_time_value"),
            ("uint8_t", "has_request_count"),
            ("int32_t", "request_count"),
            ("kmipkit_ttlv_structure_t *", "usage_limits"),
        ],
        [
            "    return kmipkit_client_batch_item_delegated_login(",
            "        params->rights, params->lease_time_kind, params->lease_time_value,",
            "        params->has_request_count, params->request_count, params->usage_limits, out_item);",
        ],
    ),
    "set_endpoint_role": (
        [
            ("uint32_t", "endpoint_role"),
            ("const uint8_t *", "vendor_identification_data"),
            ("uint64_t", "vendor_identification_length"),
        ],
        [
            "    return kmipkit_client_batch_item_set_endpoint_role(",
            "        params->endpoint_role, params->vendor_identification_data,",
            "        params->vendor_identification_length, out_item);",
        ],
    ),
    "pkcs11": (
        [
            ("kmipkit_ttlv_structure_t *", "function"),
            ("kmipkit_ttlv_structure_t *", "interface_spec"),
            ("uint8_t", "has_correlation_value"),
            ("kmipkit_byte_span_v1_t", "correlation_value"),
            ("kmipkit_ttlv_structure_t *", "input_parameters"),
            ("kmipkit_ttlv_item_t **", "payload_extensions"),
            ("uint64_t", "payload_extension_count"),
        ],
        [
            "    const kmipkit_byte_span_v1_t *corr = params->has_correlation_value ? &params->correlation_value : NULL;",
            "    if (params->payload_extension_count > 0) {",
            "        return kmipkit_client_batch_item_pkcs11_with_extensions_v1(",
            "            params->function, params->interface_spec, corr, params->input_parameters,",
            "            params->payload_extensions, params->payload_extension_count, out_item);",
            "    }",
            "    return kmipkit_client_batch_item_pkcs11(",
            "        params->function, params->interface_spec, corr, params->input_parameters, out_item);",
        ],
    ),
    "interop": (
        [
            ("uint32_t", "interop_function"),
            ("const uint8_t *", "interop_identifier_data"),
            ("uint64_t", "interop_identifier_length"),
        ],
        [
            "    return kmipkit_client_batch_item_interop(",
            "        params->interop_function, params->interop_identifier_data,",
            "        params->interop_identifier_length, out_item);",
        ],
    ),
    "reprovision": (
        [
            ("kmipkit_ttlv_structure_t *", "certificate_request"),
            ("kmipkit_ttlv_structure_t *", "certificate"),
        ],
        [
            "    return kmipkit_client_batch_item_reprovision(",
            "        params->certificate_request, params->certificate, out_item);",
        ],
    ),
    "set_defaults": (
        [("kmipkit_ttlv_structure_t *", "defaults_information")],
        ["    return kmipkit_client_batch_item_set_defaults(params->defaults_information, out_item);"],
    ),
    "set_constraints": (
        [("kmipkit_ttlv_structure_t *", "constraints")],
        ["    return kmipkit_client_batch_item_set_constraints(params->constraints, out_item);"],
    ),
    "create_group": (
        [("kmipkit_attribute_set_t *", "attributes")],
        ["    return kmipkit_client_batch_item_create_group_from_attributes(params->attributes, out_item);"],
    ),
    "create_user": (
        [("kmipkit_attribute_set_t *", "attributes")],
        ["    return kmipkit_client_batch_item_create_user_from_attributes(params->attributes, out_item);"],
    ),
    "create_credential": (
        [
            ("uint32_t", "credential_type_raw"),
            ("uint32_t", "object_type_raw"),
            ("kmipkit_attribute_set_t *", "attributes"),
            ("kmipkit_ttlv_item_t *", "object"),
        ],
        [
            "    return kmipkit_client_batch_item_create_credential_from_fields(",
            "        params->credential_type_raw, params->object_type_raw,",
            "        params->attributes, params->object, out_item);",
        ],
    ),
    "obliterate": (
        [("kmipkit_unique_identifier_v1_t", "unique_identifier")],
        ["    return kmipkit_client_batch_item_obliterate_from_identifier(&params->unique_identifier, out_item);"],
    ),
    "deactivate": (
        [
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_ttlv_structure_t *", "deactivation_reason"),
            ("uint8_t", "has_deactivation_date"),
            ("int64_t", "deactivation_date"),
        ],
        [
            "    return kmipkit_client_batch_item_deactivate_from_fields(",
            "        &params->unique_identifier, params->deactivation_reason,",
            "        params->has_deactivation_date, params->deactivation_date, out_item);",
        ],
    ),
    "encapsulate": (
        [
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("kmipkit_cryptographic_parameters_t *", "cryptographic_parameters"),
            ("kmipkit_attribute_set_t *", "attributes"),
        ],
        [
            "    return kmipkit_client_batch_item_encapsulate_from_fields(",
            "        &params->unique_identifier, params->cryptographic_parameters, params->attributes, out_item);",
        ],
    ),
    "decapsulate": (
        [
            ("kmipkit_unique_identifier_v1_t", "unique_identifier"),
            ("const uint8_t *", "ciphertext"),
            ("uint64_t", "ciphertext_length"),
            ("kmipkit_cryptographic_parameters_t *", "cryptographic_parameters"),
            ("kmipkit_attribute_set_t *", "attributes"),
        ],
        [
            "    return kmipkit_client_batch_item_decapsulate_from_fields(",
            "        &params->unique_identifier, params->ciphertext, params->ciphertext_length,",
            "        params->cryptographic_parameters, params->attributes, out_item);",
        ],
    ),
}

PREAMBLE = """/* Generated by bindings/shared/generate_fluent_c11.py from bindings/shared/fluent_ops.json.
 * DO NOT EDIT MANUALLY.
 *
 * C11 ergonomic fluent layer covering all 64 client-to-server KMIP operations (v2.0/v2.1/v3.0)
 * over the stable C ABI v1.
 */
#ifndef KMIPKIT_FLUENT_OPS_H
#define KMIPKIT_FLUENT_OPS_H

#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

static inline uint64_t kmipkit_fluent_cstr_len_(const char *s) {
    return s == NULL ? 0u : (uint64_t)strlen(s);
}

/* Ergonomic value & identifier initializers (C11 compound literals / structs). */
#define KMIPKIT_UID_TEXT(text_ptr) \\
    ((kmipkit_unique_identifier_v1_t){ \\
        sizeof(kmipkit_unique_identifier_v1_t), 1u, \\
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING, 0u, \\
        (const uint8_t *)(text_ptr), kmipkit_fluent_cstr_len_(text_ptr), 0u, 0 })

#define KMIPKIT_UID_IDENTIFIER(text_ptr) \\
    ((kmipkit_unique_identifier_v1_t){ \\
        sizeof(kmipkit_unique_identifier_v1_t), 1u, \\
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER, 0u, \\
        (const uint8_t *)(text_ptr), kmipkit_fluent_cstr_len_(text_ptr), 0u, 0 })

#define KMIPKIT_UID_ENUM(enum_val) \\
    ((kmipkit_unique_identifier_v1_t){ \\
        sizeof(kmipkit_unique_identifier_v1_t), 1u, \\
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION, 0u, \\
        NULL, 0u, (uint32_t)(enum_val), 0 })

#define KMIPKIT_UID_INT(int_val) \\
    ((kmipkit_unique_identifier_v1_t){ \\
        sizeof(kmipkit_unique_identifier_v1_t), 1u, \\
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER, 0u, \\
        NULL, 0u, 0u, (int32_t)(int_val) })

#define KMIPKIT_UID_FOR_VERSION(text_ptr, is_v3) \\
    ((is_v3) ? KMIPKIT_UID_IDENTIFIER(text_ptr) : KMIPKIT_UID_TEXT(text_ptr))

#define KMIPKIT_SPAN_BYTES(ptr, len) \\
    ((kmipkit_byte_span_v1_t){ (const uint8_t *)(ptr), (uint64_t)(len) })

#define KMIPKIT_SPAN_CSTR(text_ptr) \\
    ((kmipkit_byte_span_v1_t){ (const uint8_t *)(text_ptr), kmipkit_fluent_cstr_len_(text_ptr) })

#define KMIPKIT_ATTR_REF_TAG(raw_tag) \\
    ((kmipkit_attribute_reference_v1_t){ \\
        sizeof(kmipkit_attribute_reference_v1_t), 1u, 1u, 0u, (uint32_t)(raw_tag), 0u, NULL, 0u, NULL, 0u })

#define KMIPKIT_ATTR_REF_CUSTOM(vendor_cstr, name_cstr) \\
    ((kmipkit_attribute_reference_v1_t){ \\
        sizeof(kmipkit_attribute_reference_v1_t), 1u, 2u, 0u, 0u, 0u, \\
        (const uint8_t *)(vendor_cstr), kmipkit_fluent_cstr_len_(vendor_cstr), \\
        (const uint8_t *)(name_cstr), kmipkit_fluent_cstr_len_(name_cstr) })

#define KMIPKIT_TIMEOUT_LIMIT_NONE ((kmipkit_timeout_limit_v1_t){ 0u, 0u, 0u, 0u, 0u })
#define KMIPKIT_TIMEOUT_LIMIT_SECS(secs) ((kmipkit_timeout_limit_v1_t){ 2u, 0u, (uint64_t)(secs), 0u, 0u })

#define KMIPKIT_REQUEST_OPTIONS_INIT { \\
    sizeof(kmipkit_request_options_v1_t), 1u, \\
    { 0u, 0u, 0u, 0u, 0u }, { 0u, 0u, 0u, 0u, 0u }, \\
    { 0u, 0u, 0u, 0u, 0u }, { 0u, 0u, 0u, 0u, 0u } \\
}

/* Single-release result wrapper for fluent single-operation exchanges. */
typedef struct kmipkit_fluent_result_t {
    kmipkit_client_batch_response_t *response;
    kmipkit_client_batch_item_response_t *item;
    uint32_t operation;
    uint32_t outcome_kind;
    uint32_t result_status;
    uint8_t has_result_reason;
    uint32_t result_reason;
    uint8_t has_result_message;
    const uint8_t *result_message_data;
    uint64_t result_message_length;
} kmipkit_fluent_result_t;

#define KMIPKIT_FLUENT_RESULT_INIT { NULL, NULL, 0u, 0u, 0u, 0u, 0u, 0u, NULL, 0u }

static inline void kmipkit_fluent_result_release(kmipkit_fluent_result_t *res) {
    if (res == NULL) return;
    if (res->item != NULL) {
        kmipkit_client_batch_item_response_release(res->item);
        res->item = NULL;
    }
    if (res->response != NULL) {
        kmipkit_client_batch_response_release(res->response);
        res->response = NULL;
    }
    memset(res, 0, sizeof(*res));
}

static inline int32_t kmipkit_fluent_execute_item(
    kmipkit_client_t *client,
    kmipkit_client_batch_item_t *item,
    kmipkit_codec_limits_t *limits_or_null,
    const kmipkit_request_options_v1_t *options_or_null,
    uint8_t has_async_indicator,
    uint32_t async_indicator,
    kmipkit_fluent_result_t *out_result
) {
    if (client == NULL || item == NULL || out_result == NULL) {
        if (item != NULL) kmipkit_client_batch_item_release(item);
        return 4; /* KMIPKIT_ERROR_INVALID_INPUT */
    }
    memset(out_result, 0, sizeof(*out_result));

    kmipkit_codec_limits_t *owned_limits = NULL;
    kmipkit_codec_limits_t *eff_limits = limits_or_null;
    if (eff_limits == NULL) {
        int32_t lrc = kmipkit_codec_limits_defaults(&owned_limits);
        if (lrc != 0) {
            kmipkit_client_batch_item_release(item);
            return lrc;
        }
        eff_limits = owned_limits;
    }

    kmipkit_client_batch_t *batch = NULL;
    int32_t rc = kmipkit_client_batch_create(&batch);
    if (rc != 0) {
        kmipkit_client_batch_item_release(item);
        if (owned_limits != NULL) kmipkit_codec_limits_release(owned_limits);
        return rc;
    }

    kmipkit_client_batch_item_t *owned_item = item;
    rc = kmipkit_client_batch_add_item(batch, &owned_item);
    if (rc != 0) {
        if (owned_item != NULL) kmipkit_client_batch_item_release(owned_item);
        kmipkit_client_batch_release(batch);
        if (owned_limits != NULL) kmipkit_codec_limits_release(owned_limits);
        return rc;
    }

    if (has_async_indicator) {
        rc = kmipkit_client_batch_set_asynchronous_indicator(batch, async_indicator);
        if (rc != 0) {
            kmipkit_client_batch_release(batch);
            if (owned_limits != NULL) kmipkit_codec_limits_release(owned_limits);
            return rc;
        }
    }

    kmipkit_client_batch_response_t *resp = NULL;
    rc = options_or_null == NULL
        ? kmipkit_client_execute(client, &batch, eff_limits, &resp)
        : kmipkit_client_execute_with_options_v1(client, &batch, eff_limits, options_or_null, &resp);
    if (batch != NULL) kmipkit_client_batch_release(batch);
    if (owned_limits != NULL) kmipkit_codec_limits_release(owned_limits);
    if (rc != 0) {
        if (resp != NULL) kmipkit_client_batch_response_release(resp);
        return rc;
    }

    kmipkit_client_batch_item_response_t *resp_item = NULL;
    rc = kmipkit_client_batch_response_item_at(resp, 0u, &resp_item);
    if (rc != 0) {
        kmipkit_client_batch_response_release(resp);
        return rc;
    }

    kmipkit_client_batch_item_metadata_v2_t meta;
    memset(&meta, 0, sizeof(meta));
    meta.struct_size = (uint32_t)sizeof(meta);
    meta.abi_version = 2u;
    rc = kmipkit_client_batch_item_response_metadata_v2(resp_item, &meta);
    if (rc != 0) {
        kmipkit_client_batch_item_response_release(resp_item);
        kmipkit_client_batch_response_release(resp);
        return rc;
    }

    out_result->response = resp;
    out_result->item = resp_item;
    out_result->operation = meta.operation;
    out_result->outcome_kind = meta.outcome_kind;
    out_result->result_status = meta.result_status;
    out_result->has_result_reason = meta.has_result_reason ? 1u : 0u;
    out_result->result_reason = meta.result_reason;
    out_result->has_result_message = meta.has_result_message ? 1u : 0u;
    out_result->result_message_data = meta.result_message_data;
    out_result->result_message_length = meta.result_message_length;
    return 0;
}

static inline int32_t kmipkit_fluent_execute_item_ok(
    kmipkit_client_t *client,
    kmipkit_client_batch_item_t *item,
    kmipkit_codec_limits_t *limits_or_null,
    const kmipkit_request_options_v1_t *options_or_null,
    uint8_t has_async_indicator,
    uint32_t async_indicator,
    kmipkit_fluent_result_t *out_result
) {
    int32_t rc = kmipkit_fluent_execute_item(
        client, item, limits_or_null, options_or_null,
        has_async_indicator, async_indicator, out_result);
    if (rc != 0) return rc;
    if (out_result->result_status != 0u) {
        return KMIPKIT_ERROR_SERVER_RESULT;
    }
    return 0;
}
"""

EPILOGUE = """
#ifdef __cplusplus
}
#endif

#endif /* KMIPKIT_FLUENT_OPS_H */
"""


def render_header(spec: dict[str, Any]) -> str:
    ops = spec["operations"]
    assert len(ops) == 64, f"Expected 64 operations in spec, got {len(ops)}"
    parts = [PREAMBLE]
    parts.append(f"#define KMIPKIT_FLUENT_OPERATION_COUNT {len(ops)}u\n")
    for op in ops:
        c_name = op["c_name"]
        upper = c_name.upper()
        c_kind = op["c_kind"]
        op_name = op["name"]
        op_code = op["code"]
        min_kmip = op["min_kmip"]
        fields, body_lines = OP_SPECS[c_kind]

        all_fields = list(fields) + list(COMMON_EXEC_FIELDS)
        struct_name = f"kmipkit_fluent_{c_name}_params_t"
        macro_name = f"KMIPKIT_FLUENT_{upper}_PARAMS_INIT"

        parts.append(f"\n/* {op_name} ({op_code}, KMIP >= {min_kmip}) */")
        parts.append(f"#define KMIPKIT_OP_{upper} ((uint32_t){op_code})")
        parts.append(f"typedef struct {struct_name} {{")
        for ftype, fname in all_fields:
            sep = "" if ftype.endswith("*") else " "
            parts.append(f"    {ftype}{sep}{fname};")
        parts.append(f"}} {struct_name};")
        parts.append(f"#define {macro_name} {{ 0 }}")

        # item builder
        parts.append(
            f"static inline int32_t kmipkit_fluent_item_{c_name}("
            f"const {struct_name} *params, kmipkit_client_batch_item_t **out_item) {{"
        )
        if not fields and c_kind != "discover_versions":
            parts.append("    (void)params;")
        elif c_kind != "uid_only" and c_kind != "discover_versions":
            parts.append("    if (params == NULL) return 4;")
        for bl in body_lines:
            parts.append(bl.replace("{c_name}", c_name))
        parts.append("}")

        # _outcome executor
        parts.append(
            f"static inline int32_t kmipkit_fluent_{c_name}_outcome("
            f"kmipkit_client_t *client, const {struct_name} *params, kmipkit_fluent_result_t *out_result) {{"
        )
        parts.append("    kmipkit_client_batch_item_t *item = NULL;")
        parts.append(f"    int32_t rc = kmipkit_fluent_item_{c_name}(params, &item);")
        parts.append("    if (rc != 0) return rc;")
        parts.append("    return kmipkit_fluent_execute_item(")
        parts.append("        client, item,")
        parts.append("        params ? params->limits : NULL,")
        parts.append("        params ? params->options : NULL,")
        parts.append("        params ? params->has_asynchronous_indicator : 0u,")
        parts.append("        params ? params->asynchronous_indicator : 0u,")
        parts.append("        out_result);")
        parts.append("}")

        # strict _ok default executor
        parts.append(
            f"static inline int32_t kmipkit_fluent_{c_name}("
            f"kmipkit_client_t *client, const {struct_name} *params, kmipkit_fluent_result_t *out_result) {{"
        )
        parts.append("    kmipkit_client_batch_item_t *item = NULL;")
        parts.append(f"    int32_t rc = kmipkit_fluent_item_{c_name}(params, &item);")
        parts.append("    if (rc != 0) return rc;")
        parts.append("    return kmipkit_fluent_execute_item_ok(")
        parts.append("        client, item,")
        parts.append("        params ? params->limits : NULL,")
        parts.append("        params ? params->options : NULL,")
        parts.append("        params ? params->has_asynchronous_indicator : 0u,")
        parts.append("        params ? params->asynchronous_indicator : 0u,")
        parts.append("        out_result);")
        parts.append("}")

    parts.append(EPILOGUE)
    return "\n".join(parts)


def verify_cross_language_parity(spec: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    ops = spec["operations"]
    rust_src = (REPO_ROOT / "crates" / "kmipkit-client" / "src" / "fluent" / "mod.rs").read_text()
    py_src = (REPO_ROOT / "bindings" / "python" / "src" / "kmipkit" / "fluent.py").read_text()
    java_src = (
        REPO_ROOT / "bindings" / "java" / "src" / "main" / "java" / "org" / "kmipkit" / "fluent" / "FluentRequest.java"
    ).read_text()
    c_src = OUT_HEADER_PATH.read_text() if OUT_HEADER_PATH.exists() else ""

    for op in ops:
        r_m = op["rust_method"]
        p_m = op["python_method"]
        j_m = op["java_method"]
        c_m = op["c_name"]
        if not re.search(rf"\bpub fn {re.escape(r_m)}\b", rust_src):
            errors.append(f"Rust RequestBuilder missing method: {r_m} ({op['name']})")
        if not re.search(rf"\bdef {re.escape(p_m)}\b", py_src):
            errors.append(f"Python Request missing method: {p_m} ({op['name']})")
        if not re.search(rf"\bpublic [A-Za-z0-9_]+Builder {re.escape(j_m)}\b", java_src):
            errors.append(f"Java FluentRequest missing method: {j_m} ({op['name']})")
        if f"kmipkit_fluent_{c_m}(" not in c_src:
            errors.append(f"C header missing fluent function: kmipkit_fluent_{c_m} ({op['name']})")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="Verify generated header is up to date.")
    parser.add_argument(
        "--verify-parity",
        action="store_true",
        help="Verify all 64 KMIP operations exist across Rust, Python, Java, and C.",
    )
    args = parser.parse_args()

    spec = json.loads(SPEC_PATH.read_text())
    rendered = render_header(spec)

    if args.check:
        if not OUT_HEADER_PATH.exists() or OUT_HEADER_PATH.read_text() != rendered:
            print(f"STALE: {OUT_HEADER_PATH} does not match {SPEC_PATH}", file=sys.stderr)
            return 1
        print(f"OK: {OUT_HEADER_PATH} matches {SPEC_PATH} ({len(spec['operations'])} ops)")
    elif not args.verify_parity:
        OUT_HEADER_PATH.write_text(rendered)
        print(f"Generated {OUT_HEADER_PATH} ({len(spec['operations'])} ops)")

    if args.verify_parity:
        errs = verify_cross_language_parity(spec)
        if errs:
            for e in errs:
                print(f"PARITY ERROR: {e}", file=sys.stderr)
            return 1
        print(f"PARITY OK: 64/64 operations present in Rust, Python, Java, and C")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
