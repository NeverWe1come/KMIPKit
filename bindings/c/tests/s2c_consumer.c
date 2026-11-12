/* Public C consumer contract for KMIPKIT-0093-FR-012 / KMIPKIT-0066 REQ-066-009. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>

typedef struct kmipkit_server_listener_config_t kmipkit_server_listener_config_t;
typedef struct kmipkit_server_listener_config_v1_t kmipkit_server_listener_config_v1_t;
typedef struct kmipkit_server_listener_t kmipkit_server_listener_t;
typedef struct kmipkit_server_connection_t kmipkit_server_connection_t;
typedef struct kmipkit_server_incoming_batch_t kmipkit_server_incoming_batch_t;
typedef struct kmipkit_server_request_item_t kmipkit_server_request_item_t;
typedef struct kmipkit_server_response_builder_t kmipkit_server_response_builder_t;
typedef struct kmipkit_server_response_item_t kmipkit_server_response_item_t;
typedef struct kmipkit_server_response_t kmipkit_server_response_t;
typedef struct kmipkit_server_put_object_t kmipkit_server_put_object_t;
typedef struct kmipkit_codec_limits_t kmipkit_codec_limits_t;
typedef struct kmipkit_client_batch_item_t kmipkit_client_batch_item_t;
typedef struct kmipkit_protocol_version_v1_t kmipkit_protocol_version_v1_t;
typedef struct kmipkit_secret_bytes_t kmipkit_secret_bytes_t;
typedef struct kmipkit_secret_text_t kmipkit_secret_text_t;

typedef struct {
    const uint8_t *data;
    uint64_t length;
} kmipkit_byte_span_v1_t;
typedef struct {
    uint32_t source_kind;
    uint32_t encoding;
    kmipkit_byte_span_v1_t data;
} kmipkit_transport_input_v1_t;
struct kmipkit_server_listener_config_v1_t {
    uint32_t struct_size, abi_version, transport_kind, wire_encoding, reserved;
    kmipkit_byte_span_v1_t bind_host;
    uint16_t bind_port, reserved2;
    const kmipkit_transport_input_v1_t *listener_certificates;
    uint64_t listener_certificate_count;
    kmipkit_transport_input_v1_t listener_private_key;
    const kmipkit_transport_input_v1_t *trust_certificates;
    uint64_t trust_certificate_count;
    const kmipkit_transport_input_v1_t *revocation_lists;
    uint64_t revocation_list_count;
    kmipkit_byte_span_v1_t target_path;
    uint64_t max_request_bytes;
};

enum { KMIPKIT_ERROR_INVALID_INPUT = 4 };

extern int32_t kmipkit_server_listener_config_create_v1(
    const kmipkit_server_listener_config_v1_t *configuration,
    kmipkit_server_listener_config_t **out_configuration);
extern int32_t kmipkit_server_listener_bind_raw_tls(
    kmipkit_server_listener_config_t *configuration,
    kmipkit_server_listener_t **out_listener);
extern int32_t kmipkit_server_listener_port(kmipkit_server_listener_t *listener, uint16_t *out_port);
extern int32_t kmipkit_server_listener_accept(
    kmipkit_server_listener_t *listener,
    uint64_t timeout_seconds,
    uint32_t timeout_nanoseconds,
    kmipkit_server_connection_t **out_connection);
extern int32_t kmipkit_server_connection_recv_batch(
    kmipkit_server_connection_t *connection,
    kmipkit_codec_limits_t *limits,
    uint64_t timeout_seconds,
    uint32_t timeout_nanoseconds,
    kmipkit_server_incoming_batch_t **out_batch);
extern int32_t kmipkit_server_incoming_batch_count(
    kmipkit_server_incoming_batch_t *batch,
    uint64_t *out_count);
extern int32_t kmipkit_server_incoming_batch_item_at(
    kmipkit_server_incoming_batch_t *batch,
    uint64_t index,
    kmipkit_server_request_item_t **out_item);
extern int32_t kmipkit_server_request_item_operation(
    kmipkit_server_request_item_t *item,
    uint32_t *out_operation);
extern int32_t kmipkit_server_request_item_unique_identifier(
    kmipkit_server_request_item_t *item,
    kmipkit_byte_span_v1_t *out_identifier);
extern int32_t kmipkit_server_request_item_put_function(
    kmipkit_server_request_item_t *item,
    uint32_t *out_put_function);
extern int32_t kmipkit_server_request_item_endpoint_role(
    kmipkit_server_request_item_t *item,
    uint32_t *out_endpoint_role);
extern int32_t kmipkit_server_request_item_discover_versions_count(
    kmipkit_server_request_item_t *item,
    uint64_t *out_count);
extern int32_t kmipkit_server_request_item_discover_version_at(
    kmipkit_server_request_item_t *item,
    uint64_t index,
    void *out_version);
extern int32_t kmipkit_server_request_item_query_functions_count(
    kmipkit_server_request_item_t *item,
    uint64_t *out_count);
extern int32_t kmipkit_server_request_item_query_function_at(
    kmipkit_server_request_item_t *item,
    uint64_t index,
    uint32_t *out_function);
extern int32_t kmipkit_server_response_builder_create(
    kmipkit_server_incoming_batch_t *batch,
    kmipkit_server_response_builder_t **out_builder);
extern int32_t kmipkit_server_notify_response_create(
    kmipkit_server_request_item_t *request,
    kmipkit_server_response_item_t **out_response_item);
extern int32_t kmipkit_server_put_response_create(
    kmipkit_server_request_item_t *request,
    kmipkit_server_response_item_t **out_response_item);
extern int32_t kmipkit_server_discover_versions_response_create(
    kmipkit_server_request_item_t *request,
    const void *versions,
    uint64_t version_count,
    kmipkit_server_response_item_t **out_response_item);
extern int32_t kmipkit_server_query_response_create(
    kmipkit_server_request_item_t *request,
    const uint32_t *operations,
    uint64_t operation_count,
    const uint32_t *object_types,
    uint64_t object_type_count,
    const uint32_t *extension_tags,
    uint64_t extension_tag_count,
    kmipkit_byte_span_v1_t vendor_identification,
    kmipkit_server_response_item_t **out_response_item);
extern int32_t kmipkit_server_set_endpoint_role_response_create(
    kmipkit_server_request_item_t *request,
    uint32_t endpoint_role,
    kmipkit_byte_span_v1_t vendor_identification,
    kmipkit_server_response_item_t **out_response_item);
extern int32_t kmipkit_server_response_builder_add_failure(
    kmipkit_server_response_builder_t *builder,
    kmipkit_server_request_item_t *request,
    uint32_t result_status,
    uint8_t has_result_reason,
    uint32_t result_reason,
    kmipkit_byte_span_v1_t result_message);
extern int32_t kmipkit_server_response_builder_add_success(
    kmipkit_server_response_builder_t *builder,
    kmipkit_server_request_item_t *request,
    kmipkit_server_response_item_t *response_item);
extern int32_t kmipkit_server_response_builder_build(
    kmipkit_server_response_builder_t *builder,
    kmipkit_server_response_t **out_response);
extern int32_t kmipkit_server_connection_send_response(
    kmipkit_server_connection_t *connection,
    kmipkit_server_response_t *response,
    uint64_t timeout_seconds,
    uint32_t timeout_nanoseconds);
extern void kmipkit_server_listener_release(kmipkit_server_listener_t *listener);
extern void kmipkit_server_listener_config_release(kmipkit_server_listener_config_t *configuration);
extern void kmipkit_server_connection_release(kmipkit_server_connection_t *connection);
extern void kmipkit_server_incoming_batch_release(kmipkit_server_incoming_batch_t *batch);
extern void kmipkit_server_request_item_release(kmipkit_server_request_item_t *item);
extern void kmipkit_server_response_builder_release(kmipkit_server_response_builder_t *builder);
extern void kmipkit_server_response_item_release(kmipkit_server_response_item_t *item);
extern void kmipkit_server_response_release(kmipkit_server_response_t *response);
extern int32_t kmipkit_server_request_item_put_object(
    kmipkit_server_request_item_t *request,
    kmipkit_server_put_object_t **out_object);
extern int32_t kmipkit_server_put_object_type(kmipkit_server_put_object_t *object, uint32_t *out_object_type);
extern int32_t kmipkit_server_put_object_tag(kmipkit_server_put_object_t *object, uint32_t *out_tag);
extern int32_t kmipkit_server_put_object_field_read(
    kmipkit_server_put_object_t *object,
    uint32_t field_tag,
    uint32_t read_kind,
    uint8_t *out_present,
    uint8_t *out_item_type,
    uint32_t *out_enumeration,
    int32_t *out_integer,
    kmipkit_server_put_object_t **out_child,
    kmipkit_secret_bytes_t **out_secret,
    kmipkit_secret_text_t **out_secret_text);
extern int32_t kmipkit_secret_bytes_with_bytes(
    kmipkit_secret_bytes_t *secret,
    void (*callback)(const uint8_t *, uint64_t, void *),
    void *context);
extern void kmipkit_secret_bytes_release(kmipkit_secret_bytes_t *secret);
extern int32_t kmipkit_secret_text_with_str(
    kmipkit_secret_text_t *secret,
    void (*callback)(const uint8_t *, uint64_t, void *),
    void *context);
extern void kmipkit_secret_text_release(kmipkit_secret_text_t *secret);
extern void kmipkit_server_put_object_release(kmipkit_server_put_object_t *object);
extern int32_t kmipkit_codec_limits_defaults(kmipkit_codec_limits_t **out_limits);
extern void kmipkit_codec_limits_release(kmipkit_codec_limits_t *limits);

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
    kmipkit_server_put_object_t *object = NULL, *key_block = NULL, *key_value = NULL;
    kmipkit_server_put_object_t *password_object = NULL, *unknown_object = NULL;
    kmipkit_secret_bytes_t *secret = NULL;
    kmipkit_secret_text_t *secret_text = NULL;
    kmipkit_server_listener_config_t *configuration = NULL;
    kmipkit_server_listener_t *listener = NULL;
    kmipkit_server_connection_t *connection = NULL;
    kmipkit_server_incoming_batch_t *batch = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_server_request_item_t *request = NULL, *password_request = NULL, *unknown_request = NULL;
    kmipkit_server_response_builder_t *builder = NULL;
    kmipkit_server_response_item_t *response_item = NULL;
    kmipkit_server_response_t *response = NULL;
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
    if (status != 0 || batch_count != 3u) { status = 101; goto cleanup; }
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
    status = kmipkit_server_put_object_field_read(object, 0x00420040u, 6u,
        &present, &item_type, NULL, NULL, &key_block, NULL, NULL);
    if (status != 0 || present != 1u || key_block == NULL || item_type != 1u) { status = 105; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_block, 0x00420042u, 3u,
        &present, &item_type, &key_format, NULL, NULL, NULL, NULL);
    if (status != 0 || present != 1u || key_format != 1u) { status = 106; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_block, 0x00420045u, 6u,
        &present, &item_type, NULL, NULL, &key_value, NULL, NULL);
    if (status != 0 || present != 1u || key_value == NULL || item_type != 1u) { status = 107; goto cleanup; }
    status = kmipkit_server_put_object_field_read(key_value, 0x00420043u, 5u,
        &present, &item_type, NULL, NULL, NULL, &secret, NULL);
    if (status != 0 || present != 1u || secret == NULL || item_type != 8u) { status = 108; goto cleanup; }
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
    status = kmipkit_server_put_object_field_read(password_object, 0x004200A1u, 7u,
        &present, &item_type, NULL, NULL, NULL, NULL, &secret_text);
    if (status != 0 || present != 1u || secret_text == NULL || item_type != 7u) { status = 114; goto cleanup; }
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
    status = kmipkit_server_put_object_field_read(unknown_object, 0x00540002u, 3u,
        &present, &item_type, &key_format, NULL, NULL, NULL, NULL);
    if (status != 0 || present != 1u || key_format != 0xA1B2C3D4u) { status = 120; goto cleanup; }
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
    status = kmipkit_server_put_response_create(unknown_request, &response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_add_success(builder, unknown_request, response_item);
    if (status != 0) goto cleanup;
    status = kmipkit_server_response_builder_build(builder, &response);
    if (status != 0) goto cleanup;
    status = kmipkit_server_connection_send_response(connection, response, 5u, 0u);

cleanup:
    if (secret != NULL) kmipkit_secret_bytes_release(secret);
    if (secret_text != NULL) kmipkit_secret_text_release(secret_text);
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
    kmipkit_secret_bytes_t *secret = (void *)1;
    kmipkit_secret_text_t *secret_text = (void *)1;
    uint32_t operation = UINT32_MAX;
    uint32_t object_type = UINT32_MAX;
    uint8_t present = UINT8_MAX;
    uint64_t count = UINT64_MAX;
    kmipkit_byte_span_v1_t identifier = { (const uint8_t *)1, UINT64_MAX };

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
    if (kmipkit_server_put_object_field_read(NULL, 0x00420043u, 5u,
            &present, NULL, NULL, NULL, NULL, &secret, &secret_text) !=
            KMIPKIT_ERROR_INVALID_INPUT || present != 0u || secret != NULL) {
        return 26;
    }
    if (kmipkit_server_request_item_unique_identifier(NULL, &identifier) !=
            KMIPKIT_ERROR_INVALID_INPUT || identifier.data != NULL || identifier.length != 0u) {
        return 12;
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
