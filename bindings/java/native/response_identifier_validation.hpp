#ifndef KMIPKIT_JNI_RESPONSE_IDENTIFIER_VALIDATION_HPP
#define KMIPKIT_JNI_RESPONSE_IDENTIFIER_VALIDATION_HPP

#include "kmipkit_internal.h"

#include <cstdint>

namespace kmipkit::jni {

struct ResponseIdentifierFields {
    uint8_t present = 0;
    uint32_t kind = 0;
    uint32_t enumeration_value = 0;
    int32_t integer_value = 0;
    const uint8_t* text_data = nullptr;
    uint64_t text_length = 0;
};

inline bool normalize_response_identifier(
        uint8_t response_present, const kmipkit_unique_identifier_v1_t& value,
        ResponseIdentifierFields* output) noexcept {
    if (output == nullptr) {
        return false;
    }
    *output = {};
    if (response_present > 1) {
        return false;
    }

    // Keep operation-specific identifier requiredness in the Java response parser.
    if (value.kind == 0) {
        return value.text_data == nullptr && value.text_length == 0
                && value.enumeration_value == 0 && value.integer_value == 0;
    }
    if (response_present == 0) {
        return false;
    }

    if (value.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
        if (value.enumeration_value != 0 || value.integer_value != 0
                || (value.text_data == nullptr && value.text_length != 0)) {
            return false;
        }
    } else if (value.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION) {
        if (value.text_data != nullptr || value.text_length != 0 || value.integer_value != 0) {
            return false;
        }
    } else if (value.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER) {
        if (value.text_data != nullptr || value.text_length != 0 || value.enumeration_value != 0) {
            return false;
        }
    } else {
        return false;
    }

    output->present = 1;
    output->kind = value.kind;
    output->enumeration_value = value.enumeration_value;
    output->integer_value = value.integer_value;
    output->text_data = value.text_data;
    output->text_length = value.text_length;
    return true;
}

} // namespace kmipkit::jni

#endif // KMIPKIT_JNI_RESPONSE_IDENTIFIER_VALIDATION_HPP
