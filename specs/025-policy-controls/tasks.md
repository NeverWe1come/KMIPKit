---
description: "Implementation tasks for KMIP 2.1 Policy and Usage Controls"
---

# Tasks: KMIP 2.1 Policy and Usage Controls

**Input**: Design documents in `specs/025-policy-controls/`.

**Prerequisites**: Approved `spec.md`, `plan.md`, `research.md`, `data-model.md`, `traceability.md`, and `contracts/`.

**Tests**: Required by the approved acceptance criteria and `AGENTS.md`. Each operation and the usage guard have distinct Red, Green, and Refactor evidence commits. The listed official fixture inventory has no direct OASIS case for these operations; derive and label tests as project tests.

**Organization**: Tasks are grouped by the five P1 user stories. Shared client dispatcher edits are integrated serially. Cross-language parity follows the approved KMIPKIT-0043 manifest contract.

## Format and path conventions

Each task uses `[ ] T### [US#] action in exact/repository/path`. Every implementation story has explicit Red, Green, and Refactor commits. Unit test modules live under `tests/unit/` and are registered through the existing crate test module declarations (`crates/kmipkit-protocol/src/lib.rs` or `crates/kmipkit-client/src/lib.rs`/`execute.rs`).

## Phase 1: Setup

**Purpose**: Confirm the approved source and release baseline. No project initialization or new runtime dependency is needed.

- [ ] T001 Verify source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256`; compare the pinned §6.1.22, §6.1.23, §6.1.52, §6.1.53, §6.1.54, §4.59, §7.7, §7.11, §11.39, and §11.60 source passages with `specs/025-policy-controls/spec.md` and `traceability.md`; confirm the approved release baseline and record exact commands before coding.

## Phase 2: User Story 1 — Get Constraints (Priority: P1)

**Goal**: Send the empty §6.1.22 request and return its required Constraints structure and common result without losing extensions.

**Independent Test**: A fake transport sees exactly one Table 229 request; Table 230 success returns required Constraints, and Table 231 failure preserves result metadata and delivery.

### Red commit

- [ ] T002 [US1] Add failing protocol and fake-client tests in `crates/kmipkit-protocol/tests/unit/get_constraints_tests.rs` and `crates/kmipkit-client/tests/unit/get_constraints_execution_tests.rs`; register modules in `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit-client/src/lib.rs`; cite OASIS KMIP Specification v2.1 §6.1.22, Tables 229–231 in test documentation; cover empty request, required ordered Constraints, unknown nested members, malformed response, Table 231 reasons, Pending/correlation, one exchange, NotSent validation, redaction, and no retry; record failing command/output in a separate Red commit.

### Green commit

- [ ] T003 [US1] Implement `GetConstraintsRequest`/`GetConstraintsResponse` in `crates/kmipkit-protocol/src/get_constraints.rs`, export from `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit/src/lib.rs`, and add the typed request/response path in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T002 and commit separately as Green.

### Refactor commit

- [ ] T004 [US1] Refactor Get Constraints encoding, required-field validation, and focused assertions in `crates/kmipkit-protocol/src/get_constraints.rs`, `crates/kmipkit-protocol/tests/unit/get_constraints_tests.rs`, and `crates/kmipkit-client/tests/unit/get_constraints_execution_tests.rs` without changing wire behavior; rerun focused tests and commit separately as Refactor.

**Checkpoint**: Empty request and required Constraints result satisfy Tables 229–231.

## Phase 3: User Story 2 — Get Usage Allocation and explicit guard (Priority: P1)

**Goal**: Implement §6.1.23 allocation exchange and the approved local caller-supplied guard for known Usage-Limited protection use.

**Independent Test**: The fake transport verifies Tables 232–234; guard tests prove object/unit/cost boundaries and delivery-state reservation behavior without claiming remote consumption.

### Allocation Red commit

- [ ] T005 [US2] Add failing model/fake-client tests in `crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs` and `crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs`; register modules in `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit-client/src/lib.rs`; cite OASIS KMIP Specification v2.1 §6.1.23, Tables 232–234 in test documentation; cover optional identifier, required Usage Limits Count Integer, required response identifier, Table 234 failures, Pending, unknown result values, one exchange, and no retry; record failing output in a separate Red commit.

### Allocation Green commit

- [ ] T006 [US2] Implement `GetUsageAllocationRequest`/`GetUsageAllocationResponse` in `crates/kmipkit-protocol/src/get_usage_allocation.rs`, export through `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit/src/lib.rs`, and wire typed execution/results in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T005 and commit separately as Green.

### Allocation Refactor commit

- [ ] T007 [US2] Refactor allocation request/response field handling and tests in `crates/kmipkit-protocol/src/get_usage_allocation.rs`, `crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs`, and `crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs`; preserve Integer wire type, optionality, and response ID; rerun focused tests and commit separately as Refactor.

### Guard Red commit

- [ ] T008 [US2] Add failing `UsageAllocationContext` tests in `crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs` and a compile-fail fixture in `crates/kmipkit-client/tests/ui/usage_allocation_guard_boundary.rs` exercised by that module's trybuild harness; cite OASIS KMIP Specification v2.1 §§4.59 and 6.1.23, Tables 148 and 232–233 plus §11.60, Table 491; cover context construction only from complete Success, caller-supplied count/unit, returned/resolved object ID, non-cloneability, Encrypt/MAC/Sign scope, positive cost, exact-budget acceptance, over-budget/missing/placeholder/mismatched ID rejection with NotSent and no transport, and no accounting for Decrypt/Verify/Hash or ordinary/raw/external calls; record Red output separately.

### Guard Green commit

- [ ] T009 [US2] Implement the non-cloneable caller-owned context and guarded one-item typed protection path in `crates/kmipkit-client/src/usage_allocation.rs`, `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-client/src/lib.rs`, and `crates/kmipkit/src/lib.rs`; expose only explicit unit/cost inputs, preserve open values, reject invalid context/cost/ID before send, and make T008 pass; commit separately as Green.

### Guard Refactor commit

- [ ] T010 [US2] Refactor checked local reservation accounting and delivery-state transitions in `crates/kmipkit-client/src/usage_allocation.rs`, `crates/kmipkit-client/src/execute.rs`, and `crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs`; prove `NotSent` releases while `PossiblySent`, `ResponseStarted`, complete Success/Failure, and Pending retain locally, rerun focused tests, and commit separately as Refactor.

**Checkpoint**: A successful allocation may seed one explicit local context; guarded requests cannot exceed its caller-known local allowance. No remote usage is inferred.

## Phase 4: User Story 3 — Set Constraints (Priority: P1)

**Goal**: Send required Constraints using approved KMIPKIT-DISC-020 interpretation and preserve the common response.

**Independent Test**: A fake transport captures Table 325 and returns empty Table 326 success or Table 327 failure.

### Red commit

- [ ] T011 [US3] Add failing protocol/client tests in `crates/kmipkit-protocol/tests/unit/set_constraints_tests.rs` and `crates/kmipkit-client/tests/unit/set_constraints_execution_tests.rs`; register modules through protocol/client test module declarations; cite OASIS KMIP Specification v2.1 §6.1.52, Tables 325–327; cover required Constraints, order/unknown nested data, approved DISC-020 error assignment, empty success, malformed response, Pending, delivery state, and no retry; record Red output separately.

### Green commit

- [ ] T012 [US3] Implement `SetConstraintsRequest`/`SetConstraintsResponse` in `crates/kmipkit-protocol/src/set_constraints.rs`, export via protocol/facade `lib.rs`, and add typed execution in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T011 and commit separately as Green.

### Refactor commit

- [ ] T013 [US3] Refactor Set Constraints encoding and Table 327 result tests in `crates/kmipkit-protocol/src/set_constraints.rs`, `crates/kmipkit-protocol/tests/unit/set_constraints_tests.rs`, and `crates/kmipkit-client/tests/unit/set_constraints_execution_tests.rs` without changing Table 325 behavior; rerun focused tests and commit separately as Refactor.

## Phase 5: User Story 4 — Set Defaults (Priority: P1)

**Goal**: Preserve the semantic difference between absent Defaults Information (remove all Object Defaults) and a supplied Structure.

**Independent Test**: Fake transport verifies Table 328 omission vs presence and empty Table 329 success or Table 330 error.

### Red commit

- [ ] T014 [US4] Add failing protocol/client tests in `crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs` and `crates/kmipkit-client/tests/unit/set_defaults_execution_tests.rs`; register modules through protocol/client test module declarations; cite OASIS KMIP Specification v2.1 §6.1.53, Tables 328–330 and §7.11; cover absent vs empty/present Defaults Information, unknown nested elements, approved DISC-020 interpretation, Table 330 errors, malformed payload, Pending, delivery, and no retry; record Red output separately.

### Green commit

- [ ] T015 [US4] Implement `SetDefaultsRequest`/`SetDefaultsResponse` in `crates/kmipkit-protocol/src/set_defaults.rs`, export via protocol/facade `lib.rs`, and add explicit execution in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; preserve omission as omission, pass T014, and commit separately as Green.

### Refactor commit

- [ ] T016 [US4] Refactor Set Defaults request/result handling in `crates/kmipkit-protocol/src/set_defaults.rs`, `crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs`, and `crates/kmipkit-client/tests/unit/set_defaults_execution_tests.rs` while preserving absent-vs-empty behavior; rerun focused tests and commit separately as Refactor.

## Phase 6: User Story 5 — Set Endpoint Role (Priority: P1)

**Goal**: Expose only client-to-server §6.1.54 using open Endpoint Role enumeration values.

**Independent Test**: Fake transport captures required Table 331 role and returns required Table 332 role or Table 333 failure.

### Red commit

- [ ] T017 [US5] Add failing protocol/client tests in `crates/kmipkit-protocol/tests/unit/set_endpoint_role_tests.rs` and `crates/kmipkit-client/tests/unit/set_endpoint_role_execution_tests.rs`; register modules via protocol/client test module declarations; cite OASIS KMIP Specification v2.1 §6.1.54, Tables 331–333 and §11.39; cover required fields, open known/unknown/vendor Endpoint Role values, response requiredness, Table 333 errors, malformed payload, Pending, delivery, one exchange, and exclusion of server-initiated §6.2.5; record Red output separately.

### Green commit

- [ ] T018 [US5] Implement `SetEndpointRoleRequest`/`SetEndpointRoleResponse` in `crates/kmipkit-protocol/src/set_endpoint_role.rs`, export via protocol/facade `lib.rs`, and integrate the client-to-server typed path in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T017 and commit separately as Green.

### Refactor commit

- [ ] T019 [US5] Refactor Endpoint Role open-value handling in `crates/kmipkit-protocol/src/set_endpoint_role.rs`, `crates/kmipkit-protocol/tests/unit/set_endpoint_role_tests.rs`, and `crates/kmipkit-client/tests/unit/set_endpoint_role_execution_tests.rs` without narrowing unknown values or adding §6.2.5; rerun focused tests and commit separately as Refactor.

## Phase 7: Traceability, parity, and release evidence

**Purpose**: Complete catalog traceability, cross-language capability, examples, interop record, and quality gates.

- [ ] T020 Update `specification/catalog/kmip-2.1.json` with feature ID, implementation/verification references for all operation/tag/structure/enum elements and all four §6.1.23 client requirements; update `specs/025-policy-controls/traceability.md`, then run `python tools/normative_catalog/validate.py`, `python tools/normative_catalog/report.py --write`, and `--check`; do not edit generated report output by hand.
- [ ] T021 Add failing C ABI and Java/Python parity tests for all five operations and the explicit caller guard in `bindings/c/tests/policy_controls.c`, `bindings/java/src/test/java/org/kmipkit/PolicyControlsParityTest.java`, `bindings/python/tests/test_policy_controls.py`, and `crates/kmipkit-ffi/tests/c_api_coverage.rs`; register the C test in `bindings/c/CMakeLists.txt`; do not begin until KMIPKIT-0043's manifest contract is approved and merged; record a separate Red evidence commit.
- [ ] T022 Add approved entries to `specification/api/public-api.json`, implement C/JNI/Java/Python adapters in `crates/kmipkit-ffi/src/lib.rs`, `bindings/java/native/kmipkit_jni.cpp`, `bindings/java/src/main/java/org/kmipkit/PolicyControls.java`, and `bindings/python/src/kmipkit/policy_controls.py`; regenerate C header and language metadata with `python tools/api_manifest/generate.py --write`; pass T021 and commit the manifest/adapters/generated outputs together as separate Green evidence.
- [ ] T023 Refactor language parity adapters/tests in `crates/kmipkit-ffi/src/lib.rs`, `bindings/java/native/kmipkit_jni.cpp`, `bindings/java/src/main/java/org/kmipkit/PolicyControls.java`, `bindings/python/src/kmipkit/policy_controls.py`, and the parity test files without changing the manifest contract; run `python tools/api_manifest/generate.py --check` and binding tests, then commit separately as Refactor.
- [ ] T024 Add executable English/Spanish operation and guard examples in `docs/user-guide/en/policy-controls.md` and `docs/user-guide/es/controles-de-uso.md`; compile/verify examples through `crates/kmipkit-protocol/tests/policy_controls_guide_examples.rs`; document caller cost/unit responsibility and local-only reservation limits without claiming conformance or remote consumption.
- [ ] T025 Update `docs/development/kmip-interoperability.md` with operation rows and the exact integration evidence fields; leave results Pending until measured against identified server/version configurations; do not claim support based only on safe rejection.
- [ ] T026 Run all focused tests and commands in `specs/025-policy-controls/quickstart.md`, plus `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, and `cargo llvm-cov --workspace --all-features`; run API manifest unit tests, generated checks, relevant security checks, and both independent KMIP server integration gates where available; record exact outcomes and limitations in the PR.

## Dependencies and execution order

- T001 precedes implementation; it verifies immutable sources and the release baseline.
- US1 (T002–T004) establishes the first shared dispatcher integration.
- US2 allocation (T005–T007) follows US1 because `execute.rs` and crate exports are shared. The guard (T008–T010) follows allocation and consumes its resolved success result.
- US3, US4, and US5 follow the prior shared client integration sequence to avoid concurrent conflicting edits to `crates/kmipkit-client/src/execute.rs`, client exports, and protocol exports. Their payload models and isolated tests remain separable after their contracts are stable.
- T020–T026 depend on all five operation stories and guard. Cross-language T021–T023 additionally require KMIPKIT-0043's approved manifest contract/merge; this does not block T002–T020 Rust implementation or the documentation prep.
- T025 and external integration evidence can be prepared in parallel with parity after the Rust behavior is stable, but the interop matrix remains Pending until real measurements are recorded.

## Independent acceptance criteria

- **US1**: Tables 229–231, including empty request, required Constraints, unknown nested fields, error reasons, Pending, and one-exchange behavior.
- **US2 allocation**: Tables 232–234, exact Integer count, optional/requested identifier behavior, required resolved response ID, and common result/delivery behavior.
- **US2 guard**: successful result plus explicit exact count/unit/ID context; positive caller cost only; matching ID; no over-budget transmission; delivery reservations; local-only accounting.
- **US3**: Tables 325–327 and approved KMIPKIT-DISC-020 disposition.
- **US4**: Tables 328–330; absent Defaults Information remains absent and means removal under the approved interpretation.
- **US5**: Tables 331–333; unknown Endpoint Role values preserved; no server-initiated §6.2.5 behavior.
