# Tasks: KMIP 2.1 Encrypt and Decrypt Operations

**Input**: Design documents in specs/019-cryptographic-operations/
**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/operation-payloads.md, traceability.md
**Tests**: Required; strict Red, Green, Refactor with separate evidence commits.
**Organization**: T001–T004 prepare the specification PR. Implementation is authorized under the maintainer's standing direct instruction; the implementation branch must still be based on release/1.0.0 with all listed dependencies present. This records scope authorization, not a claim of separate line-by-line review. Reviewer-owned checklists remain untouched.

## Specification PR preparation

- [x] T001 Update the normative catalog for KMIPKIT-0019: assign Encrypt/Decrypt, six shared operation-structure elements, the Cryptographic Parameters attribute, all 21 applicable client requirements, and linked test-case records; pin the three exact official OASIS fixtures with source URLs and hashes; add KMIPKIT-DISC-045 and the §2.100 test-evidence source defect KMIPKIT-DISC-046; exclude source clause KMIPKIT-CLAUSE-SPEC-4.16-004 with rationale and no requirement link, and correct the §4.16-001-002 summary without editing upstream OASIS files. Retain related ID Placeholder server-only requirements in the catalog without assigning them to this client feature.
- [x] T002 Regenerate specification/catalog/coverage-report.md using the pinned report generator; run catalog validation, report --check, and immutable-source verification against the release base.
- [x] T003 Reconcile specs/019-cryptographic-operations/traceability.md against every assigned catalog ID, feature requirement, planned code path, test target, inherited owner, and the in-scope operation items extracted from each official case fixture.
- [x] T004 Run speckit-analyze after tasks exist, fix all critical/high inconsistencies, and leave reviewer-owned checklist boxes unchecked. Record the analyzed revision and unresolved discrepancies in the PR description.

## Phase 1: Dependency and source gate

- [x] T005 Confirm this implementation branch is based on the latest release/1.0.0 and the merged KMIPKIT-0005 TTLV codec, KMIPKIT-0006 message/batch model, KMIPKIT-0007 client execution, KMIPKIT-0009 Pending handling, and KMIPKIT-0013 production transport, and KMIPKIT-0018 Recover; record exact commits in traceability.md.
- [x] T006 Reconfirm Tables 196–198, Tables 214–216, §4.16, §§6.1/7.3/7.4/7.8/7.9/7.14/7.17 against the pinned source and verify all required tags are generated. Keep DISC-045 open and require a local pre-transmission validation error only for the ambiguous single-part form where both Init and Final are true and Data is omitted; accept the same form when Data is present. Explain the §4.16-004 source-clause disposition.

## Phase 2: Shared request data and validation

### Red

- [ ] T007 Add failing unit tests for OperationData preserving Byte String, Enumeration, and Integer request encodings, exact values, and ordering in crates/kmipkit-protocol/tests/unit/operation_data_tests.rs.
- [ ] T008 Add failing tests proving Debug/Display redact every OperationData variant and owned Byte String storage uses the existing zeroizing path.
- [ ] T009 Add failing tests for §4.16 recognized parameter conditions: variable-IV mode requires IV Length, GCM requires Tag Length, unknown parameter members survive, and parameters derived from remote object attributes are not guessed locally.
- [ ] T010 Run the focused new test targets and record the expected Red failures and command output in the Red commit.

### Green

- [ ] T011 Implement the redacted OperationData union using existing TTLV values and secret-byte ownership; preserve all three §7.9 encodings without exposing values.
- [ ] T012 Implement narrowly scoped Cryptographic Parameters inspection/validation for recognized required conditions while retaining ordered unknown values and the optional operation-level field.
- [ ] T013 Run focused OperationData and parameter tests to Green; capture command/result evidence.

### Refactor

- [ ] T014 Refactor shared test fixtures and error conversion without changing behavior; run fmt, focused tests, and protocol clippy. Record Refactor evidence in a distinct commit.

## Phase 3: Typed Encrypt and Decrypt protocol models

### Red

- [ ] T015 Add failing tests for a test-only OASIS XML fixture adapter in crates/kmipkit-test-support: pair request/response messages by sequence, preserve case/step identity, filter to Encrypt/Decrypt items before validating symbols, resolve only deterministic $NOW, $UNIQUE_IDENTIFIER_0, and $CORRELATION_VALUE tokens in those selected messages, reject unknown symbols there, and yield all 28 in-scope pairs (24 Encrypt, 4 Decrypt) with no skipped item. Do not validate symbols in out-of-scope setup/cleanup items. Add failing Table 214 Encrypt request tests for exact member order, optional fields, and every allowed Data encoding in crates/kmipkit-protocol/tests/unit/encrypt_tests.rs.
- [ ] T016 Add failing Table 196 Decrypt request tests for exact order, Decrypt-only Authenticated Encryption Tag, and all four fixture-derived Decrypt items from §2.101 in `crates/kmipkit-protocol/tests/unit/decrypt_tests.rs`.
- [ ] T017 Add failing success-payload tests for Tables 215 and 197; require Unique Identifier, preserve optional fields, allow UID-only success, preserve Byte String response Data, and cover all 24 Encrypt and 4 Decrypt fixture-derived response items.
- [ ] T018 Add failing failure-response tests proving no success Unique Identifier is required and shared Result Reasons outside Tables 198/216 remain valid; add a Pending-shape test distinguishing async and multipart correlation values.
- [ ] T019 Add malformed request/response tests for duplicate singleton fields, missing required success UID, wrong TTLV types, Data response Enumeration/Integer, and operation-inappropriate fields. Run focused tests and capture Red evidence.

### Green

- [ ] T020 Implement EncryptRequest/EncryptResponse wire conversion from Table 214/215.
- [ ] T021 Implement DecryptRequest/DecryptResponse wire conversion from Table 196/197.
- [ ] T022 Reuse the common operation result parser for Result Status, shared/unknown Result Reason, and Result Message; keep Pending in PendingOutcome.
- [ ] T023 Expose the typed models from kmipkit-protocol while retaining the full generic request/response tree and preserving all unknown values. Implement the OASIS XML adapter only in test support; add no production XML dependency.
- [ ] T024 Run focused protocol tests and confirm Green, including exact accounting for all 28 fixture-derived request/response pairs and the three documented symbolic substitutions.

### Refactor

- [ ] T025 Refactor common field parsing only where behavior remains identical; retain separate operation types and error messages with no payload data. Run fmt, protocol clippy, and focused tests; record Refactor evidence in its own commit.

## Phase 4: Multipart and batch integration

### Red

- [ ] T026 Add failing matrix tests for unframed single-part, framed single-part with Data, initial, middle, final, and framed single-part without Data. Confirm Data-present Init=true/Final=true is accepted; while KMIPKIT-DISC-045 remains open, only the Data-omission form returns a sanitized validation error before transport.
- [ ] T027 Add failing multipart tests proving the initial response Correlation Value appears unchanged on each subsequent/final request and AAD/Decrypt Tag stay on the initial request.
- [ ] T028 Add failing client-layer ID Placeholder tests: reject locally detectable ineligible shapes before send; encode a structurally eligible later item with the required Batch Order Option; and preserve the server's per-item result when an earlier operation fails at the server.
- [ ] T029 Run focused multipart/batch tests and capture Red evidence. Assert the typed client accepts Init=true/Final=true with Data and rejects only the ambiguous Data-omission form before encoding/transmission while the discrepancy remains open.

### Green

- [ ] T030 Implement the explicit supported multipart Data matrix and caller-provided indicator/correlation validation; accept a one-request Init=true/Final=true form with Data, and reuse the existing local validation-error path to gate only its Data-omission form until KMIPKIT-DISC-045 has an authoritative disposition; perform only one exchange per invocation.
- [ ] T031 Integrate Encrypt/Decrypt with KMIPKIT-0006 ID Placeholder/batch contracts; do not redefine batch ordering or Batch Order Option behavior.
- [ ] T032 Run multipart and batch tests Green, including a server-returned Correlation Value round trip.

### Refactor

- [ ] T033 Refactor request validation and client dispatch with no stream state, correlation synthesis, retry, or polling. Run focused tests and record Refactor evidence in a distinct commit.

## Phase 5: Client execution and outcome tests

### Red

- [ ] T034 Add failing fake-transport tests that execute all 28 OASIS fixture-derived Encrypt/Decrypt request-response pairs through the typed client, asserting each exact encoded request, its paired response, case/step association, one exchange, no retry, and delivery-state propagation. Also retain focused delivery-state cases for both operation types.
- [ ] T035 Add failing tests for success payloads, non-success server Result Reason/Message passthrough, PendingOutcome, malformed response handling, and local decoder limits.
- [ ] T036 Add sentinel tests ensuring plaintext, ciphertext, Data, AAD, AEAD Tag, IV, Correlation Value, and raw request/response bodies do not appear in Debug, Display, errors, or test diagnostics.
- [ ] T037 Run focused client tests and capture Red evidence.

### Green

- [ ] T038 Add Encrypt/Decrypt variants to the existing client request/operation/outcome pipeline and dispatch through current transport and response validation; make all 28 fixture-derived request/response pairs execute through the typed client fake transport.
- [ ] T039 Preserve completed failure results, success-only payload rules, PendingOutcome, and delivery classification without replay or automatic Poll.
- [ ] T040 Run protocol and client focused suites Green; confirm all 28 fixture-derived pairs execute through the typed client, then record command/output evidence.

### Refactor

- [ ] T041 Refactor shared execution/test support without widening the public API beyond the spec. Run fmt, clippy, focused tests, and rustdoc; record Refactor evidence separately.

## Phase 6: Fuzzing, traceability, docs, and release checks

- [ ] T042 Add or extend parser fuzz/property coverage for arbitrary Data variants, malformed request payloads, Result Reason values, and bounded decoding; verify no panic or secret formatting.
- [ ] T043 Update traceability.md with final implementation/test symbols, exact requirement status, source locators, and evidence; all 21 applicable client requirements must have 100% traceability.
- [ ] T044 Add executable English and Spanish Rust API examples for single-part and multipart Encrypt/Decrypt, caller-selected values, Pending handling, and explicit Recover-before-use; compile both examples as doctests or dedicated examples.
- [ ] T045 Execute all 28 in-scope Encrypt/Decrypt request-response pairs from the three pinned XML fixtures through the client test harness: 10 Encrypt pairs from §2.99, 10 Encrypt pairs from §2.100, and 4 Encrypt plus 4 Decrypt pairs from §2.101. Record case ID, fixture item, command, and result as fixture-derived operation-item evidence, never as a complete official-case pass. Run table-derived vectors and negative tests separately. Complete workflows remain a 1.0 interoperability gate after all operations in each scenario are supported.
- [ ] T046 Run cargo fmt --all --check, workspace clippy with all targets/features and -D warnings, all workspace tests, rustdoc/doctests, cargo llvm-cov, and API/ABI compatibility checks available for Rust scope. Enforce >=95% protocol/changed-code coverage and the existing >=90% workspace gate; record exact evidence.
- [ ] T047 Run Linux, Windows, and macOS CI; inspect dependency/license/security checks and ensure the immutable OASIS check passes.
- [ ] T048 Run speckit-converge, implement any added tasks, and repeat until no scope gap remains.
- [ ] T049 Obtain independent QA and security reviews; resolve blocking findings and update traceability/evidence.
- [ ] T050 Synchronize with release/1.0.0, rerun required gates, and prepare the KMIPKIT-0019 PR with Red/Green/Refactor commits, changes, evidence, risks, open discrepancies, and fixture-derived operation-item results. Do not merge or publish under this task file's repository policy.
