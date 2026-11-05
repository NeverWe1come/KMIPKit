#include "kmipkit.h"

#include <assert.h>
#include <string.h>

/* OASIS KMIP Specification v2.1 §§11.12 and 12.1; KMIPKIT-0047-FR-001–004. */
int main(void) {
    assert(KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES == 3u);
    assert(strcmp(KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name(
                      KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES),
                  "AES") == 0);
    assert(KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name(UINT32_MAX) == 0);

    const uint32_t raw = KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT | UINT32_C(0x80000000);
    assert(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_known_bits(raw)
           == KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT);
    assert(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_unknown_bits(raw) == UINT32_C(0x80000000));
    assert(strcmp(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name(
                      KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT),
                  "Encrypt") == 0);

    const char *names[1] = {0};
    assert(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_names(raw, names, 1) == 1);
    assert(strcmp(names[0], "Encrypt") == 0);
    assert(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_names(raw, 0, 0) == 1);
    assert(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_names(raw, 0, 1) == 0);
    return 0;
}
