#include <stdint.h>

#include "kmipkit.h"

_Static_assert(KMIPKIT_CREDENTIAL_TYPE_USERNAME_AND_PASSWORD == 1, "username/password value");
_Static_assert(KMIPKIT_CREDENTIAL_TYPE_DEVICE == 2, "device value");
_Static_assert(KMIPKIT_CREDENTIAL_TYPE_ATTESTATION == 3, "attestation value");
_Static_assert(KMIPKIT_CREDENTIAL_TYPE_ONE_TIME_PASSWORD == 4, "one-time password value");
_Static_assert(KMIPKIT_CREDENTIAL_TYPE_HASHED_PASSWORD == 5, "hashed password value");
_Static_assert(KMIPKIT_CREDENTIAL_TYPE_TICKET == 6, "ticket value");

int32_t kmipkit_credential_type_c_consumer_round_trip(uint32_t raw, uint32_t *out_raw) {
    return kmipkit_credential_type_from_raw((kmipkit_credential_type_t)raw, out_raw);
}
