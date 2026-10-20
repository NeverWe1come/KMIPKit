#include "../response_array_conversion.hpp"

#include <array>
#include <cassert>
#include <cstddef>

namespace {

struct Array final {};

} // namespace

int main() {
    Array partially_copied{};
    bool copy_exception_pending = true;
    bool partially_copied_array_wiped = false;
    Array* discarded = kmipkit::jni::discard_response_array_after_copy_failure(
            &partially_copied,
            [&] { return copy_exception_pending; },
            [&](Array* value) noexcept {
                assert(value == &partially_copied);
                partially_copied_array_wiped = true;
                copy_exception_pending = true;
            });
    assert(discarded == nullptr);
    assert(partially_copied_array_wiped);
    assert(copy_exception_pending);

    std::array<Array*, 3> output{};
    std::array<Array, 3> input{};
    std::size_t cleaned_count = 0;
    std::size_t conversion_count = 0;
    bool pending_exception = false;
    bool cleaned_value = false;

    auto cleanup = [&](Array*, bool wipe) noexcept {
        ++cleaned_count;
        cleaned_value = wipe;
    };
    using Scope = kmipkit::jni::ResponseSecretArrayScope<Array*, 3, decltype(cleanup)>;
    {
        Scope sensitive_arrays(cleanup);
        const bool converted = kmipkit::jni::convert_response_arrays(
                output,
                [&](std::size_t index) {
                    ++conversion_count;
                    if (index == 0) pending_exception = true;
                    return &input[index];
                },
                [&] { return pending_exception; },
                sensitive_arrays);

        assert(!converted);
        assert(conversion_count == 1);
        assert(output[0] == &input[0]);
        assert(cleaned_count == 0);
    }
    assert(cleaned_count == 1);
    assert(cleaned_value);
}
