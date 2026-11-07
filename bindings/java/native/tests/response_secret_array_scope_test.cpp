#include "../response_secret_array_scope.hpp"

#include <cassert>
#include <cstddef>

namespace {

struct ArrayState {
    bool wiped = false;
    bool local_reference_deleted = false;
};

} // namespace

int main() {
    auto cleanup = [](ArrayState* value, bool wipe) noexcept {
        value->wiped = wipe;
        value->local_reference_deleted = true;
    };
    using Scope = kmipkit::jni::ResponseSecretArrayScope<
            ArrayState*, 4, decltype(cleanup)>;

    ArrayState first{};
    ArrayState second{};
    {
        Scope scope(cleanup);
        assert(scope.add(&first));
        assert(scope.add(&second));
    }
    assert(first.wiped && first.local_reference_deleted);
    assert(second.wiped && second.local_reference_deleted);

    ArrayState transferred{};
    {
        Scope scope(cleanup);
        assert(scope.add(&transferred));
        scope.transfer_to_java();
    }
    assert(!transferred.wiped && transferred.local_reference_deleted);

    ArrayState capacity_first{};
    ArrayState capacity_second{};
    ArrayState capacity_overflow{};
    std::size_t cleanup_order = 0;
    std::size_t overflow_cleanup_order = 0;
    std::size_t registered_cleanup_count = 0;
    auto capacity_cleanup = [&](ArrayState* value, bool wipe) noexcept {
        assert(wipe);
        value->wiped = wipe;
        value->local_reference_deleted = true;
        ++cleanup_order;
        if (value == &capacity_overflow) {
            overflow_cleanup_order = cleanup_order;
        } else {
            ++registered_cleanup_count;
        }
    };
    using CapacityScope = kmipkit::jni::ResponseSecretArrayScope<
            ArrayState*, 2, decltype(capacity_cleanup)>;
    {
        CapacityScope scope(capacity_cleanup);
        assert(scope.add(nullptr));
        assert(scope.add(&capacity_first));
        assert(scope.add(&capacity_second));
        assert(!scope.add(&capacity_overflow));
        assert(capacity_overflow.local_reference_deleted);
        assert(overflow_cleanup_order == 1);
        assert(registered_cleanup_count == 0);
    }
    assert(capacity_first.wiped && capacity_first.local_reference_deleted);
    assert(capacity_second.wiped && capacity_second.local_reference_deleted);
    assert(registered_cleanup_count == 2);
    assert(cleanup_order == 3);
}
