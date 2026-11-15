#ifndef KMIPKIT_RESPONSE_ARRAY_CONVERSION_HPP
#define KMIPKIT_RESPONSE_ARRAY_CONVERSION_HPP

#include "response_secret_array_scope.hpp"

#include <array>
#include <cstddef>

namespace kmipkit::jni {

template <typename Array, typename HasPendingException, typename Cleanup>
Array discard_response_array_after_copy_failure(
        Array array,
        HasPendingException&& has_pending_exception,
        Cleanup&& cleanup) {
    if (array == nullptr || !has_pending_exception()) return array;
    cleanup(array);
    return nullptr;
}

template <typename Array, std::size_t Count, typename Convert,
        typename HasPendingException, typename Cleanup>
bool convert_response_arrays(
        std::array<Array, Count>& output,
        Convert&& convert,
        HasPendingException&& has_pending_exception,
        ResponseSecretArrayScope<Array, Count, Cleanup>& sensitive_arrays) {
    for (std::size_t index = 0; index < Count; ++index) {
        output[index] = convert(index);
        if (!sensitive_arrays.add(output[index])) return false;
        if (has_pending_exception()) return false;
    }
    return true;
}

} // namespace kmipkit::jni

#endif
