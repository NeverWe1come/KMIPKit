#include "generic_wire.h"

#include <string.h>

static void put_u32(uint8_t *output, uint32_t value) {
    output[0] = (uint8_t)(value >> 24);
    output[1] = (uint8_t)(value >> 16);
    output[2] = (uint8_t)(value >> 8);
    output[3] = (uint8_t)value;
}

static bool scalar_number(kmipkit_ttlv_item_type_t type, kmipkit_ttlv_value_view_t *value,
                          uint64_t *number, uint32_t *length) {
    int32_t integer = 0;
    uint32_t unsigned_integer = 0;
    int64_t long_integer = 0;
    uint8_t bit = 0;
    int32_t status;
    switch (type) {
        case KMIPKIT_TTLV_ITEM_TYPE_INTEGER:
            status = kmipkit_ttlv_value_view_integer(value, &integer);
            *number = (uint32_t)integer;
            *length = 4;
            return status == 0;
        case KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION:
        case KMIPKIT_TTLV_ITEM_TYPE_INTERVAL:
            status = type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION
                ? kmipkit_ttlv_value_view_enumeration(value, &unsigned_integer)
                : kmipkit_ttlv_value_view_interval(value, &unsigned_integer);
            *number = unsigned_integer;
            *length = 4;
            return status == 0;
        case KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER:
        case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME:
        case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED:
            if (type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER)
                status = kmipkit_ttlv_value_view_long_integer(value, &long_integer);
            else if (type == KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME)
                status = kmipkit_ttlv_value_view_date_time(value, &long_integer);
            else
                status = kmipkit_ttlv_value_view_date_time_extended(value, &long_integer);
            *number = (uint64_t)long_integer;
            *length = 8;
            return status == 0;
        case KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN:
            status = kmipkit_ttlv_value_view_boolean(value, &bit);
            *number = bit;
            *length = 8;
            return status == 0;
        default:
            return false;
    }
}

/* Serialize the public KMIP 2.x TTLV model. */
bool encode_generic_item(kmipkit_ttlv_item_view_t *item,
                                 uint8_t *wire, size_t capacity, size_t *offset) {
    if (*offset > capacity || capacity - *offset < 8) return false;
    size_t header = *offset;
    *offset += 8;
    uint8_t type = 0;
    uint32_t tag_value = 0;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    kmipkit_ttlv_structure_view_t *structure = NULL;
    bool valid = kmipkit_ttlv_item_view_tag(item, &tag) == 0 &&
        kmipkit_ttlv_tag_value(tag, &tag_value) == 0 &&
        kmipkit_ttlv_item_view_type(item, &type) == 0 &&
        kmipkit_ttlv_item_view_value(item, &value) == 0;
    uint32_t body_length = 0;
    if (valid && type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        uint64_t length = 0;
        valid = kmipkit_ttlv_value_view_structure(value, &structure) == 0 &&
            kmipkit_ttlv_structure_view_item_count(structure, &length) == 0;
        for (uint64_t index = 0; valid && index < length; ++index) {
            kmipkit_ttlv_item_view_t *child = NULL;
            valid = kmipkit_ttlv_structure_view_item_at(structure, index, &child) == 0 &&
                encode_generic_item(child, wire, capacity, offset);
            if (child) kmipkit_ttlv_item_view_release(child);
        }
        size_t length_on_wire = *offset - header - 8;
        valid = valid && length_on_wire <= UINT32_MAX;
        body_length = (uint32_t)length_on_wire;
    } else if (valid && (type == KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
                         type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING ||
                         type == KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER)) {
        uint64_t length = 0;
        valid = kmipkit_ttlv_value_view_byte_length(value, &length) == 0 &&
            length <= UINT32_MAX && length <= ((capacity - *offset) & ~(size_t)7) &&
            (type != KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER || (length != 0 && length % 8 == 0));
        if (valid) {
            body_length = (uint32_t)length;
            for (uint64_t index = 0; valid && index < length; ++index)
                valid = kmipkit_ttlv_value_view_byte_at(value, index, wire + *offset + index) == 0;
            size_t padded = ((size_t)length + 7) & ~(size_t)7;
            memset(wire + *offset + length, 0, padded - (size_t)length);
            *offset += padded;
        }
    } else if (valid && capacity - *offset >= 8) {
        uint64_t number = 0;
        valid = scalar_number(type, value, &number, &body_length);
        if (valid && body_length == 8) {
            put_u32(wire + *offset, (uint32_t)(number >> 32));
            put_u32(wire + *offset + 4, (uint32_t)number);
        } else if (valid) {
            put_u32(wire + *offset, (uint32_t)number);
            put_u32(wire + *offset + 4, 0);
        }
        *offset += 8;
    } else {
        valid = false;
    }
    put_u32(wire + header, (tag_value << 8) | type);
    put_u32(wire + header + 4, body_length);
    if (structure) kmipkit_ttlv_structure_view_release(structure);
    if (value) kmipkit_ttlv_value_view_release(value);
    if (tag) kmipkit_tag_release(tag);
    return valid;
}
