#pragma once

#include <cstddef>
#include <cstdint>
#include <vector>

namespace kmipkit::secure {

template <typename Allocator>
class VectorWiper final {
public:
    explicit VectorWiper(std::vector<std::uint8_t, Allocator>& bytes) noexcept
            : bytes_(bytes) {}

    VectorWiper(const VectorWiper&) = delete;
    VectorWiper& operator=(const VectorWiper&) = delete;

    ~VectorWiper() noexcept {
        volatile std::uint8_t* data = bytes_.data();
        for (std::size_t index = 0; index < bytes_.size(); ++index) {
            data[index] = 0;
        }
    }

private:
    std::vector<std::uint8_t, Allocator>& bytes_;
};

} // namespace kmipkit::secure
