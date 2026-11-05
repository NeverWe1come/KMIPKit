/* Compile and exercise the client construction surface from the public header. */
#include "kmipkit.h"

int kmipkit_public_client_surface_probe(void) {
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    int32_t status = kmipkit_client_batch_create(&batch);
    if (status != 0 || batch == NULL) {
        return 1;
    }

    status = kmipkit_client_batch_item_discover_versions(&item);
    if (status != 0 || item == NULL) {
        kmipkit_client_batch_release(batch);
        return 2;
    }
    status = kmipkit_client_batch_add_item(batch, &item);
    if (status != 0 || item != NULL) {
        if (item != NULL) {
            kmipkit_client_batch_item_release(item);
        }
        kmipkit_client_batch_release(batch);
        return 3;
    }
    kmipkit_client_batch_release(batch);

    /* A null client/batch is rejected before I/O; -1 exercises the no-override path. */
    kmipkit_client_batch_t *empty_batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    status = kmipkit_client_execute_with_options(NULL, &empty_batch, NULL, -1, &response);
    return status == 4 && response == NULL ? 0 : 4;
}

/* Implemented in the GREEN step; this stub keeps the RED integration test linkable. */
int kmipkit_public_client_execute_json_override(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length) {
    (void)port;
    (void)leaf_certificate;
    (void)leaf_certificate_length;
    (void)intermediate_certificate;
    (void)intermediate_certificate_length;
    (void)private_key;
    (void)private_key_length;
    (void)trust_root;
    (void)trust_root_length;
    return 99;
}
