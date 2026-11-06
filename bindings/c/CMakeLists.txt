cmake_minimum_required(VERSION 3.20)
project(kmipkit_c_consumer LANGUAGES C)

enable_testing()

set(KMIPKIT_TARGET_DIR "${CMAKE_CURRENT_LIST_DIR}/../../target/debug" CACHE PATH
    "Directory containing the KMIPKit FFI library selected for this consumer")
find_library(KMIPKIT_FFI_LIBRARY
    NAMES kmipkit_ffi
    PATHS "${KMIPKIT_TARGET_DIR}"
    NO_DEFAULT_PATH
    REQUIRED
)

add_executable(kmipkit_extension_registry_consumer tests/extension_registry.c)
target_compile_features(kmipkit_extension_registry_consumer PRIVATE c_std_11)
target_include_directories(kmipkit_extension_registry_consumer PRIVATE
    include
    "${CMAKE_CURRENT_LIST_DIR}/../../tests/fixtures/api/generated"
)
target_link_libraries(kmipkit_extension_registry_consumer PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmipkit_extension_registry_consumer PROPERTIES
    BUILD_RPATH "${KMIPKIT_TARGET_DIR}"
)

add_library(kmipkit_register_opaque_context_consumer OBJECT
    tests/register_opaque_context.c
)
target_compile_features(kmipkit_register_opaque_context_consumer PRIVATE c_std_11)
target_include_directories(kmipkit_register_opaque_context_consumer PRIVATE include)

add_executable(kmipkit_client_abi_consumer tests/client_abi_consumer.c)
target_compile_features(kmipkit_client_abi_consumer PRIVATE c_std_11)
target_include_directories(kmipkit_client_abi_consumer PRIVATE include)
target_link_libraries(kmipkit_client_abi_consumer PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmipkit_client_abi_consumer PROPERTIES
    BUILD_RPATH "${KMIPKIT_TARGET_DIR}"
)

add_executable(kmipkit_vendor_extension_example examples/vendor_extension.c)
target_compile_features(kmipkit_vendor_extension_example PRIVATE c_std_11)
target_include_directories(kmipkit_vendor_extension_example PRIVATE
    include
    "${CMAKE_CURRENT_LIST_DIR}/../../tests/fixtures/api/generated"
)
target_link_libraries(kmipkit_vendor_extension_example PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmipkit_vendor_extension_example PROPERTIES
    BUILD_RPATH "${KMIPKIT_TARGET_DIR}"
)

add_test(
    NAME kmipkit_extension_registry_consumer
    COMMAND kmipkit_extension_registry_consumer
)

add_test(
    NAME kmipkit_client_abi_consumer
    COMMAND kmipkit_client_abi_consumer high-level typed-protocol generic-ttlv
)

add_test(
    NAME kmipkit_vendor_extension_example
    COMMAND kmipkit_vendor_extension_example
)
