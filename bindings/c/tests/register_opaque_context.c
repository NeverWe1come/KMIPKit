/* Public-header consumer coverage for KMIPKIT-0044 Register context APIs. */

#include "kmipkit.h"

#include <stddef.h>
#include <stdint.h>

int main(void)
{
    kmipkit_register_request_t *request = NULL;
    kmipkit_client_batch_item_t *item = NULL;

    (void)kmipkit_register_request_create(UINT32_C(8), NULL, NULL, &request);
    (void)kmipkit_register_request_with_opaque_object_context(
        request, UINT32_C(1), &request);
    (void)kmipkit_client_batch_item_register(request, &item);
    return 0;
}