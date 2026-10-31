#include <stdint.h>

#include "kmipkit.h"

static const uint8_t NONCE_ID[] = {0x10, 0x00, 0xff};
static const uint8_t NONCE_VALUE[] = {0x80, 0x01, 0x7f, 0x00};

static int32_t append_byte_string(
        kmipkit_ttlv_structure_t **structure, kmipkit_codec_limits_t *limits,
        uint32_t raw_tag, const uint8_t *bytes, uint64_t length) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *next = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(limits, bytes, length, &value);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        tag = 0;
        value = 0;
        status = kmipkit_ttlv_structure_with_item(*structure, item, limits, &next);
    }
    if (next != 0) {
        kmipkit_ttlv_structure_release(*structure);
        kmipkit_ttlv_item_release(item);
        *structure = next;
    }
    kmipkit_raw_tag_release(raw);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_item_release(item);
    return status;
}

static int32_t structure_matches(
        kmipkit_ttlv_structure_view_t *view, uint64_t *out_item_count) {
    static const uint32_t expected_tags[] = {0x004200c9, 0x004200ca};
    static const uint8_t *expected_values[] = {NONCE_ID, NONCE_VALUE};
    static const uint64_t expected_lengths[] = {sizeof(NONCE_ID), sizeof(NONCE_VALUE)};
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != 2) return status == 0 ? 1 : status;
    for (uint64_t index = 0; index < count; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        kmipkit_ttlv_value_view_t *value = 0;
        uint32_t actual_tag = 0;
        uint64_t length = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual_tag);
        if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
        if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &length);
        if (status == 0 && (actual_tag != expected_tags[index]
                || length != expected_lengths[index])) status = 1;
        for (uint64_t byte_index = 0; status == 0 && byte_index < length; byte_index++) {
            uint8_t actual_byte = 0;
            status = kmipkit_ttlv_value_view_byte_at(value, byte_index, &actual_byte);
            if (status == 0 && actual_byte != expected_values[index][byte_index]) status = 1;
        }
        kmipkit_tag_release(tag);
        kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        if (status != 0) return status;
    }
    *out_item_count = count;
    return 0;
}

int32_t kmipkit_nonce_c_consumer_round_trip(uint64_t *out_item_count) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_nonce_t *nonce = 0;
    kmipkit_ttlv_structure_view_t *source_view = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    uint64_t count = 0;
    int32_t status = kmipkit_codec_limits_create(4096, 8, 32, &limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) {
        status = append_byte_string(&structure, limits, 0x004200c9, NONCE_ID, sizeof(NONCE_ID));
    }
    if (status == 0) {
        status = append_byte_string(
                &structure, limits, 0x004200ca, NONCE_VALUE, sizeof(NONCE_VALUE));
    }
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &source_view);
    if (status == 0) {
        int32_t shared_owner_status = kmipkit_nonce_from_ttlv(structure, &nonce);
        if (shared_owner_status != 11) status = 1;
    }
    if (status == 0) status = structure_matches(source_view, &count);
    kmipkit_ttlv_structure_view_release(source_view);
    source_view = 0;
    if (status == 0) status = kmipkit_nonce_from_ttlv(structure, &nonce);
    if (status == 0) {
        structure = 0;
        status = kmipkit_nonce_ttlv_view(nonce, &view);
    }
    if (status == 0) status = structure_matches(view, &count);
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_nonce_into_ttlv(nonce, &restored);
    if (status == 0) {
        nonce = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = structure_matches(restored_view, &count);
    if (status == 0) *out_item_count = count;
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_nonce_release(nonce);
    kmipkit_ttlv_structure_view_release(source_view);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    return status;
}
