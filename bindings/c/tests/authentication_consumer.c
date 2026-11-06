#include <stdint.h>
#include <stddef.h>
#include <string.h>

#include "kmipkit.h"

int32_t kmipkit_parameters_authentication_c_consumer_parameters(void) {
    static const uint8_t p_source[] = {0x51, 0x00, 0xff};
    static const uint32_t expected_tags[] = {
        0x00420011, 0x0042005f, 0x00420038, 0x00420083, 0x004200ae,
        0x00420028, 0x004200c5, 0x004200cd, 0x004200ce, 0x004200cf,
        0x004200d2, 0x004200d0, 0x004200d1, 0x00420100, 0x00420101,
        0x00420102, 0x00420103, 0x00420104
    };
    kmipkit_cryptographic_parameters_fields_v1_t fields = {0};
    kmipkit_cryptographic_parameters_t *parameters = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    int32_t status = 0;

    fields.struct_size = (uint32_t)sizeof(fields);
    fields.abi_version = 1;
    fields.block_cipher_mode_present = 1;
    fields.block_cipher_mode = 0xe1234567;
    fields.padding_method_present = 1;
    fields.padding_method = 0x80000001;
    fields.hashing_algorithm_present = 1;
    fields.hashing_algorithm = 0xf1234567;
    fields.key_role_type_present = 1;
    fields.key_role_type = 0x80001234;
    fields.digital_signature_algorithm_present = 1;
    fields.digital_signature_algorithm = 0x80005678;
    fields.cryptographic_algorithm_present = 1;
    fields.cryptographic_algorithm = 0xe1230001;
    fields.random_iv_present = 1;
    fields.random_iv = 1;
    fields.iv_length_present = 1;
    fields.iv_length = 8;
    fields.tag_length_present = 1;
    fields.tag_length = 16;
    fields.fixed_field_length_present = 1;
    fields.fixed_field_length = 2;
    fields.invocation_field_length_present = 1;
    fields.invocation_field_length = 4;
    fields.counter_length_present = 1;
    fields.counter_length = 32;
    fields.initial_counter_value_present = 1;
    fields.initial_counter_value = 1;
    fields.salt_length_present = 1;
    fields.salt_length = 16;
    fields.mask_generator_present = 1;
    fields.mask_generator = 0x80000002;
    fields.mask_generator_hashing_algorithm_present = 1;
    fields.mask_generator_hashing_algorithm = 0x80000003;
    fields.p_source_present = 1;
    fields.p_source = p_source;
    fields.p_source_length = sizeof(p_source);
    fields.trailer_field_present = 1;
    fields.trailer_field = 1;

    status = kmipkit_cryptographic_parameters_from_fields_v1(&fields, &parameters);
    if (status == 0) {
        status = kmipkit_cryptographic_parameters_into_ttlv(parameters, &structure);
        parameters = 0; /* The ABI consumes the handle on every call. */
    }
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &view);
    uint64_t count = 0;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status == 0 && count != 18) status = 1;
    for (uint64_t index = 0; status == 0 && index < count; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t actual = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual);
        if (status == 0 && actual != expected_tags[index]) status = 1;
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
    }
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_cryptographic_parameters_release(parameters);
    return status;
}

int32_t kmipkit_parameters_authentication_c_consumer_reads(void) {
    static const uint8_t expected_p_source[] = {0x00, 0xff, 0x81};
    kmipkit_cryptographic_parameters_fields_v1_t input = {0};
    kmipkit_cryptographic_parameters_fields_v1_t output = {0};
    kmipkit_cryptographic_parameters_t *parameters = 0;
    int32_t status = 0;

    input.struct_size = (uint32_t)sizeof(input);
    input.abi_version = 1;
    input.block_cipher_mode_present = 1;
    input.block_cipher_mode = UINT32_C(0x80000001);
    input.random_iv_present = 1;
    input.random_iv = 0;
    input.p_source_present = 1;
    input.p_source = expected_p_source;
    input.p_source_length = sizeof(expected_p_source);
    output.struct_size = (uint32_t)sizeof(output);
    output.abi_version = 1;

    status = kmipkit_cryptographic_parameters_from_fields_v1(&input, &parameters);
    if (status == 0) {
        status = kmipkit_cryptographic_parameters_read_fields_v1(parameters, &output);
    }
    if (status == 0 &&
        (output.block_cipher_mode_present != 1 ||
         output.block_cipher_mode != UINT32_C(0x80000001) ||
         output.random_iv_present != 1 || output.random_iv != 0 ||
         output.p_source_present != 1 ||
         output.p_source_length != sizeof(expected_p_source) ||
         output.p_source == 0 ||
         memcmp(output.p_source, expected_p_source, sizeof(expected_p_source)) != 0)) {
        status = 1;
    }

    kmipkit_cryptographic_parameters_release(parameters);
    if (status != 0) return status;
    parameters = 0;
    memset(&input, 0, sizeof(input));
    memset(&output, 0, sizeof(output));
    input.struct_size = (uint32_t)sizeof(input);
    input.abi_version = 1;
    input.random_iv_present = 1;
    input.random_iv = 0;
    input.p_source_present = 1;
    input.p_source = 0;
    input.p_source_length = 0;
    output.struct_size = (uint32_t)sizeof(output);
    output.abi_version = 1;
    status = kmipkit_cryptographic_parameters_from_fields_v1(&input, &parameters);
    if (status == 0) {
        status = kmipkit_cryptographic_parameters_read_fields_v1(parameters, &output);
    }
    if (status == 0 &&
        (output.p_source_present != 1 || output.p_source_length != 0)) {
        status = 1;
    }
    kmipkit_cryptographic_parameters_release(parameters);
    return status;
}

static int32_t create_enumeration_item(
        uint32_t raw_tag, uint32_t raw_value, kmipkit_codec_limits_t *limits,
        kmipkit_ttlv_item_t **out_item) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(raw_value, &value);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = 0;
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t create_byte_string_item(
        uint32_t raw_tag, const uint8_t *bytes, uint64_t length,
        kmipkit_codec_limits_t *limits, kmipkit_ttlv_item_t **out_item) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(limits, bytes, length, &value);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = 0;
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t append_item(
        kmipkit_ttlv_structure_t **structure, kmipkit_ttlv_item_t **item,
        kmipkit_codec_limits_t *limits) {
    kmipkit_ttlv_structure_t *updated = 0;
    int32_t status = kmipkit_ttlv_structure_with_item(*structure, *item, limits, &updated);
    if (status == 0) {
        *structure = updated;
        *item = 0;
    }
    return status;
}

int32_t kmipkit_parameters_authentication_c_consumer_from_ttlv(void) {
    static const uint8_t vendor_bytes[] = {0xca, 0xfe};
    const uint32_t expected_tags[] = {0x00420028, 0x00540031};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *source = 0;
    kmipkit_ttlv_structure_t *roundtrip = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_cryptographic_parameters_t *parameters = 0;
    kmipkit_ttlv_item_t *known = 0;
    kmipkit_ttlv_item_t *vendor = 0;
    kmipkit_cryptographic_parameters_fields_v1_t fields = {0};
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&source);
    if (status == 0) status = create_enumeration_item(
        expected_tags[0], UINT32_C(0x8000002a), limits, &known);
    if (status == 0) status = append_item(&source, &known, limits);
    if (status == 0) status = create_byte_string_item(
        expected_tags[1], vendor_bytes, sizeof(vendor_bytes), limits, &vendor);
    if (status == 0) status = append_item(&source, &vendor, limits);
    if (status == 0) status = kmipkit_cryptographic_parameters_from_ttlv(source, &parameters);

    fields.struct_size = (uint32_t)sizeof(fields);
    fields.abi_version = 1;
    if (status == 0) status = kmipkit_cryptographic_parameters_read_fields_v1(parameters, &fields);
    if (status == 0 && (fields.cryptographic_algorithm_present != 1 ||
            fields.cryptographic_algorithm != UINT32_C(0x8000002a))) status = -1;

    /* from_ttlv copies the source: the input still has both children in caller order. */
    if (status == 0) status = kmipkit_ttlv_structure_view(source, &view);
    uint64_t count = 0;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status == 0 && count != 2) status = -1;
    for (uint64_t index = 0; status == 0 && index < count; ++index) {
        kmipkit_ttlv_item_view_t *child = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t raw_tag = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &child);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(child, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &raw_tag);
        if (status == 0 && raw_tag != expected_tags[index]) status = -1;
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(child);
    }
    kmipkit_ttlv_structure_view_release(view);
    view = 0;

    if (status == 0) {
        status = kmipkit_cryptographic_parameters_into_ttlv(parameters, &roundtrip);
        parameters = 0;
    }
    if (status == 0) status = kmipkit_ttlv_structure_view(roundtrip, &view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status == 0 && count != 2) status = -1;
    for (uint64_t index = 0; status == 0 && index < count; ++index) {
        kmipkit_ttlv_item_view_t *child = 0;
        kmipkit_tag_t *tag = 0;
        kmipkit_ttlv_value_view_t *value = 0;
        uint32_t raw_tag = 0;
        uint8_t kind = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &child);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(child, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &raw_tag);
        if (status == 0) status = kmipkit_ttlv_item_view_type(child, &kind);
        if (status == 0 && raw_tag != expected_tags[index]) status = -1;
        if (status == 0) status = kmipkit_ttlv_item_view_value(child, &value);
        if (status == 0 && index == 0) {
            uint32_t enum_value = 0;
            if (kind != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) status = -1;
            if (status == 0) status = kmipkit_ttlv_value_view_enumeration(value, &enum_value);
            if (status == 0 && enum_value != UINT32_C(0x8000002a)) status = -1;
        } else if (status == 0) {
            uint64_t length = 0;
            if (kind != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING) status = -1;
            if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &length);
            if (status == 0 && length != sizeof(vendor_bytes)) status = -1;
            for (uint64_t byte_index = 0; status == 0 && byte_index < length; ++byte_index) {
                uint8_t byte = 0;
                status = kmipkit_ttlv_value_view_byte_at(value, byte_index, &byte);
                if (status == 0 && byte != vendor_bytes[byte_index]) status = -1;
            }
        }
        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(child);
    }

    kmipkit_ttlv_structure_view_release(view);
    kmipkit_cryptographic_parameters_release(parameters);
    kmipkit_ttlv_structure_release(roundtrip);
    kmipkit_ttlv_structure_release(source);
    kmipkit_ttlv_item_release(known);
    kmipkit_ttlv_item_release(vendor);
    kmipkit_codec_limits_release(limits);
    return status;
}

int32_t kmipkit_parameters_authentication_c_consumer_attach_authentication(
        kmipkit_client_batch_t *batch, kmipkit_authentication_t *authentication) {
    return kmipkit_client_batch_set_authentication(batch, authentication);
}

int32_t kmipkit_parameters_authentication_c_consumer_reject_authentication(
        kmipkit_authentication_t *authentication) {
    return kmipkit_client_batch_set_authentication(0, authentication);
}
