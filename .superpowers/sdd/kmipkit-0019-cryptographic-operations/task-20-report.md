# Task 20 Green Report

Date: 2026-10-10
Task: T020 GREEN only
Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`
Start commit: `d635f8dc802a451fb12728afd3f7a51f072d20ee`

## Scope implemented

Added the Table 214 `EncryptRequest` serializer and Table 215 `EncryptResponse` conversion in `crates/kmipkit-protocol/src/encrypt.rs`, and exported the public types from the protocol crate. Request members retain Table 214 order, caller-selected TTLV encodings, optional omission, and supplied Cryptographic Parameters order. `OperationData` and byte-string secrets move into zeroizing TTLV values. The crate-private request validator rejects repeated known singleton fields, invalid known Item Types, malformed Cryptographic Parameters children, and the Decrypt-only Authenticated Encryption Tag. Encrypt checks the defined Item Types and accepted singleton cardinality for all 18 Table 59 members before the existing §4.16 presence validator runs. Unknown members remain in the generic payload.

Successful responses parse the required permitted Unique Identifier and optional Table 215 Byte Strings, while rejecting duplicates, wrong Data types, and known request-only members. Non-success responses preserve the shared operation result without requiring a success payload or identifier. The generic `ResponseMessage` remains the source of unknown fields.

The two T015 request tests that borrowed an Item from a temporary `StructureView` failed to compile with E0716 once the Encrypt API resolved. Added local `StructureView` bindings in `encrypt_tests.rs`; this is a test-lifetime repair only and does not change assertions or behavior.

## Focused Green evidence

The current source still registers T021 Decrypt tests, which cannot compile before T021. For focused test runs, only the Decrypt request/response, mixed Encrypt/Decrypt failure, and combined malformed test registrations were isolated. The combined malformed and failure source files were copied to temporary Encrypt-only variants to run their existing Encrypt tests. All temporary source and `lib.rs` registration changes were restored in PowerShell `finally` blocks; no temporary test files or test-registration edits remain in the working tree.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-protocol --lib encrypt_tests --offline` | 4 passed |
| `cargo test -p kmipkit-protocol --lib encrypt_response_tests --offline` | 2 passed |
| `cargo test -p kmipkit-protocol --lib encrypt_failure --offline` (temporary Encrypt-only view of the mixed failure module) | 1 passed |
| `cargo test -p kmipkit-protocol --lib encrypt_request --offline` (temporary Encrypt-only view of the combined malformed module) | 4 passed |
| `cargo test -p kmipkit-protocol --lib encrypt_response --offline` (same temporary malformed view; also selected the generic Pending shape test) | 6 passed |
| `cargo test -p kmipkit-protocol --lib operation_data_tests --offline` | 5 passed |
| `cargo test -p kmipkit-protocol --lib cryptographic_parameters_tests --offline` | 5 passed |
| `cargo fmt --all --check` | passed |
| `cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings` (T021-dependent test registrations isolated) | passed |
| `cargo doc -p kmipkit-protocol --no-deps --all-features --offline` | passed; rustdoc generated |
| `git diff --check` | passed |

The normal unisolated `cargo test -p kmipkit-protocol --lib encrypt_tests --offline` compile initially confirmed the T021 limitation: missing `DecryptRequest`/`DecryptResponse` and `crate::decrypt` references in registered Decrypt-dependent test modules. After the test-lifetime repair, Encrypt tests and malformed checks passed with only those T021-dependent modules isolated. No full-feature Green or Decrypt result is claimed.

## Traceability

Updated Encrypt-owned references in `specs/019-cryptographic-operations/traceability.md` to the implemented Encrypt model and existing T015/T017/T019 tests. Decrypt paths and tests remain pending. Updated T020's task row with this focused evidence; T021–T050 are unchanged.

## Self-review and limits

- No Decrypt model, client dispatch, XML adapter, common result parser refactor, generated output, or public raw request-ingestion API was added.
- Known error variants are static and do not contain request or response payload values. Byte-string response copies are owned by `SecretBytes`; request bytes move through `OperationData`/`SecretBytes` into zeroizing TTLV values.
- The borrowed Cryptographic Parameters presence helper shares the existing §4.16 validation logic; it preserves input ordering and values.
- Decrypt-dependent modules and their tests were not run as complete modules because T021 remains unimplemented. Client execution, XML fixture adapter, multipart client rules, Refactor, and full feature gates remain future tasks.

## P2 review correction — full Table 59 Encrypt validation

Date: 2026-10-10
Base: `057bcad5950d4e6a44d29a07f5ffaca3d88082f0` (T019 RED matrix and its traceability correction are retained).

The review finding was that `validate_known_cryptographic_parameters_view` only checked TTLV Item Type and repeated occurrences for Block Cipher Mode, IV Length, and Tag Length. The registered RED matrix covers all 18 Cryptographic Parameters members listed in pinned OASIS KMIP v2.1 §4.16, Table 59, using their allocated tags from Table 487. The implementation now checks the Table 59 Item Type and accepted singleton cardinality for all 18 members in the Encrypt boundary. Unrecognized child tags continue through the default path and are not rebuilt or reordered; unknown-field preservation remains covered by `encrypt_tests::request_preserves_supplied_cryptographic_parameters_members_and_order` and `cryptographic_parameters_tests::unknown_parameter_members_survive_in_original_order_and_encoding`.

§4.16 calls Cryptographic Parameters a “set of OPTIONAL fields” and Table 59 provides the member encodings, but the pinned text does not state a separate maximum-occurrence sentence per member. Singleton enforcement is the accepted T020 feature criterion for known-child cardinality validation; this correction does not claim that the OASIS specification states an explicit per-member `MUST` against repetition. The work changes only Encrypt validation. Decrypt parity remains T021.

### RED reproduction

```text
cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests --offline
```

With only the T021-dependent registrations `decrypt_tests`, `operation_failure_tests`, `decrypt_response_tests`, and `malformed_crypto_payload_tests` temporarily isolated, the matrix compiled and ran, then exited 101 with both tests failing as expected. Each assertion reported the same 15 accepted malformed members: Cryptographic Algorithm, Hashing Algorithm, Padding Method, Key Role Type, Digital Signature Algorithm, Random IV, Fixed Field Length, Counter Length, Initial Counter Value, Invocation Field Length, Salt Length, Mask Generator, Mask Generator Hashing Algorithm, P Source, and Trailer Field. The three pre-existing members already failed their malformed cases. A PowerShell `finally` block restored `src/lib.rs` byte-for-byte.

### GREEN verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests --offline` | 2 passed; duplicate and wrong Item Type coverage for all 18 members |
| `cargo test -p kmipkit-protocol --lib encrypt_tests --offline` | 4 passed; includes supplied unknown-member order/value preservation |
| `cargo test -p kmipkit-protocol --lib encrypt_response_tests --offline` | 2 passed |
| `cargo test -p kmipkit-protocol --lib encrypt_malformed_focused_tests --offline` | 7 passed; temporary Encrypt-only view of the combined malformed module |
| `cargo test -p kmipkit-protocol --lib operation_data_tests --offline` | 5 passed |
| `cargo test -p kmipkit-protocol --lib cryptographic_parameters_tests --offline` | 5 passed; includes unknown-member order and encoding preservation |
| `cargo fmt --all --check` | passed |
| `cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings` | passed; the four T021-dependent test registrations were temporarily isolated and restored byte-for-byte |
| `cargo doc -p kmipkit-protocol --no-deps --all-features --offline` | passed; rustdoc generated |
| `git diff --check` | passed after the final source, task, traceability, and report edits |

For focused testing, the combined malformed test registration was replaced temporarily by `encrypt_malformed_focused_tests.rs`, produced from the existing mixed Encrypt/Decrypt module with only Encrypt cases enabled. The temporary test file and registration were removed in `finally`; `src/lib.rs` was verified byte-for-byte restored. The T019 18-member matrix stayed enabled in all applicable focused test and lint runs. No committed tests or generated files were changed for this correction.

### Limits and traceability

Traceability now records Encrypt's Green implementation against all 18 `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-*` entries, both matrix tests, and the existing unknown-member preservation tests. The RED history remains documented in `task-19-parameter-structure-report.md`. For this original Cryptographic Parameters correction, registered Decrypt coverage still depended on T021 and Decrypt behavior was out of scope; the separate Pending response review correction below applies to both operation converters. No full workspace or complete feature test run is claimed.

## P2 review correction — Pending is not a completed Encrypt result

The response conversion review found that the non-Success branch also accepted a valid `Operation Pending` item as a completed Encrypt response. The RED-only tests were committed separately as `065083767d2628453c46120b445468ac900927d3` (`test(KMIPKIT-0019): reject pending completed results`). Before production edits, `cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline` exited 101 as expected: the pre-existing correlation-shape test passed, while the new Encrypt and Decrypt assertions failed because both converters returned `Ok` for valid Pending items carrying Asynchronous Correlation Value and multipart Correlation Value.

The Green correction makes both completed response converters reject Pending with payload-free `EncryptError::PendingOutcomeRequired` / `DecryptError::PendingOutcomeRequired` variants. Their rustdoc requires callers to route Pending through the shared `PendingOutcome` path before conversion. Assertions match the exact variants. No `PendingOutcome` is constructed here and no client dispatch is added; that remains client integration scope. Existing Success payload conversion and completed Failure preservation are retained.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline` | passed, 3 tests including exact typed Pending rejection for Encrypt and Decrypt |
| `cargo test -p kmipkit-protocol --lib encrypt_response_tests --offline` | passed, 2 tests |
| `cargo test -p kmipkit-protocol --lib decrypt_response_tests --offline` | passed, 2 tests |
| `cargo test -p kmipkit-protocol --lib operation_failure_tests --offline` | passed, 2 tests; completed Failure common-result behavior preserved |
| `cargo fmt --all --check` | passed after `cargo fmt --all` |
| `cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo doc -p kmipkit-protocol --no-deps --all-features --offline` | passed; rustdoc generated |
| `git diff --check` | passed; Git reports only the existing tasks.md CRLF-to-LF advisory |

Updated the T020/T021 task evidence and the `KMIPKIT-REQ-SPEC-6.1-001-002` and FR-007 traceability rows. FR-007 now distinguishes completed failures and unknown reasons from Pending, which stays on the shared PendingOutcome path. This correction makes no full-workspace, coverage, fixture adapter, interoperability, or client-dispatch claim.
