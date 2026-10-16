#include <jni.h>
#include "kmipkit_internal.h"
#include "client_batch_item_transfer_guard.hpp"
#include "secure_bytes.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <new>
#include <string>
#include <utility>
#include <vector>

// Shared T076 layouts and declarations come from the generated internal
// header; this bridge retains ABI assertions and Java-specific adapters.
typedef struct kmipkit_transport_config_t kmipkit_transport_config_t;
typedef struct kmipkit_client_t kmipkit_client_t;
typedef struct kmipkit_client_batch_t kmipkit_client_batch_t;
typedef struct kmipkit_client_batch_response_t kmipkit_client_batch_response_t;
typedef struct kmipkit_client_batch_item_response_t kmipkit_client_batch_item_response_t;

extern "C" int32_t kmipkit_client_batch_set_request_time_stamp(
        kmipkit_client_batch_t* batch, int64_t timestamp);
extern "C" int32_t kmipkit_client_batch_set_order_option(
        kmipkit_client_batch_t* batch, uint8_t enabled);
extern "C" int32_t kmipkit_client_batch_add_error_continuation_value(
        kmipkit_client_batch_t* batch, uint32_t value);
extern "C" int32_t kmipkit_client_batch_set_client_correlation_value(
        kmipkit_client_batch_t* batch, const uint8_t* value, uint64_t value_length);
extern "C" int32_t kmipkit_client_batch_item_with_unique_batch_item_id(
        kmipkit_client_batch_item_t* item, const uint8_t* id_data, uint64_t id_length,
        kmipkit_client_batch_item_t** out_item);

// Layouts come from the generated internal header. Keep compile-time checks
// here so the JNI bridge consumes the same ABI definition as the C harness.
static_assert(sizeof(kmipkit_byte_span_v1_t) == 16);
static_assert(offsetof(kmipkit_byte_span_v1_t, length) == 8);
static_assert(sizeof(kmipkit_transport_input_v1_t) == 24);
static_assert(offsetof(kmipkit_transport_input_v1_t, data) == 8);
static_assert(sizeof(kmipkit_timeout_limit_v1_t) == 24);
static_assert(offsetof(kmipkit_timeout_limit_v1_t, seconds) == 8);
static_assert(offsetof(kmipkit_timeout_limit_v1_t, nanoseconds) == 16);
static_assert(sizeof(kmipkit_request_options_v1_t) == 104);
static_assert(offsetof(kmipkit_request_options_v1_t, connect_timeout) == 8);
static_assert(sizeof(kmipkit_transport_config_v1_t) == 256);
static_assert(offsetof(kmipkit_transport_config_v1_t, client_certificates) == 48);
static_assert(offsetof(kmipkit_transport_config_v1_t, connect_timeout) == 152);
static_assert(offsetof(kmipkit_transport_config_v1_t, max_request_bytes) == 248);
static_assert(sizeof(kmipkit_client_batch_item_metadata_v1_t) == 24);
static_assert(offsetof(kmipkit_client_batch_item_metadata_v1_t, outcome_kind) == 8);

extern "C" {
int32_t kmipkit_transport_config_create_v1(const kmipkit_transport_config_v1_t*, kmipkit_transport_config_t**);
void kmipkit_transport_config_release(kmipkit_transport_config_t*);
int32_t kmipkit_client_create(kmipkit_client_configuration_t*, kmipkit_transport_config_t**, kmipkit_client_t**);
void kmipkit_client_release(kmipkit_client_t*);
int32_t kmipkit_client_batch_create(kmipkit_client_batch_t**);
int32_t kmipkit_client_batch_add_item(kmipkit_client_batch_t*, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_set_asynchronous_indicator(kmipkit_client_batch_t*, uint32_t);
void kmipkit_client_batch_item_release(kmipkit_client_batch_item_t*);
void kmipkit_client_batch_release(kmipkit_client_batch_t*);
int32_t kmipkit_client_execute(kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*, kmipkit_client_batch_response_t**);
int32_t kmipkit_client_execute_with_options_v1(kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*, const kmipkit_request_options_v1_t*, kmipkit_client_batch_response_t**);
void kmipkit_client_batch_response_release(kmipkit_client_batch_response_t*);
void kmipkit_client_batch_item_response_release(kmipkit_client_batch_item_response_t*);
int32_t kmipkit_client_batch_response_item_count(kmipkit_client_batch_response_t*, uint64_t*);
int32_t kmipkit_client_batch_response_item_at(kmipkit_client_batch_response_t*, uint64_t, kmipkit_client_batch_item_response_t**);
int32_t kmipkit_client_batch_item_response_metadata_v1(kmipkit_client_batch_item_response_t*, kmipkit_client_batch_item_metadata_v1_t*);
int32_t kmipkit_client_batch_item_response_unique_batch_item_id(kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_pending_correlation_value(kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_discover_versions_supported_versions(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_protocol_version_v1_t*, uint64_t, uint64_t*);
int32_t kmipkit_last_error_get(int32_t*, uint32_t*, uint32_t*, uint32_t*);
}

namespace {

constexpr int32_t kErrorProtocol = 8;
constexpr int32_t kErrorTransport = 9;
constexpr int32_t kErrorServerResult = 10;
constexpr uint32_t kTransportEndpointRawTls = 1;
constexpr uint32_t kTransportEndpointHttps = 2;
constexpr uint32_t kTransportTrustPlatform = 1;
constexpr uint32_t kTransportTrustCertificates = 2;
constexpr uint32_t kTransportSourceMemory = 1;
constexpr uint32_t kTransportSourceFile = 2;
constexpr uint32_t kTransportEncodingPem = 1;
constexpr uint32_t kTransportEncodingDer = 2;
constexpr uint32_t kTransportTimeoutBounded = 1;
constexpr uint32_t kTransportTimeoutUnbounded = 2;
constexpr uint32_t kTransportTimeoutDefault = 0;
constexpr uint32_t kTransportFlagTlsServerName = 1;
constexpr uint32_t kTransportFlagTargetUri = 2;
constexpr uint64_t kMaximumTransportInputBytes = 16 * 1024 * 1024;
constexpr uint64_t kMaximumTransportInputCount = 4096;

constexpr uint64_t kMaximumTextBytes = 4096;
constexpr uint64_t kMaximumNativeInputArrayElements = 100000;

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

void throw_client_operation_status(JNIEnv* env, int32_t status) noexcept {
    int32_t native_status = status;
    uint32_t category = 0;
    uint32_t cause = 0;
    uint32_t delivery = 0;
    if (kmipkit_last_error_get(&native_status, &category, &cause, &delivery) != 0) {
        throw_java(env, "org/kmipkit/KmipException", "native operation failed");
        return;
    }
    const char* class_name = status == kErrorProtocol ? "org/kmipkit/ProtocolException"
            : status == kErrorTransport ? "org/kmipkit/TransportException"
            : "org/kmipkit/ServerResultException";
    jclass exception_class = env->FindClass(class_name);
    if (exception_class == nullptr) {
        return;
    }
    jmethodID constructor = env->GetMethodID(exception_class, "<init>", "(IIII)V");
    if (constructor == nullptr) {
        env->DeleteLocalRef(exception_class);
        return;
    }
    jobject exception = env->NewObject(exception_class, constructor,
            static_cast<jint>(native_status), static_cast<jint>(category),
            static_cast<jint>(cause), static_cast<jint>(delivery));
    if (exception != nullptr) {
        env->Throw(static_cast<jthrowable>(exception));
        env->DeleteLocalRef(exception);
    }
    env->DeleteLocalRef(exception_class);
}

void throw_status(JNIEnv* env, int32_t status) noexcept {
    if (status == kErrorProtocol || status == kErrorTransport || status == kErrorServerResult) {
        throw_client_operation_status(env, status);
        return;
    }
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
    case KMIPKIT_ERROR_INTERNAL:
        throw_java(env, "org/kmipkit/KmipException", "native operation failed");
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
                     std::vector<uint64_t>& output,
                     uint64_t max_elements = kMaximumNativeInputArrayElements) {
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
    if (length < 0 || static_cast<uint64_t>(length) > kMaximumNativeInputArrayElements) {
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
        kmipkit::secure::VectorWiper bytes_wiper(bytes);
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

jobject four_integer_record(
        JNIEnv* env,
        const char* class_name,
        jint first,
        jint second,
        jint third,
        jint fourth) {
    jclass record_class = env->FindClass(class_name);
    if (record_class == nullptr) {
        return nullptr;
    }
    const jmethodID constructor = env->GetMethodID(record_class, "<init>", "(IIII)V");
    if (constructor == nullptr) {
        env->DeleteLocalRef(record_class);
        return nullptr;
    }
    jobject record = env->NewObject(record_class, constructor, first, second, third, fourth);
    env->DeleteLocalRef(record_class);
    return record;
}

struct OwnedTransportInput {
    uint32_t source;
    uint32_t encoding;
    std::vector<uint8_t> bytes;
};

bool read_java_bytes(JNIEnv* env, jbyteArray input, std::vector<uint8_t>& output,
                     bool require_non_empty = true) {
    if (input == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize length = env->GetArrayLength(input);
    if (length < 0 || (require_non_empty && length == 0)) {
        throw_invalid_input(env);
        return false;
    }
    if (static_cast<uint64_t>(length) > kMaximumTransportInputBytes) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "transport material exceeds the bridge limit");
        return false;
    }
    output.resize(static_cast<size_t>(length));
    if (length != 0) {
        env->GetByteArrayRegion(input, 0, length, reinterpret_cast<jbyte*>(output.data()));
    }
    return !env->ExceptionCheck();
}

bool read_transport_inputs(JNIEnv* env, jintArray raw_sources, jintArray raw_encodings,
        jobjectArray raw_bytes, bool allow_files, std::vector<OwnedTransportInput>& output) {
    if (raw_sources == nullptr || raw_encodings == nullptr || raw_bytes == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize count = env->GetArrayLength(raw_sources);
    if (count < 0 || count > static_cast<jsize>(kMaximumTransportInputCount)
            || env->GetArrayLength(raw_encodings) != count || env->GetArrayLength(raw_bytes) != count) {
        throw_invalid_input(env);
        return false;
    }
    std::vector<jint> sources(static_cast<size_t>(count));
    std::vector<jint> encodings(static_cast<size_t>(count));
    if (count != 0) {
        env->GetIntArrayRegion(raw_sources, 0, count, sources.data());
        env->GetIntArrayRegion(raw_encodings, 0, count, encodings.data());
        if (env->ExceptionCheck()) return false;
    }
    output.clear();
    output.reserve(static_cast<size_t>(count));
    uint64_t total_bytes = 0;
    for (jsize index = 0; index < count; ++index) {
        if ((sources[static_cast<size_t>(index)] != static_cast<jint>(kTransportSourceMemory)
                && (!allow_files || sources[static_cast<size_t>(index)]
                    != static_cast<jint>(kTransportSourceFile)))
                || (encodings[static_cast<size_t>(index)] != static_cast<jint>(kTransportEncodingPem)
                    && encodings[static_cast<size_t>(index)] != static_cast<jint>(kTransportEncodingDer))) {
            throw_invalid_input(env);
            return false;
        }
        auto raw = static_cast<jbyteArray>(env->GetObjectArrayElement(raw_bytes, index));
        if (raw == nullptr) {
            throw_invalid_input(env);
            return false;
        }
        OwnedTransportInput item {
                static_cast<uint32_t>(sources[static_cast<size_t>(index)]),
                static_cast<uint32_t>(encodings[static_cast<size_t>(index)]), {}};
        const bool copied = read_java_bytes(env, raw, item.bytes);
        env->DeleteLocalRef(raw);
        if (!copied) return false;
        total_bytes += static_cast<uint64_t>(item.bytes.size());
        if (total_bytes > kMaximumTransportInputBytes) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "transport material exceeds the bridge limit");
            return false;
        }
        output.push_back(std::move(item));
    }
    return true;
}

std::vector<kmipkit_transport_input_v1_t> transport_input_views(
        const std::vector<OwnedTransportInput>& inputs) {
    std::vector<kmipkit_transport_input_v1_t> views;
    views.reserve(inputs.size());
    for (const OwnedTransportInput& input : inputs) {
        views.push_back({input.source, input.encoding,
                {input.bytes.empty() ? nullptr : input.bytes.data(),
                 static_cast<uint64_t>(input.bytes.size())}});
    }
    return views;
}

bool optional_java_string(JNIEnv* env, jstring input, std::vector<uint8_t>& output) {
    if (input == nullptr) {
        output.clear();
        return true;
    }
    return java_string_to_utf8(env, input, output);
}

jbyteArray span_to_java_bytes(JNIEnv* env, uint8_t present, kmipkit_byte_span_v1_t span) {
    if (present == 0) return nullptr;
    if (present != 1 || (span.data == nullptr && span.length != 0)
            || span.length > kMaximumTransportInputBytes
            || span.length > static_cast<uint64_t>(std::numeric_limits<jsize>::max())) {
        throw_invalid_input(env);
        return nullptr;
    }
    const jsize length = static_cast<jsize>(span.length);
    jbyteArray result = env->NewByteArray(length);
    if (result != nullptr && length != 0) {
        env->SetByteArrayRegion(result, 0, length, reinterpret_cast<const jbyte*>(span.data));
    }
    return result;
}

jlongArray make_long_array(JNIEnv* env, const std::vector<jlong>& values) {
    if (values.size() > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "native result exceeds Java array limits");
        return nullptr;
    }
    const jsize count = static_cast<jsize>(values.size());
    jlongArray result = env->NewLongArray(count);
    if (result != nullptr && count != 0) {
        env->SetLongArrayRegion(result, 0, count, values.data());
    }
    return result;
}

jobjectArray make_object_pair(JNIEnv* env, jobject first, jobject second) {
    jclass object_class = env->FindClass("java/lang/Object");
    if (object_class == nullptr) return nullptr;
    jobjectArray result = env->NewObjectArray(2, object_class, nullptr);
    env->DeleteLocalRef(object_class);
    if (result == nullptr) return nullptr;
    env->SetObjectArrayElement(result, 0, first);
    env->SetObjectArrayElement(result, 1, second);
    return env->ExceptionCheck() ? nullptr : result;
}

jobjectArray identifier_to_java(JNIEnv* env, const kmipkit_unique_identifier_v1_t& value,
        uint8_t present) {
    const std::vector<jlong> fields {static_cast<jlong>(present),
            static_cast<jlong>(value.kind), static_cast<jlong>(value.enumeration_value),
            static_cast<jlong>(value.integer_value)};
    jlongArray scalar_fields = make_long_array(env, fields);
    if (scalar_fields == nullptr) return nullptr;
    jbyteArray text = value.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
            ? span_to_java_bytes(env, 1,
                    {value.text_data, value.text_length}) : nullptr;
    if (env->ExceptionCheck()) {
        env->DeleteLocalRef(scalar_fields);
        return nullptr;
    }
    jobjectArray result = make_object_pair(env, scalar_fields, text);
    env->DeleteLocalRef(scalar_fields);
    if (text != nullptr) env->DeleteLocalRef(text);
    return result;
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCreate(
        JNIEnv* env, jclass, jlong object_type, jlong raw_attributes, jlong raw_masks) {
    return guarded<jlong>(env, 0, [&] {
        if (object_type < 0 || static_cast<uint64_t>(object_type) > UINT32_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_create(static_cast<uint32_t>(object_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCreateKeyPair(
        JNIEnv* env, jclass, jlong common_attributes, jlong private_attributes,
        jlong public_attributes, jlong common_masks, jlong private_masks, jlong public_masks) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_create_key_pair(
                    from_java_handle<kmipkit_ttlv_structure_t>(common_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(private_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(public_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(common_masks),
                    from_java_handle<kmipkit_ttlv_structure_t>(private_masks),
                    from_java_handle<kmipkit_ttlv_structure_t>(public_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCreateSplitKey(
        JNIEnv* env, jclass, jlong object_type, jint parts, jint threshold, jlong method,
        jint identifier_kind, jbyteArray identifier_text, jlong identifier_number,
        jbyteArray prime_field_size, jlong raw_attributes, jlong raw_masks) {
    return guarded<jlong>(env, 0, [&] {
        if (object_type < 0 || static_cast<uint64_t>(object_type) > UINT32_MAX
                || method < 0 || static_cast<uint64_t>(method) > UINT32_MAX
                || identifier_kind < 0 || identifier_kind > 3) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> identifier_bytes;
        if (identifier_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
                && !read_java_bytes(env, identifier_text, identifier_bytes, false)) {
            return static_cast<jlong>(0);
        }
        if (identifier_kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING && identifier_text != nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> prime_bytes;
        const uint8_t* prime_data = nullptr;
        if (prime_field_size != nullptr) {
            if (!read_java_bytes(env, prime_field_size, prime_bytes, false)) return static_cast<jlong>(0);
            prime_data = prime_bytes.empty() ? nullptr : prime_bytes.data();
        }
        kmipkit_unique_identifier_v1_t identifier{};
        const kmipkit_unique_identifier_v1_t* identifier_pointer = nullptr;
        if (identifier_kind != 0) {
            identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
            identifier.abi_version = 1;
            identifier.kind = static_cast<uint32_t>(identifier_kind);
            identifier.text_data = identifier_bytes.empty() ? nullptr : identifier_bytes.data();
            identifier.text_length = static_cast<uint64_t>(identifier_bytes.size());
            if (identifier_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION) {
                if (identifier_number < 0 || static_cast<uint64_t>(identifier_number) > UINT32_MAX) {
                    throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
                identifier.enumeration_value = static_cast<uint32_t>(identifier_number);
            } else if (identifier_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER) {
                if (identifier_number < INT32_MIN || identifier_number > INT32_MAX) {
                    throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
                identifier.integer_value = static_cast<int32_t>(identifier_number);
            } else if (identifier_kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
                throw_invalid_input(env);
                return static_cast<jlong>(0);
            }
            identifier_pointer = &identifier;
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_create_split_key(
                    static_cast<uint32_t>(object_type), identifier_pointer, parts, threshold,
                    static_cast<uint32_t>(method), static_cast<uint8_t>(prime_field_size != nullptr),
                    prime_data, static_cast<uint64_t>(prime_bytes.size()),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_registerRequestCreate(
        JNIEnv* env, jclass, jint object_type, jlong raw_attributes, jlong raw_object) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_register_request_t>(env, [&](auto output) {
            return kmipkit_register_request_create(
                    static_cast<uint32_t>(object_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_item_t>(raw_object), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_registerRequestWithOpaqueObjectContext(
        JNIEnv* env, jclass, jlong raw_request, jint context) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_register_request_t>(env, [&](auto output) {
            return kmipkit_register_request_with_opaque_object_context(
                    from_java_handle<kmipkit_register_request_t>(raw_request),
                    static_cast<uint32_t>(context), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRegister(
        JNIEnv* env, jclass, jlong raw_request) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_register(
                    from_java_handle<kmipkit_register_request_t>(raw_request), output);
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
        case 27: kmipkit_transport_config_release(from_java_handle<kmipkit_transport_config_t>(raw_handle)); break;
        case 28: kmipkit_client_release(from_java_handle<kmipkit_client_t>(raw_handle)); break;
        case 29: kmipkit_client_batch_release(from_java_handle<kmipkit_client_batch_t>(raw_handle)); break;
        case 30: kmipkit_client_batch_response_release(from_java_handle<kmipkit_client_batch_response_t>(raw_handle)); break;
        case 31: kmipkit_client_batch_item_response_release(from_java_handle<kmipkit_client_batch_item_response_t>(raw_handle)); break;
        case 32: kmipkit_register_request_release(from_java_handle<kmipkit_register_request_t>(raw_handle)); break;
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
        kmipkit_extension_registry_limits_v1_t versioned_limits{};
        versioned_limits.struct_size = static_cast<uint32_t>(sizeof(versioned_limits));
        versioned_limits.abi_version = UINT32_C(1);
        versioned_limits.max_definitions = limits[0];
        versioned_limits.max_schema_nodes = limits[1];
        versioned_limits.max_child_rules_per_structure = limits[2];
        versioned_limits.max_text_bytes_per_field = limits[3];
        versioned_limits.max_registry_text_bytes = limits[4];
        versioned_limits.max_discriminator_scalar_bytes = limits[5];
        versioned_limits.max_total_discriminator_scalar_bytes = limits[6];
        versioned_limits.max_constraint_members_per_rule = limits[7];
        versioned_limits.max_total_constraint_members = limits[8];
        versioned_limits.max_payload_index_records = limits[9];
        versioned_limits.max_lookup_comparisons = limits[10];
        versioned_limits.max_depth = limits[11];
        return return_handle<kmipkit_client_extension_registry_t>(env, [&](auto output) {
            return kmipkit_client_extension_registry_create_v1(
                    definitions.data(), definitions.size(), &versioned_limits, output);
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemWithUniqueBatchItemId(
        JNIEnv* env, jclass, jlong raw_item, jbyteArray raw_id) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_item == 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        kmipkit::jni::ClientBatchItemTransferGuard<
                kmipkit_client_batch_item_t*,
                decltype(&kmipkit_client_batch_item_release)> item_transfer(
                        from_java_handle<kmipkit_client_batch_item_t>(raw_item),
                        kmipkit_client_batch_item_release);
        if (raw_id == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const jsize length = env->GetArrayLength(raw_id);
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        if (!kmipkit::jni::valid_unique_batch_item_id_length(length)) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "unique batch item ID exceeds the configured limit");
            return static_cast<jlong>(0);
        }
        std::vector<jbyte> bytes(static_cast<size_t>(length));
        if (length > 0) {
            env->GetByteArrayRegion(raw_id, 0, length, bytes.data());
            if (env->ExceptionCheck()) return static_cast<jlong>(0);
        }
        kmipkit_client_batch_item_t* consumed_item = item_transfer.transfer_to_consumer();
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_with_unique_batch_item_id(
                    consumed_item,
                    reinterpret_cast<const uint8_t*>(bytes.data()),
                    static_cast<uint64_t>(bytes.size()), output);
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_transportConfigCreate(
        JNIEnv* env, jclass, jint endpoint_kind, jstring endpoint, jint endpoint_port, jint trust_kind,
        jintArray cert_sources, jintArray cert_encodings, jobjectArray cert_bytes,
        jint key_source, jint key_encoding, jbyteArray key_bytes,
        jintArray trust_sources, jintArray trust_encodings, jobjectArray trust_bytes,
        jintArray crl_sources, jintArray crl_encodings, jobjectArray crl_bytes,
        jstring tls_server_name, jstring target_uri, jlongArray raw_timeouts, jlong max_request_bytes) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> endpoint_data, tls_server_name_data, target_uri_data;
        if (!java_string_to_utf8(env, endpoint, endpoint_data)
                || !optional_java_string(env, tls_server_name, tls_server_name_data)
                || !optional_java_string(env, target_uri, target_uri_data)) {
            return static_cast<jlong>(0);
        }
        if (endpoint_kind < static_cast<jint>(kTransportEndpointRawTls)
                || endpoint_kind > static_cast<jint>(kTransportEndpointHttps)
                || endpoint_port < 0 || endpoint_port > 65'535
                || trust_kind < static_cast<jint>(kTransportTrustPlatform)
                || trust_kind > static_cast<jint>(kTransportTrustCertificates)
                || max_request_bytes <= 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }

        std::vector<OwnedTransportInput> certificates, trust_certificates, revocation_lists;
        if (!read_transport_inputs(env, cert_sources, cert_encodings, cert_bytes, true, certificates)
                || !read_transport_inputs(env, trust_sources, trust_encodings, trust_bytes, true, trust_certificates)
                || !read_transport_inputs(env, crl_sources, crl_encodings, crl_bytes, false, revocation_lists)) {
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> private_key;
        kmipkit::secure::VectorWiper private_key_wiper(private_key);
        if ((key_source != static_cast<jint>(kTransportSourceMemory)
                && key_source != static_cast<jint>(kTransportSourceFile))
                || (key_encoding != static_cast<jint>(kTransportEncodingPem)
                    && key_encoding != static_cast<jint>(kTransportEncodingDer))
                || !read_java_bytes(env, key_bytes, private_key)) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            return static_cast<jlong>(0);
        }

        std::vector<uint64_t> timeout_values;
        if (!read_long_array(env, raw_timeouts, 12, timeout_values, 12)) return static_cast<jlong>(0);
        auto cert_views = transport_input_views(certificates);
        auto trust_views = transport_input_views(trust_certificates);
        auto crl_views = transport_input_views(revocation_lists);
        const uint32_t key_source_value = static_cast<uint32_t>(key_source);
        const uint32_t key_encoding_value = static_cast<uint32_t>(key_encoding);
        kmipkit_transport_config_v1_t configuration{};
        configuration.struct_size = static_cast<uint32_t>(sizeof(configuration));
        configuration.abi_version = 1;
        configuration.endpoint_kind = static_cast<uint32_t>(endpoint_kind);
        configuration.trust_kind = static_cast<uint32_t>(trust_kind);
        configuration.flags = (tls_server_name == nullptr ? 0U : kTransportFlagTlsServerName)
                | (target_uri == nullptr ? 0U : kTransportFlagTargetUri);
        configuration.endpoint = {endpoint_data.data(), static_cast<uint64_t>(endpoint_data.size())};
        configuration.endpoint_port = static_cast<uint16_t>(endpoint_port);
        configuration.client_certificates = cert_views.empty() ? nullptr : cert_views.data();
        configuration.client_certificate_count = static_cast<uint64_t>(cert_views.size());
        configuration.client_private_key = {key_source_value, key_encoding_value,
                {private_key.data(), static_cast<uint64_t>(private_key.size())}};
        configuration.trust_certificates = trust_views.empty() ? nullptr : trust_views.data();
        configuration.trust_certificate_count = static_cast<uint64_t>(trust_views.size());
        configuration.revocation_lists = crl_views.empty() ? nullptr : crl_views.data();
        configuration.revocation_list_count = static_cast<uint64_t>(crl_views.size());
        configuration.tls_server_name = {tls_server_name_data.empty() ? nullptr : tls_server_name_data.data(),
                static_cast<uint64_t>(tls_server_name_data.size())};
        configuration.target_uri = {target_uri_data.empty() ? nullptr : target_uri_data.data(),
                static_cast<uint64_t>(target_uri_data.size())};
        kmipkit_timeout_limit_v1_t* timeout_outputs[] = {
                &configuration.connect_timeout, &configuration.write_timeout,
                &configuration.read_timeout, &configuration.total_timeout};
        for (size_t index = 0; index < 4; ++index) {
            const uint64_t kind = timeout_values[index * 3];
            const uint64_t seconds = timeout_values[index * 3 + 1];
            const uint64_t nanos = timeout_values[index * 3 + 2];
            if (kind > kTransportTimeoutUnbounded || nanos >= 1'000'000'000) {
                throw_invalid_input(env);
                return static_cast<jlong>(0);
            }
            timeout_outputs[index]->kind = static_cast<uint32_t>(kind);
            timeout_outputs[index]->seconds = seconds;
            timeout_outputs[index]->nanoseconds = static_cast<uint32_t>(nanos);
        }
        configuration.max_request_bytes = static_cast<uint64_t>(max_request_bytes);
        return return_handle<kmipkit_transport_config_t>(env, [&](auto output) {
            return kmipkit_transport_config_create_v1(&configuration, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientCreate(
        JNIEnv* env, jclass, jlong raw_configuration, jlong raw_transport) {
    return guarded<jlong>(env, 0, [&] {
        auto* configuration = from_java_handle<kmipkit_client_configuration_t>(raw_configuration);
        auto* transport = from_java_handle<kmipkit_transport_config_t>(raw_transport);
        if (configuration == nullptr || transport == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        kmipkit_client_t* client = nullptr;
        const int32_t status = kmipkit_client_create(configuration, &transport, &client);
        if (status != 0) throw_status(env, status);
        if (transport != nullptr) kmipkit_transport_config_release(transport);
        if (status != 0) return static_cast<jlong>(0);
        if (client == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(client);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchCreate(
        JNIEnv* env, jclass) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_t>(env, kmipkit_client_batch_create);
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchAddItem(
        JNIEnv* env, jclass, jlong raw_batch, jlong raw_item) {
    guarded<int>(env, 0, [&] {
        auto* batch = from_java_handle<kmipkit_client_batch_t>(raw_batch);
        auto* item = from_java_handle<kmipkit_client_batch_item_t>(raw_item);
        if (batch == nullptr || item == nullptr) {
            throw_invalid_input(env);
            return 0;
        }
        const int32_t status = kmipkit_client_batch_add_item(batch, &item);
        if (status != 0) throw_status(env, status);
        if (item != nullptr) kmipkit_client_batch_item_release(item);
        if (status == 0 && item != nullptr) throw_invalid_input(env);
        return 0;
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchSetAsynchronousIndicator(
        JNIEnv* env, jclass, jlong raw_batch, jlong raw_indicator) {
    guarded<int>(env, 0, [&] {
        if (raw_batch == 0 || raw_indicator < 0 || raw_indicator > 0xffff'ffffLL) {
            throw_invalid_input(env);
            return 0;
        }
        const int32_t status = kmipkit_client_batch_set_asynchronous_indicator(
                from_java_handle<kmipkit_client_batch_t>(raw_batch),
                static_cast<uint32_t>(raw_indicator));
        if (status != 0) throw_status(env, status);
        return 0;
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchSetRequestTimestamp(
        JNIEnv* env, jclass, jlong raw_batch, jlong timestamp) {
    guarded<int>(env, 0, [&] {
        if (raw_batch == 0) {
            throw_invalid_input(env);
            return 0;
        }
        const int32_t status = kmipkit_client_batch_set_request_time_stamp(
                from_java_handle<kmipkit_client_batch_t>(raw_batch),
                static_cast<int64_t>(timestamp));
        if (status != 0) throw_status(env, status);
        return 0;
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchSetOrderOption(
        JNIEnv* env, jclass, jlong raw_batch, jboolean enabled) {
    guarded<int>(env, 0, [&] {
        if (raw_batch == 0) {
            throw_invalid_input(env);
            return 0;
        }
        const int32_t status = kmipkit_client_batch_set_order_option(
                from_java_handle<kmipkit_client_batch_t>(raw_batch),
                enabled == JNI_TRUE ? UINT8_C(1) : UINT8_C(0));
        if (status != 0) throw_status(env, status);
        return 0;
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchAddErrorContinuationValue(
        JNIEnv* env, jclass, jlong raw_batch, jlong raw_value) {
    guarded<int>(env, 0, [&] {
        if (raw_batch == 0 || raw_value < 0 || raw_value > 0xffff'ffffLL) {
            throw_invalid_input(env);
            return 0;
        }
        const int32_t status = kmipkit_client_batch_add_error_continuation_value(
                from_java_handle<kmipkit_client_batch_t>(raw_batch),
                static_cast<uint32_t>(raw_value));
        if (status != 0) throw_status(env, status);
        return 0;
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchSetClientCorrelationValue(
        JNIEnv* env, jclass, jlong raw_batch, jbyteArray raw_value) {
    guarded<int>(env, 0, [&] {
        if (raw_batch == 0 || raw_value == nullptr) {
            throw_invalid_input(env);
            return 0;
        }
        const jsize length = env->GetArrayLength(raw_value);
        if (env->ExceptionCheck()) return 0;
        if (length < 0 || length > 16 * 1024 * 1024) {
            throw_invalid_input(env);
            return 0;
        }
        std::vector<jbyte> bytes(static_cast<size_t>(length));
        if (length > 0) {
            env->GetByteArrayRegion(raw_value, 0, length, bytes.data());
            if (env->ExceptionCheck()) return 0;
        }
        const int32_t status = kmipkit_client_batch_set_client_correlation_value(
                from_java_handle<kmipkit_client_batch_t>(raw_batch),
                reinterpret_cast<const uint8_t*>(bytes.data()),
                static_cast<uint64_t>(bytes.size()));
        if (status != 0) throw_status(env, status);
        return 0;
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExecute(
        JNIEnv* env, jclass, jlong raw_client, jlong raw_batch, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        auto* client = from_java_handle<kmipkit_client_t>(raw_client);
        auto* batch = from_java_handle<kmipkit_client_batch_t>(raw_batch);
        if (client == nullptr || batch == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        kmipkit_client_batch_response_t* response = nullptr;
        const int32_t status = kmipkit_client_execute(client, &batch, limits.get(), &response);
        if (status != 0) throw_status(env, status);
        if (batch != nullptr) kmipkit_client_batch_release(batch);
        if (status != 0) return static_cast<jlong>(0);
        if (response == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(response);
    });
}

struct ClientBatchReleaseGuard {
    kmipkit_client_batch_t*& handle;
    ~ClientBatchReleaseGuard() {
        if (handle != nullptr) {
            kmipkit_client_batch_release(handle);
            handle = nullptr;
        }
    }
};

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExecuteWithOptions(
        JNIEnv* env, jclass, jlong raw_client, jlong raw_batch,
        jlongArray raw_limits, jlongArray raw_timeouts) {
    return guarded<jlong>(env, 0, [&] {
        auto* batch = from_java_handle<kmipkit_client_batch_t>(raw_batch);
        ClientBatchReleaseGuard batch_guard{batch};
        auto* client = from_java_handle<kmipkit_client_t>(raw_client);
        if (client == nullptr || batch == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        std::vector<uint64_t> timeout_values;
        if (!read_long_array(env, raw_timeouts, 12, timeout_values, 12)) {
            return static_cast<jlong>(0);
        }

        kmipkit_request_options_v1_t options{};
        options.struct_size = static_cast<uint32_t>(sizeof(options));
        options.abi_version = 1;
        kmipkit_timeout_limit_v1_t* outputs[] = {
                &options.connect_timeout, &options.write_timeout,
                &options.read_timeout, &options.total_timeout};
        for (size_t index = 0; index < 4; ++index) {
            const uint64_t kind = timeout_values[index * 3];
            const uint64_t seconds = timeout_values[index * 3 + 1];
            const uint64_t nanos = timeout_values[index * 3 + 2];
            if (kind > kTransportTimeoutUnbounded || nanos >= 1'000'000'000
                    || ((kind == kTransportTimeoutDefault
                            || kind == kTransportTimeoutUnbounded)
                        && (seconds != 0 || nanos != 0))) {
                throw_invalid_input(env);
                return static_cast<jlong>(0);
            }
            outputs[index]->kind = static_cast<uint32_t>(kind);
            outputs[index]->seconds = seconds;
            outputs[index]->nanoseconds = static_cast<uint32_t>(nanos);
        }

        kmipkit_client_batch_response_t* response = nullptr;
        const int32_t status = kmipkit_client_execute_with_options_v1(
                client, &batch, limits.get(), &options, &response);
        if (status != 0) throw_status(env, status);
        if (status != 0) return static_cast<jlong>(0);
        if (response == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(response);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchResponseItemCount(
        JNIEnv* env, jclass, jlong raw_response) {
    return guarded<jlong>(env, 0, [&] {
        uint64_t count = 0;
        const int32_t status = kmipkit_client_batch_response_item_count(
                from_java_handle<kmipkit_client_batch_response_t>(raw_response), &count);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        if (count > static_cast<uint64_t>(std::numeric_limits<jlong>::max())) {
            throw_java(env, "org/kmipkit/ResourceLimitException", "response count exceeds Java range");
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(count);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchResponseItemAt(
        JNIEnv* env, jclass, jlong raw_response, jlong raw_index) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_index < 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_response_t>(env, [&](auto output) {
            return kmipkit_client_batch_response_item_at(
                    from_java_handle<kmipkit_client_batch_response_t>(raw_response),
                    static_cast<uint64_t>(raw_index), output);
        });
    });
}

JNIEXPORT jlongArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseMetadata(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jlongArray>(env, nullptr, [&] {
        kmipkit_client_batch_item_metadata_v1_t metadata{};
        metadata.struct_size = static_cast<uint32_t>(sizeof(metadata));
        metadata.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_metadata_v1(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &metadata);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlongArray>(nullptr);
        }
        const jlong values[] = {static_cast<jlong>(metadata.outcome_kind),
                static_cast<jlong>(metadata.result_status), static_cast<jlong>(metadata.has_result_reason),
                static_cast<jlong>(metadata.result_reason)};
        jlongArray result = env->NewLongArray(4);
        if (result != nullptr) env->SetLongArrayRegion(result, 0, 4, values);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseMetadataV2(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        kmipkit_client_batch_item_metadata_v2_t metadata{};
        metadata.struct_size = static_cast<uint32_t>(sizeof(metadata));
        metadata.abi_version = 2;
        const int32_t status = kmipkit_client_batch_item_response_metadata_v2(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &metadata);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (metadata.abi_version != 2 || metadata.struct_size != sizeof(metadata)
                || metadata.reserved != 0 || metadata.has_result_message > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(metadata.operation), static_cast<jlong>(metadata.outcome_kind),
                static_cast<jlong>(metadata.result_status), static_cast<jlong>(metadata.has_result_reason),
                static_cast<jlong>(metadata.result_reason), static_cast<jlong>(metadata.has_result_message)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        const kmipkit_byte_span_v1_t message_span {
                metadata.result_message_data, metadata.result_message_length};
        jbyteArray message = span_to_java_bytes(env,
                static_cast<uint8_t>(metadata.has_result_message), message_span);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_pair(env, fields, message);
        env->DeleteLocalRef(fields);
        if (message != nullptr) env->DeleteLocalRef(message);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseCreate(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint32_t object_type = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_create(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &object_type, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        const std::vector<jlong> fields {static_cast<jlong>(present),
                static_cast<jlong>(object_type), static_cast<jlong>(identifier.kind),
                static_cast<jlong>(identifier.enumeration_value),
                static_cast<jlong>(identifier.integer_value)};
        jlongArray scalar_fields = make_long_array(env, fields);
        if (scalar_fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jbyteArray text = present == 1 && identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
                ? span_to_java_bytes(env, 1,
                        {identifier.text_data, identifier.text_length}) : nullptr;
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(scalar_fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_pair(env, scalar_fields, text);
        env->DeleteLocalRef(scalar_fields);
        if (text != nullptr) env->DeleteLocalRef(text);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseCreateKeyPair(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t private_identifier{};
        kmipkit_unique_identifier_v1_t public_identifier{};
        private_identifier.struct_size = public_identifier.struct_size =
                static_cast<uint32_t>(sizeof(kmipkit_unique_identifier_v1_t));
        private_identifier.abi_version = public_identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_create_key_pair(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &present,
                &private_identifier, &public_identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray private_value = identifier_to_java(env, private_identifier, present);
        jobjectArray public_value = identifier_to_java(env, public_identifier, present);
        if (private_value == nullptr || public_value == nullptr) {
            if (private_value != nullptr) env->DeleteLocalRef(private_value);
            if (public_value != nullptr) env->DeleteLocalRef(public_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray present_value = make_long_array(env, {static_cast<jlong>(present)});
        if (present_value == nullptr) {
            env->DeleteLocalRef(private_value);
            env->DeleteLocalRef(public_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jclass object_class = env->FindClass("java/lang/Object");
        if (object_class == nullptr) return static_cast<jobjectArray>(nullptr);
        jobjectArray result = env->NewObjectArray(3, object_class, nullptr);
        env->DeleteLocalRef(object_class);
        if (result != nullptr) {
            env->SetObjectArrayElement(result, 0, present_value);
            env->SetObjectArrayElement(result, 1, private_value);
            env->SetObjectArrayElement(result, 2, public_value);
        }
        env->DeleteLocalRef(present_value);
        env->DeleteLocalRef(private_value);
        env->DeleteLocalRef(public_value);
        return env->ExceptionCheck() ? nullptr : result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseCreateSplitKey(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        uint8_t present = 0;
        uint64_t count = 0;
        int32_t status = kmipkit_client_batch_item_response_create_split_key_count(item, &present, &count);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 0 && count != 0) || (present == 1 && count == 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        if (count > static_cast<uint64_t>(std::numeric_limits<jsize>::max())) {
            throw_status(env, KMIPKIT_ERROR_RESOURCE_LIMIT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present), static_cast<jlong>(count)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jclass object_class = env->FindClass("java/lang/Object");
        if (object_class == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifiers = env->NewObjectArray(static_cast<jsize>(count), object_class, nullptr);
        env->DeleteLocalRef(object_class);
        if (identifiers == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        for (uint64_t index = 0; index < count; ++index) {
            kmipkit_unique_identifier_v1_t identifier{};
            identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
            identifier.abi_version = 1;
            status = kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
                    item, index, &identifier);
            if (status != 0) {
                throw_status(env, status);
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                return static_cast<jobjectArray>(nullptr);
            }
            jobjectArray value = identifier_to_java(env, identifier, 1);
            if (value == nullptr) {
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                return static_cast<jobjectArray>(nullptr);
            }
            env->SetObjectArrayElement(identifiers, static_cast<jsize>(index), value);
            env->DeleteLocalRef(value);
            if (env->ExceptionCheck()) {
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                return static_cast<jobjectArray>(nullptr);
            }
        }
        jobjectArray result = make_object_pair(env, fields, identifiers);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(identifiers);
        return result;
    });
}

JNIEXPORT jbyteArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseUniqueBatchItemId(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jbyteArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_byte_span_v1_t value{};
        const int32_t status = kmipkit_client_batch_item_response_unique_batch_item_id(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &present, &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jbyteArray>(nullptr);
        }
        return span_to_java_bytes(env, present, value);
    });
}

JNIEXPORT jbyteArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponsePendingCorrelationValue(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jbyteArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_byte_span_v1_t value{};
        const int32_t status = kmipkit_client_batch_item_response_pending_correlation_value(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &present, &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jbyteArray>(nullptr);
        }
        return span_to_java_bytes(env, present, value);
    });
}

JNIEXPORT jlongArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseDiscoverVersionsSupportedVersions(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jlongArray>(env, nullptr, [&] {
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        uint8_t present = 0;
        uint64_t count = 0;
        int32_t status = kmipkit_client_batch_item_response_discover_versions_supported_versions(
                item, &present, nullptr, 0, &count);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlongArray>(nullptr);
        }
        if (present == 0) {
            if (count != 0) {
                throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            }
            return static_cast<jlongArray>(nullptr);
        }
        if (present != 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jlongArray>(nullptr);
        }
        const uint64_t maximum_supported_versions =
                static_cast<uint64_t>(std::numeric_limits<jsize>::max() / 2);
        if (count > maximum_supported_versions) {
            throw_status(env, KMIPKIT_ERROR_RESOURCE_LIMIT);
            return static_cast<jlongArray>(nullptr);
        }

        std::vector<kmipkit_protocol_version_v1_t> versions(static_cast<size_t>(count));
        if (count != 0) {
            uint8_t copied_present = 0;
            uint64_t copied_count = 0;
            status = kmipkit_client_batch_item_response_discover_versions_supported_versions(
                    item, &copied_present, versions.data(), count, &copied_count);
            if (status != 0) {
                throw_status(env, status);
                return static_cast<jlongArray>(nullptr);
            }
            if (copied_present != 1 || copied_count != count) {
                throw_status(env, KMIPKIT_ERROR_INTERNAL);
                return static_cast<jlongArray>(nullptr);
            }
        }

        const jsize value_count = static_cast<jsize>(count * 2);
        jlongArray result = env->NewLongArray(value_count);
        if (result == nullptr) {
            return static_cast<jlongArray>(nullptr);
        }
        std::vector<jlong> values(static_cast<size_t>(value_count));
        for (size_t index = 0; index < versions.size(); ++index) {
            values[index * 2] = static_cast<jlong>(versions[index].major);
            values[(index * 2) + 1] = static_cast<jlong>(versions[index].minor);
        }
        if (value_count != 0) {
            env->SetLongArrayRegion(result, 0, value_count, values.data());
        }
        return result;
    });
}

JNIEXPORT jobject JNICALL Java_org_kmipkit_RuntimeMetadata_version(JNIEnv* env, jclass) {
    return guarded<jobject>(env, nullptr, [&] {
        uint32_t major = 0;
        uint32_t minor = 0;
        uint32_t patch = 0;
        uint32_t abi_version = 0;
        const int32_t status = kmipkit_version_get(&major, &minor, &patch, &abi_version);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobject>(nullptr);
        }

        return four_integer_record(
                env,
                "org/kmipkit/RuntimeMetadata$Version",
                static_cast<jint>(major),
                static_cast<jint>(minor),
                static_cast<jint>(patch),
                static_cast<jint>(abi_version));
    });
}

JNIEXPORT jboolean JNICALL Java_org_kmipkit_RuntimeMetadata_capabilityIsAvailable(
        JNIEnv* env, jclass, jstring capability_id) {
    return guarded<jboolean>(env, JNI_FALSE, [&] {
        std::vector<uint8_t> capability_id_bytes;
        if (!java_string_to_utf8(env, capability_id, capability_id_bytes)) return JNI_FALSE;
        uint8_t available = 0;
        const int32_t status = kmipkit_capability_is_available(
                capability_id_bytes.empty() ? nullptr : capability_id_bytes.data(),
                static_cast<uint64_t>(capability_id_bytes.size()),
                &available);
        if (status != 0) {
            throw_status(env, status);
            return JNI_FALSE;
        }
        return available == 0 ? JNI_FALSE : JNI_TRUE;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_RuntimeMetadata_apiLevelsNative(
        JNIEnv* env, jclass) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        kmipkit_byte_span_v1_t high_level{};
        kmipkit_byte_span_v1_t typed_protocol{};
        kmipkit_byte_span_v1_t generic_ttlv{};
        const int32_t status = kmipkit_api_levels_get_v1(
                &high_level, &typed_protocol, &generic_ttlv);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }

        const std::array<kmipkit_byte_span_v1_t, 3> spans = {
                high_level, typed_protocol, generic_ttlv};
        jclass string_class = env->FindClass("java/lang/String");
        if (string_class == nullptr) {
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = env->NewObjectArray(3, string_class, nullptr);
        env->DeleteLocalRef(string_class);
        if (result == nullptr) {
            return static_cast<jobjectArray>(nullptr);
        }

        for (jsize index = 0; index < static_cast<jsize>(spans.size()); ++index) {
            const auto& span = spans[static_cast<size_t>(index)];
            if (span.data == nullptr || span.length > static_cast<uint64_t>(INT32_MAX)) {
                throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
                return static_cast<jobjectArray>(nullptr);
            }
            const std::string value(
                    reinterpret_cast<const char*>(span.data),
                    static_cast<size_t>(span.length));
            jstring item = env->NewStringUTF(value.c_str());
            if (item == nullptr) {
                return static_cast<jobjectArray>(nullptr);
            }
            env->SetObjectArrayElement(result, index, item);
            env->DeleteLocalRef(item);
            if (env->ExceptionCheck()) {
                return static_cast<jobjectArray>(nullptr);
            }
        }
        return result;
    });
}

JNIEXPORT jobject JNICALL Java_org_kmipkit_RuntimeMetadata_lastErrorSnapshot(
        JNIEnv* env, jclass) {
    return guarded<jobject>(env, nullptr, [&] {
        int32_t status_value = 0;
        uint32_t category = 0;
        uint32_t cause = 0;
        uint32_t delivery = 0;
        const int32_t status = kmipkit_last_error_get(
                &status_value, &category, &cause, &delivery);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobject>(nullptr);
        }

        return four_integer_record(
                env,
                "org/kmipkit/RuntimeMetadata$ErrorSnapshot",
                static_cast<jint>(status_value),
                static_cast<jint>(category),
                static_cast<jint>(cause),
                static_cast<jint>(delivery));
    });
}

JNIEXPORT void JNICALL Java_org_kmipkit_RuntimeMetadata_clearLastError(JNIEnv* env, jclass) {
    const int32_t status = kmipkit_last_error_clear();
    if (status != 0) {
        throw_status(env, status);
    }
}

JNIEXPORT jint JNICALL Java_org_kmipkit_NativeExtensionRegistry_nativeAbiVersion(
        JNIEnv*, jclass) {
    uint32_t major = 0;
    uint32_t minor = 0;
    uint32_t patch = 0;
    uint32_t abi_version = 0;
    const int32_t status = kmipkit_version_get(&major, &minor, &patch, &abi_version);
    return status == 0 ? static_cast<jint>(abi_version) : static_cast<jint>(-1);
}

} // extern "C"
