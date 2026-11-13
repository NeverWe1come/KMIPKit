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
- The pre-regression Python 3.12 unit suite passed all 129 tests in 348.501 seconds.
- Repository requirement-traceability contracts passed 2 tests; feature
  traceability contracts passed 31 tests.
- The installed-package matrix at feature head `264ba33bc` completed all
  74,648/74,648 cells with zero failed or pending cells. That run used catalog
  SHA-256 `93bf7d439fe871cc467d0dd262e6dfe68340de4c81fa0671e5f967bb90a34613`.
  The current PR merge base changes the Batch Stop expected response count, so
  this earlier completion is not evidence for the current merge tree.
- Matrix run ID: `python-matrix-4691e61d-1739-4ee7-8991-9a4dce2e4fc1`.
  Evidence and closure are in `build/matrix-current/`; the report contains
  normalized tree and deterministic wire hashes, not KMIP bodies.
- The matrix exercises KMIP 2.0 and 2.1, TTLV/JSON/XML, raw TLS/HTTPS, typed
  and high-level public routes, generic TTLV construction and decoding,
  ordered batches, response accessors, stateful workflows and asynchronous
  routes using the controlled peer.

## Batch Stop response-count discrepancy

The common-base catalog at commit `66e1e73ce9e8745844686ec51719940d3aa9f8a8`
expects one response for the `test-batch-stop-on-first-failure` scenario
(catalog SHA-256 `e4a542bab4831a04e51f5bf04c9019da3f38845bc85839ae9b632dcc610a7d7f`).
The Python feature snapshot used for the prior matrix expected two responses
(catalog SHA-256
`93bf7d439fe871cc467d0dd262e6dfe68340de4c81fa0671e5f967bb90a34613`). OASIS
KMIP 2.1 Specification, OS, §11.5, Table 434 states that STOP prevents
processing subsequent operations and that responses to other batch items are
returned normally. KMIP 2.0 has the same rule in §11.5, Table 396. Thus the
one-versus-two response-count difference is a catalog-versus-normative-text
discrepancy; §9.5 defines Batch Count as the number of batch items in a
message but does not resolve this difference by itself.

The regression test builds the one-response peer reply selected by the
common-base catalog from the installed Python package, then observes
`Client.execute` reject it with `ClientExecutionError (2; delivery=3)`. The
route event is recorded as `failed` with `route_execution_failed`; it is not
counted as a successful interoperability invocation. Given the normative
text above, this result does not by itself establish a library defect. PR run
[43435461948](https://github.com/NeverWe1come/KMIPKit/actions/runs/43435461948)
also exposed the response-count mismatch. Its later independent validator
reported `FileNotFoundError` because the earlier test failure prevented the
matrix step from producing its report; that was a secondary workflow error.

The workflow now runs the matrix after consumer-test failure when the pinned
artifact manifest exists, validates only when the matrix report exists, and
warns rather than fails artifact upload when no report was produced. This keeps
the primary route result visible and avoids a misleading missing-file failure.
The validator PR does not change KMIPKit production code or the shared
generated catalog. The shared contract needs an approved normative review
before a library fix or conformance conclusion can be made.

## Pull-request CI evidence

At PR #173 head `264ba33bc0c04aa4285a2908a46ab097fd7802d8`, 49 checks succeeded
and none failed before the current common-base response-count change. The six
skips were controlled by path or workflow-event conditions. On the current
merge tree, the Python route matrix records the one-response STOP discrepancy
against the installed client. The common catalog's expected response count
requires an approved normative review before that event can be classified as a
library defect or the route can be considered closed.

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
