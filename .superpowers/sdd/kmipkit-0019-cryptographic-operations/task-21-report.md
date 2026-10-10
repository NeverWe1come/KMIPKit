# Task 21 — Decrypt GREEN report

Date: 2026-10-10
Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`
Start commit: `41d2a6bd0d11e674a73de95ebe55d4405ad62717` (T020 all-18-member correction)

## Scope implemented

Added public `DecryptRequest`, `DecryptResponse`, and `DecryptError` in `crates/kmipkit-protocol/src/decrypt.rs` and exported them from `kmipkit-protocol`. Request serialization follows Table 196 order, preserves omitted fields and all `OperationData` encodings, retains ordered unknown Cryptographic Parameters members, and carries Decrypt's Authenticated Encryption Tag input. The crate-private serialization-boundary validator rejects duplicate known request fields, known top-level fields with incorrect TTLV types, malformed known Cryptographic Parameters, and Encrypt-only response fields where applicable. It checks the TTLV Item Type and accepted singleton cardinality of every Table 59 member before the existing locally knowable §4.16 presence validation. Unknown Cryptographic Parameters children remain accepted.

Successful Table 197 response conversion requires one valid Unique Identifier, accepts the UID-only payload, and preserves optional Byte String Data and Correlation Value. It rejects duplicate or wrong-type known response members and Encrypt-only IV/Counter/Nonce and Authenticated Encryption Tag. Completed failures preserve the common `KmipOperationResult` without requiring a success payload or identifier. Error variants contain no payload values, while the original generic response message remains the source for unknown fields and values.

The full Table 59 matrix now exercises Decrypt duplicate and wrong-Item-Type rejection for all 18 members in addition to the existing Encrypt cases. The T016 Cryptographic Parameters test had a borrowed view from a temporary; binding that view fixes test compilation without changing the assertion. Three pre-existing pedantic issues in malformed-test helpers were simplified so the unisolated protocol Clippy target passes. The four-step fixture test retains its source-derived values together under a documented, test-local `clippy::too_many_lines` allowance.

## Focused Green evidence

Commands ran from the feature worktree, with no test-registration isolation or temporary test copies.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-protocol --lib decrypt_tests --offline` | passed, 4 tests |
| `cargo test -p kmipkit-protocol --lib decrypt_response_tests --offline` | passed, 2 tests |
| `cargo test -p kmipkit-protocol --lib operation_failure_tests --offline` | passed, 2 tests, including Decrypt failure preservation |
| `cargo test -p kmipkit-protocol --lib malformed_crypto_payload_tests --offline` | passed, 14 tests |
| `cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests --offline` | passed, 4 tests; Decrypt duplicate and wrong-Item-Type cases each cover all 18 Table 59 members |
| `cargo test -p kmipkit-protocol --lib operation_data_tests --offline` | passed, 5 tests |
| `cargo test -p kmipkit-protocol --lib cryptographic_parameters_tests --offline` | passed, 5 tests |
| `cargo fmt --all --check` | passed |
| `cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings` | passed without isolation |
| `cargo doc -p kmipkit-protocol --no-deps --all-features --offline` | passed; rustdoc generated |
| `git diff --check` | passed; Git emitted only its CRLF-to-LF advisory for the edited tasks file |

The required focused test target was green after the test-lifetime correction. The initial Decrypt request test compile stopped only at that temporary-borrow error; no test assertions ran before the repair. All focused targets above were then run unisolated. The final malformed target was rerun after simplifying its helpers and also passed.

## Fixture adapter status

Attempted `cargo test -p kmipkit-test-support --test oasis_crypto_fixtures --offline`. Compilation stops at the existing unresolved import `kmipkit_test_support::oasis_crypto_fixtures`; the T023 adapter module is not implemented yet. The task-21 scope does not add the XML adapter. No 28-item fixture pass is claimed; the four Decrypt operation-item request vectors in `decrypt_tests::fixture_derived_decrypt_operation_items_preserve_steps_2_6_7_and_8` pass as table-derived model assertions only.

## Traceability

Updated Decrypt-owned requirement and element references in `specs/019-cryptographic-operations/traceability.md`, including Table 196/197 request, response, result-failure, Data, Correlation Value, AAD, and Decrypt tag targets. All 18 Table 59 catalog member rows now identify both operation validators and their table-driven duplicate/type tests. Marked T021 complete in `specs/019-cryptographic-operations/tasks.md`. No upstream OASIS source, generated output, client dispatch, XML adapter, common result parser, or multipart client policy was changed.

## Limits

- The fixture XML adapter and the combined 28-item operation fixture pass remain pending T023/T024.
- Client dispatch, multipart initial-part policy, and follow-up behavior remain in later tasks.
- The §4.16 singleton treatment for known Table 59 members is the accepted feature criterion; §4.16 itself does not contain explicit per-member maximum-occurrence wording.
- No full workspace test, coverage, C/Java/Python parity, or interoperability claim is made by T021.

## P2 review correction — Pending is not a completed Decrypt result

The response conversion review found that Decrypt's non-Success branch also accepted a valid `Operation Pending` item as a completed response. RED-only coverage was committed separately in `065083767d2628453c46120b445468ac900927d3` (`test(KMIPKIT-0019): reject pending completed results`). Before production edits, `cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline` exited 101 as expected: the pre-existing correlation-shape test passed, while both new Encrypt and Decrypt rejection assertions failed against the current converters.

Decrypt now returns the payload-free `DecryptError::PendingOutcomeRequired` for Pending, and `DecryptResponse::try_from_response_item` docs direct callers to route Pending through shared `PendingOutcome` before using this completed-result converter. The paired Encrypt converter received the same correction and `EncryptError::PendingOutcomeRequired`. Tests assert these exact variants. This work does not create `PendingOutcome` or add client dispatch; that remains client integration scope. Completed Failure behavior remains covered separately.

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

The T021 task evidence, `KMIPKIT-REQ-SPEC-6.1-001-002`, and FR-007 traceability now describe Pending as separate from completed Failure/unknown-reason handling. PendingOutcome creation and client dispatch remain outside T021; no fixture, interoperability, or full-workspace claim is made.
