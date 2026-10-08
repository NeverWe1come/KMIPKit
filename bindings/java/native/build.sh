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
native_output="$script_dir/../target/native"
mkdir -p "$native_output"

zeroizing_bytes_test="$native_output/zeroizing_bytes_test"
"${CXX:-c++}" -std=c++17 -Wall -Wextra -Werror \
    "$script_dir/tests/zeroizing_bytes_test.cpp" -o "$zeroizing_bytes_test"
"$zeroizing_bytes_test"

cargo build --locked -p kmipkit-ffi --manifest-path "$repository_root/Cargo.toml"

case "$(uname -s)" in
    Linux*)
        rust_library="$repository_root/target/debug/libkmipkit_ffi.a"
        set -- -std=c++17 -fPIC -shared
        if [ "${KMIPKIT_JNI_COVERAGE:-}" = "llvm" ]; then
            set -- "$@" -fprofile-instr-generate -fcoverage-mapping
        fi
        "${CXX:-c++}" "$@" \
            -I"$java_home/include" -I"$java_home/include/linux" \
            -I"$repository_root/bindings/c/include" \
            "$script_dir/kmipkit_jni.cpp" "$rust_library" \
            -ldl -lpthread -lm -o "$native_output/libkmipkit_jni.so"
        ;;
    Darwin*)
        rust_library="$repository_root/target/debug/libkmipkit_ffi.a"
        "${CXX:-c++}" -std=c++17 -dynamiclib \
            -I"$java_home/include" -I"$java_home/include/darwin" \
            -I"$repository_root/bindings/c/include" \
            "$script_dir/kmipkit_jni.cpp" "$rust_library" \
            -framework Security -framework CoreFoundation -framework SystemConfiguration \
            -o "$native_output/libkmipkit_jni.dylib"
        ;;
    *)
        echo 'JNI bridge build supports Windows MSVC, Linux, and macOS.' >&2
        exit 2
        ;;
esac
