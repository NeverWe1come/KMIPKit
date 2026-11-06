#ifndef KMIPKIT_JNI_RESPONSE_BRIDGE_HPP
#define KMIPKIT_JNI_RESPONSE_BRIDGE_HPP

#include "kmipkit_internal.h"

#include <array>
#include <cstddef>
#include <cstdint>

namespace kmipkit::jni {

constexpr size_t kClientBatchResponseFieldCapacity = 16;

struct ResponseBridgeOperations {
    void* context;
    int32_t (*get_response)(
            void*, uint8_t*, kmipkit_unique_identifier_v1_t*, int64_t*, size_t, size_t*);
    int32_t (*get_response_ttlv)(void*, uint8_t*, void**);
    bool (*validate_response)(
            void*, uint8_t, const kmipkit_unique_identifier_v1_t*, const int64_t*, size_t);
    void (*throw_status)(void*, int32_t);
    void (*release_ttlv)(void*, void*);
    void* (*make_fields)(void*, uint8_t, const int64_t*, size_t);
    void* (*make_identifier)(void*, const kmipkit_unique_identifier_v1_t*);
    void* (*make_ttlv_handle)(void*, void*);
    void* (*make_result)(void*, void*, void*, void*);
    void (*delete_local)(void*, void*);
};

inline void* marshal_client_batch_item_response(const ResponseBridgeOperations& operations) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t identifier{};
    identifier.struct_size = static_cast<uint32_t>(sizeof(identifier));
    identifier.abi_version = 1;
    std::array<int64_t, kClientBatchResponseFieldCapacity> values{};
    size_t value_count = 0;
    const int32_t status = operations.get_response(
            operations.context, &present, &identifier, values.data(), values.size(), &value_count);
    if (status != 0) {
        operations.throw_status(operations.context, status);
        return nullptr;
    }
    if (present > 1 || value_count > values.size()) {
        operations.throw_status(operations.context, KMIPKIT_ERROR_INVALID_INPUT);
        return nullptr;
    }

    uint8_t ttlv_present = 0;
    void* response_ttlv = nullptr;
    const int32_t ttlv_status = operations.get_response_ttlv(
            operations.context, &ttlv_present, &response_ttlv);
    if (ttlv_status != 0) {
        operations.throw_status(operations.context, ttlv_status);
        return nullptr;
    }
    if (ttlv_present > 1 || ttlv_present != present
            || (ttlv_present == 1 && response_ttlv == nullptr)
            || (ttlv_present == 0 && response_ttlv != nullptr)) {
        operations.release_ttlv(operations.context, response_ttlv);
        operations.throw_status(operations.context, KMIPKIT_ERROR_INVALID_INPUT);
        return nullptr;
    }
    if (!operations.validate_response(
                operations.context, present, &identifier, values.data(), value_count)) {
        operations.release_ttlv(operations.context, response_ttlv);
        operations.throw_status(operations.context, KMIPKIT_ERROR_INVALID_INPUT);
        return nullptr;
    }

    void* fields = operations.make_fields(
            operations.context, present, values.data(), value_count);
    if (fields == nullptr) {
        operations.release_ttlv(operations.context, response_ttlv);
        return nullptr;
    }
    void* identifier_record = nullptr;
    if (present == 1) {
        identifier_record = operations.make_identifier(operations.context, &identifier);
        if (identifier_record == nullptr) {
            operations.delete_local(operations.context, fields);
            operations.release_ttlv(operations.context, response_ttlv);
            return nullptr;
        }
    }
    void* ttlv_handle = operations.make_ttlv_handle(operations.context, response_ttlv);
    if (ttlv_handle == nullptr) {
        operations.delete_local(operations.context, fields);
        if (identifier_record != nullptr) {
            operations.delete_local(operations.context, identifier_record);
        }
        operations.release_ttlv(operations.context, response_ttlv);
        return nullptr;
    }
    void* result = operations.make_result(
            operations.context, fields, identifier_record, ttlv_handle);
    if (result == nullptr) {
        operations.release_ttlv(operations.context, response_ttlv);
    }
    operations.delete_local(operations.context, fields);
    if (identifier_record != nullptr) {
        operations.delete_local(operations.context, identifier_record);
    }
    operations.delete_local(operations.context, ttlv_handle);
    return result;
}

} // namespace kmipkit::jni

#endif // KMIPKIT_JNI_RESPONSE_BRIDGE_HPP
