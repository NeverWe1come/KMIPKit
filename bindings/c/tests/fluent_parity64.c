#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "kmipkit.h"
#include "kmipkit_fluent.h"

static void check_op(int32_t rc, kmipkit_client_batch_item_t **item_ptr, uint32_t expected_op) {
    assert(rc == 0);
    assert(item_ptr != NULL && *item_ptr != NULL);
    uint32_t op = 0u;
    assert(kmipkit_client_batch_item_operation(*item_ptr, &op) == 0);
    assert(op == expected_op);
    kmipkit_client_batch_item_release(*item_ptr);
    *item_ptr = NULL;
}

int main(void) {
    assert(KMIPKIT_FLUENT_OPERATION_COUNT == 64u);

    kmipkit_unique_identifier_v1_t uid = KMIPKIT_UID_TEXT("obj-1");
    kmipkit_unique_identifier_v1_t uid_v3 = KMIPKIT_UID_FOR_VERSION("obj-1", 1);
    assert(uid.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
    assert(uid_v3.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_IDENTIFIER);

    kmipkit_fluent_result_t res = KMIPKIT_FLUENT_RESULT_INIT;
    kmipkit_fluent_result_release(&res);

    kmipkit_client_batch_item_t *item = NULL;

    /* Ping */
    kmipkit_fluent_ping_params_t ping_p = KMIPKIT_FLUENT_PING_PARAMS_INIT;
    check_op(kmipkit_fluent_item_ping(&ping_p, &item), &item, KMIPKIT_OP_PING);

    /* GetConstraints */
    kmipkit_fluent_get_constraints_params_t gc_p = KMIPKIT_FLUENT_GET_CONSTRAINTS_PARAMS_INIT;
    check_op(kmipkit_fluent_item_get_constraints(&gc_p, &item), &item, KMIPKIT_OP_GET_CONSTRAINTS);

    /* Activate */
    kmipkit_fluent_activate_params_t act_p = KMIPKIT_FLUENT_ACTIVATE_PARAMS_INIT;
    act_p.has_unique_identifier = 1u;
    act_p.unique_identifier = uid;
    check_op(kmipkit_fluent_item_activate(&act_p, &item), &item, KMIPKIT_OP_ACTIVATE);

    /* Export */
    kmipkit_fluent_export_params_t exp_p = KMIPKIT_FLUENT_EXPORT_PARAMS_INIT;
    exp_p.has_unique_identifier = 1u;
    exp_p.unique_identifier = uid;
    exp_p.has_key_format_type = 1u;
    exp_p.key_format_type = 1u;
    check_op(kmipkit_fluent_item_export(&exp_p, &item), &item, KMIPKIT_OP_EXPORT);

    /* Cancel, Poll, Process, QueryAsyncRequests */
    const uint8_t corr_bytes[] = { 0x01u, 0x02u, 0x03u, 0x04u };
    kmipkit_fluent_cancel_params_t cancel_p = KMIPKIT_FLUENT_CANCEL_PARAMS_INIT;
    cancel_p.correlation_value_data = corr_bytes;
    cancel_p.correlation_value_length = sizeof(corr_bytes);
    check_op(kmipkit_fluent_item_cancel(&cancel_p, &item), &item, KMIPKIT_OP_CANCEL);

    kmipkit_fluent_poll_params_t poll_p = KMIPKIT_FLUENT_POLL_PARAMS_INIT;
    poll_p.correlation_value_data = corr_bytes;
    poll_p.correlation_value_length = sizeof(corr_bytes);
    check_op(kmipkit_fluent_item_poll(&poll_p, &item), &item, KMIPKIT_OP_POLL);

    kmipkit_fluent_process_params_t proc_p = KMIPKIT_FLUENT_PROCESS_PARAMS_INIT;
    proc_p.correlation_value_data = corr_bytes;
    proc_p.correlation_value_length = sizeof(corr_bytes);
    check_op(kmipkit_fluent_item_process(&proc_p, &item), &item, KMIPKIT_OP_PROCESS);

    kmipkit_byte_span_v1_t corr_spans[] = { KMIPKIT_SPAN_BYTES(corr_bytes, sizeof(corr_bytes)) };
    uint32_t op_filters[] = { KMIPKIT_OP_PING };
    kmipkit_fluent_query_async_requests_params_t qa_p = KMIPKIT_FLUENT_QUERY_ASYNC_REQUESTS_PARAMS_INIT;
    qa_p.correlation_values = corr_spans;
    qa_p.correlation_value_count = 1u;
    qa_p.operations = op_filters;
    qa_p.operation_count = 1u;
    check_op(kmipkit_fluent_item_query_async_requests(&qa_p, &item), &item, KMIPKIT_OP_QUERY_ASYNC_REQUESTS);

    /* Login, SetEndpointRole, Log, Interop, ReProvision */
    kmipkit_fluent_login_params_t login_p = KMIPKIT_FLUENT_LOGIN_PARAMS_INIT;
    login_p.lease_time_kind = 1u;
    login_p.lease_time_value = 300;
    check_op(kmipkit_fluent_item_login(&login_p, &item), &item, KMIPKIT_OP_LOGIN);

    kmipkit_fluent_set_endpoint_role_params_t role_p = KMIPKIT_FLUENT_SET_ENDPOINT_ROLE_PARAMS_INIT;
    role_p.endpoint_role = 1u;
    role_p.vendor_identification_data = (const uint8_t *)"KMIPKit";
    role_p.vendor_identification_length = 7u;
    check_op(kmipkit_fluent_item_set_endpoint_role(&role_p, &item), &item, KMIPKIT_OP_SET_ENDPOINT_ROLE);

    kmipkit_fluent_log_params_t log_p = KMIPKIT_FLUENT_LOG_PARAMS_INIT;
    log_p.log_message_data = (const uint8_t *)"c11-audit";
    log_p.log_message_length = 9u;
    check_op(kmipkit_fluent_item_log(&log_p, &item), &item, KMIPKIT_OP_LOG);

    kmipkit_fluent_interop_params_t interop_p = KMIPKIT_FLUENT_INTEROP_PARAMS_INIT;
    interop_p.interop_function = 1u;
    interop_p.interop_identifier_data = (const uint8_t *)"id";
    interop_p.interop_identifier_length = 2u;
    check_op(kmipkit_fluent_item_interop(&interop_p, &item), &item, KMIPKIT_OP_INTEROP);

    kmipkit_fluent_reprovision_params_t reprov_p = KMIPKIT_FLUENT_REPROVISION_PARAMS_INIT;
    check_op(kmipkit_fluent_item_reprovision(&reprov_p, &item), &item, KMIPKIT_OP_REPROVISION);

    /* Revoke & Obliterate */
    kmipkit_fluent_revoke_params_t rev_p = KMIPKIT_FLUENT_REVOKE_PARAMS_INIT;
    rev_p.has_unique_identifier = 1u;
    rev_p.unique_identifier = uid;
    rev_p.revocation_reason_code = 1u;
    check_op(kmipkit_fluent_item_revoke(&rev_p, &item), &item, KMIPKIT_OP_REVOKE);

    kmipkit_fluent_obliterate_params_t ob_p = KMIPKIT_FLUENT_OBLITERATE_PARAMS_INIT;
    ob_p.unique_identifier = KMIPKIT_UID_IDENTIFIER("ob-1");
    check_op(kmipkit_fluent_item_obliterate(&ob_p, &item), &item, KMIPKIT_OP_OBLITERATE);

    /* Ensure executing with NULL client safely returns KMIPKIT_ERROR_INVALID_INPUT without leaks */
    assert(kmipkit_fluent_ping(NULL, &ping_p, &res) == KMIPKIT_ERROR_INVALID_INPUT);
    kmipkit_fluent_result_release(&res);

    printf("C11 fluent parity64 test passed\n");
    return 0;
}
