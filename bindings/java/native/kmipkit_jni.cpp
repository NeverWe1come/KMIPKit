#include <jni.h>
#include "kmipkit.h"

#include <algorithm>
#include <array>
#include <cstdint>
#include <limits>
#include <new>
#include <string>
#include <utility>
#include <vector>

namespace {

constexpr uint64_t kMaximumTextBytes = 4096;
constexpr uint64_t kMaximumNativeArrayElements = 100000;

void throw_java(JNIEnv* env, const char* class_name, const char* message) noexcept {
    if (env->ExceptionCheck()) {
        return;
    }
    jclass exception_class = env->FindClass(class_name);
    if (exception_class != nullptr) {
        env->ThrowNew(exception_class, message);
        env->DeleteLocalRef(exception_class);
    }
}

void throw_invalid_input(JNIEnv* env) noexcept {
    throw_java(env, "org/kmipkit/InvalidInputException", "native input is invalid");
}

void throw_status(JNIEnv* env, int32_t status) noexcept {
    switch (status) {
    case KMIPKIT_ERROR_COMPATIBILITY_MISMATCH:
        throw_java(env, "org/kmipkit/extensions/ExtensionCompatibilityException",
                   "extension compatibility is invalid");
        break;
    case KMIPKIT_ERROR_DUPLICATE_KEY:
        throw_java(env, "org/kmipkit/extensions/DuplicateExtensionKeyException",
                   "extension identity is duplicated");
        break;
    case KMIPKIT_ERROR_INVALID_IDENTITY:
        throw_java(env, "org/kmipkit/extensions/InvalidExtensionIdentityException",
                   "extension identity is invalid");
        break;
    case KMIPKIT_ERROR_INVALID_INPUT:
        throw_invalid_input(env);
        break;
    case KMIPKIT_ERROR_INVALID_SCHEMA:
        throw_java(env, "org/kmipkit/extensions/InvalidExtensionSchemaException",
                   "extension schema is invalid");
        break;
    case KMIPKIT_ERROR_RESOURCE_LIMIT:
        throw_java(env, "org/kmipkit/ResourceLimitException",
                   "configured resource limit was exceeded");
        break;
    default:
        throw_java(env, "org/kmipkit/KmipException", "native operation failed");
        break;
    }
}

template <typename Result, typename Function>
Result guarded(JNIEnv* env, Result fallback, Function&& function) noexcept {
    try {
        return function();
    } catch (const std::bad_alloc&) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "native bridge allocation failed");
    } catch (...) {
        throw_java(env, "org/kmipkit/KmipException", "native bridge operation failed");
    }
    return fallback;
}

template <typename Handle>
Handle* from_java_handle(jlong value) noexcept {
    static_assert(sizeof(uintptr_t) <= sizeof(jlong), "JNI handle does not fit in jlong");
    return reinterpret_cast<Handle*>(static_cast<uintptr_t>(value));
}

template <typename Handle>
jlong to_java_handle(Handle* value) noexcept {
    static_assert(sizeof(uintptr_t) <= sizeof(jlong), "JNI handle does not fit in jlong");
    return static_cast<jlong>(reinterpret_cast<uintptr_t>(value));
}

template <typename Handle, typename Function>
jlong return_handle(JNIEnv* env, Function&& function, bool nullable = false) noexcept {
    Handle* output = nullptr;
    const int32_t status = function(&output);
    if (status != 0) {
        throw_status(env, status);
        return 0;
    }
    if (output == nullptr && !nullable) {
        throw_invalid_input(env);
        return 0;
    }
    return to_java_handle(output);
}

template <typename Function>
bool check_status(JNIEnv* env, Function&& function) noexcept {
    const int32_t status = function();
    if (status == 0) {
        return true;
    }
    throw_status(env, status);
    return false;
}

bool append_utf8(std::vector<uint8_t>& output, uint32_t code_point) {
    if (code_point <= 0x7f) {
        output.push_back(static_cast<uint8_t>(code_point));
    } else if (code_point <= 0x7ff) {
        output.push_back(static_cast<uint8_t>(0xc0 | (code_point >> 6)));
        output.push_back(static_cast<uint8_t>(0x80 | (code_point & 0x3f)));
    } else if (code_point <= 0xffff) {
        output.push_back(static_cast<uint8_t>(0xe0 | (code_point >> 12)));
        output.push_back(static_cast<uint8_t>(0x80 | ((code_point >> 6) & 0x3f)));
        output.push_back(static_cast<uint8_t>(0x80 | (code_point & 0x3f)));
    } else if (code_point <= 0x10ffff) {
        output.push_back(static_cast<uint8_t>(0xf0 | (code_point >> 18)));
        output.push_back(static_cast<uint8_t>(0x80 | ((code_point >> 12) & 0x3f)));
        output.push_back(static_cast<uint8_t>(0x80 | ((code_point >> 6) & 0x3f)));
        output.push_back(static_cast<uint8_t>(0x80 | (code_point & 0x3f)));
    } else {
        return false;
    }
    return output.size() <= kMaximumTextBytes;
}

bool java_string_to_utf8(JNIEnv* env, jstring input, std::vector<uint8_t>& output) {
    if (input == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize length = env->GetStringLength(input);
    if (length < 0 || static_cast<uint64_t>(length) > kMaximumTextBytes) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "text input exceeds the bridge limit");
        return false;
    }
    const jchar* characters = env->GetStringChars(input, nullptr);
    if (characters == nullptr) {
        return false;
    }
    bool valid = true;
    for (jsize index = 0; index < length && valid; ++index) {
        uint32_t code_point = characters[index];
        if (code_point >= 0xd800 && code_point <= 0xdbff) {
            if (index + 1 >= length || characters[index + 1] < 0xdc00 || characters[index + 1] > 0xdfff) {
                valid = false;
                break;
            }
            const uint32_t low = characters[++index];
            code_point = 0x10000 + ((code_point - 0xd800) << 10) + (low - 0xdc00);
        } else if (code_point >= 0xdc00 && code_point <= 0xdfff) {
            valid = false;
            break;
        }
        valid = append_utf8(output, code_point);
    }
    env->ReleaseStringChars(input, characters);
    if (!valid) {
        output.clear();
        throw_invalid_input(env);
    }
    return valid;
}

bool utf8_to_java_string(JNIEnv* env, const uint8_t* bytes, uint64_t length, jstring* output) {
    if (output == nullptr || (bytes == nullptr && length != 0) || length > kMaximumTextBytes) {
        throw_invalid_input(env);
        return false;
    }
    std::vector<jchar> characters;
    characters.reserve(static_cast<size_t>(length));
    for (uint64_t index = 0; index < length;) {
        const uint8_t first = bytes[index++];
        uint32_t code_point = 0;
        uint32_t continuation_count = 0;
        if (first <= 0x7f) {
            code_point = first;
        } else if (first >= 0xc2 && first <= 0xdf) {
            code_point = first & 0x1f;
            continuation_count = 1;
        } else if (first >= 0xe0 && first <= 0xef) {
            code_point = first & 0x0f;
            continuation_count = 2;
        } else if (first >= 0xf0 && first <= 0xf4) {
            code_point = first & 0x07;
            continuation_count = 3;
        } else {
            throw_invalid_input(env);
            return false;
        }
        if (continuation_count > length - index) {
            throw_invalid_input(env);
            return false;
        }
        for (uint32_t offset = 0; offset < continuation_count; ++offset) {
            const uint8_t continuation = bytes[index++];
            if ((continuation & 0xc0) != 0x80) {
                throw_invalid_input(env);
                return false;
            }
            code_point = (code_point << 6) | (continuation & 0x3f);
        }
        const bool overlong = (continuation_count == 1 && code_point < 0x80)
                || (continuation_count == 2 && code_point < 0x800)
                || (continuation_count == 3 && code_point < 0x10000);
        if (overlong || code_point > 0x10ffff || (code_point >= 0xd800 && code_point <= 0xdfff)) {
            throw_invalid_input(env);
            return false;
        }
        if (code_point <= 0xffff) {
            characters.push_back(static_cast<jchar>(code_point));
        } else {
            code_point -= 0x10000;
            characters.push_back(static_cast<jchar>(0xd800 + (code_point >> 10)));
            characters.push_back(static_cast<jchar>(0xdc00 + (code_point & 0x3ff)));
        }
    }
    *output = env->NewString(characters.data(), static_cast<jsize>(characters.size()));
    return *output != nullptr;
}

bool read_long_array(JNIEnv* env, jlongArray input, jsize expected_length,
                     std::vector<uint64_t>& output, uint64_t max_elements = kMaximumNativeArrayElements) {
    if (input == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize length = env->GetArrayLength(input);
    if (length != expected_length || static_cast<uint64_t>(length) > max_elements) {
        throw_invalid_input(env);
        return false;
    }
    std::vector<jlong> values(static_cast<size_t>(length));
    if (length != 0) {
        env->GetLongArrayRegion(input, 0, length, values.data());
        if (env->ExceptionCheck()) {
            return false;
        }
    }
    output.clear();
    output.reserve(values.size());
    for (jlong value : values) {
        if (value < 0) {
            throw_invalid_input(env);
            output.clear();
            return false;
        }
        output.push_back(static_cast<uint64_t>(value));
    }
    return true;
}

template <typename Handle>
bool read_handle_array(JNIEnv* env, jlongArray input, std::vector<Handle*>& output) {
    if (input == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize length = env->GetArrayLength(input);
    if (length < 0 || static_cast<uint64_t>(length) > kMaximumNativeArrayElements) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "handle array exceeds the bridge limit");
        return false;
    }
    std::vector<jlong> values(static_cast<size_t>(length));
    if (length != 0) {
        env->GetLongArrayRegion(input, 0, length, values.data());
        if (env->ExceptionCheck()) {
            return false;
        }
    }
    output.clear();
    output.reserve(values.size());
    for (jlong value : values) {
        Handle* handle = from_java_handle<Handle>(value);
        if (handle == nullptr) {
            throw_invalid_input(env);
            output.clear();
            return false;
        }
        output.push_back(handle);
    }
    return true;
}

class CodecLimits {
public:
    CodecLimits() = default;
    CodecLimits(const CodecLimits&) = delete;
    CodecLimits& operator=(const CodecLimits&) = delete;
    ~CodecLimits() {
        if (value_ != nullptr) {
            kmipkit_codec_limits_release(value_);
        }
    }

    bool initialize(JNIEnv* env, jlongArray values) {
        std::vector<uint64_t> limits;
        if (!read_long_array(env, values, 3, limits)) {
            return false;
        }
        const int32_t status = kmipkit_codec_limits_create(
                limits[0], limits[1], limits[2], &value_);
        if (status != 0) {
            throw_status(env, status);
            return false;
        }
        max_message_bytes_ = limits[0];
        return value_ != nullptr;
    }

    kmipkit_codec_limits_t* get() const noexcept { return value_; }
    uint64_t max_message_bytes() const noexcept { return max_message_bytes_; }

private:
    kmipkit_codec_limits_t* value_ = nullptr;
    uint64_t max_message_bytes_ = 0;
};

template <typename Value, typename Function>
Value return_scalar(JNIEnv* env, Value fallback, Function&& function) noexcept {
    Value output{};
    const int32_t status = function(&output);
    if (status != 0) {
        throw_status(env, status);
        return fallback;
    }
    return output;
}

bool read_tag(JNIEnv* env, jlong raw_tag, uint32_t* output) {
    auto* tag = from_java_handle<kmipkit_tag_t>(raw_tag);
    if (tag == nullptr || output == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    return check_status(env, [&] { return kmipkit_ttlv_tag_value(tag, output); });
}

bool read_bytes(JNIEnv* env, jbyteArray input, uint64_t maximum, std::vector<uint8_t>& output) {
    if (input == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize length = env->GetArrayLength(input);
    if (length < 0 || static_cast<uint64_t>(length) > maximum) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "byte input exceeds configured limits");
        return false;
    }
    output.resize(static_cast<size_t>(length));
    if (length != 0) {
        env->GetByteArrayRegion(input, 0, length, reinterpret_cast<jbyte*>(output.data()));
    }
    return !env->ExceptionCheck();
}

bool registry_limit_values(JNIEnv* env, jlongArray input, std::array<uint64_t, 12>* output) {
    std::vector<uint64_t> values;
    if (output == nullptr || !read_long_array(env, input, 12, values)) {
        return false;
    }
    std::copy(values.begin(), values.end(), output->begin());
    return true;
}

bool read_value_text(JNIEnv* env, kmipkit_ttlv_value_t* value, jstring* output) {
    kmipkit_ttlv_value_view_t* view = nullptr;
    if (!check_status(env, [&] { return kmipkit_ttlv_value_view(value, &view); })) {
        return false;
    }
    uint8_t item_type = 0;
    uint64_t length = 0;
    bool success = check_status(env, [&] { return kmipkit_ttlv_value_view_type(view, &item_type); })
            && item_type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING
            && check_status(env, [&] { return kmipkit_ttlv_value_view_byte_length(view, &length); });
    if (!success) {
        kmipkit_ttlv_value_view_release(view);
        if (!env->ExceptionCheck()) {
            throw_invalid_input(env);
        }
        return false;
    }
    if (length > kMaximumTextBytes) {
        kmipkit_ttlv_value_view_release(view);
        throw_java(env, "org/kmipkit/ResourceLimitException", "identity text exceeds the bridge limit");
        return false;
    }
    std::vector<uint8_t> bytes(static_cast<size_t>(length));
    for (uint64_t index = 0; index < length; ++index) {
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_byte_at(view, index, &bytes[static_cast<size_t>(index)]);
            })) {
            kmipkit_ttlv_value_view_release(view);
            return false;
        }
    }
    kmipkit_ttlv_value_view_release(view);
    return utf8_to_java_string(env, bytes.data(), length, output);
}

jstring identity_text(JNIEnv* env, jlong raw_identity,
                      int32_t (*get_text)(kmipkit_extension_identity_t*, kmipkit_ttlv_value_t**)) {
    return guarded<jstring>(env, nullptr, [&]() -> jstring {
        auto* identity = from_java_handle<kmipkit_extension_identity_t>(raw_identity);
        if (identity == nullptr) {
            throw_invalid_input(env);
            return nullptr;
        }
        kmipkit_ttlv_value_t* value = nullptr;
        const int32_t status = get_text(identity, &value);
        if (status != 0) {
            throw_status(env, status);
            return nullptr;
        }
        if (value == nullptr) {
            throw_invalid_input(env);
            return nullptr;
        }
        jstring result = nullptr;
        const bool success = read_value_text(env, value, &result);
        kmipkit_ttlv_value_release(value);
        return success ? result : nullptr;
    });
}

template <int Kind>
jlong make_bytes_value(JNIEnv* env, jbyteArray raw_value, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        std::vector<uint8_t> bytes;
        if (!read_bytes(env, raw_value, limits.max_message_bytes(), bytes)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            if constexpr (Kind == 0) {
                return kmipkit_ttlv_value_big_integer(limits.get(), bytes.data(), bytes.size(), output);
            } else if constexpr (Kind == 1) {
                return kmipkit_ttlv_value_text_string(limits.get(), bytes.data(), bytes.size(), output);
            } else {
                return kmipkit_ttlv_value_byte_string(limits.get(), bytes.data(), bytes.size(), output);
            }
        });
    });
}

} // namespace

extern "C" {

JNIEXPORT jstring JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionIdentityVendorIdentifier(
        JNIEnv* env, jclass, jlong identity) {
    return identity_text(env, identity, kmipkit_extension_identity_vendor_identifier);
}

JNIEXPORT jstring JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionIdentityName(
        JNIEnv* env, jclass, jlong identity) {
    return identity_text(env, identity, kmipkit_extension_identity_name);
}

JNIEXPORT jstring JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionIdentityVersion(
        JNIEnv* env, jclass, jlong identity) {
    return identity_text(env, identity, kmipkit_extension_identity_version);
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDiscoverVersions(
        JNIEnv* env, jclass) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [](auto output) {
            return kmipkit_client_batch_item_discover_versions(output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemExtensionCount(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jlong>(env, 0, [&] {
        uint64_t count = 0;
        if (!check_status(env, [&] {
                return kmipkit_client_batch_item_extension_count(
                        from_java_handle<kmipkit_client_batch_item_t>(raw_item), &count);
            })) {
            return static_cast<jlong>(0);
        }
        if (count > static_cast<uint64_t>(std::numeric_limits<jlong>::max())) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "extension count exceeds Java range");
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(count);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemExtensionIdentityAt(
        JNIEnv* env, jclass, jlong raw_item, jlong index) {
    return guarded<jlong>(env, 0, [&] {
        if (index < 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_extension_identity_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_extension_identity_at(
                    from_java_handle<kmipkit_client_batch_item_t>(raw_item),
                    static_cast<uint64_t>(index), output);
        });
    });
}

JNIEXPORT jboolean JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemExtensionCriticalityIndicatorAt(
        JNIEnv* env, jclass, jlong raw_item, jlong index) {
    return guarded<jboolean>(env, JNI_FALSE, [&] {
        if (index < 0) {
            throw_invalid_input(env);
            return static_cast<jboolean>(JNI_FALSE);
        }
        uint8_t criticality = 0;
        if (!check_status(env, [&] {
                return kmipkit_client_batch_item_extension_criticality_indicator_at(
                        from_java_handle<kmipkit_client_batch_item_t>(raw_item),
                        static_cast<uint64_t>(index), &criticality);
            })) {
            return static_cast<jboolean>(JNI_FALSE);
        }
        if (criticality > 1) {
            throw_invalid_input(env);
            return static_cast<jboolean>(JNI_FALSE);
        }
        return static_cast<jboolean>(criticality == 1 ? JNI_TRUE : JNI_FALSE);
    });
}

} // extern "C"

extern "C" {

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_release(
        JNIEnv* env, jclass, jint kind, jlong raw_handle) {
    guarded<int>(env, 0, [&] {
        if (raw_handle == 0) {
            return 0;
        }
        switch (kind) {
        case 1: kmipkit_extension_identity_release(from_java_handle<kmipkit_extension_identity_t>(raw_handle)); break;
        case 2: kmipkit_extension_compatibility_release(from_java_handle<kmipkit_extension_compatibility_t>(raw_handle)); break;
        case 3:
        case 20: kmipkit_ttlv_path_release(from_java_handle<kmipkit_ttlv_path_t>(raw_handle)); break;
        case 4: kmipkit_extension_discriminator_release(from_java_handle<kmipkit_extension_discriminator_t>(raw_handle)); break;
        case 5: kmipkit_extension_schema_release(from_java_handle<kmipkit_extension_schema_t>(raw_handle)); break;
        case 6: kmipkit_extension_information_release(from_java_handle<kmipkit_extension_information_t>(raw_handle)); break;
        case 7: kmipkit_extension_definition_release(from_java_handle<kmipkit_extension_definition_t>(raw_handle)); break;
        case 8: kmipkit_client_extension_registry_release(from_java_handle<kmipkit_client_extension_registry_t>(raw_handle)); break;
        case 9: kmipkit_client_configuration_release(from_java_handle<kmipkit_client_configuration_t>(raw_handle)); break;
        case 10: kmipkit_validated_extension_value_release(from_java_handle<kmipkit_validated_extension_value_t>(raw_handle)); break;
        case 11: kmipkit_registered_extension_value_release(from_java_handle<kmipkit_registered_extension_value_t>(raw_handle)); break;
        case 12: kmipkit_client_request_message_extension_release(from_java_handle<kmipkit_client_request_message_extension_t>(raw_handle)); break;
        case 13: kmipkit_client_batch_item_release(from_java_handle<kmipkit_client_batch_item_t>(raw_handle)); break;
        case 14: kmipkit_ttlv_value_release(from_java_handle<kmipkit_ttlv_value_t>(raw_handle)); break;
        case 15: kmipkit_ttlv_structure_release(from_java_handle<kmipkit_ttlv_structure_t>(raw_handle)); break;
        case 16: kmipkit_ttlv_item_release(from_java_handle<kmipkit_ttlv_item_t>(raw_handle)); break;
        case 17: kmipkit_ttlv_structure_view_release(from_java_handle<kmipkit_ttlv_structure_view_t>(raw_handle)); break;
        case 18: kmipkit_ttlv_item_view_release(from_java_handle<kmipkit_ttlv_item_view_t>(raw_handle)); break;
        case 19: kmipkit_ttlv_value_view_release(from_java_handle<kmipkit_ttlv_value_view_t>(raw_handle)); break;
        case 21: kmipkit_raw_tag_release(from_java_handle<kmipkit_raw_tag_t>(raw_handle)); break;
        case 22: kmipkit_tag_release(from_java_handle<kmipkit_tag_t>(raw_handle)); break;
        case 23: kmipkit_extension_child_rule_release(from_java_handle<kmipkit_extension_child_rule_t>(raw_handle)); break;
        case 24: kmipkit_extension_order_constraint_release(from_java_handle<kmipkit_extension_order_constraint_t>(raw_handle)); break;
        case 25: kmipkit_extension_recognition_release(from_java_handle<kmipkit_extension_recognition_t>(raw_handle)); break;
        case 26: kmipkit_codec_limits_release(from_java_handle<kmipkit_codec_limits_t>(raw_handle)); break;
        default: throw_invalid_input(env); break;
        }
        return 0;
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionIdentityCreate(
        JNIEnv* env, jclass, jstring vendor, jstring name, jstring version) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> vendor_bytes, name_bytes, version_bytes;
        if (!java_string_to_utf8(env, vendor, vendor_bytes)
                || !java_string_to_utf8(env, name, name_bytes)
                || !java_string_to_utf8(env, version, version_bytes)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_identity_t>(env, [&](auto output) {
            return kmipkit_extension_identity_create(
                    vendor_bytes.data(), vendor_bytes.size(), name_bytes.data(), name_bytes.size(),
                    version_bytes.data(), version_bytes.size(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_compatibilityCreate(
        JNIEnv* env, jclass, jint min_major, jint min_minor, jint max_major, jint max_minor,
        jstring minimum, jstring maximum) {
    return guarded<jlong>(env, 0, [&] {
        if (min_major < 0 || min_major > UINT8_MAX || min_minor < 0 || min_minor > UINT8_MAX
                || max_major < 0 || max_major > UINT8_MAX || max_minor < 0 || max_minor > UINT8_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> minimum_bytes, maximum_bytes;
        if (!java_string_to_utf8(env, minimum, minimum_bytes)
                || !java_string_to_utf8(env, maximum, maximum_bytes)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_compatibility_t>(env, [&](auto output) {
            return kmipkit_extension_compatibility_create(
                    static_cast<uint8_t>(min_major), static_cast<uint8_t>(min_minor),
                    static_cast<uint8_t>(max_major), static_cast<uint8_t>(max_minor),
                    minimum_bytes.data(), minimum_bytes.size(), maximum_bytes.data(), maximum_bytes.size(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvPathCreate(
        JNIEnv* env, jclass, jlong first_tag_handle) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t tag = 0;
        if (!read_tag(env, first_tag_handle, &tag)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_path_t>(env, [&](auto output) {
            return kmipkit_ttlv_path_create(tag, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvPathWithChildTag(
        JNIEnv* env, jclass, jlong raw_path, jlong raw_tag) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t tag = 0;
        if (!read_tag(env, raw_tag, &tag)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_path_t>(env, [&](auto output) {
            return kmipkit_ttlv_path_with_child_tag(
                    from_java_handle<kmipkit_ttlv_path_t>(raw_path), tag, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_discriminatorCreate(
        JNIEnv* env, jclass, jlong raw_path, jlong raw_scalar) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_discriminator_t>(env, [&](auto output) {
            return kmipkit_extension_discriminator_create(
                    from_java_handle<kmipkit_ttlv_path_t>(raw_path),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_scalar), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaScalar(
        JNIEnv* env, jclass, jint item_type) {
    return guarded<jlong>(env, 0, [&] {
        if (item_type < 0 || item_type > UINT32_MAX) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_scalar(static_cast<uint32_t>(item_type), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaStructure(
        JNIEnv* env, jclass, jlongArray child_handles, jlongArray constraint_handles,
        jboolean preserve_children) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<kmipkit_extension_child_rule_t*> children;
        std::vector<kmipkit_extension_order_constraint_t*> constraints;
        if (!read_handle_array(env, child_handles, children)
                || !read_handle_array(env, constraint_handles, constraints)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_structure(
                    children.data(), children.size(), constraints.data(), constraints.size(),
                    preserve_children == JNI_TRUE ? 1 : 0, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionChildRuleRequired(
        JNIEnv* env, jclass, jlong raw_tag, jlong raw_schema) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t tag = 0;
        if (!read_tag(env, raw_tag, &tag)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_child_rule_t>(env, [&](auto output) {
            return kmipkit_extension_child_rule_required(
                    tag, from_java_handle<kmipkit_extension_schema_t>(raw_schema), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionChildRuleOptional(
        JNIEnv* env, jclass, jlong raw_tag, jlong raw_schema) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t tag = 0;
        if (!read_tag(env, raw_tag, &tag)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_child_rule_t>(env, [&](auto output) {
            return kmipkit_extension_child_rule_optional(
                    tag, from_java_handle<kmipkit_extension_schema_t>(raw_schema), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionChildRuleRepeated(
        JNIEnv* env, jclass, jlong raw_tag, jlong raw_schema) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t tag = 0;
        if (!read_tag(env, raw_tag, &tag)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_child_rule_t>(env, [&](auto output) {
            return kmipkit_extension_child_rule_repeated(
                    tag, from_java_handle<kmipkit_extension_schema_t>(raw_schema), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionOrderConstraintCreate(
        JNIEnv* env, jclass, jlong raw_before, jlong raw_after) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t before = 0, after = 0;
        if (!read_tag(env, raw_before, &before) || !read_tag(env, raw_after, &after)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_order_constraint_t>(env, [&](auto output) {
            return kmipkit_extension_order_constraint_create(before, after, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaMinimumLength(
        JNIEnv* env, jclass, jlong raw_schema, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        if (value < 0) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_minimum_length(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint64_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaMaximumLength(
        JNIEnv* env, jclass, jlong raw_schema, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        if (value < 0) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_maximum_length(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint64_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaSignedRange(
        JNIEnv* env, jclass, jlong raw_schema, jlong minimum, jlong maximum) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_signed_numeric_range(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema), minimum, maximum, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaUnsignedRange(
        JNIEnv* env, jclass, jlong raw_schema, jlong minimum, jlong maximum) {
    return guarded<jlong>(env, 0, [&] {
        if (minimum < 0 || maximum < 0) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_unsigned_numeric_range(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint64_t>(minimum), static_cast<uint64_t>(maximum), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaAllowedEnumeration(
        JNIEnv* env, jclass, jlong raw_schema, jint value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_allowed_enumeration(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint32_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaAllowedBitMask(
        JNIEnv* env, jclass, jlong raw_schema, jint value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_allowed_bit_mask(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint32_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionSchemaRequiredBitMask(
        JNIEnv* env, jclass, jlong raw_schema, jint value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_schema_t>(env, [&](auto output) {
            return kmipkit_extension_schema_required_bit_mask(
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema),
                    static_cast<uint32_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionDefinitionCreate(
        JNIEnv* env, jclass, jlong raw_identity, jlong raw_compatibility,
        jlong raw_discriminator, jlong raw_schema) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_definition_t>(env, [&](auto output) {
            return kmipkit_extension_definition_create(
                    from_java_handle<kmipkit_extension_identity_t>(raw_identity),
                    from_java_handle<kmipkit_extension_compatibility_t>(raw_compatibility),
                    from_java_handle<kmipkit_extension_discriminator_t>(raw_discriminator),
                    from_java_handle<kmipkit_extension_schema_t>(raw_schema), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionDefinitionValidate(
        JNIEnv* env, jclass, jlong raw_definition, jlong raw_value, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_validated_extension_value_t>(env, [&](auto output) {
            return kmipkit_extension_definition_validate(
                    from_java_handle<kmipkit_extension_definition_t>(raw_definition),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_value), output, limits.get());
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientConfigurationCreate(
        JNIEnv* env, jclass, jlong raw_registry) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_configuration_t>(env, [&](auto output) {
            return kmipkit_client_configuration_create(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientConfigurationExtensionRegistry(
        JNIEnv* env, jclass, jlong raw_configuration) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_extension_registry_t>(env, [&](auto output) {
            return kmipkit_client_configuration_extension_registry(
                    from_java_handle<kmipkit_client_configuration_t>(raw_configuration), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryCreate(
        JNIEnv* env, jclass, jlongArray raw_definitions, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<kmipkit_extension_definition_t*> definitions;
        std::array<uint64_t, 12> limits{};
        if (!read_handle_array(env, raw_definitions, definitions)
                || !registry_limit_values(env, raw_limits, &limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_client_extension_registry_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_create(
                    definitions.data(), definitions.size(),
                    limits[0], limits[1], limits[2], limits[3], limits[4], limits[5],
                    limits[6], limits[7], limits[8], limits[9], limits[10], limits[11], output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryInspect(
        JNIEnv* env, jclass, jlong raw_registry, jstring raw_vendor, jlong raw_value,
        jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> vendor;
        if (!java_string_to_utf8(env, raw_vendor, vendor)) return static_cast<jlong>(0);
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_recognition_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_inspect(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry),
                    vendor.data(), vendor.size(), from_java_handle<kmipkit_ttlv_structure_t>(raw_value),
                    output, limits.get());
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryDefinitionCount(
        JNIEnv* env, jclass, jlong raw_registry) {
    return guarded<jlong>(env, 0, [&] {
        return return_scalar<jlong>(env, 0, [&](auto output) {
            uint64_t count = 0;
            const int32_t status = kmipkit_client_extension_registry_definition_count(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry), &count);
            if (status == 0 && count > static_cast<uint64_t>(std::numeric_limits<jlong>::max())) {
                return KMIPKIT_ERROR_RESOURCE_LIMIT;
            }
            *output = static_cast<jlong>(count);
            return status;
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryDefinitionAt(
        JNIEnv* env, jclass, jlong raw_registry, jlong index) {
    return guarded<jlong>(env, 0, [&] {
        if (index < 0) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_definition_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_definition_at(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry),
                    static_cast<uint64_t>(index), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryValidate(
        JNIEnv* env, jclass, jlong raw_registry, jlong raw_identity,
        jlong raw_value, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_registered_extension_value_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_validate(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry),
                    from_java_handle<kmipkit_extension_identity_t>(raw_identity),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_value), limits.get(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientRequestMessageExtensionCreate(
        JNIEnv* env, jclass, jlong raw_registered, jboolean criticality) {
    return guarded<jlong>(env, 0, [&] {
        if (criticality != JNI_TRUE && criticality != JNI_FALSE) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_client_request_message_extension_t>(env, [&](auto output) {
            return kmipkit_client_request_message_extension_create(
                    from_java_handle<kmipkit_registered_extension_value_t>(raw_registered),
                    criticality == JNI_TRUE ? 1 : 0, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemWithExtension(
        JNIEnv* env, jclass, jlong raw_item, jlong raw_extension) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_with_extension(
                    from_java_handle<kmipkit_client_batch_item_t>(raw_item),
                    from_java_handle<kmipkit_client_request_message_extension_t>(raw_extension), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationCreate(
        JNIEnv* env, jclass, jstring raw_name) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> name;
        if (!java_string_to_utf8(env, raw_name, name)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_create(name.data(), name.size(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithTag(
        JNIEnv* env, jclass, jlong raw_info, jint tag) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_tag(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    static_cast<uint32_t>(tag), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithType(
        JNIEnv* env, jclass, jlong raw_info, jint type) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_type(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    static_cast<uint32_t>(type), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithEnumeration(
        JNIEnv* env, jclass, jlong raw_info, jint enumeration) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_enumeration(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    static_cast<uint32_t>(enumeration), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithAttribute(
        JNIEnv* env, jclass, jlong raw_info, jboolean attribute) {
    return guarded<jlong>(env, 0, [&] {
        if (attribute != JNI_TRUE && attribute != JNI_FALSE) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_attribute(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    attribute == JNI_TRUE ? 1 : 0, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithParentStructureTag(
        JNIEnv* env, jclass, jlong raw_info, jint tag) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_parent_structure_tag(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    static_cast<uint32_t>(tag), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationWithDescription(
        JNIEnv* env, jclass, jlong raw_info, jstring raw_description) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> description;
        if (!java_string_to_utf8(env, raw_description, description)) return static_cast<jlong>(0);
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_information_description(
                    from_java_handle<kmipkit_extension_information_t>(raw_info),
                    description.data(), description.size(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionInformationToTtlv(
        JNIEnv* env, jclass, jlong raw_info) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_t>(env, [&](auto output) {
            return kmipkit_extension_information_to_ttlv(
                    from_java_handle<kmipkit_extension_information_t>(raw_info), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionDefinitionWithInformation(
        JNIEnv* env, jclass, jlong raw_definition, jlong raw_info) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_definition_t>(env, [&](auto output) {
            return kmipkit_extension_definition_with_information(
                    from_java_handle<kmipkit_extension_definition_t>(raw_definition),
                    from_java_handle<kmipkit_extension_information_t>(raw_info), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionDefinitionInformation(
        JNIEnv* env, jclass, jlong raw_definition) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_information_t>(env, [&](auto output) {
            return kmipkit_extension_definition_information(
                    from_java_handle<kmipkit_extension_definition_t>(raw_definition), output);
        }, true);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExtensionRegistryDefinitionForIdentity(
        JNIEnv* env, jclass, jlong raw_registry, jlong raw_identity) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_definition_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_definition_for_identity(
                    from_java_handle<kmipkit_client_extension_registry_t>(raw_registry),
                    from_java_handle<kmipkit_extension_identity_t>(raw_identity), output);
        }, true);
    });
}

JNIEXPORT jboolean JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionRecognitionIsRecognized(
        JNIEnv* env, jclass, jlong raw_recognition) {
    return guarded<jboolean>(env, JNI_FALSE, [&] {
        uint32_t recognized = 0;
        if (!check_status(env, [&] {
                return kmipkit_extension_recognition_is_recognized(
                        from_java_handle<kmipkit_extension_recognition_t>(raw_recognition), &recognized);
            })) return static_cast<jboolean>(JNI_FALSE);
        if (recognized > 1) { throw_invalid_input(env); return static_cast<jboolean>(JNI_FALSE); }
        return static_cast<jboolean>(recognized == 1 ? JNI_TRUE : JNI_FALSE);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionRecognitionValidatedValue(
        JNIEnv* env, jclass, jlong raw_recognition) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_validated_extension_value_t>(env, [&](auto output) {
            return kmipkit_extension_recognition_validated_value(
                    from_java_handle<kmipkit_extension_recognition_t>(raw_recognition), output);
        }, true);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_extensionRecognitionGenericValue(
        JNIEnv* env, jclass, jlong raw_recognition) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_view_t>(env, [&](auto output) {
            return kmipkit_extension_recognition_generic_value(
                    from_java_handle<kmipkit_extension_recognition_t>(raw_recognition), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_validatedExtensionValueIdentity(
        JNIEnv* env, jclass, jlong raw_value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_extension_identity_t>(env, [&](auto output) {
            return kmipkit_validated_extension_value_identity(
                    from_java_handle<kmipkit_validated_extension_value_t>(raw_value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_validatedExtensionValueGenericValue(
        JNIEnv* env, jclass, jlong raw_value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_view_t>(env, [&](auto output) {
            return kmipkit_validated_extension_value_generic_value(
                    from_java_handle<kmipkit_validated_extension_value_t>(raw_value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_validatedExtensionValueValueAt(
        JNIEnv* env, jclass, jlong raw_value, jlong raw_path) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_view_t>(env, [&](auto output) {
            return kmipkit_validated_extension_value_value_at(
                    from_java_handle<kmipkit_validated_extension_value_t>(raw_value),
                    from_java_handle<kmipkit_ttlv_path_t>(raw_path), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_rawTagCreate(
        JNIEnv* env, jclass, jint raw) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_raw_tag_t>(env, [&](auto output) {
            return kmipkit_ttlv_raw_tag_create(static_cast<uint32_t>(raw), output);
        });
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_rawTagValue(
        JNIEnv* env, jclass, jlong raw_tag) {
    return guarded<jint>(env, 0, [&] {
        uint32_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_raw_tag_value(
                        from_java_handle<kmipkit_raw_tag_t>(raw_tag), &value);
            })) return static_cast<jint>(0);
        return static_cast<jint>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_rawTagChecked(
        JNIEnv* env, jclass, jlong raw_tag) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_tag_t>(env, [&](auto output) {
            return kmipkit_ttlv_raw_tag_try_checked(
                    from_java_handle<kmipkit_raw_tag_t>(raw_tag), output);
        });
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_tagValue(
        JNIEnv* env, jclass, jlong raw_tag) {
    return guarded<jint>(env, 0, [&] {
        uint32_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_tag_value(from_java_handle<kmipkit_tag_t>(raw_tag), &value);
            })) return static_cast<jint>(0);
        return static_cast<jint>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueInteger(
        JNIEnv* env, jclass, jint value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_integer(value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueLongInteger(
        JNIEnv* env, jclass, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_long_integer(value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueEnumeration(
        JNIEnv* env, jclass, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        if (value < 0 || static_cast<uint64_t>(value) > UINT32_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_enumeration(static_cast<uint32_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueBoolean(
        JNIEnv* env, jclass, jboolean value) {
    return guarded<jlong>(env, 0, [&] {
        if (value != JNI_TRUE && value != JNI_FALSE) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_boolean(value == JNI_TRUE ? 1 : 0, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueDateTime(
        JNIEnv* env, jclass, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_date_time(value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueInterval(
        JNIEnv* env, jclass, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        if (value < 0 || static_cast<uint64_t>(value) > UINT32_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_interval(static_cast<uint32_t>(value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueDateTimeExtended(
        JNIEnv* env, jclass, jlong value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_date_time_extended(value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueBigInteger(
        JNIEnv* env, jclass, jbyteArray value, jlongArray limits) {
    return make_bytes_value<0>(env, value, limits);
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueTextString(
        JNIEnv* env, jclass, jbyteArray value, jlongArray limits) {
    return make_bytes_value<1>(env, value, limits);
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueByteString(
        JNIEnv* env, jclass, jbyteArray value, jlongArray limits) {
    return make_bytes_value<2>(env, value, limits);
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueStructure(
        JNIEnv* env, jclass, jlong raw_structure, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_value_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_structure(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_structure), limits.get(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvStructureCreate(
        JNIEnv* env, jclass) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_t>(env, [](auto output) {
            return kmipkit_ttlv_structure_create(output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvItemCreate(
        JNIEnv* env, jclass, jlong raw_tag, jlong raw_value, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_item_t>(env, [&](auto output) {
            return kmipkit_ttlv_item_create(from_java_handle<kmipkit_tag_t>(raw_tag),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_value), limits.get(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvStructureWithItem(
        JNIEnv* env, jclass, jlong raw_structure, jlong raw_item, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        return return_handle<kmipkit_ttlv_structure_t>(env, [&](auto output) {
            return kmipkit_ttlv_structure_with_item(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_structure),
                    from_java_handle<kmipkit_ttlv_item_t>(raw_item), limits.get(), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvStructureView(
        JNIEnv* env, jclass, jlong raw_structure) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_view_t>(env, [&](auto output) {
            return kmipkit_ttlv_structure_view(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_structure), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvStructureViewItemAt(
        JNIEnv* env, jclass, jlong raw_view, jlong index) {
    return guarded<jlong>(env, 0, [&] {
        if (index < 0) { throw_invalid_input(env); return static_cast<jlong>(0); }
        return return_handle<kmipkit_ttlv_item_view_t>(env, [&](auto output) {
            return kmipkit_ttlv_structure_view_item_at(
                    from_java_handle<kmipkit_ttlv_structure_view_t>(raw_view),
                    static_cast<uint64_t>(index), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvItemViewValue(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_view_t>(env, [&](auto output) {
            return kmipkit_ttlv_item_view_value(
                    from_java_handle<kmipkit_ttlv_item_view_t>(raw_view), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueView(
        JNIEnv* env, jclass, jlong raw_value) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_value_view_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_view(
                    from_java_handle<kmipkit_ttlv_value_t>(raw_value), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewStructure(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_ttlv_structure_view_t>(env, [&](auto output) {
            return kmipkit_ttlv_value_view_structure(
                    from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), output);
        });
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvItemViewTag(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jint>(env, 0, [&] {
        kmipkit_tag_t* tag = nullptr;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_item_view_tag(
                        from_java_handle<kmipkit_ttlv_item_view_t>(raw_view), &tag);
            })) return static_cast<jint>(0);
        uint32_t raw = 0;
        const bool success = check_status(env, [&] { return kmipkit_ttlv_tag_value(tag, &raw); });
        kmipkit_tag_release(tag);
        return success ? static_cast<jint>(raw) : static_cast<jint>(0);
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvItemViewType(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jint>(env, 0, [&] {
        uint8_t type = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_item_view_type(
                        from_java_handle<kmipkit_ttlv_item_view_t>(raw_view), &type);
            })) return static_cast<jint>(0);
        return static_cast<jint>(type);
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewType(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jint>(env, 0, [&] {
        uint8_t type = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_type(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &type);
            })) return static_cast<jint>(0);
        return static_cast<jint>(type);
    });
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewInteger(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jint>(env, 0, [&] {
        int32_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_integer(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jint>(0);
        return static_cast<jint>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewLongInteger(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_long_integer(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jlong>(0);
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewEnumeration(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_enumeration(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jlong>(0);
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jboolean JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewBoolean(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jboolean>(env, JNI_FALSE, [&] {
        uint8_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_boolean(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jboolean>(JNI_FALSE);
        if (value > 1) { throw_invalid_input(env); return static_cast<jboolean>(JNI_FALSE); }
        return static_cast<jboolean>(value == 1 ? JNI_TRUE : JNI_FALSE);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewDateTime(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_date_time(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jlong>(0);
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewInterval(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_interval(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jlong>(0);
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewDateTimeExtended(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_date_time_extended(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &value);
            })) return static_cast<jlong>(0);
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewByteLength(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        uint64_t length = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_byte_length(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view), &length);
            })) return static_cast<jlong>(0);
        if (length > static_cast<uint64_t>(std::numeric_limits<jlong>::max())) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "byte length exceeds Java range");
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(length);
    });
}

JNIEXPORT jshort JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvValueViewByteAt(
        JNIEnv* env, jclass, jlong raw_view, jlong index) {
    return guarded<jshort>(env, 0, [&] {
        if (index < 0) { throw_invalid_input(env); return static_cast<jshort>(0); }
        uint8_t value = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_value_view_byte_at(
                        from_java_handle<kmipkit_ttlv_value_view_t>(raw_view),
                        static_cast<uint64_t>(index), &value);
            })) return static_cast<jshort>(0);
        return static_cast<jshort>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_ttlvStructureViewItemCount(
        JNIEnv* env, jclass, jlong raw_view) {
    return guarded<jlong>(env, 0, [&] {
        uint64_t count = 0;
        if (!check_status(env, [&] {
                return kmipkit_ttlv_structure_view_item_count(
                        from_java_handle<kmipkit_ttlv_structure_view_t>(raw_view), &count);
            })) return static_cast<jlong>(0);
        if (count > static_cast<uint64_t>(std::numeric_limits<jlong>::max())) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "item count exceeds Java range");
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(count);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_codecLimitsCreate(
        JNIEnv* env, jclass, jlong message_bytes, jlong structure_depth, jlong elements) {
    return guarded<jlong>(env, 0, [&] {
        if (message_bytes < 0 || structure_depth < 0 || elements < 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_codec_limits_t>(env, [&](auto output) {
            return kmipkit_codec_limits_create(
                    static_cast<uint64_t>(message_bytes),
                    static_cast<uint64_t>(structure_depth),
                    static_cast<uint64_t>(elements), output);
        });
    });
}

} // extern "C"
