#include "kmipkit.h"

int32_t kmipkit_0053_typed_attribute_from_item(
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_item(expected_tag, item, out_item);
}

int32_t kmipkit_0053_typed_attribute_from_value(
    uint32_t expected_tag,
    kmipkit_ttlv_value_t *value,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_value(expected_tag, value, out_item);
}

int32_t kmipkit_0053_typed_attribute_value(
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_value_t **out_value) {
    return kmipkit_typed_attribute_value(expected_tag, item, out_value);
}

int32_t kmipkit_0053_attribute_set_try_push_typed(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item) {
    return kmipkit_attribute_set_try_push_typed(attributes, expected_tag, item);
}

int32_t kmipkit_0053_attribute_set_typed_count(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    uint64_t *out_count) {
    return kmipkit_attribute_set_typed_count(attributes, expected_tag, out_count);
}

int32_t kmipkit_0053_attribute_set_typed_at(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    uint64_t occurrence,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_attribute_set_typed_at(attributes, expected_tag, occurrence, out_item);
}
