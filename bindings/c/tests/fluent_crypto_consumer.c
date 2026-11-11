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
        kmipkit_crypto_operation_builder_set_asynchronous_indicator(builder, UINT32_C(2)) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
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

    if (kmipkit_encrypt_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        return 5;
    }
    if (kmipkit_crypto_operation_builder_set_data(builder, &span) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_build_batch_item(builder, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        kmipkit_crypto_operation_builder_release(builder);
        return 6;
    }
    kmipkit_crypto_operation_builder_release(builder);
    return 0;
}
