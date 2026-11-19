#ifndef KMIPKIT_JNI_NATIVE_HANDLE_TRANSFER_GUARD_HPP
#define KMIPKIT_JNI_NATIVE_HANDLE_TRANSFER_GUARD_HPP

namespace kmipkit::jni {

template <typename Handle, typename Releaser>
class NativeHandleTransferGuard final {
public:
    NativeHandleTransferGuard(Handle handle, Releaser releaser) noexcept
            : handle_(handle), releaser_(releaser) {}

    NativeHandleTransferGuard(const NativeHandleTransferGuard&) = delete;
    NativeHandleTransferGuard& operator=(const NativeHandleTransferGuard&) = delete;

    ~NativeHandleTransferGuard() noexcept {
        if (armed_) releaser_(handle_);
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

#endif // KMIPKIT_JNI_NATIVE_HANDLE_TRANSFER_GUARD_HPP
