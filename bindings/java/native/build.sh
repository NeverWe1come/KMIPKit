#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_dir/../../.." && pwd)
java_compiler=$(command -v javac)
if [ -n "${JAVA_HOME:-}" ]; then
    java_home=$JAVA_HOME
else
    case "$(uname -s)" in
        Linux*)
            java_compiler=$(readlink -f -- "$java_compiler")
            java_home=$(CDPATH= cd -- "$(dirname -- "$(dirname -- "$java_compiler")")" && pwd)
            ;;
        Darwin*)
            java_home=$(/usr/libexec/java_home)
            ;;
        *)
            java_home=$(CDPATH= cd -- "$(dirname -- "$(dirname -- "$java_compiler")")" && pwd)
            ;;
    esac
fi
cargo_target_directory="$repository_root/target"
native_output="$script_dir/../target/native"
export CARGO_TARGET_DIR="$cargo_target_directory"
mkdir -p "$native_output"

zeroizing_bytes_test="$native_output/zeroizing_bytes_test"
"${CXX:-c++}" -std=c++17 -Wall -Wextra -Werror \
    "$script_dir/tests/zeroizing_bytes_test.cpp" -o "$zeroizing_bytes_test"
"$zeroizing_bytes_test"

client_batch_item_transfer_test="$native_output/client_batch_item_transfer_test"
"${CXX:-c++}" -std=c++17 -Wall -Wextra -Werror \
    "$script_dir/tests/client_batch_item_transfer_test.cpp" -o "$client_batch_item_transfer_test"
"$client_batch_item_transfer_test"

native_handle_transfer_test="$native_output/native_handle_transfer_test"
"${CXX:-c++}" -std=c++17 -Wall -Wextra -Werror \
    "$script_dir/tests/native_handle_transfer_test.cpp" -o "$native_handle_transfer_test"
"$native_handle_transfer_test"

response_identifier_test="$native_output/response_identifier_validation_test"
set -- -std=c++17 -Wall -Wextra -Werror
if [ "${KMIPKIT_JNI_COVERAGE:-}" = "llvm" ]; then
    set -- "$@" -fprofile-instr-generate -fcoverage-mapping
fi
"${CXX:-c++}" "$@" -I"$repository_root/tests/fixtures/api/generated" \
    "$script_dir/tests/response_identifier_validation_test.cpp" -o "$response_identifier_test"
"$response_identifier_test"

response_bridge_test="$native_output/response_bridge_test"
"${CXX:-c++}" "$@" -I"$repository_root/tests/fixtures/api/generated" \
    "$script_dir/tests/response_bridge_test.cpp" -o "$response_bridge_test"
"$response_bridge_test"

cargo build --locked -p kmipkit-ffi \
    --manifest-path "$repository_root/Cargo.toml"

case "$(uname -s)" in
    Linux*)
        rust_library="$cargo_target_directory/debug/libkmipkit_ffi.a"
        set -- -std=c++17 -fPIC -shared
        if [ "${KMIPKIT_JNI_COVERAGE:-}" = "llvm" ]; then
            set -- "$@" -fprofile-instr-generate -fcoverage-mapping
        fi
        "${CXX:-c++}" "$@" \
            -I"$java_home/include" -I"$java_home/include/linux" \
            -I"$repository_root/bindings/c/include" \
            -I"$repository_root/tests/fixtures/api/generated" \
            "$script_dir/kmipkit_jni.cpp" "$rust_library" \
            -ldl -lpthread -lm -o "$native_output/libkmipkit_jni.so"
        ;;
    Darwin*)
        rust_library="$cargo_target_directory/debug/libkmipkit_ffi.a"
        "${CXX:-c++}" -std=c++17 -dynamiclib \
            -I"$java_home/include" -I"$java_home/include/darwin" \
            -I"$repository_root/bindings/c/include" \
            -I"$repository_root/tests/fixtures/api/generated" \
            "$script_dir/kmipkit_jni.cpp" "$rust_library" \
            -framework Security -framework CoreFoundation -framework SystemConfiguration \
            -o "$native_output/libkmipkit_jni.dylib"
        ;;
    *)
        echo 'JNI bridge build supports Windows MSVC, Linux, and macOS.' >&2
        exit 2
        ;;
esac
