#include <stdint.h>

#include "kmipkit.h"

uint32_t kmipkit_credential_type_c_consumer_round_trip(uint32_t raw) {
    return kmipkit_credential_type_from_raw((kmipkit_credential_type_t)raw);
}
