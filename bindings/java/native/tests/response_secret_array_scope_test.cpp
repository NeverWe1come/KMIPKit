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
}
