# KMIPKIT-0087 implementation evidence

## Artifact under test

- Release ref: `release/1.0.0`
- Exact library commit: `52a2be180340e5da3166401a52f652dfd368ad37`
- Locally installed wheel SHA-256: `4bc56fc596d64cd8b7d63845e55831dad31d284c1fd260be5a1b0e0f7fa9cacf`
- Runtime: Python 3.12; Rust 1.94 build toolchain

The validator imports the installed wheel. It does not import `bindings/python`
from the checkout.

## Local verification

From `interop/python-kmip-validator` after the pinned artifact is installed:

```sh
PYTHONPATH=src python tools/verify_artifact.py
PYTHONPATH=src python -B -m unittest discover -s tests -p 'test_*.py' -v
PYTHONPATH=src python -m kmipkit_validator plan \
  --connection ../shared/profiles/connection.example.json
```

Observed locally:

- `verify_artifact.py` accepted the pinned commit and wheel hash listed above.
- All 25 consumer tests passed in 6.820 seconds, including six controlled
  mTLS exchange, version-gating and authentication tests.
- The catalog contains 218 rows across KMIP 2.0 and 2.1. Offline plans
  enumerate all 203 cases applicable to KMIP 2.0 and all 212 applicable to
  KMIP 2.1 without network access.
- Each version plan marks 130 (KMIP 2.0) or 139 (KMIP 2.1) cases as planned
  and 73 as documented public-API gaps. Those gaps are visible as pending and
  are not represented as server incompatibilities.
- The installed-package constructor sweep built and schema-validated every
  advertised typed request recipe; async routes with required correlations
  use dedicated runners. Local enum and bitmask catalog values round-tripped
  through the installed TTLV API.

## Final implementation-head CI

The published feature branch incorporated release commit
`2b01ab516e50035eee42fc206396c313dae8d9fb` with merge commit
`395a68ec7846fcdbabbdf0b382c65e9581bb1ae3`. The merge required no conflicts
and preserved the approved artifact pin and the locally installed wheel above.
No consumer production behavior changed in this closeout.

All five workflows for implementation head
`395a68ec7846fcdbabbdf0b382c65e9581bb1ae3` completed successfully:

- [CI, run 42909548152](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909548152): Linux, Windows and macOS core jobs, language consumers, script contracts, sanitizers, normative inventory, documentation/traceability, dependencies, coverage and selected-scope coverage gate passed.
- [Interop Python Validator, pull-request run 42909547242](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909547242): external wheel build/install and provenance, offline preflight and KMIP 2.0/2.1 plans, all 25 consumer tests and shared catalog contract checks passed. The unit suite took 8.330 seconds.
- [Interop Python Validator, push run 42909535759](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909535759): passed independently for the same feature commit.
- [CodeQL Analysis, run 42909547772](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909547772): C/C++, Java/Kotlin, Python and Rust analyses passed.
- [zizmor, run 42909548137](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909548137): GitHub Actions workflow audit passed.

The PR rollup had 45 successful checks, including SonarCloud, and no failures
or pending jobs before this evidence update. Three jobs were skipped by their
workflow event conditions: `Dependency policy / active release schedule` and
`Informational branch coverage / nightly` run only for `schedule`;
`Profile evidence / protected release aggregate` runs only for a protected
`release/1.0.0` push. This was a feature pull-request run, so these skips do
not establish release aggregate evidence.

The pull-request CI wheel was built from the same exact library commit and
verified SHA-256
`0cdc157fc9017a1064f1fbbf4cc5e488e7eceb92afdfc4f97b147cdd8372d441`.
This CI artifact is distinct from the retained local wheel; byte-identical
wheel reproducibility is not claimed. Every artifact's report uses its own
verified digest.

Evidence-only follow-up `287df0acb61e283536b1cbce48586603c4cd6e87`
started a fresh CI cycle. Its green checks, if achieved, do not complete
exhaustive validator coverage or authorize marking this PR ready.

## Maintainer coverage clarification and incomplete acceptance

After that evidence update, the maintainer explicitly required all four
validators to cover every applicable operation/case exhaustively through all
available public construction routes, attributes, generic TTLV, encodings and
transports. Successful CI with pending coverage does not satisfy that
acceptance criterion. This clarification is stricter than the original
specification's allowance for truthful documented API gaps.

T008 remains incomplete and PR #173 remains WIP/draft. The current offline
plan still reports 73 documented public-API gaps per protocol version. The
existing recipe-construction checks, selected high-level routes, local TTLV
checks and controlled exchanges do not establish exhaustive coverage through
every available public route or the full encoding/transport matrix. Missing
public API support and untested construction paths must be resolved and
verified before this validator can be accepted as complete. The successful
implementation-head CI above establishes only the exercised checks.

This closeout records the current implementation and its limits; it does not
claim that the expanded exhaustive coverage has been implemented. The exact
latest evidence-only head and check results are recorded in the closeout
report and PR description.

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

## Remaining limitations

Controlled local TLS results are not vendor interoperability evidence. No Cosmian, CipherTrust or ESKM endpoint was
available for a live run. Cross-language parity is not claimed by this
Python-only validator run.
