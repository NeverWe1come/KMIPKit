#include "kmipkit.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define T076_ERROR_INVALID_INPUT ((int32_t)4)

static int execute_with_options_returns(
    const kmipkit_request_options_v1_t *options, int32_t expected_status) {
    kmipkit_client_batch_response_t *response = 0;
    const int32_t status = kmipkit_client_execute_with_options_v1(
        0, 0, 0, options, &response);
    if (response != 0) {
        kmipkit_client_batch_response_release(response);
        return 0;
    }
    return status == expected_status;
}

static int api_level_query_matches_shared_fixture_locally(int argc, char **argv) {
    if (argc != 4 || argv == 0 || argv[1] == 0 || argv[2] == 0 || argv[3] == 0) {
        return 0;
    }
    const char *const expected[] = {argv[1], argv[2], argv[3]};
    kmipkit_byte_span_v1_t actual[3] = {{0}};
    uint8_t available = 0;
    static const uint8_t capability[] = "api_level_discovery";
    if (kmipkit_capability_is_available(capability, sizeof(capability) - 1, &available) != 0 ||
        available != 1) {
        return 0;
    }
    if (kmipkit_api_levels_get_v1(0, &actual[1], &actual[2]) !=
        T076_ERROR_INVALID_INPUT) {
        return 0;
    }
    if (kmipkit_api_levels_get_v1(&actual[0], &actual[1], &actual[2]) != 0) {
        return 0;
    }
    for (uint32_t index = 0; index < 3; ++index) {
        if (actual[index].data == 0 || actual[index].length != strlen(expected[index]) ||
            memcmp(actual[index].data, expected[index], actual[index].length) != 0) {
            return 0;
        }
    }
    return 1;
}

static int creation_create_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_create(UINT32_MAX, attributes, masks, &item) != 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int creation_create_key_pair_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *groups[6] = {0};
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    for (uint32_t index = 0; index < 6; ++index) {
        if (kmipkit_ttlv_structure_create(&groups[index]) != 0) {
            goto cleanup;
        }
    }
    if (kmipkit_client_batch_item_create_key_pair(
            groups[0], groups[1], groups[2], groups[3], groups[4], groups[5], &item) != 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    for (uint32_t index = 0; index < 6; ++index) {
        kmipkit_ttlv_structure_release(groups[index]);
    }
    return result;
}

static int creation_create_split_key_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -7;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_create_split_key(
            0xf0000000U, &identifier, -3, 4, 0xdeadbeefU, 0, 0, 0, attributes, masks, &item) != 0) {
        goto cleanup;
    }
    result = metadata.struct_size == sizeof(metadata) && metadata.abi_version == 2;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int split_key_creation_returns(
    const kmipkit_unique_identifier_v1_t *identifier,
    uint8_t has_prime_field_size,
    const uint8_t *prime_field_size_data,
    uint64_t prime_field_size_length,
    int32_t expected_status) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0) {
        goto cleanup;
    }
    const int32_t status = kmipkit_client_batch_item_create_split_key(
        1, identifier, 2, 1, 1, has_prime_field_size, prime_field_size_data,
        prime_field_size_length, attributes, 0, &item);
    if (status != expected_status ||
        (expected_status == 0 ? item == 0 : item != 0)) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int creation_rejects_malformed_split_key_inputs(void) {
    static const uint8_t invalid_utf8[] = {0xff};
    static const uint8_t valid_text[] = {'i', 'd', '-', 0xce, 0xb1};
    static const uint8_t optional_byte[] = {0x2a};
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;

    identifier.struct_size -= 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 2;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.abi_version = 1;
    identifier.reserved = 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.reserved = 0;
    identifier.kind = UINT32_MAX;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = invalid_utf8;
    identifier.text_length = sizeof(invalid_utf8);
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.text_data = 0;
    identifier.text_length = 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.text_data = valid_text;
    identifier.text_length = sizeof(valid_text);
    if (!split_key_creation_returns(&identifier, 0, 0, 0, 0)) {
        return 0;
    }
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
    identifier.enumeration_value = UINT32_MAX;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, 0)) {
        return 0;
    }
    if (!split_key_creation_returns(0, 0, optional_byte, 0, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 0, 0, 1, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 1, 0, 1, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 2, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    return 1;
}

static int creation_rejects_missing_required_attributes(void) {
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_create(UINT32_MAX, 0, 0, &item) !=
            T076_ERROR_INVALID_INPUT ||
        item != 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    if (kmipkit_client_batch_item_create_split_key(
            1, 0, 2, 1, 1, 0, 0, 0, 0, 0, &item) != T076_ERROR_INVALID_INPUT ||
        item != 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    return 1;
}

static int execute_public_tls_request(char **argv, kmipkit_codec_limits_t *limits) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint32_t expected_statuses[] = {0, 2, 1, 0, 0, 1, 0, 0};
    static const uint8_t expected_correlation[] = {0x00, 0xa1, 0x7f};
    static const uint8_t expected_ids[8][6] = {
        {'i', 't', 'e', 'm', '-', '0'},
        {'i', 't', 'e', 'm', '-', '1'},
        {'i', 't', 'e', 'm', '-', '2'},
        {'i', 't', 'e', 'm', '-', '3'},
        {'i', 't', 'e', 'm', '-', '4'},
        {'i', 't', 'e', 'm', '-', '5'},
        {'i', 't', 'e', 'm', '-', '6'},
        {'i', 't', 'e', 'm', '-', '7'},
    };
    static const uint8_t expected_failure_message[] = {
        'c', 'r', 0xc3, 0xa9, 'a', 't', 'i', 'o', 'n', ' ', 'r', 'e', 'f', 'u', 's', 0xc3,
        0xa9, 'e', ' ', 0xe2, 0x9c, 0x93};
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_extension_definition_t **definitions = 0;
    kmipkit_client_extension_registry_t *registry = 0;
    kmipkit_client_configuration_t *configuration = 0;
    kmipkit_transport_config_t *transport = 0;
    kmipkit_client_t *client = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *items[8] = {0};
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    uint64_t response_count = 0;
    char *port_end = 0;
    const unsigned long port = strtoul(argv[5], &port_end, 10);
    int result = 1;

    if (port_end == argv[5] || *port_end != '\0' || port == 0 || port > 65535UL) {
        return 2;
    }
    client_certificates[0].source_kind = 2; /* file */
    client_certificates[0].encoding = 2; /* DER */
    client_certificates[0].data.data = (const uint8_t *)argv[1];
    client_certificates[0].data.length = (uint64_t)strlen(argv[1]);
    client_certificates[1] = client_certificates[0];
    client_certificates[1].data.data = (const uint8_t *)argv[2];
    client_certificates[1].data.length = (uint64_t)strlen(argv[2]);
    trust_certificates[0].source_kind = 2;
    trust_certificates[0].encoding = 2;
    trust_certificates[0].data.data = (const uint8_t *)argv[4];
    trust_certificates[0].data.length = (uint64_t)strlen(argv[4]);

    transport_input.struct_size = (uint32_t)sizeof(transport_input);
    transport_input.abi_version = 1;
    transport_input.endpoint_kind = 1; /* raw TLS */
    transport_input.trust_kind = 2; /* explicit CA list */
    transport_input.flags = 1; /* explicit TLS server name */
    transport_input.endpoint.data = host;
    transport_input.endpoint.length = (uint64_t)(sizeof(host) - 1);
    transport_input.endpoint_port = (uint16_t)port;
    transport_input.tls_server_name.data = server_name;
    transport_input.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    transport_input.client_certificates = client_certificates;
    transport_input.client_certificate_count = 2;
    transport_input.client_private_key.source_kind = 2;
    transport_input.client_private_key.encoding = 2;
    transport_input.client_private_key.data.data = (const uint8_t *)argv[3];
    transport_input.client_private_key.data.length = (uint64_t)strlen(argv[3]);
    transport_input.trust_certificates = trust_certificates;
    transport_input.trust_certificate_count = 1;
    transport_input.max_request_bytes = UINT64_C(1048576);

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = 1;
    if (kmipkit_extension_registry_limits_v1_default_values(&registry_limits) != 0 ||
        kmipkit_client_extension_registry_create_v1(
            definitions, 0, &registry_limits, &registry) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_configuration_create(registry, &configuration) != 0) {
        goto cleanup;
    }
    registry = 0; /* consumed by client configuration */
    if (kmipkit_transport_config_create_v1(&transport_input, &transport) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_create(configuration, &transport, &client) != 0) {
        goto cleanup;
    }
    transport = 0; /* consumed by client */
    if (kmipkit_client_batch_create(&batch) != 0) {
        result = 30;
        goto cleanup;
    }
    if (kmipkit_client_batch_set_asynchronous_indicator(batch, 2) != 0) {
        result = 31;
        goto cleanup;
    }
    for (uint64_t index = 0; index < 8; ++index) {
        kmipkit_client_batch_item_t *identified_item = 0;
        if (index == 4 || index == 5) {
            kmipkit_ttlv_structure_t *attributes = 0;
            kmipkit_ttlv_structure_t *masks = 0;
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_ttlv_structure_create(&masks) != 0 ||
                kmipkit_client_batch_item_create(UINT32_MAX, attributes, masks, &items[index]) != 0) {
                kmipkit_ttlv_structure_release(masks);
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(masks);
            kmipkit_ttlv_structure_release(attributes);
        } else if (index == 6) {
            if (kmipkit_client_batch_item_create_key_pair(0, 0, 0, 0, 0, 0,
                    &items[index]) != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 7) {
            kmipkit_ttlv_structure_t *attributes = 0;
            kmipkit_ttlv_structure_t *masks = 0;
            static const uint8_t prime_field_size[] = {0, 0, 0, 0, 0, 0, 0, 17};
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_ttlv_structure_create(&masks) != 0 ||
                kmipkit_client_batch_item_create_split_key(
                    0xf0000000U, 0, -3, 4, 3, 1, prime_field_size, sizeof(prime_field_size),
                    attributes, masks, &items[index]) != 0 || items[index] == 0) {
                kmipkit_ttlv_structure_release(masks);
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(masks);
            kmipkit_ttlv_structure_release(attributes);
        } else if (kmipkit_client_batch_item_discover_versions(&items[index]) != 0 ||
                   items[index] == 0) {
            result = 40 + (int)index;
            goto cleanup;
        }
        if (kmipkit_client_batch_item_with_unique_batch_item_id(
                items[index], expected_ids[index], sizeof(expected_ids[index]),
                &identified_item) != 0 || identified_item == 0) {
            result = 60 + (int)index;
            goto cleanup;
        }
        items[index] = identified_item;
        if (kmipkit_client_batch_add_item(batch, &items[index]) != 0) {
            result = 50 + (int)index;
            goto cleanup;
        }
    }
    if (kmipkit_client_execute((kmipkit_client_t *)limits, &batch, limits, &response)
            != T076_ERROR_INVALID_INPUT ||
        batch == 0 || response != 0) {
        goto cleanup;
    }
    if (kmipkit_client_execute(client, &batch, (kmipkit_codec_limits_t *)client, &response)
            != T076_ERROR_INVALID_INPUT ||
        batch == 0 || response != 0) {
        goto cleanup;
    }
    const int32_t execute_status = kmipkit_client_execute(client, &batch, limits, &response);
    if (execute_status != 0) {
        result = 100 + (int)execute_status;
        goto cleanup;
    }
    batch = 0; /* consumed by execution */
    if (kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != 8) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < response_count; ++index) {
        kmipkit_client_batch_item_metadata_v1_t metadata = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        kmipkit_byte_span_v1_t unique_id = {0};
        uint8_t correlation_present = 0;
        uint8_t unique_id_present = 0;
        uint8_t versions_present = 0;
        uint64_t version_count = 0;
        kmipkit_protocol_version_v1_t versions[2] = {{0}};
        const uint32_t expected_outcome = index == 1 ? 2U : 1U;
        if (kmipkit_client_batch_response_item_at(response, index, &response_item) != 0 ||
            response_item == 0) {
            goto cleanup;
        }
        metadata.struct_size = (uint32_t)sizeof(metadata);
        metadata.abi_version = 1;
        if (kmipkit_client_batch_item_response_metadata_v1(response_item, &metadata) != 0 ||
            metadata.abi_version != 1 || metadata.outcome_kind != expected_outcome ||
            metadata.result_status != expected_statuses[index]) {
            goto cleanup;
        }
        if (index == 0) {
            kmipkit_client_batch_item_metadata_v2_t invalid_metadata = {0};
            uint8_t present = 0;
            uint64_t count = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(response_item, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            invalid_metadata.struct_size = (uint32_t)sizeof(invalid_metadata) - 1;
            invalid_metadata.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &invalid_metadata) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            invalid_metadata.struct_size = (uint32_t)sizeof(invalid_metadata);
            invalid_metadata.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &invalid_metadata) != T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &identifier) != T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &present, 0, 1, &count) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
        }
        if (index == 1) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 30 ||
                metadata_v2.outcome_kind != 2) {
                goto cleanup;
            }
        }
        if (index == 4) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t create_present = 0;
            uint32_t object_type = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 2;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 ||
                metadata_v2.operation != 1 || metadata_v2.outcome_kind != 1 ||
                metadata_v2.result_status != 0 || metadata_v2.has_result_reason != 0 ||
                metadata_v2.has_result_message != 0 ||
                metadata_v2.result_message_length != 0 || metadata_v2.result_message_data != 0 ||
                kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) != 0 ||
                create_present != 1 || object_type != 7 ||
                identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                identifier.integer_value != -8) {
                goto cleanup;
            }
        }
        if (index == 5) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t create_present = 1;
            uint32_t object_type = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 ||
                metadata_v2.operation != 1 || metadata_v2.outcome_kind != 1 ||
                metadata_v2.result_status != 1 || metadata_v2.has_result_reason != 1 ||
                metadata_v2.result_reason != 5 || metadata_v2.has_result_message != 1 ||
                metadata_v2.result_message_length != sizeof(expected_failure_message) ||
                metadata_v2.result_message_data == 0 ||
                memcmp(metadata_v2.result_message_data, expected_failure_message,
                    sizeof(expected_failure_message)) != 0 ||
                kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) != 0 ||
                create_present != 0) {
                goto cleanup;
            }
        }
        if (index == 6) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t pair_present = 0;
            kmipkit_unique_identifier_v1_t private_identifier = {0};
            kmipkit_unique_identifier_v1_t public_identifier = {0};
            static const uint8_t expected_public_identifier[] = {
                'p', 'u', 'b', 'l', 'i', 'c', '-', 'k', 'e', 'y', '-',
                0xce, 0xb1};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
            private_identifier.abi_version = 1;
            public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
            public_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, 0, &public_identifier) !=
                    T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            private_identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
            public_identifier.abi_version = 2;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            public_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 2 ||
                kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) != 0 ||
                pair_present != 1 ||
                private_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION ||
                private_identifier.enumeration_value != UINT32_MAX ||
                public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
                public_identifier.text_length != sizeof(expected_public_identifier) ||
                public_identifier.text_data == 0 ||
                memcmp(public_identifier.text_data, expected_public_identifier,
                    sizeof(expected_public_identifier)) != 0) {
                goto cleanup;
            }
        }
        if (index == 7) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t split_present = 0;
            uint64_t split_count = 0;
            kmipkit_unique_identifier_v1_t first_identifier = {0};
            kmipkit_unique_identifier_v1_t middle_identifier = {0};
            kmipkit_unique_identifier_v1_t last_identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 40 ||
                kmipkit_client_batch_item_response_create_split_key_count(
                    response_item, &split_present, &split_count) != 0 ||
                split_present != 1 || split_count != UINT64_C(100001)) {
                goto cleanup;
            }
            first_identifier.struct_size = (uint32_t)sizeof(first_identifier);
            first_identifier.abi_version = 1;
            middle_identifier.struct_size = (uint32_t)sizeof(middle_identifier);
            middle_identifier.abi_version = 1;
            last_identifier.struct_size = (uint32_t)sizeof(last_identifier);
            last_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, 0) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            first_identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &first_identifier) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            first_identifier.struct_size = (uint32_t)sizeof(first_identifier);
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, split_count, &first_identifier) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &first_identifier) != 0 ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, UINT64_C(50000), &middle_identifier) != 0 ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, split_count - 1, &last_identifier) != 0 ||
                first_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                first_identifier.integer_value != 0 ||
                middle_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                middle_identifier.integer_value != 50000 ||
                last_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                last_identifier.integer_value != 100000) {
                goto cleanup;
            }
        }
        if (kmipkit_client_batch_item_response_unique_batch_item_id(
                response_item, &unique_id_present, &unique_id) != 0 ||
            unique_id_present != 1 || unique_id.length != sizeof(expected_ids[index]) ||
            unique_id.data == 0 ||
            memcmp(unique_id.data, expected_ids[index], sizeof(expected_ids[index])) != 0) {
            goto cleanup;
        }
        if (index == 2) {
            if (metadata.has_result_reason != 1 || metadata.result_reason != 1) {
                goto cleanup;
            }
        } else if (index == 5) {
            if (metadata.has_result_reason != 1 || metadata.result_reason != 5) {
                goto cleanup;
            }
        } else if (metadata.has_result_reason != 0) {
            goto cleanup;
        }
        if (kmipkit_client_batch_item_response_pending_correlation_value(
                response_item, &correlation_present, &correlation) != 0) {
            goto cleanup;
        }
        if (index == 1) {
            if (correlation_present != 1 || correlation.length != sizeof(expected_correlation) ||
                correlation.data == 0 ||
                memcmp(correlation.data, expected_correlation, sizeof(expected_correlation)) != 0) {
                goto cleanup;
            }
        } else if (correlation_present != 0 || correlation.data != 0 || correlation.length != 0) {
            goto cleanup;
        }
        if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                response_item, &versions_present, 0, 0, &version_count) != 0) {
            goto cleanup;
        }
        if (index == 0) {
            if (versions_present != 1 || version_count != 2) {
                goto cleanup;
            }
            uint64_t copied_count = 0;
            uint8_t insufficient_present = 0;
            uint64_t insufficient_count = 0;
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &insufficient_present, versions, 1, &insufficient_count) !=
                    T076_ERROR_INVALID_INPUT ||
                insufficient_present != 1 || insufficient_count != 2) {
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &versions_present, versions, 2, &copied_count) != 0 ||
                versions_present != 1 || copied_count != 2 ||
                versions[0].major != 2 || versions[0].minor != 1 ||
                versions[1].major != 2 || versions[1].minor != 1) {
                goto cleanup;
            }
        } else if (index == 3) {
            if (versions_present != 1 || version_count != 0) {
                goto cleanup;
            }
        } else if (versions_present != 0 || version_count != 0) {
            goto cleanup;
        }
        kmipkit_client_batch_item_response_release(response_item);
        response_item = 0;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    for (uint64_t index = 0; index < 8; ++index) {
        kmipkit_client_batch_item_release(items[index]);
    }
    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    return result;
}

int kmipkit_client_abi_c_consumer_execute_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    if (argc != 6 || limits == 0) {
        return 22;
    }
    return execute_public_tls_request(argv, limits);
}

int main(int argc, char **argv) {
    kmipkit_client_t *client = 0;
    if (!api_level_query_matches_shared_fixture_locally(argc, argv)) {
        return 100;
    }
    if (kmipkit_client_create(0, 0, &client) != T076_ERROR_INVALID_INPUT) {
        return 1;
    }
    if (client != 0) {
        return 2;
    }
    if (!creation_create_matches_shared_fixture() ||
        !creation_create_key_pair_matches_shared_fixture() ||
        !creation_create_split_key_matches_shared_fixture() ||
        !creation_rejects_malformed_split_key_inputs() ||
        !creation_rejects_missing_required_attributes()) {
        return 7;
    }

    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_transport_config_t *transport = 0;
    if (kmipkit_transport_config_create_v1(&transport_input, &transport)
            != T076_ERROR_INVALID_INPUT) {
        return 3;
    }
    if (transport != 0) {
        return 4;
    }

    kmipkit_request_options_v1_t request_options = {0};
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = 1;
    kmipkit_client_batch_response_t *response = 0;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 5;
    }
    if (!execute_with_options_returns(0, T076_ERROR_INVALID_INPUT)) {
        return 6;
    }
    request_options.connect_timeout.kind = 1;
    request_options.connect_timeout.seconds = 1;
    request_options.connect_timeout.nanoseconds = 999999999;
    request_options.write_timeout.kind = 2;
    request_options.read_timeout.kind = 1;
    request_options.read_timeout.nanoseconds = 0;
    request_options.total_timeout.kind = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 7;
    }
    request_options = (kmipkit_request_options_v1_t){0};
    request_options.struct_size = (uint32_t)sizeof(request_options) - 1;
    request_options.abi_version = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 8;
    }
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = 2;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 9;
    }
    request_options.abi_version = 1;
    request_options.connect_timeout.reserved = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 10;
    }
    request_options.connect_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.write_timeout.reserved2 = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 11;
    }
    request_options.write_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.read_timeout.seconds = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 12;
    }
    request_options.read_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.total_timeout.kind = 1;
    request_options.total_timeout.nanoseconds = 1000000000;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 13;
    }
    request_options.total_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.connect_timeout.kind = 2;
    request_options.connect_timeout.seconds = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 14;
    }
    request_options.connect_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.write_timeout.kind = 99;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 15;
    }
    if (kmipkit_client_execute(0, 0, 0, &response) != T076_ERROR_INVALID_INPUT) {
        return 16;
    }
    kmipkit_client_batch_t *empty_batch = 0;
    if (kmipkit_client_execute(0, &empty_batch, 0, &response) != T076_ERROR_INVALID_INPUT) {
        return 17;
    }

    kmipkit_client_batch_t *batch = 0;
    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0) {
        return 18;
    }
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_discover_versions(&item) != 0 || item == 0) {
        kmipkit_client_batch_release(batch);
        return 19;
    }
    if (kmipkit_client_batch_add_item(batch, &item) != 0 || item != 0) {
        kmipkit_client_batch_item_release(item);
        kmipkit_client_batch_release(batch);
        return 20;
    }
    kmipkit_client_batch_release(batch);
    if (argc != 4) {
        return 21;
    }
    return 0;
}
