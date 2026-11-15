#ifndef KMIPKIT_JNI_UNIQUE_BATCH_ITEM_ID_VALIDATION_HPP
#define KMIPKIT_JNI_UNIQUE_BATCH_ITEM_ID_VALIDATION_HPP

#include <cstdint>

namespace kmipkit::jni {

inline bool valid_unique_batch_item_id_length(std::int64_t length) noexcept {
    constexpr std::int64_t maximum = 16LL * 1024 * 1024;
    return length >= 0 && length <= maximum;
}

} // namespace kmipkit::jni

#endif // KMIPKIT_JNI_UNIQUE_BATCH_ITEM_ID_VALIDATION_HPP
