cmake_minimum_required(VERSION 3.16)
project(kmip_validator_c C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -pedantic")

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

set(KMIPKIT_C_INSTALL_DIR "" CACHE PATH
    "Directory containing the installed KMIPKit C library and public headers")
if(NOT KMIPKIT_C_INSTALL_DIR)
    message(FATAL_ERROR
        "KMIPKIT_C_INSTALL_DIR is required; run tools/install_artifact.py first")
endif()
find_library(KMIPKIT_FFI_LIBRARY
    NAMES kmipkit_ffi
    PATHS "${KMIPKIT_C_INSTALL_DIR}/lib"
    NO_DEFAULT_PATH
    REQUIRED
)
set(KMIPKIT_C_INCLUDE_DIR "${KMIPKIT_C_INSTALL_DIR}/include")

set(VALIDATOR_SOURCES
    src/c_validator.c
)

add_executable(kmip_validator_c
    src/main.c
    ${VALIDATOR_SOURCES}
)
target_include_directories(kmip_validator_c PRIVATE "${KMIPKIT_C_INCLUDE_DIR}")
target_link_libraries(kmip_validator_c PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(kmip_validator_c PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR}/lib")

enable_testing()

add_executable(test_c_validator
    tests/test_c_validator.c
    ${VALIDATOR_SOURCES}
)
target_include_directories(test_c_validator PRIVATE "${KMIPKIT_C_INCLUDE_DIR}")
target_link_libraries(test_c_validator PRIVATE "${KMIPKIT_FFI_LIBRARY}")
set_target_properties(test_c_validator PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR}/lib")

add_test(NAME test_c_validator COMMAND test_c_validator)
set_tests_properties(test_c_validator PROPERTIES
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)
