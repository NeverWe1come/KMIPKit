/* KMIPKIT-0116 FR-001..005: KMIP 3.0 WD 2026-07-13 sections
 * 6.1.9, 6.1.10, 6.1.13, 6.1.14, 6.1.15, 6.1.22, 6.1.41. */
#include "kmip30_fluent_test_api.h"
#include <stddef.h>
#include <string.h>
#include <stdio.h>

#define CHECK(call) do { int32_t rc_ = (call); if (rc_ != 0) return __LINE__; } while (0)
#define ASSERT(condition) do { if (!(condition)) return __LINE__; } while (0)

static kmipkit_unique_identifier_v1_t identifier(void) {
    static const uint8_t text[] = "kmip30-key";
    kmipkit_unique_identifier_v1_t id = {0};
    id.struct_size = (uint32_t)sizeof(id);
    id.abi_version = UINT32_C(1);
    id.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    id.text_data = text;
    id.text_length = (uint64_t)(sizeof(text) - 1);
    return id;
}

int kmipkit_c_kmip30_builders_probe(void) {
    uint8_t caller_bytes[] = "credential";
    kmipkit_byte_span_v1_t bytes = {caller_bytes, sizeof(caller_bytes) - 1};
    kmipkit_unique_identifier_v1_t id = identifier();
    kmipkit_attribute_set_t *attributes = NULL;
    kmipkit_create_credential_object_t *object = NULL;
    kmipkit_kmip30_operation_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_cryptographic_parameters_t *parameters = NULL;
    kmipkit_cryptographic_parameters_fields_v1_t fields = {0};
    kmipkit_ttlv_structure_t *reason = NULL;
    kmipkit_ttlv_item_t *certificate = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    uint32_t operation = 0;
    const uint32_t operations[] = {0x3f, 0x3c, 0x3e, 0x40, 0x42, 0x41, 0x3d};
    CHECK(kmipkit_ttlv_structure_create(&reason));
    CHECK(kmipkit_attribute_set_create(reason, &attributes));
    CHECK(kmipkit_codec_limits_defaults(&limits));
    ASSERT(kmipkit_create_group_builder_create(NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_create_user_builder_create(NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_create_credential_builder_create(attributes, NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_deactivate_builder_create(NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_decapsulate_builder_create(&id, NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_encapsulate_builder_create(NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_obliterate_builder_create(NULL, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    ASSERT(kmipkit_create_group_builder_create((kmipkit_attribute_set_t *)reason, &builder) == KMIPKIT_ERROR_INVALID_INPUT && builder == NULL);
    _Alignas(void *) uint8_t misaligned_slot[sizeof(void *) + 1] = {0};
    ASSERT(kmipkit_encapsulate_builder_create(&id, (kmipkit_kmip30_operation_builder_t **)(misaligned_slot + 1)) == KMIPKIT_ERROR_INVALID_INPUT);
    ASSERT(kmipkit_deactivate_builder_create((const kmipkit_unique_identifier_v1_t *)((const uint8_t *)&id + 1), &builder) == KMIPKIT_ERROR_INVALID_INPUT);
    fields.struct_size = (uint32_t)sizeof(fields);
    fields.abi_version = 1;
    CHECK(kmipkit_cryptographic_parameters_from_fields_v1(&fields, &parameters));
    ASSERT(kmipkit_create_credential_object_device(NULL, NULL, NULL, NULL, NULL, &object) ==
           KMIPKIT_ERROR_INVALID_INPUT && object == NULL);
    ASSERT(kmipkit_create_credential_object_hashed_password(0, 0, NULL, NULL, &object) ==
           KMIPKIT_ERROR_INVALID_INPUT && object == NULL);
    ASSERT(kmipkit_create_credential_object_password(&bytes, NULL, 2, 0, NULL, 0, 0, &object) ==
           KMIPKIT_ERROR_INVALID_INPUT && object == NULL);
    CHECK(kmipkit_create_credential_object_device(&bytes, NULL, NULL, NULL, NULL, &object));
    kmipkit_create_credential_object_release(object);
    object = NULL;
    CHECK(kmipkit_create_credential_object_hashed_password(0, 0, &bytes, &bytes, &object));
    kmipkit_create_credential_object_release(object);
    object = NULL;
    CHECK(kmipkit_create_credential_object_one_time_password(1, 0, 0, NULL, &bytes, 1, 30, 1, 6, &object));
    kmipkit_create_credential_object_release(object);
    object = NULL;
    ASSERT(kmipkit_create_credential_object_certificate(NULL, &object) == KMIPKIT_ERROR_INVALID_INPUT);
    static const uint8_t certificate_ttlv[] = {
        0x42,0x00,0x13,0x01,0,0,0,32,
        0x42,0x00,0x1d,0x05,0,0,0,4,0,0,0,1,0,0,0,0,
        0x42,0x00,0x1e,0x08,0,0,0,4,1,2,3,4,0,0,0,0
    };
    kmipkit_ttlv_decode_error_v1_t decode_error = {0};
    decode_error.struct_size = (uint32_t)sizeof(decode_error);
    decode_error.abi_version = 1;
    CHECK(kmipkit_ttlv_decode(certificate_ttlv, sizeof(certificate_ttlv), &decode_error, &certificate));
    CHECK(kmipkit_create_credential_object_certificate(certificate, &object));
    kmipkit_create_credential_object_release(object);
    object = NULL;
    CHECK(kmipkit_create_credential_object_password(&bytes, NULL, 0, 0, NULL, 0, 0, &object));
    for (uint32_t index = 0; index < 7; ++index) {
        switch (index) {
        case 0: CHECK(kmipkit_create_credential_builder_create(attributes, object, &builder)); break;
        case 1: CHECK(kmipkit_create_group_builder_create(attributes, &builder)); break;
        case 2: CHECK(kmipkit_create_user_builder_create(attributes, &builder)); break;
        case 3: CHECK(kmipkit_deactivate_builder_create(&id, &builder)); break;
        case 4: CHECK(kmipkit_decapsulate_builder_create(&id, &bytes, &builder)); break;
        case 5: CHECK(kmipkit_encapsulate_builder_create(&id, &builder)); break;
        default: CHECK(kmipkit_obliterate_builder_create(&id, &builder)); break;
        }
        if (index == 3) {
            CHECK(kmipkit_kmip30_operation_builder_set_deactivation_reason(builder, reason));
            CHECK(kmipkit_kmip30_operation_builder_set_deactivation_date(builder, INT64_C(42)));
            ASSERT(kmipkit_kmip30_operation_builder_set_attributes(builder, attributes) == KMIPKIT_ERROR_INVALID_INPUT);
        } else {
            ASSERT(kmipkit_kmip30_operation_builder_set_deactivation_date(builder, 42) == KMIPKIT_ERROR_INVALID_INPUT);
        }
        if (index == 4 || index == 5) {
            CHECK(kmipkit_kmip30_operation_builder_set_attributes(builder, attributes));
            CHECK(kmipkit_kmip30_operation_builder_set_cryptographic_parameters(builder, parameters));
        } else {
            ASSERT(kmipkit_kmip30_operation_builder_set_cryptographic_parameters(builder, parameters) == KMIPKIT_ERROR_INVALID_INPUT);
        }
        ASSERT(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, 0) == KMIPKIT_ERROR_INVALID_INPUT);
        CHECK(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, UINT32_C(0x80000000)));
        CHECK(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, UINT32_C(0x8fffffff)));
        ASSERT(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, UINT32_C(0x90000000)) == KMIPKIT_ERROR_INVALID_INPUT);
        CHECK(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, 2));
        CHECK(kmipkit_kmip30_operation_builder_validate(builder));
        CHECK(kmipkit_kmip30_operation_builder_build_batch_item(builder, &item));
        CHECK(kmipkit_client_batch_item_operation(item, &operation));
        ASSERT(operation == operations[index]);
        kmipkit_client_batch_item_release(item);
        item = NULL;
        kmipkit_kmip30_operation_builder_release(builder);
        ASSERT(kmipkit_kmip30_operation_builder_validate(builder) == KMIPKIT_ERROR_INVALID_INPUT);
        builder = NULL;
    }
    CHECK(kmipkit_create_credential_builder_create(attributes, object, &builder));
    kmipkit_create_credential_object_release(object);
    kmipkit_attribute_set_release(attributes);
    memset(caller_bytes, 0, sizeof(caller_bytes));
    CHECK(kmipkit_kmip30_operation_builder_build_batch_item(builder, &item));
    kmipkit_client_batch_item_release(item);
    kmipkit_kmip30_operation_builder_release(builder);
    kmipkit_cryptographic_parameters_release(parameters);
    kmipkit_ttlv_structure_release(reason);
    kmipkit_ttlv_item_release(certificate);
    kmipkit_codec_limits_release(limits);
    return 0;
}

/* Called with a native controlled client by the Rust fixture; this C code owns
 * all builder, object, request and result handles created in this call. */
int kmipkit_c_kmip30_exchange_probe(kmipkit_client_t *client, kmipkit_codec_limits_t *limits,
                                  uint32_t operation_index, uint8_t builder_path,
                                  uint8_t use_options,
                                  uint32_t expected_status) {
    uint8_t caller_bytes[] = "credential";
    kmipkit_byte_span_v1_t bytes = {caller_bytes, sizeof(caller_bytes) - 1};
    kmipkit_unique_identifier_v1_t id = identifier();
    kmipkit_unique_identifier_v1_t result_id = identifier();
    kmipkit_attribute_set_t *attributes = NULL;
    kmipkit_create_credential_object_t *object = NULL;
    kmipkit_kmip30_operation_builder_t *builder = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *item = NULL;
    kmipkit_request_options_v1_t options = {0};
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    kmipkit_byte_span_v1_t public_data = {0};
    uint8_t present = 0;
    uint64_t count = 0;
    int32_t status = 0;
    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = 1;
    kmipkit_ttlv_structure_t *empty_attributes = NULL;
    CHECK(kmipkit_ttlv_structure_create(&empty_attributes));
    CHECK(kmipkit_attribute_set_create(empty_attributes, &attributes));
    kmipkit_ttlv_structure_release(empty_attributes);
    CHECK(kmipkit_create_credential_object_password(&bytes, NULL, 0, 0, NULL, 0, 0, &object));
    if (builder_path) {
        switch (operation_index) {
        case 0: CHECK(kmipkit_create_credential_builder_create(attributes, object, &builder)); break;
        case 1: CHECK(kmipkit_create_group_builder_create(attributes, &builder)); break;
        case 2: CHECK(kmipkit_create_user_builder_create(attributes, &builder)); break;
        case 3: CHECK(kmipkit_deactivate_builder_create(&id, &builder)); break;
        case 4: CHECK(kmipkit_decapsulate_builder_create(&id, &bytes, &builder)); break;
        case 5: CHECK(kmipkit_encapsulate_builder_create(&id, &builder)); break;
        default: CHECK(kmipkit_obliterate_builder_create(&id, &builder)); break;
        }
        if (operation_index == 3) {
            CHECK(kmipkit_kmip30_operation_builder_set_deactivation_date(builder, INT64_C(42)));
        }
        if (operation_index == 4 || operation_index == 5) {
            kmipkit_cryptographic_parameters_t *parameters = NULL;
            kmipkit_cryptographic_parameters_fields_v1_t fields = {0};
            fields.struct_size = (uint32_t)sizeof(fields);
            fields.abi_version = 1;
            fields.hashing_algorithm_present = 1;
            fields.hashing_algorithm = 6;
            CHECK(kmipkit_cryptographic_parameters_from_fields_v1(&fields, &parameters));
            CHECK(kmipkit_kmip30_operation_builder_set_cryptographic_parameters(builder, parameters));
            CHECK(kmipkit_kmip30_operation_builder_set_attributes(builder, attributes));
            kmipkit_cryptographic_parameters_release(parameters);
        }
        /* Copies survive source release and caller mutation before exchange. */
        kmipkit_create_credential_object_release(object); object = NULL;
        kmipkit_attribute_set_release(attributes); attributes = NULL;
        memset(caller_bytes, 0, sizeof(caller_bytes));
        CHECK(kmipkit_kmip30_operation_builder_set_asynchronous_indicator(builder, 2));
        status = use_options ? kmipkit_kmip30_operation_builder_execute_with_options_v1(
            builder, client, limits, &options, &response) :
            kmipkit_kmip30_operation_builder_execute_v1(builder, client, limits, &response);
    } else {
#define EXECUTE(name, ...) (use_options ? kmipkit_client_##name##_with_options_v1(client, __VA_ARGS__, limits, &options, &response) : kmipkit_client_##name##_v1(client, __VA_ARGS__, limits, &response))
        switch (operation_index) {
        case 0: status = EXECUTE(create_credential, 1, attributes, object); break;
        case 1: status = EXECUTE(create_group, attributes); break;
        case 2: status = EXECUTE(create_user, attributes); break;
        case 3: status = EXECUTE(deactivate, &id); break;
        case 4: status = EXECUTE(decapsulate, &id, &bytes); break;
        case 5: status = EXECUTE(encapsulate, &id); break;
        default: status = EXECUTE(obliterate, &id); break;
        }
#undef EXECUTE
    }
    kmipkit_kmip30_operation_builder_release(builder);
    kmipkit_create_credential_object_release(object);
    kmipkit_attribute_set_release(attributes);
    if (expected_status == UINT32_MAX) {
        ASSERT(status == KMIPKIT_ERROR_INVALID_INPUT && response == NULL);
        return 0;
    }
    if (status != 0 || response == NULL) {
        fprintf(stderr, "KMIP 3.0 C exchange failed: operation index %u, builder %u, status %d\n", operation_index, builder_path, status);
    }
    ASSERT(status == 0 && response != NULL);
    CHECK(kmipkit_client_batch_response_item_count(response, &count));
    ASSERT(count == 1);
    CHECK(kmipkit_client_batch_response_item_at(response, 0, &item));
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    CHECK(kmipkit_client_batch_item_response_metadata_v2(item, &metadata));
    ASSERT(metadata.result_status == expected_status);
    kmipkit_byte_span_v1_t correlation = {0};
    CHECK(kmipkit_client_batch_item_response_pending_correlation_value(item, &present, &correlation));
    ASSERT(present == (expected_status == 2));
    if (present) {
        ASSERT(correlation.length == sizeof("pending-correlation") - 1);
        ASSERT(memcmp(correlation.data, "pending-correlation", sizeof("pending-correlation") - 1) == 0);
    }
    CHECK(kmipkit_client_batch_item_response_kmip30_identifier(item, &present, &result_id));
    ASSERT(present == (expected_status == 0 && operation_index != 6));
    if (present) {
        ASSERT(result_id.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
        ASSERT(result_id.text_length == sizeof("result-id") - 1);
        ASSERT(memcmp(result_id.text_data, "result-id", sizeof("result-id") - 1) == 0);
    }
    CHECK(kmipkit_client_batch_item_response_encapsulate_data(item, &present, &public_data));
    ASSERT(present == (expected_status == 0 && operation_index == 5));
    if (present) {
        ASSERT(public_data.length == sizeof("public-ciphertext") - 1);
        ASSERT(memcmp(public_data.data, "public-ciphertext", sizeof("public-ciphertext") - 1) == 0);
    }
    kmipkit_client_batch_item_response_release(item);
    kmipkit_client_batch_response_release(response);
    return 0;
}

int kmipkit_c_kmip30_tls_probe(
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
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_client_configuration_t *unsupported_configuration = NULL;
    kmipkit_client_t *unsupported_client = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_byte_span_v1_t client_certificates[2] = {{0}};
    kmipkit_byte_span_v1_t trust_certificates[1] = {{0}};
    kmipkit_mtls_client_config_v1_t tls = {0};
    int32_t result = 1;

#define REQUIRE_KMIP30_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto kmip30_cleanup; \
        } \
    } while (0)

    if (out_version_rejection_verified == NULL || port == 0 ||
        leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0) {
        goto kmip30_cleanup;
    }
    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_KMIP30_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_KMIP30_C_API(kmipkit_client_extension_registry_create_v1(
        NULL, UINT64_C(0), &registry_limits, &registry), 3);
    REQUIRE_KMIP30_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
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
    REQUIRE_KMIP30_C_API(kmipkit_client_create_mtls_v1(configuration, &tls, &client), 5);
    REQUIRE_KMIP30_C_API(kmipkit_codec_limits_defaults(&limits), 6);


    /* The 2.1 default must reject every 3.0 route before opening a socket. */
    for (uint32_t op = 0; op < 7; ++op) {
        REQUIRE_KMIP30_C_API(kmipkit_c_kmip30_exchange_probe(
            client, limits, op, 0, 0, UINT32_MAX), 20);
        REQUIRE_KMIP30_C_API(kmipkit_c_kmip30_exchange_probe(
            client, limits, op, 0, 1, UINT32_MAX), 20);
        REQUIRE_KMIP30_C_API(kmipkit_c_kmip30_exchange_probe(
            client, limits, op, 1, 1, UINT32_MAX), 21);
        REQUIRE_KMIP30_C_API(kmipkit_c_kmip30_exchange_probe(
            client, limits, op, 1, 0, UINT32_MAX), 21);
    }
    *out_version_rejection_verified = 1;
    kmipkit_client_release(client); client = NULL;
    REQUIRE_KMIP30_C_API(kmipkit_client_configuration_with_protocol_version(
        configuration, 3, 0, &unsupported_configuration), 22);
    REQUIRE_KMIP30_C_API(kmipkit_client_create_mtls_v1(
        unsupported_configuration, &tls, &client), 23);
    for (uint32_t path = 0; path < 2; ++path) {
        for (uint32_t op = 0; op < 7; ++op) {
            uint32_t status = op == 1 && path == 1 ? 2 : (op == 2 ? 1 : 0);
            result = kmipkit_c_kmip30_exchange_probe(client, limits, op,
                (uint8_t)path, (uint8_t)(op % 2), status);
            if (result != 0) goto kmip30_cleanup;
        }
    }
    result = 0;
kmip30_cleanup:
    kmipkit_client_release(unsupported_client);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(unsupported_configuration);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
#undef REQUIRE_KMIP30_C_API
    return result;
}
