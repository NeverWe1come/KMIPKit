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
    ABSOLUTE BASE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../..")
file(READ "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/library-commit" KMIPKIT_LIBRARY_COMMIT)
string(STRIP "${KMIPKIT_LIBRARY_COMMIT}" KMIPKIT_LIBRARY_COMMIT)
string(LENGTH "${KMIPKIT_LIBRARY_COMMIT}" KMIPKIT_LIBRARY_COMMIT_LENGTH)
if(NOT KMIPKIT_LIBRARY_COMMIT_LENGTH EQUAL 40 OR NOT KMIPKIT_LIBRARY_COMMIT MATCHES "^[0-9a-f]+$")
    message(FATAL_ERROR "Invalid installed KMIPKit source commit")
endif()
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
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/../shared/catalog/kmipkit-2x-matrix-catalog.json")
file(SHA256 "${KMIPKIT_FFI_LIBRARY}" KMIPKIT_C_ARTIFACT_SHA256)
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/../shared/catalog/kmipkit-2x-matrix-catalog.json")
file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/../shared/catalog/kmipkit-2x-matrix-catalog.json"
    KMIPKIT_C_CATALOG_SHA256)
set(KMIPKIT_C_INCLUDE_DIR "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/include")
get_filename_component(KMIPKIT_C_CERTIFICATE_FIXTURE
    "${CMAKE_CURRENT_SOURCE_DIR}/../rust-kmip-validator/tests/fixtures/certificate.der" ABSOLUTE)

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
    KMIPKIT_C_CERTIFICATE_FIXTURE_PATH="${KMIPKIT_C_CERTIFICATE_FIXTURE}"
    KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE=$<BOOL:${KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE}>
)
set_target_properties(kmip_validator_c PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")


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
    KMIPKIT_C_CERTIFICATE_FIXTURE_PATH="${KMIPKIT_C_CERTIFICATE_FIXTURE}"
    KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE=$<BOOL:${KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE}>
)
set_target_properties(test_c_validator PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")

add_test(NAME test_c_validator COMMAND test_c_validator)
set_tests_properties(test_c_validator PROPERTIES
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)
find_package(Python3 COMPONENTS Interpreter QUIET)
find_program(OPENSSL_EXECUTABLE openssl)
if(Python3_Interpreter_FOUND AND OPENSSL_EXECUTABLE)
    add_test(
        NAME c_validator_controlled_tls_peer
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/controlled_kms_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${OPENSSL_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/../../.."
    )
    set_tests_properties(c_validator_controlled_tls_peer PROPERTIES TIMEOUT 60)
    add_test(
        NAME c_validator_controlled_tls_peer_kmip20
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/controlled_kms_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${OPENSSL_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/../../.."
            "2.0"
    )
    set_tests_properties(c_validator_controlled_tls_peer_kmip20 PROPERTIES TIMEOUT 60)
    add_test(
        NAME c_validator_offline_generic_route
        COMMAND "${Python3_EXECUTABLE}"
            "${CMAKE_CURRENT_SOURCE_DIR}/tests/offline_generic_route_test.py"
            "$<TARGET_FILE:kmip_validator_c>"
            "${CMAKE_CURRENT_SOURCE_DIR}/../../.."
    )
    set_tests_properties(c_validator_offline_generic_route PROPERTIES TIMEOUT 20)
endif()

add_executable(kmip30_c_constructors src/draft_constructors.c)
target_include_directories(kmip30_c_constructors PRIVATE "${KMIPKIT_C_INCLUDE_DIR}")
target_link_libraries(kmip30_c_constructors PRIVATE "${KMIPKIT_FFI_LIBRARY}")
target_compile_definitions(kmip30_c_constructors PRIVATE
    KMIPKIT_LIBRARY_COMMIT="${KMIPKIT_LIBRARY_COMMIT}"
    KMIPKIT_C_ARTIFACT_SHA256="${KMIPKIT_C_ARTIFACT_SHA256}"
)
set_target_properties(kmip30_c_constructors PROPERTIES BUILD_RPATH "${KMIPKIT_C_INSTALL_DIR_ABSOLUTE}/lib")
add_test(NAME c_validator_draft_constructors COMMAND kmip30_c_constructors)
