#include "kmipkit.h"

int32_t kmipkit_0053_managed_object_from_item(
    uint32_t object_type,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_managed_object_from_item(object_type, item, out_item);
}

int32_t kmipkit_0053_key_block_from_item(
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_block_from_item(item, out_item);
}

int32_t kmipkit_0053_key_value_from_item(
    uint32_t format_raw,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_value_from_item(format_raw, item, out_item);
}

int32_t kmipkit_0053_key_material_from_item(
    uint32_t format_raw,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_material_from_item(format_raw, item, out_item);
}

int32_t kmipkit_0053_key_wrapping_data_from_item(
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_wrapping_data_from_item(item, out_item);
}

int32_t kmipkit_0053_managed_object_from_fields(
    uint32_t object_type,
    kmipkit_ttlv_item_t *const *fields,
    uint64_t field_count,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_managed_object_from_fields(object_type, fields, field_count, out_item);
}

int32_t kmipkit_0053_key_material_from_fields(
    uint32_t format_raw,
    kmipkit_ttlv_item_t *const *fields,
    uint64_t field_count,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_material_from_fields(format_raw, fields, field_count, out_item);
}
