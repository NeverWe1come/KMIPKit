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

- `cargo test -p kmipkit-protocol --all-features`: 217 unit tests, 150
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
specific request construction and wrapper method signatures. The common
identifier TTLV mechanics became a clear three-operation duplication once
Archive was added and were extracted in the lifecycle Refactor below.

## T017–T019 Red: Archive contracts

- `d572297 test(KMIPKIT-0018): add Archive protocol contracts` added ten
  source-derived request, response, malformed-success, result, and redaction
  tests. `cargo test -p kmipkit-protocol --lib archive_operation_tests
  --all-features` failed as expected because `ArchiveRequest`, `ArchiveResponse`,
  and `ArchiveError` were not yet exported.
- `e1808a6 test(KMIPKIT-0018): add Archive client contracts` added fake-transport
  Success, Failure, Pending, malformed-success, one-exchange, redaction, and
  optional-identifier assertions. `cargo test -p kmipkit-client --lib
  archive_execution_tests --all-features` failed as expected because
  `Client::archive`, `ClientRequest::Archive`, and
  `ClientOperation::Archive` did not exist.

Archive vectors cite OASIS KMIP Specification v2.1 §6.1.4 Tables 173–175,
§4.58 Tables 145–146, and §11.56 Table 487. Table 175's `Object Archived`
result is preserved as a failure result. These derived tests are not claimed as
official OASIS Test Cases.

## T020–T022 Green: Archive model and client

- `1b68113 feat(KMIPKIT-0018): add Archive protocol model` added typed request
  and response models and protocol exports. The request preserves an omitted or
  supplied identifier; success requires exactly one supported response
  identifier; failure retains the KMIP result.
- `1622a84 feat(KMIPKIT-0018): dispatch Archive requests` added the client
  request variant, operation identity, typed response view, and
  `Client::archive`/`archive_with_options` through the shared one-exchange path.
  Its API documentation describes Archive as a preference and does not claim
  completion.
- `cargo test -p kmipkit-protocol --lib archive_operation_tests --all-features`
  passed (10 tests).
- `cargo test -p kmipkit-client --lib archive_execution_tests --all-features`
  passed (4 tests).
- `cargo test -p kmipkit-protocol --all-features`: 227 unit tests, 150
  integration tests, and 2 doctests passed after the Archive refactor.
- `cargo test -p kmipkit-client --all-features`: 264 unit tests, 62
  integration/UI tests, and 9 doctests passed.
- Protocol and client all-target/all-feature Clippy with `-D warnings` passed;
  `cargo fmt --all --check` passed.
- Normative traceability tests passed (15 tests, one Windows symlink-permission
  skip); `git diff --check` passed.

## T016/T023 Refactor: shared lifecycle TTLV mechanics

`543dd69 refactor(KMIPKIT-0018): share lifecycle TTLV mechanics` extracted
optional request identifier encoding and successful response identifier
validation into private protocol support used by Activate, Archive, and
Destroy. The operation modules retain distinct operation identifiers, public
types, response errors, and source table documentation.

After extraction, the focused Activate, Archive, and Destroy protocol suites
passed (10, 10, and 13 tests respectively), and protocol Clippy with `-D
warnings` passed. Full protocol/client suites, client Clippy, formatting, and
traceability checks also passed after this refactor.

## T024–T026 Red: Recover contracts

- `59dafa1 test(KMIPKIT-0018): add Recover protocol contracts` added ten
  source-derived request, response, malformed-success, table-error, and
  redaction tests. Before the model existed,
  `cargo test -p kmipkit-protocol --lib recover_operation_tests --all-features`
  failed because `RecoverRequest`, `RecoverResponse`, and `RecoverError` were
  not exported.
- `02b2469 test(KMIPKIT-0018): add Recover client contracts` added fake-transport
  Success, Failure, Pending, exact correlation, no Poll/Get follow-up, malformed
  response delivery evidence, partial-write delivery evidence, and one-exchange
  tests.
- After the protocol model was added but before client dispatch,
  `cargo test -p kmipkit-client --lib recover_execution_tests --all-features`
  failed on the missing `Client::recover`, `ClientRequest::Recover`, and
  `ClientOperation::Recover` API (eight expected compile errors). This is the
  T026 Red evidence.

Recover tests cite OASIS KMIP Specification v2.1 §6.1.42 Tables 288–290,
§4.58 Tables 145–146, and §11.56 Table 487. The table-defined `Object Not
Found` reason is preserved; tests remain source-derived and are not claimed as
official OASIS Test Cases.

## T027–T029 Green: Recover model and client

- `9be4ac1 feat(KMIPKIT-0018): add Recover protocol model` added the typed
  request and response exports over the shared lifecycle TTLV mechanics.
- `37b9792 feat(KMIPKIT-0018): dispatch Recover requests` added the Recover
  client request, result view, operation identity, and
  `Client::recover`/`recover_with_options` on the shared one-exchange path.
  API documentation states that any later Poll or Get is caller initiated.
- `cargo test -p kmipkit-protocol --lib recover_operation_tests --all-features`
  passed (10 tests).
- `cargo test -p kmipkit-client --lib recover_execution_tests --all-features`
  passed (5 tests).
- `cargo test -p kmipkit-protocol --all-features`: 237 unit tests, 150
  integration tests, and 2 doctests passed.
- `cargo test -p kmipkit-client --all-features`: 269 unit tests, 62
  integration/UI tests, and 9 doctests passed.
- Protocol and client all-target/all-feature Clippy with `-D warnings` passed;
  `cargo fmt --all --check` passed.
- Normative traceability tests passed (15 tests, one Windows symlink-permission
  skip); `git diff --check` passed.

## T030 Refactor review

Recover response conversion already uses `read_operation_outcome`, including
the shared Pending handling and exact correlation ownership. The common
identifier request/response TTLV mechanics were extracted in T023 and cover
Recover as well. No additional Recover-only response helper would reduce
duplication while preserving the shared operation boundaries. After this
review, the focused Recover protocol and client suites passed (10 and 5 tests).
