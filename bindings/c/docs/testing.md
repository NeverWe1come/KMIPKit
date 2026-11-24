# Testing the C Bindings

Run commands from the repository root after following [installation](installation.md):

```sh
cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"
cmake --build build/c-release --config Release
ctest --test-dir build/c-release -C Release --output-on-failure
```

On Windows, first add the absolute `target/debug` (or `target/release`) directory
to the process `PATH` so test executables can load `kmipkit_ffi.dll`. This CMake
project registers the extension registry, vendor extension example, and named-value consumers.

## Client ABI Coverage Consumer

The optional client ABI coverage consumer requires the `coverage-c-consumer` Cargo feature:

```sh
cargo build --locked -p kmipkit-ffi --features coverage-c-consumer
cmake -S bindings/c -B build/c-coverage -DKMIPKIT_BUILD_COVERAGE_CONSUMER=ON
cmake --build build/c-coverage --config Release
ctest --test-dir build/c-coverage -C Release --output-on-failure
```

Additional checks live in [Rust FFI tests](../../../crates/kmipkit-ffi/tests/) and
the independent [C parity consumer](../../../verification/consumers/0053-parity/README.md),
with shared fixtures under [bindings/shared/fixtures](../../shared/fixtures/).
