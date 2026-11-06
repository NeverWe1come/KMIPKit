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
