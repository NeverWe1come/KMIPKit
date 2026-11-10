/* Compile and exercise the client construction surface from the public header. */
#include "kmipkit.h"

#include <string.h>

int kmipkit_public_client_surface_probe(void) {
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    int32_t status = kmipkit_client_batch_create(&batch);
    if (status != 0 || batch == NULL) {
        return 1;
    }

    status = kmipkit_client_batch_item_discover_versions(&item);
    if (status != 0 || item == NULL) {
        kmipkit_client_batch_release(batch);
        return 2;
    }
    status = kmipkit_client_batch_add_item(batch, &item);
    if (status != 0 || item != NULL) {
        if (item != NULL) {
            kmipkit_client_batch_item_release(item);
        }
        kmipkit_client_batch_release(batch);
        return 3;
    }
    kmipkit_client_batch_release(batch);

    /* A null client/batch is rejected before I/O; -1 exercises the no-override path. */
    kmipkit_client_batch_t *empty_batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    status = kmipkit_client_execute_with_options(NULL, &empty_batch, NULL, -1, &response);
    return status == 4 && response == NULL ? 0 : 4;
}

int kmipkit_public_fluent_builder_probe(void) {
    static const uint8_t name[] = "primary encryption key";
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t operation = UINT32_C(0);
    int32_t status;

    status = kmipkit_symmetric_key_builder_create(&builder);
    if (status != INT32_C(0) || builder == NULL) {
        return 1;
    }
    if (kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_length(builder, INT32_C(256)) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder,
            (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                      KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT)) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_name(
            builder, name, (uint64_t)(sizeof(name) - 1)) != INT32_C(0)) {
        kmipkit_symmetric_key_builder_release(builder);
        return 2;
    }
    status = kmipkit_symmetric_key_builder_build_batch_item(builder, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CREATE) {
        kmipkit_client_batch_item_release(item);
        kmipkit_symmetric_key_builder_release(builder);
        return 3;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_symmetric_key_builder_release(builder);

    /* A larger struct_size is forward-compatible; null client fails before network I/O. */
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_client_batch_response_t *response = NULL;
    request.struct_size = (uint32_t)sizeof(request) + UINT32_C(8);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask =
        (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                  KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    status = kmipkit_client_create_symmetric_key_v1(NULL, &request, NULL, &response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        kmipkit_client_batch_response_release(response);
        return 4;
    }

    request.struct_size = (uint32_t)sizeof(request) - UINT32_C(1);
    response = NULL;
    status = kmipkit_client_create_symmetric_key_v1(NULL, &request, NULL, &response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        kmipkit_client_batch_response_release(response);
        return 5;
    }
    return 0;
}

int kmipkit_public_fluent_validation_probe(void) {
    static const uint8_t description[] = "managed without TTLV tags";
    static const uint8_t unknown_value[] = "expert value";
    static const uint8_t invalid_utf8[] = {UINT8_C(0xff)};
    static const uint8_t vendor[] = "Vendor_Name.1";
    static const uint8_t vendor_name[] = "region";
    static const uint8_t identifier_text[] = "fluent-id";
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_t *wrong_value = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_raw_tag_t *raw_vendor_tag = NULL;
    kmipkit_tag_t *vendor_tag = NULL;
    kmipkit_ttlv_value_t *vendor_value = NULL;
    kmipkit_ttlv_item_t *vendor_item = NULL;
    kmipkit_client_batch_item_t *built_item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_request_options_v1_t options = {0};
    uint32_t operation = 0;
    int32_t result = 1;

    if (kmipkit_symmetric_key_builder_set_algorithm(NULL, 3) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_length(NULL, 256) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_usage_mask(NULL, 12) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_name(NULL, description, sizeof(description) - 1) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_create(&builder) != 0 || builder == NULL) {
        result = 11;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_build_batch_item(builder, &built_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || built_item != NULL) {
        result = 121;
        goto cleanup;
    }
    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == NULL) {
        result = 122;
        goto cleanup;
    }
    if (kmipkit_ttlv_value_text_string(limits, description, sizeof(description) - 1, &value) != 0) {
        result = 123;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_add_catalogued_attribute(
            builder, UINT32_C(0x4200fc), value) != 0) {
        result = 124;
        goto cleanup;
    }
    if (kmipkit_ttlv_value_integer(1, &wrong_value) != 0) {
        result = 125;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_add_catalogued_attribute(
            builder, UINT32_C(0x4200fc), wrong_value) !=
        KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH) {
        result = 126;
        goto cleanup;
    }
    kmipkit_ttlv_value_release(wrong_value);
    wrong_value = NULL;
    if (
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x4200fe), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_text_string(limits, unknown_value, sizeof(unknown_value) - 1, &wrong_value) != 0 ||
        kmipkit_ttlv_item_create(tag, wrong_value, limits, &item) != 0) {
        result = 13;
        goto cleanup;
    }
    wrong_value = NULL; /* The successful Item constructor consumes its Value. */
    if (kmipkit_symmetric_key_builder_add_attribute_item(builder, item) != 0 ||
        kmipkit_symmetric_key_builder_add_vendor_attribute(
            builder, vendor, sizeof(vendor) - 1, vendor_name, sizeof(vendor_name) - 1, item) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x540001), &raw_vendor_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_vendor_tag, &vendor_tag) != 0 ||
        kmipkit_ttlv_value_text_string(limits, unknown_value, sizeof(unknown_value) - 1,
                                       &vendor_value) != 0 ||
        kmipkit_ttlv_item_create(vendor_tag, vendor_value, limits, &vendor_item) != 0) {
        result = 14;
        goto cleanup;
    }
    vendor_value = NULL;
    if (kmipkit_symmetric_key_builder_add_attribute_item(builder, vendor_item) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_name(builder, invalid_utf8, sizeof(invalid_utf8)) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != 0 ||
        kmipkit_symmetric_key_builder_set_length(builder, 256) != 0 ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT) != 0 ||
        kmipkit_symmetric_key_builder_build_batch_item(builder, &built_item) != 0 ||
        built_item == NULL ||
        kmipkit_client_batch_item_operation(built_item, &operation) != 0 ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CREATE) {
        result = 15;
        goto cleanup;
    }

    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    if (kmipkit_symmetric_key_builder_execute_v1(builder, NULL, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL ||
        kmipkit_symmetric_key_builder_execute_with_options_v1(
            builder, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        result = 16;
        goto cleanup;
    }
    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    if (kmipkit_client_create_symmetric_key_v1(NULL, &request, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        result = 171;
        goto cleanup;
    }
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask = KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT;
    if (kmipkit_client_create_symmetric_key_v1(NULL, &request, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_create_symmetric_key_with_options_v1(
            NULL, &request, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT) {
        result = 17;
        goto cleanup;
    }
    if (kmipkit_client_create_symmetric_key_with_options_v1(
            NULL, &request, limits, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL ||
        kmipkit_client_get_by_id_with_options_v1(
            NULL, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL ||
        kmipkit_client_destroy_by_id_with_options_v1(
            NULL, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        result = 172;
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = sizeof(identifier_text) - 1;
    if (kmipkit_client_get_by_id_v1(NULL, &identifier, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_get_by_id_with_options_v1(
            NULL, &identifier, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_destroy_by_id_v1(NULL, &identifier, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_destroy_by_id_with_options_v1(
            NULL, &identifier, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT) {
        result = 18;
        goto cleanup;
    }
    options.abi_version = UINT32_C(2);
    result = kmipkit_client_get_by_id_with_options_v1(
        NULL, &identifier, limits, &options, &response) == KMIPKIT_ERROR_INVALID_INPUT ? 0 : 2;

cleanup:
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_item_release(built_item);
    kmipkit_ttlv_item_release(vendor_item);
    kmipkit_ttlv_value_release(vendor_value);
    kmipkit_tag_release(vendor_tag);
    kmipkit_raw_tag_release(raw_vendor_tag);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_value_release(wrong_value);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    kmipkit_symmetric_key_builder_release(builder);
    return result;
}

int kmipkit_public_client_execute_json_override(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length) {
    static const uint8_t endpoint[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_transport_config_t *transport_configuration = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport = {0};
    uint64_t response_count = 0;
    int32_t result = 1;
    int32_t execute_status;

#define REQUIRE_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto cleanup; \
        } \
    } while (0)

    if (leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0 || port == 0) {
        result = 1;
        goto cleanup;
    }

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_C_API(
        kmipkit_client_extension_registry_create_v1(NULL, UINT64_C(0),
                                                    &registry_limits, &registry),
        3);
    REQUIRE_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
    registry = NULL; /* The configuration constructor consumes this handle. */

    client_certificates[0].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    client_certificates[0].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    client_certificates[0].data.data = leaf_certificate;
    client_certificates[0].data.length = leaf_certificate_length;
    client_certificates[1].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    client_certificates[1].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    client_certificates[1].data.data = intermediate_certificate;
    client_certificates[1].data.length = intermediate_certificate_length;
    trust_certificates[0].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    trust_certificates[0].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    trust_certificates[0].data.data = trust_root;
    trust_certificates[0].data.length = trust_root_length;

    transport.struct_size = (uint32_t)sizeof(transport);
    transport.abi_version = UINT32_C(1);
    transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_RAW_TLS;
    transport.trust_kind = KMIPKIT_TRANSPORT_TRUST_CERTIFICATES;
    transport.flags = KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS_SERVER_NAME;
    transport.endpoint.data = endpoint;
    transport.endpoint.length = (uint64_t)(sizeof(endpoint) - 1);
    transport.endpoint_port = port;
    transport.client_certificates = client_certificates;
    transport.client_certificate_count = UINT64_C(2);
    transport.client_private_key.source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    transport.client_private_key.encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    transport.client_private_key.data.data = private_key;
    transport.client_private_key.data.length = private_key_length;
    transport.trust_certificates = trust_certificates;
    transport.trust_certificate_count = UINT64_C(1);
    transport.tls_server_name.data = server_name;
    transport.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    transport.connect_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.write_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.read_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.total_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.max_request_bytes = UINT64_C(1048576);

    REQUIRE_C_API(kmipkit_transport_config_create_v1(&transport, &transport_configuration), 5);
    REQUIRE_C_API(kmipkit_client_create(configuration, &transport_configuration, &client), 6);
    REQUIRE_C_API(kmipkit_codec_limits_defaults(&limits), 7);
    REQUIRE_C_API(kmipkit_client_batch_create(&batch), 8);
    REQUIRE_C_API(kmipkit_client_batch_item_discover_versions(&item), 9);
    REQUIRE_C_API(kmipkit_client_batch_add_item(batch, &item), 10);
    execute_status = kmipkit_client_execute_with_options(
        client, &batch, limits, KMIPKIT_WIRE_ENCODING_JSON, &response);
    if (execute_status != INT32_C(0)) {
        result = INT32_C(100) + execute_status;
        goto cleanup;
    }
    if (response == NULL) {
        result = 12;
        goto cleanup;
    }
    REQUIRE_C_API(kmipkit_client_batch_response_item_count(response, &response_count), 13);
    if (response_count != UINT64_C(1)) {
        result = 14;
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(item);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(configuration);
    kmipkit_transport_config_release(transport_configuration);
#undef REQUIRE_C_API
    return result;
}

int kmipkit_public_fluent_lifecycle_probe(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length) {
    static const uint8_t endpoint[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint8_t expected_identifier[] = "fluent-key-42";
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_client_batch_response_t *create_response = NULL;
    kmipkit_client_batch_item_response_t *create_item = NULL;
    kmipkit_client_batch_response_t *get_response = NULL;
    kmipkit_client_batch_item_response_t *get_item = NULL;
    kmipkit_client_batch_response_t *destroy_response = NULL;
    kmipkit_client_batch_item_response_t *destroy_item = NULL;
    kmipkit_ttlv_structure_t *get_object = NULL;
    kmipkit_byte_span_v1_t client_certificates[2] = {{0}};
    kmipkit_byte_span_v1_t trust_certificates[1] = {{0}};
    kmipkit_mtls_client_config_v1_t tls = {0};
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_request_options_v1_t request_options = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t create_present = 0;
    uint32_t create_object_type = 0;
    uint8_t get_present = 0;
    uint8_t get_payload_present = 0;
    uint32_t get_object_type = 0;
    uint8_t destroy_present = 0;
    int32_t result = 1;

#define REQUIRE_LIFECYCLE_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto lifecycle_cleanup; \
        } \
    } while (0)

    if (port == 0 || leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0) {
        goto lifecycle_cleanup;
    }

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_extension_registry_create_v1(
            NULL, UINT64_C(0), &registry_limits, &registry), 3);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
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
    tls.abi_version = UINT32_C(2);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 21;
        goto lifecycle_cleanup;
    }
    tls.abi_version = UINT32_C(1);
    tls.struct_size -= UINT32_C(1);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 22;
        goto lifecycle_cleanup;
    }
    tls.struct_size = (uint32_t)sizeof(tls);
    tls.client_certificates = NULL;
    tls.client_certificate_count = UINT64_C(0);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 23;
        goto lifecycle_cleanup;
    }
    tls.client_certificates = client_certificates;
    tls.client_certificate_count = UINT64_C(2);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_create_mtls_v1(configuration, &tls, &client), 6);
    REQUIRE_LIFECYCLE_C_API(kmipkit_codec_limits_defaults(&limits), 7);

    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask = (int32_t)(
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    request.name_present = UINT8_C(1);
    request.name.data = expected_identifier;
    request.name.length = (uint64_t)(sizeof(expected_identifier) - 1);
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_create_symmetric_key_v1(
        client, &request, limits, &create_response), 8);
    if (create_response == NULL) {
        result = 9;
        goto lifecycle_cleanup;
    }
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_batch_response_item_at(create_response, UINT64_C(0), &create_item), 10);
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_create(
        create_item, &create_present, &create_object_type, &identifier), 11);
    if (create_present != UINT8_C(1) ||
        create_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_length != (uint64_t)(sizeof(expected_identifier) - 1) ||
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) != 0) {
        result = 12;
        goto lifecycle_cleanup;
    }

    REQUIRE_LIFECYCLE_C_API(kmipkit_client_get_by_id_with_options_v1(
        client, &identifier, limits, &request_options, &get_response), 13);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_batch_response_item_at(get_response, UINT64_C(0), &get_item), 14);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_get(
        get_item, &get_present, &get_payload_present, &get_object_type, &identifier,
        &get_object), 15);
    if (get_present != UINT8_C(1) || get_payload_present != UINT8_C(1) ||
        get_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY || get_object == NULL) {
        result = 16;
        goto lifecycle_cleanup;
    }

    REQUIRE_LIFECYCLE_C_API(kmipkit_client_destroy_by_id_with_options_v1(
        client, &identifier, limits, &request_options, &destroy_response), 17);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_response_item_at(
        destroy_response, UINT64_C(0), &destroy_item), 18);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_destroy(
        destroy_item, &destroy_present, &identifier), 19);
    result = destroy_present == UINT8_C(1) ? 0 : 20;

lifecycle_cleanup:
    kmipkit_ttlv_structure_release(get_object);
    kmipkit_client_batch_item_response_release(destroy_item);
    kmipkit_client_batch_response_release(destroy_response);
    kmipkit_client_batch_item_response_release(get_item);
    kmipkit_client_batch_response_release(get_response);
    kmipkit_client_batch_item_response_release(create_item);
    kmipkit_client_batch_response_release(create_response);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
#undef REQUIRE_LIFECYCLE_C_API
    return result;
}
