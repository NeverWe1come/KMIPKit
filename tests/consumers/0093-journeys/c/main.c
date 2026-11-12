#include "kmipkit.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
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
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    response_identifier.struct_size = (uint32_t)sizeof(response_identifier);
    response_identifier.abi_version = UINT32_C(1);
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

    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport_configuration);
    kmipkit_client_configuration_release(client_configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_codec_limits_release(limits);
    if (status != 0) fprintf(stderr, "C lifecycle consumer failed with status %d\n", status);
    return status == 0 ? 0 : 1;
}
