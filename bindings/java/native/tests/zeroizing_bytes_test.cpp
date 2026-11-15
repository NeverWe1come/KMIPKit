#include "../secure_bytes.hpp"

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <new>
#include <vector>

namespace {

bool deallocated_bytes_were_zeroized = false;

template <typename T>
struct InspectingAllocator {
    using value_type = T;

    T* allocate(std::size_t count) {
        return static_cast<T*>(::operator new(count * sizeof(T)));
    }

    void deallocate(T* bytes, std::size_t count) noexcept {
        constexpr std::size_t secret_length = 3;
        deallocated_bytes_were_zeroized = count >= secret_length;
        for (std::size_t index = 0; index < secret_length && index < count; ++index) {
            deallocated_bytes_were_zeroized = deallocated_bytes_were_zeroized && bytes[index] == 0;
        }
        ::operator delete(bytes);
    }

    template <typename U>
    bool operator==(const InspectingAllocator<U>&) const noexcept {
        return true;
    }
};

bool vector_wiper_clears_owned_bytes_before_deallocation() {
    deallocated_bytes_were_zeroized = false;
    {
        std::vector<std::uint8_t, InspectingAllocator<std::uint8_t>> bytes;
        bytes.push_back(0x53);
        bytes.push_back(0x45);
        bytes.push_back(0x43);
        kmipkit::secure::VectorWiper wiper(bytes);
    }
    return deallocated_bytes_were_zeroized;
}

} // namespace

int main() {
    if (!vector_wiper_clears_owned_bytes_before_deallocation()) {
        std::cerr << "JNI scratch vector was not zeroized before deallocation\n";
        return 1;
    }
    return 0;
}
