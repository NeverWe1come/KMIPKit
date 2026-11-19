#ifndef KMIPKIT_RESPONSE_BYTE_SPAN_VALIDATION_HPP
#define KMIPKIT_RESPONSE_BYTE_SPAN_VALIDATION_HPP

#include "kmipkit_internal.h"

#include <cstdint>

namespace kmipkit::jni {

inline bool is_valid_response_byte_span(
        uint8_t present, const kmipkit_byte_span_v1_t& span) noexcept {
    if (present == 0) return span.data == nullptr && span.length == 0;
    if (present != 1) return false;
    return span.data != nullptr || span.length == 0;
}

} // namespace kmipkit::jni

#endif
