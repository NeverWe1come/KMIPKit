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
get_filename_component(KMIPKIT_C_INSTALL_DIR_ABSOLUTE "${KMIPKIT_C_INSTALL_DIR}"
    ABSOLUTE BASE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(KMIPKIT_LIBRARY_COMMIT "8281fef2e9ca211fde1e1d8e21525a672287eead")
# The selected install is authoritative even when reusing an older build cache.
unset(KMIPKIT_FFI_LIBRARY CACHE)
find_library(KMIPKIT_FFI_LIBRARY
    NAMES kmipkit_ffi
    PATHS "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib"
    NO_DEFAULT_PATH
)
if(NOT KMIPKIT_FFI_LIBRARY)
    message(FATAL_ERROR
        "KMIPKit FFI library not found in ${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")
endif()
file(SHA256 "${KMIPKIT_FFI_LIBRARY}" KMIPKIT_C_ARTIFACT_SHA256)
file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/../shared/catalog/kmipkit-1.0-test-catalog.json"
    KMIPKIT_C_CATALOG_SHA256)
set(KMIPKIT_C_INCLUDE_DIR "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/include")

set(VALIDATOR_SOURCES
    src/c_validator.c
    src/live_driver.c
    src/json.c
)

option(KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE
    "Whether the pinned KMIPKit C artifact includes the KMIP 2.0 client-version fix"
    ON)

add_executable(kmip_validator_c
    src/main.c
    ${VALIDATOR_SOURCES}
)
target_include_directories(kmip_validator_c PRIVATE "${KMIPKIT_C_INCLUDE_DIR}")
target_link_libraries(kmip_validator_c PRIVATE "${KMIPKIT_FFI_LIBRARY}")
target_compile_definitions(kmip_validator_c PRIVATE
    KMIPKIT_LIBRARY_COMMIT="${KMIPKIT_LIBRARY_COMMIT}"
    KMIPKIT_C_INSTALL_DIR="${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}"
    KMIPKIT_C_ARTIFACT_SHA256="${KMIPKIT_C_ARTIFACT_SHA256}"
    KMIPKIT_C_CATALOG_SHA256="${KMIPKIT_C_CATALOG_SHA256}"
    KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE=$<BOOL:${KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE}>
)
set_target_properties(kmip_validator_c PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")


# Linux CI records the actual installed public ABI calls, never request bodies.
# Generate from the installed header so signatures cannot drift from the pin.
if(UNIX AND NOT APPLE)
    find_package(Python3 COMPONENTS Interpreter REQUIRED)
    set(ROUTE_MANIFEST "${CMAKE_CURRENT_SOURCE_DIR}/../shared/coverage/route-capabilities.json")
    set(ROUTE_WRAPPERS "${CMAKE_CURRENT_BINARY_DIR}/route_wrappers.c")
    execute_process(COMMAND "${Python3_EXECUTABLE}" -c
        "import json; m=json.load(open(r'${ROUTE_MANIFEST}')); print(';'.join(sorted({r['public_api_symbol'] for r in m['routes']+m['execution_routes'] if r['language']=='c'})))"
        OUTPUT_VARIABLE ROUTE_SYMBOLS OUTPUT_STRIP_TRAILING_WHITESPACE
        RESULT_VARIABLE ROUTE_SYMBOL_STATUS)
    if(NOT ROUTE_SYMBOL_STATUS EQUAL 0)
        message(FATAL_ERROR "Could not load public C route symbols")
    endif()
    add_custom_command(OUTPUT "${ROUTE_WRAPPERS}"
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tools/generate_route_wrappers.py"
            --header "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/include/kmipkit.h"
            --manifest "${ROUTE_MANIFEST}" --output "${ROUTE_WRAPPERS}"
        DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/tools/generate_route_wrappers.py"
            "${ROUTE_MANIFEST}" "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/include/kmipkit.h")
    target_sources(kmip_validator_c PRIVATE "${ROUTE_WRAPPERS}")
    foreach(ROUTE_SYMBOL IN LISTS ROUTE_SYMBOLS)
        target_link_options(kmip_validator_c PRIVATE "-Wl,--wrap=${ROUTE_SYMBOL}")
    endforeach()
endif()

enable_testing()

add_executable(test_c_validator
    tests/test_c_validator.c
    ${VALIDATOR_SOURCES}
)
target_include_directories(test_c_validator PRIVATE "${KMIPKIT_C_INCLUDE_DIR}")
target_include_directories(test_c_validator PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/src")
target_link_libraries(test_c_validator PRIVATE "${KMIPKIT_FFI_LIBRARY}")
target_compile_definitions(test_c_validator PRIVATE
    KMIPKIT_LIBRARY_COMMIT="${KMIPKIT_LIBRARY_COMMIT}"
    KMIPKIT_C_INSTALL_DIR="${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}"
    KMIPKIT_C_ARTIFACT_SHA256="${KMIPKIT_C_ARTIFACT_SHA256}"
    KMIPKIT_C_CATALOG_SHA256="${KMIPKIT_C_CATALOG_SHA256}"
    KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE=$<BOOL:${KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE}>
)
set_target_properties(test_c_validator PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")

add_test(NAME test_c_validator COMMAND test_c_validator)
set_tests_properties(test_c_validator PROPERTIES
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)
add_test(
    NAME c_validator_plan_2689_cases
    COMMAND kmip_validator_c plan
        --connection "${CMAKE_CURRENT_SOURCE_DIR}/../shared/profiles/connection.example.json"
        --capabilities "${CMAKE_CURRENT_SOURCE_DIR}/../shared/profiles/generic-kmip-2.1.json"
        --catalog "${CMAKE_CURRENT_SOURCE_DIR}/../shared/catalog/kmipkit-1.0-test-catalog.json"
)
set_tests_properties(c_validator_plan_2689_cases PROPERTIES
    PASS_REGULAR_EXPRESSION "catalog_cases=2689"
)

find_package(Python3 COMPONENTS Interpreter QUIET)
find_program(OPENSSL_EXECUTABLE openssl)
if(Python3_Interpreter_FOUND AND OPENSSL_EXECUTABLE)
    add_test(
        NAME c_validator_route_matrix_contracts
        COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_route_matrix.py"
    )
    add_test(
        NAME c_validator_artifact_checkout_history
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_artifact_checkout.py"
    )
    add_test(
        NAME c_validator_controlled_tls_peer
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/controlled_kms_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${OPENSSL_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/../.."
    )
    set_tests_properties(c_validator_controlled_tls_peer PROPERTIES TIMEOUT 60)
    add_test(
        NAME c_validator_controlled_tls_peer_kmip20
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/controlled_kms_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${OPENSSL_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/../.."
            "2.0"
    )
    set_tests_properties(c_validator_controlled_tls_peer_kmip20 PROPERTIES TIMEOUT 60)
    add_test(
        NAME c_validator_offline_generic_route
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/offline_generic_route_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${CMAKE_CURRENT_SOURCE_DIR}/../.."
    )
    set_tests_properties(c_validator_offline_generic_route PROPERTIES TIMEOUT 20)
endif()
