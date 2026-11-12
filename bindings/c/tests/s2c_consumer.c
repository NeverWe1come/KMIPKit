/* Public C consumer contract for KMIPKIT-0093-FR-012 / KMIPKIT-0066 REQ-066-009. */
#include <stdint.h>
#include <stddef.h>

typedef struct kmipkit_server_listener_config_v1_t kmipkit_server_listener_config_v1_t;
typedef struct kmipkit_server_listener_t kmipkit_server_listener_t;
typedef struct kmipkit_server_connection_t kmipkit_server_connection_t;
typedef struct kmipkit_server_incoming_batch_t kmipkit_server_incoming_batch_t;
typedef struct kmipkit_server_request_item_t kmipkit_server_request_item_t;
typedef struct kmipkit_server_response_t kmipkit_server_response_t;
typedef struct kmipkit_codec_limits_t kmipkit_codec_limits_t;

enum { KMIPKIT_ERROR_INVALID_INPUT = 4 };

extern int32_t kmipkit_server_listener_config_create_v1(
    const kmipkit_server_listener_config_v1_t *configuration,
    kmipkit_server_listener_config_v1_t **out_configuration);
extern int32_t kmipkit_server_listener_bind_raw_tls(
    kmipkit_server_listener_config_v1_t **configuration,
    kmipkit_server_listener_t **out_listener);
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
extern int32_t kmipkit_server_incoming_batch_item_at(
    kmipkit_server_incoming_batch_t *batch,
    uint64_t index,
    kmipkit_server_request_item_t **out_item);
extern int32_t kmipkit_server_request_item_operation(
    kmipkit_server_request_item_t *item,
    uint32_t *out_operation);
extern int32_t kmipkit_server_response_create_success(
    kmipkit_server_incoming_batch_t *batch,
    uint64_t item_index,
    kmipkit_server_response_t **out_response);
extern int32_t kmipkit_server_connection_send_response(
    kmipkit_server_connection_t *connection,
    kmipkit_server_response_t *response,
    uint64_t timeout_seconds,
    uint32_t timeout_nanoseconds);
extern void kmipkit_server_listener_release(kmipkit_server_listener_t *listener);
extern void kmipkit_server_connection_release(kmipkit_server_connection_t *connection);
extern void kmipkit_server_incoming_batch_release(kmipkit_server_incoming_batch_t *batch);
extern void kmipkit_server_request_item_release(kmipkit_server_request_item_t *item);
extern void kmipkit_server_response_release(kmipkit_server_response_t *response);

/* Null and wrong-kind inputs fail closed and initialize every writable output. */
int32_t kmipkit_s2c_consumer_rejects_invalid_handles(void) {
    kmipkit_server_listener_config_v1_t *configuration = (void *)1;
    kmipkit_server_listener_t *listener = (void *)1;
    kmipkit_server_connection_t *connection = (void *)1;
    kmipkit_server_incoming_batch_t *batch = (void *)1;
    kmipkit_server_request_item_t *item = (void *)1;
    kmipkit_server_response_t *response = (void *)1;
    uint32_t operation = UINT32_MAX;

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
    if (kmipkit_server_response_create_success(NULL, 0, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        return 7;
    }
    if (kmipkit_server_connection_send_response(NULL, NULL, 0, 0) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        return 8;
    }
    return 0;
}
