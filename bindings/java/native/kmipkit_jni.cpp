#include <jni.h>
#include "kmipkit_internal.h"
#include "native_handle_transfer_guard.hpp"
#include "unique_batch_item_id_validation.hpp"
#include "response_identifier_validation.hpp"
#include "response_byte_span_validation.hpp"
#include "response_array_conversion.hpp"
#include "response_secret_array_scope.hpp"
#include "response_bridge.hpp"
#include "secure_bytes.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <initializer_list>
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
typedef struct kmipkit_usage_allocation_context_t kmipkit_usage_allocation_context_t;
typedef struct kmipkit_revocation_reason_t kmipkit_revocation_reason_t;
typedef struct kmipkit_revoke_request_t kmipkit_revoke_request_t;

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
extern "C" int32_t kmipkit_client_batch_item_ping(kmipkit_client_batch_item_t** out_item);
extern "C" int32_t kmipkit_client_batch_item_query(
        const uint32_t* functions, uint64_t function_count, uint8_t has_object_groups,
        const kmipkit_byte_span_v1_t* object_groups, uint64_t object_group_count,
        kmipkit_client_batch_item_t** out_item);
extern "C" int32_t kmipkit_client_batch_item_query_with_limits(
        kmipkit_codec_limits_t* limits, const uint32_t* functions, uint64_t function_count,
        uint8_t has_object_groups, const kmipkit_byte_span_v1_t* object_groups,
        uint64_t object_group_count, kmipkit_client_batch_item_t** out_item);
extern "C" int32_t kmipkit_client_batch_item_pkcs11(
        kmipkit_ttlv_structure_t* function, kmipkit_ttlv_structure_t* interface_structure,
        const kmipkit_byte_span_v1_t* correlation_value,
        kmipkit_ttlv_structure_t* input_parameters, kmipkit_client_batch_item_t** out_item);
extern "C" int32_t kmipkit_client_batch_item_response_ping(
        kmipkit_client_batch_item_response_t* item, uint8_t* out_present);
extern "C" int32_t kmipkit_client_batch_item_response_query_ttlv(
        kmipkit_client_batch_item_response_t* item, uint8_t* out_present,
        kmipkit_ttlv_structure_t** out_response_ttlv);
extern "C" int32_t kmipkit_client_batch_item_response_pkcs11_ttlv(
        kmipkit_client_batch_item_response_t* item, uint8_t* out_present,
        kmipkit_ttlv_structure_t** out_response_ttlv);

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
int32_t kmipkit_client_batch_item_hash(
        kmipkit_ttlv_structure_t*, kmipkit_ttlv_value_t*, const kmipkit_byte_span_v1_t*,
        uint8_t, uint8_t, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_mac(
        const kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t*, kmipkit_ttlv_value_t*,
        const kmipkit_byte_span_v1_t*, uint8_t, uint8_t, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_mac_verify(
        const kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t*, kmipkit_ttlv_value_t*,
        const kmipkit_byte_span_v1_t*, const kmipkit_byte_span_v1_t*, uint8_t, uint8_t,
        kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_sign(
        const kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t*, kmipkit_ttlv_value_t*,
        const kmipkit_byte_span_v1_t*, const kmipkit_byte_span_v1_t*, uint8_t, uint8_t,
        kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_signature_verify(
        const kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t*, kmipkit_ttlv_value_t*,
        const kmipkit_byte_span_v1_t*, const kmipkit_byte_span_v1_t*,
        const kmipkit_byte_span_v1_t*, uint8_t, uint8_t, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_export(
        const kmipkit_unique_identifier_v1_t*, uint8_t, uint32_t, uint8_t, uint32_t,
        uint8_t, uint32_t, kmipkit_ttlv_structure_t*, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_get(
        const kmipkit_unique_identifier_v1_t*, uint8_t, uint32_t, uint8_t, uint32_t,
        uint8_t, uint32_t, kmipkit_ttlv_structure_t*, kmipkit_client_batch_item_t**);
int32_t kmipkit_client_batch_item_locate(
        uint8_t, int32_t, uint8_t, int32_t, uint8_t, uint32_t, uint8_t, uint32_t,
        kmipkit_ttlv_structure_t*, kmipkit_client_batch_item_t**);
void kmipkit_client_batch_item_release(kmipkit_client_batch_item_t*);
void kmipkit_client_batch_release(kmipkit_client_batch_t*);
int32_t kmipkit_client_execute(kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*, kmipkit_client_batch_response_t**);
int32_t kmipkit_client_execute_with_options_v1(kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*, const kmipkit_request_options_v1_t*, kmipkit_client_batch_response_t**);
int32_t kmipkit_client_execute_with_error_ttlv_v1(
        kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*,
        kmipkit_client_batch_response_t**, kmipkit_ttlv_structure_t**);
int32_t kmipkit_client_execute_with_options_and_error_ttlv_v1(
        kmipkit_client_t*, kmipkit_client_batch_t**, kmipkit_codec_limits_t*,
        const kmipkit_request_options_v1_t*, kmipkit_client_batch_response_t**,
        kmipkit_ttlv_structure_t**);
int32_t kmipkit_usage_allocation_context_create(
        kmipkit_client_batch_item_response_t*, int64_t, uint32_t,
        kmipkit_usage_allocation_context_t**);
void kmipkit_usage_allocation_context_release(kmipkit_usage_allocation_context_t*);
int32_t kmipkit_usage_allocation_context_usage_limits_unit(
        kmipkit_usage_allocation_context_t*, uint32_t*);
int32_t kmipkit_usage_allocation_context_allocated_units(
        kmipkit_usage_allocation_context_t*, int64_t*);
int32_t kmipkit_usage_allocation_context_available_units(
        kmipkit_usage_allocation_context_t*, int64_t*);
int32_t kmipkit_usage_allocation_context_reserved_units(
        kmipkit_usage_allocation_context_t*, int64_t*);
int32_t kmipkit_usage_allocation_context_unique_identifier(
        kmipkit_usage_allocation_context_t*, kmipkit_unique_identifier_v1_t*);
int32_t kmipkit_client_execute_guarded_protection_item(
        kmipkit_client_t*, kmipkit_usage_allocation_context_t*,
        kmipkit_client_batch_item_t**, int64_t, kmipkit_codec_limits_t*,
        kmipkit_client_batch_response_t**);
int32_t kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
        kmipkit_client_t*, kmipkit_usage_allocation_context_t*,
        kmipkit_client_batch_item_t**, int64_t, uint32_t, kmipkit_codec_limits_t*,
        kmipkit_client_batch_response_t**);
void kmipkit_client_batch_response_release(kmipkit_client_batch_response_t*);
void kmipkit_client_batch_item_response_release(kmipkit_client_batch_item_response_t*);
int32_t kmipkit_client_batch_response_item_count(kmipkit_client_batch_response_t*, uint64_t*);
int32_t kmipkit_client_batch_response_item_at(kmipkit_client_batch_response_t*, uint64_t, kmipkit_client_batch_item_response_t**);
int32_t kmipkit_client_batch_item_response_metadata_v1(kmipkit_client_batch_item_response_t*, kmipkit_client_batch_item_metadata_v1_t*);
int32_t kmipkit_client_batch_item_response_export(
        kmipkit_client_batch_item_response_t*, uint8_t*, uint8_t*, uint32_t*,
        kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t**, kmipkit_ttlv_structure_t**);
int32_t kmipkit_client_batch_item_response_export_ttlv(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_ttlv_structure_t**);
int32_t kmipkit_client_batch_item_response_get(
        kmipkit_client_batch_item_response_t*, uint8_t*, uint8_t*, uint32_t*,
        kmipkit_unique_identifier_v1_t*, kmipkit_ttlv_structure_t**);
int32_t kmipkit_client_batch_item_response_locate(
        kmipkit_client_batch_item_response_t*, uint8_t*, uint8_t*, uint8_t*, int32_t*, uint64_t*);
int32_t kmipkit_client_batch_item_response_locate_unique_identifier_at(
        kmipkit_client_batch_item_response_t*, uint64_t, kmipkit_unique_identifier_v1_t*);
int32_t kmipkit_client_batch_item_response_hash(
        kmipkit_client_batch_item_response_t*, uint8_t*, uint8_t*, kmipkit_byte_span_v1_t*,
        uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_mac(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_unique_identifier_v1_t*,
        uint8_t*, kmipkit_byte_span_v1_t*, uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_mac_verify(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_unique_identifier_v1_t*,
        uint8_t*, uint32_t*, uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_sign(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_unique_identifier_v1_t*,
        uint8_t*, kmipkit_byte_span_v1_t*, uint8_t*, kmipkit_byte_span_v1_t*);
int32_t kmipkit_client_batch_item_response_signature_verify(
        kmipkit_client_batch_item_response_t*, uint8_t*, kmipkit_unique_identifier_v1_t*,
        uint8_t*, uint32_t*, uint8_t*, kmipkit_byte_span_v1_t*, uint8_t*, kmipkit_byte_span_v1_t*);
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
constexpr uint64_t kMaximumTtlvItemLength = std::numeric_limits<uint32_t>::max();

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

void throw_client_operation_status(
        JNIEnv* env, int32_t status,
        kmipkit_ttlv_structure_t* response_ttlv = nullptr) noexcept {
    int32_t native_status = status;
    uint32_t category = 0;
    uint32_t cause = 0;
    uint32_t delivery = 0;
    if (kmipkit_last_error_get(&native_status, &category, &cause, &delivery) != 0) {
        if (response_ttlv != nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        throw_java(env, "org/kmipkit/KmipException", "native operation failed");
        return;
    }
    const char* class_name = status == kErrorProtocol ? "org/kmipkit/ProtocolException"
            : status == kErrorTransport ? "org/kmipkit/TransportException"
            : "org/kmipkit/ServerResultException";
    jclass exception_class = env->FindClass(class_name);
    if (exception_class == nullptr) {
        if (response_ttlv != nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        return;
    }
    const bool protocol_error = status == kErrorProtocol;
    jmethodID constructor = env->GetMethodID(
            exception_class, "<init>", protocol_error ? "(IIIIJ)V" : "(IIII)V");
    if (constructor == nullptr) {
        if (response_ttlv != nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(exception_class);
        return;
    }
    jobject exception = protocol_error
            ? env->NewObject(exception_class, constructor,
                    static_cast<jint>(native_status), static_cast<jint>(category),
                    static_cast<jint>(cause), static_cast<jint>(delivery),
                    static_cast<jlong>(reinterpret_cast<uintptr_t>(response_ttlv)))
            : env->NewObject(exception_class, constructor,
                    static_cast<jint>(native_status), static_cast<jint>(category),
                    static_cast<jint>(cause), static_cast<jint>(delivery));
    if (exception != nullptr) {
        if (protocol_error) response_ttlv = nullptr;
        env->Throw(static_cast<jthrowable>(exception));
        env->DeleteLocalRef(exception);
    }
    if (response_ttlv != nullptr) kmipkit_ttlv_structure_release(response_ttlv);
    env->DeleteLocalRef(exception_class);
}

void throw_status(
        JNIEnv* env, int32_t status,
        kmipkit_ttlv_structure_t* response_ttlv = nullptr) noexcept {
    if (status == kErrorProtocol || status == kErrorTransport || status == kErrorServerResult) {
        throw_client_operation_status(env, status, response_ttlv);
        return;
    }
    if (response_ttlv != nullptr) kmipkit_ttlv_structure_release(response_ttlv);
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
        max_structure_depth_ = limits[1];
        max_elements_ = limits[2];
        return value_ != nullptr;
    }

    kmipkit_codec_limits_t* get() const noexcept { return value_; }
    uint64_t max_message_bytes() const noexcept { return max_message_bytes_; }
    uint64_t max_structure_depth() const noexcept { return max_structure_depth_; }
    uint64_t max_elements() const noexcept { return max_elements_; }

private:
    kmipkit_codec_limits_t* value_ = nullptr;
    uint64_t max_message_bytes_ = 0;
    uint64_t max_structure_depth_ = 0;
    uint64_t max_elements_ = 0;
};

bool preflight_query_inputs(JNIEnv* env, jintArray raw_functions, jobjectArray raw_object_groups,
        uint64_t max_message_bytes, uint64_t max_structure_depth, uint64_t max_elements) {
    if (raw_functions == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    const jsize function_count = env->GetArrayLength(raw_functions);
    if (function_count <= 0) {
        throw_invalid_input(env);
        return false;
    }
    const jsize group_count = raw_object_groups == nullptr
            ? 0 : env->GetArrayLength(raw_object_groups);
    const uint64_t required_depth = raw_object_groups == nullptr ? 3 : 4;
    if (required_depth > max_structure_depth) {
        throw_status(env, 6);
        return false;
    }

    uint64_t estimated_elements = static_cast<uint64_t>(function_count) + 1;
    uint64_t estimated_bytes = 8 + static_cast<uint64_t>(function_count) * 16;
    uint64_t object_groups_value_length = 0;
    if (raw_object_groups != nullptr) {
        if (estimated_elements > std::numeric_limits<uint64_t>::max() -
                static_cast<uint64_t>(group_count) - 1) {
            throw_status(env, 6);
            return false;
        }
        estimated_elements += static_cast<uint64_t>(group_count) + 1;
        if (estimated_bytes > std::numeric_limits<uint64_t>::max() - 8) {
            throw_status(env, 6);
            return false;
        }
        estimated_bytes += 8;
    }
    if (estimated_elements > max_elements || estimated_bytes > max_message_bytes) {
        throw_status(env, 6);
        return false;
    }

    for (jsize index = 0; index < group_count; ++index) {
        jobject raw_group = env->GetObjectArrayElement(raw_object_groups, index);
        if (raw_group == nullptr) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            return false;
        }
        const jsize length = env->GetArrayLength(reinterpret_cast<jbyteArray>(raw_group));
        env->DeleteLocalRef(raw_group);
        const uint64_t raw_length = static_cast<uint64_t>(length);
        if (raw_length > kMaximumTtlvItemLength ||
                raw_length > std::numeric_limits<uint64_t>::max() - 7) {
            throw_status(env, 6);
            return false;
        }
        const uint64_t padded_length = (raw_length + 7) & ~UINT64_C(7);
        if (estimated_bytes > std::numeric_limits<uint64_t>::max() - 8 - padded_length) {
            throw_status(env, 6);
            return false;
        }
        estimated_bytes += 8 + padded_length;
        if (padded_length > kMaximumTtlvItemLength - 8 ||
                object_groups_value_length > kMaximumTtlvItemLength - 8 - padded_length) {
            throw_status(env, 6);
            return false;
        }
        object_groups_value_length += 8 + padded_length;
        if (estimated_bytes > max_message_bytes) {
            throw_status(env, 6);
            return false;
        }
    }
    if (estimated_bytes < 8 || estimated_bytes - 8 > kMaximumTtlvItemLength) {
        throw_status(env, 6);
        return false;
    }
    return true;
}

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

class ByteVectorListWiper final {
public:
    explicit ByteVectorListWiper(std::vector<std::vector<uint8_t>>& values) noexcept
            : values_(values) {}

    ByteVectorListWiper(const ByteVectorListWiper&) = delete;
    ByteVectorListWiper& operator=(const ByteVectorListWiper&) = delete;

    ~ByteVectorListWiper() noexcept {
        for (auto& value : values_) {
            kmipkit::secure::VectorWiper wiper(value);
        }
    }

private:
    std::vector<std::vector<uint8_t>>& values_;
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

bool read_optional_java_bytes(JNIEnv* env, jbyteArray input,
        std::vector<uint8_t>& output, bool* present) {
    if (present == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    if (input == nullptr) {
        output.clear();
        *present = false;
        return true;
    }
    *present = true;
    return read_bytes(env, input, kMaximumTransportInputBytes, output);
}

kmipkit_byte_span_v1_t byte_span(const std::vector<uint8_t>& bytes) {
    return {bytes.empty() ? nullptr : bytes.data(), static_cast<uint64_t>(bytes.size())};
}

bool read_optional_identifier(JNIEnv* env, jint raw_kind, jbyteArray raw_text,
        jlong raw_number, kmipkit_unique_identifier_v1_t* output,
        std::vector<uint8_t>& text, bool* present) {
    if (output == nullptr || present == nullptr) {
        throw_invalid_input(env);
        return false;
    }
    *output = {};
    *present = false;
    if (raw_kind == 0) {
        if (raw_text != nullptr || raw_number != 0) {
            throw_invalid_input(env);
            return false;
        }
        return true;
    }

    output->struct_size = static_cast<uint32_t>(sizeof(kmipkit_unique_identifier_v1_t));
    output->abi_version = 1;
    output->kind = static_cast<kmipkit_unique_identifier_kind_t>(raw_kind);
    if (raw_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
        bool text_present = false;
        if (raw_number != 0
                || !read_optional_java_bytes(env, raw_text, text, &text_present)
                || !text_present) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            return false;
        }
        output->text_data = text.empty() ? nullptr : text.data();
        output->text_length = static_cast<uint64_t>(text.size());
    } else if (raw_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION) {
        if (raw_text != nullptr || raw_number < 0
                || static_cast<uint64_t>(raw_number) > std::numeric_limits<uint32_t>::max()) {
            throw_invalid_input(env);
            return false;
        }
        output->enumeration_value = static_cast<uint32_t>(raw_number);
    } else if (raw_kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER) {
        if (raw_text != nullptr || raw_number < std::numeric_limits<int32_t>::min()
                || raw_number > std::numeric_limits<int32_t>::max()) {
            throw_invalid_input(env);
            return false;
        }
        output->integer_value = static_cast<int32_t>(raw_number);
    } else {
        throw_invalid_input(env);
        return false;
    }
    *present = true;
    return true;
}

bool read_optional_indicators(JNIEnv* env, jint raw_init, jint raw_final,
        uint8_t* init, uint8_t* final_value) {
    if (init == nullptr || final_value == nullptr
            || raw_init < 0 || raw_init > 2 || raw_final < 0 || raw_final > 2) {
        throw_invalid_input(env);
        return false;
    }
    *init = static_cast<uint8_t>(raw_init);
    *final_value = static_cast<uint8_t>(raw_final);
    return true;
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

void wipe_java_byte_array(JNIEnv* env, jbyteArray value) noexcept {
    if (value == nullptr) return;

    jthrowable original_exception = env->ExceptionOccurred();
    if (original_exception != nullptr) env->ExceptionClear();

    const jsize length = env->GetArrayLength(value);
    if (!env->ExceptionCheck()) {
        constexpr jsize kZeroChunkLength = 256;
        constexpr std::array<jbyte, kZeroChunkLength> zeros{};
        for (jsize offset = 0; offset < length;) {
            const jsize count = std::min(kZeroChunkLength, length - offset);
            env->SetByteArrayRegion(value, offset, count, zeros.data());
            if (env->ExceptionCheck()) break;
            offset += count;
        }
    }

    if (original_exception != nullptr) {
        if (env->ExceptionCheck()) env->ExceptionClear();
        env->Throw(original_exception);
        env->DeleteLocalRef(original_exception);
    }
}

struct WipeAndDeleteLocalByteArray final {
    JNIEnv* env;

    void operator()(jbyteArray value, bool wipe) const noexcept {
        if (wipe) wipe_java_byte_array(env, value);
        env->DeleteLocalRef(value);
    }
};

class DelegatedLoginResponseCleanup final {
public:
    explicit DelegatedLoginResponseCleanup(JNIEnv* env) : env_(env) {}

    DelegatedLoginResponseCleanup(const DelegatedLoginResponseCleanup&) = delete;
    DelegatedLoginResponseCleanup& operator=(const DelegatedLoginResponseCleanup&) = delete;

    ~DelegatedLoginResponseCleanup() {
        if (secret_value_ != nullptr) {
            wipe_java_byte_array(env_, secret_value_);
            env_->DeleteLocalRef(secret_value_);
        }
        kmipkit_ttlv_structure_release(ticket_ttlv_);
    }

    void set_secret_value(jbyteArray value) noexcept { secret_value_ = value; }
    void set_ticket_ttlv(kmipkit_ttlv_structure_t* value) noexcept { ticket_ttlv_ = value; }

    void transfer_to_java() noexcept {
        secret_value_ = nullptr;
        ticket_ttlv_ = nullptr;
    }

private:
    JNIEnv* env_;
    jbyteArray secret_value_ = nullptr;
    kmipkit_ttlv_structure_t* ticket_ttlv_ = nullptr;
};

class LoginResponseCleanup final {
public:
    explicit LoginResponseCleanup(JNIEnv* env) : env_(env) {}

    LoginResponseCleanup(const LoginResponseCleanup&) = delete;
    LoginResponseCleanup& operator=(const LoginResponseCleanup&) = delete;

    ~LoginResponseCleanup() {
        if (secret_value_ != nullptr) {
            wipe_java_byte_array(env_, secret_value_);
            env_->DeleteLocalRef(secret_value_);
        }
        kmipkit_ttlv_structure_release(ticket_ttlv_);
    }

    void set_secret_value(jbyteArray value) noexcept { secret_value_ = value; }
    void set_ticket_ttlv(kmipkit_ttlv_structure_t* value) noexcept { ticket_ttlv_ = value; }

    void transfer_to_java() noexcept {
        secret_value_ = nullptr;
        ticket_ttlv_ = nullptr;
    }

private:
    JNIEnv* env_;
    jbyteArray secret_value_ = nullptr;
    kmipkit_ttlv_structure_t* ticket_ttlv_ = nullptr;
};

class Pkcs11ResponseCleanup final {
public:
    using CorrelationArrayScope = kmipkit::jni::ResponseSecretArrayScope<
            jbyteArray, 1, WipeAndDeleteLocalByteArray>;

    explicit Pkcs11ResponseCleanup(JNIEnv* env)
            : correlation_scope_(WipeAndDeleteLocalByteArray{env}) {}

    Pkcs11ResponseCleanup(const Pkcs11ResponseCleanup&) = delete;
    Pkcs11ResponseCleanup& operator=(const Pkcs11ResponseCleanup&) = delete;

    ~Pkcs11ResponseCleanup() {
        for (kmipkit_ttlv_structure_t* structure : structures_) {
            kmipkit_ttlv_structure_release(structure);
        }
    }

    void set_structure(size_t index, kmipkit_ttlv_structure_t* structure) noexcept {
        structures_[index] = structure;
    }

    void set_correlation(jbyteArray correlation) noexcept {
        static_cast<void>(correlation_scope_.add(correlation));
    }

    kmipkit_ttlv_structure_t* structure(size_t index) const noexcept {
        return structures_[index];
    }

    void transfer_structures_to_java() noexcept { structures_.fill(nullptr); }

    void transfer_correlation_to_java() noexcept { correlation_scope_.transfer_to_java(); }

private:
    std::array<kmipkit_ttlv_structure_t*, 5> structures_{};
    CorrelationArrayScope correlation_scope_;
};

jbyteArray span_to_java_bytes(JNIEnv* env, uint8_t present, kmipkit_byte_span_v1_t span) {
    if (!kmipkit::jni::is_valid_response_byte_span(present, span)) {
        throw_invalid_input(env);
        return nullptr;
    }
    if (present == 0) return nullptr;
    if (span.length > static_cast<uint64_t>(std::numeric_limits<jsize>::max())) {
        throw_java(env, "org/kmipkit/ResourceLimitException",
                "native response exceeds Java array limits");
        return nullptr;
    }
    const jsize length = static_cast<jsize>(span.length);
    jbyteArray result = env->NewByteArray(length);
    if (result != nullptr && length != 0) {
        env->SetByteArrayRegion(result, 0, length, reinterpret_cast<const jbyte*>(span.data));
    }
    return kmipkit::jni::discard_response_array_after_copy_failure(
            result,
            [&] { return env->ExceptionCheck(); },
            [&](jbyteArray failed_copy) noexcept {
                wipe_java_byte_array(env, failed_copy);
                env->DeleteLocalRef(failed_copy);
            });
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

jobjectArray make_object_array(JNIEnv* env, std::initializer_list<jobject> values) {
    if (values.size() > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        throw_java(env, "org/kmipkit/ResourceLimitException", "native result exceeds Java array limits");
        return nullptr;
    }
    jclass object_class = env->FindClass("java/lang/Object");
    if (object_class == nullptr) return nullptr;
    jobjectArray result = env->NewObjectArray(static_cast<jsize>(values.size()), object_class, nullptr);
    env->DeleteLocalRef(object_class);
    if (result == nullptr) return nullptr;
    jsize index = 0;
    for (jobject value : values) {
        env->SetObjectArrayElement(result, index++, value);
        if (env->ExceptionCheck()) return nullptr;
    }
    return result;
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

bool response_identifier_to_java(JNIEnv* env, uint8_t response_present,
        const kmipkit_unique_identifier_v1_t& value, jlong* fields, jbyteArray* text) {
    if (fields == nullptr || text == nullptr || response_present > 1) {
        throw_invalid_input(env);
        return false;
    }
    *text = nullptr;
    fields[0] = 0;
    fields[1] = 0;
    fields[2] = 0;
    fields[3] = 0;
    kmipkit::jni::ResponseIdentifierFields normalized{};
    if (!kmipkit::jni::normalize_response_identifier(response_present, value, &normalized)) {
        throw_invalid_input(env);
        return false;
    }
    if (normalized.present == 0) {
        return true;
    }
    if (normalized.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
        *text = span_to_java_bytes(env, 1, {normalized.text_data, normalized.text_length});
        if (*text == nullptr) return false;
    }
    fields[0] = static_cast<jlong>(normalized.present);
    fields[1] = static_cast<jlong>(normalized.kind);
    fields[2] = static_cast<jlong>(normalized.enumeration_value);
    fields[3] = static_cast<jlong>(normalized.integer_value);
    return !env->ExceptionCheck();
}

struct JniClientBatchResponseContext {
    JNIEnv* env;
    jlong raw_item;
};

int32_t get_usage_allocation_response(
        void* raw_context, uint8_t* present,
        kmipkit_unique_identifier_v1_t* identifier,
        int64_t*, size_t, size_t* field_count) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    *field_count = 0;
    return kmipkit_client_batch_item_response_get_usage_allocation(
            from_java_handle<kmipkit_client_batch_item_response_t>(context.raw_item),
            present, identifier);
}

int32_t get_usage_allocation_response_ttlv(void* raw_context, uint8_t* present, void** ttlv) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    kmipkit_ttlv_structure_t* response_ttlv = nullptr;
    const int32_t status = kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            from_java_handle<kmipkit_client_batch_item_response_t>(context.raw_item),
            present, &response_ttlv);
    *ttlv = response_ttlv;
    return status;
}

bool validate_usage_allocation_response(
        void*, uint8_t, const kmipkit_unique_identifier_v1_t*, const int64_t*, size_t) {
    return true;
}

int32_t get_obtain_lease_response(
        void* raw_context, uint8_t* present,
        kmipkit_unique_identifier_v1_t* identifier,
        int64_t* fields, size_t, size_t* field_count) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    uint32_t lease_time = 0;
    int64_t last_change_date = 0;
    const int32_t status = kmipkit_client_batch_item_response_obtain_lease(
            from_java_handle<kmipkit_client_batch_item_response_t>(context.raw_item),
            present, identifier, &lease_time, &last_change_date);
    fields[0] = static_cast<int64_t>(lease_time);
    fields[1] = last_change_date;
    *field_count = 2;
    return status;
}

int32_t get_obtain_lease_response_ttlv(void* raw_context, uint8_t* present, void** ttlv) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    kmipkit_ttlv_structure_t* response_ttlv = nullptr;
    const int32_t status = kmipkit_client_batch_item_response_obtain_lease_ttlv(
            from_java_handle<kmipkit_client_batch_item_response_t>(context.raw_item),
            present, &response_ttlv);
    *ttlv = response_ttlv;
    return status;
}

bool validate_obtain_lease_response(
        void*, uint8_t present, const kmipkit_unique_identifier_v1_t* identifier,
        const int64_t* fields, size_t field_count) {
    if (identifier == nullptr || fields == nullptr || field_count != 2) return false;
    if (present == 1) return identifier->kind != 0;
    return identifier->kind == 0 && fields[0] == 0 && fields[1] == 0;
}

void response_bridge_throw_status(void* raw_context, int32_t status) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    throw_status(context.env, status);
}

void response_bridge_release_ttlv(void*, void* raw_ttlv) {
    kmipkit_ttlv_structure_release(static_cast<kmipkit_ttlv_structure_t*>(raw_ttlv));
}

void* response_bridge_make_fields(
        void* raw_context, uint8_t present, const int64_t* values, size_t value_count) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    std::vector<jlong> fields;
    fields.reserve(value_count + 1);
    fields.push_back(static_cast<jlong>(present));
    for (size_t index = 0; index < value_count; ++index) {
        fields.push_back(static_cast<jlong>(values[index]));
    }
    return make_long_array(context.env, fields);
}

void* response_bridge_make_identifier(
        void* raw_context, const kmipkit_unique_identifier_v1_t* identifier) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    return identifier_to_java(context.env, *identifier, 1);
}

void* response_bridge_make_ttlv_handle(void* raw_context, void* raw_ttlv) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    auto* response_ttlv = static_cast<kmipkit_ttlv_structure_t*>(raw_ttlv);
    return make_long_array(context.env, {to_java_handle(response_ttlv)});
}

void* response_bridge_make_result(
        void* raw_context, void* fields, void* identifier, void* ttlv_handle) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    return make_object_array(context.env, {
            static_cast<jobject>(fields), static_cast<jobject>(identifier),
            static_cast<jobject>(ttlv_handle)});
}

void response_bridge_delete_local(void* raw_context, void* local_reference) {
    auto& context = *static_cast<JniClientBatchResponseContext*>(raw_context);
    if (local_reference != nullptr) {
        context.env->DeleteLocalRef(static_cast<jobject>(local_reference));
    }
}

kmipkit::jni::ResponseBridgeOperations response_bridge_operations(
        JniClientBatchResponseContext& context,
        int32_t (*get_response)(
                void*, uint8_t*, kmipkit_unique_identifier_v1_t*, int64_t*, size_t, size_t*),
        int32_t (*get_response_ttlv)(void*, uint8_t*, void**),
        bool (*validate_response)(
                void*, uint8_t, const kmipkit_unique_identifier_v1_t*, const int64_t*, size_t)) {
    return {
            &context,
            get_response,
            get_response_ttlv,
            validate_response,
            &response_bridge_throw_status,
            &response_bridge_release_ttlv,
            &response_bridge_make_fields,
            &response_bridge_make_identifier,
            &response_bridge_make_ttlv_handle,
            &response_bridge_make_result,
            &response_bridge_delete_local};
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemPing(
        JNIEnv* env, jclass) {
    return guarded<jlong>(env, 0, [&] {
        kmipkit_client_batch_item_t* output = nullptr;
        const int32_t status = kmipkit_client_batch_item_ping(&output);
        if (status != 0) {
            kmipkit_client_batch_item_release(output);
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        if (output == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(output);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemLogout(
        JNIEnv* env, jclass, jlong raw_ticket) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_logout(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_ticket), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemPkcs11(
        JNIEnv* env, jclass, jlong raw_function, jlong raw_interface,
        jbyteArray raw_correlation_value, jlong raw_input_parameters) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> correlation_value;
        kmipkit::secure::VectorWiper correlation_value_wiper(correlation_value);
        bool correlation_present = false;
        if (!read_optional_java_bytes(
                    env, raw_correlation_value, correlation_value, &correlation_present)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation_value);
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        kmipkit_client_batch_item_t* output = nullptr;
        const int32_t status = kmipkit_client_batch_item_pkcs11(
                from_java_handle<kmipkit_ttlv_structure_t>(raw_function),
                from_java_handle<kmipkit_ttlv_structure_t>(raw_interface),
                correlation_pointer,
                from_java_handle<kmipkit_ttlv_structure_t>(raw_input_parameters), &output);
        if (status != 0) {
            kmipkit_client_batch_item_release(output);
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        if (output == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(output);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemQuery(
        JNIEnv* env, jclass, jintArray raw_functions, jobjectArray raw_object_groups) {
    return guarded<jlong>(env, 0, [&] {
        if (!preflight_query_inputs(env, raw_functions, raw_object_groups,
                16 * 1024 * 1024, 64, 100000)) return static_cast<jlong>(0);
        const jsize function_count = env->GetArrayLength(raw_functions);
        std::vector<jint> raw_function_values(static_cast<size_t>(function_count));
        env->GetIntArrayRegion(raw_functions, 0, function_count, raw_function_values.data());
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        std::vector<uint32_t> functions;
        functions.reserve(raw_function_values.size());
        for (jint value : raw_function_values) {
            functions.push_back(static_cast<uint32_t>(value));
        }

        const uint8_t has_object_groups = raw_object_groups == nullptr ? 0 : 1;
        std::vector<std::vector<uint8_t>> group_bytes;
        std::vector<kmipkit_byte_span_v1_t> group_spans;
        if (raw_object_groups != nullptr) {
            const jsize group_count = env->GetArrayLength(raw_object_groups);
            group_bytes.reserve(static_cast<size_t>(group_count));
            group_spans.reserve(static_cast<size_t>(group_count));
            for (jsize index = 0; index < group_count; ++index) {
                jobject raw_group = env->GetObjectArrayElement(raw_object_groups, index);
                if (raw_group == nullptr) {
                    if (!env->ExceptionCheck()) throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
                std::vector<uint8_t> bytes;
                const bool copied = read_bytes(
                        env, reinterpret_cast<jbyteArray>(raw_group),
                        static_cast<uint64_t>(std::numeric_limits<jsize>::max()), bytes);
                env->DeleteLocalRef(raw_group);
                if (!copied) return static_cast<jlong>(0);
                group_bytes.push_back(std::move(bytes));
            }
            for (const auto& bytes : group_bytes) group_spans.push_back(byte_span(bytes));
        }

        kmipkit_client_batch_item_t* output = nullptr;
        const int32_t status = kmipkit_client_batch_item_query(
                functions.data(), static_cast<uint64_t>(functions.size()), has_object_groups,
                group_spans.empty() ? nullptr : group_spans.data(),
                static_cast<uint64_t>(group_spans.size()), &output);
        if (status != 0) {
            kmipkit_client_batch_item_release(output);
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        if (output == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(output);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemQueryWithLimits(
        JNIEnv* env, jclass, jintArray raw_functions, jobjectArray raw_object_groups,
        jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        CodecLimits limits;
        if (!limits.initialize(env, raw_limits)) return static_cast<jlong>(0);
        if (!preflight_query_inputs(env, raw_functions, raw_object_groups,
                limits.max_message_bytes(), limits.max_structure_depth(), limits.max_elements())) {
            return static_cast<jlong>(0);
        }
        const jsize function_count = env->GetArrayLength(raw_functions);
        const jsize group_count = raw_object_groups == nullptr
                ? 0 : env->GetArrayLength(raw_object_groups);

        std::vector<jint> raw_function_values(static_cast<size_t>(function_count));
        env->GetIntArrayRegion(raw_functions, 0, function_count, raw_function_values.data());
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        std::vector<uint32_t> functions;
        functions.reserve(raw_function_values.size());
        for (jint value : raw_function_values) {
            functions.push_back(static_cast<uint32_t>(value));
        }

        std::vector<std::vector<uint8_t>> group_bytes;
        std::vector<kmipkit_byte_span_v1_t> group_spans;
        if (raw_object_groups != nullptr) {
            group_bytes.reserve(static_cast<size_t>(group_count));
            group_spans.reserve(static_cast<size_t>(group_count));
            for (jsize index = 0; index < group_count; ++index) {
                jobject raw_group = env->GetObjectArrayElement(raw_object_groups, index);
                if (raw_group == nullptr) {
                    if (!env->ExceptionCheck()) throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
                std::vector<uint8_t> bytes;
                const bool copied = read_bytes(
                        env, reinterpret_cast<jbyteArray>(raw_group),
                        static_cast<uint64_t>(std::numeric_limits<jsize>::max()), bytes);
                env->DeleteLocalRef(raw_group);
                if (!copied) return static_cast<jlong>(0);
                group_bytes.push_back(std::move(bytes));
            }
            for (const auto& bytes : group_bytes) group_spans.push_back(byte_span(bytes));
        }

        kmipkit_client_batch_item_t* output = nullptr;
        const int32_t status = kmipkit_client_batch_item_query_with_limits(
                limits.get(), functions.data(), static_cast<uint64_t>(functions.size()),
                raw_object_groups == nullptr ? 0 : 1,
                group_spans.empty() ? nullptr : group_spans.data(),
                static_cast<uint64_t>(group_spans.size()), &output);
        if (status != 0) {
            kmipkit_client_batch_item_release(output);
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        if (output == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return to_java_handle(output);
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCertify(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_certificate_request_type,
        jlong raw_certificate_request_type, jbyteArray raw_certificate_request_value,
        jlong raw_attributes, jlong raw_masks) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> request_value;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper request_value_wiper(request_value);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool request_value_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_certificate_request_value,
                    request_value, &request_value_present)) {
            return static_cast<jlong>(0);
        }
        if ((raw_has_certificate_request_type != JNI_TRUE
                    && raw_has_certificate_request_type != JNI_FALSE)
                || raw_certificate_request_type < 0
                || static_cast<uint64_t>(raw_certificate_request_type) > UINT32_MAX
                || (raw_has_certificate_request_type == JNI_FALSE
                    && raw_certificate_request_type != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t request_span = byte_span(request_value);
        const kmipkit_byte_span_v1_t* request_pointer =
                request_value_present ? &request_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_certify(identifier_pointer,
                    raw_has_certificate_request_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_certificate_request_type), request_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRecertify(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jint raw_request_identifier_kind,
        jbyteArray raw_request_identifier_text, jlong raw_request_identifier_number,
        jboolean raw_has_certificate_request_type, jlong raw_certificate_request_type,
        jbyteArray raw_certificate_request_value, jboolean raw_has_offset, jlong raw_offset,
        jlong raw_attributes, jlong raw_masks) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> request_identifier_text;
        std::vector<uint8_t> request_value;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper request_identifier_text_wiper(request_identifier_text);
        kmipkit::secure::VectorWiper request_value_wiper(request_value);
        kmipkit_unique_identifier_v1_t identifier{};
        kmipkit_unique_identifier_v1_t request_identifier{};
        bool identifier_present = false;
        bool request_identifier_present = false;
        bool request_value_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_identifier(env, raw_request_identifier_kind,
                    raw_request_identifier_text, raw_request_identifier_number,
                    &request_identifier, request_identifier_text, &request_identifier_present)
                || !read_optional_java_bytes(env, raw_certificate_request_value,
                    request_value, &request_value_present)) {
            return static_cast<jlong>(0);
        }
        if ((raw_has_certificate_request_type != JNI_TRUE
                    && raw_has_certificate_request_type != JNI_FALSE)
                || raw_certificate_request_type < 0
                || static_cast<uint64_t>(raw_certificate_request_type) > UINT32_MAX
                || (raw_has_certificate_request_type == JNI_FALSE
                    && raw_certificate_request_type != 0)
                || (raw_has_offset != JNI_TRUE && raw_has_offset != JNI_FALSE)
                || raw_offset < 0
                || static_cast<uint64_t>(raw_offset) > UINT32_MAX
                || (raw_has_offset == JNI_FALSE && raw_offset != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t request_span = byte_span(request_value);
        const kmipkit_byte_span_v1_t* request_pointer =
                request_value_present ? &request_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        const kmipkit_unique_identifier_v1_t* request_identifier_pointer =
                request_identifier_present ? &request_identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_recertify(identifier_pointer,
                    request_identifier_pointer,
                    raw_has_certificate_request_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_certificate_request_type), request_pointer,
                    raw_has_offset == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_offset),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRekey(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_offset, jlong raw_offset,
        jlong raw_attributes, jlong raw_masks) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        if ((raw_has_offset != JNI_TRUE && raw_has_offset != JNI_FALSE)
                || raw_offset < 0
                || static_cast<uint64_t>(raw_offset) > UINT32_MAX
                || (raw_has_offset == JNI_FALSE && raw_offset != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_rekey(identifier_pointer,
                    raw_has_offset == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_offset),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRekeyKeyPair(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_offset, jlong raw_offset,
        jlong common_attributes, jlong private_key_attributes, jlong public_key_attributes,
        jlong common_masks, jlong private_masks, jlong public_masks) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        if ((raw_has_offset != JNI_TRUE && raw_has_offset != JNI_FALSE) || raw_offset < 0
                || static_cast<uint64_t>(raw_offset) > UINT32_MAX
                || (raw_has_offset == JNI_FALSE && raw_offset != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const auto* identifier_pointer = identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_rekey_key_pair(identifier_pointer,
                    raw_has_offset == JNI_TRUE ? 1 : 0, static_cast<uint32_t>(raw_offset),
                    from_java_handle<kmipkit_ttlv_structure_t>(common_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(private_key_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(public_key_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(common_masks),
                    from_java_handle<kmipkit_ttlv_structure_t>(private_masks),
                    from_java_handle<kmipkit_ttlv_structure_t>(public_masks), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemReprovision(
        JNIEnv* env, jclass, jlong raw_certificate_request, jlong raw_certificate) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_reprovision(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_certificate_request),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_certificate), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRngRetrieve(
        JNIEnv* env, jclass, jint data_length) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_rng_retrieve(
                    static_cast<int32_t>(data_length), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRngSeed(
        JNIEnv* env, jclass, jbyteArray raw_data) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> data;
        kmipkit::secure::VectorWiper data_wiper(data);
        if (!read_java_bytes(env, raw_data, data, false)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t data_span = byte_span(data);
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_rng_seed(&data_span, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemCheck(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_usage_limits_count,
        jlong raw_usage_limits_count, jboolean raw_has_cryptographic_usage_mask,
        jint raw_cryptographic_usage_mask, jboolean raw_has_lease_time, jlong raw_lease_time) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        if ((raw_has_usage_limits_count != JNI_TRUE
                    && raw_has_usage_limits_count != JNI_FALSE)
                || (raw_has_cryptographic_usage_mask != JNI_TRUE
                    && raw_has_cryptographic_usage_mask != JNI_FALSE)
                || (raw_has_lease_time != JNI_TRUE && raw_has_lease_time != JNI_FALSE)
                || raw_lease_time < 0
                || static_cast<uint64_t>(raw_lease_time) > UINT32_MAX
                || (raw_has_lease_time == JNI_FALSE && raw_lease_time != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_check(identifier_pointer,
                    raw_has_usage_limits_count == JNI_TRUE ? 1 : 0,
                    static_cast<int64_t>(raw_usage_limits_count),
                    raw_has_cryptographic_usage_mask == JNI_TRUE ? 1 : 0,
                    static_cast<int32_t>(raw_cryptographic_usage_mask),
                    raw_has_lease_time == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_lease_time), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDelegatedLogin(
        JNIEnv* env, jclass, jlong raw_rights, jint raw_lease_time_kind,
        jlong raw_lease_time_value, jboolean raw_has_request_count,
        jint raw_request_count, jlong raw_usage_limits) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_has_request_count != JNI_TRUE && raw_has_request_count != JNI_FALSE) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_delegated_login(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_rights),
                    static_cast<uint32_t>(raw_lease_time_kind),
                    static_cast<int64_t>(raw_lease_time_value),
                    raw_has_request_count == JNI_TRUE ? 1 : 0,
                    static_cast<int32_t>(raw_request_count),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_usage_limits), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemLogin(
        JNIEnv* env, jclass, jint raw_lease_time_kind, jlong raw_lease_time_value,
        jboolean raw_has_request_count, jint raw_request_count, jlong raw_usage_limits) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_has_request_count != JNI_TRUE && raw_has_request_count != JNI_FALSE) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_login(
                    static_cast<uint32_t>(raw_lease_time_kind),
                    static_cast<int64_t>(raw_lease_time_value),
                    raw_has_request_count == JNI_TRUE ? 1 : 0,
                    static_cast<int32_t>(raw_request_count),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_usage_limits), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemGetUsageAllocation(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_usage_limits_count) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_get_usage_allocation(identifier_pointer,
                    static_cast<int64_t>(raw_usage_limits_count), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemObtainLease(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_obtain_lease(identifier_pointer, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemImport(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_object_type,
        jboolean raw_has_replace_existing, jboolean raw_replace_existing,
        jboolean raw_has_key_wrap_type, jlong raw_key_wrap_type,
        jlong raw_attributes, jlong raw_any_object) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        const auto valid_optional_u32 = [](jboolean present, jlong value) {
            return (present == JNI_TRUE || present == JNI_FALSE)
                    && value >= 0 && static_cast<uint64_t>(value) <= UINT32_MAX
                    && (present == JNI_TRUE || value == 0);
        };
        if (!identifier_present || raw_object_type < 0
                || static_cast<uint64_t>(raw_object_type) > UINT32_MAX
                || (raw_has_replace_existing != JNI_TRUE && raw_has_replace_existing != JNI_FALSE)
                || (raw_replace_existing != JNI_TRUE && raw_replace_existing != JNI_FALSE)
                || (raw_has_replace_existing == JNI_FALSE && raw_replace_existing == JNI_TRUE)
                || !valid_optional_u32(raw_has_key_wrap_type, raw_key_wrap_type)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_import(&identifier,
                    static_cast<uint32_t>(raw_object_type),
                    raw_has_replace_existing == JNI_TRUE ? 1 : 0,
                    raw_replace_existing == JNI_TRUE ? 1 : 0,
                    raw_has_key_wrap_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_wrap_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_any_object), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_splitKeyReferenceCreate(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_split_key_threshold) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        if (!identifier_present || raw_split_key_threshold < 0
                || static_cast<uint64_t>(raw_split_key_threshold) > UINT32_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_split_key_reference_t>(env, [&](auto output) {
            return kmipkit_split_key_reference_create(&identifier,
                    static_cast<uint32_t>(raw_split_key_threshold), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemJoinSplitKey(
        JNIEnv* env, jclass, jlong raw_object_type, jlongArray raw_references,
        jboolean raw_has_secret_data_type, jlong raw_secret_data_type,
        jlong raw_attributes, jlong raw_protection_storage_masks) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_references == nullptr || raw_object_type < 0
                || static_cast<uint64_t>(raw_object_type) > UINT32_MAX
                || (raw_has_secret_data_type != JNI_TRUE
                    && raw_has_secret_data_type != JNI_FALSE)
                || raw_secret_data_type < 0
                || static_cast<uint64_t>(raw_secret_data_type) > UINT32_MAX
                || (raw_has_secret_data_type == JNI_FALSE && raw_secret_data_type != 0)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const jsize count = env->GetArrayLength(raw_references);
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        if (count <= 0 || static_cast<uint64_t>(count) > 100'000) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<jlong> raw_handles(static_cast<size_t>(count));
        env->GetLongArrayRegion(raw_references, 0, count, raw_handles.data());
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        std::vector<kmipkit_split_key_reference_t*> references;
        references.reserve(raw_handles.size());
        for (jlong raw_handle : raw_handles) {
            if (raw_handle == 0) {
                throw_invalid_input(env);
                return static_cast<jlong>(0);
            }
            references.push_back(from_java_handle<kmipkit_split_key_reference_t>(raw_handle));
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_join_split_key(
                    static_cast<uint32_t>(raw_object_type), references.data(), references.size(),
                    raw_has_secret_data_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_secret_data_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_protection_storage_masks),
                    output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemGet(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_key_format_type, jlong raw_key_format_type,
        jboolean raw_has_key_wrap_type, jlong raw_key_wrap_type,
        jboolean raw_has_key_compression_type, jlong raw_key_compression_type,
        jlong raw_key_wrapping_specification) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        const auto valid_optional_u32 = [](jboolean present, jlong value) {
            return (present == JNI_TRUE || present == JNI_FALSE)
                    && value >= 0 && static_cast<uint64_t>(value) <= UINT32_MAX
                    && (present == JNI_TRUE || value == 0);
        };
        if (!valid_optional_u32(raw_has_key_format_type, raw_key_format_type)
                || !valid_optional_u32(raw_has_key_wrap_type, raw_key_wrap_type)
                || !valid_optional_u32(raw_has_key_compression_type, raw_key_compression_type)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_get(identifier_pointer,
                    raw_has_key_format_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_format_type),
                    raw_has_key_wrap_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_wrap_type),
                    raw_has_key_compression_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_compression_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_key_wrapping_specification), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemLocate(
        JNIEnv* env, jclass, jboolean raw_has_maximum_items, jint raw_maximum_items,
        jboolean raw_has_offset_items, jint raw_offset_items,
        jboolean raw_has_storage_status_mask, jlong raw_storage_status_mask,
        jboolean raw_has_object_group_member, jlong raw_object_group_member,
        jlong raw_attributes) {
    return guarded<jlong>(env, 0, [&] {
        const auto valid_optional_u32 = [](jboolean present, jlong value) {
            return (present == JNI_TRUE || present == JNI_FALSE)
                    && value >= 0 && static_cast<uint64_t>(value) <= UINT32_MAX
                    && (present == JNI_TRUE || value == 0);
        };
        if ((raw_has_maximum_items != JNI_TRUE && raw_has_maximum_items != JNI_FALSE)
                || (raw_has_offset_items != JNI_TRUE && raw_has_offset_items != JNI_FALSE)
                || (!raw_has_maximum_items && raw_maximum_items != 0)
                || (!raw_has_offset_items && raw_offset_items != 0)
                || !valid_optional_u32(raw_has_storage_status_mask, raw_storage_status_mask)
                || !valid_optional_u32(raw_has_object_group_member, raw_object_group_member)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_locate(
                    raw_has_maximum_items == JNI_TRUE ? 1 : 0,
                    static_cast<int32_t>(raw_maximum_items),
                    raw_has_offset_items == JNI_TRUE ? 1 : 0,
                    static_cast<int32_t>(raw_offset_items),
                    raw_has_storage_status_mask == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_storage_status_mask),
                    raw_has_object_group_member == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_object_group_member),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemGetConstraints(
        JNIEnv* env, jclass) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_get_constraints(output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemSetConstraints(
        JNIEnv* env, jclass, jlong raw_constraints) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_set_constraints(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_constraints), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemInterop(
        JNIEnv* env, jclass, jlong raw_interop_function, jbyteArray raw_interop_identifier) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_interop_function < 0
                || static_cast<uint64_t>(raw_interop_function) > UINT32_MAX) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> identifier;
        kmipkit::secure::VectorWiper identifier_wiper(identifier);
        if (!read_java_bytes(env, raw_interop_identifier, identifier, false)) {
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_interop(
                    static_cast<uint32_t>(raw_interop_function),
                    identifier.empty() ? nullptr : identifier.data(),
                    static_cast<uint64_t>(identifier.size()), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemLog(
        JNIEnv* env, jclass, jbyteArray raw_log_message) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> message;
        kmipkit::secure::VectorWiper message_wiper(message);
        if (!read_java_bytes(env, raw_log_message, message, false)) {
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_log(
                    message.empty() ? nullptr : message.data(),
                    static_cast<uint64_t>(message.size()), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemExport(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jboolean raw_has_key_format_type, jlong raw_key_format_type,
        jboolean raw_has_key_wrap_type, jlong raw_key_wrap_type,
        jboolean raw_has_key_compression_type, jlong raw_key_compression_type,
        jlong raw_key_wrapping_specification) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)) {
            return static_cast<jlong>(0);
        }
        const auto valid_optional_u32 = [](jboolean present, jlong value) {
            return (present == JNI_TRUE || present == JNI_FALSE)
                    && value >= 0 && static_cast<uint64_t>(value) <= UINT32_MAX
                    && (present == JNI_TRUE || value == 0);
        };
        if (!valid_optional_u32(raw_has_key_format_type, raw_key_format_type)
                || !valid_optional_u32(raw_has_key_wrap_type, raw_key_wrap_type)
                || !valid_optional_u32(raw_has_key_compression_type, raw_key_compression_type)) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_export(identifier_pointer,
                    raw_has_key_format_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_format_type),
                    raw_has_key_wrap_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_wrap_type),
                    raw_has_key_compression_type == JNI_TRUE ? 1 : 0,
                    static_cast<uint32_t>(raw_key_compression_type),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_key_wrapping_specification), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDeriveKey(
        JNIEnv* env, jclass, jlong raw_object_type, jintArray raw_identifier_kinds,
        jobjectArray raw_identifier_texts, jlongArray raw_identifier_numbers,
        jlong raw_derivation_method, jlong raw_derivation_parameters, jlong raw_attributes) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_object_type < 0 || static_cast<uint64_t>(raw_object_type) > UINT32_MAX
                || raw_derivation_method < 0
                || static_cast<uint64_t>(raw_derivation_method) > UINT32_MAX
                || raw_identifier_kinds == nullptr || raw_identifier_texts == nullptr
                || raw_identifier_numbers == nullptr) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const jsize count = env->GetArrayLength(raw_identifier_kinds);
        if (count <= 0) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        if (count > static_cast<jsize>(kMaximumNativeInputArrayElements)) {
            throw_java(env, "org/kmipkit/ResourceLimitException",
                    "Derive Key source identifier count exceeds the configured limit");
            return static_cast<jlong>(0);
        }
        if (env->GetArrayLength(raw_identifier_texts) != count
                || env->GetArrayLength(raw_identifier_numbers) != count) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }

        std::vector<jint> kinds(static_cast<size_t>(count));
        std::vector<jlong> numbers(static_cast<size_t>(count));
        env->GetIntArrayRegion(raw_identifier_kinds, 0, count, kinds.data());
        if (env->ExceptionCheck()) return static_cast<jlong>(0);
        env->GetLongArrayRegion(raw_identifier_numbers, 0, count, numbers.data());
        if (env->ExceptionCheck()) return static_cast<jlong>(0);

        uint64_t total_text_bytes = 0;
        for (jsize index = 0; index < count; ++index) {
            auto text = static_cast<jbyteArray>(env->GetObjectArrayElement(raw_identifier_texts, index));
            if (env->ExceptionCheck()) return static_cast<jlong>(0);
            const jint kind = kinds[static_cast<size_t>(index)];
            const jlong number = numbers[static_cast<size_t>(index)];
            if (kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
                if (text == nullptr || number != 0) {
                    if (text != nullptr) env->DeleteLocalRef(text);
                    throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
                const jsize length = env->GetArrayLength(text);
                if (length < 0 || static_cast<uint64_t>(length) > kMaximumTransportInputBytes
                        || total_text_bytes > kMaximumTransportInputBytes - static_cast<uint64_t>(length)) {
                    env->DeleteLocalRef(text);
                    throw_java(env, "org/kmipkit/ResourceLimitException",
                            "Derive Key source identifier bytes exceed the configured limit");
                    return static_cast<jlong>(0);
                }
                total_text_bytes += static_cast<uint64_t>(length);
            } else if (kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION) {
                if (text != nullptr || number < 0 || static_cast<uint64_t>(number) > UINT32_MAX) {
                    if (text != nullptr) env->DeleteLocalRef(text);
                    throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
            } else if (kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER) {
                if (text != nullptr || number < INT32_MIN || number > INT32_MAX) {
                    if (text != nullptr) env->DeleteLocalRef(text);
                    throw_invalid_input(env);
                    return static_cast<jlong>(0);
                }
            } else {
                if (text != nullptr) env->DeleteLocalRef(text);
                throw_invalid_input(env);
                return static_cast<jlong>(0);
            }
            if (text != nullptr) env->DeleteLocalRef(text);
        }

        std::vector<std::vector<uint8_t>> identifier_texts;
        ByteVectorListWiper text_wiper(identifier_texts);
        std::vector<kmipkit_unique_identifier_v1_t> identifiers;
        identifier_texts.reserve(static_cast<size_t>(count));
        identifiers.reserve(static_cast<size_t>(count));
        for (jsize index = 0; index < count; ++index) {
            const size_t position = static_cast<size_t>(index);
            kmipkit_unique_identifier_v1_t identifier{};
            identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
            identifier.abi_version = 1;
            identifier.kind = static_cast<kmipkit_unique_identifier_kind_t>(kinds[position]);
            if (kinds[position] == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING) {
                auto text = static_cast<jbyteArray>(env->GetObjectArrayElement(raw_identifier_texts, index));
                if (env->ExceptionCheck()) return static_cast<jlong>(0);
                identifier_texts.emplace_back();
                const bool copied = read_bytes(env, text, kMaximumTransportInputBytes,
                        identifier_texts.back());
                env->DeleteLocalRef(text);
                if (!copied) return static_cast<jlong>(0);
                identifier.text_data = identifier_texts.back().empty()
                        ? nullptr : identifier_texts.back().data();
                identifier.text_length = static_cast<uint64_t>(identifier_texts.back().size());
            } else if (kinds[position] == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION) {
                identifier.enumeration_value = static_cast<uint32_t>(numbers[position]);
            } else {
                identifier.integer_value = static_cast<int32_t>(numbers[position]);
            }
            identifiers.push_back(identifier);
        }

        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_derive_key(
                    static_cast<uint32_t>(raw_object_type), identifiers.data(),
                    static_cast<uint64_t>(identifiers.size()),
                    static_cast<uint32_t>(raw_derivation_method),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_derivation_parameters),
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_attributes), output);
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
                throw_invalid_input(env); // COVERAGE-EXCLUDE: The guard admits only zero or handled nonzero kinds.
                return static_cast<jlong>(0); // COVERAGE-EXCLUDE: Kind 0 skips this block; kinds 1-3 match the handled cases.
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

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_revocationReasonCreate(
        JNIEnv* env, jclass, jlong raw_code, jbyteArray raw_message) {
    return guarded<jlong>(env, 0, [&] {
        if (raw_code < 0
                || static_cast<uint64_t>(raw_code) > std::numeric_limits<uint32_t>::max()) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        std::vector<uint8_t> message;
        bool message_present = false;
        if (!read_optional_java_bytes(env, raw_message, message, &message_present)) {
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_revocation_reason_t>(env, [&](auto output) {
            return kmipkit_revocation_reason_create(
                    static_cast<uint32_t>(raw_code), static_cast<uint8_t>(message_present),
                    message.empty() ? nullptr : message.data(),
                    static_cast<uint64_t>(message.size()), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_revokeRequestCreate(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_reason, jboolean has_compromise_date,
        jlong compromise_date) {
    return guarded<jlong>(env, 0, [&] {
        auto* reason = from_java_handle<kmipkit_revocation_reason_t>(raw_reason);
        kmipkit::jni::NativeHandleTransferGuard<
                kmipkit_revocation_reason_t*, decltype(&kmipkit_revocation_reason_release)>
                reason_transfer(reason, kmipkit_revocation_reason_release);
        kmipkit_unique_identifier_v1_t identifier{};
        std::vector<uint8_t> identifier_text;
        bool identifier_present = false;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || (has_compromise_date != JNI_FALSE && has_compromise_date != JNI_TRUE)) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_revoke_request_t>(env, [&](auto output) {
            return kmipkit_revoke_request_create(
                    identifier_pointer, reason_transfer.transfer_to_consumer(),
                    static_cast<uint8_t>(has_compromise_date == JNI_TRUE),
                    static_cast<int64_t>(compromise_date), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemRevoke(
        JNIEnv* env, jclass, jlong raw_request) {
    return guarded<jlong>(env, 0, [&] {
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_revoke(
                    from_java_handle<kmipkit_revoke_request_t>(raw_request), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemEncrypt(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_iv, jbyteArray raw_correlation, jint raw_init, jint raw_final,
        jbyteArray raw_additional_data) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> iv;
        std::vector<uint8_t> correlation;
        std::vector<uint8_t> additional_data;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper iv_wiper(iv);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit::secure::VectorWiper additional_data_wiper(additional_data);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool iv_present = false;
        bool correlation_present = false;
        bool additional_data_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_iv, iv, &iv_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_java_bytes(env, raw_additional_data, additional_data,
                        &additional_data_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t iv_span = byte_span(iv);
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t additional_data_span = byte_span(additional_data);
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        const kmipkit_byte_span_v1_t* iv_pointer = iv_present ? &iv_span : nullptr;
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_byte_span_v1_t* additional_data_pointer =
                additional_data_present ? &additional_data_span : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_encrypt(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data), iv_pointer,
                    correlation_pointer, init, final_value, additional_data_pointer, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemDecrypt(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_iv, jbyteArray raw_tag, jbyteArray raw_correlation,
        jint raw_init, jint raw_final, jbyteArray raw_additional_data) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> iv;
        std::vector<uint8_t> tag;
        std::vector<uint8_t> correlation;
        std::vector<uint8_t> additional_data;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper iv_wiper(iv);
        kmipkit::secure::VectorWiper tag_wiper(tag);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit::secure::VectorWiper additional_data_wiper(additional_data);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool iv_present = false;
        bool tag_present = false;
        bool correlation_present = false;
        bool additional_data_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_iv, iv, &iv_present)
                || !read_optional_java_bytes(env, raw_tag, tag, &tag_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_java_bytes(env, raw_additional_data, additional_data,
                        &additional_data_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t iv_span = byte_span(iv);
        const kmipkit_byte_span_v1_t tag_span = byte_span(tag);
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t additional_data_span = byte_span(additional_data);
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        const kmipkit_byte_span_v1_t* iv_pointer = iv_present ? &iv_span : nullptr;
        const kmipkit_byte_span_v1_t* tag_pointer = tag_present ? &tag_span : nullptr;
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_byte_span_v1_t* additional_data_pointer =
                additional_data_present ? &additional_data_span : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_decrypt(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data), iv_pointer, tag_pointer,
                    correlation_pointer, init, final_value, additional_data_pointer, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemHash(
        JNIEnv* env, jclass, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_correlation, jint raw_init, jint raw_final) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> correlation;
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        bool correlation_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_hash(
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data),
                    correlation_pointer, init, final_value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemMac(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_correlation, jint raw_init, jint raw_final) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> correlation;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool correlation_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_mac(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data),
                    correlation_pointer, init, final_value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemMacVerify(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_mac_data, jbyteArray raw_correlation, jint raw_init, jint raw_final) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> mac_data;
        std::vector<uint8_t> correlation;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper mac_data_wiper(mac_data);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool mac_present = false;
        bool correlation_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_mac_data, mac_data, &mac_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t mac_span = byte_span(mac_data);
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t* mac_pointer = mac_present ? &mac_span : nullptr;
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_mac_verify(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data), mac_pointer,
                    correlation_pointer, init, final_value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemSign(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_digested_data, jbyteArray raw_correlation, jint raw_init, jint raw_final) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> digested_data;
        std::vector<uint8_t> correlation;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper digested_data_wiper(digested_data);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool digested_present = false;
        bool correlation_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_digested_data, digested_data, &digested_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t digested_span = byte_span(digested_data);
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t* digested_pointer = digested_present ? &digested_span : nullptr;
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_sign(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data), digested_pointer,
                    correlation_pointer, init, final_value, output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemSignatureVerify(
        JNIEnv* env, jclass, jint raw_identifier_kind, jbyteArray raw_identifier_text,
        jlong raw_identifier_number, jlong raw_parameters, jlong raw_data,
        jbyteArray raw_digested_data, jbyteArray raw_signature_data, jbyteArray raw_correlation,
        jint raw_init, jint raw_final) {
    return guarded<jlong>(env, 0, [&] {
        std::vector<uint8_t> identifier_text;
        std::vector<uint8_t> digested_data;
        std::vector<uint8_t> signature_data;
        std::vector<uint8_t> correlation;
        kmipkit::secure::VectorWiper identifier_text_wiper(identifier_text);
        kmipkit::secure::VectorWiper digested_data_wiper(digested_data);
        kmipkit::secure::VectorWiper signature_data_wiper(signature_data);
        kmipkit::secure::VectorWiper correlation_wiper(correlation);
        kmipkit_unique_identifier_v1_t identifier{};
        bool identifier_present = false;
        bool digested_present = false;
        bool signature_present = false;
        bool correlation_present = false;
        uint8_t init = 0;
        uint8_t final_value = 0;
        if (!read_optional_identifier(env, raw_identifier_kind, raw_identifier_text,
                    raw_identifier_number, &identifier, identifier_text, &identifier_present)
                || !read_optional_java_bytes(env, raw_digested_data, digested_data, &digested_present)
                || !read_optional_java_bytes(env, raw_signature_data, signature_data, &signature_present)
                || !read_optional_java_bytes(env, raw_correlation, correlation, &correlation_present)
                || !read_optional_indicators(env, raw_init, raw_final, &init, &final_value)) {
            return static_cast<jlong>(0);
        }
        const kmipkit_byte_span_v1_t digested_span = byte_span(digested_data);
        const kmipkit_byte_span_v1_t signature_span = byte_span(signature_data);
        const kmipkit_byte_span_v1_t correlation_span = byte_span(correlation);
        const kmipkit_byte_span_v1_t* digested_pointer = digested_present ? &digested_span : nullptr;
        const kmipkit_byte_span_v1_t* signature_pointer = signature_present ? &signature_span : nullptr;
        const kmipkit_byte_span_v1_t* correlation_pointer =
                correlation_present ? &correlation_span : nullptr;
        const kmipkit_unique_identifier_v1_t* identifier_pointer =
                identifier_present ? &identifier : nullptr;
        return return_handle<kmipkit_client_batch_item_t>(env, [&](auto output) {
            return kmipkit_client_batch_item_signature_verify(identifier_pointer,
                    from_java_handle<kmipkit_ttlv_structure_t>(raw_parameters),
                    from_java_handle<kmipkit_ttlv_value_t>(raw_data), digested_pointer,
                    signature_pointer, correlation_pointer, init, final_value, output);
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
            return static_cast<jboolean>(JNI_FALSE); // COVERAGE-EXCLUDE: The FFI converts this bool to 0 or 1.
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
        case 33: kmipkit_split_key_reference_release(from_java_handle<kmipkit_split_key_reference_t>(raw_handle)); break;
        case 34: kmipkit_usage_allocation_context_release(from_java_handle<kmipkit_usage_allocation_context_t>(raw_handle)); break;
        case 35: kmipkit_revocation_reason_release(from_java_handle<kmipkit_revocation_reason_t>(raw_handle)); break;
        case 36: kmipkit_revoke_request_release(from_java_handle<kmipkit_revoke_request_t>(raw_handle)); break;
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
        kmipkit::jni::NativeHandleTransferGuard<
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
        kmipkit_ttlv_structure_t* error_response_ttlv = nullptr;
        const int32_t status = kmipkit_client_execute_with_error_ttlv_v1(
                client, &batch, limits.get(), &response, &error_response_ttlv);
        if (status != 0) throw_status(env, status, error_response_ttlv);
        else if (error_response_ttlv != nullptr) {
            kmipkit_ttlv_structure_release(error_response_ttlv); // COVERAGE-EXCLUDE: the FFI sets this only on failure.
            throw_invalid_input(env); // COVERAGE-EXCLUDE: success carries no error TTLV.
        }
        if (batch != nullptr) kmipkit_client_batch_release(batch);
        if (status != 0) return static_cast<jlong>(0);
        if (response == nullptr) {
            throw_invalid_input(env); // COVERAGE-EXCLUDE: success always returns a response handle.
            return static_cast<jlong>(0); // COVERAGE-EXCLUDE: null success is forbidden by the FFI contract.
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
        kmipkit_ttlv_structure_t* error_response_ttlv = nullptr;
        const int32_t status = kmipkit_client_execute_with_options_and_error_ttlv_v1(
                client, &batch, limits.get(), &options, &response, &error_response_ttlv);
        if (status != 0) throw_status(env, status, error_response_ttlv);
        else if (error_response_ttlv != nullptr) {
            kmipkit_ttlv_structure_release(error_response_ttlv); // COVERAGE-EXCLUDE: the FFI sets this only on failure.
            throw_invalid_input(env); // COVERAGE-EXCLUDE: success carries no error TTLV.
        }
        if (status != 0) return static_cast<jlong>(0);
        if (response == nullptr) {
            throw_invalid_input(env); // COVERAGE-EXCLUDE: success always returns a response handle.
            return static_cast<jlong>(0); // COVERAGE-EXCLUDE: null success is forbidden by the FFI contract.
        }
        return to_java_handle(response);
    });
}

jlong execute_guarded_protection_item(JNIEnv* env, jlong raw_client, jlong raw_context,
        jlong raw_item, jlong raw_cost, jlong raw_asynchronous_indicator,
        bool has_asynchronous_indicator, jlongArray raw_limits) {
    auto* client = from_java_handle<kmipkit_client_t>(raw_client);
    auto* context = from_java_handle<kmipkit_usage_allocation_context_t>(raw_context);
    auto* item = from_java_handle<kmipkit_client_batch_item_t>(raw_item);
    if (client == nullptr || context == nullptr || item == nullptr || raw_cost <= 0
            || (has_asynchronous_indicator
                && (raw_asynchronous_indicator < 0
                    || static_cast<uint64_t>(raw_asynchronous_indicator) > 0xffff'ffffULL))) {
        throw_invalid_input(env);
        return 0;
    }

    kmipkit::jni::NativeHandleTransferGuard<
            kmipkit_client_batch_item_t*,
            decltype(&kmipkit_client_batch_item_release)> item_guard(
                    item, kmipkit_client_batch_item_release);
    CodecLimits limits;
    if (!limits.initialize(env, raw_limits)) return 0;

    kmipkit_client_batch_response_t* response = nullptr;
    const int32_t status = has_asynchronous_indicator
            ? kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
                    client, context, &item, raw_cost,
                    static_cast<uint32_t>(raw_asynchronous_indicator), limits.get(), &response)
            : kmipkit_client_execute_guarded_protection_item(
                    client, context, &item, raw_cost, limits.get(), &response);
    if (item == nullptr) (void)item_guard.transfer_to_consumer();
    if (status != 0) {
        if (response != nullptr) kmipkit_client_batch_response_release(response);
        throw_status(env, status);
        return 0;
    }
    if (item != nullptr || response == nullptr) {
        if (response != nullptr) kmipkit_client_batch_response_release(response);
        throw_invalid_input(env);
        return 0;
    }
    return to_java_handle(response);
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExecuteGuardedProtectionItem(
        JNIEnv* env, jclass, jlong raw_client, jlong raw_context, jlong raw_item,
        jlong raw_cost, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        return execute_guarded_protection_item(
                env, raw_client, raw_context, raw_item, raw_cost, 0, false, raw_limits);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientExecuteGuardedProtectionItemWithAsynchronousIndicator(
        JNIEnv* env, jclass, jlong raw_client, jlong raw_context, jlong raw_item,
        jlong raw_cost, jlong raw_asynchronous_indicator, jlongArray raw_limits) {
    return guarded<jlong>(env, 0, [&] {
        return execute_guarded_protection_item(env, raw_client, raw_context, raw_item,
                raw_cost, raw_asynchronous_indicator, true, raw_limits);
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
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: successful v2 getter normalizes this record.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: successful v2 getter normalizes this record.
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
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI normalizes presence to 0/1.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: FFI normalizes presence to 0/1.
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

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseCertify(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_certify(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI normalizes presence to 0/1.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: FFI normalizes presence to 0/1.
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_certify_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present ||
                (response_ttlv_present == 1 && response_ttlv == nullptr) ||
                (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: paired getters share one immutable result.
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRecertify(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_recertify(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && identifier.kind == 0)
                || (present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_recertify_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present
                || (response_ttlv_present == 1 && response_ttlv == nullptr)
                || (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobject identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRekey(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_rekey(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && identifier.kind == 0)
                || (present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_rekey_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present
                || (response_ttlv_present == 1 && response_ttlv == nullptr)
                || (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobject identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRekeyKeyPair(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t private_identifier{};
        private_identifier.struct_size = static_cast<uint32_t>(sizeof(private_identifier));
        private_identifier.abi_version = 1;
        kmipkit_unique_identifier_v1_t public_identifier{};
        public_identifier.struct_size = static_cast<uint32_t>(sizeof(public_identifier));
        public_identifier.abi_version = 1;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_rekey_key_pair(
                item, &present, &private_identifier, &public_identifier);
        if (status != 0) { throw_status(env, status); return static_cast<jobjectArray>(nullptr); }
        if (present > 1 || (present == 1 && (private_identifier.kind == 0 || public_identifier.kind == 0))
                || (present == 0 && (private_identifier.kind != 0 || public_identifier.kind != 0))) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); return static_cast<jobjectArray>(nullptr);
        }
        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        jobject private_record = present == 1 ? identifier_to_java(env, private_identifier, 1) : nullptr;
        jobject public_record = present == 1 ? identifier_to_java(env, public_identifier, 1) : nullptr;
        jlongArray ttlv_handle = make_long_array(env, {to_java_handle(response_ttlv)});
        if (fields == nullptr || ttlv_handle == nullptr || (present == 1 && (private_record == nullptr || public_record == nullptr))) {
            kmipkit_ttlv_structure_release(response_ttlv);
            if (fields != nullptr) env->DeleteLocalRef(fields);
            if (private_record != nullptr) env->DeleteLocalRef(private_record);
            if (public_record != nullptr) env->DeleteLocalRef(public_record);
            if (ttlv_handle != nullptr) env->DeleteLocalRef(ttlv_handle);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, private_record, public_record, ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (private_record != nullptr) env->DeleteLocalRef(private_record);
        if (public_record != nullptr) env->DeleteLocalRef(public_record);
        env->DeleteLocalRef(ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseReprovision(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t identifier_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_reprovision(
                item, &present, &identifier_present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || identifier_present > 1
                || (present == 0 && identifier_present != 0)
                || (identifier_present == 1 && identifier.kind == 0)
                || (identifier_present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_reprovision_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        jlongArray response_fields = make_long_array(env, {static_cast<jlong>(present)});
        jlongArray identifier_fields = make_long_array(
                env, {static_cast<jlong>(identifier_present)});
        jobject identifier_record = identifier_present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_fields == nullptr || identifier_fields == nullptr
                || response_ttlv_handle == nullptr
                || (identifier_present == 1 && identifier_record == nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            if (response_fields != nullptr) env->DeleteLocalRef(response_fields);
            if (identifier_fields != nullptr) env->DeleteLocalRef(identifier_fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            if (response_ttlv_handle != nullptr) env->DeleteLocalRef(response_ttlv_handle);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {response_fields, identifier_fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(response_fields);
        env->DeleteLocalRef(identifier_fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseReprovisionTtlv(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_reprovision_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRegister(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_register(
                item, &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && identifier.kind == 0)
                || (present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_register_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        jlongArray response_fields = make_long_array(env, {static_cast<jlong>(present)});
        jobject identifier_record = present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_fields == nullptr || response_ttlv_handle == nullptr
                || (present == 1 && identifier_record == nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            if (response_fields != nullptr) env->DeleteLocalRef(response_fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            if (response_ttlv_handle != nullptr) env->DeleteLocalRef(response_ttlv_handle);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {response_fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(response_fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRevoke(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_revoke(
                item, &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && identifier.kind == 0)
                || (present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_revoke_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }

        jlongArray response_fields = make_long_array(env, {static_cast<jlong>(present)});
        jobject identifier_record = present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_fields == nullptr || response_ttlv_handle == nullptr
                || (present == 1 && identifier_record == nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            if (response_fields != nullptr) env->DeleteLocalRef(response_fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            if (response_ttlv_handle != nullptr) env->DeleteLocalRef(response_ttlv_handle);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {response_fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(response_fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRngRetrieve(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t data_present = 0;
        kmipkit_byte_span_v1_t data{};
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_rng_retrieve(
                item, &present, &data_present, &data);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || data_present > 1
                || (present == 0 && data_present != 0)
                || (present == 1 && data_present != 1)
                || (data_present == 0 && (data.data != nullptr || data.length != 0))) { // COVERAGE-EXCLUDE: the C ABI initializes these paired flags and span atomically from one parsed result.
            throw_invalid_input(env); // COVERAGE-EXCLUDE: malformed paired outputs require violating the C ABI contract.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: malformed paired outputs require violating the C ABI contract.
        }

        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_rng_retrieve_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) { // COVERAGE-EXCLUDE: both C getters read the same immutable parsed item; no deterministic allocator failure hook exists.
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
        }

        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(data_present)});
        jbyteArray data_value = span_to_java_bytes(env, data_present, data);
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (fields == nullptr || env->ExceptionCheck() || response_ttlv_handle == nullptr) { // COVERAGE-EXCLUDE: requires JNI array allocation failure or pending OOM, which cannot be injected safely.
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            if (fields != nullptr) env->DeleteLocalRef(fields); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            if (data_value != nullptr) env->DeleteLocalRef(data_value); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            if (response_ttlv_handle != nullptr) env->DeleteLocalRef(response_ttlv_handle); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
        }
        jobjectArray result = make_object_array(env, {fields, data_value, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (data_value != nullptr) env->DeleteLocalRef(data_value);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseRngSeed(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t data_length_present = 0;
        int32_t data_length = 0;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_rng_seed(
                item, &present, &data_length_present, &data_length);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || data_length_present > 1
                || (present == 0 && data_length_present != 0)
                || (data_length_present == 0 && data_length != 0)) { // COVERAGE-EXCLUDE: the C ABI initializes these paired fields atomically from one parsed result.
            throw_invalid_input(env); // COVERAGE-EXCLUDE: malformed paired outputs require violating the C ABI contract.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: malformed paired outputs require violating the C ABI contract.
        }

        uint8_t ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_rng_seed_ttlv(
                item, &ttlv_present, &response_ttlv);
        if (ttlv_status != 0 || ttlv_present != present
                || (present == 1 && (data_length_present != 1 || response_ttlv == nullptr))
                || (present == 0 && response_ttlv != nullptr)) { // COVERAGE-EXCLUDE: both C getters read the same immutable parsed item; no deterministic allocator failure hook exists.
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
            throw_status(env, ttlv_status != 0 ? ttlv_status : KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: inconsistent getter outputs require an ABI invariant violation.
        }

        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(data_length_present),
                static_cast<jlong>(data_length)});
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (fields == nullptr || response_ttlv_handle == nullptr) { // COVERAGE-EXCLUDE: requires JNI array allocation failure, which cannot be injected safely.
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            if (fields != nullptr) env->DeleteLocalRef(fields); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            if (response_ttlv_handle != nullptr) env->DeleteLocalRef(response_ttlv_handle); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: cleanup after JNI allocation failure has no deterministic injection hook.
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseCheck(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t identifier_present = 0;
        uint8_t count_present = 0;
        uint8_t mask_present = 0;
        uint8_t lease_present = 0;
        int64_t count = 0;
        int32_t usage_mask = 0;
        uint32_t lease = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_check(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier_present, &identifier,
                &count_present, &count, &mask_present, &usage_mask,
                &lease_present, &lease);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || identifier_present > 1 || count_present > 1
                || mask_present > 1 || lease_present > 1
                || (present == 0 && (identifier_present != 0 || count_present != 0
                    || count != 0 || mask_present != 0 || usage_mask != 0
                    || lease_present != 0 || lease != 0))) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_check_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present
                || (response_ttlv_present == 1 && response_ttlv == nullptr)
                || (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(identifier_present),
                static_cast<jlong>(count_present), static_cast<jlong>(count),
                static_cast<jlong>(mask_present), static_cast<jlong>(usage_mask),
                static_cast<jlong>(lease_present), static_cast<jlong>(lease)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifier_record = identifier_present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        if (identifier_present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseDelegatedLogin(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        DelegatedLoginResponseCleanup cleanup(env);
        uint8_t present = 0;
        uint8_t ticket_present = 0;
        uint32_t ticket_type = 0;
        kmipkit_byte_span_v1_t ticket_value{nullptr, 0};
        kmipkit_client_batch_item_response_t* item =
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_delegated_login(
                item, &present, &ticket_present, &ticket_type, &ticket_value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || ticket_present > 1
                || (ticket_present == 0 && (ticket_type != 0
                    || ticket_value.data != nullptr || ticket_value.length != 0))
                || (present == 0 && ticket_present != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t ticket_ttlv_present = 0;
        kmipkit_ttlv_structure_t* ticket_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_delegated_login_ttlv(
                item, &ticket_ttlv_present, &ticket_ttlv);
        cleanup.set_ticket_ttlv(ticket_ttlv);
        if (ttlv_status != 0) {
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (ticket_ttlv_present > 1 || ticket_ttlv_present != ticket_present
                || (ticket_ttlv_present == 1 && ticket_ttlv == nullptr)
                || (ticket_ttlv_present == 0 && ticket_ttlv != nullptr)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray secret_value = span_to_java_bytes(env, ticket_present, ticket_value);
        cleanup.set_secret_value(secret_value);
        if (env->ExceptionCheck() || (ticket_present == 1 && secret_value == nullptr)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(ticket_present),
                static_cast<jlong>(ticket_type)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jlongArray ticket_ttlv_handle = make_long_array(
                env, {to_java_handle(ticket_ttlv)});
        if (ticket_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, secret_value, ticket_ttlv_handle});
        if (result != nullptr) cleanup.transfer_to_java();
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(ticket_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLogin(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        LoginResponseCleanup cleanup(env);
        uint8_t present = 0;
        uint8_t ticket_present = 0;
        uint32_t ticket_type = 0;
        kmipkit_byte_span_v1_t ticket_value{nullptr, 0};
        kmipkit_client_batch_item_response_t* item =
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_login(
                item, &present, &ticket_present, &ticket_type, &ticket_value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || ticket_present > 1
                || (ticket_present == 0 && (ticket_type != 0
                    || ticket_value.data != nullptr || ticket_value.length != 0))
                || (present == 0 && ticket_present != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t ticket_ttlv_present = 0;
        kmipkit_ttlv_structure_t* ticket_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_login_ttlv(
                item, &ticket_ttlv_present, &ticket_ttlv);
        cleanup.set_ticket_ttlv(ticket_ttlv);
        if (ttlv_status != 0) {
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (ticket_ttlv_present > 1 || ticket_ttlv_present != ticket_present
                || (ticket_ttlv_present == 1 && ticket_ttlv == nullptr)
                || (ticket_ttlv_present == 0 && ticket_ttlv != nullptr)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray secret_value = span_to_java_bytes(env, ticket_present, ticket_value);
        cleanup.set_secret_value(secret_value);
        if (env->ExceptionCheck() || (ticket_present == 1 && secret_value == nullptr)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(ticket_present),
                static_cast<jlong>(ticket_type)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jlongArray ticket_ttlv_handle = make_long_array(
                env, {to_java_handle(ticket_ttlv)});
        if (ticket_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, secret_value, ticket_ttlv_handle});
        if (result != nullptr) cleanup.transfer_to_java();
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(ticket_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseGetUsageAllocation(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        JniClientBatchResponseContext context{env, raw_item};
        const auto operations = response_bridge_operations(
                context, &get_usage_allocation_response,
                &get_usage_allocation_response_ttlv,
                &validate_usage_allocation_response);
        return static_cast<jobjectArray>(
                kmipkit::jni::marshal_client_batch_item_response(operations));
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextCreate(
        JNIEnv* env, jclass, jlong raw_item, jlong raw_requested_units,
        jlong raw_usage_limits_unit) {
    return guarded<jlong>(env, 0, [&] {
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        if (item == nullptr || raw_requested_units <= 0 || raw_usage_limits_unit < 0
                || static_cast<uint64_t>(raw_usage_limits_unit) > 0xffff'ffffULL) {
            throw_invalid_input(env);
            return static_cast<jlong>(0);
        }
        return return_handle<kmipkit_usage_allocation_context_t>(env, [&](auto output) {
            return kmipkit_usage_allocation_context_create(item, raw_requested_units,
                    static_cast<uint32_t>(raw_usage_limits_unit), output);
        });
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextUsageLimitsUnit(
        JNIEnv* env, jclass, jlong raw_context) {
    return guarded<jlong>(env, 0, [&] {
        uint32_t value = 0;
        const int32_t status = kmipkit_usage_allocation_context_usage_limits_unit(
                from_java_handle<kmipkit_usage_allocation_context_t>(raw_context), &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextAllocatedUnits(
        JNIEnv* env, jclass, jlong raw_context) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        const int32_t status = kmipkit_usage_allocation_context_allocated_units(
                from_java_handle<kmipkit_usage_allocation_context_t>(raw_context), &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextAvailableUnits(
        JNIEnv* env, jclass, jlong raw_context) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        const int32_t status = kmipkit_usage_allocation_context_available_units(
                from_java_handle<kmipkit_usage_allocation_context_t>(raw_context), &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jlong JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextReservedUnits(
        JNIEnv* env, jclass, jlong raw_context) {
    return guarded<jlong>(env, 0, [&] {
        int64_t value = 0;
        const int32_t status = kmipkit_usage_allocation_context_reserved_units(
                from_java_handle<kmipkit_usage_allocation_context_t>(raw_context), &value);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlong>(0);
        }
        return static_cast<jlong>(value);
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_usageAllocationContextUniqueIdentifier(
        JNIEnv* env, jclass, jlong raw_context) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_usage_allocation_context_unique_identifier(
                from_java_handle<kmipkit_usage_allocation_context_t>(raw_context), &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        return identifier_to_java(env, identifier, 1);
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseObtainLease(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        JniClientBatchResponseContext context{env, raw_item};
        const auto operations = response_bridge_operations(
                context, &get_obtain_lease_response,
                &get_obtain_lease_response_ttlv,
                &validate_obtain_lease_response);
        return static_cast<jobjectArray>(
                kmipkit::jni::marshal_client_batch_item_response(operations));
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseImport(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_import(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_import_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present ||
                (response_ttlv_present == 1 && response_ttlv == nullptr) ||
                (response_ttlv_present == 0 && response_ttlv != nullptr) ||
                (present == 1 && identifier.kind == 0) ||
                (present == 0 && identifier.kind != 0)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobject identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseJoinSplitKey(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_join_split_key(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && identifier.kind == 0)
                || (present == 0 && identifier.kind != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_join_split_key_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present
                || (response_ttlv_present == 1 && response_ttlv == nullptr)
                || (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobject identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseGet(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t payload_present = 0;
        uint32_t object_type = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_ttlv_structure_t* any_object = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_get(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &payload_present, &object_type, &identifier, &any_object);
        if (status != 0) {
            kmipkit_ttlv_structure_release(any_object);
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || payload_present > 1
                || (present == 0 && payload_present != 0)
                || (payload_present == 0 && (object_type != 0 || identifier.kind != 0
                        || any_object != nullptr))
                || (payload_present == 1 && (present != 1 || any_object == nullptr))) {
            kmipkit_ttlv_structure_release(any_object);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(payload_present),
                static_cast<jlong>(object_type)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(any_object);
            return static_cast<jobjectArray>(nullptr);
        }
        jobject identifier_record = payload_present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        if (payload_present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(any_object);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray handles = make_long_array(env, {to_java_handle(any_object)});
        if (handles == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(any_object);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, identifier_record, handles});
        if (result == nullptr) kmipkit_ttlv_structure_release(any_object);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(handles);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLocate(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t payload_present = 0;
        uint8_t located_items_present = 0;
        int32_t located_items = 0;
        uint64_t identifier_count = 0;
        auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_locate(
                const_cast<kmipkit_client_batch_item_response_t*>(item), &present,
                &payload_present, &located_items_present, &located_items, &identifier_count);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || payload_present > 1 || located_items_present > 1
                || (present == 0 && payload_present != 0)
                || (payload_present == 0 && (located_items_present != 0 || located_items != 0
                        || identifier_count != 0))
                || (located_items_present == 0 && located_items != 0)) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: The FFI derives outputs from one typed response.
            return static_cast<jobjectArray>(nullptr);
        }
        if (identifier_count > kMaximumNativeInputArrayElements
                || identifier_count > static_cast<uint64_t>(std::numeric_limits<jsize>::max())) {
            throw_java(env, "org/kmipkit/ResourceLimitException",
                    "Locate response identifier count exceeds the configured limit");
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(payload_present),
                static_cast<jlong>(located_items_present), static_cast<jlong>(located_items),
                static_cast<jlong>(identifier_count)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jclass object_class = env->FindClass("java/lang/Object");
        if (object_class == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifiers = env->NewObjectArray(
                static_cast<jsize>(identifier_count), object_class, nullptr);
        env->DeleteLocalRef(object_class);
        if (identifiers == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        for (uint64_t index = 0; index < identifier_count; ++index) {
            kmipkit_unique_identifier_v1_t identifier{};
            identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
            identifier.abi_version = 1;
            const int32_t identifier_status =
                    kmipkit_client_batch_item_response_locate_unique_identifier_at(
                            const_cast<kmipkit_client_batch_item_response_t*>(item), index,
                            &identifier);
            if (identifier_status != 0) {
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                throw_status(env, identifier_status);
                return static_cast<jobjectArray>(nullptr);
            }
            jobject record = identifier_to_java(env, identifier, 1);
            if (record == nullptr) {
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                return static_cast<jobjectArray>(nullptr);
            }
            env->SetObjectArrayElement(identifiers, static_cast<jsize>(index), record);
            env->DeleteLocalRef(record);
            if (env->ExceptionCheck()) {
                env->DeleteLocalRef(fields);
                env->DeleteLocalRef(identifiers);
                return static_cast<jobjectArray>(nullptr);
            }
        }
        jobjectArray result = make_object_array(env, {fields, identifiers});
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(identifiers);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseGetConstraints(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_get_constraints_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI returns presence and handle from one typed outcome.
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseSetConstraints(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_set_constraints_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI returns presence and handle from one typed outcome.
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jlongArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponsePing(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jlongArray>(env, nullptr, [&] {
        uint8_t present = 0;
        const int32_t status = kmipkit_client_batch_item_response_ping(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &present);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlongArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI converts this Ping option to 0 or 1.
            return static_cast<jlongArray>(nullptr);
        }
        return make_long_array(env, {static_cast<jlong>(present)});
    });
}

JNIEXPORT jlongArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLogout(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jlongArray>(env, nullptr, [&] {
        uint8_t present = 0;
        const int32_t status = kmipkit_client_batch_item_response_logout(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item), &present);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jlongArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI converts this Logout option to 0 or 1.
            return static_cast<jlongArray>(nullptr);
        }
        return make_long_array(env, {static_cast<jlong>(present)});
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseQueryTtlv(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_query_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseInterop(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_interop_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponsePkcs11(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        auto* item = const_cast<kmipkit_client_batch_item_response_t*>(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item));
        Pkcs11ResponseCleanup cleanup(env);
        std::array<uint8_t, 5> present{};
        const auto get_structure = [&](size_t index, auto getter) -> bool {
            kmipkit_ttlv_structure_t* structure = nullptr;
            const int32_t status = getter(item, &present[index], &structure);
            cleanup.set_structure(index, structure);
            if (status != 0) {
                throw_status(env, status);
                return false;
            }
            if (present[index] > 1
                    || (present[index] == 1 && structure == nullptr)
                    || (present[index] == 0 && structure != nullptr)) {
                throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: FFI derives presence and handle from one optional TTLV.
                return false;
            }
            return true;
        };
        if (!get_structure(0, kmipkit_client_batch_item_response_pkcs11_ttlv)
                || !get_structure(1,
                        kmipkit_client_batch_item_response_pkcs11_interface_ttlv)
                || !get_structure(2,
                        kmipkit_client_batch_item_response_pkcs11_function_ttlv)
                || !get_structure(3,
                        kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv)
                || !get_structure(4,
                        kmipkit_client_batch_item_response_pkcs11_return_code_ttlv)) {
            return static_cast<jobjectArray>(nullptr);
        }

        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t correlation_span{};
        const int32_t correlation_status =
                kmipkit_client_batch_item_response_pkcs11_correlation_value(
                        item, &correlation_present, &correlation_span);
        if (correlation_status != 0) {
            throw_status(env, correlation_status);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray correlation = span_to_java_bytes(env, correlation_present, correlation_span);
        if (env->ExceptionCheck() || (correlation_present == 1 && correlation == nullptr)) {
            return static_cast<jobjectArray>(nullptr);
        }
        cleanup.set_correlation(correlation);

        std::vector<jlong> field_values {
                static_cast<jlong>(present[0]), static_cast<jlong>(present[1]),
                static_cast<jlong>(correlation_present), static_cast<jlong>(present[2]),
                static_cast<jlong>(present[3]), static_cast<jlong>(present[4])};
        const auto handle_value = [&](size_t index) -> jlong {
            return to_java_handle(cleanup.structure(index));
        };
        jlongArray fields = make_long_array(env, field_values);
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jlongArray handles = make_long_array(env, {
                handle_value(0), handle_value(1), handle_value(2),
                handle_value(3), handle_value(4)});
        if (handles == nullptr) {
            env->DeleteLocalRef(fields);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, correlation, handles});
        if (result != nullptr) {
            cleanup.transfer_structures_to_java();
            cleanup.transfer_correlation_to_java();
        }
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(handles);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseLog(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t status = kmipkit_client_batch_item_response_log_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &response_ttlv);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || (present == 1 && response_ttlv == nullptr)
                || (present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        env->DeleteLocalRef(response_ttlv_handle);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseExport(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t payload_present = 0;
        uint32_t object_type = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_ttlv_structure_t* attributes = nullptr;
        kmipkit_ttlv_structure_t* any_object = nullptr;
        const auto* item = from_java_handle<kmipkit_client_batch_item_response_t>(raw_item);
        const int32_t status = kmipkit_client_batch_item_response_export(
                const_cast<kmipkit_client_batch_item_response_t*>(item), &present,
                &payload_present, &object_type, &identifier, &attributes, &any_object);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_export_ttlv(
                const_cast<kmipkit_client_batch_item_response_t*>(item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            kmipkit_ttlv_structure_release(attributes);
            kmipkit_ttlv_structure_release(any_object);
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || payload_present > 1 || response_ttlv_present > 1
                || (present == 0 && (payload_present != 0 || response_ttlv_present != 0
                    || attributes != nullptr || any_object != nullptr))
                || (present == 1 && (response_ttlv_present != 1 || response_ttlv == nullptr))
                || (payload_present == 0 && (attributes != nullptr || any_object != nullptr))
                || (payload_present == 1 && any_object == nullptr)) { // COVERAGE-EXCLUDE: paired getters share one immutable result.
            kmipkit_ttlv_structure_release(attributes); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            kmipkit_ttlv_structure_release(any_object); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            kmipkit_ttlv_structure_release(response_ttlv); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: paired getters share one immutable result.
            return static_cast<jobjectArray>(nullptr); // COVERAGE-EXCLUDE: paired getters share one immutable result.
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(payload_present),
                static_cast<jlong>(object_type)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(attributes);
            kmipkit_ttlv_structure_release(any_object);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifier_record = payload_present == 1
                ? identifier_to_java(env, identifier, 1) : nullptr;
        if (payload_present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(attributes);
            kmipkit_ttlv_structure_release(any_object);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray handle_values = make_long_array(env, {
                to_java_handle(attributes), to_java_handle(any_object), to_java_handle(response_ttlv)});
        if (handle_values == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(attributes);
            kmipkit_ttlv_structure_release(any_object);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, identifier_record, handle_values});
        if (result != nullptr) {
            attributes = nullptr;
            any_object = nullptr;
            response_ttlv = nullptr;
        }
        kmipkit_ttlv_structure_release(attributes);
        kmipkit_ttlv_structure_release(any_object);
        kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(handle_values);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseDeriveKey(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        const int32_t status = kmipkit_client_batch_item_response_derive_key(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1) {
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        uint8_t response_ttlv_present = 0;
        kmipkit_ttlv_structure_t* response_ttlv = nullptr;
        const int32_t ttlv_status = kmipkit_client_batch_item_response_derive_key_ttlv(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &response_ttlv_present, &response_ttlv);
        if (ttlv_status != 0) {
            throw_status(env, ttlv_status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (response_ttlv_present > 1 || response_ttlv_present != present ||
                (response_ttlv_present == 1 && response_ttlv == nullptr) ||
                (response_ttlv_present == 0 && response_ttlv != nullptr)) {
            kmipkit_ttlv_structure_release(response_ttlv);
            throw_status(env, KMIPKIT_ERROR_INVALID_INPUT);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {static_cast<jlong>(present)});
        if (fields == nullptr) {
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray identifier_record = present == 1 ? identifier_to_java(env, identifier, 1) : nullptr;
        if (present == 1 && identifier_record == nullptr) {
            env->DeleteLocalRef(fields);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray response_ttlv_handle = make_long_array(
                env, {to_java_handle(response_ttlv)});
        if (response_ttlv_handle == nullptr) {
            env->DeleteLocalRef(fields);
            if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
            kmipkit_ttlv_structure_release(response_ttlv);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(
                env, {fields, identifier_record, response_ttlv_handle});
        if (result == nullptr) kmipkit_ttlv_structure_release(response_ttlv);
        env->DeleteLocalRef(fields);
        if (identifier_record != nullptr) env->DeleteLocalRef(identifier_record);
        env->DeleteLocalRef(response_ttlv_handle);
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

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseEncrypt(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        auto cleanup_sensitive_array = [env](jbyteArray value, bool wipe) noexcept {
            if (wipe) wipe_java_byte_array(env, value);
            env->DeleteLocalRef(value);
        };
        kmipkit::jni::ResponseSecretArrayScope<
                jbyteArray, 4, decltype(cleanup_sensitive_array)> sensitive_arrays(
                        cleanup_sensitive_array);
        uint8_t present = 0;
        uint8_t data_present = 0;
        uint8_t iv_present = 0;
        uint8_t correlation_present = 0;
        uint8_t tag_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t data{};
        kmipkit_byte_span_v1_t iv{};
        kmipkit_byte_span_v1_t correlation{};
        kmipkit_byte_span_v1_t tag{};
        const int32_t status = kmipkit_client_batch_item_response_encrypt(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &data_present, &data, &iv_present, &iv,
                &correlation_present, &correlation, &tag_present, &tag);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || data_present > 1 || iv_present > 1
                || correlation_present > 1 || tag_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(data_present),
                static_cast<jlong>(iv_present), static_cast<jlong>(correlation_present),
                static_cast<jlong>(tag_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        std::array<jbyteArray, 4> secret_values{};
        const bool converted = kmipkit::jni::convert_response_arrays(
                secret_values,
                [&](std::size_t index) {
                    switch (index) {
                    case 0: return span_to_java_bytes(env, data_present, data);
                    case 1: return span_to_java_bytes(env, iv_present, iv);
                    case 2: return span_to_java_bytes(env, correlation_present, correlation);
                    default: return span_to_java_bytes(env, tag_present, tag);
                    }
                },
                [&] { return env->ExceptionCheck() == JNI_TRUE; },
                sensitive_arrays);
        if (!converted) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env,
                {fields, identifier_text, secret_values[0], secret_values[1],
                 secret_values[2], secret_values[3]});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (result != nullptr && !env->ExceptionCheck()) sensitive_arrays.transfer_to_java();
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseDecrypt(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        auto cleanup_sensitive_array = [env](jbyteArray value, bool wipe) noexcept {
            if (wipe) wipe_java_byte_array(env, value);
            env->DeleteLocalRef(value);
        };
        kmipkit::jni::ResponseSecretArrayScope<
                jbyteArray, 2, decltype(cleanup_sensitive_array)> sensitive_arrays(
                        cleanup_sensitive_array);
        uint8_t present = 0;
        uint8_t data_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t data{};
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_decrypt(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &data_present, &data,
                &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || data_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(data_present),
                static_cast<jlong>(correlation_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        std::array<jbyteArray, 2> secret_values{};
        const bool converted = kmipkit::jni::convert_response_arrays(
                secret_values,
                [&](std::size_t index) {
                    return index == 0
                            ? span_to_java_bytes(env, data_present, data)
                            : span_to_java_bytes(env, correlation_present, correlation);
                },
                [&] { return env->ExceptionCheck() == JNI_TRUE; },
                sensitive_arrays);
        if (!converted) {
            if (!env->ExceptionCheck()) throw_invalid_input(env);
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env,
                {fields, identifier_text, secret_values[0], secret_values[1]});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (result != nullptr && !env->ExceptionCheck()) sensitive_arrays.transfer_to_java();
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseHash(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t data_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_byte_span_v1_t data{};
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_hash(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &data_present, &data, &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || data_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), static_cast<jlong>(data_present),
                static_cast<jlong>(correlation_present)});
        if (fields == nullptr) return static_cast<jobjectArray>(nullptr);
        jbyteArray data_value = span_to_java_bytes(env, data_present, data);
        jbyteArray correlation_value = span_to_java_bytes(env, correlation_present, correlation);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            if (data_value != nullptr) env->DeleteLocalRef(data_value);
            if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, data_value, correlation_value});
        env->DeleteLocalRef(fields);
        if (data_value != nullptr) env->DeleteLocalRef(data_value);
        if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseMac(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t mac_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t mac_data{};
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_mac(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &mac_present, &mac_data,
                &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || mac_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(mac_present),
                static_cast<jlong>(correlation_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray mac_value = span_to_java_bytes(env, mac_present, mac_data);
        jbyteArray correlation_value = span_to_java_bytes(env, correlation_present, correlation);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            if (mac_value != nullptr) env->DeleteLocalRef(mac_value);
            if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, identifier_text, mac_value, correlation_value});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (mac_value != nullptr) env->DeleteLocalRef(mac_value);
        if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseMacVerify(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t validity_present = 0;
        uint32_t validity = 0;
        uint8_t correlation_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_mac_verify(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &validity_present, &validity,
                &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || validity_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(validity_present),
                static_cast<jlong>(validity), static_cast<jlong>(correlation_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray correlation_value = span_to_java_bytes(env, correlation_present, correlation);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env, {fields, identifier_text, correlation_value});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseSign(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t signature_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t signature_data{};
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_sign(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &signature_present, &signature_data,
                &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || signature_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(signature_present),
                static_cast<jlong>(correlation_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray signature_value = span_to_java_bytes(env, signature_present, signature_data);
        jbyteArray correlation_value = span_to_java_bytes(env, correlation_present, correlation);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            if (signature_value != nullptr) env->DeleteLocalRef(signature_value);
            if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env,
                {fields, identifier_text, signature_value, correlation_value});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (signature_value != nullptr) env->DeleteLocalRef(signature_value);
        if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
        return result;
    });
}

JNIEXPORT jobjectArray JNICALL Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponseSignatureVerify(
        JNIEnv* env, jclass, jlong raw_item) {
    return guarded<jobjectArray>(env, nullptr, [&] {
        uint8_t present = 0;
        uint8_t validity_present = 0;
        uint32_t validity = 0;
        uint8_t recovered_present = 0;
        uint8_t correlation_present = 0;
        kmipkit_unique_identifier_v1_t identifier{};
        identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
        identifier.abi_version = 1;
        kmipkit_byte_span_v1_t recovered_data{};
        kmipkit_byte_span_v1_t correlation{};
        const int32_t status = kmipkit_client_batch_item_response_signature_verify(
                from_java_handle<kmipkit_client_batch_item_response_t>(raw_item),
                &present, &identifier, &validity_present, &validity,
                &recovered_present, &recovered_data, &correlation_present, &correlation);
        if (status != 0) {
            throw_status(env, status);
            return static_cast<jobjectArray>(nullptr);
        }
        if (present > 1 || validity_present > 1 || recovered_present > 1 || correlation_present > 1) {
            throw_invalid_input(env);
            return static_cast<jobjectArray>(nullptr);
        }
        jlong identifier_fields[4]{};
        jbyteArray identifier_text = nullptr;
        if (!response_identifier_to_java(env, present, identifier, identifier_fields, &identifier_text)) {
            return static_cast<jobjectArray>(nullptr);
        }
        jlongArray fields = make_long_array(env, {
                static_cast<jlong>(present), identifier_fields[0], identifier_fields[1],
                identifier_fields[2], identifier_fields[3], static_cast<jlong>(validity_present),
                static_cast<jlong>(validity), static_cast<jlong>(recovered_present),
                static_cast<jlong>(correlation_present)});
        if (fields == nullptr) {
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            return static_cast<jobjectArray>(nullptr);
        }
        jbyteArray recovered_value = span_to_java_bytes(env, recovered_present, recovered_data);
        jbyteArray correlation_value = span_to_java_bytes(env, correlation_present, correlation);
        if (env->ExceptionCheck()) {
            env->DeleteLocalRef(fields);
            if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
            if (recovered_value != nullptr) env->DeleteLocalRef(recovered_value);
            if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
            return static_cast<jobjectArray>(nullptr);
        }
        jobjectArray result = make_object_array(env,
                {fields, identifier_text, recovered_value, correlation_value});
        env->DeleteLocalRef(fields);
        if (identifier_text != nullptr) env->DeleteLocalRef(identifier_text);
        if (recovered_value != nullptr) env->DeleteLocalRef(recovered_value);
        if (correlation_value != nullptr) env->DeleteLocalRef(correlation_value);
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
