/* Public-header consumer coverage for KMIPKIT-0044 Register context APIs. */

#include "kmipkit.h"

#include <stddef.h>
#include <stdint.h>

_Static_assert(KMIPKIT_OPAQUE_OBJECT_CONTEXT_GENERIC == UINT32_C(1),
               "generic opaque context must use its published numeric value");
_Static_assert(KMIPKIT_OPAQUE_OBJECT_CONTEXT_MANAGED_CRYPTOGRAPHIC_OBJECT == UINT32_C(2),
               "managed cryptographic context must use its published numeric value");

int main(void)
{
    kmipkit_register_request_t *request = NULL;
    kmipkit_client_batch_item_t *item = NULL;

    (void)kmipkit_register_request_create(UINT32_C(8), NULL, NULL, &request);
    (void)kmipkit_register_request_with_opaque_object_context(
        request, KMIPKIT_OPAQUE_OBJECT_CONTEXT_GENERIC, &request);
    (void)kmipkit_client_batch_item_register(request, &item);
    return 0;
}
