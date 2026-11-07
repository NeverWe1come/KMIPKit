#include "kmipkit.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

extern int kmipkit_test_client_batch_item_encoded_operation(
    kmipkit_client_batch_item_t **inout_item,
    uint32_t *out_operation);

static int test_encoded_operation_and_consume(
        kmipkit_client_batch_item_t **item,
        uint32_t expected_operation) {
    uint32_t encoded_operation = UINT32_MAX;
    return item != NULL && *item != NULL &&
        kmipkit_test_client_batch_item_encoded_operation(item, &encoded_operation) == 0 &&
        *item == NULL && encoded_operation == expected_operation;
}

static int response_ttlv_path_integer_matches(
        kmipkit_ttlv_structure_view_t *view,
        const uint32_t *path,
        uint64_t path_length,
        int32_t expected_integer) {
    uint64_t count = 0;
    if (view == NULL || path == NULL || path_length == 0 ||
            kmipkit_ttlv_structure_view_item_count(view, &count) != 0) {
        return 0;
    }
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        kmipkit_ttlv_structure_view_t *nested = NULL;
        uint32_t actual_tag = 0;
        uint8_t item_type = 0;
        int32_t actual_integer = 0;
        int matched = 0;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == NULL ||
                kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
                kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != path[0] ||
                kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL) {
            goto item_cleanup;
        }
        if (path_length == UINT64_C(1)) {
            matched = kmipkit_ttlv_item_view_type(item, &item_type) == 0 &&
                    item_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER &&
                    kmipkit_ttlv_value_view_integer(value, &actual_integer) == 0 &&
                    actual_integer == expected_integer;
        } else if (kmipkit_ttlv_value_view_type(value, &item_type) == 0 &&
                item_type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE &&
                kmipkit_ttlv_value_view_structure(value, &nested) == 0 && nested != NULL) {
            matched = response_ttlv_path_integer_matches(
                    nested, path + 1, path_length - UINT64_C(1), expected_integer);
        }
item_cleanup:
        kmipkit_ttlv_structure_view_release(nested);
        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (matched) return 1;
    }
    return 0;
}

static int query_ttlv_text_matches(
        kmipkit_ttlv_item_view_t *item,
        uint32_t expected_tag,
        const char *expected_text) {
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    uint32_t actual_tag = 0;
    uint8_t item_type = 0;
    uint64_t byte_length = 0;
    size_t expected_length = strlen(expected_text);
    int matches = 0;
    if (item == NULL || expected_text == NULL ||
            kmipkit_ttlv_item_view_tag(item, &tag) != 0 || tag == NULL ||
            kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 || actual_tag != expected_tag ||
            kmipkit_ttlv_item_view_type(item, &item_type) != 0 ||
            item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING ||
            kmipkit_ttlv_item_view_value(item, &value) != 0 || value == NULL ||
            kmipkit_ttlv_value_view_byte_length(value, &byte_length) != 0 ||
            byte_length != (uint64_t)expected_length) {
        goto cleanup;
    }
    matches = 1;
    for (size_t index = 0; index < expected_length; ++index) {
        uint8_t byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, (uint64_t)index, &byte) != 0 ||
                byte != (uint8_t)expected_text[index]) {
            matches = 0;
            break;
        }
    }
cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    return matches;
}

int32_t kmipkit_client_abi_consumer_query_typed_fields(
        kmipkit_client_batch_item_response_t *item) {
    static const uint32_t expected_tags[] = {
        UINT32_C(0x0042005c), UINT32_C(0x00420003), UINT32_C(0x00541234),
        UINT32_C(0x0042005c), UINT32_C(0x00420057), UINT32_C(0x00420088),
        UINT32_C(0x00420003), UINT32_C(0x0042009d), UINT32_C(0x0042015f)
    };
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    int32_t result = 1;
    uint8_t typed_present = 0;
    uint64_t typed_count = 0;
    if (kmipkit_client_batch_item_response_query_field_count(
                item, &typed_present, &typed_count) != 0 ||
            typed_present != 1 || typed_count != sizeof(expected_tags) / sizeof(expected_tags[0])) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < typed_count; ++index) {
        uint8_t field_present = 0;
        uint32_t field_tag = 0;
        kmipkit_query_response_field_kind_t field_kind = 0;
        uint32_t enumeration = 0;
        kmipkit_ttlv_item_t *field = NULL;
        kmipkit_query_response_field_kind_t expected_kind =
                (index == 0 || index == 3 || index == 4) ?
                KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION :
                ((index == 1 || index == 6 || index == 7) ?
                KMIPKIT_QUERY_RESPONSE_FIELD_KIND_TEXT_STRING :
                ((index == 5 || index == 8) ?
                KMIPKIT_QUERY_RESPONSE_FIELD_KIND_STRUCTURE :
                KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN));
        if (kmipkit_client_batch_item_response_query_field_at(
                    item, index, &field_present, &field_tag, &field_kind,
                    &enumeration, &field) != 0 || field_present != 1 ||
                field_tag != expected_tags[index] || field_kind != expected_kind ||
                (expected_kind == KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION && field != NULL) ||
                (expected_kind != KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION && field == NULL)) {
            kmipkit_ttlv_item_release(field);
            goto cleanup;
        }
        if (expected_kind == KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION) {
            const uint32_t expected_value = index == 0 ? UINT32_C(0x3b) :
                    (index == 3 ? UINT32_C(0xf1234567) : UINT32_C(7));
            if (enumeration != expected_value) goto cleanup;
        } else {
            kmipkit_ttlv_item_view_t *field_view = NULL;
            uint8_t item_type = 0;
            int field_ok = kmipkit_ttlv_item_view_from_item(field, &field_view) == 0 &&
                    field_view != NULL;
            if (field_ok && (expected_kind == KMIPKIT_QUERY_RESPONSE_FIELD_KIND_TEXT_STRING)) {
                const char *expected_text = index == 1 ? "namespace-a" :
                        (index == 2 ? "vendor-extension" :
                        (index == 6 ? "namespace-b" : "vendor-a"));
                field_ok = query_ttlv_text_matches(field_view, field_tag, expected_text);
            } else if (field_ok && expected_kind == KMIPKIT_QUERY_RESPONSE_FIELD_KIND_STRUCTURE) {
                field_ok = kmipkit_ttlv_item_view_type(field_view, &item_type) == 0 &&
                        item_type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE;
            } else if (field_ok && expected_kind == KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN) {
                field_ok = query_ttlv_text_matches(
                        field_view, UINT32_C(0x00541234), "vendor-extension");
            }
            kmipkit_ttlv_item_view_release(field_view);
            kmipkit_ttlv_item_release(field);
            if (!field_ok) goto cleanup;
            continue;
        }
    }
    if (item == NULL || kmipkit_client_batch_item_response_query_ttlv(
                item, &present, &payload) != 0 || present != 1 || payload == NULL ||
            kmipkit_ttlv_structure_view(payload, &view) != 0 || view == NULL ||
            kmipkit_ttlv_structure_view_item_count(view, &count) != 0 ||
            count != sizeof(expected_tags) / sizeof(expected_tags[0])) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *field = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t actual_tag = 0;
        uint8_t item_type = 0;
        uint32_t enumeration = 0;
        int field_ok = 0;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &field) != 0 || field == NULL ||
                kmipkit_ttlv_item_view_tag(field, &tag) != 0 || tag == NULL ||
                kmipkit_ttlv_tag_value(tag, &actual_tag) != 0 ||
                actual_tag != expected_tags[index] ||
                kmipkit_ttlv_item_view_type(field, &item_type) != 0 ||
                kmipkit_ttlv_item_view_value(field, &value) != 0 || value == NULL) {
            goto field_cleanup;
        }
        if (index == 0 || index == 3 || index == 4) {
            const uint32_t expected = index == 0 ? UINT32_C(0x3b) :
                    (index == 3 ? UINT32_C(0xf1234567) : UINT32_C(7));
            field_ok = item_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION &&
                    kmipkit_ttlv_value_view_enumeration(value, &enumeration) == 0 &&
                    enumeration == expected;
        } else if (index == 1) {
            field_ok = query_ttlv_text_matches(field, expected_tags[index], "namespace-a");
        } else if (index == 2) {
            field_ok = query_ttlv_text_matches(field, expected_tags[index], "vendor-extension");
        } else if (index == 5) {
            kmipkit_ttlv_structure_view_t *nested = NULL;
            kmipkit_ttlv_item_view_t *nested_field = NULL;
            uint64_t nested_count = 0;
            if (item_type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE &&
                    kmipkit_ttlv_value_view_structure(value, &nested) == 0 && nested != NULL &&
                    kmipkit_ttlv_structure_view_item_count(nested, &nested_count) == 0 &&
                    nested_count == 1 && kmipkit_ttlv_structure_view_item_at(
                            nested, UINT64_C(0), &nested_field) == 0 && nested_field != NULL) {
                field_ok = query_ttlv_text_matches(
                        nested_field, UINT32_C(0x00540001), "server-extension");
            }
            kmipkit_ttlv_item_view_release(nested_field);
            kmipkit_ttlv_structure_view_release(nested);
        } else if (index == 6) {
            field_ok = query_ttlv_text_matches(field, expected_tags[index], "namespace-b");
        } else if (index == 7) {
            field_ok = query_ttlv_text_matches(field, expected_tags[index], "vendor-a");
        } else if (index == 8) {
            uint64_t nested_count = UINT64_MAX;
            kmipkit_ttlv_structure_view_t *nested = NULL;
            field_ok = item_type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE &&
                    kmipkit_ttlv_value_view_structure(value, &nested) == 0 && nested != NULL &&
                    kmipkit_ttlv_structure_view_item_count(nested, &nested_count) == 0 &&
                    nested_count == 0;
            kmipkit_ttlv_structure_view_release(nested);
        }
field_cleanup:
        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(field);
        if (!field_ok) goto cleanup;
    }
    result = 0;
cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(payload);
    return result;
}

int32_t kmipkit_client_abi_consumer_response_ttlv_handles(
        kmipkit_client_batch_item_response_t *item,
        uint8_t expected_payload_present,
        uint64_t expected_payload_item_count,
        const uint32_t *expected_integer_path,
        uint64_t expected_integer_path_length,
        int32_t expected_integer) {
    uint8_t payload_present = 0;
    uint8_t header_present = 0;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_ttlv_structure_t *header = NULL;
    kmipkit_ttlv_structure_view_t *payload_view = NULL;
    kmipkit_ttlv_structure_view_t *header_view = NULL;
    uint64_t payload_item_count = UINT64_MAX;
    uint64_t header_item_count = 0;
    int32_t result = 1;

    if (item == NULL || expected_payload_present > 1 ||
            (expected_integer_path_length != 0 && expected_integer_path == NULL) ||
            kmipkit_client_batch_item_response_raw_ttlv(
                    item, &payload_present, &payload) != 0 ||
            kmipkit_client_batch_item_response_header_ttlv(
                    item, &header_present, &header) != 0 ||
            payload_present != expected_payload_present ||
            header_present != 1 || header == NULL ||
            ((payload_present == 1) != (payload != NULL))) {
        goto cleanup;
    }

    /* The caller transferred this result handle; the returned structures must outlive it. */
    kmipkit_client_batch_item_response_release(item);
    item = NULL;

    if (payload != NULL) {
        if (kmipkit_ttlv_structure_view(payload, &payload_view) != 0 ||
                kmipkit_ttlv_structure_view_item_count(
                        payload_view, &payload_item_count) != 0 ||
                payload_item_count != expected_payload_item_count) {
            goto cleanup;
        }
        if (expected_integer_path_length != 0 &&
                !response_ttlv_path_integer_matches(
                        payload_view, expected_integer_path,
                        expected_integer_path_length, expected_integer)) {
            goto cleanup;
        }
    } else if (expected_payload_item_count != 0) {
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_view(header, &header_view) != 0 ||
            kmipkit_ttlv_structure_view_item_count(header_view, &header_item_count) != 0 ||
            header_item_count == 0) {
        goto cleanup;
    }
    result = 0;

cleanup:
    if (item != NULL) kmipkit_client_batch_item_response_release(item);
    if (payload_view != NULL) kmipkit_ttlv_structure_view_release(payload_view);
    if (header_view != NULL) kmipkit_ttlv_structure_view_release(header_view);
    if (payload != NULL) kmipkit_ttlv_structure_release(payload);
    if (header != NULL) kmipkit_ttlv_structure_release(header);
    return result;
}

#define T076_ERROR_INVALID_INPUT ((int32_t)4)

int kmipkit_client_abi_c_consumer_batch_item_readback(void) {
    static const uint8_t item_id[] = {0x00, 0xff, 0x41};
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_t *second_batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_item_t *identified_item = 0;
    kmipkit_client_batch_item_t *indexed_item = 0;
    kmipkit_client_batch_item_t *out_of_range_item =
        (kmipkit_client_batch_item_t *)(uintptr_t)1;
    kmipkit_byte_span_v1_t item_id_span = {0};
    uint8_t item_id_present = 0;
    uint64_t item_count = UINT64_MAX;
    int result = 0;

    if (kmipkit_client_batch_create(&batch) != 0 ||
        kmipkit_client_batch_create(&second_batch) != 0 ||
        kmipkit_client_batch_item_ping(&item) != 0 ||
        kmipkit_client_batch_item_with_unique_batch_item_id(
            item, item_id, sizeof(item_id), &identified_item) != 0 || identified_item == 0 ||
        (item = identified_item) == 0 ||
        kmipkit_client_batch_add_item(batch, &item) != 0 || item != 0 ||
        kmipkit_client_batch_item_count(batch, &item_count) != 0 || item_count != 1 ||
        kmipkit_client_batch_item_at(batch, 0, &indexed_item) != 0 || indexed_item == 0 ||
        kmipkit_client_batch_item_at(batch, 1, &out_of_range_item) != T076_ERROR_INVALID_INPUT ||
        out_of_range_item != 0) {
        goto cleanup;
    }
    kmipkit_client_batch_release(batch);
    batch = 0;
    if (kmipkit_client_batch_item_unique_batch_item_id(
            indexed_item, &item_id_present, &item_id_span) != 0 ||
        item_id_present != 1 || item_id_span.length != sizeof(item_id) ||
        item_id_span.data == 0 || memcmp(item_id_span.data, item_id, sizeof(item_id)) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_add_item(second_batch, &indexed_item) != T076_ERROR_INVALID_INPUT ||
        indexed_item == 0) {
        goto cleanup;
    }
    item_id_present = 0;
    item_id_span.data = 0;
    item_id_span.length = 0;
    if (kmipkit_client_batch_item_unique_batch_item_id(
            indexed_item, &item_id_present, &item_id_span) != 0 ||
        item_id_present != 1 || item_id_span.length != sizeof(item_id) ||
        item_id_span.data == 0 || memcmp(item_id_span.data, item_id, sizeof(item_id)) != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_client_batch_item_release(indexed_item);
    kmipkit_client_batch_item_release(out_of_range_item);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_release(second_batch);
    return result;
}

static int check_view_tags(
    kmipkit_ttlv_structure_view_t *view, const uint32_t *expected, uint64_t expected_count);

static int execute_with_options_and_error_ttlv_clears_outputs(void) {
    kmipkit_client_batch_response_t *response = (kmipkit_client_batch_response_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *error_response = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t status = kmipkit_client_execute_with_options_and_error_ttlv_v1(
        0, 0, 0, 0, &response, &error_response);
    return status == T076_ERROR_INVALID_INPUT && response == 0 && error_response == 0;
}

static int async_operation_options_symbols_clear_outputs(void) {
    kmipkit_request_options_v1_t options = {0};
    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    kmipkit_client_async_operation_outcome_t *outcome =
        (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;

    if (kmipkit_client_execute_poll_with_options_v1(
            0, 0, 0, 0, &options, &outcome) != T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_cancel_with_options_v1(
            0, 0, 0, 0, &options, &outcome) != T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_process_with_options_v1(
            0, 0, 0, 0, 0, 0, &options, &outcome) !=
            T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_query_async_requests_with_options_v1(
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, &options, &outcome) !=
            T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    return 1;
}

static int execute_with_error_ttlv_clears_outputs(void) {
    kmipkit_client_batch_response_t *response = (kmipkit_client_batch_response_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *error_response = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t status = kmipkit_client_execute_with_error_ttlv_v1(
        0, 0, 0, &response, &error_response);
    return status == T076_ERROR_INVALID_INPUT && response == 0 && error_response == 0;
}

static int execute_with_profiles_rejects_invalid_selection_from_c(void) {
    kmipkit_profile_validation_selection_v1_t selection = {0};
    kmipkit_client_batch_response_t *response = 0;
    const int32_t status = kmipkit_client_execute_with_profiles(
        0, 0, 0, &selection, &response);
    return status == T076_ERROR_INVALID_INPUT && response == 0;
}

static int absent_profile_details_preserve_non_profile_error_from_c(void) {
    kmipkit_ttlv_structure_t *invalid_response = 0;
    kmipkit_response_message_t *response = 0;
    kmipkit_message_validation_error_t *validation_error = 0;
    kmipkit_profile_validation_error_t *profile_error = 0;
    int32_t status = 0;
    int32_t error_status = 0;
    uint32_t error_category = 0;
    uint32_t error_cause = 0;
    uint32_t error_delivery = 0;
    int result = 0;

    if (kmipkit_last_error_clear() != 0 ||
        kmipkit_ttlv_structure_create(&invalid_response) != 0) {
        goto cleanup;
    }
    status = kmipkit_response_message_from_ttlv(
        invalid_response, &response, &validation_error);
    invalid_response = 0; /* consumed whether response validation succeeds or fails */
    if (status != KMIPKIT_ERROR_PROTOCOL || response != 0 || validation_error == 0) {
        goto cleanup;
    }
    if (kmipkit_last_error_profile_validation_error_get_v1(&profile_error) != 0 ||
        profile_error != 0) {
        goto cleanup;
    }
    if (kmipkit_last_error_get(
            &error_status, &error_category, &error_cause, &error_delivery) != 0) {
        goto cleanup;
    }
    result = error_status == status && error_category == 2 && error_cause == 4 &&
        error_delivery == 0;

cleanup:
    kmipkit_ttlv_structure_release(invalid_response);
    kmipkit_response_message_release(response);
    kmipkit_message_validation_error_release(validation_error);
    kmipkit_profile_validation_error_release(profile_error);
    return result;
}

static int baseline_profile_fails_closed_before_c_transport_write(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t baseline_profile[] = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    static const uint64_t profile_id_offsets[] = {0};
    static const uint64_t profile_id_lengths[] = {sizeof(baseline_profile) - 1};
    kmipkit_profile_validation_selection_v1_t selection = {0};
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_response_t *response =
        (kmipkit_client_batch_response_t *)(uintptr_t)1;
    int32_t status = 0;
    int32_t error_status = 0;
    uint32_t error_category = 0;
    uint32_t error_cause = 0;
    uint32_t error_delivery = 0;
    kmipkit_profile_validation_error_t *profile_error = 0;
    kmipkit_profile_validation_error_view_v1_t profile_view = {0};
    int result = 0;

    selection.struct_size = (uint32_t)sizeof(selection);
    selection.abi_version = 1;
    selection.profile_id_data = baseline_profile;
    selection.profile_id_data_length = sizeof(baseline_profile) - 1;
    selection.profile_id_offsets = profile_id_offsets;
    selection.profile_id_lengths = profile_id_lengths;
    selection.profile_id_count = 1;
    if (kmipkit_client_batch_create(&batch) != 0 ||
        kmipkit_client_batch_item_discover_versions(&item) != 0 ||
        kmipkit_client_batch_add_item(batch, &item) != 0 ||
        kmipkit_last_error_clear() != 0) {
        goto cleanup;
    }
    status = kmipkit_client_execute_with_profiles(
        client, &batch, limits, &selection, &response);
    if (status == 0 || response != 0 || batch != 0 ||
        kmipkit_last_error_get(
            &error_status, &error_category, &error_cause, &error_delivery) != 0) {
        goto cleanup;
    }
    if (kmipkit_last_error_profile_validation_error_get_v1(&profile_error) != 0 ||
        profile_error == 0 || kmipkit_last_error_clear() != 0) {
        goto cleanup;
    }
    profile_view.struct_size = (uint32_t)sizeof(profile_view);
    profile_view.abi_version = 1;
    if (kmipkit_profile_validation_error_view_get_v1(profile_error, &profile_view) != 0) {
        goto cleanup;
    }
    result = error_status == status && error_category == 1 && error_delivery == 1 &&
        profile_view.source_kind == 1 && profile_view.batch_item_index_present == 0 &&
        profile_view.error_code_length == sizeof("profile_unavailable") - 1 &&
        memcmp(profile_view.error_code_data, "profile_unavailable",
               profile_view.error_code_length) == 0 &&
        profile_view.profile_id_length == sizeof(baseline_profile) - 1 &&
        memcmp(profile_view.profile_id_data, baseline_profile,
               profile_view.profile_id_length) == 0 &&
        profile_view.requirement_id_length == sizeof("KMIPKIT-REQ-SPEC-14.1-001") - 1 &&
        memcmp(profile_view.requirement_id_data, "KMIPKIT-REQ-SPEC-14.1-001",
               profile_view.requirement_id_length) == 0 &&
        profile_view.source_id_length == sizeof("KMIPKIT-CLAUSE-SPEC-14.1-001") - 1 &&
        memcmp(profile_view.source_id_data, "KMIPKIT-CLAUSE-SPEC-14.1-001",
               profile_view.source_id_length) == 0 &&
        profile_view.source_locator_length == 0;

cleanup:
    kmipkit_profile_validation_error_release(profile_error);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(item);
    return result;
}

static int unknown_profile_fails_closed_before_c_transport_write(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t unknown_profile[] = "unpublished-profile-selector-sentinel";
    static const uint64_t profile_id_offsets[] = {0};
    static const uint64_t profile_id_lengths[] = {sizeof(unknown_profile) - 1};
    kmipkit_profile_validation_selection_v1_t selection = {0};
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_response_t *response =
        (kmipkit_client_batch_response_t *)(uintptr_t)1;
    int32_t status = 0;
    int32_t error_status = 0;
    uint32_t error_category = 0;
    uint32_t error_cause = 0;
    uint32_t error_delivery = 0;
    kmipkit_profile_validation_error_t *profile_error = 0;
    kmipkit_profile_validation_error_view_v1_t profile_view = {0};
    int result = 0;

    selection.struct_size = (uint32_t)sizeof(selection);
    selection.abi_version = 1;
    selection.profile_id_data = unknown_profile;
    selection.profile_id_data_length = sizeof(unknown_profile) - 1;
    selection.profile_id_offsets = profile_id_offsets;
    selection.profile_id_lengths = profile_id_lengths;
    selection.profile_id_count = 1;
    if (kmipkit_client_batch_create(&batch) != 0 ||
        kmipkit_client_batch_item_discover_versions(&item) != 0 ||
        kmipkit_client_batch_add_item(batch, &item) != 0 ||
        kmipkit_last_error_clear() != 0) {
        goto cleanup;
    }
    status = kmipkit_client_execute_with_profiles(
        client, &batch, limits, &selection, &response);
    if (status == 0 || response != 0 || batch != 0 ||
        kmipkit_last_error_get(
            &error_status, &error_category, &error_cause, &error_delivery) != 0) {
        goto cleanup;
    }
    if (kmipkit_last_error_profile_validation_error_get_v1(&profile_error) != 0 ||
        profile_error == 0 || kmipkit_last_error_clear() != 0) {
        goto cleanup;
    }
    profile_view.struct_size = (uint32_t)sizeof(profile_view);
    profile_view.abi_version = 1;
    if (kmipkit_profile_validation_error_view_get_v1(profile_error, &profile_view) != 0) {
        goto cleanup;
    }
    result = error_status == status && error_category == 1 && error_delivery == 1 &&
        profile_view.source_kind == 0 && profile_view.batch_item_index_present == 0 &&
        profile_view.error_code_length == sizeof("unknown_profile") - 1 &&
        memcmp(profile_view.error_code_data, "unknown_profile",
               profile_view.error_code_length) == 0 &&
        profile_view.profile_id_data == 0 && profile_view.profile_id_length == 0 &&
        profile_view.requirement_id_data == 0 && profile_view.requirement_id_length == 0 &&
        profile_view.source_id_data == 0 && profile_view.source_id_length == 0 &&
        profile_view.source_locator_data == 0 && profile_view.source_locator_length == 0;

cleanup:
    kmipkit_profile_validation_error_release(profile_error);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(item);
    return result;
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

static int generic_big_integer_constructor_preserves_type_and_octets_from_c(void) {
    static const uint8_t value_bytes[] = {0x00, 0x80, 0xff};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_value_view_t *view = 0;
    uint8_t actual_type = 0;
    uint64_t actual_length = 0;
    int result = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_value_big_integer(
            limits, value_bytes, (uint64_t)sizeof(value_bytes), &value) != 0 || value == 0 ||
        kmipkit_ttlv_value_view(value, &view) != 0 || view == 0 ||
        kmipkit_ttlv_value_view_type(view, &actual_type) != 0 ||
        actual_type != KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER ||
        kmipkit_ttlv_value_view_byte_length(view, &actual_length) != 0 ||
        actual_length != (uint64_t)sizeof(value_bytes)) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < actual_length; ++index) {
        uint8_t actual_byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(view, index, &actual_byte) != 0 ||
            actual_byte != value_bytes[index]) {
            goto cleanup;
        }
    }
    result = 1;
cleanup:
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int recertify_constructor_is_callable_from_c(void) {
    static const uint8_t existing_id_bytes[] = "certificate-7";
    static const uint8_t request_id_bytes[] = "request-7";
    static const uint8_t request_value[] = {0x30, 0x82, 0x01, 0x00};
    kmipkit_unique_identifier_v1_t existing_id = {0};
    kmipkit_unique_identifier_v1_t request_id = {0};
    kmipkit_byte_span_v1_t request_span = {
        request_value,
        (uint64_t)sizeof(request_value),
    };
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    existing_id.struct_size = (uint32_t)sizeof(existing_id);
    existing_id.abi_version = 1;
    existing_id.kind = 1;
    existing_id.text_data = existing_id_bytes;
    existing_id.text_length = (uint64_t)(sizeof(existing_id_bytes) - 1);
    request_id.struct_size = (uint32_t)sizeof(request_id);
    request_id.abi_version = 1;
    request_id.kind = 1;
    request_id.text_data = request_id_bytes;
    request_id.text_length = (uint64_t)(sizeof(request_id_bytes) - 1);

    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_recertify(
            &existing_id, &request_id, 2, 0, 0, 0, 0, attributes, masks,
            &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        kmipkit_client_batch_item_release(item);
        item = 0;
        goto cleanup;
    }
    if (kmipkit_client_batch_item_recertify(
            &existing_id, &request_id, 0, 0, 0, 2, 0, attributes, masks,
            &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        kmipkit_client_batch_item_release(item);
        item = 0;
        goto cleanup;
    }
    if (kmipkit_client_batch_item_recertify(
            0, 0, 0, 0, 0, 0, 0, attributes, masks, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_recertify(
            &existing_id, &request_id, 1, UINT32_C(0xfedcba98), &request_span,
            1, UINT32_C(0xffffffff), attributes, masks, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

static int recertify_response_accessors_clear_outputs_from_c(void) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    uint8_t present = 9;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)1;
    const int32_t identifier_status = kmipkit_client_batch_item_response_recertify(
        0, &present, &identifier);
    if (identifier_status != T076_ERROR_INVALID_INPUT || present != 0 ||
        identifier.kind != 0 || identifier.text_data != 0 ||
        identifier.text_length != 0 || identifier.enumeration_value != 0 ||
        identifier.integer_value != 0) {
        return 0;
    }
    present = 9;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_recertify_ttlv(
        0, &present, &response_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
}

static int rekey_constructor_is_callable_from_c(void) {
    static const uint8_t identifier_bytes[] = "existing-key";
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = 1;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);

    if (kmipkit_client_batch_item_rekey(
            0, 0, 0, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_rekey(
            &identifier, 1, UINT32_MAX, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    return result;
}

static int reprovision_constructor_is_callable_from_c(void) {
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_reprovision(0, 0, &item) != 0 || item == 0) {
        if (item != 0) kmipkit_client_batch_item_release(item);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    kmipkit_ttlv_structure_t *certificate_request = 0;
    kmipkit_ttlv_structure_t *certificate = 0;
    if (kmipkit_ttlv_structure_create(&certificate_request) != 0 ||
        kmipkit_ttlv_structure_create(&certificate) != 0 ||
        certificate_request == 0 || certificate == 0) {
        if (certificate_request != 0) kmipkit_ttlv_structure_release(certificate_request);
        if (certificate != 0) kmipkit_ttlv_structure_release(certificate);
        return 0;
    }
    if (kmipkit_client_batch_item_reprovision(certificate_request, 0, &item) != 0 ||
        item == 0) {
        if (item != 0) kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(certificate_request);
        kmipkit_ttlv_structure_release(certificate);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_reprovision(0, certificate, &item) != 0 ||
        item == 0) {
        if (item != 0) kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(certificate_request);
        kmipkit_ttlv_structure_release(certificate);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    int rejected_both = kmipkit_client_batch_item_reprovision(
        certificate_request, certificate, &item) != 0 && item == 0;
    if (item != 0) kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(certificate_request);
    kmipkit_ttlv_structure_release(certificate);
    return rejected_both;
}

static int rng_request_builders_are_callable_from_c(void) {
    static const uint8_t seed[] = {0x00, 0x81, 0xff};
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_byte_span_v1_t seed_span = {seed, (uint64_t)sizeof(seed)};
    if (kmipkit_client_batch_item_rng_retrieve(-17, &item) != 0 || item == 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_rng_seed(&seed_span, &item) != 0 || item == 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    seed_span.data = 0;
    seed_span.length = 0;
    if (kmipkit_client_batch_item_rng_seed(&seed_span, &item) != 0 || item == 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_rng_seed(0, &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        kmipkit_client_batch_item_release(item);
        return 0;
    }
    return 1;
}

int kmipkit_client_abi_c_consumer_validate_rng_retrieve_response(
    kmipkit_client_batch_item_response_t *item,
    const uint8_t *expected_data,
    uint64_t expected_length) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {UINT32_C(0x4200c2)};
    uint8_t present = 0;
    uint8_t data_present = 0;
    uint8_t ttlv_present = 0;
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_ttlv_item_view_t *data_item = 0;
    kmipkit_ttlv_value_view_t *data_value = 0;
    uint8_t data_item_type = 0;
    uint64_t ttlv_length = 0;
    int result = 0;

    if (item == 0 || (expected_data == 0 && expected_length != 0) ||
        kmipkit_client_batch_item_response_rng_retrieve(
            item, &present, &data_present, &data) != 0 ||
        present != 1 || data_present != 1 || data.length != expected_length ||
        (expected_length != 0 && (data.data == 0 ||
            memcmp(data.data, expected_data, (size_t)expected_length) != 0)) ||
        kmipkit_client_batch_item_response_rng_retrieve_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 3, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(payload_view, 0, &data_item) != 0 ||
        kmipkit_ttlv_item_view_type(data_item, &data_item_type) != 0 ||
        data_item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
        kmipkit_ttlv_item_view_value(data_item, &data_value) != 0 ||
        kmipkit_ttlv_value_view_byte_length(data_value, &ttlv_length) != 0 ||
        ttlv_length != expected_length) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    for (uint64_t index = 0; index < ttlv_length; ++index) {
        uint8_t value = 0;
        if (kmipkit_ttlv_value_view_byte_at(data_value, index, &value) != 0 ||
            value != expected_data[index]) {
            goto cleanup;
        }
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_value_view_release(data_value);
    kmipkit_ttlv_item_view_release(data_item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_rng_seed_response(
    kmipkit_client_batch_item_response_t *item,
    int32_t expected_data_length) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {UINT32_C(0x4200c4)};
    uint8_t present = 0;
    uint8_t data_length_present = 0;
    uint8_t ttlv_present = 0;
    int32_t data_length = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_ttlv_item_view_t *length_item = 0;
    kmipkit_ttlv_value_view_t *length_value = 0;
    uint8_t item_type = 0;
    int32_t ttlv_data_length = 0;
    int result = 0;

    if (item == 0 || kmipkit_client_batch_item_response_rng_seed(
            item, &present, &data_length_present, &data_length) != 0 ||
        present != 1 || data_length_present != 1 ||
        data_length != expected_data_length ||
        kmipkit_client_batch_item_response_rng_seed_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 3, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(payload_view, 0, &length_item) != 0 ||
        kmipkit_ttlv_item_view_type(length_item, &item_type) != 0 ||
        item_type != KMIPKIT_TTLV_ITEM_TYPE_INTEGER ||
        kmipkit_ttlv_item_view_value(length_item, &length_value) != 0 ||
        kmipkit_ttlv_value_view_integer(length_value, &ttlv_data_length) != 0 ||
        ttlv_data_length != expected_data_length) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_value_view_release(length_value);
    kmipkit_ttlv_item_view_release(length_item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_rejects_rng_output_aliases(
    kmipkit_client_batch_item_response_t *item) {
    uint8_t shared_flag = 9;
    uint8_t distinct_flag = 7;
    kmipkit_byte_span_v1_t span = {(const uint8_t *)(uintptr_t)0x1234, 0x5678};
    union {
        uint8_t flag;
        kmipkit_byte_span_v1_t span;
    } retrieve_span_alias;
    union {
        uint8_t flag;
        kmipkit_ttlv_structure_t *handle;
    } retrieve_ttlv_alias;
    union {
        uint8_t flag;
        int32_t length;
    } seed_length_alias;
    union {
        uint8_t flag;
        kmipkit_ttlv_structure_t *handle;
    } seed_ttlv_alias;
    unsigned char before[sizeof(retrieve_span_alias)];

    if (item == 0) return 0;
    if (kmipkit_client_batch_item_response_rng_retrieve(
            item, &shared_flag, &shared_flag, &span) != T076_ERROR_INVALID_INPUT ||
        shared_flag != 9) return 0;

    memset(&retrieve_span_alias, 0xa5, sizeof(retrieve_span_alias));
    memcpy(before, &retrieve_span_alias, sizeof(retrieve_span_alias));
    if (kmipkit_client_batch_item_response_rng_retrieve(
            item, &retrieve_span_alias.flag, &distinct_flag,
            &retrieve_span_alias.span) != T076_ERROR_INVALID_INPUT ||
        distinct_flag != 7 ||
        memcmp(before, &retrieve_span_alias, sizeof(retrieve_span_alias)) != 0) return 0;

    shared_flag = 9;
    distinct_flag = 7;
    memset(&retrieve_span_alias, 0x5a, sizeof(retrieve_span_alias));
    memcpy(before, &retrieve_span_alias, sizeof(retrieve_span_alias));
    if (kmipkit_client_batch_item_response_rng_retrieve(
            item, &shared_flag, &retrieve_span_alias.flag,
            &retrieve_span_alias.span) != T076_ERROR_INVALID_INPUT ||
        shared_flag != 9 ||
        memcmp(before, &retrieve_span_alias, sizeof(retrieve_span_alias)) != 0) return 0;

    retrieve_ttlv_alias.handle = (kmipkit_ttlv_structure_t *)(uintptr_t)0x1234;
    memcpy(before, &retrieve_ttlv_alias, sizeof(retrieve_ttlv_alias));
    if (kmipkit_client_batch_item_response_rng_retrieve_ttlv(
            item, &retrieve_ttlv_alias.flag, &retrieve_ttlv_alias.handle) !=
                T076_ERROR_INVALID_INPUT ||
        memcmp(before, &retrieve_ttlv_alias, sizeof(retrieve_ttlv_alias)) != 0) return 0;

    shared_flag = 9;
    distinct_flag = 7;
    seed_length_alias.length = INT32_C(0x12345678);
    memcpy(before, &seed_length_alias, sizeof(seed_length_alias));
    if (kmipkit_client_batch_item_response_rng_seed(
            item, &shared_flag, &shared_flag, &seed_length_alias.length) !=
                T076_ERROR_INVALID_INPUT ||
        shared_flag != 9) return 0;
    if (kmipkit_client_batch_item_response_rng_seed(
            item, &seed_length_alias.flag, &distinct_flag,
            &seed_length_alias.length) != T076_ERROR_INVALID_INPUT ||
        distinct_flag != 7 ||
        memcmp(before, &seed_length_alias, sizeof(seed_length_alias)) != 0) return 0;

    shared_flag = 9;
    distinct_flag = 7;
    seed_length_alias.length = INT32_C(0x23456789);
    memcpy(before, &seed_length_alias, sizeof(seed_length_alias));
    if (kmipkit_client_batch_item_response_rng_seed(
            item, &shared_flag, &seed_length_alias.flag,
            &seed_length_alias.length) != T076_ERROR_INVALID_INPUT ||
        shared_flag != 9 ||
        memcmp(before, &seed_length_alias, sizeof(seed_length_alias)) != 0) return 0;

    seed_ttlv_alias.handle = (kmipkit_ttlv_structure_t *)(uintptr_t)0x5678;
    memcpy(before, &seed_ttlv_alias, sizeof(seed_ttlv_alias));
    if (kmipkit_client_batch_item_response_rng_seed_ttlv(
            item, &seed_ttlv_alias.flag, &seed_ttlv_alias.handle) !=
                T076_ERROR_INVALID_INPUT ||
        memcmp(before, &seed_ttlv_alias, sizeof(seed_ttlv_alias)) != 0) return 0;
    return 1;
}

int kmipkit_client_abi_c_consumer_validates_absent_rng_success_fields(
    kmipkit_client_batch_item_response_t *item) {
    uint8_t present = 9;
    uint8_t field_present = 9;
    int32_t data_length = 9;
    kmipkit_byte_span_v1_t data = {(const uint8_t *)(uintptr_t)0x1234, 0x5678};
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)0x1234;

    if (item == 0 ||
        kmipkit_client_batch_item_response_rng_retrieve(
            item, &present, &field_present, &data) != 0 ||
        present != 0 || field_present != 0 || data.data != 0 || data.length != 0) return 0;
    present = 9;
    if (kmipkit_client_batch_item_response_rng_retrieve_ttlv(
            item, &present, &response_ttlv) != 0 || present != 0 || response_ttlv != 0) return 0;

    present = 9;
    field_present = 9;
    data_length = 9;
    if (kmipkit_client_batch_item_response_rng_seed(
            item, &present, &field_present, &data_length) != 0 ||
        present != 0 || field_present != 0 || data_length != 0) return 0;
    present = 9;
    response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)0x1234;
    if (kmipkit_client_batch_item_response_rng_seed_ttlv(
            item, &present, &response_ttlv) != 0 || present != 0 || response_ttlv != 0) return 0;
    return 1;
}

int kmipkit_client_abi_c_consumer_validates_pending_rng_response(
    kmipkit_client_batch_item_response_t *item,
    uint32_t operation) {
    uint8_t present = 0;
    uint8_t field_present = 0;
    uint8_t ttlv_present = 0;
    int32_t data_length = 0;
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    int status = 0;

    if (item == 0) return 0;
    if (operation == UINT32_C(0x25)) {
        status = kmipkit_client_batch_item_response_rng_retrieve(
            item, &present, &field_present, &data);
        if (status != 0 || present != 1 || field_present != 0 ||
            data.data != 0 || data.length != 0) return 0;
        status = kmipkit_client_batch_item_response_rng_retrieve_ttlv(
            item, &ttlv_present, &response_ttlv);
    } else if (operation == UINT32_C(0x26)) {
        status = kmipkit_client_batch_item_response_rng_seed(
            item, &present, &field_present, &data_length);
        if (status != 0 || present != 1 || field_present != 0 || data_length != 0) return 0;
        status = kmipkit_client_batch_item_response_rng_seed_ttlv(
            item, &ttlv_present, &response_ttlv);
    } else {
        return 0;
    }

    if (status != 0 || ttlv_present != 1 || response_ttlv == 0) {
        kmipkit_ttlv_structure_release(response_ttlv);
        return 0;
    }
    kmipkit_ttlv_structure_release(response_ttlv);
    return 1;
}

static int rekey_key_pair_constructor_is_callable_from_c(void) {
    static const uint8_t identifier_bytes[] = "existing-private-key";
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *structures[6] = {0};
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = 1;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);
    if (kmipkit_client_batch_item_rekey_key_pair(
            0, 0, 0, 0, 0, 0, 0, 0, 0, &item) != 0 || item == 0) goto cleanup;
    kmipkit_client_batch_item_t *sentinel_item = item;
    const int32_t invalid_offset_status = kmipkit_client_batch_item_rekey_key_pair(
            0, 2, UINT32_C(86400), 0, 0, 0, 0, 0, 0, &item);
    kmipkit_client_batch_item_t *invalid_offset_output = item;
    item = sentinel_item;
    if (invalid_offset_output != 0 && invalid_offset_output != sentinel_item) {
        kmipkit_client_batch_item_release(invalid_offset_output);
    }
    if (invalid_offset_status != T076_ERROR_INVALID_INPUT || invalid_offset_output != 0) goto cleanup;
    kmipkit_client_batch_item_release(item);
    item = 0;
    for (size_t index = 0; index < 6; ++index) {
        if (kmipkit_ttlv_structure_create(&structures[index]) != 0 || structures[index] == 0) goto cleanup;
    }
    if (kmipkit_client_batch_item_rekey_key_pair(
            &identifier, 1, UINT32_MAX,
            structures[0], structures[1], structures[2],
            structures[3], structures[4], structures[5], &item) != 0 || item == 0) goto cleanup;
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    for (size_t index = 0; index < 6; ++index) kmipkit_ttlv_structure_release(structures[index]);
    return result;
}

static int rekey_key_pair_response_rejects_invalid_output_records_from_c(void) {
    static const uint8_t private_sentinel[] = "private-output-sentinel";
    static const uint8_t public_sentinel[] = "public-output-sentinel";
    kmipkit_unique_identifier_v1_t private_identifier = {0};
    kmipkit_unique_identifier_v1_t public_identifier = {0};
    uint8_t present = 9;

    private_identifier.struct_size = (uint32_t)sizeof(private_identifier) - 1;
    private_identifier.abi_version = 1;
    private_identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    private_identifier.text_data = private_sentinel;
    private_identifier.text_length = sizeof(private_sentinel) - 1;
    public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
    public_identifier.abi_version = 1;
    public_identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    public_identifier.text_data = public_sentinel;
    public_identifier.text_length = sizeof(public_sentinel) - 1;
    if (kmipkit_client_batch_item_response_rekey_key_pair(
            0, &present, &private_identifier, &public_identifier) != T076_ERROR_INVALID_INPUT ||
        present != 9 || private_identifier.struct_size != sizeof(private_identifier) - 1 ||
        private_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        private_identifier.text_data != private_sentinel ||
        private_identifier.text_length != sizeof(private_sentinel) - 1 ||
        public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        public_identifier.text_data != public_sentinel ||
        public_identifier.text_length != sizeof(public_sentinel) - 1) return 0;

    private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
    private_identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    private_identifier.text_data = private_sentinel;
    private_identifier.text_length = sizeof(private_sentinel) - 1;
    public_identifier.struct_size = (uint32_t)sizeof(public_identifier) - 1;
    if (kmipkit_client_batch_item_response_rekey_key_pair(
            0, &present, &private_identifier, &public_identifier) != T076_ERROR_INVALID_INPUT ||
        present != 9 || private_identifier.kind != 0 || private_identifier.text_data != 0 ||
        private_identifier.text_length != 0 ||
        public_identifier.struct_size != sizeof(public_identifier) - 1 ||
        public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        public_identifier.text_data != public_sentinel ||
        public_identifier.text_length != sizeof(public_sentinel) - 1) return 0;
    return 1;
}

static int rekey_key_pair_response_accessors_clear_outputs_from_c(void) {
    kmipkit_unique_identifier_v1_t private_identifier = {0};
    kmipkit_unique_identifier_v1_t public_identifier = {0};
    uint8_t present = 9;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
    private_identifier.abi_version = 1;
    public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
    public_identifier.abi_version = 1;
    const int32_t status = kmipkit_client_batch_item_response_rekey_key_pair(
            0, &present, &private_identifier, &public_identifier);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || private_identifier.kind != 0 ||
            public_identifier.kind != 0 || private_identifier.text_data != 0 ||
            public_identifier.text_data != 0 || private_identifier.text_length != 0 ||
            public_identifier.text_length != 0) return 0;
    present = 9;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
            0, &present, &response_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
}

static int rekey_response_accessors_clear_outputs_from_c(void) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    uint8_t present = 9;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t identifier_status = kmipkit_client_batch_item_response_rekey(
        0, &present, &identifier);
    if (identifier_status != T076_ERROR_INVALID_INPUT || present != 0 ||
        identifier.kind != 0 || identifier.text_data != 0 || identifier.text_length != 0 ||
        identifier.enumeration_value != 0 || identifier.integer_value != 0) {
        return 0;
    }
    present = 9;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_rekey_ttlv(
        0, &present, &response_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
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
    identifiers[0].kind = 0;
    identifiers[0].text_data = 0;
    identifiers[0].text_length = 0;
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_derive_key(
            2, identifiers, 2, UINT32_C(0xfeedbeef),
            derivation_parameters, attributes, &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    identifiers[0].kind = 1;
    identifiers[0].text_data = source_identifier;
    identifiers[0].text_length = sizeof(source_identifier);
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

static int login_constructor_is_callable_from_c(void) {
    kmipkit_ttlv_structure_t *usage_limits = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&usage_limits) != 0 ||
        kmipkit_client_batch_item_login(0, 1, 0, 0, usage_limits, &item) !=
            T076_ERROR_INVALID_INPUT || item != 0 ||
        kmipkit_client_batch_item_login(2, INT64_MIN, 1, INT32_MIN, usage_limits, &item) != 0 ||
        item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(usage_limits);
    return result;
}

static int append_logout_ticket_field(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **ticket,
    uint32_t raw_tag_value,
    kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *updated = 0;
    int result = 0;

    if (limits == 0 || ticket == 0 || *ticket == 0 || value == 0 || *value == 0) {
        return 0;
    }
    if (kmipkit_ttlv_raw_tag_create(raw_tag_value, &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, *value, limits, &item) != 0) {
        goto cleanup;
    }
    *value = 0;
    if (kmipkit_ttlv_structure_with_item(*ticket, item, limits, &updated) != 0) {
        goto cleanup;
    }
    *ticket = 0;
    item = 0;
    *ticket = updated;
    updated = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    return result;
}

static int logout_constructor_is_callable_from_c(void) {
    static const uint8_t ticket_value[] = {0x01, 0x02, 0x03};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *ticket = 0;
    kmipkit_ttlv_value_t *ticket_type = 0;
    kmipkit_ttlv_value_t *secret_value = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_structure_create(&ticket) != 0 ||
        kmipkit_ttlv_value_enumeration(1, &ticket_type) != 0 ||
        !append_logout_ticket_field(limits, &ticket, UINT32_C(0x42014a), &ticket_type) ||
        kmipkit_ttlv_value_byte_string(
            limits, ticket_value, (uint64_t)sizeof(ticket_value), &secret_value) != 0 ||
        !append_logout_ticket_field(limits, &ticket, UINT32_C(0x42014b), &secret_value)) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_logout(0, &item) != T076_ERROR_INVALID_INPUT || item != 0 ||
        kmipkit_client_batch_item_logout(ticket, &item) != 0 || item == 0 ||
        !test_encoded_operation_and_consume(&item, UINT32_C(0x2e))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_value_release(secret_value);
    kmipkit_ttlv_value_release(ticket_type);
    kmipkit_ttlv_structure_release(ticket);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int logout_response_accessor_clears_outputs_from_c(void) {
    uint8_t present = 9;
    const int32_t status = kmipkit_client_batch_item_response_logout(0, &present);
    return status == T076_ERROR_INVALID_INPUT && present == 0;
}

static int login_response_accessors_clear_outputs_from_c(void) {
    uint8_t present = 9;
    uint8_t ticket_present = 9;
    uint32_t ticket_type = UINT32_MAX;
    kmipkit_byte_span_v1_t ticket_value = {(const uint8_t *)1, UINT64_MAX};
    const int32_t status = kmipkit_client_batch_item_response_login(
        0, &present, &ticket_present, &ticket_type, &ticket_value);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || ticket_present != 0 ||
        ticket_type != 0 || ticket_value.data != 0 || ticket_value.length != 0) {
        return 0;
    }
    present = 9;
    kmipkit_ttlv_structure_t *ticket_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_login_ttlv(
        0, &present, &ticket_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && ticket_ttlv == 0;
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

int kmipkit_client_abi_c_consumer_validate_crypto_response_ttlv(
    kmipkit_client_batch_item_response_t *item, uint32_t operation_index) {
    static const uint32_t hash_tags[] = {UINT32_C(0x4200c2), UINT32_C(0x540001)};
    static const uint32_t mac_tags[] = {
        UINT32_C(0x420094), UINT32_C(0x4200c6), UINT32_C(0x540001)};
    static const uint32_t mac_verify_tags[] = {
        UINT32_C(0x420094), UINT32_C(0x42009b), UINT32_C(0x540001)};
    static const uint32_t sign_tags[] = {
        UINT32_C(0x420094), UINT32_C(0x4200c3), UINT32_C(0x540001)};
    static const uint32_t signature_verify_tags[] = {
        UINT32_C(0x420094), UINT32_C(0x42009b), UINT32_C(0x4200c2),
        UINT32_C(0x540001)};
    typedef int32_t (*response_ttlv_getter)(
        kmipkit_client_batch_item_response_t *, uint8_t *, kmipkit_ttlv_structure_t **);
    static const response_ttlv_getter getters[] = {
        kmipkit_client_batch_item_response_hash_ttlv,
        kmipkit_client_batch_item_response_mac_ttlv,
        kmipkit_client_batch_item_response_mac_verify_ttlv,
        kmipkit_client_batch_item_response_sign_ttlv,
        kmipkit_client_batch_item_response_signature_verify_ttlv,
    };
    static const uint32_t *const expected_tags[] = {
        hash_tags, mac_tags, mac_verify_tags, sign_tags, signature_verify_tags};
    static const uint64_t expected_tag_counts[] = {
        (uint64_t)(sizeof(hash_tags) / sizeof(hash_tags[0])),
        (uint64_t)(sizeof(mac_tags) / sizeof(mac_tags[0])),
        (uint64_t)(sizeof(mac_verify_tags) / sizeof(mac_verify_tags[0])),
        (uint64_t)(sizeof(sign_tags) / sizeof(sign_tags[0])),
        (uint64_t)(sizeof(signature_verify_tags) / sizeof(signature_verify_tags[0])),
    };
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    int result = 0;

    if (item == 0 || operation_index >=
            (uint32_t)(sizeof(getters) / sizeof(getters[0]))) {
        goto cleanup;
    }
    if (getters[operation_index](item, &present, &response_ttlv) != 0 ||
        present != 1 || response_ttlv == 0) {
        goto cleanup;
    }

    /* The C-owned TTLV copy remains readable after its source response is released. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        !check_view_tags(view, response_tags,
                         (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(view, 3, &payload_item) != 0 ||
        payload_item == 0 || kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        payload_value == 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        payload_view == 0 ||
        !check_view_tags(payload_view, expected_tags[operation_index],
                         expected_tag_counts[operation_index])) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
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

static int append_create_key_pair_fixture_attribute(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **attributes,
    uint32_t raw_tag_value) {
    kmipkit_ttlv_value_t *value = 0;
    if (kmipkit_ttlv_value_enumeration(UINT32_C(0xdeadbeef), &value) != 0) {
        return 0;
    }
    if (!append_derive_key_parameter(limits, attributes, raw_tag_value, &value)) {
        kmipkit_ttlv_value_release(value);
        return 0;
    }
    return 1;
}

static int create_key_pair_fixture_item(
    kmipkit_codec_limits_t *limits,
    kmipkit_client_batch_item_t **out_item) {
    kmipkit_ttlv_structure_t *common_attributes = 0;
    kmipkit_ttlv_structure_t *private_attributes = 0;
    kmipkit_ttlv_structure_t *public_attributes = 0;
    kmipkit_ttlv_structure_t *common_masks = 0;
    kmipkit_ttlv_structure_t *public_masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    if (limits == 0 || out_item == 0) {
        return 0;
    }
    *out_item = 0;
    if (kmipkit_ttlv_structure_create(&common_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&private_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&public_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&common_masks) != 0 ||
        kmipkit_ttlv_structure_create(&public_masks) != 0 ||
        !append_create_key_pair_fixture_attribute(
            limits, &common_attributes, UINT32_C(0x540001)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &common_attributes, UINT32_C(0x540001)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &private_attributes, UINT32_C(0x420028)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &private_attributes, UINT32_C(0x540002)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &private_attributes, UINT32_C(0x540002)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &public_attributes, UINT32_C(0x420028)) ||
        !append_create_key_pair_fixture_attribute(
            limits, &public_attributes, UINT32_C(0x540003)) ||
        kmipkit_client_batch_item_create_key_pair(
            common_attributes,
            private_attributes,
            public_attributes,
            common_masks,
            0,
            public_masks,
            &item) != 0 ||
        item == 0) {
        goto cleanup;
    }
    *out_item = item;
    item = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(public_masks);
    kmipkit_ttlv_structure_release(common_masks);
    kmipkit_ttlv_structure_release(public_attributes);
    kmipkit_ttlv_structure_release(private_attributes);
    kmipkit_ttlv_structure_release(common_attributes);
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

static int create_import_request(
    kmipkit_codec_limits_t *limits, kmipkit_client_batch_item_t **out_item) {
    static const uint8_t identifier_bytes[] = "import-key";
    static const uint8_t key_material_bytes[] = {0xde, 0xad, 0x00, 0xff};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *object_body = 0;
    kmipkit_ttlv_structure_t *key_block = 0;
    kmipkit_ttlv_structure_t *key_value = 0;
    kmipkit_ttlv_structure_t *any_object = 0;
    kmipkit_ttlv_value_t *object_value = 0;
    kmipkit_ttlv_value_t *secret_value = 0;
    int result = 0;

    if (limits == 0 || out_item == 0) {
        return 0;
    }
    *out_item = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&object_body) != 0 ||
        kmipkit_ttlv_structure_create(&key_block) != 0 ||
        kmipkit_ttlv_structure_create(&key_value) != 0 ||
        kmipkit_ttlv_structure_create(&any_object) != 0 ||
        kmipkit_ttlv_value_enumeration(UINT32_C(1), &secret_value) != 0 ||
        !append_derive_key_parameter(
            limits, &object_body, UINT32_C(0x420086), &secret_value) ||
        kmipkit_ttlv_value_enumeration(UINT32_C(1), &secret_value) != 0 ||
        !append_derive_key_parameter(
            limits, &key_block, UINT32_C(0x420042), &secret_value) ||
        kmipkit_ttlv_value_byte_string(
            limits, key_material_bytes, (uint64_t)sizeof(key_material_bytes),
            &secret_value) != 0 ||
        !append_derive_key_parameter(
            limits, &key_value, UINT32_C(0x420043), &secret_value) ||
        kmipkit_ttlv_value_structure(key_value, limits, &secret_value) != 0) {
        goto cleanup;
    }
    key_value = 0;
    if (!append_derive_key_parameter(
            limits, &key_block, UINT32_C(0x420045), &secret_value) ||
        kmipkit_ttlv_value_structure(key_block, limits, &secret_value) != 0) {
        goto cleanup;
    }
    key_block = 0;
    if (!append_derive_key_parameter(
            limits, &object_body, UINT32_C(0x420040), &secret_value) ||
        kmipkit_ttlv_value_structure(object_body, limits, &object_value) != 0) {
        goto cleanup;
    }
    object_body = 0;
    if (!append_derive_key_parameter(
            limits, &any_object, UINT32_C(0x420085), &object_value)) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_import(
            &identifier, 7, 1, 0, 0, 0, attributes, any_object, out_item) != 0 ||
        *out_item == 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_value_release(object_value);
    kmipkit_ttlv_value_release(secret_value);
    kmipkit_ttlv_structure_release(any_object);
    kmipkit_ttlv_structure_release(object_body);
    kmipkit_ttlv_structure_release(key_block);
    kmipkit_ttlv_structure_release(key_value);
    kmipkit_ttlv_structure_release(attributes);
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

static int interop_constructor_is_callable_from_c(void) {
    static const uint8_t identifier[] = "interop-private-sentinel";
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_interop(
            UINT32_C(0xf1234567), identifier, sizeof(identifier) - 1, &item) != 0) {
        return 0;
    }
    const int result = test_encoded_operation_and_consume(&item, UINT32_C(0x34));
    kmipkit_client_batch_item_release(item);
    return result;
}

static int validate_constructor_is_callable_from_c(void) {
    static const uint8_t identifier_text[] = "validate-chain-private-sentinel";
    const kmipkit_unique_identifier_v1_t identifier = {
        (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        UINT32_C(1),
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        UINT32_C(0),
        identifier_text,
        sizeof(identifier_text) - 1,
        UINT32_C(0),
        INT32_C(0)};
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_validate(
            0, 0, &identifier, 1, 0, 0, 1, INT64_MIN, &item) != 0) {
        return 0;
    }
    const int result = item != 0;
    kmipkit_client_batch_item_release(item);
    return result;
}

static int validate_constructor_copies_nonempty_certificate_array_from_c(void) {
    static const uint8_t certificate_value[] = {0x30, 0x01, 0x00};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_raw_tag_t *type_raw_tag = 0;
    kmipkit_tag_t *type_tag = 0;
    kmipkit_ttlv_value_t *type_value = 0;
    kmipkit_ttlv_item_t *type_item = 0;
    kmipkit_raw_tag_t *value_raw_tag = 0;
    kmipkit_tag_t *value_tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *value_item = 0;
    kmipkit_ttlv_structure_t *certificate = 0;
    kmipkit_ttlv_structure_t *updated_certificate = 0;
    kmipkit_ttlv_structure_t *certificates[1] = {0};
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_client_batch_item_t *item = 0;
    uint64_t field_count = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x0042001d), &type_raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(type_raw_tag, &type_tag) != 0 ||
        kmipkit_ttlv_value_enumeration(UINT32_C(1), &type_value) != 0 ||
        kmipkit_ttlv_item_create(type_tag, type_value, limits, &type_item) != 0) {
        goto cleanup;
    }
    type_value = 0;
    if (kmipkit_ttlv_structure_create(&certificate) != 0 ||
        kmipkit_ttlv_structure_with_item(
            certificate, type_item, limits, &updated_certificate) != 0) {
        goto cleanup;
    }
    certificate = 0;
    type_item = 0;
    certificate = updated_certificate;
    updated_certificate = 0;

    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x0042001e), &value_raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(value_raw_tag, &value_tag) != 0 ||
        kmipkit_ttlv_value_byte_string(
            limits, certificate_value, (uint64_t)sizeof(certificate_value), &value) != 0 ||
        kmipkit_ttlv_item_create(value_tag, value, limits, &value_item) != 0) {
        goto cleanup;
    }
    value = 0;
    if (kmipkit_ttlv_structure_with_item(
            certificate, value_item, limits, &updated_certificate) != 0) {
        goto cleanup;
    }
    certificate = 0;
    value_item = 0;
    certificate = updated_certificate;
    updated_certificate = 0;
    certificates[0] = certificate;
    if (kmipkit_client_batch_item_validate(
            certificates, 1, 0, 0, 0, 0, 0, 0, &item) != 0 || item == 0 ||
        kmipkit_ttlv_structure_view(certificate, &view) != 0 || view == 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &field_count) != 0 || field_count != 2) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(updated_certificate);
    kmipkit_ttlv_structure_release(certificate);
    kmipkit_ttlv_item_release(value_item);
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(value_tag);
    kmipkit_raw_tag_release(value_raw_tag);
    kmipkit_ttlv_item_release(type_item);
    kmipkit_ttlv_value_release(type_value);
    kmipkit_tag_release(type_tag);
    kmipkit_raw_tag_release(type_raw_tag);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int register_protection_storage_masks_builder_is_callable_from_c(void) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_ttlv_structure_t *object_fields = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *object_value = 0;
    kmipkit_ttlv_item_t *object = 0;
    kmipkit_register_request_t *request = 0;
    kmipkit_register_request_t *with_masks = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_ttlv_structure_create(&object_fields) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x0042005b), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_structure(object_fields, limits, &object_value) != 0) {
        goto cleanup;
    }
    object_fields = 0;
    if (kmipkit_ttlv_item_create(tag, object_value, limits, &object) != 0) {
        goto cleanup;
    }
    object_value = 0;
    if (kmipkit_register_request_create(8, attributes, object, &request) != 0) {
        goto cleanup;
    }
    attributes = 0;
    object = 0;
    if (kmipkit_register_request_with_protection_storage_masks(
            request, masks, &with_masks) != 0) {
        goto cleanup;
    }
    request = 0;
    result = with_masks != 0;

cleanup:
    kmipkit_register_request_release(with_masks);
    kmipkit_register_request_release(request);
    kmipkit_ttlv_item_release(object);
    kmipkit_ttlv_value_release(object_value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_structure_release(object_fields);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int validate_constructor_copies_payload_extensions(void) {
    static const uint8_t extension_text[] = "validate-extension-sentinel";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *extension = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_ttlv_item_t *extensions[] = {0};
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x00541234), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, extension_text, sizeof(extension_text) - 1, &value) != 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, &extension) != 0) {
        goto cleanup;
    }
    extensions[0] = extension;
    if (kmipkit_client_batch_item_validate(
            0, 0, 0, 0, extensions, 1, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_item_release(extension);
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int validate_response_accessors_clear_invalid_outputs_from_c(void) {
    uint8_t present = UINT8_MAX;
    uint32_t indicator = UINT32_MAX;
    const int32_t indicator_status = kmipkit_client_batch_item_response_validate(
        0, &present, &indicator);
    if (indicator_status != T076_ERROR_INVALID_INPUT || present != 0 || indicator != 0) {
        return 0;
    }

    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = UINT8_MAX;
    const int32_t ttlv_status = kmipkit_client_batch_item_response_validate_ttlv(
        0, &present, &response_ttlv);
    return ttlv_status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
}

int kmipkit_client_abi_c_consumer_validate_validate_response(
    kmipkit_client_batch_item_response_t *item,
    uint8_t expected_present,
    uint32_t expected_indicator) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x0042005c), UINT32_C(0x00420093), UINT32_C(0x0042007f),
        UINT32_C(0x0042007c)};
    uint8_t present = UINT8_MAX;
    uint32_t indicator = UINT32_MAX;
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    int result = 0;

    if (kmipkit_client_batch_item_response_validate(item, &present, &indicator) != 0 ||
        present != expected_present || indicator != expected_indicator) {
        goto cleanup;
    }
    present = UINT8_MAX;
    if (kmipkit_client_batch_item_response_validate_ttlv(item, &present, &response_ttlv) != 0 ||
        present != expected_present ||
        ((expected_present != 0) != (response_ttlv != 0))) {
        goto cleanup;
    }
    if (expected_present != 0 &&
        (kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
            response_view == 0 ||
            !check_view_tags(
                response_view,
                response_tags,
                (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))))) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
}

static int pkcs11_constructor_is_callable_from_c(void) {
    kmipkit_ttlv_structure_t *function = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&function) != 0 ||
        kmipkit_client_batch_item_pkcs11(function, 0, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    result = test_encoded_operation_and_consume(&item, UINT32_C(0x33));
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(function);
    return result;
}

static int pkcs11_constructor_preserves_opaque_payload_extensions_from_c(void) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *function = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *extension = 0;
    kmipkit_ttlv_item_t *extensions[] = {0};
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_structure_create(&function) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x00541234), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_enumeration(7, &value) != 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, &extension) != 0) {
        goto cleanup;
    }
    value = 0;
    extensions[0] = extension;
    if (kmipkit_client_batch_item_pkcs11_with_extensions_v1(
            function, 0, 0, 0, extensions, 1, &item) != 0) {
        goto cleanup;
    }
    result = item != 0;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_item_release(extension);
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_structure_release(function);
    kmipkit_codec_limits_release(limits);
    return result;
}

static int pkcs11_response_accessor_clears_invalid_outputs_from_c(void) {
    uint8_t present = UINT8_MAX;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    int32_t status = kmipkit_client_batch_item_response_pkcs11_ttlv(
        0, &present, &response_ttlv);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || response_ttlv != 0) {
        return 0;
    }

    kmipkit_ttlv_structure_t *structure = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = UINT8_MAX;
    status = kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
        0, &present, &structure);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || structure != 0) return 0;

    structure = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = UINT8_MAX;
    status = kmipkit_client_batch_item_response_pkcs11_function_ttlv(
        0, &present, &structure);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || structure != 0) return 0;

    kmipkit_byte_span_v1_t correlation = {(const uint8_t *)(uintptr_t)1, UINT64_MAX};
    present = UINT8_MAX;
    status = kmipkit_client_batch_item_response_pkcs11_correlation_value(
        0, &present, &correlation);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 ||
        correlation.data != 0 || correlation.length != 0) return 0;

    structure = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = UINT8_MAX;
    status = kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv(
        0, &present, &structure);
    if (status != T076_ERROR_INVALID_INPUT || present != 0 || structure != 0) return 0;

    structure = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    present = UINT8_MAX;
    status = kmipkit_client_batch_item_response_pkcs11_return_code_ttlv(
        0, &present, &structure);
    return status == T076_ERROR_INVALID_INPUT && present == 0 && structure == 0;
}

static int log_constructor_is_callable_from_c(void) {
    static const uint8_t message[] = "log-message-private-sentinel";
    kmipkit_client_batch_item_t *item = 0;
    if (kmipkit_client_batch_item_log(message, sizeof(message) - 1, &item) != 0 || item == 0) {
        return 0;
    }
    kmipkit_client_batch_item_release(item);

    uint8_t present = UINT8_MAX;
    kmipkit_ttlv_structure_t *response_ttlv = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_response_log_ttlv(
                0, &present, &response_ttlv) != T076_ERROR_INVALID_INPUT) {
        return 0;
    }
    return present == 0 && response_ttlv == 0;
}

static int log_response_is_readable_after_result_release_from_c(
    kmipkit_client_batch_item_response_t *item) {
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    const int32_t status = kmipkit_client_batch_item_response_log_ttlv(
        item, &present, &response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (status != 0 || present != 1 || response_ttlv == 0 ||
        kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        !check_view_tags(view, expected_tags,
            (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0])))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

static int login_response_is_readable_after_result_release_from_c(
    kmipkit_client_batch_item_response_t *item) {
    static const uint32_t expected_ticket_tags[] = {
        UINT32_C(0x42014a), UINT32_C(0x42014b)};
    static const uint8_t expected_ticket_value[] = "login-ticket-native-secret";
    uint8_t response_present = 0;
    uint8_t ticket_present = 0;
    uint32_t ticket_type = 0;
    kmipkit_byte_span_v1_t ticket_value = {0};
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *ticket_ttlv = 0;
    kmipkit_ttlv_structure_view_t *ticket_view = 0;
    int result = 0;

    if (item == 0 || kmipkit_client_batch_item_response_login(
            item, &response_present, &ticket_present, &ticket_type, &ticket_value) != 0 ||
        response_present != 1 || ticket_present != 1 || ticket_type != UINT32_C(0xfeedbeef) ||
        ticket_value.length != sizeof(expected_ticket_value) - 1 || ticket_value.data == 0 ||
        memcmp(ticket_value.data, expected_ticket_value, sizeof(expected_ticket_value) - 1) != 0 ||
        kmipkit_client_batch_item_response_login_ttlv(item, &ttlv_present, &ticket_ttlv) != 0 ||
        ttlv_present != 1 || ticket_ttlv == 0) {
        goto cleanup;
    }
    /* The span is used only while its result-item owner is still live. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(ticket_ttlv, &ticket_view) != 0 ||
        !check_view_tags(ticket_view, expected_ticket_tags,
            (uint64_t)(sizeof(expected_ticket_tags) / sizeof(expected_ticket_tags[0])))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(ticket_view);
    kmipkit_ttlv_structure_release(ticket_ttlv);
    return result;
}

static int interop_response_accessor_clears_invalid_outputs_from_c(void) {
    uint8_t present = UINT8_MAX;
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    const int32_t status = kmipkit_client_batch_item_response_interop_ttlv(
        0, &present, &response_ttlv);
    return status == T076_ERROR_INVALID_INPUT && present == 0 && response_ttlv == 0;
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

static int import_constructor_rejects_invalid_table_238_inputs_from_c(void) {
    static const uint8_t identifier_bytes[] = "import-id";
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = 1;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *empty_object = 0;
    kmipkit_client_batch_item_t *item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    int result = 0;
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&empty_object) != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_import(
            &identifier, UINT32_MAX, 1, 0, 0, 0, attributes, empty_object, &item) !=
            T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_import(
            &identifier, UINT32_MAX, 2, 0, 0, 0, attributes, empty_object, &item) !=
            T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(empty_object);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

/* KMIP Specification v2.1 §6.1.27, Tables 244–246. */
static int join_split_key_constructor_pairs_each_identifier_with_its_threshold(void) {
    static const uint8_t identifier_text[] = "repeated-split-id";
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    kmipkit_split_key_reference_t *references[2] = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_split_key_reference_create(&identifier, 1, &references[0]) != 0 ||
        kmipkit_split_key_reference_create(&identifier, 2, &references[1]) != 0 ||
        kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 ||
        kmipkit_client_batch_item_join_split_key(
            UINT32_C(0xf1234567), references, 2, 1, UINT32_C(0xfedcba98),
            attributes, masks, &item) != 0 || item == 0 ||
        !test_encoded_operation_and_consume(&item, UINT32_C(0x29))) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_join_split_key(
            1, references, 1, 0, 0, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_join_split_key(
            1, references + 1, 1, 0, 0, 0, 0, &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_join_split_key(
            1, 0, 1, 0, 0, 0, 0, &item) != T076_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_split_key_reference_release(references[1]);
    kmipkit_split_key_reference_release(references[0]);
    return result;
}

static int join_split_key_constructor_enforces_aggregate_identifier_budget_from_c(void) {
    const size_t identifier_capacity = 16U * 1024U * 1024U;
    const size_t repeated_identifier_length = identifier_capacity / 2U + 1U;
    uint8_t *identifier_bytes = malloc(identifier_capacity);
    kmipkit_split_key_reference_t *reference = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (identifier_bytes == 0) {
        goto cleanup;
    }
    memset(identifier_bytes, 'x', identifier_capacity);

    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)identifier_capacity;
    if (kmipkit_split_key_reference_create(&identifier, 1, &reference) != 0) {
        goto cleanup;
    }
    kmipkit_split_key_reference_t *exact_limit_occurrence[] = {reference};
    if (kmipkit_client_batch_item_join_split_key(
            1, exact_limit_occurrence, 1, 0, 0, 0, 0, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    kmipkit_split_key_reference_release(reference);
    reference = 0;

    identifier.text_length = (uint64_t)repeated_identifier_length;
    if (kmipkit_split_key_reference_create(&identifier, 1, &reference) != 0) {
        goto cleanup;
    }
    kmipkit_split_key_reference_t *repeated_occurrences[] = {reference, reference};
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_join_split_key(
            1, repeated_occurrences, 2, 0, 0, 0, 0, &item) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || item != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_split_key_reference_release(reference);
    free(identifier_bytes);
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
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        !create_key_pair_fixture_item(limits, &item)) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_codec_limits_release(limits);
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
    typedef int32_t (*response_ttlv_getter)(
        kmipkit_client_batch_item_response_t *, uint8_t *, kmipkit_ttlv_structure_t **);
    const response_ttlv_getter response_ttlv_getters[] = {
        kmipkit_client_batch_item_response_hash_ttlv,
        kmipkit_client_batch_item_response_mac_ttlv,
        kmipkit_client_batch_item_response_mac_verify_ttlv,
        kmipkit_client_batch_item_response_sign_ttlv,
        kmipkit_client_batch_item_response_signature_verify_ttlv,
    };
    for (size_t index = 0; index < sizeof(response_ttlv_getters) / sizeof(response_ttlv_getters[0]); ++index) {
        kmipkit_ttlv_structure_t *response_ttlv =
            (kmipkit_ttlv_structure_t *)(uintptr_t)1;
        present = UINT8_MAX;
        if (response_ttlv_getters[index](0, &present, &response_ttlv) !=
                T076_ERROR_INVALID_INPUT || present != 0 || response_ttlv != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return 0;
        }
        kmipkit_ttlv_structure_release(response_ttlv);
    }
    return 1;
}

static int encrypt_decrypt_abi_is_callable_from_compiled_c(void) {
    static const uint8_t request_bytes[] = {0x00, 0x81, 0xff};
    static const uint8_t iv_bytes[] = {0x11, 0x12};
    static const uint8_t tag_bytes[] = {0xa1, 0xa2};
    static const uint8_t aad_bytes[] = {0xb1, 0xb2, 0xb3};
    kmipkit_byte_span_v1_t iv = {iv_bytes, sizeof(iv_bytes)};
    kmipkit_byte_span_v1_t tag = {tag_bytes, sizeof(tag_bytes)};
    kmipkit_byte_span_v1_t aad = {aad_bytes, sizeof(aad_bytes)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = (const uint8_t *)"cryptographic-key";
    identifier.text_length = 17;

    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_value_t *data = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_value_byte_string(
            limits, request_bytes, sizeof(request_bytes), &data) != 0 ||
        kmipkit_client_batch_item_encrypt(
            &identifier, 0, data, &iv, 0, 1, 1, &aad, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;
    if (kmipkit_client_batch_item_decrypt(
            &identifier, 0, data, &iv, &tag, 0, 1, 1, &aad, &item) != 0 || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;

    uint8_t present = UINT8_MAX;
    uint8_t data_present = UINT8_MAX;
    uint8_t iv_present = UINT8_MAX;
    uint8_t correlation_present = UINT8_MAX;
    uint8_t tag_present = UINT8_MAX;
    kmipkit_unique_identifier_v1_t response_identifier = {0};
    response_identifier.struct_size = (uint32_t)sizeof(response_identifier);
    response_identifier.abi_version = UINT32_C(1);
    response_identifier.kind = UINT32_MAX;
    response_identifier.text_data = (const uint8_t *)(uintptr_t)1;
    response_identifier.text_length = UINT64_MAX;
    kmipkit_byte_span_v1_t response_data = {(const uint8_t *)(uintptr_t)1, UINT64_MAX};
    kmipkit_byte_span_v1_t response_iv = {(const uint8_t *)(uintptr_t)1, UINT64_MAX};
    kmipkit_byte_span_v1_t response_correlation = {(const uint8_t *)(uintptr_t)1, UINT64_MAX};
    kmipkit_byte_span_v1_t response_tag = {(const uint8_t *)(uintptr_t)1, UINT64_MAX};
    if (kmipkit_client_batch_item_response_encrypt(
            0, &present, &response_identifier, &data_present, &response_data,
            &iv_present, &response_iv, &correlation_present, &response_correlation,
            &tag_present, &response_tag) != T076_ERROR_INVALID_INPUT ||
        present != 0 || data_present != 0 || iv_present != 0 ||
        correlation_present != 0 || tag_present != 0 ||
        response_identifier.kind != 0 || response_identifier.text_data != 0 ||
        response_identifier.text_length != 0 || response_data.data != 0 ||
        response_data.length != 0 || response_iv.data != 0 || response_iv.length != 0 ||
        response_correlation.data != 0 || response_correlation.length != 0 ||
        response_tag.data != 0 || response_tag.length != 0) {
        goto cleanup;
    }

    present = UINT8_MAX;
    data_present = UINT8_MAX;
    correlation_present = UINT8_MAX;
    response_identifier.kind = UINT32_MAX;
    response_identifier.text_data = (const uint8_t *)(uintptr_t)1;
    response_identifier.text_length = UINT64_MAX;
    response_data.data = (const uint8_t *)(uintptr_t)1;
    response_data.length = UINT64_MAX;
    response_correlation.data = (const uint8_t *)(uintptr_t)1;
    response_correlation.length = UINT64_MAX;
    if (kmipkit_client_batch_item_response_decrypt(
            0, &present, &response_identifier, &data_present, &response_data,
            &correlation_present, &response_correlation) != T076_ERROR_INVALID_INPUT ||
        present != 0 || data_present != 0 || correlation_present != 0 ||
        response_identifier.kind != 0 || response_identifier.text_data != 0 ||
        response_identifier.text_length != 0 || response_data.data != 0 ||
        response_data.length != 0 || response_correlation.data != 0 ||
        response_correlation.length != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_value_release(data);
    kmipkit_codec_limits_release(limits);
    return result;
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
    kmipkit_client_batch_item_t *invalid_locate_item =
        (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_locate(
            0, 0, 0, 0, 0, 0, 0, 0, 0, &invalid_locate_item) !=
            T076_ERROR_INVALID_INPUT ||
        invalid_locate_item != 0) {
        return 0;
    }
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

static int obtain_lease_abi_is_callable_from_c(void) {
    /* OASIS KMIP Specification v2.1 §6.1.35, Tables 268–270. */
    static const uint8_t identifier_bytes[] = "lease-object";
    kmipkit_unique_identifier_v1_t identifier = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);

    kmipkit_client_batch_item_t *omitted_identifier = 0;
    kmipkit_client_batch_item_t *explicit_identifier = 0;
    if (kmipkit_client_batch_item_obtain_lease(0, &omitted_identifier) != 0 ||
        omitted_identifier == 0 ||
        kmipkit_client_batch_item_obtain_lease(&identifier, &explicit_identifier) != 0 ||
        explicit_identifier == 0) {
        kmipkit_client_batch_item_release(explicit_identifier);
        kmipkit_client_batch_item_release(omitted_identifier);
        return 0;
    }
    kmipkit_client_batch_item_release(explicit_identifier);
    kmipkit_client_batch_item_release(omitted_identifier);

    uint8_t present = 0xff;
    uint32_t lease_time = UINT32_MAX;
    int64_t last_change_date = INT64_MAX;
    kmipkit_unique_identifier_v1_t output_identifier = {0};
    output_identifier.struct_size = (uint32_t)sizeof(output_identifier);
    output_identifier.abi_version = 1;
    return kmipkit_client_batch_item_response_obtain_lease(
               0, &present, &output_identifier, &lease_time, &last_change_date) ==
            T076_ERROR_INVALID_INPUT &&
        present == 0 && lease_time == 0 && last_change_date == 0 &&
        output_identifier.kind == 0 && output_identifier.text_data == 0 &&
        output_identifier.text_length == 0;
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

static int encrypt_decrypt_response_fields_match(
    uint64_t index, kmipkit_client_batch_item_response_t *response_item) {
    static const uint8_t identifier_bytes[] = "c-response-key";
    static const uint8_t ciphertext[] = "ciphertext-c";
    static const uint8_t plaintext[] = "plaintext-c";
    static const uint8_t iv_bytes[] = {0x71, 0x72};
    static const uint8_t correlation_bytes[] = {0x81, 0x82};
    static const uint8_t tag_bytes[] = {0x91, 0x92};
    uint8_t present = 0;
    uint8_t data_present = 0;
    uint8_t iv_present = 0;
    uint8_t correlation_present = 0;
    uint8_t tag_present = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_byte_span_v1_t data = {0};
    kmipkit_byte_span_v1_t iv = {0};
    kmipkit_byte_span_v1_t correlation = {0};
    kmipkit_byte_span_v1_t tag = {0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;

    if (index == 24) {
        if (kmipkit_client_batch_item_response_encrypt(
                response_item, &present, &identifier, &data_present, &data,
                &iv_present, &iv, &correlation_present, &correlation,
                &tag_present, &tag) != 0 || present != 1 || data_present != 1 ||
            iv_present != 1 || correlation_present != 1 || tag_present != 1) {
            return 0;
        }
        return response_identifier_matches(
                    &identifier, identifier_bytes, sizeof(identifier_bytes) - 1) &&
            response_span_matches(data, ciphertext, sizeof(ciphertext) - 1) &&
            response_span_matches(iv, iv_bytes, sizeof(iv_bytes)) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes)) &&
            response_span_matches(tag, tag_bytes, sizeof(tag_bytes));
    }
    if (index == 25) {
        if (kmipkit_client_batch_item_response_decrypt(
                response_item, &present, &identifier, &data_present, &data,
                &correlation_present, &correlation) != 0 || present != 1 ||
            data_present != 1 || correlation_present != 1) {
            return 0;
        }
        return response_identifier_matches(
                    &identifier, identifier_bytes, sizeof(identifier_bytes) - 1) &&
            response_span_matches(data, plaintext, sizeof(plaintext) - 1) &&
            response_span_matches(correlation, correlation_bytes, sizeof(correlation_bytes));
    }
    return 0;
}

static int encrypt_decrypt_response_is_readable_from_c(
    kmipkit_client_batch_response_t *response, uint64_t index,
    kmipkit_client_batch_item_response_t **response_item) {
    if (response == 0 || response_item == 0 || *response_item == 0 ||
        !encrypt_decrypt_response_fields_match(index, *response_item)) {
        return 0;
    }
    kmipkit_client_batch_item_response_release(*response_item);
    *response_item = 0;
    if (kmipkit_client_batch_response_item_at(response, index, response_item) != 0 ||
        *response_item == 0) {
        return 0;
    }
    return encrypt_decrypt_response_fields_match(index, *response_item);
}

static int public_ttlv_contains_response_sentinel(kmipkit_ttlv_structure_t *structure);
static int import_response_is_readable_from_c(
    kmipkit_client_batch_item_response_t *item);

static int fixture_hex_nibble(char character, uint8_t *value) {
    if (character >= '0' && character <= '9') {
        *value = (uint8_t)(character - '0');
        return 1;
    }
    if (character >= 'a' && character <= 'f') {
        *value = (uint8_t)(character - 'a' + 10);
        return 1;
    }
    if (character >= 'A' && character <= 'F') {
        *value = (uint8_t)(character - 'A' + 10);
        return 1;
    }
    return 0;
}

static int parse_fixture_hex(
    const char *text, uint8_t *output, uint64_t capacity, uint64_t *output_length) {
    uint64_t text_length = 0;
    if (text == 0 || output == 0 || output_length == 0) {
        return 0;
    }
    text_length = (uint64_t)strlen(text);
    if (text_length == 0 || (text_length % 2) != 0 || text_length / 2 > capacity) {
        return 0;
    }
    for (uint64_t index = 0; index < text_length / 2; ++index) {
        uint8_t high = 0;
        uint8_t low = 0;
        if (!fixture_hex_nibble(text[index * 2], &high) ||
            !fixture_hex_nibble(text[index * 2 + 1], &low)) {
            return 0;
        }
        output[index] = (uint8_t)((high << 4) | low);
    }
    *output_length = text_length / 2;
    return 1;
}

static int parse_fixture_decimal(const char *text, uint64_t *value) {
    uint64_t parsed = 0;
    if (text == 0 || value == 0 || text[0] == '\0') {
        return 0;
    }
    for (uint64_t index = 0; text[index] != '\0'; ++index) {
        const char digit = text[index];
        if (digit < '0' || digit > '9') {
            return 0;
        }
        const uint64_t numeric_digit = (uint64_t)(digit - '0');
        if (parsed > (UINT64_MAX - numeric_digit) / UINT64_C(10)) {
            return 0;
        }
        parsed = parsed * UINT64_C(10) + numeric_digit;
    }
    *value = parsed;
    return 1;
}

static int rekey_key_pair_marker_structure_create(
    kmipkit_codec_limits_t *limits,
    uint32_t marker_tag,
    const uint8_t *marker_value,
    uint64_t marker_length,
    kmipkit_ttlv_structure_t **out_structure) {
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *updated = 0;
    int result = 0;

    if (limits == 0 || marker_value == 0 || out_structure == 0) return 0;
    *out_structure = 0;
    if (kmipkit_ttlv_structure_create(&structure) != 0 ||
        kmipkit_ttlv_raw_tag_create(marker_tag, &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_byte_string(limits, marker_value, marker_length, &value) != 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, &item) != 0) {
        goto cleanup;
    }
    value = 0; /* the item owns the marker bytes */
    if (kmipkit_ttlv_structure_with_item(structure, item, limits, &updated) != 0) {
        goto cleanup;
    }
    structure = 0; /* the updated structure owns the original structure and item */
    item = 0;
    *out_structure = updated;
    updated = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    return result;
}

static int execute_rekey_key_pair_request_from_c(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t private_identifier_bytes[] = "existing-private-key";
    static const uint8_t batch_item_id[] = "pair-c";
    static const uint8_t expected_private_identifier[] = "replacement-private";
    static const uint8_t expected_public_identifier[] = "replacement-public";
    static const uint8_t marker_values[6][20] = {
        "common-attributes", "private-attributes", "public-attributes",
        "common-masks", "private-masks", "public-masks",
    };
    static const uint64_t marker_lengths[6] = {
        sizeof("common-attributes") - 1,
        sizeof("private-attributes") - 1,
        sizeof("public-attributes") - 1,
        sizeof("common-masks") - 1,
        sizeof("private-masks") - 1,
        sizeof("public-masks") - 1,
    };
    static const uint32_t marker_tags[6] = {
        UINT32_C(0x541101), UINT32_C(0x541102), UINT32_C(0x541103),
        UINT32_C(0x541104), UINT32_C(0x541105), UINT32_C(0x541106),
    };
    kmipkit_ttlv_structure_t *structures[6] = {0};
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_item_t *identified_item = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    kmipkit_unique_identifier_v1_t private_identifier = {0};
    kmipkit_unique_identifier_v1_t public_identifier = {0};
    uint64_t response_count = 0;
    uint8_t present = 0;
    int result = 1;

    if (client == 0 || limits == 0) goto cleanup;
    for (uint64_t index = 0; index < 6; ++index) {
        if (!rekey_key_pair_marker_structure_create(
                limits, marker_tags[index], marker_values[index], marker_lengths[index],
                &structures[index])) {
            goto cleanup;
        }
    }
    private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
    private_identifier.abi_version = 1;
    private_identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    private_identifier.text_data = private_identifier_bytes;
    private_identifier.text_length = (uint64_t)(sizeof(private_identifier_bytes) - 1);
    if (kmipkit_client_batch_item_rekey_key_pair(
            &private_identifier, 1, UINT32_C(86400),
            structures[0], structures[1], structures[2],
            structures[3], structures[4], structures[5], &item) != 0 || item == 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_with_unique_batch_item_id(
            item, batch_item_id, (uint64_t)(sizeof(batch_item_id) - 1), &identified_item) != 0 ||
        identified_item == 0) {
        item = 0; /* the valid input was consumed by the identifier builder */
        goto cleanup;
    }
    item = 0; /* the identifier builder consumed the original item */
    item = identified_item;
    identified_item = 0;
    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0 ||
        kmipkit_client_batch_add_item(batch, &item) != 0 || item != 0 ||
        kmipkit_client_execute(client, &batch, limits, &response) != 0 ||
        batch != 0 || response == 0 ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != 1 ||
        kmipkit_client_batch_response_item_at(response, 0, &response_item) != 0 ||
        response_item == 0) {
        goto cleanup;
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    private_identifier = (kmipkit_unique_identifier_v1_t){0};
    private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
    private_identifier.abi_version = 1;
    public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
    public_identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata) != 0 ||
        metadata.operation != UINT32_C(0x1d) || metadata.outcome_kind != 1 ||
        metadata.result_status != 0 ||
        kmipkit_client_batch_item_response_rekey_key_pair(
            response_item, &present, &private_identifier, &public_identifier) != 0 ||
        present != 1 || private_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        private_identifier.text_data == 0 ||
        private_identifier.text_length != sizeof(expected_private_identifier) - 1 ||
        memcmp(private_identifier.text_data, expected_private_identifier,
            sizeof(expected_private_identifier) - 1) != 0 ||
        public_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        public_identifier.text_data == 0 ||
        public_identifier.text_length != sizeof(expected_public_identifier) - 1 ||
        memcmp(public_identifier.text_data, expected_public_identifier,
            sizeof(expected_public_identifier) - 1) != 0) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(identified_item);
    kmipkit_client_batch_item_release(item);
    for (uint64_t index = 0; index < 6; ++index) {
        kmipkit_ttlv_structure_release(structures[index]);
    }
    return result;
}

/* OASIS KMIP v2.1 §6.1.10, Tables 193–195; §5.1, Table 157. */
static int create_split_key_fixture_response_matches(
    kmipkit_client_batch_item_response_t *response_item, int pending,
    const uint8_t *expected_batch_item_id, uint64_t expected_batch_item_id_length) {
    static const uint8_t expected_text[] = "part-a";
    static const uint8_t expected_pending_text[] = "pending-part";
    static const uint8_t expected_correlation[] = {0x00, 0xff, 0x80};
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_byte_span_v1_t batch_item_id = {0};
    kmipkit_byte_span_v1_t correlation = {0};
    uint8_t batch_item_id_present = 0;
    uint8_t correlation_present = 0;
    uint8_t split_present = 0;
    uint64_t split_count = 0;

    if (response_item == 0) return 0;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    if (kmipkit_client_batch_item_response_unique_batch_item_id(
            response_item, &batch_item_id_present, &batch_item_id) != 0 ||
        batch_item_id_present != 1 ||
        !response_span_matches(batch_item_id, expected_batch_item_id,
            expected_batch_item_id_length) ||
        kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata) != 0 ||
        metadata.operation != UINT32_C(40) ||
        metadata.outcome_kind != (pending ? UINT32_C(2) : UINT32_C(1)) ||
        metadata.result_status != (pending ? UINT32_C(2) : UINT32_C(0)) ||
        metadata.has_result_reason != 0 || metadata.has_result_message != 0 ||
        metadata.result_message_data != 0 || metadata.result_message_length != 0 ||
        kmipkit_client_batch_item_response_pending_correlation_value(
            response_item, &correlation_present, &correlation) != 0) {
        return 0;
    }
    if (pending) {
        if (correlation_present != 1 ||
            !response_span_matches(correlation, expected_correlation, sizeof(expected_correlation)) ||
            kmipkit_client_batch_item_response_create_split_key_count(
                response_item, &split_present, &split_count) != 0 ||
            split_present != 1 || split_count != UINT64_C(1)) {
            return 0;
        }
        identifier.struct_size = (uint32_t)sizeof(identifier);
        identifier.abi_version = 1;
        return kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                   response_item, 0, &identifier) == 0 &&
            identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
            response_identifier_matches(
                &identifier, expected_pending_text,
                (uint64_t)(sizeof(expected_pending_text) - 1));
    }
    if (correlation_present != 0 ||
        kmipkit_client_batch_item_response_create_split_key_count(
            response_item, &split_present, &split_count) != 0 ||
        split_present != 1 || split_count != UINT64_C(4)) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            response_item, 0, &identifier) != 0 ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        !response_identifier_matches(
            &identifier, expected_text, (uint64_t)(sizeof(expected_text) - 1))) {
        return 0;
    }
    identifier = (kmipkit_unique_identifier_v1_t){0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            response_item, 1, &identifier) != 0 ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION ||
        identifier.enumeration_value != UINT32_MAX) {
        return 0;
    }
    identifier = (kmipkit_unique_identifier_v1_t){0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            response_item, 2, &identifier) != 0 ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
        identifier.integer_value != -7) {
        return 0;
    }
    identifier = (kmipkit_unique_identifier_v1_t){0};
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    return kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
               response_item, 3, &identifier) == 0 &&
        identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        response_identifier_matches(
            &identifier, expected_text, (uint64_t)(sizeof(expected_text) - 1));
}

static int execute_create_split_key_fixture_request_from_c(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits,
    const uint8_t *success_batch_item_id, uint64_t success_batch_item_id_length,
    const uint8_t *pending_batch_item_id, uint64_t pending_batch_item_id_length) {
    const uint8_t *batch_item_ids[2] = {success_batch_item_id, pending_batch_item_id};
    const uint64_t batch_item_id_lengths[2] = {
        success_batch_item_id_length, pending_batch_item_id_length};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *masks = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *items[2] = {0};
    kmipkit_client_batch_item_t *identified_item = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint64_t response_count = 0;
    int result = 1;

    if (client == 0 || limits == 0 || batch_item_ids[0] == 0 ||
        batch_item_id_lengths[0] == 0 || batch_item_ids[1] == 0 ||
        batch_item_id_lengths[1] == 0 ||
        kmipkit_ttlv_structure_create(&attributes) != 0 || attributes == 0 ||
        kmipkit_ttlv_structure_create(&masks) != 0 || masks == 0 ||
        kmipkit_client_batch_create(&batch) != 0 || batch == 0 ||
        kmipkit_client_batch_set_asynchronous_indicator(batch, 2) != 0) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -7;
    for (uint64_t index = 0; index < UINT64_C(2); ++index) {
        if (kmipkit_client_batch_item_create_split_key(
                UINT32_C(0xf0000000), &identifier, -3, 4, UINT32_C(0xdeadbeef),
                0, 0, 0, attributes, masks, &items[index]) != 0 || items[index] == 0 ||
            kmipkit_client_batch_item_with_unique_batch_item_id(
                items[index], batch_item_ids[index], batch_item_id_lengths[index],
                &identified_item) != 0 || identified_item == 0) {
            goto cleanup;
        }
        items[index] = 0; /* consumed by the Unique Batch Item ID builder */
        items[index] = identified_item;
        identified_item = 0;
        if (kmipkit_client_batch_add_item(batch, &items[index]) != 0 || items[index] != 0) {
            goto cleanup;
        }
    }
    if (kmipkit_client_execute(client, &batch, limits, &response) != 0 ||
        batch != 0 || response == 0 ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != UINT64_C(2)) {
        goto cleanup;
    }
    if (kmipkit_client_batch_response_item_at(response, 0, &response_item) != 0 ||
        response_item == 0 || !create_split_key_fixture_response_matches(
            response_item, 0, success_batch_item_id, success_batch_item_id_length)) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_item_release(identified_item);
    response_item = 0;
    if (kmipkit_client_batch_response_item_at(response, 1, &response_item) != 0 ||
        response_item == 0 || !create_split_key_fixture_response_matches(
            response_item, 1, pending_batch_item_id, pending_batch_item_id_length)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(items[1]);
    kmipkit_client_batch_item_release(items[0]);
    kmipkit_ttlv_structure_release(masks);
    kmipkit_ttlv_structure_release(attributes);
    return result;
}

enum {
    FIXTURE_VERSION_COUNT_ARG_INDEX = 15,
    FIXTURE_VERSION_VALUES_ARG_INDEX = 16,
    FIXTURE_VERSION_CAPACITY = 64,
};

static int cancel_response_payload_matches(
    kmipkit_ttlv_structure_t *payload,
    const uint8_t *expected_correlation,
    uint64_t expected_correlation_length) {
    static const uint32_t expected_tags[2] = {UINT32_C(0x420006), UINT32_C(0x420012)};
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *items[2] = {0};
    kmipkit_ttlv_value_view_t *values[2] = {0};
    kmipkit_tag_t *tags[2] = {0};
    uint32_t actual_tag = 0;
    uint8_t item_type = 0;
    uint64_t item_count = 0;
    uint64_t actual_correlation_length = 0;
    uint32_t cancellation_result = 0;
    int result = 0;

    if (payload == 0 || expected_correlation == 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &item_count) != 0 || item_count != 2) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < UINT64_C(2); ++index) {
        if (kmipkit_ttlv_structure_view_item_at(view, index, &items[index]) != 0 ||
            items[index] == 0 || kmipkit_ttlv_item_view_tag(items[index], &tags[index]) != 0 ||
            tags[index] == 0 || kmipkit_ttlv_tag_value(tags[index], &actual_tag) != 0 ||
            actual_tag != expected_tags[index] ||
            kmipkit_ttlv_item_view_value(items[index], &values[index]) != 0 ||
            values[index] == 0 || kmipkit_ttlv_item_view_type(items[index], &item_type) != 0) {
            goto cleanup;
        }
        if (index == 0) {
            if (item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
                kmipkit_ttlv_value_view_byte_length(values[index], &actual_correlation_length) !=
                    0 ||
                actual_correlation_length != expected_correlation_length) {
                goto cleanup;
            }
            for (uint64_t byte_index = 0; byte_index < expected_correlation_length; ++byte_index) {
                uint8_t actual_byte = 0;
                if (kmipkit_ttlv_value_view_byte_at(values[index], byte_index, &actual_byte) != 0 ||
                    actual_byte != expected_correlation[byte_index]) {
                    goto cleanup;
                }
            }
        } else if (item_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
                   kmipkit_ttlv_value_view_enumeration(values[index], &cancellation_result) != 0 ||
                   cancellation_result != UINT32_C(1)) {
            goto cleanup;
        }
    }
    result = 1;

cleanup:
    kmipkit_tag_release(tags[1]);
    kmipkit_tag_release(tags[0]);
    kmipkit_ttlv_value_view_release(values[1]);
    kmipkit_ttlv_value_view_release(values[0]);
    kmipkit_ttlv_item_view_release(items[1]);
    kmipkit_ttlv_item_view_release(items[0]);
    kmipkit_ttlv_structure_view_release(view);
    return result;
}

static int async_response_extension_matches(kmipkit_ttlv_structure_t *extension);

int kmipkit_client_abi_c_consumer_validate_present_empty_async_payload(
    kmipkit_client_async_operation_outcome_t *outcome) {
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint8_t present = UINT8_C(9);
    uint64_t count = UINT64_MAX;
    int result = 0;

    if (outcome == 0 ||
        kmipkit_client_async_operation_outcome_response_payload_ttlv(
            outcome, &present, &payload) != 0 ||
        present != UINT8_C(1) || payload == 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0) {
        goto cleanup;
    }
    result = count == UINT64_C(0);

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(payload);
    return result;
}

static int execute_cancel_payload_request_from_c(
    kmipkit_client_t *client,
    kmipkit_codec_limits_t *limits,
    const uint8_t *correlation,
    uint64_t correlation_length) {
    kmipkit_client_async_operation_outcome_t *outcome = 0;
    kmipkit_client_async_operation_metadata_v1_t metadata = {0};
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_t *extension = 0;
    kmipkit_ttlv_structure_view_t *extension_view = 0;
    kmipkit_byte_span_v1_t top_level_correlation = {0};
    uint8_t top_level_correlation_present = UINT8_C(9);
    uint8_t payload_present = UINT8_C(9);
    uint8_t extension_present = UINT8_C(9);
    uint64_t extension_count = UINT64_MAX;
    uint64_t extension_items = UINT64_MAX;
    int result = 1;

    if (client == 0 || limits == 0 || correlation == 0 || correlation_length == 0 ||
        kmipkit_client_execute_cancel(client, correlation, correlation_length, limits, &outcome) !=
            0 ||
        outcome == 0) {
        goto cleanup;
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    if (kmipkit_client_async_operation_outcome_metadata_v1(outcome, &metadata) != 0 ||
        metadata.operation != UINT32_C(0x19) || metadata.result_status != UINT32_C(0) ||
        metadata.has_cancellation_result != UINT32_C(1) ||
        metadata.cancellation_result != UINT32_C(1) ||
        kmipkit_client_async_operation_outcome_correlation_value(
            outcome, &top_level_correlation_present, &top_level_correlation) != 0 ||
        top_level_correlation_present != 0 || top_level_correlation.data != 0 ||
        top_level_correlation.length != 0 ||
        kmipkit_client_async_operation_outcome_response_payload_ttlv(
            outcome, &payload_present, &payload) != 0 ||
        payload_present != 1 || payload == 0 ||
        !cancel_response_payload_matches(payload, correlation, correlation_length) ||
        kmipkit_client_async_operation_outcome_extension_count(outcome, &extension_count) != 0 ||
        extension_count != UINT64_C(1) ||
        kmipkit_client_async_operation_outcome_extension_ttlv_at(
            outcome, UINT64_C(0), &extension_present, &extension) != 0 ||
        extension_present != UINT8_C(1) || extension == 0 ||
        kmipkit_ttlv_structure_view(extension, &extension_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(extension_view, &extension_items) != 0 ||
        extension_items != UINT64_C(3) || !async_response_extension_matches(extension)) {
        goto cleanup;
    }
    kmipkit_client_async_operation_outcome_release(outcome);
    outcome = 0;
    result = 0;

cleanup:
    kmipkit_ttlv_structure_view_release(extension_view);
    kmipkit_ttlv_structure_release(extension);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_client_async_operation_outcome_release(outcome);
    return result;
}

static void set_text_identifier(
    kmipkit_unique_identifier_v1_t *identifier, const uint8_t *text, uint64_t text_length) {
    identifier->struct_size = (uint32_t)sizeof(*identifier);
    identifier->abi_version = 1;
    identifier->kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier->text_data = text;
    identifier->text_length = text_length;
}

static void set_integer_identifier(
    kmipkit_unique_identifier_v1_t *identifier, int64_t value) {
    identifier->struct_size = (uint32_t)sizeof(*identifier);
    identifier->abi_version = 1;
    identifier->kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier->integer_value = value;
}

static int append_request_builder_item(
    kmipkit_client_batch_t *batch, kmipkit_client_batch_item_t **item,
    const uint8_t *batch_item_id, uint64_t batch_item_id_length) {
    kmipkit_client_batch_item_t *identified_item = 0;
    if (*item == 0 ||
        kmipkit_client_batch_item_with_unique_batch_item_id(
            *item, batch_item_id, batch_item_id_length, &identified_item) != 0 ||
        identified_item == 0) {
        kmipkit_client_batch_item_release(*item);
        *item = 0;
        return 0;
    }
    kmipkit_client_batch_item_release(*item);
    *item = identified_item;
    if (kmipkit_client_batch_add_item(batch, item) != 0 || *item != 0) {
        kmipkit_client_batch_item_release(*item);
        *item = 0;
        return 0;
    }
    return 1;
}

static int execute_request_builders_wire_from_c(
    kmipkit_client_t *client, kmipkit_codec_limits_t *limits) {
    static const uint8_t check_text[] = "check-text";
    static const uint8_t get_text[] = "get-text";
    static const uint8_t export_text[] = "export-text";
    static const uint8_t rekey_text[] = "rekey-text";
    static const uint8_t allocation_text[] = "allocation-text";
    static const uint8_t recertify_attribute_text[] = "recertify-attributes";
    static const uint8_t certify_public_key_text[] = "certify-public-key";
    static const uint8_t certify_no_type_text[] = "certify-no-type";
    static const uint8_t certify_no_structures_text[] = "certify-no-structures";
    static const uint8_t certify_type_only_text[] = "certify-type-only";
    static const uint8_t certify_request_sentinel[] = "certify-request-sentinel";
    static const uint8_t recertify_request_sentinel[] = "recertify-request-sentinel";
    static const uint8_t join_split_attribute_text[] = "join-split-attributes";
    static const uint8_t split_key_text[] = "join-split-id";
    static const uint8_t private_key_text[] = "private-key-to-rekey";
    static const uint8_t batch_item_ids[28][8] = {
        "item-0", "item-1", "item-2", "item-3", "item-4", "item-5",
        "item-6", "item-7", "item-8", "item-9", "item-10", "item-11",
        "item-12", "item-13", "item-14", "item-15", "item-16", "item-17",
        "item-18", "item-19", "item-20", "item-21", "item-22", "item-23",
        "item-24", "item-25", "item-26", "item-27"};
    kmipkit_unique_identifier_v1_t identifiers[21] = {{0}};
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_split_key_reference_t *split_key_reference = 0;
    kmipkit_ttlv_structure_t *create_attributes = 0;
    kmipkit_ttlv_structure_t *rekey_attributes = 0;
    kmipkit_ttlv_structure_t *rekey_masks = 0;
    kmipkit_ttlv_structure_t *recertify_attributes = 0;
    kmipkit_ttlv_structure_t *join_split_attributes = 0;
    kmipkit_ttlv_structure_t *certify_attributes = 0;
    kmipkit_ttlv_structure_t *certify_masks = 0;
    kmipkit_ttlv_structure_t *import_attributes = 0;
    kmipkit_ttlv_structure_t *wrapped_import_object = 0;
    kmipkit_ttlv_structure_t *wrapped_key_info = 0;
    kmipkit_ttlv_structure_t *wrapped_key_data = 0;
    kmipkit_ttlv_structure_t *wrapped_key_block = 0;
    kmipkit_ttlv_structure_t *wrapped_key_block_holder = 0;
    kmipkit_ttlv_structure_t *wrapping_specification = 0;
    kmipkit_ttlv_structure_t *specification_key_info = 0;
    kmipkit_ttlv_value_t *rekey_attribute_value = 0;
    kmipkit_ttlv_value_t *rekey_mask_value = 0;
    kmipkit_ttlv_value_t *recertify_attribute_value = 0;
    kmipkit_ttlv_value_t *join_split_attribute_value = 0;
    kmipkit_ttlv_value_t *certify_attribute_value = 0;
    kmipkit_ttlv_value_t *certify_mask_value = 0;
    kmipkit_ttlv_value_t *wrapping_value = 0;
    kmipkit_ttlv_value_t *wrapped_key_value = 0;
    int result = 21;
    uint64_t response_count = 0;

    set_text_identifier(&identifiers[0], check_text, sizeof(check_text) - 1);
    set_text_identifier(&identifiers[1], get_text, sizeof(get_text) - 1);
    set_text_identifier(&identifiers[2], export_text, sizeof(export_text) - 1);
    set_text_identifier(&identifiers[3], rekey_text, sizeof(rekey_text) - 1);
    set_integer_identifier(&identifiers[4], -7);
    set_text_identifier(&identifiers[5], allocation_text, sizeof(allocation_text) - 1);
    set_text_identifier(&identifiers[6], (const uint8_t *)"recertify-text", sizeof("recertify-text") - 1);
    set_text_identifier(&identifiers[7], (const uint8_t *)"certificate-request-text",
            sizeof("certificate-request-text") - 1);
    set_text_identifier(&identifiers[8], (const uint8_t *)"recertify-omit-text",
            sizeof("recertify-omit-text") - 1);
    set_text_identifier(&identifiers[9], split_key_text, sizeof(split_key_text) - 1);
    set_text_identifier(&identifiers[10], private_key_text, sizeof(private_key_text) - 1);
    set_text_identifier(&identifiers[11], certify_public_key_text, sizeof(certify_public_key_text) - 1);
    set_text_identifier(&identifiers[12], certify_no_type_text, sizeof(certify_no_type_text) - 1);
    set_text_identifier(&identifiers[13], certify_no_structures_text, sizeof(certify_no_structures_text) - 1);
    set_text_identifier(&identifiers[14], certify_type_only_text, sizeof(certify_type_only_text) - 1);
    set_text_identifier(
        &identifiers[15], (const uint8_t *)"recertify-type-only",
        sizeof("recertify-type-only") - 1);
    set_text_identifier(
        &identifiers[16], (const uint8_t *)"import-replace-omitted",
        sizeof("import-replace-omitted") - 1);
    set_text_identifier(
        &identifiers[17], (const uint8_t *)"import-replace-false",
        sizeof("import-replace-false") - 1);
    set_text_identifier(
        &identifiers[18], (const uint8_t *)"import-replace-true",
        sizeof("import-replace-true") - 1);
    set_text_identifier(
        &identifiers[19], (const uint8_t *)"export-wrapped",
        sizeof("export-wrapped") - 1);
    set_text_identifier(
        &identifiers[20], (const uint8_t *)"get-wrapped",
        sizeof("get-wrapped") - 1);
    const kmipkit_byte_span_v1_t certify_request_value = {
        certify_request_sentinel, sizeof(certify_request_sentinel) - 1};
    const kmipkit_byte_span_v1_t recertify_request_value = {
        recertify_request_sentinel, sizeof(recertify_request_sentinel) - 1};

    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0 ||
        kmipkit_client_batch_set_order_option(batch, 1) != 0 ||
        kmipkit_ttlv_structure_create(&create_attributes) != 0 ||
        kmipkit_client_batch_item_create(2, create_attributes, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[0], strlen((const char *)batch_item_ids[0]))) {
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(create_attributes);
    create_attributes = 0;

    if (kmipkit_ttlv_structure_create(&rekey_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&rekey_masks) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, (const uint8_t *)"rekey-attributes",
            sizeof("rekey-attributes") - 1, &rekey_attribute_value) != 0 ||
        !append_derive_key_parameter(
            limits, &rekey_attributes, UINT32_C(0x4200fd), &rekey_attribute_value) ||
        kmipkit_ttlv_value_integer(3, &rekey_mask_value) != 0 ||
        !append_derive_key_parameter(
            limits, &rekey_masks, UINT32_C(0x42015E), &rekey_mask_value)) {
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_create(&recertify_attributes) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, recertify_attribute_text, sizeof(recertify_attribute_text) - 1,
            &recertify_attribute_value) != 0 ||
        !append_derive_key_parameter(
            limits, &recertify_attributes, UINT32_C(0x4200fd),
            &recertify_attribute_value) ||
        kmipkit_ttlv_structure_create(&join_split_attributes) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, join_split_attribute_text, sizeof(join_split_attribute_text) - 1,
            &join_split_attribute_value) != 0 ||
        !append_derive_key_parameter(
            limits, &join_split_attributes, UINT32_C(0x4200fd),
            &join_split_attribute_value)) {
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_create(&certify_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&certify_masks) != 0 ||
        kmipkit_ttlv_value_enumeration(1, &certify_attribute_value) != 0 ||
        !append_derive_key_parameter(
            limits, &certify_attributes, UINT32_C(0x42001d),
            &certify_attribute_value) ||
        kmipkit_ttlv_value_integer(3, &certify_mask_value) != 0 ||
        !append_derive_key_parameter(
            limits, &certify_masks, UINT32_C(0x42015e), &certify_mask_value)) {
        goto cleanup;
    }

    if (kmipkit_ttlv_structure_create(&import_attributes) != 0 ||
        kmipkit_ttlv_structure_create(&wrapping_specification) != 0 ||
        kmipkit_ttlv_value_enumeration(1, &wrapping_value) != 0 ||
        !append_derive_key_parameter(
            limits, &wrapping_specification, UINT32_C(0x42009E), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&specification_key_info) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, (const uint8_t *)"wrapping-key-id", sizeof("wrapping-key-id") - 1,
            &wrapping_value) != 0 ||
        !append_derive_key_parameter(
            limits, &specification_key_info, UINT32_C(0x420094), &wrapping_value) ||
        kmipkit_ttlv_value_structure(specification_key_info, limits, &wrapping_value) != 0) {
        goto cleanup;
    }
    specification_key_info = 0;
    if (!append_derive_key_parameter(
            limits, &wrapping_specification, UINT32_C(0x420036), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&wrapped_key_info) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, (const uint8_t *)"wrapping-key", sizeof("wrapping-key") - 1,
            &wrapping_value) != 0 ||
        !append_derive_key_parameter(
            limits, &wrapped_key_info, UINT32_C(0x420094), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&wrapped_key_data) != 0 ||
        kmipkit_ttlv_value_enumeration(1, &wrapping_value) != 0 ||
        !append_derive_key_parameter(
            limits, &wrapped_key_data, UINT32_C(0x42009E), &wrapping_value) ||
        kmipkit_ttlv_value_structure(wrapped_key_info, limits, &wrapping_value) != 0) {
        goto cleanup;
    }
    wrapped_key_info = 0;
    if (!append_derive_key_parameter(
            limits, &wrapped_key_data, UINT32_C(0x420036), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&wrapped_key_block) != 0 ||
        kmipkit_ttlv_value_enumeration(1, &wrapping_value) != 0 ||
        !append_derive_key_parameter(
            limits, &wrapped_key_block, UINT32_C(0x420042), &wrapping_value) ||
        kmipkit_ttlv_value_byte_string(
            limits, (const uint8_t *)"synthetic-wrapped-key-value",
            sizeof("synthetic-wrapped-key-value") - 1, &wrapped_key_value) != 0 ||
        !append_derive_key_parameter(
            limits, &wrapped_key_block, UINT32_C(0x420045), &wrapped_key_value) ||
        kmipkit_ttlv_value_structure(wrapped_key_data, limits, &wrapping_value) != 0) {
        goto cleanup;
    }
    wrapped_key_data = 0;
    if (!append_derive_key_parameter(
            limits, &wrapped_key_block, UINT32_C(0x420046), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&wrapped_key_block_holder) != 0 ||
        kmipkit_ttlv_value_structure(wrapped_key_block, limits, &wrapping_value) != 0) {
        goto cleanup;
    }
    wrapped_key_block = 0;
    if (!append_derive_key_parameter(
            limits, &wrapped_key_block_holder, UINT32_C(0x420040), &wrapping_value) ||
        kmipkit_ttlv_structure_create(&wrapped_import_object) != 0 ||
        kmipkit_ttlv_value_structure(wrapped_key_block_holder, limits, &wrapping_value) != 0) {
        goto cleanup;
    }
    wrapped_key_block_holder = 0;
    if (!append_derive_key_parameter(
            limits, &wrapped_import_object, UINT32_C(0x42008F), &wrapping_value)) {
        goto cleanup;
    }

    if (kmipkit_client_batch_item_check(
            &identifiers[0], 1, 3, 1, 2, 1, 60, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[1], strlen((const char *)batch_item_ids[1])) ||
        kmipkit_client_batch_item_check(&identifiers[4], 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[2], strlen((const char *)batch_item_ids[2])) ||
        kmipkit_client_batch_item_get(
            &identifiers[1], 1, UINT32_C(0xf1234567), 1, UINT32_C(0xf1234568),
            1, UINT32_C(0xf1234569), 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[3], strlen((const char *)batch_item_ids[3])) ||
        kmipkit_client_batch_item_get(&identifiers[4], 0, 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[4], strlen((const char *)batch_item_ids[4])) ||
        kmipkit_client_batch_item_export(
            &identifiers[2], 1, UINT32_C(0xfedcba98), 1, UINT32_C(0xfedcba99),
            1, UINT32_C(0xfedcba9a), 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[5], strlen((const char *)batch_item_ids[5])) ||
        kmipkit_client_batch_item_export(&identifiers[4], 0, 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[6], strlen((const char *)batch_item_ids[6])) ||
        kmipkit_client_batch_item_rekey(
            &identifiers[3], 1, 60, rekey_attributes, rekey_masks, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[7], strlen((const char *)batch_item_ids[7])) ||
        kmipkit_client_batch_item_rekey(
            &identifiers[4], 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[8], strlen((const char *)batch_item_ids[8])) ||
        kmipkit_client_batch_item_get_usage_allocation(
            &identifiers[5], 5, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[9], strlen((const char *)batch_item_ids[9])) ||
        kmipkit_client_batch_item_get_usage_allocation(
            0, 7, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[10], strlen((const char *)batch_item_ids[10])) ||
        kmipkit_client_batch_item_recertify(
                &identifiers[6], &identifiers[7], 0, 0, 0, 1, 86400,
                recertify_attributes, rekey_masks, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[11], strlen((const char *)batch_item_ids[11])) ||
        kmipkit_client_batch_item_recertify(
                &identifiers[8], 0, 0, 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[12], strlen((const char *)batch_item_ids[12])) ||
        kmipkit_client_batch_item_create_key_pair(0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[13], strlen((const char *)batch_item_ids[13])) ||
        kmipkit_client_batch_item_rekey_key_pair(
                0, 1, 86400, 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[14], strlen((const char *)batch_item_ids[14]))) {
        goto cleanup;
    }

    if (kmipkit_client_batch_item_rekey_key_pair(
            &identifiers[10], 1, 86400, 0, 0, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[15], strlen((const char *)batch_item_ids[15]))) {
        goto cleanup;
    }

    if (kmipkit_split_key_reference_create(&identifiers[9], 1, &split_key_reference) != 0 ||
        kmipkit_client_batch_item_join_split_key(
            7, &split_key_reference, 1, 1, UINT32_C(0x81234568),
            join_split_attributes, rekey_masks, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[16], strlen((const char *)batch_item_ids[16]))) {
        goto cleanup;
    }
    kmipkit_split_key_reference_release(split_key_reference);
    split_key_reference = 0;

    if (kmipkit_split_key_reference_create(&identifiers[9], 1, &split_key_reference) != 0 ||
        kmipkit_client_batch_item_join_split_key(
            7, &split_key_reference, 1, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[17], strlen((const char *)batch_item_ids[17]))) {
        goto cleanup;
    }
    kmipkit_split_key_reference_release(split_key_reference);
    split_key_reference = 0;

    if (kmipkit_client_batch_item_certify(
            &identifiers[11], 1, 1, &certify_request_value,
            certify_attributes, certify_masks, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[18], strlen((const char *)batch_item_ids[18])) ||
        kmipkit_client_batch_item_certify(
            &identifiers[12], 0, 0, 0, certify_attributes, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[19], strlen((const char *)batch_item_ids[19])) ||
        kmipkit_client_batch_item_certify(
            &identifiers[13], 1, 1, &certify_request_value, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[20], strlen((const char *)batch_item_ids[20])) ||
        kmipkit_client_batch_item_certify(
            &identifiers[14], 1, 1, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[21], strlen((const char *)batch_item_ids[21])) ||
        kmipkit_client_batch_item_recertify(
            &identifiers[15], 0, 1, 5, &recertify_request_value, 0, 0, 0, 0, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[22], strlen((const char *)batch_item_ids[22]))) {
        goto cleanup;
    }

    if (kmipkit_client_batch_item_import(
                &identifiers[16], 2, 0, 0, 1, UINT32_C(0xf1234568),
                import_attributes, wrapped_import_object, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[23], strlen((const char *)batch_item_ids[23])) ||
        kmipkit_client_batch_item_import(
                &identifiers[17], 2, 1, 0, 1, UINT32_C(0xf1234568),
                import_attributes, wrapped_import_object, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[24], strlen((const char *)batch_item_ids[24])) ||
        kmipkit_client_batch_item_import(
                &identifiers[18], 2, 1, 1, 1, UINT32_C(0xf1234568),
                import_attributes, wrapped_import_object, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[25], strlen((const char *)batch_item_ids[25])) ||
        kmipkit_client_batch_item_export(
                &identifiers[19], 0, 0, 0, 0, 0, 0, wrapping_specification, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[26], strlen((const char *)batch_item_ids[26])) ||
        kmipkit_client_batch_item_get(
                &identifiers[20], 0, 0, 0, 0, 0, 0, wrapping_specification, &item) != 0 ||
        !append_request_builder_item(
            batch, &item, batch_item_ids[27], strlen((const char *)batch_item_ids[27]))) {
        goto cleanup;
    }

    /* The fake peer supplies ID Placeholder context and success/failure batch results. */
    if (kmipkit_client_execute(client, &batch, limits, &response) != 0 || response == 0 ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != 28) {
        result = 22;
        goto cleanup;
    }
    for (uint64_t index = 18; index < 22; ++index) {
        kmipkit_client_batch_item_metadata_v2_t metadata = {0};
        metadata.struct_size = (uint32_t)sizeof(metadata);
        metadata.abi_version = 2;
        if (kmipkit_client_batch_response_item_at(response, index, &response_item) != 0 ||
            response_item == 0 ||
            kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata) != 0 ||
            metadata.operation != UINT32_C(6)) {
            result = 23;
            goto cleanup;
        }
        kmipkit_client_batch_item_response_release(response_item);
        response_item = 0;
    }
    kmipkit_client_batch_item_metadata_v2_t recertify_metadata = {0};
    recertify_metadata.struct_size = (uint32_t)sizeof(recertify_metadata);
    recertify_metadata.abi_version = 2;
    if (kmipkit_client_batch_response_item_at(response, 22, &response_item) != 0 ||
        response_item == 0 ||
        kmipkit_client_batch_item_response_metadata_v2(response_item, &recertify_metadata) != 0 ||
        recertify_metadata.operation != UINT32_C(7)) {
        result = 24;
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(response_item);
    response_item = 0;
    result = 0;

cleanup:
    if (response_item != 0) {
        kmipkit_client_batch_item_response_release(response_item);
    }
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(item);
    kmipkit_split_key_reference_release(split_key_reference);
    kmipkit_ttlv_structure_release(create_attributes);
    kmipkit_ttlv_structure_release(rekey_attributes);
    kmipkit_ttlv_structure_release(rekey_masks);
    kmipkit_ttlv_structure_release(recertify_attributes);
    kmipkit_ttlv_structure_release(join_split_attributes);
    kmipkit_ttlv_structure_release(certify_attributes);
    kmipkit_ttlv_structure_release(certify_masks);
    kmipkit_ttlv_structure_release(import_attributes);
    kmipkit_ttlv_structure_release(wrapped_import_object);
    kmipkit_ttlv_structure_release(wrapped_key_info);
    kmipkit_ttlv_structure_release(wrapped_key_data);
    kmipkit_ttlv_structure_release(wrapped_key_block);
    kmipkit_ttlv_structure_release(wrapped_key_block_holder);
    kmipkit_ttlv_structure_release(wrapping_specification);
    kmipkit_ttlv_structure_release(specification_key_info);
    kmipkit_ttlv_value_release(rekey_attribute_value);
    kmipkit_ttlv_value_release(rekey_mask_value);
    kmipkit_ttlv_value_release(recertify_attribute_value);
    kmipkit_ttlv_value_release(join_split_attribute_value);
    kmipkit_ttlv_value_release(certify_attribute_value);
    kmipkit_ttlv_value_release(certify_mask_value);
    kmipkit_ttlv_value_release(wrapping_value);
    kmipkit_ttlv_value_release(wrapped_key_value);
    return result;
}

static int execute_public_tls_request(
    int argc, char **argv, kmipkit_codec_limits_t *limits,
    int rekey_key_pair_only, int create_split_key_fixture_only, int cancel_payload_only,
    int request_builders_wire_only) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint32_t expected_statuses[] = {
        0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0};
    static const uint8_t expected_ids[28][6] = {
        {0},
        {0},
        {0},
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
        {'i', 'm', 'p', 'o', 'r', 't'},
        {'l', 'o', 'g', '-', '0', '0'},
        {'l', 'o', 'g', 'i', 'n', '0'},
        {'e', 'n', 'c', '-', '0', '0'},
        {'d', 'e', 'c', '-', '0', '0'},
        {'p', 'i', 'n', 'g', '0', '0'},
        {'q', 'u', 'e', 'r', 'y', '0'},
    };
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
    kmipkit_client_batch_item_t *items[28] = {0};
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_ttlv_structure_t *get_constraints_ttlv = 0;
    kmipkit_ttlv_structure_t *get_any_object = 0;
    kmipkit_ttlv_structure_t *query_response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *query_response_view = 0;
    uint8_t fixture_ids[3][64] = {{0}};
    uint64_t fixture_id_lengths[3] = {0};
    uint32_t fixture_statuses[3] = {0};
    uint8_t fixture_correlation[64] = {0};
    uint64_t fixture_correlation_length = 0;
    uint64_t fixture_item_count = 0;
    uint64_t fixture_pending_count = 0;
    uint64_t fixture_version_count = 0;
    uint64_t fixture_failure_message_index = 0;
    const uint8_t *fixture_failure_message = NULL;
    uint64_t fixture_failure_message_length = 0;
    kmipkit_protocol_version_v1_t fixture_versions[FIXTURE_VERSION_CAPACITY] = {{0}};
    uint64_t actual_pending_count = 0;
    uint64_t response_count = 0;
    char *port_end = 0;
    const unsigned long port = strtoul(argv[5], &port_end, 10);
    int result = 1;

    if (port_end == argv[5] || *port_end != '\0' || port == 0 || port > 65535UL) {
        return 2;
    }
    if (!rekey_key_pair_only && !create_split_key_fixture_only && !cancel_payload_only &&
        !request_builders_wire_only) {
        for (uint64_t index = 0; index < UINT64_C(3); ++index) {
            uint64_t parsed_status = 0;
            if (!parse_fixture_hex(
                    argv[6 + index], fixture_ids[index], sizeof(fixture_ids[index]),
                    &fixture_id_lengths[index]) ||
                !parse_fixture_decimal(argv[9 + index], &parsed_status) ||
                parsed_status > UINT32_MAX) {
                return 3;
            }
            fixture_statuses[index] = (uint32_t)parsed_status;
        }
        if (!parse_fixture_hex(
                argv[12], fixture_correlation, sizeof(fixture_correlation),
                &fixture_correlation_length) ||
            !parse_fixture_decimal(argv[13], &fixture_item_count) ||
            !parse_fixture_decimal(argv[14], &fixture_pending_count) ||
            !parse_fixture_decimal(argv[FIXTURE_VERSION_COUNT_ARG_INDEX], &fixture_version_count) ||
            fixture_item_count != UINT64_C(3) ||
            fixture_pending_count > fixture_item_count ||
            fixture_version_count > FIXTURE_VERSION_CAPACITY) {
            return 3;
        }
        fixture_failure_message_index =
            FIXTURE_VERSION_VALUES_ARG_INDEX + fixture_version_count * UINT64_C(2);
        if ((uint64_t)argc != fixture_failure_message_index + UINT64_C(1) ||
            argv[fixture_failure_message_index] == NULL) {
            return 3;
        }
        fixture_failure_message = (const uint8_t *)argv[fixture_failure_message_index];
        fixture_failure_message_length = (uint64_t)strlen((const char *)fixture_failure_message);
        for (uint64_t index = 0; index < fixture_version_count; ++index) {
            uint64_t major = 0;
            uint64_t minor = 0;
            if (!parse_fixture_decimal(argv[FIXTURE_VERSION_VALUES_ARG_INDEX + index * 2], &major) ||
                !parse_fixture_decimal(argv[FIXTURE_VERSION_VALUES_ARG_INDEX + index * 2 + 1], &minor) ||
                major > INT32_MAX || minor > INT32_MAX) {
                return 3;
            }
            fixture_versions[index].major = (int32_t)major;
            fixture_versions[index].minor = (int32_t)minor;
        }
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
    transport_input.max_request_bytes = UINT64_C(8388608);

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
    if (!rekey_key_pair_only && !create_split_key_fixture_only && !cancel_payload_only &&
        !request_builders_wire_only &&
        (!baseline_profile_fails_closed_before_c_transport_write(client, limits) ||
         !unknown_profile_fails_closed_before_c_transport_write(client, limits))) {
        result = 29;
        goto cleanup;
    }
    if (rekey_key_pair_only) {
        result = execute_rekey_key_pair_request_from_c(client, limits);
        goto cleanup;
    }
    if (create_split_key_fixture_only) {
        if (argc != 9 || argv[7] == 0 || argv[8] == 0) {
            result = 2;
            goto cleanup;
        }
        result = execute_create_split_key_fixture_request_from_c(
            client, limits, (const uint8_t *)argv[7], (uint64_t)strlen(argv[7]),
            (const uint8_t *)argv[8], (uint64_t)strlen(argv[8]));
        goto cleanup;
    }
    if (cancel_payload_only) {
        uint8_t correlation[64] = {0};
        uint64_t correlation_length = 0;
        if (argc != 8 || argv[7] == 0 ||
            !parse_fixture_hex(argv[7], correlation, sizeof(correlation), &correlation_length)) {
            result = 2;
            goto cleanup;
        }
        result = execute_cancel_payload_request_from_c(
            client, limits, correlation, correlation_length);
        goto cleanup;
    }
    if (request_builders_wire_only) {
        result = execute_request_builders_wire_from_c(client, limits);
        goto cleanup;
    }
    if (kmipkit_client_batch_create(&batch) != 0) {
        result = 30;
        goto cleanup;
    }
    if (kmipkit_client_batch_set_asynchronous_indicator(batch, 2) != 0) {
        result = 31;
        goto cleanup;
    }
    for (uint64_t index = 0; index < 28; ++index) {
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
            if (!create_key_pair_fixture_item(limits, &items[index])) {
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
            static const uint8_t wrapping_key_identifier[] = "wrapping-key";
            kmipkit_unique_identifier_v1_t identifier = {0};
            kmipkit_ttlv_structure_t *wrapping_specification = 0;
            kmipkit_ttlv_structure_t *key_information = 0;
            kmipkit_ttlv_value_t *value = 0;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier_bytes;
            identifier.text_length = (uint64_t)(sizeof(request_identifier_bytes) - 1);
            if (kmipkit_ttlv_structure_create(&wrapping_specification) != 0 ||
                kmipkit_ttlv_structure_create(&key_information) != 0 ||
                kmipkit_ttlv_value_enumeration(1, &value) != 0 ||
                !append_derive_key_parameter(
                    limits, &wrapping_specification, UINT32_C(0x42009E), &value) ||
                kmipkit_ttlv_value_text_string(
                    limits, wrapping_key_identifier,
                    (uint64_t)(sizeof(wrapping_key_identifier) - 1), &value) != 0 ||
                !append_derive_key_parameter(
                    limits, &key_information, UINT32_C(0x420094), &value) ||
                kmipkit_ttlv_value_structure(key_information, limits, &value) != 0) {
                kmipkit_ttlv_value_release(value);
                kmipkit_ttlv_structure_release(key_information);
                kmipkit_ttlv_structure_release(wrapping_specification);
                result = 40 + (int)index;
                goto cleanup;
            }
            key_information = 0;
            if (!append_derive_key_parameter(
                    limits, &wrapping_specification, UINT32_C(0x420036), &value) ||
                kmipkit_client_batch_item_export(
                    &identifier, 1, UINT32_C(0xfedcba98), 1, UINT32_C(0xfedcba99),
                    1, UINT32_C(0xfedcba9a), wrapping_specification, &items[index]) != 0 ||
                items[index] == 0) {
                kmipkit_ttlv_value_release(value);
                kmipkit_ttlv_structure_release(key_information);
                kmipkit_ttlv_structure_release(wrapping_specification);
                result = 40 + (int)index;
                goto cleanup;
            }
            kmipkit_ttlv_value_release(value);
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
        } else if (index == 21) {
            if (!create_import_request(limits, &items[index])) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 22) {
            static const uint8_t log_message[] = "c-log-message";
            if (kmipkit_client_batch_item_log(
                    log_message, sizeof(log_message) - 1, &items[index]) != 0 ||
                items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 23) {
            if (kmipkit_client_batch_item_login(0, 0, 0, 0, 0, &items[index]) != 0 ||
                items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 24 || index == 25) {
            static const uint8_t request_bytes[] = {0x41, 0x00, 0x42, 0xff};
            static const uint8_t iv_bytes[] = {0x11, 0x12};
            static const uint8_t tag_bytes[] = {0x31, 0x32};
            static const uint8_t aad_bytes[] = {0x51, 0x52};
            static const uint8_t request_identifier[] = "cryptographic-key";
            kmipkit_unique_identifier_v1_t identifier = {0};
            kmipkit_byte_span_v1_t iv = {iv_bytes, sizeof(iv_bytes)};
            kmipkit_byte_span_v1_t tag = {tag_bytes, sizeof(tag_bytes)};
            kmipkit_byte_span_v1_t aad = {aad_bytes, sizeof(aad_bytes)};
            kmipkit_ttlv_value_t *data = 0;
            identifier.struct_size = (uint32_t)sizeof(identifier);
            identifier.abi_version = 1;
            identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
            identifier.text_data = request_identifier;
            identifier.text_length = (uint64_t)(sizeof(request_identifier) - 1);
            if (kmipkit_ttlv_value_byte_string(
                    limits, request_bytes, sizeof(request_bytes), &data) != 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
            const int32_t constructor_status = index == 24
                ? kmipkit_client_batch_item_encrypt(
                    &identifier, 0, data, &iv, 0, 2, 2, &aad, &items[index])
                : kmipkit_client_batch_item_decrypt(
                    &identifier, 0, data, &iv, &tag, 0, 2, 2, &aad, &items[index]);
            kmipkit_ttlv_value_release(data);
            if (constructor_status != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 26) {
            if (kmipkit_client_batch_item_ping(&items[index]) != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (index == 27) {
            const uint64_t query_count = UINT64_C(100001);
            static const uint8_t group[] = "group-a";
            uint32_t *functions = (uint32_t *)malloc(
                (size_t)query_count * sizeof(*functions));
            kmipkit_byte_span_v1_t *groups = (kmipkit_byte_span_v1_t *)calloc(
                (size_t)query_count, sizeof(*groups));
            if (functions == 0 || groups == 0) {
                free(groups);
                free(functions);
                result = 40 + (int)index;
                goto cleanup;
            }
            for (uint64_t query_index = 0; query_index < query_count; ++query_index) {
                functions[query_index] = 5;
                groups[query_index].data = group;
                groups[query_index].length = sizeof(group) - 1;
            }
            const int32_t query_status = kmipkit_client_batch_item_query_with_limits(
                limits, functions, query_count, 1, groups, query_count, &items[index]);
            free(groups);
            free(functions);
            if (query_status != 0 || items[index] == 0) {
                result = 40 + (int)index;
                goto cleanup;
            }
        } else if (kmipkit_client_batch_item_discover_versions(&items[index]) != 0 ||
                   items[index] == 0) {
            result = 40 + (int)index;
            goto cleanup;
        }
        const uint8_t *batch_item_id = index < fixture_item_count
            ? fixture_ids[index]
            : expected_ids[index];
        const uint64_t batch_item_id_length = index < fixture_item_count
            ? fixture_id_lengths[index]
            : sizeof(expected_ids[index]);
        if (kmipkit_client_batch_item_with_unique_batch_item_id(
                items[index], batch_item_id, batch_item_id_length, &identified_item) != 0 ||
            identified_item == 0) {
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
        response_count != 28) {
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
        kmipkit_protocol_version_v1_t versions[FIXTURE_VERSION_CAPACITY] = {{0}};
        const uint32_t expected_status = index < fixture_item_count
            ? fixture_statuses[index]
            : expected_statuses[index];
        const uint32_t expected_outcome = expected_status == 2U ? 2U : 1U;
        if (kmipkit_client_batch_response_item_at(response, index, &response_item) != 0 ||
            response_item == 0) {
            goto cleanup;
        }
        metadata.struct_size = (uint32_t)sizeof(metadata);
        metadata.abi_version = 1;
        if (kmipkit_client_batch_item_response_metadata_v1(response_item, &metadata) != 0 ||
            metadata.abi_version != 1 || metadata.outcome_kind != expected_outcome ||
            metadata.result_status != expected_status) {
            result = 210 + (int)index;
            goto cleanup;
        }
        if (metadata.outcome_kind == 2U) {
            ++actual_pending_count;
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
                metadata_v2.result_message_length != fixture_failure_message_length ||
                metadata_v2.result_message_data == 0 ||
                memcmp(metadata_v2.result_message_data, fixture_failure_message,
                    fixture_failure_message_length) != 0 ||
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
        if ((index == 24 || index == 25) &&
            !encrypt_decrypt_response_is_readable_from_c(response, index, &response_item)) {
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
            kmipkit_ttlv_structure_t *saved_get_any_object = get_any_object;
            present = 0xff;
            payload_present = 0xff;
            object_type = UINT32_MAX;
            get_any_object = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
            if (kmipkit_client_batch_item_response_get(
                    response_item, &present, &payload_present, &object_type,
                    NULL, &get_any_object) != T076_ERROR_INVALID_INPUT ||
                present != 0 || payload_present != 0 || object_type != 0 ||
                get_any_object != 0) {
                result = 235;
                goto cleanup;
            }
            get_any_object = saved_get_any_object;
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
            kmipkit_unique_identifier_v1_t out_of_range_identifier = {0};
            out_of_range_identifier.struct_size =
                (uint32_t)sizeof(out_of_range_identifier);
            out_of_range_identifier.abi_version = 1;
            if (kmipkit_client_batch_item_response_locate_unique_identifier_at(
                    response_item, identifier_count, &out_of_range_identifier) !=
                    T076_ERROR_INVALID_INPUT ||
                out_of_range_identifier.kind != 0 ||
                out_of_range_identifier.text_data != 0 ||
                out_of_range_identifier.text_length != 0) {
                result = 236;
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_locate_unique_identifier_at(
                    response_item, 0, NULL) != T076_ERROR_INVALID_INPUT) {
                result = 237;
                goto cleanup;
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
        const uint8_t *expected_item_id = index < fixture_item_count
            ? fixture_ids[index]
            : expected_ids[index];
        const uint64_t expected_item_id_length = index < fixture_item_count
            ? fixture_id_lengths[index]
            : sizeof(expected_ids[index]);
        if (kmipkit_client_batch_item_response_unique_batch_item_id(
                response_item, &unique_id_present, &unique_id) != 0 ||
            unique_id_present != 1 || unique_id.length != expected_item_id_length ||
            unique_id.data == 0 ||
            memcmp(unique_id.data, expected_item_id, (size_t)expected_item_id_length) != 0) {
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
        if (index < fixture_item_count && fixture_statuses[index] == 2U) {
            if (correlation_present != 1 ||
                correlation.length != fixture_correlation_length || correlation.data == 0 ||
                memcmp(correlation.data, fixture_correlation,
                    (size_t)fixture_correlation_length) != 0) {
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
            if (versions_present != 1 || version_count != fixture_version_count) {
                goto cleanup;
            }
            uint64_t copied_count = 0;
            uint8_t insufficient_present = 0;
            uint64_t insufficient_count = 0;
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &insufficient_present, versions, 1, &insufficient_count) !=
                    T076_ERROR_INVALID_INPUT ||
                insufficient_present != 1 || insufficient_count != fixture_version_count) {
                goto cleanup;
            }
            if (kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    response_item, &versions_present, versions, fixture_version_count, &copied_count) != 0 ||
                versions_present != 1 || copied_count != fixture_version_count) {
                goto cleanup;
            }
            for (uint64_t version_index = 0; version_index < fixture_version_count; ++version_index) {
                if (versions[version_index].major != fixture_versions[version_index].major ||
                    versions[version_index].minor != fixture_versions[version_index].minor) {
                    goto cleanup;
                }
            }
        } else if (index == 3) {
            if (versions_present != 1 || version_count != 0) {
                goto cleanup;
            }
        } else if (versions_present != 0 || version_count != 0) {
            goto cleanup;
        }
        if (index == 26) {
            uint8_t present = 0;
            if (kmipkit_client_batch_item_response_ping(response_item, &present) != 0 ||
                present != 1) {
                result = 253;
                goto cleanup;
            }
            kmipkit_client_batch_item_response_release(response_item);
            response_item = 0;
        } else if (index == 27) {
            uint8_t present = 0;
            if (kmipkit_client_batch_item_response_query_ttlv(
                    response_item, &present, &query_response_ttlv) != 0 ||
                present != 1 || query_response_ttlv == 0) {
                result = 254;
                goto cleanup;
            }
            kmipkit_client_batch_item_response_release(response_item);
            response_item = 0;
        } else if (index == 21) {
            if (!import_response_is_readable_from_c(response_item)) {
                goto cleanup;
            }
            response_item = 0; /* the Import ownership check releases the borrowed item */
        } else if (index == 22) {
            if (!log_response_is_readable_after_result_release_from_c(response_item)) {
                goto cleanup;
            }
            response_item = 0; /* the Log ownership check releases the borrowed item */
        } else if (index == 23) {
            if (!login_response_is_readable_after_result_release_from_c(response_item)) {
                goto cleanup;
            }
            response_item = 0; /* the Login ownership check releases the borrowed item */
        } else {
            kmipkit_client_batch_item_response_release(response_item);
            response_item = 0;
        }
    }
    if (actual_pending_count != fixture_pending_count) {
        result = 256;
        goto cleanup;
    }
    if (!public_ttlv_contains_response_sentinel(get_any_object)) {
        result = 252;
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(get_any_object);
    get_any_object = 0;
    kmipkit_client_batch_response_release(response);
    response = 0;
    static const uint32_t expected_query_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x42015f), UINT32_C(0x420012)};
    if (kmipkit_ttlv_structure_view(query_response_ttlv, &query_response_view) != 0 ||
        !check_view_tags(
            query_response_view, expected_query_tags,
            sizeof(expected_query_tags) / sizeof(expected_query_tags[0]))) {
        result = 255;
        goto cleanup;
    }
    kmipkit_ttlv_structure_view_release(query_response_view);
    query_response_view = 0;
    kmipkit_ttlv_structure_release(query_response_ttlv);
    query_response_ttlv = 0;
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
    kmipkit_ttlv_structure_view_release(query_response_view);
    kmipkit_ttlv_structure_release(query_response_ttlv);
    for (uint64_t index = 0; index < 28; ++index) {
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

static int import_response_is_readable_from_c(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "imported-id";
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    uint8_t identifier_present = 0;
    uint8_t ttlv_present = 0;
    int identifier_matches = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_import(
            item, &identifier_present, &identifier) != 0 || identifier_present != 1 ||
        kmipkit_client_batch_item_response_import_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    identifier_matches = response_identifier_matches(
        &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1));
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (!identifier_matches || kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        !public_ttlv_view_contains_response_sentinel(response_view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int shared_ttlv_fixture_preserves_open_values_and_order(
    const uint32_t *raw_tags,
    uint32_t enumeration_value,
    const uint8_t *byte_string,
    uint64_t byte_string_length) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_view_t *structure_view = 0;
    uint64_t item_count = 0;
    int result = 1;

    if (raw_tags == 0 || byte_string == 0 || byte_string_length == 0 ||
        kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_structure_create(&structure) != 0) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < UINT64_C(2); ++index) {
        kmipkit_raw_tag_t *raw_tag = 0;
        kmipkit_tag_t *tag = 0;
        kmipkit_ttlv_value_t *value = 0;
        kmipkit_ttlv_item_t *item = 0;
        kmipkit_ttlv_structure_t *updated = 0;
        int valid = kmipkit_ttlv_raw_tag_create(raw_tags[index], &raw_tag) == 0 &&
            kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) == 0;
        if (valid && index == 0) {
            valid = kmipkit_ttlv_value_enumeration(enumeration_value, &value) == 0;
        } else if (valid) {
            valid = kmipkit_ttlv_value_byte_string(
                limits, byte_string, byte_string_length, &value) == 0;
        }
        if (valid) {
            kmipkit_ttlv_value_t *transferred_value = value;
            value = 0;
            valid = kmipkit_ttlv_item_create(tag, transferred_value, limits, &item) == 0;
        }
        if (valid) {
            kmipkit_ttlv_structure_t *transferred_structure = structure;
            kmipkit_ttlv_item_t *transferred_item = item;
            structure = 0;
            item = 0;
            valid = kmipkit_ttlv_structure_with_item(
                transferred_structure, transferred_item, limits, &updated) == 0;
            structure = updated;
            updated = 0;
        }
        kmipkit_ttlv_structure_release(updated);
        kmipkit_ttlv_item_release(item);
        kmipkit_ttlv_value_release(value);
        kmipkit_tag_release(tag);
        kmipkit_raw_tag_release(raw_tag);
        if (!valid) {
            goto cleanup;
        }
    }
    if (kmipkit_ttlv_structure_view(structure, &structure_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(structure_view, &item_count) != 0 ||
        item_count != UINT64_C(2)) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < item_count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = 0;
        kmipkit_tag_t *tag = 0;
        kmipkit_ttlv_value_view_t *value_view = 0;
        uint32_t actual_tag = 0;
        uint8_t item_type = 0;
        int valid = kmipkit_ttlv_structure_view_item_at(
                structure_view, index, &item_view) == 0 &&
            kmipkit_ttlv_item_view_tag(item_view, &tag) == 0 &&
            kmipkit_ttlv_tag_value(tag, &actual_tag) == 0 && actual_tag == raw_tags[index] &&
            kmipkit_ttlv_item_view_type(item_view, &item_type) == 0 &&
            kmipkit_ttlv_item_view_value(item_view, &value_view) == 0;
        if (valid && index == 0) {
            uint32_t actual_enumeration = 0;
            valid = item_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION &&
                kmipkit_ttlv_value_view_enumeration(value_view, &actual_enumeration) == 0 &&
                actual_enumeration == enumeration_value;
        } else if (valid) {
            uint64_t actual_length = 0;
            valid = item_type == KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING &&
                kmipkit_ttlv_value_view_byte_length(value_view, &actual_length) == 0 &&
                actual_length == byte_string_length;
            for (uint64_t byte_index = 0; valid && byte_index < actual_length; ++byte_index) {
                uint8_t actual_byte = 0;
                valid = kmipkit_ttlv_value_view_byte_at(
                            value_view, byte_index, &actual_byte) == 0 &&
                    actual_byte == byte_string[byte_index];
            }
        }
        kmipkit_ttlv_value_view_release(value_view);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item_view);
        if (!valid) {
            goto cleanup;
        }
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_codec_limits_release(limits);
    return result;
}

int codec_limits_shared_fixture_matches_defaults_ranges_and_errors(
    uint64_t expected_default_message_bytes,
    uint64_t expected_default_structure_depth,
    uint64_t expected_default_elements,
    uint64_t configured_message_bytes,
    uint64_t configured_structure_depth,
    uint64_t configured_elements,
    uint64_t invalid_structure_depth) {
    kmipkit_codec_limits_t *defaults = 0;
    kmipkit_codec_limits_t *configured = 0;
    kmipkit_codec_limits_t *invalid = 0;
    uint64_t actual_message_bytes = 0;
    uint64_t actual_structure_depth = 0;
    uint64_t actual_elements = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&defaults) != 0 || defaults == 0 ||
        kmipkit_codec_limits_max_message_bytes(defaults, &actual_message_bytes) != 0 ||
        actual_message_bytes != expected_default_message_bytes ||
        kmipkit_codec_limits_max_structure_depth(defaults, &actual_structure_depth) != 0 ||
        actual_structure_depth != expected_default_structure_depth ||
        kmipkit_codec_limits_max_elements(defaults, &actual_elements) != 0 ||
        actual_elements != expected_default_elements) {
        goto cleanup;
    }
    kmipkit_codec_limits_release(defaults);
    defaults = 0;

    if (kmipkit_codec_limits_create(
            configured_message_bytes,
            configured_structure_depth,
            configured_elements,
            &configured) != 0 || configured == 0) {
        goto cleanup;
    }
    actual_message_bytes = 0;
    actual_structure_depth = 0;
    actual_elements = 0;
    if (kmipkit_codec_limits_max_message_bytes(configured, &actual_message_bytes) != 0 ||
        actual_message_bytes != configured_message_bytes ||
        kmipkit_codec_limits_max_structure_depth(configured, &actual_structure_depth) != 0 ||
        actual_structure_depth != configured_structure_depth ||
        kmipkit_codec_limits_max_elements(configured, &actual_elements) != 0 ||
        actual_elements != configured_elements) {
        goto cleanup;
    }
    kmipkit_codec_limits_release(configured);
    configured = 0;

    const int32_t invalid_status = kmipkit_codec_limits_create(
        expected_default_message_bytes,
        invalid_structure_depth,
        expected_default_elements,
        &invalid);
    result = invalid_status == KMIPKIT_ERROR_RESOURCE_LIMIT && invalid == 0;

cleanup:
    kmipkit_codec_limits_release(invalid);
    kmipkit_codec_limits_release(configured);
    kmipkit_codec_limits_release(defaults);
    return result;
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

int kmipkit_client_abi_c_consumer_validate_usage_allocation_context(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "allocation-object";
    kmipkit_usage_allocation_context_t *context = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint32_t unit = 0;
    int64_t allocated = 0;
    int64_t available = 0;
    int64_t reserved = 0;
    int result = 0;

    if (item == 0 || kmipkit_usage_allocation_context_create(
            item, 12, UINT32_MAX, &context) != 0 || context == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_usage_allocation_context_usage_limits_unit(context, &unit) != 0 ||
        kmipkit_usage_allocation_context_allocated_units(context, &allocated) != 0 ||
        kmipkit_usage_allocation_context_available_units(context, &available) != 0 ||
        kmipkit_usage_allocation_context_reserved_units(context, &reserved) != 0 ||
        kmipkit_usage_allocation_context_unique_identifier(context, &identifier) != 0 ||
        unit != UINT32_MAX || allocated != 12 || available != 12 || reserved != 0 ||
        !response_identifier_matches(
            &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_usage_allocation_context_release(context);
    return result;
}

static int usage_allocation_context_matches(
    kmipkit_usage_allocation_context_t *context,
    const uint8_t *identifier_data,
    uint64_t identifier_length,
    uint32_t expected_unit,
    int64_t expected_allocated,
    int64_t expected_reserved,
    int64_t expected_available) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint32_t unit = 0;
    int64_t allocated = 0;
    int64_t reserved = 0;
    int64_t available = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    return kmipkit_usage_allocation_context_usage_limits_unit(context, &unit) == 0 &&
        kmipkit_usage_allocation_context_allocated_units(context, &allocated) == 0 &&
        kmipkit_usage_allocation_context_reserved_units(context, &reserved) == 0 &&
        kmipkit_usage_allocation_context_available_units(context, &available) == 0 &&
        kmipkit_usage_allocation_context_unique_identifier(context, &identifier) == 0 &&
        unit == expected_unit && allocated == expected_allocated &&
        reserved == expected_reserved && available == expected_available &&
        response_identifier_matches(&identifier, identifier_data, identifier_length);
}

int shared_usage_allocation_guard_matches_fixture(
    kmipkit_client_t *client,
    const uint8_t *identifier_data,
    uint64_t identifier_length,
    int64_t allocation_count,
    uint32_t usage_limits_unit,
    int64_t successful_cost,
    int64_t over_budget_cost,
    int64_t expected_initial_allocated,
    int64_t expected_initial_reserved,
    int64_t expected_initial_available,
    int64_t expected_allocated,
    int64_t expected_reserved,
    int64_t expected_available,
    kmipkit_client_batch_item_t **item,
    kmipkit_codec_limits_t *limits) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_client_batch_item_t *allocation_item = 0;
    kmipkit_client_batch_t *allocation_batch = 0;
    kmipkit_client_batch_response_t *allocation_response = 0;
    kmipkit_client_batch_item_response_t *allocation_response_item = 0;
    kmipkit_usage_allocation_context_t *context = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    kmipkit_client_batch_item_t *excess_item = 0;
    kmipkit_client_batch_response_t *excess_response = 0;
    uint64_t response_count = 0;
    int result = 0;

    if (client == 0 || identifier_data == 0 || identifier_length == 0 || allocation_count <= 0 ||
        successful_cost <= 0 || over_budget_cost <= 0 || item == 0 || *item == 0 || limits == 0) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_data;
    identifier.text_length = identifier_length;
    if (kmipkit_client_batch_item_get_usage_allocation(
            &identifier, allocation_count, &allocation_item) != 0 || allocation_item == 0 ||
        kmipkit_client_batch_create(&allocation_batch) != 0 || allocation_batch == 0 ||
        kmipkit_client_batch_add_item(allocation_batch, &allocation_item) != 0 ||
        allocation_item != 0) {
        goto cleanup;
    }
    const int32_t allocation_status =
        kmipkit_client_execute(client, &allocation_batch, limits, &allocation_response);
    if (allocation_status != 0) {
        goto cleanup;
    }
    if (allocation_batch != 0 || allocation_response == 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_response_item_count(allocation_response, &response_count) != 0 ||
        response_count != UINT64_C(1)) {
        goto cleanup;
    }
    if (kmipkit_client_batch_response_item_at(
            allocation_response, 0, &allocation_response_item) != 0 ||
        allocation_response_item == 0) {
        goto cleanup;
    }
    if (kmipkit_usage_allocation_context_create(
            allocation_response_item, allocation_count, usage_limits_unit, &context) != 0 ||
        context == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(allocation_response_item);
    allocation_response_item = 0;
    kmipkit_client_batch_response_release(allocation_response);
    allocation_response = 0;
    if (!usage_allocation_context_matches(
            context, identifier_data, identifier_length, usage_limits_unit,
            expected_initial_allocated, expected_initial_reserved, expected_initial_available)) {
        goto cleanup;
    }
    const int32_t execute_status = kmipkit_client_execute_guarded_protection_item(
        client, context, item, successful_cost, limits, &response);
    if (execute_status != 0) {
        goto cleanup;
    }
    if (*item != 0 || response == 0 ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != UINT64_C(1) ||
        kmipkit_client_batch_response_item_at(response, 0, &response_item) != 0 ||
        response_item == 0) {
        goto cleanup;
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    if (kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata) != 0 ||
        metadata.operation != UINT32_C(0x1f) || metadata.result_status != 0 ||
        !usage_allocation_context_matches(
            context, identifier_data, identifier_length, usage_limits_unit,
            expected_allocated, expected_reserved, expected_available)) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(response_item);
    response_item = 0;
    kmipkit_client_batch_response_release(response);
    response = 0;

    if (kmipkit_client_batch_item_encrypt(
            &identifier, 0, 0, 0, 0, 0, 0, 0, &excess_item) != 0 || excess_item == 0 ||
        kmipkit_client_execute_guarded_protection_item(
            client, context, &excess_item, over_budget_cost, limits, &excess_response) !=
            T076_ERROR_INVALID_INPUT ||
        excess_item != 0 || excess_response != 0 ||
        !usage_allocation_context_matches(
            context, identifier_data, identifier_length, usage_limits_unit,
            expected_allocated, expected_reserved, expected_available)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(excess_item);
    kmipkit_client_batch_response_release(excess_response);
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_usage_allocation_context_release(context);
    kmipkit_client_batch_item_response_release(allocation_response_item);
    kmipkit_client_batch_response_release(allocation_response);
    kmipkit_client_batch_release(allocation_batch);
    kmipkit_client_batch_item_release(allocation_item);
    if (item != 0) {
        kmipkit_client_batch_item_release(*item);
        *item = 0;
    }
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

int kmipkit_client_abi_c_consumer_validate_recertify_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "replacement-certificate";
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t identifier_present = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    int identifier_matches = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_recertify(
            item, &identifier_present, &identifier) != 0 || identifier_present != 1 ||
        kmipkit_client_batch_item_response_recertify_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    identifier_matches = response_identifier_matches(
        &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1));
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (!identifier_matches) {
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0) {
        goto cleanup;
    }
    if (!check_view_tags(
            view, expected_tags, (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0])))) {
        goto cleanup;
    }
    if (!public_ttlv_view_contains_response_sentinel(view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_rekey_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "replacement-key";
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t identifier_present = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    int identifier_matches = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_rekey(
            item, &identifier_present, &identifier) != 0 || identifier_present != 1 ||
        kmipkit_client_batch_item_response_rekey_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    identifier_matches = response_identifier_matches(
        &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1));
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (!identifier_matches || kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        !check_view_tags(
            view, expected_tags, (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0]))) ||
        !public_ttlv_view_contains_response_sentinel(view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_rekey_key_pair_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_private[] = "replacement-private-key";
    static const uint8_t expected_public[] = "replacement-public-key";
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {
        UINT32_C(0x420066), UINT32_C(0x42006f), UINT32_C(0x540001)};
    uint8_t present = 0, ttlv_present = 0, payload_type = 0;
    kmipkit_unique_identifier_v1_t private_identifier = {0}, public_identifier = {0};
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    int result = 0;
    if (item == 0) return 0;
    private_identifier.struct_size = (uint32_t)sizeof(private_identifier);
    private_identifier.abi_version = 1;
    public_identifier.struct_size = (uint32_t)sizeof(public_identifier);
    public_identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_rekey_key_pair(item, &present,
            &private_identifier, &public_identifier) != 0 || present != 1 ||
        !response_identifier_matches(&private_identifier, expected_private,
                (uint64_t)(sizeof(expected_private) - 1)) ||
        !response_identifier_matches(&public_identifier, expected_public,
                (uint64_t)(sizeof(expected_public) - 1)) ||
        kmipkit_client_batch_item_response_rekey_key_pair_ttlv(item, &ttlv_present,
                &response_ttlv) != 0 || ttlv_present != 1 || response_ttlv == 0) goto cleanup;
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0) goto cleanup;
    if (!check_view_tags(view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(view, 3, &payload_item) != 0 ||
        payload_item == 0 ||
        kmipkit_ttlv_item_view_type(payload_item, &payload_type) != 0 ||
        payload_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0]))) ||
        !public_ttlv_view_contains_response_sentinel(payload_view, 0)) goto cleanup;
    result = 1;
cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_reprovision_response(
    kmipkit_client_batch_item_response_t *item,
    uint8_t expect_success,
    uint8_t expected_identifier_present,
    const uint8_t *expected_identifier,
    uint64_t expected_identifier_length) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = 0;
    uint8_t identifier_present = 0;
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t item_count = 0;
    int result = 0;

    if (item == 0 || expect_success > 1 || expected_identifier_present > 1 ||
        (!expect_success && expected_identifier_present != 0) ||
        (expected_identifier_present != 0 && expected_identifier == 0) ||
        (expected_identifier_present == 0 &&
            (expected_identifier != 0 || expected_identifier_length != 0))) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_response_reprovision(
            item, (uint8_t *)(uintptr_t)UINTPTR_MAX, &identifier_present, &identifier) !=
        T076_ERROR_INVALID_INPUT) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_response_reprovision(
            item, &present, (uint8_t *)(uintptr_t)UINTPTR_MAX, &identifier) !=
        T076_ERROR_INVALID_INPUT) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_reprovision(
            item, &present, &identifier_present, &identifier) != 0) {
        goto cleanup;
    }
    if (expect_success == 0) {
        if (present != 0 || identifier_present != 0 || identifier.kind != 0 ||
            identifier.text_data != 0 || identifier.text_length != 0 ||
            identifier.enumeration_value != 0 || identifier.integer_value != 0 ||
            kmipkit_client_batch_item_response_reprovision_ttlv(
                item, &ttlv_present, &response_ttlv) != 0 ||
            ttlv_present != 0 || response_ttlv != 0) {
            goto cleanup;
        }
        result = 1;
        goto cleanup;
    }
    if (present != 1 || identifier_present != expected_identifier_present) {
        goto cleanup;
    }
    if (expected_identifier_present != 0) {
        if (!response_identifier_matches(
                &identifier, expected_identifier, expected_identifier_length)) {
            goto cleanup;
        }
    } else if (identifier.kind != 0 || identifier.text_data != 0 ||
        identifier.text_length != 0 || identifier.enumeration_value != 0 ||
        identifier.integer_value != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_response_reprovision_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &item_count) != 0 || item_count != 4) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_join_split_key_response_ttlv(
    kmipkit_client_batch_item_response_t *item) {
    static const uint8_t expected_identifier[] = "joined-id";
    static const uint32_t expected_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t identifier_present = 0;
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t count = 0;
    int result = 0;

    if (item == 0) {
        return 0;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (kmipkit_client_batch_item_response_join_split_key(
            item, &identifier_present, &identifier) != 0 || identifier_present != 1 ||
        !response_identifier_matches(
            &identifier, expected_identifier, (uint64_t)(sizeof(expected_identifier) - 1)) ||
        kmipkit_client_batch_item_response_join_split_key_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 || ttlv_present != 1 ||
        response_ttlv == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 ||
        count != (uint64_t)(sizeof(expected_tags) / sizeof(expected_tags[0])) ||
        !check_view_tags(view, expected_tags, count) ||
        !public_ttlv_view_contains_response_sentinel(view, 0)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_execute_error_ttlv_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint8_t key_identifier_text[] = "mac-key";
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
    kmipkit_unique_identifier_v1_t key_identifier = {0};
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
    key_identifier.struct_size = (uint32_t)sizeof(key_identifier);
    key_identifier.abi_version = 1;
    key_identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    key_identifier.text_data = key_identifier_text;
    key_identifier.text_length = (uint64_t)(sizeof(key_identifier_text) - 1);
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
        kmipkit_client_batch_item_mac(&key_identifier, 0, mac_data, 0, 0, 0, &item) != 0 ||
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

static int append_pkcs11_ttlv_value(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **structure,
    uint32_t raw_tag_value,
    kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *updated = 0;
    int result = 0;

    if (limits == 0 || structure == 0 || *structure == 0 || value == 0 || *value == 0) {
        return 0;
    }
    if (kmipkit_ttlv_raw_tag_create(raw_tag_value, &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, *value, limits, &item) != 0) {
        goto cleanup;
    }
    *value = 0;
    if (kmipkit_ttlv_structure_with_item(*structure, item, limits, &updated) != 0) {
        goto cleanup;
    }
    *structure = 0;
    item = 0;
    *structure = updated;
    updated = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    return result;
}

static int pkcs11_view_item_tag(
    kmipkit_ttlv_structure_view_t *view, uint64_t index, uint32_t *out_tag) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_tag_t *tag = 0;
    const int valid = kmipkit_ttlv_structure_view_item_at(view, index, &item) == 0 &&
        item != 0 && kmipkit_ttlv_item_view_tag(item, &tag) == 0 && tag != 0 &&
        kmipkit_ttlv_tag_value(tag, out_tag) == 0;
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return valid;
}

static int pkcs11_view_contains_tag(kmipkit_ttlv_structure_view_t *view, uint32_t expected) {
    uint64_t count = 0;
    if (kmipkit_ttlv_structure_view_item_count(view, &count) != 0) {
        return 0;
    }
    for (uint64_t index = 0; index < count; ++index) {
        uint32_t actual = 0;
        if (!pkcs11_view_item_tag(view, index, &actual)) {
            return 0;
        }
        if (actual == expected) {
            return 1;
        }
    }
    return 0;
}

static int pkcs11_bytes_match(
    kmipkit_ttlv_value_view_t *value,
    const uint8_t *expected,
    uint64_t expected_length) {
    uint8_t type = 0;
    uint64_t actual_length = 0;
    if (value == 0 || expected == 0 ||
        kmipkit_ttlv_value_view_type(value, &type) != 0 ||
        type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
        kmipkit_ttlv_value_view_byte_length(value, &actual_length) != 0 ||
        actual_length != expected_length) {
        return 0;
    }
    for (uint64_t index = 0; index < expected_length; ++index) {
        uint8_t actual = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, index, &actual) != 0 ||
            actual != expected[index]) {
            return 0;
        }
    }
    return 1;
}

static int async_item_value_at(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    uint8_t expected_type,
    kmipkit_ttlv_item_view_t **out_item,
    kmipkit_ttlv_value_view_t **out_value) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t actual_tag = 0;
    uint8_t actual_type = 0;
    if (view == 0 || out_item == 0 || out_value == 0 ||
        kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0 || item == 0 ||
        !pkcs11_view_item_tag(view, index, &actual_tag) || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_type(item, &actual_type) != 0 || actual_type != expected_type ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 || value == 0) {
        kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        return 0;
    }
    *out_item = item;
    *out_value = value;
    return 1;
}

static int async_text_value_matches(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    const uint8_t *expected,
    uint64_t expected_length) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint64_t actual_length = 0;
    int matches = 0;
    if (expected == 0 || !async_item_value_at(
            view, index, expected_tag, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, &item, &value) ||
        kmipkit_ttlv_value_view_byte_length(value, &actual_length) != 0 ||
        actual_length != expected_length) {
        goto cleanup;
    }
    for (uint64_t byte_index = 0; byte_index < expected_length; ++byte_index) {
        uint8_t actual = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, byte_index, &actual) != 0 ||
            actual != expected[byte_index]) {
            goto cleanup;
        }
    }
    matches = 1;
cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return matches;
}

static int async_boolean_value_matches(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    uint8_t expected) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint8_t actual = UINT8_MAX;
    const int matches = async_item_value_at(
            view, index, expected_tag, KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN, &item, &value) &&
        kmipkit_ttlv_value_view_boolean(value, &actual) == 0 && actual == expected;
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return matches;
}

static int async_enumeration_value_matches(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    uint32_t expected) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t actual = 0;
    const int matches = async_item_value_at(
            view, index, expected_tag, KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, &item, &value) &&
        kmipkit_ttlv_value_view_enumeration(value, &actual) == 0 && actual == expected;
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return matches;
}

static int async_nested_structure_view(
    kmipkit_ttlv_structure_view_t *view,
    uint64_t index,
    uint32_t expected_tag,
    kmipkit_ttlv_structure_view_t **out_nested) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    kmipkit_ttlv_structure_view_t *nested = 0;
    if (out_nested == 0 || !async_item_value_at(
            view, index, expected_tag, KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &item, &value) ||
        kmipkit_ttlv_value_view_structure(value, &nested) != 0 || nested == 0) {
        kmipkit_ttlv_structure_view_release(nested);
        kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        return 0;
    }
    *out_nested = nested;
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return 1;
}

static int async_response_extension_matches(kmipkit_ttlv_structure_t *extension) {
    static const uint8_t expected_vendor[] = "OpaqueVendor";
    static const uint8_t expected_text[] = "opaque-v1";
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_view_t *vendor_view = 0;
    uint64_t count = 0;
    int matches = 0;
    if (extension == 0 || kmipkit_ttlv_structure_view(extension, &view) != 0 || view == 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != UINT64_C(3) ||
        !async_text_value_matches(
            view, UINT64_C(0), UINT32_C(0x42009d), expected_vendor,
            (uint64_t)(sizeof(expected_vendor) - 1)) ||
        !async_boolean_value_matches(view, UINT64_C(1), UINT32_C(0x420026), UINT8_C(0)) ||
        !async_nested_structure_view(view, UINT64_C(2), UINT32_C(0x42009c), &vendor_view) ||
        kmipkit_ttlv_structure_view_item_count(vendor_view, &count) != 0 ||
        count != UINT64_C(2) ||
        !async_text_value_matches(
            vendor_view, UINT64_C(0), UINT32_C(0x540173), expected_text,
            (uint64_t)(sizeof(expected_text) - 1)) ||
        !async_enumeration_value_matches(
            vendor_view, UINT64_C(1), UINT32_C(0x540174), UINT32_MAX)) {
        goto cleanup;
    }
    matches = 1;
cleanup:
    kmipkit_ttlv_structure_view_release(vendor_view);
    kmipkit_ttlv_structure_view_release(view);
    return matches;
}

static int pkcs11_nested_integer_matches(
    kmipkit_ttlv_structure_view_t *parent,
    uint64_t index,
    uint32_t expected_tag,
    int32_t expected_value) {
    kmipkit_ttlv_item_view_t *outer_item = 0;
    kmipkit_ttlv_value_view_t *outer_value = 0;
    kmipkit_ttlv_structure_view_t *nested = 0;
    kmipkit_ttlv_item_view_t *inner_item = 0;
    kmipkit_ttlv_value_view_t *inner_value = 0;
    kmipkit_tag_t *outer_tag = 0;
    kmipkit_tag_t *inner_tag = 0;
    uint32_t actual_outer_tag = 0;
    uint32_t actual_inner_tag = 0;
    uint64_t count = 0;
    uint8_t type = 0;
    int32_t actual_value = 0;
    int result = 0;
    if (parent == 0 ||
        kmipkit_ttlv_structure_view_item_at(parent, index, &outer_item) != 0 ||
        kmipkit_ttlv_item_view_tag(outer_item, &outer_tag) != 0 ||
        kmipkit_ttlv_tag_value(outer_tag, &actual_outer_tag) != 0 ||
        actual_outer_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(outer_item, &outer_value) != 0 ||
        kmipkit_ttlv_value_view_structure(outer_value, &nested) != 0 ||
        kmipkit_ttlv_structure_view_item_count(nested, &count) != 0 || count != 1 ||
        kmipkit_ttlv_structure_view_item_at(nested, 0, &inner_item) != 0 ||
        kmipkit_ttlv_item_view_tag(inner_item, &inner_tag) != 0 ||
        kmipkit_ttlv_tag_value(inner_tag, &actual_inner_tag) != 0 ||
        actual_inner_tag != expected_tag ||
        kmipkit_ttlv_item_view_type(inner_item, &type) != 0 ||
        type != KMIPKIT_TTLV_ITEM_TYPE_INTEGER ||
        kmipkit_ttlv_item_view_value(inner_item, &inner_value) != 0 ||
        kmipkit_ttlv_value_view_integer(inner_value, &actual_value) != 0 ||
        actual_value != expected_value) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_tag_release(inner_tag);
    kmipkit_tag_release(outer_tag);
    kmipkit_ttlv_value_view_release(inner_value);
    kmipkit_ttlv_item_view_release(inner_item);
    kmipkit_ttlv_structure_view_release(nested);
    kmipkit_ttlv_value_view_release(outer_value);
    kmipkit_ttlv_item_view_release(outer_item);
    return result;
}

static int pkcs11_nested_bytes_match(
    kmipkit_ttlv_structure_view_t *parent,
    uint64_t index,
    uint32_t expected_tag,
    const uint8_t *expected,
    uint64_t expected_length) {
    kmipkit_ttlv_item_view_t *outer_item = 0;
    kmipkit_ttlv_value_view_t *outer_value = 0;
    kmipkit_ttlv_structure_view_t *nested = 0;
    kmipkit_ttlv_item_view_t *inner_item = 0;
    kmipkit_ttlv_value_view_t *inner_value = 0;
    kmipkit_tag_t *outer_tag = 0;
    kmipkit_tag_t *inner_tag = 0;
    uint32_t actual_outer_tag = 0;
    uint32_t actual_inner_tag = 0;
    uint64_t count = 0;
    int result = 0;
    if (parent == 0 ||
        kmipkit_ttlv_structure_view_item_at(parent, index, &outer_item) != 0 ||
        kmipkit_ttlv_item_view_tag(outer_item, &outer_tag) != 0 ||
        kmipkit_ttlv_tag_value(outer_tag, &actual_outer_tag) != 0 ||
        actual_outer_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(outer_item, &outer_value) != 0 ||
        kmipkit_ttlv_value_view_structure(outer_value, &nested) != 0 ||
        kmipkit_ttlv_structure_view_item_count(nested, &count) != 0 || count != 1 ||
        kmipkit_ttlv_structure_view_item_at(nested, 0, &inner_item) != 0 ||
        kmipkit_ttlv_item_view_tag(inner_item, &inner_tag) != 0 ||
        kmipkit_ttlv_tag_value(inner_tag, &actual_inner_tag) != 0 ||
        actual_inner_tag != expected_tag ||
        kmipkit_ttlv_item_view_value(inner_item, &inner_value) != 0 ||
        !pkcs11_bytes_match(inner_value, expected, expected_length)) {
        goto cleanup;
    }
    result = 1;
cleanup:
    kmipkit_tag_release(inner_tag);
    kmipkit_tag_release(outer_tag);
    kmipkit_ttlv_value_view_release(inner_value);
    kmipkit_ttlv_item_view_release(inner_item);
    kmipkit_ttlv_structure_view_release(nested);
    kmipkit_ttlv_value_view_release(outer_value);
    kmipkit_ttlv_item_view_release(outer_item);
    return result;
}

static int pkcs11_owned_scalar_matches(
    kmipkit_ttlv_structure_t *structure,
    uint32_t expected_tag,
    uint8_t expected_type,
    int32_t expected_integer,
    uint32_t expected_enumeration) {
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t actual_tag = 0;
    uint8_t actual_type = 0;
    int32_t actual_integer = 0;
    uint32_t actual_enumeration = 0;
    uint64_t count = 0;
    int result = 0;
    if (structure == 0 || kmipkit_ttlv_structure_view(structure, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != 1 ||
        kmipkit_ttlv_structure_view_item_at(view, 0, &item) != 0 ||
        !pkcs11_view_item_tag(view, 0, &actual_tag) || actual_tag != expected_tag ||
        kmipkit_ttlv_item_view_type(item, &actual_type) != 0 ||
        actual_type != expected_type || kmipkit_ttlv_item_view_value(item, &value) != 0) {
        goto cleanup;
    }
    if (expected_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER) {
        result = kmipkit_ttlv_value_view_integer(value, &actual_integer) == 0 &&
            actual_integer == expected_integer;
    } else if (expected_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) {
        result = kmipkit_ttlv_value_view_enumeration(value, &actual_enumeration) == 0 &&
            actual_enumeration == expected_enumeration;
    }
cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    kmipkit_ttlv_structure_view_release(view);
    return result;
}

static int pkcs11_owned_bytes_match(
    kmipkit_ttlv_structure_t *structure,
    uint32_t expected_tag,
    const uint8_t *expected,
    uint64_t expected_length) {
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t actual_tag = 0;
    int result = 0;
    if (structure == 0 || kmipkit_ttlv_structure_view(structure, &view) != 0) {
        goto cleanup;
    }
    if (!pkcs11_view_item_tag(view, 0, &actual_tag) || actual_tag != expected_tag ||
        kmipkit_ttlv_structure_view_item_at(view, 0, &item) != 0 ||
        kmipkit_ttlv_item_view_value(item, &value) != 0) {
        goto cleanup;
    }
    result = pkcs11_bytes_match(value, expected, expected_length);
cleanup:
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    kmipkit_ttlv_structure_view_release(view);
    return result;
}

static int pkcs11_hex_nibble(char value) {
    if (value >= '0' && value <= '9') return value - '0';
    if (value >= 'a' && value <= 'f') return value - 'a' + 10;
    if (value >= 'A' && value <= 'F') return value - 'A' + 10;
    return -1;
}

static int pkcs11_decode_hex(
    const char *text, uint8_t *output, uint64_t output_capacity, uint64_t *output_length) {
    const size_t text_length = text == 0 ? 0 : strlen(text);
    if (text == 0 || output == 0 || output_length == 0 || text_length % 2 != 0 ||
        (uint64_t)(text_length / 2) > output_capacity) {
        return 0;
    }
    for (size_t index = 0; index < text_length; index += 2) {
        const int high = pkcs11_hex_nibble(text[index]);
        const int low = pkcs11_hex_nibble(text[index + 1]);
        if (high < 0 || low < 0) return 0;
        output[index / 2] = (uint8_t)((high << 4) | low);
    }
    *output_length = (uint64_t)(text_length / 2);
    return 1;
}

static int pkcs11_response_matches_fixture(
    kmipkit_ttlv_structure_t *response_ttlv,
    const uint32_t *expected_tags,
    uint64_t expected_tag_count,
    int expected_correlation_present,
    uint32_t expected_function_tag,
    int32_t expected_function_value,
    uint32_t expected_correlation_tag,
    const uint8_t *expected_correlation,
    uint64_t expected_correlation_length,
    uint32_t expected_output_tag,
    const uint8_t *expected_output,
    uint64_t expected_output_length,
    uint32_t expected_return_code_tag,
    uint32_t expected_return_code,
    uint32_t expected_vendor_tag,
    int32_t expected_vendor_value) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    kmipkit_ttlv_structure_view_t *response_view = 0;
    kmipkit_ttlv_item_view_t *operation_item = 0;
    kmipkit_ttlv_value_view_t *operation_value = 0;
    kmipkit_ttlv_item_view_t *status_item = 0;
    kmipkit_ttlv_value_view_t *status_value = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_ttlv_item_view_t *correlation_item = 0;
    kmipkit_ttlv_value_view_t *correlation_value = 0;
    kmipkit_tag_t *correlation_tag = 0;
    kmipkit_tag_t *payload_tag = 0;
    kmipkit_ttlv_item_view_t *return_item = 0;
    kmipkit_ttlv_value_view_t *return_value = 0;
    kmipkit_ttlv_structure_view_t *return_code_view = 0;
    kmipkit_ttlv_item_view_t *code_item = 0;
    kmipkit_ttlv_value_view_t *code_value = 0;
    kmipkit_ttlv_item_view_t *vendor_item = 0;
    kmipkit_ttlv_value_view_t *vendor_value = 0;
    kmipkit_tag_t *vendor_tag = 0;
    uint64_t nested_count = 0;
    uint32_t actual_return_code = 0;
    uint32_t actual_vendor_tag = 0;
    uint32_t actual_nested_tag = 0;
    uint32_t actual_operation = 0;
    uint32_t actual_status = UINT32_MAX;
    uint32_t actual_payload_tag = 0;
    int32_t actual_vendor_value = 0;
    uint8_t actual_type = 0;
    uint32_t filtered_expected_tags[5] = {0};
    uint64_t filtered_expected_tag_count = 0;
    uint64_t output_index = expected_correlation_present ? 2 : 1;
    uint64_t return_code_index = output_index + 1;
    uint64_t vendor_index = return_code_index + 1;
    int result = 0;

    if (response_ttlv == 0 || expected_tags == 0 || expected_tag_count != 5) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < expected_tag_count; ++index) {
        if (!expected_correlation_present && expected_tags[index] == expected_correlation_tag) {
            continue;
        }
        filtered_expected_tags[filtered_expected_tag_count++] = expected_tags[index];
    }
    if (
        kmipkit_ttlv_structure_view(response_ttlv, &response_view) != 0 ||
        !check_view_tags(response_view, response_tags, 3) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 0, &operation_item) != 0 ||
        kmipkit_ttlv_item_view_type(operation_item, &actual_type) != 0 ||
        actual_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
        kmipkit_ttlv_item_view_value(operation_item, &operation_value) != 0 ||
        kmipkit_ttlv_value_view_enumeration(operation_value, &actual_operation) != 0 ||
        actual_operation != UINT32_C(0x33) ||
        kmipkit_ttlv_structure_view_item_at(response_view, 1, &status_item) != 0 ||
        kmipkit_ttlv_item_view_type(status_item, &actual_type) != 0 ||
        actual_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
        kmipkit_ttlv_item_view_value(status_item, &status_value) != 0 ||
        kmipkit_ttlv_value_view_enumeration(status_value, &actual_status) != 0 ||
        actual_status != 0 ||
        kmipkit_ttlv_structure_view_item_at(response_view, 2, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_tag(payload_item, &payload_tag) != 0 ||
        kmipkit_ttlv_tag_value(payload_tag, &actual_payload_tag) != 0 ||
        actual_payload_tag != UINT32_C(0x42007c) ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(payload_view, filtered_expected_tags, filtered_expected_tag_count) ||
        pkcs11_view_contains_tag(payload_view, UINT32_C(0x420159)) ||
        pkcs11_view_contains_tag(payload_view, UINT32_C(0x42015b)) ||
        !pkcs11_nested_integer_matches(payload_view, 0, expected_function_tag,
            expected_function_value) ||
        !pkcs11_nested_bytes_match(payload_view, output_index, expected_output_tag, expected_output,
            expected_output_length) ||
        kmipkit_ttlv_structure_view_item_at(payload_view, return_code_index, &return_item) != 0 ||
        kmipkit_ttlv_item_view_value(return_item, &return_value) != 0 ||
        kmipkit_ttlv_value_view_structure(return_value, &return_code_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(return_code_view, &nested_count) != 0 ||
        nested_count != 1 || !pkcs11_view_item_tag(return_code_view, 0, &actual_nested_tag) ||
        actual_nested_tag != expected_return_code_tag ||
        kmipkit_ttlv_structure_view_item_at(return_code_view, 0, &code_item) != 0 ||
        kmipkit_ttlv_item_view_type(code_item, &actual_type) != 0 ||
        actual_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
        kmipkit_ttlv_item_view_value(code_item, &code_value) != 0 ||
        kmipkit_ttlv_value_view_enumeration(code_value, &actual_return_code) != 0 ||
        actual_return_code != expected_return_code ||
        kmipkit_ttlv_structure_view_item_at(payload_view, vendor_index, &vendor_item) != 0 ||
        kmipkit_ttlv_item_view_tag(vendor_item, &vendor_tag) != 0 ||
        kmipkit_ttlv_tag_value(vendor_tag, &actual_vendor_tag) != 0 ||
        actual_vendor_tag != expected_vendor_tag ||
        kmipkit_ttlv_item_view_type(vendor_item, &actual_type) != 0 ||
        actual_type != KMIPKIT_TTLV_ITEM_TYPE_INTEGER ||
        kmipkit_ttlv_item_view_value(vendor_item, &vendor_value) != 0 ||
        kmipkit_ttlv_value_view_integer(vendor_value, &actual_vendor_value) != 0 ||
        actual_vendor_value != expected_vendor_value) {
        goto cleanup;
    }
    if (expected_correlation_present) {
        if (kmipkit_ttlv_structure_view_item_at(payload_view, 1, &correlation_item) != 0 ||
            kmipkit_ttlv_item_view_tag(correlation_item, &correlation_tag) != 0 ||
            kmipkit_ttlv_tag_value(correlation_tag, &actual_nested_tag) != 0 ||
            actual_nested_tag != expected_correlation_tag ||
            kmipkit_ttlv_item_view_type(correlation_item, &actual_type) != 0 ||
            actual_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
            kmipkit_ttlv_item_view_value(correlation_item, &correlation_value) != 0 ||
            !pkcs11_bytes_match(correlation_value, expected_correlation,
                expected_correlation_length)) {
            goto cleanup;
        }
    } else if (pkcs11_view_contains_tag(payload_view, expected_correlation_tag)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_value_view_release(vendor_value);
    kmipkit_tag_release(vendor_tag);
    kmipkit_ttlv_item_view_release(vendor_item);
    kmipkit_ttlv_value_view_release(code_value);
    kmipkit_ttlv_item_view_release(code_item);
    kmipkit_ttlv_structure_view_release(return_code_view);
    kmipkit_ttlv_value_view_release(return_value);
    kmipkit_ttlv_item_view_release(return_item);
    kmipkit_ttlv_value_view_release(correlation_value);
    kmipkit_tag_release(correlation_tag);
    kmipkit_ttlv_item_view_release(correlation_item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_value_view_release(status_value);
    kmipkit_ttlv_item_view_release(status_item);
    kmipkit_ttlv_value_view_release(operation_value);
    kmipkit_ttlv_item_view_release(operation_item);
    kmipkit_tag_release(payload_tag);
    kmipkit_tag_release(vendor_tag);
    kmipkit_ttlv_structure_view_release(response_view);
    return result;
}

int kmipkit_client_abi_c_consumer_execute_pkcs11_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    static const uint8_t host[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint8_t interface_text[] = "test-interface";
    static const uint8_t input_bytes[] = "opaque-input";
    static const uint8_t correlation_bytes[] = "pkcs11-correlation";
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport_input = {0};
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_byte_span_v1_t correlation_span = {correlation_bytes, sizeof(correlation_bytes) - 1};
    kmipkit_client_extension_registry_t *registry = 0;
    kmipkit_client_configuration_t *configuration = 0;
    kmipkit_transport_config_t *transport = 0;
    kmipkit_client_t *client = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_response_t *response = 0;
    kmipkit_client_batch_item_response_t *response_item = 0;
    kmipkit_ttlv_structure_t *function = 0;
    kmipkit_ttlv_structure_t *interface_structure = 0;
    kmipkit_ttlv_structure_t *input_parameters = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_t *interface_ttlv = 0;
    kmipkit_ttlv_structure_t *function_ttlv = 0;
    kmipkit_ttlv_structure_t *output_parameters_ttlv = 0;
    kmipkit_ttlv_structure_t *return_code_ttlv = 0;
    kmipkit_byte_span_v1_t response_correlation = {0};
    uint8_t correlation_copy[4096] = {0};
    uint8_t expected_correlation[4096] = {0};
    uint8_t expected_output[4096] = {0};
    uint8_t interface_present = 0;
    uint8_t function_present = 0;
    uint8_t correlation_present = 0;
    uint8_t output_parameters_present = 0;
    uint8_t return_code_present = 0;
    kmipkit_ttlv_value_t *function_scalar = 0;
    kmipkit_ttlv_value_t *interface_scalar = 0;
    kmipkit_ttlv_value_t *input_scalar = 0;
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    uint32_t expected_tags[5] = {0};
    uint32_t expected_return_code = 0;
    uint32_t expected_vendor_tag = 0;
    uint32_t expected_function_tag = 0;
    uint32_t expected_output_tag = 0;
    uint32_t expected_return_code_tag = 0;
    int32_t expected_vendor_value = 0;
    int32_t expected_function_value = 0;
    uint64_t expected_correlation_length = 0;
    uint64_t expected_output_length = 0;
    uint64_t response_count = 0;
    int expected_correlation_present = 0;
    unsigned long port = 0;
    char *parse_end = 0;
    int result = 1;

    if (argc != 22 || argv == 0 || limits == 0) {
        return 2;
    }
    for (int index = 1; index < argc; ++index) {
        if (argv[index] == 0) {
            return 2;
        }
    }
    port = strtoul(argv[5], &parse_end, 10);
    if (parse_end == argv[5] || *parse_end != '\0' || port == 0 || port > 65535UL) {
        return 3;
    }
    for (uint64_t index = 0; index < 5; ++index) {
        unsigned long raw = strtoul(argv[6 + index], &parse_end, 0);
        if (parse_end == argv[6 + index] || *parse_end != '\0' || raw > UINT32_MAX) {
            return 4;
        }
        expected_tags[index] = (uint32_t)raw;
    }
    unsigned long raw_return = strtoul(argv[11], &parse_end, 0);
    if (parse_end == argv[11] || *parse_end != '\0' || raw_return > UINT32_MAX) {
        return 5;
    }
    expected_return_code = (uint32_t)raw_return;
    unsigned long raw_vendor_tag = strtoul(argv[12], &parse_end, 0);
    if (parse_end == argv[12] || *parse_end != '\0' || raw_vendor_tag > UINT32_MAX) {
        return 6;
    }
    expected_vendor_tag = (uint32_t)raw_vendor_tag;
    long raw_vendor_value = strtol(argv[13], &parse_end, 10);
    if (parse_end == argv[13] || *parse_end != '\0' || raw_vendor_value < INT32_MIN ||
        raw_vendor_value > INT32_MAX) {
        return 7;
    }
    expected_vendor_value = (int32_t)raw_vendor_value;
    unsigned long raw_function_tag = strtoul(argv[14], &parse_end, 0);
    if (parse_end == argv[14] || *parse_end != '\0' || raw_function_tag > UINT32_MAX) {
        return 8;
    }
    expected_function_tag = (uint32_t)raw_function_tag;
    long raw_function_value = strtol(argv[15], &parse_end, 10);
    if (parse_end == argv[15] || *parse_end != '\0' || raw_function_value < INT32_MIN ||
        raw_function_value > INT32_MAX) {
        return 9;
    }
    expected_function_value = (int32_t)raw_function_value;
    if (!pkcs11_decode_hex(argv[16], expected_correlation, sizeof(expected_correlation),
            &expected_correlation_length)) {
        return 10;
    }
    unsigned long raw_output_tag = strtoul(argv[17], &parse_end, 0);
    if (parse_end == argv[17] || *parse_end != '\0' || raw_output_tag > UINT32_MAX) {
        return 11;
    }
    expected_output_tag = (uint32_t)raw_output_tag;
    if (!pkcs11_decode_hex(argv[18], expected_output, sizeof(expected_output),
            &expected_output_length)) {
        return 12;
    }
    unsigned long raw_return_code_tag = strtoul(argv[19], &parse_end, 0);
    if (parse_end == argv[19] || *parse_end != '\0' || raw_return_code_tag > UINT32_MAX) {
        return 13;
    }
    expected_return_code_tag = (uint32_t)raw_return_code_tag;
    unsigned long raw_correlation_tag = strtoul(argv[20], &parse_end, 0);
    if (parse_end == argv[20] || *parse_end != '\0' || raw_correlation_tag > UINT32_MAX) {
        return 14;
    }
    const uint32_t expected_correlation_tag = (uint32_t)raw_correlation_tag;
    unsigned long raw_correlation_present = strtoul(argv[21], &parse_end, 10);
    if (parse_end == argv[21] || *parse_end != '\0' || raw_correlation_present > 1) {
        return 15;
    }
    expected_correlation_present = (int)raw_correlation_present;

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

    if (kmipkit_ttlv_structure_create(&function) != 0 ||
        kmipkit_ttlv_value_integer(-12345, &function_scalar) != 0 ||
        !append_pkcs11_ttlv_value(
            limits, &function, UINT32_C(0x42015a), &function_scalar) ||
        kmipkit_ttlv_structure_create(&interface_structure) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, interface_text, (uint64_t)(sizeof(interface_text) - 1),
            &interface_scalar) != 0 ||
        !append_pkcs11_ttlv_value(
            limits, &interface_structure, UINT32_C(0x420159), &interface_scalar) ||
        kmipkit_ttlv_structure_create(&input_parameters) != 0 ||
        kmipkit_ttlv_value_byte_string(
            limits, input_bytes, (uint64_t)(sizeof(input_bytes) - 1), &input_scalar) != 0 ||
        !append_pkcs11_ttlv_value(
            limits, &input_parameters, UINT32_C(0x42015b), &input_scalar) ||
        kmipkit_client_batch_create(&batch) != 0 ||
        kmipkit_client_batch_item_pkcs11(
            function, interface_structure, &correlation_span, input_parameters, &item) != 0 ||
        item == 0 || kmipkit_client_batch_add_item(batch, &item) != 0) {
        goto cleanup;
    }
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    int32_t execute_status = kmipkit_client_execute(client, &batch, limits, &response);
    if (execute_status != 0) {
        goto cleanup;
    }
    if (batch != 0 || response == 0 ||
        kmipkit_client_batch_response_item_count(response, &response_count) != 0 ||
        response_count != 1) {
        goto cleanup;
    }
    if (kmipkit_client_batch_response_item_at(response, 0, &response_item) != 0 ||
        response_item == 0) {
        goto cleanup;
    }
    int32_t metadata_status =
        kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata);
    if (metadata_status != 0) {
        goto cleanup;
    }
    if (metadata.operation != UINT32_C(0x33)) {
        goto cleanup;
    }
    if (metadata.result_status != 0) {
        goto cleanup;
    }
    uint8_t response_present = 0;
    if (kmipkit_client_batch_item_response_pkcs11_ttlv(
            response_item, &response_present, &response_ttlv) != 0 ||
        response_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
            response_item, &interface_present, &interface_ttlv) != 0 ||
        kmipkit_client_batch_item_response_pkcs11_function_ttlv(
            response_item, &function_present, &function_ttlv) != 0 ||
        kmipkit_client_batch_item_response_pkcs11_correlation_value(
            response_item, &correlation_present, &response_correlation) != 0 ||
        kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv(
            response_item, &output_parameters_present, &output_parameters_ttlv) != 0 ||
        kmipkit_client_batch_item_response_pkcs11_return_code_ttlv(
            response_item, &return_code_present, &return_code_ttlv) != 0) {
        goto cleanup;
    }
    if (interface_present != 0 || interface_ttlv != 0 || function_present != 1 ||
        function_ttlv == 0 || output_parameters_present != 1 ||
        output_parameters_ttlv == 0 || return_code_present != 1 || return_code_ttlv == 0) {
        goto cleanup;
    }
    if (correlation_present != expected_correlation_present) {
        goto cleanup;
    }
    if (response_correlation.length != expected_correlation_length) {
        goto cleanup;
    }
    if (response_correlation.length > (uint64_t)sizeof(correlation_copy) ||
        (response_correlation.length > 0 && response_correlation.data == 0) ||
        (!expected_correlation_present && response_correlation.data != 0)) {
        goto cleanup;
    }
    if (response_correlation.length > 0) {
        memcpy(correlation_copy, response_correlation.data, (size_t)response_correlation.length);
    }
    kmipkit_client_batch_item_response_release(response_item);
    response_item = 0;
    kmipkit_client_batch_response_release(response);
    response = 0;
    if (expected_correlation_length > 0 &&
        memcmp(correlation_copy, expected_correlation, (size_t)expected_correlation_length) != 0) {
        goto cleanup;
    }
    if (!pkcs11_owned_scalar_matches(
            function_ttlv, expected_function_tag, KMIPKIT_TTLV_ITEM_TYPE_INTEGER,
            expected_function_value, 0) ||
        !pkcs11_owned_bytes_match(output_parameters_ttlv, expected_output_tag,
            expected_output, expected_output_length) ||
        !pkcs11_owned_scalar_matches(return_code_ttlv, expected_return_code_tag,
            KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, 0, expected_return_code)) {
        goto cleanup;
    }
    if (!pkcs11_response_matches_fixture(
            response_ttlv, expected_tags, 5, expected_correlation_present,
            expected_function_tag, expected_function_value,
            expected_correlation_tag, expected_correlation, expected_correlation_length,
            expected_output_tag, expected_output, expected_output_length,
            expected_return_code_tag, expected_return_code, expected_vendor_tag,
            expected_vendor_value)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_value_release(input_scalar);
    kmipkit_ttlv_value_release(interface_scalar);
    kmipkit_ttlv_value_release(function_scalar);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_ttlv_structure_release(return_code_ttlv);
    kmipkit_ttlv_structure_release(output_parameters_ttlv);
    kmipkit_ttlv_structure_release(function_ttlv);
    kmipkit_ttlv_structure_release(interface_ttlv);
    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_item_release(item);
    kmipkit_client_batch_release(batch);
    kmipkit_client_release(client);
    kmipkit_transport_config_release(transport);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_ttlv_structure_release(input_parameters);
    kmipkit_ttlv_structure_release(interface_structure);
    kmipkit_ttlv_structure_release(function);
    return result;
}

static int ping_query_abi_is_callable(void) {
    static const uint32_t functions[] = {5, 1, UINT32_C(0x80000001), 5};
    static const uint8_t first_group[] = "group-a";
    static const uint8_t second_group[] = "group-a";
    const kmipkit_byte_span_v1_t groups[] = {
        {first_group, sizeof(first_group) - 1},
        {second_group, sizeof(second_group) - 1},
    };
    kmipkit_client_batch_item_t *ping = 0;
    kmipkit_client_batch_item_t *query = 0;
    kmipkit_client_batch_item_t *invalid = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_ping(&ping) != 0 || ping == 0 ||
        kmipkit_client_batch_item_query(
            functions, sizeof(functions) / sizeof(functions[0]), 1, groups,
            sizeof(groups) / sizeof(groups[0]), &query) != 0 || query == 0) {
        goto failure;
    }
    if (kmipkit_client_batch_item_query(
            0, 1, 0, 0, 0, &invalid) != T076_ERROR_INVALID_INPUT || invalid != 0) {
        goto failure;
    }
    kmipkit_client_batch_item_release(query);
    kmipkit_client_batch_item_release(ping);
    return 1;

failure:
    kmipkit_client_batch_item_release(query);
    kmipkit_client_batch_item_release(ping);
    return 0;
}

static int lifecycle_constructors_are_callable_from_c(void) {
    static const uint8_t activate_text[] = "activate-target";
    static const uint8_t recover_text[] = "recover-target";
    kmipkit_unique_identifier_v1_t activate_identifier = {
        (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        1,
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        0,
        activate_text,
        sizeof(activate_text) - 1,
        0,
        0
    };
    kmipkit_unique_identifier_v1_t archive_identifier = {
        (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        1,
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
        0,
        0,
        0,
        UINT32_C(0x8fedcba9),
        0
    };
    kmipkit_unique_identifier_v1_t destroy_identifier = {
        (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        1,
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        0,
        0,
        0,
        0,
        -723
    };
    kmipkit_unique_identifier_v1_t recover_identifier = {
        (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        1,
        KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        0,
        recover_text,
        sizeof(recover_text) - 1,
        0,
        0
    };
    kmipkit_client_batch_item_t *activate = 0;
    kmipkit_client_batch_item_t *archive = 0;
    kmipkit_client_batch_item_t *destroy = 0;
    kmipkit_client_batch_item_t *recover = 0;
    if (kmipkit_client_batch_item_activate(&activate_identifier, &activate) != 0 || activate == 0 ||
        kmipkit_client_batch_item_archive(&archive_identifier, &archive) != 0 || archive == 0 ||
        kmipkit_client_batch_item_destroy(&destroy_identifier, &destroy) != 0 || destroy == 0 ||
        kmipkit_client_batch_item_recover(&recover_identifier, &recover) != 0 || recover == 0) {
        goto failure;
    }
    kmipkit_client_batch_item_release(activate);
    kmipkit_client_batch_item_release(archive);
    kmipkit_client_batch_item_release(destroy);
    kmipkit_client_batch_item_release(recover);
    return 1;

failure:
    kmipkit_client_batch_item_release(activate);
    kmipkit_client_batch_item_release(archive);
    kmipkit_client_batch_item_release(destroy);
    kmipkit_client_batch_item_release(recover);
    return 0;
}

int kmipkit_client_abi_c_consumer_execute_tls(
    int argc, char **argv, kmipkit_codec_limits_t *limits) {
    int rekey_key_pair_only = 0;
    int create_split_key_fixture_only = 0;
    int cancel_payload_only = 0;
    int request_builders_wire_only = 0;
    if (argv == 0 || limits == 0) {
        return 22;
    }
    if (!absent_profile_details_preserve_non_profile_error_from_c()) {
        return 25;
    }
    if (argc == 7 && argv[6] != 0 && strcmp(argv[6], "rekey-key-pair") == 0) {
        rekey_key_pair_only = 1;
    } else if (argc == 9 && argv[6] != 0 && argv[7] != 0 && argv[8] != 0 &&
               strcmp(argv[6], "create-split-key-fixture") == 0) {
        create_split_key_fixture_only = 1;
    } else if (argc == 8 && argv[6] != 0 && argv[7] != 0 &&
               strcmp(argv[6], "cancel-payload") == 0) {
        cancel_payload_only = 1;
    } else if (argc == 7 && argv[6] != 0 &&
               strcmp(argv[6], "request-builders-wire") == 0) {
        request_builders_wire_only = 1;
    } else if (argc < 16) {
        return 22;
    }
    return execute_public_tls_request(
        argc, argv, limits, rekey_key_pair_only, create_split_key_fixture_only, cancel_payload_only,
        request_builders_wire_only);
}

static int request_batch_item_unique_id_accessor_preserves_presence_and_bytes(void) {
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_client_batch_item_t *replacement = 0;
    const uint8_t identifier[] = {0x00, 0xff, 0x69, 0x64};
    uint8_t present = UINT8_MAX;
    kmipkit_byte_span_v1_t value = {(const uint8_t *)1, UINT64_MAX};
    int32_t status = 0;

    if (kmipkit_client_batch_item_discover_versions(&item) != 0 || item == 0) {
        goto failure;
    }
    if (kmipkit_client_batch_item_unique_batch_item_id(item, &present, &value) != 0 ||
        present != 0 || value.data != 0 || value.length != 0) {
        goto failure;
    }
    status = kmipkit_client_batch_item_with_unique_batch_item_id(item, 0, 0, &replacement);
    item = 0;
    if (status != 0 || replacement == 0) {
        goto failure;
    }
    item = replacement;
    replacement = 0;
    if (kmipkit_client_batch_item_unique_batch_item_id(item, &present, &value) != 0 ||
        present != 1 || value.length != 0) {
        goto failure;
    }
    status = kmipkit_client_batch_item_with_unique_batch_item_id(
            item, identifier, sizeof(identifier), &replacement);
    item = 0;
    if (status != 0 || replacement == 0) {
        goto failure;
    }
    item = replacement;
    replacement = 0;
    if (kmipkit_client_batch_item_unique_batch_item_id(item, &present, &value) != 0 ||
        present != 1 || value.data == 0 || value.length != sizeof(identifier) ||
        memcmp(value.data, identifier, sizeof(identifier)) != 0) {
        goto failure;
    }
    if (kmipkit_client_batch_item_unique_batch_item_id(item, &present, &value) != 0 ||
        present != 1 || value.data == 0 || value.length != sizeof(identifier) ||
        memcmp(value.data, identifier, sizeof(identifier)) != 0) {
        goto failure;
    }
    kmipkit_client_batch_item_release(item);
    return 1;

failure:
    kmipkit_client_batch_item_release(replacement);
    kmipkit_client_batch_item_release(item);
    return 0;
}

int main(int argc, char **argv) {
    if (KMIPKIT_PROFILE_TRANSPORT_RAW_TTLV_TLS != UINT32_C(1) ||
        KMIPKIT_PROFILE_TRANSPORT_HTTPS != UINT32_C(2)) {
        return 26;
    }
    if (!execute_with_options_and_error_ttlv_clears_outputs()) {
        return 22;
    }
    if (!async_operation_options_symbols_clear_outputs()) {
        return 25;
    }
    if (!execute_with_error_ttlv_clears_outputs()) {
        return 23;
    }
    if (!execute_with_profiles_rejects_invalid_selection_from_c()) {
        return 24;
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
        !rekey_constructor_is_callable_from_c() ||
        !reprovision_constructor_is_callable_from_c() ||
        !rng_request_builders_are_callable_from_c() ||
        !rekey_key_pair_constructor_is_callable_from_c() ||
        !rekey_key_pair_response_rejects_invalid_output_records_from_c() ||
        !rekey_key_pair_response_accessors_clear_outputs_from_c() ||
        !rekey_response_accessors_clear_outputs_from_c() ||
        !recertify_constructor_is_callable_from_c() ||
        !recertify_response_accessors_clear_outputs_from_c() ||
        !generic_big_integer_constructor_preserves_type_and_octets_from_c() ||
        !certify_constructor_is_callable_from_c() ||
        !certify_response_accessor_is_callable_from_c() ||
        !check_constructor_is_callable_from_c() ||
        !delegated_login_constructor_is_callable_from_c() ||
        !login_constructor_is_callable_from_c() ||
        !derive_key_constructor_is_callable_from_c() ||
        !export_constructor_is_callable_from_c() ||
        !export_constructor_rejects_invalid_presence_from_c() ||
        !delegated_login_response_accessors_clear_outputs_from_c() ||
        !login_response_accessors_clear_outputs_from_c() ||
        !logout_response_accessor_clears_outputs_from_c() ||
        !check_response_accessors_clear_outputs_from_c() ||
        !get_constraints_constructor_is_callable_from_c() ||
        !validate_constructor_is_callable_from_c() ||
        !validate_constructor_copies_nonempty_certificate_array_from_c() ||
        !validate_constructor_copies_payload_extensions() ||
        !validate_response_accessors_clear_invalid_outputs_from_c() ||
        !log_constructor_is_callable_from_c() ||
        !interop_response_accessor_clears_invalid_outputs_from_c() ||
        !get_constraints_response_accessor_clears_outputs_from_c() ||
        !get_locate_abi_is_callable_from_c() ||
        !ping_query_abi_is_callable() ||
        !lifecycle_constructors_are_callable_from_c() ||
        !register_protection_storage_masks_builder_is_callable_from_c() ||
        !obtain_lease_abi_is_callable_from_c() ||
        !get_usage_allocation_constructor_is_callable_from_c() ||
        !import_constructor_rejects_invalid_table_238_inputs_from_c() ||
        !get_usage_allocation_response_accessor_clears_outputs() ||
        !join_split_key_constructor_enforces_aggregate_identifier_budget_from_c() ||
        !creation_create_key_pair_matches_shared_fixture() ||
        !creation_create_split_key_matches_shared_fixture() ||
        !creation_rejects_malformed_split_key_inputs() ||
        !creation_rejects_missing_required_attributes() ||
        !cryptographic_abi_symbols_are_callable() ||
        !request_batch_item_unique_id_accessor_preserves_presence_and_bytes() ||
        !encrypt_decrypt_abi_is_callable_from_compiled_c()) {
        return 7;
    }
    if (!logout_constructor_is_callable_from_c()) {
        return 71;
    }
    if (!interop_constructor_is_callable_from_c()) {
        return 72;
    }
    if (!join_split_key_constructor_pairs_each_identifier_with_its_threshold()) {
        return 73;
    }
    if (!pkcs11_constructor_is_callable_from_c() ||
        !pkcs11_constructor_preserves_opaque_payload_extensions_from_c() ||
        !pkcs11_response_accessor_clears_invalid_outputs_from_c()) {
        return 74;
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

int kmipkit_client_abi_c_consumer_async_output_errors_clear(void) {
    kmipkit_client_async_operation_outcome_t *outcome =
        (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_poll(0, 0, 0, 0, &outcome) != T076_ERROR_INVALID_INPUT ||
        outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_cancel(0, 0, 0, 0, &outcome) != T076_ERROR_INVALID_INPUT ||
        outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_process(0, 0, 0, 0, 0, 0, &outcome) !=
            T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_query_async_requests(
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, &outcome) !=
            T076_ERROR_INVALID_INPUT || outcome != 0) {
        return 0;
    }

    kmipkit_client_async_operation_metadata_v1_t metadata = {0};
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = UINT32_C(1);
    if (kmipkit_client_async_operation_outcome_metadata_v1(0, &metadata) !=
        T076_ERROR_INVALID_INPUT) {
        return 0;
    }

    uint8_t present = UINT8_C(9);
    kmipkit_byte_span_v1_t correlation = {
        (const uint8_t *)(uintptr_t)9,
        UINT64_C(9),
    };
    if (kmipkit_client_async_operation_outcome_correlation_value(
            0, &present, &correlation) != T076_ERROR_INVALID_INPUT ||
        present != 0 || correlation.data != 0 || correlation.length != 0) {
        return 0;
    }

    present = UINT8_C(9);
    kmipkit_ttlv_structure_t *payload =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;
    if (kmipkit_client_async_operation_outcome_response_payload_ttlv(
            0, &present, &payload) != T076_ERROR_INVALID_INPUT ||
        present != 0 || payload != 0) {
        return 0;
    }

    kmipkit_client_async_operation_outcome_release(0);
    return 1;
}

static int async_request_value_matches_bytes(
    kmipkit_ttlv_value_view_t *value,
    const uint8_t *expected,
    uint64_t expected_length) {
    uint64_t actual_length = 0;
    if (value == 0 || (expected == 0 && expected_length != 0) ||
        kmipkit_ttlv_value_view_byte_length(value, &actual_length) != 0 ||
        actual_length != expected_length) {
        return 0;
    }
    for (uint64_t index = 0; index < expected_length; ++index) {
        uint8_t actual = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, index, &actual) != 0 ||
            actual != expected[index]) {
            return 0;
        }
    }
    return 1;
}

static int async_request_payload_matches_correlation(
    kmipkit_ttlv_structure_t *payload,
    const uint8_t *expected,
    uint64_t expected_length) {
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    kmipkit_tag_t *tag = 0;
    uint64_t item_count = 0;
    uint32_t raw_tag = 0;
    uint8_t item_type = 0;
    int matches = 0;
    if (payload == 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &item_count) != 0 || item_count != 1 ||
        kmipkit_ttlv_structure_view_item_at(view, 0, &item) != 0 ||
        kmipkit_ttlv_item_view_tag(item, &tag) != 0 ||
        kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 || raw_tag != UINT32_C(0x00420006) ||
        kmipkit_ttlv_item_view_type(item, &item_type) != 0 ||
        item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 ||
        !async_request_value_matches_bytes(value, expected, expected_length)) {
        goto cleanup;
    }
    matches = 1;

cleanup:
    kmipkit_tag_release(tag);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    kmipkit_ttlv_structure_view_release(view);
    return matches;
}

static int async_query_payload_matches_filters(
    kmipkit_ttlv_structure_t *payload,
    const kmipkit_byte_span_v1_t *correlations,
    uint64_t correlation_count,
    const uint32_t *operations,
    uint64_t operation_count) {
    kmipkit_ttlv_structure_view_t *root = 0;
    kmipkit_ttlv_structure_view_t *nested = 0;
    kmipkit_ttlv_item_view_t *outer_item = 0;
    kmipkit_ttlv_item_view_t *child_item = 0;
    kmipkit_ttlv_value_view_t *outer_value = 0;
    kmipkit_ttlv_value_view_t *child_value = 0;
    kmipkit_tag_t *outer_tag = 0;
    kmipkit_tag_t *child_tag = 0;
    uint64_t child_count = 0;
    uint32_t raw_tag = 0;
    uint32_t raw_operation = 0;
    uint8_t item_type = 0;
    int matches = 0;

    if (payload == 0 || (correlations == 0 && correlation_count != 0) ||
        (operations == 0 && operation_count != 0) ||
        kmipkit_ttlv_structure_view(payload, &root) != 0) {
        goto cleanup;
    }
    for (uint64_t filter_index = 0; filter_index < UINT64_C(2); ++filter_index) {
        const int is_correlation_filter = filter_index == 0;
        const uint32_t expected_outer_tag = is_correlation_filter
            ? UINT32_C(0x00420176)
            : UINT32_C(0x0042014f);
        const uint32_t expected_child_tag = is_correlation_filter
            ? UINT32_C(0x00420006)
            : UINT32_C(0x0042005c);
        const uint64_t expected_count = is_correlation_filter
            ? correlation_count
            : operation_count;
        if (kmipkit_ttlv_structure_view_item_at(root, filter_index, &outer_item) != 0 ||
            kmipkit_ttlv_item_view_tag(outer_item, &outer_tag) != 0 ||
            kmipkit_ttlv_tag_value(outer_tag, &raw_tag) != 0 ||
            raw_tag != expected_outer_tag ||
            kmipkit_ttlv_item_view_type(outer_item, &item_type) != 0 ||
            item_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
            kmipkit_ttlv_item_view_value(outer_item, &outer_value) != 0 ||
            kmipkit_ttlv_value_view_structure(outer_value, &nested) != 0 ||
            kmipkit_ttlv_structure_view_item_count(nested, &child_count) != 0 ||
            child_count != expected_count) {
            goto cleanup;
        }
        for (uint64_t index = 0; index < expected_count; ++index) {
            if (kmipkit_ttlv_structure_view_item_at(nested, index, &child_item) != 0 ||
                kmipkit_ttlv_item_view_tag(child_item, &child_tag) != 0 ||
                kmipkit_ttlv_tag_value(child_tag, &raw_tag) != 0 ||
                raw_tag != expected_child_tag ||
                kmipkit_ttlv_item_view_type(child_item, &item_type) != 0 ||
                kmipkit_ttlv_item_view_value(child_item, &child_value) != 0) {
                goto cleanup;
            }
            if (is_correlation_filter) {
                if (item_type != KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING ||
                    !async_request_value_matches_bytes(
                        child_value, correlations[index].data, correlations[index].length)) {
                    goto cleanup;
                }
            } else if (item_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
                       kmipkit_ttlv_value_view_enumeration(child_value, &raw_operation) != 0 ||
                       raw_operation != operations[index]) {
                goto cleanup;
            }
            kmipkit_tag_release(child_tag);
            child_tag = 0;
            kmipkit_ttlv_value_view_release(child_value);
            child_value = 0;
            kmipkit_ttlv_item_view_release(child_item);
            child_item = 0;
        }
        kmipkit_ttlv_structure_view_release(nested);
        nested = 0;
        kmipkit_ttlv_value_view_release(outer_value);
        outer_value = 0;
        kmipkit_tag_release(outer_tag);
        outer_tag = 0;
        kmipkit_ttlv_item_view_release(outer_item);
        outer_item = 0;
    }
    matches = 1;

cleanup:
    kmipkit_tag_release(child_tag);
    kmipkit_ttlv_value_view_release(child_value);
    kmipkit_ttlv_item_view_release(child_item);
    kmipkit_ttlv_structure_view_release(nested);
    kmipkit_ttlv_value_view_release(outer_value);
    kmipkit_tag_release(outer_tag);
    kmipkit_ttlv_item_view_release(outer_item);
    kmipkit_ttlv_structure_view_release(root);
    return matches;
}

/* KMIP 2.1 §§6.1.5, 6.1.38, 6.1.39, and 6.1.41: typed request models build payloads only. */
int kmipkit_client_abi_c_consumer_async_request_models(void) {
    static const uint8_t correlation[] = {0x00, 0xff, 0x80};
    static const uint8_t second_correlation[] = {0x10, 0x00};
    const kmipkit_byte_span_v1_t correlation_filters[] = {
        {correlation, sizeof(correlation)},
        {second_correlation, sizeof(second_correlation)},
        {correlation, sizeof(correlation)},
    };
    const uint32_t operation_filters[] = {UINT32_C(0x0000001a), UINT32_C(0xf1234567)};
    kmipkit_cancel_request_t *cancel = 0;
    kmipkit_poll_request_t *poll = 0;
    kmipkit_process_request_t *process = 0;
    kmipkit_query_async_requests_request_t *query = 0;
    kmipkit_query_async_requests_request_t *next_query = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_view_t *nested = 0;
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    kmipkit_ttlv_item_view_t *child_item = 0;
    kmipkit_ttlv_value_view_t *child_value = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_byte_span_v1_t borrowed = {0};
    uint64_t count = 0;
    uint32_t raw_tag = 0;
    uint32_t raw_operation = 0;
    int result = 0;

    if (kmipkit_cancel_request_create(correlation, sizeof(correlation), &cancel) != 0 ||
        kmipkit_cancel_request_asynchronous_correlation_value(cancel, &borrowed) != 0 ||
        borrowed.length != sizeof(correlation) || borrowed.data == 0 ||
        memcmp(borrowed.data, correlation, sizeof(correlation)) != 0 ||
        kmipkit_cancel_request_to_ttlv_payload(cancel, &payload) != 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != 1 ||
        kmipkit_ttlv_structure_view_item_at(view, 0, &item) != 0 ||
        kmipkit_ttlv_item_view_tag(item, &tag) != 0 ||
        kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 || raw_tag != UINT32_C(0x00420006)) {
        goto cleanup;
    }
    if (!async_request_payload_matches_correlation(payload, correlation, sizeof(correlation))) {
        goto cleanup;
    }
    kmipkit_tag_release(tag);
    tag = 0;
    kmipkit_ttlv_item_view_release(item);
    item = 0;
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    kmipkit_ttlv_structure_release(payload);
    payload = 0;

    if (kmipkit_poll_request_create(correlation, sizeof(correlation), &poll) != 0 ||
        kmipkit_poll_request_to_ttlv_payload(poll, &payload) != 0 ||
        !async_request_payload_matches_correlation(payload, correlation, sizeof(correlation))) {
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(payload);
    payload = 0;
    if (kmipkit_process_request_create(correlation, sizeof(correlation), &process) != 0 ||
        kmipkit_process_request_to_ttlv_payload(process, &payload) != 0 ||
        !async_request_payload_matches_correlation(payload, correlation, sizeof(correlation))) {
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(payload);
    payload = 0;

    if (kmipkit_query_async_requests_request_create(&query) != 0 ||
        kmipkit_query_async_requests_request_to_ttlv_payload(query, &payload) != 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != 0) {
        goto cleanup;
    }
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    kmipkit_ttlv_structure_release(payload);
    payload = 0;

    if (kmipkit_query_async_requests_request_with_correlation_values(
            query, correlation_filters, 3, &next_query) != 0) {
        goto cleanup;
    }
    kmipkit_query_async_requests_request_release(query);
    query = next_query;
    next_query = 0;
    if (kmipkit_query_async_requests_request_with_operations(
            query, operation_filters, 2, &next_query) != 0) {
        goto cleanup;
    }
    kmipkit_query_async_requests_request_release(query);
    query = next_query;
    next_query = 0;
    if (kmipkit_query_async_requests_request_to_ttlv_payload(query, &payload) != 0 ||
        kmipkit_ttlv_structure_view(payload, &view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != 2 ||
        !async_query_payload_matches_filters(
            payload,
            correlation_filters,
            sizeof(correlation_filters) / sizeof(correlation_filters[0]),
            operation_filters,
            sizeof(operation_filters) / sizeof(operation_filters[0])) ||
        kmipkit_ttlv_structure_view_item_at(view, 1, &item) != 0 ||
        kmipkit_ttlv_item_view_value(item, &value) != 0 ||
        kmipkit_ttlv_value_view_structure(value, &nested) != 0 ||
        kmipkit_ttlv_structure_view_item_count(nested, &count) != 0 || count != 2 ||
        kmipkit_ttlv_structure_view_item_at(nested, 1, &child_item) != 0 ||
        kmipkit_ttlv_item_view_value(child_item, &child_value) != 0 ||
        kmipkit_ttlv_value_view_enumeration(child_value, &raw_operation) != 0 ||
        raw_operation != UINT32_C(0xf1234567)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_cancel_request_release(cancel);
    kmipkit_poll_request_release(poll);
    kmipkit_process_request_release(process);
    kmipkit_query_async_requests_request_release(query);
    kmipkit_query_async_requests_request_release(next_query);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_view_release(nested);
    kmipkit_ttlv_item_view_release(item);
    kmipkit_ttlv_item_view_release(child_item);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_value_view_release(child_value);
    kmipkit_tag_release(tag);
    return result;
}

static int expect_ttlv_decode_error(
    const uint8_t *wire,
    uint64_t wire_length,
    kmipkit_codec_limits_t *limits,
    uint32_t expected_kind,
    int32_t expected_status,
    uint64_t expected_offset) {
    kmipkit_ttlv_decode_error_v1_t error = {0};
    kmipkit_ttlv_item_t *item = (kmipkit_ttlv_item_t *)(uintptr_t)1;
    int32_t status;

    error.struct_size = (uint32_t)sizeof(error);
    error.abi_version = 1;
    status = limits == 0
        ? kmipkit_ttlv_decode(wire, wire_length, &error, &item)
        : kmipkit_ttlv_decode_with_limits(wire, wire_length, limits, &error, &item);
    return status == expected_status && item == 0 && error.kind == expected_kind &&
        error.byte_offset == expected_offset;
}

int kmipkit_client_abi_c_consumer_extra_ffi_paths(void) {
    static const uint8_t correlation[] = {0x00, 0xff, 0x80};
    static const uint8_t bad_integer_length[] = {
        0x42, 0x00, 0x2a, 0x02, 0x00, 0x00, 0x00, 0x01, 0x7f};
    static const uint8_t empty_big_integer[] = {
        0x54, 0x12, 0x34, 0x04, 0x00, 0x00, 0x00, 0x00};
    static const uint8_t missing_padding[] = {
        0x54, 0x12, 0x34, 0x08, 0x00, 0x00, 0x00, 0x01, 0xaa};
    static const uint8_t structure_boundary[] = {
        0x54, 0x12, 0x34, 0x01, 0x00, 0x00, 0x00, 0x08,
        0x54, 0x12, 0x35, 0x02, 0x00, 0x00, 0x00, 0x04};
    static const uint8_t reserved_tag[] = {
        0x42, 0x00, 0x09, 0x05, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00};
    static const uint8_t unallocated_tag[] = {
        0x00, 0x00, 0x01, 0x05, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00};
    static const uint8_t nested_structure[] = {
        0x54, 0x12, 0x34, 0x01, 0x00, 0x00, 0x00, 0x08,
        0x54, 0x12, 0x35, 0x01, 0x00, 0x00, 0x00, 0x00};
    kmipkit_cancel_request_t *cancel = (kmipkit_cancel_request_t *)(uintptr_t)1;
    kmipkit_poll_request_t *poll = (kmipkit_poll_request_t *)(uintptr_t)1;
    kmipkit_process_request_t *process = (kmipkit_process_request_t *)(uintptr_t)1;
    kmipkit_query_async_requests_request_t *query = 0;
    kmipkit_query_async_requests_request_t *filtered =
        (kmipkit_query_async_requests_request_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *payload = (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    kmipkit_codec_limits_t *depth_limits = 0;
    kmipkit_codec_limits_t *element_limits = 0;
    kmipkit_byte_span_v1_t bad_span = {0, 1};
    kmipkit_byte_span_v1_t oversized_span = {correlation, UINT64_MAX};
    kmipkit_byte_span_v1_t borrowed = {correlation, sizeof(correlation)};
    uint32_t operation = UINT32_C(0xf1234567);
    int result = 0;

    if (kmipkit_cancel_request_create(0, 1, &cancel) != T076_ERROR_INVALID_INPUT || cancel != 0) {
        goto cleanup;
    }
    if (kmipkit_poll_request_create(correlation, UINT64_MAX, &poll) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || poll != 0) {
        goto cleanup;
    }
    if (kmipkit_process_request_create(0, 1, &process) != T076_ERROR_INVALID_INPUT || process != 0) {
        goto cleanup;
    }
    if (kmipkit_query_async_requests_request_create(&query) != 0 || query == 0) {
        goto cleanup;
    }
    if (kmipkit_query_async_requests_request_with_correlation_values(
            query, 0, 1, &filtered) != T076_ERROR_INVALID_INPUT || filtered != 0 ||
        kmipkit_query_async_requests_request_with_correlation_values(
            query, &bad_span, 1, &filtered) != T076_ERROR_INVALID_INPUT || filtered != 0 ||
        kmipkit_query_async_requests_request_with_correlation_values(
            query, &oversized_span, 1, &filtered) != KMIPKIT_ERROR_RESOURCE_LIMIT || filtered != 0 ||
        kmipkit_query_async_requests_request_with_operations(
            query, 0, 1, &filtered) != T076_ERROR_INVALID_INPUT || filtered != 0 ||
        kmipkit_query_async_requests_request_with_operations(
            (kmipkit_query_async_requests_request_t *)poll, &operation, 1, &filtered) !=
            T076_ERROR_INVALID_INPUT || filtered != 0 ||
        kmipkit_query_async_requests_request_to_ttlv_payload(
            (kmipkit_query_async_requests_request_t *)poll, &payload) !=
            T076_ERROR_INVALID_INPUT || payload != 0) {
        goto cleanup;
    }
    if (kmipkit_cancel_request_create(correlation, sizeof(correlation), &cancel) != 0 ||
        kmipkit_poll_request_create(correlation, sizeof(correlation), &poll) != 0 ||
        kmipkit_cancel_request_asynchronous_correlation_value(
            (kmipkit_cancel_request_t *)poll, &borrowed) != T076_ERROR_INVALID_INPUT ||
        kmipkit_poll_request_to_ttlv_payload((kmipkit_poll_request_t *)cancel, &payload) !=
            T076_ERROR_INVALID_INPUT || payload != 0) {
        goto cleanup;
    }

    if (!expect_ttlv_decode_error(
            bad_integer_length, sizeof(bad_integer_length), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_INVALID_ITEM_LENGTH,
            KMIPKIT_ERROR_PROTOCOL, 4) ||
        !expect_ttlv_decode_error(
            empty_big_integer, sizeof(empty_big_integer), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_EMPTY_BIG_INTEGER,
            KMIPKIT_ERROR_PROTOCOL, 4) ||
        !expect_ttlv_decode_error(
            missing_padding, sizeof(missing_padding), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_INVALID_PADDING_EXTENT,
            KMIPKIT_ERROR_PROTOCOL, sizeof(missing_padding)) ||
        !expect_ttlv_decode_error(
            structure_boundary, sizeof(structure_boundary), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_STRUCTURE_BOUNDARY,
            KMIPKIT_ERROR_PROTOCOL, 8) ||
        !expect_ttlv_decode_error(
            reserved_tag, sizeof(reserved_tag), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_RESERVED_TAG,
            KMIPKIT_ERROR_PROTOCOL, 0) ||
        !expect_ttlv_decode_error(
            unallocated_tag, sizeof(unallocated_tag), 0,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_UNALLOCATED_TAG,
            KMIPKIT_ERROR_PROTOCOL, 0)) {
        goto cleanup;
    }
    if (kmipkit_codec_limits_create(128, 1, 8, &depth_limits) != 0 || depth_limits == 0 ||
        !expect_ttlv_decode_error(
            nested_structure, sizeof(nested_structure), depth_limits,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_STRUCTURE_DEPTH_EXCEEDED,
            KMIPKIT_ERROR_RESOURCE_LIMIT, 8) ||
        kmipkit_codec_limits_create(128, 64, 1, &element_limits) != 0 || element_limits == 0 ||
        !expect_ttlv_decode_error(
            nested_structure, sizeof(nested_structure), element_limits,
            KMIPKIT_TTLV_DECODE_ERROR_KIND_ELEMENT_LIMIT_EXCEEDED,
            KMIPKIT_ERROR_RESOURCE_LIMIT, 8)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_cancel_request_release(cancel);
    kmipkit_poll_request_release(poll);
    kmipkit_process_request_release(process);
    kmipkit_query_async_requests_request_release(query);
    kmipkit_query_async_requests_request_release(filtered);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_codec_limits_release(depth_limits);
    kmipkit_codec_limits_release(element_limits);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_register_response(
    kmipkit_client_batch_item_response_t *item,
    const uint8_t *expected_identifier,
    uint64_t expected_identifier_length) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {UINT32_C(0x420094), UINT32_C(0x541234)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = 0;
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    uint8_t payload_type = 0;
    int result = 0;

    if (item == 0 || expected_identifier == 0) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_register(
            item, &present, &identifier) != 0 || present != 1 ||
        !response_identifier_matches(
            &identifier, expected_identifier, expected_identifier_length) ||
        kmipkit_client_batch_item_response_register_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }

    /* The TTLV copy must outlive the response-item handle that produced it. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        !check_view_tags(
            view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(view, 3, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_type(payload_item, &payload_type) != 0 ||
        payload_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(
            payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0])))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_item_response_release(item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    return result;
}

int kmipkit_client_abi_c_consumer_build_revoke_request(void)
{
    static const uint8_t audit_message[] = "C revoke audit";
    kmipkit_revocation_reason_t *reason = 0;
    kmipkit_revoke_request_t *request = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    if (kmipkit_revocation_reason_create(
            UINT32_MAX, 1, audit_message,
            (uint64_t)(sizeof(audit_message) - 1), &reason) != 0 ||
        reason == 0) {
        goto cleanup;
    }
    if (kmipkit_revoke_request_create(0, reason, 0, 0, &request) != 0 ||
        request == 0) {
        reason = 0;
        goto cleanup;
    }
    reason = 0;
    if (kmipkit_client_batch_item_revoke(request, &item) != 0 || item == 0) {
        request = 0;
        goto cleanup;
    }
    request = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_revoke_request_release(request);
    kmipkit_revocation_reason_release(reason);
    return result;
}

int kmipkit_client_abi_c_consumer_build_set_constraints_request(
    kmipkit_client_batch_item_t **out_item) {
    kmipkit_ttlv_structure_t *constraints = 0;
    kmipkit_client_batch_item_t *item = 0;
    int result = 0;

    if (out_item == 0) return 0;
    *out_item = 0;
    if (kmipkit_ttlv_structure_create(&constraints) != 0 || constraints == 0 ||
        kmipkit_client_batch_item_set_constraints(constraints, &item) != 0 ||
        item == 0) {
        goto cleanup;
    }

    /* The request owns its copy after the caller releases the input structure. */
    kmipkit_ttlv_structure_release(constraints);
    constraints = 0;
    *out_item = item;
    item = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(constraints);
    kmipkit_client_batch_item_release(item);
    return result;
}

int kmipkit_client_abi_c_consumer_build_set_defaults_requests(
    kmipkit_client_batch_item_t **out_omitted,
    kmipkit_client_batch_item_t **out_present_empty) {
    kmipkit_ttlv_structure_t *defaults_information = 0;
    kmipkit_client_batch_item_t *omitted = 0;
    kmipkit_client_batch_item_t *present_empty = 0;

    if (out_omitted == 0 || out_present_empty == 0 ||
        out_omitted == (void *)out_present_empty) {
        return 0;
    }
    *out_omitted = 0;
    *out_present_empty = 0;
    if (kmipkit_client_batch_item_set_defaults(0, &omitted) != 0 || omitted == 0 ||
        kmipkit_ttlv_structure_create(&defaults_information) != 0 ||
        defaults_information == 0 ||
        kmipkit_client_batch_item_set_defaults(defaults_information, &present_empty) != 0 ||
        present_empty == 0) {
        goto cleanup;
    }

    /* Set Defaults copies a present Structure and never consumes the caller's handle. */
    kmipkit_ttlv_structure_release(defaults_information);
    defaults_information = 0;
    *out_omitted = omitted;
    *out_present_empty = present_empty;
    return 1;

cleanup:
    kmipkit_ttlv_structure_release(defaults_information);
    kmipkit_client_batch_item_release(omitted);
    kmipkit_client_batch_item_release(present_empty);
    return 0;
}

int kmipkit_client_abi_c_consumer_build_get_attribute_list_request(void) {
    kmipkit_client_batch_item_t *item = 0;
    const int32_t status = kmipkit_client_batch_item_get_attribute_list(0, &item);
    if (status != 0 || item == 0) {
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    return 1;
}

int kmipkit_client_abi_c_consumer_validate_get_attribute_list_response(
    kmipkit_client_batch_item_response_t *item,
    uint8_t expected_present) {
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    const int32_t status = kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv(
        item, &present, &response_ttlv);
    if (status != 0 || present != expected_present) {
        if (response_ttlv != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
        }
        return 0;
    }
    if (expected_present != 0 && response_ttlv == 0) {
        return 0;
    }
    if (response_ttlv != 0) {
        kmipkit_ttlv_structure_release(response_ttlv);
    }
    return 1;
}

int kmipkit_client_abi_c_consumer_validate_typed_get_attribute_list_response(
    kmipkit_client_batch_item_response_t *item,
    uint8_t expected_present) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_attribute_reference_v1_t references[3] = {{0}};
    uint8_t present = UINT8_C(9);
    uint64_t count = UINT64_MAX;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    for (uint64_t index = 0; index < UINT64_C(3); ++index) {
        references[index].struct_size = (uint32_t)sizeof(references[index]);
        references[index].abi_version = 1;
    }
    if (item == 0 || expected_present > 1 ||
        kmipkit_client_batch_item_response_get_attribute_list(
            item, &present, &identifier, 0, 0, &count) != 0 ||
        present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return count == 0 && identifier.kind == 0;
    }
    if (count != UINT64_C(3) || identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_length != UINT64_C(8) || identifier.text_data == 0 ||
        memcmp(identifier.text_data, "object-7", 8) != 0) {
        return 0;
    }
    if (kmipkit_client_batch_item_response_get_attribute_list(
            item, &present, &identifier, references, UINT64_C(3), &count) != 0 ||
        present != 1 || count != UINT64_C(3) ||
        references[0].kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG ||
        references[0].tag_value != UINT32_C(0x00540001) ||
        references[1].kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME ||
        references[2].kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME) {
        return 0;
    }
    return references[1].vendor_identification_length == UINT64_C(6) &&
        memcmp(references[1].vendor_identification_data, "vendor", 6) == 0 &&
        references[1].attribute_name_length == UINT64_C(11) &&
        memcmp(references[1].attribute_name_data, "Opaque.Name", 11) == 0 &&
        references[2].vendor_identification_length == UINT64_C(6) &&
        memcmp(references[2].vendor_identification_data, "vendor", 6) == 0 &&
        references[2].attribute_name_length == UINT64_C(11) &&
        memcmp(references[2].attribute_name_data, "Opaque.Name", 11) == 0;
}

int kmipkit_client_abi_c_consumer_build_get_attributes_request(void) {
    static const uint8_t vendor[] = "KMIPKit";
    static const uint8_t name[] = "Comment";
    kmipkit_attribute_reference_v1_t references[2] = {{0}};
    kmipkit_client_batch_item_t *item = 0;
    references[0].struct_size = (uint32_t)sizeof(references[0]);
    references[0].abi_version = 1;
    references[0].kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
    references[0].tag_value = UINT32_C(0x42002f);
    references[1].struct_size = (uint32_t)sizeof(references[1]);
    references[1].abi_version = 1;
    references[1].kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
    references[1].vendor_identification_data = vendor;
    references[1].vendor_identification_length = UINT64_C(7);
    references[1].attribute_name_data = name;
    references[1].attribute_name_length = UINT64_C(7);
    if (kmipkit_client_batch_item_get_attributes(0, references, UINT64_C(2), &item) != 0 ||
        item == 0) {
        return 0;
    }
    kmipkit_client_batch_item_release(item);
    return 1;
}

int kmipkit_client_abi_c_consumer_validates_standalone_attribute_requests(void) {
    static const uint8_t identifier_bytes[] = "attribute-object";
    static const uint8_t vendor_bytes[] = "KMIPKit";
    static const uint8_t attribute_name_bytes[] = "Comment";
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t identifier_view = {0};
    kmipkit_attribute_reference_v1_t references[2] = {{0}};
    kmipkit_attribute_reference_v1_t reference_view = {0};
    kmipkit_get_attributes_request_t *get_attributes = 0;
    kmipkit_get_attribute_list_request_t *get_attribute_list = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_client_batch_item_t *batch_item = 0;
    uint64_t reference_count = UINT64_MAX;
    uint64_t payload_item_count = UINT64_MAX;
    int valid = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = UINT64_C(16);
    references[0].struct_size = (uint32_t)sizeof(references[0]);
    references[0].abi_version = 1;
    references[0].kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
    references[0].tag_value = UINT32_C(0x0042002f);
    references[1].struct_size = (uint32_t)sizeof(references[1]);
    references[1].abi_version = 1;
    references[1].kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
    references[1].vendor_identification_data = vendor_bytes;
    references[1].vendor_identification_length = UINT64_C(7);
    references[1].attribute_name_data = attribute_name_bytes;
    references[1].attribute_name_length = UINT64_C(7);

    if (kmipkit_get_attributes_request_create(
            &identifier, references, UINT64_C(2), &get_attributes) != 0 ||
        get_attributes == 0) {
        goto cleanup;
    }
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = 1;
    if (kmipkit_get_attributes_request_unique_identifier(
            get_attributes, &identifier_view) != 0 ||
        identifier_view.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier_view.text_data == 0 ||
        identifier_view.text_length != UINT64_C(16) ||
        memcmp(identifier_view.text_data, identifier_bytes, UINT64_C(16)) != 0 ||
        kmipkit_get_attributes_request_attribute_reference_count(
            get_attributes, &reference_count) != 0 ||
        reference_count != UINT64_C(2)) {
        goto cleanup;
    }
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = 1;
    if (kmipkit_get_attributes_request_attribute_reference_at(
            get_attributes, UINT64_C(0), &reference_view) != 0 ||
        reference_view.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG ||
        reference_view.tag_value != UINT32_C(0x0042002f)) {
        goto cleanup;
    }
    reference_view = (kmipkit_attribute_reference_v1_t){0};
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = 1;
    if (kmipkit_get_attributes_request_attribute_reference_at(
            get_attributes, UINT64_C(1), &reference_view) != 0 ||
        reference_view.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME ||
        reference_view.vendor_identification_data == 0 ||
        reference_view.vendor_identification_length != UINT64_C(7) ||
        memcmp(reference_view.vendor_identification_data, vendor_bytes, UINT64_C(7)) != 0 ||
        reference_view.attribute_name_data == 0 ||
        reference_view.attribute_name_length != UINT64_C(7) ||
        memcmp(reference_view.attribute_name_data, attribute_name_bytes, UINT64_C(7)) != 0 ||
        kmipkit_get_attributes_request_to_ttlv_payload(get_attributes, &payload) != 0 ||
        payload == 0 || kmipkit_ttlv_structure_view(payload, &payload_view) != 0 ||
        payload_view == 0 ||
        kmipkit_ttlv_structure_view_item_count(payload_view, &payload_item_count) != 0 ||
        payload_item_count != UINT64_C(3) ||
        kmipkit_client_batch_item_get_attributes_from_request(
            get_attributes, &batch_item) != 0 ||
        batch_item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(batch_item);
    batch_item = 0;
    kmipkit_ttlv_structure_view_release(payload_view);
    payload_view = 0;
    kmipkit_ttlv_structure_release(payload);
    payload = 0;
    kmipkit_get_attributes_request_release(get_attributes);
    get_attributes = 0;

    if (kmipkit_get_attribute_list_request_create(&identifier, &get_attribute_list) != 0 ||
        get_attribute_list == 0) {
        goto cleanup;
    }
    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = 1;
    if (kmipkit_get_attribute_list_request_unique_identifier(
            get_attribute_list, &identifier_view) != 0 ||
        identifier_view.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier_view.text_data == 0 ||
        identifier_view.text_length != UINT64_C(16) ||
        memcmp(identifier_view.text_data, identifier_bytes, UINT64_C(16)) != 0 ||
        kmipkit_get_attribute_list_request_to_ttlv_payload(get_attribute_list, &payload) != 0 ||
        payload == 0 || kmipkit_ttlv_structure_view(payload, &payload_view) != 0 ||
        payload_view == 0 ||
        kmipkit_ttlv_structure_view_item_count(payload_view, &payload_item_count) != 0 ||
        payload_item_count != UINT64_C(1) ||
        kmipkit_client_batch_item_get_attribute_list_from_request(
            get_attribute_list, &batch_item) != 0 ||
        batch_item == 0) {
        goto cleanup;
    }
    valid = 1;

cleanup:
    kmipkit_client_batch_item_release(batch_item);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_get_attributes_request_release(get_attributes);
    kmipkit_get_attribute_list_request_release(get_attribute_list);
    return valid;
}

static kmipkit_ttlv_item_t *attribute_set_test_item(
    uint32_t raw_tag, uint8_t value_kind, kmipkit_codec_limits_t *limits) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *item = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status != 0 || raw == 0 || kmipkit_ttlv_raw_tag_try_checked(raw, &tag) != 0 ||
        tag == 0) {
        goto cleanup;
    }
    if (value_kind == 1) {
        static const uint8_t name[] = "Name";
        status = kmipkit_ttlv_value_text_string(limits, name, UINT64_C(4), &value);
    } else if (value_kind == 2) {
        status = kmipkit_ttlv_value_enumeration(UINT32_C(0xffffffff), &value);
    } else {
        status = kmipkit_ttlv_value_integer(INT32_C(7), &value);
    }
    if (status != 0 || value == 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, &item) != 0 || item == 0) {
        goto cleanup;
    }
    value = 0;
cleanup:
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return item;
}

int kmipkit_client_abi_c_consumer_validates_attribute_set(void) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *source = 0;
    kmipkit_ttlv_item_t *first = 0;
    kmipkit_ttlv_item_t *second = 0;
    kmipkit_ttlv_item_t *invalid = 0;
    kmipkit_attribute_set_t *attributes = 0;
    kmipkit_ttlv_structure_t *copy = 0;
    kmipkit_ttlv_structure_t *consumed = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *item_view = 0;
    kmipkit_tag_t *tag = 0;
    uint64_t count = UINT64_MAX;
    uint8_t is_empty = UINT8_MAX;
    uint32_t raw_tag = 0;
    int valid = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == 0 ||
        kmipkit_ttlv_structure_create(&source) != 0 || source == 0) {
        goto cleanup;
    }
    /* KMIP Specification v2.1 §§4.12–4.13 define Contact Information as Text String and
     * Cryptographic Algorithm as Enumeration; the latter is reused with Integer as a negative
     * type-mismatch case. */
    first = attribute_set_test_item(UINT32_C(0x00420022), 1, limits);
    kmipkit_ttlv_structure_t *updated = 0;
    if (first == 0 || kmipkit_ttlv_structure_with_item(source, first, limits, &updated) != 0 ||
        updated == 0) {
        goto cleanup;
    }
    source = 0;
    source = updated;
    first = 0;
    if (kmipkit_attribute_set_create(source, &attributes) != 0 || attributes == 0 ||
        kmipkit_attribute_set_len(attributes, &count) != 0 || count != UINT64_C(1) ||
        kmipkit_attribute_set_is_empty(attributes, &is_empty) != 0 || is_empty != 0) {
        goto cleanup;
    }
    second = attribute_set_test_item(UINT32_C(0x00420028), 2, limits);
    if (second == 0 || kmipkit_attribute_set_try_push(attributes, second) != 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(second);
    second = 0;

    invalid = attribute_set_test_item(UINT32_C(0x00420028), 3, limits);
    if (invalid == 0 ||
        kmipkit_attribute_set_try_push(attributes, invalid) != T076_ERROR_INVALID_INPUT ||
        kmipkit_attribute_set_len(attributes, &count) != 0 || count != UINT64_C(2)) {
        goto cleanup;
    }

    if (kmipkit_attribute_set_as_items(attributes, &copy) != 0 || copy == 0 ||
        kmipkit_ttlv_structure_view(copy, &view) != 0 || view == 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != UINT64_C(2) ||
        kmipkit_ttlv_structure_view_item_at(view, 0, &item_view) != 0 || item_view == 0 ||
        kmipkit_ttlv_item_view_tag(item_view, &tag) != 0 || tag == 0 ||
        kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 || raw_tag != UINT32_C(0x00420022)) {
        goto cleanup;
    }
    kmipkit_tag_release(tag);
    tag = 0;
    kmipkit_ttlv_item_view_release(item_view);
    item_view = 0;
    if (kmipkit_ttlv_structure_view_item_at(view, 1, &item_view) != 0 || item_view == 0 ||
        kmipkit_ttlv_item_view_tag(item_view, &tag) != 0 || tag == 0 ||
        kmipkit_ttlv_tag_value(tag, &raw_tag) != 0 || raw_tag != UINT32_C(0x00420028)) {
        goto cleanup;
    }
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (kmipkit_attribute_set_into_items(attributes, &consumed) != 0 || consumed == 0) {
        goto cleanup;
    }
    attributes = 0;
    valid = kmipkit_ttlv_structure_view(consumed, &view) == 0 && view != 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0 && count == UINT64_C(2);
cleanup:
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item_view);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(consumed);
    kmipkit_ttlv_structure_release(copy);
    kmipkit_attribute_set_release(attributes);
    kmipkit_ttlv_item_release(invalid);
    kmipkit_ttlv_item_release(second);
    kmipkit_ttlv_item_release(first);
    kmipkit_ttlv_structure_release(source);
    kmipkit_codec_limits_release(limits);
    return valid;
}

int kmipkit_client_abi_c_consumer_validate_get_attributes_response(
    kmipkit_client_batch_item_response_t *item,
    uint8_t expected_present) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    uint8_t present = UINT8_C(9);
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (item == 0 || expected_present > 1 ||
        kmipkit_client_batch_item_response_get_attributes(
            item, &present, &identifier, &attributes) != 0 ||
        present != expected_present) {
        if (attributes != 0) kmipkit_ttlv_structure_release(attributes);
        return 0;
    }
    if (expected_present == 0) {
        return attributes == 0 && identifier.kind == 0 &&
            identifier.text_data == 0 && identifier.text_length == 0;
    }
    const int valid = attributes != 0 &&
        identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == UINT64_C(8) && identifier.text_data != 0 &&
        memcmp(identifier.text_data, "object-7", 8) == 0;
    if (attributes != 0) kmipkit_ttlv_structure_release(attributes);
    return valid;
}

int kmipkit_client_abi_c_consumer_validate_get_attributes_no_match_response(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    uint8_t present = UINT8_C(9);
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (item == 0 || kmipkit_client_batch_item_response_get_attributes(
            item, &present, &identifier, &attributes) != 0) {
        if (attributes != 0) kmipkit_ttlv_structure_release(attributes);
        return 0;
    }
    return present == 1 && attributes == 0 &&
        identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == UINT64_C(8) && identifier.text_data != 0 &&
        memcmp(identifier.text_data, "object-7", 8) == 0;
}

int kmipkit_client_abi_c_consumer_validate_get_attributes_empty_response(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    uint8_t present = UINT8_C(9);
    uint64_t count = UINT64_MAX;
    int valid = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    if (item == 0 ||
        kmipkit_client_batch_item_response_get_attributes(
            item, &present, &identifier, &attributes) != 0 ||
        present != 1 || attributes == 0 ||
        kmipkit_ttlv_structure_view(attributes, &view) != 0 || view == 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &count) != 0 || count != 0) {
        goto cleanup;
    }
    valid = identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == UINT64_C(8) && identifier.text_data != 0 &&
        memcmp(identifier.text_data, "object-7", 8) == 0;
cleanup:
    if (view != 0) kmipkit_ttlv_structure_view_release(view);
    if (attributes != 0) kmipkit_ttlv_structure_release(attributes);
    return valid;
}

int kmipkit_client_abi_c_consumer_validate_set_defaults_response_ttlv(
    kmipkit_client_batch_item_response_t *item, uint8_t expected_present) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t present = UINT8_C(9);
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;
    kmipkit_ttlv_structure_view_t *view = 0;
    int result = 0;

    if (item == 0 || expected_present > 1) goto cleanup;
    if (kmipkit_client_batch_item_response_set_defaults_ttlv(
            item, &present, &response_ttlv) != 0 ||
        present != expected_present ||
        (expected_present == 1 && response_ttlv == 0) ||
        (expected_present == 0 && response_ttlv != 0)) {
        goto cleanup;
    }

    /* The caller-owned copy remains valid after releasing its source response. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (expected_present == 1 &&
        (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
         !check_view_tags(
             view, response_tags,
             (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_set_defaults_failure_metadata(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    uint8_t ttlv_present = UINT8_C(9);
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;

    if (item == 0) return 0;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    if (kmipkit_client_batch_item_response_metadata_v2(item, &metadata) != 0 ||
        metadata.operation != UINT32_C(0x36) || metadata.outcome_kind != 1 ||
        metadata.result_status != 1 || metadata.has_result_reason != 1 ||
        metadata.result_reason != 6 || metadata.has_result_message != 0 ||
        metadata.result_message_data != 0 || metadata.result_message_length != 0 ||
        kmipkit_client_batch_item_response_set_defaults_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 0 || response_ttlv != 0) {
        return 0;
    }
    return 1;
}

int kmipkit_client_abi_c_consumer_validate_set_constraints_response_ttlv(
    kmipkit_client_batch_item_response_t *item, uint8_t expected_present) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    uint8_t present = UINT8_C(9);
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;
    kmipkit_ttlv_structure_view_t *view = 0;
    int result = 0;

    if (item == 0 || expected_present > 1) goto cleanup;
    if (kmipkit_client_batch_item_response_set_constraints_ttlv(
            item, &present, &response_ttlv) != 0 ||
        present != expected_present ||
        (expected_present == 1 && response_ttlv == 0) ||
        (expected_present == 0 && response_ttlv != 0)) {
        goto cleanup;
    }

    /* Verify that the copied response survives release of its originating item. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (expected_present == 1 &&
        (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
         !check_view_tags(
             view, response_tags,
             (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))))) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_set_constraints_failure_metadata(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_client_batch_item_metadata_v2_t metadata = {0};
    uint8_t ttlv_present = UINT8_C(9);
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;

    if (item == 0) return 0;
    metadata.struct_size = (uint32_t)sizeof(metadata);
    metadata.abi_version = 2;
    if (kmipkit_client_batch_item_response_metadata_v2(item, &metadata) != 0 ||
        metadata.operation != UINT32_C(0x37) || metadata.outcome_kind != 1 ||
        metadata.result_status != 1 || metadata.has_result_reason != 1 ||
        metadata.result_reason != 6 || metadata.has_result_message != 0 ||
        metadata.result_message_data != 0 || metadata.result_message_length != 0 ||
        kmipkit_client_batch_item_response_set_constraints_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 0 || response_ttlv != 0) {
        return 0;
    }
    return 1;
}

int kmipkit_client_abi_c_consumer_reject_set_constraints_ttlv_output_overlap(void) {
    union {
        kmipkit_ttlv_structure_t *response;
        uint8_t bytes[sizeof(kmipkit_ttlv_structure_t *)];
    } outputs;
    outputs.response = (kmipkit_ttlv_structure_t *)(uintptr_t)9;
    const int32_t status = kmipkit_client_batch_item_response_set_constraints_ttlv(
        0, outputs.bytes, &outputs.response);
    return status == T076_ERROR_INVALID_INPUT &&
        outputs.response == (kmipkit_ttlv_structure_t *)(uintptr_t)9;
}

int kmipkit_client_abi_c_consumer_validate_revoke_response(
    kmipkit_client_batch_item_response_t *item,
    const uint8_t *expected_identifier,
    uint64_t expected_identifier_length) {
    static const uint32_t response_tags[] = {
        UINT32_C(0x42005c), UINT32_C(0x420093), UINT32_C(0x42007f), UINT32_C(0x42007c)};
    static const uint32_t payload_tags[] = {UINT32_C(0x420094), UINT32_C(0x541234)};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = 0;
    uint8_t ttlv_present = 0;
    kmipkit_ttlv_structure_t *response_ttlv = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    uint8_t payload_type = 0;
    int result = 0;
    union {
        uint8_t present;
        kmipkit_unique_identifier_v1_t identifier;
    } overlapping_output = {0};

    if (item == 0 || expected_identifier == 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_item_response_revoke(
            item, 0, &identifier) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_revoke(
            0, &present, &identifier) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_revoke(
            item, &overlapping_output.present, &overlapping_output.identifier) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_revoke_ttlv(
            item, 0, &response_ttlv) != T076_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_item_response_revoke_ttlv(
            0, &ttlv_present, &response_ttlv) != T076_ERROR_INVALID_INPUT) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_revoke(
            item, &present, &identifier) != 0 || present != 1 ||
        !response_identifier_matches(
            &identifier, expected_identifier, expected_identifier_length) ||
        kmipkit_client_batch_item_response_revoke_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 1 || response_ttlv == 0) {
        goto cleanup;
    }

    /* The complete TTLV copy remains valid after its response-item owner closes. */
    kmipkit_client_batch_item_response_release(item);
    item = 0;
    if (kmipkit_ttlv_structure_view(response_ttlv, &view) != 0 ||
        !check_view_tags(
            view, response_tags,
            (uint64_t)(sizeof(response_tags) / sizeof(response_tags[0]))) ||
        kmipkit_ttlv_structure_view_item_at(view, 3, &payload_item) != 0 ||
        kmipkit_ttlv_item_view_value(payload_item, &payload_value) != 0 ||
        kmipkit_ttlv_value_view_type(payload_value, &payload_type) != 0 ||
        payload_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
        kmipkit_ttlv_value_view_structure(payload_value, &payload_view) != 0 ||
        !check_view_tags(
            payload_view, payload_tags,
            (uint64_t)(sizeof(payload_tags) / sizeof(payload_tags[0]))) ) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_absent_revoke_response(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);
    uint8_t ttlv_present = UINT8_C(9);
    kmipkit_ttlv_structure_t *response_ttlv =
        (kmipkit_ttlv_structure_t *)(uintptr_t)9;

    if (item == 0) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = UINT32_C(9);
    identifier.reserved = UINT32_C(9);
    identifier.text_data = (const uint8_t *)(uintptr_t)9;
    identifier.text_length = UINT64_C(9);
    identifier.enumeration_value = UINT32_C(9);
    identifier.integer_value = INT64_C(9);

    if (kmipkit_client_batch_item_response_revoke(
            item, &present, &identifier) != 0 || present != 0 ||
        identifier.kind != 0 || identifier.reserved != 0 ||
        identifier.text_data != 0 || identifier.text_length != 0 ||
        identifier.enumeration_value != 0 || identifier.integer_value != 0 ||
        kmipkit_client_batch_item_response_revoke_ttlv(
            item, &ttlv_present, &response_ttlv) != 0 ||
        ttlv_present != 0 || response_ttlv != 0) {
        return 0;
    }
    return 1;
}

int kmipkit_client_abi_c_consumer_rejects_register_output_aliases(
    kmipkit_client_batch_item_response_t *item) {
    kmipkit_unique_identifier_v1_t identifier = {0};
    union {
        uint8_t flag;
        kmipkit_ttlv_structure_t *handle;
    } ttlv_alias;
    kmipkit_unique_identifier_v1_t identifier_before;
    uint8_t ttlv_alias_before[sizeof(ttlv_alias)];
    int status;

    if (item == 0) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = UINT32_C(9);
    memcpy(&identifier_before, &identifier, sizeof(identifier));
    status = kmipkit_client_batch_item_response_register(
        item, (uint8_t *)(uintptr_t)&identifier.kind, &identifier);
    if (status != T076_ERROR_INVALID_INPUT ||
        memcmp(&identifier, &identifier_before, sizeof(identifier)) != 0) {
        return 0;
    }

    memset(&ttlv_alias, 0, sizeof(ttlv_alias));
    ttlv_alias.flag = 9;
    memcpy(ttlv_alias_before, &ttlv_alias, sizeof(ttlv_alias));
    status = kmipkit_client_batch_item_response_register_ttlv(
        item, &ttlv_alias.flag, &ttlv_alias.handle);
    if (status != T076_ERROR_INVALID_INPUT ||
        memcmp(&ttlv_alias, ttlv_alias_before, sizeof(ttlv_alias)) != 0) {
        if (status == 0 && ttlv_alias.handle != 0 &&
            ttlv_alias.handle != (kmipkit_ttlv_structure_t *)(uintptr_t)9) {
            kmipkit_ttlv_structure_release(ttlv_alias.handle);
        }
        return 0;
    }
    return 1;
}

static int add_attribute_append_value(
    kmipkit_codec_limits_t *limits,
    kmipkit_ttlv_structure_t **structure,
    uint32_t raw_tag,
    kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *updated = 0;
    int result = 0;

    if (limits == 0 || structure == 0 || *structure == 0 || value == 0 || *value == 0) {
        return 0;
    }
    if (kmipkit_ttlv_raw_tag_create(raw_tag, &raw) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, *value, limits, &item) != 0) {
        goto cleanup;
    }
    *value = 0;
    if (kmipkit_ttlv_structure_with_item(*structure, item, limits, &updated) != 0) {
        goto cleanup;
    }
    *structure = 0;
    item = 0;
    *structure = updated;
    updated = 0;
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return result;
}

int kmipkit_client_abi_c_consumer_build_add_attribute_request(
    kmipkit_client_batch_item_t **out_request) {
    static const uint8_t identifier_text[] = "object-7";
    static const uint8_t name_text[] = "new name";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *name_fields = 0;
    kmipkit_ttlv_structure_t *borrowed_input_check = 0;
    kmipkit_ttlv_structure_t *updated_input_check = 0;
    kmipkit_ttlv_structure_view_t *input_view = 0;
    kmipkit_ttlv_item_view_t *input_item_view = 0;
    kmipkit_raw_tag_t *name_raw_tag = 0;
    kmipkit_tag_t *name_tag = 0;
    kmipkit_tag_t *input_item_tag = 0;
    kmipkit_ttlv_value_t *name_value = 0;
    kmipkit_ttlv_item_t *name_item = 0;
    kmipkit_client_batch_item_t *request = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint64_t input_count = 0;
    uint32_t input_tag = 0;
    int result = 0;

    if (out_request == 0) return 0;
    *out_request = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_structure_create(&name_fields) != 0) {
        goto cleanup;
    }
    if (kmipkit_ttlv_value_text_string(
            limits, name_text, (uint64_t)(sizeof(name_text) - 1), &name_value) != 0 ||
        !add_attribute_append_value(limits, &name_fields, UINT32_C(0x4200a5), &name_value) ||
        kmipkit_ttlv_value_enumeration(1, &name_value) != 0 ||
        !add_attribute_append_value(limits, &name_fields, UINT32_C(0x4200a6), &name_value) ||
        kmipkit_ttlv_value_structure(name_fields, limits, &name_value) != 0) {
        goto cleanup;
    }
    name_fields = 0;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x42000a), &name_raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(name_raw_tag, &name_tag) != 0 ||
        kmipkit_ttlv_item_create(name_tag, name_value, limits, &name_item) != 0) {
        goto cleanup;
    }
    name_value = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    if (kmipkit_client_batch_item_add_attribute(&identifier, name_item, &request) != 0 ||
        request == 0 || kmipkit_ttlv_structure_create(&borrowed_input_check) != 0 ||
        kmipkit_ttlv_structure_with_item(
                borrowed_input_check, name_item, limits, &updated_input_check) != 0) {
        goto cleanup;
    }
    borrowed_input_check = 0;
    borrowed_input_check = updated_input_check;
    updated_input_check = 0;
    name_item = 0;
    if (kmipkit_ttlv_structure_view(borrowed_input_check, &input_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(input_view, &input_count) != 0 ||
        input_count != UINT64_C(1) ||
        kmipkit_ttlv_structure_view_item_at(input_view, 0, &input_item_view) != 0 ||
        kmipkit_ttlv_item_view_tag(input_item_view, &input_item_tag) != 0 ||
        kmipkit_ttlv_tag_value(input_item_tag, &input_tag) != 0 ||
        input_tag != UINT32_C(0x42000a)) {
        goto cleanup;
    }
    *out_request = request;
    request = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(request);
    kmipkit_ttlv_item_view_release(input_item_view);
    kmipkit_ttlv_structure_view_release(input_view);
    kmipkit_ttlv_structure_release(updated_input_check);
    kmipkit_ttlv_structure_release(borrowed_input_check);
    kmipkit_ttlv_item_release(name_item);
    kmipkit_tag_release(input_item_tag);
    kmipkit_ttlv_value_release(name_value);
    kmipkit_tag_release(name_tag);
    kmipkit_raw_tag_release(name_raw_tag);
    kmipkit_ttlv_structure_release(name_fields);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_add_attribute_response(
    kmipkit_client_batch_item_response_t *response, uint8_t expected_present) {
    static const uint8_t expected_identifier[] = "object-7";
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);

    if (response == 0 || expected_present > 1) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_add_attribute(
            response, &present, &identifier) != 0 || present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return identifier.kind == 0 && identifier.text_data == 0 &&
            identifier.text_length == 0;
    }
    return identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == (uint64_t)(sizeof(expected_identifier) - 1) &&
        identifier.text_data != 0 &&
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) == 0;
}

static int modify_attribute_create_item(
    kmipkit_codec_limits_t *limits,
    const uint8_t *text,
    uint64_t text_length,
    kmipkit_ttlv_item_t **out_item) {
    kmipkit_ttlv_structure_t *fields = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    int result = 0;

    if (limits == 0 || text == 0 || out_item == 0) return 0;
    *out_item = 0;
    if (kmipkit_ttlv_structure_create(&fields) != 0 ||
        kmipkit_ttlv_value_text_string(limits, text, text_length, &value) != 0 ||
        !add_attribute_append_value(limits, &fields, UINT32_C(0x4200c0), &value) ||
        kmipkit_ttlv_value_enumeration(UINT32_C(1), &value) != 0 ||
        !add_attribute_append_value(limits, &fields, UINT32_C(0x4200c1), &value) ||
        kmipkit_ttlv_value_structure(fields, limits, &value) != 0) {
        goto cleanup;
    }
    fields = 0;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x4200bf), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, out_item) != 0) {
        goto cleanup;
    }
    value = 0;
    result = 1;

cleanup:
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(fields);
    return result;
}

int kmipkit_client_abi_c_consumer_build_modify_attribute(
    kmipkit_client_batch_item_t **out_request) {
    static const uint8_t identifier_text[] = "object-7";
    static const uint8_t current_text[] = "selected current value";
    static const uint8_t new_text[] = "replacement value";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_item_t *current_attribute = 0;
    kmipkit_ttlv_item_t *new_attribute = 0;
    kmipkit_ttlv_structure_t *current_input_check = 0;
    kmipkit_ttlv_structure_t *updated_current_check = 0;
    kmipkit_ttlv_structure_t *new_input_check = 0;
    kmipkit_ttlv_structure_t *updated_new_check = 0;
    kmipkit_client_batch_item_t *request = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    int result = 0;

    if (out_request == 0) return 0;
    *out_request = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        !modify_attribute_create_item(
            limits, current_text, (uint64_t)(sizeof(current_text) - 1),
            &current_attribute) ||
        !modify_attribute_create_item(
            limits, new_text, (uint64_t)(sizeof(new_text) - 1), &new_attribute)) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    if (kmipkit_client_batch_item_modify_attribute(
            &identifier, current_attribute, new_attribute, &request) != 0 ||
        request == 0 ||
        kmipkit_ttlv_structure_create(&current_input_check) != 0) {
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_with_item(
            current_input_check, current_attribute, limits, &updated_current_check) != 0) {
        goto cleanup;
    }
    current_input_check = 0;
    if (kmipkit_ttlv_structure_create(&new_input_check) != 0 ||
        kmipkit_ttlv_structure_with_item(
            new_input_check, new_attribute, limits, &updated_new_check) != 0) {
        goto cleanup;
    }
    new_input_check = 0;
    *out_request = request;
    request = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(request);
    kmipkit_ttlv_structure_release(updated_new_check);
    kmipkit_ttlv_structure_release(new_input_check);
    kmipkit_ttlv_structure_release(updated_current_check);
    kmipkit_ttlv_structure_release(current_input_check);
    kmipkit_ttlv_item_release(new_attribute);
    kmipkit_ttlv_item_release(current_attribute);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_modify_attribute_response(
    kmipkit_client_batch_item_response_t *response, uint8_t expected_present) {
    static const uint8_t expected_identifier[] = "object-7";
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);

    if (response == 0 || expected_present > 1) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_modify_attribute(
            response, &present, &identifier) != 0 || present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return identifier.kind == 0 && identifier.text_data == 0 &&
            identifier.text_length == 0;
    }
    return identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == (uint64_t)(sizeof(expected_identifier) - 1) &&
        identifier.text_data != 0 &&
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) == 0;
}

int kmipkit_client_abi_c_consumer_build_set_attribute_request(
    kmipkit_client_batch_item_t **out_request) {
    static const uint8_t identifier_text[] = "object-7";
    static const uint8_t new_comment[] = "updated comment";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *new_attribute = 0;
    kmipkit_ttlv_structure_t *input_check = 0;
    kmipkit_ttlv_structure_t *updated_check = 0;
    kmipkit_client_batch_item_t *request = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    int result = 0;

    if (out_request == 0) return 0;
    *out_request = 0;
    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        kmipkit_ttlv_value_text_string(
            limits, new_comment, (uint64_t)(sizeof(new_comment) - 1), &value) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x4200fd), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_item_create(tag, value, limits, &new_attribute) != 0) {
        goto cleanup;
    }
    value = 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    if (kmipkit_client_batch_item_set_attribute(
            &identifier, new_attribute, &request) != 0 || request == 0 ||
        kmipkit_ttlv_structure_create(&input_check) != 0 ||
        kmipkit_ttlv_structure_with_item(
            input_check, new_attribute, limits, &updated_check) != 0) {
        goto cleanup;
    }
    input_check = 0;
    *out_request = request;
    request = 0;
    result = 1;

cleanup:
    kmipkit_client_batch_item_release(request);
    kmipkit_ttlv_structure_release(updated_check);
    kmipkit_ttlv_structure_release(input_check);
    kmipkit_ttlv_item_release(new_attribute);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_validate_set_attribute_response(
    kmipkit_client_batch_item_response_t *response, uint8_t expected_present) {
    static const uint8_t expected_identifier[] = "object-7";
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);

    if (response == 0 || expected_present > 1) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_set_attribute(
            response, &present, &identifier) != 0 || present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return identifier.kind == 0 && identifier.text_data == 0 &&
            identifier.text_length == 0;
    }
    return identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == (uint64_t)(sizeof(expected_identifier) - 1) &&
        identifier.text_data != 0 &&
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) == 0;
}

int kmipkit_client_abi_c_consumer_validate_adjust_attribute_response(
    kmipkit_client_batch_item_response_t *response, uint8_t expected_present) {
    static const uint8_t expected_identifier[] = "adjusted-object-1";
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);

    if (response == 0 || expected_present > 1) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_adjust_attribute(
            response, &present, &identifier) != 0 || present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return identifier.kind == 0 && identifier.text_data == 0 &&
            identifier.text_length == 0;
    }
    return identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == (uint64_t)(sizeof(expected_identifier) - 1) &&
        identifier.text_data != 0 &&
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) == 0;
}

int kmipkit_client_abi_c_consumer_build_delete_attribute(
    kmipkit_client_batch_item_t **out_request) {
    kmipkit_client_batch_item_t *request = 0;

    if (out_request == 0) return 0;
    *out_request = 0;
    if (kmipkit_client_batch_item_delete_attribute(0, 0, 0, &request) != 0 ||
        request == 0) {
        kmipkit_client_batch_item_release(request);
        return 0;
    }
    *out_request = request;
    return 1;
}

int kmipkit_client_abi_c_consumer_validate_delete_attribute_response(
    kmipkit_client_batch_item_response_t *response, uint8_t expected_present) {
    static const uint8_t expected_identifier[] = "deleted-object-1";
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t present = UINT8_C(9);

    if (response == 0 || expected_present > 1) return 0;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    if (kmipkit_client_batch_item_response_delete_attribute(
            response, &present, &identifier) != 0 || present != expected_present) {
        return 0;
    }
    if (expected_present == 0) {
        return identifier.kind == 0 && identifier.text_data == 0 &&
            identifier.text_length == 0;
    }
    return identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        identifier.text_length == (uint64_t)(sizeof(expected_identifier) - 1) &&
        identifier.text_data != 0 &&
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) == 0;
}

int kmipkit_client_abi_c_consumer_build_adjust_attribute(
    kmipkit_client_batch_item_t **out_request) {
    kmipkit_attribute_reference_v1_t reference = {0};
    kmipkit_ttlv_value_t *adjustment_value = 0;
    kmipkit_client_batch_item_t *request = 0;
    int32_t status;

    if (out_request == 0) return 0;
    *out_request = 0;
    reference.struct_size = (uint32_t)sizeof(reference);
    reference.abi_version = UINT32_C(1);
    reference.kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
    reference.tag_value = UINT32_C(0x42000a);
    if (kmipkit_ttlv_value_integer(INT32_C(5), &adjustment_value) != 0 ||
        adjustment_value == 0) {
        goto cleanup;
    }
    status = kmipkit_client_batch_item_adjust_attribute(
        0, &reference, UINT32_C(1), adjustment_value, &request);
    kmipkit_ttlv_value_release(adjustment_value);
    adjustment_value = 0;
    if (status != 0 || request == 0) goto cleanup;
    *out_request = request;
    return 1;

cleanup:
    kmipkit_ttlv_value_release(adjustment_value);
    kmipkit_client_batch_item_release(request);
    return 0;
}


int shared_ttlv_decoder_fixture_matches_values_and_errors(
    const uint8_t *wire,
    uint64_t wire_length,
    const uint8_t *truncated_wire,
    uint64_t truncated_length,
    uint64_t message_limit_bytes) {
    kmipkit_ttlv_decode_error_v1_t error = {0};
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_item_view_t *root_view = 0;
    kmipkit_tag_t *root_tag = 0;
    kmipkit_ttlv_value_view_t *root_value = 0;
    kmipkit_ttlv_structure_view_t *structure_view = 0;
    kmipkit_ttlv_item_view_t *enumeration_item = 0;
    kmipkit_ttlv_item_view_t *byte_string_item = 0;
    kmipkit_tag_t *enumeration_tag = 0;
    kmipkit_tag_t *byte_string_tag = 0;
    kmipkit_ttlv_value_view_t *enumeration_value = 0;
    kmipkit_ttlv_value_view_t *byte_string_value = 0;
    kmipkit_codec_limits_t *limits = 0;
    uint32_t root_tag_value = 0;
    uint32_t enumeration_tag_value = 0;
    uint32_t byte_string_tag_value = 0;
    uint32_t actual_enumeration = 0;
    uint64_t item_count = 0;
    uint64_t byte_length = 0;
    uint8_t root_type = 0;
    int result = 0;

    if (wire == 0 || truncated_wire == 0) goto cleanup;
    error.struct_size = (uint32_t)sizeof(error);
    error.abi_version = 1;
    if (kmipkit_ttlv_decode(wire, wire_length, &error, &item) != 0 || item == 0 ||
        error.kind != 0 || error.byte_offset != 0 ||
        kmipkit_ttlv_item_view_from_item(item, &root_view) != 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(item);
    item = 0;
    if (kmipkit_ttlv_item_view_tag(root_view, &root_tag) != 0 ||
        kmipkit_ttlv_tag_value(root_tag, &root_tag_value) != 0 ||
        root_tag_value != UINT32_C(0x00540003) ||
        kmipkit_ttlv_item_view_type(root_view, &root_type) != 0 ||
        root_type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
        kmipkit_ttlv_item_view_value(root_view, &root_value) != 0 ||
        kmipkit_ttlv_value_view_structure(root_value, &structure_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(structure_view, &item_count) != 0 ||
        item_count != UINT64_C(2) ||
        kmipkit_ttlv_structure_view_item_at(structure_view, 0, &enumeration_item) != 0 ||
        kmipkit_ttlv_structure_view_item_at(structure_view, 1, &byte_string_item) != 0 ||
        kmipkit_ttlv_item_view_tag(enumeration_item, &enumeration_tag) != 0 ||
        kmipkit_ttlv_tag_value(enumeration_tag, &enumeration_tag_value) != 0 ||
        enumeration_tag_value != UINT32_C(0x00540001) ||
        kmipkit_ttlv_item_view_tag(byte_string_item, &byte_string_tag) != 0 ||
        kmipkit_ttlv_tag_value(byte_string_tag, &byte_string_tag_value) != 0 ||
        byte_string_tag_value != UINT32_C(0x00540002) ||
        kmipkit_ttlv_item_view_value(enumeration_item, &enumeration_value) != 0 ||
        kmipkit_ttlv_value_view_enumeration(enumeration_value, &actual_enumeration) != 0 ||
        actual_enumeration != UINT32_C(0xe1020304) ||
        kmipkit_ttlv_item_view_value(byte_string_item, &byte_string_value) != 0 ||
        kmipkit_ttlv_value_view_byte_length(byte_string_value, &byte_length) != 0 ||
        byte_length != UINT64_C(3)) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < byte_length; ++index) {
        static const uint8_t expected_bytes[] = {0x80, 0xff, 0x01};
        uint8_t actual_byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(byte_string_value, index, &actual_byte) != 0 ||
            actual_byte != expected_bytes[index]) {
            goto cleanup;
        }
    }

    error.kind = 0;
    error.byte_offset = 0;
    if (kmipkit_ttlv_decode(
            truncated_wire, truncated_length, &error, &item) != KMIPKIT_ERROR_PROTOCOL ||
        item != 0 || error.kind != KMIPKIT_TTLV_DECODE_ERROR_KIND_TRUNCATED_VALUE ||
        error.byte_offset != UINT64_C(8)) {
        goto cleanup;
    }

    if (kmipkit_codec_limits_create(message_limit_bytes, 64, 100000, &limits) != 0 ||
        limits == 0) {
        goto cleanup;
    }
    error.kind = 0;
    error.byte_offset = 0;
    if (kmipkit_ttlv_decode_with_limits(
            wire, wire_length, limits, &error, &item) != KMIPKIT_ERROR_RESOURCE_LIMIT ||
        item != 0 || error.kind != KMIPKIT_TTLV_DECODE_ERROR_KIND_MESSAGE_TOO_LARGE ||
        error.byte_offset != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_item_view_release(root_view);
    kmipkit_tag_release(root_tag);
    kmipkit_ttlv_value_view_release(root_value);
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_item_view_release(enumeration_item);
    kmipkit_ttlv_item_view_release(byte_string_item);
    kmipkit_tag_release(enumeration_tag);
    kmipkit_tag_release(byte_string_tag);
    kmipkit_ttlv_value_view_release(enumeration_value);
    kmipkit_ttlv_value_view_release(byte_string_value);
    kmipkit_codec_limits_release(limits);
    return result;
}


static int profile_span_equals(kmipkit_byte_span_v1_t span, const char *expected) {
    size_t length = strlen(expected);
    return span.data != 0 && span.length == (uint64_t)length &&
        memcmp(span.data, expected, length) == 0;
}

int kmipkit_client_abi_c_consumer_profile_validation_result_contract(void) {
    static const uint8_t baseline[] = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    static const uint64_t profile_id_offsets[] = {0};
    static const uint64_t profile_id_lengths[] = {sizeof(baseline) - 1};
    kmipkit_profile_validation_selection_v1_t selection = {0};
    kmipkit_profile_validation_result_t *result =
        (kmipkit_profile_validation_result_t *)(uintptr_t)1;
    kmipkit_profile_validation_error_t *error = 0;
    kmipkit_profile_validation_error_view_v1_t error_view = {0};
    uint64_t profile_count = 1;
    uint64_t requirement_count = UINT64_MAX;

    selection.struct_size = (uint32_t)sizeof(selection);
    selection.abi_version = 1;
    selection.profile_id_data = baseline;
    selection.profile_id_data_length = sizeof(baseline) - 1;
    selection.profile_id_offsets = profile_id_offsets;
    selection.profile_id_lengths = profile_id_lengths;
    selection.profile_id_count = 1;
    error_view.struct_size = (uint32_t)sizeof(error_view);
    error_view.abi_version = 1;

    if (kmipkit_profile_validation_selection_validate_v1(&selection, &result) !=
            T076_ERROR_INVALID_INPUT || result != 0 ||
        kmipkit_last_error_profile_validation_error_get_v1(&error) != 0 || error == 0 ||
        kmipkit_profile_validation_error_view_get_v1(error, &error_view) != 0 ||
        error_view.source_kind != 1 ||
        !profile_span_equals(
            (kmipkit_byte_span_v1_t){error_view.error_code_data, error_view.error_code_length},
            "profile_unavailable") ||
        !profile_span_equals(
            (kmipkit_byte_span_v1_t){error_view.profile_id_data, error_view.profile_id_length},
            "KMIPKIT-PROFILE-BASELINE-CLIENT")) {
        kmipkit_profile_validation_error_release(error);
        return 0;
    }
    kmipkit_profile_validation_error_release(error);
    if (kmipkit_profile_validation_result_profile_id_count_v1(0, &profile_count) !=
            T076_ERROR_INVALID_INPUT || profile_count != 0 ||
        kmipkit_profile_validation_result_requirement_id_count_v1(0, &requirement_count) !=
            T076_ERROR_INVALID_INPUT || requirement_count != 0 ||
        kmipkit_profile_validation_result_validate_transport_v1(0, 99) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_validation_result_validate_request_v1(0, 0) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_validation_result_validate_request_with_context_v1(0, 0, 0) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_validation_result_validate_response_v1(0, 0, 0) !=
            T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_validation_result_validate_response_with_context_v1(0, 0, 0, 0) !=
            T076_ERROR_INVALID_INPUT) {
        return 0;
    }
    kmipkit_profile_validation_result_release(result);
    return 1;
}

int kmipkit_client_abi_c_consumer_profile_readiness_contract(void) {
    kmipkit_profile_readiness_t *readiness = 0;
    kmipkit_profile_readiness_t *long_id_readiness = 0;
    kmipkit_profile_readiness_t *unknown = (kmipkit_profile_readiness_t *)(uintptr_t)1;
    kmipkit_profile_readiness_view_v1_t readiness_view = {0};
    kmipkit_profile_readiness_blocker_view_v1_t blocker = {0};
    kmipkit_profile_catalog_row_view_v1_t row = {0};
    kmipkit_profile_catalog_row_view_v1_t relation = {0};
    uint64_t profile_count = 0;
    uint64_t requirement_count = 0;
    uint64_t test_case_count = 0;
    uint64_t baseline_requirement_count = 0;
    static const struct {
        uint32_t table;
        uint64_t row;
        uint32_t relation;
    } relation_cases[] = {
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_DIRECT_SOURCE_CLAUSE_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_REQUIREMENT_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_UNRESOLVED_SCOPE_OBLIGATIONS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_DEPENDENCY_PROFILE_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_TRANSPORT_REQUIREMENTS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_ENCODING_REQUIREMENTS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_TEST_CASES},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_REQUIREMENTS, 0,
            KMIPKIT_PROFILE_CATALOG_RELATION_SOURCE_CLAUSE_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_REQUIREMENTS, 0,
            KMIPKIT_PROFILE_CATALOG_RELATION_PROFILE_IDS},
        {KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_REQUIREMENTS, 0,
            KMIPKIT_PROFILE_CATALOG_RELATION_OFFICIAL_TEST_CASE_IDS},
    };
    int result = 0;
    uint8_t long_requirement_id[8 * 1024];
    kmipkit_profile_evidence_record_v1_t long_requirement_record = {0};
    static const uint8_t baseline[] = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    static const uint8_t unknown_id[] = "KMIPKIT-PROFILE-UNKNOWN";
    const kmipkit_byte_span_v1_t baseline_span = {baseline, sizeof(baseline) - 1};
    const kmipkit_byte_span_v1_t unknown_span = {unknown_id, sizeof(unknown_id) - 1};

    readiness_view.struct_size = (uint32_t)sizeof(readiness_view);
    readiness_view.abi_version = 1;
    blocker.struct_size = (uint32_t)sizeof(blocker);
    blocker.abi_version = 1;
    row.struct_size = (uint32_t)sizeof(row);
    row.abi_version = 1;
    relation.struct_size = (uint32_t)sizeof(relation);
    relation.abi_version = 1;
    memset(long_requirement_id, 'x', sizeof(long_requirement_id));
    long_requirement_record.struct_size = (uint32_t)sizeof(long_requirement_record);
    long_requirement_record.abi_version = 1;
    long_requirement_record.requirement_id.data = long_requirement_id;
    long_requirement_record.requirement_id.length = sizeof(long_requirement_id);
    long_requirement_record.passed = 1;
    for (size_t index = 0; index < sizeof(relation_cases) / sizeof(relation_cases[0]); ++index) {
        uint64_t relation_count = 0;
        if (kmipkit_profile_catalog_relation_count_v1(
                relation_cases[index].table, relation_cases[index].row,
                relation_cases[index].relation, &relation_count) != 0) {
            goto cleanup;
        }
        if (relation_count != 0 &&
            (kmipkit_profile_catalog_relation_at_v1(
                relation_cases[index].table, relation_cases[index].row,
                relation_cases[index].relation, 0, &relation) != 0 ||
             relation.row_kind == 0)) {
            goto cleanup;
        }
    }
    if (kmipkit_profile_readiness_evaluate_v1(
            &baseline_span, 0, 0, &readiness) != 0 ||
        readiness == 0 ||
        kmipkit_profile_readiness_view_get_v1(readiness, &readiness_view) != 0 ||
        !profile_span_equals(readiness_view.profile_id, "KMIPKIT-PROFILE-BASELINE-CLIENT") ||
        !profile_span_equals(readiness_view.effective_state, "evidence_incomplete") ||
        readiness_view.target_selected != 1 || readiness_view.runtime_validation_available != 0 ||
        readiness_view.published_claim != 0 || readiness_view.blocker_count == 0 ||
        kmipkit_profile_readiness_blocker_at_v1(
            readiness, readiness_view.blocker_count, &blocker) != T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_catalog_table_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, &profile_count) != 0 || profile_count != 18 ||
        kmipkit_profile_catalog_table_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_REQUIREMENTS, &requirement_count) != 0 || requirement_count != 483 ||
        kmipkit_profile_catalog_table_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_PROFILE_TEST_CASES, &test_case_count) != 0 || test_case_count != 93 ||
        kmipkit_profile_catalog_row_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2, &row) != 0 ||
        !profile_span_equals(row.profile_id, "KMIPKIT-PROFILE-BASELINE-CLIENT") ||
        !profile_span_equals(row.claim_state, "evidence_incomplete") || row.selected_target != 1 ||
        kmipkit_profile_catalog_relation_count_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_REQUIREMENT_IDS, &baseline_requirement_count) != 0 ||
        baseline_requirement_count == 0 ||
        kmipkit_profile_catalog_relation_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, 2,
            KMIPKIT_PROFILE_CATALOG_RELATION_REQUIREMENT_IDS, baseline_requirement_count - 1,
            &relation) != 0 || relation.text_value.length == 0 || relation.text_value.data == 0 ||
        kmipkit_profile_catalog_row_at_v1(
            KMIPKIT_PROFILE_CATALOG_TABLE_CLIENT_PROFILES, profile_count, &row) != T076_ERROR_INVALID_INPUT ||
        kmipkit_profile_readiness_evaluate_v1(
            &unknown_span, 0, 0, &unknown) != 0 ||
        unknown != 0 ||
        kmipkit_profile_readiness_evaluate_v1(
            &baseline_span, &long_requirement_record, 1, &long_id_readiness) != 0 ||
        long_id_readiness == 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_profile_readiness_release(readiness);
    kmipkit_profile_readiness_release(long_id_readiness);
    if (unknown != 0 && unknown != (kmipkit_profile_readiness_t *)(uintptr_t)1) {
        kmipkit_profile_readiness_release(unknown);
    }
    return result;
}

static int profile_span_contains(kmipkit_byte_span_v1_t span, const char *needle) {
    const size_t needle_length = strlen(needle);
    if (span.data == 0 || span.length < needle_length) {
        return 0;
    }
    for (uint64_t offset = 0; offset <= span.length - needle_length; ++offset) {
        if (memcmp(span.data + offset, needle, needle_length) == 0) {
            return 1;
        }
    }
    return 0;
}

int kmipkit_client_abi_c_consumer_profile_validation_result_exercise(
    kmipkit_profile_validation_result_t *result,
    kmipkit_ttlv_structure_t *request) {
    static const uint8_t context_identifier[] = "context-id-must-not-leak";
    kmipkit_profile_object_state_v1_t state = {0};
    kmipkit_profile_validation_context_v1_t context = {0};
    kmipkit_profile_validation_error_t *error = 0;
    kmipkit_profile_validation_error_view_v1_t view = {0};
    kmipkit_byte_span_v1_t profile_id = {0};
    kmipkit_byte_span_v1_t diagnostics[5];
    uint64_t profile_count = 0;
    uint64_t requirement_count = 0;
    size_t index;

    state.struct_size = (uint32_t)sizeof(state);
    state.abi_version = 1;
    state.batch_item_index = 0;
    state.unique_identifier_data = context_identifier;
    state.unique_identifier_length = sizeof(context_identifier) - 1;
    state.known_fields = 1;
    state.boolean_values = 1;
    context.struct_size = (uint32_t)sizeof(context);
    context.abi_version = 1;
    context.object_states = &state;
    context.object_state_count = 1;
    view.struct_size = (uint32_t)sizeof(view);
    view.abi_version = 1;

    if (result == 0 || request == 0 ||
        kmipkit_profile_validation_result_validate_transport_v1(result, 2) != 0 ||
        kmipkit_profile_validation_result_profile_id_count_v1(result, &profile_count) != 0 ||
        profile_count == 0 ||
        kmipkit_profile_validation_result_requirement_id_count_v1(result, &requirement_count) != 0 ||
        requirement_count == 0 ||
        kmipkit_profile_validation_result_profile_id_at_v1(result, 0, &profile_id) != 0 ||
        !profile_span_equals(profile_id, "KMIPKIT-PROFILE-BASELINE-CLIENT")) {
        return 0;
    }

    if (kmipkit_profile_validation_result_validate_request_with_context_v1(
            result, request, &context) != T076_ERROR_INVALID_INPUT ||
        kmipkit_last_error_profile_validation_error_get_v1(&error) != 0 || error == 0 ||
        kmipkit_profile_validation_error_view_get_v1(error, &view) != 0) {
        kmipkit_profile_validation_error_release(error);
        return 0;
    }

    diagnostics[0] =
        (kmipkit_byte_span_v1_t){view.error_code_data, view.error_code_length};
    diagnostics[1] =
        (kmipkit_byte_span_v1_t){view.profile_id_data, view.profile_id_length};
    diagnostics[2] =
        (kmipkit_byte_span_v1_t){view.requirement_id_data, view.requirement_id_length};
    diagnostics[3] =
        (kmipkit_byte_span_v1_t){view.source_id_data, view.source_id_length};
    diagnostics[4] =
        (kmipkit_byte_span_v1_t){view.source_locator_data, view.source_locator_length};
    for (index = 0; index < sizeof(diagnostics) / sizeof(diagnostics[0]); ++index) {
        if (profile_span_contains(diagnostics[index], "request-id-must-not-leak") ||
            profile_span_contains(diagnostics[index], (const char *)context_identifier)) {
            kmipkit_profile_validation_error_release(error);
            return 0;
        }
    }

    kmipkit_profile_validation_error_release(error);
    return 1;
}

static int standalone_attribute_item_has_tag(
    kmipkit_ttlv_item_t *item,
    uint32_t expected_tag) {
    kmipkit_ttlv_item_view_t *view = 0;
    kmipkit_tag_t *tag = 0;
    uint32_t raw_tag = 0;
    int result = 0;

    if (item == 0 || kmipkit_ttlv_item_view_from_item(item, &view) != 0 || view == 0 ||
        kmipkit_ttlv_item_view_tag(view, &tag) != 0 || tag == 0 ||
        kmipkit_ttlv_tag_value(tag, &raw_tag) != 0) {
        goto cleanup;
    }
    result = raw_tag == expected_tag;

cleanup:
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(view);
    return result;
}

static int standalone_attribute_payload_has_item_count(
    kmipkit_ttlv_structure_t *payload,
    uint64_t expected_count) {
    kmipkit_ttlv_structure_view_t *view = 0;
    uint64_t item_count = UINT64_MAX;
    int result = 0;

    if (payload == 0 || kmipkit_ttlv_structure_view(payload, &view) != 0 || view == 0 ||
        kmipkit_ttlv_structure_view_item_count(view, &item_count) != 0) {
        goto cleanup;
    }
    result = item_count == expected_count;

cleanup:
    kmipkit_ttlv_structure_view_release(view);
    return result;
}

int kmipkit_client_abi_c_consumer_exercises_attribute_request_models(void) {
    static const uint8_t identifier_bytes[] = "attribute-model-object";
    static const uint8_t attribute_name[] = "Comment";
    static const uint8_t vendor_name[] = "KMIPKit";
    static const uint8_t attribute_text[] = "standalone attribute value";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_item_t *attribute = 0;
    kmipkit_ttlv_item_t *add_attribute = 0;
    kmipkit_ttlv_item_t *delete_attribute = 0;
    kmipkit_ttlv_item_t *modify_current = 0;
    kmipkit_ttlv_item_t *modify_new = 0;
    kmipkit_ttlv_item_t *set_attribute = 0;
    kmipkit_ttlv_value_t *adjustment_value = 0;
    kmipkit_ttlv_value_t *owned_adjustment_value = 0;
    kmipkit_ttlv_value_view_t *adjustment_value_view = 0;
    kmipkit_add_attribute_request_t *add_request = 0;
    kmipkit_add_attribute_request_t *invalid_add_request =
        (kmipkit_add_attribute_request_t *)(uintptr_t)1;
    kmipkit_adjust_attribute_request_t *adjust_request = 0;
    kmipkit_adjust_attribute_request_t *tag_adjust_request = 0;
    kmipkit_adjust_attribute_request_t *invalid_adjust_request = 0;
    kmipkit_delete_attribute_request_t *delete_request = 0;
    kmipkit_delete_attribute_request_t *empty_delete_request = 0;
    kmipkit_modify_attribute_request_t *modify_request = 0;
    kmipkit_modify_attribute_request_t *modify_without_current = 0;
    kmipkit_set_attribute_request_t *set_request = 0;
    kmipkit_client_batch_item_t *add_batch_item = 0;
    kmipkit_client_batch_item_t *adjust_batch_item = 0;
    kmipkit_client_batch_item_t *tag_adjust_batch_item = 0;
    kmipkit_client_batch_item_t *delete_batch_item = 0;
    kmipkit_client_batch_item_t *modify_batch_item = 0;
    kmipkit_client_batch_item_t *modify_optional_batch_item = 0;
    kmipkit_client_batch_item_t *set_batch_item = 0;
    kmipkit_ttlv_structure_t *invalid_adjust_payload =
        (kmipkit_ttlv_structure_t *)(uintptr_t)1;
    kmipkit_ttlv_structure_t *add_payload = 0;
    kmipkit_ttlv_structure_t *adjust_payload = 0;
    kmipkit_ttlv_structure_t *tag_adjust_payload = 0;
    kmipkit_ttlv_structure_t *delete_payload = 0;
    kmipkit_ttlv_structure_t *empty_delete_payload = 0;
    kmipkit_ttlv_structure_t *modify_payload = 0;
    kmipkit_ttlv_structure_t *modify_optional_payload = 0;
    kmipkit_ttlv_structure_t *set_payload = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t invalid_identifier = {0};
    kmipkit_unique_identifier_v1_t oversized_identifier = {0};
    kmipkit_unique_identifier_v1_t identifier_view = {0};
    kmipkit_attribute_reference_v1_t reference = {0};
    kmipkit_attribute_reference_v1_t tag_reference = {0};
    kmipkit_attribute_reference_v1_t invalid_reference = {0};
    kmipkit_attribute_reference_v1_t oversized_reference = {0};
    kmipkit_attribute_reference_v1_t reference_view = {0};
    kmipkit_adjust_attribute_request_t *invalid_reference_request =
        (kmipkit_adjust_attribute_request_t *)(uintptr_t)1;
    kmipkit_adjust_attribute_request_t *oversized_reference_request =
        (kmipkit_adjust_attribute_request_t *)(uintptr_t)1;
    uint8_t present = UINT8_C(9);
    uint8_t adjustment_value_type = 0;
    uint32_t adjustment_type = UINT32_MAX;
    uint32_t adjustment_value_raw = UINT32_MAX;
    int32_t status = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == 0 ||
        !modify_attribute_create_item(
            limits, attribute_text, sizeof(attribute_text) - 1, &attribute)) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = sizeof(identifier_bytes) - 1;
    invalid_identifier = identifier;
    invalid_identifier.kind = UINT32_C(99);
    if (kmipkit_add_attribute_request_create(
            &invalid_identifier, attribute, &invalid_add_request) !=
            KMIPKIT_ERROR_INVALID_INPUT || invalid_add_request != 0) {
        goto cleanup;
    }
    oversized_identifier = identifier;
    oversized_identifier.text_length = UINT64_MAX;
    if (kmipkit_add_attribute_request_create(
            &oversized_identifier, attribute, &invalid_add_request) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || invalid_add_request != 0) {
        goto cleanup;
    }
    reference.struct_size = (uint32_t)sizeof(reference);
    reference.abi_version = UINT32_C(1);
    reference.kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
    reference.vendor_identification_data = vendor_name;
    reference.vendor_identification_length = sizeof(vendor_name) - 1;
    reference.attribute_name_data = attribute_name;
    reference.attribute_name_length = sizeof(attribute_name) - 1;
    invalid_reference = reference;
    invalid_reference.attribute_name_data = 0;
    invalid_reference.attribute_name_length = 1;
    if (kmipkit_adjust_attribute_request_create(
            &identifier, &invalid_reference, UINT32_C(1), 0,
            &invalid_reference_request) != KMIPKIT_ERROR_INVALID_INPUT ||
        invalid_reference_request != 0) {
        goto cleanup;
    }
    oversized_reference = reference;
    oversized_reference.attribute_name_length = UINT64_MAX;
    if (kmipkit_adjust_attribute_request_create(
            &identifier, &oversized_reference, UINT32_C(1), 0,
            &oversized_reference_request) != KMIPKIT_ERROR_RESOURCE_LIMIT ||
        oversized_reference_request != 0) {
        goto cleanup;
    }

    if (kmipkit_ttlv_value_enumeration(UINT32_C(17), &adjustment_value) != 0 ||
        kmipkit_add_attribute_request_create(&identifier, attribute, &add_request) != 0 ||
        add_request == 0) {
        goto cleanup;
    }
    status = kmipkit_adjust_attribute_request_create(
        &identifier, &reference, UINT32_C(99), 0, &invalid_adjust_request);
    if (status != 0 || invalid_adjust_request == 0) {
        goto cleanup;
    }
    status = kmipkit_adjust_attribute_request_to_ttlv_payload(
        invalid_adjust_request, &invalid_adjust_payload);
    if (status != KMIPKIT_ERROR_INVALID_SCHEMA || invalid_adjust_payload != 0) {
        goto cleanup;
    }
    kmipkit_adjust_attribute_request_release(invalid_adjust_request);
    kmipkit_adjust_attribute_request_release(oversized_reference_request);
    kmipkit_adjust_attribute_request_release(invalid_reference_request);
    invalid_adjust_request = 0;
    if (kmipkit_adjust_attribute_request_create(
            &identifier, &reference, UINT32_C(1), adjustment_value, &adjust_request) != 0 ||
        adjust_request == 0 ||
        kmipkit_delete_attribute_request_create(
            &identifier, attribute, &reference, &delete_request) != 0 || delete_request == 0 ||
        kmipkit_delete_attribute_request_create(0, 0, 0, &empty_delete_request) != 0 ||
        empty_delete_request == 0 ||
        kmipkit_modify_attribute_request_create(
            &identifier, attribute, attribute, &modify_request) != 0 || modify_request == 0 ||
        kmipkit_modify_attribute_request_create(
            0, 0, attribute, &modify_without_current) != 0 || modify_without_current == 0 ||
        kmipkit_set_attribute_request_create(&identifier, attribute, &set_request) != 0 ||
        set_request == 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(attribute);
    attribute = 0;
    kmipkit_ttlv_value_release(adjustment_value);
    adjustment_value = 0;

    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    if (kmipkit_add_attribute_request_unique_identifier(
            add_request, &identifier_view) != 0 ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1)) {
        goto cleanup;
    }
    if (kmipkit_add_attribute_request_unique_identifier(
            (kmipkit_add_attribute_request_t *)adjust_request, &identifier_view) !=
        KMIPKIT_ERROR_INVALID_INPUT ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1) ||
        kmipkit_add_attribute_request_new_attribute(add_request, &add_attribute) != 0 ||
        !standalone_attribute_item_has_tag(add_attribute, UINT32_C(0x4200bf)) ||
        kmipkit_add_attribute_request_to_ttlv_payload(add_request, &add_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(add_payload, UINT64_C(2)) ||
        kmipkit_client_batch_item_add_attribute_from_request(add_request, &add_batch_item) != 0 ||
        add_batch_item == 0) {
        goto cleanup;
    }

    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = UINT32_C(1);
    if (kmipkit_adjust_attribute_request_unique_identifier(
            adjust_request, &identifier_view) != 0 ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1) ||
        kmipkit_adjust_attribute_request_attribute_reference(
            adjust_request, &reference_view) != 0 ||
        reference_view.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME ||
        reference_view.vendor_identification_data == 0 ||
        reference_view.vendor_identification_length != sizeof(vendor_name) - 1 ||
        memcmp(reference_view.vendor_identification_data, vendor_name, sizeof(vendor_name) - 1) != 0 ||
        reference_view.attribute_name_data == 0 ||
        reference_view.attribute_name_length != sizeof(attribute_name) - 1 ||
        memcmp(reference_view.attribute_name_data, attribute_name, sizeof(attribute_name) - 1) != 0 ||
        kmipkit_adjust_attribute_request_adjustment_type(
            adjust_request, &adjustment_type) != 0 || adjustment_type != UINT32_C(1) ||
        kmipkit_adjust_attribute_request_adjustment_value(
            adjust_request, &present, &owned_adjustment_value) != 0 || present != 1 ||
        owned_adjustment_value == 0 ||
        kmipkit_ttlv_value_view(owned_adjustment_value, &adjustment_value_view) != 0 ||
        kmipkit_ttlv_value_view_type(
            adjustment_value_view, &adjustment_value_type) != 0 ||
        adjustment_value_type != KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION ||
        kmipkit_ttlv_value_view_enumeration(
            adjustment_value_view, &adjustment_value_raw) != 0 ||
        adjustment_value_raw != UINT32_C(17) ||
        kmipkit_adjust_attribute_request_to_ttlv_payload(adjust_request, &adjust_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(adjust_payload, UINT64_C(4)) ||
        kmipkit_client_batch_item_adjust_attribute_from_request(
            adjust_request, &adjust_batch_item) != 0 || adjust_batch_item == 0) {
        goto cleanup;
    }
    kmipkit_ttlv_value_view_release(adjustment_value_view);
    adjustment_value_view = 0;
    kmipkit_ttlv_value_release(owned_adjustment_value);
    owned_adjustment_value = 0;

    tag_reference.struct_size = (uint32_t)sizeof(tag_reference);
    tag_reference.abi_version = UINT32_C(1);
    tag_reference.kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG;
    tag_reference.tag_value = UINT32_C(0x42002f);
    if (kmipkit_adjust_attribute_request_create(
            0, &tag_reference, UINT32_C(2), 0, &tag_adjust_request) != 0 ||
        tag_adjust_request == 0) {
        goto cleanup;
    }
    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    identifier_view.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier_view.text_data = identifier_bytes;
    identifier_view.text_length = sizeof(identifier_bytes) - 1;
    reference_view = (kmipkit_attribute_reference_v1_t){0};
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = UINT32_C(1);
    if (kmipkit_adjust_attribute_request_unique_identifier(
            tag_adjust_request, &identifier_view) != 0 ||
        identifier_view.kind != 0 || identifier_view.text_data != 0 ||
        identifier_view.text_length != 0 ||
        kmipkit_adjust_attribute_request_attribute_reference(
            tag_adjust_request, &reference_view) != 0 ||
        reference_view.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG ||
        reference_view.tag_value != UINT32_C(0x42002f) ||
        reference_view.vendor_identification_data != 0 ||
        reference_view.vendor_identification_length != 0 ||
        reference_view.attribute_name_data != 0 || reference_view.attribute_name_length != 0 ||
        kmipkit_adjust_attribute_request_adjustment_type(
            tag_adjust_request, &adjustment_type) != 0 || adjustment_type != UINT32_C(2)) {
        goto cleanup;
    }
    present = UINT8_C(9);
    owned_adjustment_value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_adjust_attribute_request_adjustment_value(
            tag_adjust_request, &present, &owned_adjustment_value) != 0 || present != 0 ||
        owned_adjustment_value != 0 ||
        kmipkit_adjust_attribute_request_to_ttlv_payload(
            tag_adjust_request, &tag_adjust_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(tag_adjust_payload, UINT64_C(2)) ||
        kmipkit_client_batch_item_adjust_attribute_from_request(
            tag_adjust_request, &tag_adjust_batch_item) != 0 || tag_adjust_batch_item == 0) {
        goto cleanup;
    }

    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    reference_view = (kmipkit_attribute_reference_v1_t){0};
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = UINT32_C(1);
    if (kmipkit_delete_attribute_request_unique_identifier(
            delete_request, &identifier_view) != 0 ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1) ||
        kmipkit_delete_attribute_request_current_attribute(
            delete_request, &present, &delete_attribute) != 0 || present != 1 ||
        !standalone_attribute_item_has_tag(delete_attribute, UINT32_C(0x4200bf)) ||
        kmipkit_delete_attribute_request_attribute_reference(
            delete_request, &present, &reference_view) != 0 || present != 1 ||
        reference_view.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME ||
        reference_view.attribute_name_data == 0 ||
        reference_view.attribute_name_length != sizeof(attribute_name) - 1 ||
        memcmp(reference_view.attribute_name_data, attribute_name, sizeof(attribute_name) - 1) != 0 ||
        kmipkit_delete_attribute_request_to_ttlv_payload(delete_request, &delete_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(delete_payload, UINT64_C(3)) ||
        kmipkit_client_batch_item_delete_attribute_from_request(
            delete_request, &delete_batch_item) != 0 || delete_batch_item == 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(delete_attribute);
    delete_attribute = 0;
    present = UINT8_C(9);
    delete_attribute = (kmipkit_ttlv_item_t *)(uintptr_t)1;
    if (kmipkit_delete_attribute_request_current_attribute(
            empty_delete_request, &present, &delete_attribute) != 0 || present != 0 ||
        delete_attribute != 0) {
        goto cleanup;
    }
    present = UINT8_C(9);
    reference_view = (kmipkit_attribute_reference_v1_t){0};
    reference_view.struct_size = (uint32_t)sizeof(reference_view);
    reference_view.abi_version = UINT32_C(1);
    if (kmipkit_delete_attribute_request_attribute_reference(
            empty_delete_request, &present, &reference_view) != 0 || present != 0 ||
        reference_view.kind != 0 || reference_view.attribute_name_data != 0 ||
        kmipkit_delete_attribute_request_to_ttlv_payload(
            empty_delete_request, &empty_delete_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(empty_delete_payload, UINT64_C(0))) {
        goto cleanup;
    }

    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    if (kmipkit_modify_attribute_request_unique_identifier(
            modify_request, &identifier_view) != 0 ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1) ||
        kmipkit_modify_attribute_request_current_attribute(
            modify_request, &present, &modify_current) != 0 || present != 1 ||
        !standalone_attribute_item_has_tag(modify_current, UINT32_C(0x4200bf)) ||
        kmipkit_modify_attribute_request_new_attribute(modify_request, &modify_new) != 0 ||
        !standalone_attribute_item_has_tag(modify_new, UINT32_C(0x4200bf)) ||
        kmipkit_modify_attribute_request_to_ttlv_payload(modify_request, &modify_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(modify_payload, UINT64_C(3)) ||
        kmipkit_client_batch_item_modify_attribute_from_request(
            modify_request, &modify_batch_item) != 0 || modify_batch_item == 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(modify_current);
    modify_current = 0;
    kmipkit_ttlv_item_release(modify_new);
    modify_new = 0;
    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    identifier_view.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier_view.text_data = identifier_bytes;
    identifier_view.text_length = sizeof(identifier_bytes) - 1;
    present = UINT8_C(9);
    modify_current = (kmipkit_ttlv_item_t *)(uintptr_t)1;
    if (kmipkit_modify_attribute_request_unique_identifier(
            modify_without_current, &identifier_view) != 0 ||
        identifier_view.kind != 0 || identifier_view.text_data != 0 ||
        identifier_view.text_length != 0 ||
        kmipkit_modify_attribute_request_current_attribute(
            modify_without_current, &present, &modify_current) != 0 || present != 0 ||
        modify_current != 0 ||
        kmipkit_modify_attribute_request_new_attribute(
            modify_without_current, &modify_new) != 0 ||
        !standalone_attribute_item_has_tag(modify_new, UINT32_C(0x4200bf)) ||
        kmipkit_modify_attribute_request_to_ttlv_payload(
            modify_without_current, &modify_optional_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(
            modify_optional_payload, UINT64_C(1)) ||
        kmipkit_client_batch_item_modify_attribute_from_request(
            modify_without_current, &modify_optional_batch_item) != 0 ||
        modify_optional_batch_item == 0) {
        goto cleanup;
    }

    identifier_view = (kmipkit_unique_identifier_v1_t){0};
    identifier_view.struct_size = (uint32_t)sizeof(identifier_view);
    identifier_view.abi_version = UINT32_C(1);
    if (kmipkit_set_attribute_request_unique_identifier(
            set_request, &identifier_view) != 0 ||
        !response_identifier_matches(
            &identifier_view, identifier_bytes, sizeof(identifier_bytes) - 1) ||
        kmipkit_set_attribute_request_new_attribute(set_request, &set_attribute) != 0 ||
        !standalone_attribute_item_has_tag(set_attribute, UINT32_C(0x4200bf)) ||
        kmipkit_set_attribute_request_to_ttlv_payload(set_request, &set_payload) != 0 ||
        !standalone_attribute_payload_has_item_count(set_payload, UINT64_C(2)) ||
        kmipkit_client_batch_item_set_attribute_from_request(
            set_request, &set_batch_item) != 0 || set_batch_item == 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(set_payload);
    kmipkit_ttlv_structure_release(modify_payload);
    kmipkit_ttlv_structure_release(modify_optional_payload);
    kmipkit_ttlv_structure_release(empty_delete_payload);
    kmipkit_ttlv_structure_release(delete_payload);
    kmipkit_ttlv_structure_release(tag_adjust_payload);
    kmipkit_ttlv_structure_release(adjust_payload);
    kmipkit_ttlv_structure_release(add_payload);
    kmipkit_ttlv_structure_release(invalid_adjust_payload);
    kmipkit_client_batch_item_release(set_batch_item);
    kmipkit_client_batch_item_release(modify_batch_item);
    kmipkit_client_batch_item_release(modify_optional_batch_item);
    kmipkit_client_batch_item_release(delete_batch_item);
    kmipkit_client_batch_item_release(tag_adjust_batch_item);
    kmipkit_client_batch_item_release(adjust_batch_item);
    kmipkit_client_batch_item_release(add_batch_item);
    kmipkit_ttlv_item_release(set_attribute);
    kmipkit_ttlv_item_release(modify_new);
    kmipkit_ttlv_item_release(modify_current);
    kmipkit_ttlv_item_release(delete_attribute);
    kmipkit_ttlv_item_release(add_attribute);
    kmipkit_set_attribute_request_release(set_request);
    kmipkit_modify_attribute_request_release(modify_request);
    kmipkit_modify_attribute_request_release(modify_without_current);
    kmipkit_delete_attribute_request_release(empty_delete_request);
    kmipkit_delete_attribute_request_release(delete_request);
    kmipkit_adjust_attribute_request_release(invalid_adjust_request);
    kmipkit_adjust_attribute_request_release(tag_adjust_request);
    kmipkit_adjust_attribute_request_release(adjust_request);
    kmipkit_add_attribute_request_release(invalid_add_request);
    kmipkit_add_attribute_request_release(add_request);
    kmipkit_ttlv_value_view_release(adjustment_value_view);
    kmipkit_ttlv_value_release(owned_adjustment_value);
    kmipkit_ttlv_value_release(adjustment_value);
    kmipkit_ttlv_item_release(attribute);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_exercises_batch_header_fields(void) {
    static const uint8_t correlation_value[] = "client-correlation";
    static const uint8_t invalid_utf8[] = {0xff};
    kmipkit_client_batch_t *batch = 0;
    uint64_t item_count = UINT64_MAX;
    uint8_t present = UINT8_C(9);
    int64_t time_stamp = INT64_MAX;
    int result = 0;

    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0 ||
        kmipkit_client_batch_item_count(batch, &item_count) != 0 || item_count != 0) {
        goto cleanup;
    }
    if (kmipkit_client_batch_request_time_stamp(batch, &present, &time_stamp) != 0 ||
        present != 0 || time_stamp != 0 ||
        kmipkit_client_batch_request_time_stamp(batch, 0, &time_stamp) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_request_time_stamp(
            batch, (uint8_t *)(void *)&time_stamp, &time_stamp) !=
            KMIPKIT_ERROR_INVALID_INPUT) {
        goto cleanup;
    }
    if (kmipkit_client_batch_set_request_time_stamp(batch, -INT64_C(123456789)) != 0) {
        goto cleanup;
    }
    present = 0;
    time_stamp = 0;
    if (kmipkit_client_batch_request_time_stamp(batch, &present, &time_stamp) != 0 ||
        present != 1 || time_stamp != -INT64_C(123456789) ||
        kmipkit_client_batch_set_order_option(batch, 1) != 0 ||
        kmipkit_client_batch_set_order_option(batch, 2) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_add_error_continuation_value(batch, UINT32_C(0)) != 0 ||
        kmipkit_client_batch_add_error_continuation_value(batch, UINT32_MAX) != 0 ||
        kmipkit_client_batch_set_asynchronous_indicator(batch, UINT32_C(0x80000001)) != 0 ||
        kmipkit_client_batch_set_client_correlation_value(
            batch, correlation_value, sizeof(correlation_value) - 1) != 0 ||
        kmipkit_client_batch_set_client_correlation_value(batch, 0, 1) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_set_client_correlation_value(
            batch, invalid_utf8, sizeof(invalid_utf8)) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_batch_set_client_correlation_value(
            batch, correlation_value, UINT64_MAX) != KMIPKIT_ERROR_RESOURCE_LIMIT) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_client_batch_release(batch);
    return result;
}

int kmipkit_client_abi_c_consumer_rejects_attribute_output_headers(void) {
    static const uint8_t identifier_bytes[] = "attribute-output-owner";
    static const uint8_t vendor_name[] = "KMIPKit";
    static const uint8_t attribute_name[] = "Comment";
    static const uint8_t attribute_text[] = "owned by each request";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_item_t *attribute = 0;
    kmipkit_add_attribute_request_t *add_request = 0;
    kmipkit_adjust_attribute_request_t *adjust_request = 0;
    kmipkit_delete_attribute_request_t *delete_request = 0;
    kmipkit_modify_attribute_request_t *modify_request = 0;
    kmipkit_set_attribute_request_t *set_request = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t identifier_output = {0};
    kmipkit_unique_identifier_v1_t identifier_before = {0};
    kmipkit_attribute_reference_v1_t reference = {0};
    kmipkit_attribute_reference_v1_t reference_output = {0};
    kmipkit_attribute_reference_v1_t reference_before = {0};
    uint8_t present = UINT8_C(9);
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 ||
        !modify_attribute_create_item(
            limits, attribute_text, sizeof(attribute_text) - 1, &attribute)) {
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = sizeof(identifier_bytes) - 1;
    reference.struct_size = (uint32_t)sizeof(reference);
    reference.abi_version = 1;
    reference.kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME;
    reference.vendor_identification_data = vendor_name;
    reference.vendor_identification_length = sizeof(vendor_name) - 1;
    reference.attribute_name_data = attribute_name;
    reference.attribute_name_length = sizeof(attribute_name) - 1;

    if (kmipkit_add_attribute_request_create(&identifier, attribute, &add_request) != 0 ||
        kmipkit_adjust_attribute_request_create(
            &identifier, &reference, 1, 0, &adjust_request) != 0 ||
        kmipkit_delete_attribute_request_create(
            &identifier, 0, &reference, &delete_request) != 0 ||
        kmipkit_modify_attribute_request_create(
            &identifier, 0, attribute, &modify_request) != 0 ||
        kmipkit_set_attribute_request_create(&identifier, attribute, &set_request) != 0) {
        goto cleanup;
    }
    kmipkit_ttlv_item_release(attribute);
    attribute = 0;

    identifier_output.struct_size = (uint32_t)sizeof(identifier_output);
    identifier_output.abi_version = 2;
    identifier_output.kind = UINT32_C(99);
    identifier_output.text_data = (const uint8_t *)(uintptr_t)1;
    identifier_output.text_length = UINT64_C(7);
    identifier_before = identifier_output;
    if (kmipkit_add_attribute_request_unique_identifier(
            add_request, &identifier_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&identifier_output, &identifier_before, sizeof(identifier_output)) != 0 ||
        kmipkit_adjust_attribute_request_unique_identifier(
            adjust_request, &identifier_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&identifier_output, &identifier_before, sizeof(identifier_output)) != 0 ||
        kmipkit_delete_attribute_request_unique_identifier(
            delete_request, &identifier_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&identifier_output, &identifier_before, sizeof(identifier_output)) != 0 ||
        kmipkit_modify_attribute_request_unique_identifier(
            modify_request, &identifier_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&identifier_output, &identifier_before, sizeof(identifier_output)) != 0 ||
        kmipkit_set_attribute_request_unique_identifier(
            set_request, &identifier_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&identifier_output, &identifier_before, sizeof(identifier_output)) != 0) {
        goto cleanup;
    }

    reference_output.struct_size = (uint32_t)sizeof(reference_output);
    reference_output.abi_version = 2;
    reference_output.kind = UINT32_C(99);
    reference_output.tag_value = UINT32_C(0xdeadbeef);
    reference_output.vendor_identification_data = (const uint8_t *)(uintptr_t)1;
    reference_output.vendor_identification_length = UINT64_C(7);
    reference_output.attribute_name_data = (const uint8_t *)(uintptr_t)2;
    reference_output.attribute_name_length = UINT64_C(8);
    reference_before = reference_output;
    if (kmipkit_adjust_attribute_request_attribute_reference(
            adjust_request, &reference_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&reference_output, &reference_before, sizeof(reference_output)) != 0) {
        goto cleanup;
    }
    present = UINT8_C(9);
    if (kmipkit_delete_attribute_request_attribute_reference(
            delete_request, &present, &reference_output) != KMIPKIT_ERROR_INVALID_INPUT ||
        present != 0 ||
        memcmp(&reference_output, &reference_before, sizeof(reference_output)) != 0) {
        goto cleanup;
    }

    reference_output = (kmipkit_attribute_reference_v1_t){0};
    reference_output.struct_size = (uint32_t)sizeof(reference_output);
    reference_output.abi_version = 1;
    if (kmipkit_adjust_attribute_request_attribute_reference(
            adjust_request, &reference_output) != 0 ||
        reference_output.kind != KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME ||
        reference_output.vendor_identification_length != sizeof(vendor_name) - 1 ||
        reference_output.attribute_name_length != sizeof(attribute_name) - 1 ||
        memcmp(reference_output.vendor_identification_data, vendor_name,
               sizeof(vendor_name) - 1) != 0 ||
        memcmp(reference_output.attribute_name_data, attribute_name,
               sizeof(attribute_name) - 1) != 0) {
        goto cleanup;
    }

    union {
        uint8_t present;
        kmipkit_attribute_reference_v1_t reference;
    } overlapping_output = {0};
    overlapping_output.reference.struct_size = (uint32_t)sizeof(overlapping_output.reference);
    overlapping_output.reference.abi_version = 1;
    overlapping_output.reference.kind = UINT32_C(77);
    unsigned char overlapping_before[sizeof(overlapping_output)];
    memcpy(overlapping_before, &overlapping_output, sizeof(overlapping_output));
    if (kmipkit_delete_attribute_request_attribute_reference(
            delete_request, &overlapping_output.present, &overlapping_output.reference) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        memcmp(&overlapping_output, overlapping_before, sizeof(overlapping_output)) != 0) {
        goto cleanup;
    }

    identifier_output.abi_version = 1;
    identifier_output.kind = UINT32_C(99);
    if (kmipkit_add_attribute_request_unique_identifier(
            add_request, &identifier_output) != 0 ||
        !response_identifier_matches(
            &identifier_output, identifier_bytes, sizeof(identifier_bytes) - 1)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_set_attribute_request_release(set_request);
    kmipkit_modify_attribute_request_release(modify_request);
    kmipkit_delete_attribute_request_release(delete_request);
    kmipkit_adjust_attribute_request_release(adjust_request);
    kmipkit_add_attribute_request_release(add_request);
    kmipkit_ttlv_item_release(attribute);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_query_span_copy_budget(void) {
    const uint64_t half_budget = UINT64_C(8) * UINT64_C(1024) * UINT64_C(1024);
    uint8_t *first_bytes = 0;
    uint8_t *second_bytes = 0;
    kmipkit_query_async_requests_request_t *query = 0;
    kmipkit_query_async_requests_request_t *empty_filter = 0;
    kmipkit_query_async_requests_request_t *empty_filter_with_operation = 0;
    kmipkit_query_async_requests_request_t *over_budget = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_structure_view_t *payload_view = 0;
    kmipkit_byte_span_v1_t empty_span = {0};
    kmipkit_byte_span_v1_t over_budget_spans[2] = {{0}};
    uint32_t operation = UINT32_C(0xf1234567);
    uint64_t item_count = UINT64_MAX;
    int result = 0;

    first_bytes = (uint8_t *)calloc((size_t)half_budget, sizeof(*first_bytes));
    second_bytes = (uint8_t *)calloc((size_t)(half_budget + 1), sizeof(*second_bytes));
    if (first_bytes == 0 || second_bytes == 0 ||
        kmipkit_query_async_requests_request_create(&query) != 0 || query == 0 ||
        kmipkit_query_async_requests_request_with_correlation_values(
            query, &empty_span, 1, &empty_filter) != 0 || empty_filter == 0 ||
        kmipkit_query_async_requests_request_with_operations(
            empty_filter, &operation, 1, &empty_filter_with_operation) != 0 ||
        empty_filter_with_operation == 0 ||
        kmipkit_query_async_requests_request_to_ttlv_payload(
            empty_filter_with_operation, &payload) != 0 ||
        kmipkit_ttlv_structure_view(payload, &payload_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(payload_view, &item_count) != 0 ||
        item_count != 2 ||
        !async_query_payload_matches_filters(
            payload, &empty_span, 1, &operation, 1)) {
        goto cleanup;
    }

    over_budget_spans[0].data = first_bytes;
    over_budget_spans[0].length = half_budget;
    over_budget_spans[1].data = second_bytes;
    over_budget_spans[1].length = half_budget + 1;
    over_budget = (kmipkit_query_async_requests_request_t *)(uintptr_t)1;
    const int32_t over_budget_status =
        kmipkit_query_async_requests_request_with_correlation_values(
            query, over_budget_spans, 2, &over_budget);
    if (over_budget_status != KMIPKIT_ERROR_RESOURCE_LIMIT || over_budget != 0) {
        if (over_budget != (kmipkit_query_async_requests_request_t *)(uintptr_t)1) {
            kmipkit_query_async_requests_request_release(over_budget);
        }
        over_budget = 0;
        goto cleanup;
    }
    kmipkit_ttlv_structure_view_release(payload_view);
    payload_view = 0;
    kmipkit_ttlv_structure_release(payload);
    payload = 0;
    if (kmipkit_query_async_requests_request_to_ttlv_payload(query, &payload) != 0 ||
        kmipkit_ttlv_structure_view(payload, &payload_view) != 0 ||
        kmipkit_ttlv_structure_view_item_count(payload_view, &item_count) != 0 ||
        item_count != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    free(second_bytes);
    free(first_bytes);
    kmipkit_ttlv_structure_view_release(payload_view);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_query_async_requests_request_release(over_budget);
    kmipkit_query_async_requests_request_release(empty_filter_with_operation);
    kmipkit_query_async_requests_request_release(empty_filter);
    kmipkit_query_async_requests_request_release(query);
    return result;
}

static int ttlv_value_view_matches_bytes(
    kmipkit_ttlv_value_t *value,
    uint8_t expected_type,
    const uint8_t *expected_bytes,
    uint64_t expected_length) {
    kmipkit_ttlv_value_view_t *view = 0;
    uint8_t actual_type = 0;
    uint64_t actual_length = 0;
    int result = 0;

    if (value == 0 || kmipkit_ttlv_value_view(value, &view) != 0 || view == 0 ||
        kmipkit_ttlv_value_view_type(view, &actual_type) != 0 ||
        actual_type != expected_type ||
        kmipkit_ttlv_value_view_byte_length(view, &actual_length) != 0 ||
        actual_length != expected_length) {
        goto cleanup;
    }
    for (uint64_t index = 0; index < expected_length; ++index) {
        uint8_t actual_byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(view, index, &actual_byte) != 0 ||
            actual_byte != expected_bytes[index]) {
            goto cleanup;
        }
    }
    result = 1;

cleanup:
    kmipkit_ttlv_value_view_release(view);
    return result;
}

int kmipkit_client_abi_c_consumer_validates_variable_ttlv_value_inputs(void) {
    static const uint8_t big_integer[] = {0x80, 0x01, 0xff};
    static const uint8_t text[] = {'K', 'M', 'I', 'P'};
    static const uint8_t byte_string[] = {0x00, 0xff, 0x7f};
    static const uint8_t invalid_utf8[] = {0xc3, 0x28};
    static const uint8_t oversized[] = {'x', 'x', 'x', 'x', 'x'};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_value_t *value = 0;
    int result = 0;

    if (kmipkit_codec_limits_create(4, 8, 16, &limits) != 0 || limits == 0 ||
        kmipkit_ttlv_value_big_integer(
            limits, big_integer, (uint64_t)sizeof(big_integer), &value) != 0 ||
        !ttlv_value_view_matches_bytes(
            value, KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER,
            big_integer, (uint64_t)sizeof(big_integer))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(value);
    value = 0;

    if (kmipkit_ttlv_value_text_string(
            limits, text, (uint64_t)sizeof(text), &value) != 0 ||
        !ttlv_value_view_matches_bytes(
            value, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, text, (uint64_t)sizeof(text))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(value);
    value = 0;

    if (kmipkit_ttlv_value_byte_string(
            limits, byte_string, (uint64_t)sizeof(byte_string), &value) != 0 ||
        !ttlv_value_view_matches_bytes(
            value, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
            byte_string, (uint64_t)sizeof(byte_string))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(value);
    value = 0;

    value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_ttlv_value_text_string(
            limits, invalid_utf8, (uint64_t)sizeof(invalid_utf8), &value) !=
            KMIPKIT_ERROR_INVALID_INPUT || value != 0) {
        goto cleanup;
    }
    value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_ttlv_value_big_integer(
            limits, oversized, (uint64_t)sizeof(oversized), &value) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || value != 0) {
        goto cleanup;
    }
    value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_ttlv_value_text_string(
            limits, oversized, (uint64_t)sizeof(oversized), &value) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || value != 0) {
        goto cleanup;
    }
    value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_ttlv_value_byte_string(
            limits, oversized, (uint64_t)sizeof(oversized), &value) !=
            KMIPKIT_ERROR_RESOURCE_LIMIT || value != 0) {
        goto cleanup;
    }
    value = (kmipkit_ttlv_value_t *)(uintptr_t)1;
    if (kmipkit_ttlv_value_byte_string(limits, 0, 1, &value) !=
            KMIPKIT_ERROR_INVALID_INPUT || value != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    if (value != (kmipkit_ttlv_value_t *)(uintptr_t)1) {
        kmipkit_ttlv_value_release(value);
    }
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_exercises_import_request_validation(void) {
    static const uint8_t identifier_bytes[] = "import-contract";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_ttlv_structure_t *attributes = 0;
    kmipkit_ttlv_structure_t *any_object = 0;
    int result = 0;

    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == 0 ||
        !create_import_request(limits, &item) || item == 0) {
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes;
    identifier.text_length = (uint64_t)(sizeof(identifier_bytes) - 1);
    if (kmipkit_ttlv_structure_create(&attributes) != 0 ||
        kmipkit_ttlv_structure_create(&any_object) != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_import(
            0, 7, 0, 0, 0, 0, attributes, any_object, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_import(
            &identifier, 7, 2, 0, 0, 0, attributes, any_object, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_import(
            &identifier, 7, 0, 0, 2, 0, attributes, any_object, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    item = (kmipkit_client_batch_item_t *)(uintptr_t)1;
    if (kmipkit_client_batch_item_import(
            &identifier, 7, 0, 0, 0, 0, 0, any_object, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    if (item != (kmipkit_client_batch_item_t *)(uintptr_t)1) {
        kmipkit_client_batch_item_release(item);
    }
    kmipkit_ttlv_structure_release(any_object);
    kmipkit_ttlv_structure_release(attributes);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_client_abi_c_consumer_maps_message_validation_error_kind(void) {
    kmipkit_ttlv_structure_t *request = 0;
    kmipkit_request_message_t *message = 0;
    kmipkit_message_validation_error_t *validation_error = 0;
    uint32_t error_kind = 0;
    int result = 0;

    if (kmipkit_ttlv_structure_create(&request) != 0 || request == 0 ||
        kmipkit_request_message_from_ttlv(request, &message, &validation_error) == 0 ||
        message != 0 || validation_error == 0 ||
        kmipkit_message_validation_error_kind(validation_error, &error_kind) != 0 ||
        error_kind != UINT32_C(1)) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_message_validation_error_release(validation_error);
    kmipkit_request_message_release(message);
    kmipkit_ttlv_structure_release(request);
    return result;
}

int kmipkit_client_abi_c_consumer_reads_async_response_v1(
        kmipkit_client_async_operation_outcome_t *outcome,
        uint32_t expected_operation,
        uint32_t expected_result_status,
        uint32_t expected_is_pending,
        uint32_t expected_has_cancellation_result,
        uint32_t expected_cancellation_result,
        const uint8_t *expected_correlation,
        uint64_t expected_correlation_length,
        uint32_t expected_has_payload,
        uint64_t expected_payload_member_count) {
    typedef int32_t (*reader_fn)(
            kmipkit_client_async_operation_outcome_t *, uint8_t *,
            kmipkit_client_async_operation_response_v1_t *);
    const uint32_t operations[] = {UINT32_C(0x19), UINT32_C(0x1a),
            UINT32_C(0x3a), UINT32_C(0x39)};
    const reader_fn readers[] = {
        kmipkit_client_async_operation_outcome_cancel_response_v1,
        kmipkit_client_async_operation_outcome_poll_response_v1,
        kmipkit_client_async_operation_outcome_process_response_v1,
        kmipkit_client_async_operation_outcome_query_async_requests_response_v1};
    size_t selected = sizeof(operations) / sizeof(operations[0]);
    kmipkit_client_async_operation_response_v1_t response = {0};
    uint8_t present = UINT8_MAX;
    kmipkit_ttlv_structure_t *payload = NULL;
    uint8_t payload_present = UINT8_MAX;
    int result = 0;
    int failure_stage = 1;

    if (outcome == NULL || expected_correlation_length > UINT64_C(16777216) ||
            (expected_correlation_length != 0 && expected_correlation == NULL) ||
            expected_is_pending > 1 || expected_has_cancellation_result > 1 ||
            expected_has_payload > 1) {
        return 0;
    }
    for (size_t index = 0; index < sizeof(operations) / sizeof(operations[0]); ++index) {
        if (operations[index] == expected_operation) selected = index;
    }
    if (selected == sizeof(operations) / sizeof(operations[0]) ||
            readers[selected](outcome, &present, &response) != 0 || present != 1 ||
            response.struct_size != sizeof(response) || response.abi_version != 1 ||
            response.operation != expected_operation ||
            response.result_status != expected_result_status ||
            response.is_pending != expected_is_pending ||
            response.has_cancellation_result != expected_has_cancellation_result ||
            response.cancellation_result != expected_cancellation_result ||
            response.has_asynchronous_correlation_value !=
                    (expected_correlation_length != 0 || expected_correlation != NULL) ||
            response.has_response_payload != expected_has_payload ||
            response.response_payload_member_count != expected_payload_member_count ||
            response.reserved != 0 ||
            response.asynchronous_correlation_value.length != expected_correlation_length ||
            (expected_correlation_length == 0 &&
                    response.asynchronous_correlation_value.data != NULL) ||
            (expected_correlation_length != 0 &&
                    (response.asynchronous_correlation_value.data == NULL ||
                            memcmp(response.asynchronous_correlation_value.data,
                                    expected_correlation,
                                    (size_t)expected_correlation_length) != 0))) {
        failure_stage = 2;
        goto cleanup;
    }

    if (kmipkit_client_async_operation_outcome_response_payload_ttlv(
                outcome, &payload_present, &payload) != 0 ||
            payload_present != expected_has_payload ||
            ((payload_present == 0) != (payload == NULL))) {
        failure_stage = 3;
        goto cleanup;
    }

    for (size_t index = 0; index < sizeof(operations) / sizeof(operations[0]); ++index) {
        if (index == selected) continue;
        kmipkit_client_async_operation_response_v1_t mismatch = {0};
        uint8_t mismatch_present = UINT8_MAX;
        if (readers[index](outcome, &mismatch_present, &mismatch) != 0 ||
                mismatch_present != 0 || mismatch.struct_size != 0 ||
                mismatch.abi_version != 0) {
            failure_stage = 4 + (int)index;
            goto cleanup;
        }
    }
    result = 1;

cleanup:
    kmipkit_ttlv_structure_release(payload);
    return result == 1 ? 1 : -failure_stage;
}
