# KMIPKIT-0018 implementation evidence

## T014 Red

Baseline: `ad544a2` (`feature/KMIPKIT-0018-managed-object-lifecycle-implementation`).

- `cargo test -p kmipkit-client --lib activate_execution_tests` failed during
  compilation with `E0599` for the missing `Client::activate`, `Client::destroy`,
  and Activate/Destroy request and operation variants.
- `cargo test -p kmipkit-client --lib destroy_execution_tests --message-format short`
  failed during compilation with the same expected missing-dispatch `E0599`
  errors from both existing fake-transport contract modules.

Both commands exercise the committed T009/T010 contracts before T014 changes.

## T014 Green

- `cargo test -p kmipkit-client --all-features` passed: 260 client unit tests,
  all client integration/UI tests, and all client doctests.
- `cargo fmt --all --check` passed.
- `cargo check -p kmipkit-client --all-features` passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
  passed.
- `python3 -B -m unittest discover -s tools/normative_catalog/tests -p
  'test_feature_traceability.py' -v` passed (15 tests, 1 Windows symlink
  permission skip).
- `git diff --check` passed.
- The specific client Debug redaction assertions were committed in
  `8b65124` after both failed against the forwarding Debug implementations.

The client-root re-exports already expose `ClientRequest`, `ClientOperation`,
`ClientBatchOutcome`, and `ClientResponseView`; the new operation variants and
accessors are therefore public through the existing facade. Protocol models
remain imported from `kmipkit-protocol`, consistent with the existing API.

### FR-010 scope note

Directly formatting Activate/Destroy protocol requests and responses no longer
exposes a `UniqueIdentifier`. The fix centralizes redaction in
`UniqueIdentifier`'s `Debug` implementation, which also protects other protocol
models that contain the same identifier type. Client outcomes, response views,
and malformed-success errors retain their existing redaction behavior.

The regression tests were added before the fix and verified failing:

- `e315b3b test(KMIPKIT-0018): cover lifecycle identifier redaction` added
  request/response tests for Activate and Destroy; the focused Activate suite
  showed 2 expected failures and 8 passes, and Destroy showed 2 expected
  failures and 11 passes.
- `ae3dcf2 test(KMIPKIT-0018): cover identifier Debug wire forms` added
  coverage for TextString, Enumeration, and Integer identifiers; the focused
  test failed because the wire value was formatted.
- `f381666 fix(KMIPKIT-0018): redact identifier Debug values` implemented
  central identifier redaction; all five redaction tests passed.
- `061815c refactor(KMIPKIT-0018): share identifier redaction assertions`
  extracted the common assertion into lifecycle test support and reran all
  focused protocol suites successfully.

The initial security diff scan independently reproduced the four protocol
Debug outputs before the fix. Its final policy classified these instances as
non-reportable because it did not establish a secret classification or an
unauthorized trust-boundary crossing; the FR-010 policy gap was fixed anyway.
A fresh scan of the remediation head is recorded separately before PR review.

## T015 Green: Activate and Destroy

Focused protocol and client tests were rerun after the identifier redaction
change and shared assertion extraction:

- `cargo test -p kmipkit-protocol --lib activate_operation_tests --all-features`
  passed (10 tests).
- `cargo test -p kmipkit-protocol --lib destroy_operation_tests --all-features`
  passed (13 tests).
- `cargo test -p kmipkit-client --lib activate_execution_tests --all-features`
  passed (4 tests).
- `cargo test -p kmipkit-client --lib destroy_execution_tests --all-features`
  passed (4 tests).

Full package verification also passed:

- `cargo test -p kmipkit-protocol --all-features`: 217 unit tests, 144
  integration tests, and 2 doctests passed.
- `cargo test -p kmipkit-client --all-features`: 260 unit tests, 62
  integration/UI tests, and 9 doctests passed.
- `cargo fmt --all --check` passed.
- `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D
  warnings` passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D
  warnings` passed.
- `python -B -m unittest discover -s tools/normative_catalog/tests -p
  'test_feature_traceability.py' -v` passed (15 tests, one Windows symlink
  permission skip).
- `git diff --check` passed.

These are scoped protocol/client checks; they do not claim the workspace-wide
coverage thresholds or the remaining lifecycle operations are complete.

## T014 Refactor review

The Activate and Destroy convenience methods both route through
`Client::execute_with_options`, and their response conversion already shares
`read_operation_outcome`. Their remaining repeated code is the operation-
specific request construction and wrapper method signatures; extracting it
would add a generic helper without simplifying the flow. No behavior-preserving
production encoding or response helper refactor was justified. The test-only
identifier redaction assertion was consolidated in shared lifecycle fixtures
in commit `061815c`; focused protocol tests passed after extraction. This
completes the T016 review without making the operation-specific protocol
modules less distinct.
