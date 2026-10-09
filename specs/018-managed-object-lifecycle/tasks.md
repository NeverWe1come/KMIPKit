# Tasks: KMIP 2.1 Managed-Object State Transitions

**Input**: Design documents from `specs/018-managed-object-lifecycle/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/public-rust.md`, `quickstart.md`

**Implementation gate**: Do not start code until this specification is approved and merged. Keep Red, Green, and Refactor evidence in separate commits.

## Phase 1: Setup and normative traceability

- [x] T001 Assign Activate, Archive, Destroy, Recover, and the three applicable client requirements to KMIPKIT-0018 in `specification/catalog/kmip-2.1.json`.
- [x] T002 Add the Archive and Recover requirement-to-operation links and retain the server-only classification for Activate and Destroy clauses in `specification/catalog/kmip-2.1.json`.
- [x] T003 Regenerate `specification/catalog/coverage-report.md` with the pinned catalog generator and verify the report with its `--check` mode.
- [x] T004 Complete the planned traceability rows in `specs/018-managed-object-lifecycle/traceability.md` with catalog IDs, concrete source tables, planned code paths, and planned executable tests.

---

## Phase 2: Foundational shared test support

**Purpose**: Reuse the existing client execution and fake-transport contracts; add only lifecycle-specific fixture support needed by all stories.

- [x] T005 [P] Add shared lifecycle TTLV fixtures in `crates/kmipkit-protocol/tests/support/lifecycle_fixtures.rs` for optional identifiers, successful response identifiers, and malformed payloads.
- [x] T006 Confirm public exports and operation-dispatch extension points in `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit-client/src/lib.rs` without changing existing behavior.

---

## Phase 3: User Story 1 — Activate and Destroy (Priority: P1)

**Goal**: Send either explicit operation and return the correctly typed result without simulating remote object state.

**Independent Test**: Protocol vectors validate both request/response table shapes; fake transport validates one exchange, response association, error preservation, malformed-success rejection, permitted Pending, and no local state changes.

### Tests first (Red)

- [x] T007 [P] [US1] Add Activate request/response and malformed-payload tests in `crates/kmipkit-protocol/tests/unit/activate_operation_tests.rs`.
- [x] T008 [P] [US1] Add Destroy request/response and malformed-payload tests in `crates/kmipkit-protocol/tests/unit/destroy_operation_tests.rs`.
- [x] T009 [P] [US1] Add Activate fake-transport execution tests in `crates/kmipkit-client/tests/unit/activate_execution_tests.rs`.
- [x] T010 [P] [US1] Add Destroy fake-transport execution tests in `crates/kmipkit-client/tests/unit/destroy_execution_tests.rs`.
- [x] T011 [US1] Run the focused Activate and Destroy tests and record expected Red failures before implementation.

### Implementation (Green)

- [x] T012 [P] [US1] Implement the Activate typed request/response model in `crates/kmipkit-protocol/src/activate.rs` and export it from `crates/kmipkit-protocol/src/lib.rs`.
- [x] T013 [P] [US1] Implement the Destroy typed request/response model in `crates/kmipkit-protocol/src/destroy.rs` and export it from `crates/kmipkit-protocol/src/lib.rs`.
- [x] T014 [US1] Add Activate and Destroy request, response, and operation dispatch to `crates/kmipkit-client/src/execute.rs` and public client exports in `crates/kmipkit-client/src/lib.rs`.
- [ ] T015 [US1] Run focused protocol and client tests and record Green evidence for Activate and Destroy.

### Refactor

- [ ] T016 [US1] Refactor shared lifecycle encoding or response helpers only where they remove duplication, keep operation modules distinct, and rerun the focused tests.

---

## Phase 4: User Story 2 — Archive (Priority: P2)

**Goal**: Allow the caller to request archival while describing the result as a server response, not proof that archival completed.

**Independent Test**: Archive vectors verify optional request and required response identifiers; fake transport verifies Success, Failure, permitted Pending, one exchange, and no archival claim.

### Tests first (Red)

- [ ] T017 [P] [US2] Add Archive request/response and malformed-payload tests in `crates/kmipkit-protocol/tests/unit/archive_operation_tests.rs`.
- [ ] T018 [P] [US2] Add Archive fake-transport execution tests in `crates/kmipkit-client/tests/unit/archive_execution_tests.rs`.
- [ ] T019 [US2] Run focused Archive tests and record expected Red failures before implementation.

### Implementation (Green)

- [ ] T020 [US2] Implement the Archive typed request/response model in `crates/kmipkit-protocol/src/archive.rs` and export it from `crates/kmipkit-protocol/src/lib.rs`.
- [ ] T021 [US2] Add Archive request, response, and operation dispatch to `crates/kmipkit-client/src/execute.rs` and public client exports in `crates/kmipkit-client/src/lib.rs`.
- [ ] T022 [US2] Run focused Archive tests and record Green evidence.

### Refactor

- [ ] T023 [US2] Refactor Archive code only where justified by established lifecycle patterns and rerun its focused tests.

---

## Phase 5: User Story 3 — Recover (Priority: P2)

**Goal**: Return a typed completed result or the shared Pending result so callers control Poll and Get follow-ups.

**Independent Test**: Recover vectors validate payload shape; fake transport verifies exact correlation bytes, one exchange, delivery evidence, operation errors, malformed success, and that no Poll/Get/retry is issued automatically.

### Tests first (Red)

- [ ] T024 [P] [US3] Add Recover request/response and malformed-payload tests in `crates/kmipkit-protocol/tests/unit/recover_operation_tests.rs`.
- [ ] T025 [P] [US3] Add Recover fake-transport success, failure, Pending, delivery-state, and no-follow-up tests in `crates/kmipkit-client/tests/unit/recover_execution_tests.rs`.
- [ ] T026 [US3] Run focused Recover tests and record expected Red failures before implementation.

### Implementation (Green)

- [ ] T027 [US3] Implement the Recover typed request/response model in `crates/kmipkit-protocol/src/recover.rs` and export it from `crates/kmipkit-protocol/src/lib.rs`.
- [ ] T028 [US3] Add Recover request, response, and operation dispatch to `crates/kmipkit-client/src/execute.rs` and public client exports in `crates/kmipkit-client/src/lib.rs`.
- [ ] T029 [US3] Run focused Recover tests and record Green evidence.

### Refactor

- [ ] T030 [US3] Refactor shared lifecycle response handling only where justified, preserve exact correlation bytes, and rerun focused tests.

---

## Phase 6: Polish and cross-cutting verification

- [ ] T031 [P] Add English lifecycle usage documentation in `docs/user-guide/en/lifecycle-operations.md` with tested request and result examples.
- [ ] T032 [P] Add Spanish lifecycle usage documentation in `docs/user-guide/es/operaciones-ciclo-vida.md` with equivalent behavior and limitations.
- [ ] T033 [P] Update the public Rust API reference in `docs/architecture/public-api.md` and API examples to list all four operations and their limits.
- [ ] T034 Complete `specs/018-managed-object-lifecycle/traceability.md` with final source, implementation, and passing test references for every applicable client requirement.
- [ ] T035 Run the pinned catalog validation, `cargo fmt --all --check`, focused tests, workspace Clippy, workspace tests, and workspace coverage checks; record platform and coverage results in the PR.
- [ ] T036 Run the documented quickstart examples for all four operations and correct any example that does not compile or match the public API.
- [ ] T037 Add regression tests that lifecycle request/response values and raw KMIP bodies never appear in public error or Debug output in `crates/kmipkit-client/tests/unit/lifecycle_redaction_tests.rs`.
- [ ] T038 Add tests that lifecycle responses preserve supported unknown result values and accepted generic extension data in `crates/kmipkit-client/tests/unit/lifecycle_execution_tests.rs`.

## Dependencies and execution order

- Setup and normative traceability tasks T001–T004 precede implementation; T003 depends on T001–T002.
- Shared fixture task T005 may proceed alongside catalog updates; T006 is a read-only design confirmation.
- US1, US2, and US3 share the existing execution foundation and can be implemented independently after approval. Within each story, tests must fail before its implementation begins.
- T014, T021, and T028 touch the same client dispatch file and must be applied serially.
- Documentation, final traceability, catalog validation, and full checks follow all three stories.

## Implementation strategy

Deliver the P1 Activate/Destroy slice first, then Archive and Recover. Keep the three operation families independently tested, and keep all changes on the approved feature branch. No C, Java, Python, JSON/XML, server-initiated operation, or transport changes belong to this specification.
