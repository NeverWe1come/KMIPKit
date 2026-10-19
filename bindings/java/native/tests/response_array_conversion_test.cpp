#include "../response_array_conversion.hpp"

#include <array>
#include <cassert>
#include <cstddef>

namespace {

struct Array final {};

} // namespace

int main() {
    std::array<Array*, 3> output{};
    std::array<Array, 3> input{};
    std::array<Array*, 3> registered{};
    std::size_t registered_count = 0;
    std::size_t conversion_count = 0;
    bool pending_exception = false;

    const bool converted = kmipkit::jni::convert_response_arrays(
            output,
            [&](std::size_t index) {
                ++conversion_count;
                if (index == 0) pending_exception = true;
                return &input[index];
            },
            [&] { return pending_exception; },
            [&](Array* value) {
                registered[registered_count++] = value;
                return true;
            });

    assert(!converted);
    assert(conversion_count == 1);
    assert(registered_count == 1);
    assert(output[0] == &input[0]);
    assert(registered[0] == &input[0]);
}
