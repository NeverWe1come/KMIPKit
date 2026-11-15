#!/usr/bin/env sh
set -eu

consumer_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$consumer_root/../../.." && pwd)
target_dir="$repository_root/target/debug"
java_native="$repository_root/bindings/java/target/native"
fixture_root="$repository_root/bindings/python/tests/fixtures/login-mtls"
temporary_root=$(mktemp -d "${TMPDIR:-/tmp}/kmipkit-0053.XXXXXX")
ready_file="$temporary_root/ready.txt"
responder_pid_file="$temporary_root/responder.pid"
uv run --locked --python 3.12 --project "$consumer_root/python" python \
    "$consumer_root/local_ping_responder.py" \
    --ready-file "$ready_file" --pid-file "$responder_pid_file" \
    --fixture-dir "$fixture_root" --connections 8 &
responder_pid=$!
cleanup() {
    if [ -f "$responder_pid_file" ]; then
        fixture_pid=$(cat "$responder_pid_file")
        kill "$fixture_pid" 2>/dev/null || true
    fi
    kill "$responder_pid" 2>/dev/null || true
    wait "$responder_pid" 2>/dev/null || true
    rm -rf -- "$temporary_root"
}
trap cleanup EXIT INT TERM
attempt=0
while [ ! -f "$ready_file" ] && [ "$attempt" -lt 1800 ]; do
    if ! kill -0 "$responder_pid" 2>/dev/null; then
        echo "The loopback mTLS Ping fixture exited before becoming ready." >&2
        exit 1
    fi
    sleep 0.1
    attempt=$((attempt + 1))
done
[ -f "$ready_file" ] || { echo "The loopback mTLS Ping fixture did not become ready." >&2; exit 1; }
export KMIPKIT_CONSUMER_TEST_PORT="$(cat "$ready_file")"
export KMIPKIT_CONSUMER_TEST_FIXTURES="$fixture_root"

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
