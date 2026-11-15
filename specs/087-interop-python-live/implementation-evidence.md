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

## Historical Batch Stop response-count discrepancy

An earlier PR merge tree at common commit
`66e1e73ce9e8745844686ec51719940d3aa9f8a8` expected one response for the
`test-batch-stop-on-first-failure` scenario (catalog SHA-256
`e4a542bab4831a04e51f5bf04c9019da3f38845bc85839ae9b632dcc610a7d7f`). The
current PR base is commit `4eb22e220c4e016f1cb218879a7e952588bbe6fc`, whose
catalog expects two responses (SHA-256
`93bf7d439fe871cc467d0dd262e6dfe68340de4c81fa0671e5f967bb90a34613`), matching
the Python feature snapshot and the prior full matrix. OASIS KMIP 2.1
Specification, OS, §11.5, Table 434 states that STOP prevents
processing subsequent operations and that responses to other batch items are
returned normally. KMIP 2.0 has the same rule in §11.5, Table 396. Thus the
temporary one-versus-two response-count difference came from the earlier
common catalog conflicting with the cited table text; §9.5 defines Batch
Count as the number of batch items in a message but does not resolve that
table-text discrepancy by itself.

The regression test for the earlier base commit builds its one-response peer
reply from the installed Python package, then observes
`Client.execute` reject it with `ClientExecutionError (2; delivery=3)`. The
route event is recorded as `failed` with `route_execution_failed`; it is not
counted as a successful interoperability invocation. Given the normative
text above, that result did not establish a library defect. The push matrix
passed every cell, including Batch Stop, using the current two-response catalog
and the installed pinned package. This is controlled-fixture evidence, not a
vendor-server interoperability result. The independent base-merge run against
the earlier one-response catalog is recorded below.

The workflow now runs the matrix after consumer-test failure when the pinned
artifact manifest exists, validates only when the matrix report exists, and
warns rather than fails artifact upload when no report was produced. This keeps
the primary route result visible and avoids a misleading missing-file failure.
The validator PR does not change KMIPKit production code or the shared
generated catalog.

## Red-Green-Refactor evidence

- **Red** (`beac2767c`): added the regression tests, restored the prior
  workflow condition, then ran
  `PYTHONPATH=src .venv/bin/python -m unittest discover -s tests -p 'test_route_matrix.py' -k test_ci_runs_matrix_after_test_failure_and_guards_missing_evidence -v`.
  It failed because the matrix step did not include `always()`.
- **Green** (`9429bef92`): made the matrix run when the installed artifact
  manifest exists, guarded independent validation on report existence, and
  guarded evidence upload. The same test passed, as did both tests selected by
  `-k batch_stop`.
- **Red/Green for failed-event evidence** (`0f84eba34`, `cef007b9c`,
  `876ca8903`): added assertions that a rejected partial response still
  preserves the observed request batch sequence/count, response count and
  delivery state. With the new assertions and the prior route handler, the
  focused test failed with `KeyError: batch_operation_sequence`; after the
  route handler change, it passed and the shared validator's batch-event
  structural check accepted the failure event.
- **Red/Green for generic batch failures** (`5b5427dfa`, `7a07d5592`): the
  test first reproduced `KeyError: batch_operation_sequence` for an observed
  batch route failure. The fix records only catalogued attempted operation
  order and count; it does not claim that the peer received the request.
  The focused test and shared batch-event structural check pass.
- **Refactor/evidence** (`89d5c6228`): updated README and implementation
  evidence to distinguish the OASIS text from the older common-catalog
  response count; the failed event is retained as historical evidence.
- Full Python consumer suite: 131 tests passed in 323.323 seconds.
- After preserving failed-route evidence, the full Python consumer suite passed
  again: 131 tests in 304.723 seconds.
- After adding generic failed-batch evidence, the full Python consumer suite
  passed: 132 tests in 297.611 seconds.
- Repository requirement traceability: 2 tests passed; feature traceability:
  33 tests passed in 66.351 seconds. The exact pinned wheel was accepted by
  `tools/verify_artifact.py`.

## Exact-head workflow run

The push run [43445600626](https://github.com/NeverWe1come/KMIPKit/actions/runs/43445600626)
at head `ca2e79c1bbb52fcbc3de6f00d4e0be9efc5644f1` passed all steps and produced
74,648/74,648 passed cells, zero failures and zero pending cells against the
feature catalog SHA `93bf7d439fe871cc467d0dd262e6dfe68340de4c81fa0671e5f967bb90a34613`.
The push run [43453206635](https://github.com/NeverWe1come/KMIPKit/actions/runs/43453206635)
at head `a322c94a39aa6604d81e57bd98b66f77dad75722` also passed all steps:
74,648/74,648 cells, zero failed, zero pending; independent validation
reported `all_invocations_passed: true` with the same catalog SHA. The installed
artifact SHA was
`aee5cf55655b0a6ef9390ee6827813b1b7a29ff874aa685c0e0ca04f53127385`.
The earlier PR merge run
[43445611360](https://github.com/NeverWe1come/KMIPKit/actions/runs/43445611360)
used the then-current one-response catalog (`e4a542...`) and reported 74,573
passed, 75 failed and zero pending cells. It uploaded all three artifacts. The
failures were 72 Batch Stop cells and three isolated Continue/placeholder
cells that passed when reproduced individually with the pinned wheel. The
independent validator's first error was missing `batch_operation_sequence` on
one failed event. Commits `5b5427dfa` and `7a07d5592` preserve attempted batch
metadata for that case without claiming peer receipt. The current base has
since moved to the two-response catalog; a new PR merge run is required to
verify this exact head against that base.

## Pull-request CI evidence

The PR workflow [43453215609](https://github.com/NeverWe1come/KMIPKit/actions/runs/43453215609)
at head `a322c94a39aa6604d81e57bd98b66f77dad75722` checked out merge commit
`7f1c64dd1e405738f34311db34f9595b703856d8`, whose message identifies the old
base `66e1e73ce9e8745844686ec51719940d3aa9f8a8`. It used that tree's one-response
catalog (`e4a542...`) and reported 74,576 passed, 72 failed, zero pending; all
72 failures were Batch Stop cells whose controlled peer returned the one
response prescribed by that stale catalog. The independent validator accepted
the full 74,648-cell evidence report, with `all_invocations_passed: false` and
no secondary missing-file or structural error. The PR base is now
`4eb22e220c4e016f1cb218879a7e952588bbe6fc`; its catalog SHA is
`93bf7d439fe871cc467d0dd262e6dfe68340de4c81fa0671e5f967bb90a34613` and
expects two responses. The matching push workflow passed the full matrix at the
same code head. A new PR event is required to verify the current-base merge
tree. This discrepancy is attributable to the stale PR merge event; no
library defect is established.

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
