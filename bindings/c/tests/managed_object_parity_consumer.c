#include "kmipkit.h"

/* This declaration deliberately lives in the consumer test until the public ABI is implemented. */
int32_t kmipkit_managed_value_field_read(
    uint32_t owner_kind,
    uint32_t context_raw,
    kmipkit_ttlv_item_t *item,
    uint32_t field_tag,
    uint32_t read_kind,
    uint8_t *out_present,
    uint8_t *out_item_type,
    uint32_t *out_enumeration,
    int32_t *out_integer,
    uint8_t *out_bytes,
    uint64_t byte_capacity,
    uint64_t *out_byte_length);

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
    kmipkit_ttlv_item_t **fields,
    uint64_t field_count,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_managed_object_from_fields(object_type, fields, field_count, out_item);
}

int32_t kmipkit_0053_key_material_from_fields(
    uint32_t format_raw,
    kmipkit_ttlv_item_t **fields,
    uint64_t field_count,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_material_from_fields(format_raw, fields, field_count, out_item);
}

int32_t kmipkit_0053_key_block_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_block_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_key_value_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_value_from_fields(1, fields, field_count, out_item);
}

int32_t kmipkit_0053_key_material_from_fields_default(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_material_from_fields(1, fields, field_count, out_item);
}

int32_t kmipkit_0053_key_wrapping_data_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_key_wrapping_data_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_certificate_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_certificate_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_symmetric_key_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_symmetric_key_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_public_key_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_public_key_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_private_key_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_private_key_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_split_key_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_split_key_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_secret_data_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_secret_data_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_opaque_object_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_opaque_object_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_pgp_key_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_pgp_key_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_certificate_request_from_fields(
    kmipkit_ttlv_item_t **fields, uint64_t field_count, kmipkit_ttlv_item_t **out_item) {
    return kmipkit_certificate_request_from_fields(fields, field_count, out_item);
}

int32_t kmipkit_0053_certificate_roundtrip(
    kmipkit_ttlv_item_t **fields, uint64_t field_count) {
    kmipkit_ttlv_item_t *certificate = NULL;
    kmipkit_ttlv_item_view_t *certificate_view = NULL;
    kmipkit_ttlv_value_view_t *certificate_value = NULL;
    kmipkit_ttlv_structure_view_t *children = NULL;
    int32_t status = kmipkit_certificate_from_fields(fields, field_count, &certificate);
    uint8_t seen = 0;
    uint64_t count = 0;
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_from_item(certificate, &certificate_view);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_value(certificate_view, &certificate_value);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_value_view_structure(certificate_value, &children);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_structure_view_item_count(children, &count);
    if (status != 0) goto cleanup;
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *field = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t raw_tag = 0;
        status = kmipkit_ttlv_structure_view_item_at(children, index, &field);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(field, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &raw_tag);
        if (status == 0) status = kmipkit_ttlv_item_view_value(field, &value);
        if (status == 0 && raw_tag == 0x0042001d) {
            uint32_t enumeration = 0;
            status = kmipkit_ttlv_value_view_enumeration(value, &enumeration);
            if (status == 0 && enumeration == 0x80000042) seen |= 1;
            else if (status == 0) status = -1;
        } else if (status == 0 && raw_tag == 0x0042001e) {
            uint64_t length = 0;
            status = kmipkit_ttlv_value_view_byte_length(value, &length);
            if (status == 0 && length != 4) status = -1;
            for (uint64_t byte_index = 0; status == 0 && byte_index < length; ++byte_index) {
                uint8_t byte = 0;
                status = kmipkit_ttlv_value_view_byte_at(value, byte_index, &byte);
                if (status == 0 && byte != (uint8_t) "cert"[byte_index]) status = -1;
            }
            if (status == 0) seen |= 2;
        }
        if (value != NULL) kmipkit_ttlv_value_view_release(value);
        if (tag != NULL) kmipkit_tag_release(tag);
        if (field != NULL) kmipkit_ttlv_item_view_release(field);
        if (status != 0) goto cleanup;
    }
    if (seen != 3) status = -1;

cleanup:
    if (children != NULL) kmipkit_ttlv_structure_view_release(children);
    if (certificate_value != NULL) kmipkit_ttlv_value_view_release(certificate_value);
    if (certificate_view != NULL) kmipkit_ttlv_item_view_release(certificate_view);
    if (certificate != NULL) kmipkit_ttlv_item_release(certificate);
    return status;
}

int32_t kmipkit_0053_managed_field_reader_roundtrip(
    kmipkit_ttlv_item_t **fields, uint64_t field_count) {
    kmipkit_ttlv_item_t *certificate = NULL;
    int32_t status = kmipkit_certificate_from_fields(fields, field_count, &certificate);
    uint8_t present = 0;
    uint8_t item_type = 0;
    uint32_t enumeration = 0;
    uint8_t value_bytes[4] = {0, 0, 0, 0};
    uint64_t byte_length = 0;
    if (status != 0) goto cleanup;
    status = kmipkit_managed_value_field_read(
        1, 1, certificate, 0x0042001d, 3,
        &present, &item_type, &enumeration, NULL, NULL, 0, NULL);
    if (status != 0 || present != 1 || item_type != 5 || enumeration != 0x80000042) {
        if (status == 0) status = -1;
        goto cleanup;
    }
    status = kmipkit_managed_value_field_read(
        1, 1, certificate, 0x0042001e, 5,
        &present, &item_type, NULL, NULL, value_bytes, sizeof(value_bytes), &byte_length);
    if (status != 0 || present != 1 || item_type != 8 || byte_length != sizeof(value_bytes)
        || value_bytes[0] != 'c' || value_bytes[1] != 'e'
        || value_bytes[2] != 'r' || value_bytes[3] != 't') {
        if (status == 0) status = -1;
    }

cleanup:
    if (certificate != NULL) kmipkit_ttlv_item_release(certificate);
    return status;
}

int32_t kmipkit_0053_managed_field_reader_check(
    uint32_t owner_kind, uint32_t context_raw, kmipkit_ttlv_item_t *item,
    uint32_t field_tag, uint32_t read_kind, uint8_t expected_item_type,
    uint32_t expected_enumeration, int32_t expected_integer,
    const uint8_t *expected_bytes, uint64_t expected_byte_length,
    int32_t expected_status) {
    uint8_t present = 0;
    uint8_t item_type = 0;
    uint32_t enumeration = 0;
    int32_t integer = 0;
    uint64_t byte_length = 0;
    int32_t status = kmipkit_managed_value_field_read(
        owner_kind, context_raw, item, field_tag, read_kind,
        &present, &item_type, &enumeration, &integer, NULL, 0, &byte_length);
    if ((expected_status < 0 && status == 0)
        || (expected_status >= 0 && status != expected_status)) return -1;
    if (expected_status != 0) return 0;
    if (read_kind == 1 && present != expected_enumeration) return -1;
    if (read_kind != 1 && present != 1) return -1;
    if (read_kind == 2 && item_type != expected_item_type) return -1;
    if (read_kind == 3 && (item_type != expected_item_type || enumeration != expected_enumeration))
        return -1;
    if (read_kind == 4 && (item_type != expected_item_type || integer != expected_integer))
        return -1;
    if (read_kind == 5) {
        if (item_type != expected_item_type || byte_length != expected_byte_length) return -1;
        uint8_t bytes[256] = {0};
        if (expected_byte_length > sizeof(bytes)) return -1;
        status = kmipkit_managed_value_field_read(
            owner_kind, context_raw, item, field_tag, read_kind,
            &present, &item_type, &enumeration, &integer,
            bytes, sizeof(bytes), &byte_length);
        if (status != 0 || byte_length != expected_byte_length) return -1;
        for (uint64_t index = 0; index < expected_byte_length; ++index) {
            if (bytes[index] != expected_bytes[index]) return -1;
        }
    }
    return 0;
}

int32_t kmipkit_0053_managed_field_reader_rejects_short_buffer(
    uint32_t owner_kind, uint32_t context_raw, kmipkit_ttlv_item_t *item,
    uint32_t field_tag, uint64_t expected_byte_length) {
    uint8_t present = 0;
    uint64_t byte_length = 0;
    int32_t status = kmipkit_managed_value_field_read(
        owner_kind, context_raw, item, field_tag, 5,
        &present, NULL, NULL, NULL, NULL, 0, &byte_length);
    if (status != 0 || present != 1 || byte_length != expected_byte_length || byte_length == 0)
        return -1;
    uint8_t bytes[256];
    for (uint64_t index = 0; index < sizeof(bytes); ++index) bytes[index] = 0xa5;
    status = kmipkit_managed_value_field_read(
        owner_kind, context_raw, item, field_tag, 5,
        &present, NULL, NULL, NULL, bytes, byte_length - 1, &byte_length);
    if (status == 0 || byte_length != expected_byte_length) return -1;
    for (uint64_t index = 0; index < sizeof(bytes); ++index) {
        if (bytes[index] != 0xa5) return -1;
    }
    return 0;
}

static int32_t check_nested_item(
    kmipkit_ttlv_item_t *item, uint32_t expected_tag, uint8_t expected_type) {
    kmipkit_ttlv_item_view_t *view = NULL;
    kmipkit_tag_t *tag = NULL;
    uint32_t raw_tag = 0;
    uint8_t item_type = 0;
    int32_t status = kmipkit_ttlv_item_view_from_item(item, &view);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_tag(view, &tag);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_tag_value(tag, &raw_tag);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_type(view, &item_type);
    if (status != 0) goto cleanup;
    if (raw_tag != expected_tag || item_type != expected_type) status = -1;

cleanup:
    if (tag != NULL) kmipkit_tag_release(tag);
    if (view != NULL) kmipkit_ttlv_item_view_release(view);
    return status;
}

static int32_t check_nested_structure_children(
    kmipkit_ttlv_item_t *item, const uint32_t *expected_tags,
    const uint8_t *expected_types, const uint8_t *const *expected_bytes,
    const uint64_t *expected_lengths, uint64_t expected_count) {
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    kmipkit_ttlv_structure_view_t *structure = NULL;
    int32_t status = kmipkit_ttlv_item_view_from_item(item, &item_view);
    uint8_t item_type = 0;
    uint64_t count = 0;
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_type(item_view, &item_type);
    if (status != 0) goto cleanup;
    if (item_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        status = -1;
        goto cleanup;
    }
    status = kmipkit_ttlv_item_view_value(item_view, &value_view);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_value_view_structure(value_view, &structure);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_structure_view_item_count(structure, &count);
    if (status != 0 || count != expected_count) goto cleanup;

    for (uint64_t index = 0; index < expected_count; ++index) {
        kmipkit_ttlv_item_view_t *child = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *child_value = NULL;
        uint32_t raw_tag = 0;
        uint8_t child_type = 0;
        status = kmipkit_ttlv_structure_view_item_at(structure, index, &child);
        if (status != 0) goto child_cleanup;
        status = kmipkit_ttlv_item_view_tag(child, &tag);
        if (status != 0) goto child_cleanup;
        status = kmipkit_ttlv_tag_value(tag, &raw_tag);
        if (status != 0) goto child_cleanup;
        status = kmipkit_ttlv_item_view_type(child, &child_type);
        if (status != 0) goto child_cleanup;
        if (raw_tag != expected_tags[index] || child_type != expected_types[index]) {
            status = -1;
            goto child_cleanup;
        }
        if (expected_lengths != NULL && expected_lengths[index] > 0) {
            uint64_t length = 0;
            status = kmipkit_ttlv_item_view_value(child, &child_value);
            if (status != 0) goto child_cleanup;
            status = kmipkit_ttlv_value_view_byte_length(child_value, &length);
            if (status != 0 || length != expected_lengths[index]) goto child_cleanup;
            for (uint64_t byte_index = 0; byte_index < length; ++byte_index) {
                uint8_t byte = 0;
                status = kmipkit_ttlv_value_view_byte_at(child_value, byte_index, &byte);
                if (status != 0 || byte != expected_bytes[index][byte_index]) goto child_cleanup;
            }
        }

child_cleanup:
        if (child_value != NULL) kmipkit_ttlv_value_view_release(child_value);
        if (tag != NULL) kmipkit_tag_release(tag);
        if (child != NULL) kmipkit_ttlv_item_view_release(child);
        if (status != 0) goto cleanup;
    }

cleanup:
    if (structure != NULL) kmipkit_ttlv_structure_view_release(structure);
    if (value_view != NULL) kmipkit_ttlv_value_view_release(value_view);
    if (item_view != NULL) kmipkit_ttlv_item_view_release(item_view);
    return status;
}

static int32_t params_0053_create_item(
    uint32_t raw_tag, uint8_t kind, uint32_t enumeration,
    const uint8_t *bytes, uint64_t byte_length,
    kmipkit_codec_limits_t *limits, kmipkit_ttlv_item_t **out_item) {
    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0 && kind == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION)
        status = kmipkit_ttlv_value_enumeration(enumeration, &value);
    else if (status == 0 && kind == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING)
        status = kmipkit_ttlv_value_text_string(limits, bytes, byte_length, &value);
    else if (status == 0 && kind == KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING)
        status = kmipkit_ttlv_value_byte_string(limits, bytes, byte_length, &value);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = NULL;
    if (value != NULL) kmipkit_ttlv_value_release(value);
    if (tag != NULL) kmipkit_tag_release(tag);
    if (raw != NULL) kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t params_0053_append(
    kmipkit_ttlv_structure_t **structure, kmipkit_ttlv_item_t **item,
    kmipkit_codec_limits_t *limits) {
    kmipkit_ttlv_structure_t *updated = NULL;
    int32_t status = kmipkit_ttlv_structure_with_item(*structure, *item, limits, &updated);
    if (status == 0) {
        *structure = updated;
        *item = NULL;
    }
    return status;
}

static int32_t params_0053_typed_parameters_item(
    uint32_t hashing_algorithm, kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_item_t **out_item) {
    static const uint8_t vendor[] = {0xca, 0xfe};
    kmipkit_ttlv_structure_t *source = NULL;
    kmipkit_ttlv_structure_t *raw_parameters = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_cryptographic_parameters_t *typed = NULL;
    kmipkit_ttlv_item_t *hash = NULL;
    kmipkit_ttlv_item_t *extension = NULL;
    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    int32_t status = kmipkit_ttlv_structure_create(&source);
    if (status == 0) status = params_0053_create_item(
        0x00420038, KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, hashing_algorithm,
        NULL, 0, limits, &hash);
    if (status == 0) status = params_0053_append(&source, &hash, limits);
    if (status == 0) status = params_0053_create_item(
        0x00540031, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING, 0,
        vendor, sizeof(vendor), limits, &extension);
    if (status == 0) status = params_0053_append(&source, &extension, limits);
    if (status == 0) status = kmipkit_cryptographic_parameters_from_ttlv(source, &typed);
    if (status == 0) {
        status = kmipkit_cryptographic_parameters_into_ttlv(typed, &raw_parameters);
        typed = NULL;
    }
    if (status == 0) status = kmipkit_ttlv_value_structure(raw_parameters, limits, &value);
    if (status == 0) raw_parameters = NULL;
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(0x0042002b, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = NULL;

    if (typed != NULL) kmipkit_cryptographic_parameters_release(typed);
    if (value != NULL) kmipkit_ttlv_value_release(value);
    if (tag != NULL) kmipkit_tag_release(tag);
    if (raw != NULL) kmipkit_raw_tag_release(raw);
    if (source != NULL) kmipkit_ttlv_structure_release(source);
    if (raw_parameters != NULL) kmipkit_ttlv_structure_release(raw_parameters);
    if (hash != NULL) kmipkit_ttlv_item_release(hash);
    if (extension != NULL) kmipkit_ttlv_item_release(extension);
    return status;
}

static int32_t params_0053_key_information_item(
    uint32_t information_tag, const uint8_t *identifier, uint64_t identifier_length,
    uint32_t hashing_algorithm, kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_item_t **out_item) {
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *identifier_item = NULL;
    kmipkit_ttlv_item_t *parameters_item = NULL;
    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    int32_t status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = params_0053_create_item(
        0x00420094, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, 0,
        identifier, identifier_length, limits, &identifier_item);
    if (status == 0) status = params_0053_append(&structure, &identifier_item, limits);
    if (status == 0) status = params_0053_typed_parameters_item(
        hashing_algorithm, limits, &parameters_item);
    if (status == 0) status = params_0053_append(&structure, &parameters_item, limits);
    if (status == 0) status = kmipkit_ttlv_value_structure(structure, limits, &value);
    if (status == 0) structure = NULL;
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(information_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = NULL;

    if (value != NULL) kmipkit_ttlv_value_release(value);
    if (tag != NULL) kmipkit_tag_release(tag);
    if (raw != NULL) kmipkit_raw_tag_release(raw);
    if (structure != NULL) kmipkit_ttlv_structure_release(structure);
    if (identifier_item != NULL) kmipkit_ttlv_item_release(identifier_item);
    if (parameters_item != NULL) kmipkit_ttlv_item_release(parameters_item);
    return status;
}

static int32_t params_0053_check_key_information(
    kmipkit_ttlv_item_t *information, const uint8_t *expected_identifier,
    uint64_t expected_identifier_length, uint32_t expected_hashing_algorithm) {
    const uint32_t information_tags[] = {0x00420094, 0x0042002b};
    const uint8_t information_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE};
    const uint8_t *information_values[] = {expected_identifier, NULL};
    const uint64_t information_lengths[] = {expected_identifier_length, 0};
    const uint32_t parameter_tags[] = {0x00420038, 0x00540031};
    const uint8_t parameter_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING};
    static const uint8_t vendor[] = {0xca, 0xfe};
    kmipkit_ttlv_item_view_t *information_view = NULL;
    kmipkit_ttlv_value_view_t *information_value = NULL;
    kmipkit_ttlv_structure_view_t *information_fields = NULL;
    kmipkit_ttlv_item_view_t *parameters_field = NULL;
    kmipkit_tag_t *parameters_tag = NULL;
    kmipkit_ttlv_value_view_t *parameters_value = NULL;
    kmipkit_ttlv_structure_view_t *parameter_fields = NULL;
    uint32_t raw_tag = 0;
    uint8_t item_type = 0;
    uint64_t count = 0;
    int32_t status = check_nested_structure_children(
        information, information_tags, information_types,
        information_values, information_lengths, 2);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_from_item(information, &information_view);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_value(information_view, &information_value);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_value_view_structure(information_value, &information_fields);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_structure_view_item_at(information_fields, 1, &parameters_field);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_item_view_tag(parameters_field, &parameters_tag);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_tag_value(parameters_tag, &raw_tag);
    if (status != 0 || raw_tag != 0x0042002b) {
        if (status == 0) status = -1;
        goto cleanup;
    }
    status = kmipkit_ttlv_item_view_type(parameters_field, &item_type);
    if (status != 0 || item_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        if (status == 0) status = -1;
        goto cleanup;
    }
    status = kmipkit_ttlv_item_view_value(parameters_field, &parameters_value);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_value_view_structure(parameters_value, &parameter_fields);
    if (status != 0) goto cleanup;
    status = kmipkit_ttlv_structure_view_item_count(parameter_fields, &count);
    if (status != 0 || count != 2) {
        if (status == 0) status = -1;
        goto cleanup;
    }

    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *field = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t field_tag = 0;
        uint8_t field_type = 0;
        status = kmipkit_ttlv_structure_view_item_at(parameter_fields, index, &field);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(field, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &field_tag);
        if (status == 0) status = kmipkit_ttlv_item_view_type(field, &field_type);
        if (status == 0 && (field_tag != parameter_tags[index]
                || field_type != parameter_types[index])) status = -1;
        if (status == 0) status = kmipkit_ttlv_item_view_value(field, &value);
        if (status == 0 && index == 0) {
            uint32_t enumeration = 0;
            status = kmipkit_ttlv_value_view_enumeration(value, &enumeration);
            if (status == 0 && enumeration != expected_hashing_algorithm) status = -1;
        } else if (status == 0) {
            uint64_t length = 0;
            status = kmipkit_ttlv_value_view_byte_length(value, &length);
            if (status == 0 && length != sizeof(vendor)) status = -1;
            for (uint64_t byte_index = 0; status == 0 && byte_index < length; ++byte_index) {
                uint8_t byte = 0;
                status = kmipkit_ttlv_value_view_byte_at(value, byte_index, &byte);
                if (status == 0 && byte != vendor[byte_index]) status = -1;
            }
        }
        if (value != NULL) kmipkit_ttlv_value_view_release(value);
        if (tag != NULL) kmipkit_tag_release(tag);
        if (field != NULL) kmipkit_ttlv_item_view_release(field);
        if (status != 0) goto cleanup;
    }

cleanup:
    if (parameter_fields != NULL) kmipkit_ttlv_structure_view_release(parameter_fields);
    if (parameters_value != NULL) kmipkit_ttlv_value_view_release(parameters_value);
    if (parameters_tag != NULL) kmipkit_tag_release(parameters_tag);
    if (parameters_field != NULL) kmipkit_ttlv_item_view_release(parameters_field);
    if (information_fields != NULL) kmipkit_ttlv_structure_view_release(information_fields);
    if (information_value != NULL) kmipkit_ttlv_value_view_release(information_value);
    if (information_view != NULL) kmipkit_ttlv_item_view_release(information_view);
    return status;
}

int32_t kmipkit_0053_key_information_parameters_consumer(void) {
    static const uint8_t encryption_id[] = "encryption-key";
    static const uint8_t mac_id[] = "mac-key";
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_item_t *method = NULL;
    kmipkit_ttlv_item_t *encryption = NULL;
    kmipkit_ttlv_item_t *mac = NULL;
    kmipkit_ttlv_item_t *wrapping = NULL;
    kmipkit_ttlv_item_t *nested_encryption = NULL;
    kmipkit_ttlv_item_t *nested_mac = NULL;
    kmipkit_ttlv_item_t *fields[3] = {NULL, NULL, NULL};
    uint8_t present = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = params_0053_create_item(
        0x0042009e, KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, 1, NULL, 0, limits, &method);
    if (status == 0) status = params_0053_key_information_item(
        0x00420036, encryption_id, sizeof(encryption_id) - 1,
        UINT32_C(0x80000033), limits, &encryption);
    if (status == 0) status = params_0053_key_information_item(
        0x0042004e, mac_id, sizeof(mac_id) - 1,
        UINT32_C(0x80000044), limits, &mac);
    fields[0] = method;
    fields[1] = encryption;
    fields[2] = mac;
    if (status == 0) status = kmipkit_key_wrapping_data_from_fields(fields, 3, &wrapping);
    if (status == 0) status = kmipkit_managed_value_field_item_read(
        5, 0, wrapping, 0x00420036, &present, &nested_encryption);
    if (status == 0 && (present != 1 || nested_encryption == NULL)) status = -1;
    if (status == 0) status = params_0053_check_key_information(
        nested_encryption, encryption_id, sizeof(encryption_id) - 1, UINT32_C(0x80000033));
    present = 0;
    if (status == 0) status = kmipkit_managed_value_field_item_read(
        5, 0, wrapping, 0x0042004e, &present, &nested_mac);
    if (status == 0 && (present != 1 || nested_mac == NULL)) status = -1;
    if (status == 0) status = params_0053_check_key_information(
        nested_mac, mac_id, sizeof(mac_id) - 1, UINT32_C(0x80000044));

    kmipkit_ttlv_item_release(nested_encryption);
    kmipkit_ttlv_item_release(nested_mac);
    kmipkit_ttlv_item_release(wrapping);
    kmipkit_ttlv_item_release(method);
    kmipkit_ttlv_item_release(encryption);
    kmipkit_ttlv_item_release(mac);
    kmipkit_codec_limits_release(limits);
    return status;
}

int32_t kmipkit_0053_managed_nested_nodes_roundtrip(
    kmipkit_ttlv_item_t *key_block,
    kmipkit_ttlv_item_t *wrapped_key_block) {
    const uint32_t key_value_children[] = {0x00420043, 0x00420125};
    const uint8_t key_value_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE};
    const uint8_t vendor_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING};
    const uint32_t vendor_tags[] = {0x00540031, 0x00540031};
    const uint8_t vendor_one[] = "vendor-one";
    const uint8_t vendor_two[] = "vendor-two";
    const uint8_t *vendor_values[] = {vendor_one, vendor_two};
    const uint64_t vendor_lengths[] = {sizeof(vendor_one) - 1, sizeof(vendor_two) - 1};
    const uint32_t encryption_tags[] = {0x00420094, 0x00540031, 0x00540031};
    const uint8_t encryption_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING};
    const uint8_t encryption_id[] = "encryption-id";
    const uint8_t *encryption_values[] = {encryption_id, vendor_one, vendor_two};
    const uint64_t encryption_lengths[] = {
        sizeof(encryption_id) - 1, sizeof(vendor_one) - 1, sizeof(vendor_two) - 1};
    const uint8_t mac_id[] = "mac-id";
    const uint8_t *mac_values[] = {mac_id, vendor_one, vendor_two};
    const uint64_t mac_lengths[] = {
        sizeof(mac_id) - 1, sizeof(vendor_one) - 1, sizeof(vendor_two) - 1};
    kmipkit_ttlv_item_t *key_value = NULL;
    kmipkit_ttlv_item_t *attributes = NULL;
    kmipkit_ttlv_item_t *key_wrapping_data = NULL;
    kmipkit_ttlv_item_t *encryption_info = NULL;
    kmipkit_ttlv_item_t *mac_info = NULL;
    kmipkit_ttlv_item_t *wrapped_value = NULL;
    uint8_t present = 0;
    uint8_t item_type = 0;
    uint8_t value_bytes[8] = {0};
    uint64_t value_length = 0;
    int32_t status;

    status = kmipkit_managed_value_field_item_read(
        2, 0, key_block, 0x00420045, &present, &key_value);
    if (status != 0 || present != 1 || key_value == NULL) goto cleanup;
    status = check_nested_item(key_value, 0x00420045, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE);
    if (status != 0) goto cleanup;
    status = check_nested_structure_children(
        key_value, key_value_children, key_value_types, NULL, NULL, 2);
    if (status != 0) goto cleanup;

    status = kmipkit_managed_value_field_read(
        3, 1, key_value, 0x00420043, 5, &present, &item_type,
        NULL, NULL, value_bytes, sizeof(value_bytes), &value_length);
    if (status != 0 || present != 1 || item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING
            || value_length != 8 || value_bytes[0] != 'k' || value_bytes[7] != 'a') {
        if (status == 0) status = -1;
        goto cleanup;
    }
    status = kmipkit_managed_value_field_item_read(
        3, 1, key_value, 0x00420125, &present, &attributes);
    if (status != 0 || present != 1 || attributes == NULL) goto cleanup;
    status = check_nested_item(attributes, 0x00420125, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE);
    if (status != 0) goto cleanup;
    status = check_nested_structure_children(
        attributes, vendor_tags, vendor_types, vendor_values, vendor_lengths, 2);
    if (status != 0) goto cleanup;

    status = kmipkit_managed_value_field_item_read(
        2, 0, wrapped_key_block, 0x00420046, &present, &key_wrapping_data);
    if (status != 0 || present != 1 || key_wrapping_data == NULL) goto cleanup;
    status = check_nested_item(
        key_wrapping_data, 0x00420046, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE);
    if (status != 0) goto cleanup;

    status = kmipkit_managed_value_field_item_read(
        2, 0, key_block, 0x00540002, &present, &wrapped_value);
    if (status != 0 || present != 0 || wrapped_value != NULL) goto cleanup;

    status = kmipkit_managed_value_field_item_read(
        5, 0, key_wrapping_data, 0x00420036, &present, &encryption_info);
    if (status != 0 || present != 1 || encryption_info == NULL) goto cleanup;
    status = check_nested_item(
        encryption_info, 0x00420036, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE);
    if (status != 0) goto cleanup;
    status = check_nested_structure_children(
        encryption_info, encryption_tags, encryption_types,
        encryption_values, encryption_lengths, 3);
    if (status != 0) goto cleanup;

    status = kmipkit_managed_value_field_item_read(
        5, 0, key_wrapping_data, 0x0042004e, &present, &mac_info);
    if (status != 0 || present != 1 || mac_info == NULL) goto cleanup;
    status = check_nested_item(mac_info, 0x0042004e, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE);
    if (status != 0) goto cleanup;
    status = check_nested_structure_children(
        mac_info, encryption_tags, encryption_types, mac_values, mac_lengths, 3);
    if (status != 0) goto cleanup;

    status = kmipkit_managed_value_field_item_read(
        2, 0, wrapped_key_block, 0x00420045, &present, &wrapped_value);
    if (status != 0 || present != 1 || wrapped_value == NULL) goto cleanup;
    status = check_nested_item(wrapped_value, 0x00420045, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING);
    if (status != 0) goto cleanup;
    value_length = 0;
    status = kmipkit_managed_value_field_read(
        2, 0, wrapped_key_block, 0x00420045, 5, &present, &item_type,
        NULL, NULL, value_bytes, sizeof(value_bytes), &value_length);
    if (status == 0 && (present != 1 || item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING
            || value_length != 7 || value_bytes[0] != 'w' || value_bytes[6] != 'd')) {
        status = -1;
    }

cleanup:
    if (key_value != NULL) kmipkit_ttlv_item_release(key_value);
    if (attributes != NULL) kmipkit_ttlv_item_release(attributes);
    if (key_wrapping_data != NULL) kmipkit_ttlv_item_release(key_wrapping_data);
    if (encryption_info != NULL) kmipkit_ttlv_item_release(encryption_info);
    if (mac_info != NULL) kmipkit_ttlv_item_release(mac_info);
    if (wrapped_value != NULL) kmipkit_ttlv_item_release(wrapped_value);
    return status;
}
