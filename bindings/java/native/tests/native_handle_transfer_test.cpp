#include "../native_handle_transfer_guard.hpp"

#include <iostream>
#include <new>

namespace {

struct RevocationReasonHandle final {
    int value;
};

RevocationReasonHandle* released_handle = nullptr;
int release_count = 0;

void release_reason(RevocationReasonHandle* handle) noexcept {
    released_handle = handle;
    ++release_count;
}

bool pre_consumer_failure_releases_transferred_reason() {
    released_handle = nullptr;
    release_count = 0;
    RevocationReasonHandle reason{41};
    try {
        kmipkit::jni::NativeHandleTransferGuard<
                RevocationReasonHandle*, decltype(&release_reason)> reason_transfer(
                        &reason, &release_reason);
        throw std::bad_alloc();
    } catch (const std::bad_alloc&) {
    }
    return released_handle == &reason && release_count == 1;
}

bool ffi_consumption_disarms_reason_guard() {
    released_handle = nullptr;
    release_count = 0;
    RevocationReasonHandle reason{73};
    RevocationReasonHandle* consumed_handle = nullptr;
    {
        kmipkit::jni::NativeHandleTransferGuard<
                RevocationReasonHandle*, decltype(&release_reason)> reason_transfer(
                        &reason, &release_reason);
        consumed_handle = reason_transfer.transfer_to_consumer();
    }
    release_reason(consumed_handle);
    return released_handle == &reason && release_count == 1;
}

} // namespace

int main() {
    if (!pre_consumer_failure_releases_transferred_reason()) {
        std::cerr << "pre-consumer JNI failure leaked the transferred Revoke reason handle\n";
        return 1;
    }
    if (!ffi_consumption_disarms_reason_guard()) {
        std::cerr << "JNI released the Revoke reason after FFI took ownership\n";
        return 1;
    }
    return 0;
}
