#ifndef KMIPKIT_JNI_CLIENT_BATCH_ITEM_TRANSFER_GUARD_HPP
#define KMIPKIT_JNI_CLIENT_BATCH_ITEM_TRANSFER_GUARD_HPP

#include <cstdint>

namespace kmipkit::jni {

inline bool valid_unique_batch_item_id_length(std::int64_t length) noexcept {
    constexpr std::int64_t maximum = 16LL * 1024 * 1024;
    return length >= 0 && length <= maximum;
}

template <typename Handle, typename Releaser>
class ClientBatchItemTransferGuard final {
public:
    ClientBatchItemTransferGuard(Handle handle, Releaser releaser) noexcept
            : handle_(handle), releaser_(releaser) {}

    ClientBatchItemTransferGuard(const ClientBatchItemTransferGuard&) = delete;
    ClientBatchItemTransferGuard& operator=(const ClientBatchItemTransferGuard&) = delete;

    ~ClientBatchItemTransferGuard() noexcept {
        if (armed_) {
            releaser_(handle_);
        }
    }

    Handle transfer_to_consumer() noexcept {
        armed_ = false;
        return handle_;
    }

private:
    Handle handle_;
    Releaser releaser_;
    bool armed_ = true;
};

} // namespace kmipkit::jni

#endif // KMIPKIT_JNI_CLIENT_BATCH_ITEM_TRANSFER_GUARD_HPP
