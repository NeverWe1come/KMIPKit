# Task 6: Native C consumer linker repair

## Scope and investigation

Approved KMIPKIT-0093-FR-013 / F001 journey closeout; preserve all C consumer,
ABI, sanitizer, and coverage checks. Starting head: `cc8747256bc5c04137ab376de2f948c9535f1800`.

Hosted Linux core job `131573740597` and coverage job `131573740610` use
Rust/Cargo 1.100.0. Local `stable` is 1.99.1, explaining why the earlier clean
workspace build passed. Cargo 1.100 places intermediate artifacts in
`debug/build/<package>/<hash>/out`; `OUT_DIR/../../../deps` points at
`debug/build/deps` and cannot locate the package's native library. The explicit
`#[link(name = "kmipkit_ffi")]` directives redundantly bypass Cargo's existing
Rust library dependency. The C coverage collector also assumes `debug/deps`.

## Red

Command (exit 101):

```sh
CARGO_TARGET_DIR=target/ci-110-layout TMPDIR="$PWD/target/tmp-kmipkit-0110" cargo +1.100.0 test -p kmipkit-ffi --all-features --locked --test operation_parity_0053 --test c_api_coverage --no-run
```

Both existing integration tests fail with `rust-lld: error: unable to find
library -lkmipkit_ffi`. Full local output: `target/ci-110-layout-red.log`.
The library exists at
`target/ci-110-layout/debug/build/kmipkit-ffi/40dc6e8db935a7b9/out/libkmipkit_ffi.so`;
there is no `debug/deps` directory. This reproduces the hosted failure with the
same toolchain, before any implementation changes.

### Coverage object regression

`python3 -B -m unittest scripts.tests.test_ffi_coverage_objects -v` fails before
adding the artifact resolver (3 tests, 6 assertion failures: resolver absent).
The regression requires all four existing export inputs (DSO, static library,
FFI unit test, C consumer test) for both Cargo layouts, including spaces in
paths, and fails closed when any required file is missing.
