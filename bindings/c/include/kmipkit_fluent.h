/* kmipkit_fluent.h - Fluent API declarations for KMIPKit C bindings. */
#ifndef KMIPKIT_FLUENT_H
#define KMIPKIT_FLUENT_H

#if !defined(KMIPKIT_H) && !defined(KMIPKIT_INTERNAL_H)
#include "kmipkit.h"
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct kmipkit_ping_builder_t kmipkit_ping_builder_t;
typedef struct kmipkit_create_builder_t kmipkit_create_builder_t;
typedef struct kmipkit_create_credential_object_t kmipkit_create_credential_object_t;
typedef struct kmipkit_kmip30_operation_builder_t kmipkit_kmip30_operation_builder_t;
typedef struct kmipkit_create_key_pair_request_v1_t kmipkit_create_key_pair_request_v1_t;
struct kmipkit_create_key_pair_request_v1_t {
    uint32_t struct_size;
    uint32_t abi_version;
    uint32_t cryptographic_algorithm;
    int32_t cryptographic_length;
    uint8_t private_usage_mask_present;
    uint8_t public_usage_mask_present;
    uint8_t reserved_0;
    uint8_t reserved_1;
    int32_t private_usage_mask;
    int32_t public_usage_mask;
};

void kmipkit_ping_builder_release(kmipkit_ping_builder_t *handle);
void kmipkit_create_builder_release(kmipkit_create_builder_t *handle);
void kmipkit_create_credential_object_release(kmipkit_create_credential_object_t *handle);
void kmipkit_kmip30_operation_builder_release(kmipkit_kmip30_operation_builder_t *handle);
int32_t kmipkit_create_credential_object_certificate(kmipkit_ttlv_item_t * certificate, kmipkit_create_credential_object_t ** out_object);
int32_t kmipkit_create_credential_object_device(const kmipkit_byte_span_v1_t * serial, const kmipkit_byte_span_v1_t * identifier, const kmipkit_byte_span_v1_t * network, const kmipkit_byte_span_v1_t * machine, const kmipkit_byte_span_v1_t * media, kmipkit_create_credential_object_t ** out_object);
int32_t kmipkit_create_credential_object_hashed_password(uint8_t has_hashing_algorithm, uint32_t hashing_algorithm, const kmipkit_byte_span_v1_t * hashed_username_password, const kmipkit_byte_span_v1_t * hashed_password_username, kmipkit_create_credential_object_t ** out_object);
int32_t kmipkit_create_credential_object_one_time_password(uint32_t algorithm, uint8_t has_digest, uint32_t digest, const kmipkit_byte_span_v1_t * serial, const kmipkit_byte_span_v1_t * seed, uint8_t has_interval, uint32_t interval, uint8_t has_digits, int32_t digits, kmipkit_create_credential_object_t ** out_object);
int32_t kmipkit_create_credential_object_password(const kmipkit_byte_span_v1_t * password, const kmipkit_byte_span_v1_t * password_salt, uint8_t has_salt_algorithm, uint32_t salt_algorithm, const kmipkit_byte_span_v1_t * salted_password, uint8_t has_iteration_count, int32_t iteration_count, kmipkit_create_credential_object_t ** out_object);
int32_t kmipkit_create_credential_builder_create(kmipkit_attribute_set_t * attributes, kmipkit_create_credential_object_t * object, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_create_group_builder_create(kmipkit_attribute_set_t * attributes, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_create_user_builder_create(kmipkit_attribute_set_t * attributes, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_deactivate_builder_create(const kmipkit_unique_identifier_v1_t * identifier, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_decapsulate_builder_create(const kmipkit_unique_identifier_v1_t * identifier, const kmipkit_byte_span_v1_t * ciphertext, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_encapsulate_builder_create(const kmipkit_unique_identifier_v1_t * identifier, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_obliterate_builder_create(const kmipkit_unique_identifier_v1_t * identifier, kmipkit_kmip30_operation_builder_t ** out_builder);
int32_t kmipkit_kmip30_operation_builder_set_asynchronous_indicator(kmipkit_kmip30_operation_builder_t * builder, uint32_t indicator);
int32_t kmipkit_kmip30_operation_builder_set_attributes(kmipkit_kmip30_operation_builder_t * builder, kmipkit_attribute_set_t * attributes);
int32_t kmipkit_kmip30_operation_builder_set_cryptographic_parameters(kmipkit_kmip30_operation_builder_t * builder, kmipkit_cryptographic_parameters_t * parameters);
int32_t kmipkit_kmip30_operation_builder_set_deactivation_date(kmipkit_kmip30_operation_builder_t * builder, int64_t unix_seconds);
int32_t kmipkit_kmip30_operation_builder_set_deactivation_reason(kmipkit_kmip30_operation_builder_t * builder, kmipkit_ttlv_structure_t * reason);
int32_t kmipkit_kmip30_operation_builder_validate(kmipkit_kmip30_operation_builder_t * builder);
int32_t kmipkit_kmip30_operation_builder_build_batch_item(kmipkit_kmip30_operation_builder_t * builder, kmipkit_client_batch_item_t ** out_item);
int32_t kmipkit_kmip30_operation_builder_execute_v1(kmipkit_kmip30_operation_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_kmip30_operation_builder_execute_with_options_v1(kmipkit_kmip30_operation_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_credential_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_create_credential_object_t * object, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_credential_with_options_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_create_credential_object_t * object, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_group_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_group_with_options_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_user_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_user_with_options_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_deactivate_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_deactivate_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_decapsulate_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, const kmipkit_byte_span_v1_t * ciphertext, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_decapsulate_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, const kmipkit_byte_span_v1_t * ciphertext, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_encapsulate_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_encapsulate_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_obliterate_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_obliterate_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_batch_item_response_encapsulate_data(kmipkit_client_batch_item_response_t * item, uint8_t * out_present, kmipkit_byte_span_v1_t * out_data);
int32_t kmipkit_client_batch_item_response_kmip30_identifier(kmipkit_client_batch_item_response_t * item, uint8_t * out_present, kmipkit_unique_identifier_v1_t * out_identifier);
int32_t kmipkit_client_create_key_pair_v1(kmipkit_client_t * client, const kmipkit_create_key_pair_request_v1_t * request, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_key_pair_with_options_v1(kmipkit_client_t * client, const kmipkit_create_key_pair_request_v1_t * request, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_create_key_pair_with_common_attributes_v1(kmipkit_client_t * client, const kmipkit_create_key_pair_request_v1_t * request, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_activate_by_id_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_activate_by_id_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_archive_by_id_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_archive_by_id_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_locate_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_locate_with_options_v1(kmipkit_client_t * client, kmipkit_attribute_set_t * attributes, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_recover_by_id_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_recover_by_id_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_revoke_by_id_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_revocation_reason_t * revocation_reason, uint8_t has_compromise_occurrence_date, int64_t compromise_occurrence_date, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_revoke_by_id_with_options_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_revocation_reason_t * revocation_reason, uint8_t has_compromise_occurrence_date, int64_t compromise_occurrence_date, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_create_builder_create(kmipkit_create_builder_t ** out_builder);
int32_t kmipkit_create_builder_set_object_type(kmipkit_create_builder_t * builder, uint32_t object_type);
int32_t kmipkit_create_builder_set_attributes(kmipkit_create_builder_t * builder, kmipkit_ttlv_structure_t * attributes);
int32_t kmipkit_create_builder_set_protection_storage_masks(kmipkit_create_builder_t * builder, kmipkit_ttlv_structure_t * protection_storage_masks);
int32_t kmipkit_create_builder_set_asynchronous_indicator(kmipkit_create_builder_t * builder, uint32_t value);
int32_t kmipkit_create_builder_validate(kmipkit_create_builder_t * builder);
int32_t kmipkit_create_builder_build_batch_item(kmipkit_create_builder_t * builder, kmipkit_client_batch_item_t ** out_item);
/* Execute one Create. NULL options use client defaults. The returned item carries
 * success, rejection, or Pending and is released with kmipkit_client_batch_item_response_release. */
int32_t kmipkit_create_builder_execute_one_v1(
    kmipkit_create_builder_t *builder, kmipkit_client_t *client, kmipkit_codec_limits_t *limits,
    const kmipkit_request_options_v1_t *options, kmipkit_client_batch_item_response_t **out_result);
int32_t kmipkit_symmetric_key_builder_execute_one_v1(
    kmipkit_symmetric_key_builder_t *builder, kmipkit_client_t *client, kmipkit_codec_limits_t *limits,
    const kmipkit_request_options_v1_t *options, kmipkit_client_batch_item_response_t **out_result);
/* Borrow the complete received message, including framing fields and all items.
 * Keep the item alive throughout use of this span, and keep the bytes out of logs. */
int32_t kmipkit_client_batch_item_response_wire_bytes_v1(
    kmipkit_client_batch_item_response_t *item, kmipkit_byte_span_v1_t *out_bytes);
int32_t kmipkit_create_builder_execute_v1(kmipkit_create_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_create_builder_execute_with_options_v1(kmipkit_create_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_ping_builder_create(kmipkit_ping_builder_t ** out_builder);
int32_t kmipkit_ping_builder_set_asynchronous_indicator(kmipkit_ping_builder_t * builder, uint32_t value);
int32_t kmipkit_ping_builder_validate(kmipkit_ping_builder_t * builder);
int32_t kmipkit_ping_builder_build_batch_item(kmipkit_ping_builder_t * builder, kmipkit_client_batch_item_t ** out_item);
int32_t kmipkit_ping_builder_execute_v1(kmipkit_ping_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_ping_builder_execute_with_options_v1(kmipkit_ping_builder_t * builder, kmipkit_client_t * client, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_ping_v1(kmipkit_client_t * client, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
/* Direct single-item fluent/high-level execution helpers. */
int32_t kmipkit_client_execute_item_v1(kmipkit_client_t * client, kmipkit_client_batch_item_t ** inout_item, kmipkit_codec_limits_t * limits, const kmipkit_request_options_v1_t * options, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_discover_versions_v1(kmipkit_client_t * client, const kmipkit_protocol_version_v1_t * versions, uint64_t version_count, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_query_v1(kmipkit_client_t * client, const uint32_t * query_functions, uint64_t query_function_count, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_rng_retrieve_v1(kmipkit_client_t * client, int32_t data_length, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_rng_seed_v1(kmipkit_client_t * client, const uint8_t * data, uint64_t data_length, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);
int32_t kmipkit_client_obtain_lease_by_id_v1(kmipkit_client_t * client, const kmipkit_unique_identifier_v1_t * unique_identifier, kmipkit_codec_limits_t * limits, kmipkit_client_batch_response_t ** out_response);


#ifdef __cplusplus
}
#endif

#if defined(KMIPKIT_H) || !defined(KMIPKIT_INTERNAL_H)
#include "kmipkit_fluent_ops.h"
#include "kmipkit_presets.h"
#include "kmipkit_flows.h"
#endif

#endif /* KMIPKIT_FLUENT_H */
