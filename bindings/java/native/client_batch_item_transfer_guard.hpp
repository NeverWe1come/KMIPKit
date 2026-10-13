#ifndef KMIPKIT_JNI_CLIENT_BATCH_ITEM_TRANSFER_GUARD_HPP
#define KMIPKIT_JNI_CLIENT_BATCH_ITEM_TRANSFER_GUARD_HPP

namespace kmipkit::jni {

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
