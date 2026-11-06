#ifndef KMIPKIT_RESPONSE_SECRET_ARRAY_SCOPE_HPP
#define KMIPKIT_RESPONSE_SECRET_ARRAY_SCOPE_HPP

#include <array>
#include <cstddef>
#include <type_traits>
#include <utility>

namespace kmipkit::jni {

template <typename Array, std::size_t Capacity, typename Cleanup>
class ResponseSecretArrayScope final {
public:
    explicit ResponseSecretArrayScope(Cleanup cleanup) noexcept(
            std::is_nothrow_move_constructible_v<Cleanup>)
            : cleanup_(std::move(cleanup)) {}

    ResponseSecretArrayScope(const ResponseSecretArrayScope&) = delete;
    ResponseSecretArrayScope& operator=(const ResponseSecretArrayScope&) = delete;

    ~ResponseSecretArrayScope() {
        for (std::size_t index = 0; index < count_; ++index) {
            cleanup_(arrays_[index], !transferred_to_java_);
        }
    }

    bool add(Array value) noexcept(noexcept(cleanup_(value, true))) {
        if (value == nullptr) return true;
        if (count_ == Capacity) {
            cleanup_(value, true);
            return false;
        }
        arrays_[count_++] = value;
        return true;
    }

    void transfer_to_java() noexcept { transferred_to_java_ = true; }

private:
    Cleanup cleanup_;
    std::array<Array, Capacity> arrays_{};
    std::size_t count_ = 0;
    bool transferred_to_java_ = false;
};

} // namespace kmipkit::jni

#endif
