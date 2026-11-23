#include "kmipkit.h"

#include <assert.h>
#include <string.h>

/* OASIS KMIP Specification v2.1 §§11.12 and 12.1. */
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

    /* KMIP 2.1 §§11.36 and 11.56 Tables 470 and 487; KMIP 3.0 §§11.37 and 11.58 Tables 584 and 605. */
    assert(KMIPKIT_V2_1_OPERATION_SET_DEFAULTS == UINT32_C(0x36));
    assert(strcmp(KMIPKIT_V2_1_OPERATION_name(KMIPKIT_V2_1_OPERATION_PING), "Ping") == 0);
    assert(KMIPKIT_V3_0_OPERATION_OBLITERATE == UINT32_C(0x3D));
    assert(strcmp(KMIPKIT_V3_0_OPERATION_name(KMIPKIT_V3_0_OPERATION_CREATE_CREDENTIAL),
                  "Create Credential") == 0);
    assert(strcmp(KMIPKIT_V3_0_OBJECT_CLASS_name(UINT32_C(1)), "User") == 0);
    assert(KMIPKIT_V3_0_OBJECT_CLASS_name(UINT32_C(3)) == 0);
    assert(strcmp(KMIPKIT_V3_0_OBJECT_TYPE_name(UINT32_C(0x0D)), "Password Credential") == 0);
    assert(KMIPKIT_V3_0_OBJECT_TYPE_name(UINT32_C(6)) == 0);

    /* Assigned tag names remain version scoped. */
    assert(KMIPKIT_ENUMERATION_TAG_OBJECT_GROUPS == UINT32_C(0x420166));
    assert(strcmp(KMIPKIT_ENUMERATION_TAG_name(KMIPKIT_ENUMERATION_TAG_OBJECT_GROUPS),
                  "Object Groups") == 0);
    assert(KMIPKIT_ENUMERATION_TAG_name(UINT32_C(0x4201FF)) == 0);
    assert(KMIPKIT_V3_0_TAG_CERTIFICATE_LINK == UINT32_C(0x420190));
    assert(strcmp(KMIPKIT_V3_0_TAG_name(KMIPKIT_V3_0_TAG_CERTIFICATE_LINK),
                  "Certificate Link") == 0);
    assert(KMIPKIT_V3_0_TAG_name(UINT32_C(0x42018F)) == 0);

    /* KMIP 3.0 Ephemeral Mode & Query Function */
    assert(KMIPKIT_V3_0_EPHEMERAL_MODE_EMPTY == UINT32_C(2));
    assert(strcmp(KMIPKIT_V3_0_EPHEMERAL_MODE_name(KMIPKIT_V3_0_EPHEMERAL_MODE_EMPTY), "Empty") == 0);
    assert(strcmp(KMIPKIT_V3_0_QUERY_FUNCTION_name(UINT32_C(15)), "Query Credential Information") == 0);
    return 0;
}
