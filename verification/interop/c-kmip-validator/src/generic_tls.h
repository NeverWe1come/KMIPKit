#ifndef KMIP_C_GENERIC_TLS_H
#define KMIP_C_GENERIC_TLS_H

#include "c_validator.h"

bool kmip_generic_tls_exchange(const kmip_connection_config_t *configuration,
                               const uint8_t *request, size_t request_size, size_t response_limit,
                               uint8_t **response, size_t *response_size);

#endif
