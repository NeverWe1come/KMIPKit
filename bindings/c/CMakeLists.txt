cmake_minimum_required(VERSION 3.20)
project(kmipkit_c_consumer LANGUAGES C)

enable_testing()

set(KMIPKIT_TARGET_DIR "${CMAKE_CURRENT_LIST_DIR}/../../target/debug")
find_library(KMIPKIT_FFI_LIBRARY
    NAMES kmipkit_ffi
    PATHS "${KMIPKIT_TARGET_DIR}"
    NO_DEFAULT_PATH
    REQUIRED
)

add_executable(kmipkit_extension_registry_consumer tests/extension_registry.c)
target_compile_features(kmipkit_extension_registry_consumer PRIVATE c_std_11)
target_include_directories(kmipkit_extension_registry_consumer PRIVATE include)
target_link_libraries(kmipkit_extension_registry_consumer PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmipkit_extension_registry_consumer PROPERTIES
    BUILD_RPATH "${KMIPKIT_TARGET_DIR}"
)

add_executable(kmipkit_vendor_extension_example examples/vendor_extension.c)
target_compile_features(kmipkit_vendor_extension_example PRIVATE c_std_11)
target_include_directories(kmipkit_vendor_extension_example PRIVATE include)
target_link_libraries(kmipkit_vendor_extension_example PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmipkit_vendor_extension_example PROPERTIES
    BUILD_RPATH "${KMIPKIT_TARGET_DIR}"
)

add_test(
    NAME kmipkit_extension_registry_consumer
    COMMAND kmipkit_extension_registry_consumer
        "${CMAKE_CURRENT_LIST_DIR}/../../tests/fixtures/extensions/cases.json"
)

add_test(
    NAME kmipkit_vendor_extension_example
    COMMAND kmipkit_vendor_extension_example
)
