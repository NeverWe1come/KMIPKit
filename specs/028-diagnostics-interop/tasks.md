---
description: "Implementation task list for KMIP 2.1 interop and diagnostics"
---

# Tasks: KMIP 2.1 Interop and Diagnostics

**Input**: Design documents from specs/028-diagnostics-interop/

**Prerequisites**: Approved spec.md; plan.md, research.md, data-model.md, contracts/rust-client.md, traceability.md, quickstart.md

**Testing**: Required by the approved acceptance scenarios and repository TDD rules.

**Organization**: Separate user-story phases for Interop, Log, and Validate.

## Phase 1: Setup

**Purpose**: Confirm source, branch, and existing operation conventions.

- [x] T001 [P] Verify the assigned branch and pinned OASIS sections/tables for specs 028 in specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html; run python tools/normative_catalog/validate.py.
- [x] T002 [P] Review existing protocol/client typed-operation patterns and response results in crates/kmipkit-protocol/src/ping.rs, crates/kmipkit-protocol/src/query.rs, crates/kmipkit-client/src/execute.rs, and crates/kmipkit-client/tests/unit/ping_execution_tests.rs.

## Phase 2: Foundational

**Purpose**: Establish source decisions and traceability before implementation.

- [x] T003 Map all Interop, Log, and Validate request/response/error table fields and stable element IDs in specs/028-diagnostics-interop/traceability.md.
- [x] T004 Map all six KMIPKIT-REQ-SPEC-6.1.57-* IDs to implementation and executable tests in specs/028-diagnostics-interop/traceability.md.
- [x] T005 Record KMIPKIT-DISC-017's approved Log Table 252 interpretation and the server-only Interop production restriction in specs/028-diagnostics-interop/research.md and specs/028-diagnostics-interop/traceability.md.
- [x] T006 Verify existing operation, tag, and Validity Indicator catalog records in specification/catalog/kmip-2.1.json; add source-backed catalog inputs only if a required record is absent.

## Phase 3: User Story 1 — Interop (Priority: P1)

**Goal**: Send the required Interop Function and Identifier and inspect an empty success payload or preserved common result.

**Independent Test**: Fake transport observes one Table 241 request and returns empty Table 242 success or Table 243 failure; operation code, result, delivery, one exchange, and no retry are verified.

### Protocol Red/Green/Refactor

- [x] T007 [P] [US1] Add failing Table 241/242 tests in crates/kmipkit-protocol/tests/unit/interop_operation_tests.rs for required Enumeration/Text String, empty success payload, unknown enum preservation, wrong types, and malformed input; run cargo test -p kmipkit-protocol interop_operation_tests and capture expected Red failure.
- [x] T008 [US1] Commit only failing protocol tests in crates/kmipkit-protocol/tests/unit/interop_operation_tests.rs as the distinct Red evidence commit.
- [x] T009 [US1] Implement InteropRequest/Response/Error and table conversion in crates/kmipkit-protocol/src/interop.rs; export from crates/kmipkit-protocol/src/lib.rs; pass focused tests.
- [x] T010 [US1] Commit the minimal passing protocol implementation in crates/kmipkit-protocol/src/interop.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T011 [US1] Refactor Interop conversion without behavior changes in crates/kmipkit-protocol/src/interop.rs; rerun its tests and formatting.
- [x] T012 [US1] Commit protocol refactoring in crates/kmipkit-protocol/src/interop.rs as the distinct Refactor evidence commit.

### Client Red/Green/Refactor

- [x] T013 [P] [US1] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/interop_execution_tests.rs for correct operation code, empty response, one exchange, result/Pending/delivery preservation, and no retry; capture expected Red failures.
- [x] T014 [US1] Commit only failing client tests in crates/kmipkit-client/tests/unit/interop_execution_tests.rs as the distinct Red evidence commit.
- [x] T015 [US1] Add Interop request selection, response association, explicit client method, and sanitized errors in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs; pass the focused suite.
- [x] T016 [US1] Commit minimal passing Interop client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate the shared dispatcher.
- [x] T017 [US1] Refactor Interop dispatch without behavior changes in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/interop_execution_tests.rs; rerun focused suites.
- [x] T018 [US1] Commit client refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/interop_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 4: User Story 2 — Log (Priority: P1)

**Goal**: Send a required Log Message while preserving common results and keeping the message out of diagnostics.

**Independent Test**: Fake transport observes one Table 250 request and returns empty Table 251 success or Table 252 failure; sentinel message is absent from logs, Debug, Display, and errors.

### Protocol Red/Green/Refactor

- [x] T019 [P] [US2] Add failing Table 250/251 tests in crates/kmipkit-protocol/tests/unit/log_operation_tests.rs for required Text String, empty success, malformed input, common result behavior, and secret sentinel redaction; run cargo test -p kmipkit-protocol log_operation_tests and capture expected Red failure.
- [x] T020 [US2] Commit only failing protocol tests in crates/kmipkit-protocol/tests/unit/log_operation_tests.rs as the distinct Red evidence commit.
- [x] T021 [US2] Implement LogRequest/Response/Error and Table 250/251 conversion in crates/kmipkit-protocol/src/log.rs; export from crates/kmipkit-protocol/src/lib.rs; use Table 252 and approved KMIPKIT-DISC-017.
- [x] T022 [US2] Commit minimal passing Log protocol implementation in crates/kmipkit-protocol/src/log.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T023 [US2] Refactor Log conversion without behavior change in crates/kmipkit-protocol/src/log.rs; rerun protocol tests and formatting.
- [x] T024 [US2] Commit protocol refactoring in crates/kmipkit-protocol/src/log.rs as the distinct Refactor evidence commit.

### Client Red/Green/Refactor

- [x] T025 [P] [US2] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/log_execution_tests.rs for operation code, one exchange, empty success, Table 252 result preservation, no retry, and message redaction; capture expected Red failures.
- [x] T026 [US2] Commit only failing client tests in crates/kmipkit-client/tests/unit/log_execution_tests.rs as the distinct Red evidence commit.
- [x] T027 [US2] Add Log request selection, response association, explicit client method, and redacted formatting in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs; pass focused suite.
- [x] T028 [US2] Commit minimal passing Log client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate shared dispatcher edits.
- [x] T029 [US2] Refactor Log dispatch without behavior changes in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/log_execution_tests.rs; rerun focused protocol/client suites.
- [x] T030 [US2] Commit client refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/log_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 5: User Story 3 — Validate (Priority: P1)

**Goal**: Send one certificate chain represented by repeated Certificate and/or Unique Identifier fields, optionally specify Validity Date, and inspect the server's Validity Indicator.

**Independent Test**: Fake transport observes one Table 340 request and returns required Table 341 indicator or Table 342 failure; tests cover all six stable Validate requirement IDs and no local certificate validation.

### Protocol Red/Green/Refactor

- [x] T031 [P] [US3] Add failing Table 340/341 tests in crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs for one-chain model shape, Certificate-only/identifier-only/mixed and repeated inputs, optional date, omitted-date behavior without timestamp synthesis, required open Validity Indicator, wrong types, and unknown values; retain expected Red failure.
- [x] T032 [US3] Commit only failing protocol tests in crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs as the distinct Red evidence commit.
- [x] T033 [US3] Implement ValidateRequest/Response/Error in crates/kmipkit-protocol/src/validate.rs and export from crates/kmipkit-protocol/src/lib.rs; preserve repeated fields/order and avoid local chain/trust validation.
- [x] T034 [US3] Commit minimal passing Validate protocol implementation in crates/kmipkit-protocol/src/validate.rs and crates/kmipkit-protocol/src/lib.rs as the distinct Green evidence commit.
- [x] T035 [US3] Refactor Validate conversion without changing behavior in crates/kmipkit-protocol/src/validate.rs; rerun all six requirement-linked tests and formatting.
- [x] T036 [US3] Commit protocol refactoring in crates/kmipkit-protocol/src/validate.rs as the distinct Refactor evidence commit.

### Client Red/Green/Refactor

- [x] T037 [P] [US3] Add failing fake-transport tests in crates/kmipkit-client/tests/unit/validate_execution_tests.rs for correct operation code, request/response association, one exchange, indicator/result/Pending/delivery preservation, redaction, and no retry; capture expected Red failures.
- [x] T038 [US3] Commit only failing client tests in crates/kmipkit-client/tests/unit/validate_execution_tests.rs as the distinct Red evidence commit.
- [x] T039 [US3] Add Validate request selection, response association, explicit client method, and sanitized errors in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs; pass focused suite.
- [x] T040 [US3] Commit minimal passing Validate client integration in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/src/lib.rs as the distinct Green evidence commit; coordinate the shared dispatcher.
- [x] T041 [US3] Refactor Validate dispatch without behavior changes in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/validate_execution_tests.rs; rerun all three operation suites.
- [x] T042 [US3] Commit client refactoring in crates/kmipkit-client/src/execute.rs and crates/kmipkit-client/tests/unit/validate_execution_tests.rs as the distinct Refactor evidence commit.

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Close traceability, parity, security, documentation, and quality evidence.

- [x] T043 Update implementation and verification references for operation fields and six Validate requirements in specification/catalog/kmip-2.1.json; regenerate specification/catalog/coverage-report.md using python tools/normative_catalog/report.py --write and validate/check it.
- [ ] T044 After KMIPKIT-0043's approved operation mapping is integrated, update specification/api/public-api.json, mapped C/Java/Python adapters and parity tests, and regenerate only with python tools/api_manifest/generate.py --write; keep this task gated until 0043 is available.
- [x] T045 Update specs/028-diagnostics-interop/traceability.md and specs/028-diagnostics-interop/quickstart.md with actual named tests, results, stable requirement coverage, DISC-017 evidence, and unavailable fixture status.
- [x] T046 Verify Log Message, certificate material, and generic payload sentinel values do not appear in Debug, Display, errors, or logs in crates/kmipkit-protocol/tests/unit/log_operation_tests.rs, crates/kmipkit-protocol/tests/unit/validate_operation_tests.rs, and the client execution suites.
- [ ] T047 Run required checks from specs/028-diagnostics-interop/quickstart.md; record command/results and enforce the documented coverage and 100% traceability gates. Before 1.0, exercise Interop, Log, and Validate against at least two independent KMIP server implementations and record evidence in docs/development/kmip-interoperability.md; unsupported cases remain SKIP/N/A, not passing evidence.

## Dependencies and Execution Order

### Phase dependencies

- Complete source/traceability checks before implementation.
- Each story's protocol Red/Green/Refactor cycle precedes its client Red/Green/Refactor cycle.
- Interop, Log, and Validate protocol modules/tests are independent. Integrations into shared crates/kmipkit-client/src/execute.rs must be centrally coordinated and sequenced.
- Parity tasks wait only on KMIPKIT-0043. Other Rust work is not blocked.

### Parallel opportunities

- T001 and T002 are independent.
- Separate protocol test/model files for the three stories can proceed in parallel after Phase 2.
- Fake-transport tests are separate files. Shared execute.rs changes are a single coordinated integration stream.
- Do not claim an Interop server implementation, Log server logging, or local Validate certificate-chain implementation.

### Implementation strategy

Implement the Interop protocol/client vertical slice first, then Log with redaction, then Validate with six-requirement traceability. Run each story's focused tests before continuing; finish cross-cutting gates last.

## Notes

- Keep Red, Green, and Refactor evidence in distinct DCO-signed development commits.
- Never edit generated files or pinned OASIS upstream files by hand.
- Reuse shared structures/results; never retry, Poll automatically, or infer server policy.


## Implementation status

T001–T043, T045 and T046 are complete for the Rust scope, with distinct
Red/Green/Refactor commits for each operation and layer. T044 remains gated
until the approved KMIPKIT-0043 mapping is integrated into `release/1.0.0`.
T047 remains open: final local workspace/lint/dependency checks passed; the
coverage aggregate and independent two-server 1.0 release evidence remain
pending. A published support statement or fake
transport result is not a substitute for that evidence. See
`implementation-evidence.md` for actual commands, results and remaining gates.


## Supplemental normative-order correction

- [x] T048 Record distinct Red evidence for Table 340 Certificate/identifier grouping and Table 2 Certificate Type-before-Value, citing section 10.1.2's unnumbered Type table Structure row.
- [x] T049 Implement grouped request lists preserving each list's relative order and reject reversed Certificate fields structurally; record distinct Green evidence.
- [x] T050 Clarify ordering state and redacted list formatting, rerun focused protocol/client suites, and record distinct Refactor evidence. Update specification, plan, data model, contract, research and traceability to supersede mixed insertion/wire-order wording.
