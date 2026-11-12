/* Public C consumer contract for KMIPKIT-0093-FR-012 / KMIPKIT-0066 REQ-066-009. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>

#include <kmipkit.h>

typedef void (*kmipkit_start_test_client_fn)(uint16_t, void *);

static void kmipkit_check_test_secret(const uint8_t *bytes, uint64_t length, void *context) {
    uint32_t *valid = (uint32_t *)context;
    static const uint8_t expected[] = { 0x11u, 0x22u, 0x33u, 0x44u };
    *valid = (length == sizeof(expected) && memcmp(bytes, expected, sizeof(expected)) == 0) ? 1u : 0u;
}

static void kmipkit_check_test_secret_text(const uint8_t *bytes, uint64_t length, void *context) {
    uint32_t *valid = (uint32_t *)context;
    static const uint8_t expected[] = "s3cr3t";
    *valid = (length == sizeof(expected) - 1u &&
        memcmp(bytes, expected, sizeof(expected) - 1u) == 0) ? 1u : 0u;
}

_Static_assert(
    _Generic((kmipkit_server_request_item_t *)0, kmipkit_client_batch_item_t *: 1, default: 0) == 0,
    "incoming server request handles must remain distinct from client send items");

/* Drives the complete C listener route against a real TLS client started by the Rust harness. */
int32_t kmipkit_s2c_consumer_receives_put_and_sends_response(
    const uint8_t *leaf, uint64_t leaf_length,
    const uint8_t *root, uint64_t root_length,
    const uint8_t *key, uint64_t key_length,
    kmipkit_start_test_client_fn start_client, void *context) {
    uint16_t port = 0u;
    uint32_t operation = 0u, put_function = 0u, object_type = 0u, object_tag = 0u;
    uint32_t key_format = 0u, secret_valid = 0u, secret_text_valid = 0u;
    uint64_t batch_count = 0u;
    uint8_t present = 0u, item_type = 0u;
    kmipkit_byte_span_v1_t identifier = { NULL, 0u };
    kmipkit_server_put_object_t *object = NULL, *key_block = NULL, *key_value = NULL;
    kmipkit_server_put_object_t *password_object = NULL, *unknown_object = NULL;
    kmipkit_attribute_set_t *notify_attributes = NULL;
    kmipkit_secret_bytes_t *secret = NULL;
    kmipkit_secret_text_t *secret_text = NULL;
    kmipkit_server_listener_config_t *configuration = NULL;
    kmipkit_server_listener_t *listener = NULL;
    kmipkit_server_connection_t *connection = NULL;
    kmipkit_server_incoming_batch_t *batch = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_server_request_item_t *request = NULL, *password_request = NULL, *unknown_request = NULL;
    kmipkit_server_request_item_t *discover_request = NULL, *notify_request = NULL;
    kmipkit_server_request_item_t *query_request = NULL, *role_request = NULL;
    kmipkit_server_response_builder_t *builder = NULL;
    kmipkit_server_response_item_t *response_item = NULL;
    kmipkit_server_response_t *response = NULL;
    kmipkit_protocol_version_v1_t incoming_version = { 0, 0 };
    kmipkit_protocol_version_v1_t accepted_versions[1] = { { 2, 1 } };
    uint32_t query_function = 0u;
    uint32_t supported_operations[1] = { 0x0000000Fu };
    uint32_t supported_objects[1] = { 2u };
    uint64_t deleted_reference_count = 0u, changed_attribute_count = 0u;
    kmipkit_attribute_reference_v1_t deleted_reference = {
        sizeof(kmipkit_attribute_reference_v1_t), 1u, 0u, 0u, 0u, 0u,
        NULL, 0u, NULL, 0u,
    };
    static const uint8_t vendor_name[] = "KMIPKit C test";
    static const uint8_t host[] = "127.0.0.1";
    kmipkit_transport_input_v1_t chain[2] = {
        { 1u, 2u, { leaf, leaf_length } },
        { 1u, 2u, { root, root_length } },
    };
    kmipkit_transport_input_v1_t authority = { 1u, 2u, { root, root_length } };
    kmipkit_server_listener_config_v1_t input = {
        sizeof(kmipkit_server_listener_config_v1_t), 1u,
        KMIPKIT_SERVER_LISTENER_RAW_TLS, KMIPKIT_SERVER_ENCODING_TTLV, 0u,
        { host, (uint64_t)(sizeof(host) - 1u) }, 0u, 0u, chain, 2u,
        { 1u, 2u, { key, key_length } }, &authority, 1u, NULL, 0u,
        { NULL, 0u }, 0u,
    };
    int32_t status = kmipkit_server_listener_config_create_v1(&input, &configuration);
    if (status != 0) goto cleanup;
    status = kmipkit_server_listener_bind_raw_tls(configuration, &listener);
    if (status != 0) goto cleanup;
    status = kmipkit_server_listener_port(listener, &port);
    if (status != 0 || port == 0u || start_client == NULL) goto cleanup;
    start_client(port, context);
    status = kmipkit_server_listener_accept(listener, 5u, 0u, &connection);
    if (status != 0) goto cleanup;
    status = kmipkit_codec_limits_defaults(&limits);
    if (status != 0) goto cleanup;
    status = kmipkit_server_connection_recv_batch(connection, limits, 5u, 0u, &batch);
    if (status != 0 || batch == NULL) goto cleanup;
    status = kmipkit_server_incoming_batch_count(batch, &batch_count);
    if (status != 0 || batch_count != 7u) { status = 101; goto cleanup; }
    status = kmipkit_server_incoming_batch_protocol_version(batch, &incoming_version);
    if (status != 0 || incoming_version.major != 2 || incoming_version.minor != 1) {
        status = 135; goto cleanup;
    }
    status = kmipkit_server_incoming_batch_item_at(batch, 0u, &request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(request, &operation);
    if (status != 0 || operation != 0x0000001Cu) { status = 102; goto cleanup; }
    status = kmipkit_server_request_item_put_function(request, &put_function);
    if (status != 0 || put_function != 1u) { status = 103; goto cleanup; }
    status = kmipkit_server_request_item_put_object(request, &object);
    if (status != 0 || object == NULL) goto cleanup;
    status = kmipkit_server_put_object_type(object, &object_type);
    if (status != 0 || object_type != 2u) { status = 104; goto cleanup; }
    status = kmipkit_server_put_object_tag(object, &object_tag);
    if (status != 0 || object_tag != 0x0042008Fu) { status = 117; goto cleanup; }
    status = kmipkit_server_put_object_field_read(object, 0x00420040u, KMIPKIT_SERVER_PUT_READ_STRUCTURE_VIEW,
        &present, &item_type, NULL, NULL, &key_block, NULL, NULL);
    if (status != 0 || present != 1u || key_block == NULL || item_type != 1u) { status = 105; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_block, 0x00420042u, KMIPKIT_SERVER_PUT_READ_ENUMERATION,
        &present, &item_type, &key_format, NULL, NULL, NULL, NULL);
    if (status != 0 || present != 1u || key_format != 1u) { status = 106; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_block, 0x00420045u, KMIPKIT_SERVER_PUT_READ_STRUCTURE_VIEW,
        &present, &item_type, NULL, NULL, &key_value, NULL, NULL);
    if (status != 0 || present != 1u || key_value == NULL || item_type != 1u) { status = 107; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_value, 0x00420043u, KMIPKIT_SERVER_PUT_READ_SECRET_BYTES,
        &present, &item_type, NULL, NULL, NULL, &secret, NULL);
    if (status != 0 || present != 1u || secret == NULL || item_type != 8u) { status = 108; goto cleanup; }
    kmipkit_server_put_object_release(key_value);
    key_value = NULL;
    kmipkit_server_put_object_release(key_block);
    key_block = NULL;
    kmipkit_server_put_object_release(object);
    object = NULL;
    if (kmipkit_secret_bytes_with_bytes(secret, NULL, NULL) != KMIPKIT_ERROR_INVALID_INPUT) {
        status = 109; goto cleanup;
    }
    status = kmipkit_secret_bytes_with_bytes(secret, kmipkit_check_test_secret, &secret_valid);
    if (status != 0 || secret_valid != 1u) { status = 110; goto cleanup; }
    status = kmipkit_server_incoming_batch_item_at(batch, 1u, &password_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(password_request, &operation);
    if (status != 0 || operation != 0x0000001Cu) { status = 112; goto cleanup; }
    status = kmipkit_server_request_item_put_object(password_request, &password_object);
    if (status != 0 || password_object == NULL) goto cleanup;
    status = kmipkit_server_put_object_type(password_object, &object_type);
    if (status != 0 || object_type != 13u) { status = 113; goto cleanup; }
    status = kmipkit_server_put_object_field_read(password_object, 0x004200A1u, KMIPKIT_SERVER_PUT_READ_SECRET_TEXT,
        &present, &item_type, NULL, NULL, NULL, NULL, &secret_text);
    if (status != 0 || present != 1u || secret_text == NULL || item_type != 7u) { status = 114; goto cleanup; }
    kmipkit_server_put_object_release(password_object);
    password_object = NULL;
    if (kmipkit_secret_text_with_str(secret_text, NULL, NULL) != KMIPKIT_ERROR_INVALID_INPUT) {
        status = 115; goto cleanup;
    }
    status = kmipkit_secret_text_with_str(secret_text, kmipkit_check_test_secret_text, &secret_text_valid);
    if (status != 0 || secret_text_valid != 1u) { status = 116; goto cleanup; }
    status = kmipkit_server_incoming_batch_item_at(batch, 2u, &unknown_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_put_object(unknown_request, &unknown_object);
    if (status != 0 || unknown_object == NULL) goto cleanup;
    status = kmipkit_server_put_object_type(unknown_object, &object_type);
    if (status != 0 || object_type != 0u) { status = 118; goto cleanup; }
    status = kmipkit_server_put_object_tag(unknown_object, &object_tag);
    if (status != 0 || object_tag != 0x00540001u) { status = 119; goto cleanup; }
    status = kmipkit_server_put_object_field_read(unknown_object, 0x00540002u, KMIPKIT_SERVER_PUT_READ_ENUMERATION,
        &present, &item_type, &key_format, NULL, NULL, NULL, NULL);
    if (status != 0 || present != 1u || key_format != 0xA1B2C3D4u) { status = 120; goto cleanup; }
    status = kmipkit_server_incoming_batch_item_at(batch, 3u, &discover_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(discover_request, &operation);
    if (status != 0 || operation != 0x0000001Eu) { status = 121; goto cleanup; }
    status = kmipkit_server_request_item_discover_versions_count(discover_request, &batch_count);
    if (status != 0 || batch_count != 1u) { status = 122; goto cleanup; }
    status = kmipkit_server_request_item_discover_version_at(discover_request, 0u, &incoming_version);
    if (status != 0 || incoming_version.major != 2 || incoming_version.minor != 1) { status = 123; goto cleanup; }
    status = kmipkit_server_incoming_batch_item_at(batch, 4u, &notify_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(notify_request, &operation);
    if (status != 0 || operation != 0x0000001Bu) { status = 124; goto cleanup; }
    status = kmipkit_server_request_item_unique_identifier(notify_request, &identifier);
    if (status != 0 || identifier.length != 13u || memcmp(identifier.data, "notify-object", 13u) != 0) { status = 125; goto cleanup; }
    status = kmipkit_server_request_item_notify_attributes(notify_request, &present, &notify_attributes);
    if (status != 0 || present != 1u || notify_attributes == NULL) { status = 131; goto cleanup; }
    status = kmipkit_attribute_set_len(notify_attributes, &changed_attribute_count);
    if (status != 0 || changed_attribute_count != 1u) { status = 132; goto cleanup; }
    status = kmipkit_server_request_item_notify_deleted_attribute_reference_count(
        notify_request, &deleted_reference_count);
    if (status != 0 || deleted_reference_count != 1u) { status = 133; goto cleanup; }
    status = kmipkit_server_request_item_notify_deleted_attribute_reference_at(
        notify_request, 0u, &deleted_reference);
    if (status != 0 || deleted_reference.kind != 1u || deleted_reference.tag_value != 0x00420028u) {
        status = 134; goto cleanup;
    }
    status = kmipkit_server_incoming_batch_item_at(batch, 5u, &query_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(query_request, &operation);
    if (status != 0 || operation != 0x00000018u) { status = 126; goto cleanup; }
    status = kmipkit_server_request_item_query_functions_count(query_request, &batch_count);
    if (status != 0 || batch_count != 1u) { status = 127; goto cleanup; }
    status = kmipkit_server_request_item_query_function_at(query_request, 0u, &query_function);
    if (status != 0 || query_function != 1u) { status = 128; goto cleanup; }
    status = kmipkit_server_incoming_batch_item_at(batch, 6u, &role_request);
    if (status != 0) goto cleanup;
    status = kmipkit_server_request_item_operation(role_request, &operation);
    if (status != 0 || operation != 0x00000032u) { status = 129; goto cleanup; }
    status = kmipkit_server_request_item_endpoint_role(role_request, &operation);
    if (status != 0 || operation != 1u) { status = 130; goto cleanup; }
    status = kmipkit_server_response_builder_create(batch, &builder);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_failure(builder, request, 0u, 1u, 1u,
        (kmipkit_byte_span_v1_t){ NULL, 0u });
    if (status != KMIPKIT_ERROR_INVALID_INPUT) { status = 111; goto cleanup; }
    status = kmipkit_server_put_response_create(request, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, request, response_item);
    if (status != 0) goto cleanup;
    kmipkit_server_response_item_release(response_item);
    response_item = NULL;
    status = kmipkit_server_put_response_create(password_request, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, password_request, response_item);
    if (status != 0) goto cleanup;
    kmipkit_server_response_item_release(response_item);
    response_item = NULL;
    status = kmipkit_server_response_builder_add_failure(
        builder, unknown_request,
        KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_FAILED,
        1u, KMIPKIT_ENUMERATION_RESULT_REASON_GENERAL_FAILURE,
        (kmipkit_byte_span_v1_t){ (const uint8_t *)"unsupported object type", 23u });
    if (status != 0) goto cleanup;
    status = kmipkit_server_discover_versions_response_create(
        discover_request, accepted_versions, 1u, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, discover_request, response_item);
    if (status != 0) goto cleanup;
    kmipkit_server_response_item_release(response_item);
    response_item = NULL;
    status = kmipkit_server_notify_response_create(notify_request, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, notify_request, response_item);
    if (status != 0) goto cleanup;
    kmipkit_server_response_item_release(response_item);
    response_item = NULL;
    status = kmipkit_server_query_response_create(
        query_request, supported_operations, 1u, supported_objects, 1u, NULL, 0u,
        (kmipkit_byte_span_v1_t){ vendor_name, sizeof(vendor_name) - 1u }, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, query_request, response_item);
    if (status != 0) goto cleanup;
    kmipkit_server_response_item_release(response_item);
    response_item = NULL;
    status = kmipkit_server_set_endpoint_role_response_create(
        role_request, 2u, (kmipkit_byte_span_v1_t){ NULL, 0u }, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, role_request, response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_build(builder, &response);
    if (status != 0) goto cleanup;
    status = kmipkit_server_connection_send_response(connection, response, 5u, 0u);

cleanup:
    if (secret != NULL) kmipkit_secret_bytes_release(secret);
    if (secret_text != NULL) kmipkit_secret_text_release(secret_text);
    if (notify_attributes != NULL) kmipkit_attribute_set_release(notify_attributes);
    if (key_value != NULL) kmipkit_server_put_object_release(key_value);
    if (key_block != NULL) kmipkit_server_put_object_release(key_block);
    if (object != NULL) kmipkit_server_put_object_release(object);
    if (password_object != NULL) kmipkit_server_put_object_release(password_object);
    if (unknown_object != NULL) kmipkit_server_put_object_release(unknown_object);
    if (response != NULL) kmipkit_server_response_release(response);
    if (response_item != NULL) kmipkit_server_response_item_release(response_item);
    if (builder != NULL) kmipkit_server_response_builder_release(builder);
    if (request != NULL) kmipkit_server_request_item_release(request);
    if (password_request != NULL) kmipkit_server_request_item_release(password_request);
    if (unknown_request != NULL) kmipkit_server_request_item_release(unknown_request);
    if (discover_request != NULL) kmipkit_server_request_item_release(discover_request);
    if (notify_request != NULL) kmipkit_server_request_item_release(notify_request);
    if (query_request != NULL) kmipkit_server_request_item_release(query_request);
    if (role_request != NULL) kmipkit_server_request_item_release(role_request);
    if (batch != NULL) kmipkit_server_incoming_batch_release(batch);
    if (limits != NULL) kmipkit_codec_limits_release(limits);
    if (connection != NULL) kmipkit_server_connection_release(connection);
    if (listener != NULL) kmipkit_server_listener_release(listener);
    if (configuration != NULL) kmipkit_server_listener_config_release(configuration);
    return status;
}

/* Null and wrong-kind inputs fail closed and initialize every writable output. */
int32_t kmipkit_s2c_consumer_rejects_invalid_handles(void) {
    kmipkit_server_listener_config_t *configuration = (void *)1;
    kmipkit_server_listener_t *listener = (void *)1;
    kmipkit_server_connection_t *connection = (void *)1;
    kmipkit_server_incoming_batch_t *batch = (void *)1;
    kmipkit_server_request_item_t *item = (void *)1;
    kmipkit_server_response_builder_t *builder = (void *)1;
    kmipkit_server_response_item_t *response_item = (void *)1;
    kmipkit_server_response_t *response = (void *)1;
    kmipkit_server_put_object_t *put_object = (void *)1;
    kmipkit_attribute_set_t *attributes = (void *)1;
    kmipkit_secret_bytes_t *secret = (void *)1;
    kmipkit_secret_text_t *secret_text = (void *)1;
    uint32_t operation = UINT32_MAX;
    uint32_t object_type = UINT32_MAX;
    uint8_t present = UINT8_MAX;
    uint64_t count = UINT64_MAX;
    kmipkit_byte_span_v1_t identifier = { (const uint8_t *)1, UINT64_MAX };
    kmipkit_attribute_reference_v1_t deleted_reference = {
        sizeof(kmipkit_attribute_reference_v1_t), 1u, 0u, 0u, 0u, 0u,
        NULL, 0u, NULL, 0u,
    };

    if (kmipkit_server_listener_config_create_v1(NULL, &configuration) !=
            KMIPKIT_ERROR_INVALID_INPUT || configuration != NULL) {
        return 1;
    }
    if (kmipkit_server_listener_bind_raw_tls(NULL, &listener) !=
            KMIPKIT_ERROR_INVALID_INPUT || listener != NULL) {
        return 2;
    }
    if (kmipkit_server_listener_accept(NULL, 0, 0, &connection) !=
            KMIPKIT_ERROR_INVALID_INPUT || connection != NULL) {
        return 3;
    }
    if (kmipkit_server_connection_recv_batch(NULL, NULL, 0, 0, &batch) !=
            KMIPKIT_ERROR_INVALID_INPUT || batch != NULL) {
        return 4;
    }
    if (kmipkit_server_incoming_batch_item_at(NULL, 0, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != NULL) {
        return 5;
    }
    if (kmipkit_server_request_item_operation(NULL, &operation) !=
            KMIPKIT_ERROR_INVALID_INPUT || operation != 0) {
        return 6;
    }
    if (kmipkit_server_request_item_put_object(NULL, &put_object) !=
            KMIPKIT_ERROR_INVALID_INPUT || put_object != NULL) {
        return 24;
    }
    if (kmipkit_server_put_object_type(NULL, &object_type) !=
            KMIPKIT_ERROR_INVALID_INPUT || object_type != 0u) {
        return 25;
    }
    if (kmipkit_server_put_object_tag(NULL, &object_type) !=
            KMIPKIT_ERROR_INVALID_INPUT || object_type != 0u) {
        return 27;
    }
    if (kmipkit_server_put_object_field_read(NULL, 0x00420043u, 5u,
            &present, NULL, NULL, NULL, NULL, &secret, &secret_text) !=
            KMIPKIT_ERROR_INVALID_INPUT || present != 0u || secret != NULL || secret_text != NULL) {
        return 26;
    }
    if (kmipkit_server_request_item_unique_identifier(NULL, &identifier) !=
            KMIPKIT_ERROR_INVALID_INPUT || identifier.data != NULL || identifier.length != 0u) {
        return 12;
    }
    if (kmipkit_server_request_item_notify_attributes(NULL, &present, &attributes) !=
            KMIPKIT_ERROR_INVALID_INPUT || present != 0u || attributes != NULL) {
        return 28;
    }
    if (kmipkit_server_request_item_notify_deleted_attribute_reference_count(NULL, &count) !=
            KMIPKIT_ERROR_INVALID_INPUT || count != 0u) {
        return 29;
    }
    if (kmipkit_server_request_item_notify_deleted_attribute_reference_at(
            NULL, 0u, &deleted_reference) != KMIPKIT_ERROR_INVALID_INPUT) {
        return 30;
    }
    if (kmipkit_server_request_item_put_function(NULL, &operation) !=
            KMIPKIT_ERROR_INVALID_INPUT || operation != 0u) {
        return 13;
    }
    if (kmipkit_server_request_item_endpoint_role(NULL, &operation) !=
            KMIPKIT_ERROR_INVALID_INPUT || operation != 0u) {
        return 14;
    }
    if (kmipkit_server_request_item_discover_versions_count(NULL, &count) !=
            KMIPKIT_ERROR_INVALID_INPUT || count != 0u) {
        return 15;
    }
    if (kmipkit_server_request_item_discover_version_at(NULL, 0, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        return 16;
    }
    if (kmipkit_server_request_item_query_functions_count(NULL, &count) !=
            KMIPKIT_ERROR_INVALID_INPUT || count != 0u) {
        return 17;
    }
    if (kmipkit_server_request_item_query_function_at(NULL, 0, &operation) !=
            KMIPKIT_ERROR_INVALID_INPUT || operation != 0u) {
        return 18;
    }
    if (kmipkit_server_response_builder_create(NULL, &builder) !=
            KMIPKIT_ERROR_INVALID_INPUT || builder != NULL) {
        return 7;
    }
    if (kmipkit_server_notify_response_create(NULL, &response_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || response_item != NULL) {
        return 8;
    }
    if (kmipkit_server_put_response_create(NULL, &response_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || response_item != NULL) {
        return 19;
    }
    if (kmipkit_server_discover_versions_response_create(NULL, NULL, 0, &response_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || response_item != NULL) {
        return 20;
    }
    if (kmipkit_server_query_response_create(NULL, NULL, 0, NULL, 0, NULL, 0,
            (kmipkit_byte_span_v1_t){ NULL, 0 }, &response_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || response_item != NULL) {
        return 21;
    }
    if (kmipkit_server_set_endpoint_role_response_create(NULL, 1,
            (kmipkit_byte_span_v1_t){ NULL, 0 }, &response_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || response_item != NULL) {
        return 22;
    }
    if (kmipkit_server_response_builder_add_success(NULL, NULL, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        return 9;
    }
    if (kmipkit_server_response_builder_add_failure(NULL, NULL, 1, 0, 0,
            (kmipkit_byte_span_v1_t){ NULL, 0 }) != KMIPKIT_ERROR_INVALID_INPUT) {
        return 23;
    }
    if (kmipkit_server_response_builder_build(NULL, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        return 10;
    }
    if (kmipkit_server_connection_send_response(NULL, NULL, 0, 0) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        return 11;
    }
    return 0;
}

int32_t kmipkit_s2c_consumer_binds_tls_listener(
    const uint8_t *leaf, uint64_t leaf_length,
    const uint8_t *root, uint64_t root_length,
    const uint8_t *key, uint64_t key_length,
    uint16_t *out_port) {
    static const uint8_t host[] = "127.0.0.1";
    kmipkit_transport_input_v1_t chain[2] = {
        { 1u, 2u, { leaf, leaf_length } },
        { 1u, 2u, { root, root_length } },
    };
    kmipkit_transport_input_v1_t authority = { 1u, 2u, { root, root_length } };
    kmipkit_server_listener_config_v1_t input = {
        sizeof(kmipkit_server_listener_config_v1_t), 1u, 1u, 1u, 0u,
        { host, (uint64_t)(sizeof(host) - 1u) }, 0u, 0u, chain, 2u,
        { 1u, 2u, { key, key_length } }, &authority, 1u, NULL, 0u,
        { NULL, 0u }, 0u,
    };
    kmipkit_server_listener_config_t *configuration = NULL;
    kmipkit_server_listener_t *listener = NULL;
    int32_t status;
    if (out_port == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    *out_port = 0u;
    status = kmipkit_server_listener_config_create_v1(&input, &configuration);
    if (status != 0) return status;
    status = kmipkit_server_listener_bind_raw_tls(configuration, &listener);
    kmipkit_server_listener_config_release(configuration);
    if (status != 0) return status;
    status = kmipkit_server_listener_port(listener, out_port);
    kmipkit_server_listener_release(listener);
    return status;
}
