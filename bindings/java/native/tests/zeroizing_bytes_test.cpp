#include "../secure_bytes.hpp"

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <new>
#include <vector>

namespace {

bool deallocated_bytes_were_zeroized = false;

struct InspectingAllocator {
    using value_type = std::uint8_t;

    std::uint8_t* allocate(std::size_t count) {
        return static_cast<std::uint8_t*>(::operator new(count));
    }

    void deallocate(std::uint8_t* bytes, std::size_t count) noexcept {
        deallocated_bytes_were_zeroized = true;
        for (std::size_t index = 0; index < count; ++index) {
            deallocated_bytes_were_zeroized = deallocated_bytes_were_zeroized && bytes[index] == 0;
        }
        ::operator delete(bytes);
    }

    friend bool operator==(const InspectingAllocator&, const InspectingAllocator&) noexcept {
        return true;
    }
};

bool vector_wiper_clears_owned_bytes_before_deallocation() {
    deallocated_bytes_were_zeroized = false;
    {
        std::vector<std::uint8_t, InspectingAllocator> bytes;
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
