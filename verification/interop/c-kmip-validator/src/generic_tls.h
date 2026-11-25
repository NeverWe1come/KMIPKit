#ifndef KMIP_C_GENERIC_TLS_H
#define KMIP_C_GENERIC_TLS_H

#include "c_validator.h"

bool kmip_generic_tls_exchange(const kmip_connection_config_t *configuration,
                               const uint8_t *request, size_t request_size, size_t response_limit,
                               uint8_t **response, size_t *response_size);

typedef struct kmip_generic_listener kmip_generic_listener_t;

typedef bool (*kmip_generic_reply_fn)(const uint8_t *, size_t, uint8_t **, size_t *, void *);
typedef enum {
    KMIP_GENERIC_PEER_UNAVAILABLE,
    KMIP_GENERIC_INCOMING_FAILED,
    KMIP_GENERIC_REPLIED
} kmip_generic_reception_t;

kmip_generic_listener_t *kmip_generic_tls_listen(
    const kmip_connection_config_t *configuration, const char *address, const char *certificate,
    const char *key, const char *ready_path);
void kmip_generic_tls_listener_close(kmip_generic_listener_t *listener);
kmip_generic_reception_t kmip_generic_tls_receive(
    kmip_generic_listener_t *listener, size_t limit, kmip_generic_reply_fn reply, void *context);

#endif
