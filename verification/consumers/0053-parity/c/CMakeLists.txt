cmake_minimum_required(VERSION 3.20)
project(kmipkit_0053_c_consumer LANGUAGES C)

if(NOT DEFINED KMIPKIT_ROOT)
    message(FATAL_ERROR "KMIPKIT_ROOT must point to the KMIPKit checkout")
endif()
if(NOT DEFINED KMIPKIT_TARGET_DIR)
    set(KMIPKIT_TARGET_DIR "${KMIPKIT_ROOT}/target/debug")
endif()

find_library(KMIPKIT_FFI_LIBRARY NAMES kmipkit_ffi
    PATHS "${KMIPKIT_TARGET_DIR}" NO_DEFAULT_PATH REQUIRED)

add_executable(kmipkit_0053_c_consumer main.c)
target_compile_features(kmipkit_0053_c_consumer PRIVATE c_std_11)
target_include_directories(kmipkit_0053_c_consumer PRIVATE
    "${KMIPKIT_ROOT}/bindings/c/include")
target_link_libraries(kmipkit_0053_c_consumer PRIVATE "${KMIPKIT_FFI_LIBRARY}")
