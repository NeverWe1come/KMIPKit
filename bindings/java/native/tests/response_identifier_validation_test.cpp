#include "../response_identifier_validation.hpp"
#include "../response_byte_span_validation.hpp"

#include <cassert>
#include <cstdint>

namespace {

kmipkit_unique_identifier_v1_t identifier(
        uint32_t kind, const uint8_t* text, uint64_t text_length,
        uint32_t enumeration, int32_t integer) {
    return {
            static_cast<uint32_t>(sizeof(kmipkit_unique_identifier_v1_t)),
            1,
            kind,
            0,
            text,
            text_length,
            enumeration,
            integer};
}

bool is_zero(const kmipkit::jni::ResponseIdentifierFields& fields) {
    return fields.present == 0 && fields.kind == 0 && fields.enumeration_value == 0
            && fields.integer_value == 0 && fields.text_data == nullptr && fields.text_length == 0;
}

bool rejects(uint8_t response_present, const kmipkit_unique_identifier_v1_t& value) {
    kmipkit::jni::ResponseIdentifierFields output{};
    return !kmipkit::jni::normalize_response_identifier(response_present, value, &output);
}

} // namespace

int main() {
    using kmipkit::jni::ResponseIdentifierFields;
    using kmipkit::jni::normalize_response_identifier;
    using kmipkit::jni::is_valid_response_byte_span;

    const uint8_t response_bytes[] = {0x41, 0x42};
    assert(is_valid_response_byte_span(0, {nullptr, 0}));
    assert(!is_valid_response_byte_span(0, {response_bytes, sizeof(response_bytes)}));
    assert(!is_valid_response_byte_span(0, {nullptr, 1}));
    assert(is_valid_response_byte_span(1, {nullptr, 0}));
    assert(is_valid_response_byte_span(1, {response_bytes, sizeof(response_bytes)}));
    assert(!is_valid_response_byte_span(1, {nullptr, sizeof(response_bytes)}));
    assert(!is_valid_response_byte_span(2, {nullptr, 0}));

    const uint8_t text[] = {0x41, 0x00, 0x42};
    ResponseIdentifierFields output{};
    const kmipkit_unique_identifier_v1_t absent = identifier(0, nullptr, 0, 0, 0);
    assert(normalize_response_identifier(0, absent, &output));
    assert(is_zero(output));
    assert(normalize_response_identifier(1, absent, &output));
    assert(is_zero(output));

    const auto text_identifier = identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING, text, sizeof(text), 0, 0);
    assert(normalize_response_identifier(1, text_identifier, &output));
    assert(output.present == 1);
    assert(output.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING);
    assert(output.text_data == text);
    assert(output.text_length == sizeof(text));
    assert(output.enumeration_value == 0);
    assert(output.integer_value == 0);

    const auto enumeration_identifier = identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION, nullptr, 0, UINT32_MAX, 0);
    assert(normalize_response_identifier(1, enumeration_identifier, &output));
    assert(output.present == 1);
    assert(output.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION);
    assert(output.enumeration_value == UINT32_MAX);
    assert(output.text_data == nullptr);
    assert(output.text_length == 0);
    assert(output.integer_value == 0);

    const auto integer_identifier = identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER, nullptr, 0, 0, INT32_MIN);
    assert(normalize_response_identifier(1, integer_identifier, &output));
    assert(output.present == 1);
    assert(output.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER);
    assert(output.integer_value == INT32_MIN);
    assert(output.text_data == nullptr);
    assert(output.text_length == 0);
    assert(output.enumeration_value == 0);

    assert(!normalize_response_identifier(2, absent, &output));
    assert(!normalize_response_identifier(0, text_identifier, &output));
    assert(rejects(1, identifier(0, text, 0, 0, 0)));
    assert(rejects(1, identifier(0, nullptr, 1, 0, 0)));
    assert(rejects(1, identifier(0, nullptr, 0, 1, 0)));
    assert(rejects(1, identifier(0, nullptr, 0, 0, 1)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING, nullptr, 1, 0, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING, text, sizeof(text), 1, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING, text, sizeof(text), 0, 1)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION, text, sizeof(text), 0, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION, nullptr, 1, 0, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION, nullptr, 0, 0, 1)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER, text, sizeof(text), 0, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER, nullptr, 1, 0, 0)));
    assert(rejects(1, identifier(
            KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER, nullptr, 0, 1, 0)));
    assert(rejects(1, identifier(77, nullptr, 0, 0, 0)));
    assert(!normalize_response_identifier(1, absent, nullptr));

    return 0;
}
