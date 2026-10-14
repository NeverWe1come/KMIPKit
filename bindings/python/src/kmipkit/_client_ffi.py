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

    typedef struct kmipkit_client_configuration_t kmipkit_client_configuration_t;
    typedef struct kmipkit_transport_config_t kmipkit_transport_config_t;
    typedef struct kmipkit_client_t kmipkit_client_t;
    typedef struct kmipkit_client_batch_t kmipkit_client_batch_t;
    typedef struct kmipkit_client_batch_response_t kmipkit_client_batch_response_t;
    typedef struct kmipkit_client_batch_item_response_t kmipkit_client_batch_item_response_t;
    typedef struct kmipkit_client_batch_item_t kmipkit_client_batch_item_t;
    typedef struct kmipkit_register_request_t kmipkit_register_request_t;
    typedef struct kmipkit_ttlv_structure_t kmipkit_ttlv_structure_t;
    typedef struct kmipkit_ttlv_item_t kmipkit_ttlv_item_t;
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
    int32_t kmipkit_client_batch_item_response_unique_batch_item_id(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_byte_span_v1_t *out_value);
    int32_t kmipkit_client_batch_item_response_pending_correlation_value(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_byte_span_v1_t *out_value);
    int32_t kmipkit_client_batch_item_discover_versions(
        kmipkit_client_batch_item_t **out_item);
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
