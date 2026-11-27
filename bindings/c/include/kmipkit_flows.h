/* kmipkit_flows.h - Phase 4 batch & pending-operation helpers for C11 callers. */
#ifndef KMIPKIT_FLOWS_H
#define KMIPKIT_FLOWS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#if defined(KMIPKIT_H)
typedef struct kmipkit_fluent_batch_t {
    kmipkit_client_batch_t *batch;
} kmipkit_fluent_batch_t;

static inline int32_t kmipkit_fluent_batch_create(kmipkit_fluent_batch_t *out) {
    if (out == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    out->batch = NULL;
    return kmipkit_client_batch_create(&out->batch);
}

static inline int32_t kmipkit_fluent_batch_add(
    kmipkit_fluent_batch_t *b,
    kmipkit_client_batch_item_t **inout_item
) {
    if (b == NULL || b->batch == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    return kmipkit_client_batch_add_item(b->batch, inout_item);
}

static inline int32_t kmipkit_fluent_batch_ordered(
    kmipkit_fluent_batch_t *b,
    uint8_t enabled
) {
    if (b == NULL || b->batch == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    return kmipkit_client_batch_set_order_option(b->batch, enabled);
}

static inline int32_t kmipkit_fluent_batch_set_authentication(
    kmipkit_fluent_batch_t *b,
    kmipkit_authentication_t *auth
) {
    if (b == NULL || b->batch == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    return kmipkit_client_batch_set_authentication(b->batch, auth);
}

static inline int32_t kmipkit_fluent_batch_execute(
    kmipkit_fluent_batch_t *b,
    kmipkit_client_t *client,
    kmipkit_codec_limits_t *limits,
    const kmipkit_request_options_v1_t *options,
    kmipkit_client_batch_response_t **out_response
) {
    if (b == NULL || b->batch == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    if (options != NULL) {
        return kmipkit_client_execute_with_options_v1(
            client, &b->batch, limits, options, out_response);
    }
    return kmipkit_client_execute(client, &b->batch, limits, out_response);
}

static inline void kmipkit_fluent_batch_release(kmipkit_fluent_batch_t *b) {
    if (b != NULL && b->batch != NULL) {
        kmipkit_client_batch_release(b->batch);
        b->batch = NULL;
    }
}

typedef struct kmipkit_pending_op_t {
    const uint8_t *correlation_data;
    uint64_t correlation_length;
} kmipkit_pending_op_t;

static inline int32_t kmipkit_pending_op_poll(
    const kmipkit_pending_op_t *op,
    kmipkit_client_t *client,
    kmipkit_codec_limits_t *limits,
    kmipkit_client_async_operation_outcome_t **out_outcome
) {
    if (op == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    return kmipkit_client_execute_poll(
        client, op->correlation_data, op->correlation_length, limits, out_outcome);
}

static inline int32_t kmipkit_pending_op_cancel(
    const kmipkit_pending_op_t *op,
    kmipkit_client_t *client,
    kmipkit_codec_limits_t *limits,
    kmipkit_client_async_operation_outcome_t **out_outcome
) {
    if (op == NULL) return KMIPKIT_ERROR_INVALID_INPUT;
    return kmipkit_client_execute_cancel(
        client, op->correlation_data, op->correlation_length, limits, out_outcome);
}
#endif

#ifdef __cplusplus
}
#endif

#endif /* KMIPKIT_FLOWS_H */
