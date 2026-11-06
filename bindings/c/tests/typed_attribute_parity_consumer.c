#include "kmipkit.h"

int32_t kmipkit_0053_typed_attribute_from_item(
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_item(expected_tag, item, out_item);
}
