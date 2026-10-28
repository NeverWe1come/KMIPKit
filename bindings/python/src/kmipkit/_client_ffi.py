"""Private CFFI declarations for the T076 client ABI adapter.

The declarations stay here until the shared manifest can promote T076's
blocked exports after both Java and Python mappings are complete. They mirror
the reviewed versioned C records and do not add a public C header surface.
"""

from __future__ import annotations

import os

from cffi import FFI

from ._ffi import _bundled_path


ffi = FFI()
ffi.cdef(
    """
    typedef unsigned char uint8_t;
    typedef unsigned int uint32_t;
    typedef int int32_t;
    typedef unsigned long long uint64_t;
    typedef long long int64_t;
    typedef unsigned short uint16_t;
    typedef unsigned long long uintptr_t;

    typedef struct kmipkit_client_configuration_t kmipkit_client_configuration_t;
    typedef struct kmipkit_transport_config_t kmipkit_transport_config_t;
    typedef struct kmipkit_client_t kmipkit_client_t;
    typedef struct kmipkit_client_batch_t kmipkit_client_batch_t;
    typedef struct kmipkit_client_batch_response_t kmipkit_client_batch_response_t;
    typedef struct kmipkit_client_batch_item_response_t kmipkit_client_batch_item_response_t;
    typedef struct kmipkit_client_batch_item_t kmipkit_client_batch_item_t;
    typedef struct kmipkit_usage_allocation_context_t kmipkit_usage_allocation_context_t;
    typedef struct kmipkit_split_key_reference_t kmipkit_split_key_reference_t;
    typedef struct kmipkit_register_request_t kmipkit_register_request_t;
    typedef struct kmipkit_ttlv_structure_t kmipkit_ttlv_structure_t;
    typedef struct kmipkit_ttlv_item_t kmipkit_ttlv_item_t;
    typedef struct kmipkit_ttlv_value_t kmipkit_ttlv_value_t;
    typedef struct kmipkit_codec_limits_t kmipkit_codec_limits_t;

    typedef struct {
        const uint8_t *data;
        uint64_t length;
    } kmipkit_byte_span_v1_t;

    typedef struct {
        uint32_t source_kind;
        uint32_t encoding;
        kmipkit_byte_span_v1_t data;
    } kmipkit_transport_input_v1_t;

    typedef struct {
        uint32_t kind;
        uint32_t reserved;
        uint64_t seconds;
        uint32_t nanoseconds;
        uint32_t reserved2;
    } kmipkit_timeout_limit_v1_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        kmipkit_timeout_limit_v1_t connect_timeout;
        kmipkit_timeout_limit_v1_t write_timeout;
        kmipkit_timeout_limit_v1_t read_timeout;
        kmipkit_timeout_limit_v1_t total_timeout;
    } kmipkit_request_options_v1_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        uint32_t endpoint_kind;
        uint32_t trust_kind;
        uint32_t flags;
        uint32_t reserved;
        kmipkit_byte_span_v1_t endpoint;
        uint16_t endpoint_port;
        uint16_t reserved2;
        const kmipkit_transport_input_v1_t *client_certificates;
        uint64_t client_certificate_count;
        kmipkit_transport_input_v1_t client_private_key;
        const kmipkit_transport_input_v1_t *trust_certificates;
        uint64_t trust_certificate_count;
        const kmipkit_transport_input_v1_t *revocation_lists;
        uint64_t revocation_list_count;
        kmipkit_byte_span_v1_t tls_server_name;
        kmipkit_byte_span_v1_t target_uri;
        kmipkit_timeout_limit_v1_t connect_timeout;
        kmipkit_timeout_limit_v1_t write_timeout;
        kmipkit_timeout_limit_v1_t read_timeout;
        kmipkit_timeout_limit_v1_t total_timeout;
        uint64_t max_request_bytes;
    } kmipkit_transport_config_v1_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        uint32_t outcome_kind;
        uint32_t result_status;
        uint32_t has_result_reason;
        uint32_t result_reason;
    } kmipkit_client_batch_item_metadata_v1_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        uint32_t operation;
        uint32_t outcome_kind;
        uint32_t result_status;
        uint32_t has_result_reason;
        uint32_t result_reason;
        uint32_t has_result_message;
        uint32_t reserved;
        const uint8_t *result_message_data;
        uint64_t result_message_length;
    } kmipkit_client_batch_item_metadata_v2_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        uint32_t kind;
        uint32_t reserved;
        const uint8_t *text_data;
        uint64_t text_length;
        uint32_t enumeration_value;
        int32_t integer_value;
    } kmipkit_unique_identifier_v1_t;

    typedef struct {
        uint32_t struct_size;
        uint32_t abi_version;
        uint32_t kind;
        uint32_t reserved;
        uint32_t tag_value;
        uint32_t reserved2;
        const uint8_t *vendor_identification_data;
        uint64_t vendor_identification_length;
        const uint8_t *attribute_name_data;
        uint64_t attribute_name_length;
    } kmipkit_attribute_reference_v1_t;

    typedef struct {
        int32_t major;
        int32_t minor;
    } kmipkit_protocol_version_v1_t;

    int32_t kmipkit_transport_config_create_v1(
        const kmipkit_transport_config_v1_t *configuration,
        kmipkit_transport_config_t **out_configuration);
    void kmipkit_transport_config_release(kmipkit_transport_config_t *handle);
    int32_t kmipkit_client_create(
        kmipkit_client_configuration_t *configuration,
        kmipkit_transport_config_t **transport_configuration,
        kmipkit_client_t **out_client);
    void kmipkit_client_release(kmipkit_client_t *handle);

    int32_t kmipkit_client_batch_create(kmipkit_client_batch_t **out_batch);
    void kmipkit_client_batch_release(kmipkit_client_batch_t *handle);
    int32_t kmipkit_client_batch_add_item(
        kmipkit_client_batch_t *batch,
        kmipkit_client_batch_item_t **inout_item);
    int32_t kmipkit_client_batch_set_request_time_stamp(
        kmipkit_client_batch_t *batch, int64_t time_stamp);
    int32_t kmipkit_client_batch_set_order_option(
        kmipkit_client_batch_t *batch, uint8_t enabled);
    int32_t kmipkit_client_batch_add_error_continuation_value(
        kmipkit_client_batch_t *batch, uint32_t value);
    int32_t kmipkit_client_batch_set_asynchronous_indicator(
        kmipkit_client_batch_t *batch, uint32_t indicator);
    int32_t kmipkit_client_batch_set_client_correlation_value(
        kmipkit_client_batch_t *batch, const uint8_t *value, uint64_t value_length);
    int32_t kmipkit_client_execute(
        kmipkit_client_t *client,
        kmipkit_client_batch_t **batch,
        kmipkit_codec_limits_t *limits,
        kmipkit_client_batch_response_t **out_response);
    int32_t kmipkit_client_execute_with_options_v1(
        kmipkit_client_t *client,
        kmipkit_client_batch_t **batch,
        kmipkit_codec_limits_t *limits,
        const kmipkit_request_options_v1_t *options,
        kmipkit_client_batch_response_t **out_response);
    int32_t kmipkit_client_execute_with_error_ttlv_v1(
        kmipkit_client_t *client,
        kmipkit_client_batch_t **batch,
        kmipkit_codec_limits_t *limits,
        kmipkit_client_batch_response_t **out_response,
        kmipkit_ttlv_structure_t **out_error_response_ttlv);
    int32_t kmipkit_client_execute_with_options_and_error_ttlv_v1(
        kmipkit_client_t *client,
        kmipkit_client_batch_t **batch,
        kmipkit_codec_limits_t *limits,
        const kmipkit_request_options_v1_t *options,
        kmipkit_client_batch_response_t **out_response,
        kmipkit_ttlv_structure_t **out_error_response_ttlv);

    int32_t kmipkit_usage_allocation_context_create(
        kmipkit_client_batch_item_response_t *item,
        int64_t requested_units,
        uint32_t usage_limits_unit,
        kmipkit_usage_allocation_context_t **out_context);
    void kmipkit_usage_allocation_context_release(
        kmipkit_usage_allocation_context_t *handle);
    int32_t kmipkit_usage_allocation_context_usage_limits_unit(
        kmipkit_usage_allocation_context_t *context, uint32_t *out_unit);
    int32_t kmipkit_usage_allocation_context_allocated_units(
        kmipkit_usage_allocation_context_t *context, int64_t *out_units);
    int32_t kmipkit_usage_allocation_context_available_units(
        kmipkit_usage_allocation_context_t *context, int64_t *out_units);
    int32_t kmipkit_usage_allocation_context_reserved_units(
        kmipkit_usage_allocation_context_t *context, int64_t *out_units);
    int32_t kmipkit_usage_allocation_context_unique_identifier(
        kmipkit_usage_allocation_context_t *context,
        kmipkit_unique_identifier_v1_t *out_identifier);
    int32_t kmipkit_client_execute_guarded_protection_item(
        kmipkit_client_t *client,
        kmipkit_usage_allocation_context_t *context,
        kmipkit_client_batch_item_t **item,
        int64_t cost,
        kmipkit_codec_limits_t *limits,
        kmipkit_client_batch_response_t **out_response);
    int32_t kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
        kmipkit_client_t *client,
        kmipkit_usage_allocation_context_t *context,
        kmipkit_client_batch_item_t **item,
        int64_t cost,
        uint32_t asynchronous_indicator,
        kmipkit_codec_limits_t *limits,
        kmipkit_client_batch_response_t **out_response);
    void kmipkit_ttlv_structure_release(kmipkit_ttlv_structure_t *handle);
    void kmipkit_client_batch_response_release(kmipkit_client_batch_response_t *handle);
    int32_t kmipkit_client_batch_response_item_count(
        kmipkit_client_batch_response_t *response, uint64_t *out_count);
    int32_t kmipkit_client_batch_response_item_at(
        kmipkit_client_batch_response_t *response, uint64_t index,
        kmipkit_client_batch_item_response_t **out_item);
    void kmipkit_client_batch_item_response_release(
        kmipkit_client_batch_item_response_t *handle);
    int32_t kmipkit_client_batch_item_response_metadata_v1(
        kmipkit_client_batch_item_response_t *item,
        kmipkit_client_batch_item_metadata_v1_t *out_metadata);
    int32_t kmipkit_client_batch_item_response_metadata_v2(
        kmipkit_client_batch_item_response_t *item,
        kmipkit_client_batch_item_metadata_v2_t *out_metadata);
    int32_t kmipkit_client_batch_item_response_unique_batch_item_id(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_byte_span_v1_t *out_value);
    int32_t kmipkit_client_batch_item_response_pending_correlation_value(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_byte_span_v1_t *out_value);
    int32_t kmipkit_client_batch_item_response_discover_versions_supported_versions(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_protocol_version_v1_t *out_versions,
        uint64_t capacity,
        uint64_t *out_count);
    int32_t kmipkit_client_batch_item_response_create(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint32_t *out_object_type,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_certify(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_recertify(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_recertify_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_reprovision(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_identifier_present,
        kmipkit_unique_identifier_v1_t *out_identifier);
    int32_t kmipkit_client_batch_item_response_reprovision_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_register(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_register_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_rng_retrieve(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_data_present,
        kmipkit_byte_span_v1_t *out_data);
    int32_t kmipkit_client_batch_item_response_rng_retrieve_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_rng_seed(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_data_length_present,
        int32_t *out_data_length);
    int32_t kmipkit_client_batch_item_response_rng_seed_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_rekey(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_rekey_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_rekey_key_pair(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_private_key_unique_identifier,
        kmipkit_unique_identifier_v1_t *out_public_key_unique_identifier);
    int32_t kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_check(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_unique_identifier_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_usage_limits_count_present,
        int64_t *out_usage_limits_count,
        uint8_t *out_cryptographic_usage_mask_present,
        int32_t *out_cryptographic_usage_mask,
        uint8_t *out_lease_time_present,
        uint32_t *out_lease_time);
    int32_t kmipkit_client_batch_item_response_check_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_delegated_login(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_ticket_present,
        uint32_t *out_ticket_type,
        kmipkit_byte_span_v1_t *out_ticket_value);
    int32_t kmipkit_client_batch_item_response_delegated_login_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_ticket_ttlv);
    int32_t kmipkit_client_batch_item_response_login(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_ticket_present,
        uint32_t *out_ticket_type,
        kmipkit_byte_span_v1_t *out_ticket_value);
    int32_t kmipkit_client_batch_item_response_login_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_ticket_ttlv);
    int32_t kmipkit_client_batch_item_response_get_usage_allocation(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_activate(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_archive(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_destroy(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_recover(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_obtain_lease(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint32_t *out_lease_time,
        int64_t *out_last_change_date);
    int32_t kmipkit_client_batch_item_response_obtain_lease_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_import(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_import_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_join_split_key(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_join_split_key_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_derive_key(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_derive_key_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_export(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_payload_present,
        uint32_t *out_object_type,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        kmipkit_ttlv_structure_t **out_attributes,
        kmipkit_ttlv_structure_t **out_any_object);
    int32_t kmipkit_client_batch_item_response_export_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_get(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_payload_present,
        uint32_t *out_object_type,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        kmipkit_ttlv_structure_t **out_any_object);
    int32_t kmipkit_client_batch_item_response_locate(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_payload_present,
        uint8_t *out_located_items_present,
        int32_t *out_located_items,
        uint64_t *out_unique_identifier_count);
    int32_t kmipkit_client_batch_item_response_locate_unique_identifier_at(
        kmipkit_client_batch_item_response_t *item,
        uint64_t index,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_create_key_pair(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_private_identifier,
        kmipkit_unique_identifier_v1_t *out_public_identifier);
    int32_t kmipkit_client_batch_item_response_create_split_key_count(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint64_t *out_count);
    int32_t kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
        kmipkit_client_batch_item_response_t *item,
        uint64_t index,
        kmipkit_unique_identifier_v1_t *out_identifier);
    int32_t kmipkit_client_batch_item_response_hash(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint8_t *out_data_present,
        kmipkit_byte_span_v1_t *out_data,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_response_encrypt(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_data_present,
        kmipkit_byte_span_v1_t *out_data,
        uint8_t *out_iv_counter_nonce_present,
        kmipkit_byte_span_v1_t *out_iv_counter_nonce,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value,
        uint8_t *out_authenticated_encryption_tag_present,
        kmipkit_byte_span_v1_t *out_authenticated_encryption_tag);
    int32_t kmipkit_client_batch_item_response_decrypt(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_data_present,
        kmipkit_byte_span_v1_t *out_data,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_response_mac(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_mac_data_present,
        kmipkit_byte_span_v1_t *out_mac_data,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_response_mac_verify(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_validity_present,
        uint32_t *out_validity_indicator,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_response_sign(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_signature_data_present,
        kmipkit_byte_span_v1_t *out_signature_data,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_response_signature_verify(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        uint8_t *out_validity_present,
        uint32_t *out_validity_indicator,
        uint8_t *out_recovered_data_present,
        kmipkit_byte_span_v1_t *out_recovered_data,
        uint8_t *out_correlation_present,
        kmipkit_byte_span_v1_t *out_correlation_value);
    int32_t kmipkit_client_batch_item_create(
        uint32_t object_type,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_create_key_pair(
        kmipkit_ttlv_structure_t *common_attributes,
        kmipkit_ttlv_structure_t *private_key_attributes,
        kmipkit_ttlv_structure_t *public_key_attributes,
        kmipkit_ttlv_structure_t *common_protection_storage_masks,
        kmipkit_ttlv_structure_t *private_protection_storage_masks,
        kmipkit_ttlv_structure_t *public_protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_create_split_key(
        uint32_t object_type,
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        int32_t split_key_parts,
        int32_t split_key_threshold,
        uint32_t split_key_method,
        uint8_t has_prime_field_size,
        const uint8_t *prime_field_size_data,
        uint64_t prime_field_size_length,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_hash(
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_encrypt(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *iv_counter_nonce,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        const kmipkit_byte_span_v1_t *authenticated_encryption_additional_data,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_decrypt(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *iv_counter_nonce,
        const kmipkit_byte_span_v1_t *authenticated_encryption_tag,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        const kmipkit_byte_span_v1_t *authenticated_encryption_additional_data,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_certify(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint8_t has_certificate_request_type,
        uint32_t certificate_request_type,
        const kmipkit_byte_span_v1_t *certificate_request_value,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_recertify(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        const kmipkit_unique_identifier_v1_t *certificate_request_unique_identifier,
        uint8_t has_certificate_request_type,
        uint32_t certificate_request_type,
        const kmipkit_byte_span_v1_t *certificate_request_value,
        uint8_t has_offset,
        uint32_t offset,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_reprovision(
        kmipkit_ttlv_structure_t *certificate_request,
        kmipkit_ttlv_structure_t *certificate,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_rng_retrieve(
        int32_t data_length,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_rng_seed(
        const kmipkit_byte_span_v1_t *data,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_rekey(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint8_t has_offset,
        uint32_t offset,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_rekey_key_pair(
        const kmipkit_unique_identifier_v1_t *private_key_unique_identifier,
        uint8_t has_offset,
        uint32_t offset,
        kmipkit_ttlv_structure_t *common_attributes,
        kmipkit_ttlv_structure_t *private_key_attributes,
        kmipkit_ttlv_structure_t *public_key_attributes,
        kmipkit_ttlv_structure_t *common_protection_storage_masks,
        kmipkit_ttlv_structure_t *private_protection_storage_masks,
        kmipkit_ttlv_structure_t *public_protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_check(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint8_t has_usage_limits_count,
        int64_t usage_limits_count,
        uint8_t has_cryptographic_usage_mask,
        int32_t cryptographic_usage_mask,
        uint8_t has_lease_time,
        uint32_t lease_time,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_delegated_login(
        kmipkit_ttlv_structure_t *rights,
        uint32_t lease_time_kind,
        int64_t lease_time_value,
        uint8_t has_request_count,
        int32_t request_count,
        kmipkit_ttlv_structure_t *usage_limits,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_login(
        uint32_t lease_time_kind,
        int64_t lease_time_value,
        uint8_t has_request_count,
        int32_t request_count,
        kmipkit_ttlv_structure_t *usage_limits,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_derive_key(
        uint32_t object_type,
        const kmipkit_unique_identifier_v1_t *unique_identifiers,
        uint64_t unique_identifier_count,
        uint32_t derivation_method,
        kmipkit_ttlv_structure_t *derivation_parameters,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_export(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint8_t has_key_format_type,
        uint32_t key_format_type,
        uint8_t has_key_wrap_type,
        uint32_t key_wrap_type,
        uint8_t has_key_compression_type,
        uint32_t key_compression_type,
        kmipkit_ttlv_structure_t *key_wrapping_specification,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_get(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint8_t has_key_format_type,
        uint32_t key_format_type,
        uint8_t has_key_wrap_type,
        uint32_t key_wrap_type,
        uint8_t has_key_compression_type,
        uint32_t key_compression_type,
        kmipkit_ttlv_structure_t *key_wrapping_specification,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_get_attribute_list(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_add_attribute(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_item_t *new_attribute,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_adjust_attribute(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        const kmipkit_attribute_reference_v1_t *attribute_reference,
        uint32_t adjustment_type,
        kmipkit_ttlv_value_t *adjustment_value,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_set_constraints(
        kmipkit_ttlv_structure_t *constraints,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_set_defaults(
        kmipkit_ttlv_structure_t *defaults_information,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_validate(
        kmipkit_ttlv_structure_t **certificates,
        uint64_t certificate_count,
        const kmipkit_unique_identifier_v1_t *identifiers,
        uint64_t identifier_count,
        kmipkit_ttlv_item_t **payload_extensions,
        uint64_t payload_extension_count,
        uint8_t has_validity_date,
        int64_t validity_date,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_response_set_constraints_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_set_defaults_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_get_attribute_list(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier,
        kmipkit_attribute_reference_v1_t *out_references,
        uint64_t capacity,
        uint64_t *out_count);
    int32_t kmipkit_client_batch_item_response_add_attribute(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_adjust_attribute(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_validate(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        uint32_t *out_validity_indicator);
    int32_t kmipkit_client_batch_item_response_validate_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_locate(
        uint8_t has_maximum_items,
        int32_t maximum_items,
        uint8_t has_offset_items,
        int32_t offset_items,
        uint8_t has_storage_status_mask,
        uint32_t storage_status_mask,
        uint8_t has_object_group_member,
        uint32_t object_group_member,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_get_usage_allocation(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        int64_t usage_limits_count,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_activate(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_archive(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_destroy(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_recover(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_obtain_lease(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_import(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint32_t object_type,
        uint8_t has_replace_existing,
        uint8_t replace_existing,
        uint8_t has_key_wrap_type,
        uint32_t key_wrap_type,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *any_object,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_split_key_reference_create(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        uint32_t split_key_threshold,
        kmipkit_split_key_reference_t **out_reference);
    void kmipkit_split_key_reference_release(kmipkit_split_key_reference_t *handle);
    int32_t kmipkit_client_batch_item_join_split_key(
        uint32_t object_type,
        kmipkit_split_key_reference_t **references,
        uint64_t reference_count,
        uint8_t has_secret_data_type,
        uint32_t secret_data_type,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_structure_t *protection_storage_masks,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_mac(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_mac_verify(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *mac_data,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_sign(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *digested_data,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_signature_verify(
        const kmipkit_unique_identifier_v1_t *unique_identifier,
        kmipkit_ttlv_structure_t *parameters,
        kmipkit_ttlv_value_t *data,
        const kmipkit_byte_span_v1_t *digested_data,
        const kmipkit_byte_span_v1_t *signature_data,
        const kmipkit_byte_span_v1_t *correlation_value,
        uint8_t init_indicator,
        uint8_t final_indicator,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_discover_versions(
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_ping(
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_query(
        const uint32_t *functions,
        uint64_t function_count,
        uint8_t has_object_groups,
        const kmipkit_byte_span_v1_t *object_groups,
        uint64_t object_group_count,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_query_with_limits(
        kmipkit_codec_limits_t *limits,
        const uint32_t *functions,
        uint64_t function_count,
        uint8_t has_object_groups,
        const kmipkit_byte_span_v1_t *object_groups,
        uint64_t object_group_count,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_interop(
        uint32_t interop_function,
        const uint8_t *interop_identifier_data,
        uint64_t interop_identifier_length,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_pkcs11(
        kmipkit_ttlv_structure_t *function,
        kmipkit_ttlv_structure_t *interface_structure,
        const kmipkit_byte_span_v1_t *correlation_value,
        kmipkit_ttlv_structure_t *input_parameters,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_log(
        const uint8_t *log_message_data,
        uint64_t log_message_length,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_response_interop_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_pkcs11_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_interface);
    int32_t kmipkit_client_batch_item_response_pkcs11_function_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_function);
    int32_t kmipkit_client_batch_item_response_pkcs11_correlation_value(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_byte_span_v1_t *out_value);
    int32_t kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_output_parameters);
    int32_t kmipkit_client_batch_item_response_pkcs11_return_code_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_return_code);
    int32_t kmipkit_client_batch_item_response_log_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_ping(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present);
    int32_t kmipkit_client_batch_item_response_logout(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present);
    int32_t kmipkit_client_batch_item_response_query_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_client_batch_item_response_revoke(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_unique_identifier_v1_t *out_unique_identifier);
    int32_t kmipkit_client_batch_item_response_revoke_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    int32_t kmipkit_register_request_create(
        uint32_t object_type,
        kmipkit_ttlv_structure_t *attributes,
        kmipkit_ttlv_item_t *object,
        kmipkit_register_request_t **out_request);
    int32_t kmipkit_register_request_with_opaque_object_context(
        kmipkit_register_request_t *request,
        uint32_t context,
        kmipkit_register_request_t **out_request);
    int32_t kmipkit_client_batch_item_register(
        kmipkit_register_request_t *request,
        kmipkit_client_batch_item_t **out_item);
    void kmipkit_register_request_release(kmipkit_register_request_t *handle);
    int32_t kmipkit_client_batch_item_with_unique_batch_item_id(
        kmipkit_client_batch_item_t *item, const uint8_t *id_data,
        uint64_t id_length, kmipkit_client_batch_item_t **out_item);
    void kmipkit_client_batch_item_release(kmipkit_client_batch_item_t *handle);
    int32_t kmipkit_last_error_get(
        int32_t *out_status,
        uint32_t *out_category,
        uint32_t *out_cause,
        uint32_t *out_delivery);
    int32_t kmipkit_last_error_clear(void);
    """
)

_native_path = os.environ.get("KMIPKIT_NATIVE_LIBRARY", str(_bundled_path))
try:
    lib = ffi.dlopen(_native_path)
except OSError:
    raise ImportError("KMIPKit native client ABI is unavailable") from None


__all__ = ["ffi", "lib"]
