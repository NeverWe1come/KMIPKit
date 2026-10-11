# Tasks: KMIP 2.1 Object Lifecycle and Certificates

**Input**: Design documents from `specs/022-object-lifecycle/`.

**Prerequisites**: Approved [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/public-rust.md](contracts/public-rust.md), and [traceability.md](traceability.md).

**TDD and commit evidence**: Every operation story requires distinct Red test, Green implementation, and Refactor commits. Do not combine these phases. A Rust compile-time Red is acceptable only when the diagnostic is directly caused by the operation API that the paired Green task introduces; test syntax/type errors are not valid Red evidence. Record the exact focused command and expected diagnostic. PR evidence follows [quickstart.md](quickstart.md).

**Organization**: Tasks are grouped by the nine P1 operation stories. Operation model/test files are isolated; edits to `execute.rs`, public module exports, catalog, and generated report are serialized.

## Phase 1: Setup and prerequisite gate

**Purpose**: Confirm branch, immutable sources, shared foundation, and binding dependency boundary.

- [ ] T001 Confirm worktree is on `feature/KMIPKIT-0036-object-lifecycle`, based on active `release/1.0.0`, and record release base SHA and clean-worktree result in the implementation PR.
- [ ] T002 Verify pinned source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and confirm no change under `specification/oasis/kmip-2.1/upstream/`.
- [ ] T003 Confirm reusable TTLV/object/attribute/credential/result/batch/client/transport contracts in `crates/kmipkit-protocol/src/` and `crates/kmipkit-client/src/execute.rs`; keep language mapping under KMIPKIT-0043.

## Phase 2: Shared behavior baseline

**Purpose**: Confirm reusable common behavior and avoid duplicating tests for APIs that do not exist yet. Each operation story owns its feature-specific Red tests.

- [X] T004 Audit existing fake-transport coverage for common result delivery, Pending/correlation, batch association, one-exchange behavior, delivery-state failures, and no retries. Reuse examples in `crates/kmipkit-client/tests/unit/activate_execution_tests.rs`, `archive_execution_tests.rs`, and `create_execution_tests.rs`; add the nine operation-code assertions within each operation's Red task after its expected API is specified.
- [X] T005 Audit existing protocol coverage for unknown result reasons, unknown generic response children, and redacted malformed-success errors. Existing cases are `common_operation_result_tests::shared_helper_and_encrypt_decrypt_preserve_unknown_reason_and_result_message`, `create_tests::typed_response_conversion_leaves_unknown_generic_payload_fields_available`, and `destroy_operation_tests::malformed_success_errors_do_not_expose_payload_values`; do not add duplicate tests before operation response types exist.

## Phase 3: User Story 1 — Certify (Priority: P1)

**Goal**: Construct and send one typed Certify request and inspect its response.

**Independent Test**: Tables 179–181; conditional Certificate Request Type; one-certificate behavior; returned ID; one exchange; generic response retention.

- [ ] T006 [P] [US1] Add failing protocol and fake-client tests in `crates/kmipkit-protocol/tests/unit/certify_operation_tests.rs` and `crates/kmipkit-client/tests/unit/certify_execution_tests.rs` for §6.1.6 Tables 179–181 and all §6.1.6 entries in `specs/022-object-lifecycle/traceability.md`; commit Red.
- [ ] T007 [US1] Implement Certify models in `crates/kmipkit-protocol/src/certify.rs`, export in `crates/kmipkit-protocol/src/lib.rs`, and serialize its `ClientRequest`/operation-code/response/convenience integration in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T006 passes.
- [ ] T008 [US1] Refactor Certify model/dispatch/test helpers in `crates/kmipkit-protocol/src/certify.rs`, `crates/kmipkit-client/src/execute.rs`, and both Certify test files without changing wire behavior; commit Refactor separately.

## Phase 4: User Story 2 — Check (Priority: P1)

**Goal**: Send Check and inspect policy values returned by the server.

**Independent Test**: Tables 183–185; caller-selected Check fields and recommended ordered Stop/Undo options; no hidden policy evaluation.

- [ ] T009 [P] [US2] Add failing Check protocol/fake-client tests in `crates/kmipkit-protocol/tests/unit/check_operation_tests.rs` and `crates/kmipkit-client/tests/unit/check_execution_tests.rs`; cite §6.1.7 Tables 183–185 and its traceability rows; commit Red.
- [ ] T010 [US2] Implement `CheckRequest/Response/Error` in `crates/kmipkit-protocol/src/check.rs`, export it, and serialize shared dispatcher/client additions in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; keep batch choices explicit; commit Green after T009 passes.
- [ ] T011 [US2] Refactor Check model, dispatch, and tests in `crates/kmipkit-protocol/src/check.rs`, `crates/kmipkit-client/src/execute.rs`, and both Check test files; commit Refactor separately.

## Phase 5: User Story 3 — Obtain Lease (Priority: P1)

**Goal**: Return Obtain Lease values without hidden client state.

**Independent Test**: Tables 268–270; identifier, Lease Time, Last Change Date, zero-time value, common errors, no timer/cache.

- [ ] T012 [P] [US3] Add failing Obtain Lease tests in `crates/kmipkit-protocol/tests/unit/obtain_lease_operation_tests.rs` and `crates/kmipkit-client/tests/unit/obtain_lease_execution_tests.rs` for §6.1.35 tables and all its traceability rows; commit Red.
- [ ] T013 [US3] Implement `ObtainLeaseRequest/Response/Error` in `crates/kmipkit-protocol/src/obtain_lease.rs`, export it, and integrate dispatch/client methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T012 passes.
- [ ] T014 [US3] Refactor Obtain Lease model/dispatch/tests in `crates/kmipkit-protocol/src/obtain_lease.rs`, `crates/kmipkit-client/src/execute.rs`, and its protocol/client test files; commit Refactor separately.

## Phase 6: User Story 4 — Re-certify (Priority: P1)

**Goal**: Send one Re-certify request and inspect the replacement certificate result.

**Independent Test**: Tables 298–302; conditional request form, Offset/date behavior, response shape, and server-owned replacement links.

- [ ] T015 [P] [US4] Add failing Re-certify tests in `crates/kmipkit-protocol/tests/unit/recertify_operation_tests.rs` and `crates/kmipkit-client/tests/unit/recertify_execution_tests.rs` for §6.1.45 and its traceability rows; commit Red.
- [ ] T016 [US4] Implement `RecertifyRequest/Response/Error` in `crates/kmipkit-protocol/src/recertify.rs`, export it, and integrate the client dispatcher/method in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; keep link behavior server-side; commit Green after T015 passes.
- [ ] T017 [US4] Refactor Re-certify model/dispatch/tests in `crates/kmipkit-protocol/src/recertify.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 7: User Story 5 — Re-key (Priority: P1)

**Goal**: Send Re-key and inspect its replacement identifier.

**Independent Test**: Tables 303–307; explicit Offset/attributes; 12 linked Test Case IDs; no local key generation.

- [ ] T018 [P] [US5] Add failing Re-key tests in `crates/kmipkit-protocol/tests/unit/rekey_operation_tests.rs` and `crates/kmipkit-client/tests/unit/rekey_execution_tests.rs` for §6.1.46, including the catalog cases KMIPKIT-TEST-CN01-2-76 through -87 as available; commit Red.
- [ ] T019 [US5] Implement `RekeyRequest/Response/Error` in `crates/kmipkit-protocol/src/rekey.rs`, export it, and integrate dispatch/client methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T018 passes.
- [ ] T020 [US5] Refactor Re-key model/dispatch/tests in `crates/kmipkit-protocol/src/rekey.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 8: User Story 6 — Re-key Key Pair (Priority: P1)

**Goal**: Send Re-key Key Pair with separate attribute/mask groups and inspect both returned IDs.

**Independent Test**: Tables 308–312; common/private/public fields, result, one exchange, no local key-pair generation.

- [ ] T021 [P] [US6] Add failing Re-key Key Pair tests in `crates/kmipkit-protocol/tests/unit/rekey_key_pair_operation_tests.rs` and `crates/kmipkit-client/tests/unit/rekey_key_pair_execution_tests.rs` for §6.1.47 and its traceability rows; commit Red.
- [ ] T022 [US6] Implement `RekeyKeyPairRequest/Response/Error` in `crates/kmipkit-protocol/src/rekey_key_pair.rs`, export it, and integrate dispatch/client method in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T021 passes.
- [ ] T023 [US6] Refactor Re-key Key Pair model/dispatch/tests in `crates/kmipkit-protocol/src/rekey_key_pair.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 9: User Story 7 — Re-Provision (Priority: P1)

**Goal**: Request credentials in one of the three maintainer-approved forms.

**Independent Test**: Tables 313–315; CSR, certificate, or parameterless form; approved caption discrepancy; sensitive redaction; no follow-up Get.

- [ ] T024 [P] [US7] Add failing Re-Provision form/payload/result and fake-client tests in `crates/kmipkit-protocol/tests/unit/reprovision_operation_tests.rs` and `crates/kmipkit-client/tests/unit/reprovision_execution_tests.rs`; cite §6.1.48, KMIPKIT-DISC-040, and its traceability rows; commit Red.
- [ ] T025 [US7] Implement `ReProvisionRequest/Response/Error` in `crates/kmipkit-protocol/src/reprovision.rs`, export it, and integrate client dispatch/method in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T024 passes.
- [ ] T026 [US7] Refactor Re-Provision model/dispatch/tests in `crates/kmipkit-protocol/src/reprovision.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 10: User Story 8 — Register (Priority: P1)

**Goal**: Register a caller-provided object with caller-provided attributes.

**Independent Test**: Tables 291–294; required Object Type/Attributes/Any Object; algorithm/length/signature rules; unknown values; sensitive-object handling.

- [ ] T027 [P] [US8] Add failing Register tests in `crates/kmipkit-protocol/tests/unit/register_operation_tests.rs` and `crates/kmipkit-client/tests/unit/register_execution_tests.rs` for §6.1.43 and each of its traceability rows; commit Red.
- [ ] T028 [US8] Implement `RegisterRequest/Response/Error` in `crates/kmipkit-protocol/src/register.rs`, export it, and integrate client dispatch/method in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T027 passes.
- [ ] T029 [US8] Refactor Register model/dispatch/tests in `crates/kmipkit-protocol/src/register.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 11: User Story 9 — Revoke (Priority: P1)

**Goal**: Revoke an object with explicit reason and applicable compromise date.

**Independent Test**: Tables 295–297; open reason preservation, SHOULD recommendation, SHALL NOT date prohibition for other reasons, result passthrough.

- [ ] T030 [P] [US9] Add failing Revoke tests in `crates/kmipkit-protocol/tests/unit/revoke_operation_tests.rs` and `crates/kmipkit-client/tests/unit/revoke_execution_tests.rs` for §6.1.44 and its traceability rows; commit Red.
- [ ] T031 [US9] Implement `RevokeRequest/Response/Error` in `crates/kmipkit-protocol/src/revoke.rs`, export it, and integrate dispatcher/client methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T030 passes.
- [ ] T032 [US9] Refactor Revoke model/dispatch/tests in `crates/kmipkit-protocol/src/revoke.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 12: Cross-language parity, documentation, and release evidence

**Purpose**: Close binding, traceability, safety, quality, and release evidence without duplicating the manifest owner.

- [ ] T033 After KMIPKIT-0043's v2 manifest and canonical Rust mapping are available, complete its operation mapping tasks T043, T044, T056, and T058–T063 in specs/043-public-api-bindings/tasks.md; reconcile specs/043-public-api-bindings/contracts/operation-dependencies.md and specification/api/public-api.json, then verify generated declarations in `specification/api/`, `crates/kmipkit-ffi/`, `bindings/c/`, `bindings/java/`, and `bindings/python/` are generated, not hand-edited.
- [ ] T034 Add executable English and Spanish examples in `docs/user-guide/en/lifecycle-and-certificates.md` and `docs/user-guide/es/operaciones-ciclo-vida-y-certificados.md`; compile/run them using the approved guide mechanism and redact sample object material.
- [ ] T035 Update the mapping from all 44 ledger IDs and every mandatory/conditional payload row to code and tests in `specs/022-object-lifecycle/traceability.md` and `specification/catalog/kmip-2.1.json`.
- [ ] T036 Regenerate and verify `specification/catalog/coverage-report.md` and affected generated operation artifacts with pinned tools; do not edit generated output by hand.
- [ ] T037 Run focused operation tests, format, workspace Clippy/tests/coverage, dependency/security/parser/FFI checks, and supported-platform CI gates; record actual commands/results in `specs/022-object-lifecycle/quickstart.md`.
- [ ] T038 Obtain sequential independent QA and security reviews, address findings, update from current `release/1.0.0`, and prepare a draft PR with Red/Green/Refactor SHAs, traceability/catalog evidence, parity state, fixture limitations, risks, and test results.

## Dependencies & Execution Order

- T001–T003 precede implementation. T004–T005 confirm the shared foundation; every operation's own Red task precedes its paired Green task.
- In every operation story, Red precedes Green and Refactor follows Green as a separate commit.
- Operation-specific models and test files are independent. Client dispatcher, public exports, catalog, and generated-report integration are serialized.
- Rust protocol/client stories may proceed before KMIPKIT-0043 is complete. T033 waits for its canonical manifest and operation mapping tasks; that dependency applies to four-language parity only.
- Documentation and traceability closure follow operation behavior; catalog reports follow catalog edits.

## Parallel Opportunities

- Separate owners may author protocol Red tests and modules in disjoint files after the shared API conventions are fixed.
- Independent operation vectors and fake-transport tests can be reviewed in parallel.
- Do not parallel-edit `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-protocol/src/lib.rs`, `specification/catalog/kmip-2.1.json`, or its generated report.

## Implementation Strategy

Deliver each operation as a request/response/error slice with separate Red, Green, and Refactor commits. Begin with one story to validate execution integration, then add remaining P1 stories while serializing dispatcher updates. Keep language parity as a visible 0043 dependency. Finish with bilingual documentation, complete traceability, generated-output checks, coverage/security/platform evidence, and human-reviewed draft PR. Table-derived tests alone do not establish formal conformance, profile support, or certification.
