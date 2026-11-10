#ifndef KMIP_C_LIVE_DRIVER_H
#define KMIP_C_LIVE_DRIVER_H

#include "c_validator.h"
#include "kmipkit.h"

typedef struct {
    kmipkit_client_t *client;
    kmipkit_codec_limits_t *limits;
    int32_t protocol_major;
    int32_t protocol_minor;
    uint32_t wire_encoding;
    char api_level[32];
    char username_env[KMIP_MAX_NAME_LEN];
    char password_env[KMIP_MAX_NAME_LEN];
    bool configured_version_propagation_available;
    bool allow_mutations;
} kmip_live_client_t;

bool kmip_live_client_open(
    const kmip_connection_config_t *configuration,
    kmip_live_client_t *out_client,
    char *error,
    size_t error_capacity
);

void kmip_live_client_close(kmip_live_client_t *client);

kmip_execution_outcome_t kmip_live_execute_case(
    const kmip_test_case_t *test_case,
    const kmip_attempt_context_t *attempt,
    void *user_data
);

#endif
