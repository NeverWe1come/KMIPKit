#include "kmipkit.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void initialize_identifier_v1(kmipkit_unique_identifier_v1_t *identifier) {
    identifier->struct_size = (uint32_t)sizeof(*identifier);
    identifier->abi_version = UINT32_C(1);
}

static int32_t execute_key_lifecycle(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t key_name[] = "fluent-key";
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_client_batch_response_t *create_response = 0;
    kmipkit_client_batch_response_t *get_response = 0;
    kmipkit_client_batch_response_t *destroy_response = 0;
    kmipkit_client_batch_item_response_t *create_item = 0;
    kmipkit_client_batch_item_response_t *get_item = 0;
    kmipkit_client_batch_item_response_t *destroy_item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t created_identifier = {0};
    kmipkit_ttlv_structure_t *get_object = 0;
    kmipkit_client_batch_item_metadata_v1_t metadata = {0};
    uint8_t present = 0;
    uint8_t payload_present = 0;
    uint32_t object_type = 0;
    int32_t status = 1;
    int32_t completed_operations = 0;
    int32_t failure_step = 0;

    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask = (int32_t)(
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    request.name_present = UINT8_C(1);
    request.name.data = key_name;
    request.name.length = (uint64_t)(sizeof(key_name) - 1);

    failure_step = 1;
    status = kmipkit_client_create_symmetric_key_v1(client, &request, limits, &create_response);
    if (status != 0 || create_response == 0) {
        goto cleanup;
    }
    failure_step = 2;
    status = kmipkit_client_batch_response_item_at(create_response, UINT64_C(0), &create_item);
    if (status != 0 || create_item == 0) goto cleanup;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    initialize_identifier_v1(&identifier);
    failure_step = 3;
    status = kmipkit_client_batch_item_response_metadata_v1(create_item, &metadata);
    if (status != 0 || metadata.result_status != 0) goto cleanup;
    failure_step = 4;
    status = kmipkit_client_batch_item_response_create(
        create_item, &present, &object_type, &identifier);
    if (status != 0 ||
        !present || object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_data == 0 ||
        identifier.text_length != (uint64_t)(sizeof(key_name) - 1) ||
        memcmp(identifier.text_data, key_name, sizeof(key_name) - 1) != 0) {
        goto cleanup;
    }
    created_identifier = identifier;
    completed_operations = 1;

    failure_step = 5;
    status = kmipkit_client_get_by_id_v1(client, &identifier, limits, &get_response);
    if (status != 0 || get_response == 0) goto cleanup;
    failure_step = 6;
    status = kmipkit_client_batch_response_item_at(get_response, UINT64_C(0), &get_item);
    if (status != 0 || get_item == 0) {
        goto cleanup;
    }
    completed_operations = 2;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    failure_step = 7;
    status = kmipkit_client_batch_item_response_metadata_v1(get_item, &metadata);
    if (status != 0 || metadata.result_status != 0) goto cleanup;
    failure_step = 8;
    status = kmipkit_client_batch_item_response_get(
        get_item, &present, &payload_present, &object_type, &identifier, &get_object);
    if (status != 0 ||
        !present || !payload_present ||
        object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY || get_object == 0 ||
        identifier.text_data == 0 ||
        identifier.kind != created_identifier.kind ||
        identifier.text_length != created_identifier.text_length ||
        memcmp(identifier.text_data, created_identifier.text_data,
               (size_t)created_identifier.text_length) != 0) {
        goto cleanup;
    }

    failure_step = 9;
    status = kmipkit_client_destroy_by_id_v1(client, &identifier, limits, &destroy_response);
    if (status != 0 || destroy_response == 0) goto cleanup;
    failure_step = 10;
    status = kmipkit_client_batch_response_item_at(destroy_response, UINT64_C(0), &destroy_item);
    if (status != 0 || destroy_item == 0) {
        goto cleanup;
    }
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    present = UINT8_C(0);
    failure_step = 11;
    status = kmipkit_client_batch_item_response_metadata_v1(destroy_item, &metadata);
    if (status != 0 || metadata.result_status != 0) goto cleanup;
    failure_step = 12;
    status = kmipkit_client_batch_item_response_destroy(destroy_item, &present, &identifier);
    if (status != 0 ||
        !present || identifier.text_data == 0 ||
        identifier.text_length != (uint64_t)(sizeof(key_name) - 1) ||
        memcmp(identifier.text_data, key_name, sizeof(key_name) - 1) != 0) {
        goto cleanup;
    }
    status = 0;

cleanup:
    kmipkit_ttlv_structure_release(get_object);
    kmipkit_client_batch_item_response_release(destroy_item);
    kmipkit_client_batch_response_release(destroy_response);
    kmipkit_client_batch_item_response_release(get_item);
    kmipkit_client_batch_response_release(get_response);
    kmipkit_client_batch_item_response_release(create_item);
    kmipkit_client_batch_response_release(create_response);
    if (status == 0) puts("c: Create/Get/Destroy journey verified");
    else fprintf(stderr, "C lifecycle failed at step %d after %d operations (status %d)\n",
                 failure_step, completed_operations, status);
    return status;
}

static int32_t execute_aes_gcm_journey(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t key_name[] = "fluent-aes-gcm-key";
    static const uint8_t plaintext[] = "KMIPKit AES-GCM journey";
    static const uint8_t expected_ciphertext[] = {
        0x0c, 0x4f, 0x9f, 0x4b, 0x8e, 0x8c, 0xb6, 0x3b, 0xcc, 0x04, 0xc4, 0xa6,
        0xf6, 0xaa, 0x35, 0x4d, 0xe9, 0xb9, 0xf2, 0x46, 0x9e, 0x1e, 0x26};
    static const uint8_t iv[] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11};
    static const uint8_t aad[] = "kmipkit-gcm-aad";
    static const uint8_t expected_tag[] = {
        0xb5, 0x16, 0xad, 0x2a, 0x79, 0xbf, 0x55, 0xd1,
        0x47, 0x48, 0x2f, 0x13, 0xfe, 0xed, 0x6e, 0xa6};
    kmipkit_symmetric_key_request_v1_t create_request = {0};
    kmipkit_cryptographic_parameters_fields_v1_t parameter_fields = {0};
    kmipkit_cryptographic_parameters_t *parameters = NULL;
    kmipkit_crypto_operation_builder_t *builder = NULL;
    kmipkit_client_batch_response_t *create_response = NULL;
    kmipkit_client_batch_response_t *encrypt_response = NULL;
    kmipkit_client_batch_response_t *decrypt_response = NULL;
    kmipkit_client_batch_response_t *destroy_response = NULL;
    kmipkit_client_batch_item_response_t *item = NULL;
    kmipkit_client_batch_item_metadata_v1_t metadata = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t response_identifier = {0};
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_byte_span_v1_t iv_span = {0};
    kmipkit_byte_span_v1_t aad_span = {0};
    kmipkit_byte_span_v1_t tag_span = {0};
    kmipkit_byte_span_v1_t encrypted_data = {0};
    kmipkit_byte_span_v1_t response_iv = {0};
    kmipkit_byte_span_v1_t response_tag = {0};
    kmipkit_byte_span_v1_t correlation = {0};
    kmipkit_byte_span_v1_t decrypted_data = {0};
    uint8_t present = 0;
    uint8_t data_present = 0;
    uint8_t iv_present = 0;
    uint8_t correlation_present = 0;
    uint8_t tag_present = 0;
    uint32_t object_type = 0;
    int32_t status = 0;
    int32_t failure_step = 0;

    create_request.struct_size = (uint32_t)sizeof(create_request);
    create_request.abi_version = UINT32_C(1);
    create_request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    create_request.cryptographic_length = INT32_C(256);
    create_request.cryptographic_usage_mask = (int32_t)(
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    create_request.name_present = UINT8_C(1);
    create_request.name.data = key_name;
    create_request.name.length = (uint64_t)(sizeof(key_name) - 1);
    initialize_identifier_v1(&identifier);
    initialize_identifier_v1(&response_identifier);
    failure_step = 1;
    status = kmipkit_client_create_symmetric_key_v1(client, &create_request, limits, &create_response);
    if (status != 0 || create_response == NULL) goto cleanup;
    failure_step = 2;
    status = kmipkit_client_batch_response_item_at(create_response, UINT64_C(0), &item);
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    if (status != 0 || item == NULL) goto cleanup;
    status = kmipkit_client_batch_item_response_metadata_v1(item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    failure_step = 3;
    status = kmipkit_client_batch_item_response_create(item, &present, &object_type, &identifier);
    if (status != 0 || present != UINT8_C(1) ||
        object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_data == NULL || identifier.text_length != (uint64_t)(sizeof(key_name) - 1) ||
        memcmp(identifier.text_data, key_name, sizeof(key_name) - 1) != 0) {
        status = 100;
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = NULL;

    parameter_fields.struct_size = (uint32_t)sizeof(parameter_fields);
    parameter_fields.abi_version = UINT32_C(1);
    parameter_fields.cryptographic_algorithm_present = UINT8_C(1);
    parameter_fields.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    parameter_fields.block_cipher_mode_present = UINT8_C(1);
    parameter_fields.block_cipher_mode = UINT32_C(9);
    parameter_fields.iv_length_present = UINT8_C(1);
    parameter_fields.iv_length = INT32_C(96);
    parameter_fields.tag_length_present = UINT8_C(1);
    parameter_fields.tag_length = INT32_C(128);
    failure_step = 3;
    status = kmipkit_cryptographic_parameters_from_fields_v1(&parameter_fields, &parameters);
    if (status != 0 || parameters == NULL) goto cleanup;

    data.data = plaintext;
    data.length = (uint64_t)(sizeof(plaintext) - 1);
    iv_span.data = iv;
    iv_span.length = (uint64_t)sizeof(iv);
    aad_span.data = aad;
    aad_span.length = (uint64_t)(sizeof(aad) - 1);
    failure_step = 4;
    status = kmipkit_encrypt_builder_create(&builder);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, &identifier);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(builder, parameters);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_data(builder, &data);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, &iv_span);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(builder, &aad_span);
    if (status == 0) status = kmipkit_crypto_operation_builder_execute_v1(builder, client, limits, &encrypt_response);
    kmipkit_crypto_operation_builder_release(builder);
    builder = NULL;
    if (status != 0 || encrypt_response == NULL) goto cleanup;
    failure_step = 5;
    status = kmipkit_client_batch_response_item_at(encrypt_response, UINT64_C(0), &item);
    if (status != 0 || item == NULL) goto cleanup;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    status = kmipkit_client_batch_item_response_metadata_v1(item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = kmipkit_client_batch_item_response_encrypt(
        item, &present, &response_identifier, &data_present, &encrypted_data,
        &iv_present, &response_iv, &correlation_present, &correlation,
        &tag_present, &response_tag);
    if (status != 0 || present != UINT8_C(1) || data_present != UINT8_C(1) ||
        iv_present != UINT8_C(1) || correlation_present != UINT8_C(0) || tag_present != UINT8_C(1) ||
        response_identifier.kind != identifier.kind || response_identifier.text_data == NULL ||
        response_identifier.text_length != identifier.text_length ||
        memcmp(response_identifier.text_data, identifier.text_data, (size_t)identifier.text_length) != 0 ||
        encrypted_data.data == NULL || encrypted_data.length != (uint64_t)sizeof(expected_ciphertext) ||
        memcmp(encrypted_data.data, expected_ciphertext, sizeof(expected_ciphertext)) != 0 ||
        response_iv.data == NULL || response_iv.length != (uint64_t)sizeof(iv) ||
        memcmp(response_iv.data, iv, sizeof(iv)) != 0 || response_tag.data == NULL ||
        response_tag.length != (uint64_t)sizeof(expected_tag) ||
        memcmp(response_tag.data, expected_tag, sizeof(expected_tag)) != 0) {
        status = 101;
        goto cleanup;
    }

    data = encrypted_data;
    tag_span = response_tag;
    failure_step = 6;
    kmipkit_client_batch_item_response_release(item);
    item = NULL;
    status = kmipkit_decrypt_builder_create(&builder);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, &identifier);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(builder, parameters);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_data(builder, &data);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, &iv_span);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(builder, &aad_span);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_authenticated_encryption_tag(builder, &tag_span);
    if (status == 0) status = kmipkit_crypto_operation_builder_execute_v1(builder, client, limits, &decrypt_response);
    kmipkit_crypto_operation_builder_release(builder);
    builder = NULL;
    if (status != 0 || decrypt_response == NULL) goto cleanup;
    failure_step = 7;
    status = kmipkit_client_batch_response_item_at(decrypt_response, UINT64_C(0), &item);
    if (status != 0 || item == NULL) goto cleanup;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    status = kmipkit_client_batch_item_response_metadata_v1(item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = kmipkit_client_batch_item_response_decrypt(
        item, &present, &response_identifier, &data_present, &decrypted_data,
        &correlation_present, &correlation);
    if (status != 0 || present != UINT8_C(1) || data_present != UINT8_C(1) ||
        correlation_present != UINT8_C(0) || response_identifier.kind != identifier.kind ||
        response_identifier.text_data == NULL || response_identifier.text_length != identifier.text_length ||
        memcmp(response_identifier.text_data, identifier.text_data, (size_t)identifier.text_length) != 0 ||
        decrypted_data.data == NULL || decrypted_data.length != (uint64_t)(sizeof(plaintext) - 1) ||
        memcmp(decrypted_data.data, plaintext, sizeof(plaintext) - 1) != 0) {
        status = 102;
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = NULL;

    failure_step = 8;
    status = kmipkit_client_destroy_by_id_v1(client, &identifier, limits, &destroy_response);
    if (status != 0 || destroy_response == NULL) goto cleanup;
    status = kmipkit_client_batch_response_item_at(destroy_response, UINT64_C(0), &item);
    if (status != 0 || item == NULL) goto cleanup;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    status = kmipkit_client_batch_item_response_metadata_v1(item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = 0;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_client_batch_response_release(destroy_response);
    kmipkit_client_batch_response_release(decrypt_response);
    kmipkit_client_batch_response_release(encrypt_response);
    kmipkit_client_batch_response_release(create_response);
    kmipkit_crypto_operation_builder_release(builder);
    kmipkit_cryptographic_parameters_release(parameters);
    if (status == 0) puts("c: fluent AES-GCM journey verified");
    else fprintf(stderr, "C AES-GCM journey failed at step %d (status %d)\n", failure_step, status);
    return status;
}

/* KMIP 2.1 sections 4.14 and 11.45: typed domain parameters select P-256. */
static int32_t p256_common_attributes(
    kmipkit_codec_limits_t *limits, kmipkit_attribute_set_t **out_attributes) {
    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *curve = NULL;
    kmipkit_ttlv_value_t *domain_value = NULL;
    kmipkit_ttlv_item_t *curve_item = NULL;
    kmipkit_ttlv_item_t *domain_attribute = NULL;
    kmipkit_ttlv_structure_t *domain = NULL;
    kmipkit_ttlv_structure_t *empty = NULL;
    int32_t status = kmipkit_ttlv_raw_tag_create(UINT32_C(0x420075), &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(
        KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_256, &curve);
    if (status == 0) {
        status = kmipkit_ttlv_item_create(tag, curve, limits, &curve_item);
        curve = NULL; /* Consumed by item_create, including failure. */
    }
    if (status == 0) status = kmipkit_ttlv_structure_create(&domain);
    if (status == 0) {
        kmipkit_ttlv_structure_t *source = domain;
        kmipkit_ttlv_item_t *item = curve_item;
        domain = NULL;
        curve_item = NULL;
        status = kmipkit_ttlv_structure_with_item(source, item, limits, &domain);
    }
    if (status == 0) {
        status = kmipkit_ttlv_value_structure(domain, limits, &domain_value);
        domain = NULL;
    }
    if (status == 0) status = kmipkit_typed_attribute_from_value(
        UINT32_C(0x420029), domain_value, &domain_attribute);
    if (status == 0) status = kmipkit_ttlv_structure_create(&empty);
    if (status == 0) status = kmipkit_attribute_set_create(empty, out_attributes);
    if (status == 0) status = kmipkit_attribute_set_try_push_typed(
        *out_attributes, UINT32_C(0x420029), domain_attribute);
    kmipkit_ttlv_structure_release(empty);
    kmipkit_ttlv_structure_release(domain);
    kmipkit_ttlv_item_release(domain_attribute);
    kmipkit_ttlv_item_release(curve_item);
    kmipkit_ttlv_value_release(domain_value);
    kmipkit_ttlv_value_release(curve);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t execute_key_pair_signature_journey(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t message[] = "KMIPKit ECDSA signature journey";
    static const uint8_t expected_signature[] = {
        0x30, 0x45, 0x02, 0x21, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11,
        0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x02, 0x21, 0x00, 0x33, 0x44,
        0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11};
    kmipkit_create_key_pair_request_v1_t request = {0};
    kmipkit_attribute_set_t *common_attributes = NULL;
    kmipkit_cryptographic_parameters_fields_v1_t parameter_fields = {0};
    kmipkit_cryptographic_parameters_t *parameters = NULL;
    kmipkit_client_batch_response_t *create_response = NULL;
    kmipkit_client_batch_response_t *sign_response = NULL;
    kmipkit_client_batch_response_t *verify_response = NULL;
    kmipkit_client_batch_item_response_t *create_item = NULL;
    kmipkit_client_batch_item_response_t *sign_item = NULL;
    kmipkit_client_batch_item_response_t *verify_item = NULL;
    kmipkit_crypto_operation_builder_t *builder = NULL;
    kmipkit_client_batch_item_metadata_v1_t metadata = {0};
    kmipkit_unique_identifier_v1_t private_identifier = {0};
    kmipkit_unique_identifier_v1_t public_identifier = {0};
    kmipkit_unique_identifier_v1_t response_identifier = {0};
    kmipkit_byte_span_v1_t data = {message, (uint64_t)(sizeof(message) - 1)};
    kmipkit_byte_span_v1_t signature = {0};
    kmipkit_byte_span_v1_t correlation = {0};
    uint8_t pair_present = 0;
    uint8_t present = 0;
    uint8_t signature_present = 0;
    uint8_t correlation_present = 0;
    uint8_t validity_present = 0;
    uint8_t recovered_present = 0;
    uint32_t validity = 0;
    kmipkit_byte_span_v1_t recovered = {0};
    int32_t status = 0;
    int32_t failure_step = 0;

    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = UINT32_C(6);
    request.cryptographic_length = INT32_C(256);
    request.private_usage_mask_present = UINT8_C(1);
    request.public_usage_mask_present = UINT8_C(1);
    request.private_usage_mask = INT32_C(1);
    request.public_usage_mask = INT32_C(2);
    initialize_identifier_v1(&private_identifier);
    initialize_identifier_v1(&public_identifier);
    initialize_identifier_v1(&response_identifier);
    /* KMIPKIT-0093-FR-011: invalid ABI records must fail before transport. */
    request.abi_version = UINT32_C(0);
    status = kmipkit_client_create_key_pair_v1(client, &request, limits, &create_response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || create_response != NULL) return 114;
    request.abi_version = UINT32_C(1);
    request.private_usage_mask_present = UINT8_C(2);
    status = kmipkit_client_create_key_pair_v1(client, &request, limits, &create_response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || create_response != NULL) return 115;
    request.private_usage_mask_present = UINT8_C(1);
    failure_step = 1;
    status = p256_common_attributes(limits, &common_attributes);
    if (status == 0) status = kmipkit_client_create_key_pair_with_common_attributes_v1(
        client, &request, common_attributes, limits, &create_response);
    if (status != 0 || create_response == NULL) goto cleanup;
    failure_step = 2;
    status = kmipkit_client_batch_response_item_at(create_response, UINT64_C(0), &create_item);
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    if (status != 0 || create_item == NULL) goto cleanup;
    status = kmipkit_client_batch_item_response_metadata_v1(create_item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = kmipkit_client_batch_item_response_create_key_pair(
        create_item, &pair_present, &private_identifier, &public_identifier);
    if (status != 0 || pair_present != UINT8_C(1) ||
        private_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        private_identifier.text_data == NULL || public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        public_identifier.text_data == NULL) {
        status = 110;
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(create_item);
    create_item = NULL;

    parameter_fields.struct_size = (uint32_t)sizeof(parameter_fields);
    parameter_fields.abi_version = UINT32_C(1);
    parameter_fields.hashing_algorithm_present = UINT8_C(1);
    parameter_fields.hashing_algorithm = UINT32_C(6);
    parameter_fields.digital_signature_algorithm_present = UINT8_C(1);
    parameter_fields.digital_signature_algorithm = UINT32_C(14);
    failure_step = 3;
    status = kmipkit_cryptographic_parameters_from_fields_v1(&parameter_fields, &parameters);
    if (status != 0 || parameters == NULL) goto cleanup;

    failure_step = 4;
    status = kmipkit_sign_builder_create(&builder);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, &private_identifier);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(builder, parameters);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_data(builder, &data);
    if (status == 0) status = kmipkit_crypto_operation_builder_execute_v1(builder, client, limits, &sign_response);
    kmipkit_crypto_operation_builder_release(builder);
    builder = NULL;
    if (status != 0 || sign_response == NULL) goto cleanup;
    failure_step = 5;
    status = kmipkit_client_batch_response_item_at(sign_response, UINT64_C(0), &sign_item);
    if (status != 0 || sign_item == NULL) goto cleanup;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    status = kmipkit_client_batch_item_response_metadata_v1(sign_item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = kmipkit_client_batch_item_response_sign(
        sign_item, &present, &response_identifier, &signature_present, &signature,
        &correlation_present, &correlation);
    if (status != 0 || present != UINT8_C(1) || signature_present != UINT8_C(1) ||
        correlation_present != UINT8_C(0) || response_identifier.text_data == NULL ||
        response_identifier.text_length != private_identifier.text_length ||
        memcmp(response_identifier.text_data, private_identifier.text_data,
               (size_t)private_identifier.text_length) != 0 || signature.data == NULL ||
        signature.length != (uint64_t)sizeof(expected_signature) ||
        memcmp(signature.data, expected_signature, sizeof(expected_signature)) != 0) {
        status = 111;
        goto cleanup;
    }

    failure_step = 6;
    status = kmipkit_signature_verify_builder_create(&builder);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, &public_identifier);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(builder, parameters);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_data(builder, &data);
    if (status == 0) status = kmipkit_crypto_operation_builder_set_signature_data(builder, &signature);
    if (status == 0) status = kmipkit_crypto_operation_builder_execute_v1(builder, client, limits, &verify_response);
    kmipkit_crypto_operation_builder_release(builder);
    builder = NULL;
    if (status != 0 || verify_response == NULL) goto cleanup;
    failure_step = 7;
    status = kmipkit_client_batch_response_item_at(verify_response, UINT64_C(0), &verify_item);
    if (status != 0 || verify_item == NULL) goto cleanup;
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    status = kmipkit_client_batch_item_response_metadata_v1(verify_item, &metadata);
    if (status != 0 || metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) goto cleanup;
    status = kmipkit_client_batch_item_response_signature_verify(
        verify_item, &present, &response_identifier, &validity_present, &validity,
        &recovered_present, &recovered, &correlation_present, &correlation);
    if (status != 0 || present != UINT8_C(1) || validity_present != UINT8_C(1) ||
        validity != UINT32_C(1) || response_identifier.text_data == NULL ||
        response_identifier.text_length != public_identifier.text_length ||
        memcmp(response_identifier.text_data, public_identifier.text_data,
               (size_t)public_identifier.text_length) != 0) {
        status = 112;
        goto cleanup;
    }
    failure_step = 0;
    status = 0;

cleanup:
    if (status == 0 && failure_step != 0) status = 113;
    kmipkit_attribute_set_release(common_attributes);
    kmipkit_client_batch_item_response_release(verify_item);
    kmipkit_client_batch_response_release(verify_response);
    kmipkit_client_batch_item_response_release(sign_item);
    kmipkit_client_batch_response_release(sign_response);
    kmipkit_client_batch_item_response_release(create_item);
    kmipkit_client_batch_response_release(create_response);
    kmipkit_crypto_operation_builder_release(builder);
    kmipkit_cryptographic_parameters_release(parameters);
    if (status == 0) puts("c: fluent Create Key Pair/Sign/Signature Verify journey verified");
    else fprintf(stderr, "C key-pair signature journey failed at step %d (status %d)\n",
                 failure_step, status);
    return status;
}

static int32_t execute_expert_attribute_journey(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t name[] = "fluent-expert-attributes";
    static const uint8_t group[] = "fluent-expert-group";
    kmipkit_symmetric_key_builder_t *invalid_builder = NULL;
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_ttlv_value_t *wrong_value = NULL;
    kmipkit_ttlv_value_t *group_value = NULL;
    kmipkit_client_batch_response_t *created = NULL;
    kmipkit_client_batch_response_t *destroyed = NULL;
    kmipkit_client_batch_item_response_t *create_item = NULL;
    kmipkit_client_batch_item_response_t *destroy_item = NULL;
    kmipkit_client_batch_item_metadata_v1_t metadata = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint32_t object_type = 0;
    uint32_t error_category = 0;
    uint32_t error_cause = 0;
    uint32_t error_delivery = 0;
    int32_t error_status = 0;
    uint8_t present = 0;
    int32_t status = 1;

    status = kmipkit_symmetric_key_builder_create(&invalid_builder);
    if (status != 0 || invalid_builder == NULL ||
        kmipkit_symmetric_key_builder_set_algorithm(
            invalid_builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != 0 ||
        kmipkit_symmetric_key_builder_set_length(invalid_builder, INT32_C(256)) != 0 ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            invalid_builder, (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                                       KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT)) != 0 ||
        kmipkit_ttlv_value_integer(INT32_C(1), &wrong_value) != 0 ||
        kmipkit_symmetric_key_builder_add_catalogued_attribute(
            invalid_builder, KMIPKIT_STANDARD_ATTRIBUTE_OBJECT_GROUP, wrong_value) !=
            KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH ||
        kmipkit_last_error_get(&error_status, &error_category, &error_cause,
                               &error_delivery) != 0 ||
        error_status != KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH ||
        error_category != UINT32_C(1) || error_cause != UINT32_C(1) ||
        error_delivery != UINT32_C(1)) {
        status = 91;
        goto cleanup;
    }
    kmipkit_ttlv_value_release(wrong_value);
    wrong_value = NULL;
    kmipkit_symmetric_key_builder_release(invalid_builder);
    invalid_builder = NULL;

    status = kmipkit_symmetric_key_builder_create(&builder);
    if (status != 0 || builder == NULL ||
        kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != 0 ||
        kmipkit_symmetric_key_builder_set_length(builder, INT32_C(256)) != 0 ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder, (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                               KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT)) != 0 ||
        kmipkit_symmetric_key_builder_set_name(builder, name, sizeof(name) - 1) != 0 ||
        kmipkit_ttlv_value_text_string(limits, group, sizeof(group) - 1, &group_value) != 0 ||
        kmipkit_symmetric_key_builder_add_catalogued_attribute(
            builder, KMIPKIT_STANDARD_ATTRIBUTE_OBJECT_GROUP, group_value) != 0) {
        status = 92;
        goto cleanup;
    }
    kmipkit_ttlv_value_release(group_value);
    group_value = NULL;
    status = kmipkit_symmetric_key_builder_execute_v1(builder, client, limits, &created);
    if (status != 0 || created == NULL ||
        kmipkit_client_batch_response_item_at(created, 0, &create_item) != 0 ||
        create_item == NULL) {
        status = 93;
        goto cleanup;
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    initialize_identifier_v1(&identifier);
    if (kmipkit_client_batch_item_response_metadata_v1(create_item, &metadata) != 0 ||
        metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
        kmipkit_client_batch_item_response_create(
            create_item, &present, &object_type, &identifier) != 0 ||
        !present || object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_data == NULL || identifier.text_length != sizeof(name) - 1 ||
        memcmp(identifier.text_data, name, sizeof(name) - 1) != 0) {
        status = 94;
        goto cleanup;
    }

    status = kmipkit_client_destroy_by_id_v1(client, &identifier, limits, &destroyed);
    if (status != 0 || destroyed == NULL ||
        kmipkit_client_batch_response_item_at(destroyed, 0, &destroy_item) != 0 ||
        destroy_item == NULL) {
        status = 95;
        goto cleanup;
    }
    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    present = UINT8_C(0);
    if (kmipkit_client_batch_item_response_metadata_v1(destroy_item, &metadata) != 0 ||
        metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
        kmipkit_client_batch_item_response_destroy(destroy_item, &present, &identifier) != 0 ||
        !present || identifier.text_data == NULL || identifier.text_length != sizeof(name) - 1 ||
        memcmp(identifier.text_data, name, sizeof(name) - 1) != 0) {
        status = 96;
        goto cleanup;
    }
    status = 0;

cleanup:
    kmipkit_client_batch_item_response_release(destroy_item);
    kmipkit_client_batch_response_release(destroyed);
    kmipkit_client_batch_item_response_release(create_item);
    kmipkit_client_batch_response_release(created);
    kmipkit_ttlv_value_release(group_value);
    kmipkit_ttlv_value_release(wrong_value);
    kmipkit_symmetric_key_builder_release(builder);
    kmipkit_symmetric_key_builder_release(invalid_builder);
    if (status == 0) puts("c: fluent catalogued expert-attribute journey verified");
    return status;
}

static int32_t execute_batch_id_placeholder_journey(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t name[] = "fluent-batch-placeholder";
    static const uint8_t server_identifier[] = "server-created-batch-key";
    static const uint8_t create_batch_id[] = "create-placeholder-source";
    static const uint8_t get_batch_id[] = "get-placeholder-target";
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *create_item = NULL;
    kmipkit_client_batch_item_t *get_item = NULL;
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *create_result = NULL;
    kmipkit_client_batch_item_response_t *get_result = NULL;
    kmipkit_client_batch_item_t *identified_item = NULL;
    kmipkit_ttlv_structure_t *get_object = NULL;
    kmipkit_unique_identifier_v1_t placeholder = {0};
    kmipkit_unique_identifier_v1_t created_identifier = {0};
    kmipkit_unique_identifier_v1_t get_identifier = {0};
    kmipkit_client_batch_item_metadata_v1_t metadata = {0};
    kmipkit_byte_span_v1_t create_response_batch_id = {0};
    kmipkit_byte_span_v1_t get_response_batch_id = {0};
    uint64_t response_count = 0;
    uint32_t create_object_type = 0;
    uint32_t get_object_type = 0;
    uint8_t create_present = 0;
    uint8_t get_present = 0;
    uint8_t get_payload_present = 0;
    uint8_t create_batch_id_present = 0;
    uint8_t get_batch_id_present = 0;
    int32_t status = 1;

    status = kmipkit_symmetric_key_builder_create(&builder);
    if (status != 0 || builder == NULL ||
        kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != 0 ||
        kmipkit_symmetric_key_builder_set_length(builder, INT32_C(256)) != 0 ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder, (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                               KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT)) != 0 ||
        kmipkit_symmetric_key_builder_set_name(builder, name, sizeof(name) - 1) != 0 ||
        kmipkit_symmetric_key_builder_build_batch_item(builder, &create_item) != 0 ||
        create_item == NULL) {
        status = 101;
        goto cleanup;
    }

    {
        kmipkit_client_batch_item_t *untagged_item = create_item;
        create_item = NULL;
        if (kmipkit_client_batch_item_with_unique_batch_item_id(
                untagged_item, create_batch_id, sizeof(create_batch_id) - 1,
                &identified_item) != 0 || identified_item == NULL) {
            status = 102;
            goto cleanup;
        }
        create_item = identified_item;
        identified_item = NULL;
    }

    initialize_identifier_v1(&placeholder);
    placeholder.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
    placeholder.enumeration_value = KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_ID_PLACEHOLDER;
    if (kmipkit_client_batch_item_get(
            &placeholder, UINT8_C(0), 0, UINT8_C(0), 0, UINT8_C(0), 0, NULL,
            &get_item) != 0 || get_item == NULL ||
        kmipkit_client_batch_create(&batch) != 0 || batch == NULL) {
        status = 103;
        goto cleanup;
    }
    {
        kmipkit_client_batch_item_t *untagged_item = get_item;
        get_item = NULL;
        if (kmipkit_client_batch_item_with_unique_batch_item_id(
                untagged_item, get_batch_id, sizeof(get_batch_id) - 1,
                &identified_item) != 0 || identified_item == NULL) {
            status = 104;
            goto cleanup;
        }
        get_item = identified_item;
        identified_item = NULL;
    }
    if (kmipkit_client_batch_set_order_option(batch, UINT8_C(1)) != 0 ||
        kmipkit_client_batch_add_item(batch, &create_item) != 0 || create_item != NULL ||
        kmipkit_client_batch_add_item(batch, &get_item) != 0 || get_item != NULL) {
        status = 105;
        goto cleanup;
    }
    status = kmipkit_client_execute(client, &batch, limits, &response);
    if (status != 0 || response == NULL ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != UINT64_C(2) ||
        kmipkit_client_batch_response_item_at(response, UINT64_C(0), &create_result) != 0 ||
        create_result == NULL ||
        kmipkit_client_batch_response_item_at(response, UINT64_C(1), &get_result) != 0 ||
        get_result == NULL) {
        status = 106;
        goto cleanup;
    }

    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    initialize_identifier_v1(&created_identifier);
    if (kmipkit_client_batch_item_response_metadata_v1(create_result, &metadata) != 0 ||
        metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
        kmipkit_client_batch_item_response_create(
            create_result, &create_present, &create_object_type, &created_identifier) != 0 ||
        kmipkit_client_batch_item_response_unique_batch_item_id(
            create_result, &create_batch_id_present, &create_response_batch_id) != 0 ||
        !create_batch_id_present || create_response_batch_id.data == NULL ||
        create_response_batch_id.length != sizeof(create_batch_id) - 1 ||
        memcmp(create_response_batch_id.data, create_batch_id, sizeof(create_batch_id) - 1) != 0 ||
        !create_present || create_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        created_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        created_identifier.text_data == NULL ||
        created_identifier.text_length != sizeof(server_identifier) - 1 ||
        memcmp(created_identifier.text_data, server_identifier, sizeof(server_identifier) - 1) != 0) {
        status = 107;
        goto cleanup;
    }

    metadata = (kmipkit_client_batch_item_metadata_v1_t){0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    initialize_identifier_v1(&get_identifier);
    if (kmipkit_client_batch_item_response_metadata_v1(get_result, &metadata) != 0 ||
        metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
        kmipkit_client_batch_item_response_get(
            get_result, &get_present, &get_payload_present, &get_object_type,
            &get_identifier, &get_object) != 0 ||
        kmipkit_client_batch_item_response_unique_batch_item_id(
            get_result, &get_batch_id_present, &get_response_batch_id) != 0 ||
        !get_batch_id_present || get_response_batch_id.data == NULL ||
        get_response_batch_id.length != sizeof(get_batch_id) - 1 ||
        memcmp(get_response_batch_id.data, get_batch_id, sizeof(get_batch_id) - 1) != 0 ||
        !get_present || !get_payload_present || get_object_type != create_object_type ||
        get_identifier.kind != created_identifier.kind ||
        get_identifier.text_length != created_identifier.text_length ||
        get_identifier.text_data == NULL || get_object == NULL ||
        memcmp(get_identifier.text_data, created_identifier.text_data,
               (size_t)created_identifier.text_length) != 0) {
        status = 108;
        goto cleanup;
    }
    status = 0;

cleanup:
    kmipkit_ttlv_structure_release(get_object);
    kmipkit_client_batch_item_release(identified_item);
    kmipkit_client_batch_item_response_release(get_result);
    kmipkit_client_batch_item_response_release(create_result);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(get_item);
    kmipkit_client_batch_item_release(create_item);
    kmipkit_symmetric_key_builder_release(builder);
    if (status == 0) puts("c: fluent Create/Get batch ID Placeholder journey verified");
    return status;
}

int main(void) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = 0;
    kmipkit_client_configuration_t *client_configuration = 0;
    kmipkit_transport_config_t *transport_configuration = 0;
    kmipkit_client_t *client = 0;
    kmipkit_transport_input_v1_t client_certificate = {0};
    kmipkit_transport_input_v1_t private_key = {0};
    kmipkit_transport_input_v1_t server_certificate = {0};
    kmipkit_transport_config_v1_t transport_values = {0};
    int32_t status = kmipkit_codec_limits_defaults(&limits);

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    if (status == 0) status = kmipkit_extension_registry_limits_v1_default_values(&registry_limits);
    if (status == 0) {
        status = kmipkit_client_extension_registry_create_v1(0, 0, &registry_limits, &registry);
    }
    if (status == 0) status = kmipkit_client_configuration_create(registry, &client_configuration);
    if (status == 0) registry = 0;

    {
        const char *port_text = getenv("KMIPKIT_CONSUMER_TEST_PORT");
        const char *fixtures = getenv("KMIPKIT_CONSUMER_TEST_FIXTURES");
        char client_certificate_path[4096] = {0};
        char private_key_path[4096] = {0};
        char server_certificate_path[4096] = {0};
        char *end = 0;
        unsigned long port = port_text ? strtoul(port_text, &end, 10) : 0;
        if (status == 0 && (!fixtures || !port_text || end == port_text || *end != '\0' ||
                            port == 0 || port > 65535)) {
            status = 4;
        }
        if (status == 0) {
            int written = snprintf(client_certificate_path, sizeof(client_certificate_path),
                                   "%s/client-cert.pem", fixtures);
            if (written < 0 || written >= (int)sizeof(client_certificate_path)) status = 4;
            written = snprintf(private_key_path, sizeof(private_key_path),
                               "%s/client-key.pem", fixtures);
            if (written < 0 || written >= (int)sizeof(private_key_path)) status = 4;
            written = snprintf(server_certificate_path, sizeof(server_certificate_path),
                               "%s/server-cert.pem", fixtures);
            if (written < 0 || written >= (int)sizeof(server_certificate_path)) status = 4;
        }
        if (status == 0) {
            static const char endpoint[] = "127.0.0.1";
            static const char server_name[] = "localhost";
            client_certificate.source_kind = 2;
            client_certificate.encoding = 1;
            client_certificate.data.data = (const uint8_t *)client_certificate_path;
            client_certificate.data.length = (uint64_t)strlen(client_certificate_path);
            private_key.source_kind = 2;
            private_key.encoding = 1;
            private_key.data.data = (const uint8_t *)private_key_path;
            private_key.data.length = (uint64_t)strlen(private_key_path);
            server_certificate.source_kind = 2;
            server_certificate.encoding = 1;
            server_certificate.data.data = (const uint8_t *)server_certificate_path;
            server_certificate.data.length = (uint64_t)strlen(server_certificate_path);
            transport_values.struct_size = (uint32_t)sizeof(transport_values);
            transport_values.abi_version = UINT32_C(1);
            transport_values.endpoint_kind = 1;
            transport_values.trust_kind = 2;
            transport_values.flags = 1;
            transport_values.endpoint.data = (const uint8_t *)endpoint;
            transport_values.endpoint.length = (uint64_t)(sizeof(endpoint) - 1);
            transport_values.endpoint_port = (uint16_t)port;
            transport_values.client_certificates = &client_certificate;
            transport_values.client_certificate_count = 1;
            transport_values.client_private_key = private_key;
            transport_values.trust_certificates = &server_certificate;
            transport_values.trust_certificate_count = 1;
            transport_values.tls_server_name.data = (const uint8_t *)server_name;
            transport_values.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
            transport_values.max_request_bytes = UINT64_C(16) * 1024 * 1024;
            status = kmipkit_transport_config_create_v1(&transport_values, &transport_configuration);
        }
    }
    if (status == 0) status = kmipkit_client_create(client_configuration, &transport_configuration, &client);
    if (status == 0 && transport_configuration != 0) {
        kmipkit_transport_config_release(transport_configuration);
        transport_configuration = 0;
    }
    if (status == 0) status = execute_key_lifecycle(client, limits);
    if (status == 0) status = execute_aes_gcm_journey(client, limits);
    if (status == 0) status = execute_key_pair_signature_journey(client, limits);
    if (status == 0) status = execute_expert_attribute_journey(client, limits);
    if (status == 0) status = execute_batch_id_placeholder_journey(client, limits);

    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport_configuration);
    kmipkit_client_configuration_release(client_configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_codec_limits_release(limits);
    if (status != 0) fprintf(stderr, "C lifecycle consumer failed with status %d\n", status);
    return status == 0 ? 0 : 1;
}
