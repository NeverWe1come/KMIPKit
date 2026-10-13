#include "kmipkit.h"

#include <stdint.h>

/*
 * T076 APIs remain blocked from the generated public header until their
 * Java/Python mappings are verified. This narrow test-local surface lets the
 * compiled consumer exercise the C implementation without claiming public
 * header or four-language parity evidence.
 */
typedef struct kmipkit_client_t kmipkit_client_t;
typedef struct kmipkit_transport_config_t kmipkit_transport_config_t;
typedef struct kmipkit_client_batch_t kmipkit_client_batch_t;
#define T076_ERROR_INVALID_INPUT ((int32_t)4)

int32_t kmipkit_transport_config_create_v1(
    const kmipkit_transport_config_v1_t *configuration,
    kmipkit_transport_config_t **out_configuration);
int32_t kmipkit_client_create(
    kmipkit_client_configuration_t *configuration,
    kmipkit_transport_config_t **transport_configuration,
    kmipkit_client_t **out_client);
int32_t kmipkit_client_batch_create(kmipkit_client_batch_t **out_batch);
int32_t kmipkit_client_batch_add_item(
    kmipkit_client_batch_t *batch,
    kmipkit_client_batch_item_t **inout_item);
void kmipkit_client_release(kmipkit_client_t *handle);
void kmipkit_client_batch_release(kmipkit_client_batch_t *handle);
void kmipkit_client_batch_item_release(kmipkit_client_batch_item_t *handle);

int main(void) {
    kmipkit_client_t *client = 0;
    if (kmipkit_client_create(0, 0, &client) != T076_ERROR_INVALID_INPUT) {
        return 1;
    }
    if (client != 0) {
        return 2;
    }

    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_transport_config_t *transport = 0;
    if (kmipkit_transport_config_create_v1(&transport_input, &transport)
            != T076_ERROR_INVALID_INPUT) {
        return 3;
    }
    if (transport != 0) {
        return 4;
    }

    kmipkit_client_batch_t *batch = 0;
    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0) {
        return 5;
    }
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_discover_versions(&item) != 0 || item == 0) {
        kmipkit_client_batch_release(batch);
        return 6;
    }
    if (kmipkit_client_batch_add_item(batch, &item) != 0 || item != 0) {
        kmipkit_client_batch_item_release(item);
        kmipkit_client_batch_release(batch);
        return 7;
    }
    kmipkit_client_batch_release(batch);
    return 0;
}
