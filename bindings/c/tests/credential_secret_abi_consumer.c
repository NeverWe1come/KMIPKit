#include <stdint.h>

#include "kmipkit.h"

typedef struct {
    kmipkit_secret_bytes_t *handle;
    const uint8_t *expected;
    uint64_t expected_length;
    int release_handle;
    int matched;
} bytes_context_t;

typedef struct {
    const uint8_t *expected;
    uint64_t expected_length;
    int matched;
} text_context_t;

static void compare_bytes(const uint8_t *data, uint64_t length, void *opaque) {
    bytes_context_t *context = (bytes_context_t *)opaque;
    context->matched = length == context->expected_length;
    for (uint64_t index = 0; context->matched && index < length; index++) {
        context->matched = data[index] == context->expected[index];
    }
    if (context->release_handle) {
        kmipkit_secret_bytes_release(context->handle);
    }
}

static void compare_text(const uint8_t *utf8, uint64_t length, void *opaque) {
    text_context_t *context = (text_context_t *)opaque;
    context->matched = length == context->expected_length;
    for (uint64_t index = 0; context->matched && index < length; index++) {
        context->matched = utf8[index] == context->expected[index];
    }
}

static int32_t value_matches(
        kmipkit_ttlv_value_t *value,
        kmipkit_ttlv_item_type_t expected_type,
        const uint8_t *expected,
        uint64_t expected_length) {
    kmipkit_ttlv_value_view_t *view = 0;
    kmipkit_ttlv_item_type_t actual_type = 0;
    uint64_t actual_length = 0;
    int32_t status = kmipkit_ttlv_value_view(value, &view);
    if (status == 0) status = kmipkit_ttlv_value_view_type(view, &actual_type);
    if (status == 0 && actual_type != expected_type) status = 1;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(view, &actual_length);
    if (status == 0 && actual_length != expected_length) status = 1;
    for (uint64_t index = 0; status == 0 && index < actual_length; index++) {
        uint8_t actual = 0;
        status = kmipkit_ttlv_value_view_byte_at(view, index, &actual);
        if (status == 0 && actual != expected[index]) status = 1;
    }
    kmipkit_ttlv_value_view_release(view);
    return status;
}

int32_t kmipkit_secret_c_consumer_contract(void) {
    static const uint8_t binary[] = {0x00, 0x7f, 0xff, 0x00};
    static const uint8_t unicode_text[] = {0xE2, 0x98, 0x83, 0x00, 0xC3, 0xA9};
    static const uint8_t invalid_utf8[] = {0xC0, 0xAF};
    uint8_t caller_copy[] = {0x00, 0x7f, 0xff, 0x00};
    kmipkit_secret_bytes_t *bytes = 0;
    kmipkit_secret_text_t *text = 0;
    kmipkit_ttlv_value_t *value = 0;
    bytes_context_t bytes_context = {0};
    text_context_t text_context = {0};
    int32_t status = kmipkit_secret_bytes_create(
            caller_copy, sizeof(caller_copy), &bytes);
    if (status != 0) return status;
    for (uint64_t index = 0; index < sizeof(binary); index++) {
        if (caller_copy[index] != binary[index]) return 1;
    }
    bytes_context.handle = bytes;
    bytes_context.expected = binary;
    bytes_context.expected_length = sizeof(binary);
    bytes_context.release_handle = 1;
    status = kmipkit_secret_bytes_with_bytes(bytes, compare_bytes, &bytes_context);
    if (status != 0 || !bytes_context.matched) return status == 0 ? 1 : status;
    status = kmipkit_secret_bytes_with_bytes(bytes, compare_bytes, &bytes_context);
    if (status == 0) return 1;

    status = kmipkit_secret_bytes_create(binary, sizeof(binary), &bytes);
    if (status == 0) status = kmipkit_secret_bytes_into_ttlv_value(bytes, &value);
    if (status == 0) {
        bytes = 0;
        status = value_matches(value, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
                binary, sizeof(binary));
    }
    kmipkit_ttlv_value_release(value);
    value = 0;
    if (status != 0) return status;

    status = kmipkit_secret_bytes_create(0, 0, &bytes);
    if (status != 0) return status;
    bytes_context.handle = bytes;
    bytes_context.expected = 0;
    bytes_context.expected_length = 0;
    bytes_context.release_handle = 0;
    bytes_context.matched = 0;
    status = kmipkit_secret_bytes_with_bytes(bytes, compare_bytes, &bytes_context);
    kmipkit_secret_bytes_release(bytes);
    if (status != 0 || !bytes_context.matched) return status == 0 ? 1 : status;
    if (kmipkit_secret_bytes_create(0, 1, &bytes) == 0) return 1;

    status = kmipkit_secret_text_create(unicode_text, sizeof(unicode_text), &text);
    if (status != 0) return status;
    text_context.expected = unicode_text;
    text_context.expected_length = sizeof(unicode_text);
    text_context.matched = 0;
    status = kmipkit_secret_text_with_str(text, compare_text, &text_context);
    if (status != 0 || !text_context.matched) return status == 0 ? 1 : status;
    status = kmipkit_secret_text_into_ttlv_value(text, &value);
    if (status == 0) {
        text = 0;
        status = value_matches(value, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
                unicode_text, sizeof(unicode_text));
    }
    kmipkit_ttlv_value_release(value);
    if (status != 0) return status;

    text = 0;
    if (kmipkit_secret_text_create(invalid_utf8, sizeof(invalid_utf8), &text) == 0) {
        kmipkit_secret_text_release(text);
        return 1;
    }
    return 0;
}
