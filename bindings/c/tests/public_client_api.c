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
