# C testing

Run commands from the repository root after following [installation](installation.md):

```sh
ctest --test-dir build/c-consumer -C Release --output-on-failure
```

On Windows, first add the absolute `target/debug` directory to the process
`PATH` so test executables can load the DLL. This CMake project registers the
extension registry, vendor extension example, and named-value consumers.

The optional client ABI coverage consumer needs the corresponding Rust feature:

```sh
cargo build --locked -p kmipkit-ffi --features coverage-c-consumer
cmake -S bindings/c -B build/c-coverage -DKMIPKIT_BUILD_COVERAGE_CONSUMER=ON
cmake --build build/c-coverage --config Release
ctest --test-dir build/c-coverage -C Release --output-on-failure
```

The [Rust FFI tests](../../../crates/kmipkit-ffi/tests/) and the independent
[C parity consumer](../../../verification/consumers/0053-parity/README.md)
provide additional checks beyond this CMake project. Shared generated inputs
live in [bindings/shared/fixtures](../../shared/fixtures/).

See the [verification index](../../../verification/README.md) for sanitizer,
coverage, API, and interoperability tooling. Record the commands and outcomes;
these local consumers do not establish server interoperability by themselves.
