#include "kmipkit.h"

#include <stdint.h>

int main(void) {
    kmipkit_client_batch_item_t *ping = 0;
    uint32_t operation = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t item_count = UINT64_MAX;
    int32_t status = kmipkit_client_batch_item_ping(&ping);
    if (status == 0) status = kmipkit_client_batch_item_operation(ping, &operation);
    if (status == 0 && operation != UINT32_C(0x3b)) status = 4;
    if (status == 0) status = kmipkit_ttlv_structure_create(&payload);
    if (status == 0) status = kmipkit_ttlv_structure_view(payload, &view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &item_count);
    if (status == 0 && item_count != 0) status = 4;
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_client_batch_item_release(ping);
    return status == 0 ? 0 : 1;
}
