#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_dir/../../.." && pwd)
java_compiler=$(command -v javac)
java_home=$(CDPATH= cd -- "$(dirname -- "$(dirname -- "$java_compiler")")" && pwd)
native_output="$script_dir/../target/native"
mkdir -p "$native_output"

cargo build --locked -p kmipkit-ffi --manifest-path "$repository_root/Cargo.toml"

case "$(uname -s)" in
    Linux*)
        rust_library="$repository_root/target/debug/libkmipkit_ffi.a"
        "${CXX:-c++}" -std=c++17 -fPIC -shared \
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
