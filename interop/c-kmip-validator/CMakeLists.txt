cmake_minimum_required(VERSION 3.16)
project(kmip_validator_c C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -pedantic")

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

set(VALIDATOR_SOURCES
    src/c_validator.c
)

add_executable(kmip_validator_c
    src/main.c
    ${VALIDATOR_SOURCES}
)

enable_testing()

add_executable(test_c_validator
    tests/test_c_validator.c
    ${VALIDATOR_SOURCES}
)

add_test(NAME test_c_validator COMMAND test_c_validator)
