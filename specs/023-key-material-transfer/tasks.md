# Tasks: KMIP 2.1 Key Derivation and Transfer

**Input**: Design documents from `specs/023-key-material-transfer/`.

**Prerequisites**: Approved [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/public-rust.md](contracts/public-rust.md), and [traceability.md](traceability.md).

**TDD and commit evidence**: Each operation story requires a distinct Red test commit, Green implementation commit, and Refactor commit. Join Split Key tests include the approved 2026-10-11 threshold context rule. PR evidence follows [quickstart.md](quickstart.md).

**Organization**: Tasks are grouped by the four P1 stories. Operation payload files are independent; client dispatcher, public exports, catalog, and generated report are serialized.

## Phase 1: Setup and prerequisite gate

**Purpose**: Confirm the worktree, immutable sources, common types, and public API boundary.

- [x] T001 Confirm the managed worktree is on `feature/KMIPKIT-0037-key-material-transfer-implementation`, based on active `release/1.0.0` at `af172974cd5c6f56f7f938574e1546d4e295a9ef`, and record that the worktree was clean at creation; identify the occupied prescribed branch in implementation evidence.
- [x] T002 Verify source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and confirm no changes under `specification/oasis/kmip-2.1/upstream/`.
- [x] T003 Confirm shared object/attribute, generic TTLV, operation-data, Cryptographic Parameters, result, batch, client, and secret ownership APIs in `crates/kmipkit-protocol/src/` and `crates/kmipkit-client/src/execute.rs`; reserve four-language mappings for KMIPKIT-0043.

## Phase 2: Shared execution and secret contract tests

**Purpose**: Establish common failing tests before production dispatch/model changes.

- [ ] T004 [P] Add failing protocol tests in `crates/kmipkit-protocol/tests/unit/key_transfer_operation_contract_tests.rs` for open result/error values, unknown child retention, secret redaction, and owned-byte handling; commit Red evidence.
- [ ] T005 [P] Add failing fake-transport tests in `crates/kmipkit-client/tests/unit/key_transfer_operations_execution_tests.rs` for all four operation codes, one exchange, result/batch association, Pending, delivery-state failures, and no retry; commit before implementation.

## Phase 3: User Story 1 — Derive Key (Priority: P1)

**Goal**: Request server-side derivation with caller-selected input, method, algorithm, length, and output attributes.

**Independent Test**: Tables 205–207 and §7.12/Table 364; ID Placeholder prohibition; explicit source IDs and algorithm details; one exchange and no local derivation.

- [x] T006 [P] [US1] Add failing Derive Key payload and fake-client tests in `crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs` and `crates/kmipkit-client/tests/unit/derive_key_execution_tests.rs` for §6.1.14 Tables 205–207, §7.12 Table 364 and traceability rows, plus the both-inputs server-error passthrough without local pre-rejection; commit Red.
- [x] T007 [US1] Implement `DeriveKeyRequest/Response/Error` in `crates/kmipkit-protocol/src/derive_key.rs`, export it, and integrate `ClientRequest`/operation code/response/convenience dispatch in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T006 passes.
- [ ] T008 [US1] Refactor Derive Key payload/dispatch/tests in `crates/kmipkit-protocol/src/derive_key.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files without changing caller-controlled semantics; commit Refactor separately.

## Phase 4: User Story 2 — Export (Priority: P1)

**Goal**: Request a server-side export with explicit optional format/wrapping/compression choices.

**Independent Test**: Tables 217–219; all response object/attribute fields; secret redaction; no local transformation.

- [ ] T009 [P] [US2] Add failing Export request, success/error response, and fake-client tests in `crates/kmipkit-protocol/tests/unit/export_operation_tests.rs` and `crates/kmipkit-client/tests/unit/export_execution_tests.rs` for §6.1.18 Tables 217–219 and its traceability rows; commit Red.
- [ ] T010 [US2] Implement `ExportRequest/Response/Error` in `crates/kmipkit-protocol/src/export.rs`, export it, and integrate dispatch/client convenience methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T009 passes.
- [ ] T011 [US2] Refactor Export model/dispatch/tests in `crates/kmipkit-protocol/src/export.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 5: User Story 3 — Import (Priority: P1)

**Goal**: Send an object and caller-provided attributes for server-side import, with replacement only by explicit intent.

**Independent Test**: Tables 238–240; required identifier/type/attributes/object; conditional wrapping; explicit Replace Existing; one exchange; no local unwrap/import.

- [ ] T012 [P] [US3] Add failing Import request, response/error, and fake-client tests in `crates/kmipkit-protocol/tests/unit/import_operation_tests.rs` and `crates/kmipkit-client/tests/unit/import_execution_tests.rs` for §6.1.25 Tables 238–240 and its traceability rows; commit Red.
- [ ] T013 [US3] Implement `ImportRequest/Response/Error` in `crates/kmipkit-protocol/src/import.rs`, export it, and integrate dispatch/client methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T012 passes.
- [ ] T014 [US3] Refactor Import payload/dispatch/tests in `crates/kmipkit-protocol/src/import.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files without local unwrapping; commit Refactor separately.

## Phase 6: User Story 4 — Join Split Key (Priority: P1)

**Goal**: Combine an ordered set of caller-identified split keys after validating the caller-known threshold context.

**Independent Test**: Tables 244–246; one caller-known threshold per identifier occurrence; ID order/repetitions preserved; entry count >= maximum threshold before network transmission; no lookup; one exchange.

- [ ] T015 [P] [US4] Add failing Join Split Key protocol and fake-client tests in `crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs` and `crates/kmipkit-client/tests/unit/join_split_key_execution_tests.rs` for missing/misaligned threshold context, count below/equal/above max, different thresholds, repeated field/value entries, exact wire ID order, and §6.1.27 Tables 244–246; commit Red.
- [ ] T016 [US4] Implement `SplitKeyReference` and `JoinSplitKeyRequest/Response/Error` in `crates/kmipkit-protocol/src/join_split_key.rs`; require explicit non-wire per-entry threshold context, validate request entry count >= maximum threshold, preserve all ID occurrences, and integrate client dispatch in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T015 passes.
- [ ] T017 [US4] Refactor Join Split Key model/validation/dispatch/tests in `crates/kmipkit-protocol/src/join_split_key.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; preserve the 2026-10-11 interpretation and commit Refactor separately.

## Phase 7: Cross-language parity, documentation, and release evidence

**Purpose**: Complete manifest-backed public mapping, traceability, docs, generated outputs, and release evidence.

- [ ] T018 After KMIPKIT-0043's v2 manifest and canonical Rust mapping are available, complete its operation mapping tasks T046, T047, T050, and T052 in `specs/043-public-api-bindings/tasks.md`; reconcile `specs/043-public-api-bindings/contracts/operation-dependencies.md` and `specification/api/public-api.json`, then verify generated declarations in `crates/kmipkit-ffi/`, `bindings/c/`, `bindings/java/`, and `bindings/python/` are not hand-edited.
- [ ] T019 Add executable English and Spanish examples in `docs/user-guide/en/key-derivation-and-transfer.md` and `docs/user-guide/es/derivacion-y-transferencia-de-claves.md`; compile/run them with the approved guide mechanism and use synthetic redacted material.
- [ ] T020 Update traceability for all 18 stable ledger IDs and every mandatory/conditional payload-table row in `specs/023-key-material-transfer/traceability.md` and `specification/catalog/kmip-2.1.json`.
- [ ] T021 Regenerate and validate `specification/catalog/coverage-report.md` and affected generated operation artifacts using pinned tools; do not edit generated outputs manually.
- [ ] T022 Run focused operation tests, formatter, workspace Clippy/tests/coverage, dependency/security/parser/FFI checks, and supported-platform CI; record actual commands/results in `specs/023-key-material-transfer/quickstart.md`.
- [ ] T023 Obtain sequential independent QA/security reviews, address findings, update from current `release/1.0.0`, and prepare a draft PR with Red/Green/Refactor SHAs, traceability/catalog and generator evidence, parity status, fixture limitations, risks, and verification results.

## Dependencies & Execution Order

- T001–T003 precede implementation; T004–T005 establish shared Red evidence first.
- Each story's Red task precedes Green; its Refactor commit follows the passing Green state.
- Four Rust operation model/test slices are independent. Dispatcher, protocol exports, catalog, and generated report are shared serialized edits.
- Rust operations may start without KMIPKIT-0043. T018 waits for the canonical manifest and its operation-specific mapping work; that gate applies to four-language parity only.
- Documentation, traceability, generator, and release checks follow operation behavior.

## Parallel Opportunities

- Protocol Red tests and model work for distinct operations can be prepared on separate files after shared API conventions are fixed.
- Derive Key, Export, Import, and Join Split Key table vectors are operation-specific.
- Do not parallel-edit `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-protocol/src/lib.rs`, `specification/catalog/kmip-2.1.json`, or the generated coverage report.

## Implementation Strategy

Deliver each operation as an independently testable typed request/response/error slice with distinct Red, Green, and Refactor commits. Start with Derive Key to exercise reuse of operation-data and Cryptographic Parameters, then complete other operations while serializing shared client integration. Join Split Key's context/threshold contract is fixed by the approved 2026-10-11 interpretation. Close with 0043 parity mappings, bilingual docs, complete requirement/table traceability, generated outputs, coverage/security/platform evidence, and a human-reviewed draft PR. Do not equate source-derived table vectors with official fixture passes or formal profile/certification.
