#include "kmipkit.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define T076_ERROR_INVALID_INPUT ((int32_t)4)

extern int32_t kmipkit_client_execute_with_options_and_error_ttlv_v1(
    kmipkit_client_t *client,
    kmipkit_client_batch_t **batch,
    kmipkit_codec_limits_t *limits,
    const kmipkit_request_options_v1_t *options,
    kmipkit_client_batch_response_t **out_response,
    kmipkit_ttlv_structure_t **out_error_response_ttlv);
/* These generic TTLV builder exports let the C ABI fixture form valid Hash parameters. */
extern int32_t kmipkit_ttlv_value_byte_string(
    kmipkit_codec_limits_t *limits,
    const uint8_t *value,
    uint64_t value_length,
    kmipkit_ttlv_value_t **out_value);
extern int32_t kmipkit_ttlv_item_create(
    kmipkit_tag_t *tag,
    kmipkit_ttlv_value_t *value,
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_item_t **out_item);
extern int32_t kmipkit_ttlv_structure_with_item(
    kmipkit_ttlv_structure_t *structure,
    kmipkit_ttlv_item_t *item,
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **out_structure);
extern void kmipkit_raw_tag_release(kmipkit_raw_tag_t *handle);
extern void kmipkit_ttlv_value_release(kmipkit_ttlv_value_t *handle);
extern void kmipkit_ttlv_item_release(kmipkit_ttlv_item_t *handle);

static int execute_with_options_and_error_ttlv_clears_outputs(void) {
    kmipkit_client_batch_response_t *response = (kmipkit_client_batch_response_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *error_response = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t status = kmipkit_client_execute_with_options_and_error_ttlv_v1(
        0, 0, 0, 0, &response, &error_response);
    return status == T076_ERROR_INVALID_INPUT && response == 0 && error_response == 0;
}

static int execute_with_error_ttlv_clears_outputs(void) {
    kmipkit_client_batch_response_t *response = (kmipkit_client_batch_response_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *error_response = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t status = kmipkit_client_execute_with_error_ttlv_v1(
        0, 0, 0, &response, &error_response);
    return status == T076_ERROR_INVALID_INPUT && response == 0 && error_response == 0;
}

static int execute_with_options_returns(
    const kmipkit_request_options_v1_t *options, int32_t expected_status) {
    kmipkit_client_batch_response_t *response = 0;
    const int32_t status = kmipkit_client_execute_with_options_v1(
        0, 0, 0, options, &response);
    if (response != 0) {
        kmipkit_client_batch_response_release(response);
        return 0;
    }
    return status == expected_status;
}

static int api_level_query_matches_shared_fixture_locally(int argc, char **argv) {
    if (argc != 4 || argv == 0 || argv[1] == 0 || argv[2] == 0 || argv[3] == 0) {
        return 0;
    }
    const char *const expected[] = {argv[1], argv[2], argv[3]};
    kmipkit_byte_span_v1_t actual[3] = {{0}};
    uint8_t available = 0;
    static const uint8_t capability[] = "api_level_discovery";
    if (kmipkit_capability_is_available(capability, sizeof(capability) - 1, &available) != 0 ||
        available != 1) {
        return 0;
    }
    if (kmipkit_api_levels_get_v1(0, &actual[1], &actual[2]) !=
        T076_ERROR_INVALID_INPUT) {
        return 0;
    }
    if (kmipkit_api_levels_get_v1(&actual[0], &actual[1], &actual[2]) != 0) {
        return 0;
    }
    for (uint32_t index = 0; index < 3; ++index) {
        if (actual[index].data == 0 || actual[index].length != strlen(expected[index]) ||
            memcmp(actual[index].data, expected[index], actual[index].length) != 0) {
            return 0;
        }
    }
    return 1;
}

static int creation_create_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_create(UINT32_MAX, attributes, masks, &item) != 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int certify_constructor_is_callable_from_c(void) {
    static const uint8_t request_value[] = {0x30, 0x82, 0x01, 0x00};
    kmipkit_byte_span_v1_t request_span = {
        request_value,
        (uint64_t)sizeof(request_value),
    };
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_certify(
            0, 1, UINT32_C(0xfedcba98), &request_span, attributes, masks, &item) != 0) {
        goto cleanup;
    }
    result = item != 0;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int certify_response_accessor_is_callable_from_c(void) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    uint8_t present = 9;
    const int32_t status = kmipkit_client_batch_item_response_certify(
        0, &present, &identifier);
    return status == T076_ERROR_INVALID_INPUT && present == 0 &&
        identifier.kind == 0 && identifier.text_data == 0 &&
        identifier.text_length == 0 && identifier.enumeration_value == 0 &&
        identifier.integer_value == 0;
}

static int check_constructor_is_callable_from_c(void) {
    kmipkit_client_batch_item_t *item = 0;
    const int32_t status = kmipkit_client_batch_item_check(
        0, 1, INT64_MIN, 1, INT32_MIN, 1, 0, &item);
    const int result = status == 0 && item != 0;
    kmipkit_client_batch_item_release(item);
    return result;
}

static int derive_key_constructor_is_callable_from_c(void) {
    static const uint8_t source_identifier[] = {'s', 'o', 'u', 'r', 'c', 'e', '-', 'a'};
    kmipkit_unique_identifier_v1_t identifiers[2] = {{0}};
    kmipkit_ttlv_structure_t *derivation_parameters = 0;
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    for (uint64_t index = 0; index < 2; ++index) {
        identifiers[index].struct_size = (uint32_t)sizeof(identifiers[index]);
        identifiers[index].abi_version = 1;
        identifiers[index].kind = 1;
        identifiers[index].text_data = source_identifier;
        identifiers[index].text_length = sizeof(source_identifier);
    }
    if (kmipkit_ttlv_structure_create(&derivation_parameters) != 0 ||
        kmipkit_ttlv_structure_create(&attributes) != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_derive_key(
            2, identifiers, 2, UINT32_C(0xfeedbeef),
            derivation_parameters, 0, &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_derive_key(
            2, identifiers, 2, UINT32_C(0xfeedbeef),
            derivation_parameters, attributes, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_ttlv_structure_release(derivation_parameters);
    return result;
}

static int export_constructor_is_callable_from_c(void) {
    static const uint8_t identifier_bytes[] = "export-key";
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *wrapping_specification = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);
    if (kmipkit_ttlv_structure_create(&wrapping_specification) != 0 ||
        kmipkit_client_batch_item_export(
            &identifier, 1, UINT32_C(0xfedcba98), 1, UINT32_C(0xfedcba99),
            1, UINT32_C(0xfedcba9a), wrapping_specification, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(wrapping_specification);
    return result;
}

static int export_constructor_rejects_invalid_presence_from_c(void) {
    kmipkit_client_batch_item_t *item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    const int32_t key_format_status = kmipkit_client_batch_item_export(
        0, 2, 0, 0, 0, 0, 0, 0, &item);
    if (key_format_status != T076_ERROR_INVALID_INPUT || item != 0) return 0;

    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    const int32_t key_wrap_status = kmipkit_client_batch_item_export(
        0, 0, 0, 2, 0, 0, 0, 0, &item);
    if (key_wrap_status != T076_ERROR_INVALID_INPUT || item != 0) return 0;

    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    const int32_t compression_status = kmipkit_client_batch_item_export(
        0, 0, 0, 0, 0, 2, 0, 0, &item);
    return compression_status == T076_ERROR_INVALID_INPUT && item == 0;
}
static int delegated_login_constructor_is_callable_from_c(void) {
    kmipkit_ttlv_structure_t *rights = 0;
    kmipkit_ttlv_structure_t *usage_limits = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&rights) != 0 ||
        kmipkit_ttlv_structure_create(&usage_limits) != 0 ||
        kmipkit_client_batch_item_delegated_login(
            0, 0, 0, 0, 0, usage_limits, &item) != T076_ERROR_INVALID_INPUT || item != 0 ||
        kmipkit_client_batch_item_delegated_login(
            rights, 2, INT64_MIN, 1, INT32_MIN, usage_limits, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(usage_limits);
    kmipkit_ttlv_structure_release(rights);
    return result;
}

static int delegated_login_response_accessors_clear_outputs_from_c(void) {
    uint8_t present = 9;
    uint8_t ticket_present = 9;
    uint32_t ticket_type = UINT32_MAX;
    kmipkit_byte_span_v1_t ticket_value = {(const uint8_t *)1, UINT64_MAX};
    const int32_t status = kmipkit_client_batch_item_response_delegated_login(
        0, &present, &ticket_present, &ticket_type, &ticket_value);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || ticket_present != 0 ||
        ticket_type != 0 || ticket_value.data != 0 || ticket_value.length != 0) {
        return 0;
    }
    present = 9;
    kmipkit_ttlv_structure_t *ticket_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_delegated_login_ttlv(
        0, &present, &ticket_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && ticket_ttlv == 0;
}

static int check_response_accessors_clear_outputs_from_c(void) {
    uint8_t present = 9;
    uint8_t identifier_present = 9;
    uint8_t count_present = 9;
    uint8_t mask_present = 9;
    uint8_t lease_present = 9;
    int64_t count = 9;
    int32_t mask = 9;
    uint32_t lease = 9;
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    const int32_t status = kmipkit_client_batch_item_response_check(
        0, &present, &identifier_present, &identifier,
        &count_present, &count, &mask_present, &mask, &lease_present, &lease);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || identifier_present != 0 ||
        count_present != 0 || count != 0 || mask_present != 0 || mask != 0 ||
        lease_present != 0 || lease != 0 || identifier.kind != 0 ||
        identifier.text_data != 0 || identifier.text_length != 0 ||
        identifier.enumeration_value != 0 || identifier.integer_value != 0) {
        return 0;
    }
    present = 9;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_check_ttlv(
        0, &present, &response_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
}

static int export_response_accessors_clear_outputs_from_c(
    kmipkit_client_batch_item_response_t *item) {
    uint8_t present = 9;
    uint8_t payload_present = 9;
    uint32_t object_type = UINT32_MAX;
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    kmipkit_ttlv_structure_t *attributes = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *any_object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_response_export(
            0, &present, &payload_present, &object_type, &identifier,
            &attributes, &any_object) != T076_ERROR_INVALID_INPUT ||
        present != 0 || payload_present != 0 || object_type != 0 ||
        identifier.kind != 0 || identifier.text_data != 0 || identifier.text_length != 0 ||
        identifier.enumeration_value != 0 || identifier.integer_value != 0 ||
        attributes != 0 || any_object != 0) {
        return 0;
    }
    kmipkit_ttlv_structure_t *wrong_kind = 0;
    if (kmipkit_ttlv_structure_create(&wrong_kind) != 0 || wrong_kind == 0) {
        return 0;
    }
    present = 9;
    payload_present = 9;
    object_type = UINT32_MAX;
    attributes = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    any_object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t wrong_kind_status = kmipkit_client_batch_item_response_export(
        (kmipkit_client_batch_item_response_t *)wrong_kind, &present, &payload_present,
        &object_type, &identifier, &attributes, &any_object);
    kmipkit_ttlv_structure_release(wrong_kind);
    if (wrong_kind_status != T076_ERROR_INVALID_INPUT || present != 0 ||
        payload_present != 0 || object_type != 0 || identifier.kind != 0 ||
        identifier.text_data != 0 || identifier.text_length != 0 ||
        identifier.enumeration_value != 0 || identifier.integer_value != 0 ||
        attributes != 0 || any_object != 0) {
        return 0;
    }
    present = 9;
    payload_present = 9;
    object_type = UINT32_MAX;
    attributes = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    any_object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_response_export(
            item, &present, &payload_present, &object_type, &identifier,
            &attributes, &any_object) != 0 || present != 0 || payload_present != 0 ||
        object_type != 0 || identifier.kind != 0 || identifier.text_data != 0 ||
        identifier.text_length != 0 || identifier.enumeration_value != 0 ||
        identifier.integer_value != 0 || attributes != 0 || any_object != 0) {
        return 0;
    }
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = 9;
    if (kmipkit_client_batch_item_response_export_ttlv(
            0, &present, &response_ttlv) != T076_ERROR_INVALID_INPUT ||
        present != 0 || response_ttlv != 0) {
        return 0;
    }
    response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = 9;
    return kmipkit_client_batch_item_response_export_ttlv(
               item, &present, &response_ttlv) == 0 &&
        present == 0 && response_ttlv == 0;
}

static int check_view_tags(
    kmipkit_ttlv_structure_view_t *view, const uint32_t *expected, uint64_t expected_count) {
    uint64_t count = 0;
    if (kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != expected_count) {
        return 0;
    }
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t raw = 0;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == 0 ||
            kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == 0 ||
            kmipkit_ttlv_tag_value(tag, &raw) != 0 || raw != expected[index]) {
            kmipkit_tag_release(tag);
            kmipkit_ttlv_item_view_release(item);
            return 0;
        }
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
    }
    return 1;
}

static int response_identifier_matches(
    const kmipkit_unique_identifier_v1_t *identifier,
    const uint8_t *expected,
    uint64_t expected_length);

static int derive_key_response_is_readable_from_c(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "derived-key";
    static const uint8_t expected_extension[] = {0xde, 0xad, 0x00, 0xff};
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {UINT32_C(0x420094), UINT32_C(0x540001)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_ttlv_item_view_t *extension_item = 0;
    kmipkit_ttlv_value_view_t *extension_value = 0;
    kmipkit_tag_t *extension_tag = 0;
    uint8_t present = 0;
    uint32_t extension_raw_tag = 0;
    uint8_t extension_type = 0;
    uint64_t extension_length = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_derive_key(item, &present, &identifier) != 0 ||
        present != 1 || !response_identifier_matches(
            &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1)) ||
        kmipkit_client_batch_item_response_derive_key_ttlv(
            item, &present, &response_ttlv) != 0 ||
        present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 3, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(payload_view, 1, &extension_item) != 0 ||
        kmipkit_ttlv_item_view_tag(extension_item, &extension_tag) != 0 ||
        kmipkit_ttlv_tag_value(extension_tag, &extension_raw_tag) != 0 ||
        extension_raw_tag != UINT32_C(0x540001) ||
        kmipkit_ttlv_item_view_type(extension_item, &extension_type) != 0 ||
        extension_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
        kmipkit_ttlv_item_view_value(extension_item, &extension_value) != 0 ||
        kmipkit_ttlv_value_view_byte_length(extension_value, &extension_length) != 0 ||
        extension_length != (uint64_t)sizeof(expected_extension)) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < extension_length; ++index) {
        uint8_t value = 0;
        if (kmipkit_ttlv_value_view_byte_at(extension_value, index, &value) != 0 ||
            value != expected_extension[index]) {
            goto cleanup;
        }
    }
    result = 1;

cleanup:
    kmipkit_tag_release(extension_tag);
    kmipkit_ttlv_value_view_release(extension_value);
    kmipkit_ttlv_item_view_release(extension_item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

static int export_response_is_readable_from_c(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "exported-key";
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t attribute_tags[] = {UINT32_C(0x42001d)};
    static const uint32_t object_tags[] = {UINT32_C(0x42008f)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *any_object = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *attributes_view = 0;
    kmipkit_ttlv_structure_view_t *object_view = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    uint8_t present = 0;
    uint8_t payload_present = 0;
    uint32_t object_type = 0;
    int result = 0;
    if (item == 0) {
        return 0;
    }
    uint8_t rejected_present = 9;
    uint8_t rejected_payload_present = 9;
    uint32_t rejected_object_type = UINT32_MAX;
    kmipkit_unique_identifier_v1_t invalid_identifier = {0};
    invalid_identifier.struct_size = (uint32_t)sizeof(invalid_identifier) - 1;
    invalid_identifier.abi_version = 1;
    kmipkit_ttlv_structure_t *rejected_attributes =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *rejected_any_object =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_response_export(
            item, &rejected_present, &rejected_payload_present, &rejected_object_type,
            &invalid_identifier, &rejected_attributes, &rejected_any_object) !=
            T076_ERROR_INVALID_INPUT || rejected_present != 0 || rejected_payload_present != 0 ||
        rejected_object_type != 0 || rejected_attributes != 0 || rejected_any_object != 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_export(
            item, &present, &payload_present, &object_type, &identifier,
            &attributes, &any_object) != 0 || present != 1 || payload_present != 1 ||
        object_type != UINT32_C(0xf1234567) ||
        !response_identifier_matches(
            &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1)) ||
        attributes == 0 || any_object == 0 ||
        kmipkit_client_batch_item_response_export_ttlv(item, &present, &response_ttlv) != 0 ||
        present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(attributes, &attributes_view) != 0 ||
        !check_view_tags(attributes_view, attribute_tags,
            (uint64_t)(sizeof(attribute_tags) / sizeof(attribute_tags[0]))) ||
        kmipkit_ttlv_structure_view(any_object, &object_view) != 0 ||
        !check_view_tags(object_view, object_tags,
            (uint64_t)(sizeof(object_tags) / sizeof(object_tags[0]))) ||
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0])))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_view_release(object_view);
    kmipkit_ttlv_structure_view_release(attributes_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_ttlv_structure_release(any_object);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int append_derive_key_parameter(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **parameters,
    uint32_t raw_tag_value,
    kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *updated = 0;
    int result = 0;

    if (limits == 0 || parameters == 0 || *parameters == 0 || value == 0 || *value == 0) {
        return 0;
    }
    if (kmipkit_ttlv_raw_tag_create(raw_tag_value, &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, *value, limits, &item) != 0) {
        goto cleanup;
    }
    *value = 0;
    if (kmipkit_ttlv_structure_with_item(*parameters, item, limits, &updated) != 0) {
        goto cleanup;
    }
    *parameters = 0;
    item = 0;
    *parameters = updated;
    updated = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    return result;
}

static int create_derive_key_request(
    kmipkit_codec_limits_t *limits, kmipkit_client_batch_item_t **out_item) {
    static const uint8_t source_identifier[] = "source-key";
    static const uint8_t first_derivation_data[] = {0xa1, 0x00, 0xb2};
    static const uint8_t second_derivation_data[] = {0xc3, 0x00, 0xd4};
    static const uint8_t salt_bytes[] = {0xe5, 0x00, 0xf6};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *parameters = 0;
    kmipkit_ttlv_structure_t *empty_attributes = 0;
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *parameter_value = 0;
    kmipkit_ttlv_value_t *length = 0;
    kmipkit_ttlv_item_t *length_item = 0;
    int result = 0;

    if (out_item == 0 || limits == 0) {
        return 0;
    }
    *out_item = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = source_identifier;
    identifier.text_length = (uint64_t)(sizeof(source_identifier) - 1);
    if (kmipkit_ttlv_structure_create(&parameters) != 0 ||
        kmipkit_ttlv_structure_create(&empty_attributes) != 0 ||
        kmipkit_ttlv_value_byte_string(
            limits,
            first_derivation_data,
            (uint64_t)sizeof(first_derivation_data),
            &parameter_value) != 0 ||
        !append_derive_key_parameter(
            limits, &parameters, UINT32_C(0x420030), &parameter_value) ||
        kmipkit_ttlv_value_byte_string(
            limits,
            second_derivation_data,
            (uint64_t)sizeof(second_derivation_data),
            &parameter_value) != 0 ||
        !append_derive_key_parameter(
            limits, &parameters, UINT32_C(0x420030), &parameter_value) ||
        kmipkit_ttlv_value_byte_string(
            limits, salt_bytes, (uint64_t)sizeof(salt_bytes), &parameter_value) != 0 ||
        !append_derive_key_parameter(
            limits, &parameters, UINT32_C(0x420084), &parameter_value) ||
        kmipkit_ttlv_value_integer(4096, &parameter_value) != 0 ||
        !append_derive_key_parameter(
            limits, &parameters, UINT32_C(0x42003c), &parameter_value) ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x42002a), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_integer(128, &length) != 0 ||
        kmipkit_ttlv_item_create(tag, length, limits, &length_item) != 0 ||
        kmipkit_ttlv_structure_with_item(
            empty_attributes, length_item, limits, &attributes) != 0) {
        goto cleanup;
    }
    empty_attributes = 0;
    length_item = 0;
    if (kmipkit_client_batch_item_derive_key(
            7, &identifier, 1, UINT32_C(0xfeedbeef), parameters, attributes,
            out_item) != 0 || *out_item == 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_item_release(length_item);
    kmipkit_ttlv_value_release(length);
    kmipkit_ttlv_value_release(parameter_value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_ttlv_structure_release(empty_attributes);
    kmipkit_ttlv_structure_release(parameters);
    if (!result) {
        kmipkit_client_batch_item_release(*out_item);
        *out_item = 0;
    }
    return result;
}

int kmipkit_client_abi_c_consumer_validate_failed_check_response(
    kmipkit_client_batch_item_response_t *item) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x0042005c), UINT32_C(0x00420093), UINT32_C(0x0042007f),
        UINT32_C(0x0042007e), UINT32_C(0x0042007c),
    };
    static const uint32_t payload_tags[] = {
        UINT32_C(0x00420094), UINT32_C(0x00420096), UINT32_C(0x0042002c),
        UINT32_C(0x00420049), UINT32_C(0x00541234),
    };
    uint8_t present = 0;
    uint8_t identifier_present = 0;
    uint8_t count_present = 0;
    uint8_t mask_present = 0;
    uint8_t lease_present = 0;
    int64_t count = 0;
    int32_t mask = 0;
    uint32_t lease = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    int result = 0;
    if (kmipkit_client_batch_item_response_check(
            item, &present, &identifier_present, &identifier,
            &count_present, &count, &mask_present, &mask, &lease_present, &lease) != 0 ||
        present != 1 || identifier_present != 0 || identifier.kind != 0 ||
        count_present != 1 || count != INT64_C(-9) || mask_present != 1 ||
        mask != INT32_C(-1) || lease_present != 1 || lease != 0 ||
        kmipkit_client_batch_item_response_check_ttlv(item, &present, &response_ttlv) != 0 ||
        present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 4, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0])))) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
}

static int get_constraints_constructor_is_callable_from_c(void) {
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_get_constraints(&item) != 0) {
        return 0;
    }
    const int result = item != 0;
    kmipkit_client_batch_item_release(item);
    return result;
}

static int get_constraints_response_accessor_clears_outputs_from_c(void) {
    uint8_t present = 9;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)1;
    const int32_t status = kmipkit_client_batch_item_response_get_constraints_ttlv(
        0, &present, &response_ttlv);
    return status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
}

static int get_constraints_response_ttlv_matches_sentinel(
    kmipkit_ttlv_structure_t *response_ttlv) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t constraints_tags[] = {UINT32_C(0x420168)};
    static const uint32_t unknown_member_tags[] = {UINT32_C(0x541234)};
    static const uint8_t expected_value[] = "constraints-secret-sentinel";
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *response_payload_item = 0;
    kmipkit_ttlv_value_view_t *response_payload_value = 0;
    kmipkit_ttlv_structure_view_t *response_payload_view = 0;
    kmipkit_ttlv_item_view_t *constraints_item = 0;
    kmipkit_ttlv_value_view_t *constraints_value = 0;
    kmipkit_ttlv_structure_view_t *constraints_view = 0;
    kmipkit_ttlv_item_view_t *unknown_item = 0;
    kmipkit_ttlv_value_view_t *unknown_value = 0;
    uint64_t value_length = 0;
    uint8_t item_type = 0;
    int matches = 0;

    if (response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags, 4) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 3, &response_payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(response_payload_item, &response_payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(response_payload_value, &response_payload_view) != 0 ||
        !check_view_tags(response_payload_view, constraints_tags, 1) ||
        kmipkit_ttlv_structure_view_item_at(response_payload_view, 0, &constraints_item) != 0 ||
        kmipkit_ttlv_item_view_value(constraints_item, &constraints_value) != 0 ||
        kmipkit_ttlv_value_view_structure(constraints_value, &constraints_view) != 0 ||
        !check_view_tags(constraints_view, unknown_member_tags, 1) ||
        kmipkit_ttlv_structure_view_item_at(constraints_view, 0, &unknown_item) != 0 ||
        kmipkit_ttlv_item_view_value(unknown_item, &unknown_value) != 0 ||
        kmipkit_ttlv_value_view_type(unknown_value, &item_type) != 0 || item_type != 7 ||
        kmipkit_ttlv_value_view_byte_length(unknown_value, &value_length) != 0 ||
        value_length != sizeof(expected_value) - 1) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < value_length; ++index) {
        uint8_t actual = 0;
        if (kmipkit_ttlv_value_view_byte_at(unknown_value, index, &actual) != 0 ||
            actual != expected_value[index]) {
            goto cleanup;
        }
    }
    matches = 1;

cleanup:
    kmipkit_ttlv_value_view_release(unknown_value);
    kmipkit_ttlv_item_view_release(unknown_item);
    kmipkit_ttlv_structure_view_release(constraints_view);
    kmipkit_ttlv_value_view_release(constraints_value);
    kmipkit_ttlv_item_view_release(constraints_item);
    kmipkit_ttlv_structure_view_release(response_payload_view);
    kmipkit_ttlv_value_view_release(response_payload_value);
    kmipkit_ttlv_item_view_release(response_payload_item);
    kmipkit_ttlv_structure_view_release(response_view);
    return matches;
}
static int get_usage_allocation_constructor_is_callable_from_c(void) {
    static const uint8_t identifier_text[] = "allocation-object";
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_get_usage_allocation(
            &identifier, INT64_MAX, &item) != 0) {
        return 0;
    }
    const int result = item != 0;
    kmipkit_client_batch_item_release(item);
    return result;
}

static int get_usage_allocation_response_accessor_clears_outputs(void) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    uint8_t present = 9;
    const int32_t status = kmipkit_client_batch_item_response_get_usage_allocation(
        0, &present, &identifier);
    return status == T076_ERROR_INVALID_INPUT && present == 0 &&
        identifier.kind == 0 && identifier.text_data == 0 &&
        identifier.text_length == 0 && identifier.enumeration_value == 0 &&
        identifier.integer_value == 0;
}

static int creation_create_key_pair_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *groups[6] = {0};
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    for (uint32_t index = 0; index < 6; ++index) {
        if (kmipkit_ttlv_structure_create(&groups[index]) != 0) {
            goto cleanup;
        }
    }
    if (kmipkit_client_batch_item_create_key_pair(
            groups[0], groups[1], groups[2], groups[3], groups[4], groups[5], &item) != 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    for (uint32_t index = 0; index < 6; ++index) {
        kmipkit_ttlv_structure_release(groups[index]);
    }
    return result;
}

static int creation_create_split_key_matches_shared_fixture(void) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -7;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_create_split_key(
            0xf0000000U, &identifier, -3, 4, 0xdeadbeefU, 0, 0, 0, attributes, masks, &item) != 0) {
        goto cleanup;
    }
    result = metadata.struct_size == sizeof(metadata) && metadata.abi_version == 2;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int split_key_creation_returns(
    const kmipkit_unique_identifier_v1_t *identifier,
    uint8_t has_prime_field_size,
    const uint8_t *prime_field_size_data,
    uint64_t prime_field_size_length,
    int32_t expected_status) {
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0) {
        goto cleanup;
    }
    const int32_t status = kmipkit_client_batch_item_create_split_key(
        1, identifier, 2, 1, 1, has_prime_field_size, prime_field_size_data,
        prime_field_size_length, attributes, 0, &item);
    if (status != expected_status ||
        (expected_status == 0 ? item == 0 : item != 0)) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int creation_rejects_malformed_split_key_inputs(void) {
    static const uint8_t invalid_utf8[] = {0xff};
    static const uint8_t valid_text[] = {'i', 'd', '-', 0xce, 0xb1};
    static const uint8_t optional_byte[] = {0x2a};
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;

    identifier.struct_size -= 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 2;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.abi_version = 1;
    identifier.reserved = 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.reserved = 0;
    identifier.kind = UINT32_MAX;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = invalid_utf8;
    identifier.text_length = sizeof(invalid_utf8);
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.text_data = 0;
    identifier.text_length = 1;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    identifier.text_data = valid_text;
    identifier.text_length = sizeof(valid_text);
    if (!split_key_creation_returns(&identifier, 0, 0, 0, 0)) {
        return 0;
    }
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
    identifier.enumeration_value = UINT32_MAX;
    if (!split_key_creation_returns(&identifier, 0, 0, 0, 0)) {
        return 0;
    }
    if (!split_key_creation_returns(0, 0, optional_byte, 0, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 0, 0, 1, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 1, 0, 1, T076_ERROR_INVALID_INPUT) ||
        !split_key_creation_returns(0, 2, 0, 0, T076_ERROR_INVALID_INPUT)) {
        return 0;
    }
    return 1;
}

static int creation_rejects_missing_required_attributes(void) {
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_create(UINT32_MAX, 0, 0, &item) !=
            T076_ERROR_INVALID_INPUT ||
        item != 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    if (kmipkit_client_batch_item_create_split_key(
            1, 0, 2, 1, 1, 0, 0, 0, 0, 0, &item) != T076_ERROR_INVALID_INPUT ||
        item != 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    return 1;
}

static int crypto_constructor_result_is_valid(
    int32_t status, kmipkit_client_batch_item_t **item) {
    if ((status != 0 && status != T076_ERROR_INVALID_INPUT) ||
        (status == 0 && *item == 0) ||
        (status != 0 && *item != 0)) {
        kmipkit_client_batch_item_release(*item);
        *item = 0;
        return 0;
    }
    kmipkit_client_batch_item_release(*item);
    *item = 0;
    return 1;
}

static int hash_request_inputs_create(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **out_parameters,
    kmipkit_ttlv_value_t **out_data) {
    static const uint8_t data_bytes[] = "request-hash-data";
    kmipkit_ttlv_structure_t *empty = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *algorithm = 0;
    kmipkit_ttlv_item_t *item = 0;
    int result = 0;
    if (out_parameters == 0 || out_data == 0) {
        return 0;
    }
    *out_parameters = 0;
    *out_data = 0;
    if (kmipkit_ttlv_structure_create(&empty) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x420038), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_enumeration(6, &algorithm) != 0 ||
        kmipkit_ttlv_item_create(tag, algorithm, limits, &item) != 0) {
        goto cleanup;
    }
    algorithm = 0;
    if (kmipkit_ttlv_structure_with_item(empty, item, limits, out_parameters) != 0) {
        goto cleanup;
    }
    empty = 0;
    item = 0;
    if (kmipkit_ttlv_value_byte_string(
            limits, data_bytes, (uint64_t)(sizeof(data_bytes) - 1), out_data) != 0) {
        kmipkit_ttlv_structure_release(*out_parameters);
        *out_parameters = 0;
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_value_release(algorithm);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_structure_release(empty);
    return result;
}

static int cryptographic_abi_symbols_are_callable(void) {
    static const uint8_t correlation_bytes[] = {0x71, 0x00, 0x72};
    static const uint8_t mac_bytes[] = {0x81, 0x82, 0x83};
    static const uint8_t digest_bytes[] = {0x91, 0x92};
    static const uint8_t signature_bytes[] = {0xa1, 0xa2, 0xa3};
    kmipkit_byte_span_v1_t correlation = {correlation_bytes, sizeof(correlation_bytes)};
    kmipkit_byte_span_v1_t mac = {mac_bytes, sizeof(mac_bytes)};
    kmipkit_byte_span_v1_t digest = {digest_bytes, sizeof(digest_bytes)};
    kmipkit_byte_span_v1_t signature = {signature_bytes, sizeof(signature_bytes)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = (const uint8_t *)"ffi-key";
    identifier.text_length = 7;
    kmipkit_client_batch_item_t *item = 0;

    if (!crypto_constructor_result_is_valid(
            kmipkit_client_batch_item_hash(0, 0, &correlation, 1, 2, &item), &item) ||
        !crypto_constructor_result_is_valid(
            kmipkit_client_batch_item_mac(&identifier, 0, 0, &correlation, 1, 2, &item), &item) ||
        !crypto_constructor_result_is_valid(
            kmipkit_client_batch_item_mac_verify(
                &identifier, 0, 0, &mac, &correlation, 1, 2, &item), &item) ||
        !crypto_constructor_result_is_valid(
            kmipkit_client_batch_item_sign(
                &identifier, 0, 0, &digest, &correlation, 1, 2, &item), &item) ||
        !crypto_constructor_result_is_valid(
            kmipkit_client_batch_item_signature_verify(
                &identifier, 0, 0, &digest, &signature, &correlation, 1, 2, &item), &item)) {
        return 0;
    }

    kmipkit_client_batch_item_response_t *response_item = 0;
    uint8_t present = 0xff;
    uint8_t data_present = 0xff;
    uint8_t correlation_present = 0xff;
    uint8_t validity_present = 0xff;
    uint8_t signature_present = 0xff;
    uint8_t recovered_present = 0xff;
    uint32_t indicator = UINT32_MAX;
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_byte_span_v1_t correlation_value = {0};
    kmipkit_byte_span_v1_t signature_value = {0};
    kmipkit_byte_span_v1_t recovered_data = {0};
    if (kmipkit_client_batch_item_response_hash(
            response_item, &present, &data_present, &data,
            &correlation_present, &correlation_value) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_mac(
            response_item, &present, &identifier, &data_present, &data,
            &correlation_present, &correlation_value) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_mac_verify(
            response_item, &present, &identifier, &validity_present, &indicator,
            &correlation_present, &correlation_value) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_sign(
            response_item, &present, &identifier, &signature_present, &signature_value,
            &correlation_present, &correlation_value) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_signature_verify(
            response_item, &present, &identifier, &validity_present, &indicator,
            &recovered_present, &recovered_data,
            &correlation_present, &correlation_value) != T076_ERROR_INVALID_INPUT) {
        return 0;
    }
    return 1;
}

static int response_span_matches(
    kmipkit_byte_span_v1_t span, const uint8_t *expected, uint64_t expected_length) {
    return span.length == expected_length && span.data != 0 && expected != 0 &&
        memcmp(span.data, expected, (size_t)expected_length) == 0;
}

static int response_identifier_matches(
    const kmipkit_unique_identifier_v1_t *identifier,
    const uint8_t *expected,
    uint64_t expected_length) {
    return identifier != 0 && identifier->kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier->text_data != 0 && identifier->text_length == expected_length &&
        memcmp(identifier->text_data, expected, (size_t)expected_length) == 0;
}

static int get_locate_abi_is_callable_from_c(void) {
    /* OASIS KMIP Specification v2.1 §§6.1.19 and 6.1.28, Tables 220–222 and 247–249. */
    static const uint8_t identifier_bytes[] = "get-locate-key";
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);

    kmipkit_ttlv_structure_t *wrapping_specification = 0;
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_client_batch_item_t *get_item = 0;
    kmipkit_client_batch_item_t *locate_item = 0;
    if (kmipkit_ttlv_structure_create(&wrapping_specification) != 0 ||
        kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_client_batch_item_get(
            &identifier, 1, UINT32_C(0xf1234567), 1, UINT32_C(0xf1234568),
            1, UINT32_C(0xf1234569), wrapping_specification, &get_item) != 0 ||
        get_item == 0 ||
        kmipkit_client_batch_item_locate(
            1, -4, 1, 0, 1, UINT32_C(0x80000001), 1, UINT32_C(0xf1234567),
            attributes, &locate_item) != 0 ||
        locate_item == 0) {
        kmipkit_client_batch_item_release(locate_item);
        kmipkit_client_batch_item_release(get_item);
        kmipkit_ttlv_structure_release(attributes);
        kmipkit_ttlv_structure_release(wrapping_specification);
        return 0;
    }
    kmipkit_client_batch_item_release(locate_item);
    kmipkit_client_batch_item_release(get_item);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_ttlv_structure_release(wrapping_specification);

    uint8_t present = 0xff;
    uint8_t payload_present = 0xff;
    uint8_t located_items_present = 0xff;
    uint32_t object_type = UINT32_MAX;
    int32_t located_items = INT32_MAX;
    uint64_t identifier_count = UINT64_MAX;
    kmipkit_unique_identifier_v1_t output_identifier = {0};
    output_identifier.struct_size = (uint32_t)sizeof(output_identifier);
    output_identifier.abi_version = 1;
    kmipkit_ttlv_structure_t *object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_response_get(
            0, &present, &payload_present, &object_type, &output_identifier, &object) !=
            T076_ERROR_INVALID_INPUT ||
        present != 0 || payload_present != 0 || object_type != 0 ||
        output_identifier.kind != 0 || object != 0) {
        return 0;
    }
    present = 0xff;
    payload_present = 0xff;
    located_items_present = 0xff;
    if (kmipkit_client_batch_item_response_locate(
            0, &present, &payload_present, &located_items_present,
            &located_items, &identifier_count) != T076_ERROR_INVALID_INPUT ||
        present != 0 || payload_present != 0 || located_items_present != 0 ||
        located_items != 0 || identifier_count != 0) {
        return 0;
    }
    return kmipkit_client_batch_item_response_locate_unique_identifier_at(
               0, 0, &output_identifier) == T076_ERROR_INVALID_INPUT &&
        output_identifier.kind == 0;
}

static int successful_crypto_response_is_readable(
    uint64_t index, kmipkit_client_batch_item_response_t *response_item) {
    static const uint8_t correlation_bytes[] = {0x41, 0x00, 0x42};
    static const uint8_t response_identifier_bytes[] = "response-id";
    static const uint8_t hash_bytes[] = "hash-response-data";
    static const uint8_t mac_bytes[] = "mac-response-data";
    static const uint8_t signature_bytes[] = "signature-response-data";
    static const uint8_t recovered_bytes[] = "recovered-response-data";
    static const uint8_t issued_certificate[] = "issued-certificate";
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;

    if (index == 8) {
        uint8_t data_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t data = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        return kmipkit_client_batch_item_response_hash(
                    response_item, &present, &data_present, &data,
                    &correlation_present, &correlation) == 0 &&
            present == 1 && data_present == 1 && correlation_present == 1 &&
            response_span_matches(data, hash_bytes, sizeof(hash_bytes) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    if (index == 9) {
        uint8_t data_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t data = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        return kmipkit_client_batch_item_response_mac(
                    response_item, &present, &identifier, &data_present, &data,
                    &correlation_present, &correlation) == 0 &&
            present == 1 && data_present == 1 && correlation_present == 1 &&
            response_identifier_matches(
                &identifier, response_identifier_bytes, sizeof(response_identifier_bytes) - 1) &&
            response_span_matches(data, mac_bytes, sizeof(mac_bytes) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    if (index == 10) {
        uint8_t validity_present = 0;
        uint32_t validity = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t correlation = {0};
        return kmipkit_client_batch_item_response_mac_verify(
                    response_item, &present, &identifier, &validity_present, &validity,
                    &correlation_present, &correlation) == 0 &&
            present == 1 && validity_present == 1 && validity == 1 &&
            correlation_present == 1 &&
            response_identifier_matches(
                &identifier, response_identifier_bytes, sizeof(response_identifier_bytes) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    if (index == 11) {
        uint8_t signature_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t signature = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        return kmipkit_client_batch_item_response_sign(
                    response_item, &present, &identifier, &signature_present, &signature,
                    &correlation_present, &correlation) == 0 &&
            present == 1 && signature_present == 1 && correlation_present == 1 &&
            response_identifier_matches(
                &identifier, response_identifier_bytes, sizeof(response_identifier_bytes) - 1) &&
            response_span_matches(signature, signature_bytes, sizeof(signature_bytes) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    if (index == 12) {
        uint8_t validity_present = 0;
        uint32_t validity = 0;
        uint8_t recovered_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t recovered = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        return kmipkit_client_batch_item_response_signature_verify(
                    response_item, &present, &identifier, &validity_present, &validity,
                    &recovered_present, &recovered, &correlation_present, &correlation) == 0 &&
            present == 1 && validity_present == 1 && validity == 1 && recovered_present == 1 &&
            correlation_present == 1 &&
            response_identifier_matches(
                &identifier, response_identifier_bytes, sizeof(response_identifier_bytes) - 1) &&
            response_span_matches(recovered, recovered_bytes, sizeof(recovered_bytes) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    if (index == 13) {
        return kmipkit_client_batch_item_response_certify(
                    response_item, &present, &identifier) == 0 && present == 1 &&
            response_identifier_matches(&identifier, issued_certificate, sizeof(issued_certificate) - 1);
    }
    return 0;
}

static int public_ttlv_contains_response_sentinel(kmipkit_ttlv_structure_t *structure);

static int execute_public_tls_request(char **argv, kmipkit_codec_limits_t *limits) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint32_t expected_statuses[] = {
        0, 2, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1};
    static const uint8_t expected_correlation[] = {0x00, 0xa1, 0x7f};
    static const uint8_t expected_ids[21][6] = {
        {'i', 't', 'e', 'm', '-', '0'},
        {'i', 't', 'e', 'm', '-', '1'},
        {'i', 't', 'e', 'm', '-', '2'},
        {'i', 't', 'e', 'm', '-', '3'},
        {'i', 't', 'e', 'm', '-', '4'},
        {'i', 't', 'e', 'm', '-', '5'},
        {'i', 't', 'e', 'm', '-', '6'},
        {'i', 't', 'e', 'm', '-', '7'},
        {'h', 'a', 's', 'h', '-', '0'},
        {'m', 'a', 'c', '-', '0', '0'},
        {'m', 'a', 'c', 'v', 'f', 'y'},
        {'s', 'i', 'g', 'n', '0', '0'},
        {'s', 'i', 'g', 'v', 'f', 'y'},
        {'c', 'e', 'r', 't', 'i', 'f'},
        {'d', 'e', 'r', 'i', 'v', 'e'},
        {'e', 'x', 'p', 'o', 'r', 't'},
        {'c', 'o', 'n', 's', 't', 'r'},
        {'g', 'e', 't', '-', '0', '0'},
        {'l', 'o', 'c', '-', '0', '0'},
        {'g', 'e', 't', '-', 'f', '0'},
        {'l', 'o', 'c', '-', 'f', '0'},
    };
    static const uint8_t expected_failure_message[] = {
        'c', 'r', 0xc3, 0xa9, 'a', 't', 'i', 'o', 'n', ' ', 'r', 'e', 'f', 'u', 's', 0xc3,
        0xa9, 'e', ' ', 0xe2, 0x9c, 0x93};
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_extension_definition_t **definitions = 0;
    kmipkit_client_extension_registry_t *registry = 0;
    kmipkit_client_configuration_t *configuration = 0;
    kmipkit_transport_config_t *transport = 0;
    kmipkit_client_t *client = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *items[21] = {0};
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_ttlv_structure_t *get_constraints_ttlv = 0;
    kmipkit_ttlv_structure_t *get_any_object = 0;
    uint64_t response_count = 0;
    char *port_end = 0;
    const unsigned long port = strtoul(argv[5], &port_end, 10);
    int result = 1;

    if (port_end == argv[5] || *port_end != '\0' || port == 0 || port > 65535UL) {
        return 2;
    }
    client_certificates[0].source_kind = 2; /* file */
    client_certificates[0].encoding = 2; /* DER */
    client_certificates[0].data.data = (const uint8_t *)argv[1];
    client_certificates[0].data.length = (uint64_t)strlen(argv[1]);
    client_certificates[1] = client_certificates[0];
    client_certificates[1].data.data = (const uint8_t *)argv[2];
    client_certificates[1].data.length = (uint64_t)strlen(argv[2]);
    trust_certificates[0].source_kind = 2;
    trust_certificates[0].encoding = 2;
    trust_certificates[0].data.data = (const uint8_t *)argv[4];
    trust_certificates[0].data.length = (uint64_t)strlen(argv[4]);

    transport_input.struct_size = (uint32_t)sizeof(transport_input);
    transport_input.abi_version = 1;
    transport_input.endpoint_kind = 1; /* raw TLS */
    transport_input.trust_kind = 2; /* explicit CA list */
    transport_input.flags = 1; /* explicit TLS server name */
    transport_input.endpoint.data = host;
    transport_input.endpoint.length = (uint64_t)(sizeof(host) - 1);
    transport_input.endpoint_port = (uint16_t)port;
    transport_input.tls_server_name.data = server_name;
    transport_input.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    transport_input.client_certificates = client_certificates;
    transport_input.client_certificate_count = 2;
    transport_input.client_private_key.source_kind = 2;
    transport_input.client_private_key.encoding = 2;
    transport_input.client_private_key.data.data = (const uint8_t *)argv[3];
    transport_input.client_private_key.data.length = (uint64_t)strlen(argv[3]);
    transport_input.trust_certificates = trust_certificates;
    transport_input.trust_certificate_count = 1;
    transport_input.max_request_bytes = UINT64_C(1048576);

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = 1;
    if (kmipkit_extension_registry_limits_v1_default_values(&registry_limits) != 0 ||
        kmipkit_client_extension_registry_create_v1(
            definitions, 0, &registry_limits, &registry) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_configuration_create(registry, &configuration) != 0) {
        goto cleanup;
    }
    registry = 0; /* consumed by client configuration */
    if (kmipkit_transport_config_create_v1(&transport_input, &transport) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_create(configuration, &transport, &client) != 0) {
        goto cleanup;
    }
    transport = 0; /* consumed by client */
    if (kmipkit_client_batch_create(&batch) != 0) {
        result = 30;
        goto cleanup;
    }
    if (kmipkit_client_batch_set_asynchronous_indicator(batch, 2) != 0) {
        result = 31;
        goto cleanup;
    }
    for (uint64_t index = 0; index < 21; ++index) {
        kmipkit_client_batch_item_t *identified_item = 0;
        if (index == 4 || index == 5) {
            kmipkit_ttlv_structure_t *attributes = 0;
            kmipkit_ttlv_structure_t *masks = 0;
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_ttlv_structure_create(&masks) != 0 ||
                kmipkit_client_batch_item_create(UINT32_MAX, attributes, masks, &items[index]) != 0) {
                kmipkit_ttlv_structure_release(masks);
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(masks);
            kmipkit_ttlv_structure_release(attributes);
        } else if (index == 6) {
            if (kmipkit_client_batch_item_create_key_pair(0, 0, 0, 0, 0, 0,
                    &items[index]) != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 7) {
            kmipkit_ttlv_structure_t *attributes = 0;
            kmipkit_ttlv_structure_t *masks = 0;
            static const uint8_t prime_field_size[] = {0, 0, 0, 0, 0, 0, 0, 17};
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_ttlv_structure_create(&masks) != 0 ||
                kmipkit_client_batch_item_create_split_key(
                    0xf0000000U, 0, -3, 4, 3, 1, prime_field_size, sizeof(prime_field_size),
                    attributes, masks, &items[index]) != 0 || items[index] == 0) {
                kmipkit_ttlv_structure_release(masks);
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(masks);
            kmipkit_ttlv_structure_release(attributes);
        } else if (index >= 8 && index <= 12) {
            static const uint8_t mac_request_bytes[] = {0x81, 0x82, 0x83};
            static const uint8_t digest_request_bytes[] = {0x91, 0x92};
            static const uint8_t signature_request_bytes[] = {0xa1, 0xa2, 0xa3};
            kmipkit_byte_span_v1_t mac_data = {
                mac_request_bytes, sizeof(mac_request_bytes)};
            kmipkit_byte_span_v1_t digest_data = {
                digest_request_bytes, sizeof(digest_request_bytes)};
            kmipkit_byte_span_v1_t signature_data = {
                signature_request_bytes, sizeof(signature_request_bytes)};
            int32_t constructor_status = T076_ERROR_INVALID_INPUT;
            if (index == 8) {
                kmipkit_ttlv_structure_t *parameters = 0;
                kmipkit_ttlv_value_t *data = 0;
                if (!hash_request_inputs_create(limits, &parameters, &data)) {
                    result = 40 + (int)index;
                    goto cleanup;
                }
                constructor_status = kmipkit_client_batch_item_hash(
                    parameters, data, 0, 0, 0, &items[index]);
                kmipkit_ttlv_value_release(data);
                kmipkit_ttlv_structure_release(parameters);
            } else if (index == 9) {
                kmipkit_ttlv_value_t *data = 0;
                if (kmipkit_ttlv_value_byte_string(
                        limits, mac_request_bytes, sizeof(mac_request_bytes), &data) != 0) {
                    result = 40 + (int)index;
                    goto cleanup;
                }
                constructor_status = kmipkit_client_batch_item_mac(
                    0, 0, data, 0, 0, 0, &items[index]);
                kmipkit_ttlv_value_release(data);
            } else if (index == 10) {
                constructor_status = kmipkit_client_batch_item_mac_verify(
                    0, 0, 0, &mac_data, 0, 0, 0, &items[index]);
            } else if (index == 11) {
                constructor_status = kmipkit_client_batch_item_sign(
                    0, 0, 0, &digest_data, 0, 0, 0, &items[index]);
            } else {
                constructor_status = kmipkit_client_batch_item_signature_verify(
                    0, 0, 0, &digest_data, &signature_data, 0, 0, 0, &items[index]);
            }
            if (constructor_status != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 13) {
            static const uint8_t request_identifier_bytes[] = "public-key";
            kmipkit_unique_identifier_v1_t identifier = {0};
            kmipkit_ttlv_structure_t *attributes = 0;
            kmipkit_ttlv_structure_t *masks = 0;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier_bytes;
            identifier.text_length = (uint64_t)(sizeof(request_identifier_bytes) - 1);
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_ttlv_structure_create(&masks) != 0 ||
                kmipkit_client_batch_item_certify(
                    &identifier, 1, 1, 0, attributes, masks, &items[index]) != 0 ||
                items[index] == 0) {
                kmipkit_ttlv_structure_release(masks);
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(masks);
            kmipkit_ttlv_structure_release(attributes);
        } else if (index == 14) {
            if (!create_derive_key_request(limits, &items[index])) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 15) {
            static const uint8_t request_identifier_bytes[] = "export-key";
            kmipkit_unique_identifier_v1_t identifier = {0};
            kmipkit_ttlv_structure_t *wrapping_specification = 0;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier_bytes;
            identifier.text_length = (uint64_t)(sizeof(request_identifier_bytes) - 1);
            if (kmipkit_ttlv_structure_create(&wrapping_specification) != 0 ||
                kmipkit_client_batch_item_export(
                    &identifier, 1, UINT32_C(0xfedcba98), 1, UINT32_C(0xfedcba99),
                    1, UINT32_C(0xfedcba9a), wrapping_specification, &items[index]) != 0 ||
                items[index] == 0) {
                kmipkit_ttlv_structure_release(wrapping_specification);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(wrapping_specification);
        } else if (index == 16) {
            if (kmipkit_client_batch_item_get_constraints(&items[index]) != 0 ||
                items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 17) {
            static const uint8_t request_identifier_bytes[] = "get-source";
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier_bytes;
            identifier.text_length = (uint64_t)(sizeof(request_identifier_bytes) - 1);
            if (kmipkit_client_batch_item_get(
                    &identifier, 0, 0, 0, 0, 0, 0, 0, &items[index]) != 0 ||
                items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 18) {
            kmipkit_ttlv_structure_t *attributes = 0;
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_client_batch_item_locate(
                    0, 0, 0, 0, 0, 0, 0, 0, attributes, &items[index]) != 0 ||
                items[index] == 0) {
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(attributes);
        } else if (index == 19) {
            static const uint8_t request_identifier_bytes[] = "failed-get";
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier_bytes;
            identifier.text_length = (uint64_t)(sizeof(request_identifier_bytes) - 1);
            if (kmipkit_client_batch_item_get(
                    &identifier, 0, 0, 0, 0, 0, 0, 0, &items[index]) != 0 ||
                items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 20) {
            kmipkit_ttlv_structure_t *attributes = 0;
            if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
                kmipkit_client_batch_item_locate(
                    0, 0, 0, 0, 0, 0, 0, 0, attributes, &items[index]) != 0 ||
                items[index] == 0) {
                kmipkit_ttlv_structure_release(attributes);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_structure_release(attributes);
        } else if (kmipkit_client_batch_item_discover_versions(&items[index]) != 0 ||
                   items[index] == 0) {
            result = 40 + (int)index;
            goto cleanup;
        }
        if (kmipkit_client_batch_item_with_unique_batch_item_id(
                items[index], expected_ids[index], sizeof(expected_ids[index]),
                &identified_item) != 0 || identified_item == 0) {
            result = 60 + (int)index;
            goto cleanup;
        }
        kmipkit_client_batch_item_release(items[index]);
        items[index] = identified_item;
        if (kmipkit_client_batch_add_item(batch, &items[index]) != 0) {
            result = 50 + (int)index;
            goto cleanup;
        }
    }
    if (kmipkit_client_execute((kmipkit_client_t *)limits, &batch, limits, &response)
            != T076_ERROR_INVALID_INPUT ||
        batch == 0 || response != 0) {
        goto cleanup;
    }
    if (kmipkit_client_execute(client, &batch, (kmipkit_codec_limits_t *)client, &response)
            != T076_ERROR_INVALID_INPUT ||
        batch == 0 || response != 0) {
        goto cleanup;
    }
    const int32_t execute_status = kmipkit_client_execute(client, &batch, limits, &response);
    if (execute_status != 0) {
        result = 100 + (int)execute_status;
        goto cleanup;
    }
    batch = 0; /* consumed by execution */
    if (kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != 21) {
        result = 201;
        goto cleanup;
    }
    for (uint64_t index = 0; index < response_count; ++index) {
        kmipkit_client_batch_item_metadata_v1_t metadata = {0};
        kmipkit_byte_span_v1_t correlation = {0};
        kmipkit_byte_span_v1_t unique_id = {0};
        uint8_t correlation_present = 0;
        uint8_t unique_id_present = 0;
        uint8_t versions_present = 0;
        uint64_t version_count = 0;
        kmipkit_protocol_version_v1_t versions[2] = {{0}};
        const uint32_t expected_outcome = index == 1 ? 2U : 1U;
        if (kmipkit_client_batch_response_item_at(response, index, &response_item) != 0 ||
            response_item == 0) {
            goto cleanup;
        }
        metadata.struct_size = (uint32_t)sizeof(metadata);
        metadata.abi_version = 1;
        if (kmipkit_client_batch_item_response_metadata_v1(response_item, &metadata) != 0 ||
            metadata.abi_version != 1 || metadata.outcome_kind != expected_outcome ||
            metadata.result_status != expected_statuses[index]) {
            result = 210 + (int)index;
            goto cleanup;
        }
        if (index == 0) {
            kmipkit_client_batch_item_metadata_v2_t invalid_metadata = {0};
            uint8_t present = 0;
            uint64_t count = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(response_item, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            invalid_metadata.struct_size = (uint32_t)sizeof(invalid_metadata) - 1;
            invalid_metadata.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &invalid_metadata) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            invalid_metadata.struct_size = (uint32_t)sizeof(invalid_metadata);
            invalid_metadata.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &invalid_metadata) != T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &identifier) != T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &present, 0, 1, &count) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
        }
        if (index == 1) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 30 ||
                metadata_v2.outcome_kind != 2) {
                goto cleanup;
            }
        }
        if (index == 4) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t create_present = 0;
            uint32_t object_type = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 2;
            if (kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 ||
                metadata_v2.operation != 1 || metadata_v2.outcome_kind != 1 ||
                metadata_v2.result_status != 0 || metadata_v2.has_result_reason != 0 ||
                metadata_v2.has_result_message != 0 ||
                metadata_v2.result_message_length != 0 || metadata_v2.result_message_data != 0 ||
                kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) != 0 ||
                create_present != 1 || object_type != 7 ||
                identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                identifier.integer_value != -8) {
                goto cleanup;
            }
        }
        if (index == 5) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t create_present = 1;
            uint32_t object_type = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 ||
                metadata_v2.operation != 1 || metadata_v2.outcome_kind != 1 ||
                metadata_v2.result_status != 1 || metadata_v2.has_result_reason != 1 ||
                metadata_v2.result_reason != 5 || metadata_v2.has_result_message != 1 ||
                metadata_v2.result_message_length != sizeof(expected_failure_message) ||
                metadata_v2.result_message_data == 0 ||
                memcmp(metadata_v2.result_message_data, expected_failure_message,
                    sizeof(expected_failure_message)) != 0 ||
                kmipkit_client_batch_item_response_create(
                    response_item, &create_present, &object_type, &identifier) != 0 ||
                create_present != 0) {
                goto cleanup;
            }
        }
        if (index == 6) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t pair_present = 0;
            kmipkit_unique_identifier_v1_t private_identifier = {0};
            kmipkit_unique_identifier_v1_t public_identifier = {0};
            static const uint8_t expected_public_identifier[] = {
                'p', 'u', 'b', 'l', 'i', 'c', '-', 'k', 'e', 'y', '-',
                0xce, 0xb1};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
            private_identifier.abi_version = 1;
            public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
            public_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, 0, &public_identifier) !=
                    T076_ERROR_INVALID_INPUT ||
                kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, 0) !=
                    T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            private_identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
            public_identifier.abi_version = 2;
            if (kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) !=
                T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            public_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 2 ||
                kmipkit_client_batch_item_response_create_key_pair(
                    response_item, &pair_present, &private_identifier, &public_identifier) != 0 ||
                pair_present != 1 ||
                private_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION ||
                private_identifier.enumeration_value != UINT32_MAX ||
                public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
                public_identifier.text_length != sizeof(expected_public_identifier) ||
                public_identifier.text_data == 0 ||
                memcmp(public_identifier.text_data, expected_public_identifier,
                    sizeof(expected_public_identifier)) != 0) {
                goto cleanup;
            }
        }
        if (index == 7) {
            kmipkit_client_batch_item_metadata_v2_t metadata_v2 = {0};
            uint8_t split_present = 0;
            uint64_t split_count = 0;
            kmipkit_unique_identifier_v1_t first_identifier = {0};
            kmipkit_unique_identifier_v1_t middle_identifier = {0};
            kmipkit_unique_identifier_v1_t last_identifier = {0};
            metadata_v2.struct_size = (uint32_t)sizeof(metadata_v2);
            metadata_v2.abi_version = 2;
            if (kmipkit_client_batch_item_response_metadata_v2(
                    response_item, &metadata_v2) != 0 || metadata_v2.operation != 40 ||
                kmipkit_client_batch_item_response_create_split_key_count(
                    response_item, &split_present, &split_count) != 0 ||
                split_present != 1 || split_count != UINT64_C(100001)) {
                goto cleanup;
            }
            first_identifier.struct_size = (uint32_t)sizeof(first_identifier);
            first_identifier.abi_version = 1;
            middle_identifier.struct_size = (uint32_t)sizeof(middle_identifier);
            middle_identifier.abi_version = 1;
            last_identifier.struct_size = (uint32_t)sizeof(last_identifier);
            last_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, 0) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            first_identifier.struct_size -= 1;
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &first_identifier) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            first_identifier.struct_size = (uint32_t)sizeof(first_identifier);
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, split_count, &first_identifier) != T076_ERROR_INVALID_INPUT) {
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, 0, &first_identifier) != 0 ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, UINT64_C(50000), &middle_identifier) != 0 ||
                kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    response_item, split_count - 1, &last_identifier) != 0 ||
                first_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                first_identifier.integer_value != 0 ||
                middle_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                middle_identifier.integer_value != 50000 ||
                last_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                last_identifier.integer_value != 100000) {
                goto cleanup;
            }
        }
        if (index >= 8 && index <= 13 &&
            !successful_crypto_response_is_readable(index, response_item)) {
            goto cleanup;
        }
        if (index == 14 && !derive_key_response_is_readable_from_c(response_item)) {
            goto cleanup;
        }
        if (index == 0 && !export_response_accessors_clear_outputs_from_c(response_item)) {
            goto cleanup;
        }
        if (index == 15 && !export_response_is_readable_from_c(response_item)) {
            goto cleanup;
        }
        if (index == 16) {
            uint8_t present = 0;
            if (kmipkit_client_batch_item_response_get_constraints_ttlv(
                    response_item, &present, &get_constraints_ttlv) != 0 ||
                present != 1 || get_constraints_ttlv == 0) {
                result = 230;
                goto cleanup;
            }
        }
        if (index == 17) {
            uint8_t present = 0;
            uint8_t payload_present = 0;
            uint32_t object_type = 0;
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_get(
                    response_item, &present, &payload_present, &object_type,
                    &identifier, &get_any_object) != 0 ||
                present != 1 || payload_present != 1 || object_type != UINT32_C(0xf1234567) ||
                identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
                identifier.text_length != 6 || identifier.text_data == 0 ||
                memcmp(identifier.text_data, "get-id", 6) != 0 || get_any_object == 0) {
                result = 230;
                goto cleanup;
            }
        }
        if (index == 18) {
            uint8_t present = 0;
            uint8_t payload_present = 0;
            uint8_t located_items_present = 0;
            int32_t located_items = 0;
            uint64_t identifier_count = 0;
            static const uint8_t expected_located_identifiers[3][6] = {
                {'f', 'i', 'r', 's', 't', 0},
                {'s', 'e', 'c', 'o', 'n', 'd'},
                {'f', 'i', 'r', 's', 't', 0},
            };
            static const uint64_t expected_lengths[3] = {5, 6, 5};
            if (kmipkit_client_batch_item_response_locate(
                    response_item, &present, &payload_present, &located_items_present,
                    &located_items, &identifier_count) != 0 ||
                present != 1 || payload_present != 1 || located_items_present != 1 ||
                located_items != 3 || identifier_count != 3) {
                result = 231;
                goto cleanup;
            }
            for (uint64_t identifier_index = 0; identifier_index < identifier_count;
                 ++identifier_index) {
                kmipkit_unique_identifier_v1_t identifier = {0};
                identifier.struct_size = (uint32_t)sizeof(identifier);
                identifier.abi_version = 1;
                if (kmipkit_client_batch_item_response_locate_unique_identifier_at(
                        response_item, identifier_index, &identifier) != 0 ||
                    identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
                    identifier.text_length != expected_lengths[identifier_index] ||
                    identifier.text_data == 0 ||
                    memcmp(identifier.text_data, expected_located_identifiers[identifier_index],
                           (size_t)expected_lengths[identifier_index]) != 0) {
                    result = 232;
                    goto cleanup;
                }
            }
        }
        if (index == 19) {
            uint8_t present = 0;
            uint8_t payload_present = 0;
            uint32_t object_type = UINT32_MAX;
            kmipkit_unique_identifier_v1_t identifier = {0};
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            kmipkit_ttlv_structure_t *object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
            if (kmipkit_client_batch_item_response_get(
                    response_item, &present, &payload_present, &object_type,
                    &identifier, &object) != 0 || present != 1 || payload_present != 0 ||
                object_type != 0 || identifier.kind != 0 || object != 0) {
                result = 233;
                goto cleanup;
            }
        }
        if (index == 20) {
            uint8_t present = 0;
            uint8_t payload_present = 0;
            uint8_t located_items_present = 0;
            int32_t located_items = INT32_MAX;
            uint64_t identifier_count = UINT64_MAX;
            if (kmipkit_client_batch_item_response_locate(
                    response_item, &present, &payload_present, &located_items_present,
                    &located_items, &identifier_count) != 0 || present != 1 ||
                payload_present != 0 || located_items_present != 0 || located_items != 0 ||
                identifier_count != 0) {
                result = 234;
                goto cleanup;
            }
        }
        if (kmipkit_client_batch_item_response_unique_batch_item_id(
                response_item, &unique_id_present, &unique_id) != 0 ||
            unique_id_present != 1 || unique_id.length != sizeof(expected_ids[index]) ||
            unique_id.data == 0 ||
            memcmp(unique_id.data, expected_ids[index], sizeof(expected_ids[index])) != 0) {
            result = 240 + (int)index;
            goto cleanup;
        }
        if (index == 2) {
            if (metadata.has_result_reason != 1 || metadata.result_reason != 1) {
                goto cleanup;
            }
        } else if (index == 5) {
            if (metadata.has_result_reason != 1 || metadata.result_reason != 5) {
                goto cleanup;
            }
        } else if (index == 19 || index == 20) {
            if (metadata.has_result_reason != 1 || metadata.result_reason != 5) {
                goto cleanup;
            }
        } else if (metadata.has_result_reason != 0) {
            goto cleanup;
        }
        if (kmipkit_client_batch_item_response_pending_correlation_value(
                response_item, &correlation_present, &correlation) != 0) {
            goto cleanup;
        }
        if (index == 1) {
            if (correlation_present != 1 || correlation.length != sizeof(expected_correlation) ||
                correlation.data == 0 ||
                memcmp(correlation.data, expected_correlation, sizeof(expected_correlation)) != 0) {
                goto cleanup;
            }
        } else if (correlation_present != 0 || correlation.data != 0 || correlation.length != 0) {
            goto cleanup;
        }
        if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                response_item, &versions_present, 0, 0, &version_count) != 0) {
            goto cleanup;
        }
        if (index == 0) {
            if (versions_present != 1 || version_count != 2) {
                goto cleanup;
            }
            uint64_t copied_count = 0;
            uint8_t insufficient_present = 0;
            uint64_t insufficient_count = 0;
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &insufficient_present, versions, 1, &insufficient_count) !=
                    T076_ERROR_INVALID_INPUT ||
                insufficient_present != 1 || insufficient_count != 2) {
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &versions_present, versions, 2, &copied_count) != 0 ||
                versions_present != 1 || copied_count != 2 ||
                versions[0].major != 2 || versions[0].minor != 1 ||
                versions[1].major != 2 || versions[1].minor != 1) {
                goto cleanup;
            }
        } else if (index == 3) {
            if (versions_present != 1 || version_count != 0) {
                goto cleanup;
            }
        } else if (versions_present != 0 || version_count != 0) {
            goto cleanup;
        }
        kmipkit_client_batch_item_response_release(response_item);
        response_item = 0;
    }
    if (!public_ttlv_contains_response_sentinel(get_any_object)) {
        result = 252;
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(get_any_object);
    get_any_object = 0;
    kmipkit_client_batch_response_release(response);
    response = 0;
    if (batch != 0) {
        result = 250;
        goto cleanup;
    }
    if (!get_constraints_response_ttlv_matches_sentinel(get_constraints_ttlv)) {
        result = 251;
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_ttlv_structure_release(get_constraints_ttlv);
    for (uint64_t index = 0; index < 21; ++index) {
        kmipkit_client_batch_item_release(items[index]);
    }
    kmipkit_ttlv_structure_release(get_any_object);
    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    return result;
}

static int public_ttlv_view_contains_response_sentinel(
    kmipkit_ttlv_structure_view_t *structure_view, uint64_t depth) {
    static const uint8_t expected_value[] = {0xde, 0xad, 0x00, 0xff};
    uint64_t item_count = 0;
    if (depth > UINT64_C(64) ||
        kmipkit_ttlv_structure_view_item_count(structure_view, &item_count) != 0) {
        return 0;
    }

    for (uint64_t index = 0; index < item_count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = 0;
        kmipkit_tag_t *tag = 0;
        kmipkit_ttlv_value_view_t *value_view = 0;
        kmipkit_ttlv_structure_view_t *nested_view = 0;
        uint32_t raw_tag = 0;
        uint8_t item_type = 0;
        int matched = 0;

        if (kmipkit_ttlv_structure_view_item_at(structure_view, index, &item_view) != 0 ||
            kmipkit_ttlv_item_view_tag(item_view, &tag) != 0 ||
            kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 ||
            kmipkit_ttlv_item_view_type(item_view, &item_type) != 0 ||
            kmipkit_ttlv_item_view_value(item_view, &value_view) != 0) {
            goto item_cleanup;
        }
        if ((raw_tag == UINT32_C(0x540001) || raw_tag == UINT32_C(0x541234)) &&
            item_type == KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING) {
            uint64_t value_length = 0;
            if (kmipkit_ttlv_value_view_byte_length(value_view, &value_length) == 0 &&
                value_length == (uint64_t)sizeof(expected_value)) {
                matched = 1;
                for (uint64_t byte_index = 0; byte_index < value_length; ++byte_index) {
                    uint8_t value_byte = 0;
                    if (kmipkit_ttlv_value_view_byte_at(
                            value_view, byte_index, &value_byte) != 0 ||
                        value_byte != expected_value[byte_index]) {
                        matched = 0;
                        break;
                    }
                }
            }
        }
        if (!matched && item_type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE && depth < UINT64_C(64) &&
            kmipkit_ttlv_value_view_structure(value_view, &nested_view) == 0) {
            matched = public_ttlv_view_contains_response_sentinel(nested_view, depth + 1);
        }

item_cleanup:
        kmipkit_ttlv_structure_view_release(nested_view);
        kmipkit_ttlv_value_view_release(value_view);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item_view);
        if (matched) {
            return 1;
        }
    }
    return 0;
}

static int public_ttlv_contains_response_sentinel(kmipkit_ttlv_structure_t *structure) {
    kmipkit_ttlv_structure_view_t *structure_view = 0;
    int matched = 0;
    if (kmipkit_ttlv_structure_view(structure, &structure_view) == 0) {
        matched = public_ttlv_view_contains_response_sentinel(structure_view, 0);
    }
    kmipkit_ttlv_structure_view_release(structure_view);
    return matched;
}

int kmipkit_client_abi_c_consumer_validate_usage_allocation_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t count = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    const int32_t status = kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
        item, &present, &response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (status != 0 || present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 ||
        count != (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0]))) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t raw_tag = 0;
        const int valid = kmipkit_ttlv_structure_view_item_at(view, index, &item_view) == 0 &&
            kmipkit_ttlv_item_view_tag(item_view, &tag) == 0 &&
            kmipkit_ttlv_tag_value(tag, &raw_tag) == 0 && raw_tag == expected_tags[index];
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item_view);
        if (!valid) {
            goto cleanup;
        }
    }
    if (!public_ttlv_view_contains_response_sentinel(view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_certify_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t identifier_present = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t count = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    const int32_t identifier_status = kmipkit_client_batch_item_response_certify(
        item, &identifier_present, &identifier);
    if (identifier_status != 0 || identifier_present != 1 ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_length != UINT64_C(18) || identifier.text_data == 0) {
        kmipkit_client_batch_item_response_release(item);
        return 0;
    }
    const int32_t ttlv_status = kmipkit_client_batch_item_response_certify_ttlv(
        item, &ttlv_present, &response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (ttlv_status != 0 || ttlv_present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 ||
        count != (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0]))) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t raw_tag = 0;
        const int valid = kmipkit_ttlv_structure_view_item_at(view, index, &item_view) == 0 &&
            kmipkit_ttlv_item_view_tag(item_view, &tag) == 0 &&
            kmipkit_ttlv_tag_value(tag, &raw_tag) == 0 && raw_tag == expected_tags[index];
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item_view);
        if (!valid) {
            goto cleanup;
        }
    }
    if (!public_ttlv_view_contains_response_sentinel(view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_execute_error_ttlv_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = 0;
    kmipkit_client_configuration_t *configuration = 0;
    kmipkit_transport_config_t *transport = 0;
    kmipkit_client_t *client = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_item_t *identified_item = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_ttlv_structure_t *error_response = 0;
    kmipkit_ttlv_value_t *mac_data = 0;
    char *port_end = 0;
    unsigned long port = 0;
    int result = 1;

    if (argc != 6 || argv == 0 || limits == 0 || argv[1] == 0 || argv[2] == 0 ||
        argv[3] == 0 || argv[4] == 0 || argv[5] == 0) {
        return 2;
    }
    port = strtoul(argv[5], &port_end, 10);
    if (port_end == argv[5] || *port_end != '\0' || port == 0 || port > 65535UL) {
        return 3;
    }
    client_certificates[0].source_kind = 2;
    client_certificates[0].encoding = 2;
    client_certificates[0].data.data = (const uint8_t *)argv[1];
    client_certificates[0].data.length = (uint64_t)strlen(argv[1]);
    client_certificates[1] = client_certificates[0];
    client_certificates[1].data.data = (const uint8_t *)argv[2];
    client_certificates[1].data.length = (uint64_t)strlen(argv[2]);
    trust_certificates[0].source_kind = 2;
    trust_certificates[0].encoding = 2;
    trust_certificates[0].data.data = (const uint8_t *)argv[4];
    trust_certificates[0].data.length = (uint64_t)strlen(argv[4]);

    transport_input.struct_size = (uint32_t)sizeof(transport_input);
    transport_input.abi_version = 1;
    transport_input.endpoint_kind = 1;
    transport_input.trust_kind = 2;
    transport_input.flags = 1;
    transport_input.endpoint.data = host;
    transport_input.endpoint.length = (uint64_t)(sizeof(host) - 1);
    transport_input.endpoint_port = (uint16_t)port;
    transport_input.tls_server_name.data = server_name;
    transport_input.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    transport_input.client_certificates = client_certificates;
    transport_input.client_certificate_count = 2;
    transport_input.client_private_key.source_kind = 2;
    transport_input.client_private_key.encoding = 2;
    transport_input.client_private_key.data.data = (const uint8_t *)argv[3];
    transport_input.client_private_key.data.length = (uint64_t)strlen(argv[3]);
    transport_input.trust_certificates = trust_certificates;
    transport_input.trust_certificate_count = 1;
    transport_input.max_request_bytes = UINT64_C(1048576);

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = 1;
    if (kmipkit_extension_registry_limits_v1_default_values(&registry_limits) != 0 ||
        kmipkit_client_extension_registry_create_v1(0, 0, &registry_limits, &registry) != 0 ||
        kmipkit_client_configuration_create(registry, &configuration) != 0) {
        goto cleanup;
    }
    registry = 0;
    if (kmipkit_transport_config_create_v1(&transport_input, &transport) != 0 ||
        kmipkit_client_create(configuration, &transport, &client) != 0) {
        goto cleanup;
    }
    transport = 0;
    static const uint8_t request_data[] = {0x41, 0x42};
    if (kmipkit_client_batch_create(&batch) != 0 ||
        kmipkit_ttlv_value_byte_string(
            limits, request_data, (uint64_t)sizeof(request_data), &mac_data) != 0 ||
        kmipkit_client_batch_item_mac(0, 0, mac_data, 0, 0, 0, &item) != 0 ||
        kmipkit_client_batch_item_with_unique_batch_item_id(
            item, (const uint8_t *)"item-0", UINT64_C(6), &identified_item) != 0) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(mac_data);
    mac_data = 0;
    kmipkit_client_batch_item_release(item);
    item = identified_item;
    identified_item = 0;
    if (kmipkit_client_batch_add_item(batch, &item) != 0) {
        goto cleanup;
    }
    const int32_t status = kmipkit_client_execute_with_error_ttlv_v1(
        client, &batch, limits, &response, &error_response);
    if (status != KMIPKIT_ERROR_PROTOCOL || batch != 0 || response != 0 ||
        error_response == 0 || !public_ttlv_contains_response_sentinel(error_response)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_release(mac_data);
    kmipkit_ttlv_structure_release(error_response);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_item_release(identified_item);
    kmipkit_client_batch_item_release(item);
    kmipkit_client_batch_release(batch);
    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    return result;
}

int kmipkit_client_abi_c_consumer_execute_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    if (argc != 6 || limits == 0) {
        return 22;
    }
    return execute_public_tls_request(argv, limits);
}

int main(int argc, char **argv) {
    if (!execute_with_options_and_error_ttlv_clears_outputs()) {
        return 22;
    }
    if (!execute_with_error_ttlv_clears_outputs()) {
        return 23;
    }
    kmipkit_client_t *client = 0;
    if (!api_level_query_matches_shared_fixture_locally(argc, argv)) {
        return 100;
    }
    if (kmipkit_client_create(0, 0, &client) != T076_ERROR_INVALID_INPUT) {
        return 1;
    }
    if (client != 0) {
        return 2;
    }
    if (!creation_create_matches_shared_fixture() ||
        !certify_constructor_is_callable_from_c() ||
        !certify_response_accessor_is_callable_from_c() ||
        !check_constructor_is_callable_from_c() ||
        !delegated_login_constructor_is_callable_from_c() ||
        !derive_key_constructor_is_callable_from_c() ||
        !export_constructor_is_callable_from_c() ||
        !export_constructor_rejects_invalid_presence_from_c() ||
        !delegated_login_response_accessors_clear_outputs_from_c() ||
        !check_response_accessors_clear_outputs_from_c() ||
        !get_constraints_constructor_is_callable_from_c() ||
        !get_constraints_response_accessor_clears_outputs_from_c() ||
        !get_locate_abi_is_callable_from_c() ||
        !get_usage_allocation_constructor_is_callable_from_c() ||
        !get_usage_allocation_response_accessor_clears_outputs() ||
        !creation_create_key_pair_matches_shared_fixture() ||
        !creation_create_split_key_matches_shared_fixture() ||
        !creation_rejects_malformed_split_key_inputs() ||
        !creation_rejects_missing_required_attributes() ||
        !cryptographic_abi_symbols_are_callable()) {
        return 7;
    }

    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_transport_config_t *transport = 0;
    if (kmipkit_transport_config_create_v1(&transport_input, &transport)
            != T076_ERROR_INVALID_INPUT) {
        return 3;
    }
    if (transport != 0) {
        return 4;
    }

    kmipkit_request_options_v1_t request_options = {0};
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = 1;
    kmipkit_client_batch_response_t *response = 0;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 5;
    }
    if (!execute_with_options_returns(0, T076_ERROR_INVALID_INPUT)) {
        return 6;
    }
    request_options.connect_timeout.kind = 1;
    request_options.connect_timeout.seconds = 1;
    request_options.connect_timeout.nanoseconds = 999999999;
    request_options.write_timeout.kind = 2;
    request_options.read_timeout.kind = 1;
    request_options.read_timeout.nanoseconds = 0;
    request_options.total_timeout.kind = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 7;
    }
    request_options = (kmipkit_request_options_v1_t){0};
    request_options.struct_size = (uint32_t)sizeof(request_options) - 1;
    request_options.abi_version = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 8;
    }
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = 2;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 9;
    }
    request_options.abi_version = 1;
    request_options.connect_timeout.reserved = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 10;
    }
    request_options.connect_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.write_timeout.reserved2 = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 11;
    }
    request_options.write_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.read_timeout.seconds = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 12;
    }
    request_options.read_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.total_timeout.kind = 1;
    request_options.total_timeout.nanoseconds = 1000000000;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 13;
    }
    request_options.total_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.connect_timeout.kind = 2;
    request_options.connect_timeout.seconds = 1;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 14;
    }
    request_options.connect_timeout = (kmipkit_timeout_limit_v1_t){0};
    request_options.write_timeout.kind = 99;
    if (!execute_with_options_returns(&request_options, T076_ERROR_INVALID_INPUT)) {
        return 15;
    }
    if (kmipkit_client_execute(0, 0, 0, &response) != T076_ERROR_INVALID_INPUT) {
        return 16;
    }
    kmipkit_client_batch_t *empty_batch = 0;
    if (kmipkit_client_execute(0, &empty_batch, 0, &response) != T076_ERROR_INVALID_INPUT) {
        return 17;
    }

    kmipkit_client_batch_t *batch = 0;
    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0) {
        return 18;
    }
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_discover_versions(&item) != 0 || item == 0) {
        kmipkit_client_batch_release(batch);
        return 19;
    }
    if (kmipkit_client_batch_add_item(batch, &item) != 0 || item != 0) {
        kmipkit_client_batch_item_release(item);
        kmipkit_client_batch_release(batch);
        return 20;
    }
    kmipkit_client_batch_release(batch);
    if (argc != 4) {
        return 21;
    }
    return 0;
}
