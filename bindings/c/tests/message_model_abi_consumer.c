#include <stddef.h>
#include <stdint.h>

#include "kmipkit.h"

/* Private prototypes for KMIPKIT-0006 declarations withheld from the public header until
 * Java and Python parity is complete. The manifest-generated internal header will replace these. */
struct kmipkit_request_message_t;
struct kmipkit_request_header_view_t;
struct kmipkit_request_batch_item_view_t;
struct kmipkit_response_message_t;
struct kmipkit_response_header_view_t;
struct kmipkit_response_batch_item_view_t;
struct kmipkit_message_extension_view_t;
struct kmipkit_message_validation_error_t;

int32_t kmipkit_request_message_from_ttlv(
    kmipkit_ttlv_structure_t *structure,
    struct kmipkit_request_message_t **out_message,
    struct kmipkit_message_validation_error_t **out_validation_error);
int32_t kmipkit_request_message_header(
    struct kmipkit_request_message_t *message,
    struct kmipkit_request_header_view_t **out_header);
int32_t kmipkit_request_header_view_protocol_version(
    struct kmipkit_request_header_view_t *header,
    kmipkit_protocol_version_v1_t *out_version);
int32_t kmipkit_request_header_view_effective_asynchronous_indicator(
    struct kmipkit_request_header_view_t *header,
    uint32_t *out_indicator);
int32_t kmipkit_request_header_view_effective_attestation_capable_indicator(
    struct kmipkit_request_header_view_t *header,
    uint8_t *out_indicator);
int32_t kmipkit_request_header_view_effective_batch_error_continuation_option(
    struct kmipkit_request_header_view_t *header,
    uint32_t *out_option);
int32_t kmipkit_request_header_view_effective_batch_order_option(
    struct kmipkit_request_header_view_t *header,
    uint8_t *out_option);
int32_t kmipkit_request_header_view_ttlv(
    struct kmipkit_request_header_view_t *header,
    kmipkit_ttlv_structure_t **out_structure);
int32_t kmipkit_request_message_batch_item_count(
    struct kmipkit_request_message_t *message,
    uint64_t *out_count);
int32_t kmipkit_request_message_batch_item_at(
    struct kmipkit_request_message_t *message,
    uint64_t index,
    struct kmipkit_request_batch_item_view_t **out_item);
int32_t kmipkit_request_batch_item_view_operation(
    struct kmipkit_request_batch_item_view_t *item,
    uint8_t *out_present,
    uint32_t *out_operation);
int32_t kmipkit_request_batch_item_view_ttlv(
    struct kmipkit_request_batch_item_view_t *item,
    kmipkit_ttlv_structure_t **out_structure);
int32_t kmipkit_request_batch_item_extension_count(
    struct kmipkit_request_batch_item_view_t *item,
    uint64_t *out_count);
int32_t kmipkit_request_batch_item_extension_at(
    struct kmipkit_request_batch_item_view_t *item,
    uint64_t index,
    struct kmipkit_message_extension_view_t **out_extension);
int32_t kmipkit_message_extension_view_ttlv(
    struct kmipkit_message_extension_view_t *extension,
    kmipkit_ttlv_structure_t **out_structure);
void kmipkit_request_message_release(struct kmipkit_request_message_t *handle);
void kmipkit_request_header_view_release(struct kmipkit_request_header_view_t *handle);
void kmipkit_request_batch_item_release(struct kmipkit_request_batch_item_view_t *handle);
void kmipkit_message_validation_error_release(
    struct kmipkit_message_validation_error_t *handle);
int32_t kmipkit_message_validation_error_kind(
    struct kmipkit_message_validation_error_t *error,
    uint32_t *out_kind);
int32_t kmipkit_message_validation_error_structure_index(
    struct kmipkit_message_validation_error_t *error,
    uint8_t *out_present,
    uint64_t *out_index);
int32_t kmipkit_message_validation_error_field_index(
    struct kmipkit_message_validation_error_t *error,
    uint8_t *out_present,
    uint64_t *out_index);

int32_t kmipkit_response_message_from_ttlv(
    kmipkit_ttlv_structure_t *structure,
    struct kmipkit_response_message_t **out_message,
    struct kmipkit_message_validation_error_t **out_validation_error);
int32_t kmipkit_response_message_header(
    struct kmipkit_response_message_t *message,
    struct kmipkit_response_header_view_t **out_header);
int32_t kmipkit_response_header_view_protocol_version(
    struct kmipkit_response_header_view_t *header,
    kmipkit_protocol_version_v1_t *out_version);
int32_t kmipkit_response_header_view_time_stamp(
    struct kmipkit_response_header_view_t *header,
    int64_t *out_time_stamp);
int32_t kmipkit_response_message_batch_item_count(
    struct kmipkit_response_message_t *message,
    uint64_t *out_count);
int32_t kmipkit_response_message_batch_item_at(
    struct kmipkit_response_message_t *message,
    uint64_t index,
    struct kmipkit_response_batch_item_view_t **out_item);
int32_t kmipkit_response_batch_item_view_result_status(
    struct kmipkit_response_batch_item_view_t *item,
    uint8_t *out_present,
    uint32_t *out_status);
int32_t kmipkit_response_batch_item_extension_count(
    struct kmipkit_response_batch_item_view_t *item,
    uint64_t *out_count);
int32_t kmipkit_response_batch_item_extension_at(
    struct kmipkit_response_batch_item_view_t *item,
    uint64_t index,
    struct kmipkit_message_extension_view_t **out_extension);
int32_t kmipkit_message_extension_view_ttlv(
    struct kmipkit_message_extension_view_t *extension,
    kmipkit_ttlv_structure_t **out_structure);
void kmipkit_response_message_release(struct kmipkit_response_message_t *handle);
void kmipkit_response_header_view_release(struct kmipkit_response_header_view_t *handle);
void kmipkit_response_batch_item_release(struct kmipkit_response_batch_item_view_t *handle);
void kmipkit_message_extension_view_release(struct kmipkit_message_extension_view_t *handle);

static int c_ttlv_structure_has_tags(
    kmipkit_ttlv_structure_t *structure,
    const uint32_t *expected_tags,
    uint64_t expected_count) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t actual_count = 0;
    int result = -1;

    if (structure == NULL || expected_tags == NULL ||
        kmipkit_ttlv_structure_view(structure, &view) != 0 || view == NULL ||
        kmipkit_ttlv_structure_view_item_count(view, &actual_count) != 0 ||
        actual_count != expected_count) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < expected_count; index++) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_tag_t *tag = NULL;
        uint32_t raw_tag = 0;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == NULL ||
            kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
            kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 || raw_tag != expected_tags[index]) {
            kmipkit_tag_release(tag);
            kmipkit_ttlv_item_view_release(item);
            goto cleanup;
        }
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    return result;
}

static int c_ttlv_structure_view_integer_equals(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    uint32_t expected_value) {
    kmipkit_ttlv_item_view_t *item = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    uint32_t actual_tag = 0;
    int32_t actual_value = 0;
    int result = -1;

    if (view == NULL || kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 ||
        item == NULL || kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
        kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL ||
        kmipkit_ttlv_value_view_integer(value, &actual_value) != 0 ||
        (uint32_t)actual_value != expected_value) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return result;
}

static int c_ttlv_structure_view_bytes_equal(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    const uint8_t *expected_bytes,
    uint64_t expected_length) {
    kmipkit_ttlv_item_view_t *item = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    uint32_t actual_tag = 0;
    uint64_t actual_length = 0;
    int result = -1;

    if (view == NULL || (expected_length > 0 && expected_bytes == NULL) ||
        kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == NULL ||
        kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
        kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL ||
        kmipkit_ttlv_value_view_byte_length(value, &actual_length) != 0 ||
        actual_length != expected_length) {
        goto cleanup;
    }
    for (uint64_t byte_index = 0; byte_index < expected_length; byte_index++) {
        uint8_t actual_byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, byte_index, &actual_byte) != 0 ||
            actual_byte != expected_bytes[byte_index]) {
            goto cleanup;
        }
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return result;
}

static int c_ttlv_structure_view_boolean_equals(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    uint8_t expected_value) {
    kmipkit_ttlv_item_view_t *item = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    uint32_t actual_tag = 0;
    uint8_t actual_value = 0;
    int result = -1;

    if (view == NULL || kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 ||
        item == NULL || kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
        kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL ||
        kmipkit_ttlv_value_view_boolean(value, &actual_value) != 0 ||
        actual_value != expected_value) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return result;
}

static int c_ttlv_structure_view_nested_at(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    kmipkit_ttlv_structure_view_t **out_nested) {
    kmipkit_ttlv_item_view_t *item = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    uint32_t actual_tag = 0;
    int result = -1;

    if (out_nested == NULL || view == NULL ||
        kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == NULL ||
        kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
        kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL ||
        kmipkit_ttlv_value_view_structure(value, out_nested) != 0 || *out_nested == NULL) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return result;
}

static int c_ttlv_structure_view_nested_integer_equals(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t nested_index,
    uint32_t nested_tag,
    uint64_t value_index,
    uint32_t value_tag,
    uint32_t expected_value) {
    kmipkit_ttlv_structure_view_t *nested = NULL;
    int result = -1;

    if (c_ttlv_structure_view_nested_at(view, nested_index, nested_tag, &nested) == 0 &&
        c_ttlv_structure_view_integer_equals(nested, value_index, value_tag, expected_value) == 0) {
        result = 0;
    }
    kmipkit_ttlv_structure_view_release(nested);
    return result;
}

int kmipkit_message_model_c_consumer_typed_parity_request(
    kmipkit_ttlv_structure_t *structure,
    uint32_t expected_operation_0,
    uint32_t expected_operation_1,
    uint32_t expected_asynchronous_default,
    uint8_t expected_attestation_default,
    uint32_t expected_batch_error_default,
    uint8_t expected_batch_order_default,
    uint64_t expected_extension_item_index,
    const uint8_t *expected_id_0,
    uint64_t expected_id_0_length,
    const uint8_t *expected_id_1,
    uint64_t expected_id_1_length,
    uint32_t expected_bit_mask_0,
    uint32_t expected_bit_mask_1,
    const uint8_t *expected_extension_value,
    uint64_t expected_extension_value_length,
    uint8_t expected_extension_criticality) {
    static const uint32_t expected_header_tags[] = {UINT32_C(0x00420069), UINT32_C(0x0042000D)};
    static const uint32_t expected_first_item_tags[] = {
        UINT32_C(0x0042005C), UINT32_C(0x00420093), UINT32_C(0x00420079)};
    static const uint32_t expected_second_item_tags[] = {
        UINT32_C(0x0042005C), UINT32_C(0x00420093), UINT32_C(0x00420079),
        UINT32_C(0x00420051)};
    struct kmipkit_request_message_t *message = NULL;
    struct kmipkit_request_header_view_t *header = NULL;
    struct kmipkit_request_batch_item_view_t *first = NULL;
    struct kmipkit_request_batch_item_view_t *second = NULL;
    struct kmipkit_message_extension_view_t *extension = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_ttlv_structure_t *header_ttlv = NULL;
    kmipkit_ttlv_structure_t *first_ttlv = NULL;
    kmipkit_ttlv_structure_t *second_ttlv = NULL;
    kmipkit_ttlv_structure_t *extension_ttlv = NULL;
    kmipkit_ttlv_structure_view_t *first_view = NULL;
    kmipkit_ttlv_structure_view_t *second_view = NULL;
    kmipkit_ttlv_structure_view_t *extension_view = NULL;
    kmipkit_ttlv_structure_view_t *vendor_extension_view = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    uint64_t item_count = 0;
    uint64_t first_extension_count = 0;
    uint64_t second_extension_count = 0;
    uint32_t asynchronous_default = 0;
    uint32_t batch_error_default = 0;
    uint8_t attestation_default = 0;
    uint8_t batch_order_default = 0;
    uint8_t has_operation = 0;
    uint32_t operation = 0;
    int result = -1;

    if (kmipkit_request_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL ||
        kmipkit_request_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_request_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_request_header_view_effective_asynchronous_indicator(
            header, &asynchronous_default) != 0 ||
        asynchronous_default != expected_asynchronous_default ||
        kmipkit_request_header_view_effective_attestation_capable_indicator(
            header, &attestation_default) != 0 ||
        attestation_default != expected_attestation_default ||
        kmipkit_request_header_view_effective_batch_error_continuation_option(
            header, &batch_error_default) != 0 ||
        batch_error_default != expected_batch_error_default ||
        kmipkit_request_header_view_effective_batch_order_option(
            header, &batch_order_default) != 0 ||
        batch_order_default != expected_batch_order_default ||
        kmipkit_request_header_view_ttlv(header, &header_ttlv) != 0 || header_ttlv == NULL ||
        c_ttlv_structure_has_tags(header_ttlv, expected_header_tags, 2) != 0 ||
        kmipkit_request_message_batch_item_count(message, &item_count) != 0 || item_count != 2 ||
        kmipkit_request_message_batch_item_at(message, 0, &first) != 0 || first == NULL ||
        kmipkit_request_message_batch_item_at(message, 1, &second) != 0 || second == NULL ||
        kmipkit_request_batch_item_view_operation(first, &has_operation, &operation) != 0 ||
        has_operation != 1 || operation != expected_operation_0 ||
        kmipkit_request_batch_item_view_operation(second, &has_operation, &operation) != 0 ||
        has_operation != 1 || operation != expected_operation_1 ||
        kmipkit_request_batch_item_view_ttlv(first, &first_ttlv) != 0 || first_ttlv == NULL ||
        c_ttlv_structure_has_tags(first_ttlv, expected_first_item_tags, 3) != 0 ||
        kmipkit_ttlv_structure_view(first_ttlv, &first_view) != 0 || first_view == NULL ||
        c_ttlv_structure_view_bytes_equal(
            first_view, 1, UINT32_C(0x00420093), expected_id_0, expected_id_0_length) != 0 ||
        c_ttlv_structure_view_nested_integer_equals(
            first_view, 2, UINT32_C(0x00420079), 0, UINT32_C(0x0042002C),
            expected_bit_mask_0) != 0 ||
        kmipkit_request_batch_item_view_ttlv(second, &second_ttlv) != 0 || second_ttlv == NULL ||
        c_ttlv_structure_has_tags(second_ttlv, expected_second_item_tags, 4) != 0 ||
        kmipkit_ttlv_structure_view(second_ttlv, &second_view) != 0 || second_view == NULL ||
        c_ttlv_structure_view_bytes_equal(
            second_view, 1, UINT32_C(0x00420093), expected_id_1, expected_id_1_length) != 0 ||
        c_ttlv_structure_view_nested_integer_equals(
            second_view, 2, UINT32_C(0x00420079), 0, UINT32_C(0x0042002C),
            expected_bit_mask_1) != 0 ||
        kmipkit_request_batch_item_extension_count(first, &first_extension_count) != 0 ||
        first_extension_count != 0 ||
        kmipkit_request_batch_item_extension_count(second, &second_extension_count) != 0 ||
        second_extension_count != 1 || expected_extension_item_index != 1 ||
        kmipkit_request_batch_item_extension_at(second, 0, &extension) != 0 || extension == NULL) {
        goto cleanup;
    }

    kmipkit_request_message_release(message);
    message = NULL;
    kmipkit_request_batch_item_release(first);
    first = NULL;
    kmipkit_request_batch_item_release(second);
    second = NULL;
    if (kmipkit_request_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_message_extension_view_ttlv(extension, &extension_ttlv) != 0 ||
        extension_ttlv == NULL) {
        goto cleanup;
    }
    {
        static const uint32_t expected_extension_tags[] = {
            UINT32_C(0x0042009D), UINT32_C(0x00420026), UINT32_C(0x0042009C)};
        static const uint8_t expected_vendor_id[] = {'A', 'c', 'm', 'e'};
        if (c_ttlv_structure_has_tags(extension_ttlv, expected_extension_tags, 3) != 0 ||
            kmipkit_ttlv_structure_view(extension_ttlv, &extension_view) != 0 ||
            extension_view == NULL ||
            c_ttlv_structure_view_bytes_equal(
                extension_view, 0, UINT32_C(0x0042009D), expected_vendor_id,
                sizeof(expected_vendor_id)) != 0 ||
            c_ttlv_structure_view_boolean_equals(
                extension_view, 1, UINT32_C(0x00420026), expected_extension_criticality) != 0 ||
            c_ttlv_structure_view_nested_at(
                extension_view, 2, UINT32_C(0x0042009C), &vendor_extension_view) != 0 ||
            c_ttlv_structure_view_bytes_equal(
                vendor_extension_view, 0, UINT32_C(0x00540001), expected_extension_value,
                expected_extension_value_length) != 0) {
            goto cleanup;
        }
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_view_release(vendor_extension_view);
    kmipkit_ttlv_structure_view_release(extension_view);
    kmipkit_ttlv_structure_view_release(second_view);
    kmipkit_ttlv_structure_view_release(first_view);
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_ttlv_structure_release(second_ttlv);
    kmipkit_ttlv_structure_release(first_ttlv);
    kmipkit_ttlv_structure_release(header_ttlv);
    kmipkit_message_extension_view_release(extension);
    kmipkit_request_batch_item_release(second);
    kmipkit_request_batch_item_release(first);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}

int kmipkit_message_model_c_consumer_typed_parity_invalid_request(
    kmipkit_ttlv_structure_t *structure,
    uint32_t expected_kind,
    uint64_t expected_structure_index) {
    struct kmipkit_request_message_t *message = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    uint32_t kind = 0;
    uint8_t has_structure_index = 0;
    uint8_t has_field_index = 0;
    uint64_t structure_index = 0;
    uint64_t field_index = 0;
    int result = -1;

    if (kmipkit_request_message_from_ttlv(structure, &message, &validation_error) == 0 ||
        message != NULL || validation_error == NULL ||
        kmipkit_message_validation_error_kind(validation_error, &kind) != 0 ||
        kind != expected_kind ||
        kmipkit_message_validation_error_structure_index(
            validation_error, &has_structure_index, &structure_index) != 0 ||
        has_structure_index != 1 || structure_index != expected_structure_index ||
        kmipkit_message_validation_error_field_index(
            validation_error, &has_field_index, &field_index) != 0 ||
        has_field_index != 0) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_message_validation_error_release(validation_error);
    kmipkit_request_message_release(message);
    return result;
}

int kmipkit_message_model_c_consumer_typed_parity_response(
    kmipkit_ttlv_structure_t *structure,
    uint32_t expected_operation,
    uint32_t expected_status,
    uint32_t expected_bit_mask,
    int64_t expected_time_stamp,
    uint64_t expected_extension_item_index,
    const uint8_t *expected_extension_value,
    uint64_t expected_extension_value_length,
    uint8_t expected_extension_criticality) {
    static const uint32_t expected_header_tags[] = {
        UINT32_C(0x00420069), UINT32_C(0x00420092), UINT32_C(0x0042000D)};
    static const uint32_t expected_item_tags[] = {
        UINT32_C(0x0042005C), UINT32_C(0x0042007F), UINT32_C(0x0042007C),
        UINT32_C(0x00420051)};
    static const uint8_t expected_vendor_id[] = {'A', 'c', 'm', 'e'};
    struct kmipkit_response_message_t *message = NULL;
    struct kmipkit_response_header_view_t *header = NULL;
    struct kmipkit_response_batch_item_view_t *item = NULL;
    struct kmipkit_message_extension_view_t *extension = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_ttlv_structure_t *header_ttlv = NULL;
    kmipkit_ttlv_structure_t *item_ttlv = NULL;
    kmipkit_ttlv_structure_t *extension_ttlv = NULL;
    kmipkit_ttlv_structure_view_t *item_view = NULL;
    kmipkit_ttlv_structure_view_t *extension_view = NULL;
    kmipkit_ttlv_structure_view_t *vendor_extension_view = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    int64_t time_stamp = 0;
    uint64_t item_count = 0;
    uint64_t extension_count = 0;
    uint8_t has_operation = 0;
    uint8_t has_status = 0;
    uint32_t operation = 0;
    uint32_t status = 0;
    int result = -1;

    if (kmipkit_response_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL ||
        kmipkit_response_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_response_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_response_header_view_time_stamp(header, &time_stamp) != 0 ||
        time_stamp != expected_time_stamp ||
        kmipkit_response_header_view_ttlv(header, &header_ttlv) != 0 || header_ttlv == NULL ||
        c_ttlv_structure_has_tags(header_ttlv, expected_header_tags, 3) != 0 ||
        kmipkit_response_message_batch_item_count(message, &item_count) != 0 || item_count != 1 ||
        kmipkit_response_message_batch_item_at(message, 0, &item) != 0 || item == NULL ||
        kmipkit_response_batch_item_view_operation(item, &has_operation, &operation) != 0 ||
        has_operation != 1 || operation != expected_operation ||
        kmipkit_response_batch_item_view_result_status(item, &has_status, &status) != 0 ||
        has_status != 1 || status != expected_status ||
        kmipkit_response_batch_item_view_ttlv(item, &item_ttlv) != 0 || item_ttlv == NULL ||
        c_ttlv_structure_has_tags(item_ttlv, expected_item_tags, 4) != 0 ||
        kmipkit_ttlv_structure_view(item_ttlv, &item_view) != 0 || item_view == NULL ||
        c_ttlv_structure_view_nested_integer_equals(
            item_view, 2, UINT32_C(0x0042007C), 0, UINT32_C(0x0042002C),
            expected_bit_mask) != 0 ||
        kmipkit_response_batch_item_extension_count(item, &extension_count) != 0 ||
        extension_count != 1 || expected_extension_item_index != 0 ||
        kmipkit_response_batch_item_extension_at(item, 0, &extension) != 0 || extension == NULL) {
        goto cleanup;
    }

    kmipkit_response_message_release(message);
    message = NULL;
    kmipkit_response_batch_item_release(item);
    item = NULL;
    kmipkit_response_header_view_release(header);
    header = NULL;
    if (kmipkit_message_extension_view_ttlv(extension, &extension_ttlv) != 0 ||
        extension_ttlv == NULL ||
        kmipkit_ttlv_structure_view(extension_ttlv, &extension_view) != 0 ||
        extension_view == NULL ||
        c_ttlv_structure_view_bytes_equal(
            extension_view, 0, UINT32_C(0x0042009D), expected_vendor_id,
            sizeof(expected_vendor_id)) != 0 ||
        c_ttlv_structure_view_boolean_equals(
            extension_view, 1, UINT32_C(0x00420026), expected_extension_criticality) != 0 ||
        c_ttlv_structure_view_nested_at(
            extension_view, 2, UINT32_C(0x0042009C), &vendor_extension_view) != 0 ||
        c_ttlv_structure_view_bytes_equal(
            vendor_extension_view, 0, UINT32_C(0x00540001), expected_extension_value,
            expected_extension_value_length) != 0) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_view_release(vendor_extension_view);
    kmipkit_ttlv_structure_view_release(extension_view);
    kmipkit_ttlv_structure_view_release(item_view);
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_ttlv_structure_release(item_ttlv);
    kmipkit_ttlv_structure_release(header_ttlv);
    kmipkit_message_extension_view_release(extension);
    kmipkit_response_batch_item_release(item);
    kmipkit_response_header_view_release(header);
    kmipkit_response_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}

int32_t kmipkit_message_model_c_consumer_validate_request(kmipkit_ttlv_structure_t *structure) {
    struct kmipkit_request_message_t *message = NULL;
    struct kmipkit_request_header_view_t *header = NULL;
    struct kmipkit_request_batch_item_view_t *item = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    uint64_t item_count = 0;
    uint8_t has_operation = 0;
    uint32_t operation = 0;
    int32_t result = -1;

    if (kmipkit_request_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL) {
        goto cleanup;
    }
    if (kmipkit_request_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_request_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_request_message_batch_item_count(message, &item_count) != 0 || item_count != 1 ||
        kmipkit_request_message_batch_item_at(message, 0, &item) != 0 || item == NULL ||
        kmipkit_request_batch_item_view_operation(item, &has_operation, &operation) != 0 ||
        has_operation != 1 || operation != UINT32_C(0xF1234567)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_request_batch_item_release(item);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}

int32_t kmipkit_message_model_c_consumer_validate_response(kmipkit_ttlv_structure_t *structure) {
    struct kmipkit_response_message_t *message = NULL;
    struct kmipkit_response_header_view_t *header = NULL;
    struct kmipkit_response_batch_item_view_t *item = NULL;
    struct kmipkit_message_extension_view_t *extension = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    kmipkit_ttlv_structure_t *extension_ttlv = NULL;
    int64_t time_stamp = 0;
    uint64_t item_count = 0;
    uint64_t extension_count = 0;
    uint8_t has_status = 0;
    uint32_t status = 0;
    int32_t result = -1;

    if (kmipkit_response_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL) {
        goto cleanup;
    }
    if (kmipkit_response_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_response_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_response_header_view_time_stamp(header, &time_stamp) != 0 || time_stamp != 5678 ||
        kmipkit_response_message_batch_item_count(message, &item_count) != 0 || item_count != 1 ||
        kmipkit_response_message_batch_item_at(message, 0, &item) != 0 || item == NULL ||
        kmipkit_response_batch_item_view_result_status(item, &has_status, &status) != 0 ||
        has_status != 1 || status != 2 ||
        kmipkit_response_batch_item_extension_count(item, &extension_count) != 0 ||
        extension_count != 1 ||
        kmipkit_response_batch_item_extension_at(item, 0, &extension) != 0 || extension == NULL ||
        kmipkit_message_extension_view_ttlv(extension, &extension_ttlv) != 0 ||
        extension_ttlv == NULL) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_message_extension_view_release(extension);
    kmipkit_response_batch_item_release(item);
    kmipkit_response_header_view_release(header);
    kmipkit_response_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}
