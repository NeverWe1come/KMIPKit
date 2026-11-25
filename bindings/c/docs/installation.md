# C Binding Installation & Build

Run commands from the repository root. Source builds require the Rust toolchain
specified by [Cargo.toml](../../../Cargo.toml), a C11 compiler, CMake 3.20 or
newer, and the native build prerequisites for the Rust crypto dependencies
(pinned OpenSSL 3.5.9).

## 1. Build `kmipkit-ffi` with Pinned OpenSSL

```sh
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1
cargo build --locked -p kmipkit-ffi
cmake -S bindings/c -B build/c-consumer
cmake --build build/c-consumer --config Release
```

Cargo produces the native library under `target/debug/`. The CMake project
builds repository consumers and the example against that library.

To select a separately built Rust release library, pass `-DKMIPKIT_TARGET_DIR`:

```sh
cargo build --locked --release -p kmipkit-ffi
cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"
cmake --build build/c-release --config Release
```

## 2. Downstream Application Integration

The current CMake project in `bindings/c/` is a local consumer build (it does not
install a `find_package(KMIPKit)` config package; [package.toml](../package.toml)
records distribution metadata).

To integrate into an external C11 application, add `bindings/c/include` (canonical
header: [kmipkit.h](../include/kmipkit.h)) to your include path and link against
the compiled `kmipkit_ffi` library (`libkmipkit_ffi.so`, `libkmipkit_ffi.dylib`,
or `kmipkit_ffi.dll`). See [native linking](native-library.md) and
[testing](testing.md) for runtime resolution and verification.
