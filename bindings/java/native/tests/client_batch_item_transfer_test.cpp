#include "../client_batch_item_transfer_guard.hpp"

#include <iostream>
#include <new>

namespace {

int released_handle = 0;
int release_count = 0;

void release_handle(int handle) noexcept {
    released_handle = handle;
    ++release_count;
}

bool preparation_failure_releases_transferred_handle() {
    released_handle = 0;
    release_count = 0;
    try {
        kmipkit::jni::ClientBatchItemTransferGuard<int, decltype(&release_handle)> guard(
            41,
            &release_handle);
        throw std::bad_alloc();
    } catch (const std::bad_alloc&) {
    }
    return released_handle == 41 && release_count == 1;
}

bool consumer_transfer_disarms_guard() {
    released_handle = 0;
    release_count = 0;
    int consumed_handle = 0;
    {
        kmipkit::jni::ClientBatchItemTransferGuard<int, decltype(&release_handle)> guard(
            73,
            &release_handle);
        consumed_handle = guard.transfer_to_consumer();
    }
    release_handle(consumed_handle);
    return released_handle == 73 && release_count == 1;
}

} // namespace

int main() {
    if (!preparation_failure_releases_transferred_handle()) {
        std::cerr << "pre-consumer JNI failure leaked the transferred client batch item handle\n";
        return 1;
    }
    if (!consumer_transfer_disarms_guard()) {
        std::cerr << "JNI transfer guard released a handle after ownership reached the consumer\n";
        return 1;
    }
    return 0;
}
