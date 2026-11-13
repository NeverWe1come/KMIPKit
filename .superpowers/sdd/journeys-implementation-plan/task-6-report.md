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

## Green

- The focused Rust command above, without `--no-run`, passes on 1.100.0:
  `c_api_coverage`: 56 passed; `operation_parity_0053`: 1 passed. Output:
  `target/ci-110-layout-green.log`.
- Remove the redundant package-native link directives and layout-derived
  package library search. Cargo links the existing rlib; every `extern "C"`
  declaration, compiled C consumer, ABI assertion, and test remains enabled.
- Keep the coverage collector's complete four-object export. A no-run Cargo
  invocation under `cargo llvm-cov show-env` reports exact instrumented
  compiler-artifact paths. Missing/ambiguous artifacts fail the collection;
  there is no fallback that could publish a partial report.
- `python3 -B -m unittest scripts.tests.test_ffi_coverage_objects scripts.tests.test_workflow -v`:
  98 passed. Output: `target/ci-110-python-green.log`.
- `cargo +1.100.0 fmt --all --check`, `bash -n scripts/collect_ffi_coverage.sh`,
  and `git diff --check`: exit 0.
- Workspace Clippy and real instrumented FFI coverage validation are in
  progress; their final outcomes are recorded below before closeout.

### Self-review: retain parity coverage after removing the dynamic link

The parity test's counters now live in its Cargo-linked executable. Keep all
four original coverage export objects and additionally export
`operation_parity_0053`; otherwise its executed C operation routes would stop
contributing to this focused report. The enhanced resolver regression fails
before that addition: 4 tests, 3 assertion failures (two layout cases omit the
parity object; the missing-parity negative case incorrectly succeeds). Output:
`target/ci-110-parity-objects-red.log`.
