/* Compiled public-header probe for the semantic C cryptographic builders. */
#include "kmipkit.h"

#include <stddef.h>

typedef int32_t (*crypto_builder_create_fn)(kmipkit_crypto_operation_builder_t **);

static int create_and_release(crypto_builder_create_fn create) {
    kmipkit_crypto_operation_builder_t *builder = NULL;
    if (create(NULL) != KMIPKIT_ERROR_INVALID_INPUT ||
        create(&builder) != INT32_C(0) || builder == NULL) {
        kmipkit_crypto_operation_builder_release(builder);
        return 1;
    }
    kmipkit_crypto_operation_builder_release(builder);
    return 0;
}

int kmipkit_public_crypto_builders_probe(void) {
    if (create_and_release(kmipkit_encrypt_builder_create) != 0 ||
        create_and_release(kmipkit_decrypt_builder_create) != 0 ||
        create_and_release(kmipkit_sign_builder_create) != 0 ||
        create_and_release(kmipkit_signature_verify_builder_create) != 0 ||
        create_and_release(kmipkit_mac_builder_create) != 0 ||
        create_and_release(kmipkit_mac_verify_builder_create) != 0 ||
        create_and_release(kmipkit_hash_builder_create) != 0) {
        return 1;
    }

    static const uint8_t input[] = "caller buffer";
    kmipkit_byte_span_v1_t span = {0};
    kmipkit_crypto_operation_builder_t *builder = NULL;
    span.data = input;
    span.length = (uint64_t)(sizeof(input) - 1);
    if (kmipkit_encrypt_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        return 2;
    }
    if (kmipkit_crypto_operation_builder_set_data(builder, &span) != INT32_C(0) ||
        kmipkit_crypto_operation_builder_build_batch_item(builder, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        kmipkit_crypto_operation_builder_release(builder);
        return 3;
    }
    kmipkit_crypto_operation_builder_release(builder);
    return 0;
}
