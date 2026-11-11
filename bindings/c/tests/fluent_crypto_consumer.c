/* Compiled public-header probe for the semantic C cryptographic builders. */
#include "kmipkit.h"

#include <stddef.h>
#include <string.h>

typedef int32_t (*crypto_builder_create_fn)(kmipkit_crypto_operation_builder_t **);

static int exercise_operation(
    crypto_builder_create_fn create,
    uint32_t expected_operation,
    const kmipkit_unique_identifier_v1_t *identifier,
    const kmipkit_byte_span_v1_t *data,
    kmipkit_cryptographic_parameters_t *parameters,
    uint8_t special_field) {
    kmipkit_crypto_operation_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t operation = 0;
    int32_t status = create(&builder);
    if (status == INT32_C(0) && identifier != NULL) {
        status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, identifier);
    }
    if (status == INT32_C(0) && special_field != UINT8_C(3)) {
        status = kmipkit_crypto_operation_builder_set_data(builder, data);
    }
    if (status == INT32_C(0) && parameters != NULL) {
        status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(
            builder, parameters);
    }
    if (status == INT32_C(0) && special_field == UINT8_C(1)) {
        status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, data);
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(
                builder, data);
        }
    } else if (status == INT32_C(0) && special_field == UINT8_C(2)) {
        status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, data);
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_tag(
                builder, data);
        }
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(
                builder, data);
        }
    } else if (status == INT32_C(0) && special_field == UINT8_C(3)) {
        status = kmipkit_crypto_operation_builder_set_digested_data(builder, data);
    } else if (status == INT32_C(0) && special_field == UINT8_C(4)) {
        status = kmipkit_crypto_operation_builder_set_signature_data(builder, data);
    } else if (status == INT32_C(0) && special_field == UINT8_C(5)) {
        status = kmipkit_crypto_operation_builder_set_mac_data(builder, data);
    }
    if (status == INT32_C(0)) {
        status = kmipkit_crypto_operation_builder_build_batch_item(builder, &item);
    }
    if (status == INT32_C(0)) {
        status = kmipkit_client_batch_item_operation(item, &operation);
    }
    if (status == INT32_C(0) && operation != expected_operation) {
        status = INT32_C(2);
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_crypto_operation_builder_release(builder);
    return status;
}

static int create_and_release(crypto_builder_create_fn create) {
    kmipkit_crypto_operation_builder_t *builder = NULL;
    if (create(NULL) != KMIPKIT_ERROR_INVALID_INPUT ||
        create(&builder) != INT32_C(0) || builder == NULL) {
        kmipkit_crypto_operation_builder_release(builder);
        return 1;
    }
    kmipkit_crypto_operation_builder_release(builder);
    return 0;
}

int kmipkit_public_crypto_builders_probe(void) {
    if (create_and_release(kmipkit_encrypt_builder_create) != 0 ||
        create_and_release(kmipkit_decrypt_builder_create) != 0 ||
        create_and_release(kmipkit_sign_builder_create) != 0 ||
        create_and_release(kmipkit_signature_verify_builder_create) != 0 ||
        create_and_release(kmipkit_mac_builder_create) != 0 ||
        create_and_release(kmipkit_mac_verify_builder_create) != 0 ||
        create_and_release(kmipkit_hash_builder_create) != 0) {
        return 1;
    }

    static const uint8_t input[] = "caller buffer";
    static const uint8_t identifier_text[] = "c-fluent-key";
    kmipkit_byte_span_v1_t span = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_cryptographic_parameters_fields_v1_t parameter_fields = {0};
    kmipkit_cryptographic_parameters_t *parameters = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_value_t *data_value = NULL;
    kmipkit_crypto_operation_builder_t *builder = NULL;
    span.data = input;
    span.length = (uint64_t)(sizeof(input) - 1);
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    parameter_fields.struct_size = (uint32_t)sizeof(parameter_fields);
    parameter_fields.abi_version = UINT32_C(1);
    parameter_fields.hashing_algorithm_present = UINT8_C(1);
    parameter_fields.hashing_algorithm = KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_256;
    if (kmipkit_cryptographic_parameters_from_fields_v1(&parameter_fields, &parameters) !=
            INT32_C(0) ||
        kmipkit_codec_limits_create(UINT64_C(16777216), UINT64_C(64), UINT64_C(100000),
                                    &limits) != INT32_C(0)) {
        kmipkit_cryptographic_parameters_release(parameters);
        kmipkit_codec_limits_release(limits);
        return 2;
    }

    if (exercise_operation(kmipkit_encrypt_builder_create,
                           KMIPKIT_ENUMERATION_OPERATION_ENCRYPT, &identifier, &span, NULL,
                           UINT8_C(1)) != INT32_C(0) ||
        exercise_operation(kmipkit_decrypt_builder_create,
                           KMIPKIT_ENUMERATION_OPERATION_DECRYPT, &identifier, &span, NULL,
                           UINT8_C(2)) != INT32_C(0) ||
        exercise_operation(kmipkit_sign_builder_create,
                           KMIPKIT_ENUMERATION_OPERATION_SIGN, &identifier, &span, NULL,
                           UINT8_C(3)) != INT32_C(0) ||
        exercise_operation(kmipkit_signature_verify_builder_create,
                           KMIPKIT_ENUMERATION_OPERATION_SIGNATURE_VERIFY, &identifier, &span,
                           NULL, UINT8_C(4)) != INT32_C(0) ||
        exercise_operation(kmipkit_mac_builder_create, KMIPKIT_ENUMERATION_OPERATION_MAC,
                           &identifier, &span, NULL, UINT8_C(0)) != INT32_C(0) ||
        exercise_operation(kmipkit_mac_verify_builder_create,
                           KMIPKIT_ENUMERATION_OPERATION_MAC_VERIFY, &identifier, &span, NULL,
                           UINT8_C(5)) != INT32_C(0) ||
        exercise_operation(kmipkit_hash_builder_create, KMIPKIT_ENUMERATION_OPERATION_HASH,
                           NULL, &span, parameters, UINT8_C(0)) != INT32_C(0)) {
        kmipkit_cryptographic_parameters_release(parameters);
        kmipkit_codec_limits_release(limits);
        return 3;
    }
    if (kmipkit_ttlv_value_byte_string(limits, input, (uint64_t)(sizeof(input) - 1),
                                       &data_value) != INT32_C(0) ||
        kmipkit_sign_builder_create(&builder) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_set_data_value(builder, data_value) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_set_data(builder, NULL) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_crypto_operation_builder_set_mac_data(builder, &span) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_crypto_operation_builder_set_init_indicator(builder, UINT8_C(3)) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_crypto_operation_builder_set_final_indicator(builder, UINT8_C(3)) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(builder, UINT32_C(0)) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(
            builder, KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_MANDATORY) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(
            builder, KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_OPTIONAL) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(
            builder, KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_PROHIBITED) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(
            builder, UINT32_C(0x80000000)) != INT32_C(0)) {
        kmipkit_ttlv_value_release(data_value);
        kmipkit_crypto_operation_builder_release(builder);
        kmipkit_cryptographic_parameters_release(parameters);
        kmipkit_codec_limits_release(limits);
        return 4;
    }
    kmipkit_ttlv_value_release(data_value);
    kmipkit_crypto_operation_builder_release(builder);
    kmipkit_cryptographic_parameters_release(parameters);
    kmipkit_codec_limits_release(limits);

    builder = NULL;
    if (kmipkit_hash_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        kmipkit_crypto_operation_builder_release(builder);
        return 7;
    }
    kmipkit_client_batch_item_t *missing_hash_parameters = NULL;
    if (kmipkit_crypto_operation_builder_build_batch_item(builder, &missing_hash_parameters) !=
            KMIPKIT_ERROR_INVALID_INPUT || missing_hash_parameters != NULL ||
        kmipkit_crypto_operation_builder_set_unique_identifier(builder, &identifier) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        kmipkit_client_batch_item_release(missing_hash_parameters);
        kmipkit_crypto_operation_builder_release(builder);
        return 8;
    }
    kmipkit_crypto_operation_builder_release(builder);

    if (kmipkit_encrypt_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        return 9;
    }
    if (kmipkit_crypto_operation_builder_set_data(builder, &span) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_build_batch_item(builder, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        kmipkit_crypto_operation_builder_release(builder);
        return 10;
    }
    kmipkit_crypto_operation_builder_release(builder);
    return 0;
}

static int execute_crypto_operation(
    crypto_builder_create_fn create,
    uint32_t expected_operation,
    kmipkit_client_t *client,
    kmipkit_codec_limits_t *limits,
    kmipkit_unique_identifier_v1_t *identifier,
    kmipkit_byte_span_v1_t *data,
    kmipkit_cryptographic_parameters_t *parameters,
    uint8_t special_field) {
    kmipkit_crypto_operation_builder_t *builder = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *item = NULL;
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    kmipkit_request_options_v1_t options = {0};
    kmipkit_byte_span_v1_t correlation = {0};
    kmipkit_byte_span_v1_t owned_input = {0};
    uint8_t owned_input_bytes[256] = {0};
    uint64_t item_count = 0;
    uint8_t correlation_present = 0;
    uint32_t expected_status = KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_FAILED;
    uint32_t expected_outcome_kind = UINT32_C(1);
    if (data == NULL || data->length > (uint64_t)sizeof(owned_input_bytes) ||
        (data->length != 0 && data->data == NULL)) {
        return 24;
    }
    owned_input = *data;
    owned_input.data = owned_input_bytes;
    if (data->length != 0) {
        memcpy(owned_input_bytes, data->data, (size_t)data->length);
    }
    int32_t status = create(&builder);
    if (status == INT32_C(0) && identifier != NULL) {
        status = kmipkit_crypto_operation_builder_set_unique_identifier(builder, identifier);
    }
    if (status == INT32_C(0)) {
        static const uint8_t replaced_input[] = "replaced caller value";
        if (expected_operation != KMIPKIT_ENUMERATION_OPERATION_SIGN) {
            kmipkit_byte_span_v1_t first_data = {0};
            first_data.data = replaced_input;
            first_data.length = (uint64_t)(sizeof(replaced_input) - 1);
            status = kmipkit_crypto_operation_builder_set_data(builder, &first_data);
        }
        if (status == INT32_C(0) && expected_operation != KMIPKIT_ENUMERATION_OPERATION_SIGN) {
            status = kmipkit_crypto_operation_builder_set_data(builder, &owned_input);
        }
    }
    if (status == INT32_C(0) && parameters != NULL) {
        status = kmipkit_crypto_operation_builder_set_cryptographic_parameters(
            builder, parameters);
    }
    if (status == INT32_C(0) && special_field == UINT8_C(1)) {
        status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, &owned_input);
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(
                builder, &owned_input);
        }
    } else if (status == INT32_C(0) && special_field == UINT8_C(2)) {
        status = kmipkit_crypto_operation_builder_set_iv_counter_nonce(builder, &owned_input);
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_tag(
                builder, &owned_input);
        }
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(
                builder, &owned_input);
        }
    } else if (status == INT32_C(0) && special_field == UINT8_C(3)) {
        status = kmipkit_crypto_operation_builder_set_digested_data(builder, &owned_input);
    } else if (status == INT32_C(0) && special_field == UINT8_C(4)) {
        status = kmipkit_crypto_operation_builder_set_digested_data(builder, &owned_input);
        if (status == INT32_C(0)) {
            status = kmipkit_crypto_operation_builder_set_signature_data(builder, &owned_input);
        }
    } else if (status == INT32_C(0) && special_field == UINT8_C(5)) {
        status = kmipkit_crypto_operation_builder_set_mac_data(builder, &owned_input);
    }
    if (status == INT32_C(0) && expected_operation == KMIPKIT_ENUMERATION_OPERATION_ENCRYPT) {
        status = kmipkit_crypto_operation_builder_set_asynchronous_indicator(
            builder, KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_OPTIONAL);
    }
    memset(owned_input_bytes, 0, sizeof(owned_input_bytes));
    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    if (status == INT32_C(0)) {
        status = kmipkit_crypto_operation_builder_execute_with_options_v1(
            builder, client, limits, &options, &response);
    }
    if (status == INT32_C(0) && response != NULL) {
        status = kmipkit_client_batch_response_item_count(response, &item_count);
    }
    if (status == INT32_C(0) && item_count == UINT64_C(1)) {
        status = kmipkit_client_batch_response_item_at(response, UINT64_C(0), &item);
    } else if (status == INT32_C(0)) {
        status = INT32_C(20);
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(2);
    if (status == INT32_C(0)) {
        if (expected_operation == KMIPKIT_ENUMERATION_OPERATION_ENCRYPT) {
            expected_status = KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_PENDING;
            expected_outcome_kind = UINT32_C(2);
        } else if (expected_operation == KMIPKIT_ENUMERATION_OPERATION_HASH) {
            expected_status = KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS;
        }
        status = kmipkit_client_batch_item_response_metadata_v2(item, &metadata);
    }
    if (status == INT32_C(0) &&
        (metadata.operation != expected_operation ||
         metadata.outcome_kind != expected_outcome_kind ||
         metadata.result_status != expected_status)) {
        status = INT32_C(21);
    }
    if (status == INT32_C(0)) {
        status = kmipkit_client_batch_item_response_pending_correlation_value(
            item, &correlation_present, &correlation);
    }
    if (status == INT32_C(0) && expected_operation == KMIPKIT_ENUMERATION_OPERATION_ENCRYPT &&
        (correlation_present != UINT8_C(1) ||
         correlation.length != (uint64_t)(sizeof("pending-correlation") - 1) ||
         memcmp(correlation.data, "pending-correlation", sizeof("pending-correlation") - 1) != 0)) {
        status = INT32_C(22);
    }
    if (status == INT32_C(0) && expected_operation != KMIPKIT_ENUMERATION_OPERATION_ENCRYPT &&
        correlation_present != UINT8_C(0)) {
        status = INT32_C(23);
    }
    kmipkit_client_batch_item_response_release(item);
    kmipkit_client_batch_response_release(response);
    kmipkit_crypto_operation_builder_release(builder);
    return status;
}

int kmipkit_public_crypto_execute_tls_probe(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length,
    uint8_t *out_version_rejection_verified) {
    static const uint8_t endpoint[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint8_t identifier_text[] = "fluent-crypto-key";
    static const uint8_t input[] = "fluent crypto input";
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_client_configuration_t *unsupported_configuration = NULL;
    kmipkit_client_t *unsupported_client = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_cryptographic_parameters_t *parameters = NULL;
    kmipkit_byte_span_v1_t client_certificates[2] = {{0}};
    kmipkit_byte_span_v1_t trust_certificates[1] = {{0}};
    kmipkit_mtls_client_config_v1_t tls = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_cryptographic_parameters_fields_v1_t parameter_fields = {0};
    int32_t result = 1;

#define REQUIRE_CRYPTO_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto crypto_cleanup; \
        } \
    } while (0)

    if (out_version_rejection_verified == NULL || port == 0 ||
        leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0) {
        goto crypto_cleanup;
    }
    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_CRYPTO_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_CRYPTO_C_API(kmipkit_client_extension_registry_create_v1(
        NULL, UINT64_C(0), &registry_limits, &registry), 3);
    REQUIRE_CRYPTO_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
    registry = NULL;

    client_certificates[0].data = leaf_certificate;
    client_certificates[0].length = leaf_certificate_length;
    client_certificates[1].data = intermediate_certificate;
    client_certificates[1].length = intermediate_certificate_length;
    trust_certificates[0].data = trust_root;
    trust_certificates[0].length = trust_root_length;
    tls.struct_size = (uint32_t)sizeof(tls);
    tls.abi_version = UINT32_C(1);
    tls.endpoint_host.data = endpoint;
    tls.endpoint_host.length = (uint64_t)(sizeof(endpoint) - 1);
    tls.endpoint_port = port;
    tls.tls_server_name.data = server_name;
    tls.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    tls.client_certificates = client_certificates;
    tls.client_certificate_count = UINT64_C(2);
    tls.client_private_key.data = private_key;
    tls.client_private_key.length = private_key_length;
    tls.trust_certificates = trust_certificates;
    tls.trust_certificate_count = UINT64_C(1);
    tls.max_request_bytes = UINT64_C(1024 * 1024);
    REQUIRE_CRYPTO_C_API(kmipkit_client_create_mtls_v1(configuration, &tls, &client), 5);
    REQUIRE_CRYPTO_C_API(kmipkit_codec_limits_defaults(&limits), 6);

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    data.data = input;
    data.length = (uint64_t)(sizeof(input) - 1);
    parameter_fields.struct_size = (uint32_t)sizeof(parameter_fields);
    parameter_fields.abi_version = UINT32_C(1);
    parameter_fields.hashing_algorithm_present = UINT8_C(1);
    parameter_fields.hashing_algorithm = KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_256;
    REQUIRE_CRYPTO_C_API(kmipkit_cryptographic_parameters_from_fields_v1(
        &parameter_fields, &parameters), 7);

    REQUIRE_CRYPTO_C_API(kmipkit_client_configuration_with_protocol_version(
        configuration, INT32_C(99), INT32_C(99), &unsupported_configuration), 8);
    REQUIRE_CRYPTO_C_API(kmipkit_client_create_mtls_v1(
        unsupported_configuration, &tls, &unsupported_client), 8);
    result = execute_crypto_operation(
        kmipkit_encrypt_builder_create, KMIPKIT_ENUMERATION_OPERATION_ENCRYPT,
        unsupported_client, limits, &identifier, &data, NULL, UINT8_C(1));
    if (result != KMIPKIT_ERROR_INVALID_INPUT) {
        result = 15;
        goto crypto_cleanup;
    }
    *out_version_rejection_verified = UINT8_C(1);
    kmipkit_client_release(unsupported_client);
    unsupported_client = NULL;
    kmipkit_client_configuration_release(unsupported_configuration);
    unsupported_configuration = NULL;

    result = execute_crypto_operation(
        kmipkit_encrypt_builder_create, KMIPKIT_ENUMERATION_OPERATION_ENCRYPT,
        client, limits, &identifier, &data, NULL, UINT8_C(1));
    if (result != INT32_C(0)) {
        goto crypto_cleanup;
    }
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_decrypt_builder_create, KMIPKIT_ENUMERATION_OPERATION_DECRYPT,
        client, limits, &identifier, &data, NULL, UINT8_C(2)), 9);
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_sign_builder_create, KMIPKIT_ENUMERATION_OPERATION_SIGN,
        client, limits, &identifier, &data, NULL, UINT8_C(3)), 10);
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_signature_verify_builder_create,
        KMIPKIT_ENUMERATION_OPERATION_SIGNATURE_VERIFY, client, limits,
        &identifier, &data, NULL, UINT8_C(4)), 11);
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_mac_builder_create, KMIPKIT_ENUMERATION_OPERATION_MAC,
        client, limits, &identifier, &data, NULL, UINT8_C(0)), 12);
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_mac_verify_builder_create, KMIPKIT_ENUMERATION_OPERATION_MAC_VERIFY,
        client, limits, &identifier, &data, NULL, UINT8_C(5)), 13);
    REQUIRE_CRYPTO_C_API(execute_crypto_operation(
        kmipkit_hash_builder_create, KMIPKIT_ENUMERATION_OPERATION_HASH,
        client, limits, NULL, &data, parameters, UINT8_C(0)), 14);
    result = 0;

crypto_cleanup:
    kmipkit_cryptographic_parameters_release(parameters);
    kmipkit_client_release(unsupported_client);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(unsupported_configuration);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
#undef REQUIRE_CRYPTO_C_API
    return result;
}
