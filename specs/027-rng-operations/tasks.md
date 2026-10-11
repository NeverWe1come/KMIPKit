---
description: "Implementation task list for KMIP 2.1 remote RNG operations"
---

# Tasks: KMIP 2.1 Remote RNG Operations

**Input**: Design documents from specs/027-rng-operations/

**Prerequisites**: Approved spec.md; plan.md, research.md, data-model.md, contracts/rust-client.md, traceability.md, quickstart.md

**Testing**: Required by the approved independent acceptance scenarios and repository TDD rules.

**Organization**: Separate user-story phases for RNG Retrieve and RNG Seed.

## Phase 1: Setup

**Purpose**: Confirm sources and existing operation conventions.

- [x] T001 [P] Verify the assigned feature branch and pinned OASIS §§6.1.49–6.1.50 Tables 316–321 in specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html; run python tools/normative_catalog/validate.py.
- [x] T002 [P] Review shared Data, Data Length, asynchronous result, secret ownership, and client test patterns in crates/kmipkit-protocol/src/operation_data.rs, crates/kmipkit-protocol/src/poll.rs, crates/kmipkit-client/src/execute.rs, and crates/kmipkit-client/tests/unit/ping_execution_tests.rs.

## Phase 2: Foundational

**Purpose**: Establish exact catalog and traceability inputs before coding.

- [x] T003 Map both operation elements, enum values, Table fields, errors, and KMIPKIT-REQ-SPEC-6.1.50-001 to source, implementation path, and named tests in specs/027-rng-operations/traceability.md.
- [x] T004 Verify Data Length and Data records and existing operation values in specification/catalog/kmip-2.1.json; add a catalog source record only if absent and regenerate with the pinned tool.
- [x] T005 Record shared Data field ownership and preserve its current catalog feature assignment in specs/027-rng-operations/traceability.md.

## Phase 3: User Story 1 — RNG Retrieve (Priority: P1)

**Goal**: Request random output by the required Data Length and inspect the required returned Data without local RNG behavior.

**Independent Test**: Fake transport observes one Table 316 request and returns Table 317 success or Table 318 failure; tests verify output bytes, result/delivery, redaction, and no retry.

### Protocol Red/Green/Refactor

- [x] T006 [P] [US1] Add failing Table 316/317 tests in crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs for required Integer Data Length, required opaque Data, unknown-field round trip, malformed types, and secret redaction; run cargo test -p kmipkit-protocol rng_retrieve_operation_tests and capture expected Red failure.
- [x] T007 [US1] Commit only the failing protocol tests in crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs as the distinct Red evidence commit.
- [x] T008 [US1] Implement RngRetrieveRequest/Response/Error and table conversion in crates/kmipkit-protocol/src/rng_retrieve.rs; export from crates/kmipkit-protocol/src/lib.rs; pass the focused tests without imposing an unstated response-length rule.
- [x] T009 [US1] Commit the minimal passing protocol implementation in crates/kmipkit-protocol/src/rng_retrieve.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T010 [US1] Refactor RNG Retrieve conversion without changing behavior in crates/kmipkit-protocol/src/rng_retrieve.rs; rerun its tests and formatting.
- [x] T011 [US1] Commit protocol refactoring in crates/kmipkit-protocol/src/rng_retrieve.rs as the distinct Refactor evidence commit.

### Client Red/Green/Refactor

- [x] T012 [P] [US1] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs for operation code, exactly one exchange, result/Pending/delivery, output redaction, and no retry; capture expected Red failures.
- [x] T013 [US1] Commit only the failing client tests in crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs as the distinct Red evidence commit.
- [x] T014 [US1] Add RNG Retrieve request selection, response association, explicit client method, and redacted request handling in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs; pass the focused suite.
- [x] T015 [US1] Commit the minimal passing RNG Retrieve client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate the shared dispatcher.
- [x] T016 [US1] Refactor RNG Retrieve client handling without changing behavior in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs; rerun protocol/client tests.
- [x] T017 [US1] Commit client refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/rng_retrieve_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 4: User Story 2 — RNG Seed (Priority: P1)

**Goal**: Send caller seed bytes and accept the server-reported number of bytes used, including zero.

**Independent Test**: Fake transport observes one Table 319 request and returns Table 320 success with both positive and zero Data Length plus Table 321 failure.

### Protocol Red/Green/Refactor

- [x] T018 [P] [US2] Add failing Table 319/320 tests in crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs for required secret Data, required Integer Data Length, zero response length, unknown values, malformed types, and redaction; run cargo test -p kmipkit-protocol rng_seed_operation_tests and capture expected Red failure.
- [x] T019 [US2] Commit only the failing protocol tests in crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs as the distinct Red evidence commit.
- [x] T020 [US2] Implement RngSeedRequest/Response/Error and table conversion in crates/kmipkit-protocol/src/rng_seed.rs; export from crates/kmipkit-protocol/src/lib.rs; preserve zero Data Length as success per KMIPKIT-REQ-SPEC-6.1.50-001.
- [x] T021 [US2] Commit the minimal passing protocol implementation in crates/kmipkit-protocol/src/rng_seed.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T022 [US2] Refactor RNG Seed conversion without behavior changes in crates/kmipkit-protocol/src/rng_seed.rs; rerun focused tests and formatting.
- [x] T023 [US2] Commit protocol refactoring in crates/kmipkit-protocol/src/rng_seed.rs as the distinct Refactor evidence commit.

### Client Red/Green/Refactor

- [x] T024 [P] [US2] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs for operation code, one exchange, zero-length success, Failure/Pending/delivery, seed redaction, and no retry; retain expected Red failures.
- [x] T025 [US2] Commit only the failing client tests in crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs as the distinct Red evidence commit.
- [x] T026 [US2] Add RNG Seed request selection, response association, explicit client method, and redacted request handling in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs; pass the focused suite.
- [x] T027 [US2] Commit the minimal passing RNG Seed client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate this shared dispatcher edit.
- [x] T028 [US2] Refactor RNG Seed client handling without behavior changes in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs; rerun both RNG focused suites.
- [x] T029 [US2] Commit client refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/rng_seed_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 5: Polish and Cross-Cutting Concerns

**Purpose**: Close catalog traceability, security, parity, documentation, and quality gates.

- [x] T030 Update the implementation/verification references for both operations and KMIPKIT-REQ-SPEC-6.1.50-001 in specification/catalog/kmip-2.1.json; regenerate specification/catalog/coverage-report.md with python tools/normative_catalog/report.py --write and validate/check it.
- [ ] T031 After approved KMIPKIT-0043 mapping is integrated, add both operations to specification/api/public-api.json, implement mapped C/Java/Python adapters/parity tests, regenerate with python tools/api_manifest/generate.py --write, and verify --check; keep this task gated until 0043 is available.
- [x] T032 Update specs/027-rng-operations/traceability.md and specs/027-rng-operations/quickstart.md with actual test names/results, all normative links, and source-derived fixture status.
- [x] T033 Verify RNG Seed and RNG Retrieve byte sentinels never appear in Debug, Display, errors, or logs in crates/kmipkit-protocol/tests/unit/rng_seed_operation_tests.rs, crates/kmipkit-protocol/tests/unit/rng_retrieve_operation_tests.rs, and the client test suites.
- [ ] T034 Run required checks from specs/027-rng-operations/quickstart.md; record command/results and enforce the documented coverage and 100% traceability gates. Before 1.0, exercise RNG Retrieve and RNG Seed against at least two independent KMIP server implementations and record evidence in docs/development/kmip-interoperability.md; unsupported cases remain SKIP/N/A, not passing evidence.

## Dependencies and Execution Order

### Phase dependencies

- Setup and catalog checks precede implementation.
- Each story's protocol Red/Green/Refactor cycle precedes its client cycle.
- The RNG Retrieve and RNG Seed protocol modules are independent. Their additions to the shared execute.rs dispatcher must be centrally coordinated and sequenced.
- Polish follows both operation stories. Only language-binding tasks wait on 0043.

### Parallel opportunities

- T001/T002 and the separate protocol test/model files for US1 and US2 can proceed independently after baseline checks.
- Client tests are separate files; edits to crates/kmipkit-client/src/execute.rs are shared and must be coordinated.
- Neither operation blocks the other at the protocol-model layer.

### Implementation strategy

Deliver RNG Retrieve first as the MVP operation, validate it independently, then deliver RNG Seed and its zero-length success requirement. Finish shared catalog/binding/review gates last.

## Notes

- Each protocol and client layer has distinct DCO-signed Red, Green, and Refactor commits.
- Do not edit generated files or pinned OASIS copies manually.
- The library forwards RNG requests; it never generates randomness locally or rejects zero seed-use length.
