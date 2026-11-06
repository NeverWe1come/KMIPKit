#!/usr/bin/env sh
set -eu

consumer_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$consumer_root/../../.." && pwd)
target_dir="$repository_root/target/debug"
java_native="$repository_root/bindings/java/target/native"

cargo +1.94 run --locked --manifest-path "$consumer_root/rust/Cargo.toml"
cargo build --locked -p kmipkit-ffi

cc -std=c11 -Wall -Wextra -Werror \
    -I "$repository_root/bindings/c/include" \
    "$consumer_root/c/main.c" \
    -L "$target_dir" -Wl,-rpath,"$target_dir" -lkmipkit_ffi \
    -o "$consumer_root/c/kmipkit_0053_c_consumer"
"$consumer_root/c/kmipkit_0053_c_consumer"

sh "$repository_root/bindings/java/native/build.sh"
mvn -q -f "$repository_root/bindings/java/pom.xml" \
    -DskipTests -Dexec.skip=true package
mvn -q install:install-file \
    "-Dfile=$repository_root/bindings/java/target/kmipkit-java-1.0.0.jar" \
    "-DpomFile=$repository_root/bindings/java/pom.xml"

case "$(uname -s)" in
    Darwin) jni_library="$java_native/libkmipkit_jni.dylib" ;;
    *) jni_library="$java_native/libkmipkit_jni.so" ;;
esac
mvn -q -f "$consumer_root/java/pom.xml" \
    "-Dkmipkit.nativeLibrary=$jni_library" compile exec:java

uv run --locked --python 3.12 --project "$consumer_root/python" \
    python "$consumer_root/python/main.py"
