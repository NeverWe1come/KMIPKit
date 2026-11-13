# KMIPKIT-0087 implementation evidence

## Artifact under test

- Release ref: `release/1.0.0`
- Exact library commit: `9a16cc90a12205101d22a38501a5e74c665df4dc`
- Locally installed wheel SHA-256: `0df48056bc28fad41d4c8bd6343155756027f707422fa6c446df9d34963cbeff`
- Runtime: Python 3.12.13; Rust 1.94 build toolchain
- Current route-manifest SHA-256: `5fdddfb8f004cd858b18f27b432fe6b802d2ac7d969d49abeb5db0b265738116`

The validator imports the installed wheel. It does not import the library from
`bindings/python` in the checkout.

## Local verification

From `interop/python-kmip-validator` after installing the pinned artifact:

```sh
PYTHONPATH=src python tools/verify_artifact.py
PYTHONPATH=src python -B -m unittest discover -s tests -p 'test_*.py' -v
PYTHONPATH=src python -m kmipkit_validator matrix --output-dir build/matrix-current
PYTHONPATH=src python ../shared/tools/validate_route_evidence.py \
  --report build/matrix-current/route-execution-evidence.json \
  --artifact-sha256 0df48056bc28fad41d4c8bd6343155756027f707422fa6c446df9d34963cbeff
```

Observed on 2026-11-13:

- `verify_artifact.py` accepted the installed wheel, exact release commit and digest.
- The Python 3.12 unit suite passed all 129 tests in 348.501 seconds.
- Repository requirement-traceability contracts passed 2 tests; feature
  traceability contracts passed 31 tests.
- The installed-package matrix completed all 74,648/74,648 route cells, with
  zero failed or pending cells. The shared route-evidence validator accepted
  all 74,648 invocations with zero duplicate events.
- Matrix run ID: `python-matrix-4691e61d-1739-4ee7-8991-9a4dce2e4fc1`.
  Evidence and closure are in `build/matrix-current/`; the report contains
  normalized tree and deterministic wire hashes, not KMIP bodies.
- The matrix exercises KMIP 2.0 and 2.1, TTLV/JSON/XML, raw TLS/HTTPS, typed
  and high-level public routes, generic TTLV construction and decoding,
  ordered batches, response accessors, stateful workflows and asynchronous
  routes using the controlled peer.

The complete matrix means every route cell in the current shared route
manifest was exercised through the installed wheel. It does not claim that a
KMIP server supports every operation. Public APIs that the pinned library does
not expose, including generic outbound TTLV requests and server-initiated
request handling, remain documented API boundaries and are not reported as
successful invocations.

## Pull-request CI evidence

At PR #173 head `264ba33bc0c04aa4285a2908a46ab097fd7802d8`, 49 checks succeeded
and none failed. Six checks were skipped by workflow conditions: the C, Java
and Rust interop consumer jobs were outside this PR's path scope; nightly and
active-release checks require a scheduled event; and the protected-release
evidence job requires a protected `release/1.0.0` push. The successful Python
consumer job built and installed the exact pinned release wheel and ran the
consumer contract against controlled TLS fixtures.

No Cosmian, CipherTrust or ESKM endpoint was available. These fixture results
are not vendor interoperability evidence. Cross-language parity still
requires the joint run against matching artifacts, catalogs and server
configuration.

## Red: clean Rust 1.100 workspace test link failure

On the Python validator branch at `a860b0c810e73818e7a30b7a34b32cb3d0fa1ce2`,
a clean Rust 1.100.0 workspace test run reproduces the cross-platform FFI
consumer link failure:

```sh
CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<empty-target> \
  cargo +1.100.0 test --workspace --all-features --locked
```

The focused failure is `unable to find library -lkmipkit_ffi` for both
`operation_parity_0053` and `c_api_coverage`. The same clean workspace command
passes under Rust 1.99.1. Rust 1.100.0 places the crate's `.rlib`, `.a`, and
`.so` outputs under `debug/build/kmipkit-ffi/<library-hash>/out`; the separate
integration test executables are emitted under different hash directories.
Their explicit `#[link(name = "kmipkit_ffi")]` declarations do not receive a
native search path to the library output directory. This is a test-link setup
failure, not a KMIP route or validator failure.

## Green: resolve C consumer symbols from the Rust crate dependency

The integration tests now declare the ABI functions without an explicit
native-library link attribute. The operation parity test uses the crate's
opaque batch handle type, and the C API coverage test already imports public
FFI types from the same crate. This makes Cargo link the FFI implementation
through its Rust crate dependency, while the compiled C consumers still invoke
the exported `extern "C"` symbols.

On Rust 1.100.0, both focused tests passed after cleaning the `kmipkit-ffi`
package artifacts:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=<target> \
  cargo +1.100.0 test -p kmipkit-ffi --features coverage-c-consumer \
    --test operation_parity_0053 --test c_api_coverage --locked
```

The result was 1/1 operation parity test and 50/50 C API coverage tests. The
full `cargo +1.100.0 test --workspace --all-features --locked` command then
passed with the FFI package artifacts cleaned and rebuilt.

## Refactor: centralize Rust-backed ABI declarations

The `extern "C"` declarations used by both integration tests now live in
`crates/kmipkit-ffi/tests/support/ffi_link_exports.rs`. This keeps the
Rust-crate linkage assumption and pointer types in one test-only module. After
the extraction, `cargo +1.100.0 fmt --all --check` and the same two focused
Rust 1.100.0 tests passed again (50 C API coverage tests and one operation
parity test).

The post-refactor Rust 1.100.0 workspace and coverage checks also passed:

```sh
cargo +1.100.0 fmt --all --check
cargo +1.100.0 test --workspace --all-features --locked
cargo +1.100.0 llvm-cov --workspace --all-features --locked --json \
  --output-path target-rust100-red/coverage-raw.json
python3 scripts/coverage_gate.py normalize --workspace . \
  --input target-rust100-red/coverage-raw.json \
  --output target-rust100-red/coverage.json
```

The workspace run rebuilt the cleaned FFI package, the LLVM coverage report
was generated from its own clean instrumented target, and normalization
succeeded. Cross-platform confirmation remains the PR CI run on Linux,
Windows, and macOS.

## Red: C FFI coverage collector assumes the old Cargo output directory

The `Coverage / ubuntu` job exposed a second Rust 1.100 layout assumption in
`scripts/collect_ffi_coverage.sh`. The collector ran all 471 FFI unit tests, 50
C consumer coverage tests, and the operation parity test, then exited because
it looked for the instrumented library and test executables under
`debug/deps`. Rust 1.100 emitted them under
`debug/build/kmipkit-ffi/<hash>/out`. A local checkout name also showed that
the merged `.profdata` filename follows Cargo's target-directory name and
cannot be assumed to be `KMIPKit.profdata`.

The clean failure was reproduced with the CI Rust 1.100 toolchain and
`scripts/collect_ffi_coverage.sh`; setting `TMPDIR` on the root filesystem
avoided the host's full 3.8 GiB `/tmp` tmpfs during AWS-LC compilation. The
missing-artifact result occurred after the FFI tests passed and the files
existed in Cargo's hashed `build/.../out` directories.

## Green: discover instrumented Cargo artifacts and use reported test paths

The collector now locates the newest `.profdata`, shared library, and static
library in the target tree. It extracts the complete test executable paths
from Cargo's `Running ...` output instead of reconstructing them under
`debug/deps`. The new workflow contract test failed before the fix and passes
after it.

The collector completed on Rust 1.100 with a dedicated target directory and
produced `coverage.json`. The workflow contract suite passed all 96 tests:

```sh
python3 -B -m unittest scripts.tests.test_workflow -v
bash -n scripts/collect_ffi_coverage.sh
RUSTUP_TOOLCHAIN=1.100.0 TMPDIR=<root-filesystem-temp-dir> CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=<dedicated-target-dir> \
  bash scripts/collect_ffi_coverage.sh <report-dir>
```

## Refactor: share the collector reader in workflow tests

The workflow tests now use one helper to read the collector source, keeping
the collector contracts consistent across the related checks. The 96-test
workflow suite and shell syntax check passed after this extraction.

## Remaining limitations

Controlled local TLS results are not vendor interoperability evidence. No Cosmian, CipherTrust or ESKM endpoint was
available for a live run. Cross-language parity is not claimed by this
Python-only validator run.
